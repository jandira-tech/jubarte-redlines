// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
use ComparisonUnitGroupType::{Cell, Paragraph, Row, Table, Textbox};
use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

fn group(
    kind: ComparisonUnitGroupType,
    contents: Vec<ComparisonUnit>,
    hash: &str,
) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: kind,
        contents,
        level: 0,
        sha1: Sha1Keyed::new(hash.to_string()),
        correlated_sha1_hash: None,
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

fn word(dom: &mut Dom, text: &str) -> ComparisonUnit {
    let node = dom.new_element(W::t());
    dom.add_text(node, text);
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        node,
        Vec::<NodeId>::new(),
        text,
    )]))
}

fn empty_word(hash: &str) -> ComparisonUnit {
    ComparisonUnit::Word(ComparisonUnitWord {
        correlation_status: CorrelationStatus::Nil,
        contents: Vec::new(),
        sha1: Sha1Keyed::new(hash.to_string()),
    })
}

fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    texts
        .iter()
        .map(|text| {
            let p = dom.new_element(W::p());
            dom.add(body, p);
            let ppr = dom.new_element(W::p_pr());
            dom.add(p, ppr);
            // Properties remain attached to the original nodes.
            let spacing = dom.new_element(W::spacing_el());
            dom.set_attribute_value(spacing, &W::name("before"), Some("120"));
            dom.add(ppr, spacing);
            let mut contents = Vec::new();
            if !text.is_empty() {
                let run = dom.new_element(W::r());
                dom.add(p, run);
                let node = dom.new_element(W::t());
                dom.add_text(node, text);
                dom.add(run, node);
                contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                    ComparisonUnitAtom::new(node, vec![p, run], *text),
                ])));
            }
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(ppr, vec![p], "mark"),
            ])));
            group(Paragraph, contents, &format!("paragraph:{text}"))
        })
        .collect()
}

fn inline(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
    paragraphs(dom, texts)
        .iter()
        .flat_map(group_contents)
        .collect()
}

fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| {
            if atom_is_ppr(dom, a) {
                "¶".to_string()
            } else {
                dom.value_str(a.content_element).into_owned()
            }
        })
        .collect()
}

fn geometry(units: &[ComparisonUnit]) -> Vec<(NodeId, Vec<NodeId>, String)> {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| {
            (
                a.content_element,
                a.ancestor_elements.to_vec(),
                a.sha1_hash.to_hex_string(),
            )
        })
        .collect()
}

fn check(
    dom: &Dom,
    out: &[CorrelatedSequence],
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
    expected: &[(CorrelationStatus, &str, &str)],
) {
    let actual: Vec<_> = out
        .iter()
        .map(|s| {
            (
                s.correlation_status,
                text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                text(dom, s.com_units_2.as_deref().unwrap_or_default()),
            )
        })
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|&(s, a, b)| (s, a.to_string(), b.to_string()))
        .collect();
    assert_eq!(actual, expected);
    for (is_left, source) in [(true, left), (false, right)] {
        let emitted: Vec<_> = out
            .iter()
            .flat_map(|s| {
                geometry(if is_left {
                    s.com_units_1.as_deref().unwrap_or_default()
                } else {
                    s.com_units_2.as_deref().unwrap_or_default()
                })
            })
            .collect();
        assert_eq!(emitted, geometry(source));
    }
}

fn unknown(left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>) -> CorrelatedSequence {
    CorrelatedSequence::paired(Unknown, left, right)
}

fn faithful() -> WmlComparerSettings {
    WmlComparerSettings {
        merge_replaced_paragraphs: false,
        detail_threshold: 0.0,
        ..WmlComparerSettings::default()
    }
}

