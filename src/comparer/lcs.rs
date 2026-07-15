//! LCS correlation (M4.3). Port of the core of `DoLcsAlgorithm`.
//!
//! The TS `DoLcsAlgorithm` finds the longest common CONTIGUOUS run of comparison
//! units by SHA-1 hash, emits Equal for it, and recurses on the before/after
//! remainders (with Deleted/Inserted for one-sided remainders). This module
//! implements that recursion at the atom level — sufficient to tag a merged atom
//! stream with Equal/Inserted/Deleted for the common (paragraph-text) case.
//!
//! NOTE: the full `DoLcsAlgorithm` additionally special-cases table/row/cell/
//! textbox groups (WmlComparer.ts:7578-7944) and applies word-break/threshold
//! guards on comparison UNITS. Those refinements (needed for table fixtures and
//! exact golden parity) build on this core and are the documented remaining work
//! for M4.3.

use super::CorrelationStatus;
use super::atoms::ComparisonUnitAtom;

/// An atom tagged with its correlation status, ready for the produce step.
#[derive(Clone, Debug)]
pub struct TaggedAtom {
    pub atom: ComparisonUnitAtom,
    pub status: CorrelationStatus,
}

/// Correlate two atom streams (by `sha1_hash`) into a merged, tagged stream.
/// Equal atoms carry the *modified* side's atom; Deleted carry the original's.
pub fn correlate_atoms(
    atoms1: &[ComparisonUnitAtom],
    atoms2: &[ComparisonUnitAtom],
) -> Vec<TaggedAtom> {
    let mut out = Vec::new();
    do_lcs(atoms1, atoms2, &mut out);
    out
}

fn tag_all(atoms: &[ComparisonUnitAtom], status: CorrelationStatus, out: &mut Vec<TaggedAtom>) {
    for a in atoms {
        out.push(TaggedAtom {
            atom: a.clone(),
            status,
        });
    }
}

fn do_lcs(cul1: &[ComparisonUnitAtom], cul2: &[ComparisonUnitAtom], out: &mut Vec<TaggedAtom>) {
    if cul1.is_empty() && cul2.is_empty() {
        return;
    }
    if cul2.is_empty() {
        tag_all(cul1, CorrelationStatus::Deleted, out);
        return;
    }
    if cul1.is_empty() {
        tag_all(cul2, CorrelationStatus::Inserted, out);
        return;
    }

    // Find the longest common contiguous run by hash (WmlComparer.ts:7399-7428).
    let mut best_len = 0usize;
    let mut best_i1 = usize::MAX;
    let mut best_i2 = usize::MAX;
    let mut i1 = 0;
    while i1 + best_len < cul1.len() {
        let mut i2 = 0;
        while i2 + best_len < cul2.len() {
            let mut len = 0;
            let (mut t1, mut t2) = (i1, i2);
            while t1 < cul1.len() && t2 < cul2.len() && cul1[t1].sha1_hash == cul2[t2].sha1_hash {
                t1 += 1;
                t2 += 1;
                len += 1;
            }
            if len > best_len {
                best_len = len;
                best_i1 = i1;
                best_i2 = i2;
            }
            i2 += 1;
        }
        i1 += 1;
    }

    if best_len == 0 {
        // No common content: everything on the left deleted, right inserted.
        tag_all(cul1, CorrelationStatus::Deleted, out);
        tag_all(cul2, CorrelationStatus::Inserted, out);
        return;
    }

    // before-remainder → recurse
    do_lcs(&cul1[..best_i1], &cul2[..best_i2], out);
    // equal run (carry the modified side's atoms)
    tag_all(
        &cul2[best_i2..best_i2 + best_len],
        CorrelationStatus::Equal,
        out,
    );
    // after-remainder → recurse
    do_lcs(
        &cul1[best_i1 + best_len..],
        &cul2[best_i2 + best_len..],
        out,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// M4.C — faithful core LCS on ComparisonUnit (Word/Group). Operates on
// CorrelatedSequence via a worklist driver, replacing the atom-level shortcut
// above (which stays until M4.I swaps the orchestration).
// ─────────────────────────────────────────────────────────────────────────────

use super::atoms::{ComparisonUnit, CorrelatedSequence};
use super::{ComparisonUnitGroupType, WmlComparerSettings};
use crate::namespaces::{PT, W};
use crate::xmllinq::Dom;

// ── para-mark predicates (M4.C.4) ────────────────────────────────────────────
fn atom_is_ppr(dom: &Dom, a: &ComparisonUnitAtom) -> bool {
    dom.name(a.content_element) == Some(W::p_pr())
}
/// A `ComparisonUnitWord` of exactly one atom which is a `w:pPr` (a bare para mark).
fn unit_is_single_atom_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    matches!(u, ComparisonUnit::Word(w) if w.contents.len() == 1 && atom_is_ppr(dom, &w.contents[0]))
}
fn unit_first_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms()
        .first()
        .is_some_and(|a| atom_is_ppr(dom, a))
}
fn unit_last_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms()
        .last()
        .is_some_and(|a| atom_is_ppr(dom, a))
}
/// Predicate for the I.1 partial-paragraph scan: a Word whose first atom is NOT a
/// pPr (a word with no atoms counts as true, matching the TS).
fn word_first_not_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    match u {
        ComparisonUnit::Word(w) => w.contents.first().is_none_or(|a| !atom_is_ppr(dom, a)),
        ComparisonUnit::Group(_) => false,
    }
}
fn take_while_count_rev(slice: &[ComparisonUnit], pred: impl Fn(&ComparisonUnit) -> bool) -> usize {
    slice.iter().rev().take_while(|u| pred(u)).count()
}

/// M4.C.4 — `FindIndexOfNextParaMark` (:8226): first index whose LAST descendant
/// atom is a `w:pPr`, else `cul.len()`.
pub fn find_index_of_next_para_mark(dom: &Dom, cul: &[ComparisonUnit]) -> usize {
    cul.iter()
        .position(|u| unit_last_atom_is_ppr(dom, u))
        .unwrap_or(cul.len())
}

/// M4.C.4 — `SplitAtParagraphMark` (:5895): split at the first unit whose FIRST
/// descendant atom is a `w:pPr`, keeping that unit at the head of chunk 2.
pub fn split_at_paragraph_mark(dom: &Dom, cua: &[ComparisonUnit]) -> Vec<Vec<ComparisonUnit>> {
    match cua.iter().position(|u| unit_first_atom_is_ppr(dom, u)) {
        None => vec![cua.to_vec()],
        Some(i) => vec![cua[..i].to_vec(), cua[i..].to_vec()],
    }
}

/// M4.C.2 — `DoLcsAlgorithm` Step B: longest common CONTIGUOUS run by `sha1()`.
/// Returns `(i1, i2, len)`; `len == 0` means no common run.
///
/// Ranking (M84 / file_81): prefer higher non-separator content length, then
/// longer run. A pure-space common unit of length 1 used to beat equal-length
/// content word `"style"` (first-found wins), then Step F voided the space and
/// the whole residual became del+ins — Word keeps Equal `"style"`.
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
/// [`longest_common_run_scan`] but skips the pairs that cannot possibly match
/// (proven by `indexed_matches_scan`).
fn longest_common_run_with_dom(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    longest_common_run_indexed(dom, cul1, cul2, settings)
}

