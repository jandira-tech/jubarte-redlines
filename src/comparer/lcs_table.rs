// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.D — table / row / cell LCS and row revision marking.
//!
//! Port of `ApplyLcsToTableRows` (:8241), `DoLcsAlgorithmForTable` (:8348),
//! `MarkRowsAsDeletedOrInserted` (:4216), and the
//! `rows_preserving_horizontal_partitions` gate that keeps whole rows for
//! a horizontal repartition outside Word mode. The classic DP LCS over rows
//! keyed on `sha1()` produces Deleted/Inserted/Unknown sequences; the row
//! marker pass turns those into `w:ins`/`w:del` children of each affected
//! row's `w:trPr`, keeping `w:trPrChange` last in `CT_TrPr` order so the
//! output survives Word's PDF-save path.

use super::atoms::{ComparisonUnit, ComparisonUnitGroup, CorrelatedSequence};
use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerSettings};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

fn as_group(u: &ComparisonUnit) -> Option<&ComparisonUnitGroup> {
    match u {
        ComparisonUnit::Group(g) => Some(g),
        ComparisonUnit::Word(_) => None,
    }
}

/// `w:tbl`/`w:tr` ancestor of a group's FIRST descendant atom.
fn ancestor_named(
    dom: &Dom,
    g: &ComparisonUnitGroup,
    name: &crate::xmllinq::XName,
) -> Option<NodeId> {
    let cu = ComparisonUnit::Group(g.clone());
    let atoms = cu.descendant_atoms();
    let first = atoms.first()?;
    first
        .ancestor_elements
        .iter()
        .rev()
        .copied()
        .find(|&a| dom.name(a).as_ref() == Some(name))
}

/// M4.D.1 — `ApplyLcsToTableRows` (WmlComparer.ts :8241): classic DP LCS
/// over rows keyed on [`ComparisonUnit::sha1`], emitting
/// Deleted/Inserted/Unknown sequences in document order. The match list is
/// reversed so the output is forward order; an inconsistency between the
/// matched/deleted/inserted vectors and the row indexes is a `unreachable!`
/// bug rather than silent data loss.
pub fn apply_lcs_to_table_rows(
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Vec<CorrelatedSequence> {
    let m = rows1.len();
    let n = rows2.len();
    let mut lcs = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            lcs[i][j] = if rows1[i - 1].sha1() == rows2[j - 1].sha1() {
                lcs[i - 1][j - 1] + 1
            } else {
                lcs[i - 1][j].max(lcs[i][j - 1])
            };
        }
    }
    let (mut ii, mut jj) = (m, n);
    let mut deleted = Vec::new();
    let mut inserted = Vec::new();
    let mut matched = Vec::new();
    while ii > 0 || jj > 0 {
        if ii > 0 && jj > 0 && rows1[ii - 1].sha1() == rows2[jj - 1].sha1() {
            matched.push((ii - 1, jj - 1));
            ii -= 1;
            jj -= 1;
        } else if jj > 0 && (ii == 0 || lcs[ii][jj - 1] >= lcs[ii - 1][jj]) {
            inserted.push(jj - 1);
            jj -= 1;
        } else {
            deleted.push(ii - 1);
            ii -= 1;
        }
    }
    matched.reverse();
    deleted.reverse();
    inserted.reverse();

    if m == 0 && n == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let (mut i1, mut i2) = (0usize, 0usize);
    let (mut mi, mut di, mut si) = (0usize, 0usize, 0usize);
    while i1 < m || i2 < n {
        if di < deleted.len() && i1 == deleted[di] {
            result.push(CorrelatedSequence::deleted(vec![rows1[i1].clone()]));
            di += 1;
            i1 += 1;
            continue;
        }
        if si < inserted.len() && i2 == inserted[si] {
            result.push(CorrelatedSequence::inserted(vec![rows2[i2].clone()]));
            si += 1;
            i2 += 1;
            continue;
        }
        if mi < matched.len() && i1 == matched[mi].0 && i2 == matched[mi].1 {
            result.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![rows1[i1].clone()],
                vec![rows2[i2].clone()],
            ));
            mi += 1;
            i1 += 1;
            i2 += 1;
            continue;
        }
        break; // unreachable: the DP emits a monotone del/ins/match ordering that
        // covers every row index exactly once. If this ever fires, the DP
        // or the matched/deleted/inserted vectors are inconsistent — fail
        // loudly rather than silently dropping rows.
    }
    if i1 < m || i2 < n {
        unreachable!(
            "apply_lcs_to_table_rows: matched/deleted/inserted exhausted without covering all rows (i1={}, m={}, i2={}, n={})",
            i1, m, i2, n
        );
    }
    result
}

/// One row operation from Word's shared-word alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WordRowOp {
    Pair(usize, usize),
    Delete(usize),
    Insert(usize),
}

/// M4.D.5 — Word's row alignment for paired tables with no identical rows
/// (Task 22, redline `3d4318d7e9`): `score(i, j)` is the number of shared
/// case-insensitive alphanumeric word tokens; global max-weight monotone
/// matching with gap cost 0; the backtrack deletes an old row on a tie,
/// inserts a new row on a tie, and pairs only when strictly better than
/// both; gaps between positive-score anchors fill positionally (pair, then
/// deleted rest, then inserted rest).
pub(crate) fn align_rows_by_shared_words(
    old: &[std::collections::HashSet<String>],
    new: &[std::collections::HashSet<String>],
) -> Vec<WordRowOp> {
    let (m, n) = (old.len(), new.len());
    let score = |i: usize, j: usize| old[i].intersection(&new[j]).count();
    let mut best = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            best[i][j] = best[i - 1][j]
                .max(best[i][j - 1])
                .max(best[i - 1][j - 1] + score(i - 1, j - 1));
        }
    }
    // Backtrack: a tied deletion or insertion wins over pairing, so a pair
    // is an anchor only when strictly better than both one-sided moves.
    // (A zero-score diagonal can never be strictly better: it would need
    // `best[i-1][j-1] > best[i-1][j]`, against the table's monotonicity.)
    let mut anchors = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if best[i - 1][j] == best[i][j] {
            i -= 1;
        } else if best[i][j - 1] == best[i][j] {
            j -= 1;
        } else {
            debug_assert_eq!(best[i - 1][j - 1] + score(i - 1, j - 1), best[i][j]);
            if score(i - 1, j - 1) > 0 {
                anchors.push((i - 1, j - 1));
            }
            i -= 1;
            j -= 1;
        }
    }
    anchors.sort_unstable();
    // Positional gap fill between anchors (and around them).
    let mut ops = Vec::new();
    let (mut taken_old, mut taken_new) = (0usize, 0usize);
    for (a, b) in anchors.iter().copied().chain([(m, n)]) {
        let (gap_old, gap_new) = (a - taken_old, b - taken_new);
        let paired = gap_old.min(gap_new);
        for t in 0..paired {
            ops.push(WordRowOp::Pair(taken_old + t, taken_new + t));
        }
        for t in paired..gap_old {
            ops.push(WordRowOp::Delete(taken_old + t));
        }
        for t in paired..gap_new {
            ops.push(WordRowOp::Insert(taken_new + t));
        }
        if (a, b) != (m, n) {
            ops.push(WordRowOp::Pair(a, b));
        }
        (taken_old, taken_new) = (a + 1, b + 1);
    }
    ops
}