#[test]
fn structurally_empty_groups_are_not_bare_marks_or_content_anchors() {
    let dom = Dom::new();
    for kind in [Paragraph, Table, Row, Cell, Textbox] {
        let u = group(kind, vec![], "empty");
        assert_eq!(group_contents(&u).len(), 0);
        assert_eq!(u.descendant_content_atoms_count(), 0);
        assert!(u.first_atom().is_none());
        assert!(u.last_atom().is_none());
        assert_eq!(
            last_atom_overall_is_ppr(&dom, std::slice::from_ref(&u)),
            None
        );
        assert!(!unit_is_single_atom_ppr(&dom, &u));
        assert!(!unit_first_atom_is_ppr(&dom, &u));
        assert!(!unit_last_atom_is_ppr(&dom, &u));
        assert!(!word_first_not_ppr(&dom, &u));
        assert!(!unit_has_text_token(&dom, &u));
        assert!(!unit_closes_story(&dom, &u));
        assert_eq!(
            unit_is_textless_paragraph_matter(&dom, &u),
            kind == Paragraph
        );
        assert!(contentful_group_sha1s(&dom, &[u]).is_empty());
    }
    let u = empty_word("empty-word");
    assert!(word_first_not_ppr(&dom, &u));
    assert!(!unit_is_paragraph_matter(&dom, &u));
    assert!(first_direct_atom(&u).is_none());
    assert_eq!(find_index_of_next_para_mark(&dom, &[u]), 1);
}

#[test]
fn empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score() {
    let mut dom = Dom::new();
    let mut left = inline(&mut dom, &["substance"]);
    left.push(group(Table, vec![], "empty-table"));
    assert_eq!(last_atom_overall_is_ppr(&dom, &left), Some(true));
    assert_eq!(first_contentful_group_index(&dom, &left), None);
    let p = paragraphs(&mut dom, &["substance"]);
    let units = vec![group(Paragraph, vec![], "empty"), p[0].clone()];
    assert_eq!(first_contentful_group_index(&dom, &units), Some(1));
    assert_eq!(
        non_separator_prefix_sums(&dom, &units, &WmlComparerSettings::default()),
        vec![0, 0, 9]
    );
    assert_eq!(
        closing_paragraphs(&dom, &units),
        vec![(1, false), (0, true)]
    );
    assert!(!within_one_paragraph(&dom, &units));
    assert_eq!(text(&dom, &flatten_groups_one_level(&units)), "substance¶");
}

#[test]
fn zero_atom_correlated_groups_decline_without_discarding_structure() {
    let make = || {
        (0..4)
            .map(|i| {
                let mut u = group(Paragraph, vec![], &format!("empty-{i}"));
                if let ComparisonUnit::Group(g) = &mut u {
                    g.correlated_sha1_hash = Some(format!("c{i}"));
                }
                u
            })
            .collect::<Vec<_>>()
    };
    let input = unknown(make(), make());
    assert!(correlated_hash_run_scan(&input).is_none());
    assert!(correlated_hash_run_indexed(&input).is_none());
    let returned = process_correlated_hashes_owned(input).unwrap_err();
    for side in [&returned.com_units_1, &returned.com_units_2] {
        assert_eq!(
            side.as_ref()
                .unwrap()
                .iter()
                .map(ComparisonUnit::sha1)
                .collect::<Vec<_>>(),
            vec!["empty-0", "empty-1", "empty-2", "empty-3"]
        );
    }
}

#[test]
fn correlated_scan_rejects_short_and_ineligible_leading_units_on_either_side() {
    let mut dom = Dom::new();
    let base = paragraphs(&mut dom, &["a", "b", "c", "d"]);
    for left_len in [0, 1, 2] {
        let input = unknown(base[..left_len].to_vec(), base.clone());
        assert!(correlated_hash_run_scan(&input).is_none());
        assert!(correlated_hash_run_indexed(&input).is_none());
    }
    for kind in [Cell, Textbox] {
        for is_left in [true, false] {
            let mut a = base.clone();
            let mut b = base.clone();
            if is_left {
                a[0] = group(kind, vec![], "wrong-kind");
            } else {
                b[0] = group(kind, vec![], "wrong-kind");
            }
            let input = unknown(a, b);
            assert!(correlated_hash_run_scan(&input).is_none());
            assert!(correlated_hash_run_indexed(&input).is_none());
        }
    }
    for is_left in [true, false] {
        let mut a = base.clone();
        let mut b = base.clone();
        if is_left {
            a[0] = empty_word("word");
        } else {
            b[0] = empty_word("word");
        }
        let input = unknown(a, b);
        assert!(correlated_hash_run_scan(&input).is_none());
        assert!(correlated_hash_run_indexed(&input).is_none());
    }
}

