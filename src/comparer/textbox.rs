// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word marks a change to a text box's text inside the box.
//!
//! The diff sees a text box as one opaque run child (`mc:AlternateContent`
//! holding the DrawingML shape and its VML fallback, or a bare `w:drawing` or
//! `w:pict`), so any edit to its text comes out as the old box deleted beside
//! the new box inserted. Word's Compare keeps the one box and marks the words
//! that changed inside it, in the shape and in the fallback alike
//! (fixtures_500 00b81efae883: "Overall purpose of the post" → "Overall aim of
//! the post" is one box with "aim" inserted and "purpose" deleted).

use super::WmlComparerSettings;
use crate::namespaces::{MC, W};
use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
use crate::xmllinq::{Dom, NodeId};

/// Replace each deleted shape that sits among insertions beside an inserted
/// shape differing from it only in text-box text with the inserted shape,
/// each of its text boxes holding a compare of the old story against the new
/// one. A shape with no text box pairs only with an identical copy, which the
/// diff deleted and inserted again beside a changed box; Word leaves it
/// alone.
pub fn diff_inside_replaced_text_boxes(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
) {
    let txbx = W::name("txbxContent");
    let paragraphs: Vec<NodeId> = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .filter(|&p| {
            !dom.ancestors(p, None)
                .into_iter()
                .any(|a| dom.name_is(a, &txbx))
        })
        .collect();
    for p in paragraphs {
        for cluster in revision_clusters(dom, p) {
            let boxes = |dom: &Dom, kind: &crate::xmllinq::XName| -> Vec<(NodeId, NodeId)> {
                cluster
                    .iter()
                    .filter(|&&w| dom.name_is(w, kind))
                    .flat_map(|&w| dom.elements(w, None))
                    .filter_map(|r| shape_run(dom, r))
                    .collect()
            };
            let deleted = boxes(dom, &W::del());
            let inserted = boxes(dom, &W::ins());
            let mut next = 0;
            for &(old_run, old) in &deleted {
                let old_shell = shell(dom, old);
                let found = (next..inserted.len()).find(|&j| {
                    let new = inserted[j].1;
                    dom.name(old) == dom.name(new) && shell(dom, new) == old_shell
                });
                let Some(j) = found else {
                    continue;
                };
                next = j + 1;
                let (new_run, new) = inserted[j];
                if redline_boxes(dom, old, new, settings) {
                    detach_run(dom, old_run);
                    lift_run(dom, new_run);
                }
            }
        }
    }
}

/// Each maximal run of adjacent `w:ins`/`w:del` children of a paragraph that
/// holds both kinds.
fn revision_clusters(dom: &Dom, p: NodeId) -> Vec<Vec<NodeId>> {
    let mut clusters = Vec::new();
    let mut current: Vec<NodeId> = Vec::new();
    let mut flush = |current: &mut Vec<NodeId>| {
        let (ins, del) = (W::ins(), W::del());
        let has_ins = current.iter().any(|&w| dom.name_is(w, &ins));
        let has_del = current.iter().any(|&w| dom.name_is(w, &del));
        if has_ins && has_del {
            clusters.push(std::mem::take(current));
        }
        current.clear();
    };
    for k in dom.elements(p, None) {
        if dom.name_is(k, &W::ins()) || dom.name_is(k, &W::del()) {
            current.push(k);
        } else {
            flush(&mut current);
        }
    }
    flush(&mut current);
    clusters
}

/// A run whose only content is one shape (`mc:AlternateContent`, `w:drawing`
/// or `w:pict`): `(run, shape)`.
fn shape_run(dom: &Dom, run: NodeId) -> Option<(NodeId, NodeId)> {
    if !dom.name_is(run, &W::r()) {
        return None;
    }
    let content: Vec<NodeId> = dom
        .elements(run, None)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::r_pr()))
        .collect();
    let [shape] = content[..] else {
        return None;
    };
    let is_shape = dom.name_is(shape, &MC::name("AlternateContent"))
        || dom.name_is(shape, &W::drawing())
        || dom.name_is(shape, &W::pict());
    is_shape.then_some((run, shape))
}

