// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Correlated-hash runs: pair runs of groups by `pt:CorrelatedSHA1Hash`.

use super::*;

/// M4.C.11 — `ProcessCorrelatedHashes` (WmlComparer.ts :7184):
/// pre-correlate runs of groups (Paragraph/Table/Row) by
/// `pt:CorrelatedSHA1Hash`, emitting one `Unknown` per matched group, with
/// before/after `Deleted`/`Inserted`/`Unknown`. Returns `None` (decline)
/// when there are fewer than three units or no qualifying run.
///
/// The result carries every unit; callers' [`CorrelatedSequence`] vectors
/// move (no clone) into the output. The test-only `process_correlated_hashes_owned`
/// is the equivalence oracle for the production path.
#[derive(Clone, Copy)]
pub(super) struct CorrelatedHashRun {
    pub(super) left_start: usize,
    pub(super) right_start: usize,
    pub(super) len: usize,
}

/// Shared threshold gate for correlated-hash run selection.
pub(super) fn correlated_hash_run_threshold(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    bi1: usize,
    bi2: usize,
    best_len: usize,
) -> bool {
    match best_len {
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
    }
}

/// Historical nested start-pair + suffix-extension scanner. Kept as the
/// CORR-IDX-01 reference oracle; production dispatches to the indexed form.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn correlated_hash_run_scan(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun> {
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
    // First-found (i1, i2) wins on atom-count ties (strict `>` only).
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

    if !correlated_hash_run_threshold(cul1, cul2, bi1, bi2, best_len) {
        return None;
    }

    Some(CorrelatedHashRun {
        left_start: bi1,
        right_start: bi2,
        len: best_len,
    })
}

/// CORR-IDX-01 — index right-hand groups by (group_type, correlated hash) and
/// only extend diagonals from matching starts. Must match
/// `correlated_hash_run_scan` exactly (atom-max + first-found `(i1,i2)`).
pub(super) fn correlated_hash_run_indexed(
    unknown: &CorrelatedSequence,
) -> Option<CorrelatedHashRun> {
    use ComparisonUnitGroupType::*;
    use std::collections::HashMap;

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

    // Positions in cul2 that can start a match, ordered ascending (first-found).
    let mut index: HashMap<(ComparisonUnitGroupType, &str), Vec<usize>> =
        HashMap::with_capacity(cul2.len());
    for (i2, u) in cul2.iter().enumerate() {
        if let Some(g) = as_group(u)
            && let Some(h) = g.correlated_sha1_hash.as_deref()
        {
            index.entry((g.group_type, h)).or_default().push(i2);
        }
    }

    let (mut best_len, mut best_atoms, mut bi1, mut bi2) = (0usize, 0usize, usize::MAX, usize::MAX);
    for i1 in 0..cul1.len() {
        let Some(g1) = as_group(&cul1[i1]) else {
            continue;
        };
        let Some(h1) = g1.correlated_sha1_hash.as_deref() else {
            continue;
        };
        let Some(starts) = index.get(&(g1.group_type, h1)) else {
            continue;
        };
        for &i2 in starts {
            let (mut len, mut atoms, mut t1, mut t2) = (0usize, 0usize, i1, i2);
            loop {
                let m = match (
                    cul1.get(t1).and_then(as_group),
                    cul2.get(t2).and_then(as_group),
                ) {
                    (Some(ga), Some(gb)) => {
                        ga.group_type == gb.group_type
                            && ga.correlated_sha1_hash.is_some()
                            && ga.correlated_sha1_hash == gb.correlated_sha1_hash
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

    if !correlated_hash_run_threshold(cul1, cul2, bi1, bi2, best_len) {
        return None;
    }

    Some(CorrelatedHashRun {
        left_start: bi1,
        right_start: bi2,
        len: best_len,
    })
}

/// Production correlated-hash run selection (CORR-IDX-01 indexed path).
pub(super) fn correlated_hash_run(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun> {
    crate::perf::inc_corr_run_scans();
    let run = correlated_hash_run_indexed(unknown);
    if run.is_some() {
        crate::perf::inc_corr_run_hits();
    }
    run
}

/// Production correlated-hash run selection (the production entry).
/// Returns `None` to decline; the next branch in `resolve_correlated_sequences`
/// tries [`find_common_at_beginning_and_end`] and then [`do_lcs_algorithm`].
pub fn process_correlated_hashes(unknown: &CorrelatedSequence) -> Option<Vec<CorrelatedSequence>> {
    let run = correlated_hash_run(unknown)?;
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);

    let mut out = Vec::new();
    // before-region
    cascade(
        cul1[..run.left_start].to_vec(),
        cul2[..run.right_start].to_vec(),
        &mut out,
    );
    // one Unknown per matched group
    for i in 0..run.len {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![cul1[run.left_start + i].clone()],
            vec![cul2[run.right_start + i].clone()],
        ));
    }
    // after-region
    cascade(
        cul1[run.left_start + run.len..].to_vec(),
        cul2[run.right_start + run.len..].to_vec(),
        &mut out,
    );
    Some(out)
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
/// Ownership-only form of [`process_correlated_hashes`], kept for the
/// CORR-IDX tests; production runs [`process_correlated_hashes_in_story`]. A decline
/// returns the original sequence intact so the next resolver can inspect it;
/// an accepted run is split into the same regions while moving every unit.
pub(super) fn process_correlated_hashes_owned(
    unknown: CorrelatedSequence,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let run = correlated_hash_run(&unknown);
    split_at_correlated_run(unknown, run)
}

/// `process_correlated_hashes_owned` with Word's structural final pair: a
/// run ending on the revised story's closing mark against a blank the
/// original runs on past stops before that pair, which then goes to the two
/// closing marks (92075b7449: the blank after a shared closing table).
pub(super) fn process_correlated_hashes_in_story(
    dom: &Dom,
    unknown: CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let mut run = correlated_hash_run(&unknown);
    if let Some(r) = run.as_mut()
        && settings.merge_replaced_paragraphs
        && r.len > 0
    {
        let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
        let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
        let (l, rr) = (
            &cul1[r.left_start + r.len - 1],
            &cul2[r.right_start + r.len - 1],
        );
        if unit_closes_story(dom, rr)
            && !unit_closes_story(dom, l)
            && unit_is_textless_paragraph_matter(dom, l)
            && cul1.last().is_some_and(|u| unit_closes_story(dom, u))
        {
            r.len -= 1;
        }
    }
    let run = run.filter(|r| r.len > 0);
    split_at_correlated_run(unknown, run)
}

pub(super) fn split_at_correlated_run(
    mut unknown: CorrelatedSequence,
    run: Option<CorrelatedHashRun>,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let Some(run) = run else {
        return Err(unknown);
    };

    let mut cul1 = unknown.com_units_1.take().unwrap_or_default();
    let mut cul2 = unknown.com_units_2.take().unwrap_or_default();

    let after1 = cul1.split_off(run.left_start + run.len);
    let matched1 = cul1.split_off(run.left_start);
    let after2 = cul2.split_off(run.right_start + run.len);
    let matched2 = cul2.split_off(run.right_start);

    let mut out = Vec::with_capacity(run.len + 2);
    cascade(cul1, cul2, &mut out);
    for (left, right) in matched1.into_iter().zip(matched2) {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![left],
            vec![right],
        ));
    }
    cascade(after1, after2, &mut out);
    Ok(out)
}
