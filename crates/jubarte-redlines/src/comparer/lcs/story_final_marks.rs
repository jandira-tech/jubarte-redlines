// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! `pair_story_final_marks`: welds the story-final paragraph marks behind a deleted, replaced or inserted tail.

use super::*;

/// Word pairs the two documents' final paragraph marks and joins the
/// revised document's last paragraph to the original's first deleted
/// paragraph. Full LCS emits `[Deleted D, Inserted I¶, Deleted …,
/// Deleted ¶]`; Word's shape is `[Inserted I, Deleted D, Deleted …,
/// Equal ¶]`, so the joined paragraph keeps the original's properties
/// and deleted mark, and the story-final paragraph carries the revised
/// properties.
///
/// Tries three helpers in order (inserted-tail, replaced-tail, deleted-tail)
/// and returns silently when none matches; the caller still pairs the two
/// stories' final empty paragraphs when both are blank.
pub fn pair_story_final_marks(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) {
    if pair_final_marks_behind_inserted_tail(dom, seqs)
        || pair_final_marks_behind_replaced_tail(dom, seqs)
        || pair_final_marks_past_deleted_tail(dom, seqs)
    {
        return;
    }
    let n = seqs.len();
    if n < 4 {
        return;
    }
    fn status(s: &CorrelatedSequence) -> CorrelationStatus {
        s.correlation_status
    }
    fn units1(s: &CorrelatedSequence) -> &[ComparisonUnit] {
        s.com_units_1.as_deref().unwrap_or_default()
    }
    fn units2(s: &CorrelatedSequence) -> &[ComparisonUnit] {
        s.com_units_2.as_deref().unwrap_or_default()
    }
    let last = &seqs[n - 1];
    if status(last) != CorrelationStatus::Deleted
        || !matches!(units1(last), [u] if unit_is_single_atom_ppr(dom, u))
    {
        return;
    }
    let Some(k) = seqs[..n - 1]
        .iter()
        .rposition(|s| status(s) != CorrelationStatus::Deleted)
    else {
        return;
    };
    // [Equal …¶] [Deleted D] [Inserted I¶] [Deleted …]+ [Deleted ¶]
    if k < 2 || k + 2 > n - 1 {
        return;
    }
    let (prev, del, ins) = (&seqs[k - 2], &seqs[k - 1], &seqs[k]);
    let ends_para =
        |v: &[ComparisonUnit]| v.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u));
    if status(prev) != CorrelationStatus::Equal
        || !ends_para(units2(prev))
        || status(del) != CorrelationStatus::Deleted
        || units1(del)
            .first()
            .is_none_or(|u| unit_is_single_atom_ppr(dom, u))
        || status(ins) != CorrelationStatus::Inserted
        || units2(ins).len() < 2
        || !ends_para(units2(ins))
    {
        return;
    }
    let Some(pb) = seqs[k].com_units_2.as_mut().and_then(Vec::pop) else {
        return;
    };
    let Some(pa) = seqs.pop().and_then(|s| s.com_units_1) else {
        return;
    };
    seqs.swap(k - 1, k);
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        pa,
        vec![pb],
    ));
}

/// The units end on a paragraph mark, bare or closing a paragraph group.
pub(super) fn ends_with_mark(dom: &Dom, units: &[ComparisonUnit]) -> bool {
    units.last().is_some_and(|u| {
        unit_is_single_atom_ppr(dom, u)
            || as_group(u).is_some_and(|g| {
                g.group_type == ComparisonUnitGroupType::Paragraph
                    && g.contents
                        .last()
                        .is_some_and(|c| unit_is_single_atom_ppr(dom, c))
            })
    })
}

/// Split the paragraph mark off the units' last paragraph; a paragraph
/// group leaves its words behind.
pub(super) fn split_final_mark(
    dom: &Dom,
    units: &mut Vec<ComparisonUnit>,
) -> Option<ComparisonUnit> {
    let last = units.pop()?;
    if unit_is_single_atom_ppr(dom, &last) {
        return Some(last);
    }
    let mut contents = group_contents(&last);
    let mark = contents.pop();
    units.extend(contents);
    mark
}

/// The revised story's closing mark paired with an interior mark of the
/// original, whose last paragraphs follow deleted: `[Equal …¶] [Deleted
/// …¶]`. Word keeps the two stories' final marks paired whatever precedes
/// them, so the interior mark is deleted with the paragraphs after it and
/// the revised closing mark pairs the original's. Left as it was, the
/// original's deleted closing mark stayed (a body ends on a paragraph) and
/// accepting the redline kept an empty paragraph the revision never had
/// (file_86 × file_88).
pub(super) fn pair_final_marks_past_deleted_tail(
    dom: &Dom,
    seqs: &mut Vec<CorrelatedSequence>,
) -> bool {
    let n = seqs.len();
    if n < 2 || seqs[n - 1].correlation_status != CorrelationStatus::Deleted {
        return false;
    }
    let Some(k) = seqs
        .iter()
        .rposition(|s| s.correlation_status != CorrelationStatus::Deleted)
    else {
        return false;
    };
    fn units(v: &Option<Vec<ComparisonUnit>>) -> &[ComparisonUnit] {
        v.as_deref().unwrap_or_default()
    }
    let closes = |v: &[ComparisonUnit]| {
        ends_with_mark(dom, v) && v.last().is_some_and(|u| unit_closes_story(dom, u))
    };
    let (equal, deleted) = (&seqs[k], &seqs[n - 1]);
    if equal.correlation_status != CorrelationStatus::Equal
        || !ends_with_mark(dom, units(&equal.com_units_1))
        || !closes(units(&equal.com_units_2))
        || !closes(units(&deleted.com_units_1))
    {
        return false;
    }
    let split = |v: &mut Option<Vec<ComparisonUnit>>| {
        v.as_mut().and_then(|units| split_final_mark(dom, units))
    };
    let (Some(interior), Some(revised_close), Some(original_close)) = (
        split(&mut seqs[k].com_units_1),
        split(&mut seqs[k].com_units_2),
        split(&mut seqs[n - 1].com_units_1),
    ) else {
        return false;
    };
    seqs[k + 1]
        .com_units_1
        .get_or_insert_with(Vec::new)
        .insert(0, interior);
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        vec![original_close],
        vec![revised_close],
    ));
    for i in [n - 1, k] {
        if [&seqs[i].com_units_1, &seqs[i].com_units_2]
            .into_iter()
            .flatten()
            .all(Vec::is_empty)
        {
            seqs.remove(i);
        }
    }
    true
}