/// Remove a run from its revision wrapper, and the wrapper once empty.
fn detach_run(dom: &mut Dom, run: NodeId) {
    let wrapper = dom.parent(run);
    dom.remove(run);
    if let Some(w) = wrapper
        && dom.elements(w, None).is_empty()
    {
        dom.remove(w);
    }
}

/// Move a run out of its revision wrapper to the same place, splitting the
/// wrapper around it.
fn lift_run(dom: &mut Dom, run: NodeId) {
    let Some(wrapper) = dom.parent(run) else {
        return;
    };
    let siblings = dom.nodes(wrapper);
    let at = siblings.iter().position(|&n| n == run).unwrap_or(0);
    let after = &siblings[at + 1..];
    if !after.is_empty() {
        let tail = dom.new_element(dom.name(wrapper).unwrap_or_else(W::ins));
        for (name, value) in dom.attributes(wrapper) {
            dom.set_attribute_value(tail, &name, Some(&value));
        }
        for &n in after {
            dom.remove(n);
            dom.add(tail, n);
        }
        dom.add_after_self(wrapper, tail);
    }
    dom.remove(run);
    dom.add_after_self(wrapper, run);
    if dom.elements(wrapper, None).is_empty() {
        dom.remove(wrapper);
    }
}

/// A shape with its text boxes emptied and the ids the id fixups renumber
/// per copy left out (shape and drawing ids, and a VML shape's `#id`
/// reference to its shapetype), so two boxes that differ only in text
/// compare equal.
fn shell(dom: &mut Dom, shape: NodeId) -> String {
    let copy = dom.clone_subtree(shape);
    for b in dom.descendants(copy, Some(&W::name("txbxContent"))) {
        for c in dom.nodes(b) {
            dom.remove(c);
        }
    }
    for e in dom.descendants_and_self(copy, None) {
        for (name, value) in dom.attributes(e) {
            let renumbered = matches!(name.local_name(), "id" | "spid")
                || (name.local_name() == "type" && value.starts_with('#'));
            if renumbered {
                dom.set_attribute_value(e, &name, None);
            }
        }
    }
    dom.serialize_element(copy)
}

/// One text box's paragraphs as a story of their own, with its revisions
/// accepted (the new box) or rejected (the old box).
fn story(dom: &mut Dom, text_box: NodeId, accept: bool) -> NodeId {
    let story = dom.new_element(W::name("hdr"));
    for c in dom.elements(text_box, None) {
        let copy = dom.clone_subtree(c);
        dom.add(story, copy);
    }
    if accept {
        accept_revisions_document(dom, story)
    } else {
        reject_revisions_document(dom, story)
    }
}