/// The joined `w:t` text of a row as lowercase alphanumeric word tokens.
fn row_word_tokens(dom: &Dom, row: &ComparisonUnit) -> std::collections::HashSet<String> {
    let mut text = String::new();
    let mut previous_paragraph = None;
    for atom in row.descendant_atoms() {
        if dom.name_is(atom.content_element, &W::t()) {
            let paragraph = atom
                .ancestor_elements
                .iter()
                .rev()
                .copied()
                .find(|&node| dom.name_is(node, &W::p()));
            if previous_paragraph.is_some() && previous_paragraph != paragraph {
                text.push(' ');
            }
            text.push_str(&dom.value_str(atom.content_element));
            previous_paragraph = paragraph;
        } else if dom.name_is(atom.content_element, &W::name("tab"))
            || dom.name_is(atom.content_element, &W::name("br"))
            || dom.name_is(atom.content_element, &W::name("cr"))
        {
            text.push(' ');
        }
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

fn table_has_merges(dom: &Dom, rows: &[ComparisonUnit]) -> bool {
    let tbl_name = W::tbl();
    rows.iter()
        .filter_map(as_group)
        .find_map(|g| ancestor_named(dom, g, &tbl_name))
        .is_some_and(|t| contains_merged(dom, t))
}

/// Word's shared-word row alignment owns this table pair when Word mode is
/// on, both sides have rows, no row is identical (the `sha1` LCS owns that
/// case), and neither table uses merges (those keep their dedicated
/// positional routes).
pub(crate) fn shared_word_row_sequences(
    dom: &Dom,
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Option<Vec<CorrelatedSequence>> {
    if rows1.is_empty() || rows2.is_empty() {
        return None;
    }
    if table_has_merges(dom, rows1) || table_has_merges(dom, rows2) {
        return None;
    }
    if rows1
        .iter()
        .any(|r1| rows2.iter().any(|r2| r1.sha1() == r2.sha1()))
    {
        return None;
    }
    let old: Vec<_> = rows1.iter().map(|r| row_word_tokens(dom, r)).collect();
    let new: Vec<_> = rows2.iter().map(|r| row_word_tokens(dom, r)).collect();
    // Content-free rows carry no signal to score: decline (the historical
    // contract) rather than pairing emptiness positionally.
    if !old.iter().any(|t| !t.is_empty()) || !new.iter().any(|t| !t.is_empty()) {
        return None;
    }
    Some(
        align_rows_by_shared_words(&old, &new)
            .into_iter()
            .map(|op| match op {
                WordRowOp::Pair(a, b) => CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![rows1[a].clone()],
                    vec![rows2[b].clone()],
                ),
                WordRowOp::Delete(a) => CorrelatedSequence::deleted(vec![rows1[a].clone()]),
                WordRowOp::Insert(b) => CorrelatedSequence::inserted(vec![rows2[b].clone()]),
            })
            .collect(),
    )
}

fn contains_merged(dom: &Dom, tbl: NodeId) -> bool {
    !dom.descendants(tbl, Some(&W::v_merge())).is_empty()
        || !dom.descendants(tbl, Some(&W::grid_span())).is_empty()
}

fn positional_rows(rows1: &[ComparisonUnit], rows2: &[ComparisonUnit]) -> Vec<CorrelatedSequence> {
    rows1
        .iter()
        .zip(rows2.iter())
        .map(|(r1, r2)| {
            CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![r1.clone()],
                vec![r2.clone()],
            )
        })
        .collect()
}

/// Positional row pairing for unequal row counts: zip `min(len)` pairs as
/// Unknown (cell-level LCS follows), then pure del/ins for the longer side's
/// remainder. Word-mode uses this when one table has vMerge/gridSpan and the
/// structure hashes differ — whole-table del+ins shreds the GT shape
/// (batch_to_fix pair 02: cell 0 is `AAA[D:R1C1]`, not a deleted base row
/// followed by an inserted next row).
fn positional_rows_with_remainder(
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Vec<CorrelatedSequence> {
    let n = rows1.len().min(rows2.len());
    let mut out = Vec::with_capacity(rows1.len() + rows2.len());
    for i in 0..n {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![rows1[i].clone()],
            vec![rows2[i].clone()],
        ));
    }
    for r in &rows1[n..] {
        out.push(CorrelatedSequence::deleted(vec![r.clone()]));
    }
    for r in &rows2[n..] {
        out.push(CorrelatedSequence::inserted(vec![r.clone()]));
    }
    out
}

/// Physical cell boundaries, independent of vertical merges and nested tables.
/// A row's atoms can start inside a nested table; use their deepest COMMON row
/// ancestor so that an inner row cannot masquerade as the outer row's geometry.
pub(crate) fn row_container(dom: &Dom, row: &ComparisonUnit) -> Option<NodeId> {
    let atoms = row.descendant_atoms();
    let first = atoms.first()?;
    let ancestor = first.ancestor_elements.iter().rev().copied().find(|&a| {
        dom.name_is(a, &W::tr()) && atoms.iter().all(|atom| atom.ancestor_elements.contains(&a))
    })?;
    Some(ancestor)
}

fn horizontal_cell_partition(dom: &Dom, row: &ComparisonUnit) -> Option<Vec<u64>> {
    Some(row_partition(dom, row_container(dom, row)?))
}

/// A `w:tr`'s cell boundaries: the grid column each of its cells ends at.
pub(crate) fn row_partition(dom: &Dom, tr: NodeId) -> Vec<u64> {
    let mut edge = 0u64;
    dom.elements(tr, Some(&W::tc()))
        .into_iter()
        .map(|tc| {
            let span = dom
                .element(tc, &W::tc_pr())
                .and_then(|pr| dom.element(pr, &W::grid_span()))
                .and_then(|span| dom.attribute(span, &W::val()))
                .and_then(|val| val.parse::<u32>().ok())
                .unwrap_or(1)
                .max(1);
            edge += u64::from(span);
            edge
        })
        .collect()
}

/// Word pairs an original table with one revised table (#1817: in w23 the
/// original's table pairs with the first of two adjacent revised tables, and
/// the second stays a table of its own). In Word mode, rows that H4 flattened
/// out of several tables therefore pair table by table: in order, each table
/// with the one sharing the most rows, the earlier one when they share none.
/// The tables left over are deleted or inserted whole. `None` when each side's
/// rows come from one table, or outside Word mode.
pub(crate) fn rows_by_table(
    dom: &Dom,
    original: &[ComparisonUnit],
    revised: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    if !settings.merge_replaced_paragraphs {
        return None;
    }
    let tables = |rows: &[ComparisonUnit]| -> Vec<Vec<ComparisonUnit>> {
        crate::util::group_adjacent(rows.iter().cloned(), |row| {
            row_container(dom, row)
                .and_then(|tr| dom.ancestors(tr, Some(&W::tbl())).first().copied())
        })
        .into_iter()
        .map(|(_, rows)| rows)
        .collect()
    };
    let (old, new) = (tables(original), tables(revised));
    if old.len() < 2 && new.len() < 2 {
        return None;
    }
    let shared = |a: &[ComparisonUnit], b: &[ComparisonUnit]| {
        a.iter()
            .filter(|row| b.iter().any(|other| other.sha1() == row.sha1()))
            .count()
    };
    // best[i][j]: (rows shared, tables paired) over old[i..] and new[j..].
    let (m, n) = (old.len(), new.len());
    let mut best = vec![vec![(0usize, 0usize); n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            let (s, p) = best[i + 1][j + 1];
            best[i][j] = (s + shared(&old[i], &new[j]), p + 1)
                .max(best[i][j + 1])
                .max(best[i + 1][j]);
        }
    }
    let mut out = Vec::new();
    let (mut old, mut new) = (old.into_iter(), new.into_iter());
    let (mut i, mut j) = (0, 0);
    while i < m || j < n {
        let pairs = i < m && j < n && {
            let (s, p) = best[i + 1][j + 1];
            best[i][j] == (s + shared(&old.as_slice()[0], &new.as_slice()[0]), p + 1)
        };
        if pairs {
            let (a, b) = (old.next()?, new.next()?);
            out.push(CorrelatedSequence::paired(CorrelationStatus::Unknown, a, b));
            (i, j) = (i + 1, j + 1);
        } else if j < n && (i == m || best[i][j] == best[i][j + 1]) {
            out.push(CorrelatedSequence::inserted(new.next()?));
            j += 1;
        } else {
            out.push(CorrelatedSequence::deleted(old.next()?));
            i += 1;
        }
    }
    Some(out)
}

/// A horizontal repartition — a paired row whose cell boundaries moved — in
/// Word mode is meshed the way Word 16 meshes it (#1817; 21 Word Compare
/// redlines in jubarte-loop/fix23/ab1817): the rows pair and the cells keep
/// their own revisions, so `None` here. The other modes keep complete
/// rows under row revisions ([`rows_preserving_horizontal_partitions`]),
/// which conserves both source shapes: Word's mesh gives up the round trip
/// (its Reject All keeps B's extra cell, its Accept All A's), and a
/// row-level revision cannot carry a changed cell count.
pub(crate) fn rows_preserving_horizontal_partitions_for(
    dom: &Dom,
    original: &[ComparisonUnit],
    revised: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    if settings.merge_replaced_paragraphs {
        return None;
    }
    rows_preserving_horizontal_partitions(dom, original, revised)
}

/// Complete revised/original rows under ordinary row revisions for every
/// paired row whose horizontal partition differs. An unmatched cell's atoms
/// would otherwise lose their cell group, and closing marks could give it
/// another cell's revised properties; this conserves both source shapes.
pub(crate) fn rows_preserving_horizontal_partitions(
    dom: &Dom,
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Option<Vec<CorrelatedSequence>> {
    let differs = |a: &ComparisonUnit, b: &ComparisonUnit| match (
        horizontal_cell_partition(dom, a),
        horizontal_cell_partition(dom, b),
    ) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    };
    if !rows1.iter().zip(rows2).any(|(a, b)| differs(a, b)) {
        return None;
    }
    let mut out = Vec::new();
    let n = rows1.len().min(rows2.len());
    for (a, b) in rows1[..n].iter().zip(&rows2[..n]) {
        if differs(a, b) {
            out.push(CorrelatedSequence::inserted(vec![b.clone()]));
            out.push(CorrelatedSequence::deleted(vec![a.clone()]));
        } else {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![a.clone()],
                vec![b.clone()],
            ));
        }
    }
    out.extend(
        rows1[n..]
            .iter()
            .map(|a| CorrelatedSequence::deleted(vec![a.clone()])),
    );
    out.extend(
        rows2[n..]
            .iter()
            .map(|b| CorrelatedSequence::inserted(vec![b.clone()])),
    );
    Some(out)
}