/// A replaced tail the LCS left deleted-first, `[Equal …¶] [Deleted …¶]
/// [Inserted …¶]` (a kept title over a rewritten body), or a whole story
/// replaced, `[Deleted …¶] [Inserted …¶]`: Word inserts first and pairs the
/// final marks as behind an inserted tail (Word 16 probes 2026-10-01,
/// tests/fixtures/word_probes/final_marks; tests/m_whole_story_final_marks.rs).
pub(super) fn pair_final_marks_behind_replaced_tail(
    dom: &Dom,
    seqs: &mut Vec<CorrelatedSequence>,
) -> bool {
    let n = seqs.len();
    // An insertion ahead of the deleted run placed it mid-revision on purpose
    // (employment × lease: the original spliced in after "3. Rent").
    if n < 2
        || seqs[n - 2].correlation_status != CorrelationStatus::Deleted
        || seqs[n - 1].correlation_status != CorrelationStatus::Inserted
        || n > 2 && seqs[n - 3].correlation_status == CorrelationStatus::Inserted
    {
        return false;
    }
    seqs.swap(n - 2, n - 1);
    if pair_final_marks_behind_inserted_tail(dom, seqs) {
        return true;
    }
    seqs.swap(n - 2, n - 1);
    false
}

/// A replaced tail with the inserted paragraphs ahead of the deleted ones,
/// `[Equal …¶] [Inserted …¶]+ [Deleted …¶]+` (bullet_list_bold ×
/// bullet_list), or a whole story so replaced, `[Inserted …¶]+ [Deleted
/// …¶]+`: Word still pairs the two final marks, so the last inserted
/// paragraph joins the first deleted one under its deleted mark, and the
/// original's last mark stands for the revised one. Left unpaired, that last
/// mark stayed live and accepting the redline kept an empty paragraph the
/// revised document never had.
pub(super) fn pair_final_marks_behind_inserted_tail(
    dom: &Dom,
    seqs: &mut Vec<CorrelatedSequence>,
) -> bool {
    let n = seqs.len();
    let status = |i: usize| seqs[i].correlation_status;
    if n < 2 || status(n - 1) != CorrelationStatus::Deleted {
        return false;
    }
    let Some(k) = (0..n)
        .rev()
        .find(|&i| status(i) != CorrelationStatus::Deleted)
    else {
        return false;
    };
    if status(k) != CorrelationStatus::Inserted {
        return false;
    }
    let ends_para = |v: &[ComparisonUnit]| ends_with_mark(dom, v);
    // A whole story replaced has no kept paragraph before its inserted run.
    let kept_before = (0..k)
        .rev()
        .find(|&i| status(i) != CorrelationStatus::Inserted)
        .is_none_or(|prev| {
            status(prev) == CorrelationStatus::Equal
                && ends_para(seqs[prev].com_units_2.as_deref().unwrap_or_default())
        });
    // The last inserted words join the first deleted paragraph; a deleted
    // table there leaves them no paragraph to join.
    let joins_paragraph = seqs[k + 1]
        .com_units_1
        .as_deref()
        .and_then(<[ComparisonUnit]>::first)
        .is_some_and(|u| {
            as_group(u).is_none_or(|g| g.group_type == ComparisonUnitGroupType::Paragraph)
        });
    // Both marks close their stories; a paragraph closing a block content
    // control is followed by the story's own closing paragraph.
    let closes = |v: Option<&[ComparisonUnit]>| {
        v.unwrap_or_default()
            .last()
            .is_some_and(|u| unit_closes_story(dom, u))
    };
    if !kept_before
        || !joins_paragraph
        || !ends_para(seqs[k].com_units_2.as_deref().unwrap_or_default())
        || !ends_para(seqs[n - 1].com_units_1.as_deref().unwrap_or_default())
        || !closes(seqs[k].com_units_2.as_deref())
        || !closes(seqs[n - 1].com_units_1.as_deref())
    {
        return false;
    }
    let split_mark = |units: &mut Vec<ComparisonUnit>| split_final_mark(dom, units);
    let (Some(pb), Some(pa)) = (
        seqs[k].com_units_2.as_mut().and_then(split_mark),
        seqs[n - 1].com_units_1.as_mut().and_then(split_mark),
    ) else {
        return false;
    };
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        vec![pa],
        vec![pb],
    ));
    for i in [n - 1, k] {
        let empty = [&seqs[i].com_units_1, &seqs[i].com_units_2]
            .into_iter()
            .flatten()
            .all(Vec::is_empty);
        if empty {
            seqs.remove(i);
        }
    }
    true
}
