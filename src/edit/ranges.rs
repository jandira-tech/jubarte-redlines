// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Edit-only restoration after comparison. Compare's range policy is unchanged.
use std::collections::HashMap;

use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::inspect::Opened;
use crate::namespaces::{PT, W};
use crate::xmllinq::{Dom, NodeId};

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
/// both the edited copy and the comparison. Only endpoints directly in those
/// paragraphs are moved; all other endpoints keep bookmark offset semantics.
fn restore_empty(dom: &mut Dom, original: NodeId, revised: NodeId, output: NodeId) {
    let source = dom.descendants(original, Some(&W::p()));
    let edited = dom.descendants(revised, Some(&W::p()));
    let target = dom.descendants(output, Some(&W::p()));
    // Keep marker identity on this leg: adjacent empty paragraphs can be
    // identical apart from their ranges, and one may have been removed.
    let first = unchanged(dom, &source, &edited, true);
    let second = unchanged(dom, &edited, &target, false);
    let mut ids = HashMap::new();
    let mut next_id = dom
        .descendants(output, None)
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id())?.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    for start in dom.descendants(original, Some(&W::bookmark_start())) {
        let Some((id, name)) = dom
            .attribute(start, &W::id())
            .zip(dom.attribute(start, &W::name("name")))
        else {
            continue;
        };
        if name == "_GoBack" {
            continue;
        }
        let matching = dom
            .descendants(output, Some(&W::bookmark_start()))
            .into_iter()
            .find(|&n| dom.attribute(n, &W::name("name")) == Some(name));
        let new = matching
            .and_then(|n| dom.attribute(n, &W::id()))
            .map(str::to_string)
            .unwrap_or_else(|| {
                let id = next_id.to_string();
                next_id += 1;
                id
            });
        ids.insert(id.to_string(), new);
    }
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
        let children = dom.elements(p, None);
        let anchors: Vec<_> = dom
            .elements(destination, None)
            .into_iter()
            .filter(|&n| !is_marker(dom, n))
            .collect();
        let mut gap = 0;
        for node in children {
            if !is_marker(dom, node) {
                gap += 1;
                continue;
            }
            let Some(name) = dom.name(node) else { continue };
            let Some(id) = dom.attribute(node, &W::id()) else {
                continue;
            };
            if name == W::bookmark_start()
                && dom.attribute(node, &W::name("name")) == Some("_GoBack")
            {
                continue;
            }
            let bookmark = name == W::bookmark_start() || name == W::bookmark_end();
            let id = if bookmark {
                let Some(id) = ids.get(id) else { continue };
                id.clone()
            } else {
                id.to_string()
            };
            // Remove the endpoint already placed by text. Its paired endpoint
            // may live in a text paragraph and is deliberately left there.
            for old in dom.descendants(output, Some(&name)) {
                if dom.attribute(old, &W::id()) == Some(id.as_str()) {
                    dom.remove(old);
                }
            }
            let copy = dom.clone_subtree(node);
            if bookmark {
                dom.set_attribute_value(copy, &W::id(), Some(&id));
            }
            if let Some(&anchor) = anchors.get(gap) {
                dom.add_before_self(anchor, copy);
            } else {
                dom.add(destination, copy);
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
    let open = |bytes| Opened::open(bytes).map_err(|e| err("COMPARE_FAILED", None, e.to_string()));
    let before = open(original)?;
    let after = open(revised)?;
    let mut out = open(redline)?;
    let mut parts = vec![before.main.clone()];
    parts.extend(before.story_parts().into_iter().map(|(_, _, part)| part));
    for part in parts {
        let Some(xml) = out.pkg.part_string(&part) else {
            continue;
        };
        let Some(a) = before.pkg.part_string(&part) else {
            continue;
        };
        let Some(b) = after.pkg.part_string(&part) else {
            continue;
        };
        if ![&a, &b]
            .iter()
            .any(|s| s.contains("permStart") || s.contains("bookmarkStart"))
        {
            continue;
        }
        let d = out.dom.parse_xdocument(&xml);
        let Some(root) = out.dom.root(d) else {
            continue;
        };
        crate::comparer::bookmarks::carry_permissions(
            &mut out.dom,
            root,
            (&before.pkg, &part),
            (&after.pkg, &part),
            author,
        );
        let ad = out.dom.parse_xdocument(&a);
        let bd = out.dom.parse_xdocument(&b);
        if let Some((a, b)) = out.dom.root(ad).zip(out.dom.root(bd)) {
            restore_empty(&mut out.dom, a, b, root);
        }
        let xml = out.dom.serialize_document(d);
        out.pkg.set_part(&part, xml.into_bytes());
    }
    out.pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}
