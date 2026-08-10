// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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
        .find(|&a| dom.name_is(a, &W::r()))?;
    dom.element(r, &W::r_pr())
}

fn is_rsid_attr(n: &XName) -> bool {
    n.local_name().to_lowercase().starts_with("rsid")
}

/// One canonical `w:rPr` child: element name plus its filtered, sorted attrs.
type CanonicalRprChild = (XName, Vec<(XName, String)>);

/// `NormalizeRunProperties` (read half) — extract `rpr`'s canonical child spec from
/// `src` with PURE READS: drop `w:rPrChange` + pt-ns children; children sorted by
/// local name; each child keeps name + filtered (no rsid*, no pt) attrs sorted by
/// local name; nested sub-elements dropped. Because it only reads, the spec can then
/// be materialized into ANY arena — `dom` itself (same-dom normalize) or a throwaway
/// scratch arena (see [`normalized_rpr_serialized`]).
fn canonical_rpr_spec(src: &Dom, rpr: Option<NodeId>) -> Vec<CanonicalRprChild> {
    let mut spec: Vec<CanonicalRprChild> = Vec::new();
    if let Some(rpr) = rpr {
        let mut kids: Vec<NodeId> = src
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| {
                let n = src.name(c).unwrap();
                n != W::name("rPrChange") && n.namespace_name() != PT::URI
            })
            .collect();
        // ALLOC-LEAN-01: sort_by (compare &str) not sort_by_key(to_string) — the
        // latter heap-allocates a String on EVERY comparison (keys are not cached),
        // ~21M allocs on the dissertation. Same total order ⇒ byte-identical.
        kids.sort_by(|&a, &b| {
            let na = src.name(a).unwrap();
            let nb = src.name(b).unwrap();
            na.local_name().cmp(nb.local_name())
        });
        for c in kids {
            let cn = src.name(c).unwrap();
            let mut attrs: Vec<(XName, String)> = src
                .attributes(c)
                .into_iter()
                .filter(|(an, _)| !is_rsid_attr(an) && an.namespace_name() != PT::URI)
                .collect();
            attrs.sort_by(|(a, _), (b, _)| a.local_name().cmp(b.local_name()));
            spec.push((cn, attrs));
        }
    }
    spec
}

/// `NormalizeRunProperties` (build half) — materialize a canonical `w:rPr` from a
/// spec into `dst`, returning its NodeId. Reproduces the original single-pass build's
/// exact node/attr order, so `serialize_element` is byte-identical to the pre-split
/// code (guarded by `format_changes_cache_matches_direct`).
fn build_canonical_rpr(dst: &mut Dom, spec: &[CanonicalRprChild]) -> NodeId {
    let ne = dst.new_element(W::r_pr());
    for (cn, attrs) in spec {
        let child = dst.new_element(cn.clone());
        for (an, av) in attrs {
            dst.set_attribute_value(child, an, Some(av.as_str()));
        }
        dst.add(ne, child);
    }
    ne
}

/// `NormalizeRunProperties` — canonical `w:rPr` built in the same arena `dom`. The
/// scratch-serialization path ([`normalized_rpr_serialized`]) instead builds into a
/// DEDICATED arena so the persistent one is never enlarged (MEM-ATTRIBUTE-01).
pub fn normalize_run_properties(dom: &mut Dom, rpr: Option<NodeId>) -> NodeId {
    let spec = canonical_rpr_spec(dom, rpr);
    build_canonical_rpr(dom, &spec)
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
    dom: &Dom,
    scratch: &mut Dom,
    cache: &mut std::collections::HashMap<Option<NodeId>, String>,
    rpr: Option<NodeId>,
) -> String {
    if let Some(s) = cache.get(&rpr) {
        return s.clone();
    }
    // Normalization is SCRATCH: it materializes a canonical `w:rPr` element only to
    // serialize it to a `String`. Read the spec off the persistent arena (pure reads)
    // and build+serialize it in a DEDICATED scratch arena, reclaimed per call. On
    // run-fragmented documents the distinct-NodeId rPrs miss this cache in the
    // thousands; building into the persistent arena reallocated its backing `Vec` to
    // the next doubling tier and pinned it — the multi-GB single allocation that
    // dominated the compare peak (MEM-ATTRIBUTE-01). `with_scratch` alone truncated
    // LENGTH but not CAPACITY, so a separate arena is required, not just reclamation.
    // The serialized bytes are unchanged, so the format-change verdict is identical.
    let spec = canonical_rpr_spec(dom, rpr);
    let s = scratch.with_scratch(|d| {
        let ne = build_canonical_rpr(d, &spec);
        d.serialize_element(ne)
    });
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
    // ALLOC-LEAN-01: compare &str, don't allocate a String key per comparison.
    attrs.sort_by(|(a, _), (b, _)| a.local_name().cmp(b.local_name()));
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
    kids.len() == 1 && dom.name_is(kids[0], &W::name("jc"))
}

