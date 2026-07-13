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
                    })
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
            s.push_str(&dom.value(ce));
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

/// M4.G.3 — `DetectMovesInAtomList` (:4711): greedy match deleted↔inserted blocks
/// by Jaccard ≥ threshold (min word count), retag MovedSource/MovedDestination.
///
/// Word-visual extensions:
/// - after consecutive-status grouping, **split on `w:pPr`** (paragraph units)
/// - M117: length ratio ≥0.90 and thr = max(settings, 0.97)
/// - M118: drop all pending when expansion thrash (pending≥12, near_exact≥8, size_ratio≥2)
pub fn detect_moves_in_atom_list(
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
