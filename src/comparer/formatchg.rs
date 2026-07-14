//! M4.G — format-change detection. Port of DetectFormatChangesInAtomList (:4824),
//! GetRunPropertiesFromAtom (:4854), NormalizeRunProperties (:4884),
//! AreRunPropertiesEqual (:4868), GetChangedPropertyNames (:4919),
//! GetFriendlyPropertyName (:4974). settings.detect_format_changes defaults TRUE.

use std::collections::BTreeMap;

use crate::namespaces::{PT, W};
use crate::xmllinq::{Dom, NodeId, XName};

use super::atoms::{ComparisonUnitAtom, FormatChangeInfo};
use super::{CorrelationStatus, WmlComparerSettings};

/// `GetRunPropertiesFromAtom` — nearest `w:r` ancestor's `w:rPr` (None if none).
pub fn get_run_properties_from_atom(dom: &Dom, atom: &ComparisonUnitAtom) -> Option<NodeId> {
    let r = atom
        .ancestor_elements
        .iter()
        .copied()
        .find(|&a| dom.name(a) == Some(W::r()))?;
    dom.element(r, &W::r_pr())
}

fn is_rsid_attr(n: &XName) -> bool {
    n.local_name().to_lowercase().starts_with("rsid")
}

/// `NormalizeRunProperties` — canonical `w:rPr`: drop `w:rPrChange` + pt-ns
/// children; children sorted by local name; each child keeps name + filtered
/// (no rsid*, no pt) attrs sorted by local name; nested sub-elements dropped.
pub fn normalize_run_properties(dom: &mut Dom, rpr: Option<NodeId>) -> NodeId {
    let ne = dom.new_element(W::r_pr());
    if let Some(rpr) = rpr {
        let mut kids: Vec<NodeId> = dom
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| {
                let n = dom.name(c).unwrap();
                n != W::name("rPrChange") && n.namespace_name() != PT::URI
            })
            .collect();
        kids.sort_by_key(|&c| dom.name(c).unwrap().local_name().to_string());
        for c in kids {
            let cn = dom.name(c).unwrap();
            let child = dom.new_element(cn);
            let mut attrs: Vec<(XName, String)> = dom
                .attributes(c)
                .into_iter()
                .filter(|(an, _)| !is_rsid_attr(an) && an.namespace_name() != PT::URI)
                .collect();
            attrs.sort_by_key(|(an, _)| an.local_name().to_string());
            for (an, av) in attrs {
                dom.set_attribute_value(child, &an, Some(&av));
            }
            dom.add(ne, child);
        }
    }
    ne
}

/// `AreRunPropertiesEqual` — canonical-form equality (null ≡ empty rPr).
pub fn are_run_properties_equal(dom: &mut Dom, a: Option<NodeId>, b: Option<NodeId>) -> bool {
    let na = normalize_run_properties(dom, a);
    let nb = normalize_run_properties(dom, b);
    dom.serialize_element(na) == dom.serialize_element(nb)
}

/// Cached `serialize(normalize_run_properties(rpr))` keyed by the `rPr` NodeId.
///
/// [`are_run_properties_equal`] normalizes AND serializes both operands on every
/// call. In [`detect_format_changes_in_atom_list`] the same handful of distinct
/// `rPr` elements recur across thousands of `Equal` atoms (runs share one `rPr`),
/// so caching the normalized serialization collapses O(atoms) normalizations to
/// O(distinct rPr). The value equals `are_run_properties_equal`'s per-operand
/// serialization exactly, so a cached `==` of two such strings is that predicate.
fn normalized_rpr_serialized(
    dom: &mut Dom,
    cache: &mut std::collections::HashMap<Option<NodeId>, String>,
    rpr: Option<NodeId>,
) -> String {
    if let Some(s) = cache.get(&rpr) {
        return s.clone();
    }
    let ne = normalize_run_properties(dom, rpr);
    let s = dom.serialize_element(ne);
    cache.insert(rpr, s.clone());
    s
}