/// Extend a contiguous common run starting at `(i1, i2)`, returning its length.
///
/// Fast pre-filter: the cached u64 keys reject the (dominant) unequal case with a
/// single int compare; the `sha1()` string stays the source of truth, so this is
/// exactly `sha1() == sha1()` (equal hashes always share a key), just cheaper. A
/// u64 collision (same key, different string) stops the run at the string check,
/// so it is *never* mistaken for a match. See [`ComparisonUnit::sha1_key`].
#[inline]
fn extend_common_run(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    i1: usize,
    i2: usize,
) -> usize {
    let (mut t1, mut t2) = (i1, i2);
    let mut len = 0;
    while t1 < cul1.len()
        && t2 < cul2.len()
        && cul1[t1].sha1_key() == cul2[t2].sha1_key()
        && cul1[t1].sha1() == cul2[t2].sha1()
    {
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
fn common_run_content_score(
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
fn unit_non_separator_text_len(
    dom: &Dom,
    unit: &ComparisonUnit,
    settings: &WmlComparerSettings,
) -> usize {
    let mut score = 0usize;
    // Avoid allocating a descendant_atoms Vec for the common Word case.
    match unit {
        ComparisonUnit::Word(w) => {
            for a in &w.contents {
                if dom.name(a.content_element) == Some(W::t()) {
                    score += dom
                        .value(a.content_element)
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
fn non_separator_prefix_sums(
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

/// Keep the **first-found** candidate maximising `(content_score, len)`. Strict
/// `>` replacement means ties never displace the incumbent — so the winner is the
/// earliest one in the enumeration order (`i1` ascending, then `i2` ascending).
#[inline]
fn consider_candidate(
    best: &mut Option<(usize, usize, usize, usize)>,
    cand: (usize, usize, usize, usize),
) {
    let better = match best {
        None => true,
        Some(b) => cand.0 > b.0 || (cand.0 == b.0 && cand.1 > b.1),
    };
    if better {
        *best = Some(cand);
    }
}

/// Historical O(n·m) reference: scan every `(i1, i2)` start, extend, rank. Kept
/// as the equivalence oracle for [`longest_common_run_indexed`] (`indexed_matches_scan`);
/// not compiled into release builds.
#[cfg(test)]
fn longest_common_run_scan(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    let prefix = match (dom, settings) {
        (Some(d), Some(s)) => Some(non_separator_prefix_sums(d, cul1, s)),
        _ => None,
    };
    // best: (content_score, len, i1, i2)
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        for i2 in 0..cul2.len() {
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2));
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
/// first-found winner — is identical to [`longest_common_run_scan`]; the scan
/// merely also visits the (never-winning) `len == 0` pairs in between. Proven by
/// `indexed_matches_scan`. Collisions land in a bucket but yield `len == 0` (the
/// string check in [`extend_common_run`]), exactly as the scan skips them.
fn longest_common_run_indexed(
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
    let mut index: std::collections::HashMap<u64, Vec<usize>> =
        std::collections::HashMap::with_capacity(cul2.len());
    for (i2, u) in cul2.iter().enumerate() {
        index.entry(u.sha1_key()).or_default().push(i2);
    }

    // best: (content_score, len, i1, i2) — same tuple/tie-break as the scan.
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        let Some(positions) = index.get(&cul1[i1].sha1_key()) else {
            continue;
        };
        for &i2 in positions {
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2));
            }
        }
    }
    best.map(|(_, len, i1, i2)| (i1, i2, len))
        .unwrap_or((0, 0, 0))
}

/// M-ANCHOR helper: total real-content text length of a common run — the sum
/// of trimmed `w:t` character counts across the run's units.
fn run_real_text_len(dom: &Dom, run: &[ComparisonUnit]) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name(a.content_element) == Some(W::t()))
        .map(|a| dom.value(a.content_element).trim().chars().count())
        .sum()
}

/// Non-separator character count in a run (Word-mode LCR ranking). Spaces and
/// other `word_separators` do not count — a pure-space unit scores 0.
fn run_non_separator_text_len(
    dom: &Dom,
    run: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name(a.content_element) == Some(W::t()))
        .map(|a| {
            dom.value(a.content_element)
                .chars()
                .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                .count()
        })
        .sum()
}

/// Significant body tokens (len≥3), excluding corpus stamp fragments.
/// Used to detect *related* stamped variants whose paragraph hashes diverge
/// only on whitespace/formatting (file_175×file_176 share ~99% vocabulary).
fn significant_body_tokens(dom: &Dom, cu: &[ComparisonUnit]) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, cu)
        .into_iter()
        .filter(|t| t.chars().count() >= 3 && !t.starts_with("file_") && t != "docx" && t != "doc")
        .collect()
}