/// M4.D.2/D.3 — `DoLcsAlgorithmForTable` (WmlComparer.ts :8348):
/// single-table-vs-single-table dispatch. Tries horizontal-partition
/// preservation first, then a row-level LCS on same-row-count content
/// differences, then a merged-cell / structure-hash fallback. `None` means
/// "the caller should fall back to whole-table del+ins".
pub fn do_lcs_algorithm_for_table(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let g1 = as_group(cul1.first()?)?;
    let g2 = as_group(cul2.first()?)?;
    let rows1 = &g1.contents;
    let rows2 = &g2.contents;
    // M4.D.5 — Word's shared-word row alignment owns pairs with no identical
    // rows in Word mode; every other shape keeps its route below.
    if settings.merge_replaced_paragraphs
        && let Some(rows) = shared_word_row_sequences(dom, rows1, rows2)
    {
        return Some(rows);
    }
    if let Some(rows) = rows_preserving_horizontal_partitions_for(dom, rows1, rows2, settings) {
        return Some(rows);
    }

    // same-row-count path
    if rows1.len() == rows2.len() {
        let total = rows1.len();
        let can_collapse = rows1
            .iter()
            .zip(rows2.iter())
            .all(|(r1, r2)| r1.correlated_sha1() == r2.correlated_sha1());
        let hash_diff = rows1
            .iter()
            .zip(rows2.iter())
            .filter(|(r1, r2)| r1.sha1() != r2.sha1())
            .count();
        // FAITHFUL — mirrored from WmlComparer.cs:7912-7918 (DoLcsAlgorithmForTable
        // same-row-count content-LCS heuristic). Guards the "mostly-equal table with
        // a few changed rows" case: do not collapse positionally when at least two
        // rows differ but most still match, fall through to row-level LCS.
        let use_content_lcs = hash_diff > 1 && hash_diff < total && hash_diff > total / 3;
        if can_collapse && use_content_lcs && total >= 7 {
            let r = apply_lcs_to_table_rows(rows1, rows2);
            if !r.is_empty() {
                return Some(r);
            }
        }
        if can_collapse {
            return Some(positional_rows(rows1, rows2));
        }
    }

    // merged-cell / structure-hash fallback
    let tbl_name = W::tbl();
    let left_merged = ancestor_named(dom, g1, &tbl_name).is_some_and(|t| contains_merged(dom, t));
    let right_merged = ancestor_named(dom, g2, &tbl_name).is_some_and(|t| contains_merged(dom, t));
    if left_merged || right_merged {
        if let (Some(s1), Some(s2)) = (&g1.structure_sha1_hash, &g2.structure_sha1_hash)
            && s1 == s2
        {
            return Some(positional_rows(rows1, rows2));
        }
        // Word-mode: still positionally pair rows so cells can mix (pair 02 /
        // table-bookmark-end_table-vmerge-colspan GT: first cell holds both
        // ins AAA and del R1C1). Faithful keeps C# whole-table del+ins.
        if settings.merge_replaced_paragraphs {
            return Some(positional_rows_with_remainder(rows1, rows2));
        }
        return Some(vec![
            CorrelatedSequence::deleted(rows1.to_vec()),
            CorrelatedSequence::inserted(rows2.to_vec()),
        ]);
    }
    None
}