/// `GetFriendlyPropertyName`.
pub fn friendly_property_name(local: &str) -> String {
    let s = match local {
        "b" => "bold",
        "bCs" => "boldComplex",
        "i" => "italic",
        "iCs" => "italicComplex",
        "u" => "underline",
        "strike" => "strikethrough",
        "dstrike" => "doubleStrikethrough",
        "sz" => "fontSize",
        "szCs" => "fontSizeComplex",
        "rFonts" => "font",
        "color" => "color",
        "highlight" => "highlight",
        "shd" => "shading",
        "vertAlign" => "verticalAlign",
        "caps" => "allCaps",
        "smallCaps" => "smallCaps",
        "outline" => "outline",
        "shadow" => "shadow",
        "emboss" => "emboss",
        "imprint" => "imprint",
        "vanish" => "hidden",
        "spacing" => "characterSpacing",
        "w" => "characterWidth",
        "kern" => "kerning",
        "position" => "position",
        other => other,
    };
    s.to_string()
}

fn prop_signature(dom: &mut Dom, prop: NodeId) -> String {
    // NormalizePropertyElement: name + non-rsid attrs sorted; compare by serialization.
    let cn = dom.name(prop).unwrap();
    let pe = dom.new_element(cn);
    let mut attrs: Vec<(XName, String)> = dom
        .attributes(prop)
        .into_iter()
        .filter(|(an, _)| !is_rsid_attr(an))
        .collect();
    attrs.sort_by_key(|(an, _)| an.local_name().to_string());
    for (an, av) in attrs {
        dom.set_attribute_value(pe, &an, Some(&av));
    }
    dom.serialize_element(pe)
}

/// `GetChangedPropertyNames` — friendly names of props that differ (presence or value).
pub fn get_changed_property_names(
    dom: &mut Dom,
    old: Option<NodeId>,
    new: Option<NodeId>,
) -> Vec<String> {
    let collect = |dom: &Dom, rpr: Option<NodeId>| -> BTreeMap<String, NodeId> {
        let mut m = BTreeMap::new();
        if let Some(rpr) = rpr {
            for c in dom.elements(rpr, None) {
                let n = dom.name(c).unwrap();
                if n != W::name("rPrChange") {
                    m.insert(n.clark(), c);
                }
            }
        }
        m
    };
    let oldm = collect(dom, old);
    let newm = collect(dom, new);
    let mut names: Vec<String> = Vec::new();
    let mut keys: Vec<String> = oldm.keys().chain(newm.keys()).cloned().collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let changed = match (oldm.get(&k), newm.get(&k)) {
            (Some(&o), Some(&n)) => prop_signature(dom, o) != prop_signature(dom, n),
            _ => true,
        };
        if changed {
            names.push(friendly_property_name(XName::from_clark(&k).local_name()));
        }
    }
    names
}

/// A pending run format change: (atom index, old rPr, new rPr, changed names).
type PendingRunFormatChange = (usize, Option<NodeId>, Option<NodeId>, Vec<String>);
/// A pending para format change: (atom index, projected old pPr node).
type PendingParaFormatChange = (usize, NodeId);

/// Children of `w:pPr` that do not participate in paragraph-format comparison
/// (docxodus `IsParaComparisonNoiseChild`).
fn is_para_comparison_noise(dom: &Dom, child: NodeId) -> bool {
    let Some(n) = dom.name(child) else {
        return true;
    };
    n == W::r_pr()
        || n == W::name("sectPr")
        || n == W::name("pPrChange")
        || n.namespace_name() == PT::URI
}

/// True when projected pPr has only `w:jc` (center-alignment addition class).
fn projected_ppr_is_jc_only(dom: &Dom, ppr: NodeId) -> bool {
    let kids: Vec<_> = dom
        .elements(ppr, None)
        .into_iter()
        .filter(|&c| !is_para_comparison_noise(dom, c))
        .collect();
    kids.len() == 1 && dom.name(kids[0]) == Some(W::name("jc"))
}

/// True when projected pPr has only `w:spacing` (bare A → spaced B body class).
/// M130 (file_165): Word keeps live spacing + `pPrChange(empty old)` on
/// Verdana bare × Ultimate Demo spaced bodies. Broader addition floods file_8.
fn projected_ppr_is_spacing_only(dom: &Dom, ppr: NodeId) -> bool {
    let kids: Vec<_> = dom
        .elements(ppr, None)
        .into_iter()
        .filter(|&c| !is_para_comparison_noise(dom, c))
        .collect();
    kids.len() == 1 && dom.name(kids[0]) == Some(W::name("spacing"))
}

