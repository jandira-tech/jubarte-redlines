// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Longest common run of comparison units: the indexed search, its scan oracle and the content score.

use super::*;

/// M4.C.2 — `DoLcsAlgorithm` Step B: longest common CONTIGUOUS run by
/// [`ComparisonUnit::sha1`]. Returns `(i1, i2, len)`; `len == 0` means
/// no common run.
///
/// Ranking (M84 / file_81): in Word mode prefer higher non-separator
/// content length, then longer run. A pure-space common unit of length 1
/// used to beat an equal-length content word `"style"` (first-found wins);
/// Step F then voided the space and the whole residual became del+ins,
/// while Word keeps `Equal "style"`. The PowerTools-faithful preset ranks
/// by pure length.
pub fn longest_common_run(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> (usize, usize, usize) {
    longest_common_run_with_dom(None, cul1, cul2, None)
}

/// Word-mode LCR: when `dom`+`settings` are provided, rank ties by non-separator
/// content so glue spaces do not steal equal-length content matches.
///
/// Dispatches to [`longest_common_run_indexed`] — a hash-indexed rewrite that
/// returns the exact same `(i1, i2, len)` as the historical O(n·m)
/// `longest_common_run_scan` but skips the pairs that cannot possibly match
/// (proven by `indexed_matches_scan`).
pub(super) fn longest_common_run_with_dom(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    longest_common_run_indexed(dom, cul1, cul2, settings)
}

/// Extend a contiguous common run starting at `(i1, i2)`, returning its length.
///
/// Equality is decided by the cached 128-bit FNV-1a fingerprint of the `sha1()`
/// hex string ([`ComparisonUnit::sha1_key128`]), one int compare per step; the
/// hex string itself is not compared. Equal hashes always share a fingerprint, so
/// no true match is missed. A distinct pair is taken for equal only on a 128-bit
/// FNV-1a collision between two SHA-1 hex strings (the inputs are digests, so a
/// caller cannot choose the bytes being hashed). A collision of the 64-bit
/// [`ComparisonUnit::sha1_key`] alone is harmless: the 128-bit values differ and
/// the run stops there.
#[inline]
pub(super) fn extend_common_run(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    i1: usize,
    i2: usize,
) -> usize {
    let (mut t1, mut t2) = (i1, i2);
    let mut len = 0;
    // The 128-bit FNV-1a fingerprint stands in for comparing the hash strings,
    // and no string compare follows: equal hashes always share the
    // fingerprint, and distinct 40-character hex digests are not expected to
    // collide at 128 bits. It avoids the per-step 40-byte hex memcmp.
    while t1 < cul1.len() && t2 < cul2.len() && cul1[t1].sha1_key128() == cul2[t2].sha1_key128() {
        t1 += 1;
        t2 += 1;
        len += 1;
    }
    len
}

/// Content score of the run `cul1[i1..i1 + len]`: non-separator character count
/// in Word mode (`dom`+`settings` present), else the historical pure-length rank.
///
/// When `prefix` is `Some` (LCS-SCORE-01), uses O(1) prefix sums of per-unit
/// non-separator scores — exact equal to walking the run each time.
#[inline]
pub(super) fn common_run_content_score(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    i1: usize,
    len: usize,
    settings: Option<&WmlComparerSettings>,
    prefix: Option<&[usize]>,
) -> usize {
    if let Some(p) = prefix {
        // p[0]=0, p[k]=sum of unit scores for first k units.
        debug_assert_eq!(p.len(), cul1.len() + 1);
        return p[i1 + len] - p[i1];
    }
    if let (Some(d), Some(s)) = (dom, settings) {
        run_non_separator_text_len(d, &cul1[i1..i1 + len], s)
    } else {
        // Faithful / no-dom: pure length ranking (historical).
        len
    }
}

/// LCS-SCORE-01: non-separator content score of a single comparison unit.
pub(super) fn unit_non_separator_text_len(
    dom: &Dom,
    unit: &ComparisonUnit,
    settings: &WmlComparerSettings,
) -> usize {
    let mut score = 0usize;
    // Avoid allocating a descendant_atoms Vec for the common Word case.
    match unit {
        ComparisonUnit::Word(w) => {
            for a in &w.contents {
                if dom.name_is(a.content_element, &W::t()) {
                    // ATOM-TEXT-01: borrow single-text-child leaves.
                    score += dom
                        .value_str(a.content_element)
                        .chars()
                        .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                        .count();
                }
            }
        }
        ComparisonUnit::Group(g) => {
            for c in &g.contents {
                score += unit_non_separator_text_len(dom, c, settings);
            }
        }
    }
    score
}

/// LCS-SCORE-01: prefix sums of per-unit non-separator scores.
/// `prefix[0] == 0`, `prefix[i+1] == prefix[i] + score(cul[i])`.
pub(super) fn non_separator_prefix_sums(
    dom: &Dom,
    cul: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Vec<usize> {
    let mut prefix = Vec::with_capacity(cul.len() + 1);
    prefix.push(0);
    for u in cul {
        let s = unit_non_separator_text_len(dom, u, settings);
        prefix.push(prefix.last().copied().unwrap_or(0) + s);
    }
    prefix
}

/// Whether a window holds the words of one paragraph at most: words only, and
/// no paragraph mark before its last word.
pub(super) fn within_one_paragraph(dom: &Dom, cul: &[ComparisonUnit]) -> bool {
    let Some((_, before_last)) = cul.split_last() else {
        return true;
    };
    cul.iter().all(|u| matches!(u, ComparisonUnit::Word(_)))
        && !before_last.iter().any(|u| {
            matches!(u, ComparisonUnit::Word(w) if w.contents.iter().any(|a| atom_is_ppr(dom, a)))
        })
}

/// Keep the **first-found** candidate maximising `(content_score, len)`. Strict
/// `>` replacement means ties never displace the incumbent — so the winner is the
/// earliest one in the enumeration order (`i1` ascending, then `i2` ascending).
///
/// With `diagonal` (Word mode, within one paragraph), a tie goes to the
/// run nearer the diagonal (`|i1 - i2|` smaller), as Word pairs repeated text
/// in place: of two equal `1,000 again.` sentences turned into `1,500 again.`
/// and `2,000 again.`, Word changes `000` and `1` in place
/// (tests/m_word_tokens.rs), where the first-found run paired the first
/// sentence with the second. Windows across paragraphs keep first-found (the
/// list and equal-count pairings of m393 and m45 depend on it).
#[inline]
pub(super) fn consider_candidate(
    best: &mut Option<(usize, usize, usize, usize)>,
    cand: (usize, usize, usize, usize),
    diagonal: bool,
) {
    let better = match best {
        None => true,
        Some(b) => {
            cand.0 > b.0
                || (cand.0 == b.0
                    && (cand.1 > b.1
                        || (diagonal
                            && cand.1 == b.1
                            && cand.2.abs_diff(cand.3) < b.2.abs_diff(b.3))))
        }
    };
    if better {
        *best = Some(cand);
    }
}

/// Historical O(n·m) reference: scan every `(i1, i2)` start, extend, rank. Kept
/// as the equivalence oracle for [`longest_common_run_indexed`] (`indexed_matches_scan`);
/// not compiled into release builds.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn longest_common_run_scan(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    let prefix = match (dom, settings) {
        (Some(d), Some(s)) => Some(non_separator_prefix_sums(d, cul1, s)),
        _ => None,
    };
    let diagonal = settings.is_some_and(|s| s.merge_replaced_paragraphs)
        && dom.is_some_and(|d| within_one_paragraph(d, cul1) && within_one_paragraph(d, cul2));
    // best: (content_score, len, i1, i2)
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        for i2 in 0..cul2.len() {
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2), diagonal);
            }
        }
    }
    best.map(|(_, len, i1, i2)| (i1, i2, len))
        .unwrap_or((0, 0, 0))
}