/// First non-default `w:jc` child of a projected pPr, if any.
fn projected_ppr_jc(dom: &Dom, ppr: NodeId) -> Option<NodeId> {
    for c in dom.elements(ppr, None) {
        if is_para_comparison_noise(dom, c) {
            continue;
        }
        if dom.name_is(c, &W::name("jc")) {
            let val = dom.attribute(c, &W::val()).unwrap_or("");
            if val != "left" && val != "start" {
                return Some(c);
            }
        }
    }
    None
}

/// Project only the non-default `w:jc` from old pPr (justify/center removal class).
fn project_jc_only_from(dom: &mut Dom, ppr: NodeId) -> Option<NodeId> {
    let jc = projected_ppr_jc(dom, ppr)?;
    let out = dom.new_element(W::p_pr());
    let clone = dom.clone_subtree(jc);
    dom.add(out, clone);
    Some(out)
}

/// Signature of projected pPr with jc children ignored (for partial-removal gate).
fn normalize_para_properties_without_jc(dom: &mut Dom, ppr: NodeId) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in dom.elements(ppr, None) {
        if is_para_comparison_noise(dom, c) {
            continue;
        }
        if dom.name_is(c, &W::name("jc")) {
            continue;
        }
        parts.push(prop_signature(dom, c));
    }
    parts.sort();
    parts.join("\u{1}")
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
    kids.len() == 1 && dom.name_is(kids[0], &W::name("spacing"))
}

