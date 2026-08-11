// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.G — move detection. Port of DetectMovesInAtomList (:4711),
//! GroupConsecutiveAtomsByStatus (:4776), ExtractTextFromAtomBlock (:4805),
//! CalculateJaccardSimilarity (:4664), TokenizeForComparison/CountWords/splitByChars.
//! NOTE: settings.detect_moves defaults to FALSE.
//!
//! Word-visual extensions (M117/M118):
//! - split consecutive del/ins runs on `w:pPr` so each paragraph matches independently
//! - collapse whitespace for Jaccard (stamp re-spacing ≠ move)
//! - M117: length ratio ≥0.90 and thr ≥0.97
//! - M118: thrash-drop expansions (pending≥12, near_exact≥8, size_ratio≥2)

use std::collections::HashSet;

use crate::namespaces::W;
use crate::xmllinq::Dom;

use super::atoms::{AtomBlock, ComparisonUnitAtom};
use super::{CorrelationStatus, WmlComparerSettings};

/// `splitByChars` (PtUtil) — split at any separator char; keeps empty segments
/// (leading/trailing/adjacent) and always pushes a trailing segment.
pub fn split_by_chars(text: &str, seps: &[char]) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if seps.contains(&ch) {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(ch);
        }
    }
    out.push(cur);
    out
}

/// `TokenizeForComparison` — unique non-empty words (uppercased if case-insensitive).
pub fn tokenize(text: &str, settings: &WmlComparerSettings) -> HashSet<String> {
    split_by_chars(text, &settings.word_separators)
        .into_iter()
        .filter(|s| !s.is_empty())
        .map(|s| {
            if settings.case_insensitive {
                s.to_uppercase()
            } else {
                s
            }
        })
        .collect()
}

/// `CountWords` — non-empty word count (NO dedup).
pub fn count_words(text: &str, settings: &WmlComparerSettings) -> usize {
    split_by_chars(text, &settings.word_separators)
        .into_iter()
        .filter(|s| !s.is_empty())
        .count()
}