#[test]
fn correlated_group_type_mismatch_breaks_a_run_even_with_equal_hashes() {
    let mut dom = Dom::new();
    let mut left = paragraphs(&mut dom, &["a", "b", "c", "d", "e"]);
    let mut right = paragraphs(&mut dom, &["a", "b", "c", "d", "e"]);
    for units in [&mut left, &mut right] {
        for (i, u) in units.iter_mut().enumerate() {
            if let ComparisonUnit::Group(g) = u {
                g.correlated_sha1_hash = Some(format!("corr-{i}"));
            }
        }
    }
    if let ComparisonUnit::Group(g) = &mut right[4] {
        g.group_type = Row;
    }
    let input = unknown(left.clone(), right.clone());
    for run in [
        correlated_hash_run_scan(&input),
        correlated_hash_run_indexed(&input),
    ] {
        let r = run.unwrap();
        assert_eq!((r.left_start, r.right_start, r.len), (0, 0, 4));
    }
    let out = process_correlated_hashes_owned(input).unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Unknown, "a¶", "a¶"),
            (Unknown, "b¶", "b¶"),
            (Unknown, "c¶", "c¶"),
            (Unknown, "d¶", "d¶"),
            (Unknown, "e¶", "e¶"),
        ],
    );
    assert_eq!(
        as_group(&out[4].com_units_2.as_ref().unwrap()[0])
            .unwrap()
            .group_type,
        Row
    );
}

#[test]
fn correlated_thresholds_require_atom_evidence_from_both_sides() {
    let mut dom = Dom::new();
    for (n, a, b, accepted) in [
        (1, 16, 17, false),
        (1, 17, 16, false),
        (1, 17, 17, true),
        (2, 16, 17, false),
        (2, 17, 16, false),
        (2, 17, 17, true),
        (3, 10, 11, false),
        (3, 11, 10, false),
        (3, 11, 11, true),
    ] {
        let make = |dom: &mut Dom, count: usize| {
            (0..n)
                .map(|i| {
                    group(
                        Paragraph,
                        (0..count).map(|_| word(dom, "x")).collect(),
                        &format!("g{i}"),
                    )
                })
                .collect::<Vec<_>>()
        };
        let left = make(&mut dom, a);
        let right = make(&mut dom, b);
        assert_eq!(
            correlated_hash_run_threshold(&left, &right, 0, 0, n),
            accepted,
            "n={n}, {a}/{b}"
        );
    }
    assert!(!correlated_hash_run_threshold(
        &[],
        &[],
        usize::MAX,
        usize::MAX,
        0
    ));
    let four = (0..4)
        .map(|_| group(Paragraph, vec![], "empty"))
        .collect::<Vec<_>>();
    assert!(correlated_hash_run_threshold(&four, &four, 0, 0, 4));
}

#[test]
fn residual_cousins_require_nonempty_bodies_in_both_orientations() {
    let mut dom = Dom::new();
    for (a, b, expected) in [
        ("", "This clause", false),
        ("This clause", "", false),
        ("", "", false),
        ("THIS quartz", "this zebra", true),
        ("quartz anchor", "zebra anchor", true),
        ("the and", "the or", false),
        ("with and", "with or", true),
        ("alpha", "zebra", false),
    ] {
        let left = paragraphs(&mut dom, &["left title", a]);
        let right = paragraphs(&mut dom, &["right title", b]);
        assert_eq!(
            residual_bodies_this_cousins(&dom, &left, &right),
            expected,
            "{a}/{b}"
        );
    }
    let two = paragraphs(&mut dom, &["title", "body"]);
    assert!(!residual_bodies_this_cousins(&dom, &two, &two[..1]));
    assert!(!residual_first_body_starts_this(&dom, &two[..1]));
    assert!(!body_residual_unrelated(&dom, &two, &two[..1]));
    assert!(!body_residual_unrelated(&dom, &two[..1], &two));
}

#[test]
fn empty_residual_statistics_and_colon_majorities_have_exact_boundaries() {
    let mut dom = Dom::new();
    let empty = paragraphs(&mut dom, &["", ""]);
    let nonempty = paragraphs(&mut dom, &["alpha", "beta"]);
    assert_eq!(m123_diagonal_stats(&dom, &[], &[]), (0.0, 0.0, 0.0));
    assert_eq!(m123_diagonal_stats(&dom, &empty, &empty), (1.0, 1.0, 1.0));
    assert!(!para_zip_diagonal_dominant(&dom, &nonempty, &nonempty[..1]));
    assert!(!para_zip_diagonal_dominant(&dom, &empty, &empty));
    assert!(!residual_sets_weakly_related(&dom, &nonempty, &empty));
    assert_eq!(body_token_overlap_ratio(&dom, &nonempty, &empty), 0.0);
    assert!(!residual_looks_like_colon_list(&dom, &[]));
    for (texts, expected) in [
        (vec!["a:", "plain"], true),
        (vec!["a:", "plain", "other"], false),
        (vec!["a:", "b:", "plain"], true),
        (vec!["", ""], false),
    ] {
        let rest = paragraphs(&mut dom, &texts);
        assert_eq!(residual_looks_like_colon_list(&dom, &rest), expected);
    }
}

