// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `MarkupSimplifier` comparison subset (M2).
//!
//! Port of `SingleCharacterRunTransform` (MarkupSimplifier.ts), the
//! rsid-stripper `remove_rsid_transform`, and the entry point that runs
//! it on a non-run container (`transform_element_to_single_character_runs`).
//! These are tree-rebuild transforms in the TS source; in the arena we
//! build fresh nodes and return them. The comparer calls
//! `remove_rsid_transform` from every accept / reject pass; the
//! single-character-run transform is exposed for the WASM bindings and
//! for the corpus reconciliation tests.

use crate::namespaces::{MC, W};
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

/// Port of `SingleCharacterRunTransform(node)` — returns the transformed
/// node(s).
///
/// Non-elements pass through; a `w:r` explodes into one run per character
/// (text) or per child element (non-text); other elements rebuild
/// recursively. A one-character `w:t` that holds XML whitespace (space,
/// tab, CR, LF) gets `xml:space="preserve"`.
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

/// `TransformElementToSingleCharacterRuns(element)` — returns the single
/// rebuilt root of a non-`w:r` container such as the body or a paragraph.
///
/// # Behaviour
///
/// A `w:r` explodes into zero, one or many runs, so it has no single
/// rebuilt root. A `w:r` `element` is outside this function's contract: it
/// is returned unchanged and the DOM is left as it was (use
/// [`single_character_run_transform`] to explode a run into its runs).
/// Every other element, and a non-element node, yields exactly one node.
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

/// Port of `RemoveRsidTransform(node)` — drops every `w:rsid*` element and
/// every `w:rsid*` attribute across the subtree, but a table row's own:
/// Word keeps them, and the renderer keeps the saved grid of a table whose
/// rows carry them (#1716), so a redline or Accept All of a Word-saved
/// document lays its tables out as the source does (#1873). Returns `None`
/// when the node itself is dropped (a `w:rsid` element).
///
/// RSID-INPLACE-01: mutates the existing subtree in place (same root
/// `NodeId`) instead of rebuilding every node. Observable XML after the
/// transform is unchanged; only intermediate allocation / drop work is
/// removed.
pub fn remove_rsid_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    strip_rsids(dom, node, true)
}

/// [`remove_rsid_transform`] without its exception: a table row's
/// `w:rsid*` go too, as scrub promises every edit-session id is gone.
pub fn remove_all_rsids(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    strip_rsids(dom, node, false)
}

fn strip_rsids(dom: &mut Dom, node: NodeId, keep_rows: bool) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::name("rsid") {
        return None;
    }
    // Strip rsid* attributes without rebuilding the element.
    let rsid_attrs: Vec<_> = if keep_rows && name == W::tr() {
        Vec::new()
    } else {
        dom.attributes_iter(node)
            .filter(|(an, _)| is_rsid_attr(an))
            .map(|(an, _)| an.clone())
            .collect()
    };
    for an in &rsid_attrs {
        dom.set_attribute_value(node, an, None);
    }
    // Snapshot children — recursion may remove `w:rsid` siblings.
    let kids: Vec<NodeId> = (0..dom.child_count(node))
        .map(|i| dom.child_at(node, i))
        .collect();
    for k in kids {
        if strip_rsids(dom, k, keep_rows).is_none() {
            dom.remove(k);
        }
    }
    Some(node)
}

