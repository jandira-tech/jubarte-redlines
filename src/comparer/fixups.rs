// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drawing / shape id repair.
//!
//! The produced document splices content from two source packages. DrawingML
//! ids are renumbered; VML ids are preserved unless they collide, sharing one
//! space across shapes, shapetypes, groups and other VML elements (#1800).
//! References follow the renamed definition. The compare pipeline calls
//! [`fix_up_drawing_ids`] mid-stream and after the Word-mode merge passes;
//! [`fix_up_drawing_ids_in_package`] covers the identical-input shortcut.
//!
//! [`fix_up_group_ids`] remains the standalone upstream consolidate helper;
//! compare uses the shared VML repair instead.

use std::collections::{HashMap, HashSet};

use crate::namespaces::{O, VML};
use crate::xmllinq::{Dom, NodeId, XName};

fn id_attr() -> XName {
    XName::get("id", "")
}

/// Repair drawing ids after splicing sources. An edit's redline
/// (`preserve_untouched`) also keeps unique DrawingML ids (#1872).
/// VML always keeps unique source ids, regardless of compare mode (#1800).
pub fn fix_up_drawing_ids(dom: &mut Dom, root: NodeId, preserve_untouched: bool) {
    if preserve_untouched {
        rename_repeated_doc_pr_ids(dom, root);
    } else {
        fix_up_doc_pr_ids(dom, root);
    }
    fix_up_shape_ids(dom, root);
}

/// Every drawing's `wp:docPr`, in document order: any element named
/// `docPr`, whatever its namespace (Strict and remapped drawings too).
fn doc_pr_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.descendants(root, None)
        .into_iter()
        .filter(|&e| dom.name(e).is_some_and(|n| n.local_name() == "docPr"))
        .collect()
}

/// `old` with the first `_N` suffix no id in `taken` has; the result is
/// taken from then on.
fn unused_id(old: &str, taken: &mut HashSet<String>) -> String {
    let fresh = (1u32..)
        .map(|k| format!("{old}_{k}"))
        .find(|candidate| !taken.contains(candidate))
        .unwrap_or_else(|| old.to_string());
    taken.insert(fresh.clone());
    fresh
}

/// Give a `wp:docPr` whose id an earlier one already holds (or that holds
/// none a number can read) the next id above every other.
fn rename_repeated_doc_pr_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let nodes = doc_pr_nodes(dom, root);
    // Every id the drawings hold: a new one may be none of them, nor one
    // given before. Past `u32::MAX` the search wraps (never to 0) instead
    // of handing `u32::MAX` out again.
    let mut taken: HashSet<u32> = nodes
        .iter()
        .filter_map(|&n| dom.attribute(n, &id)?.parse::<u32>().ok())
        .collect();
    let mut next = taken.iter().copied().max().unwrap_or(0);
    let mut seen = HashSet::new();
    for node in nodes {
        if let Some(own) = dom.attribute(node, &id).and_then(|v| v.parse::<u32>().ok())
            && seen.insert(own)
        {
            continue;
        }
        next = next.wrapping_add(1);
        while next == 0 || taken.contains(&next) {
            next = next.wrapping_add(1);
        }
        taken.insert(next);
        dom.set_attribute_value(node, &id, Some(&next.to_string()));
    }
}

/// Renumber every drawing's `wp:docPr/@id` starting from 1, in document
/// order. Matches elements whose local name is `docPr` (and not only the
/// transitional `WP` URI) so Strict and remapped drawings are renumbered
/// too. Calling order must be `1..n` (no gaps) so Word doesn't prompt for
/// a repair dialog on open.
pub fn fix_up_doc_pr_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    for (next, e) in (1u32..).zip(doc_pr_nodes(dom, root)) {
        dom.set_attribute_value(e, &id, Some(&next.to_string()));
    }
}

/// Keep source VML ids unless they collide within this part. All VML
/// elements share the same id space, including `shape`, `shapetype`, `rect`
/// and `group`. Rewrite shapetype references and sibling OLE ShapeIDs when
/// their targets are renamed.
pub fn fix_up_shape_ids(dom: &mut Dom, root: NodeId) {
    repair_vml_ids(dom, root);
}

/// Repair VML ids and their type references in one pass, just as
/// [`fix_up_shape_ids`] does. Kept as an entry point for callers that used
/// the former separate shapetype pass; running both is idempotent.
pub fn fix_up_shape_type_ids(dom: &mut Dom, root: NodeId) {
    repair_vml_ids(dom, root);
}

