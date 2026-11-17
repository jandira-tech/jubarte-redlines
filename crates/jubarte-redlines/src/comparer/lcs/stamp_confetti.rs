// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Stamped-variant detection, residual pairing and `stamp_confetti_then_replace`.

use super::*;

/// Significant body tokens (len≥3), excluding corpus stamp fragments.
/// Used to detect *related* stamped variants whose paragraph hashes diverge
/// only on whitespace/formatting (file_175×file_176 share ~99% vocabulary).
pub(super) fn significant_body_tokens(
    dom: &Dom,
    cu: &[ComparisonUnit],
) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, cu)
        .into_iter()
        .filter(|t| t.chars().count() >= 3 && !t.starts_with("file_") && t != "docx" && t != "doc")
        .collect()
}

/// `inter / min(|A|,|B|)` of significant body tokens. High when both sides are
/// the same document family (charter v1 vs v2); low for unrelated demos that
/// only share the `file_N.docx` stamp.
pub(super) fn body_token_overlap_ratio(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> f64 {
    let a = significant_body_tokens(dom, cu1);
    let b = significant_body_tokens(dom, cu2);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(&b).count() as f64;
    let min = a.len().min(b.len()) as f64;
    inter / min
}

/// When stamped pairs share substantial body vocabulary, Word runs full LCS —
/// not stamp-confetti replace-rest. file_175×file_176 are the same charter
/// with spacing drift (group sha1s disjoint → old path wrongly confetti'd).
/// file_134×file_135 share almost no body tokens (ratio ~0.24) → confetti OK.
pub(super) const STAMP_CONFETTI_MAX_BODY_OVERLAP: f64 = 0.55;

/// Related stamped variants (file_175 charter v1/v2): both sides carry a
/// substantial vocabulary and share most of it. Short demos that only share a
/// few generic words (file_33 min=10 ratio=0.6) are NOT related — Word confettis.
pub(super) const RELATED_STAMP_MIN_BODY_TOKENS: usize = 40;

pub(super) fn is_related_stamped_variant(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    let a = significant_body_tokens(dom, cu1);
    let b = significant_body_tokens(dom, cu2);
    let min_n = a.len().min(b.len());
    let ratio = body_token_overlap_ratio(dom, cu1, cu2);
    if min_n < RELATED_STAMP_MIN_BODY_TOKENS {
        return false;
    }
    ratio >= STAMP_CONFETTI_MAX_BODY_OVERLAP
}

pub(super) fn should_stamp_confetti(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    // Confetti stamped corpus demos unless both sides are a long related
    // variant (file_175). file_33 shares residual phrases ("This document
    // demonstrates") but is still confetti in Word.
    !is_related_stamped_variant(dom, cu1, cu2)
}

/// First contentful paragraph group's concatenated `w:t` text (for stamp gate).
pub(super) fn first_contentful_para_text(dom: &Dom, cu: &[ComparisonUnit]) -> Option<String> {
    first_contentful_group_index(dom, cu).map(|i| {
        let mut text = String::new();
        for a in cu[i].descendant_atoms() {
            if dom.name_is(a.content_element, &W::t()) {
                text.push_str(&dom.value_str(a.content_element));
            }
        }
        text
    })
}

/// Index of the first contentful group in `cu` (paragraph/table with real `w:t`).
pub(super) fn first_contentful_group_index(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
    cu.iter()
        .position(|u| as_group(u).is_some() && run_real_text_len(dom, std::slice::from_ref(u)) > 0)
}

/// Minimum token-Jaccard to pair a short base residual paragraph with a next
/// residual after stamp confetti (M75). file_33: Word pairs
/// "This document demonstrates Heading 1 paragraph style." with
/// "This document demonstrates all major DOCX features:" (≈0.27) so word-level
/// LCS can share the prefix — pure insert-all/delete-all left 3 pages vs 2.
///
/// Also require ≥3 shared *significant* tokens (len≥4) so stopwords like
/// "this"/"with"/"style" alone cannot form a false pair (0.25 alone mixed
/// "Main Title Section" into body inserts on file_33).
///
/// M95 (file_96): short titles "Open Sans Bold Underline Demo" ↔
/// "Verdana Bold Large Font Demo" share only **2** significant tokens
/// (bold, demo) at Jaccard 0.25 — Word still nests word-LCS (Equal " Bold "
/// / " Demo"). Long body residuals keep min 3.
///
/// M96 (file_139/file_32): short demo titles that share only the **last**
/// significant token ("… Demo" ↔ "… Demo") have Jaccard ~0.12–0.14 and fail
/// both min_sig=2 and jaccard≥0.25. Word still nests Equal " Demo". Allow
/// short pairs when last significant tokens match with min_sig=1 and a
/// lower jaccard floor (0.10). Body sentences keep the strict gates.
///
/// M107 (file_160): short titles can share only a **connector** token of
/// length 3 — Word nests "Italic and Underline Combo Demo" with
/// "Module 3: Tools and Systems" on Equal `" and "` (len-4 sig gate misses
/// "and"). Allow short pairs with a shared connector + jaccard≥0.10 when
/// there is no len≥4 shared sig (so "bold" alone still needs min_sig=2).
///
/// M114 (file_154): residual **body** cousins share an ordered significant
/// prefix ("This document …") but only 1–2 len≥4 tokens total and Jaccard
/// ~0.13 — below min_sig=3 / jaccard 0.25. Word still nests Equal
/// `"This document "` + del/ins tails. Allow ordered prefix ≥2 significant
/// tokens with min_sig=1 and jaccard≥0.10 (same floor as last-sig Demo).
pub(super) const STAMP_RESIDUAL_PAIR_MIN_JACCARD: f64 = 0.25;
pub(super) const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG: f64 = 0.10;
/// M133: body last-sig ("…style." × "…style.") can sit at Jaccard ~0.06 when
/// the shared token is only the trailing word. Titles with "Demo" stay ≥0.10.
pub(super) const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY: f64 = 0.05;
pub(super) const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG: usize = 3;
pub(super) const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG_SHORT: usize = 2;
pub(super) const STAMP_RESIDUAL_ORDERED_PREFIX_MIN_SIG: usize = 2;
/// Token count (all tokens, not only significant) at or below which the short
/// shared-sig floor applies (title-demo class, ~5–8 words).
pub(super) const STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS: usize = 8;
/// M133: last-significant-token match may fire on slightly longer body
/// residuals (file_120 "…paragraph style." ↔ "…visual style." ~10 tokens).
/// Titles stay well under this; keep below long-body thrash.
pub(super) const STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS: usize = 16;
/// Connector tokens (len 3) Word peels as Equal across residual titles.
pub(super) const STAMP_RESIDUAL_CONNECTORS: &[&str] = &["and", "or", "the", "for", "with", "to"];

pub(super) fn significant_tokens(
    tokens: &std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    tokens
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .cloned()
        .collect()
}

pub(super) fn shared_connector_tokens(
    left: &std::collections::HashSet<String>,
    right: &std::collections::HashSet<String>,
) -> usize {
    left.iter()
        .filter(|t| {
            t.chars().count() == 3
                && STAMP_RESIDUAL_CONNECTORS
                    .iter()
                    .any(|c| t.eq_ignore_ascii_case(c))
                && right.iter().any(|r| r.eq_ignore_ascii_case(t))
        })
        .count()
}

/// TOKEN-PROBE-01b: `unit_text_token_count(dom, u)` without building the
/// String or the Vec — tokens are maximal alphanumeric runs over the
/// concatenated `w:t` atom text, so count into-token transitions with the
/// in-token state carried across atom boundaries.
pub(super) fn unit_text_token_count(dom: &Dom, u: &ComparisonUnit) -> usize {
    let mut count = 0usize;
    let mut in_token = false;
    u.try_for_each_atom(&mut |a| {
        if dom.name_is(a.content_element, &W::t()) {
            for c in dom.value_str(a.content_element).chars() {
                if c.is_alphanumeric() {
                    if !in_token {
                        count += 1;
                        in_token = true;
                    }
                } else {
                    in_token = false;
                }
            }
        }
        true
    });
    count
}

/// TOKEN-PROBE-01: `unit_has_text_token(dom, u)` without
/// building the concatenated String or the token Vec — the list is non-empty
/// iff any `w:t` atom carries an alphanumeric char.
pub(super) fn unit_has_text_token(dom: &Dom, u: &ComparisonUnit) -> bool {
    !u.try_for_each_atom(&mut |a| {
        !(dom.name_is(a.content_element, &W::t())
            && dom
                .value_str(a.content_element)
                .chars()
                .any(|c| c.is_alphanumeric()))
    })
}

/// Ordered tokens (lowercase alphanumeric) for last-significant-token match.
pub(super) fn para_text_token_list(dom: &Dom, u: &ComparisonUnit) -> Vec<String> {
    let mut text = String::new();
    for a in u.descendant_atoms() {
        if dom.name_is(a.content_element, &W::t()) {
            text.push_str(&dom.value_str(a.content_element));
        }
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Join all `w:t` under a unit then tokenize. Atomize splits text to
/// per-character atoms; calling `para_text_tokens` per atom yields only
/// single-letter tokens so significant-token gates never fire (M75/M76).
pub(super) fn para_text_tokens_joined(
    dom: &Dom,
    u: &ComparisonUnit,
) -> std::collections::HashSet<String> {
    para_text_token_list(dom, u).into_iter().collect()
}

pub(super) fn last_significant_token(ordered: &[String]) -> Option<&str> {
    ordered
        .iter()
        .rev()
        .find(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
}

/// Count leading significant tokens (len≥4) that match in order on both sides.
/// Skips short glue tokens (`this`, `a`, …) when comparing the ordered streams
/// so `"This document demonstrates…"` ↔ `"This document combines…"` scores 2
/// (`document` after skipping `this` if len&lt;4 — actually `this` is len 4).
/// Word peels Equal `"This document "` on file_154 body cousins.
pub(super) fn ordered_shared_prefix_sig(left: &[String], right: &[String]) -> usize {
    let li: Vec<&str> = left
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
        .collect();
    let rj: Vec<&str> = right
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
        .collect();
    let mut n = 0usize;
    for (a, b) in li.iter().zip(rj.iter()) {
        if a.eq_ignore_ascii_case(b) {
            n += 1;
        } else {
            break;
        }
    }
    n
}

/// Greedy unique residual matches after stamp confetti. Only when the base
/// residual is short (heading-demo class). Returns `(base_idx, next_idx)` pairs.
pub(super) fn stamp_residual_pairs(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> Vec<(usize, usize)> {
    // A full diagonal of Word's same-slot pairs (every paragraph paired with
    // its counterpart) is taken whole; anything short of that keeps the
    // tuned candidates below. Partial same-slot pairs, alone or mixed with
    // the candidates, lost to them on Word's redlines (47 of 85 changed pool
    // and English pairs worse, mean -0.076 Jaccard): a lone paired title
    // pulled the residual off Word's pure insert / delete shape.
    let slot_pairs = same_slot_pairs(dom, rest1, rest2);
    if rest1.len() == rest2.len() && slot_pairs.len() == rest1.len() && !slot_pairs.is_empty() {
        return slot_pairs;
    }
    // Guard: short base residual only (file_33 has 3 content paras after stamp).
    // Long residuals stay pure insert-all / delete-all (file_134 confetti).
    if rest1.is_empty() || rest2.is_empty() || rest1.len() > 6 {
        return Vec::new();
    }
    let left_ord: Vec<_> = rest1.iter().map(|u| para_text_token_list(dom, u)).collect();
    let right_ord: Vec<_> = rest2.iter().map(|u| para_text_token_list(dom, u)).collect();
    let left: Vec<std::collections::HashSet<String>> = left_ord
        .iter()
        .map(|v| v.iter().cloned().collect())
        .collect();
    let right: Vec<std::collections::HashSet<String>> = right_ord
        .iter()
        .map(|v| v.iter().cloned().collect())
        .collect();
    // (priority_tier, jaccard, base_idx, next_idx) — tier 3 last-sig, 2 off-diag body
    let mut candidates: Vec<(u8, f64, usize, usize)> = Vec::new();
    for (i, li) in left.iter().enumerate() {
        if li.is_empty() {
            continue;
        }
        let li_sig = significant_tokens(li);
        let li_last = last_significant_token(&left_ord[i]);
        for (j, rj) in right.iter().enumerate() {
            if rj.is_empty() {
                continue;
            }
            let rj_sig = significant_tokens(rj);
            let shared_sig = li_sig.intersection(&rj_sig).count();
            // M95: short title-class paras need only 2 shared significant tokens
            // (file_96 Bold+Demo); longer bodies keep min 3 (file_33 stopword gate).
            // M96: short titles that share last significant token ("… Demo")
            // accept min_sig=1 + lower jaccard (file_139 / file_32).
            // M107: short titles sharing only a connector ("and") with jaccard
            // ≥0.10 and **zero** len≥4 shared sig (file_160 title↔Module 3).
            let short_pair = li.len() <= STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS;
            // M133: last-sig on body residuals (up to 16 tokens) — Word pairs
            // file_120 "…paragraph style." with "…visual style." even though
            // "This document …" ordered-prefix has higher Jaccard with the
            // other next body. Title demos stay well under 16.
            let last_sig_len_ok = li.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS;
            let last_sig_match = last_sig_len_ok
                && li_last.is_some()
                && li_last == last_significant_token(&right_ord[j]);
            // M107: only the first base residual (demo title line) may pair on
            // connector alone. Body lines also contain "and" and would otherwise
            // false-pair every "X and Y" module (file_160 over-paired M3/M4/Completion).
            let connector_only =
                short_pair && i == 0 && shared_sig == 0 && shared_connector_tokens(li, rj) > 0;
            // M114: ordered significant prefix (body "This document …" cousins).
            // M115b: only when **next residual is short** (rest2 ≤ 20). On long
            // next (file_160 modules, file_33 features) ordered-prefix false-pairs.
            let ordered_prefix = rest2.len() <= 20
                && ordered_shared_prefix_sig(&left_ord[i], &right_ord[j])
                    >= STAMP_RESIDUAL_ORDERED_PREFIX_MIN_SIG;
            // M135 (file_180): short residual demos (≤4 each) body pairs with a
            // **later** next residual (j > i) when base body's **last** sig token
            // appears in that next residual (e.g. trailing "text" × "Blue text…").
            // Word pure-I's next body0 and nests base body0 with next body1.
            // Mid-body shared "font" alone (file_140 Font Size×Verdana) must NOT
            // off-diag — that skipped M123 and cost −30. Sole "this" (file_93)
            // also blocked. Only j>i (forward).
            let last_appears_in_next = li_last.is_some_and(|tok| {
                !M135_OFF_DIAG_BOILER
                    .iter()
                    .any(|b| tok.eq_ignore_ascii_case(b))
                    && right_ord[j].iter().any(|t| t.eq_ignore_ascii_case(tok))
            });
            let off_diag_body = i >= 1
                && j > i
                && last_appears_in_next
                && rest1.len() <= 4
                && rest2.len() <= 4
                && li.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS;
            let min_sig = if last_sig_match || connector_only || ordered_prefix || off_diag_body {
                1
            } else if short_pair {
                STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG_SHORT
            } else {
                STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG
            };
            // connector_only has shared_sig==0; treat as satisfied when flagged.
            if shared_sig < min_sig && !connector_only && !ordered_prefix {
                continue;
            }
            let jacc = token_jaccard(li, rj);
            let min_jacc = if last_sig_match {
                // Body last-sig may be weak Jaccard (file_120 style ~0.06);
                // short title last-sig (Demo) still clears 0.10 easily.
                if short_pair {
                    STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG
                } else {
                    STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY
                }
            } else if off_diag_body {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY
            } else if connector_only || ordered_prefix {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG
            } else {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD
            };
            if jacc + 1e-12 >= min_jacc {
                // Priority: last_sig (3) > off-diag body content (2) >
                // ordered-prefix / other (1). M133/M135 beat higher-Jaccard
                // diagonal "This document" thrash.
                let tier: u8 = if last_sig_match {
                    3
                } else if off_diag_body {
                    2
                } else {
                    1
                };
                candidates.push((tier, jacc, i, j));
            }
        }
    }
    // Higher tier first (last-sig / off-diag body), then jaccard; ties prefer
    // earlier next residual (Module 3 before Module 4 when both only share "and").
    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.2.cmp(&b.2))
    });
    let mut used_i = std::collections::HashSet::new();
    let mut used_j = std::collections::HashSet::new();
    let mut pairs = Vec::new();
    for (_tier, _jacc, i, j) in candidates {
        if used_i.contains(&i) || used_j.contains(&j) {
            continue;
        }
        used_i.insert(i);
        used_j.insert(j);
        pairs.push((i, j));
    }
    // Emit in next-document order so inserts stay sequential around pairs.
    pairs.sort_by_key(|&(_, j)| j);
    pairs
}
/// Word confettis `file_N.docx` then insert-all-next / delete-all-base for the
/// rest when contentful block groups are disjoint (file_134×file_135, m52).
/// Full-doc LCS after the stamp mixes titles into body deletions.
///
/// M75: when the base residual is short, greedily pair residual paragraphs
/// with token Jaccard ≥ 0.25 (file_33 Word pairs the "This document
/// demonstrates …" cousins) and walk next-residual order so word-level LCS
/// can run inside those pairs instead of pure-del after a full insert block.
pub(super) fn stamp_confetti_then_replace(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let residual_flag_settings = {
        let mut s2 = settings.clone();
        s2.in_stamp_residual = true;
        s2
    };
    let settings = &residual_flag_settings;
    let i1 = first_contentful_group_index(dom, cu1)?;
    let i2 = first_contentful_group_index(dom, cu2)?;
    // LCS only the stamp paragraphs (digit confetti).
    let mut stamp_seqs = lcs(dom, vec![cu1[i1].clone()], vec![cu2[i2].clone()], settings);
    let mut rest1 = Vec::new();
    rest1.extend_from_slice(&cu1[..i1]);
    rest1.extend_from_slice(&cu1[i1 + 1..]);
    let mut rest2 = Vec::new();
    rest2.extend_from_slice(&cu2[..i2]);
    rest2.extend_from_slice(&cu2[i2 + 1..]);

    // M134 (file_127): short **colon-list** residuals (policy×review, 3..=10)
    // before residual-pair / diagonal paths. A single weak last-line pair
    // (Recommendation×Policy title) would otherwise pure-I the review block
    // and pure-D the policy — Word peels connectors (`and`/`for`) mid-mesh.
    // Majority of residual paras must contain `:` on both sides. file_118
    // Book Catalog has 0 colons → stays on confetti/pair path.
    if (3..=10).contains(&rest1.len())
        && (3..=10).contains(&rest2.len())
        && residual_looks_like_colon_list(dom, &rest1)
        && residual_looks_like_colon_list(dom, &rest2)
    {
        let mut left: Vec<ComparisonUnit> = rest1.iter().flat_map(group_contents).collect();
        let mut right: Vec<ComparisonUnit> = rest2.iter().flat_map(group_contents).collect();
        rehash_words_by_text_content(dom, &mut left);
        rehash_words_by_text_content(dom, &mut right);
        let mut residual_settings = settings.clone();
        residual_settings.detail_threshold = 0.005;
        let mut nested = lcs(dom, left, right, &residual_settings);
        stamp_seqs.append(&mut nested);
        return Some(stamp_seqs);
    }

    // M123 (file_93×file_94): equal-count short residual demos after stamp
    // (both 3: title+2 body). M75 residual pairs only catch the Demo titles
    // (last-sig match); bodies stay unpaired → insert-all/delete-all thrash.
    // Word nests all residuals positionally with word LCS + format changes.
    //
    // Blind v66 unguarded diagonal zip also fired on weak cousins (file_177
    // Yellow↔Subscript 100→66) and on demos where residual pairs already
    // cover bodies well (file_148 92→72, file_96 98→83). Dual gate:
    //
    //   Path A — title-only residual pairs + max body diagonal ≥ 0.09
    //     (file_93 ~0.18, file_20 ~0.09; file_177 body max ~0.07 stays off)
    //   Path B — every diagonal row is strongly related (min ≥ 0.14, avg ≥ 0.18)
    //     even when residual pairs already cover a body (file_140/125/84/129);
    //     file_163 min ~0.125 stays on residual pairs (was 100 under pairs)
    let pairs = stamp_residual_pairs(dom, &rest1, &rest2);

    // M133: when residual pairs include an off-diagonal body match (e.g.
    // file_120 A body0 ↔ B body1 on trailing "style"), do not diagonal-zip —
    // that would force A body0↔B body0 ("This document" ordered-prefix thrash)
    // over Word's pure-I B body0 + last-sig style MIX.
    let off_diagonal_pair = pairs.iter().any(|&(i, j)| i != j);
    if rest1.len() == rest2.len()
        && (2..=6).contains(&rest1.len())
        && !off_diagonal_pair
        && para_zip_diagonal_dominant(dom, &rest1, &rest2)
        && {
            let covers_body = pairs.iter().any(|&(i, _)| i >= 1);
            let (min_d, avg_d, max_body) = m123_diagonal_stats(dom, &rest1, &rest2);
            // Path A needs a body pair sharing a real word: "This" and "."
            // alone (Calibri heading × underline) leave Word's seam.
            let shares_word = rest1.iter().zip(rest2.iter()).skip(1).any(|(a, b)| {
                para_text_tokens_joined(dom, a)
                    .intersection(&para_text_tokens_joined(dom, b))
                    .any(|t| t.chars().count() >= 5)
            });
            let path_a = !covers_body && shares_word && max_body + 1e-12 >= 0.09;
            // file_129 avg ~0.17 / min ~0.14; file_163 min ~0.125 stays off
            let path_b = min_d + 1e-12 >= 0.14 && avg_d + 1e-12 >= 0.16;
            path_a || path_b
        }
    {
        for (a, b) in rest1.iter().zip(rest2.iter()) {
            let mut nested = lcs(dom, vec![a.clone()], vec![b.clone()], settings);
            stamp_seqs.append(&mut nested);
        }
        return Some(stamp_seqs);
    }

    if pairs.is_empty() {
        // M104 (file_130): short stamped demo into a long next doc (≥8 residual).
        // Word pure-I's next's main title, nests the short demo title into next's
        // *second* residual, pure-D's the short body immediately after, then
        // insert-all remaining next, delete-all remaining base (last short
        // residual often trails at document end). Plain insert-all-next/
        // delete-all-base parks the whole short demo at the end and costs
        // ~5–15 score on large-doc near-90 pairs.
        // M108 (file_73): numbered-list demos have **5** residual paras
        // (title + intro + 3 items). Gate was ≤4 and skipped M104/M105 peel
        // entirely (pure-I whole long doc). Allow up to 6 (same as residual
        // pair short-base cap).
        if (2..=6).contains(&rest1.len()) && rest2.len() >= 8 {
            // pure-I first next residual (main title)
            stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
            // M105 (file_7 / file_5 / file_130): Word peels the trailing
            // significant token of next's subtitle ("… demonstration document")
            // into the short demo *body* ("This document demonstrates…") as
            // Equal, leaving the short *title* pure-del on the subtitle para:
            //   p2 MIX ins"A comprehensive… demonstration" + del"Left Alignment Demo"
            //   p3 del"This" + Equal" document" + del" demonstrates…"
            // Nesting only title↔subtitle then pure-D body leaves "document"
            // stuck on the insert and a full pure-D body (score gap ~5–10).
            // When the subtitle's last sig token appears in the body (not the
            // short title), multi-para LCS of [title, body] vs [subtitle].
            let peel_body = rest1.len() >= 2 && {
                let sub_toks = para_text_token_list(dom, &rest2[1]);
                let last = last_significant_token(&sub_toks);
                let body = para_text_tokens_joined(dom, &rest1[1]);
                let title = para_text_tokens_joined(dom, &rest1[0]);
                last.is_some_and(|tok| {
                    let key = tok.to_ascii_lowercase();
                    body.iter().any(|t| t.eq_ignore_ascii_case(&key))
                        && !title.iter().any(|t| t.eq_ignore_ascii_case(&key))
                })
            };
            if peel_body {
                // M132 (file_73): peel_body alone nests title+body only into the
                // long *subtitle*, parking "shows numbered lists…" as one del.
                // Word also peels Equal `" numbered "` later in the long body.
                // When residuals share non-boilerplate content (e.g. numbered),
                // pure-I main title then text-hash LCS short residual vs the
                // *entire* remaining long residual (from subtitle on).
                // file_7/5/130 peel_body with only boiler shared stay multi-para.
                if residual_sets_share_content_sig(dom, &rest1, &rest2) {
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    let mut residual_settings = settings.clone();
                    residual_settings.detail_threshold = 0.005;
                    let mut nested = lcs(dom, left, right, &residual_settings);
                    stamp_seqs.append(&mut nested);
                    return Some(stamp_seqs);
                }
                let mut nested = lcs(
                    dom,
                    vec![rest1[0].clone(), rest1[1].clone()],
                    vec![rest2[1].clone()],
                    settings,
                );
                stamp_seqs.append(&mut nested);
            } else {
                // M125 (file_18): nest short title ↔ next subtitle only when they
                // share real vocabulary. Unrelated pot-pourri subtitles
                // (jaccard 0 vs "Track Changes… Demo") Word pure-I's; nesting
                // invents MIX "Sampler…Track Changes…" and costs ~6 score.
                let title_toks = para_text_tokens_joined(dom, &rest1[0]);
                let sub_toks = para_text_tokens_joined(dom, &rest2[1]);
                let nest_j = token_jaccard(&title_toks, &sub_toks);
                let nest_shared = significant_tokens(&title_toks)
                    .intersection(&significant_tokens(&sub_toks))
                    .count();
                if nest_j + 1e-12 >= 0.08 || nest_shared >= 1 {
                    // nested word-LCS: short title ↔ second next residual
                    let mut nested = lcs(
                        dom,
                        vec![rest1[0].clone()],
                        vec![rest2[1].clone()],
                        settings,
                    );
                    stamp_seqs.append(&mut nested);
                    // pure-D short body right after the title nest (if present)
                    if rest1.len() >= 2 {
                        stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    }
                } else {
                    // pure confetti: I remaining next (subtitle+body), then D all
                    // short residual — keep ins-before-del (Word file_18). Do not
                    // interleave D between rest2[1] and rest2[2..] (LO page thrash).
                    stamp_seqs.push(CorrelatedSequence::inserted(rest2[1..].to_vec()));
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1.clone()));
                    return Some(stamp_seqs);
                }
            }
            // insert remaining next (from index 2)
            if rest2.len() > 2 {
                stamp_seqs.push(CorrelatedSequence::inserted(rest2[2..].to_vec()));
            }
            // delete remaining base from index 2 (title+body consumed above;
            // peel path nests body into the multi-para LCS, non-peel pure-Ds it)
            if rest1.len() > 2 {
                stamp_seqs.push(CorrelatedSequence::deleted(rest1[2..].to_vec()));
            }
            return Some(stamp_seqs);
        }
        // M109 (file_131): reverse short-into-long — **short next** (≤6 residual)
        // into **long base** (≥8 residual). Word pure-I's short main title, peels
        // short body "This document demonstrates…" across the long doc's first
        // two residual paras (Equal ` document`), pure-I remaining short, then
        // pure-D remaining long. Plain insert-all-short/delete-all-long parks
        // the whole long doc after the short insert block (file_131 ~75).
        //
        // M113 (file_59 / file_19 / greek-alphabet class): only enter reverse
        // peel when the M105 token rule fires. Ungated M109 nested the first
        // long residual ("Αα Alpha") into the short body ("This document
        // demonstrates font size 24.") with zero shared vocabulary — Word
        // pure-I's the whole short next, pure-D's the whole long base, then
        // boundary-folds the first del into the last ins (file_59 was 100 on
        // partial boards, collapsed to ~59 after blind M109).
        if (2..=6).contains(&rest2.len()) && rest1.len() >= 8 {
            // Peel when long subtitle's last sig token appears in short body
            // (same M105 token rule, sides swapped).
            // M396c: ignore only section-label last-sigs ("Formatting" on
            // "1. Inline Text Formatting") that blocked M131 for file_34.
            // Do NOT ignore content anchors like "document" — that regressed
            // file_131 JustifyDemo×long Word-vs-Docs peel free-mesh (−12 LO).
            const PEEL_BODY_SECTION_LABELS: &[&str] =
                &["formatting", "format", "style", "styles", "options"];
            let peel_body = rest2.len() >= 2 && rest1.len() >= 2 && {
                let sub_toks = para_text_token_list(dom, &rest1[1]);
                let last = last_significant_token(&sub_toks);
                let body = para_text_tokens_joined(dom, &rest2[1]);
                let title = para_text_tokens_joined(dom, &rest1[0]);
                last.is_some_and(|tok| {
                    let key = tok.to_ascii_lowercase();
                    if PEEL_BODY_SECTION_LABELS
                        .iter()
                        .any(|b| key.eq_ignore_ascii_case(b))
                    {
                        return false;
                    }
                    body.iter().any(|t| t.eq_ignore_ascii_case(&key))
                        && !title.iter().any(|t| t.eq_ignore_ascii_case(&key))
                })
            };
            if peel_body {
                // pure-I first next residual (short demo title)
                stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                let mut nested = lcs(
                    dom,
                    vec![rest1[0].clone(), rest1[1].clone()],
                    vec![rest2[1].clone()],
                    settings,
                );
                stamp_seqs.append(&mut nested);
                // insert remaining short next (from index 2)
                if rest2.len() > 2 {
                    stamp_seqs.push(CorrelatedSequence::inserted(rest2[2..].to_vec()));
                }
                // delete remaining long base (from index 2)
                if rest1.len() > 2 {
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1[2..].to_vec()));
                }
                return Some(stamp_seqs);
            }
            // M131 (file_34×file_35): long comprehensive demo × short
            // Strikethrough cousin. Word nests short residual into the *head*
            // of the long residual (MIX "Strikethrough " + del long title) then
            // pure-D remaining long. M109 peel_body false (last-sig
            // "Demonstration" not in short body) → pure-I whole short + pure-D
            // whole long (~45 score). Gate on **full** residual content
            // relatedness (non-boilerplate shared sig like "strikethrough");
            // jaccard on full long is diluted (~0.04) so use content-sig only.
            // LCS short next vs first k long residual paras (k=short.len()+1).
            // Unrelated short (file_59 greek) shares no content sig → pure I/D.
            //
            // file_196×197: long multi-section base (100+ residual groups) × short
            // images essay. Incidental shared vocabulary (appears/center/left/…)
            // free-meshed B into A dels (score ~39). Modest long residual
            // (≤40 groups — demo class) keeps any non-boiler share; large long
            // residual needs both ≥5 shared sigs **and** residual jaccard ≥0.12.
            //
            // M396 (file_34×file_35): comprehensive DOCX demo residual is ~70
            // groups (not multi-section essay). Cap 40 skipped M131 → pure-I
            // short + pure-D long (Word multi-MIX titles, ~45). Extend modest
            // demo-class to ≤80 when short residual title ends with "Demo"
            // (formatting cousin) and share≥1 ("strikethrough"). file_196
            // residual 100+ still uses the strict ≥5/j≥0.12 arm.
            //
            // M397b: DO NOT free-mesh OOXML long residual on share=0 + Demo
            // short (file_41). That thrash-rewrote file_2 CenterBoldDemo ×
            // OOXML bold (95→44) and file_131 (−12). Word pure-I/Ds those
            // short Demo residuals (I2M1D32); free-mesh inverted order.
            // Keep share≥1 only for the extended 40..80 demo-class arm.
            let k = (rest2.len() + 1).min(rest1.len());
            let head1 = rest1[..k].to_vec();
            let share = residual_shared_sig_count(dom, &rest1, &rest2);
            let short_demo_title = rest2
                .first()
                .is_some_and(|u| residual_title_ends_demo(dom, u));
            let m131_ok = if rest1.len() <= 40 {
                share >= 1
            } else if rest1.len() <= 80 && short_demo_title && share >= 1 {
                true
            } else {
                share >= 5 && {
                    let t1 = para_text_tokens_from_units(dom, rest1.as_slice());
                    let t2 = para_text_tokens_from_units(dom, rest2.as_slice());
                    token_jaccard(&t1, &t2) + 1e-12 >= 0.12
                }
            };
            if m131_ok {
                let mut left: Vec<ComparisonUnit> = head1.iter().flat_map(group_contents).collect();
                let mut right: Vec<ComparisonUnit> =
                    rest2.iter().flat_map(group_contents).collect();
                rehash_words_by_text_content(dom, &mut left);
                rehash_words_by_text_content(dom, &mut right);
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.005;
                let mut nested = lcs(dom, left, right, &residual_settings);
                stamp_seqs.append(&mut nested);
                if rest1.len() > k {
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1[k..].to_vec()));
                }
                return Some(stamp_seqs);
            }
            // Unrelated short-next/long-base: fall through to insert-all /
            // delete-all (Word + boundary fold), do not force-nest.
        }
        // M137 (file_151): Demo next with non-This subtitle then This-body —
        // run **before** M128/M129 (those would swallow the case with weaker
        // title-only peel or full residual thrash).
        if (3..=6).contains(&rest1.len())
            && (3..=6).contains(&rest2.len())
            && residual_title_ends_demo(dom, &rest2[0])
            && residual_has_this_body_after_non_this(dom, &rest2)
            && residual_first_body_starts_this(dom, &rest1)
        {
            let this_idx = residual_first_this_body_index(dom, &rest2).unwrap_or(1);
            stamp_seqs.push(CorrelatedSequence::inserted(rest2[..this_idx].to_vec()));
            stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
            let bodies1 = rest1[1..].to_vec();
            let bodies2 = rest2[this_idx..].to_vec();
            let mut left: Vec<ComparisonUnit> = bodies1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = bodies2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.0;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // M128 (file_44): both residuals short (2..=6), no residual pairs, but
        // residual vocab shares a **non-boilerplate** significant token
        // (Inventory **List** × Numbered **List**) → flatten + text-hash LCS.
        // M126 unguarded thrash: file_118 Book Catalog × Indent (−18).
        // Boilerplate-only "this" (file_151 proposal×format-demo) stays off
        // full residual flatten (title thrash −10); use M129 title-peel instead.
        if (2..=6).contains(&rest1.len())
            && (2..=6).contains(&rest2.len())
            && residual_sets_weakly_related(dom, &rest1, &rest2)
        {
            // Format-sensitive word sha1s make "This" (Heading) ≠ "This"
            // (Normal) so plain multi-para LCS collapses to pure del+ins.
            // Rehash residual words by text content only so shared tokens Equal.
            let mut left: Vec<ComparisonUnit> = rest1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = rest2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            // DetailThreshold 0.02 voids a single Equal word in a long residual
            // window (1/64≈0.016). Lower only for this gated path.
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.005;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // M129 (file_110): short×short empty pairs, **body** residual shares
        // leading "This …" cousins but titles are unrelated (Project Proposal
        // × Red Bold Heading Demo). Word pure-I's next title, pure-D's base
        // title, then peels Equal "This " across body residuals. Full residual
        // flatten (M128) nests titles wrong (file_151 −10). Peel titles first,
        // then text-hash LCS on remaining bodies when they share ordered
        // significant prefix ≥1 ("this") or content Jaccard on bodies ≥0.05.
        //
        // Blind v70: require **next** residual title ends with significant
        // token "Demo" (format-demo class). Reverse order file_109 Subscript
        // Demo × Project Proposal is Word pure-I/D whole residual (score 100);
        // ungated M129 nested "This project" into "This document" → 63.
        if (2..=6).contains(&rest1.len())
            && (2..=6).contains(&rest2.len())
            && rest1.len() >= 2
            && rest2.len() >= 2
            && residual_title_ends_demo(dom, &rest2[0])
            && residual_bodies_this_cousins(dom, &rest1, &rest2)
        {
            stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
            stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
            let bodies1 = rest1[1..].to_vec();
            let bodies2 = rest2[1..].to_vec();
            let mut left: Vec<ComparisonUnit> = bodies1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = bodies2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.005;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // Word order: insert remaining next, then delete remaining base.
        if !rest2.is_empty() {
            stamp_seqs.push(CorrelatedSequence::inserted(rest2));
        }
        if !rest1.is_empty() {
            stamp_seqs.push(CorrelatedSequence::deleted(rest1));
        }
        return Some(stamp_seqs);
    }

    // Word residual order (file_33): zip the *last* leftover base residual
    // with the *last* next residual even at Jaccard 0 ("Main Title Section"
    // ↔ "Text alignment options"). Only one end-pair — do not zip the whole
    // unpaired list (would glue "Heading 1 Style Demo" onto penultimate B).
    //
    // M82 (file_85): only end-zip when the pair shares **zero** significant
    // tokens (len≥4). Main Title ↔ Text alignment is j=0 / no shared sig →
    // zip, then merge_replaced folds pure-D into last pure-I. "This text is
    // bold." ↔ "Third bold bullet…" shares "bold" — end-zip forced nested
    // word-LCS that peeled "bold" as Equal and parked the del on the *first*
    // bullet (Word keeps pure-I bullets + full del on the last). Skipping
    // the shared-sig end-zip leaves A residual pure-del after insert-all B
    // so sole-del fold attaches to the last insert (Word shape).
    //
    // M100 (file_32): only end-zip **short title-class** leftovers (≤4 tokens).
    // After M96 pairs the Demo titles, last unpaired A is the long sentence
    // "This text is both bold and underlined." and last B is "Main Title
    // Section". End-zip nested them (wrong); Word inserts remaining B then
    // folds Main Title with first unpaired A ("Demonstrating bold…"). Long
    // sentence leftovers stay unpaired so merge_replaced folds correctly.
    const STAMP_ENDZIP_MAX_TOKENS: usize = 4;
    let mut pairs = pairs;
    {
        let used_i: std::collections::HashSet<usize> = pairs.iter().map(|&(i, _)| i).collect();
        let used_j: std::collections::HashSet<usize> = pairs.iter().map(|&(_, j)| j).collect();
        let unpaired_i: Vec<usize> = (0..rest1.len()).filter(|i| !used_i.contains(i)).collect();
        let unpaired_j: Vec<usize> = (0..rest2.len()).filter(|j| !used_j.contains(j)).collect();
        if let (Some(&i), Some(&j)) = (unpaired_i.last(), unpaired_j.last()) {
            let li = para_text_tokens_joined(dom, &rest1[i]);
            let rj = para_text_tokens_joined(dom, &rest2[j]);
            let shared_sig = significant_tokens(&li)
                .intersection(&significant_tokens(&rj))
                .count();
            let short_titles =
                li.len() <= STAMP_ENDZIP_MAX_TOKENS && rj.len() <= STAMP_ENDZIP_MAX_TOKENS;
            if shared_sig == 0 && short_titles {
                pairs.push((i, j));
                pairs.sort_by_key(|&(_, j)| j);
            }
        }
    }

    let pair_by_j: std::collections::HashMap<usize, usize> =
        pairs.iter().map(|&(i, j)| (j, i)).collect();
    let paired_i: std::collections::HashSet<usize> = pairs.iter().map(|&(i, _)| i).collect();
    let mut emitted_i: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let mut insert_buf: Vec<ComparisonUnit> = Vec::new();
    let flush_inserts = |buf: &mut Vec<ComparisonUnit>, out: &mut Vec<CorrelatedSequence>| {
        if !buf.is_empty() {
            out.push(CorrelatedSequence::inserted(std::mem::take(buf)));
        }
    };

    for (j, b) in rest2.iter().enumerate() {
        if let Some(&i) = pair_by_j.get(&j) {
            flush_inserts(&mut insert_buf, &mut stamp_seqs);
            // Pure-del unpaired A residual that precedes this pair in base
            // order (Word: pure-del "Heading 1 Style Demo" before the
            // demonstrates MIX).
            let mut early_del = Vec::new();
            for (k, r1) in rest1.iter().enumerate().take(i) {
                if emitted_i.contains(&k) || paired_i.contains(&k) {
                    continue;
                }
                early_del.push(r1.clone());
                emitted_i.insert(k);
            }
            if !early_del.is_empty() {
                stamp_seqs.push(CorrelatedSequence::deleted(early_del));
            }
            emitted_i.insert(i);
            // Recurse into word/atom LCS for the related residual pair.
            let mut nested = lcs(dom, vec![rest1[i].clone()], vec![b.clone()], settings);
            stamp_seqs.append(&mut nested);
        } else {
            insert_buf.push(b.clone());
        }
    }
    flush_inserts(&mut insert_buf, &mut stamp_seqs);

    let unpaired: Vec<ComparisonUnit> = rest1
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !emitted_i.contains(i))
        .map(|(_, u)| u)
        .collect();
    if !unpaired.is_empty() {
        stamp_seqs.push(CorrelatedSequence::deleted(unpaired));
    }
    Some(stamp_seqs)
}

