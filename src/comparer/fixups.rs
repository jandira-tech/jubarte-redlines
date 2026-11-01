// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drawing / shape id repair.
//!
//! The produced document splices content from two source packages. DrawingML
//! ids are renumbered; VML ids are preserved unless they collide, sharing one
//! space across shapes, shapetypes, groups and other VML elements (#1800).
//! References follow the renamed definition. The compare pipeline calls
//! [`fix_up_drawing_ids`] mid-stream and after the Word-mode merge passes.
//! The identical-input shortcut calls `rename_repeated_doc_pr_ids` only:
//! unique DrawingML ids and schema-permitted repeated VML ids stay untouched.
//!
//! [`fix_up_group_ids`] remains the standalone upstream consolidate helper;
//! compare uses the shared VML repair instead.

use std::collections::HashSet;

use crate::namespaces::{MC, O, PT, VML, W};
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
pub(crate) fn rename_repeated_doc_pr_ids(dom: &mut Dom, root: NodeId) {
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
/// and `group`, plus `o:spid`. Rewrite references when their targets change;
/// leave AlternateContent Fallback copies untouched.
pub fn fix_up_shape_ids(dom: &mut Dom, root: NodeId) {
    repair_vml_ids(dom, root);
}

/// Repair VML ids and their type references in one pass, just as
/// [`fix_up_shape_ids`] does. Kept as an entry point for callers that used
/// the former separate shapetype pass; running both is idempotent.
pub fn fix_up_shape_type_ids(dom: &mut Dom, root: NodeId) {
    repair_vml_ids(dom, root);
}

/// Carry source ownership through cloning and merging. Stamping after block
/// hashing keeps it out of content identity; the normal scratch cleanup drops
/// it after the first repair. Forward references must not bind the other
/// source's same-named definition just because it was emitted first (#1800).
pub(crate) fn stamp_vml_source(dom: &mut Dom, root: NodeId) {
    let source = root.0.to_string();
    for node in dom.descendants(root, None) {
        if (is_vml(dom, node)
            || dom.name_is(node, &O::ole_object())
            || dom.name_is(node, &W::name("control")))
            && !in_fallback(dom, node)
        {
            dom.set_attribute_value(node, &PT::name("VmlSource"), Some(&source));
        }
    }
}

fn is_vml(dom: &Dom, node: NodeId) -> bool {
    dom.name(node)
        .is_some_and(|name| name.namespace_name() == VML::URI)
}

fn in_fallback(dom: &Dom, node: NodeId) -> bool {
    !dom.ancestors(node, Some(&MC::name("Fallback"))).is_empty()
}

struct VmlDefinition {
    node: NodeId,
    position: usize,
    old: String,
    new: String,
}

/// Resolve within the originating copy first, then the merged part (a shared
/// definition can have been coalesced). Prefer an enclosing/self definition,
/// otherwise the nearest preceding definition, else the first following one.
fn vml_target<'a>(
    dom: &Dom,
    definitions: &'a [VmlDefinition],
    node: NodeId,
    position: usize,
    key: &str,
    kind: Option<&XName>,
) -> Option<&'a str> {
    let candidates: Vec<_> = definitions
        .iter()
        .filter(|d| d.old == key && kind.is_none_or(|kind| dom.name_is(d.node, kind)))
        .collect();
    let source_attr = PT::name("VmlSource");
    let source = dom.attribute(node, &source_attr);
    let same_source =
        |d: &&VmlDefinition| source.is_none() || dom.attribute(d.node, &source_attr) == source;
    let own: Vec<_> = candidates.iter().copied().filter(same_source).collect();
    let candidates = if own.is_empty() { &candidates } else { &own };
    let ancestors = dom.ancestors(node, None);
    candidates
        .iter()
        .find(|d| d.node == node || ancestors.contains(&d.node))
        .or_else(|| candidates.iter().rev().find(|d| d.position <= position))
        .or_else(|| candidates.first())
        .map(|d| d.new.as_str())
}