#[test]
fn blank_paragraphs_do_not_supply_a_diagonal_overlap_majority() {
    let mut dom = Dom::new();
    for (left_texts, right_texts, expected) in [
        (["", "", "alpha"], ["", "", "alpha"], false),
        (["alpha", "beta", ""], ["alpha", "beta", ""], true),
        (["alpha", "", "beta"], ["alpha", "", "zebra"], false),
        (["", "", ""], ["", "", ""], false),
    ] {
        let left = paragraphs(&mut dom, &left_texts);
        let right = paragraphs(&mut dom, &right_texts);
        assert_eq!(para_zip_diagonal_dominant(&dom, &left, &right), expected);
        assert_eq!(para_zip_diagonal_dominant(&dom, &right, &left), expected);
    }
}

#[test]
fn interior_blank_chain_truth_table_preserves_asymmetric_fusion_rules() {
    // Bits name blank paragraphs, from head to tail. These are all
    // sixteen combinations of a two-paragraph replacement region.
    let expected = [
        [true, false, true, false],
        [true, true, false, false],
        [true, false, true, false],
        [false, false, false, true],
    ];
    let mut dom = Dom::new();
    for (a, row) in expected.iter().enumerate() {
        for (b, &holds) in row.iter().enumerate() {
            let texts = |bits: usize| {
                [
                    if bits & 2 == 0 { "head" } else { "" },
                    if bits & 1 == 0 { "tail" } else { "" },
                ]
            };
            let left = paragraphs(&mut dom, &texts(a));
            let right = paragraphs(&mut dom, &texts(b));
            assert_eq!(
                interior_blank_chain_holds(&dom, &left, &right),
                holds,
                "blank bits {a}/{b}"
            );
            let lf: Vec<_> = left.iter().flat_map(group_contents).collect();
            let rf: Vec<_> = right.iter().flat_map(group_contents).collect();
            assert_eq!(
                interior_blank_chain_holds(&dom, &lf, &rf),
                holds,
                "inline bits {a}/{b}"
            );
        }
    }
    let left = paragraphs(&mut dom, &["head", "tail"]);
    let right = paragraphs(&mut dom, &["tail"]);
    assert!(!interior_blank_chain_holds(&dom, &left, &right));
}

#[test]
fn same_slot_pairing_keeps_real_unit_indices_and_ignores_already_paired_bidders() {
    let mut dom = Dom::new();
    let mut left = paragraphs(&mut dom, &["alpha beta gamma", "alpha beta"]);
    let mut right = paragraphs(&mut dom, &["alpha beta gamma", "alpha"]);
    left.insert(0, group(Table, vec![], "left-table"));
    right.insert(1, group(Row, vec![], "right-row"));
    assert_eq!(same_slot_pairs(&dom, &left, &right), vec![(1, 0), (2, 2)]);
    let left = paragraphs(&mut dom, &["alpha beta", "unrelated"]);
    let right = paragraphs(&mut dom, &["alpha", "alpha beta gamma"]);
    assert_eq!(
        same_slot_pairs(&dom, &left, &right),
        Vec::<(usize, usize)>::new()
    );
    let left = paragraphs(&mut dom, &["alpha"]);
    let right = paragraphs(&mut dom, &["alpha one two", "beta", "gamma"]);
    assert_eq!(same_slot_pairs(&dom, &left, &right), vec![(0, 0)]);
    assert!(same_slot_pairs(&dom, &right, &[]).is_empty());
}

#[test]
fn paragraph_duplication_distinguishes_blank_buckets_and_group_positions() {
    let mut dom = Dom::new();
    let units = paragraphs(&mut dom, &["repeat", "", "unique", "repeat"]);
    for (pos, expected) in [(0, true), (1, false), (2, false), (3, true)] {
        assert_eq!(
            containing_paragraph_is_duplicated(&dom, &units, pos),
            expected
        );
    }
    let mut flattened: Vec<_> = units.iter().flat_map(group_contents).collect();
    flattened.insert(0, empty_word("empty"));
    assert!(containing_paragraph_is_duplicated(&dom, &flattened, 1));
    assert!(!containing_paragraph_is_duplicated(&dom, &flattened, 3));
    assert_eq!(text(&dom, &flattened), "repeat¶¶unique¶repeat¶");
}