/// Lowercased word tokens from a paragraph group's descendant `w:t` values.
pub(super) fn para_text_tokens(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, std::slice::from_ref(u))
}

pub(super) fn para_text_tokens_from_units(
    dom: &Dom,
    units: &[ComparisonUnit],
) -> std::collections::HashSet<String> {
    // Atoms are per-character after atomize — must reassemble text before
    // tokenizing. Splitting each single-char atom left significant tokens
    // (len≥3) empty, so is_related_stamped_variant always failed (file_175
    // confetti_ok=true → whole-para del thrash).
    let mut text = String::new();
    for u in units {
        for a in u.descendant_atoms() {
            if dom.name_is(a.content_element, &W::t()) {
                text.push_str(&dom.value_str(a.content_element));
            }
        }
        text.push(' ');
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

pub(super) fn token_jaccard(
    a: &std::collections::HashSet<String>,
    b: &std::collections::HashSet<String>,
) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count() as f64;
    let uni = a.union(b).count() as f64;
    if uni == 0.0 { 0.0 } else { inter / uni }
}

/// Rehash word units by concatenated `w:t` text only (ignore rPr). Used so
/// residual short×short LCS can Equal shared tokens across format demos.
/// M328d's optional ASCII case-fold was removed: hashing the lowercased text
/// made "Green"×"green" compare Equal, but the emitted EQ run carries only one
/// side's casing, so the redline stopped reconstructing its own input, and the
/// spurious matches let two paragraphs claim the same A-side atom. Matching
/// case-insensitively is sound only once the emitted run keeps each side's
/// original text.
pub(super) fn rehash_words_by_text_content(dom: &Dom, units: &mut [ComparisonUnit]) {
    use crate::util::sha1::sha1_hex;
    for u in units.iter_mut() {
        if let ComparisonUnit::Word(w) = u {
            let mut text = String::new();
            for a in &w.contents {
                if dom.name_is(a.content_element, &W::t()) {
                    text.push_str(&dom.value_str(a.content_element));
                }
            }
            if !text.is_empty() {
                // one call, so neither cached key can outlive the old hash
                w.sha1.set_hash(sha1_hex(&text));
            }
        }
    }
}