/// Fill each text box of `new` with a compare of `old`'s story against it.
/// Leaves both untouched and returns false when the compare cannot be used.
fn redline_boxes(dom: &mut Dom, old: NodeId, new: NodeId, settings: &WmlComparerSettings) -> bool {
    let txbx = W::name("txbxContent");
    let old_boxes = dom.descendants(old, Some(&txbx));
    let new_boxes = dom.descendants(new, Some(&txbx));
    if old_boxes.len() != new_boxes.len() {
        return false;
    }
    let mut redlined = Vec::with_capacity(new_boxes.len());
    for (&before, &after) in old_boxes.iter().zip(&new_boxes) {
        // A linked box's VML fallback holds an empty story; two empty
        // stories need no compare.
        if dom.nodes(before).is_empty() && dom.nodes(after).is_empty() {
            redlined.push(None);
            continue;
        }
        let before = story(dom, before, false);
        let after = story(dom, after, true);
        let result = super::compare_bodies_faithful(dom, before, after, before, after, settings);
        let Some(body) = dom.element(result, &W::body()) else {
            return false;
        };
        let paragraphs: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::sect_pr()))
            .collect();
        if paragraphs.is_empty() {
            return false;
        }
        redlined.push(Some(paragraphs));
    }
    for (&text_box, paragraphs) in new_boxes.iter().zip(redlined) {
        let Some(paragraphs) = paragraphs else {
            continue;
        };
        for c in dom.nodes(text_box) {
            dom.remove(c);
        }
        for p in paragraphs {
            dom.remove(p);
            dom.add(text_box, p);
        }
    }
    true
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod complete_shape_revision_contract_tests {
    use super::*;
    fn document(source: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc=dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:pPr><w:spacing w:after="160"/></w:pPr>{source}</w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#,W::URI));
        (dom, doc)
    }
    #[test]
    fn matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split() {
        let shape = r#"<w:r><w:rPr><w:color w:val="0000FF"/></w:rPr><w:pict><v:rect id="owned" style="width:20pt;height:10pt"/></w:pict></w:r>"#;
        let old_shape = shape.replace("owned", "prior");
        let prefix = r#"<w:r><w:rPr><w:b/></w:rPr><w:t>owned before</w:t></w:r>"#;
        let tail = r#"<w:r><w:rPr><w:i/></w:rPr><w:t>owned after</w:t></w:r>"#;
        for (old_neighbor, new_before, new_after) in [
            ("", "", ""),
            (tail, prefix, tail),
            (tail, "", tail),
            (tail, prefix, ""),
        ] {
            let del_open = r#"<w:del w:id="4" w:author="Old" w:date="2026-10-09T00:00:00Z">"#;
            let ins_open = r#"<w:ins w:id="5" w:author="New" w:date="2026-10-09T00:00:00Z">"#;
            let input = format!(
                "{del_open}{old_shape}{old_neighbor}</w:del>{ins_open}{new_before}{shape}{new_after}</w:ins>"
            );
            let expected = format!(
                "{}{}{shape}{}",
                if old_neighbor.is_empty() {
                    String::new()
                } else {
                    format!("{del_open}{old_neighbor}</w:del>")
                },
                if new_before.is_empty() {
                    String::new()
                } else {
                    format!("{ins_open}{new_before}</w:ins>")
                },
                if new_after.is_empty() {
                    String::new()
                } else {
                    format!("{ins_open}{new_after}</w:ins>")
                }
            );
            let (mut dom, doc) = document(&input);
            let root = dom.root(doc).unwrap();
            let (expected_dom, expected_doc) = document(&expected);
            diff_inside_replaced_text_boxes(&mut dom, root, &WmlComparerSettings::default());
            assert_eq!(
                dom.serialize_document(doc),
                expected_dom.serialize_document(expected_doc)
            );
            diff_inside_replaced_text_boxes(&mut dom, root, &WmlComparerSettings::default());
            assert_eq!(
                dom.serialize_document(doc),
                expected_dom.serialize_document(expected_doc)
            );
        }
    }
    #[test]
    fn unrelated_or_mixed_shape_carriers_retain_complete_source_revisions() {
        let shape = r#"<w:r><w:pict><v:rect style="width:20pt;height:10pt"/></w:pict></w:r>"#;
        for (old, new) in [
            (shape, shape.replace("20pt", "21pt")),
            (shape, shape.replace("w:pict", "w:drawing")),
            (
                shape,
                shape.replace("</w:r>", "<w:t>mixed authored text</w:t></w:r>"),
            ),
            (
                shape,
                format!("<w:hyperlink w:anchor=\"owned\">{shape}</w:hyperlink>"),
            ),
        ] {
            let input = format!(
                "<w:del w:id=\"1\" w:author=\"A\">{old}</w:del><w:ins w:id=\"2\" w:author=\"B\">{new}</w:ins>"
            );
            let (mut dom, doc) = document(&input);
            let root = dom.root(doc).unwrap();
            let before = dom.serialize_document(doc);
            diff_inside_replaced_text_boxes(&mut dom, root, &WmlComparerSettings::default());
            assert_eq!(dom.serialize_document(doc), before);
        }
    }
}
