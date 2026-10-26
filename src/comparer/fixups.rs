// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drawing / shape id renumbering.
//!
//! The produced document splices content from two source packages, so the
//! original ids collide and Word offers to repair it. Each fixup renumbers a
//! target element's `id` attribute (no namespace) sequentially and rewrites
//! the linked attribute where one exists. The compare pipeline calls
//! [`fix_up_doc_pr_ids`], [`fix_up_shape_ids`], and [`fix_up_shape_type_ids`]
//! once mid-stream and again after the Word-mode merge passes; the package
//! facade [`fix_up_drawing_ids_in_package`] covers the IDENTICAL-INPUT
//! short-circuit (empty redline still renumbers to avoid the collision).
//!
//! `FixUpGroupIds` is intentionally not wired into the produce path; the
//! C# oracle only calls it from the consolidate path (:1524), and wiring
//! it in here would diverge from upstream.

use std::collections::{HashMap, HashSet};

use crate::namespaces::{O, VML, WP};
use crate::xmllinq::{Dom, NodeId, XName};

fn id_attr() -> XName {
    XName::get("id", "")
}

/// The three renumbers the compare pipeline runs: [`fix_up_doc_pr_ids`],
/// [`fix_up_shape_ids`] and [`fix_up_shape_type_ids`]. An edit's redline
/// (`preserve_untouched`) renames only an id that repeats: an id the
/// document holds once stays as written, so Accept All gives back the
/// clean copy's drawings (#1872).
pub fn fix_up_drawing_ids(dom: &mut Dom, root: NodeId, preserve_untouched: bool) {
    if preserve_untouched {
        rename_repeated_doc_pr_ids(dom, root);
        rename_repeated_shape_ids(dom, root);
        relink_shape_types(dom, root, true);
    } else {
        fix_up_doc_pr_ids(dom, root);
        fix_up_shape_ids(dom, root);
        fix_up_shape_type_ids(dom, root);
    }
}

/// Every drawing's `wp:docPr`, in document order.
fn doc_pr_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    let wp_doc_pr = WP::name("docPr");
    let nodes: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&e| {
            dom.name(e).is_some_and(|n| n.local_name() == "docPr")
                || dom.name_is(e, &wp_doc_pr)
        })
        .collect();
    // Dedup while preserving document order (filter may match twice if WP==local).
    let mut seen = HashSet::new();
    nodes.into_iter().filter(|n| seen.insert(*n)).collect()
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
    let mut next = nodes
        .iter()
        .filter_map(|&n| dom.attribute(n, &id)?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    let mut seen = HashSet::new();
    for node in nodes {
        if let Some(own) = dom.attribute(node, &id).and_then(|v| v.parse::<u32>().ok())
            && seen.insert(own)
        {
            continue;
        }
        next = next.saturating_add(1);
        seen.insert(next);
        dom.set_attribute_value(node, &id, Some(&next.to_string()));
    }
}

/// Rename a `v:shape` whose id an earlier shape already holds, and the
/// sibling `o:OLEObject/@ShapeID` that named it.
fn rename_repeated_shape_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let shape_id = XName::get("ShapeID", "");
    let vml_shape = VML::name("shape");
    let o_ole_object = O::name("OLEObject");
    let shapes = dom.descendants(root, Some(&vml_shape));
    let mut taken: HashSet<String> = shapes
        .iter()
        .filter_map(|&s| dom.attribute(s, &id).map(str::to_string))
        .collect();
    let mut seen = HashSet::new();
    for shape in shapes {
        let Some(old) = dom.attribute(shape, &id).map(str::to_string) else {
            continue;
        };
        if seen.insert(old.clone()) {
            continue;
        }
        let new = unused_id(&old, &mut taken);
        dom.set_attribute_value(shape, &id, Some(&new));
        if let Some(parent) = dom.parent(shape) {
            for ole in dom.elements(parent, Some(&o_ole_object)) {
                if dom.attribute(ole, &shape_id) == Some(old.as_str()) {
                    dom.set_attribute_value(ole, &shape_id, Some(&new));
                }
            }
        }
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

/// Renumber every `v:shape/@id` from 1, in document order, and rewrite
/// any sibling `o:OLEObject/@ShapeID` that referenced the old id so the
/// OLE ↔ shape linkage survives the renumber. An unlinked OLE object is
/// left with its old `ShapeID`.
pub fn fix_up_shape_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let shape_id = XName::get("ShapeID", "");
    for (next, shape) in (1u32..).zip(dom.descendants(root, Some(&VML::name("shape")))) {
        let old = dom.attribute(shape, &id).map(|s| s.to_string());
        let new = next.to_string();
        dom.set_attribute_value(shape, &id, Some(&new));
        // rewrite any sibling o:OLEObject whose ShapeID referenced the old id
        if let (Some(old), Some(parent)) = (old, dom.parent(shape)) {
            for ole in dom.elements(parent, Some(&O::name("OLEObject"))) {
                if dom.attribute(ole, &shape_id).map(|s| s.to_string()) == Some(old.clone()) {
                    dom.set_attribute_value(ole, &shape_id, Some(&new));
                }
            }
        }
    }
}

/// Renumber every `v:shapetype/@id` from 1, in document order, and rewrite
/// every `v:shape/@type` that named one (both the bare value and the `#…`
/// prefixed form). Word writes a shapetype once, beside the first shape
/// that uses it, and later shapes in other `w:pict`s name it by id: C#
/// rewrote only the sibling's `type`, so a later shape named an id that no
/// longer existed and Word drew it without the shapetype's path (#1872).
pub fn fix_up_shape_type_ids(dom: &mut Dom, root: NodeId) {
    relink_shape_types(dom, root, false);
}

/// Give each `v:shapetype` its new id (the next number, or under
/// `keep_unique` its own unless an earlier shapetype holds it), then point
/// each `v:shape/@type` at the nearest shapetype before it that had the id
/// it names, else the first one after it.
fn relink_shape_types(dom: &mut Dom, root: NodeId, keep_unique: bool) {
    let id = id_attr();
    let type_attr = XName::get("type", "");
    let shapetype = VML::name("shapetype");
    let shape = VML::name("shape");
    let nodes: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &shapetype) || dom.name_is(n, &shape))
        .collect();
    let shapetypes: Vec<NodeId> = nodes
        .iter()
        .copied()
        .filter(|&n| dom.name_is(n, &shapetype))
        .collect();
    let mut taken: HashSet<String> = shapetypes
        .iter()
        .filter_map(|&n| dom.attribute(n, &id).map(str::to_string))
        .collect();
    let mut seen = HashSet::new();
    // Each shapetype's (old id, new id), and the new id of the first
    // shapetype that had each old one.
    let mut renamed: HashMap<NodeId, (String, String)> = HashMap::new();
    let mut first: HashMap<String, String> = HashMap::new();
    let mut next = 0u32;
    for node in shapetypes {
        let old = dom.attribute(node, &id).map(str::to_string);
        let new = if keep_unique {
            let Some(old) = old.clone() else {
                continue;
            };
            if seen.insert(old.clone()) {
                old
            } else {
                unused_id(&old, &mut taken)
            }
        } else {
            next += 1;
            next.to_string()
        };
        dom.set_attribute_value(node, &id, Some(&new));
        if let Some(old) = old {
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
        if !dom.name_is(node, &shape) {
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
    for (next, g) in (1u32..).zip(dom.descendants(root, Some(&VML::name("group")))) {
        dom.set_attribute_value(g, &id, Some(&next.to_string()));
    }
}

/// Apply the three drawing-id fixups (doc / shape / shape-type) to a
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
