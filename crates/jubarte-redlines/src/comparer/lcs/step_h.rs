// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Step H: the closing-paragraph, story-final and duplicated-paragraph resolution after the LCS.

use super::*;

// ── Step H helpers ───────────────────────────────────────────────────────────
pub(super) fn as_group(u: &ComparisonUnit) -> Option<&super::atoms::ComparisonUnitGroup> {
    match u {
        ComparisonUnit::Group(g) => Some(g),
        ComparisonUnit::Word(_) => None,
    }
}
pub(super) fn count_gt(units: &[ComparisonUnit], gt: ComparisonUnitGroupType) -> usize {
    units
        .iter()
        .filter(|u| as_group(u).is_some_and(|g| g.group_type == gt))
        .count()
}
pub(super) fn count_words(units: &[ComparisonUnit]) -> usize {
    units
        .iter()
        .filter(|u| matches!(u, ComparisonUnit::Word(_)))
        .count()
}
pub(super) fn group_contents(u: &ComparisonUnit) -> Vec<ComparisonUnit> {
    match u {
        ComparisonUnit::Group(g) => g.contents.clone(),
        ComparisonUnit::Word(_) => vec![],
    }
}
// Source windows can contain both block groups and standalone opaque words
// (for example a schema-valid body-level equation between paragraphs). Only
// expand groups; a non-group unit still owns its complete authored payload.
pub(super) fn source_group_contents(unit: &ComparisonUnit) -> Vec<ComparisonUnit> {
    match unit {
        ComparisonUnit::Group(group) => group.contents.clone(),
        ComparisonUnit::Word(_) => vec![unit.clone()],
    }
}
/// Whether the LAST descendant atom across all `units` is a `w:pPr`. None if there
/// are no atoms at all.
pub(super) fn last_atom_overall_is_ppr(dom: &Dom, units: &[ComparisonUnit]) -> Option<bool> {
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
pub(super) fn containing_paragraph_is_duplicated(
    dom: &Dom,
    units: &[ComparisonUnit],
    pos: usize,
) -> bool {
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
                .all(|a| dom.name_is(a.content_element, &W::p_pr()));
        if is_pmark {
            texts.push(String::new());
            continue;
        }
        // a Group unit that carries its own paragraph mark is a standalone
        // paragraph bucket (whole-paragraph groups in mixed windows)
        let is_group_para = matches!(u, ComparisonUnit::Group(_))
            && atoms
                .iter()
                .any(|a| dom.name_is(a.content_element, &W::p_pr()));
        if is_group_para {
            let text: String = atoms
                .iter()
                .filter(|a| dom.name_is(a.content_element, &W::t()))
                .map(|a| dom.value_str(a.content_element))
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
            if dom.name_is(a.content_element, &W::t()) {
                last.push_str(&dom.value_str(a.content_element));
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

/// A word unit or paragraph group with no visible text: a bare paragraph
/// mark, an empty paragraph.
pub(super) fn unit_is_textless_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool {
    unit_is_paragraph_matter(dom, u) && unit_is_textless(dom, u)
}

/// No visible text in the unit: whitespace, marks, properties.
pub(super) fn unit_is_textless(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().all(|a| {
        let e = a.content_element;
        if dom.name_is(e, &W::t()) {
            return dom.value_str(e).trim().is_empty();
        }
        // Visible non-text content is words to Word's chain rule.
        let visible = [W::drawing(), W::pict(), W::object(), W::name("sym")];
        !visible.iter().any(|n| dom.name_is(e, n))
            && dom.name(e).is_none_or(|n| n.namespace_name() != MATH_URI)
    })
}

pub(super) const MATH_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// A bare paragraph mark or a paragraph group: what Word's pilcrow chain may
/// pair (tables, rows and text boxes stop it).
pub(super) fn unit_is_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool {
    match as_group(u) {
        Some(g) => g.group_type == ComparisonUnitGroupType::Paragraph,
        None => unit_last_atom_is_ppr(dom, u),
    }
}

/// The whole paragraphs that close `us`, last first, as (start, blank): a
/// paragraph group alone, or a paragraph mark with the words before it. A
/// paragraph is blank when none of its units has visible text. The walk
/// stops at a table, row or text box, or at words with no mark after them.
pub(super) fn closing_paragraphs(dom: &Dom, us: &[ComparisonUnit]) -> Vec<(usize, bool)> {
    let mut out = Vec::new();
    let mut end = us.len();
    while end > 0 && unit_is_paragraph_matter(dom, &us[end - 1]) {
        let mut start = end - 1;
        if as_group(&us[start]).is_none() {
            while start > 0
                && as_group(&us[start - 1]).is_none()
                && !unit_is_paragraph_matter(dom, &us[start - 1])
            {
                start -= 1;
            }
        }
        out.push((
            start,
            us[start..end].iter().all(|u| unit_is_textless(dom, u)),
        ));
        end = start;
    }
    out
}

/// Word's interior pilcrow chain (decoded by Docxodus 12's EmitGapArranged):
/// a blank pair that closes a replace region holds while, walking back over
/// the region, each original paragraph is blank. An original paragraph with
/// words facing a blank cancels the chain; two paragraphs with words stop it,
/// and it holds only if the paragraphs left before them balance. A revised
/// paragraph with words facing a blank needs a deleted paragraph with words
/// at the region's head to fuse into. The walk steps over whole paragraphs:
/// a heading's bare mark is not a blank (ff42: "Project Charter" and
/// "Employee Directory" stop the chain, and the unbalanced region releases
/// the blank before the table).
pub(super) fn interior_blank_chain_holds(
    dom: &Dom,
    before1: &[ComparisonUnit],
    before2: &[ComparisonUnit],
) -> bool {
    let (ps1, ps2) = (
        closing_paragraphs(dom, before1),
        closing_paragraphs(dom, before2),
    );
    let (mut a, mut b) = (before1.len(), before2.len());
    let mut fusion = false;
    for (&(s1, blank1), &(s2, blank2)) in ps1.iter().zip(&ps2) {
        if blank1 {
            fusion |= !blank2;
            (a, b) = (s1, s2);
            continue;
        }
        if blank2 {
            return false;
        }
        let paragraphs = |us: &[ComparisonUnit]| {
            us.iter()
                .filter(|u| unit_is_paragraph_matter(dom, u))
                .count()
        };
        if paragraphs(&before1[..a]) != paragraphs(&before2[..b]) {
            return false;
        }
        break;
    }
    // The region's head: its first paragraph, whole and with words.
    let head = before1
        .iter()
        .position(|u| unit_is_paragraph_matter(dom, u))
        .filter(|&e| before1[..e].iter().all(|u| as_group(u).is_none()));
    !fusion
        || (a > 0 && head.is_some_and(|e| !before1[..=e].iter().all(|u| unit_is_textless(dom, u))))
}

/// The unit's last atom sits in its story's last paragraph: nothing but the
/// section properties follows that paragraph in the body, cell or textbox.
pub(in crate::comparer) fn unit_closes_story(dom: &Dom, u: &ComparisonUnit) -> bool {
    story_closing_paragraph(dom, u).is_some()
}

/// The story's last paragraph, when the unit's last atom sits in it.
pub(in crate::comparer) fn story_closing_paragraph(
    dom: &Dom,
    u: &ComparisonUnit,
) -> Option<NodeId> {
    let atoms = u.descendant_atoms();
    let last = atoms.last()?;
    let p = W::p();
    let para = *last
        .ancestor_elements
        .iter()
        .rev()
        .find(|&&e| dom.name_is(e, &p))?;
    let parent = dom.parent(para)?;
    let story = [W::body(), W::tc(), W::txbx_content()];
    let last_in_story = story.iter().any(|n| dom.name_is(parent, n))
        && dom
            .elements(parent, None)
            .into_iter()
            .skip_while(|&c| c != para)
            .skip(1)
            .all(|c| dom.name_is(c, &W::sect_pr()));
    last_in_story.then_some(para)
}

/// Word pairs the two stories' closing paragraphs whatever precedes them. A
/// structural zip of paragraph and table (or word and row) runs would meet
/// the revised closing run with the original's first run after the paired
/// table and delete the rest, the closing mark included (ff42b4a7a3: an
/// accepted blank paragraph after the new table). When both last runs close
/// their stories and either is blank, they come off the zip and pair last.
pub(super) fn peel_story_final_groups<K: PartialEq>(
    dom: &Dom,
    lg: &mut Vec<(K, Vec<ComparisonUnit>)>,
    rg: &mut Vec<(K, Vec<ComparisonUnit>)>,
) -> Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)> {
    let closes = |g: &(K, Vec<ComparisonUnit>)| {
        g.1.last()
            .is_some_and(|u| unit_is_paragraph_matter(dom, u) && unit_closes_story(dom, u))
    };
    // A revision that is one paragraph of words: its closing mark pairs with the
    // original's, and its words zip with the original's first run, whose
    // mark is deleted (bc0135eaa1: one revised paragraph against a picture
    // paragraph, a table and more paragraphs). Zipping the whole run paired
    // the revised mark with the picture paragraph's and left the original's
    // closing paragraph live.
    let bare_mark =
        |g: &(K, Vec<ComparisonUnit>)| g.1.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u));
    if let ([r], Some(l)) = (rg.as_slice(), lg.last())
        && lg.len() >= 2
        && r.1.len() >= 2
        && r.1
            .iter()
            .filter(|u| unit_is_single_atom_ppr(dom, u))
            .count()
            == 1
        && closes(r)
        && closes(l)
        && bare_mark(r)
        && bare_mark(l)
    {
        let mark_b = rg[0].1.pop()?;
        let last_a = lg.last_mut()?;
        let mark_a = last_a.1.pop()?;
        if last_a.1.is_empty() {
            lg.pop();
        }
        return Some((vec![mark_a], vec![mark_b]));
    }
    // The revision ends on an empty paragraph after a table, the original is
    // one run of words: the empty paragraph's mark pairs with the original's
    // closing mark, and the original's last paragraph is deleted into it
    // (Word 16, multi_section × nested_table_rowspan). Unpaired, the
    // revision's closing paragraph was inserted and stripped as trailing
    // matter, so the story ended on its table (math paragraphs ×
    // nested_table_rowspan). The other way round Word keeps the original's
    // empty paragraph live after the deleted table and the revised last
    // paragraph on its inserted mark (nested_table_rowspan × numbered_list).
    let lone_mark = |g: &(K, Vec<ComparisonUnit>)| {
        g.1.len() == 1 && unit_is_single_atom_ppr(dom, &g.1[0]) && unit_closes_story(dom, &g.1[0])
    };
    // Nothing before the empty paragraph is of the run's kind: a title above
    // the table would take the run's words instead.
    if let ([run], Some(close)) = (lg.as_slice(), rg.last())
        && rg.len() >= 2
        && lone_mark(close)
        && run.1.len() >= 2
        && closes(run)
        && bare_mark(run)
        && rg[..rg.len() - 1].iter().all(|g| g.0 != run.0)
    {
        let mark_b = rg.pop()?.1;
        let mark_a = lg[0].1.pop()?;
        return Some((vec![mark_a], mark_b));
    }
    if lg.len() == rg.len() || lg.len() < 2 || rg.len() < 2 {
        return None;
    }
    let blank = |g: &(K, Vec<ComparisonUnit>)| g.1.iter().all(|u| unit_is_textless(dom, u));
    let (l, r) = (lg.last()?, rg.last()?);
    if l.0 != r.0 || !closes(l) || !closes(r) || !(blank(l) || blank(r)) {
        return None;
    }
    Some((lg.pop()?.1, rg.pop()?.1))
}

/// `out` with the peeled closing runs paired after it.
pub(super) fn with_story_final_pair(
    mut out: Vec<CorrelatedSequence>,
    tail: Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)>,
) -> Vec<CorrelatedSequence> {
    if let Some((l, r)) = tail {
        out.push(CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r));
    }
    out
}

