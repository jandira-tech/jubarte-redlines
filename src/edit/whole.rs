// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `replace` with `whole: true`.
//!
//! The comparer shows a replacement as Word Compare does, word by word,
//! keeping shared words as unchanged text. Typing over a selection with
//! Track Changes on shows the whole selection deleted and the new text
//! inserted after it, which is what an agent replacing a clause usually
//! wants. The edited copy carries a helper bookmark around the new text, the
//! comparer carries it into the redline, and [`rewrite`] turns the
//! bookmark's span, grown over adjacent deletions until its deleted side
//! reads `find`, into one deletion followed by one insertion.

use std::collections::HashSet;

use crate::inspect::Opened;
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

use super::{EditError, err, wrap_range};

const PREFIX: &str = "_jubarte_whole_";

/// Range markup that may sit inside the rewritten span.
const MARKERS: &[&str] = &[
    "bookmarkStart",
    "bookmarkEnd",
    "commentRangeStart",
    "commentRangeEnd",
    "proofErr",
    "permStart",
    "permEnd",
];

/// Reference runs that stay once, on the inserted side.
const REFERENCES: &[&str] = &["commentReference", "footnoteReference", "endnoteReference"];

/// One helper bookmark and the change it marks.
#[derive(Clone, Debug)]
pub(super) struct Mark {
    op: usize,
    /// The header, footer or notes part holding the text; `None` for the body.
    part: Option<String>,
    name: String,
    find: String,
    replacement: String,
}

/// Wrap the new text at projection offsets `start..end` of `paragraph` in a
/// helper bookmark. `None` when the range holds no run.
pub(super) fn mark(
    dom: &mut Dom,
    paragraph: NodeId,
    (start, end): (usize, usize),
    (op, part): (usize, Option<String>),
    (find, replacement): (&str, &str),
) -> Option<Mark> {
    let root = dom
        .ancestors(paragraph, None)
        .last()
        .copied()
        .unwrap_or(paragraph);
    let id = dom
        .descendants(root, Some(&W::bookmark_start()))
        .into_iter()
        .filter_map(|b| dom.attribute(b, &W::id())?.parse::<u64>().ok())
        .max()
        .map_or(0, |max| max + 1)
        .to_string();
    let name = format!("{PREFIX}{op}");
    let open = dom.new_element(W::bookmark_start());
    dom.set_attribute_value(open, &W::id(), Some(&id));
    dom.set_attribute_value(open, &W::name("name"), Some(&name));
    let close = dom.new_element(W::bookmark_end());
    dom.set_attribute_value(close, &W::id(), Some(&id));
    wrap_range(dom, paragraph, start, end, open, close).then(|| Mark {
        op,
        part,
        name,
        find: find.to_string(),
        replacement: replacement.to_string(),
    })
}

/// Remove every helper bookmark under `root`, and any revision container
/// left empty by that.
pub(super) fn strip(dom: &mut Dom, root: NodeId) {
    let starts: Vec<NodeId> = dom
        .descendants(root, Some(&W::bookmark_start()))
        .into_iter()
        .filter(|&b| is_helper(dom, b))
        .collect();
    let ids: HashSet<String> = starts
        .iter()
        .filter_map(|&b| dom.attribute(b, &W::id()).map(str::to_string))
        .collect();
    let ends: Vec<NodeId> = dom
        .descendants(root, Some(&W::bookmark_end()))
        .into_iter()
        .filter(|&b| {
            dom.attribute(b, &W::id())
                .is_some_and(|id| ids.contains(id))
        })
        .collect();
    for node in starts.into_iter().chain(ends) {
        let parent = dom.parent(node);
        dom.remove(node);
        if let Some(parent) = parent
            && (dom.name_is(parent, &W::ins()) || dom.name_is(parent, &W::del()))
            && dom.nodes(parent).is_empty()
        {
            dom.remove(parent);
        }
    }
}

fn is_helper(dom: &Dom, bookmark: NodeId) -> bool {
    dom.attribute(bookmark, &W::name("name"))
        .is_some_and(|name| name.starts_with(PREFIX))
}

/// An operation index and why its change stayed word-level.
pub(super) type Fallback = (usize, String);

