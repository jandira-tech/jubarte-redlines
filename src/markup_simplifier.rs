// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `MarkupSimplifier.ts` (the comparison subset the comparer uses) — M2.
//!
//! Faithful transcription of `SingleCharacterRunTransform`, `RemoveRsidTransform`,
//! and the comparison-relevant element removal. These are tree-REBUILD transforms
//! (functional in the TS); in the arena we build fresh nodes and return them.

use crate::namespaces::W;
use crate::util::group_adjacent;
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

/// xml:space attribute name.
fn xml_space() -> XName {
    XNamespace::xml().name("space")
}

/// Build `<w:r>` with the given rPr element clones followed by `inner`.
fn build_run(dom: &mut Dom, rpr_elems: &[NodeId], inner: NodeId) -> NodeId {
    let run = dom.new_element(W::r());
    for &rpr in rpr_elems {
        dom.add(run, rpr); // already-parented → cloned by `add`
    }
    dom.add(run, inner);
    run
}

/// Copy `src`'s attributes onto `dst`.
fn copy_attrs(dom: &mut Dom, src: NodeId, dst: NodeId) {
    for (name, value) in dom.attributes(src) {
        dom.set_attribute_value(dst, &name, Some(&value));
    }
}

/// Port of `SingleCharacterRunTransform(node)` — returns the transformed node(s).
/// Non-elements pass through; a `w:r` explodes into one run per character (text)
/// or per child element (non-text); other elements rebuild recursively. A
/// one-character `w:t` that holds XML whitespace (space, tab, CR, LF) gets
/// `xml:space="preserve"`.
pub fn single_character_run_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![node];
    }
    let name = dom.name(node).unwrap();

    if name == W::r() {
        // rPr elements (cloned into each emitted run).
        let rpr_elems = dom.elements(node, Some(&W::r_pr()));
        // Child elements excluding rPr.
        let non_rpr: Vec<NodeId> = dom
            .elements(node, None)
            .into_iter()
            .filter(|&e| dom.name(e).unwrap() != W::r_pr())
            .collect();

        let mut out: Vec<NodeId> = Vec::new();
        let groups = group_adjacent(non_rpr, |&e| dom.name(e).unwrap() == W::t());
        for (is_t, group) in groups {
            if is_t {
                // Concatenate text of all <w:t> in the group, one run per char.
                let mut s = String::new();
                for &t in &group {
                    s.push_str(&dom.value(t));
                }
                for c in s.chars() {
                    let t = dom.new_element(W::t());
                    if matches!(c, ' ' | '\t' | '\r' | '\n') {
                        dom.set_attribute_value(t, &xml_space(), Some("preserve"));
                    }
                    dom.add_text(t, &c.to_string());
                    let run = build_run(dom, &rpr_elems, t);
                    out.push(run);
                }
            } else {
                // Non-text child: one run per element, element rebuilt recursively.
                for sr in group {
                    let sr_name = dom.name(sr).unwrap();
                    let ne = dom.new_element(sr_name);
                    copy_attrs(dom, sr, ne);
                    for child in dom.nodes(sr) {
                        let transformed = single_character_run_transform(dom, child);
                        for tn in transformed {
                            dom.add(ne, tn);
                        }
                    }
                    let run = build_run(dom, &rpr_elems, ne);
                    out.push(run);
                }
            }
        }
        return out;
    }

    // Other element: rebuild with transformed children.
    let ne = dom.new_element(name);
    copy_attrs(dom, node, ne);
    for child in dom.nodes(node) {
        let transformed = single_character_run_transform(dom, child);
        for tn in transformed {
            dom.add(ne, tn);
        }
    }
    vec![ne]
}

/// `TransformElementToSingleCharacterRuns(element)` — returns the single rebuilt
/// root of a non-`w:r` container such as the body or a paragraph.
///
/// # Behaviour
///
/// A `w:r` explodes into zero, one or many runs, so it has no single rebuilt
/// root. A `w:r` `element` is outside this function's contract: it is returned
/// unchanged and the DOM is left as it was (use [`single_character_run_transform`]
/// to explode a run into its runs). Every other element, and a non-element
/// node, yields exactly one node.
pub fn transform_element_to_single_character_runs(dom: &mut Dom, element: NodeId) -> NodeId {
    if dom.is_element(element) && dom.name(element) == Some(W::r()) {
        return element;
    }
    let v = single_character_run_transform(dom, element);
    debug_assert_eq!(v.len(), 1, "root transform must yield exactly one node");
    v[0]
}

/// True if `name` is a volatile `w:rsid*` attribute (the set stripped by
/// `RemoveRsidTransform`).
fn is_rsid_attr(name: &XName) -> bool {
    if name.namespace_name() != W::URI {
        return false;
    }
    matches!(
        name.local_name(),
        "rsid" | "rsidDel" | "rsidP" | "rsidR" | "rsidRDefault" | "rsidRPr" | "rsidSect" | "rsidTr"
    )
}

/// Port of `RemoveRsidTransform(node)` — drops `<w:rsid>` elements and all
/// `w:rsid*` attributes. Returns `None` if the node itself is dropped
/// (a `<w:rsid>`).
///
/// RSID-INPLACE-01: mutates the existing subtree in place (same root
/// `NodeId`) instead of rebuilding every node. Observable XML after the
/// transform is unchanged; only intermediate allocation/drop work is removed.
pub fn remove_rsid_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::name("rsid") {
        return None;
    }
    // Strip rsid* attributes without rebuilding the element.
    let rsid_attrs: Vec<_> = dom
        .attributes(node)
        .into_iter()
        .filter(|(an, _)| is_rsid_attr(an))
        .map(|(an, _)| an)
        .collect();
    for an in &rsid_attrs {
        dom.set_attribute_value(node, an, None);
    }
    // Snapshot children — recursion may remove `w:rsid` siblings.
    let kids: Vec<NodeId> = (0..dom.child_count(node))
        .map(|i| dom.child_at(node, i))
        .collect();
    for k in kids {
        if remove_rsid_transform(dom, k).is_none() {
            dom.remove(k);
        }
    }
    Some(node)
}