#[test]
fn prefix_islands_keep_empty_words_and_group_properties_in_the_residual() {
    let mut dom = Dom::new();
    for empty_boundary in [false, true] {
        let a = if empty_boundary {
            empty_word("anchor")
        } else {
            word(&mut dom, "anchor")
        };
        let b = if empty_boundary {
            empty_word("anchor")
        } else {
            word(&mut dom, "anchor")
        };
        let mut left = vec![a];
        let mut right = vec![b];
        left.extend(inline(&mut dom, &["old", "left"]));
        right.extend(inline(&mut dom, &["new"]));
        let out = find_common_at_beginning_and_end(
            &dom,
            &unknown(left.clone(), right.clone()),
            &faithful(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (
                    Equal,
                    if empty_boundary { "" } else { "anchor" },
                    if empty_boundary { "" } else { "anchor" },
                ),
                (Unknown, "old¶left¶", "new¶"),
            ],
        );
    }
    let mut left = inline(&mut dom, &["kept", "old"]);
    let mut right = inline(&mut dom, &["kept", "new"]);
    // The prefix includes a mark; partial-paragraph splitting must stop.
    let out =
        find_common_at_beginning_and_end(&dom, &unknown(left.clone(), right.clone()), &faithful())
            .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[(Equal, "kept¶", "kept¶"), (Unknown, "old¶", "new¶")],
    );
    left.clear();
    right.clear();
    assert!(find_common_at_beginning_and_end(&dom, &unknown(left, right), &faithful()).is_none());
}

#[test]
fn prefix_two_chunk_islands_pair_symmetric_marks_and_retain_asymmetric_tails() {
    let mut dom = Dom::new();
    for (left_tail, right_tail) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut left = vec![word(&mut dom, "anchor")];
        let mut right = vec![word(&mut dom, "anchor")];
        left.extend(inline(&mut dom, &["old"]));
        right.extend(inline(&mut dom, &["new"]));
        if left_tail {
            left.extend(inline(&mut dom, &["left"]));
        }
        if right_tail {
            right.extend(inline(&mut dom, &["right"]));
        }
        let out = find_common_at_beginning_and_end(
            &dom,
            &unknown(left.clone(), right.clone()),
            &faithful(),
        )
        .unwrap();
        let lt = if left_tail { "¶left¶" } else { "¶" };
        let rt = if right_tail { "¶right¶" } else { "¶" };
        if left_tail == right_tail {
            check(
                &dom,
                &out,
                &left,
                &right,
                &[
                    (Equal, "anchor", "anchor"),
                    (Unknown, "old", "new"),
                    (Unknown, lt, rt),
                ],
            );
        } else {
            let a = format!("old{lt}");
            let b = format!("new{rt}");
            check(
                &dom,
                &out,
                &left,
                &right,
                &[(Equal, "anchor", "anchor"), (Unknown, &a, &b)],
            );
        }
    }
}

#[test]
fn suffix_islands_peel_each_partial_paragraph_without_crossing_prior_marks() {
    let mut dom = Dom::new();
    let mut left = inline(&mut dom, &["left block"]);
    let mut right = inline(&mut dom, &["right block"]);
    left.push(word(&mut dom, "old"));
    right.push(word(&mut dom, "new"));
    for units in [&mut left, &mut right] {
        units.push(word(&mut dom, "shared"));
        units.push(word(&mut dom, "tail"));
        units.extend(inline(&mut dom, &[""]));
    }
    let out =
        find_common_at_beginning_and_end(&dom, &unknown(left.clone(), right.clone()), &faithful())
            .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Unknown, "left block¶", "right block¶"),
            (Unknown, "old", "new"),
            (Equal, "sharedtail¶", "sharedtail¶"),
        ],
    );
}