/// Rewrite each mark's span in `redline` as one deletion then one insertion
/// and drop every helper bookmark. A mark that cannot be rewritten keeps the
/// comparer's word-level diff and comes back with the reason.
pub(super) fn rewrite(
    redline: &[u8],
    (base, marked): (&[u8], &[u8]),
    marks: &[Mark],
    author: &str,
    date: &str,
) -> Result<(Vec<u8>, Vec<Fallback>), EditError> {
    let mut opened =
        Opened::open(redline).map_err(|e| err("COMPARE_FAILED", None, e.to_string()))?;
    // The body, then every story part a mark lives in.
    let mut parts = vec![(opened.main.clone(), opened.document, opened.body)];
    let stories: std::collections::BTreeSet<&String> =
        marks.iter().filter_map(|m| m.part.as_ref()).collect();
    if !stories.is_empty() {
        let open = |bytes| {
            crate::opc::PartFs::open(bytes).map_err(|e| err("COMPARE_FAILED", None, e.to_string()))
        };
        let (base, marked) = (open(base)?, open(marked)?);
        for part in stories {
            let missing = || err("COMPARE_FAILED", None, format!("redline lost {part}"));
            let xml = opened.pkg.part_string(part).ok_or_else(missing)?;
            let document = opened.dom.parse_xdocument(&xml);
            let root = opened.dom.root(document).ok_or_else(missing)?;
            // The comparer carries bookmarks in the body only; place the
            // helper bookmarks of this part by their text, as it does there.
            crate::comparer::bookmarks::carry_matching_bookmarks(
                &mut opened.dom,
                root,
                (&base, part),
                (&marked, part),
                author,
                |name| name.starts_with(PREFIX),
            );
            parts.push((part.clone(), document, root));
        }
    }
    let mut next_id = parts
        .iter()
        .flat_map(|&(_, _, root)| opened.dom.descendants(root, None))
        .filter_map(|n| opened.dom.attribute(n, &W::id())?.parse::<u64>().ok())
        .max()
        .map_or(1, |max| max + 1);
    let mut fallbacks = Vec::new();
    for mark in marks {
        let stamp = Stamp {
            author,
            date,
            next_id: &mut next_id,
        };
        let root = parts
            .iter()
            .find(|(part, ..)| mark.part.as_ref() == Some(part))
            .map_or(opened.body, |&(_, _, root)| root);
        if let Err(reason) = rewrite_one(&mut opened.dom, root, mark, stamp) {
            fallbacks.push((mark.op, reason));
        }
    }
    for (part, document, root) in parts {
        strip(&mut opened.dom, root);
        let xml = opened.dom.serialize_document(document);
        opened.pkg.set_part(&part, xml.into_bytes());
    }
    let bytes = opened
        .pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
    Ok((bytes, fallbacks))
}

/// Revision attribution for the containers a rewrite creates.
struct Stamp<'a> {
    author: &'a str,
    date: &'a str,
    next_id: &'a mut u64,
}

impl Stamp<'_> {
    fn container(&mut self, dom: &mut Dom, name: crate::xmllinq::XName) -> NodeId {
        let node = dom.new_element(name);
        dom.set_attribute_value(node, &W::id(), Some(&self.next_id.to_string()));
        *self.next_id += 1;
        dom.set_attribute_value(node, &W::author(), Some(self.author));
        dom.set_attribute_value(node, &W::date(), Some(self.date));
        node
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Kept,
    Inserted,
    Deleted,
    /// Range markup or a reference run: kept once, on the inserted side.
    Marker,
    /// Anything the rewrite does not handle (fields, hyperlinks, moves...).
    Other,
}

/// A paragraph child, or a child of a paragraph-level `w:ins`/`w:del`.
struct Item {
    node: NodeId,
    kind: Kind,
    container: Option<NodeId>,
}