/// Collapse runs of whitespace so reformatted charter paragraphs (dense vs
/// spaced) compare as the same text for move detection.
fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `CalculateJaccardSimilarity` — |∩| / |∪| over unique word sets.
/// Whitespace is collapsed first so pure re-spacing is not a "move".
pub fn jaccard(text1: &str, text2: &str, settings: &WmlComparerSettings) -> f64 {
    let t1 = collapse_ws(text1);
    let t2 = collapse_ws(text2);
    if t1.is_empty() || t2.is_empty() {
        return 0.0;
    }
    let w1 = tokenize(&t1, settings);
    let w2 = tokenize(&t2, settings);
    if w1.is_empty() || w2.is_empty() {
        return 0.0;
    }
    let inter = w1.intersection(&w2).count();
    let union = w1.union(&w2).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

/// `GroupConsecutiveAtomsByStatus` — maximal runs of atoms with `status`.
pub fn group_consecutive_atoms_by_status(
    atoms: &[ComparisonUnitAtom],
    status: CorrelationStatus,
) -> Vec<AtomBlock> {
    let mut blocks = Vec::new();
    let mut cur: Option<AtomBlock> = None;
    for (i, a) in atoms.iter().enumerate() {
        if a.correlation_status == status {
            match &mut cur {
                Some(b) => b.atoms.push(i),
                None => {
                    cur = Some(AtomBlock {
                        atoms: vec![i],
                        start_index: i,
                    });
                }
            }
        } else if let Some(b) = cur.take() {
            blocks.push(b);
        }
    }
    if let Some(b) = cur {
        blocks.push(b);
    }
    blocks
}

/// `ExtractTextFromAtomBlock` — concat atom text (pPr → "\n"), no separator.
pub fn extract_text_from_atom_block(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    block: &AtomBlock,
) -> String {
    let mut s = String::new();
    for &i in &block.atoms {
        let ce = atoms[i].content_element;
        if dom.name(ce) == Some(W::p_pr()) {
            s.push('\n');
        } else {
            // ATOM-TEXT-01: borrow single-text-child leaves (no intermediate String).
            s.push_str(&dom.value_str(ce));
        }
    }
    s
}

/// Split a consecutive status run on paragraph marks (`w:pPr` atoms) so Word-like
/// multi-paragraph deletions/insertions can match **per paragraph**.
///
/// PowerTools matched whole consecutive runs; Word Compare pairs relocated
/// paragraphs (broken_ones_two `file_8_file_9`: one LCS del run of many paras
/// vs many moveFrom/moveTo). Without this split, Jaccard on mega-blocks stays
/// ~0.3 and no moves are emitted.
pub fn split_block_on_paragraphs(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    block: &AtomBlock,
) -> Vec<AtomBlock> {
    let mut out = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut start = block.start_index;
    for &i in &block.atoms {
        let is_ppr = dom.name(atoms[i].content_element) == Some(W::p_pr());
        if is_ppr {
            // pPr closes the current paragraph unit (include the mark).
            if cur.is_empty() {
                start = i;
            }
            cur.push(i);
            out.push(AtomBlock {
                atoms: std::mem::take(&mut cur),
                start_index: start,
            });
            start = i + 1;
        } else {
            if cur.is_empty() {
                start = i;
            }
            cur.push(i);
        }
    }
    if !cur.is_empty() {
        out.push(AtomBlock {
            atoms: cur,
            start_index: start,
        });
    }
    // If the run had no pPr, keep the original block as one unit.
    if out.is_empty() {
        out.push(block.clone());
    }
    out
}

/// When B skips ahead of pure A-only deletes (LCS: `Equal → Deleted+ → Equal+`
/// with no inserts in the gap), Word often shows the matched continuation as a
/// **move** to the early (B) position: `moveTo` before the gap, `moveFrom` after.
/// Leaving equals at A's late position keeps gap deletes in the middle and
/// shifts later pages (docx_lots_of_comments_addition_removal_redline × clean:
/// Capability matrix lands pages later than Word).
///
/// Rewrite `… Equal, Deleted(gap), Equal+(matched) …` into
/// `… Equal, Inserted(matched₂), Deleted(gap), Deleted(matched₁) …` so
/// [`detect_moves_in_atom_list`] can retag the matched pair as a move.
///
/// The matched run is the consecutive Equals after the gap, **stopping after
/// the first table** (or after 3 equals) so a heading alone is never moved
/// without its following table body.
pub fn promote_skip_ahead_equals(
    seqs: &mut Vec<super::atoms::CorrelatedSequence>,
    settings: &WmlComparerSettings,
) {
    use super::ComparisonUnitGroupType;
    use super::atoms::{ComparisonUnit, CorrelatedSequence};
    if !settings.merge_replaced_paragraphs || !settings.detect_moves {
        return;
    }
    let unit_is_table = |u: &ComparisonUnit| match u {
        ComparisonUnit::Group(g) => g.group_type == ComparisonUnitGroupType::Table,
        _ => false,
    };
    let seq_has_table = |s: &CorrelatedSequence| {
        s.com_units_1
            .as_ref()
            .into_iter()
            .flatten()
            .any(unit_is_table)
            || s.com_units_2
                .as_ref()
                .into_iter()
                .flatten()
                .any(unit_is_table)
    };

    let mut out: Vec<CorrelatedSequence> = Vec::with_capacity(seqs.len().saturating_mul(2));
    let mut i = 0usize;
    while i < seqs.len() {
        if seqs[i].correlation_status == CorrelationStatus::Deleted {
            let del_start = i;
            while i < seqs.len() && seqs[i].correlation_status == CorrelationStatus::Deleted {
                i += 1;
            }
            let gap = &seqs[del_start..i];
            let gap_units: usize = gap
                .iter()
                .map(|s| s.com_units_1.as_ref().map(|u| u.len()).unwrap_or(0))
                .sum();
            let prev_ok = out.last().is_none_or(|s| {
                s.correlation_status == CorrelationStatus::Equal
                    || s.correlation_status == CorrelationStatus::Inserted
            });
            // Collect a short Equal run after the gap (through first table).
            if gap_units >= 1 && prev_ok && i < seqs.len() {
                let eq_start = i;
                let mut eq_end = i;
                let mut saw_table = false;
                let mut n_eq = 0usize;
                while eq_end < seqs.len()
                    && seqs[eq_end].correlation_status == CorrelationStatus::Equal
                    && n_eq < 3
                {
                    if seqs[eq_end]
                        .com_units_1
                        .as_ref()
                        .map(|u| u.is_empty())
                        .unwrap_or(true)
                        || seqs[eq_end]
                            .com_units_2
                            .as_ref()
                            .map(|u| u.is_empty())
                            .unwrap_or(true)
                    {
                        break;
                    }
                    n_eq += 1;
                    if seq_has_table(&seqs[eq_end]) {
                        saw_table = true;
                        eq_end += 1;
                        break;
                    }
                    eq_end += 1;
                }
                // Only promote when the run includes a table (heading-only moves
                // orphan the table and regress page layout).
                if saw_table && eq_end > eq_start {
                    let mut u1_all = Vec::new();
                    let mut u2_all = Vec::new();
                    for s in &seqs[eq_start..eq_end] {
                        if let Some(u) = &s.com_units_1 {
                            u1_all.extend(u.iter().cloned());
                        }
                        if let Some(u) = &s.com_units_2 {
                            u2_all.extend(u.iter().cloned());
                        }
                    }
                    if !u1_all.is_empty() && !u2_all.is_empty() {
                        out.push(CorrelatedSequence::inserted(u2_all));
                        for g in gap {
                            out.push(g.clone());
                        }
                        out.push(CorrelatedSequence::deleted(u1_all));
                        i = eq_end;
                        continue;
                    }
                }
            }
            for g in &seqs[del_start..i] {
                out.push(g.clone());
            }
            continue;
        }
        out.push(seqs[i].clone());
        i += 1;
    }
    *seqs = out;
}

/// M4.G.3 — `DetectMovesInAtomList` (:4711): greedy match deleted↔inserted blocks
/// by Jaccard ≥ threshold (min word count), retag MovedSource/MovedDestination.
///
/// Word-visual extensions:
/// - after consecutive-status grouping, **split on `w:pPr`** (paragraph units)
/// - M117: length ratio ≥0.90 and thr = max(settings, 0.97)
/// - M118: drop all pending when expansion thrash (pending≥12, near_exact≥8, size_ratio≥2)
///
/// Dispatches to [`detect_moves_memoized`], which precomputes each block's text /
/// word count / Jaccard token-set ONCE instead of re-extracting and re-tokenizing
/// on every (deleted × inserted) pair. Its retagging is identical to the historical
/// [`detect_moves_reference`], proven by `memoized_matches_reference`.
pub fn detect_moves_in_atom_list(
    dom: &Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) {
    detect_moves_memoized(dom, atoms, settings);
}

/// Per-block precomputed text features, shared by every pair comparison in
/// [`detect_moves_memoized`] (computed once per block instead of O(del × ins)).
struct BlockText {
    /// Raw concatenated block text (`extract_text_from_atom_block`).
    text: String,
    /// Whitespace-collapsed text — the empty-check operand inside [`jaccard`].
    collapsed: String,
    /// `count_words(collapse_ws(text))` — the min-word / length-ratio gate.
    words: usize,
    /// `tokenize(collapse_ws(text))` — the Jaccard word set.
    tokens: HashSet<String>,
    /// `text.chars().count()` — the M118 size-ratio operand.
    chars: usize,
}

fn precompute_block(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    block: &AtomBlock,
    settings: &WmlComparerSettings,
) -> BlockText {
    let text = extract_text_from_atom_block(dom, atoms, block);
    let collapsed = collapse_ws(&text);
    let words = count_words(&collapsed, settings);
    let tokens = tokenize(&collapsed, settings);
    let chars = text.chars().count();
    BlockText {
        text,
        collapsed,
        words,
        tokens,
        chars,
    }
}

/// Jaccard from precomputed features — exactly [`jaccard`]'s body with the
/// collapse+tokenize already done. Identical empty-checks and float result:
/// `jaccard(t1,t2)` collapses both, tokenizes both, then `|∩| / |∪|`; here the
/// collapsed strings and token sets are supplied, so every intermediate matches.
fn jaccard_precomputed(a: &BlockText, b: &BlockText) -> f64 {
    if a.collapsed.is_empty() || b.collapsed.is_empty() {
        return 0.0;
    }
    if a.tokens.is_empty() || b.tokens.is_empty() {
        return 0.0;
    }
    let inter = a.tokens.intersection(&b.tokens).count();
    let union = a.tokens.union(&b.tokens).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

/// Memoized `detect_moves` — the asymptotic fix for the fixture-B hotspot.
///
/// Structurally identical to [`detect_moves_reference`], except each block's text
/// features (text / collapsed / word count / Jaccard tokens / char count) are
/// computed **once** into `del_info`/`ins_info` and reused, instead of
/// re-extracting and re-tokenizing every inserted block on every deleted block
/// (O(del × ins) → O(del + ins) extractions). `atoms` is not mutated until after
/// all comparisons, so the precomputed features are stable and every intermediate
/// value — `dw`, `iw`, `sim`, `del_chars`, `ins_chars`, iteration order, the
/// greedy `matched` set, `pending`, the M118 decision, and the final retag — is
/// bit-for-bit what the reference computes. Proven by `memoized_matches_reference`.
fn detect_moves_memoized(
    dom: &Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) {
    if !settings.detect_moves {
        return;
    }
    let deleted_runs = group_consecutive_atoms_by_status(atoms, CorrelationStatus::Deleted);
    let inserted_runs = group_consecutive_atoms_by_status(atoms, CorrelationStatus::Inserted);
    if deleted_runs.is_empty() || inserted_runs.is_empty() {
        return;
    }

    let deleted: Vec<AtomBlock> = deleted_runs
        .iter()
        .flat_map(|b| split_block_on_paragraphs(dom, atoms, b))
        .collect();
    let inserted: Vec<AtomBlock> = inserted_runs
        .iter()
        .flat_map(|b| split_block_on_paragraphs(dom, atoms, b))
        .collect();

    // Precompute per-block text features ONCE (kills the O(del × ins) re-extract).
    let del_info: Vec<BlockText> = deleted
        .iter()
        .map(|b| precompute_block(dom, atoms, b, settings))
        .collect();
    let ins_info: Vec<BlockText> = inserted
        .iter()
        .map(|b| precompute_block(dom, atoms, b, settings))
        .collect();

    let debug = std::env::var_os("OOXMLSDK_DEBUG_MOVES").is_some();
    if debug {
        eprintln!(
            "[moves] del_runs={} ins_runs={} del_paras={} ins_paras={} min_words={} thr={}",
            deleted_runs.len(),
            inserted_runs.len(),
            deleted.len(),
            inserted.len(),
            settings.move_minimum_word_count,
            settings.move_similarity_threshold
        );
    }
    let mut next_group_id = 1u32;
    let mut matched: HashSet<usize> = HashSet::new();
    let mut skipped_short_del = 0usize;
    let mut considered = 0usize;
    let mut best_overall = 0.0f64;
    let mut pending: Vec<(usize, usize, f64)> = Vec::new();
    for (di, dinfo) in del_info.iter().enumerate() {
        let dw = dinfo.words;
        if dw < settings.move_minimum_word_count {
            skipped_short_del += 1;
            continue;
        }
        considered += 1;
        let mut best: Option<usize> = None;
        let mut best_sim = 0.0f64;
        for (ii, iinfo) in ins_info.iter().enumerate() {
            if matched.contains(&ii) {
                continue;
            }
            let iw = iinfo.words;
            if iw < settings.move_minimum_word_count {
                continue;
            }
            // M117: length ratio 0.90 (expansions thrash short×long).
            let longer = dw.max(iw);
            let shorter = dw.min(iw);
            if longer > 0 && (shorter as f64) / (longer as f64) < 0.90 {
                continue;
            }
            let sim = jaccard_precomputed(dinfo, iinfo);
            if sim > best_overall {
                best_overall = sim;
            }
            // M117: thr ≥0.97 for all pairs.
            let thr = settings.move_similarity_threshold.max(0.97);
            if sim >= thr && sim > best_sim {
                best_sim = sim;
                best = Some(ii);
            }
        }
        if let Some(ii) = best {
            pending.push((di, ii, best_sim));
            matched.insert(ii);
        } else if debug && considered <= 5 {
            eprintln!(
                "[moves] unmatched del words={} chars={} preview={:?}",
                dw,
                dinfo.text.len(),
                dinfo.text.chars().take(80).collect::<String>()
            );
        }
    }
    // M118: expansion thrash — drop all pending when many near-exact matches +
    // size-skewed del/ins text.
    let near_exact = pending.iter().filter(|(_, _, s)| *s >= 0.999).count();
    let del_chars: usize = del_info.iter().map(|b| b.chars).sum();
    let ins_chars: usize = ins_info.iter().map(|b| b.chars).sum();
    let size_shorter = del_chars.min(ins_chars).max(1);
    let size_longer = del_chars.max(ins_chars);
    let size_ratio = size_longer as f64 / size_shorter as f64;
    if debug {
        eprintln!(
            "[moves] pending={} near_exact={near_exact} del_chars={del_chars} ins_chars={ins_chars} size_ratio={size_ratio:.2}",
            pending.len()
        );
    }
    if pending.len() >= 12 && near_exact >= 8 && size_ratio >= 2.0 {
        if debug {
            eprintln!(
                "[moves] thrash drop (expansion): pending={} near_exact={near_exact} size_ratio={size_ratio:.2}",
                pending.len()
            );
        }
        pending.clear();
    }
    for (di, ii, _sim) in pending {
        let id = next_group_id;
        let name = format!("move{id}");
        for &ai in &deleted[di].atoms {
            atoms[ai].correlation_status = CorrelationStatus::MovedSource;
            atoms[ai].move_group_id = Some(id);
            atoms[ai].move_name = Some(name.clone());
        }
        for &ai in &inserted[ii].atoms {
            atoms[ai].correlation_status = CorrelationStatus::MovedDestination;
            atoms[ai].move_group_id = Some(id);
            atoms[ai].move_name = Some(name.clone());
        }
        next_group_id += 1;
    }
    if debug {
        eprintln!(
            "[moves] matched={} considered_del={} skipped_short_del={} best_jaccard={:.3}",
            next_group_id - 1,
            considered,
            skipped_short_del,
            best_overall
        );
    }
}

/// Historical reference: O(del × ins) with per-pair text re-extraction and
/// re-tokenization. Kept as the equivalence oracle for [`detect_moves_memoized`]
/// (`memoized_matches_reference`); not compiled into release builds now that
/// production dispatches to the memoized path (PR3 Phase D).
#[cfg(test)]
fn detect_moves_reference(
    dom: &Dom,
    atoms: &mut [ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) {
    if !settings.detect_moves {
        return;
    }
    let deleted_runs = group_consecutive_atoms_by_status(atoms, CorrelationStatus::Deleted);
    let inserted_runs = group_consecutive_atoms_by_status(atoms, CorrelationStatus::Inserted);
    if deleted_runs.is_empty() || inserted_runs.is_empty() {
        return;
    }

    // Paragraph-split candidates (Word-like). Keep run-level blocks as a
    // fallback first element when a run is a single paragraph already.
    let deleted: Vec<AtomBlock> = deleted_runs
        .iter()
        .flat_map(|b| split_block_on_paragraphs(dom, atoms, b))
        .collect();
    let inserted: Vec<AtomBlock> = inserted_runs
        .iter()
        .flat_map(|b| split_block_on_paragraphs(dom, atoms, b))
        .collect();

    let debug = std::env::var_os("OOXMLSDK_DEBUG_MOVES").is_some();
    if debug {
        eprintln!(
            "[moves] del_runs={} ins_runs={} del_paras={} ins_paras={} min_words={} thr={}",
            deleted_runs.len(),
            inserted_runs.len(),
            deleted.len(),
            inserted.len(),
            settings.move_minimum_word_count,
            settings.move_similarity_threshold
        );
    }
    let mut next_group_id = 1u32;
    let mut matched: HashSet<usize> = HashSet::new();
    let mut skipped_short_del = 0usize;
    let mut considered = 0usize;
    let mut best_overall = 0.0f64;
    // Collect matches then thrash-drop expansions (M118). M117: length ratio
    // 0.90 + thr ≥0.97 to cut charter thrash before the size gate.
    let mut pending: Vec<(usize, usize, f64)> = Vec::new();
    for (di, db) in deleted.iter().enumerate() {
        let dtext = extract_text_from_atom_block(dom, atoms, db);
        let dw = count_words(&collapse_ws(&dtext), settings);
        if dw < settings.move_minimum_word_count {
            skipped_short_del += 1;
            continue;
        }
        considered += 1;
        let mut best: Option<usize> = None;
        let mut best_sim = 0.0f64;
        for (ii, ib) in inserted.iter().enumerate() {
            if matched.contains(&ii) {
                continue;
            }
            let itext = extract_text_from_atom_block(dom, atoms, ib);
            let iw = count_words(&collapse_ws(&itext), settings);
            if iw < settings.move_minimum_word_count {
                continue;
            }
            // M117: length ratio 0.90 (was 0.75) — expansions thrash short×long.
            let longer = dw.max(iw);
            let shorter = dw.min(iw);
            if longer > 0 && (shorter as f64) / (longer as f64) < 0.90 {
                continue;
            }
            let sim = jaccard(&dtext, &itext, settings);
            if sim > best_overall {
                best_overall = sim;
            }
            // M117: thr ≥0.97 for all pairs (file_175 had 70 moves at 0.9).
            let thr = settings.move_similarity_threshold.max(0.97);
            if sim >= thr && sim > best_sim {
                best_sim = sim;
                best = Some(ii);
            }
        }
        if let Some(ii) = best {
            pending.push((di, ii, best_sim));
            matched.insert(ii);
        } else if debug && considered <= 5 {
            eprintln!(
                "[moves] unmatched del words={} chars={} preview={:?}",
                dw,
                dtext.len(),
                dtext.chars().take(80).collect::<String>()
            );
        }
    }
    // M118: expansions (file_175 size_ratio~2.8) thrash many near-exact para
    // pairs as moves. Pure reorders (file_8 ~1.7) keep them. Drop all pending
    // when many near-exact matches + size-skewed del/ins text.
    let near_exact = pending.iter().filter(|(_, _, s)| *s >= 0.999).count();
    let mut del_chars = 0usize;
    for db in &deleted {
        del_chars += extract_text_from_atom_block(dom, atoms, db).chars().count();
    }
    let mut ins_chars = 0usize;
    for ib in &inserted {
        ins_chars += extract_text_from_atom_block(dom, atoms, ib).chars().count();
    }
    let size_shorter = del_chars.min(ins_chars).max(1);
    let size_longer = del_chars.max(ins_chars);
    let size_ratio = size_longer as f64 / size_shorter as f64;
    if debug {
        eprintln!(
            "[moves] pending={} near_exact={near_exact} del_chars={del_chars} ins_chars={ins_chars} size_ratio={size_ratio:.2}",
            pending.len()
        );
    }
    if pending.len() >= 12 && near_exact >= 8 && size_ratio >= 2.0 {
        if debug {
            eprintln!(
                "[moves] thrash drop (expansion): pending={} near_exact={near_exact} size_ratio={size_ratio:.2}",
                pending.len()
            );
        }
        pending.clear();
    }
    for (di, ii, _sim) in pending {
        let id = next_group_id;
        let name = format!("move{id}");
        for &ai in &deleted[di].atoms {
            atoms[ai].correlation_status = CorrelationStatus::MovedSource;
            atoms[ai].move_group_id = Some(id);
            atoms[ai].move_name = Some(name.clone());
        }
        for &ai in &inserted[ii].atoms {
            atoms[ai].correlation_status = CorrelationStatus::MovedDestination;
            atoms[ai].move_group_id = Some(id);
            atoms[ai].move_name = Some(name.clone());
        }
        next_group_id += 1;
    }
    if debug {
        eprintln!(
            "[moves] matched={} considered_del={} skipped_short_del={} best_jaccard={:.3}",
            next_group_id - 1,
            considered,
            skipped_short_del,
            best_overall
        );
    }
}

/// PR3 — the memoized `detect_moves` MUST retag atoms identically to the
/// historical [`detect_moves_reference`]. These tests are the equivalence oracle:
/// they build atoms over a real in-memory `Dom` and assert both paths produce the
/// same `(correlation_status, move_group_id, move_name)` on every atom, across an
/// obvious-move case and thousands of seeded-random sequences under several
/// settings profiles.
#[cfg(test)]
mod memoized_moves_tests {
    use super::*;
    use crate::xmllinq::NodeId;

    /// A `Dom` with a fixed vocabulary of text-bearing `w:t` elements plus a
    /// `w:pPr` mark. Atoms reference these node ids; `dom.value` returns the word.
    fn make_pool(vocab: &[&str]) -> (Dom, Vec<NodeId>, NodeId) {
        let mut dom = Dom::new();
        // Trailing space per word so a block's concatenated text tokenizes into
        // separate words — mirroring real atomization, where inter-word spaces are
        // their own atoms. Without it every block collapses to one token.
        let words = vocab
            .iter()
            .map(|w| {
                let e = dom.new_element(W::t());
                dom.set_value(e, &format!("{w} "));
                e
            })
            .collect();
        let ppr = dom.new_element(W::p_pr());
        (dom, words, ppr)
    }

    fn atom(ce: NodeId, status: CorrelationStatus) -> ComparisonUnitAtom {
        let mut a = ComparisonUnitAtom::new(ce, Vec::new(), String::new());
        a.correlation_status = status;
        a
    }

    /// Retagging signature used for equivalence assertions.
    fn sig(atoms: &[ComparisonUnitAtom]) -> Vec<(CorrelationStatus, Option<u32>, Option<String>)> {
        atoms
            .iter()
            .map(|a| (a.correlation_status, a.move_group_id, a.move_name.clone()))
            .collect()
    }

    /// Tiny deterministic LCG (Numerical Recipes constants) — no external rng.
    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, n: usize) -> usize {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) as usize) % n
        }
    }

    /// A deleted paragraph and an identical inserted paragraph → one move. Proves
    /// the reference actually retags (so the Phase-C-less stub genuinely fails) and
    /// that the memoized path reproduces it exactly.
    #[test]
    fn memoized_matches_reference_obvious_move() {
        let vocab = [
            "parity",
            "realtime",
            "coauthoring",
            "comments",
            "mentions",
            "sharing",
            "links",
            "version",
            "history",
        ];
        let (dom, words, _ppr) = make_pool(&vocab);
        let mut base = Vec::new();
        base.push(atom(words[0], CorrelationStatus::Equal));
        for &w in &words {
            base.push(atom(w, CorrelationStatus::Deleted));
        }
        base.push(atom(words[1], CorrelationStatus::Equal));
        for &w in &words {
            base.push(atom(w, CorrelationStatus::Inserted));
        }
        let settings = WmlComparerSettings::default();

        let mut a_ref = base.clone();
        detect_moves_reference(&dom, &mut a_ref, &settings);
        assert!(
            a_ref
                .iter()
                .any(|a| a.correlation_status == CorrelationStatus::MovedSource),
            "reference must detect the move (identical 9-word blocks, jaccard=1.0)"
        );

        let mut a_memo = base.clone();
        detect_moves_memoized(&dom, &mut a_memo, &settings);
        assert_eq!(sig(&a_ref), sig(&a_memo), "memoized must match reference");
    }

    /// Thousands of seeded-random deleted/inserted/equal runs over a small,
    /// overlapping vocabulary — exercises the min-word gate, the 0.90 length
    /// ratio, the 0.97 threshold, greedy `matched` skipping, and the M118 tail,
    /// across three settings profiles. `memoized == reference` must hold for all.
    #[test]
    fn memoized_matches_reference_random() {
        const VOCAB: &[&str] = &[
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "the", "and",
            "of", "to", "shall", "party",
        ];
        let (dom, words, ppr) = make_pool(VOCAB);
        let settings_variants = [
            WmlComparerSettings::default(),
            WmlComparerSettings {
                detect_moves: true,
                move_minimum_word_count: 1,
                move_similarity_threshold: 0.5,
                ..WmlComparerSettings::default()
            },
            WmlComparerSettings {
                detect_moves: true,
                move_minimum_word_count: 3,
                move_similarity_threshold: 0.8,
                ..WmlComparerSettings::default()
            },
            // Closes a review gap: exercise case-insensitive tokenization and a
            // custom word-separator set (shared by both paths, previously untested).
            WmlComparerSettings {
                detect_moves: true,
                case_insensitive: true,
                word_separators: " -".chars().collect(),
                move_minimum_word_count: 2,
                move_similarity_threshold: 0.6,
                ..WmlComparerSettings::default()
            },
        ];
        let mut rng = Lcg(0x0C0F_FEE1_2345_6789);
        for trial in 0..3000 {
            let target = rng.below(40) + 2;
            let mut base: Vec<ComparisonUnitAtom> = Vec::with_capacity(target);
            while base.len() < target {
                let status = match rng.below(5) {
                    0 | 1 => CorrelationStatus::Deleted,
                    2 | 3 => CorrelationStatus::Inserted,
                    _ => CorrelationStatus::Equal,
                };
                let run_len = rng.below(8) + 1;
                for _ in 0..run_len {
                    let ce = if rng.below(12) == 0 {
                        ppr
                    } else {
                        words[rng.below(words.len())]
                    };
                    base.push(atom(ce, status));
                }
            }
            for settings in &settings_variants {
                let mut a_ref = base.clone();
                let mut a_memo = base.clone();
                detect_moves_reference(&dom, &mut a_ref, settings);
                detect_moves_memoized(&dom, &mut a_memo, settings);
                assert_eq!(
                    sig(&a_ref),
                    sig(&a_memo),
                    "trial {trial}: memoized != reference"
                );
            }
        }
    }

    /// In-memory speedup report — reference vs memoized on a `n×n`-paragraph
    /// move-detection workload, timed back-to-back so machine load cancels in the
    /// ratio. Prints only (no wall-clock threshold assert, per the perf plan); the
    /// equivalence assert doubles as a scale check. Run with
    /// `cargo test --lib -- --ignored memoized_speedup_report --nocapture`.
    #[test]
    #[ignore = "perf timing; run with --ignored --nocapture"]
    fn memoized_speedup_report() {
        use std::time::Instant;
        const VOCAB: &[&str] = &[
            "the",
            "party",
            "shall",
            "deliver",
            "goods",
            "within",
            "thirty",
            "days",
            "of",
            "receipt",
            "written",
            "notice",
            "and",
            "any",
            "failure",
            "to",
            "perform",
            "under",
            "this",
            "agreement",
            "constitutes",
            "material",
            "breach",
            "subject",
            "termination",
            "remedies",
            "at",
            "law",
            "or",
            "equity",
        ];
        let (dom, words, ppr) = make_pool(VOCAB);
        let mut rng = Lcg(0x0BAD_C0FF_EE12_3456);
        let n_para = 300usize;
        let para_words = 15usize;
        let mut base = Vec::new();
        for _ in 0..n_para {
            for _ in 0..para_words {
                base.push(atom(
                    words[rng.below(words.len())],
                    CorrelationStatus::Deleted,
                ));
            }
            base.push(atom(ppr, CorrelationStatus::Deleted));
        }
        base.push(atom(words[0], CorrelationStatus::Equal));
        for _ in 0..n_para {
            for _ in 0..para_words {
                base.push(atom(
                    words[rng.below(words.len())],
                    CorrelationStatus::Inserted,
                ));
            }
            base.push(atom(ppr, CorrelationStatus::Inserted));
        }
        let settings = WmlComparerSettings {
            detect_moves: true,
            move_minimum_word_count: 1,
            move_similarity_threshold: 0.5,
            ..WmlComparerSettings::default()
        };

        let mut a_ref = base.clone();
        let t0 = Instant::now();
        detect_moves_reference(&dom, &mut a_ref, &settings);
        let ref_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let mut a_memo = base.clone();
        let t1 = Instant::now();
        detect_moves_memoized(&dom, &mut a_memo, &settings);
        let memo_ms = t1.elapsed().as_secs_f64() * 1000.0;

        assert_eq!(
            sig(&a_ref),
            sig(&a_memo),
            "speedup workload must stay equivalent"
        );
        eprintln!(
            "[perf] detect_moves {n_para}x{n_para} paras: reference={ref_ms:.1}ms \
             memoized={memo_ms:.1}ms speedup={:.1}x",
            ref_ms / memo_ms.max(0.0001)
        );
    }
}
