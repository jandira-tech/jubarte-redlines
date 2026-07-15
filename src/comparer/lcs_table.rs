//! M4.D — table/row/cell LCS: `ApplyLcsToTableRows` (:8241),
//! `DoLcsAlgorithmForTable` (:8348), `MarkRowsAsDeletedOrInserted` (:4216).

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

/// M4.D.1 — `ApplyLcsToTableRows` (:8241): classic DP LCS over rows keyed on
/// `sha1`, emitting Deleted/Inserted/Unknown in document order.
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

fn contains_merged(dom: &Dom, tbl: NodeId) -> bool {
    !dom.descendants(tbl, Some(&W::name("vMerge"))).is_empty()
        || !dom.descendants(tbl, Some(&W::name("gridSpan"))).is_empty()
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

/// M4.D.2/D.3 — `DoLcsAlgorithmForTable` (:8348): single-table-vs-single-table.
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
    let tbl_name = W::name("tbl");
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
/// if absent). Only acts on Row groups (never a whole Table — invariant).
pub fn mark_rows_as_deleted_or_inserted(
    dom: &mut Dom,
    settings: &WmlComparerSettings,
    sequences: &[CorrelatedSequence],
    next_id: &mut u32,
) {
    let tr_name = W::name("tr");
    for cs in sequences {
        let (units, rev_name) = match cs.correlation_status {
            CorrelationStatus::Deleted => (cs.com_units_1.as_ref(), W::del()),
            CorrelationStatus::Inserted => (cs.com_units_2.as_ref(), W::ins()),
            _ => continue,
        };
        let Some(units) = units else { continue };
        for unit in units {
            let Some(g) = as_group(unit) else { continue };
            if g.group_type != ComparisonUnitGroupType::Row {
                continue;
            }
            let Some(tr) = ancestor_named(dom, g, &tr_name) else {
                continue;
            };
            let trpr = match dom.element(tr, &W::name("trPr")) {
                Some(p) => p,
                None => {
                    let p = dom.new_element(W::name("trPr"));
                    dom.add_first(tr, p);
                    p
                }
            };
            let rev = dom.new_element(rev_name.clone());
            dom.set_attribute_value(rev, &W::author(), Some(&settings.author_for_revisions));
            dom.set_attribute_value(rev, &W::id(), Some(&next_id.to_string()));
            *next_id += 1;
            dom.set_attribute_value(rev, &W::date(), Some(&settings.date_time_for_revisions));
            dom.add(trpr, rev);
        }
    }
}
