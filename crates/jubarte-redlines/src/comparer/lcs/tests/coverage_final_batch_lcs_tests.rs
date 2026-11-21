// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};

fn group(
    kind: ComparisonUnitGroupType,
    contents: Vec<ComparisonUnit>,
    key: &str,
) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: kind,
        contents,
        level: 0,
        sha1: Sha1Keyed::new(key.to_string()),
        correlated_sha1_hash: None,
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

fn paragraph(dom: &mut Dom, parent: NodeId, text: &str, marked: bool) -> ComparisonUnit {
    let p = dom.new_element(W::p());
    dom.add(parent, p);
    let ppr = dom.new_element(W::p_pr());
    dom.add(p, ppr);
    let mut contents = Vec::new();
    for text in text.split_inclusive(' ') {
        let r = dom.new_element(W::r());
        dom.add(p, r);
        let t = dom.new_element(W::t());
        dom.add(r, t);
        dom.add_text(t, text);
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(t, vec![p, r], text),
        ])));
    }
    if marked {
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
        ])));
    }
    group(
        ComparisonUnitGroupType::Paragraph,
        contents,
        &format!("para:{text}:{marked}"),
    )
}

fn document(dom: &mut Dom, texts: &[&str], marked: bool) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    texts
        .iter()
        .map(|text| paragraph(dom, body, text, marked))
        .collect()
}

fn atom_ids(units: &[ComparisonUnit]) -> Vec<NodeId> {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| a.content_element)
        .collect()
}

fn side_ids(seqs: &[CorrelatedSequence], revised: bool) -> Vec<NodeId> {
    seqs.iter()
        .flat_map(|s| {
            if revised {
                s.com_units_2.as_deref().unwrap_or_default()
            } else {
                s.com_units_1.as_deref().unwrap_or_default()
            }
        })
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| a.content_element)
        .collect()
}

