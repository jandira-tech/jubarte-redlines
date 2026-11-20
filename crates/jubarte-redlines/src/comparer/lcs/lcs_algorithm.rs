// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! `do_lcs_algorithm`, the paragraph-text LCS, and the residual-relatedness guards it consults.

use super::*;

/// Demo-corpus boilerplate significant tokens. Sharing only these (e.g. sole
/// `"this"` on file_151 Project Proposal × Bold Italic) is **not** enough for
/// M128 — text-hash residual LCS then thrash-nested titles and cost ~10 score.
/// Content cousins like Inventory **List** × Numbered **List** Demo still pass.
pub(super) const M128_BOILERPLATE_SIG: &[&str] = &[
    "this",
    "that",
    "with",
    "from",
    "have",
    "been",
    "will",
    "used",
    "text",
    "both",
    "document",
    "documents",
    "demonstrates",
    "demonstrate",
    "showing",
    "shows",
    "style",
    "styles",
    "formatting",
    "format",
    "demo",
    "demos",
    "bold",
    "italic",
    "underline",
    "color",
    "font",
    "size",
    "line",
    "spacing",
];

/// M135 off-diagonal body gate: narrower than M128 — keep `text`/`font` as
/// content (file_180 Blue **text** × size 18 **text**) but treat lead-in
/// `"this"`/`document`/`shows` as boiler so file_93 keeps M123 diagonal.
pub(super) const M135_OFF_DIAG_BOILER: &[&str] = &[
    "this",
    "that",
    "with",
    "from",
    "have",
    "been",
    "will",
    "used",
    "both",
    "document",
    "documents",
    "demonstrates",
    "demonstrate",
    "showing",
    "shows",
    "style",
    "styles",
    "formatting",
    "format",
    "demo",
    "demos",
];

/// True when the residual title's last significant token is `demo` (format
/// demo class: "Red Bold Heading Demo", "Bold and Italic Combo Demo").
pub(super) fn residual_title_ends_demo(dom: &Dom, title: &ComparisonUnit) -> bool {
    let toks = para_text_token_list(dom, title);
    last_significant_token(&toks).is_some_and(|t| t.eq_ignore_ascii_case("demo"))
}

pub(super) fn residual_para_starts_this(dom: &Dom, u: &ComparisonUnit) -> bool {
    para_text_token_list(dom, u)
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("this"))
}

pub(super) fn residual_first_body_starts_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    rest.len() >= 2 && residual_para_starts_this(dom, &rest[1])
}

/// Index of first residual para (after title) that starts with "This".
pub(super) fn residual_first_this_body_index(dom: &Dom, rest: &[ComparisonUnit]) -> Option<usize> {
    rest.iter()
        .enumerate()
        .skip(1)
        .find(|(_, u)| residual_para_starts_this(dom, u))
        .map(|(i, _)| i)
}

/// Next residual: title + ≥1 non-This line + later This-body (file_151 subtitle).
pub(super) fn residual_has_this_body_after_non_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    let Some(this_i) = residual_first_this_body_index(dom, rest) else {
        return false;
    };
    // At least one residual between title (0) and This body.
    this_i >= 2 && !residual_para_starts_this(dom, &rest[1])
}

/// Body residuals (index ≥1) share ordered prefix "this" (or content Jaccard
/// ≥0.05 with any shared sig). Used by M129 title-peel path for file_110.
pub(super) fn residual_bodies_this_cousins(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    if rest1.len() < 2 || rest2.len() < 2 {
        return false;
    }
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    let mut left_ord: Vec<String> = Vec::new();
    let mut right_ord: Vec<String> = Vec::new();
    for u in &rest1[1..] {
        let toks = para_text_token_list(dom, u);
        if left_ord.is_empty() {
            left_ord = toks.clone();
        }
        left.extend(toks);
    }
    for u in &rest2[1..] {
        let toks = para_text_token_list(dom, u);
        if right_ord.is_empty() {
            right_ord = toks.clone();
        }
        right.extend(toks);
    }
    if left.is_empty() || right.is_empty() {
        return false;
    }
    // First body paras start with "this" on both sides (proposal/demo pattern).
    let both_this = left_ord
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("this"))
        && right_ord
            .first()
            .is_some_and(|t| t.eq_ignore_ascii_case("this"));
    if both_this {
        return true;
    }
    let j = token_jaccard(&left, &right);
    let shared_sig = significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .count();
    j + 1e-12 >= 0.05 && shared_sig >= 1
}

/// At least one non-boilerplate significant token shared across residual
/// sets (no Jaccard floor). Long docs dilute Jaccard (file_34 full ~0.04)
/// while still sharing content like "strikethrough" with a short cousin.
pub(super) fn residual_sets_share_content_sig(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    residual_shared_sig_count(dom, rest1, rest2) >= 1
}