fn rewrite_one(
    dom: &mut Dom,
    body: NodeId,
    mark: &Mark,
    mut stamp: Stamp<'_>,
) -> Result<(), String> {
    let start = dom
        .descendants(body, Some(&W::bookmark_start()))
        .into_iter()
        .find(|&b| dom.attribute(b, &W::name("name")) == Some(mark.name.as_str()))
        .ok_or("the comparer dropped the change's bookmark")?;
    let id = dom
        .attribute(start, &W::id())
        .unwrap_or_default()
        .to_string();
    let p = dom
        .ancestors(start, Some(&W::p()))
        .first()
        .copied()
        .ok_or("the change is outside a paragraph")?;
    let items = flatten(dom, p);
    let s = items
        .iter()
        .position(|i| i.node == start)
        .ok_or("the change sits inside a field, hyperlink or content control")?;
    let e = items
        .iter()
        .position(|i| {
            dom.name_is(i.node, &W::bookmark_end()) && dom.attribute(i.node, &W::id()) == Some(&id)
        })
        .filter(|&e| e > s)
        .ok_or("the change spans paragraphs")?;
    // Deletions the comparer placed just outside the bookmark belong to the
    // change when they complete `find`.
    let deleted_at = |i: &usize| items[*i].kind == Kind::Deleted;
    let left = (0..s).rev().take_while(deleted_at).count();
    let right = (e + 1..items.len()).take_while(deleted_at).count();
    let (from, to) = (0..=left)
        .flat_map(|l| (0..=right).map(move |r| (s - l, e + r)))
        .find(|&(from, to)| side_text(dom, &items[from..=to], Kind::Deleted) == mark.find)
        .ok_or_else(|| format!("the comparer's diff does not line up with {:?}", mark.find))?;
    let span = &items[from..=to];
    if span.iter().any(|i| i.kind == Kind::Other) {
        return Err("the change crosses a field, hyperlink or other structure".to_string());
    }
    if side_text(dom, span, Kind::Inserted) != mark.replacement {
        return Err(format!(
            "the comparer's diff does not line up with {:?}",
            mark.replacement
        ));
    }

    let mut deleted = Vec::new();
    let mut inserted = Vec::new();
    for item in span {
        if item.node == start || item.node == items[e].node {
            continue;
        }
        match item.kind {
            Kind::Kept => {
                let copy = dom.clone_subtree(item.node);
                for t in dom.descendants(copy, Some(&W::t())) {
                    dom.set_name(t, W::del_text());
                }
                deleted.push(copy);
                inserted.push(item.node);
            }
            Kind::Deleted => deleted.push(item.node),
            Kind::Inserted | Kind::Marker => inserted.push(item.node),
            Kind::Other => unreachable!("checked above"),
        }
    }
    let del = stamp.container(dom, W::del());
    let ins = stamp.container(dom, W::ins());

    // Take the paragraph apart, then lay it out again: items before the span
    // in their own containers, the deletion, the insertion, the rest.
    let containers: Vec<NodeId> = items.iter().filter_map(|i| i.container).collect();
    for item in &items {
        dom.remove(item.node);
    }
    for container in containers {
        dom.remove(container);
    }
    let mut current: Option<(NodeId, NodeId)> = None;
    let mut placed: HashSet<NodeId> = HashSet::new();
    for (i, item) in items.iter().enumerate() {
        if i == from {
            current = None;
            for node in deleted.drain(..) {
                dom.add(del, node);
            }
            for node in inserted.drain(..) {
                dom.add(ins, node);
            }
            dom.add(p, del);
            dom.add(p, ins);
        }
        if (from..=to).contains(&i) {
            continue;
        }
        let Some(original) = item.container else {
            current = None;
            dom.add(p, item.node);
            continue;
        };
        let target = match current {
            Some((open, target)) if open == original => target,
            _ => {
                let target = if placed.insert(original) {
                    original
                } else {
                    // The span split this container; the second half needs its
                    // own revision id.
                    let copy = dom.new_element(dom.name(original).expect("element").clone());
                    for (name, value) in dom.attributes(original) {
                        dom.set_attribute_value(copy, &name, Some(&value));
                    }
                    dom.set_attribute_value(copy, &W::id(), Some(&stamp.next_id.to_string()));
                    *stamp.next_id += 1;
                    copy
                };
                dom.add(p, target);
                current = Some((original, target));
                target
            }
        };
        dom.add(target, item.node);
    }
    Ok(())
}

fn flatten(dom: &Dom, p: NodeId) -> Vec<Item> {
    let mut items = Vec::new();
    for child in dom.elements(p, None) {
        if dom.name_is(child, &W::p_pr()) {
            continue;
        }
        let side = if dom.name_is(child, &W::ins()) {
            Kind::Inserted
        } else if dom.name_is(child, &W::del()) {
            Kind::Deleted
        } else {
            items.push(Item {
                node: child,
                kind: classify(dom, child, Kind::Kept),
                container: None,
            });
            continue;
        };
        for grandchild in dom.elements(child, None) {
            items.push(Item {
                node: grandchild,
                kind: classify(dom, grandchild, side),
                container: Some(child),
            });
        }
    }
    items
}

fn classify(dom: &Dom, node: NodeId, side: Kind) -> Kind {
    let named = |names: &[&str], n: NodeId| names.iter().any(|m| dom.name_is(n, &W::name(m)));
    if dom.name_is(node, &W::r()) {
        if run_text(dom, node).is_some() {
            side
        } else if dom
            .elements(node, None)
            .into_iter()
            .all(|c| dom.name_is(c, &W::r_pr()) || named(REFERENCES, c))
        {
            Kind::Marker
        } else {
            Kind::Other
        }
    } else if named(MARKERS, node) {
        Kind::Marker
    } else {
        Kind::Other
    }
}

/// A run's text when it holds nothing but text.
fn run_text(dom: &Dom, run: NodeId) -> Option<String> {
    let mut text = String::new();
    for child in dom.elements(run, None) {
        if dom.name_is(child, &W::t()) || dom.name_is(child, &W::del_text()) {
            text.push_str(&dom.value(child));
        } else if !dom.name_is(child, &W::r_pr())
            && !dom.name_is(child, &W::name("lastRenderedPageBreak"))
        {
            return None;
        }
    }
    Some(text)
}