/// Project old-side pPr children for `w:pPrChange` (CT_PPrBase noise-stripped).
fn project_para_properties_for_change(dom: &mut Dom, ppr: NodeId) -> NodeId {
    let out = dom.new_element(W::p_pr());
    for c in dom.elements(ppr, None) {
        if is_para_comparison_noise(dom, c) {
            continue;
        }
        // Schema-implicit defaults: jc left/start restates the default.
        if dom.name_is(c, &W::name("jc")) {
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
        if dom.name_is(c, &W::name("jc")) {
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
    detect_format_changes_impl(dom, atoms, settings, true);
}

/// Uncached oracle (pre-PR4 behaviour: `are_run_properties_equal` directly). Kept
/// to prove the per-rPr serialization cache preserves the EXACT format-change
/// retagging on the same atoms — see `cached_format_changes_match_uncached`. This
/// is the direct guard the review asked for (the corpus goldens guard it only
/// through the volatility-tolerant structural comparator).
#[cfg(test)]
fn detect_format_changes_reference(
    dom: &mut Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) {
    detect_format_changes_impl(dom, atoms, settings, false);
}

fn detect_format_changes_impl(
    dom: &mut Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
    use_cache: bool,
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
    // Dedicated throwaway arena for rPr-normalization serialization, kept SEPARATE
    // from `dom`: the per-rPr scratch build must never enlarge the persistent arena
    // (MEM-ATTRIBUTE-01 — that build was the multi-GB single allocation at peak).
    let mut scratch = Dom::new();
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
        if dom.name_is(atom.content_element, &W::p_pr()) {
            let old_ppr = before.content_element;
            let new_ppr = atom.content_element;
            if dom.name_is(old_ppr, &W::p_pr())
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
                } else if projected_ppr_jc(dom, projected_old).is_some()
                    && projected_ppr_jc(dom, projected_new).is_none()
                    && normalize_para_properties_without_jc(dom, projected_old)
                        == normalize_para_properties_without_jc(dom, projected_new)
                {
                    // M227 (justify×large_font ~78, center×center_bold): A had
                    // non-default jc, B dropped it while other layout (e.g.
                    // line=276) stayed equal. Full-clear M81 misses this —
                    // new_sig is non-empty. Word emits pPrChange(jc only).
                    if let Some(jc_old) = project_jc_only_from(dom, projected_old) {
                        para_changes.push((i, jc_old));
                    }
                }
            }
            continue;
        }
        let old = get_run_properties_from_atom(dom, &before);
        let new = get_run_properties_from_atom(dom, atom);
        // Behavior-identical to `!are_run_properties_equal(dom, old, new)`: that
        // predicate is `serialize(normalize(old)) == serialize(normalize(new))`,
        // and the cache stores exactly those per-operand strings. The uncached
        // branch is the equivalence oracle (test builds only).
        let differ = if use_cache {
            normalized_rpr_serialized(dom, &mut scratch, &mut norm_cache, old)
                != normalized_rpr_serialized(dom, &mut scratch, &mut norm_cache, new)
        } else {
            !are_run_properties_equal(dom, old, new)
        };
        if differ {
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
            if dom.name_is(b.content_element, &W::p_pr()) {
                dom.element(b.content_element, &W::r_pr())
            } else {
                None
            }
        });
        let new_mark_rpr = if dom.name_is(atoms[i].content_element, &W::p_pr()) {
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
        // Duplicate local names — relies on normalize's stable sort; must cache the
        // same string as a direct call (review residual risk).
        let dup = rpr_with(&mut dom, &[("b", &[("val", "1")]), ("b", &[("val", "0")])]);
        let empty = dom.new_element(W::r_pr());
        let cases = [
            Some(bold),
            Some(bold_sz),
            Some(sz_bold),
            Some(dup),
            Some(empty),
            None,
        ];

        let mut cache = std::collections::HashMap::new();
        let mut scratch = Dom::new();
        for &rpr in &cases {
            let want = direct(&mut dom, rpr);
            let got = normalized_rpr_serialized(&dom, &mut scratch, &mut cache, rpr);
            assert_eq!(got, want, "cached != direct for {rpr:?}");
            // Cache hit must return the same value, not diverge.
            let got2 = normalized_rpr_serialized(&dom, &mut scratch, &mut cache, rpr);
            assert_eq!(got2, want, "cache-hit != direct for {rpr:?}");
        }
        // bold_sz and sz_bold have identical properties ⇒ identical normal form.
        assert_eq!(
            normalized_rpr_serialized(&dom, &mut scratch, &mut cache, Some(bold_sz)),
            normalized_rpr_serialized(&dom, &mut scratch, &mut cache, Some(sz_bold)),
            "canonicalization must ignore source child order"
        );
    }

    /// MEM-ATTRIBUTE-01 regression: normalizing an `rPr` is SCRATCH work whose only
    /// output is a `String`, so it must not leave throwaway nodes in the persistent
    /// arena. On run-fragmented documents each of thousands of runs carries its own
    /// (identical-content, distinct-NodeId) `rPr`, defeating the NodeId-keyed cache;
    /// the old code then leaked ~one normalized subtree per run, growing the arena
    /// `Vec` into the multi-GB single block that dominated the compare peak.
    #[test]
    fn normalized_rpr_serialization_reclaims_scratch_nodes() {
        let mut dom = Dom::new();
        // 500 DISTINCT rPr nodes with identical content — every lookup is a cache
        // miss (distinct NodeId keys), so every call normalizes afresh.
        let rprs: Vec<NodeId> = (0..500)
            .map(|_| {
                rpr_with(
                    &mut dom,
                    &[("b", &[]), ("sz", &[("val", "24")]), ("i", &[])],
                )
            })
            .collect();
        let mut cache = std::collections::HashMap::new();
        let mut scratch = Dom::new();
        let before = dom.node_count();
        let mut last = String::new();
        for &rpr in &rprs {
            last = normalized_rpr_serialized(&dom, &mut scratch, &mut cache, Some(rpr));
        }
        let grew = dom.node_count() - before;
        assert!(!last.is_empty(), "sanity: normalization produced output");
        // Each normalization builds `w:rPr` + 3 children in the DEDICATED scratch arena
        // and reclaims them per call. The persistent `dom` is only read, so it must not
        // grow by a single node — the old same-arena path leaked all 4 per distinct rPr
        // (~2000 here) into the persistent `Vec`.
        assert_eq!(
            grew,
            0,
            "normalizing {} distinct rPr grew the persistent arena by {grew} nodes; \
             scratch normalization must build in its own arena",
            rprs.len()
        );
    }

    /// FMT-SCRATCH-02 (MEM-ATTRIBUTE-01): rPr normalization must not merely RECLAIM
    /// its throwaway nodes from the production arena (that is FMT-SCRATCH-01) — it
    /// must never ALLOCATE into it at all. `with_scratch` truncates the shared arena's
    /// LENGTH but not its CAPACITY, so the first normalize push against a near-full
    /// arena reallocs the entire backing `Vec` to the next doubling tier and pins it
    /// (on the dissertation, the 3 GiB single block that dominated the compare peak).
    /// Building the scratch element in a DEDICATED arena leaves production capacity
    /// untouched. Driven through the public entry point, whose signature is stable.
    #[test]
    fn detect_format_changes_never_grows_production_arena_capacity() {
        type Props<'a> = &'a [(&'a str, &'a [(&'a str, &'a str)])];
        let mut dom = Dom::new();
        // before == after CONTENT (so no atom is retagged — isolates the pure
        // normalize/serialize scratch path from the get_changed_property_names and
        // pPr-projection paths), but each rPr is a DISTINCT NodeId so every one of the
        // two per-atom normalizations is a cache miss and actually runs.
        let props: Props = &[("b", &[]), ("sz", &[("val", "24")]), ("i", &[])];
        let mut atoms = Vec::new();
        for _ in 0..1500 {
            let mut a = run_atom(&mut dom, props);
            let before = run_atom(&mut dom, props);
            a.comparison_unit_atom_before = Some(std::sync::Arc::new(before));
            atoms.push(a);
        }
        // Pin the production arena at its exact fill (capacity == length). Any push
        // into it now provably reallocs, growing capacity — the leak we forbid.
        dom.shrink_arena_to_fit();
        let cap_before = dom.node_capacity();
        let settings = WmlComparerSettings::default();
        detect_format_changes_in_atom_list(&mut dom, &mut atoms, &settings);
        // Guard the isolation premise: identical before/after ⇒ nothing retagged, so
        // the only node work this exercised is rPr normalization scratch.
        assert!(
            atoms
                .iter()
                .all(|a| a.correlation_status == CorrelationStatus::Equal),
            "test setup: identical before/after must produce zero format changes"
        );
        assert_eq!(
            dom.node_capacity(),
            cap_before,
            "normalizing {} distinct rPr reallocated the production arena \
             (cap {cap_before} -> {}); rPr normalization scratch must build in a \
             dedicated arena, never the persistent one",
            atoms.len(),
            dom.node_capacity(),
        );
    }

    /// An Equal run atom whose nearest `w:r` carries a `w:rPr` with the given
    /// children — so `get_run_properties_from_atom` finds it.
    fn run_atom(dom: &mut Dom, rpr_children: &[(&str, &[(&str, &str)])]) -> ComparisonUnitAtom {
        let r = dom.new_element(W::r());
        let rpr = rpr_with(dom, rpr_children);
        dom.add(r, rpr);
        let t = dom.new_element(W::t());
        dom.set_value(t, "x");
        dom.add(r, t);
        let mut a = ComparisonUnitAtom::new(t, vec![r], "h".to_string());
        a.correlation_status = CorrelationStatus::Equal;
        a
    }

    /// The direct guard the review asked for: the cached run-property comparison
    /// must produce the EXACT same format-change retagging (status + changed
    /// property names) as the uncached `are_run_properties_equal` path, on the
    /// same atoms — covering adds, removes, value changes, reorders, and no-ops.
    #[test]
    fn cached_format_changes_match_uncached() {
        type Props<'a> = &'a [(&'a str, &'a [(&'a str, &'a str)])];
        let mut dom = Dom::new();
        // (before rPr, after rPr) per Equal atom.
        let specs: &[(Props, Props)] = &[
            (&[("b", &[])], &[("b", &[]), ("i", &[])]), // add italic
            (&[("b", &[])], &[("b", &[])]),             // identical
            (&[("sz", &[("val", "20")])], &[("sz", &[("val", "24")])]), // size change
            (&[], &[]),                                 // both empty
            (&[("b", &[]), ("i", &[])], &[("i", &[]), ("b", &[])]), // reordered = same
            (&[("color", &[("val", "FF0000")])], &[]),  // remove color
        ];
        let mut atoms = Vec::new();
        for (before_props, after_props) in specs {
            let mut a = run_atom(&mut dom, after_props);
            let before = run_atom(&mut dom, before_props);
            a.comparison_unit_atom_before = Some(std::sync::Arc::new(before));
            atoms.push(a);
        }
        let settings = WmlComparerSettings::default();

        let mut a_ref = atoms.clone();
        detect_format_changes_reference(&mut dom, &mut a_ref, &settings);
        let mut a_cached = atoms.clone();
        detect_format_changes_in_atom_list(&mut dom, &mut a_cached, &settings);

        let sig = |v: &[ComparisonUnitAtom]| -> Vec<(CorrelationStatus, Option<Vec<String>>)> {
            v.iter()
                .map(|a| {
                    (
                        a.correlation_status,
                        a.format_change
                            .as_ref()
                            .map(|f| f.changed_properties.clone()),
                    )
                })
                .collect()
        };
        assert_eq!(
            sig(&a_ref),
            sig(&a_cached),
            "cached format-change retagging must match the uncached oracle"
        );
        // The workload must actually exercise both a change and a non-change.
        assert!(
            a_ref
                .iter()
                .any(|a| a.correlation_status == CorrelationStatus::FormatChanged),
            "expected at least one FormatChanged"
        );
        assert!(
            a_ref
                .iter()
                .any(|a| a.correlation_status == CorrelationStatus::Equal),
            "expected at least one unchanged Equal"
        );
    }
}