/// Resolve each non-drawing `mc:AlternateContent` under `root`, picking the
/// first `mc:Choice` whose `@Requires` namespaces satisfy `understood`, or
/// else `mc:Fallback`.
pub(crate) fn resolve_alternate_content_with(
    dom: &mut Dom,
    root: NodeId,
    understood: fn(&str) -> bool,
) {
    for ac in dom.descendants_and_self(root, Some(&MC::alternate_content())) {
        // A nested AltContent inside a discarded Fallback may already be detached.
        if dom.parent(ac).is_none() {
            continue;
        }
        // Keep drawing/VML fallbacks (Word does); resolve only text feature-gating.
        let has_drawing = !dom.descendants(ac, Some(&W::drawing())).is_empty()
            || !dom.descendants(ac, Some(&W::pict())).is_empty();
        if has_drawing {
            continue;
        }
        // Anchor-internal wrappers (e.g. wp:positionH gating wp14:pctPosHOffset
        // vs a wp:posOffset fallback) must survive verbatim: Word keeps them,
        // and resolving to the wp14 Choice leaves the parent schema-EMPTY for
        // consumers that ignore wp14 → "unreadable content" (strict01 cover
        // page, bisected in real Word). The whole w:drawing is one atom, so
        // the original hoisting concern doesn't apply inside it.
        let inside_drawing = dom
            .ancestors_and_self(ac, None)
            .into_iter()
            .any(|a| dom.name_is(a, &W::drawing()));
        if inside_drawing {
            continue;
        }
        let src = dom
            .elements(ac, Some(&MC::choice()))
            .into_iter()
            .find(|&choice| choice_requires_understood(dom, choice, understood))
            .or_else(|| dom.element(ac, &MC::fallback()));
        if let Some(src) = src {
            let kids = dom.nodes(src);
            dom.replace_with(ac, &kids);
        }
    }
}

/// True if every namespace prefix listed in a `mc:Choice`'s `@Requires` is bound,
/// where it is named, to a namespace `understood` accepts (Word's:
/// [`word_understands_namespace`]). Word 16 passes over a Choice gated on a
/// namespace it does not know, though the document declares it, for the next
/// Choice, else the Fallback (#1876, probed). An absent/empty `@Requires` is
/// vacuously true (the schema requires it, but be lenient). The `@Requires`
/// attribute is unprefixed (no namespace) per the MCE schema.
pub(crate) fn choice_requires_understood(
    dom: &Dom,
    choice: NodeId,
    understood: fn(&str) -> bool,
) -> bool {
    static REQUIRES: std::sync::OnceLock<XName> = std::sync::OnceLock::new();
    let requires_name = REQUIRES.get_or_init(|| XNamespace::none().name("Requires"));
    let Some(requires) = dom.attribute(choice, requires_name) else {
        return true;
    };
    requires
        .split_whitespace()
        .all(|prefix| prefix_namespace(dom, choice, prefix).is_some_and(|uri| understood(&uri)))
}

/// The namespace the nearest `xmlns:prefix` declaration on `node` or its
/// ancestors binds `prefix` to.
fn prefix_namespace(dom: &Dom, node: NodeId, prefix: &str) -> Option<String> {
    dom.ancestors_and_self(node, None)
        .into_iter()
        .find_map(|anc| {
            dom.attributes_iter(anc)
                .find(|(name, _)| dom.is_namespace_declaration(name) && name.local_name() == prefix)
                .map(|(_, uri)| uri.to_string())
        })
}

/// A namespace Word knows: the OOXML standard's (transitional and strict),
/// Office's own extensions, and VML's.
pub(crate) fn word_understands_namespace(uri: &str) -> bool {
    [
        "http://schemas.openxmlformats.org/",
        "http://purl.oclc.org/ooxml/",
        "http://schemas.microsoft.com/office/",
        "urn:schemas-microsoft-com:",
    ]
    .iter()
    .any(|root| uri.starts_with(root))
}

/// Every prefix in a Requires list passes `prefix_understood`. No list means
/// the choice stands.
pub(crate) fn requires_list_understood(
    requires: Option<&str>,
    prefix_understood: impl Fn(&str) -> bool,
) -> bool {
    requires.is_none_or(|requires| requires.split_whitespace().all(prefix_understood))
}

/// The branch Word shows: the first Choice whose Requires it understands,
/// otherwise the Fallback.
pub(crate) fn selected_alternate_branch(dom: &Dom, alternate: NodeId) -> Option<NodeId> {
    dom.elements(alternate, Some(&MC::choice()))
        .into_iter()
        .find(|&choice| {
            requires_list_understood(
                dom.attribute(choice, &XNamespace::none().name("Requires")),
                |prefix| {
                    prefix_namespace(dom, choice, prefix)
                        .is_some_and(|uri| word_understands_namespace(&uri))
                },
            )
        })
        .or_else(|| dom.element(alternate, &MC::fallback()))
}
