//! M4.F.6 — id-renumbering fixups. Port of FixUpDocPrIds (:5937),
//! FixUpShapeIds (:5964), FixUpGroupIds (:5986), FixUpShapeTypeIds (:6002).
//! The produced document splices content from two source docs, so original ids
//! collide → Word "needs repair". Each fixup renumbers a target element's `id`
//! (no-namespace) sequentially, fixing the linked attribute where applicable.

use crate::namespaces::{O, VML, WP};
use crate::xmllinq::{Dom, NodeId, XName};

fn id_attr() -> XName {
    XName::get("id", "")
}

/// `FixUpDocPrIds` (:5937) — `wp:docPr/@id` from 1.
///
/// Matches every element whose local name is `docPr` (not only the transitional
/// WP URI) so Strict / remapped drawings are renumbered too.
pub fn fix_up_doc_pr_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let nodes: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&e| {
            dom.name(e).is_some_and(|n| n.local_name() == "docPr")
                || dom.name(e).as_ref() == Some(&WP::name("docPr"))
        })
        .collect();
    // Dedup while preserving document order (filter may match twice if WP==local).
    let mut seen = std::collections::HashSet::new();
    let nodes: Vec<NodeId> = nodes.into_iter().filter(|n| seen.insert(*n)).collect();
    for (next, e) in (1u32..).zip(nodes) {
        dom.set_attribute_value(e, &id, Some(&next.to_string()));
    }
}

/// `FixUpShapeIds` (:5964) — `v:shape/@id` from 1, rewriting the sibling
/// `o:OLEObject/@ShapeID` to match (keeps OLE ↔ shape linkage).
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

/// `FixUpShapeTypeIds` (:6002) — `v:shapetype/@id` from 1, rewriting sibling
/// `v:shape/@type` to match.
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

/// `FixUpGroupIds` (:5986) — `v:group/@id` from 1.
pub fn fix_up_group_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    for (next, g) in (1u32..).zip(dom.descendants(root, Some(&VML::name("group")))) {
        dom.set_attribute_value(g, &id, Some(&next.to_string()));
    }
}

/// Apply the produce-path drawing/shape id fixups to a package's main document.
/// Used by the IDENTICAL-INPUT short-circuit so empty redlines still renumber
/// colliding source `wp:docPr/@id` values the full pipeline would have fixed.
pub fn fix_up_drawing_ids_in_package(docx: &[u8]) -> Result<Vec<u8>, crate::opc::OpcError> {
    use crate::opc::PartFs;

    let mut pkg = PartFs::open(docx)?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return Ok(docx.to_vec());
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return Ok(docx.to_vec());
    };
    fix_up_doc_pr_ids(&mut dom, root);
    fix_up_shape_ids(&mut dom, root);
    fix_up_shape_type_ids(&mut dom, root);
    pkg.set_part(&main, dom.serialize_element(root).into_bytes());
    pkg.to_zip()
}
