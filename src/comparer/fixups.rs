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

use crate::namespaces::{O, VML, WP};
use crate::xmllinq::{Dom, NodeId, XName};

fn id_attr() -> XName {
    XName::get("id", "")
}

/// Renumber every drawing's `wp:docPr/@id` starting from 1, in document
/// order. Matches elements whose local name is `docPr` (and not only the
/// transitional `WP` URI) so Strict and remapped drawings are renumbered
/// too. Calling order must be `1..n` (no gaps) so Word doesn't prompt for
/// a repair dialog on open.
pub fn fix_up_doc_pr_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let nodes: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&e| {
            dom.name(e).is_some_and(|n| n.local_name() == "docPr")
                || dom.name(e).as_ref() == Some(&WP::doc_pr())
        })
        .collect();
    // Dedup while preserving document order (filter may match twice if WP==local).
    let mut seen = std::collections::HashSet::new();
    let nodes: Vec<NodeId> = nodes.into_iter().filter(|n| seen.insert(*n)).collect();
    for (next, e) in (1u32..).zip(nodes) {
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
            for ole in dom.elements(parent, Some(&O::ole_object())) {
                if dom.attribute(ole, &shape_id).map(|s| s.to_string()) == Some(old.clone()) {
                    dom.set_attribute_value(ole, &shape_id, Some(&new));
                }
            }
        }
    }
}

/// Renumber every `v:shapetype/@id` from 1, in document order, and rewrite
/// the sibling `v:shape/@type` reference to the new id (both the bare
/// value and the `#…` prefixed form).
pub fn fix_up_shape_type_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let type_attr = XName::get("type", "");
    for (next, st) in (1u32..).zip(dom.descendants(root, Some(&VML::name("shapetype")))) {
        let old = dom.attribute(st, &id).map(|s| s.to_string());
        let new = next.to_string();
        dom.set_attribute_value(st, &id, Some(&new));
        if let (Some(old), Some(parent)) = (old, dom.parent(st)) {
            let want = format!("#{old}");
            for shape in dom.elements(parent, Some(&VML::name("shape"))) {
                if let Some(t) = dom.attribute(shape, &type_attr).map(|s| s.to_string()) {
                    if t == old {
                        dom.set_attribute_value(shape, &type_attr, Some(&new));
                    } else if t == want {
                        dom.set_attribute_value(shape, &type_attr, Some(&format!("#{new}")));
                    }
                }
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
    fix_up_drawing_ids_in_package_within(docx, crate::admission::InputLimits::compare())
}

/// Keep the identical-input comparer's already-admitted budget through the fixup.
pub(crate) fn fix_up_drawing_ids_in_package_within(
    docx: &[u8],
    limits: crate::admission::InputLimits,
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
    fix_up_doc_pr_ids(&mut dom, root);
    fix_up_shape_ids(&mut dom, root);
    fix_up_shape_type_ids(&mut dom, root);
    pkg.set_part(&main, dom.serialize_element(root).into_bytes());
    pkg.to_zip()
}