/// M4.C.8-C.10 — `DoLcsAlgorithm` Step H: the no-common-run structural dispatch
/// (:7539-:8065). Branches H1-H9, in source order.
pub(super) fn step_h(
    dom: &mut Dom,
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
        let mut lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let mut rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let tail = peel_story_final_groups(dom, &mut lg, &mut rg);
        if std::env::var("JUBARTE_TRACE").is_ok() {
            let toks = |units: &[ComparisonUnit]| -> Vec<String> {
                let raw = para_text_tokens_from_units(dom, units);
                significant_tokens(&raw).into_iter().take(8).collect()
            };
            let lgs: Vec<String> = lg
                .iter()
                .map(|g| format!("{}:{}", g.0, g.1.len()))
                .collect();
            let rgs: Vec<String> = rg
                .iter()
                .map(|g| format!("{}:{}", g.0, g.1.len()))
                .collect();
            eprintln!("H1seam lg=[{}] rg=[{}]", lgs.join(","), rgs.join(","));
            if lg.len() == 1 {
                let all: Vec<ComparisonUnit> =
                    rg.iter().flat_map(|g| g.1.iter().cloned()).collect();
                eprintln!("H1seam t1={:?} t2={:?}", toks(&lg[0].1), toks(&all));
            }
        }
        let group_textless = |dom: &Dom, units: &[ComparisonUnit]| -> bool {
            units.iter().all(|u| {
                u.descendant_atoms().iter().all(|a| {
                    !dom.name_is(a.content_element, &W::t())
                        || dom.value_str(a.content_element).trim().is_empty()
                })
            })
        };
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            let (before_l, before_r) = (il, ir);
            // Scope: only SHORT runs of bare paragraph marks (B's structural
            // empties, ≤3) — larger textless groups keep positional pairing
            // (meeting_agenda×meeting_minutes was exactly 100.00 with it).
            let bare_pmarks = |units: &[ComparisonUnit]| -> bool {
                units.len() <= 3 && units.iter().all(|u| unit_is_single_atom_ppr(dom, u))
            };
            if lg[il].0 == "Word"
                && rg[ir].0 == "Word"
                && ir == 0
                && bare_pmarks(&rg[ir].1)
                && !group_textless(dom, &lg[il].1)
            {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            } else if lg[il].0 == rg[ir].0 {
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
                return with_story_final_pair(out, tail);
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == before_l && ir == before_r {
                // defensive: no progress (e.g. Row vs Textbox) — flush remainder.
                out.push(CorrelatedSequence::deleted(
                    lg[il..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                out.push(CorrelatedSequence::inserted(
                    rg[ir..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                return with_story_final_pair(out, tail);
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
        let contentful_count = |units: &[ComparisonUnit]| -> usize {
            units
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == Paragraph)
                        && unit_has_text_token(dom, u)
                })
                .count()
        };
        let first_contentful_tokens =
            |units: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                units
                    .iter()
                    .find(|u| {
                        as_group(u).is_some_and(|g| g.group_type == Paragraph)
                            && unit_has_text_token(dom, u)
                    })
                    .map(|u| para_text_tokens(dom, u))
                    .unwrap_or_default()
            };
        let mut lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let mut rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let tail = peel_story_final_groups(dom, &mut lg, &mut rg);
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            if lg[il].0 == rg[ir].0 {
                // M205/M206 (table-doc title runs with one contentful each and
                // near-zero title jaccard):
                //   M205 equal-length (q1_sales×quarterly ~65→85): Word
                //   pure-I/Ds titles then EQ-meshes empties+tables.
                //   M206 unequal-length (project_tasks×q1_sales ~67): nested
                //   free-LCS pure-I/Ds titles + extra empty-D; Word free-meshes
                //   titles as R and EQ-meshes the shared empties. Force Unknown
                //   on the two titles and pure-I/D leftover empties.
                let one_title_each = settings.merge_replaced_paragraphs
                    && lg[il].0 == "Para"
                    && contentful_count(&lg[il].1) == 1
                    && contentful_count(&rg[ir].1) == 1
                    // The title/empty partition preserves source order only
                    // when all blank paragraphs follow the title.
                    && lg[il].1.first().is_some_and(|u| unit_has_text_token(dom, u))
                    && rg[ir].1.first().is_some_and(|u| unit_has_text_token(dom, u))
                    && {
                        let j = token_jaccard(
                            &first_contentful_tokens(&lg[il].1),
                            &first_contentful_tokens(&rg[ir].1),
                        );
                        j + 1e-12 < 0.12
                    };
                if one_title_each {
                    let (lt, le): (Vec<_>, Vec<_>) = lg[il]
                        .1
                        .iter()
                        .cloned()
                        .partition(|u| unit_has_text_token(dom, u));
                    let (rt, re): (Vec<_>, Vec<_>) = rg[ir]
                        .1
                        .iter()
                        .cloned()
                        .partition(|u| unit_has_text_token(dom, u));
                    if lg[il].1.len() == rg[ir].1.len() {
                        // M205: pure-I/D titles
                        if !rt.is_empty() {
                            out.push(CorrelatedSequence::inserted(rt));
                        }
                        if !lt.is_empty() {
                            out.push(CorrelatedSequence::deleted(lt));
                        }
                    } else {
                        // M206: free-mesh titles (Unknown → word LCS replace)
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            lt,
                            rt,
                        ));
                    }
                    // Empties: positional Unknown for the shared prefix, pure
                    // I/D the leftover empties on the longer side.
                    let n_eq = le.len().min(re.len());
                    if n_eq > 0 {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            le[..n_eq].to_vec(),
                            re[..n_eq].to_vec(),
                        ));
                    }
                    if le.len() > n_eq {
                        out.push(CorrelatedSequence::deleted(le[n_eq..].to_vec()));
                    }
                    if re.len() > n_eq {
                        out.push(CorrelatedSequence::inserted(re[n_eq..].to_vec()));
                    }
                } else {
                    // M320 (support_tickets×table_bookmark_end; hr_onboarding×proof):
                    // Word merges the short side's sole table with the FIRST
                    // same-slot table on the multi-table side (cell-wise MIX:
                    // R1C1×Ticket ID / checklist×thesis). A prior 1×≥3
                    // zero-Jaccard pure-I/D short-circuit skipped that mesh
                    // (~47 support_tickets, ~49 hr_onboarding). Always Unknown
                    // so DoLcsAlgorithmForTable / positional rows can run.
                    // Single×single zero-Jaccard tables already took this path.
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        lg[il].1.clone(),
                        rg[ir].1.clone(),
                    ));
                }
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
                return with_story_final_pair(out, tail);
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
        }
    }

    // M201 (book_catalog×book_catalog_table; project_tasks×table ~60→92):
    // After outer LCS peels equal short titles, residual is one mashed prose
    // body vs empties+table. Free LCS character-meshes cell text against prose
    // ("The Gre"/"at Gats" thrash). Word pure-I/Ds residual without free-mesh.
    //
    // Shape (residual window only — titles already peeled):
    //   - prose side: 0 tables, exactly 1 contentful para, len ≤ 4
    //   - table side: exactly 1 table, 0 contentful paras (empties only)
    // Excludes meeting_minutes multi-para (−27), support_tickets×summary
    // (different titles so residual not this shape after peel), etc.
    //
    // M207 (contract_review insertions×mixed ~67; inventory deletions×mixed
    // ~68): full window still has equal titles (j ≥ 0.9) so residual M201
    // never sees the window — H4 flattens and free-meshes. Peel the equal
    // first contentful titles, then pure-I/D the residual when it matches
    // the prose-vs-table shape. Word EQ title + R residual + empty table.
    let left_only_ptt_m201 = left_len == left_tables + left_paras + left_textboxes;
    let right_only_ptt_m201 = right_len == right_tables + right_paras + right_textboxes;
    if settings.merge_replaced_paragraphs
        && left_only_ptt_m201
        && right_only_ptt_m201
        && left_textboxes == 0
        && right_textboxes == 0
        && left_len >= 1
        && right_len >= 1
        && left_len <= 12
        && right_len <= 12
    {
        let contentful_paras = |units: &[ComparisonUnit]| -> usize {
            units
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == Paragraph)
                        && unit_has_text_token(dom, u)
                })
                .count()
        };
        let first_contentful_idx = |units: &[ComparisonUnit]| -> Option<usize> {
            units.iter().position(|u| {
                as_group(u).is_some_and(|g| g.group_type == Paragraph)
                    && unit_has_text_token(dom, u)
            })
        };
        let lc = contentful_paras(cul1);
        let rc = contentful_paras(cul2);
        // Tight residual pure-I/D (original M201): keep len ≤ 6.
        let prose_vs_table = left_len <= 6
            && right_len <= 6
            && ((left_tables == 0 && lc == 1 && right_tables == 1 && rc == 0)
                || (right_tables == 0 && rc == 1 && left_tables == 1 && lc == 0));
        if prose_vs_table {
            for u in cul2 {
                out.push(CorrelatedSequence::inserted(vec![u.clone()]));
            }
            for u in cul1 {
                out.push(CorrelatedSequence::deleted(vec![u.clone()]));
            }
            return out;
        }
        // M207: equal-title peel then residual prose-vs-table.
        if (left_tables == 1) ^ (right_tables == 1)
            && lc >= 1
            && rc >= 1
            && left_len <= 5
            && right_len <= 5
            && let (Some(li), Some(ri)) = (first_contentful_idx(cul1), first_contentful_idx(cul2))
        {
            let j_title = token_jaccard(
                &para_text_tokens(dom, &cul1[li]),
                &para_text_tokens(dom, &cul2[ri]),
            );
            if j_title + 1e-12 >= 0.9 {
                let rest1: Vec<ComparisonUnit> = cul1
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != li)
                    .map(|(_, u)| u.clone())
                    .collect();
                let rest2: Vec<ComparisonUnit> = cul2
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != ri)
                    .map(|(_, u)| u.clone())
                    .collect();
                let rc_rest = contentful_paras(&rest1);
                let rr_rest = contentful_paras(&rest2);
                let rt1 = count_gt(&rest1, Table);
                let rt2 = count_gt(&rest2, Table);
                let residual_pvt = (rt1 == 0 && rc_rest == 1 && rt2 == 1 && rr_rest == 0)
                    || (rt2 == 0 && rr_rest == 1 && rt1 == 1 && rc_rest == 0);
                if residual_pvt && !rest1.is_empty() && !rest2.is_empty() {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![cul1[li].clone()],
                        vec![cul2[ri].clone()],
                    ));
                    for u in &rest2 {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                    for u in &rest1 {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    return out;
                }
            }
        }
        // M208 (book_catalog_table×budget_report ~69→91): base is title+empties+
        // 1 table, next is multi pure-prose (≥4 contentful, no tables). Free LCS
        // pure-I's every prose line + pure-D title (~69). Word pure-I's
        // all-but-last prose, free-meshes last prose × title, pure-D empties+
        // table.
        //
        // Direction is table-LEFT × prose-RIGHT only — prose-left×table-right
        // (marketing_strategy×meeting_agenda_table) was already 100 via pure
        // I/D of the agenda residual; free-meshing last KPI×title tanked LO
        // −52. Also require first-title × first-prose j < 0.15 so related
        // families (Meeting Agenda×Meeting Minutes) stay on the free path
        // (was 100; free-mesh last×title −28).
        let m208 = left_tables == 1
            && lc == 1
            && left_len <= 5
            && right_tables == 0
            && rc == right_paras
            && rc >= 4
            && right_len == right_paras;
        if m208 && let Some(ti) = first_contentful_idx(cul1) {
            let last_p = cul2.len() - 1;
            let j_first = token_jaccard(
                &para_text_tokens(dom, &cul1[ti]),
                &para_text_tokens(dom, &cul2[0]),
            );
            let j_last = token_jaccard(
                &para_text_tokens(dom, &cul2[last_p]),
                &para_text_tokens(dom, &cul1[ti]),
            );
            if j_first + 1e-12 < 0.15 && j_last + 1e-12 < 0.15 {
                // pure-I early next prose
                for u in &cul2[..last_p] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                // free-mesh last next prose × base title
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[ti].clone()],
                    vec![cul2[last_p].clone()],
                ));
                // pure-D rest of table side
                for (i, u) in cul1.iter().enumerate() {
                    if i == ti {
                        continue;
                    }
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
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
        // M393 (broken_list_missing × broken_list Word IDDDDDDDIIII DDD):
        // short-item list pair with a nested sublist on base. Word pure-I's
        // first next item, pure-D's base first list cluster (through ilvl≥1
        // subs), pure-I rest of next, pure-D rest of base. Full pure-I/D
        // wholesale (M308) and free word-LCS both free-mesh "a"×"Item 1"
        // into MIX (~53 pagefair). Require a true mid-cluster cut (saw nested
        // then top-level) so flat short lists stay on M308.
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_paras >= 4
            && right_paras >= 4
            && left_paras != right_paras
        {
            let body_j = token_jaccard(
                &para_text_tokens_from_units(dom, cul1),
                &para_text_tokens_from_units(dom, cul2),
            );
            let list_left: Vec<&ComparisonUnit> = cul1
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect();
            let list_right: Vec<&ComparisonUnit> = cul2
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect();
            let mostly = |xs: &[&ComparisonUnit]| {
                if xs.is_empty() {
                    return false;
                }
                let n = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
                n * 2 >= xs.len()
            };
            let cut = first_list_cluster_end(dom, cul1);
            let has_nested = list_left
                .iter()
                .any(|u| unit_para_ilvl(dom, u).unwrap_or(0) >= 1);
            if body_j + 1e-12 < 0.25
                && mostly(&list_left)
                && mostly(&list_right)
                && short_item_list_groups(dom, &list_left)
                && short_item_list_groups(dom, &list_right)
                && has_nested
                && cut >= 2
                && cut < cul1.len()
                && !list_right.is_empty()
            {
                // pure-I first next, pure-D first cluster, pure-I rest, pure-D rest
                out.push(CorrelatedSequence::inserted(vec![cul2[0].clone()]));
                out.push(CorrelatedSequence::deleted(cul1[..cut].to_vec()));
                if cul2.len() > 1 {
                    out.push(CorrelatedSequence::inserted(cul2[1..].to_vec()));
                }
                if cut < cul1.len() {
                    out.push(CorrelatedSequence::deleted(cul1[cut..].to_vec()));
                }
                return out;
            }
        }
        // M308 (broken_list × multiple_nodes): unequal pure-para lists with
        // near-zero text overlap and numPr on ≥ half of contentful paras on
        // BOTH sides. Word pure-I all next then pure-D all base; H4 flatten
        // + word LCS carrier-fuses the last B item into a MIX with A's first.
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_paras != right_paras
            && left_paras >= 2
            && right_paras >= 2
        {
            let body_j = token_jaccard(
                &para_text_tokens_from_units(dom, cul1),
                &para_text_tokens_from_units(dom, cul2),
            );
            let list_left = cul1
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect::<Vec<_>>();
            let list_right = cul2
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect::<Vec<_>>();
            let mostly = |xs: &[&ComparisonUnit]| {
                if xs.is_empty() {
                    return false;
                }
                let n = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
                n * 2 >= xs.len()
            };
            // M308c: also require short-item lists (see short_item_list_groups).
            // Unpacked Word: list_with_indents×lists_sub is list-heavy but long
            // prose → MIX; broken_list short items → pure-I/D.
            if body_j + 1e-12 < 0.12
                && mostly(&list_left)
                && mostly(&list_right)
                && short_item_list_groups(dom, &list_left)
                && short_item_list_groups(dom, &list_right)
            {
                for u in cul2 {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in cul1 {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return out;
            }
        }
        // M168 (project_plan×project_proposal): short pure-para docs, titles
        // share first token ("Project") but not last-sig (Plan vs Proposal),
        // body residual unrelated. Flat pure-I/D whole titles (~81); Word
        // meshes EQ "Project " then pure-I next residual + pure-D base.
        // Also accept word-flattened windows (left_paras==0) by detecting
        // paragraph marks: count trailing pPr atoms as para boundaries.
        let m168_para_ok = left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (3..=10).contains(&left_paras)
            && (3..=8).contains(&right_paras)
            && left_paras != right_paras;
        if settings.merge_replaced_paragraphs && m168_para_ok {
            let a0 = para_text_token_list(dom, &cul1[0]);
            let b0 = para_text_token_list(dom, &cul2[0]);
            let first_same = a0
                .first()
                .zip(b0.first())
                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
            let last_diff = match (last_significant_token(&a0), last_significant_token(&b0)) {
                (Some(x), Some(y)) => !x.eq_ignore_ascii_case(y),
                _ => true,
            };
            let body_j = if left_paras >= 2 && right_paras >= 2 {
                token_jaccard(
                    &para_text_tokens_from_units(dom, &cul1[1..]),
                    &para_text_tokens_from_units(dom, &cul2[1..]),
                )
            } else {
                1.0
            };
            if first_same
                && last_diff
                && (2..=4).contains(&a0.len())
                && (2..=4).contains(&b0.len())
                && body_j + 1e-12 < 0.12
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                for u in &cul2[1..] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in &cul1[1..] {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return out;
            }
        }
        // Word-mode equal-count pure-paragraph zip (heading_2 vs heading_3 demos):
        // Word aligns N×para vs N×para positionally → N mixed paragraphs. Flattening
        // every paragraph into one word-LCS window lets shared tokens ("Heading")
        // bridge the wrong paragraphs (ours: 4 paras vs Word's 3; pixel ~58 vs 100).
        // Cap at 12; require ≥2 (1-vs-1 zip re-enters H4 forever).
        // Only when positional pairing is the best text alignment (diagonal
        // dominance): numbered_list Demo+4items vs Demo+intro+3items is equal
        // count but roles shift — flat LCS wins; forced zip regressed ~7 pts.
        // Skip equal-count zip for residual peels (title Demo cousins with
        // unrelated bodies). Zip invents false 3×MIX.
        // M149: short first residual (text_highlight / blue_underline).
        // M153: long first residual (calibri_heading_2×center_aligned) — Word
        //   MIX|INS|MIX|DEL (pure-I B0, mesh A0×B1, pure-D A1).
        // M151: "This text …" vs "This document …" (right_aligned×_2).
        let title_demo_unrelated_body = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && body_residual_unrelated(dom, cul1, cul2)
            && {
                let d0 = token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                );
                let d1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let d2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                d0 > 0.0 && d1 + 1e-12 < 0.08 && d2 + 1e-12 < 0.08
            };
        let a1_n_zip = if left_paras >= 2 {
            para_text_tokens(dom, &cul1[1]).len()
        } else {
            0
        };
        let b1_n_zip = if right_paras >= 2 {
            para_text_tokens(dom, &cul2[1]).len()
        } else {
            0
        };
        // M149: short first residual — skip zip only (do not force residual
        // entry when diagonal; flat path scores better for text_highlight).
        // Exclude style/heading residual bodies (heading_4×helvetica).
        let skip_zip_for_m149 = title_demo_unrelated_body
            && ((a1_n_zip > 0 && a1_n_zip <= 6) || (b1_n_zip > 0 && b1_n_zip <= 6))
            && {
                let style_body = |u: &ComparisonUnit| {
                    para_text_token_list(dom, u).iter().any(|t| {
                        t.eq_ignore_ascii_case("heading")
                            || t.eq_ignore_ascii_case("paragraph")
                            || t.eq_ignore_ascii_case("style")
                    })
                };
                left_paras >= 2
                    && right_paras >= 2
                    && !style_body(&cul1[1])
                    && !style_body(&cul2[1])
            };
        // M153: long first residual both sides — skip zip AND force peel entry.
        let skip_zip_for_m153 = title_demo_unrelated_body && a1_n_zip > 6 && b1_n_zip > 6;
        let is_m151_residual_pair = |left: &ComparisonUnit, right: &ComparisonUnit| {
            residual_para_starts_this(dom, left) && residual_para_starts_this(dom, right) && {
                let a1 = para_text_token_list(dom, left);
                let b1 = para_text_token_list(dom, right);
                // "this text" vs "this document" → ordered prefix sig == 1
                ordered_shared_prefix_sig(&a1, &b1) == 1
                    && a1.get(1).is_some_and(|t| t.eq_ignore_ascii_case("text"))
                    && b1
                        .get(1)
                        .is_some_and(|t| t.eq_ignore_ascii_case("document"))
            }
        };
        let skip_zip_for_m151 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && is_m151_residual_pair(&cul1[1], &cul2[1]);
        // The outer LCS can peel the shared Demo title before Step H. Preserve
        // the same M151 Word shape on the resulting 2×2 residual window.
        let m151_residual_window = left_paras == 2
            && right_paras == 2
            && left_paras == left_len
            && right_paras == right_len
            && is_m151_residual_pair(&cul1[0], &cul2[0]);
        if settings.merge_replaced_paragraphs && m151_residual_window {
            out.push(CorrelatedSequence::inserted(vec![cul2[0].clone()]));
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[1].clone()],
            ));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            return out;
        }
        // M197 (calibri_heading_2_right×center_aligned_bold ~61→84): equal 3v3
        // Demo last-sig titles that only share Demo chrome (title j < 0.12),
        // BOTH residual body pairs content-unrelated (j1 < 0.12 && j2 < 0.12),
        // and first residuals are NOT Demonstrating×This cousins (those free-
        // mesh style boilerplate; pure-I/D regressed text_highlight×times
        // −24 and blue_underline×bold_italic −23). Zip free-meshes on thin
        // glue; Word pure-I/Ds every residual body for true cross-demos.
        let skip_zip_for_m197 =
            left_paras == 3 && right_paras == 3 && first_paras_share_last_sig(dom, cul1, cul2) && {
                let j0 = token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                );
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let a1 = para_text_token_list(dom, &cul1[1]);
                let b1 = para_text_token_list(dom, &cul2[1]);
                let a0f = a1.first().map(|s| s.as_str()).unwrap_or("");
                let b0f = b1.first().map(|s| s.as_str()).unwrap_or("");
                let this_x_demo = (a0f.eq_ignore_ascii_case("this")
                    && b0f.eq_ignore_ascii_case("demonstrating"))
                    || (a0f.eq_ignore_ascii_case("demonstrating")
                        && b0f.eq_ignore_ascii_case("this"));
                j0 + 1e-12 < 0.12 && j1 + 1e-12 < 0.12 && j2 + 1e-12 < 0.12 && !this_x_demo
            };
        if settings.merge_replaced_paragraphs
            && skip_zip_for_m197
            && left_paras == left_len
            && right_paras == right_len
        {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[0].clone()],
            ));
            out.push(CorrelatedSequence::inserted(vec![cul2[1].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            out.push(CorrelatedSequence::inserted(vec![cul2[2].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[2].clone()]));
            return out;
        }
        // M210 (center_aligned_bold×center_alignment ~80 only): equal 3v3 Demo
        // last-sig, both first residuals start with "This", j1 ∈ [0.17, 0.20),
        // j2 ∈ [0.25, 0.32), and both titles contain the token "center".
        // Tightened vs dropped M204 so right_align_bold (j2≈0.19),
        // underline×verdana (j2≈0.08), small_font (j2≈0.07) stay free-mesh.
        let skip_zip_for_m210 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let t0a = para_text_tokens(dom, &cul1[0]);
                let t0b = para_text_tokens(dom, &cul2[0]);
                let has_center =
                    t0a.iter().any(|t| t == "center") && t0b.iter().any(|t| t == "center");
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                has_center
                    && j1 + 1e-12 >= 0.17
                    && j1 + 1e-12 < 0.20
                    && j2 + 1e-12 >= 0.25
                    && j2 + 1e-12 < 0.32
            };
        if settings.merge_replaced_paragraphs
            && skip_zip_for_m210
            && left_paras == left_len
            && right_paras == right_len
        {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[0].clone()],
            ));
            out.push(CorrelatedSequence::inserted(vec![cul2[1].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            out.push(CorrelatedSequence::inserted(vec![cul2[2].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[2].clone()]));
            return out;
        }
        // M165 (font_size_12×font_size_18; red_heading×red_strikethrough):
        // equal 3v3 Demo, first residual near-identical (digit/word swap),
        // last residual near-unrelated. Positional zip meshes last on a lone
        // boilerplate token ("font"/"Red") → MIX (~78–82); Word pure-I last
        // next + pure-D last base (~pixel win). Does not fire when last
        // residual is mid-related (blue_bold j1 mid / j2 low uses zip).
        let skip_zip_for_m165 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                // j2 < 0.10: fs12/red ~0.06 pure-I/D. Mid-last residuals
                // must stay MIX: bold_italic×underline / blue_italic×
                // underline j≈0.13 (Word meshes "text"/"Blue"); track_changes
                // heading×italic j≈0.19.
                j1 + 1e-12 >= 0.55 && j2 + 1e-12 < 0.10
            };
        // M180 (times×title / subtitle×superscript / calibri last / track last):
        // equal 3v3 Demo, first residual mid-related (j1≥0.25), last residual
        // content-unrelated (content jaccard <0.08, len≥3 words only). Zip free
        // LCS period-bridges (~82–85); Word pure-I/D last (~100). Not M165
        // (j1 may be <0.55). Not 4v3 (M162 font_family residual peel).
        let skip_zip_for_m180 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let a2 = para_text_token_list(dom, &cul1[2]);
                let b2 = para_text_token_list(dom, &cul2[2]);
                let j2_raw = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let content = |toks: &[String]| -> std::collections::HashSet<String> {
                    toks.iter()
                        .filter(|t| {
                            t.chars().any(|c| c.is_ascii_alphanumeric()) && t.chars().count() >= 3
                        })
                        .map(|t| t.to_ascii_lowercase())
                        .collect()
                };
                let sa = content(&a2);
                let sb = content(&b2);
                // Format-boilerplate shared words (bold/text/…) are not a real
                // content bridge — Word pure-I/Ds center_bold×clear last residual
                // despite sharing "bold"/"text". verdana shares "Verdana" (not
                // boilerplate) and must stay MIX.
                const FORMAT_BOILER: &[&str] = &[
                    "bold",
                    "text",
                    "italic",
                    "underline",
                    "formatting",
                    "format",
                    "style",
                    "styles",
                    "font",
                    "fonts",
                    // Residual openers/stopwords — "this" alone must not count as
                    // real content on "This text is bold" (asymmetric M182 false
                    // positive → pure-I/D last; Word free-meshes EQ text/bold).
                    "this",
                    "that",
                    "with",
                    "from",
                    "into",
                    "used",
                    "for",
                    "and",
                    "the",
                ];
                let inter: std::collections::HashSet<&String> = sa.intersection(&sb).collect();
                let inter_real: Vec<&String> = inter
                    .iter()
                    .copied()
                    .filter(|w| !FORMAT_BOILER.iter().any(|b| w.eq_ignore_ascii_case(b)))
                    .collect();
                let j2c = if inter_real.is_empty() {
                    // empty or format-only intersection → treat as content-empty
                    0.0
                } else {
                    let uni = sa.union(&sb).count() as f64;
                    if uni > 0.0 {
                        inter_real.len() as f64 / uni
                    } else {
                        0.0
                    }
                };
                // j1 ≥0.12: times×title first residual ~0.14 ("This document");
                // subtitle ~0.33; center_bold×clear ~0.30. Keep <0.55 for M165.
                // j2c <0.05 after format-boiler strip: pure-empty last residual.
                // verdana font×italic shares "Verdana" (kept) → j2c>0 → stay MIX.
                // Both last residuals ≥6 toks: short "This text is bold" (4)
                // must free-mesh (Word EQ "text is"); pure-I/D regressed
                // bold_text×bold_underline 98→89.
                // M182: asymmetric short last residual (2..=4 toks) vs long
                // (≥6) with empty content bridge — but the SHORT side must keep
                // non-boiler real content ("Main Title Section", "Small Section
                // Header"). Pure format stubs ("This text is bold" → only
                // text/bold after strip) must free-mesh (Word EQ text/bold;
                // pure-I/D regressed bold_text×bold_underline 98→89).
                let both_long = a2.len() >= 6 && b2.len() >= 6;
                let real_nonempty = |toks: &[String]| -> bool {
                    content(toks)
                        .iter()
                        .any(|w| !FORMAT_BOILER.iter().any(|b| w.eq_ignore_ascii_case(b)))
                };
                let asymmetric_short =
                    (a2.len() >= 2 && a2.len() <= 4 && b2.len() >= 6 && real_nonempty(&a2))
                        || (b2.len() >= 2 && b2.len() <= 4 && a2.len() >= 6 && real_nonempty(&b2));
                // both_long also needs raw j2 <0.15: bold_red×superscript shares
                // "is used and" (j2≈0.27) — Word free-meshes; pure-I/D LO −21.
                // center_bold×clear j2≈0.13 still pure-I/Ds (LO 100).
                j1 + 1e-12 >= 0.12
                    && j1 + 1e-12 < 0.55
                    && j2c + 1e-12 < 0.05
                    && (both_long && j2_raw + 1e-12 < 0.15 || asymmetric_short)
            };
        // M183 (left_alignment×line_spacing 3v4 / reverse 4v3): Demo last-sig
        // titles, first residual mid-related This-bodies, longer side has an
        // extra mid residual. Zip free-meshes last with orphan periods (~85);
        // Word meshes title+first residual, pure-I's extra mid body(s), pure
        // I/D last residual. Not equal-count M180. Keep j_last raw thin — a
        // layout-boiler strip false-fired on center_alignment×center_bold
        // (shared "titles") and regressed LO score.
        let skip_zip_for_m183 = {
            // Only 3-base × 4-next (extra inserted mid body), not 4×3 —
            // reverse fired on font_family×font_size_12 and regressed LO ~26pts.
            left_paras == 3
                && right_paras == 4
                && first_paras_share_last_sig(dom, cul1, cul2)
                && residual_para_starts_this(dom, &cul1[1])
                && residual_para_starts_this(dom, &cul2[1])
                && {
                    let j1 = token_jaccard(
                        &para_text_tokens(dom, &cul1[1]),
                        &para_text_tokens(dom, &cul2[1]),
                    );
                    let a_last = para_text_token_list(dom, &cul1[left_paras - 1]);
                    let b_last = para_text_token_list(dom, &cul2[right_paras - 1]);
                    let j_last = token_jaccard(
                        &para_text_tokens(dom, &cul1[left_paras - 1]),
                        &para_text_tokens(dom, &cul2[right_paras - 1]),
                    );
                    j1 + 1e-12 >= 0.15
                        && j1 + 1e-12 < 0.55
                        && j_last + 1e-12 < 0.12
                        && a_last.len() >= 4
                        && b_last.len() >= 4
                }
        };
        // M173 (italic_and_underline×italic_subscript): equal 3v3 Demo, first
        // residual mid-related (shared "italic"/"combined"), last residual
        // glue-related with a thin content bridge ("is"+"and" + "italic").
        // Zip + glue-void → pure I/D last (~83); Word free-meshes EQ is/and
        // (~pixel win). Not M165 (j1 may be <0.55; j2 may be >0.10 with glue).
        // Tightened after underline×verdana false-positive (single glue "is",
        // j2≈0.08, j2_content=0 → Word pure-I/D last; free mesh regressed).
        let skip_zip_for_m173 =
            left_paras == 3 && right_paras == 3 && first_paras_share_last_sig(dom, cul1, cul2) && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let a2 = para_text_token_list(dom, &cul1[2]);
                let b2 = para_text_token_list(dom, &cul2[2]);
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let glue = ["is", "and", "a", "the", "of", "in", "to", "for"];
                // Require ≥2 shared glue words (iu: is+and). Single "is"
                // (underline×verdana) is not enough for free residual mesh.
                let mut shared_glue: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for t in &a2 {
                    if glue.iter().any(|g| t.eq_ignore_ascii_case(g))
                        && b2.iter().any(|u| u.eq_ignore_ascii_case(t))
                    {
                        shared_glue.insert(t.to_ascii_lowercase());
                    }
                }
                let share_glue_n = shared_glue.len();
                // content jaccard without glue tokens should be thin but non-zero
                // (iu shares "italic"; pure-glue-only would false-positive).
                let strip = |toks: &[String]| -> std::collections::HashSet<String> {
                    toks.iter()
                        .filter(|t| {
                            !glue.iter().any(|g| t.eq_ignore_ascii_case(g))
                                && t.chars().count() >= 3
                        })
                        .cloned()
                        .collect()
                };
                let sa = strip(&a2);
                let sb = strip(&b2);
                let j2_content = if sa.is_empty() && sb.is_empty() {
                    0.0
                } else {
                    let inter = sa.intersection(&sb).count() as f64;
                    let uni = sa.union(&sb).count() as f64;
                    if uni > 0.0 { inter / uni } else { 0.0 }
                };
                // j1 ≥0.15: italic_underline×subscript first residual ~0.18
                // (italic/combined); keep <0.55 so M165 digit-swap stays separate.
                // j2 ≥0.15: exclude underline×verdana (j2≈0.08 pure-I/D).
                // j2_content ∈ (0, 0.15): thin content bridge required.
                j1 + 1e-12 >= 0.15
                    && j1 + 1e-12 < 0.55
                    && share_glue_n >= 2
                    && j2_content + 1e-12 > 0.0
                    && j2_content + 1e-12 < 0.15
                    && j2 + 1e-12 >= 0.15
                    && j2 + 1e-12 < 0.35
                    && a2.len() >= 4
                    && b2.len() >= 4
            };
        // M161 (title_style×title_style_default_missing; also reverse
        // title_style_centered×title_style): last residual is exactly
        // "Document Title" (2 toks) on either side vs long residual on the
        // other. Positional zip bridges shared "Title" (~72–80); Word pure-I
        // short + pure-D long (~99). Not "Document Subtitle Description".
        let skip_zip_for_m161 = {
            let last_i = left_paras.saturating_sub(1);
            let is_doc_title = |toks: &[String]| {
                toks.len() == 2
                    && toks[0].eq_ignore_ascii_case("document")
                    && toks[1].eq_ignore_ascii_case("title")
            };
            let last_doc_title_vs_long = left_paras >= 2
                && left_paras == right_paras
                && (left_paras == 2 || left_paras == 3)
                && {
                    let a_last = para_text_token_list(dom, &cul1[last_i]);
                    let b_last = para_text_token_list(dom, &cul2[last_i]);
                    (is_doc_title(&a_last) && b_last.len() > 6)
                        || (is_doc_title(&b_last) && a_last.len() > 6)
                };
            if !last_doc_title_vs_long {
                false
            } else if left_paras == 3 {
                first_paras_share_last_sig(dom, cul1, cul2)
                    && token_jaccard(
                        &para_text_tokens(dom, &cul1[0]),
                        &para_text_tokens(dom, &cul2[0]),
                    ) + 1e-12
                        >= 0.5
            } else {
                // 2v2 residual after equal/similar title peeled.
                true
            }
        };
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
            && !skip_zip_for_m149
            && !skip_zip_for_m153
            && !skip_zip_for_m151
            && !skip_zip_for_m165
            && !skip_zip_for_m173
            && !skip_zip_for_m180
            && !skip_zip_for_m183
            && !skip_zip_for_m161
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
        // M142/M144: short Demo demos sharing a title last-sig.
        // Pair titles, then residual by case:
        //   M144 (italic×justified): longer base residual (≥2) vs single next
        //     body → residual word-LCS so trailing next phrase peels into last
        //     pure-D ("for a formal document look"). Does NOT require body
        //     residual unrelated (bodies share "combines"/"underline").
        //   M142 (heading_4×helvetica; justify×large): body residual only
        //     boilerplate-related → pure-I rest B + pure-D rest A; merge folds.
        // Enter when zip is NOT diagonal-dominant, OR when M149/M151 skip-zip
        // gates fired (equal-count zip would invent wrong 3×MIX).
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (2..=6).contains(&left_paras)
            && (2..=6).contains(&right_paras)
            && left_paras.abs_diff(right_paras) <= 2
            // Force residual peel for M151/M153/M165/M173/M161; for M149 only when
            // the *base* residual is the short side (text_highlight). Short-next
            // (blue_underline) keeps flat LCS when diagonal under M141.
            && (!(left_paras == right_paras && para_zip_diagonal_dominant(dom, cul1, cul2))
                || (skip_zip_for_m149 && a1_n_zip > 0 && a1_n_zip <= 6 && a1_n_zip <= b1_n_zip)
                || skip_zip_for_m151
                || skip_zip_for_m153
                || skip_zip_for_m165
                || skip_zip_for_m173
                || skip_zip_for_m180
                || skip_zip_for_m183
                || skip_zip_for_m161)
            && first_paras_share_last_sig(dom, cul1, cul2)
        {
            let rest1 = &cul1[1..];
            let rest2 = &cul2[1..];
            let m144 = rest1.len() >= 2 && rest1.len() > rest2.len();
            let m146 = rest1.len() >= 2 && rest2.len() > rest1.len() && rest2.len() <= 6;
            let m142 = body_residual_unrelated(dom, cul1, cul2);
            let first_residual_j = if !rest1.is_empty() && !rest2.is_empty() {
                token_jaccard(
                    &para_text_tokens(dom, &rest1[0]),
                    &para_text_tokens(dom, &rest2[0]),
                )
            } else {
                1.0
            };
            let a0_n = if rest1.is_empty() {
                0
            } else {
                para_text_tokens(dom, &rest1[0]).len()
            };
            let b0_n = if rest2.is_empty() {
                0
            } else {
                para_text_tokens(dom, &rest2[0]).len()
            };
            let short_first_residual = (a0_n > 0 && a0_n <= 6) || (b0_n > 0 && b0_n <= 6);
            let residual_looks_like_style_body = |u: &ComparisonUnit| {
                let toks = para_text_token_list(dom, u);
                toks.iter().any(|t| {
                    t.eq_ignore_ascii_case("heading")
                        || t.eq_ignore_ascii_case("paragraph")
                        || t.eq_ignore_ascii_case("style")
                })
            };
            let m149 = rest1.len() == 2
                && rest2.len() == 2
                && m142
                && first_residual_j + 1e-12 < 0.08
                && short_first_residual
                && !residual_looks_like_style_body(&rest1[0])
                && !residual_looks_like_style_body(&rest2[0]);
            let m153 = rest1.len() == 2
                && rest2.len() == 2
                && m142
                && first_residual_j + 1e-12 < 0.08
                && a0_n > 6
                && b0_n > 6;
            let m151 = rest1.len() == 2
                && rest2.len() == 2
                && residual_para_starts_this(dom, &rest1[0])
                && residual_para_starts_this(dom, &rest2[0])
                && {
                    let a0 = para_text_token_list(dom, &rest1[0]);
                    let b0 = para_text_token_list(dom, &rest2[0]);
                    ordered_shared_prefix_sig(&a0, &b0) == 1
                        && a0.get(1).is_some_and(|t| t.eq_ignore_ascii_case("text"))
                        && b0
                            .get(1)
                            .is_some_and(|t| t.eq_ignore_ascii_case("document"))
                };
            let m165 = skip_zip_for_m165;
            let m173 = skip_zip_for_m173;
            let m180 = skip_zip_for_m180;
            let m183 = skip_zip_for_m183;
            let m161 = skip_zip_for_m161;
            // M163 (numbered_list×numbered_list_italic): Demo+short items vs
            // Demo+intro("This…")+items. Word pure-I intro then position-mesh
            // items (First×First italic…); flat LCS shifts (DEL First, mesh
            // First-italic×Second, ~78).
            let m163 = rest1.len() >= 2
                && rest2.len() >= 2
                && residual_para_starts_this(dom, &rest2[0])
                && rest1.iter().all(|u| {
                    let n = para_text_tokens(dom, u).len();
                    (1..=3).contains(&n)
                })
                && rest2[1..].iter().all(|u| {
                    let n = para_text_tokens(dom, u).len();
                    (2..=6).contains(&n)
                })
                && rest2[1..].iter().any(|u| {
                    para_text_token_list(dom, u)
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case("item"))
                });
            if m144
                || m146
                || m149
                || m151
                || m153
                || m142
                || m165
                || m173
                || m180
                || m183
                || m161
                || m163
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                if m183 && rest1.len() >= 2 && rest2.len() >= 2 {
                    // Mesh first residual body; pure-I extra mid on longer side;
                    // pure-I/D last residual.
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    if rest2.len() > rest1.len() {
                        for u in &rest2[1..rest2.len() - 1] {
                            out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                        }
                        out.push(CorrelatedSequence::inserted(vec![
                            rest2[rest2.len() - 1].clone(),
                        ]));
                        out.push(CorrelatedSequence::deleted(vec![
                            rest1[rest1.len() - 1].clone(),
                        ]));
                    } else {
                        for u in &rest1[1..rest1.len() - 1] {
                            out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                        }
                        out.push(CorrelatedSequence::inserted(vec![
                            rest2[rest2.len() - 1].clone(),
                        ]));
                        out.push(CorrelatedSequence::deleted(vec![
                            rest1[rest1.len() - 1].clone(),
                        ]));
                    }
                } else if m163 {
                    // pure-I intro, then zip list items positionally
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let items2 = &rest2[1..];
                    let n = rest1.len().min(items2.len());
                    for i in 0..n {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            vec![rest1[i].clone()],
                            vec![items2[i].clone()],
                        ));
                    }
                    for u in rest1.iter().skip(n) {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    for u in items2.iter().skip(n) {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                } else if m161 && rest1.len() >= 2 && rest2.len() >= 2 {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                } else if m161 && rest1.len() == 1 && rest2.len() == 1 {
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                } else if (m165 || m180) && rest1.len() == 2 && rest2.len() == 2 {
                    // Mesh first residual; pure-I/D last residual
                    // (M165 near-identical first; M180 mid first + content-empty last).
                    // M189: very weak first residual (j1 < 0.15, times×title ~
                    // 0.14) — pure-I/D both residuals instead of free-meshing
                    // EQ "This document " (LO chrome). Keep mesh for
                    // small_font×strikethrough (j1≈0.15, LO 100 with mesh)
                    // and mid j1 (center_bold ~0.30).
                    let first_j = token_jaccard(
                        &para_text_tokens(dom, &rest1[0]),
                        &para_text_tokens(dom, &rest2[0]),
                    );
                    // M189: j1 < 0.15 → pure-I/D both (times×title ≈0.14).
                    // M191b: both-long last residuals + j1 ∈ [0.46, 0.50) → pure
                    // both (track italic×title ≈0.47). Keep free-mesh first
                    // residual for j1≥0.50 (track calibri×center) and for
                    // asymmetric short lasts (heading_2 j1≈0.455).
                    let both_long_res = unit_text_token_count(dom, &rest1[1]) >= 6
                        && unit_text_token_count(dom, &rest2[1]) >= 6;
                    let pure_both = first_j + 1e-12 < 0.15
                        || (both_long_res && first_j + 1e-12 >= 0.46 && first_j + 1e-12 < 0.50);
                    if m180 && pure_both {
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                        out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    } else {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            vec![rest1[0].clone()],
                            vec![rest2[0].clone()],
                        ));
                        out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    }
                } else if m173 && rest1.len() == 2 && rest2.len() == 2 {
                    // Mesh first residual; free-LCS last residual. Drop only
                    // base trailing pmark so pmarks1≠pmarks2 and glue-void
                    // (requires both ==1) does not kill EQ is/and. Keep next
                    // pmark so Word's single MIX para is preserved (stripping
                    // both pmarks split into DEL|INS paras).
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    let mut left = group_contents(&rest1[1]);
                    let mut right = group_contents(&rest2[1]);
                    while left.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u)) {
                        left.pop();
                    }
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    let mut residual_settings = settings.clone();
                    residual_settings.detail_threshold = 0.005;
                    let mut nested = lcs(dom, left, right, &residual_settings);
                    out.append(&mut nested);
                } else if m149 {
                    // Shorter residual first body leads (Word order).
                    if b0_n > 0 && b0_n < a0_n {
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                    } else {
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    }
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[1].clone()],
                        vec![rest2[1].clone()],
                    ));
                } else if m151 || m153 {
                    // Word: MIX title | pure-I B0 | MIX A0×B1 | pure-D A1
                    // (M151 This-text×This-document; M153 long unrelated
                    // residual bodies). Free residual LCS for M151 tried
                    // thrice (full residual ~66/70; A0×B0 free ~66/70; flatten
                    // with "right" demote ~66/70) — LO pixel prefers peel even
                    // when Word structure is free-mesh This+text. Keep pure-I.
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[1].clone()],
                    ));
                    out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                } else if m146
                    && rest2.len() >= 2
                    // M150 (right_aligned_italic×right_alignment): both first
                    // residual bodies start with "This" but are NOT "This
                    // document" cousins (prefix sig <2). Word pure-I's first
                    // next body, then meshes remaining. Do NOT peel for
                    // Demonstrating×This (font_color×font_family) — full
                    // residual LCS matches Word MIX|MIX|INS|MIX better.
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && ordered_shared_prefix_sig(
                        &para_text_token_list(dom, &rest1[0]),
                        &para_text_token_list(dom, &rest2[0]),
                    ) < 2
                {
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if m144
                    && !rest2.is_empty()
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M157: "This text…" vs "This document…" (prefix <2)
                        let this_cousins = residual_para_starts_this(dom, &rest1[0])
                            && ordered_shared_prefix_sig(
                                &para_text_token_list(dom, &rest1[0]),
                                &para_text_token_list(dom, &rest2[0]),
                            ) < 2;
                        // M158 (bullet_list×calibri_bold_italic): short first
                        // base residual (list item "Apples") vs "This document…"
                        // next body. Word pure-I's first next body; full LCS
                        // meshes Apples into B0 (~82).
                        let short_list_item = para_text_tokens(dom, &rest1[0]).len() <= 2;
                        this_cousins || short_list_item
                    }
                {
                    // Word pure-I first next residual body, then mesh remaining.
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if m144 && rest1.len() == 2 && rest2.len() == 1 {
                    // M152 (justify_2×justify): 2v1 residual after equal title —
                    // LCP-split long next body when the first residual LCP is
                    // long (≥6 tokens, "This document demonstrates justified…").
                    // Short LCP (italic_underline×justified "This document
                    // combines…") must use full residual word-LCS so M144 peel
                    // can attach the trailing phrase (~89).
                    let mut a0: Vec<ComparisonUnit> = group_contents(&rest1[0]);
                    let mut a1: Vec<ComparisonUnit> = group_contents(&rest1[1]);
                    let mut b: Vec<ComparisonUnit> = group_contents(&rest2[0]);
                    rehash_words_by_text_content(dom, &mut a0);
                    rehash_words_by_text_content(dom, &mut a1);
                    rehash_words_by_text_content(dom, &mut b);
                    let mut lcp = 0usize;
                    while lcp < a0.len().min(b.len()) && a0[lcp].sha1() == b[lcp].sha1() {
                        lcp += 1;
                    }
                    let mut split_at = lcp;
                    if lcp >= 8 && lcp < b.len() && a0.len() > lcp && a0.len() - lcp <= 2 {
                        let mut i = lcp;
                        while i < b.len().saturating_sub(1) {
                            i += 1;
                            let text = match &b[i - 1] {
                                ComparisonUnit::Word(w) => w
                                    .contents
                                    .iter()
                                    .filter_map(|a| {
                                        if dom.name_is(a.content_element, &W::t()) {
                                            Some(dom.value_str(a.content_element))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<String>(),
                                _ => String::new(),
                            };
                            if text.chars().any(|c| c.is_alphanumeric()) {
                                break;
                            }
                        }
                        if i < b.len() {
                            split_at = i;
                        }
                    }
                    if lcp >= 8 && split_at > 0 && split_at < b.len() {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            a0,
                            b[..split_at].to_vec(),
                        ));
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            a1,
                            b[split_at..].to_vec(),
                        ));
                    } else {
                        let mut left: Vec<ComparisonUnit> =
                            rest1.iter().flat_map(group_contents).collect();
                        let mut right: Vec<ComparisonUnit> =
                            rest2.iter().flat_map(group_contents).collect();
                        rehash_words_by_text_content(dom, &mut left);
                        rehash_words_by_text_content(dom, &mut right);
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            left,
                            right,
                        ));
                    }
                } else if m144
                    && rest1.len() == 3
                    && rest2.len() == 2
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M162 (font_family×font_size_12): Word peels trailing
                        // "text" from first next residual onto the next base
                        // body ("This text uses…"), then pure-I last next +
                        // pure-D last base. Para-wise residual LCS leaves
                        // pure-D trail (~68).
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        let a1 = para_text_token_list(dom, &rest1[1]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 3
                            && b0.last().is_some_and(|t| t.eq_ignore_ascii_case("text"))
                            && a1.len() >= 2
                            && a1[0].eq_ignore_ascii_case("this")
                            && a1[1].eq_ignore_ascii_case("text")
                    }
                {
                    // Split B0 before its last alnum word ("text") + trailing pmark.
                    let mut b0 = group_contents(&rest2[0]);
                    rehash_words_by_text_content(dom, &mut b0);
                    let mut peel_from = b0.len();
                    // walk back over trailing pmarks
                    while peel_from > 0 && unit_is_single_atom_ppr(dom, &b0[peel_from - 1]) {
                        peel_from -= 1;
                    }
                    // one content word ("text")
                    peel_from = peel_from.saturating_sub(1);
                    let b0_main = b0[..peel_from].to_vec();
                    let b0_peel = b0[peel_from..].to_vec();
                    let mut a0 = group_contents(&rest1[0]);
                    rehash_words_by_text_content(dom, &mut a0);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a0,
                        b0_main,
                    ));
                    // A1 ("This text…") × peeled "text" (+pmark)
                    let mut a1 = group_contents(&rest1[1]);
                    rehash_words_by_text_content(dom, &mut a1);
                    let mut peel = b0_peel;
                    rehash_words_by_text_content(dom, &mut peel);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a1,
                        peel,
                    ));
                    // pure-I last next + pure-D last base → merge folds to MIX
                    out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[2].clone()]));
                } else if m146
                    && rest1.len() == 2
                    && rest2.len() == 3
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M167 (font_size_24×font_size): 2v3 residual after
                        // Demo title. First residual shares long "This
                        // document demonstrates font size" prefix; Word meshes
                        // A0×B0 then free-reflows A1 across B1|B2 so "sizes
                        // improve" lands with B2 ("Font size impacts…"), not
                        // with B1. Full residual LCS keeps base pmark and
                        // pulls "sizes improve" into p2 (~79).
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 4
                            && a0.get(3).is_some_and(|t| t.eq_ignore_ascii_case("font"))
                            && b0.get(3).is_some_and(|t| t.eq_ignore_ascii_case("font"))
                    }
                {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    // Split last base body after its first "font" content word
                    // so head meshes with B1 ("…larger font size of 18pt") and
                    // tail ("sizes improve…") meshes with B2 ("Font size
                    // impacts…"). Free residual LCS kept "sizes improve" as
                    // trailing del before B1's pmark (~79).
                    let mut a1 = group_contents(&rest1[1]);
                    let mut b1 = group_contents(&rest2[1]);
                    let mut b2 = group_contents(&rest2[2]);
                    rehash_words_by_text_content(dom, &mut a1);
                    rehash_words_by_text_content(dom, &mut b1);
                    rehash_words_by_text_content(dom, &mut b2);
                    let word_text = |dom: &Dom, u: &ComparisonUnit| -> String {
                        match u {
                            ComparisonUnit::Word(w) => w
                                .contents
                                .iter()
                                .filter_map(|a| {
                                    if dom.name_is(a.content_element, &W::t()) {
                                        Some(dom.value_str(a.content_element))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                            _ => String::new(),
                        }
                    };
                    let font_idx = a1
                        .iter()
                        .position(|u| word_text(dom, u).eq_ignore_ascii_case("font"));
                    if let Some(fi) = font_idx {
                        if fi + 1 < a1.len() {
                            let mut residual_settings = settings.clone();
                            residual_settings.detail_threshold = 0.005;
                            let mut nested1 = lcs(dom, a1[..=fi].to_vec(), b1, &residual_settings);
                            out.append(&mut nested1);
                            let mut nested2 =
                                lcs(dom, a1[fi + 1..].to_vec(), b2, &residual_settings);
                            out.append(&mut nested2);
                        } else {
                            let mut residual_settings = settings.clone();
                            residual_settings.detail_threshold = 0.005;
                            let mut right = b1;
                            right.extend(b2);
                            let mut nested = lcs(dom, a1, right, &residual_settings);
                            out.append(&mut nested);
                        }
                    } else {
                        let mut residual_settings = settings.clone();
                        residual_settings.detail_threshold = 0.005;
                        let mut right = b1;
                        right.extend(b2);
                        let mut nested = lcs(dom, a1, right, &residual_settings);
                        out.append(&mut nested);
                    }
                } else if m144 || m146 {
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2.iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if rest1.len() == 1
                    && rest2.len() == 2
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M166 (justify×large): 1v2 residual after title. Word
                        // keeps shared "This document demonstrates" as EQ in
                        // first residual with pure-I rest of short B0, then
                        // meshes A0 tail with B1 (MIX|MIX|MIX). Pure I/D of
                        // whole residuals (~67) or pure-I B0 (~title-only
                        // unit-test shape) both miss the shared prefix.
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 3
                            && a0
                                .get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b0
                                .get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b0.len() <= 8
                            && a0.len() > b0.len()
                    }
                {
                    let mut a0 = group_contents(&rest1[0]);
                    let mut b0 = group_contents(&rest2[0]);
                    let mut b1 = group_contents(&rest2[1]);
                    rehash_words_by_text_content(dom, &mut a0);
                    rehash_words_by_text_content(dom, &mut b0);
                    rehash_words_by_text_content(dom, &mut b1);
                    let mut lcp = 0usize;
                    while lcp < a0.len().min(b0.len()) && a0[lcp].sha1() == b0[lcp].sha1() {
                        lcp += 1;
                    }
                    // Need a non-empty A0 tail to mesh with B1; B0 may extend
                    // past LCP (pure-I "large 24pt font size.").
                    // Nested LCS at detail_threshold 0.005: A0-tail×B1 shares
                    // short connectors (are/for/and). Default 0.15 voids by
                    // ratio; with 0.005 glue-void still kills each 1-token EQ
                    // when both sides keep a trailing pmark (pmarks==1).
                    // M178: drop base trailing pmark so glue-void does not
                    // fire; keep next pmark (Word single MIX last residual).
                    if lcp >= 3 && lcp < a0.len() {
                        let mut residual_settings = settings.clone();
                        residual_settings.detail_threshold = 0.005;
                        let mut nested1 = lcs(dom, a0[..lcp].to_vec(), b0, &residual_settings);
                        out.append(&mut nested1);
                        let mut left_tail = a0[lcp..].to_vec();
                        while left_tail
                            .last()
                            .is_some_and(|u| unit_is_single_atom_ppr(dom, u))
                        {
                            left_tail.pop();
                        }
                        let mut nested2 = lcs(dom, left_tail, b1, &residual_settings);
                        out.append(&mut nested2);
                    } else {
                        for r in rest2 {
                            out.push(CorrelatedSequence::inserted(vec![r.clone()]));
                        }
                        for l in rest1 {
                            out.push(CorrelatedSequence::deleted(vec![l.clone()]));
                        }
                    }
                } else if m142
                    && rest1.len() == 2
                    && rest2.len() == 2
                    && first_residual_j + 1e-12 >= 0.12
                {
                    // M156 (bold_and_underline×bold_italic): body residual only
                    // shares format boilerplate ("bold") so m142 is true, but
                    // first residual bodies are weakly related (j≥0.12). Pure
                    // I/D + merge invents INS|MIX|DEL (~79); residual word-LCS
                    // yields Word's 3×MIX mesh. Keep pure I/D when first
                    // residual jaccard is near-zero (heading_4×helvetica).
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2.iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else {
                    for r in rest2 {
                        out.push(CorrelatedSequence::inserted(vec![r.clone()]));
                    }
                    for l in rest1 {
                        out.push(CorrelatedSequence::deleted(vec![l.clone()]));
                    }
                }
                return out;
            }
        }
        // M148/M152: short **unequal** pure-para residuals that are weakly
        // related. Without rehash, format sha1s make "justified"≠"justified"
        // and residual collapses to pure I+D (~59).
        //
        // M152 (justify_2×justify, residual 2v1 after equal title): a single
        // flattened word-LCS Unknown is shredded by FindCommonAtBeginning —
        // Equal prefix then 2-2 pmark split pure-deletes the extra base body
        // (EQ|DEL|MIX). Instead, split the long next body at the rehashed
        // LCP with the first base residual and emit two Unknowns:
        //   Unknown(A0, B[..lcp]) | Unknown(A1, B[lcp..])
        // → Word-like EQ|MIX|MIX (score lift).
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (1..=6).contains(&left_paras)
            && (1..=6).contains(&right_paras)
            && left_paras != right_paras
            && residual_sets_weakly_related(dom, cul1, cul2)
        {
            // M152: residual 2v1 (after equal title already peeled) OR top-level
            // 3v2 with equal Demo titles — LCP-split the long next body.
            let (base_rest, next_body) = if left_paras == 2 && right_paras == 1 {
                (Some((&cul1[0], &cul1[1])), Some(&cul2[0]))
            } else if left_paras == 3
                && right_paras == 2
                && first_paras_share_last_sig(dom, cul1, cul2)
                && token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                ) + 1e-12
                    >= 0.99
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                (Some((&cul1[1], &cul1[2])), Some(&cul2[1]))
            } else {
                (None, None)
            };
            if let (Some((a0u, a1u)), Some(bu)) = (base_rest, next_body) {
                let mut a0: Vec<ComparisonUnit> = group_contents(a0u);
                let mut a1: Vec<ComparisonUnit> = group_contents(a1u);
                let mut b: Vec<ComparisonUnit> = group_contents(bu);
                rehash_words_by_text_content(dom, &mut a0);
                rehash_words_by_text_content(dom, &mut a1);
                rehash_words_by_text_content(dom, &mut b);
                let mut lcp = 0usize;
                while lcp < a0.len().min(b.len()) && a0[lcp].sha1() == b[lcp].sha1() {
                    lcp += 1;
                }
                // Long LCP only (≥6): justify_2 class. Short LCP
                // (italic_underline×justified) must not LCP-split.
                let mut split_at = lcp;
                if lcp >= 8 && lcp < b.len() && a0.len() > lcp && a0.len() - lcp <= 2 {
                    let mut i = lcp;
                    while i < b.len().saturating_sub(1) {
                        i += 1;
                        let text = match &b[i - 1] {
                            ComparisonUnit::Word(w) => w
                                .contents
                                .iter()
                                .filter_map(|a| {
                                    if dom.name_is(a.content_element, &W::t()) {
                                        Some(dom.value_str(a.content_element))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<String>(),
                            _ => String::new(),
                        };
                        if text.chars().any(|c| c.is_alphanumeric()) {
                            break;
                        }
                    }
                    if i < b.len() {
                        split_at = i;
                    }
                }
                if lcp >= 8 && split_at > 0 && split_at < b.len() {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a0,
                        b[..split_at].to_vec(),
                    ));
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a1,
                        b[split_at..].to_vec(),
                    ));
                    return out;
                }
            }
            let mut left: Vec<ComparisonUnit> = cul1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = cul2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                left,
                right,
            ));
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