/// Keep ids and spids in one part-scoped space. Fallback twins reserve names
/// but are never rewritten: their ids pair with the Choice's docPr names.
/// Snapshot definitions before relinking all id-valued references.
fn repair_vml_ids(dom: &mut Dom, root: NodeId) {
    let id = id_attr();
    let spid = O::name("spid");
    let nodes = dom.descendants(root, None);
    let mut taken = HashSet::new();
    for &node in &nodes {
        if is_vml(dom, node) {
            for attr in [&id, &spid] {
                if let Some(value) = dom.attribute(node, attr) {
                    taken.insert(value.to_string());
                }
            }
        }
    }
    let mut seen = HashSet::new();
    let mut definitions = Vec::new();
    for (position, &node) in nodes.iter().enumerate() {
        if !is_vml(dom, node) || in_fallback(dom, node) {
            continue;
        }
        let old_id = dom.attribute(node, &id).map(str::to_string);
        let mut new_id = None;
        for attr in [&id, &spid] {
            let Some(old) = dom.attribute(node, attr).map(str::to_string) else {
                continue;
            };
            // id and spid may be aliases on the same shape, not a collision.
            let new = if attr == &spid && old_id.as_deref() == Some(&old) {
                new_id.clone().unwrap_or_else(|| old.clone())
            } else if seen.insert(old.clone()) {
                old.clone()
            } else if old.starts_with("_x0000_s")
                && (attr == &spid || dom.attribute(node, &spid) == Some(old.as_str()))
            {
                let fresh = (1025u32..)
                    .map(|n| format!("_x0000_s{n}"))
                    .find(|candidate| !taken.contains(candidate))
                    .unwrap();
                taken.insert(fresh.clone());
                fresh
            } else {
                unused_id(&old, &mut taken)
            };
            if new != old {
                dom.set_attribute_value(node, attr, Some(&new));
            }
            if attr == &id {
                new_id = Some(new.clone());
                definitions.push(VmlDefinition {
                    node,
                    position,
                    old,
                    new,
                });
            }
        }
    }
    for (position, &node) in nodes.iter().enumerate() {
        let mut refs = Vec::new();
        if is_vml(dom, node) {
            refs.push((XName::get("type", ""), Some(VML::shapetype()), false));
            refs.push((O::name("regroupid"), Some(VML::group()), false));
            refs.push((XName::get("href", ""), None, true));
        } else if dom.name_is(node, &O::ole_object()) {
            refs.push((XName::get("ShapeID", ""), Some(VML::shape()), false));
        } else if dom.name_is(node, &W::name("control")) {
            refs.push((W::name("shapeid"), Some(VML::shape()), false));
        }
        if refs.is_empty() || in_fallback(dom, node) {
            continue;
        }
        for (attr, kind, fragment_only) in refs {
            let Some(value) = dom.attribute(node, &attr).map(str::to_string) else {
                continue;
            };
            let (prefix, key) = match value.strip_prefix('#') {
                Some(key) => ("#", key),
                None if fragment_only => continue,
                None => ("", value.as_str()),
            };
            if let Some(new) = vml_target(dom, &definitions, node, position, key, kind.as_ref()) {
                let new = format!("{prefix}{new}");
                if new != value {
                    dom.set_attribute_value(node, &attr, Some(&new));
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
    for (next, g) in (1u32..).zip(dom.descendants(root, Some(&VML::group()))) {
        dom.set_attribute_value(g, &id, Some(&next.to_string()));
    }
}

/// Apply the DrawingML and shared VML id fixups to a
/// package's main document and write the redlined package back to bytes.
///
/// Standalone byte facade for callers that need full drawing normalization.
/// The identical-input comparer repairs only invalid DrawingML ids directly
/// in its admitted package, without this facade's serialization/reopening.
//
///
/// # Errors
///
/// [`crate::opc::OpcError`] when the package cannot be opened, the main
/// document part is absent, or the part has no root element after parsing.
pub fn fix_up_drawing_ids_in_package(docx: &[u8]) -> Result<Vec<u8>, crate::opc::OpcError> {
    fix_up_drawing_ids_in_package_within(docx, crate::admission::InputLimits::compare(), false)
}

/// Keep the caller's admission budget through the byte facade;
/// `preserve_untouched` as in [`fix_up_drawing_ids`].
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