#[test]
fn mark_only_suffixes_decline_and_zero_text_groups_remain_structural_equals() {
    let mut dom = Dom::new();
    for extra_word in [false, true] {
        let mut left = vec![word(&mut dom, "old")];
        let mut right = vec![word(&mut dom, "new")];
        if extra_word {
            left.push(word(&mut dom, "shared"));
            right.push(word(&mut dom, "shared"));
        }
        left.extend(inline(&mut dom, &[""]));
        right.extend(inline(&mut dom, &[""]));
        assert!(
            find_common_at_beginning_and_end(&dom, &unknown(left, right), &faithful()).is_none()
        );
    }
    let left = vec![
        group(Paragraph, vec![], "same-empty"),
        group(Row, vec![], "old-row"),
    ];
    let right = vec![
        group(Paragraph, vec![], "same-empty"),
        group(Row, vec![], "new-row"),
    ];
    let out =
        find_common_at_beginning_and_end(&dom, &unknown(left.clone(), right.clone()), &faithful())
            .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[(Equal, "", ""), (Unknown, "", "")],
    );
    assert_eq!(out[1].com_units_1.as_ref().unwrap()[0].sha1(), "old-row");
    assert_eq!(out[1].com_units_2.as_ref().unwrap()[0].sha1(), "new-row");
}

#[test]
fn seam_empty_side_truth_table_never_manufactures_a_paragraph_mark() {
    let mut dom = Dom::new();
    for (has_left, has_right, expected) in [
        (false, false, vec![]),
        (true, false, vec![(Deleted, "old¶", "")]),
        (false, true, vec![(Inserted, "", "new¶")]),
        (
            true,
            true,
            vec![
                (Inserted, "", "new"),
                (Deleted, "old", ""),
                (Equal, "¶", "¶"),
            ],
        ),
    ] {
        let left = if has_left {
            inline(&mut dom, &["old"])
        } else {
            vec![]
        };
        let right = if has_right {
            inline(&mut dom, &["new"])
        } else {
            vec![]
        };
        let mut out = Vec::new();
        seam_region(&dom, &left, &right, &mut out);
        check(&dom, &out, &left, &right, &expected);
    }
    let left = vec![word(&mut dom, "unmarked")];
    let right = inline(&mut dom, &["revised"]);
    let mut out = Vec::new();
    seam_region(&dom, &left, &right, &mut out);
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "revised"),
            (Deleted, "unmarked", ""),
            (Inserted, "", "¶"),
        ],
    );
}

