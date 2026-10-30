// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word pairs the tables of a replaced region in order (#1817).
//!
//! Word's Compare diffs one flat token stream in which a table's cell and row
//! marks are tokens like any word. Where a region of the original gives way
//! to a region of the revision, the two regions' tables share those marks,
//! so Word pairs the original's k-th table with the revision's k-th table
//! and diffs them row by row, whatever their text. The LCS here matches a
//! table only with an equal table, so such a region came out as the
//! revision's tables inserted whole and the original's deleted whole; Word
//! 16.115 crashed on some of those deleted rows (w12, the empty one-row form
//! boxes).
//!
//! Word pairs tables in all 72 comparisons of the bench corpus whose
//! documents both hold two or more tables, the k-th with the k-th from the
//! region's start (fix23 round 31).

use super::atoms::{ComparisonUnit, CorrelatedSequence};
use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerSettings, lcs};
use crate::xmllinq::Dom;

/// One piece of a rebuilt region, in order: so many of the revision's or
/// the original's next units, or the next table of each paired.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    Inserted(usize),
    Deleted(usize),
    Paired,
}

fn group_type(u: &ComparisonUnit) -> Option<ComparisonUnitGroupType> {
    match u {
        ComparisonUnit::Group(g) => Some(g.group_type),
        ComparisonUnit::Word(_) => None,
    }
}

fn is_block(u: &ComparisonUnit) -> bool {
    matches!(
        group_type(u),
        Some(ComparisonUnitGroupType::Paragraph | ComparisonUnitGroupType::Table)
    )
}

/// The rebuilt region: before each table pair the revision's blocks, then
/// the original's (Word writes a replaced region revision first), then the
/// pair; after the last pair the rest the same way. `None` leaves the region
/// as it is: no table on one side, or a unit the reordering could misplace.
/// Only whole paragraphs and tables may precede the last pair; loose words
/// (a paragraph whose mark pairs past the region) only end the rebuilt
/// region, where they still end their paragraph.
fn plan(deleted: &[&ComparisonUnit], inserted: &[&ComparisonUnit]) -> Option<Vec<Step>> {
    let tables = |units: &[&ComparisonUnit]| -> Vec<usize> {
        (0..units.len())
            .filter(|&i| group_type(units[i]) == Some(ComparisonUnitGroupType::Table))
            .collect()
    };
    let (td, ti) = (tables(deleted), tables(inserted));
    let pairs = td.len().min(ti.len());
    if pairs == 0 {
        return None;
    }
    let (dl, il) = (td[pairs - 1] + 1, ti[pairs - 1] + 1);
    let words_end_tail = inserted[il..]
        .iter()
        .chain(&deleted[dl..])
        .skip_while(|u| is_block(u))
        .all(|u| matches!(u, ComparisonUnit::Word(_)));
    if !words_end_tail
        || !deleted[..dl]
            .iter()
            .chain(&inserted[..il])
            .all(|u| is_block(u))
    {
        return None;
    }
    let mut steps = Vec::new();
    let (mut d, mut i) = (0, 0);
    for (&dt, &it) in td.iter().zip(&ti) {
        if i < it {
            steps.push(Step::Inserted(it - i));
        }
        if d < dt {
            steps.push(Step::Deleted(dt - d));
        }
        steps.push(Step::Paired);
        (d, i) = (dt + 1, it + 1);
    }
    if i < inserted.len() {
        steps.push(Step::Inserted(inserted.len() - i));
    }
    if d < deleted.len() {
        steps.push(Step::Deleted(deleted.len() - d));
    }
    Some(steps)
}

/// Rebuilds each replaced region of a body's correlated sequences (a run of
/// deletions and insertions between kept content) whose two sides hold
/// tables, pairing them in order and diffing each pair as one table.
pub fn pair_replaced_tables(
    dom: &mut Dom,
    seqs: &mut Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) {
    let mut out = Vec::with_capacity(seqs.len());
    let mut region = Vec::new();
    for s in std::mem::take(seqs) {
        if matches!(
            s.correlation_status,
            CorrelationStatus::Deleted | CorrelationStatus::Inserted
        ) {
            region.push(s);
        } else {
            rebuild(dom, std::mem::take(&mut region), &mut out, settings);
            out.push(s);
        }
    }
    rebuild(dom, region, &mut out, settings);
    *seqs = out;
}