/// Project old-side pPr children for `w:pPrChange` (CT_PPrBase noise-stripped).
fn project_para_properties_for_change(dom: &mut Dom, ppr: NodeId) -> NodeId {
    let out = dom.new_element(W::p_pr());
    for c in dom.elements(ppr, None) {
        if is_para_comparison_noise(dom, c) {
            continue;
        }
        // Schema-implicit defaults: jc left/start restates the default.
        if dom.name(c) == Some(W::name("jc")) {
            let val = dom.attribute(c, &W::val()).unwrap_or("");
            if val == "left" || val == "start" {
                continue;
            }
        }
        let clone = dom.clone_subtree(c);
        dom.add(out, clone);
    }
    out
}

/// Canonical form of comparable pPr children for equality (local name + attrs).
fn normalize_para_properties(dom: &mut Dom, ppr: NodeId) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in dom.elements(ppr, None) {
        if is_para_comparison_noise(dom, c) {
            continue;
        }
        if dom.name(c) == Some(W::name("jc")) {
            let val = dom.attribute(c, &W::val()).unwrap_or("");
            if val == "left" || val == "start" {
                continue;
            }
        }
        parts.push(prop_signature(dom, c));
    }
    parts.sort();
    parts.join("\u{1}")
}

fn are_para_properties_equal(dom: &mut Dom, a: NodeId, b: NodeId) -> bool {
    normalize_para_properties(dom, a) == normalize_para_properties(dom, b)
}

/// M4.G.5 — `DetectFormatChangesInAtomList` (:4824 / docxodus :7111):
/// - Equal **run** atoms whose rPr differs → FormatChanged + OldRPr path
/// - Equal **pPr** (pilcrow) atoms whose para props differ → FormatChanged +
///   OldPPr path (M81: file_69 stamp after=20 → w:pPrChange)
pub fn detect_format_changes_in_atom_list(
    dom: &mut Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) {
    if !settings.detect_format_changes {
        return;
    }
    let mut run_changes: Vec<PendingRunFormatChange> = Vec::new();
    let mut para_changes: Vec<PendingParaFormatChange> = Vec::new();
    // Cache each distinct rPr's normalized serialization: runs share one rPr, so
    // this collapses O(Equal atoms) normalizations to O(distinct rPr). See
    // [`normalized_rpr_serialized`].
    let mut norm_cache: std::collections::HashMap<Option<NodeId>, String> =
        std::collections::HashMap::new();
    for (i, atom) in atoms.iter().enumerate() {
        if atom.correlation_status != CorrelationStatus::Equal {
            continue;
        }
        let Some(before) = atom.comparison_unit_atom_before.clone() else {
            continue;
        };
        // M81: paragraph-mark format change → w:pPrChange (docxodus :7127).
        // Gate: only when B clears properties A had (projected new empty, old
        // non-empty). Ungated equality fired ~99 pPrChange on file_8 (Word: 0)
        // and cost −2.8 score — Word emits pPrChange sparingly (file_69 stamp
        // after=20 → empty is the canonical case).
        if dom.name(atom.content_element) == Some(W::p_pr()) {
            let old_ppr = before.content_element;
            let new_ppr = atom.content_element;
            if dom.name(old_ppr) == Some(W::p_pr())
                && !are_para_properties_equal(dom, old_ppr, new_ppr)
            {
                let projected_old = project_para_properties_for_change(dom, old_ppr);
                let projected_new = project_para_properties_for_change(dom, new_ppr);
                let old_sig = normalize_para_properties(dom, projected_old);
                let new_sig = normalize_para_properties(dom, projected_new);
                // M81: property *removal* (A had layout, B cleared) → pPrChange(old).
                if !old_sig.is_empty() && new_sig.is_empty() {
                    para_changes.push((i, projected_old));
                } else if old_sig.is_empty()
                    && !new_sig.is_empty()
                    && (projected_ppr_is_jc_only(dom, projected_new)
                        || projected_ppr_is_spacing_only(dom, projected_new))
                {
                    // M102 (file_148): property *addition* of jc only (A bare,
                    // B center). Word keeps live jc + pPrChange(empty old).
                    // M130 (file_165): same for spacing-only addition (A bare
                    // Normal body, B before/after/line spacing). Broader
                    // addition flooded file_8; jc-only + spacing-only only.
                    let empty_old = dom.new_element(W::p_pr());
                    para_changes.push((i, empty_old));
                }
            }
            continue;
        }
        let old = get_run_properties_from_atom(dom, &before);
        let new = get_run_properties_from_atom(dom, atom);
        // Behavior-identical to `!are_run_properties_equal(dom, old, new)`: that
        // predicate is `serialize(normalize(old)) == serialize(normalize(new))`,
        // and the cache stores exactly those per-operand strings.
        if normalized_rpr_serialized(dom, &mut norm_cache, old)
            != normalized_rpr_serialized(dom, &mut norm_cache, new)
        {
            let changed = get_changed_property_names(dom, old, new);
            run_changes.push((i, old, new, changed));
        }
    }
    for (i, old, new, changed) in run_changes {
        atoms[i].correlation_status = CorrelationStatus::FormatChanged;
        atoms[i].format_change = Some(FormatChangeInfo {
            old_run_properties: old,
            new_run_properties: new,
            old_para_properties: None,
            changed_properties: changed,
        });
    }
    for (i, old_ppr) in para_changes {
        // M97 (file_30 stamp): when structural pPr FormatChanged, also capture
        // differing mark `pPr/rPr` so finalize can nest `w:rPrChange` under
        // live mark rPr (Word: Aptos/b/sz20/u → sz32).
        let old_mark_rpr = atoms[i].comparison_unit_atom_before.as_ref().and_then(|b| {
            if dom.name(b.content_element) == Some(W::p_pr()) {
                dom.element(b.content_element, &W::r_pr())
            } else {
                None
            }
        });
        let new_mark_rpr = if dom.name(atoms[i].content_element) == Some(W::p_pr()) {
            dom.element(atoms[i].content_element, &W::r_pr())
        } else {
            None
        };
        let mark_old = if !are_run_properties_equal(dom, old_mark_rpr, new_mark_rpr) {
            old_mark_rpr
        } else {
            None
        };
        atoms[i].correlation_status = CorrelationStatus::FormatChanged;
        atoms[i].format_change = Some(FormatChangeInfo {
            old_run_properties: mark_old,
            new_run_properties: new_mark_rpr,
            old_para_properties: Some(old_ppr),
            changed_properties: vec!["paragraphFormatting".into()],
        });
    }
}