/// Text of the kept runs plus the runs of one side.
fn side_text(dom: &Dom, items: &[Item], side: Kind) -> String {
    items
        .iter()
        .filter(|i| i.kind == Kind::Kept || i.kind == side)
        .filter_map(|i| run_text(dom, i.node))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

    fn body(xml: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{NS}"><w:body>{xml}</w:body></w:document>"#
        ));
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (dom, body)
    }

    fn mark(find: &str, replacement: &str) -> Mark {
        Mark {
            op: 0,
            part: None,
            name: format!("{PREFIX}0"),
            find: find.to_string(),
            replacement: replacement.to_string(),
        }
    }

    fn run(dom: &mut Dom, body: NodeId, m: &Mark) -> Result<(), String> {
        let mut next_id = 100;
        let stamp = Stamp {
            author: "A",
            date: "2026-09-28T00:00:00Z",
            next_id: &mut next_id,
        };
        rewrite_one(dom, body, m, stamp)
    }

    const BM: &str = r#"<w:bookmarkStart w:id="7" w:name="_jubarte_whole_0"/>"#;
    const BE: &str = r#"<w:bookmarkEnd w:id="7"/>"#;

    #[test]
    fn a_dropped_bookmark_keeps_the_word_level_diff() {
        let (mut dom, b) = body("<w:p><w:r><w:t>x</w:t></w:r></w:p>");
        let before = dom.serialize_element(b);
        assert_eq!(
            run(&mut dom, b, &mark("x", "y")).unwrap_err(),
            "the comparer dropped the change's bookmark"
        );
        assert_eq!(dom.serialize_element(b), before);
    }

    #[test]
    fn a_span_crossing_a_hyperlink_is_left_alone() {
        let xml = format!(
            r#"<w:p>{BM}<w:r><w:t>a </w:t></w:r><w:hyperlink><w:r><w:t>b</w:t></w:r></w:hyperlink><w:ins w:id="1"><w:r><w:t>c</w:t></w:r></w:ins>{BE}</w:p>"#
        );
        let (mut dom, b) = body(&xml);
        let before = dom.serialize_element(b);
        let reason = run(&mut dom, b, &mark("a ", "a c")).unwrap_err();
        assert!(reason.contains("crosses"), "{reason}");
        assert_eq!(dom.serialize_element(b), before);
    }

    /// The span starts inside a comparer `w:ins` that also holds text before
    /// it; the part after the span keeps its container under a new id.
    #[test]
    fn a_container_split_by_the_span_gets_a_second_id() {
        let xml = format!(
            r#"<w:p><w:ins w:id="1" w:author="A" w:date="d"><w:r><w:t>new </w:t></w:r>{BM}<w:r><w:t>ten</w:t></w:r></w:ins><w:del w:id="2" w:author="A" w:date="d"><w:r><w:delText>five</w:delText></w:r></w:del><w:r><w:t> days</w:t></w:r>{BE}<w:ins w:id="3" w:author="A" w:date="d"><w:r><w:t>!</w:t></w:r></w:ins></w:p>"#
        );
        let (mut dom, b) = body(&xml);
        run(&mut dom, b, &mark("five days", "ten days")).unwrap();
        let out = dom.serialize_element(b);
        let text: Vec<(String, String)> = dom
            .descendants(b, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del()))
            .map(|n| {
                let words: String = dom
                    .descendants(n, None)
                    .into_iter()
                    .filter(|&t| dom.name_is(t, &W::t()) || dom.name_is(t, &W::del_text()))
                    .map(|t| dom.value(t))
                    .collect();
                (dom.attribute(n, &W::id()).unwrap().to_string(), words)
            })
            .collect();
        assert_eq!(
            text,
            [
                ("1".to_string(), "new ".to_string()),
                ("100".to_string(), "five days".to_string()),
                ("101".to_string(), "ten days".to_string()),
                ("3".to_string(), "!".to_string()),
            ],
            "{out}"
        );
        assert!(!out.contains("_jubarte_whole"), "{out}");
    }

    #[test]
    fn strip_removes_helper_bookmarks_and_the_containers_they_emptied() {
        let xml = format!(
            r#"<w:p><w:ins w:id="1">{BM}</w:ins><w:r><w:t>a</w:t></w:r>{BE}<w:bookmarkStart w:id="8" w:name="keep"/><w:bookmarkEnd w:id="8"/></w:p>"#
        );
        let (mut dom, b) = body(&xml);
        strip(&mut dom, b);
        let out = dom.serialize_element(b);
        assert!(
            !out.contains("_jubarte_whole") && !out.contains("w:ins"),
            "{out}"
        );
        assert!(out.contains(r#"w:name="keep""#), "{out}");
        assert_eq!(out.matches("bookmarkEnd").count(), 1, "{out}");
    }
}