/// Bind each type reference to the nearest preceding definition with that
/// original id, else the first following one (#1872). Snapshot each
/// definition's old/new ids before relinking so collisions across spliced
/// sources cannot redirect every reference to the last definition.
fn repair_vml_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let type_attr = XName::get("type", "");
    let shape_id = XName::get("ShapeID", "");
    let shapetype = VML::shapetype();
    let nodes: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| {
            dom.name(n)
                .is_some_and(|name| name.namespace_name() == VML::URI)
        })
        .collect();
    // Reserve even ids that occur later: a generated suffix must never
    // force an otherwise unique source id to change.
    let mut taken: HashSet<String> = nodes
        .iter()
        .filter_map(|&n| dom.attribute(n, &id).map(str::to_string))
        .collect();
    let mut seen = HashSet::new();
    let mut renamed: HashMap<NodeId, (String, String)> = HashMap::new();
    let mut first: HashMap<String, String> = HashMap::new();
    for &node in &nodes {
        let Some(old) = dom.attribute(node, &id).map(str::to_string) else {
            continue;
        };
        let new = if seen.insert(old.clone()) {
            old.clone()
        } else {
            unused_id(&old, &mut taken)
        };
        if new != old {
            dom.set_attribute_value(node, &id, Some(&new));
            if dom.name_is(node, &VML::shape())
                && let Some(parent) = dom.parent(node)
            {
                for ole in dom.elements(parent, Some(&O::ole_object())) {
                    if dom.attribute(ole, &shape_id) == Some(old.as_str()) {
                        dom.set_attribute_value(ole, &shape_id, Some(&new));
                    }
                }
            }
        }
        if dom.name_is(node, &shapetype) {
            first.entry(old.clone()).or_insert_with(|| new.clone());
            renamed.insert(node, (old, new));
        }
    }
    let mut latest: HashMap<String, String> = HashMap::new();
    for node in nodes {
        if let Some((old, new)) = renamed.get(&node) {
            latest.insert(old.clone(), new.clone());
            continue;
        }
        let Some(named) = dom.attribute(node, &type_attr).map(str::to_string) else {
            continue;
        };
        let (hash, key) = match named.strip_prefix('#') {
            Some(key) => ("#", key),
            None => ("", named.as_str()),
        };
        if let Some(new) = latest.get(key).or_else(|| first.get(key)) {
            let value = format!("{hash}{new}");
            if value != named {
                dom.set_attribute_value(node, &type_attr, Some(&value));
            }
        }
    }
}

/// Renumber every `v:group/@id` from 1, in document order.
///
/// Not called from the compare/produce path (the upstream oracle only
/// calls it in the consolidate path); exposed for hosts that replicate the
/// consolidate flow.
pub fn fix_up_group_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    for (next, g) in (1u32..).zip(dom.descendants(root, Some(&VML::group()))) {
        dom.set_attribute_value(g, &id, Some(&next.to_string()));
    }
}

/// Apply the DrawingML and shared VML id fixups to a
/// package's main document and write the redlined package back to bytes.
///
/// Used by the IDENTICAL-INPUT short-circuit so an empty redline still
/// renumbers colliding source `wp:docPr/@id` values the full pipeline
/// would have fixed.
//
///
/// # Errors
///
/// [`crate::opc::OpcError`] when the package cannot be opened, the main
/// document part is absent, or the part has no root element after parsing.
pub fn fix_up_drawing_ids_in_package(docx: &[u8]) -> Result<Vec<u8>, crate::opc::OpcError> {
    fix_up_drawing_ids_in_package_within(docx, crate::admission::InputLimits::compare(), false)
}

/// Keep the identical-input comparer's already-admitted budget through the
/// fixup; `preserve_untouched` as in [`fix_up_drawing_ids`].
pub(crate) fn fix_up_drawing_ids_in_package_within(
    docx: &[u8],
    limits: crate::admission::InputLimits,
    preserve_untouched: bool,
) -> Result<Vec<u8>, crate::opc::OpcError> {
    use crate::opc::PartFs;

    let mut pkg = PartFs::open_within(docx, limits)?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return Err(crate::opc::OpcError::PartNotFound(format!(
            "main document missing: {main}"
        )));
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return Err(crate::opc::OpcError::PartNotFound(format!(
            "main document has no root element: {main}"
        )));
    };
    fix_up_drawing_ids(&mut dom, root, preserve_untouched);
    pkg.set_part(&main, dom.serialize_element(root).into_bytes());
    pkg.to_zip()
}

#[cfg(test)]
mod repeated_doc_pr_id_tests {
    use super::*;
    use crate::namespaces::WP;

    fn drawings(ids: &[&str]) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let body: String = ids
            .iter()
            .map(|id| format!(r#"<wp:docPr id="{id}" name="Picture"/>"#))
            .collect();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:body xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wp="{}">{body}</w:body>"#,
            WP::URI
        ));
        let root = dom.root(doc).expect("a root");
        (dom, root)
    }

    fn ids(dom: &Dom, root: NodeId) -> Vec<String> {
        doc_pr_nodes(dom, root)
            .into_iter()
            .map(|n| dom.attribute(n, &id_attr()).unwrap_or_default().to_string())
            .collect()
    }

    /// Repeats of the largest id each get their own, wrapping past
    /// `u32::MAX` instead of saturating there (Jules #4395711592).
    #[test]
    fn repeats_of_the_largest_id_get_distinct_ids() {
        let max = u32::MAX.to_string();
        let (mut dom, root) = drawings(&[&max, &max, &max, "1"]);
        rename_repeated_doc_pr_ids(&mut dom, root);
        assert_eq!(ids(&dom, root), [max.as_str(), "2", "3", "1"]);
    }

    /// A new id is none a later drawing already holds.
    #[test]
    fn a_new_id_skips_the_ids_later_drawings_hold() {
        let (mut dom, root) = drawings(&["7", "7", "8", "x"]);
        rename_repeated_doc_pr_ids(&mut dom, root);
        assert_eq!(ids(&dom, root), ["7", "9", "8", "10"]);
    }
}