/// Hash-indexed longest-common-run — the asymptotic fix.
///
/// Only `(i1, i2)` starts whose first units share a hash can produce a run, so we
/// bucket `cul2` positions by their u64 fingerprint key and, for each `i1`, probe
/// **only** the matching bucket. Buckets are built in ascending `i2` order, so
/// probing one visits the same starts, in the same `i2`-ascending order, that the
/// scan would reach for that `i1`. The candidate sequence — and therefore the
/// first-found winner — is identical to `longest_common_run_scan`; the scan
/// merely also visits the (never-winning) `len == 0` pairs in between. Proven by
/// `indexed_matches_scan`. A 64-bit key collision lands in a bucket but yields
/// `len == 0` (the 128-bit compare in [`extend_common_run`] differs), exactly as
/// the scan skips it.
pub(super) fn longest_common_run_indexed(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    // LCS-SCORE-01: precompute per-unit non-separator scores once per LCR call.
    let prefix = match (dom, settings) {
        (Some(d), Some(s)) => Some(non_separator_prefix_sums(d, cul1, s)),
        _ => None,
    };

    // Bucket cul2 positions by their u64 fingerprint key. Pushing in ascending
    // i2 order keeps each bucket ascending, so a probe reproduces the scan's
    // i2-ascending visitation for a given i1.
    let mut index: std::collections::HashMap<u64, Vec<usize>, U64BuildHasher> =
        std::collections::HashMap::with_capacity_and_hasher(cul2.len(), U64BuildHasher::default());
    for (i2, u) in cul2.iter().enumerate() {
        index.entry(u.sha1_key()).or_default().push(i2);
    }

    let diagonal = settings.is_some_and(|s| s.merge_replaced_paragraphs)
        && dom.is_some_and(|d| within_one_paragraph(d, cul1) && within_one_paragraph(d, cul2));
    // best: (content_score, len, i1, i2) — same tuple/tie-break as the scan.
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        let Some(positions) = index.get(&cul1[i1].sha1_key()) else {
            continue;
        };
        for &i2 in positions {
            // Interior-run skip: when the diagonal predecessor (i1-1, i2-1) is
            // itself a true match, the run through (i1, i2) is a strictly
            // shorter suffix of a run already considered from (i1-1, i2-1)
            // (content_score is monotone in the run, and consider_candidate
            // requires a *strict* improvement — the longer super-run always
            // wins or ties). So (i1, i2) can never become `best`; skipping it is
            // output-identical and removes the O(run_len^2) re-extension of long
            // shared passages. The 128-bit fingerprint compare mirrors
            // extend_common_run, so a 64-bit key collision at the predecessor
            // never triggers a wrongful skip.
            if i1 > 0 && i2 > 0 && cul1[i1 - 1].sha1_key128() == cul2[i2 - 1].sha1_key128() {
                continue;
            }
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2), diagonal);
            }
        }
    }
    best.map(|(_, len, i1, i2)| (i1, i2, len))
        .unwrap_or((0, 0, 0))
}

/// M-ANCHOR helper: total real-content text length of a common run — the sum
/// of trimmed `w:t` character counts across the run's units.
pub(super) fn run_real_text_len(dom: &Dom, run: &[ComparisonUnit]) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name_is(a.content_element, &W::t()))
        .map(|a| dom.value_str(a.content_element).trim().chars().count())
        .sum()
}

/// Non-separator character count in a run (Word-mode LCR ranking). Spaces and
/// other `word_separators` do not count — a pure-space unit scores 0.
pub(super) fn run_non_separator_text_len(
    dom: &Dom,
    run: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name_is(a.content_element, &W::t()))
        .map(|a| {
            // ATOM-TEXT-01: borrow single-char / single-text-child atoms.
            dom.value_str(a.content_element)
                .chars()
                .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                .count()
        })
        .sum()
}
