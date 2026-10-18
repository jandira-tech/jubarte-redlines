// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Edit-only restoration after comparison. Compare's range policy is unchanged.
use std::collections::{HashMap, HashSet, VecDeque};

use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::namespaces::{PT, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

use super::{EditError, err};

fn is_marker(dom: &Dom, node: NodeId) -> bool {
    ["bookmarkStart", "bookmarkEnd", "permStart", "permEnd"]
        .iter()
        .any(|name| dom.name_is(node, &W::name(name)))
}

/// Namespace/prefix-independent structural key, excluding carried markers and
/// volatile producer ids. Revisions remain in the key: a changed paragraph is
/// not an unchanged anchor. Empty property containers are immaterial.
fn key(dom: &Dom, node: NodeId, keep_markers: bool) -> String {
    if !keep_markers && is_marker(dom, node) {
        return String::new();
    }
    let Some(name) = dom.name(node) else {
        return dom.text_value(node).unwrap_or("").to_string();
    };
    let children: String = dom
        .nodes(node)
        .into_iter()
        .map(|n| key(dom, n, keep_markers))
        .collect();
    let mut attrs: Vec<_> = dom
        .attributes(node)
        .into_iter()
        .filter(|(name, _)| {
            name.namespace_name() != PT::URI
                && name.namespace_name() != "http://www.w3.org/2000/xmlns/"
                && !name.local_name().starts_with("rsid")
                && !matches!(name.local_name(), "paraId" | "textId")
        })
        .collect();
    attrs.sort_by(|a, b| {
        (a.0.namespace_name(), a.0.local_name()).cmp(&(b.0.namespace_name(), b.0.local_name()))
    });
    if children.is_empty() && attrs.is_empty() && [W::p_pr(), W::r_pr()].contains(&name) {
        return String::new();
    }
    format!("<{name:?}{attrs:?}>{children}</{name:?}>")
}

fn unchanged(
    dom: &Dom,
    from: &[NodeId],
    to: &[NodeId],
    keep_markers: bool,
) -> HashMap<NodeId, NodeId> {
    let a: Vec<_> = from.iter().map(|&p| key(dom, p, keep_markers)).collect();
    let b: Vec<_> = to.iter().map(|&p| key(dom, p, keep_markers)).collect();
    capture_diff_slices(Algorithm::Myers, &a, &b)
        .into_iter()
        .flat_map(|op| {
            let (tag, a, b) = op.as_tag_tuple();
            if tag == DiffTag::Equal {
                from[a]
                    .iter()
                    .copied()
                    .zip(to[b].iter().copied())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// Find original empty paragraphs among the paragraphs left unchanged in
/// both the edited copy and the comparison. Index endpoints once per part;
/// each source occurrence moves at most one corresponding output endpoint.
fn restore_empty(dom: &mut Dom, original: NodeId, revised: NodeId, output: NodeId) {
    let source = dom.descendants(original, Some(&W::p()));
    let edited = dom.descendants(revised, Some(&W::p()));
    let target = dom.descendants(output, Some(&W::p()));
    let first = unchanged(dom, &source, &edited, true);
    let second = unchanged(dom, &edited, &target, false);
    let mut candidates = Vec::new();
    for p in source {
        if dom
            .descendants(p, Some(&W::t()))
            .into_iter()
            .any(|n| !dom.value(n).is_empty())
        {
            continue;
        }
        let Some(&destination) = first.get(&p).and_then(|p| second.get(p)) else {
            continue;
        };
        let mut gap = 0;
        for node in dom.elements(p, None) {
            if is_marker(dom, node) {
                candidates.push((node, destination, gap));
            } else {
                gap += 1;
            }
        }
    }
    if candidates.is_empty() {
        return;
    }
    let restoring: HashSet<_> = candidates.iter().map(|c| c.0).collect();
    let mut endpoints: HashMap<(XName, String), VecDeque<NodeId>> = HashMap::new();
    let mut names: HashMap<String, VecDeque<String>> = HashMap::new();
    let mut next_id = 0;
    for node in dom.descendants(output, None) {
        let Some(id) = dom.attribute(node, &W::id()) else {
            continue;
        };
        next_id = next_id.max(id.parse::<u64>().unwrap_or(0));
        if is_marker(dom, node) {
            endpoints
                .entry((dom.name(node).unwrap(), id.to_string()))
                .or_default()
                .push_back(node);
        }
        if dom.name_is(node, &W::bookmark_start())
            && let Some(name) = dom.attribute(node, &W::name("name"))
        {
            names
                .entry(name.to_string())
                .or_default()
                .push_back(id.to_string());
        }
    }
    // Pair by source occurrence, not by name (which malformed-but-readable
    // inputs can repeat). Keep the comparer id of each existing occurrence.
    let available: HashSet<_> = endpoints.keys().cloned().collect();
    let mut open: HashMap<String, VecDeque<NodeId>> = HashMap::new();
    let mut partners = HashMap::new();
    let mut ids = HashMap::new();
    for node in dom.descendants(original, None) {
        let Some(id) = dom.attribute(node, &W::id()) else {
            continue;
        };
        if dom.name_is(node, &W::bookmark_start()) {
            let Some(name) = dom.attribute(node, &W::name("name")) else {
                continue;
            };
            if name == "_GoBack" {
                continue;
            }
            let new = names
                .get_mut(name)
                .and_then(VecDeque::pop_front)
                .unwrap_or_else(|| {
                    next_id += 1;
                    next_id.to_string()
                });
            ids.insert(node, new);
            open.entry(id.to_string()).or_default().push_back(node);
        } else if dom.name_is(node, &W::bookmark_end())
            && let Some(start) = open.get_mut(id).and_then(VecDeque::pop_front)
        {
            ids.insert(node, ids[&start].clone());
            partners.insert(node, start);
            partners.insert(start, node);
        }
    }
    let anchors: HashMap<_, Vec<_>> = candidates
        .iter()
        .map(|&(_, p, _)| p)
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|p| {
            (
                p,
                dom.elements(p, None)
                    .into_iter()
                    .filter(|&n| !is_marker(dom, n))
                    .collect(),
            )
        })
        .collect();
    for (node, destination, gap) in candidates {
        let Some(name) = dom.name(node) else { continue };
        let Some(source_id) = dom.attribute(node, &W::id()) else {
            continue;
        };
        let bookmark = name == W::bookmark_start() || name == W::bookmark_end();
        let id = if bookmark {
            let Some(id) = ids.get(&node) else { continue };
            let Some(&partner) = partners.get(&node) else {
                continue;
            };
            if !restoring.contains(&partner)
                && !available.contains(&(dom.name(partner).unwrap(), id.clone()))
            {
                continue; // Restoring half a bookmark must never create an orphan.
            }
            id.clone()
        } else {
            source_id.to_string()
        };
        if let Some(old) = endpoints
            .get_mut(&(name, id.clone()))
            .and_then(VecDeque::pop_front)
        {
            dom.remove(old);
        }
        let copy = dom.clone_subtree(node);
        if bookmark {
            dom.set_attribute_value(copy, &W::id(), Some(&id));
        }
        if let Some(&anchor) = anchors[&destination].get(gap) {
            dom.add_before_self(anchor, copy);
        } else {
            dom.add(destination, copy);
        }
    }
}

/// A streaming eligibility scan keeps ordinary bookmarked text paragraphs off
/// the restoration path. Prefixes do not matter; unknown XML stays conservative.
fn needs_restore(xml: &str) -> bool {
    if xml.contains("permStart") || xml.contains("permEnd") {
        return true;
    }
    if !xml.contains("bookmarkStart") && !xml.contains("bookmarkEnd") {
        return false;
    }
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.local_name().as_ref() {
                "p" => paragraphs.push((false, false)),
                "t" => in_text = true,
                "bookmarkStart" | "bookmarkEnd" => {
                    if let Some(p) = paragraphs.last_mut() {
                        p.0 = true;
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e))
                if matches!(e.local_name().as_ref(), "bookmarkStart" | "bookmarkEnd") =>
            {
                if let Some(p) = paragraphs.last_mut() {
                    p.0 = true;
                }
            }
            Ok(Event::End(e)) => match e.local_name().as_ref() {
                "p" => {
                    if paragraphs
                        .pop()
                        .is_some_and(|(marker, text)| marker && !text)
                    {
                        return true;
                    }
                }
                "t" => in_text = false,
                _ => {}
            },
            Ok(Event::Text(t)) if in_text && !t.is_empty() => {
                for p in &mut paragraphs {
                    p.1 = true;
                }
            }
            Ok(Event::CData(t)) if in_text && !t.is_empty() => {
                for p in &mut paragraphs {
                    p.1 = true;
                }
            }
            Ok(Event::Eof) => return false,
            Err(_) => return true,
            _ => {}
        }
    }
}

/// Text has no offset projection in an empty story. Index structural parent
/// paths and gaps instead; the revised story supplies the current properties.
fn restore_textless_permissions(dom: &mut Dom, revised: NodeId, output: NodeId) {
    fn index(dom: &Dom, node: NodeId, path: &[usize], out: &mut HashMap<Vec<usize>, Vec<NodeId>>) {
        let children: Vec<_> = dom
            .elements(node, None)
            .into_iter()
            .filter(|&n| !is_marker(dom, n))
            .collect();
        out.insert(path.to_vec(), children.clone());
        for (i, child) in children.into_iter().enumerate() {
            let mut path = path.to_vec();
            path.push(i);
            index(dom, child, &path, out);
        }
    }
    let mut target = HashMap::new();
    index(dom, output, &[], &mut target);
    let mut parents = HashMap::from([(Vec::new(), output)]);
    for (path, children) in &target {
        for (i, &child) in children.iter().enumerate() {
            let mut path = path.clone();
            path.push(i);
            parents.insert(path, child);
        }
    }
    for name in [W::name("permStart"), W::name("permEnd")] {
        for node in dom.descendants(output, Some(&name)) {
            dom.remove(node);
        }
    }
    let mut stack = vec![(revised, Vec::new())];
    while let Some((node, path)) = stack.pop() {
        let mut gap = 0;
        for child in dom.elements(node, None) {
            if is_marker(dom, child) {
                if dom
                    .name(child)
                    .is_some_and(|n| [W::name("permStart"), W::name("permEnd")].contains(&n))
                    && let Some(&parent) = parents.get(&path)
                {
                    let copy = dom.clone_subtree(child);
                    if let Some(&anchor) = target.get(&path).and_then(|c| c.get(gap)) {
                        dom.add_before_self(anchor, copy);
                    } else {
                        dom.add(parent, copy);
                    }
                }
            } else {
                let mut child_path = path.clone();
                child_path.push(gap);
                stack.push((child, child_path));
                gap += 1;
            }
        }
    }
}

/// Apply to the body and all related header/footer/footnote/endnote parts.
pub(super) fn restore(
    redline: &[u8],
    original: &[u8],
    revised: &[u8],
    author: &str,
) -> Result<Vec<u8>, EditError> {
    let open = |bytes| PartFs::open(bytes).map_err(|e| err("COMPARE_FAILED", None, e.to_string()));
    let before = open(original)?;
    let after = open(revised)?;
    let mut out = open(redline)?;
    let main = before
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let mut parts = vec![main.clone()];
    parts.extend(
        before
            .read_rels_for(&main)
            .into_iter()
            .flat_map(|rels| &rels.items)
            .filter(|rel| {
                rel.target_mode.as_deref() != Some("External")
                    && matches!(
                        rel.rel_type.rsplit('/').next(),
                        Some("header" | "footer" | "footnotes" | "endnotes")
                    )
            })
            .map(|rel| before.resolve_rel_target(&main, &rel.target)),
    );
    parts.sort();
    parts.dedup();
    let mut dom = Dom::new();
    let mut changed = false;
    for part in parts {
        let Some(a) = before.part_string(&part) else {
            continue;
        };
        let Some(b) = after.part_string(&part) else {
            continue;
        };
        if !needs_restore(&a) && !needs_restore(&b) {
            continue;
        }
        let Some(xml) = out.part_string(&part) else {
            continue;
        };
        let d = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(d) else { continue };
        if a.contains("permStart")
            || a.contains("permEnd")
            || b.contains("permStart")
            || b.contains("permEnd")
        {
            crate::comparer::bookmarks::carry_permissions(
                &mut dom,
                root,
                (&before, &part),
                (&after, &part),
                author,
            );
        }
        let ad = dom.parse_xdocument(&a);
        let bd = dom.parse_xdocument(&b);
        if let Some((a, b)) = dom.root(ad).zip(dom.root(bd)) {
            if dom
                .descendants(b, Some(&W::t()))
                .into_iter()
                .all(|n| dom.value(n).is_empty())
            {
                restore_textless_permissions(&mut dom, b, root);
            }
            restore_empty(&mut dom, a, b, root);
        }
        out.set_part(&part, dom.serialize_document(d).into_bytes());
        changed = true;
    }
    if !changed {
        return Ok(redline.to_vec());
    }
    out.to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}