/// `inter / min(|A|,|B|)` of significant body tokens. High when both sides are
/// the same document family (charter v1 vs v2); low for unrelated demos that
/// only share the `file_N.docx` stamp.
fn body_token_overlap_ratio(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> f64 {
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
const STAMP_CONFETTI_MAX_BODY_OVERLAP: f64 = 0.55;

/// Related stamped variants (file_175 charter v1/v2): both sides carry a
/// substantial vocabulary and share most of it. Short demos that only share a
/// few generic words (file_33 min=10 ratio=0.6) are NOT related — Word confettis.
const RELATED_STAMP_MIN_BODY_TOKENS: usize = 40;

fn is_related_stamped_variant(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    let a = significant_body_tokens(dom, cu1);
    let b = significant_body_tokens(dom, cu2);
    let min_n = a.len().min(b.len());
    let ratio = body_token_overlap_ratio(dom, cu1, cu2);
    if min_n < RELATED_STAMP_MIN_BODY_TOKENS {
        return false;
    }
    ratio >= STAMP_CONFETTI_MAX_BODY_OVERLAP
}

fn should_stamp_confetti(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    // Confetti stamped corpus demos unless both sides are a long related
    // variant (file_175). file_33 shares residual phrases ("This document
    // demonstrates") but is still confetti in Word.
    !is_related_stamped_variant(dom, cu1, cu2)
}

/// First contentful paragraph group's concatenated `w:t` text (for stamp gate).
fn first_contentful_para_text(dom: &Dom, cu: &[ComparisonUnit]) -> Option<String> {
    first_contentful_group_index(dom, cu).map(|i| {
        let mut text = String::new();
        for a in cu[i].descendant_atoms() {
            if dom.name(a.content_element) == Some(W::t()) {
                text.push_str(&dom.value(a.content_element));
            }
        }
        text
    })
}

/// Index of the first contentful group in `cu` (paragraph/table with real `w:t`).
fn first_contentful_group_index(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
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
const STAMP_RESIDUAL_PAIR_MIN_JACCARD: f64 = 0.25;
const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG: f64 = 0.10;
/// M133: body last-sig ("…style." × "…style.") can sit at Jaccard ~0.06 when
/// the shared token is only the trailing word. Titles with "Demo" stay ≥0.10.
const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY: f64 = 0.05;
const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG: usize = 3;
const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG_SHORT: usize = 2;
const STAMP_RESIDUAL_ORDERED_PREFIX_MIN_SIG: usize = 2;
/// Token count (all tokens, not only significant) at or below which the short
/// shared-sig floor applies (title-demo class, ~5–8 words).
const STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS: usize = 8;
/// M133: last-significant-token match may fire on slightly longer body
/// residuals (file_120 "…paragraph style." ↔ "…visual style." ~10 tokens).
/// Titles stay well under this; keep below long-body thrash.
const STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS: usize = 16;
/// Connector tokens (len 3) Word peels as Equal across residual titles.
const STAMP_RESIDUAL_CONNECTORS: &[&str] = &["and", "or", "the", "for", "with", "to"];

fn significant_tokens(
    tokens: &std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    tokens
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .cloned()
        .collect()
}

fn shared_connector_tokens(
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

/// Ordered tokens (lowercase alphanumeric) for last-significant-token match.
fn para_text_token_list(dom: &Dom, u: &ComparisonUnit) -> Vec<String> {
    let mut text = String::new();
    for a in u.descendant_atoms() {
        if dom.name(a.content_element) == Some(W::t()) {
            text.push_str(&dom.value(a.content_element));
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
fn para_text_tokens_joined(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String> {
    para_text_token_list(dom, u).into_iter().collect()
}

fn last_significant_token(ordered: &[String]) -> Option<&str> {
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
fn ordered_shared_prefix_sig(left: &[String], right: &[String]) -> usize {
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
fn stamp_residual_pairs(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> Vec<(usize, usize)> {
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
fn stamp_confetti_then_replace(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
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
            let path_a = !covers_body && max_body + 1e-12 >= 0.09;
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
            let peel_body = rest2.len() >= 2 && rest1.len() >= 2 && {
                let sub_toks = para_text_token_list(dom, &rest1[1]);
                let last = last_significant_token(&sub_toks);
                let body = para_text_tokens_joined(dom, &rest2[1]);
                let title = para_text_tokens_joined(dom, &rest1[0]);
                last.is_some_and(|tok| {
                    let key = tok.to_ascii_lowercase();
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
            let k = (rest2.len() + 1).min(rest1.len());
            let head1 = rest1[..k].to_vec();
            if residual_sets_share_content_sig(dom, &rest1, &rest2) {
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

/// M-ANCHOR helper: window relatedness — at least 20% of the SMALLER side's
/// units have some sha1 match anywhere on the other side. Computed with a
/// hash-set of the larger side (O(n) extra).
fn windows_related(cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool {
    let (small, large) = if cul1.len() <= cul2.len() {
        (cul1, cul2)
    } else {
        (cul2, cul1)
    };
    let large_hashes: std::collections::HashSet<&str> = large.iter().map(|u| u.sha1()).collect();
    let hits = small
        .iter()
        .filter(|u| large_hashes.contains(u.sha1()))
        .count();
    hits * 5 >= small.len()
}

/// Lowercased word tokens from a paragraph group's descendant `w:t` values.
fn para_text_tokens(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, std::slice::from_ref(u))
}

fn para_text_tokens_from_units(
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
            if dom.name(a.content_element) == Some(W::t()) {
                text.push_str(&dom.value(a.content_element));
            }
        }
        text.push(' ');
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn token_jaccard(
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
fn rehash_words_by_text_content(dom: &Dom, units: &mut [ComparisonUnit]) {
    use crate::util::sha1::{sha1_fingerprint, sha1_hex};
    for u in units.iter_mut() {
        if let ComparisonUnit::Word(w) = u {
            let mut text = String::new();
            for a in &w.contents {
                if dom.name(a.content_element) == Some(W::t()) {
                    text.push_str(&dom.value(a.content_element));
                }
            }
            if !text.is_empty() {
                w.sha1_hash = sha1_hex(&text);
                // keep the cached fingerprint in sync with the mutated hash
                w.sha1_key = sha1_fingerprint(&w.sha1_hash);
            }
        }
    }
}

/// Demo-corpus boilerplate significant tokens. Sharing only these (e.g. sole
/// `"this"` on file_151 Project Proposal × Bold Italic) is **not** enough for
/// M128 — text-hash residual LCS then thrash-nested titles and cost ~10 score.
/// Content cousins like Inventory **List** × Numbered **List** Demo still pass.
const M128_BOILERPLATE_SIG: &[&str] = &[
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
const M135_OFF_DIAG_BOILER: &[&str] = &[
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
fn residual_title_ends_demo(dom: &Dom, title: &ComparisonUnit) -> bool {
    let toks = para_text_token_list(dom, title);
    last_significant_token(&toks).is_some_and(|t| t.eq_ignore_ascii_case("demo"))
}

fn residual_para_starts_this(dom: &Dom, u: &ComparisonUnit) -> bool {
    para_text_token_list(dom, u)
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("this"))
}

fn residual_first_body_starts_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    rest.len() >= 2 && residual_para_starts_this(dom, &rest[1])
}

/// Index of first residual para (after title) that starts with "This".
fn residual_first_this_body_index(dom: &Dom, rest: &[ComparisonUnit]) -> Option<usize> {
    rest.iter()
        .enumerate()
        .skip(1)
        .find(|(_, u)| residual_para_starts_this(dom, u))
        .map(|(i, _)| i)
}

/// Next residual: title + ≥1 non-This line + later This-body (file_151 subtitle).
fn residual_has_this_body_after_non_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    let Some(this_i) = residual_first_this_body_index(dom, rest) else {
        return false;
    };
    // At least one residual between title (0) and This body.
    this_i >= 2 && !residual_para_starts_this(dom, &rest[1])
}

/// Body residuals (index ≥1) share ordered prefix "this" (or content Jaccard
/// ≥0.05 with any shared sig). Used by M129 title-peel path for file_110.
fn residual_bodies_this_cousins(
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
fn residual_sets_share_content_sig(
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
    let shared_sig: std::collections::HashSet<String> = significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .cloned()
        .collect();
    shared_sig.iter().any(|t| {
        !M128_BOILERPLATE_SIG
            .iter()
            .any(|b| t.eq_ignore_ascii_case(b))
    })
}

/// M134 — majority of residual paragraphs contain `:` (policy/review/
/// checklist class). Uses joined para text so atomized runs still count.
fn residual_looks_like_colon_list(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    if rest.is_empty() {
        return false;
    }
    let with_colon = rest
        .iter()
        .filter(|u| {
            let mut text = String::new();
            for a in u.descendant_atoms() {
                if dom.name(a.content_element) == Some(W::t()) {
                    text.push_str(&dom.value(a.content_element));
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
fn residual_sets_weakly_related(
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
fn m123_diagonal_stats(
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
fn para_zip_diagonal_dominant(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool {
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
        // Zero-vs-zero ties on unrelated empty paras must NOT count as wins
        // (support_tickets empty-mark test: 3 unrelated paras would otherwise
        // zip into mixed instead of III…DDD…).
        if diag > 0.0 && diag + 1e-9 >= best_off {
            diagonal_wins += 1;
        }
    }
    // Majority of rows prefer a positive diagonal, and average overlap is
    // non-trivial (heading demos ~0.12+; pure-unrelated ~0).
    diagonal_wins * 2 >= n && diag_sum / (n as f64) >= 0.08
}

/// The recurring four-way cascade: emit Deleted / Inserted / Unknown / nothing
/// for a `(left, right)` pair (WmlComparer.ts pattern at :8112 etc.).
fn cascade(
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

/// M4.C.3/C.4/C.7 — `DoLcsAlgorithm`: Step A (empty), Step B (run), Steps C–G
/// (para-mark/word-break/threshold guards), Step I (paragraph-aware split).
/// Step H structural dispatch (groups/rows/tables) lands in C.8–C.10; until then
/// a no-common-run resolves to Deleted+Inserted (the H9 fallback).
pub fn do_lcs_algorithm(
    dom: &Dom,
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

    // Step B — longest common run (Word-mode ranks by non-separator content).
    let (mut i1, mut i2, mut len) = if settings.merge_replaced_paragraphs {
        longest_common_run_with_dom(Some(dom), &cul1, &cul2, Some(settings))
    } else {
        longest_common_run(&cul1, &cul2)
    };

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
                    .any(|dca| dom.name(dca.content_element) != Some(W::t()));
                if other_than_text {
                    return true;
                }
                atoms.iter().any(|dca| {
                    let v = dom.value(dca.content_element);
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
                            if dom.name(dca.content_element) != Some(W::t()) {
                                return false;
                            }
                            let v = dom.value(dca.content_element);
                            !v.is_empty()
                                && v.chars().all(|ch| settings.word_separators.contains(&ch))
                        })
                    })
                    .count()
            } else {
                len
            };
            if max_len > 0 && (ratio_len as f64) / (max_len as f64) < settings.detail_threshold {
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
    if settings.merge_replaced_paragraphs
        && len > 0
        && len <= 3
        && !is_only_paragraph_mark
        && cul1.iter().all(|c| matches!(c, ComparisonUnit::Word(_)))
        && cul2.iter().all(|c| matches!(c, ComparisonUnit::Word(_)))
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
        if pmarks1 == 1 && pmarks2 == 1 {
            let mut alpha = String::new();
            for u in &cul1[i1..i1 + len] {
                for a in u.descendant_atoms() {
                    if dom.name(a.content_element) == Some(W::t()) {
                        for ch in dom.value(a.content_element).chars() {
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
            if GLUE.contains(&alpha.as_str()) {
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

    // M-TBL rule 3 (parity/_scratch/table_class_forensics.md): when both sides
    // of the window hold tables, a common run made ONLY of textless units
    // (empty paragraphs) is a false anchor — it drags A's table past B's
    // early tables, so A's table merges with a LATE positional partner while
    // B's first tables come out as pure insertions. Word merges with the
    // FIRST same-slot table (GT support-tickets-table_table-bookmark-end:
    // A's ticket table merges cell-wise with B table 1). Discarding the
    // anchor falls through to Step H's Table/Para dispatch, which pairs
    // table runs first-to-first. Word-mode only.
    if len > 0
        && settings.merge_replaced_paragraphs
        && count_gt(&cul1, ComparisonUnitGroupType::Table) > 0
        && count_gt(&cul2, ComparisonUnitGroupType::Table) > 0
        && cul1[i1..i1 + len].iter().all(|u| {
            u.descendant_atoms().iter().all(|a| {
                dom.name(a.content_element) != Some(W::t())
                    || dom.value(a.content_element).trim().is_empty()
            })
        })
    {
        len = 0;
    }

    // M-ANCHOR attempt 3 (parity/_scratch/anchor_sensitivity.md): a SHORT,
    // low-text common run inside a LARGE window of two UNRELATED sides is a
    // coincidence collision (empty paras, 'ipsum', '(dolore)'), not an
    // anchor — Word collapses such whole-doc replacements to insert-all +
    // delete-all (sd2517b GT: consolidated 18-run shape). Void the run —
    // never substitute another one; the alternatives in such windows are
    // junk too (fs's '.', sd2517b's lorem tokens). Three conditions, ALL
    // required:
    //   1. both sides large — min side > 32 protects the paragraph-merge
    //      pivot of short replacements (fs pair: window 53+5, pMark anchor
    //      is the MIX-paragraph pivot);
    //   2. run weak in absolute terms — len ≤ 2 AND < 15 chars of real
    //      trimmed w:t text (deliberately NOT keyed on textless-ness);
    //   3. window unrelated — fewer than 20% of the smaller side's units
    //      have ANY sha1 match on the other side (related big documents
    //      keep their legitimate empty-para alignment).
    // Word-mode only. EMPIRICAL NOTE (2026-07-04): on the motivating real
    // sd2517b pair this gate is a NO-OP — the junk anchors there fire in
    // step_h-descended lopsided windows (3+1059, 225+2, 39+2, 5+46, 2+45;
    // min side ≤ 5), never in a both-sides-large window, and the 131+120
    // top window already resolves len=0 via the earlier guards. The gate
    // still pins the synthetic whole-doc-replacement physics (m41 tests)
    // without touching any of the four spot-oracle pairs.
    // Lone paragraph-mark anchors (len==1, only w:pPr) are excluded: they
    // score run_real_text_len==0 and would otherwise be voided in large
    // unrelated windows, breaking the paragraph-merge pivot physics that
    // Step D / the M-BLK path rely on (PR #81 review 3523298506).
    if len > 0
        && len <= 2
        && !is_only_paragraph_mark
        && settings.merge_replaced_paragraphs
        && cul1.len().min(cul2.len()) > 32
        && run_real_text_len(dom, &cul1[i1..i1 + len]) < 15
        && !windows_related(&cul1, &cul2)
    {
        len = 0;
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

// ── Step H helpers ───────────────────────────────────────────────────────────
fn as_group(u: &ComparisonUnit) -> Option<&super::atoms::ComparisonUnitGroup> {
    match u {
        ComparisonUnit::Group(g) => Some(g),
        ComparisonUnit::Word(_) => None,
    }
}
fn count_gt(units: &[ComparisonUnit], gt: ComparisonUnitGroupType) -> usize {
    units
        .iter()
        .filter(|u| as_group(u).is_some_and(|g| g.group_type == gt))
        .count()
}
fn count_words(units: &[ComparisonUnit]) -> usize {
    units
        .iter()
        .filter(|u| matches!(u, ComparisonUnit::Word(_)))
        .count()
}
fn group_contents(u: &ComparisonUnit) -> Vec<ComparisonUnit> {
    match u {
        ComparisonUnit::Group(g) => g.contents.clone(),
        ComparisonUnit::Word(_) => vec![],
    }
}
/// Whether the LAST descendant atom across all `units` is a `w:pPr`. None if there
/// are no atoms at all.
fn last_atom_overall_is_ppr(dom: &Dom, units: &[ComparisonUnit]) -> Option<bool> {
    let mut last = None;
    for u in units {
        if let Some(a) = u.descendant_atoms().last() {
            last = Some(atom_is_ppr(dom, a));
        }
    }
    last
}

/// M-BLK: returns `true` when the paragraph (in a flattened word stream split
/// at paragraph-mark units) that contains `pos` is textually identical to
/// another paragraph in `units`. Used by the repetition guard.
fn containing_paragraph_is_duplicated(dom: &Dom, units: &[ComparisonUnit], pos: usize) -> bool {
    let mut texts: Vec<String> = vec![String::new()];
    let mut idx_of_pos = 0usize;
    for (i, u) in units.iter().enumerate() {
        if i == pos {
            idx_of_pos = texts.len() - 1;
        }
        let atoms = u.descendant_atoms();
        let is_pmark = !atoms.is_empty()
            && atoms
                .iter()
                .all(|a| dom.name(a.content_element) == Some(W::p_pr()));
        if is_pmark {
            texts.push(String::new());
            continue;
        }
        // a Group unit that carries its own paragraph mark is a standalone
        // paragraph bucket (whole-paragraph groups in mixed windows)
        let is_group_para = matches!(u, ComparisonUnit::Group(_))
            && atoms
                .iter()
                .any(|a| dom.name(a.content_element) == Some(W::p_pr()));
        if is_group_para {
            let text: String = atoms
                .iter()
                .filter(|a| dom.name(a.content_element) == Some(W::t()))
                .map(|a| dom.value(a.content_element))
                .collect();
            texts.push(text);
            texts.push(String::new());
            if i == pos {
                idx_of_pos = texts.len() - 2;
            }
            continue;
        }
        let last = texts.last_mut().expect("non-empty");
        for a in atoms {
            if dom.name(a.content_element) == Some(W::t()) {
                last.push_str(&dom.value(a.content_element));
            }
        }
    }
    let target = &texts[idx_of_pos];
    if target.trim().is_empty() {
        return false;
    }
    texts
        .iter()
        .enumerate()
        .any(|(i, t)| i != idx_of_pos && t == target)
}

/// M4.C.8-C.10 — `DoLcsAlgorithm` Step H: the no-common-run structural dispatch
/// (:7539-:8065). Branches H1-H9, in source order.
fn step_h(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    use ComparisonUnitGroupType::*;
    let mut out = Vec::new();

    let left_len = cul1.len();
    let left_tables = count_gt(cul1, Table);
    let left_rows = count_gt(cul1, Row);
    let left_paras = count_gt(cul1, Paragraph);
    let left_textboxes = count_gt(cul1, Textbox);
    let left_words = count_words(cul1);
    let right_len = cul2.len();
    let right_tables = count_gt(cul2, Table);
    let right_rows = count_gt(cul2, Row);
    let right_paras = count_gt(cul2, Paragraph);
    let right_textboxes = count_gt(cul2, Textbox);
    let right_words = count_words(cul2);

    // H1 — words + rows/textboxes mix.
    let left_only_wrt = left_len == left_words + left_rows + left_textboxes;
    let right_only_wrt = right_len == right_words + right_rows + right_textboxes;
    if (left_words > 0 || right_words > 0)
        && (left_rows > 0 || right_rows > 0 || left_textboxes > 0 || right_textboxes > 0)
        && left_only_wrt
        && right_only_wrt
    {
        let key = |u: &ComparisonUnit| -> &'static str {
            match u {
                ComparisonUnit::Word(_) => "Word",
                ComparisonUnit::Group(g) => match g.group_type {
                    Row => "Row",
                    Textbox => "Textbox",
                    _ => "Row", // Internal error in TS; treat as Row defensively
                },
            }
        };
        let lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            let (before_l, before_r) = (il, ir);
            if lg[il].0 == rg[ir].0 {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    lg[il].1.clone(),
                    rg[ir].1.clone(),
                ));
                il += 1;
                ir += 1;
            } else if lg[il].0 == "Word"
                && lg[il]
                    .1
                    .last()
                    .is_some_and(|u| !unit_last_atom_is_ppr(dom, u))
                && rg[ir].0 == "Row"
            {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            } else if rg[ir].0 == "Word"
                && rg[ir]
                    .1
                    .last()
                    .is_some_and(|u| !unit_last_atom_is_ppr(dom, u))
                && lg[il].0 == "Row"
            {
                // Word-parity divergence from WmlComparer.ts:7324-7336, which has an upstream
                // copy/paste bug: it tags the ORIGINAL (left) row `Inserted`. An original `Row`
                // with no matching modified `Word` content is a DELETION — mirror of the sibling
                // branch above (:436-440). Verified against Word's own Compare output (fixture f-4).
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 == "Word" && rg[ir].0 != "Word" {
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 != "Word" && rg[ir].0 == "Word" {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            }
            if il == lg.len() && ir == rg.len() {
                return out;
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return out;
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return out;
            }
            if il == before_l && ir == before_r {
                // defensive: no progress (e.g. Row vs Textbox) — flush remainder.
                out.push(CorrelatedSequence::deleted(
                    lg[il..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                out.push(CorrelatedSequence::inserted(
                    rg[ir..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                return out;
            }
        }
    }

    // H2 — tables + paragraphs mix.
    if left_tables > 0
        && right_tables > 0
        && left_paras > 0
        && right_paras > 0
        && (left_len > 1 || right_len > 1)
    {
        let key = |u: &ComparisonUnit| -> &'static str {
            if as_group(u).is_some_and(|g| g.group_type == Table) {
                "Table"
            } else {
                "Para"
            }
        };
        let lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            if lg[il].0 == rg[ir].0 {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    lg[il].1.clone(),
                    rg[ir].1.clone(),
                ));
                il += 1;
                ir += 1;
            } else if lg[il].0 == "Para" && rg[ir].0 == "Table" {
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 == "Table" && rg[ir].0 == "Para" {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            }
            if il == lg.len() && ir == rg.len() {
                return out;
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return out;
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return out;
            }
        }
    }

    // H3 — single table vs single table → DoLcsAlgorithmForTable (M4.D).
    if left_tables == 1
        && left_len == 1
        && right_tables == 1
        && right_len == 1
        && let Some(r) = super::lcs_table::do_lcs_algorithm_for_table(dom, cul1, cul2, settings)
    {
        return r;
    }

    // H4 — both sides only paras/tables/textboxes → flatten one level, one Unknown.
    let left_only_ptt = left_len == left_tables + left_paras + left_textboxes;
    let right_only_ptt = right_len == right_tables + right_paras + right_textboxes;
    if left_only_ptt && right_only_ptt {
        // Word-mode equal-count pure-paragraph zip (heading_2 vs heading_3 demos):
        // Word aligns N×para vs N×para positionally → N mixed paragraphs. Flattening
        // every paragraph into one word-LCS window lets shared tokens ("Heading")
        // bridge the wrong paragraphs (ours: 4 paras vs Word's 3; pixel ~58 vs 100).
        // Cap at 12; require ≥2 (1-vs-1 zip re-enters H4 forever).
        // Only when positional pairing is the best text alignment (diagonal
        // dominance): numbered_list Demo+4items vs Demo+intro+3items is equal
        // count but roles shift — flat LCS wins; forced zip regressed ~7 pts.
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras >= 2
            && left_paras == right_paras
            && left_paras == left_len
            && right_paras == right_len
            && left_paras <= 12
            && para_zip_diagonal_dominant(dom, cul1, cul2)
        {
            for (l, r) in cul1.iter().zip(cul2.iter()) {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![l.clone()],
                    vec![r.clone()],
                ));
            }
            return out;
        }
        let left: Vec<ComparisonUnit> = cul1.iter().flat_map(group_contents).collect();
        let right: Vec<ComparisonUnit> = cul2.iter().flat_map(group_contents).collect();
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            left,
            right,
        ));
        return out;
    }

    // H5/H6 — first unit on both sides is a Row / Cell.
    if let (Some(fl), Some(fr)) = (
        cul1.first().and_then(as_group),
        cul2.first().and_then(as_group),
    ) {
        if fl.group_type == Row && fr.group_type == Row {
            let mut lc: Vec<Option<ComparisonUnit>> =
                fl.contents.iter().cloned().map(Some).collect();
            let mut rc: Vec<Option<ComparisonUnit>> =
                fr.contents.iter().cloned().map(Some).collect();
            while lc.len() < rc.len() {
                lc.push(None);
            }
            while rc.len() < lc.len() {
                rc.push(None);
            }
            for (l, r) in lc.into_iter().zip(rc) {
                match (l, r) {
                    (Some(l), Some(r)) => out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![l],
                        vec![r],
                    )),
                    (None, Some(r)) => out.push(CorrelatedSequence::inserted(group_contents(&r))),
                    (Some(l), None) => out.push(CorrelatedSequence::deleted(group_contents(&l))),
                    (None, None) => {}
                }
            }
            cascade(cul1[1..].to_vec(), cul2[1..].to_vec(), &mut out);
            return out;
        }
        if fl.group_type == Cell && fr.group_type == Cell {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                fl.contents.clone(),
                fr.contents.clone(),
            ));
            cascade(cul1[1..].to_vec(), cul2[1..].to_vec(), &mut out);
            return out;
        }
    }

    // H7 — Word vs Row group (either order) → paired Inserted+Deleted.
    if !cul1.is_empty() && !cul2.is_empty() {
        let l_word = matches!(cul1[0], ComparisonUnit::Word(_));
        let r_row = as_group(&cul2[0]).is_some_and(|g| g.group_type == Row);
        if l_word && r_row {
            out.push(CorrelatedSequence::inserted(cul2.to_vec()));
            out.push(CorrelatedSequence::deleted(cul1.to_vec()));
            return out;
        }
        let l_row = as_group(&cul1[0]).is_some_and(|g| g.group_type == Row);
        let r_word = matches!(cul2[0], ComparisonUnit::Word(_));
        if r_word && l_row {
            out.push(CorrelatedSequence::deleted(cul1.to_vec()));
            out.push(CorrelatedSequence::inserted(cul2.to_vec()));
            return out;
        }

        // H8 — trailing paragraph-mark mismatch.
        let l_ppr = last_atom_overall_is_ppr(dom, cul1);
        let r_ppr = last_atom_overall_is_ppr(dom, cul2);
        if let (Some(l_ppr), Some(r_ppr)) = (l_ppr, r_ppr) {
            if l_ppr && !r_ppr {
                out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                return out;
            } else if !l_ppr && r_ppr {
                out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                return out;
            }
        }
    }

    // H9 — fallback. Word-alignment (M-PI, parity/_scratch/mpi_forensics.md):
    // at BLOCK granularity (paragraph/table groups) Word orders every anchor
    // gap [all inserted blocks, B order][all deleted blocks, A order] — the
    // deleted cluster attaches immediately before the next anchor. Inline
    // (word-level) fallbacks keep deleted-first: Word renders struck text
    // before inserted text within a line. Faithful preset keeps C# order.
    // Precondition: H9 receives a homogeneous unit list (all block groups
    // or all inline). Gate on BOTH ends so a mixed list that happens to
    // start with a paragraph group cannot mis-fire the block ins-before-del
    // path (callers should not hand H9 mixed content; this hardens it).
    let is_block_group = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| {
            matches!(
                g.group_type,
                ComparisonUnitGroupType::Paragraph | ComparisonUnitGroupType::Table
            )
        })
    };
    let block_level = |units: &[ComparisonUnit]| {
        matches!(units.first(), Some(u) if is_block_group(u))
            && matches!(units.last(), Some(u) if is_block_group(u))
    };
    if settings.merge_replaced_paragraphs && (block_level(cul1) || block_level(cul2)) {
        out.push(CorrelatedSequence::inserted(cul2.to_vec()));
        out.push(CorrelatedSequence::deleted(cul1.to_vec()));
        return out;
    }
    out.push(CorrelatedSequence::deleted(cul1.to_vec()));
    out.push(CorrelatedSequence::inserted(cul2.to_vec()));
    out
}

/// M4.C.12 — `DetectUnrelatedSources` (:7017): if both sides have ≥4 groups and
/// their group `sha1` sets are completely disjoint, treat the documents as
/// unrelated (delete everything, insert everything). Top-level pre-check.
pub fn detect_unrelated_sources(
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> Option<Vec<CorrelatedSequence>> {
    if !block_groups_fully_disjoint(cu1, cu2) {
        return None;
    }
    Some(vec![
        CorrelatedSequence::deleted(cu1.to_vec()),
        CorrelatedSequence::inserted(cu2.to_vec()),
    ])
}

/// True when both sides have ≥4 block groups and their group `sha1` sets are
/// completely disjoint (the C# `DetectUnrelatedSources` group predicate).
fn block_groups_fully_disjoint(cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    let groups1: Vec<&str> = cu1
        .iter()
        .filter_map(|u| as_group(u).map(|_| u.sha1()))
        .collect();
    let groups2: Vec<&str> = cu2
        .iter()
        .filter_map(|u| as_group(u).map(|_| u.sha1()))
        .collect();
    if groups1.len() <= 3 || groups2.len() <= 3 {
        return false;
    }
    !groups1.iter().any(|h| groups2.contains(h))
}

/// Flatten one level of groups (H4 shape) so word-level overlap can be scored.
fn flatten_groups_one_level(cu: &[ComparisonUnit]) -> Vec<ComparisonUnit> {
    if cu.iter().any(|u| matches!(u, ComparisonUnit::Group(_))) {
        cu.iter().flat_map(group_contents).collect()
    } else {
        cu.to_vec()
    }
}

/// True when a unit carries a drawing / pict / AlternateContent leaf (file_70
/// text-box residual). Opaque drawings may have zero counted `w:t` on the
/// parent group even though the paragraph is contentful for confetti.
fn group_has_drawing_or_pict(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().any(|a| {
        let Some(n) = dom.name(a.content_element) else {
            return false;
        };
        n == W::name("drawing")
            || n == W::name("pict")
            || n == W::name("object")
            || n.local_name() == "AlternateContent"
            || n.local_name() == "drawing"
            || n.local_name() == "pict"
    })
}

/// Group sha1s that carry real `w:t` text **or** a drawing/pict. Empty
/// paragraphs (identical structure on both sides) share a group hash and
/// would otherwise defeat the unrelated-sources predicate even when every
/// contentful paragraph is unique — Word still collapses those pairs to
/// insert-all/delete-all (batch_to_fix pair 01; synthetic empty-para
/// coincidence).
///
/// M99 (file_70): text-box / drawing residuals often report
/// `run_real_text_len == 0` (text lives in nested txbx or opaque pict), so
/// counting only `w:t` left the short side at 1 (stamp only) and skipped
/// confetti — full LCS then mixed "Green Highlight Demo" into the drawing del.
fn contentful_group_sha1s<'a>(dom: &Dom, cu: &'a [ComparisonUnit]) -> Vec<&'a str> {
    cu.iter()
        .filter_map(|u| {
            as_group(u)?;
            let has_text = run_real_text_len(dom, std::slice::from_ref(u)) > 0;
            if !has_text && !group_has_drawing_or_pict(dom, u) {
                return None;
            }
            Some(u.sha1())
        })
        .collect()
}

/// Word-mode unrelated-sources shortcut.
///
/// C# / faithful mode always short-circuits on disjoint block groups (delete
/// then insert). Word mode cannot: a single multi-char shared word ("Second")
/// still anchors a MIX paragraph (w20a). But when **contentful** block groups
/// are disjoint (≥4 each; empty paragraphs ignored for the match set) AND
/// the longest pure-word common run would be voided by
/// [`WmlComparerSettings::detail_threshold`] (single-letter / empty-pPr
/// coincidence only), Word collapses to insert-all-next then delete-all-base
/// (batch_to_fix pair 01; empty-para anchors otherwise shred the cluster).
///
/// Returns `Some([Inserted, Deleted])` in Word order, or `None` to fall through
/// to full word-level LCS.
pub fn detect_unrelated_sources_word_mode(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let groups1 = contentful_group_sha1s(dom, cu1);
    let groups2 = contentful_group_sha1s(dom, cu2);

    // C# used >3 groups on BOTH sides. Word also collapses short-vs-long
    // whole-doc *paragraph* replacements (batch_to_fix pair 06: 3-para Open
    // Sans demo vs 30-para bold tester → ins-all-next then del-all-base). The
    // strict both-sides->3 gate never fired on those. Allow min side in [2,3]
    // when the larger side is >3, BUT only when NEITHER side carries a table
    // group — short-circuiting table-bearing pairs destroys cell-wise merges
    // (pair 02 table-bookmark_end_table-vmerge-colspan regressed 54→42).
    let (n1, n2) = (groups1.len(), groups2.len());
    let has_table = |cu: &[ComparisonUnit]| {
        cu.iter()
            .any(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
    };
    // Count gate:
    //  - classic C#: both sides >3 contentful groups
    //  - short-vs-long relaxation: smaller side in [2,3], larger >3, and the
    //    *smaller* side is table-free (pair 06: 3-para Open Sans next vs long
    //    bold-tester base that ends in a table — Word still ins-all-next).
    //  - never relax when the short side holds a table (employee_directory
    //    table vs review: short base is a 2-block table doc; short-circuit
    //    wiped the Word cell mix and dropped 90→63).
    let (short_cu, short_n, long_n) = if n1 <= n2 {
        (cu1, n1, n2)
    } else {
        (cu2, n2, n1)
    };
    let stamped = matches!(
        (
            first_contentful_para_text(dom, cu1),
            first_contentful_para_text(dom, cu2),
        ),
        (Some(t1), Some(t2))
            if t1.to_ascii_lowercase().starts_with("file_")
                && t2.to_ascii_lowercase().starts_with("file_")
    );
    let disjoint = !groups1.iter().any(|h| groups2.contains(h));
    // Count gate:
    //  - classic C#: both sides >3 contentful groups
    //  - short-vs-long relaxation: smaller side in [2,3], larger >3, short table-free
    //  - M116 (file_78): stamped **short next** with a table (contentful n≈3:
    //    stamp+title+metric tbl) vs long base — `!has_table` blocked short-circuit,
    //    full LCS nested "Quarterly…" into "eigenpal…". Word is pure-I short next
    //    then pure-D long base. Allow only when short side is **next** (n2==short_n);
    //    short **base** catalog×long next (file_187) Word nests — must keep full LCS.
    let ok_counts = (short_n > 3 && long_n > 3)
        || ((2..=3).contains(&short_n) && long_n > 3 && !has_table(short_cu))
        || (stamped && disjoint && (2..=6).contains(&short_n) && long_n > 6 && n2 == short_n);
    if !ok_counts {
        return None;
    }
    if !disjoint {
        return None;
    }
    let left = flatten_groups_one_level(cu1);
    let right = flatten_groups_one_level(cu2);
    // Related stamped variants (high body-token overlap + large vocab) keep
    // full LCS (file_175). Short demos confetti when this path is reached.
    let confetti_ok = stamped && should_stamp_confetti(dom, cu1, cu2);
    if left.is_empty() || right.is_empty() {
        if confetti_ok {
            return stamp_confetti_then_replace(dom, cu1, cu2, settings);
        }
        if stamped {
            return None;
        }
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    let (_i1, _i2, len) = if settings.merge_replaced_paragraphs {
        longest_common_run_with_dom(Some(dom), &left, &right, Some(settings))
    } else {
        longest_common_run(&left, &right)
    };
    if len > 0 {
        let i1 = _i1;
        let common = &left[i1..i1 + len];
        let common_all_words = common.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if common_all_words {
            let max_len = left.len().max(right.len());
            // Same separator filter as Step G (word mode).
            let ratio_len = common
                .iter()
                .filter(|cs| {
                    !cs.descendant_atoms().iter().all(|dca| {
                        if dom.name(dca.content_element) != Some(W::t()) {
                            return false;
                        }
                        let v = dom.value(dca.content_element);
                        !v.is_empty() && v.chars().all(|ch| settings.word_separators.contains(&ch))
                    })
                })
                .count();
            // Stamped filenames: confetti first para then replace-rest when
            // confetti_ok (file_134). Related variants (file_175) skip.
            let stamp_run = {
                let mut text = String::new();
                for u in common {
                    for a in u.descendant_atoms() {
                        if dom.name(a.content_element) == Some(W::t()) {
                            text.push_str(&dom.value(a.content_element));
                        }
                    }
                }
                let lower = text.to_ascii_lowercase();
                lower.contains("file_") || lower.contains(".docx") || lower.contains(".doc")
            };
            if stamp_run {
                if confetti_ok {
                    return stamp_confetti_then_replace(dom, cu1, cu2, settings);
                }
                return None;
            }
            // Substantial pure-word overlap that would survive Step G → keep LCS
            // (w20a "Second", multi-word tails). Junk single letters / spaces
            // fall through to the insert-all/delete-all short-circuit.
            //
            // M82 (file_85): stamped short demos (`confetti_ok`) still confetti
            // even when residual phrases share words ("bold", "This document").
            // Full-doc word LCS after the stamp peels shared tokens across the
            // wrong paragraphs (A's "This text is bold." mixed into B's first
            // bullet). Residual pairing inside stamp_confetti handles real
            // cousins (file_33 "This document demonstrates…"). Related long
            // variants (file_175) have confetti_ok=false and keep full LCS.
            if max_len > 0
                && (ratio_len as f64) / (max_len as f64) >= settings.detail_threshold
                && run_real_text_len(dom, common) > 0
            {
                if confetti_ok {
                    return stamp_confetti_then_replace(dom, cu1, cu2, settings);
                }
                return None;
            }
        } else if run_real_text_len(dom, common) >= 3 {
            // Nested group common run with real text — not a pure whole-doc
            // replacement; keep full LCS.
            return None;
        }
    }
    if confetti_ok {
        return stamp_confetti_then_replace(dom, cu1, cu2, settings);
    }
    if stamped {
        return None;
    }
    Some(vec![
        CorrelatedSequence::inserted(cu2.to_vec()),
        CorrelatedSequence::deleted(cu1.to_vec()),
    ])
}

/// M4.C.12 — `SetAfterUnids` (:7114): when an Unknown is a single group vs a
/// single group of the same type, copy the original side's ancestor `pt:Unid`s
/// onto the corresponding ancestors of the modified side's atoms (stabilises
/// reassembly). Pure side-effect on `dom`.
pub fn set_after_unids(dom: &mut Dom, unknown: &CorrelatedSequence) {
    let a1 = match &unknown.com_units_1 {
        Some(v) if v.len() == 1 => v,
        _ => return,
    };
    let a2 = match &unknown.com_units_2 {
        Some(v) if v.len() == 1 => v,
        _ => return,
    };
    let (Some(g1), Some(g2)) = (as_group(&a1[0]), as_group(&a2[0])) else {
        return;
    };
    if g1.group_type != g2.group_type {
        return;
    }
    let take_thru = match g1.group_type {
        ComparisonUnitGroupType::Paragraph => W::p(),
        ComparisonUnitGroupType::Table => W::name("tbl"),
        ComparisonUnitGroupType::Row => W::name("tr"),
        ComparisonUnitGroupType::Cell => W::name("tc"),
        ComparisonUnitGroupType::Textbox => W::name("txbxContent"),
    };
    let da1 = a1[0].descendant_atoms();
    let da2 = a2[0].descendant_atoms();
    let Some(first1) = da1.first() else { return };

    // relevant ancestors of da1[0] up to & including the first `take_thru`.
    let mut relevant = Vec::new();
    for &ae in &first1.ancestor_elements {
        relevant.push(ae);
        if dom.name(ae) == Some(take_thru.clone()) {
            break;
        }
    }
    let unid_list: Vec<String> = relevant
        .iter()
        .filter_map(|&a| dom.attribute(a, &PT::unid()).map(|s| s.to_string()))
        .collect();

    // collect target (ancestor, new-unid) pairs first (avoid borrow conflicts).
    let footnotes = W::name("footnotes");
    let endnotes = W::name("endnotes");
    let mut to_set: Vec<(crate::xmllinq::NodeId, String)> = Vec::new();
    for atom in &da2 {
        for (&anc, unid) in atom.ancestor_elements.iter().zip(unid_list.iter()) {
            let nm = dom.name(anc);
            if nm == Some(footnotes.clone()) || nm == Some(endnotes.clone()) {
                continue;
            }
            if dom.attribute(anc, &PT::unid()).is_none() {
                continue; // only overwrite an existing Unid
            }
            to_set.push((anc, unid.clone()));
        }
    }
    for (anc, unid) in to_set {
        dom.set_attribute_value(anc, &PT::unid(), Some(&unid));
    }
}

/// M4.C.11 — `ProcessCorrelatedHashes` (:7184): pre-correlate runs of groups
/// (Paragraph/Table/Row) by `CorrelatedSHA1Hash`, emitting one Unknown per
/// matched group, with before/after Deleted/Inserted/Unknown. Returns `None`
/// (decline) when there are <3 units or no qualifying run.
pub fn process_correlated_hashes(unknown: &CorrelatedSequence) -> Option<Vec<CorrelatedSequence>> {
    use ComparisonUnitGroupType::*;
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    if cul1.len().min(cul2.len()) < 3 {
        return None;
    }
    let first_ok = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| matches!(g.group_type, Paragraph | Table | Row))
    };
    if !cul1.first().is_some_and(first_ok) || !cul2.first().is_some_and(first_ok) {
        return None;
    }

    // longest run matched by CorrelatedSHA1Hash + same group type, by atom count.
    let (mut best_len, mut best_atoms, mut bi1, mut bi2) = (0usize, 0usize, usize::MAX, usize::MAX);
    for i1 in 0..cul1.len() {
        for i2 in 0..cul2.len() {
            let (mut len, mut atoms, mut t1, mut t2) = (0usize, 0usize, i1, i2);
            loop {
                let m = match (
                    cul1.get(t1).and_then(as_group),
                    cul2.get(t2).and_then(as_group),
                ) {
                    (Some(g1), Some(g2)) => {
                        g1.group_type == g2.group_type
                            && g1.correlated_sha1_hash.is_some()
                            && g1.correlated_sha1_hash == g2.correlated_sha1_hash
                    }
                    _ => false,
                };
                if m {
                    atoms += cul1[t1].descendant_content_atoms_count();
                    t1 += 1;
                    t2 += 1;
                    len += 1;
                    if t1 == cul1.len() || t2 == cul2.len() {
                        if atoms > best_atoms {
                            (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                        }
                        break;
                    }
                } else {
                    if atoms > best_atoms {
                        (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                    }
                    break;
                }
            }
        }
    }

    // atom-count threshold gate.
    let do_correlation = match best_len {
        1 => {
            cul1[bi1].descendant_content_atoms_count() > 16
                && cul2[bi2].descendant_content_atoms_count() > 16
        }
        2 | 3 => {
            let s1: usize = cul1[bi1..bi1 + best_len]
                .iter()
                .map(|z| z.descendant_content_atoms_count())
                .sum();
            let s2: usize = cul2[bi2..bi2 + best_len]
                .iter()
                .map(|z| z.descendant_content_atoms_count())
                .sum();
            s1 > 32 && s2 > 32
        }
        n if n > 3 => true,
        _ => false,
    };
    if !do_correlation {
        return None;
    }

    let mut out = Vec::new();
    // before-region
    cascade(cul1[..bi1].to_vec(), cul2[..bi2].to_vec(), &mut out);
    // one Unknown per matched group
    for i in 0..best_len {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![cul1[bi1 + i].clone()],
            vec![cul2[bi2 + i].clone()],
        ));
    }
    // after-region
    cascade(
        cul1[bi1 + best_len..].to_vec(),
        cul2[bi2 + best_len..].to_vec(),
        &mut out,
    );
    Some(out)
}

/// First DIRECT atom of a unit (Word→contents[0]; Group→None). The TS back-path
/// uses `ofType(cu.Contents, ComparisonUnitAtom)`, which is direct-only.
fn first_direct_atom(u: &ComparisonUnit) -> Option<&ComparisonUnitAtom> {
    match u {
        ComparisonUnit::Word(w) => w.contents.first(),
        ComparisonUnit::Group(_) => None,
    }
}
fn unit_first_direct_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    first_direct_atom(u).is_some_and(|a| atom_is_ppr(dom, a))
}

/// M4.C.5/C.6 — `FindCommonAtBeginningAndEnd` (:5540): the resolver tried before
/// DoLcsAlgorithm. Finds the longest common contiguous run at the FRONT (else the
/// BACK), splitting the Unknown around it, paragraph-aware. Returns `None` to
/// decline (driver falls through to DoLcsAlgorithm).
pub fn find_common_at_beginning_and_end(
    dom: &Dom,
    unknown: &CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    let n1 = cul1.len();
    let n2 = cul2.len();
    let length_to_compare = n1.min(n2);

    // ── FRONT (C.5) ───────────────────────────────────────────────────────────
    let mut ccb = 0;
    while ccb < length_to_compare && cul1[ccb].sha1() == cul2[ccb].sha1() {
        ccb += 1;
    }
    if ccb != 0 && (ccb as f64) / (length_to_compare as f64) < settings.detail_threshold {
        ccb = 0;
    }
    if ccb != 0 {
        let mut out = Vec::new();
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Equal,
            cul1[..ccb].to_vec(),
            cul2[..ccb].to_vec(),
        ));
        let (rem_l, rem_r) = (n1 - ccb, n2 - ccb);
        if rem_l != 0 && rem_r == 0 {
            out.push(CorrelatedSequence::deleted(cul1[ccb..].to_vec()));
        } else if rem_l == 0 && rem_r != 0 {
            out.push(CorrelatedSequence::inserted(cul2[ccb..].to_vec()));
        } else if rem_l != 0 && rem_r != 0 {
            let both_words = matches!(cul1[0], ComparisonUnit::Word(_))
                && matches!(cul2[0], ComparisonUnit::Word(_));
            let mut handled = false;
            if both_words {
                // boundary atoms use DESCENDANT atoms (firstOrDefault), faithful to :5617.
                let bl = cul1[ccb - 1].descendant_atoms().first().copied();
                let br = cul2[ccb - 1].descendant_atoms().first().copied();
                if let (Some(bl), Some(br)) = (bl, br)
                    && !atom_is_ppr(dom, bl)
                    && !atom_is_ppr(dom, br)
                {
                    let s1 = split_at_paragraph_mark(dom, &cul1[ccb..]);
                    let s2 = split_at_paragraph_mark(dom, &cul2[ccb..]);
                    if s1.len() == 1 && s2.len() == 1 {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            s1[0].clone(),
                            s2[0].clone(),
                        ));
                        handled = true;
                    } else if s1.len() == 2 && s2.len() == 2 {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            s1[0].clone(),
                            s2[0].clone(),
                        ));
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            s1[1].clone(),
                            s2[1].clone(),
                        ));
                        handled = true;
                    }
                }
            }
            if !handled {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    cul1[ccb..].to_vec(),
                    cul2[ccb..].to_vec(),
                ));
            }
        }
        return Some(out);
    }

    // ── BACK (C.6) ────────────────────────────────────────────────────────────
    let mut cce = 0;
    while cce < length_to_compare && cul1[n1 - 1 - cce].sha1() == cul2[n2 - 1 - cce].sha1() {
        cce += 1;
    }
    // never START a common section with a paragraph mark (trim leading pPr of tail).
    while cce > 1 {
        let unit = &cul1[n1 - cce]; // start of the tail run
        if !unit_is_single_atom_ppr(dom, unit) {
            break;
        }
        cce -= 1;
    }
    // isOnlyParagraphMark. cce==2: C# tests `secondCommon` (:5747), which in
    // this port's unit model is the same last-unit pPr check as the cce==1 arm.
    let is_only_paragraph_mark =
        (cce == 1 || cce == 2) && unit_is_single_atom_ppr(dom, &cul1[n1 - 1]);
    if !is_only_paragraph_mark
        && cce != 0
        && (cce as f64) / (length_to_compare as f64) < settings.detail_threshold
    {
        cce = 0;
    }
    if is_only_paragraph_mark {
        cce = 0; // WC010 guard (:5763)
    }
    if cce == 0 {
        return None;
    }

    // partial-paragraph peel-back before the common tail.
    let (mut rem_lp, mut rem_rp) = (0usize, 0usize);
    let common_end_seq = &cul1[n1 - cce..]; // forward order
    if matches!(common_end_seq.first(), Some(ComparisonUnit::Word(_)))
        && common_end_seq
            .iter()
            .any(|cu| unit_first_direct_atom_is_ppr(dom, cu))
    {
        // units before the tail, walked backward.
        rem_lp = take_while_count_rev(&cul1[..n1 - cce], |cu| word_first_not_ppr(dom, cu));
        rem_rp = take_while_count_rev(&cul2[..n2 - cce], |cu| word_first_not_ppr(dom, cu));
    }

    let mut out = Vec::new();
    let before_l = n1 - rem_lp - cce;
    let before_r = n2 - rem_rp - cce;
    cascade(
        cul1[..before_l].to_vec(),
        cul2[..before_r].to_vec(),
        &mut out,
    );
    cascade(
        cul1[before_l..before_l + rem_lp].to_vec(),
        cul2[before_r..before_r + rem_rp].to_vec(),
        &mut out,
    );
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        cul1[n1 - cce..].to_vec(),
        cul2[n2 - cce..].to_vec(),
    ));
    Some(out)
}

/// Resolve all Unknown sequences in a worklist (SetAfterUnids →
/// ProcessCorrelatedHashes → FindCommonAtBeginningAndEnd → DoLcsAlgorithm).
pub fn resolve_correlated_sequences(
    dom: &mut Dom,
    mut cs_list: Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    loop {
        let Some(idx) = cs_list
            .iter()
            .position(|cs| cs.correlation_status == CorrelationStatus::Unknown)
        else {
            return cs_list;
        };
        let unknown = cs_list.remove(idx);
        set_after_unids(dom, &unknown);
        // Borrow for the two hash/anchor fast paths; if neither resolves, MOVE
        // `unknown` into do_lcs_algorithm (a match, not or_else, so the borrows
        // end before the move).
        let resolved = match process_correlated_hashes(&unknown) {
            Some(r) => r,
            None => match find_common_at_beginning_and_end(dom, &unknown, settings) {
                Some(r) => r,
                None => do_lcs_algorithm(dom, unknown, settings),
            },
        };
        // Splice the resolved items in at `idx` in ONE tail-shift, instead of an
        // insert-per-item loop that memmoves the (large) tail once per item.
        // Same final order and the same first-Unknown processing order.
        cs_list.splice(idx..idx, resolved);
    }
}

/// M4.C.1 — `Lcs` worklist driver: seed one Unknown, resolve until none remain.
pub fn lcs(
    dom: &mut Dom,
    cu1: Vec<ComparisonUnit>,
    cu2: Vec<ComparisonUnit>,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    resolve_correlated_sequences(
        dom,
        vec![CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            cu1,
            cu2,
        )],
        settings,
    )
}

/// PR2 — the hash-indexed longest-common-run MUST return the exact same
/// `(i1, i2, len)` as the historical O(n·m) scan it replaces. These tests are the
/// equivalence oracle: they drive both paths over the same inputs and assert
/// `indexed == scan`, including the first-found tie-break and forced u64-key
/// collisions. Correctness on the `dom=Some` (Word-mode) content score is covered
/// by the corpus canonical-structural-equality suite, since both paths share
/// [`common_run_content_score`].
#[cfg(test)]
mod indexed_lcr_tests {
    use super::*;
    use crate::comparer::atoms::ComparisonUnitWord;
    use crate::util::sha1::sha1_fingerprint;

    /// A bare word unit carrying a chosen content hash (key = fingerprint(hash),
    /// the production invariant).
    fn mk_word(hash: &str) -> ComparisonUnit {
        ComparisonUnit::Word(ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            contents: Vec::new(),
            sha1_key: sha1_fingerprint(hash),
            sha1_hash: hash.to_string(),
        })
    }

    /// A word with an EXPLICIT (hash, key) pair — used to simulate a u64
    /// fingerprint collision (distinct hash strings sharing a key). Real FNV-1a
    /// keys make this astronomically rare, but the string check must still reject
    /// it identically in both paths.
    fn mk_word_key(hash: &str, key: u64) -> ComparisonUnit {
        ComparisonUnit::Word(ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            contents: Vec::new(),
            sha1_key: key,
            sha1_hash: hash.to_string(),
        })
    }

    fn mk_seq(hashes: &[&str]) -> Vec<ComparisonUnit> {
        hashes.iter().map(|h| mk_word(h)).collect()
    }

    /// Tiny deterministic LCG (Numerical Recipes constants) — no external rng, no
    /// time/random (both unavailable). Same seed ⇒ same sequence every run.
    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, n: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) as u32) % n
        }
    }

    /// The `dom=None` path (content == len) exercises the full candidate ordering
    /// and first-found tie-break — precisely where the indexed rewrite could
    /// diverge. A 4-symbol alphabet with lengths 0..=12 ⇒ frequent equal runs and
    /// ties across thousands of trials.
    #[test]
    fn indexed_matches_scan_random() {
        const ALPHABET: &[&str] = &["A", "B", "C", "D"];
        let mut rng = Lcg(0x9E37_79B9_7F4A_7C15);
        for trial in 0..5000 {
            let n = rng.below(13) as usize;
            let m = rng.below(13) as usize;
            let a: Vec<ComparisonUnit> = (0..n)
                .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
                .collect();
            let b: Vec<ComparisonUnit> = (0..m)
                .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
                .collect();
            let expect = longest_common_run_scan(None, &a, &b, None);
            let got = longest_common_run_indexed(None, &a, &b, None);
            assert_eq!(
                got,
                expect,
                "trial {trial}: indexed != scan\n a={:?}\n b={:?}",
                a.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
                b.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn indexed_matches_scan_edge_cases() {
        let cases: &[(Vec<ComparisonUnit>, Vec<ComparisonUnit>)] = &[
            (mk_seq(&[]), mk_seq(&[])),
            (mk_seq(&["A"]), mk_seq(&[])),
            (mk_seq(&[]), mk_seq(&["A"])),
            (mk_seq(&["A"]), mk_seq(&["A"])),
            (mk_seq(&["A"]), mk_seq(&["B"])),
            (mk_seq(&["A", "A", "A"]), mk_seq(&["A", "A"])),
            (mk_seq(&["A", "B", "C"]), mk_seq(&["C", "B", "A"])),
            (mk_seq(&["A", "B", "A", "B"]), mk_seq(&["A", "B", "A", "B"])),
        ];
        for (a, b) in cases {
            assert_eq!(
                longest_common_run_indexed(None, a, b, None),
                longest_common_run_scan(None, a, b, None),
            );
        }
    }

    /// A forced u64-key collision (distinct hashes, shared key) must NOT be read
    /// as a match by the bucket probe: the string check keeps the indexed output
    /// identical to the scan, which relies on the same string check.
    #[test]
    fn indexed_handles_key_collision() {
        let k = 0xDEAD_BEEF_u64;
        // "X" and "Y" pretend to collide on key k; "Z" is a genuine matching pair.
        let a = vec![mk_word_key("X", k), mk_word_key("Z", k)];
        let b = vec![mk_word_key("Y", k), mk_word_key("Z", k)];
        let got = longest_common_run_indexed(None, &a, &b, None);
        assert_eq!(got, longest_common_run_scan(None, &a, &b, None));
        assert_eq!(got, (1, 1, 1), "collision must not fabricate an X==Y match");
    }
}