/// M4.D.4 — `MarkRowsAsDeletedOrInserted` (:4216): for each Deleted/Inserted
/// sequence, add `w:del`/`w:ins` into each Row's `w:trPr` (created as first child
/// if absent). Explicit whole-Table lifetimes mark their descendant rows;
/// a Row lifetime marks only its own common row, including nested content.
pub fn mark_rows_as_deleted_or_inserted(
    dom: &mut Dom,
    settings: &WmlComparerSettings,
    sequences: &[CorrelatedSequence],
    next_id: &mut u32,
) {
    let tr_name = W::tr();
    let mut marked = std::collections::HashSet::new();
    for cs in sequences {
        let (units, rev_name) = match cs.correlation_status {
            CorrelationStatus::Deleted => (cs.com_units_1.as_ref(), W::del()),
            CorrelationStatus::Inserted => (cs.com_units_2.as_ref(), W::ins()),
            _ => continue,
        };
        let Some(units) = units else { continue };
        for unit in units {
            let Some(g) = as_group(unit) else { continue };
            let owner_name = match g.group_type {
                ComparisonUnitGroupType::Row => tr_name.clone(),
                ComparisonUnitGroupType::Table => W::tbl(),
                _ => continue,
            };
            let atoms = unit.descendant_atoms();
            let Some(first) = atoms.first() else { continue };
            // A nested table's first atom is also inside its outer row. The
            // lifecycle unit owns the deepest common container, not every
            // row ancestor of that atom.
            let Some(owner) = first.ancestor_elements.iter().rev().copied().find(|&node| {
                dom.name_is(node, &owner_name)
                    && atoms
                        .iter()
                        .all(|atom| atom.ancestor_elements.contains(&node))
            }) else {
                continue;
            };
            let rows = if g.group_type == ComparisonUnitGroupType::Table {
                dom.descendants(owner, Some(&tr_name))
            } else {
                vec![owner]
            };
            for tr in rows {
                if !marked.insert(tr) {
                    continue;
                }
                let trpr = row_properties(dom, tr);
                let rev = dom.new_element(rev_name.clone());
                dom.set_attribute_value(rev, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(rev, &W::id(), Some(&next_id.to_string()));
                *next_id += 1;
                dom.set_attribute_value(rev, &W::date(), Some(&settings.date_time_for_revisions));
                add_row_mark(dom, trpr, rev);
            }
        }
    }
}

/// The row's `w:trPr`, created when absent after a `w:tblPrEx` (CT_Row
/// order), else as the row's first child.
pub(crate) fn row_properties(dom: &mut Dom, tr: NodeId) -> NodeId {
    if let Some(p) = dom.element(tr, &W::tr_pr()) {
        return p;
    }
    let p = dom.new_element(W::tr_pr());
    match dom.element(tr, &W::name("tblPrEx")) {
        Some(ex) => dom.add_after_self(ex, p),
        None => dom.add_first(tr, p),
    }
    p
}

/// Add a row's `w:ins` / `w:del` to its `w:trPr`: CT_TrPr keeps them ahead
/// of a `w:trPrChange` (acce1b593c's header rows wrote the mark after it:
/// the schema rejects that, and Word timed out saving the redline as PDF).
pub(crate) fn add_row_mark(dom: &mut Dom, trpr: NodeId, rev: NodeId) {
    match dom.element(trpr, &W::name("trPrChange")) {
        Some(change) => dom.add_before_self(change, rev),
        None => dom.add(trpr, rev),
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod row_mark_tests {
    use super::*;

    fn child_names(dom: &Dom, n: NodeId) -> Vec<String> {
        dom.elements(n, None)
            .into_iter()
            .filter_map(|c| dom.name(c).map(|x| x.local_name().to_string()))
            .collect()
    }

    #[test]
    fn a_row_mark_goes_before_the_row_property_change() {
        // acce1b593c vs 4e2d5a0b0c: header rows came out as
        // jc, trPrChange, del, which the validator rejects.
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(
            r#"<w:trPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:jc w:val="center"/><w:trPrChange w:id="1" w:author="A" w:date="1970-01-01T00:00:00Z"><w:trPr/></w:trPrChange></w:trPr>"#,
        );
        let trpr = dom.root(doc).expect("trPr");
        let del = dom.new_element(W::del());
        add_row_mark(&mut dom, trpr, del);
        assert_eq!(child_names(&dom, trpr), ["jc", "del", "trPrChange"]);

        let doc = dom.parse_xdocument(
            r#"<w:trPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:jc w:val="center"/></w:trPr>"#,
        );
        let plain = dom.root(doc).expect("trPr");
        let ins = dom.new_element(W::ins());
        add_row_mark(&mut dom, plain, ins);
        assert_eq!(child_names(&dom, plain), ["jc", "ins"]);
    }
}
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod horizontal_partition_tests {
    use super::*;

    fn cell(span: u32, width: u32, shade: &str, text: &str) -> String {
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/><w:gridSpan w:val=\"{span}\"/><w:shd w:val=\"clear\" w:fill=\"{shade}\"/></w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"
        )
    }
    fn table(cells: &str) -> String {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:tr xmlns:w=\"{}\">{cells}</w:tr>", W::URI,));
        let row = dom.root(doc).unwrap();
        let columns: u32 = dom
            .elements(row, Some(&W::tc()))
            .into_iter()
            .map(|tc| {
                dom.element(tc, &W::tc_pr())
                    .and_then(|pr| dom.element(pr, &W::grid_span()))
                    .and_then(|span| dom.attribute(span, &W::val()))
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1)
                    .max(1)
            })
            .sum();
        let columns = columns.max(1);
        let grid: String = (0..columns)
            .map(|i| {
                let width = 5000 / columns + u32::from(i == columns - 1) * (5000 % columns);
                format!("<w:gridCol w:w=\"{width}\"/>")
            })
            .collect();
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:trPr><w:trHeight w:val=\"360\"/></w:trPr>{cells}</w:tr></w:tbl>"
        )
    }
    fn package(body: &str) -> Vec<u8> {
        let mut pkg =
            crate::opc::PartFs::open(include_bytes!("../../tests/fixtures/relids/image_doc.docx"))
                .unwrap();
        pkg.set_part("word/document.xml", format!("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{body}<w:p/><w:sectPr/></w:body></w:document>").into_bytes());
        pkg.to_zip().unwrap()
    }
    fn prop(dom: &Dom, n: NodeId) -> String {
        let mut attrs: Vec<_> = dom
            .attributes_iter(n)
            .map(|(k, v)| format!("{}={v}", k.local_name()))
            .collect();
        attrs.sort();
        let mut kids: Vec<_> = dom
            .elements(n, None)
            .into_iter()
            .map(|c| prop(dom, c))
            .collect();
        kids.sort();
        format!("{}:{attrs:?}:{kids:?}", dom.name(n).unwrap().local_name())
    }
    type CellState = (String, String);
    type RowState = (String, Vec<CellState>);
    fn geometry(bytes: &[u8]) -> Vec<Vec<RowState>> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let xml = std::str::from_utf8(pkg.part_bytes("word/document.xml").unwrap()).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::tbl()))
            .into_iter()
            .map(|tbl| {
                dom.elements(tbl, Some(&W::tr()))
                    .into_iter()
                    .map(|tr| {
                        let row_props = dom
                            .element(tr, &W::tr_pr())
                            .filter(|&p| !dom.elements(p, None).is_empty())
                            .map(|p| prop(&dom, p))
                            .unwrap_or_default();
                        let cells = dom
                            .elements(tr, Some(&W::tc()))
                            .into_iter()
                            .map(|tc| {
                                let props = dom
                                    .element(tc, &W::tc_pr())
                                    .map(|p| prop(&dom, p))
                                    .unwrap_or_default();
                                let text = dom
                                    .descendants(tc, Some(&W::t()))
                                    .into_iter()
                                    .map(|t| dom.value(t))
                                    .collect();
                                (props, text)
                            })
                            .collect();
                        (row_props, cells)
                    })
                    .collect()
            })
            .collect()
    }
    fn grids(bytes: &[u8]) -> Vec<Vec<String>> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::tbl()))
            .into_iter()
            .map(|tbl| {
                dom.element(tbl, &W::name("tblGrid"))
                    .map(|grid| {
                        dom.elements(grid, Some(&W::grid_col()))
                            .into_iter()
                            .map(|column| prop(&dom, column))
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect()
    }

    #[test]
    fn horizontal_repartition_roundtrips_source_cells_properties_and_nested_tables() {
        let long = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        let two = table(&(cell(1, 2500, "AAAAAA", long) + &cell(1, 2500, "BBBBBB", "Right")));
        let spanning = table(&cell(2, 5000, "CCCCCC", long));
        let left_edges =
            table(&(cell(1, 1600, "AAAAAA", long) + &cell(2, 3400, "BBBBBB", "Right")));
        let right_edges =
            table(&(cell(2, 3400, "CCCCCC", long) + &cell(1, 1600, "DDDDDD", "Right")));
        let nested_a = table(&cell(2, 5000, "EEEEEE", long))
            .replace("</w:tc>", &format!("{two}<w:p/></w:tc>"));
        let nested_b = table(&cell(2, 5000, "EEEEEE", long))
            .replace("</w:tc>", &format!("{spanning}<w:p/></w:tc>"));
        let prose =
            "<w:p><w:r><w:t>The supplementary paragraph explains the table.</w:t></w:r></w:p>";
        let before = format!("{prose}{two}");
        let after = format!("{two}{prose}");
        for (a, b) in [
            (&two, &spanning),
            (&spanning, &two),
            (&left_edges, &right_edges),
            (&nested_a, &nested_b),
            (&nested_b, &nested_a),
            (&before, &spanning),
            (&after, &spanning),
            (&spanning, &before),
            (&spanning, &after),
        ] {
            let a = package(a);
            let b = package(b);
            // Outside Word mode only: Word's mesh gives up this round trip
            // (word_mode_meshes_horizontal_repartitions_like_word).
            {
                let settings = WmlComparerSettings::powertools_faithful();
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    geometry(&accepted),
                    geometry(&b),
                    "accepted cell/row geometry"
                );
                assert_eq!(
                    geometry(&rejected),
                    geometry(&a),
                    "rejected cell/row geometry"
                );
                assert_eq!(grids(&accepted), grids(&b), "accepted authored grids");
                assert_eq!(grids(&rejected), grids(&a), "rejected authored grids");
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
            }
        }
    }
    fn offset_table(before: u32, after: u32, cells: &str) -> String {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:tr xmlns:w=\"{}\">{cells}</w:tr>", W::URI));
        let row = dom.root(doc).unwrap();
        let occupied: u32 = dom
            .elements(row, Some(&W::tc()))
            .into_iter()
            .map(|tc| {
                dom.element(tc, &W::tc_pr())
                    .and_then(|pr| dom.element(pr, &W::grid_span()))
                    .and_then(|span| dom.attribute(span, &W::val()))
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(1)
                    .max(1)
            })
            .sum();
        assert_eq!(before + occupied + after, 3, "authored three-column grid");
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"6000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:trPr><w:gridBefore w:val=\"{before}\"/><w:gridAfter w:val=\"{after}\"/><w:wBefore w:w=\"{}\" w:type=\"dxa\"/><w:wAfter w:w=\"{}\" w:type=\"dxa\"/><w:trHeight w:val=\"360\"/></w:trPr>{cells}</w:tr></w:tbl>",
            2000 * before,
            2000 * after,
        )
    }

    #[test]
    fn horizontal_row_offsets_roundtrip_exact_authored_grid_and_row_properties() {
        let text = "The shared contractual paragraph identifies the table row and its original cell ownership.";
        let cells = cell(1, 2000, "AAAAAA", text) + &cell(1, 2000, "BBBBBB", "Right cell");
        let left = offset_table(0, 1, &cells);
        let right = offset_table(1, 0, &cells);
        let spanning = offset_table(1, 0, &cell(2, 4000, "CCCCCC", text));
        for (original, revised) in [
            (&left, &right),
            (&right, &left),
            (&left, &spanning),
            (&spanning, &left),
        ] {
            let a = package(original);
            let b = package(revised);
            // Outside Word mode only: Word's mesh gives up this round trip
            // (word_mode_meshes_horizontal_repartitions_like_word).
            {
                let settings = WmlComparerSettings::powertools_faithful();
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    geometry(&accepted),
                    geometry(&b),
                    "accepted gridBefore/gridAfter and cell ownership"
                );
                assert_eq!(
                    geometry(&rejected),
                    geometry(&a),
                    "rejected gridBefore/gridAfter and cell ownership"
                );
                assert_eq!(grids(&accepted), grids(&b), "accepted authored table grid");
                assert_eq!(grids(&rejected), grids(&a), "rejected authored table grid");
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
            }
        }
    }

    #[test]
    fn horizontal_row_and_cell_lifetimes_survive_with_format_tracking_disabled() {
        // Cell counts and text ownership are structural lifetimes, independent
        // of whether the caller requests revision records for formatting.
        let text = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        let two = table(
            &(cell(1, 2500, "AAAAAA", text) + &cell(1, 2500, "BBBBBB", "Original right cell")),
        );
        let spanning = table(&cell(2, 5000, "CCCCCC", text));
        let structure = |bytes: &[u8]| -> Vec<Vec<Vec<String>>> {
            geometry(bytes)
                .into_iter()
                .map(|rows| {
                    rows.into_iter()
                        .map(|(_, cells)| cells.into_iter().map(|(_, text)| text).collect())
                        .collect()
                })
                .collect()
        };
        for (original, revised) in [(&two, &spanning), (&spanning, &two)] {
            let a = package(original);
            let b = package(revised);
            // Outside Word mode only, as in the round trips above.
            {
                let mut settings = WmlComparerSettings::powertools_faithful();
                settings.detect_format_changes = false;
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    structure(&accepted),
                    structure(&b),
                    "accepted row/cell counts and source text"
                );
                assert_eq!(
                    structure(&rejected),
                    structure(&a),
                    "rejected row/cell counts and source text"
                );
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
                let pkg = crate::opc::PartFs::open(&compared).unwrap();
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                let root = dom.root(doc).unwrap();
                let rows = dom.descendants(root, Some(&W::tr()));
                assert_eq!(
                    rows.len(),
                    2,
                    "complete authored rows under replacement lifetimes"
                );
                let mut inserted = 0;
                let mut deleted = 0;
                for row in rows {
                    let props = dom.element(row, &W::tr_pr()).unwrap();
                    inserted += dom.elements(props, Some(&W::ins())).len();
                    deleted += dom.elements(props, Some(&W::del())).len();
                }
                assert_eq!((inserted, deleted), (1, 1));
            }
        }
    }

    fn body_paragraph_texts(bytes: &[u8]) -> Vec<String> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        dom.elements(body, Some(&W::p()))
            .into_iter()
            .map(|p| {
                dom.descendants(p, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect()
            })
            .collect()
    }

    /// Per cell: its cell marks, whether it has a `tcPrChange`, its old span
    /// and its text.
    type MeshCell = (Vec<String>, bool, u32, String);
    /// A row's `trPr` revision marks and its cells.
    type MeshRow = (Vec<String>, Vec<MeshCell>);
    /// Each row of the first table, and the old grid's column count.
    fn mesh_rows(dom: &Dom, root: NodeId) -> (Vec<MeshRow>, usize) {
        let tbl = dom.descendants(root, Some(&W::tbl()))[0];
        let grid = dom.element(tbl, &W::name("tblGrid")).unwrap();
        let old_grid = dom
            .element(grid, &W::name("tblGridChange"))
            .and_then(|c| dom.element(c, &W::name("tblGrid")))
            .unwrap_or(grid);
        let old_columns = dom.elements(old_grid, Some(&W::grid_col())).len();
        let span = |pr: Option<NodeId>| {
            pr.and_then(|pr| dom.element(pr, &W::grid_span()))
                .and_then(|s| dom.attribute(s, &W::val()))
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1)
        };
        let marks = |pr: Option<NodeId>, names: &[&str]| -> Vec<String> {
            pr.map(|pr| dom.elements(pr, None))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
                .filter(|n| names.contains(&n.as_str()))
                .collect()
        };
        let rows = dom
            .elements(tbl, Some(&W::tr()))
            .into_iter()
            .map(|tr| {
                let cells = dom
                    .elements(tr, Some(&W::tc()))
                    .into_iter()
                    .map(|tc| {
                        let pr = dom.element(tc, &W::tc_pr());
                        let change = pr.and_then(|pr| dom.element(pr, &W::name("tcPrChange")));
                        let old = change.and_then(|c| dom.element(c, &W::tc_pr())).or(pr);
                        let text = dom
                            .descendants(tc, None)
                            .into_iter()
                            .filter(|&t| dom.name_is(t, &W::t()) || dom.name_is(t, &W::del_text()))
                            .map(|t| dom.value(t))
                            .collect();
                        (
                            marks(pr, &["cellIns", "cellDel", "cellMerge"]),
                            change.is_some(),
                            span(old),
                            text,
                        )
                    })
                    .collect();
                (marks(dom.element(tr, &W::tr_pr()), &["ins", "del"]), cells)
            })
            .collect();
        (rows, old_columns)
    }

    /// The texts of the first table's cells, empty cells left out.
    fn cell_texts(bytes: &[u8]) -> Vec<String> {
        geometry(bytes)
            .into_iter()
            .next()
            .unwrap_or_default()
            .into_iter()
            .flat_map(|(_, cells)| cells.into_iter().map(|(_, text)| text))
            .filter(|text| !text.is_empty())
            .collect()
    }

    /// #1817 — Word mode meshes a horizontal repartition the way Word 16
    /// does (21 Word Compare redlines, jubarte-loop/fix23/ab1817): the rows
    /// pair; the row keeps its larger cell count with no `cellIns`/`cellDel`
    /// (B's extra cell holds inserted text, A's extra cell deleted text);
    /// every cell of the row records its old properties, since Word's Reject
    /// All gives a cell without a `tcPrChange` in such a row width 0 auto;
    /// and no span overflows its side's grid, since Word's Reject All of an
    /// overflowing row re-spans the table's other rows. Accept keeps A's
    /// extra cell empty after a merge, and reject keeps B's after a split, as
    /// Word's own do.
    #[test]
    fn word_mode_meshes_horizontal_repartitions_like_word() {
        let long = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        let two = table(&(cell(1, 2500, "AAAAAA", long) + &cell(1, 2500, "BBBBBB", "Right")));
        let spanning = table(&cell(2, 5000, "CCCCCC", long));
        for (original, revised) in [(&spanning, &two), (&two, &spanning)] {
            let a = package(original);
            let b = package(revised);
            let settings = WmlComparerSettings::default();
            let compared =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .unwrap();
            let pkg = crate::opc::PartFs::open(&compared).unwrap();
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
            let root = dom.root(doc).unwrap();
            let (rows, old_columns) = mesh_rows(&dom, root);
            assert_eq!(rows.len(), 1, "one paired row: {rows:?}");
            let (row_marks, cells) = &rows[0];
            assert!(row_marks.is_empty(), "no row ins/del: {rows:?}");
            assert_eq!(cells.len(), 2, "the larger cell count: {rows:?}");
            assert!(
                cells.iter().all(|(marks, _, _, _)| marks.is_empty()),
                "no cellIns/cellDel: {rows:?}"
            );
            assert!(
                cells.iter().all(|&(_, change, _, _)| change),
                "a tcPrChange on every cell: {rows:?}"
            );
            let old_spans: u32 = cells.iter().map(|&(_, _, span, _)| span).sum();
            assert!(
                old_spans as usize <= old_columns,
                "old spans {old_spans} within the old grid's {old_columns} columns: {rows:?}"
            );
            let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
            let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
            assert_eq!(cell_texts(&accepted), cell_texts(&b), "accepted cell texts");
            assert_eq!(cell_texts(&rejected), cell_texts(&a), "rejected cell texts");
            // The live row fits the live grid too: an overflowing one had the
            // grid rebuilt, and neither side's grid came back.
            assert_eq!(grids(&accepted), grids(&b), "accepted grid");
            assert_eq!(grids(&rejected), grids(&a), "rejected grid");
        }
    }

    #[test]
    fn split_changed_prose_keeps_original_literal_space_and_tracks_its_new_boundary() {
        let original = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
        let first = "The first party shall deliver the complete report";
        let second = "within thirty days after receiving the written request from the other party.";
        let para = |text| format!("<w:p><w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>");
        for with_table in [false, true] {
            let old_table = if with_table {
                table(
                    &(cell(1, 2500, "AAAAAA", "Heading")
                        + &cell(1, 2500, "BBBBBB", "Revised value")),
                )
            } else {
                String::new()
            };
            let a = package(&(para(original) + &old_table));
            let b = package(&(para(first) + &para(second)));
            for reverse in [false, true] {
                let (left, right) = if reverse { (&b, &a) } else { (&a, &b) };
                let compared = crate::document_comparer::compare_documents_with_settings(
                    left,
                    right,
                    &WmlComparerSettings::default(),
                )
                .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    body_paragraph_texts(&accepted),
                    body_paragraph_texts(right),
                    "accepted paragraphs, with_table={with_table}, reverse={reverse}"
                );
                assert_eq!(
                    body_paragraph_texts(&rejected),
                    body_paragraph_texts(left),
                    "rejected original space/boundary, with_table={with_table}, reverse={reverse}"
                );
                assert_eq!(geometry(&accepted), geometry(right));
                assert_eq!(geometry(&rejected), geometry(left));
            }
        }
    }

    #[test]
    fn actual_atomized_split_with_deleted_table_conserves_source_through_production() {
        use super::super::{
            atomize, cross_para, formatchg, lcs, moves, preprocess, produce, units,
        };
        let original = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
        let first = "The first party shall deliver the complete report";
        let second = "within thirty days after receiving the written request from the other party.";
        let paragraph = |t| format!("<w:p><w:r><w:t xml:space=\"preserve\">{t}</w:t></w:r></w:p>");
        let mut dom = Dom::new();
        let mut bodies = Vec::new();
        for fragment in [
            paragraph(original)
                + &table(
                    &(cell(1, 2500, "AAAAAA", "Heading")
                        + &cell(1, 2500, "BBBBBB", "Revised value")),
                ),
            paragraph(first) + &paragraph(second),
        ] {
            let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\"><w:body>{fragment}<w:p/><w:sectPr/></w:body></w:document>",W::URI));
            let root = dom.root(doc).unwrap();
            bodies.push(dom.element(root, &W::body()).unwrap());
        }
        let settings = WmlComparerSettings::default();
        let mut source_atoms = Vec::new();
        let mut source_units = Vec::new();
        for &body in &bodies {
            crate::unid::assign_to_all_elements(&mut dom, body);
            let _ = preprocess::hash_block_level_content(
                &mut dom,
                body,
                body,
                &settings,
                &preprocess::null_rel_resolver,
            );
            preprocess::add_sha1_hash_to_block_level_content(
                &mut dom,
                body,
                &settings,
                &preprocess::null_rel_resolver,
            );
            let atoms = atomize::create_comparison_unit_atom_list(&mut dom, body, &settings);
            source_units.push(units::get_comparison_unit_list(&dom, &atoms, &settings));
            source_atoms.push(atoms);
        }
        let text = |dom: &Dom, atoms: &[super::super::atoms::ComparisonUnitAtom]| -> String {
            atoms
                .iter()
                .filter(|a| dom.name_is(a.content_element, &W::t()))
                .map(|a| dom.value(a.content_element))
                .collect()
        };
        let expected = text(&dom, &source_atoms[0]);
        let before = source_units.remove(0);
        let after = source_units.remove(0);
        let mut seqs = lcs::lcs(&mut dom, before, after, &settings);
        let check = |dom: &Dom, seqs: &[CorrelatedSequence], stage| {
            let left: Vec<_> = seqs
                .iter()
                .flat_map(|s| s.com_units_1.iter().flatten())
                .flat_map(ComparisonUnit::descendant_atoms)
                .cloned()
                .collect();
            assert_eq!(text(dom, &left), expected, "source text after {stage}");
        };
        check(&dom, &seqs, "actual LCS");
        lcs::pair_story_final_marks(&dom, &mut seqs);
        check(&dom, &seqs, "final mark pairing");
        cross_para::restream_cross_paragraph_regions(&mut dom, &mut seqs, &settings);
        check(&dom, &seqs, "cross-paragraph restream");
        moves::promote_skip_ahead_equals(&mut seqs, &settings);
        check(&dom, &seqs, "move promotion");
        let mut id = 1;
        mark_rows_as_deleted_or_inserted(&mut dom, &settings, &seqs, &mut id);
        let mut flat = produce::flatten_to_comparison_unit_atom_list(&dom, &seqs);
        let rejected_text =
            |dom: &Dom, flat: &[super::super::atoms::ComparisonUnitAtom]| -> String {
                flat.iter()
                    .filter_map(|a| {
                        let a = match a.correlation_status {
                            CorrelationStatus::Inserted | CorrelationStatus::MovedDestination => {
                                return None;
                            }
                            CorrelationStatus::Equal => a.comparison_unit_atom_before.as_deref()?,
                            _ => a,
                        };
                        dom.name_is(a.content_element, &W::t())
                            .then(|| dom.value(a.content_element))
                    })
                    .collect()
            };
        assert_eq!(
            rejected_text(&dom, &flat),
            expected,
            "source text after actual flatten"
        );
        moves::detect_moves_in_atom_list(&dom, &mut flat, &settings);
        assert_eq!(
            rejected_text(&dom, &flat),
            expected,
            "source text after move detection"
        );
        formatchg::detect_format_changes_in_atom_list(&mut dom, &mut flat, &settings);
        produce::assemble_ancestor_unids(&mut dom, &mut flat);
        let children = produce::produce_new_wml_markup_from_correlated_sequence(
            &mut dom, &flat, &settings, &mut id,
        );
        let root = dom.new_element(W::document());
        let body = dom.new_element(W::body());
        dom.add(root, body);
        for child in children {
            dom.add(body, child);
        }
        let root = super::super::finalize::mark_content_as_deleted_or_inserted(
            &mut dom, root, &settings, &mut id,
        );
        let copy = dom.clone_subtree(root);
        let rejected = crate::revision_processor::reject_revisions_document(&mut dom, copy);
        let text: String = dom
            .descendants(rejected, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        assert_eq!(
            text, expected,
            "source text after markup production and revision marking"
        );
    }

    #[test]
    fn justified_underline_peel_preserves_both_sources_and_word_three_mix_shape() {
        let paragraph = |text: &str| format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>");
        let a_body = paragraph("Justified Underline Demo")
            + &paragraph(
                "This document combines justify alignment with underline formatting for a formal document look.",
            );
        let b_body = paragraph("Justify Alignment Demo")
            + &paragraph("This document demonstrates justified text alignment.")
            + &paragraph("Justified text spreads evenly across the full width of the line.");
        let source_package = |body: &str| {
            let mut pkg = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/relids/image_doc.docx"
            ))
            .unwrap();
            pkg.set_part(
                "word/document.xml",
                format!(
                    "<w:document xmlns:w=\"{}\"><w:body>{body}<w:sectPr/></w:body></w:document>",
                    W::URI
                )
                .into_bytes(),
            );
            pkg.to_zip().unwrap()
        };
        let a = source_package(&a_body);
        let b = source_package(&b_body);
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let compared =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .unwrap();
            let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
            let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
            let pkg = crate::opc::PartFs::open(&compared).unwrap();
            let xml = pkg.part_string("word/document.xml").unwrap();
            assert_eq!(
                body_paragraph_texts(&accepted),
                body_paragraph_texts(&b),
                "accepted authored family paragraphs"
            );
            assert_eq!(
                body_paragraph_texts(&rejected),
                body_paragraph_texts(&a),
                "rejected authored family paragraphs"
            );
            if settings.merge_replaced_paragraphs {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&xml);
                let root = dom.root(doc).unwrap();
                let body = dom.element(root, &W::body()).unwrap();
                let paragraphs = dom.elements(body, Some(&W::p()));
                assert_eq!(paragraphs.len(), 3);
                for p in paragraphs {
                    assert!(
                        !dom.elements(p, Some(&W::ins())).is_empty(),
                        "Word MIX keeps revised runs"
                    );
                    assert!(
                        !dom.elements(p, Some(&W::del())).is_empty(),
                        "Word MIX keeps original runs"
                    );
                }
            }
        }
    }

    fn row_unit(dom: &mut Dom, cells: &str) -> ComparisonUnit {
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{}</w:body></w:document>",table(cells)));
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let settings = WmlComparerSettings::default();
        let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, &settings);
        let units = super::super::units::get_comparison_unit_list(dom, &atoms, &settings);
        as_group(&units[0]).unwrap().contents[0].clone()
    }
    #[test]
    fn same_horizontal_partition_keeps_vertical_merge_pairing_and_normalizes_zero_span() {
        let mut dom = Dom::new();
        let a = row_unit(&mut dom, &cell(1, 2500, "AAAAAA", "Alpha"));
        let b = row_unit(
            &mut dom,
            &cell(0, 2500, "BBBBBB", "Beta")
                .replace("</w:tcPr>", "<w:vMerge w:val=\"restart\"/></w:tcPr>"),
        );
        assert_eq!(horizontal_cell_partition(&dom, &a), Some(vec![1]));
        assert_eq!(horizontal_cell_partition(&dom, &b), Some(vec![1]));
        assert!(rows_preserving_horizontal_partitions(&dom, &[a], &[b]).is_none());
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod nested_lifecycle_owner_regressions {
    use super::*;

    fn group_owned_by(
        units: &[ComparisonUnit],
        owner: NodeId,
        kind: ComparisonUnitGroupType,
    ) -> Option<ComparisonUnit> {
        for unit in units {
            let Some(group) = as_group(unit) else {
                continue;
            };
            let atoms = unit.descendant_atoms();
            if group.group_type == kind
                && !atoms.is_empty()
                && atoms
                    .iter()
                    .all(|atom| atom.ancestor_elements.contains(&owner))
            {
                return Some(unit.clone());
            }
            if let Some(found) = group_owned_by(&group.contents, owner, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn lifecycle_units_mark_their_own_rows_even_when_the_first_atom_is_nested() {
        for table_unit in [false, true] {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}"><w:body><w:tbl><w:tr><w:tc><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Inner first</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Inner second</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>Outer tail</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#, W::URI));
            let root = dom.root(doc).unwrap();
            let body = dom.element(root, &W::body()).unwrap();
            let outer = dom.elements(body, Some(&W::tbl()))[0];
            let outer_row = dom.elements(outer, Some(&W::tr()))[0];
            let inner = dom.descendants(outer, Some(&W::tbl()))[0];
            let inner_rows = dom.elements(inner, Some(&W::tr()));
            let settings = WmlComparerSettings::powertools_faithful();
            let atoms =
                super::super::atomize::create_comparison_unit_atom_list(&mut dom, body, &settings);
            let units = super::super::units::get_comparison_unit_list(&dom, &atoms, &settings);
            let (owner, kind) = if table_unit {
                (inner, ComparisonUnitGroupType::Table)
            } else {
                (outer_row, ComparisonUnitGroupType::Row)
            };
            let unit = group_owned_by(&units, owner, kind).expect("production lifecycle group");
            let mut next_id = 100;
            mark_rows_as_deleted_or_inserted(
                &mut dom,
                &settings,
                &[CorrelatedSequence::inserted(vec![unit])],
                &mut next_id,
            );
            assert_eq!(next_id, if table_unit { 102 } else { 101 });
            for row in std::iter::once(outer_row).chain(inner_rows) {
                let insertion = dom
                    .element(row, &W::tr_pr())
                    .and_then(|pr| dom.element(pr, &W::ins()));
                assert_eq!(
                    insertion.is_some(),
                    if table_unit {
                        row != outer_row
                    } else {
                        row == outer_row
                    }
                );
                if let Some(mark) = insertion {
                    assert_eq!(
                        dom.attribute(mark, &W::author()),
                        Some(settings.author_for_revisions.as_str())
                    );
                    assert_eq!(
                        dom.attribute(mark, &W::date()),
                        Some(settings.date_time_for_revisions.as_str())
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod horizontal_gate_by_mode {
    use super::*;
    use crate::namespaces::PT;

    fn table(rows: &[Vec<(u32, &str)>]) -> String {
        let mut xml = String::from(
            "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"1000\"/><w:gridCol w:w=\"1000\"/><w:gridCol w:w=\"1000\"/><w:gridCol w:w=\"1000\"/></w:tblGrid>",
        );
        for row in rows {
            xml.push_str("<w:tr>");
            for &(span, merge) in row {
                let merged = if merge.is_empty() {
                    String::new()
                } else {
                    format!("<w:vMerge w:val=\"{merge}\"/>")
                };
                let span = if span == 1 {
                    String::new()
                } else {
                    format!("<w:gridSpan w:val=\"{span}\"/>")
                };
                xml.push_str(&format!("<w:tc><w:tcPr>{span}{merged}</w:tcPr><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc>"));
            }
            xml.push_str("</w:tr>");
        }
        xml.push_str("</w:tbl>");
        xml
    }

    fn units(dom: &mut Dom, body: &str, prefix: &str) -> Vec<ComparisonUnit> {
        let document = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\"><w:body>{body}</w:body></w:document>",
            W::URI
        ));
        let root = dom.root(document).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        for (index, table) in dom.elements(body, Some(&W::tbl())).into_iter().enumerate() {
            dom.set_attribute_value(table, &PT::unid(), Some(&format!("{prefix}{index}")));
        }
        let settings = WmlComparerSettings::default();
        let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, &settings);
        super::super::units::get_comparison_unit_list(dom, &atoms, &settings)
    }

    #[test]
    /// Every repartition, M42's package geometry or not: whole rows outside
    /// Word mode, Word's mesh in it (#1817).
    fn word_mode_meshes_every_horizontal_repartition_and_the_others_keep_rows() {
        for mutation in 0..4 {
            let mut dom = Dom::new();
            let original: String = [
                (3, 3),
                (3, 4),
                (2, 2),
                (2, 3),
                (3, 5),
                (3, 3),
                (3, 3),
                (3, 3),
            ]
            .into_iter()
            .map(|(rows, cells)| table(&vec![vec![(1, ""); cells]; rows]))
            .collect();
            let mut revised_rows = vec![
                vec![(2, "")],
                vec![(1, "restart"), (1, "")],
                vec![(1, "continue"), (1, "")],
                vec![(1, ""), (1, "restart")],
                vec![(1, ""), (1, "continue")],
            ];
            if mutation == 1 {
                revised_rows[0][0].0 = 1;
            }
            if mutation == 2 {
                revised_rows[2][0].1 = "restart";
            }
            let mut revised = table(&revised_rows);
            revised.push_str(&table(&[
                vec![(1, ""); 4],
                vec![(1, ""), (2, "restart"), (1, "")],
                vec![(1, ""), (2, "continue"), (1, "")],
                vec![(1, ""); 4],
            ]));
            if mutation == 3 {
                revised.push_str(&table(&[vec![(1, "")]]));
            }
            let a = units(&mut dom, &original, "a");
            let b = units(&mut dom, &revised, "b");
            let rows_a = &as_group(&a[0]).unwrap().contents;
            let rows_b = &as_group(&b[0]).unwrap().contents;
            assert!(
                rows_preserving_horizontal_partitions_for(
                    &dom,
                    rows_a,
                    rows_b,
                    &WmlComparerSettings::powertools_faithful()
                )
                .is_some(),
                "powertools keeps whole rows (mutation {mutation})"
            );
            assert!(
                rows_preserving_horizontal_partitions_for(
                    &dom,
                    rows_a,
                    rows_b,
                    &WmlComparerSettings::default()
                )
                .is_none(),
                "Word mode meshes (mutation {mutation})"
            );
        }
    }

    /// A table of text rows, one cell per string.
    fn text_table(rows: &[&[&str]]) -> String {
        let mut xml = String::from(
            "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid>",
        );
        for row in rows {
            xml.push_str("<w:tr>");
            for text in *row {
                xml.push_str(&format!(
                    "<w:tc><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"
                ));
            }
            xml.push_str("</w:tr>");
        }
        xml.push_str("</w:tbl>");
        xml
    }

    /// The rows of every table in `body`, hashed as the comparer hashes them,
    /// then flattened as H4 flattens them.
    fn flat_rows(dom: &mut Dom, body: &str) -> Vec<ComparisonUnit> {
        use crate::comparer::{atomize, preprocess, units};
        let document = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\"><w:body>{body}</w:body></w:document>",
            W::URI
        ));
        let root = dom.root(document).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let settings = WmlComparerSettings::default();
        crate::unid::assign_to_all_elements(dom, body);
        preprocess::hash_block_level_content(
            dom,
            body,
            body,
            &settings,
            &preprocess::null_rel_resolver,
        )
        .unwrap();
        preprocess::add_sha1_hash_to_block_level_content(
            dom,
            body,
            &settings,
            &preprocess::null_rel_resolver,
        );
        let atoms = atomize::create_comparison_unit_atom_list(dom, body, &settings);
        units::get_comparison_unit_list(dom, &atoms, &settings)
            .iter()
            .filter_map(as_group)
            .filter(|g| g.group_type == ComparisonUnitGroupType::Table)
            .flat_map(|g| g.contents.clone())
            .collect()
    }

    /// Each sequence's status and its original and revised row counts.
    fn shape(sequences: &[CorrelatedSequence]) -> Vec<(CorrelationStatus, usize, usize)> {
        sequences
            .iter()
            .map(|s| {
                (
                    s.correlation_status,
                    s.com_units_1.as_ref().map_or(0, Vec::len),
                    s.com_units_2.as_ref().map_or(0, Vec::len),
                )
            })
            .collect()
    }

    #[test]
    /// #1817 — rows flattened out of several tables pair table by table in
    /// Word mode: a table with the revised table that shares its rows (a new
    /// table inserted right above an edited one), with the first one when
    /// none shares a row (w23), and the others are inserted whole.
    fn word_mode_pairs_flattened_rows_table_by_table() {
        use CorrelationStatus::{Inserted, Unknown};
        let parties: &[&[&str]] = &[
            &["Party", "Role"],
            &["Alice Example", "Seller"],
            &["Bob Example", "Buyer"],
        ];
        let edited: &[&[&str]] = &[
            &["Party", "Role"],
            &["Alice Example", "Seller"],
            &["Bob Example", "Purchaser"],
        ];
        let schedule: &[&[&str]] = &[&["Schedule", "Date"], &["Closing", "Friday"]];
        let summary: &[&[&str]] = &[&["Summary of the transaction"]];
        let mut dom = Dom::new();
        let original = flat_rows(&mut dom, &text_table(parties));
        let above = flat_rows(&mut dom, &(text_table(schedule) + &text_table(edited)));
        let unrelated = flat_rows(&mut dom, &(text_table(summary) + &text_table(schedule)));
        let one = flat_rows(&mut dom, &text_table(edited));
        let word = WmlComparerSettings::default();
        assert_eq!(
            shape(&rows_by_table(&dom, &original, &above, &word).unwrap()),
            [(Inserted, 0, 2), (Unknown, 3, 3)],
            "the table sharing rows pairs"
        );
        assert_eq!(
            shape(&rows_by_table(&dom, &original, &unrelated, &word).unwrap()),
            [(Unknown, 3, 1), (Inserted, 0, 2)],
            "the first table pairs when none shares a row"
        );
        assert!(
            rows_by_table(&dom, &original, &one, &word).is_none(),
            "one table a side"
        );
        let powertools = WmlComparerSettings::powertools_faithful();
        assert!(rows_by_table(&dom, &original, &above, &powertools).is_none());
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod deterministic_table_dispatch_tests {
    use super::*;
    use crate::namespaces::PT;

    fn source(dom: &mut Dom, labels: &[&str], merge: Option<&str>) -> ComparisonUnit {
        let rows: String = labels.iter().enumerate().map(|(index, label)| {
            let merged = if index == 0 {
                merge.map(|merge| format!("<w:vMerge w:val=\"{merge}\"/>")).unwrap_or_default()
            } else { String::new() };
            format!("<w:tr><w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/>{merged}</w:tcPr><w:p><w:r><w:t>{label}</w:t></w:r></w:p></w:tc></w:tr>")
        }).collect();
        let document = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\"><w:body><w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"2400\"/></w:tblGrid>{rows}</w:tbl></w:body></w:document>", W::URI));
        let root = dom.root(document).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let settings = WmlComparerSettings::powertools_faithful();
        super::super::preprocess::add_sha1_hash_to_block_level_content(
            dom,
            body,
            &settings,
            &super::super::preprocess::null_rel_resolver,
        );
        let table = dom.element(body, &W::tbl()).unwrap();
        // The dispatch can receive positionally correlated, content-different
        // rows. All rows here share the same one-cell structural ownership.
        for row in dom.elements(table, Some(&W::tr())) {
            dom.set_attribute_value(
                row,
                &PT::correlated_sha1_hash(),
                Some("one-cell-aligned-row"),
            );
        }
        let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, &settings);
        let units = super::super::units::get_comparison_unit_list(dom, &atoms, &settings);
        units
            .into_iter()
            .find(|unit| {
                as_group(unit)
                    .is_some_and(|group| group.group_type == ComparisonUnitGroupType::Table)
            })
            .unwrap()
    }
    fn atoms(units: &[ComparisonUnit]) -> Vec<(NodeId, Vec<NodeId>)> {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|atom| (atom.content_element, atom.ancestor_elements.to_vec()))
            .collect()
    }
    fn assert_sources(
        out: &[CorrelatedSequence],
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
    ) {
        let a: Vec<_> = out
            .iter()
            .flat_map(|sequence| sequence.com_units_1.iter().flatten())
            .cloned()
            .collect();
        let b: Vec<_> = out
            .iter()
            .flat_map(|sequence| sequence.com_units_2.iter().flatten())
            .cloned()
            .collect();
        assert_eq!(
            atoms(&a),
            atoms(left),
            "every original payload and paragraph mark stays ordered under its own row/cell"
        );
        assert_eq!(
            atoms(&b),
            atoms(right),
            "every revised payload and paragraph mark stays ordered under its own row/cell"
        );
    }

    #[test]
    fn row_lcs_ties_and_both_one_sided_remainders_conserve_actual_atom_ownership() {
        for (left_labels, right_labels, statuses) in [
            (
                vec!["A", "B"],
                vec!["B", "A"],
                vec![
                    CorrelationStatus::Deleted,
                    CorrelationStatus::Unknown,
                    CorrelationStatus::Inserted,
                ],
            ),
            (
                vec!["A", "B", "C"],
                vec!["A", "C"],
                vec![
                    CorrelationStatus::Unknown,
                    CorrelationStatus::Deleted,
                    CorrelationStatus::Unknown,
                ],
            ),
            (
                vec!["A", "C"],
                vec!["A", "B", "C"],
                vec![
                    CorrelationStatus::Unknown,
                    CorrelationStatus::Inserted,
                    CorrelationStatus::Unknown,
                ],
            ),
            (
                vec!["A"],
                vec!["B"],
                vec![CorrelationStatus::Deleted, CorrelationStatus::Inserted],
            ),
        ] {
            let mut dom = Dom::new();
            let a = source(&mut dom, &left_labels, None);
            let b = source(&mut dom, &right_labels, None);
            let left = &as_group(&a).unwrap().contents;
            let right = &as_group(&b).unwrap().contents;
            let out = apply_lcs_to_table_rows(left, right);
            assert_eq!(
                out.iter()
                    .map(|sequence| sequence.correlation_status)
                    .collect::<Vec<_>>(),
                statuses
            );
            assert_sources(&out, left, right);
            for sequence in out
                .iter()
                .filter(|sequence| sequence.correlation_status == CorrelationStatus::Unknown)
            {
                assert_eq!(
                    sequence.com_units_1.as_ref().unwrap()[0].sha1(),
                    sequence.com_units_2.as_ref().unwrap()[0].sha1()
                );
            }
            for (one_side, inserted) in [(left, false), (right, true)] {
                let out = if inserted {
                    apply_lcs_to_table_rows(&[], one_side)
                } else {
                    apply_lcs_to_table_rows(one_side, &[])
                };
                assert_sources(
                    &out,
                    if inserted { &[] } else { one_side },
                    if inserted { one_side } else { &[] },
                );
                assert!(out.iter().all(|sequence| sequence.correlation_status
                    == if inserted {
                        CorrelationStatus::Inserted
                    } else {
                        CorrelationStatus::Deleted
                    }));
            }
        }
        assert!(apply_lcs_to_table_rows(&[], &[]).is_empty());
    }

    #[test]
    fn seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs() {
        for count in [6, 7] {
            let mut dom = Dom::new();
            let a = source(
                &mut dom,
                &["A", "B", "C", "D", "E", "F", "G"][..count],
                None,
            );
            let b = source(
                &mut dom,
                &["A", "X", "B", "C", "Y", "F", "G"][..count],
                None,
            );
            let out = do_lcs_algorithm_for_table(
                &dom,
                std::slice::from_ref(&a),
                std::slice::from_ref(&b),
                &WmlComparerSettings::powertools_faithful(),
            )
            .unwrap();
            assert_sources(
                &out,
                &as_group(&a).unwrap().contents,
                &as_group(&b).unwrap().contents,
            );
            if count == 7 {
                assert!(
                    out.iter()
                        .any(|sequence| sequence.correlation_status == CorrelationStatus::Deleted)
                );
                assert!(
                    out.iter()
                        .any(|sequence| sequence.correlation_status == CorrelationStatus::Inserted)
                );
            } else {
                assert_eq!(out.len(), 6);
                assert!(
                    out.iter()
                        .all(|sequence| sequence.correlation_status == CorrelationStatus::Unknown)
                );
            }
        }
    }

    #[test]
    fn vertical_merge_structure_dispatch_preserves_same_partition_and_each_remainder() {
        for longer_original in [false, true] {
            for faithful in [false, true] {
                let mut dom = Dom::new();
                let left_labels = if longer_original {
                    vec!["A", "B", "C"]
                } else {
                    vec!["A", "B"]
                };
                let right_labels = if longer_original {
                    vec!["X", "Y"]
                } else {
                    vec!["X", "Y", "Z"]
                };
                let a = source(&mut dom, &left_labels, Some("restart"));
                let b = source(&mut dom, &right_labels, None);
                let settings = if faithful {
                    WmlComparerSettings::powertools_faithful()
                } else {
                    WmlComparerSettings::default()
                };
                let out = do_lcs_algorithm_for_table(
                    &dom,
                    std::slice::from_ref(&a),
                    std::slice::from_ref(&b),
                    &settings,
                )
                .unwrap();
                let left = &as_group(&a).unwrap().contents;
                let right = &as_group(&b).unwrap().contents;
                assert_sources(&out, left, right);
                if faithful {
                    assert_eq!(
                        out.iter()
                            .map(|sequence| sequence.correlation_status)
                            .collect::<Vec<_>>(),
                        [CorrelationStatus::Deleted, CorrelationStatus::Inserted]
                    );
                } else {
                    assert_eq!(
                        out.iter()
                            .filter(|sequence| sequence.correlation_status
                                == CorrelationStatus::Unknown)
                            .count(),
                        2
                    );
                    assert_eq!(
                        out.last().unwrap().correlation_status,
                        if longer_original {
                            CorrelationStatus::Deleted
                        } else {
                            CorrelationStatus::Inserted
                        }
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod shared_word_alignment_tests {
    use super::*;
    use std::collections::HashSet;

    fn toks(words: &[&str]) -> HashSet<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    fn rows(lists: &[&[&str]]) -> Vec<HashSet<String>> {
        lists.iter().map(|words| toks(words)).collect()
    }

    use WordRowOp::{Delete, Insert, Pair};

    /// Task 22 first table: 3 old rows against 21 new, anchors at new 0/3/7
    /// (here 0/3/7 of 8). Gaps fill positionally around the anchors.
    #[test]
    fn anchors_win_over_positional_heads_with_inserted_gap_rows() {
        let old = rows(&[&["rhythmia"], &["left", "atrium"], &["right", "atrium"]]);
        let new = rows(&[
            &["motivation"],
            &["contribution"],
            &["post"],
            &["atrium", "skills"],
            &["post"],
            &["build"],
            &["post"],
            &["atrium", "range"],
        ]);
        // `atrium` is the only shared token: old row 1 anchors new 3, old
        // row 2 anchors new 7; the header pair comes from gap fill.
        assert_eq!(
            align_rows_by_shared_words(&old, &new),
            vec![
                Pair(0, 0),
                Insert(1),
                Insert(2),
                Pair(1, 3),
                Insert(4),
                Insert(5),
                Insert(6),
                Pair(2, 7),
            ]
        );
    }

    /// No shared token at all: heads pair positionally, the rest inserts.
    #[test]
    fn zero_score_tables_pair_heads_positionally() {
        let old = rows(&[&["february"], &["sat"]]);
        let new = rows(&[&["nev"], &["beosztas"], &["mobilitas"]]);
        assert_eq!(
            align_rows_by_shared_words(&old, &new),
            vec![Pair(0, 0), Pair(1, 1), Insert(2)]
        );
    }

    /// Deleting the old row beats pairing it when both reach the optimum
    /// (Task 22 second table tail: A18 stays deleted, B9 keeps A14).
    #[test]
    fn deletion_wins_ties_against_pairing() {
        let old = rows(&[&["m"], &["n"], &["n"]]);
        let new = rows(&[&["m"], &["n"]]);
        assert_eq!(
            align_rows_by_shared_words(&old, &new),
            vec![Pair(0, 0), Pair(1, 1), Delete(2)]
        );
    }

    /// Symmetric inference (no direct Word case): inserting the new row
    /// beats pairing it on a tie.
    #[test]
    fn insertion_wins_ties_against_pairing() {
        let old = rows(&[&["m"], &["n"]]);
        let new = rows(&[&["m"], &["n"], &["n"]]);
        assert_eq!(
            align_rows_by_shared_words(&old, &new),
            vec![Pair(0, 0), Pair(1, 1), Insert(2)]
        );
    }

    /// A higher-weight anchor pulls the alignment even when an earlier
    /// single-share pairing is available (Task 22: A10 takes B5 for 2).
    #[test]
    fn heavier_anchor_pull_beats_earlier_single() {
        let old = rows(&[&["a", "solo"], &["solo", "double", "trouble"]]);
        let new = rows(&[&["solo"], &["double", "trouble"]]);
        assert_eq!(
            align_rows_by_shared_words(&old, &new),
            vec![Pair(0, 0), Pair(1, 1)]
        );
    }
    #[test]
    fn row_tokens_keep_cell_paragraph_and_tab_boundaries_but_join_run_fragments() {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{}"><w:body><w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="1000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Al</w:t></w:r><w:r><w:t>pha</w:t></w:r></w:p><w:p><w:r><w:t>Beta</w:t><w:tab/><w:t>Gamma</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Delta</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#, W::URI
        ));
        let root = dom.root(document).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let settings = WmlComparerSettings::default();
        let atoms =
            super::super::atomize::create_comparison_unit_atom_list(&mut dom, body, &settings);
        let units = super::super::units::get_comparison_unit_list(&dom, &atoms, &settings);
        let table = as_group(&units[0]).unwrap();
        assert_eq!(
            row_word_tokens(&dom, &table.contents[0]),
            toks(&["alpha", "beta", "gamma", "delta"])
        );
    }
}