#[test]
fn final_mark_repair_declines_each_invalid_tail_without_mutation() {
    let mut dom = Dom::new();
    // One invalid prerequisite per row: kept status/mark, first deleted
    // kind, revised mark, original mark, revised close, original close.
    for invalid in 0..7 {
        let left = paragraphs(&mut dom, &["kept", "old", "later"]);
        let right = paragraphs(&mut dom, &["kept", "new", "later"]);
        let mut kept_right = group_contents(&right[0]);
        let mut inserted = group_contents(&right[1]);
        let mut deleted = group_contents(&left[1]);
        if invalid == 1 {
            kept_right.pop();
        }
        if invalid == 3 {
            inserted.pop();
        }
        if invalid == 4 {
            deleted.pop();
        }
        if invalid != 5 {
            // Detach the later revised paragraph so the insertion closes.
            let later = right[2].first_atom().unwrap().ancestor_elements[0];
            dom.remove(later);
        }
        if invalid != 6 {
            let later = left[2].first_atom().unwrap().ancestor_elements[0];
            dom.remove(later);
        }
        let first_deleted = if invalid == 2 {
            vec![group(Table, deleted, "table")]
        } else {
            deleted
        };
        let mut seqs = vec![
            CorrelatedSequence::paired(
                if invalid == 0 { Unknown } else { Equal },
                group_contents(&left[0]),
                kept_right,
            ),
            CorrelatedSequence::inserted(inserted),
            CorrelatedSequence::deleted(first_deleted),
        ];
        let before: Vec<_> = seqs
            .iter()
            .map(|s| {
                (
                    s.correlation_status,
                    geometry(s.com_units_1.as_deref().unwrap_or_default()),
                    geometry(s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect();
        assert!(
            !pair_final_marks_behind_inserted_tail(&dom, &mut seqs),
            "invalid prerequisite {invalid}"
        );
        let after: Vec<_> = seqs
            .iter()
            .map(|s| {
                (
                    s.correlation_status,
                    geometry(s.com_units_1.as_deref().unwrap_or_default()),
                    geometry(s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect();
        assert_eq!(after, before);
    }
}

#[test]
fn final_mark_repair_restores_deleted_first_order_when_pairing_declines() {
    let mut dom = Dom::new();
    let left = inline(&mut dom, &["old", "still later"]);
    let right = inline(&mut dom, &["new"]);
    let mut seqs = vec![
        CorrelatedSequence::deleted(left[..2].to_vec()),
        CorrelatedSequence::inserted(right.clone()),
    ];
    assert!(!pair_final_marks_behind_replaced_tail(&dom, &mut seqs));
    check(
        &dom,
        &seqs,
        &left[..2],
        &right,
        &[(Deleted, "old¶", ""), (Inserted, "", "new¶")],
    );
    let mut inserted_before = vec![
        CorrelatedSequence::inserted(right.clone()),
        CorrelatedSequence::deleted(left.clone()),
        CorrelatedSequence::inserted(right.clone()),
    ];
    assert!(!pair_final_marks_behind_replaced_tail(
        &dom,
        &mut inserted_before
    ));
    check(
        &dom,
        &inserted_before,
        &left,
        &[right.clone(), right].concat(),
        &[
            (Inserted, "", "new¶"),
            (Deleted, "old¶still later¶", ""),
            (Inserted, "", "new¶"),
        ],
    );
}

#[test]
fn inserted_blank_story_tail_pairs_properties_and_removes_empty_sequences() {
    let mut dom = Dom::new();
    let left = inline(&mut dom, &[""]);
    let right = inline(&mut dom, &[""]);
    let revised_mark = right[0].first_atom().unwrap().content_element;
    let revised_spacing = dom.elements(revised_mark, Some(&W::spacing_el()))[0];
    dom.set_attribute_value(revised_spacing, &W::name("before"), Some("240"));
    let mut out = vec![
        CorrelatedSequence::inserted(right.clone()),
        CorrelatedSequence::deleted(left.clone()),
    ];
    assert!(pair_final_marks_behind_inserted_tail(&dom, &mut out));
    check(&dom, &out, &left, &right, &[(Equal, "¶", "¶")]);
    for (u, before) in [(&left[0], "120"), (&right[0], "240")] {
        let mark = u.first_atom().unwrap().content_element;
        let spacing = dom.elements(mark, Some(&W::spacing_el()))[0];
        assert_eq!(dom.attribute(spacing, &W::name("before")), Some(before));
    }
}

#[test]
fn empty_rows_and_cells_keep_their_dispatch_shape_and_one_sided_residuals() {
    let mut dom = Dom::new();
    for kind in [Row, Cell] {
        for (left_tail, right_tail) in [(false, false), (true, false), (false, true), (true, true)]
        {
            let mut left = vec![group(kind, vec![], "empty-left")];
            let mut right = vec![group(kind, vec![], "empty-right")];
            if left_tail {
                left.push(word(&mut dom, "left"));
            }
            if right_tail {
                right.push(word(&mut dom, "right"));
            }
            let out = step_h(&mut dom, &left, &right, &faithful());
            let mut expected = Vec::new();
            if kind == Cell || left_tail || right_tail {
                expected.push((Unknown, "", ""));
            }
            match (left_tail, right_tail) {
                (true, false) => expected.push((Deleted, "left", "")),
                (false, true) => expected.push((Inserted, "", "right")),
                (true, true) => expected.push((Unknown, "left", "right")),
                (false, false) => {}
            }
            check(&dom, &out, &left, &right, &expected);
        }
    }
}

#[test]
fn ordered_paragraph_matching_observes_reach_and_majority_in_both_directions() {
    let sets = |texts: &[&str]| {
        texts
            .iter()
            .map(|t| t.split_whitespace().map(str::to_string).collect())
            .collect::<Vec<std::collections::HashSet<String>>>()
    };
    for (a, b, expected) in [
        (vec!["a", "b", "c"], vec!["a", "b", "c"], false),
        (vec!["a", "b", "c", "d"], vec!["a", "b", "x", "y"], true),
        (vec!["a", "b", "c", "d"], vec!["a", "x", "y", "z"], false),
        (vec!["a", "b", "c", "d"], vec!["d", "c", "b", "a"], false),
        (
            vec!["a b", "c d", "e f", "g h"],
            vec!["a b c", "c d e", "x", "y"],
            true,
        ),
    ] {
        assert_eq!(word_sets_pair_in_order(sets(&a), sets(&b)), expected);
        assert_eq!(word_sets_pair_in_order(sets(&b), sets(&a)), expected);
    }
    let short = ["alpha", "beta", "gamma", "delta"];
    for (padding, expected) in [(7, true), (8, false)] {
        let mut long = vec!["unrelated"; padding];
        long.extend(short);
        assert_eq!(word_sets_pair_in_order(sets(&short), sets(&long)), expected);
        assert_eq!(word_sets_pair_in_order(sets(&long), sets(&short)), expected);
    }
}

#[test]
fn ancestor_pairing_preserves_note_collection_ids_and_stops_at_a_name_mismatch() {
    let mut dom = Dom::new();
    for (left_wrapper, right_wrapper, expected_wrapper) in [
        ("footnotes", "footnotes", "right-wrapper"),
        ("endnotes", "endnotes", "right-wrapper"),
        ("sdtContent", "sdtContent", "left-wrapper"),
        ("sdtContent", "body", "right-wrapper"),
    ] {
        let make = |dom: &mut Dom, wrapper: &str, id: &str| {
            let ancestor = dom.new_element(W::name(wrapper));
            let p = dom.new_element(W::p());
            dom.add(ancestor, p);
            let t = dom.new_element(W::t());
            dom.add(p, t);
            dom.add_text(t, "anchor");
            dom.set_attribute_value(ancestor, &PT::unid(), Some(id));
            dom.set_attribute_value(p, &PT::unid(), Some(id));
            let u = group(
                Paragraph,
                vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                    ComparisonUnitAtom::new(t, vec![ancestor, p], "anchor"),
                ]))],
                id,
            );
            (ancestor, p, u)
        };
        let (a, ap, left) = make(&mut dom, left_wrapper, "left-wrapper");
        let (b, bp, right) = make(&mut dom, right_wrapper, "right-wrapper");
        let before_left = geometry(std::slice::from_ref(&left));
        let before_right = geometry(std::slice::from_ref(&right));
        let input = unknown(vec![left], vec![right]);
        set_after_unids(&mut dom, &input);
        assert_eq!(dom.attribute(bp, &PT::unid()), Some("left-wrapper"));
        assert_eq!(dom.attribute(b, &PT::unid()), Some(expected_wrapper));
        assert_eq!(dom.attribute(a, &PT::unid()), Some("left-wrapper"));
        assert_eq!(dom.attribute(ap, &PT::unid()), Some("left-wrapper"));
        assert_eq!(geometry(input.com_units_1.as_ref().unwrap()), before_left);
        assert_eq!(geometry(input.com_units_2.as_ref().unwrap()), before_right);
    }
}

#[test]
fn structural_final_peel_declines_nonclosing_or_nonparagraph_runs_intact() {
    let mut dom = Dom::new();
    for invalid in 0..5 {
        let left = inline(&mut dom, &["first", "last", "unpaired"]);
        let right = inline(&mut dom, &["revision", "unpaired"]);
        let mut lg = vec![("Word", left[..2].to_vec()), ("Word", left[2..4].to_vec())];
        let mut rg = vec![("Word", right[..2].to_vec())];
        match invalid {
            0 => {} // Both last runs have a following paragraph.
            1 => {
                rg[0].1.pop();
            }
            2 => {
                lg[1].1.pop();
            }
            3 => {
                rg[0].1 = vec![group(Row, vec![], "row")];
            }
            4 => {
                rg[0].1 = vec![empty_word("empty")];
            }
            _ => unreachable!(),
        }
        let before_l: Vec<_> = lg
            .iter()
            .map(|(k, u)| {
                (
                    *k,
                    geometry(u),
                    u.iter()
                        .map(ComparisonUnit::sha1)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let before_r: Vec<_> = rg
            .iter()
            .map(|(k, u)| {
                (
                    *k,
                    geometry(u),
                    u.iter()
                        .map(ComparisonUnit::sha1)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert!(peel_story_final_groups(&dom, &mut lg, &mut rg).is_none());
        let after_l: Vec<_> = lg
            .iter()
            .map(|(k, u)| {
                (
                    *k,
                    geometry(u),
                    u.iter()
                        .map(ComparisonUnit::sha1)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let after_r: Vec<_> = rg
            .iter()
            .map(|(k, u)| {
                (
                    *k,
                    geometry(u),
                    u.iter()
                        .map(ComparisonUnit::sha1)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        assert_eq!(after_l, before_l);
        assert_eq!(after_r, before_r);
    }
}