fn rebuild(
    dom: &mut Dom,
    region: Vec<CorrelatedSequence>,
    out: &mut Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) {
    let units = |status| -> Vec<&ComparisonUnit> {
        region
            .iter()
            .filter(|s| s.correlation_status == status)
            .flat_map(|s| match status {
                CorrelationStatus::Deleted => s.com_units_1.iter().flatten(),
                _ => s.com_units_2.iter().flatten(),
            })
            .collect()
    };
    let Some(steps) = plan(
        &units(CorrelationStatus::Deleted),
        &units(CorrelationStatus::Inserted),
    ) else {
        out.extend(region);
        return;
    };
    let (mut deleted, mut inserted) = (Vec::new(), Vec::new());
    for s in region {
        deleted.extend(s.com_units_1.unwrap_or_default());
        inserted.extend(s.com_units_2.unwrap_or_default());
    }
    let (mut deleted, mut inserted) = (deleted.into_iter(), inserted.into_iter());
    for step in steps {
        match step {
            Step::Inserted(n) => out.push(CorrelatedSequence::inserted(
                inserted.by_ref().take(n).collect(),
            )),
            Step::Deleted(n) => out.push(CorrelatedSequence::deleted(
                deleted.by_ref().take(n).collect(),
            )),
            Step::Paired => {
                if let (Some(a), Some(b)) = (deleted.next(), inserted.next()) {
                    out.extend(lcs::resolve_correlated_sequences(
                        dom,
                        vec![CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            vec![a],
                            vec![b],
                        )],
                        settings,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod plan_tests {
    use super::*;
    use crate::comparer::atoms::{
        ComparisonUnitAtom, ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed,
    };
    use crate::xmllinq::NodeId;

    /// `P` a paragraph, `T` a table, `w` a loose word.
    fn units(shape: &str) -> Vec<ComparisonUnit> {
        shape
            .chars()
            .map(|c| {
                let atom = ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), c.to_string());
                let word = ComparisonUnitWord::new(vec![atom]);
                let group_type = match c {
                    'P' => ComparisonUnitGroupType::Paragraph,
                    'T' => ComparisonUnitGroupType::Table,
                    _ => return ComparisonUnit::Word(word),
                };
                ComparisonUnit::Group(ComparisonUnitGroup {
                    correlation_status: CorrelationStatus::Nil,
                    group_type,
                    contents: vec![ComparisonUnit::Word(word)],
                    level: 0,
                    sha1: Sha1Keyed::new(c.to_string()),
                    correlated_sha1_hash: None,
                    structure_sha1_hash: None,
                    atom_count_memo: std::cell::Cell::new(usize::MAX),
                })
            })
            .collect()
    }

    fn plan_of(deleted: &str, inserted: &str) -> Option<Vec<Step>> {
        let (d, i) = (units(deleted), units(inserted));
        plan(&d.iter().collect::<Vec<_>>(), &i.iter().collect::<Vec<_>>())
    }

    #[test]
    fn the_kth_tables_pair_with_the_revisions_blocks_first() {
        use Step::*;
        assert_eq!(
            plan_of("PTPT", "PPTTPT"),
            Some(vec![
                Inserted(2),
                Deleted(1),
                Paired,
                Deleted(1),
                Paired,
                Inserted(2)
            ])
        );
    }

    #[test]
    fn the_unpaired_tables_end_the_region_revision_first() {
        use Step::*;
        assert_eq!(
            plan_of("TPTT", "PT"),
            Some(vec![Inserted(1), Paired, Deleted(3)])
        );
        assert_eq!(plan_of("T", "TPT"), Some(vec![Paired, Inserted(2)]));
    }

    #[test]
    fn a_side_without_a_table_leaves_the_region() {
        assert_eq!(plan_of("PP", "PTP"), None);
        assert_eq!(plan_of("TP", "PP"), None);
    }

    #[test]
    fn a_loose_word_before_the_last_pair_leaves_the_region() {
        assert_eq!(plan_of("PwT", "T"), None);
        assert_eq!(plan_of("T", "wT"), None);
    }

    #[test]
    fn loose_words_may_only_end_the_rebuilt_region() {
        use Step::*;
        assert_eq!(plan_of("T", "TPww"), Some(vec![Paired, Inserted(3)]));
        assert_eq!(
            plan_of("TPw", "TP"),
            Some(vec![Paired, Inserted(1), Deleted(2)])
        );
        // The revision's words would be cut off from their paragraph's mark
        // by the original's paragraph written after them.
        assert_eq!(plan_of("TP", "Tw"), None);
        assert_eq!(plan_of("TwP", "T"), None);
    }
}