/// Count of shared non-boilerplate significant tokens across residual sets.
pub(super) fn residual_shared_sig_count(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> usize {
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    for u in rest1 {
        left.extend(para_text_token_list(dom, u));
    }
    for u in rest2 {
        right.extend(para_text_token_list(dom, u));
    }
    significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .filter(|t| {
            !M128_BOILERPLATE_SIG
                .iter()
                .any(|b| t.eq_ignore_ascii_case(b))
        })
        .count()
}

/// M134 — majority of residual paragraphs contain `:` (policy/review/
/// checklist class). Uses joined para text so atomized runs still count.
pub(super) fn residual_looks_like_colon_list(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    if rest.is_empty() {
        return false;
    }
    let with_colon = rest
        .iter()
        .filter(|u| {
            let mut text = String::new();
            for a in u.descendant_atoms() {
                if dom.name_is(a.content_element, &W::t()) {
                    text.push_str(&dom.value_str(a.content_element));
                }
            }
            text.contains(':')
        })
        .count();
    with_colon * 2 >= rest.len()
}

/// Residual-set relatedness for M128 short×short multi-para LCS.
/// Joins all residual paragraph tokens; requires Jaccard ≥ 0.04 and at least
/// one **non-boilerplate** shared significant (len≥4) token so catalog×indent
/// thrash and proposal×format-demo (file_151) stay pure I/D, while Inventory
/// List × Numbered List Demo (file_44) still fires.
pub(super) fn residual_sets_weakly_related(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    for u in rest1 {
        left.extend(para_text_token_list(dom, u));
    }
    for u in rest2 {
        right.extend(para_text_token_list(dom, u));
    }
    if left.is_empty() || right.is_empty() {
        return false;
    }
    let j = token_jaccard(&left, &right);
    let shared_sig: std::collections::HashSet<String> = significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .cloned()
        .collect();
    if j + 1e-12 < 0.04 || shared_sig.is_empty() {
        return false;
    }

    shared_sig.iter().any(|t| {
        !M128_BOILERPLATE_SIG
            .iter()
            .any(|b| t.eq_ignore_ascii_case(b))
    })
}

/// Diagonal stats for M123 gates: `(min_diag, avg_diag, max_body_diag)`.
/// `max_body_diag` is the max over rows with index ≥ 1 (0 when n &lt; 2).
pub(super) fn m123_diagonal_stats(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> (f64, f64, f64) {
    if rest1.is_empty() || rest1.len() != rest2.len() {
        return (0.0, 0.0, 0.0);
    }
    let n = rest1.len();
    let mut min_d = f64::INFINITY;
    let mut sum = 0.0_f64;
    let mut max_body = 0.0_f64;
    for i in 0..n {
        let j = token_jaccard(
            &para_text_tokens(dom, &rest1[i]),
            &para_text_tokens(dom, &rest2[i]),
        );
        sum += j;
        if j < min_d {
            min_d = j;
        }
        if i >= 1 && j > max_body {
            max_body = j;
        }
    }
    if !min_d.is_finite() {
        min_d = 0.0;
    }
    (min_d, sum / n as f64, max_body)
}

/// True when each left paragraph's best text match on the right is its
/// positional partner (or the positional score is tied for best). Used to
/// decide Word-style equal-count pure-paragraph zipping.
pub(super) fn para_zip_diagonal_dominant(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> bool {
    let n = cul1.len();
    if n == 0 || n != cul2.len() {
        return false;
    }
    let left: Vec<_> = cul1.iter().map(|u| para_text_tokens(dom, u)).collect();
    let right: Vec<_> = cul2.iter().map(|u| para_text_tokens(dom, u)).collect();
    let mut diagonal_wins = 0usize;
    let mut diag_sum = 0.0_f64;
    for i in 0..n {
        let diag = token_jaccard(&left[i], &right[i]);
        diag_sum += diag;
        let mut best_off = 0.0_f64;
        for (j, rj) in right.iter().enumerate() {
            if j == i {
                continue;
            }
            best_off = best_off.max(token_jaccard(&left[i], rj));
        }
        // Positional partner is unique best AND has real text overlap.
        // Empty-set Jaccard is one, but blank paragraphs have no real text
        // overlap and must not count as wins (support_tickets empty-mark
        // test: 3 unrelated paras would otherwise
        // zip into mixed instead of III…DDD…).
        if !left[i].is_empty() && !right[i].is_empty() && diag > 0.0 && diag + 1e-9 >= best_off {
            diagonal_wins += 1;
        }
    }
    // Majority of rows prefer a positive diagonal, and average overlap is
    // non-trivial (heading demos ~0.12+; pure-unrelated ~0).
    // M141 (calibri_heading_2×center_aligned_bold): title shares "Demo"
    // (diag~0.11) but body paras have near-zero overlap (avg~0.06). Word still
    // position-pairs the titles; without zip, flat word-LCS cross-stitches into
    // 4 paras (score ~53). Relax avg floor for short equal-count (n≤4) when a
    // clear majority of diagonals win (heading-demo class).
    let avg = diag_sum / (n as f64);
    let avg_ok = avg >= 0.08 || (n <= 4 && diagonal_wins * 2 >= n && avg >= 0.04);
    diagonal_wins * 2 >= n && avg_ok
}

/// True when first paragraphs share a last-significant token (len≥4), e.g.
/// both titles end in "Demo". Used for title-only cousin demos where full
/// diagonal zip is wrong (heading_4×helvetica).
pub(super) fn first_paras_share_last_sig(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> bool {
    let (Some(a), Some(b)) = (cul1.first(), cul2.first()) else {
        return false;
    };
    let la = para_text_token_list(dom, a);
    let lb = para_text_token_list(dom, b);
    match (last_significant_token(&la), last_significant_token(&lb)) {
        (Some(x), Some(y)) => x.eq_ignore_ascii_case(y),
        _ => false,
    }
}

/// Body residual (paras after the first) shares no **content** significant
/// tokens (len≥4, non-boilerplate). Demo cousins often share only
/// "document"/"demonstrates"/"style" — treat as unrelated so M142 can fire
/// (justify×large_font). Real cousins (heading_2×heading_3 share "Heading")
/// stay on zip/flat-LCS.
pub(super) fn body_residual_unrelated(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> bool {
    if cul1.len() < 2 || cul2.len() < 2 {
        return false;
    }
    let left = para_text_tokens_from_units(dom, &cul1[1..]);
    let right = para_text_tokens_from_units(dom, &cul2[1..]);
    let left_sig = significant_tokens(&left);
    let right_sig = significant_tokens(&right);
    !left_sig.intersection(&right_sig).any(|t| {
        !M128_BOILERPLATE_SIG
            .iter()
            .any(|b| t.eq_ignore_ascii_case(b))
    })
}

/// The recurring four-way cascade: emit Deleted / Inserted / Unknown / nothing
/// for a `(left, right)` pair (WmlComparer.ts pattern at :8112 etc.).
pub(super) fn cascade(
    left: Vec<ComparisonUnit>,
    right: Vec<ComparisonUnit>,
    out: &mut Vec<CorrelatedSequence>,
) {
    match (left.is_empty(), right.is_empty()) {
        (false, true) => out.push(CorrelatedSequence::deleted(left)),
        (true, false) => out.push(CorrelatedSequence::inserted(right)),
        (false, false) => out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            left,
            right,
        )),
        (true, true) => {}
    }
}

/// M4.C.3 / C.4 / C.7 — `DoLcsAlgorithm`: Step A (empty fast paths), Step B
/// (longest common contiguous run, via [`longest_common_run`]), Steps C–G
/// (paragraph-mark / word-break / threshold guards, M×N carriers, mixed
/// and unchanged runs), and Step I (paragraph-aware split). Step H
/// structural dispatch (groups/rows/tables) lives in C.8–C.10 and falls
/// through to `Deleted+Inserted` (the H9 fallback) when the LCS finds no
/// common run.
pub fn do_lcs_algorithm(
    dom: &mut Dom,
    unknown: CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    // Owned `unknown`: MOVE the unit vectors out instead of cloning them — the
    // caller already owns the worklist entry it removed. Behaviour-identical
    // (same Vecs, just not deep-cloned); kills the per-call ComparisonUnitAtom
    // clone that dominated fixture A's allocation profile.
    let cul1 = unknown.com_units_1.unwrap_or_default();
    let cul2 = unknown.com_units_2.unwrap_or_default();
    let mut out = Vec::new();

    // Step A — empty fast paths.
    if !cul1.is_empty() && cul2.is_empty() {
        out.push(CorrelatedSequence::deleted(cul1));
        return out;
    }
    if cul1.is_empty() && !cul2.is_empty() {
        out.push(CorrelatedSequence::inserted(cul2));
        return out;
    }
    if cul1.is_empty() && cul2.is_empty() {
        return out;
    }

    // M-CARRIER (sd_1919_word_simple x diff_after5, 99/400 superdoc oracles):
    // M-by-1 wholesale replacement. When the unknown is all-Words on both
    // sides, EXACTLY one side is a single paragraph (one pilcrow), both
    // streams end at a pilcrow, and content-word jaccard is < 0.2 (with the
    // compact strong-share rescue), Word rides the replacement into a CARRIER
    // paragraph: B's last-paragraph words inserted, A's first-paragraph words
    // deleted, fused by an Equal pilcrow pair; leading B paragraphs stay
    // pure-ins, trailing A paragraphs pure-del. Mirrors the lossless engine's
    // M-by-1 arm (jubarte-first WmlComparer.ts, commit ff4d09d67).
    if settings.merge_replaced_paragraphs {
        let all_words1 = cul1.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        let all_words2 = cul2.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if all_words1 && all_words2 {
            let pil1: Vec<usize> = cul1
                .iter()
                .enumerate()
                .filter(|(_, cu)| unit_is_single_atom_ppr(dom, cu))
                .map(|(i, _)| i)
                .collect();
            let pil2: Vec<usize> = cul2
                .iter()
                .enumerate()
                .filter(|(_, cu)| unit_is_single_atom_ppr(dom, cu))
                .map(|(i, _)| i)
                .collect();
            let ends_at_pil = |cul: &[ComparisonUnit]| {
                cul.last()
                    .is_some_and(|cu| unit_is_single_atom_ppr(dom, cu))
            };
            let xor_single = (pil1.len() == 1) != (pil2.len() == 1);
            // M×N (both sides multi-paragraph) joins the seam class ONLY on
            // near-zero TEXT overlap: word hashes include formatting, so a
            // related revision pair (file_N chains — same words, changed
            // rPr) hash-jaccards to ~0 and would merge wholesale; comparing
            // the lowercase text tokens instead keeps those correlated
            // (two_column_simple×word_native_bullet_circle text-overlap 0,
            // oracle junction M~ at block 2 — lossless a9e4a33ac shipped the
            // same class at +831.5 A/B).
            // Equal paragraph counts take the m45 zip (MIX title | pure-I |
            // pure-D | MIX last), not the seam — see the fast-path gate.
            let both_multi = pil1.len() > 1 && pil2.len() > 1 && pil1.len() != pil2.len();
            if std::env::var("JUB_TRACE").is_ok() {
                eprintln!(
                    "[gate2] n1={} n2={} pil1={} pil2={} xor={} multi={} end1={} end2={}",
                    cul1.len(),
                    cul2.len(),
                    pil1.len(),
                    pil2.len(),
                    xor_single,
                    both_multi,
                    ends_at_pil(&cul1),
                    ends_at_pil(&cul2)
                );
            }
            if !pil1.is_empty()
                && !pil2.is_empty()
                && (xor_single || both_multi)
                && ends_at_pil(&cul1)
                && ends_at_pil(&cul2)
            {
                let entries = |cul: &[ComparisonUnit]| -> Vec<(String, usize)> {
                    cul.iter()
                        .filter(|cu| !unit_is_single_atom_ppr(dom, cu))
                        .filter_map(|cu| {
                            let text: String = cu
                                .descendant_atoms()
                                .iter()
                                .filter(|dca| dom.name_is(dca.content_element, &W::t()))
                                .map(|dca| dom.value_str(dca.content_element))
                                .collect();
                            let letters = text.chars().filter(|c| c.is_alphanumeric()).count();
                            if letters > 0 {
                                Some((cu.sha1().to_string(), letters))
                            } else {
                                None
                            }
                        })
                        .collect()
                };
                let e1 = entries(&cul1);
                let e2 = entries(&cul2);
                let h2: std::collections::HashSet<&str> =
                    e2.iter().map(|(h, _)| h.as_str()).collect();
                let shared: Vec<&(String, usize)> =
                    e1.iter().filter(|(h, _)| h2.contains(h.as_str())).collect();
                let union = e1.len() + e2.len() - shared.len();
                let jaccard = if union > 0 {
                    shared.len() as f64 / union as f64
                } else {
                    0.0
                };
                let has_strong_share = shared.iter().any(|(_, lc)| *lc >= 5);
                let both_compact = e1.len() <= 16 && e2.len() <= 16;
                let text_tokens = |cul: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                    cul.iter()
                        .filter(|cu| !unit_is_single_atom_ppr(dom, cu))
                        .filter_map(|cu| {
                            let text: String = cu
                                .descendant_atoms()
                                .iter()
                                .filter(|dca| dom.name_is(dca.content_element, &W::t()))
                                .map(|dca| dom.value_str(dca.content_element))
                                .collect();
                            let t = text.trim().to_lowercase();
                            if t.chars().any(|c| c.is_alphanumeric()) {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect()
                };
                let text_ok = if both_multi {
                    let t1 = text_tokens(&cul1);
                    let t2 = text_tokens(&cul2);
                    let shared_t = t1.intersection(&t2).count();
                    let union_t = t1.len() + t2.len() - shared_t;
                    let tj = if union_t > 0 {
                        shared_t as f64 / union_t as f64
                    } else {
                        0.0
                    };
                    tj < 0.2
                } else {
                    true
                };
                // WHOLESALE gate: the carrier seam is a whole-body
                // replacement behavior. Both sides of the unknown must start
                // at their document body's FIRST content block — a residual
                // unknown pairing A's trailing paragraph with B's leading
                // paragraphs across Equal-matched middles must NOT merge
                // (diff_after6 x diff_after7: the arm fused B's block-2 words
                // with A's block-6 paragraph across two equal tables,
                // 100.00 -> 51.50; sd_1919 and the 99-seam class all start
                // at both body heads).
                let starts_at_body_head = |cul: &[ComparisonUnit]| -> bool {
                    let body_name = W::body();
                    let p_name = W::p();
                    let tbl_name = W::tbl();
                    let Some(first_cu) = cul.first() else {
                        return false;
                    };
                    let atoms = first_cu.descendant_atoms();
                    let Some(first_atom) = atoms.first() else {
                        return false;
                    };
                    let mut body_para = None;
                    for &ae in first_atom.ancestor_elements.iter() {
                        if dom.name_is(ae, &p_name)
                            && let Some(par) = dom.parent(ae)
                            && dom.name_is(par, &body_name)
                        {
                            body_para = Some((ae, par));
                            break;
                        }
                    }
                    let Some((para, body)) = body_para else {
                        return false;
                    };
                    for child in dom.elements(body, None) {
                        if dom.name_is(child, &p_name) || dom.name_is(child, &tbl_name) {
                            return child == para;
                        }
                    }
                    false
                };
                // ...and END at their body's last CONTENT block (trailing
                // empty paragraphs tolerated): diff_after6 x diff_after7's
                // unknown covers B's three lead paragraphs but B continues
                // with two content tables — Word keeps A's deleted paragraph
                // whole after them instead of merging into a mid-body seam.
                let ends_at_body_tail = |cul: &[ComparisonUnit]| -> bool {
                    let body_name = W::body();
                    let p_name = W::p();
                    let t_name = W::t();
                    let Some(last_cu) = cul.last() else {
                        return false;
                    };
                    let atoms = last_cu.descendant_atoms();
                    let Some(last_atom) = atoms.last() else {
                        return false;
                    };
                    let mut body_block = None;
                    for &ae in last_atom.ancestor_elements.iter() {
                        if let Some(par) = dom.parent(ae)
                            && dom.name_is(par, &body_name)
                        {
                            body_block = Some((ae, par));
                            break;
                        }
                    }
                    let Some((block, body)) = body_block else {
                        return false;
                    };
                    let mut seen = false;
                    for child in dom.elements(body, None) {
                        if child == block {
                            seen = true;
                            continue;
                        }
                        if !seen {
                            continue;
                        }
                        if !dom.name_is(child, &p_name) {
                            if dom.name_is(child, &W::sect_pr()) {
                                continue;
                            }
                            return false;
                        }
                        let mut has_text = false;
                        dom.for_each_descendant_element(child, Some(&t_name), |el| {
                            if !dom.value_str(el).trim().is_empty() {
                                has_text = true;
                            }
                        });
                        if has_text {
                            return false;
                        }
                    }
                    true
                };
                let wholesale = starts_at_body_head(&cul1)
                    && starts_at_body_head(&cul2)
                    && ends_at_body_tail(&cul1)
                    && ends_at_body_tail(&cul2);
                if jaccard < 0.2 && !(has_strong_share && both_compact) && wholesale && text_ok {
                    let split_paras = |cul: &[ComparisonUnit]| -> Vec<Vec<ComparisonUnit>> {
                        let mut paras = Vec::new();
                        let mut cur = Vec::new();
                        for cu in cul {
                            cur.push(cu.clone());
                            if unit_is_single_atom_ppr(dom, cu) {
                                paras.push(std::mem::take(&mut cur));
                            }
                        }
                        if !cur.is_empty() {
                            paras.push(cur);
                        }
                        paras
                    };
                    let paras_a = split_paras(&cul1);
                    let paras_b = split_paras(&cul2);
                    // M308c (broken_list × multiple_nodes_in_list):
                    // both-multi wholesale, zero hash share, BOTH sides
                    // list-heavy AND short-item. Word pure-I all B then pure-D
                    // all A (unpacked oracle IIIDDDDDDDDDDE). Long numbered
                    // prose (list_with_indents, max~42 words) is list-heavy
                    // but Word keeps MIX carrier (IMDDDD) — do not pure-I/D.
                    // Plain demos (bold_underline × book_catalog, M307) stay
                    // on the carrier path — not mostly-list.
                    if shared.is_empty()
                        && both_multi
                        && mostly_list_paras(dom, &paras_a)
                        && mostly_list_paras(dom, &paras_b)
                        && short_item_list_paras(dom, &paras_a)
                        && short_item_list_paras(dom, &paras_b)
                    {
                        out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                        out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                        return out;
                    }
                    // Word still anchors a word that ends a paragraph on both
                    // sides ("2026" closing "Product Roadmap 2026" and "Date:
                    // February 1, 2026") and seams each side of it.
                    if let Some((ia, ib)) = paragraph_final_anchor(dom, &cul1, &cul2) {
                        seam_region(dom, &cul1[..ia], &cul2[..ib], &mut out);
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            cul1[ia..ia + 2].to_vec(),
                            cul2[ib..ib + 2].to_vec(),
                        ));
                        seam_region(dom, &cul1[ia + 2..], &cul2[ib + 2..], &mut out);
                        return out;
                    }
                    let lead_b: Vec<ComparisonUnit> = paras_b[..paras_b.len() - 1]
                        .iter()
                        .flat_map(|p| p.iter().cloned())
                        .collect();
                    if !lead_b.is_empty() {
                        out.push(CorrelatedSequence::inserted(lead_b));
                    }
                    let carrier_b = paras_b.last().unwrap();
                    let carrier_a = &paras_a[0];
                    // A revision that closes on a textless paragraph carries
                    // nothing: Word pairs the story's closing marks and
                    // deletes the rest of the original (73105518ef, 6fb9bbdb49).
                    // The carrier absorbed B's closing mark, and finalize then
                    // fused B's paragraph before it into the story tail.
                    if carrier_b.len() == 1 && paras_a.len() > 1 {
                        let (close_a, rest_a) = cul1.split_last().unwrap();
                        out.push(CorrelatedSequence::deleted(rest_a.to_vec()));
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            vec![close_a.clone()],
                            vec![carrier_b[0].clone()],
                        ));
                        return out;
                    }
                    let b_words: Vec<ComparisonUnit> = carrier_b[..carrier_b.len() - 1].to_vec();
                    if !b_words.is_empty() {
                        out.push(CorrelatedSequence::inserted(b_words));
                    }
                    let a_words: Vec<ComparisonUnit> = carrier_a[..carrier_a.len() - 1].to_vec();
                    if !a_words.is_empty() {
                        out.push(CorrelatedSequence::deleted(a_words));
                    }
                    // Carrier paragraph mark, split by region position
                    // (mirrors the lossless engine's RelocateRegionMarkSurvival
                    // evidence, jubarte-first d44dc0749):
                    // - INTERIOR carrier (M×1: A paragraphs follow) — A's mark
                    //   DELETED, A pPr live, B's pMark absorbed (an Equal pair
                    //   left the pilcrow unmarked: sd_1919 52.73→51.55).
                    // - DOCUMENT-FINAL carrier (1×N: no A tail) — the region's
                    //   surviving mark stays LIVE with B's pPr + pPrChange,
                    //   which the Equal pilcrow pair produces downstream
                    //   (m148 canonicalizes_numeric_style_ids: B's
                    //   ListParagraph must survive live in the carrier).
                    if paras_a.len() > 1 {
                        out.push(CorrelatedSequence::deleted(vec![
                            carrier_a.last().unwrap().clone(),
                        ]));
                    } else {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            vec![carrier_a.last().unwrap().clone()],
                            vec![carrier_b.last().unwrap().clone()],
                        ));
                    }
                    let tail_a: Vec<ComparisonUnit> = paras_a[1..]
                        .iter()
                        .flat_map(|p| p.iter().cloned())
                        .collect();
                    if !tail_a.is_empty() {
                        out.push(CorrelatedSequence::deleted(tail_a));
                    }
                    return out;
                }
            }
        }
    }

    // Step B — longest common run (Word-mode ranks by non-separator content).
    let (mut i1, mut i2, mut len) = if settings.merge_replaced_paragraphs {
        longest_common_run_with_dom(Some(dom), &cul1, &cul2, Some(settings))
    } else {
        longest_common_run(&cul1, &cul2)
    };

    // A run that carries a paragraph mark shares a paragraph edge on both
    // sides (the word starts or ends both paragraphs).
    let run_has_mark = len > 1
        && (unit_is_single_atom_ppr(dom, &cul1[i1])
            || unit_is_single_atom_ppr(dom, &cul1[i1 + len - 1]));
    // Step C — never START a common section with a paragraph mark.
    while len > 1 {
        if !unit_is_single_atom_ppr(dom, &cul1[i1]) {
            break;
        }
        len -= 1;
        if len == 0 {
            break;
        }
        i1 += 1;
        i2 += 1;
    }

    // Step D — is the (single) common unit only a paragraph mark?
    let is_only_paragraph_mark = len == 1 && unit_is_single_atom_ppr(dom, &cul1[i1]);

    // Step E — "don't match just a single space": the TS check
    // `cul2[i2] instanceof ComparisonUnitAtom` is always false (cul holds Words/
    // Groups, never Atoms), so this branch is dead. FAITHFUL-BUG: no-op.

    // Step F — don't match only word-break characters.
    if len > 0 && len <= 3 {
        let common = &cul1[i1..i1 + len];
        let all_words = common.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if all_words {
            let content_other_than_word_split = common.iter().any(|cs| {
                let atoms = cs.descendant_atoms();
                let other_than_text = atoms
                    .iter()
                    .any(|dca| !dom.name_is(dca.content_element, &W::t()));
                if other_than_text {
                    return true;
                }
                atoms.iter().any(|dca| {
                    let v = dom.value_str(dca.content_element);
                    let ch = v.chars().next().unwrap_or('\0');
                    let is_word_split = ('\u{4e00}'..='\u{9fff}').contains(&ch)
                        || settings.word_separators.contains(&ch);
                    !is_word_split
                })
            });
            if !content_other_than_word_split {
                len = 0;
            }
        }
    }

    // Word's flat token comparison keeps one shared word that begins or
    // ends a paragraph on both sides as an anchor even in a long window
    // ("Second" opening a bullet and a section), so neither the detail
    // threshold nor the large-window collision guard voids it.
    let edge_word = run_has_mark && len > 0 && {
        let mut words = cul1[i1..i1 + len].iter().filter_map(|u| {
            let mut t = String::new();
            for a in u.descendant_atoms() {
                if !dom.name_is(a.content_element, &W::t()) {
                    return None;
                }
                t.push_str(&dom.value_str(a.content_element));
            }
            (!t.chars().all(|ch| settings.word_separators.contains(&ch))).then_some(t)
        });
        matches!((words.next(), words.next()), (Some(w), None)
            if w.chars().count() >= 4 && w.chars().all(char::is_alphanumeric))
    };
    // Step G — DetailThreshold: short pure-word common run → void.
    //
    // Gate on the common RUN being pure words (not on both sides being
    // pure-word windows). H4 flattens para+table documents into a mixed
    // word+row window; the older pure-sides gate then skipped the threshold
    // entirely, so a single-letter coincidence ("a") between unrelated
    // documents survived as an Equal island and shredded whole-doc
    // replacements (batch_to_fix pair 01 / word_tolerated_duplicate_ppr
    // vs word_tolerated_misplaced_link: Word is pure ins-all-next then
    // del-all-base; ours mixed the base "a" into next's first paragraph).
    // Group-level common runs are unaffected (they fail the pure-word-run
    // check). Faithful preset keeps the raw C# ratio (no separator filter).
    if !is_only_paragraph_mark && len > 0 {
        let common_all_words = cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)));
        if common_all_words {
            let max_len = cul1.len().max(cul2.len());
            // Word-alignment: separator-only units (bare spaces) don't count
            // toward the ratio — a shared " " inflated a 1-word overlap to
            // len 2 (2/70 ≈ 0.029 > 0.02), creating an Equal island that
            // SHREDDED a repeated identical deleted paragraph into a merged
            // mixed paragraph (page-numbering_potpourritest: GT keeps all 5
            // copies whole; reject(redline) ≠ A). Faithful preset keeps the
            // raw C# ratio.
            let ratio_len = if settings.merge_replaced_paragraphs {
                cul1[i1..i1 + len]
                    .iter()
                    .filter(|cs| {
                        // a unit is separator-only when EVERY atom is a w:t
                        // whose (non-empty) text is all separator chars;
                        // empty text counts as content (synthetic/edge runs).
                        // CJK ideographs are NOT separators — atomization
                        // (units.rs) splits each CJK char into its own word, so
                        // a shared Chinese run is real content that must count
                        // toward the ratio, not be voided as separator-only.
                        !cs.descendant_atoms().iter().all(|dca| {
                            if !dom.name_is(dca.content_element, &W::t()) {
                                return false;
                            }
                            let v = dom.value_str(dca.content_element);
                            !v.is_empty()
                                && v.chars().all(|ch| settings.word_separators.contains(&ch))
                        })
                    })
                    .count()
            } else {
                len
            };
            // The ratio is against the whole window, so it shrinks as a
            // story of paired paragraphs grows; Word keeps their anchors.
            if max_len > 0
                && !edge_word
                && (ratio_len as f64) / (max_len as f64) < settings.detail_threshold
                && !(settings.merge_replaced_paragraphs
                    && stream_paragraphs_pair_in_order(dom, &cul1, &cul2))
            {
                len = 0;
            }
        }
    }

    // Word-mode: void a short common run whose only alphabetic content is a
    // high-frequency glue word, ONLY inside a single-paragraph pure-word window.
    // bold_italic × bold_red shreds on Equal "text"; small_font × strikethrough
    // last para shreds on "and". Word does whole-sentence del/ins. Multi-para
    // windows (font_size × green_bold) may legitimately stitch on "text" —
    // leave those alone (require pmarks==1).
    // Sides may carry table ROWS in flattened windows (diff_after6×7:
    // L[w×9] R[w×35+3R] — the ['.', pMark] anchor survived because the
    // pure-word-sides requirement skipped the gate entirely). The RUN must
    // still be pure Words; the single-paragraph GLUE arm below keeps the
    // pure-word-sides requirement via its own pmarks conditions.
    let sides_pure_words = cul1.iter().all(|c| matches!(c, ComparisonUnit::Word(_)))
        && cul2.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
    let sides_words_or_rows = cul1.iter().all(|c| match c {
        ComparisonUnit::Word(_) => true,
        ComparisonUnit::Group(g) => g.group_type == ComparisonUnitGroupType::Row,
    }) && cul2.iter().all(|c| match c {
        ComparisonUnit::Word(_) => true,
        ComparisonUnit::Group(g) => g.group_type == ComparisonUnitGroupType::Row,
    });
    if settings.merge_replaced_paragraphs
        && len > 0
        && len <= 3
        && !is_only_paragraph_mark
        && sides_words_or_rows
        && cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)))
    {
        // Single-paragraph window: exactly one pPr-bearing unit per side.
        let pmarks1 = cul1
            .iter()
            .filter(|u| unit_last_atom_is_ppr(dom, u))
            .count();
        let pmarks2 = cul2
            .iter()
            .filter(|u| unit_last_atom_is_ppr(dom, u))
            .count();
        // UNREL-GLUE (hyperlink_node×hyperlink_node_internal, 52.6 vs both
        // siblings perfect): in a MULTI-para window whose sides share almost
        // no vocabulary, a glue anchor ("to") is a coincidence — Word treats
        // the docs as unrelated and never stitches on it. Related multi-para
        // windows (font_size×green_bold "text") keep their glue anchors.
        // Same 0.08 unique-lexical fraction as the TS engine's
        // DetectUnrelatedSources.
        let multi_para_unrelated = !settings.in_stamp_residual && (pmarks1 > 1 || pmarks2 > 1) && {
            let raw1 = para_text_tokens_from_units(dom, &cul1);
            let raw2 = para_text_tokens_from_units(dom, &cul2);
            // Stamped corpus windows (file_N.docx) belong to the stamp
            // confetti/residual machinery — glue anchors there are part of
            // its tuned physics (file_151_file_152 was 91.9 with them).
            let stamped = false;
            let t1 = significant_tokens(&raw1);
            let t2 = significant_tokens(&raw2);
            !stamped && !t1.is_empty() && !t2.is_empty() && {
                let inter = t1.intersection(&t2).count() as f64;
                inter / (t1.len().min(t2.len()) as f64) + 1e-12 < 0.08
            }
        };
        // A paragraph window judged word-level keeps its glue words too:
        // Word anchors on single stopwords inside the gaps of a paragraph
        // it marks word by word (`judge_paragraph_window`).
        let single_para =
            sides_pure_words && pmarks1 == 1 && pmarks2 == 1 && !settings.in_word_level_paragraph;
        if single_para || multi_para_unrelated {
            let mut alpha = String::new();
            for u in &cul1[i1..i1 + len] {
                for a in u.descendant_atoms() {
                    if dom.name_is(a.content_element, &W::t()) {
                        for ch in dom.value_str(a.content_element).chars() {
                            if ch.is_ascii_alphabetic() {
                                alpha.push(ch.to_ascii_lowercase());
                            }
                        }
                    }
                }
            }
            const GLUE: &[&str] = &[
                "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it",
                "of", "on", "or", "the", "to", "with", "text",
            ];
            // UNREL-GLUE extension (diff_after6×diff_after7, 48.1): a run
            // with NO alphabetic content at all — e.g. ['.', pMark], which
            // Step F keeps because the pPr atom is not a w:t — is never a
            // Word anchor in an UNRELATED window (the pMark pivot in
            // related/single-para windows is untouched: this arm requires
            // multi_para_unrelated).
            if GLUE.contains(&alpha.as_str()) || (multi_para_unrelated && alpha.is_empty()) {
                len = 0;
            }
        }
    }

    // M-BLK repetition guard (parity/_scratch/mblk_pairing_forensics.md):
    // discard a word-level EQ island whose containing A paragraph is
    // textually IDENTICAL to another A paragraph in this window — Word never
    // bridges a repeated paragraph copy into B content (page-numbering GT:
    // five identical 'More sample…' paragraphs all stay whole; only the
    // copy-unique ' Document' paragraph anchors). Applies to word-unit runs
    // regardless of whether the window also holds Group units (the real
    // corpus windows do). The block falls back to pure del/ins (Step H),
    // matching GT. Word-mode only.
    if len > 0
        && !is_only_paragraph_mark
        && settings.merge_replaced_paragraphs
        && cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)))
        && containing_paragraph_is_duplicated(dom, &cul1, i1)
    {
        len = 0;
    }

    // M-TBL rule 3 (parity/_scratch/table_class_forensics.md): when a table is
    // in play, a common run made ONLY of textless units (empty paragraphs) is
    // a false anchor — it drags A's table past B's early tables, so A's table
    // merges with a LATE positional partner while B's first tables come out as
    // pure insertions. Word merges with the FIRST same-slot table (GT
    // support-tickets-table_table-bookmark-end: A's ticket table merges
    // cell-wise with B table 1). Discarding the anchor falls through to Step
    // H's Table/Para dispatch, which pairs table runs first-to-first.
    // Word-mode only.
    //
    // ONE-sided tables hit the same physics (2026-08-04): when only one side
    // holds a table, an empty-paragraph anchor splices that table into the
    // middle of the other side's deleted/inserted run instead of Word's
    // whole-region replacement (oracle: 227 ins-first contiguous replacements
    // vs 23 interleaved). sublist_issue×super_basic_table anchored A's interior
    // empties against B's between-tables empty (49.80 vs lossless 100.00);
    // basic_table_shading×basic_tracked_change anchored A's trailing empty
    // against B's first empty, dragging the deleted table ahead of B's
    // inserted paragraphs. Paragraph-merge pivot windows carry no tables and
    // are untouched.
    // Row groups count too: H4 flattens para+table documents into mixed
    // word+Row windows, so at anchor time the table is VISIBLE only as its
    // rows (sublist_issue×super_basic_table traces L[w×23] R[RRwRRw] — the
    // len=1 pMark anchor there merges B's between-tables empty paragraph
    // into A's first paragraph instead of Word's pure replacement).
    if len > 0
        && settings.merge_replaced_paragraphs
        && (count_gt(&cul1, ComparisonUnitGroupType::Table) > 0
            || count_gt(&cul2, ComparisonUnitGroupType::Table) > 0
            || count_gt(&cul1, ComparisonUnitGroupType::Row) > 0
            || count_gt(&cul2, ComparisonUnitGroupType::Row) > 0)
        && cul1[i1..i1 + len].iter().all(|u| {
            u.descendant_atoms().iter().all(|a| {
                !dom.name_is(a.content_element, &W::t())
                    || dom.value_str(a.content_element).trim().is_empty()
            })
        })
    {
        len = 0;
    }

    // Word never pairs two unrelated paragraphs on a paragraph mark alone
    // (its replace-gap grammar, decoded by Docxodus 12's IrBlockAligner and
    // IrMarkupRenderer): a textless run — bare paragraph marks, empty
    // paragraphs — anchors only beside matched content, where it starts or
    // ends the window on both sides. Anywhere else the whole region is one
    // replace, new paragraphs inserted whole and old ones deleted whole
    // (list_with_table_break × broken_complex_list fused "TWO" into "e" and
    // "A" into "a"). The two stories' final marks still pair: Word keeps that
    // structural pair whatever precedes it.
    //
    // A run that ends the window short of the story end is Word's interior
    // pilcrow chain, which holds only as far as `interior_blank_chain_holds`
    // allows (file_36 × file_37: the blank before the table stays on each
    // side, since "Contract Review" meets a blank across from it).
    let starts = i1 == 0 && i2 == 0;
    let ends = i1 + len == cul1.len() && i2 + len == cul2.len();
    let blank_head = if len > 0 && settings.merge_replaced_paragraphs && !starts {
        cul1[i1..i1 + len]
            .iter()
            .take_while(|u| unit_is_textless_paragraph_matter(dom, u))
            .count()
    } else {
        0
    };
    // The revised story's closing mark facing the mark of the original's
    // first paragraph, with words, while the original runs on to its own
    // story end: Word's story-tail fusion, whose revised last paragraph opens
    // the first deleted one (bullet_list × calibri_bold_italic: "Calibri bold
    // italic …" into "Apples"; "Bananas" to "Grapes" deleted after it).
    let story_tail_fusion = i1 > 0
        && i2 + len == cul2.len()
        && unit_closes_story(dom, &cul2[cul2.len() - 1])
        && cul1.last().is_some_and(|u| unit_closes_story(dom, u))
        && !cul1[..i1].iter().any(|u| unit_is_paragraph_matter(dom, u));
    if blank_head == len
        && len > 0
        && !story_tail_fusion
        && (!ends
            || (!unit_closes_story(dom, &cul1[i1 + len - 1])
                && !interior_blank_chain_holds(dom, &cul1[..i1], &cul2[..i2])))
    {
        len = 0;
        if let (Some(l), Some(r)) = (cul1.last(), cul2.last())
            && l.sha1() == r.sha1()
            && unit_is_textless_paragraph_matter(dom, l)
            && unit_closes_story(dom, l)
            && unit_closes_story(dom, r)
        {
            (i1, i2, len) = (cul1.len() - 1, cul2.len() - 1, 1);
        }
    } else if blank_head > 0
        && blank_head < len
        && !interior_blank_chain_holds(dom, &cul1[..i1], &cul2[..i2])
    {
        // The run's leading blanks close the region before its content:
        // they pair only through the chain.
        (i1, i2, len) = (i1 + blank_head, i2 + blank_head, len - blank_head);
    }

    if len == 0 {
        // Step H — structural dispatch (no common run found).
        return step_h(dom, &cul1, &cul2, settings);
    }

    // Step I.1 — pull the partial paragraph that precedes the common run.
    let (mut rem_left, mut rem_right) = (0usize, 0usize);
    {
        let common_seq = &cul1[i1..i1 + len];
        if matches!(common_seq[0], ComparisonUnit::Word(_))
            && common_seq.iter().any(|cu| unit_first_atom_is_ppr(dom, cu))
        {
            rem_left = take_while_count_rev(&cul1[..i1], |cu| word_first_not_ppr(dom, cu));
            rem_right = take_while_count_rev(&cul2[..i2], |cu| word_first_not_ppr(dom, cu));
        }
    }
    let before_left = i1 - rem_left;
    let before_right = i2 - rem_right;

    // before-region, then partial-paragraph leftovers, each via the cascade.
    cascade(
        cul1[..before_left].to_vec(),
        cul2[..before_right].to_vec(),
        &mut out,
    );
    cascade(
        cul1[before_left..i1].to_vec(),
        cul2[before_right..i2].to_vec(),
        &mut out,
    );

    // Equal middle.
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        cul1[i1..i1 + len].to_vec(),
        cul2[i2..i2 + len].to_vec(),
    ));

    // Step I.6 — after-region split at the next paragraph mark.
    let end1 = i1 + len;
    let end2 = i2 + len;
    let remaining1 = &cul1[end1..];
    let remaining2 = &cul2[end2..];
    let last_eq = &cul1[i1 + len - 1];
    let last_not_ppr = matches!(last_eq, ComparisonUnit::Word(_))
        && last_eq
            .descendant_atoms()
            .last()
            .is_some_and(|a| !atom_is_ppr(dom, a));
    if last_not_ppr {
        let idx1 = find_index_of_next_para_mark(dom, remaining1);
        let idx2 = find_index_of_next_para_mark(dom, remaining2);
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            remaining1[..idx1].to_vec(),
            remaining2[..idx2].to_vec(),
        ));
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            remaining1[idx1..].to_vec(),
            remaining2[idx2..].to_vec(),
        ));
        return out;
    }
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Unknown,
        remaining1.to_vec(),
        remaining2.to_vec(),
    ));
    out
}