#[test]
fn seam_matrix_preserves_every_source_atom_once_in_source_order() {
    for left_count in 0..=4 {
        for right_count in 0..=4 {
            for left_mark in [false, true] {
                for right_mark in [false, true] {
                    let mut dom = Dom::new();
                    let left = document(&mut dom, &vec!["original text"; left_count], left_mark)
                        .iter()
                        .flat_map(group_contents)
                        .collect::<Vec<_>>();
                    let right = document(&mut dom, &vec!["revised text"; right_count], right_mark)
                        .iter()
                        .flat_map(group_contents)
                        .collect::<Vec<_>>();
                    let mut seqs = Vec::new();
                    seam_region(&dom, &left, &right, &mut seqs);
                    assert_eq!(side_ids(&seqs, false), atom_ids(&left));
                    let mut expected_right = atom_ids(&right);
                    let original_carrier =
                        left.iter().position(|u| unit_is_single_atom_ppr(&dom, u));
                    let absorbs_revised_mark = original_carrier.is_some_and(|i| i + 1 < left.len())
                        && right
                            .last()
                            .is_some_and(|u| unit_is_single_atom_ppr(&dom, u));
                    if absorbs_revised_mark {
                        let mark = expected_right.pop().unwrap();
                        assert!(dom.name_is(mark, &W::p_pr()));
                        let carrier = left[original_carrier.unwrap()]
                            .first_atom()
                            .unwrap()
                            .content_element;
                        assert!(
                            seqs.iter().any(|seq| seq.correlation_status
                                == CorrelationStatus::Deleted
                                && side_ids(std::slice::from_ref(seq), false).contains(&carrier))
                        );
                    }
                    assert_eq!(side_ids(&seqs, true), expected_right);
                    for seq in &seqs {
                        if seq.correlation_status == CorrelationStatus::Equal {
                            assert_eq!(seq.com_units_1.as_ref().unwrap().len(), 1);
                            assert_eq!(seq.com_units_2.as_ref().unwrap().len(), 1);
                            assert!(unit_is_single_atom_ppr(
                                &dom,
                                &seq.com_units_1.as_ref().unwrap()[0]
                            ));
                            assert!(unit_is_single_atom_ppr(
                                &dom,
                                &seq.com_units_2.as_ref().unwrap()[0]
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn title_page_and_math_fingerprints_follow_tokenization_and_respect_bounds() {
    // The detector sees normalized alphanumeric tokens: punctuation alone
    // cannot manufacture an email or bracket marker after normalization.
    for marker in [
        "agreement",
        "apprenticeship",
        "January 2026",
        "report@example.test",
        "[Client Name]",
        "routine heading",
    ] {
        for count in [3, 4, 12, 13, 15] {
            let mut dom = Dom::new();
            let mut texts = vec!["ordinary heading"; count];
            texts[0] = marker;
            texts[1] = "Prepared by Alice";
            let units = document(&mut dom, &texts, true);
            assert_eq!(
                looks_like_short_title_page(&dom, &units),
                (4..=12).contains(&count)
                    && ["agreement", "apprenticeship", "January 2026"].contains(&marker),
                "{marker} {count}"
            );
        }
    }
    for marker in [
        "m:box",
        "m:borderbox",
        "border math box",
        "borderBox",
        "border box",
        "math alone",
    ] {
        for blank_prefix in [0, 29, 30] {
            let mut dom = Dom::new();
            let mut texts = vec![""; blank_prefix];
            texts.push(marker);
            let units = document(&mut dom, &texts, true);
            assert_eq!(
                looks_like_math_borderbox_doc(&dom, &units),
                blank_prefix < 30
                    && ["m:borderbox", "border math box", "borderBox"].contains(&marker),
                "{marker} {blank_prefix}"
            );
        }
    }
}

#[test]
fn junction_seam_declines_equal_counts_nonparagraph_carriers_and_blank_revised_tail() {
    for left_count in [1, 2, 3] {
        for right_count in [1, 2, 3] {
            for blank_tail in [false, true] {
                for table_first in [false, true] {
                    let mut dom = Dom::new();
                    let mut left = document(
                        &mut dom,
                        &vec!["original contract provisions"; left_count],
                        true,
                    );
                    let mut right_texts = vec!["new regional instructions"; right_count];
                    if blank_tail {
                        right_texts[right_count - 1] = "";
                    }
                    let right = document(&mut dom, &right_texts, true);
                    if table_first {
                        left[0] = group(
                            ComparisonUnitGroupType::Table,
                            vec![left[0].clone()],
                            "table",
                        );
                    }
                    let seqs = junction_seam(&dom, &left, &right, left_count, right_count);
                    let eligible = left_count != right_count && !blank_tail && !table_first;
                    assert_eq!(
                        seqs.is_some(),
                        eligible,
                        "{left_count} {right_count} {blank_tail} {table_first}"
                    );
                    if let Some(seqs) = seqs {
                        assert_eq!(side_ids(&seqs, false), atom_ids(&left));
                        // Junction pairs only a one-paragraph original's closing mark.
                        let mut expected_right = atom_ids(&right);
                        if left_count > 1 {
                            expected_right.pop();
                        }
                        assert_eq!(side_ids(&seqs, true), expected_right);
                        assert_eq!(
                            seqs.iter()
                                .filter(|s| s.correlation_status == CorrelationStatus::Equal)
                                .count(),
                            usize::from(left_count == 1)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn grouped_blank_closing_runs_peel_only_matching_kinds_and_closing_stories() {
    for left_blank in [false, true] {
        for right_blank in [false, true] {
            for matching_kind in [false, true] {
                for left_closes in [false, true] {
                    for right_closes in [false, true] {
                        let mut dom = Dom::new();
                        let body_a = dom.new_element(W::body());
                        let body_b = dom.new_element(W::body());
                        let a0 = paragraph(&mut dom, body_a, "earlier source", true);
                        let a1 = paragraph(
                            &mut dom,
                            body_a,
                            if left_blank { "" } else { "source tail" },
                            true,
                        );
                        let b0 = paragraph(&mut dom, body_b, "earlier revision", true);
                        let b1 = paragraph(&mut dom, body_b, "middle revision", true);
                        let b2 = paragraph(
                            &mut dom,
                            body_b,
                            if right_blank { "" } else { "revision tail" },
                            true,
                        );
                        if !left_closes {
                            let _ = paragraph(&mut dom, body_a, "following live source", true);
                        }
                        if !right_closes {
                            let _ = paragraph(&mut dom, body_b, "following live revision", true);
                        }
                        let mut a = vec![(0, vec![a0]), (1, vec![a1])];
                        let mut b = vec![
                            (0, vec![b0]),
                            (0, vec![b1]),
                            (if matching_kind { 1 } else { 2 }, vec![b2]),
                        ];
                        let original_a = a.clone();
                        let original_b = b.clone();
                        let tail = peel_story_final_groups(&dom, &mut a, &mut b);
                        let eligible = matching_kind
                            && left_closes
                            && right_closes
                            && (left_blank || right_blank);
                        assert_eq!(tail.is_some(), eligible);
                        if let Some((a_tail, b_tail)) = tail {
                            assert_eq!(a.len(), 1);
                            assert_eq!(b.len(), 2);
                            assert_eq!(atom_ids(&a_tail), atom_ids(&original_a[1].1));
                            assert_eq!(atom_ids(&b_tail), atom_ids(&original_b[2].1));
                        } else {
                            assert_eq!(a.len(), original_a.len());
                            assert_eq!(b.len(), original_b.len());
                            assert_eq!(atom_ids(&a[1].1), atom_ids(&original_a[1].1));
                            assert_eq!(atom_ids(&b[2].1), atom_ids(&original_b[2].1));
                        }
                    }
                }
            }
        }
    }
}