/// PR4 — the per-`rPr` serialization cache MUST return exactly what a direct
/// `normalize_run_properties` + `serialize_element` produces, on the first call
/// and on cache hits, so that swapping [`are_run_properties_equal`] for a cached
/// `==` in [`detect_format_changes_in_atom_list`] is behavior-preserving.
#[cfg(test)]
mod format_change_cache_tests {
    use super::*;

    fn rpr_with(dom: &mut Dom, children: &[(&str, &[(&str, &str)])]) -> NodeId {
        let rpr = dom.new_element(W::r_pr());
        for (local, attrs) in children {
            let c = dom.new_element(W::name(local));
            for (an, av) in *attrs {
                dom.set_attribute_value(c, &W::name(an), Some(av));
            }
            dom.add(rpr, c);
        }
        rpr
    }

    fn direct(dom: &mut Dom, rpr: Option<NodeId>) -> String {
        let ne = normalize_run_properties(dom, rpr);
        dom.serialize_element(ne)
    }

    #[test]
    fn format_changes_cache_matches_direct() {
        let mut dom = Dom::new();
        let bold = rpr_with(&mut dom, &[("b", &[])]);
        let bold_sz = rpr_with(&mut dom, &[("b", &[]), ("sz", &[("val", "24")])]);
        // Same props, different source order — normalization must canonicalize both.
        let sz_bold = rpr_with(&mut dom, &[("sz", &[("val", "24")]), ("b", &[])]);
        let empty = dom.new_element(W::r_pr());
        let cases = [Some(bold), Some(bold_sz), Some(sz_bold), Some(empty), None];

        let mut cache = std::collections::HashMap::new();
        for &rpr in &cases {
            let want = direct(&mut dom, rpr);
            let got = normalized_rpr_serialized(&mut dom, &mut cache, rpr);
            assert_eq!(got, want, "cached != direct for {rpr:?}");
            // Cache hit must return the same value, not diverge.
            let got2 = normalized_rpr_serialized(&mut dom, &mut cache, rpr);
            assert_eq!(got2, want, "cache-hit != direct for {rpr:?}");
        }
        // bold_sz and sz_bold have identical properties ⇒ identical normal form.
        assert_eq!(
            normalized_rpr_serialized(&mut dom, &mut cache, Some(bold_sz)),
            normalized_rpr_serialized(&mut dom, &mut cache, Some(sz_bold)),
            "canonicalization must ignore source child order"
        );
    }
}
