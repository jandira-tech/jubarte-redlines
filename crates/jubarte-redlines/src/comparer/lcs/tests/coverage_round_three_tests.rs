// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

fn word(dom: &mut Dom, text: &str) -> ComparisonUnit {
    let leaf = dom.new_element(W::t());
    dom.add_text(leaf, text);
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        leaf,
        Vec::<NodeId>::new(),
        text,
    )]))
}

fn leaf(dom: &mut Dom, local: &str, hash: &str) -> ComparisonUnit {
    let node = dom.new_element(W::name(local));
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        node,
        Vec::<NodeId>::new(),
        hash,
    )]))
}

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

// Every paragraph belongs to a real in-memory body. This matters to the
// wholesale and story-tail gates; detached synthetic marks do not qualify.
fn paragraph(dom: &mut Dom, body: NodeId, tokens: &[&str]) -> ComparisonUnit {
    let p = dom.new_element(W::p());
    dom.add(body, p);
    let ppr = dom.new_element(W::p_pr());
    dom.add(p, ppr);
    let mut contents = Vec::new();
    for token in tokens {
        let run = dom.new_element(W::r());
        dom.add(p, run);
        let node = dom.new_element(W::t());
        dom.add_text(node, token);
        dom.add(run, node);
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(node, vec![p, run], *token),
        ])));
    }
    contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
        ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
    ])));
    group(
        ComparisonUnitGroupType::Paragraph,
        contents,
        &format!("p:{}", tokens.concat()),
    )
}

fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    texts
        .iter()
        .map(|text| {
            if text.is_empty() {
                paragraph(dom, body, &[])
            } else {
                paragraph(dom, body, &[text])
            }
        })
        .collect()
}

fn inline_paragraph(dom: &mut Dom, tokens: &[&str]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    group_contents(&paragraph(dom, body, tokens))
}

fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|atom| {
            if atom_is_ppr(dom, atom) {
                "¶".to_string()
            } else if dom.name_is(atom.content_element, &W::t()) {
                dom.value_str(atom.content_element).into_owned()
            } else {
                format!("<{}>", dom.name(atom.content_element).unwrap().local_name())
            }
        })
        .collect()
}

fn signature(dom: &Dom, seqs: &[CorrelatedSequence]) -> Vec<(CorrelationStatus, String, String)> {
    seqs.iter()
        .map(|s| {
            (
                s.correlation_status,
                text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                text(dom, s.com_units_2.as_deref().unwrap_or_default()),
            )
        })
        .collect()
}

fn expect(dom: &Dom, seqs: &[CorrelatedSequence], expected: &[(CorrelationStatus, &str, &str)]) {
    let expected: Vec<_> = expected
        .iter()
        .map(|&(status, left, right)| (status, left.to_string(), right.to_string()))
        .collect();
    assert_eq!(signature(dom, seqs), expected);
}

fn unknown(left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>) -> CorrelatedSequence {
    CorrelatedSequence::paired(Unknown, left, right)
}

fn assert_sides(dom: &Dom, seqs: &[CorrelatedSequence], left: &str, right: &str) {
    let sig = signature(dom, seqs);
    assert_eq!(sig.iter().map(|s| s.1.as_str()).collect::<String>(), left);
    assert_eq!(sig.iter().map(|s| s.2.as_str()).collect::<String>(), right);
}

#[test]
fn paragraph_replacement_keeps_matching_textless_shells_and_final_mark() {
    for (leading, trailing) in [(true, true), (true, false), (false, true)] {
        let mut dom = Dom::new();
        let mut left = inline_paragraph(&mut dom, &["obsolete vocabulary"]);
        let mut right = inline_paragraph(&mut dom, &["fresh wording"]);
        if leading {
            left.insert(0, leaf(&mut dom, "fldChar", "field-begin"));
            right.insert(0, leaf(&mut dom, "fldChar", "field-begin"));
        }
        if trailing {
            let n = left.len() - 1;
            left.insert(n, leaf(&mut dom, "fldChar", "field-end"));
            let n = right.len() - 1;
            right.insert(n, leaf(&mut dom, "fldChar", "field-end"));
        }
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        let mut expected = Vec::new();
        if leading {
            expected.push((Equal, "<fldChar>", "<fldChar>"));
        }
        expected.extend([
            (Deleted, "obsolete vocabulary", ""),
            (Inserted, "", "fresh wording"),
        ]);
        if trailing {
            expected.push((Equal, "<fldChar>", "<fldChar>"));
        }
        expected.push((Equal, "¶", "¶"));
        expect(&dom, &out, &expected);
    }
}

#[test]
fn paragraph_replacement_does_not_pair_different_shell_keys() {
    for mismatch_at_start in [true, false] {
        let mut dom = Dom::new();
        let mut left = inline_paragraph(&mut dom, &["obsolete"]);
        let mut right = inline_paragraph(&mut dom, &["fresh"]);
        let index = usize::from(!mismatch_at_start);
        left.insert(index, leaf(&mut dom, "fldChar", "first-field"));
        right.insert(index, leaf(&mut dom, "fldChar", "second-field"));
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        if mismatch_at_start {
            expect(
                &dom,
                &out,
                &[
                    (Deleted, "<fldChar>obsolete", ""),
                    (Inserted, "", "<fldChar>fresh"),
                    (Equal, "¶", "¶"),
                ],
            );
        } else {
            expect(
                &dom,
                &out,
                &[
                    (Deleted, "obsolete<fldChar>", ""),
                    (Inserted, "", "fresh<fldChar>"),
                    (Equal, "¶", "¶"),
                ],
            );
        }
    }
}

#[test]
fn paragraph_unique_anchors_resolve_middle_replacement_exactly() {
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &["opening", " ", "obsolete", " ", "ending"]);
    let right = inline_paragraph(&mut dom, &["opening", " ", "fresh", " ", "ending"]);
    let out = resolve_paragraph_window(
        &mut dom,
        unknown(left, right),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    expect(
        &dom,
        &out,
        &[
            (Equal, "opening ", "opening "),
            (Deleted, "obsolete", ""),
            (Inserted, "", "fresh"),
            (Equal, " ending", " ending"),
            (Equal, "¶", "¶"),
        ],
    );
    assert_sides(
        &dom,
        &out,
        "opening obsolete ending¶",
        "opening fresh ending¶",
    );
}

#[test]
fn paragraph_anchors_keep_inserted_and_deleted_edges_separate() {
    for insertion in [true, false] {
        let mut dom = Dom::new();
        let short = inline_paragraph(&mut dom, &["anchor"]);
        let long = inline_paragraph(&mut dom, &["prefix", "anchor", "suffix"]);
        let (left, right) = if insertion {
            (short, long)
        } else {
            (long, short)
        };
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        if insertion {
            expect(
                &dom,
                &out,
                &[
                    (Inserted, "", "prefix"),
                    (Equal, "anchor", "anchor"),
                    (Inserted, "", "suffix"),
                    (Equal, "¶", "¶"),
                ],
            );
        } else {
            expect(
                &dom,
                &out,
                &[
                    (Deleted, "prefix", ""),
                    (Equal, "anchor", "anchor"),
                    (Deleted, "suffix", ""),
                    (Equal, "¶", "¶"),
                ],
            );
        }
    }
}

#[test]
fn paragraph_fragments_are_judged_only_inside_an_existing_word_window() {
    let mut dom = Dom::new();
    let left = vec![word(&mut dom, "obsolete")];
    let right = vec![word(&mut dom, "fresh")];
    let settings = WmlComparerSettings::default();
    let declined =
        resolve_paragraph_window(&mut dom, unknown(left.clone(), right.clone()), &settings)
            .unwrap_err();
    expect(&dom, &[declined], &[(Unknown, "obsolete", "fresh")]);
    let settings = WmlComparerSettings {
        in_word_level_paragraph: true,
        ..settings
    };
    let out = resolve_paragraph_window(&mut dom, unknown(left, right), &settings).unwrap();
    expect(
        &dom,
        &out,
        &[(Deleted, "obsolete", ""), (Inserted, "", "fresh")],
    );
}

#[test]
fn paragraph_declines_groups_interior_marks_and_wordless_content_intact() {
    let mut dom = Dom::new();
    let right = inline_paragraph(&mut dom, &["real words"]);
    let group_left = paragraphs(&mut dom, &["group words"]);
    let mut interior = inline_paragraph(&mut dom, &["first"]);
    interior.extend(inline_paragraph(&mut dom, &["second"]));
    let punctuation = inline_paragraph(&mut dom, &[";", " "]);
    let empty = inline_paragraph(&mut dom, &[]);
    for (left, expected) in [
        (group_left, "group words¶"),
        (interior, "first¶second¶"),
        (punctuation, "; ¶"),
        (empty, "¶"),
    ] {
        let declined = resolve_paragraph_window(
            &mut dom,
            unknown(left, right.clone()),
            &WmlComparerSettings::default(),
        )
        .unwrap_err();
        expect(&dom, &[declined], &[(Unknown, expected, "real words¶")]);
    }
    let left = inline_paragraph(&mut dom, &["real words"]);
    let right = inline_paragraph(&mut dom, &[";"]);
    let declined = resolve_paragraph_window(
        &mut dom,
        unknown(left, right),
        &WmlComparerSettings::default(),
    )
    .unwrap_err();
    expect(&dom, &[declined], &[(Unknown, "real words¶", ";¶")]);
}

#[test]
fn paragraph_cell_cap_declines_before_quadratic_allocation() {
    let mut dom = Dom::new();
    let token = word(&mut dom, "bounded");
    // 10,001 squared is the first square above the 100,000,000-cell cap.
    let side = vec![token; 10_001];
    assert!(side.len().saturating_mul(side.len()) > PARAGRAPH_WINDOW_CELL_CAP);
    let declined = resolve_paragraph_window(
        &mut dom,
        unknown(side.clone(), side),
        &WmlComparerSettings::default(),
    )
    .unwrap_err();
    assert_eq!(declined.correlation_status, Unknown);
    for units in [declined.com_units_1.unwrap(), declined.com_units_2.unwrap()] {
        assert_eq!(units.len(), 10_001);
        assert_eq!(text(&dom, &units), "bounded".repeat(10_001));
    }
}

#[test]
fn faithful_paragraph_resolver_preserves_the_unknown_window() {
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &["same", " ", "old"]);
    let right = inline_paragraph(&mut dom, &["same", " ", "new"]);
    let declined = resolve_paragraph_window(
        &mut dom,
        unknown(left, right),
        &WmlComparerSettings::powertools_faithful(),
    )
    .unwrap_err();
    expect(&dom, &[declined], &[(Unknown, "same old¶", "same new¶")]);
}

#[test]
fn lcs_wholesale_carriers_observe_direction_and_paragraph_counts() {
    for (left_texts, right_texts, expected) in [
        (
            vec!["alpha", "beta"],
            vec!["omega"],
            vec![
                (Inserted, "", "omega"),
                (Deleted, "alpha", ""),
                (Deleted, "¶", ""),
                (Deleted, "beta¶", ""),
            ],
        ),
        (
            vec!["alpha"],
            vec!["omega", "sigma"],
            vec![
                (Inserted, "", "omega¶"),
                (Inserted, "", "sigma"),
                (Deleted, "alpha", ""),
                (Equal, "¶", "¶"),
            ],
        ),
        (
            vec!["alpha", "beta"],
            vec!["omega", "sigma", "tau"],
            vec![
                (Inserted, "", "omega¶sigma¶"),
                (Inserted, "", "tau"),
                (Deleted, "alpha", ""),
                (Deleted, "¶", ""),
                (Deleted, "beta¶", ""),
            ],
        ),
    ] {
        let mut dom = Dom::new();
        let left: Vec<_> = paragraphs(&mut dom, &left_texts)
            .iter()
            .flat_map(group_contents)
            .collect();
        let right: Vec<_> = paragraphs(&mut dom, &right_texts)
            .iter()
            .flat_map(group_contents)
            .collect();
        let out = do_lcs_algorithm(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        );
        expect(&dom, &out, &expected);
    }
}

#[test]
fn lcs_carrier_accepts_trailing_empty_paragraphs_and_section_properties() {
    let mut dom = Dom::new();
    let body1 = dom.new_element(W::body());
    let body2 = dom.new_element(W::body());
    let a = paragraph(&mut dom, body1, &["alpha"]);
    let b = paragraph(&mut dom, body1, &["beta"]);
    let c = paragraph(&mut dom, body2, &["omega"]);
    paragraph(&mut dom, body1, &[]);
    paragraph(&mut dom, body2, &[" "]);
    for body in [body1, body2] {
        let section = dom.new_element(W::sect_pr());
        dom.add(body, section);
    }
    let left = [group_contents(&a), group_contents(&b)].concat();
    let out = do_lcs_algorithm(
        &mut dom,
        unknown(left, group_contents(&c)),
        &WmlComparerSettings::default(),
    );
    expect(
        &dom,
        &out,
        &[
            (Inserted, "", "omega"),
            (Deleted, "alpha", ""),
            (Deleted, "¶", ""),
            (Deleted, "beta¶", ""),
        ],
    );
}

#[test]
fn lcs_wholesale_empty_revised_tail_keeps_only_story_final_marks() {
    let mut dom = Dom::new();
    let left: Vec<_> = paragraphs(&mut dom, &["alpha", "beta", "gamma"])
        .iter()
        .flat_map(group_contents)
        .collect();
    let right: Vec<_> = paragraphs(&mut dom, &["omega", ""])
        .iter()
        .flat_map(group_contents)
        .collect();
    let out = do_lcs_algorithm(
        &mut dom,
        unknown(left, right),
        &WmlComparerSettings::default(),
    );
    expect(
        &dom,
        &out,
        &[
            (Inserted, "", "omega¶"),
            (Deleted, "alpha¶beta¶gamma", ""),
            (Equal, "¶", "¶"),
        ],
    );
}

#[test]
fn lcs_common_inline_run_splits_unknown_prefix_and_suffix() {
    let mut dom = Dom::new();
    let left: Vec<_> = ["old", "anchor", "tail"]
        .iter()
        .map(|s| word(&mut dom, s))
        .collect();
    let right: Vec<_> = ["new", "anchor", "end"]
        .iter()
        .map(|s| word(&mut dom, s))
        .collect();
    let out = do_lcs_algorithm(
        &mut dom,
        unknown(left, right),
        &WmlComparerSettings::powertools_faithful(),
    );
    expect(
        &dom,
        &out,
        &[
            (Unknown, "old", "new"),
            (Equal, "anchor", "anchor"),
            (Unknown, "tail", "end"),
            (Unknown, "", ""),
        ],
    );
    assert_sides(&dom, &out, "oldanchortail", "newanchorend");
}

#[test]
fn lcs_word_threshold_keeps_the_exact_boundary_and_voids_below_it() {
    for (threshold, survives) in [(0.5, true), (0.500_001, false)] {
        let mut dom = Dom::new();
        let left = vec![word(&mut dom, "anchor"), word(&mut dom, "old")];
        let right = vec![word(&mut dom, "anchor"), word(&mut dom, "new")];
        let settings = WmlComparerSettings {
            detail_threshold: threshold,
            ..WmlComparerSettings::powertools_faithful()
        };
        let out = do_lcs_algorithm(&mut dom, unknown(left, right), &settings);
        if survives {
            expect(
                &dom,
                &out,
                &[
                    (Equal, "anchor", "anchor"),
                    (Unknown, "old", "new"),
                    (Unknown, "", ""),
                ],
            );
        } else {
            expect(
                &dom,
                &out,
                &[(Deleted, "anchorold", ""), (Inserted, "", "anchornew")],
            );
        }
    }
}

#[test]
fn lcs_separator_only_short_runs_cannot_anchor_replacement() {
    for separator in [" ", ";", "中"] {
        let mut dom = Dom::new();
        let left = vec![word(&mut dom, "old"), word(&mut dom, separator)];
        let right = vec![word(&mut dom, "new"), word(&mut dom, separator)];
        let out = do_lcs_algorithm(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::powertools_faithful(),
        );
        expect(
            &dom,
            &out,
            &[
                (Deleted, &format!("old{separator}"), ""),
                (Inserted, "", &format!("new{separator}")),
            ],
        );
    }
}

#[test]
fn step_h_mixed_word_and_row_dispatch_preserves_directional_order() {
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let words = vec![word(&mut dom, "prose")];
        let cell = word(&mut dom, "cell");
        let rows = vec![group(ComparisonUnitGroupType::Row, vec![cell], "row")];
        let (left, right) = if reverse {
            (rows, words)
        } else {
            (words, rows)
        };
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        if reverse {
            expect(
                &dom,
                &out,
                &[(Deleted, "cell", ""), (Inserted, "", "prose")],
            );
        } else {
            expect(
                &dom,
                &out,
                &[(Inserted, "", "cell"), (Deleted, "prose", "")],
            );
        }
    }
}

#[test]
fn step_h_word_textbox_mismatch_advances_and_flushes_the_other_side() {
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let words = vec![word(&mut dom, "prose")];
        let cell = word(&mut dom, "textbox");
        let boxes = vec![group(
            ComparisonUnitGroupType::Textbox,
            vec![cell],
            "textbox",
        )];
        let (left, right) = if reverse {
            (boxes, words)
        } else {
            (words, boxes)
        };
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        if reverse {
            expect(
                &dom,
                &out,
                &[(Inserted, "", "prose"), (Deleted, "textbox", "")],
            );
        } else {
            expect(
                &dom,
                &out,
                &[(Deleted, "prose", ""), (Inserted, "", "textbox")],
            );
        }
    }
}

#[test]
fn step_h_row_textbox_mismatch_flushes_without_losing_prior_word_pair() {
    let mut dom = Dom::new();
    let lw = word(&mut dom, "left");
    let rw = word(&mut dom, "right");
    let row_text = word(&mut dom, "row");
    let box_text = word(&mut dom, "box");
    let left = vec![
        lw,
        group(ComparisonUnitGroupType::Row, vec![row_text], "row"),
    ];
    let right = vec![
        rw,
        group(ComparisonUnitGroupType::Textbox, vec![box_text], "box"),
    ];
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    expect(
        &dom,
        &out,
        &[
            (Unknown, "left", "right"),
            (Deleted, "row", ""),
            (Inserted, "", "box"),
        ],
    );
    assert_sides(&dom, &out, "leftrow", "rightbox");
}

#[test]
fn step_h_table_titles_replace_then_pair_shared_empty_paragraphs() {
    for (left_blanks, right_blanks) in [(0, 0), (1, 1), (2, 1), (1, 2)] {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["alpha"]);
        let mut right = paragraphs(&mut dom, &["omega"]);
        for _ in 0..left_blanks {
            left.extend(paragraphs(&mut dom, &[""]));
        }
        for _ in 0..right_blanks {
            right.extend(paragraphs(&mut dom, &[""]));
        }
        let lc = word(&mut dom, "left-table");
        let rc = word(&mut dom, "right-table");
        left.push(group(
            ComparisonUnitGroupType::Table,
            vec![lc],
            "left-table",
        ));
        right.push(group(
            ComparisonUnitGroupType::Table,
            vec![rc],
            "right-table",
        ));
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        let mut expected = if left_blanks == right_blanks {
            vec![(Inserted, "", "omega¶"), (Deleted, "alpha¶", "")]
        } else {
            vec![(Unknown, "alpha¶", "omega¶")]
        };
        if left_blanks.min(right_blanks) > 0 {
            expected.push((Unknown, "¶", "¶"));
        }
        if left_blanks > right_blanks {
            expected.push((Deleted, "¶", ""));
        }
        if right_blanks > left_blanks {
            expected.push((Inserted, "", "¶"));
        }
        expected.push((Unknown, "left-table", "right-table"));
        expect(&dom, &out, &expected);
        assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
    }
}

#[test]
fn step_h_table_paragraph_mismatches_advance_both_directions() {
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let la = word(&mut dom, "left-table");
        let rb = word(&mut dom, "right-table");
        let mut left = paragraphs(&mut dom, &["left-title"]);
        left.push(group(
            ComparisonUnitGroupType::Table,
            vec![la],
            "left-table",
        ));
        let mut right = vec![group(
            ComparisonUnitGroupType::Table,
            vec![rb],
            "right-table",
        )];
        right.extend(paragraphs(&mut dom, &["right-title"]));
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        let out = step_h(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::powertools_faithful(),
        );
        if reverse {
            expect(
                &dom,
                &out,
                &[
                    (Inserted, "", "left-title¶"),
                    (Unknown, "right-table", "left-table"),
                    (Deleted, "right-title¶", ""),
                ],
            );
        } else {
            expect(
                &dom,
                &out,
                &[
                    (Deleted, "left-title¶", ""),
                    (Unknown, "left-table", "right-table"),
                    (Inserted, "", "right-title¶"),
                ],
            );
        }
    }
}

#[test]
fn step_h_row_cells_pair_positionally_and_emit_extra_cells_on_their_side() {
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let a = word(&mut dom, "alpha");
        let b = word(&mut dom, "omega");
        let extra = word(&mut dom, "extra");
        let left = vec![group(
            ComparisonUnitGroupType::Row,
            vec![group(ComparisonUnitGroupType::Cell, vec![a], "cell-a")],
            "row-a",
        )];
        let right = vec![group(
            ComparisonUnitGroupType::Row,
            vec![
                group(ComparisonUnitGroupType::Cell, vec![b], "cell-b"),
                group(ComparisonUnitGroupType::Cell, vec![extra], "cell-extra"),
            ],
            "row-b",
        )];
        let (left, right) = if reverse {
            (right, left)
        } else {
            (left, right)
        };
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        if reverse {
            expect(
                &dom,
                &out,
                &[(Unknown, "omega", "alpha"), (Deleted, "extra", "")],
            );
        } else {
            expect(
                &dom,
                &out,
                &[(Unknown, "alpha", "omega"), (Inserted, "", "extra")],
            );
        }
        assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
    }
}

#[test]
fn step_h_cells_flatten_one_level_and_keep_remaining_cells_unknown() {
    let mut dom = Dom::new();
    let a = word(&mut dom, "alpha");
    let b = word(&mut dom, "omega");
    let c = word(&mut dom, "tail-a");
    let d = word(&mut dom, "tail-b");
    let left = vec![
        group(ComparisonUnitGroupType::Cell, vec![a], "a"),
        group(ComparisonUnitGroupType::Cell, vec![c], "c"),
    ];
    let right = vec![
        group(ComparisonUnitGroupType::Cell, vec![b], "b"),
        group(ComparisonUnitGroupType::Cell, vec![d], "d"),
    ];
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    expect(
        &dom,
        &out,
        &[(Unknown, "alpha", "omega"), (Unknown, "tail-a", "tail-b")],
    );
    assert!(matches!(
        out[0].com_units_1.as_ref().unwrap()[0],
        ComparisonUnit::Word(_)
    ));
    assert!(matches!(
        out[1].com_units_1.as_ref().unwrap()[0],
        ComparisonUnit::Group(_)
    ));
}

#[test]
fn unrelated_three_paragraph_demo_replaces_two_paragraph_plain_document() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["Scarlet Demo", "crimson", "vermilion"]);
    let right = paragraphs(&mut dom, &["Azure Catalog", "cobalt"]);
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    expect(
        &dom,
        &out,
        &[
            (Inserted, "", "Azure Catalog¶cobalt¶"),
            (Deleted, "Scarlet Demo¶crimson¶vermilion¶", ""),
        ],
    );
    assert_sides(
        &dom,
        &out,
        "Scarlet Demo¶crimson¶vermilion¶",
        "Azure Catalog¶cobalt¶",
    );
}

#[test]
fn unrelated_equal_counts_ignore_shared_empty_paragraph_hashes() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["alpha", "beta", "gamma", "delta", ""]);
    let right = paragraphs(&mut dom, &["omega", "sigma", "tau", "zeta", ""]);
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    expect(
        &dom,
        &out,
        &[
            (Inserted, "", "omega¶sigma¶tau¶zeta¶¶"),
            (Deleted, "alpha¶beta¶gamma¶delta¶¶", ""),
        ],
    );
}

#[test]
fn unrelated_short_count_empty_tails_replace_without_junction_fusion() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["alpha", ""]);
    let right = paragraphs(&mut dom, &["omega", ""]);
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    expect(
        &dom,
        &out,
        &[(Inserted, "", "omega¶¶"), (Deleted, "alpha¶¶", "")],
    );
}

#[test]
fn stamp_confetti_handles_empty_residuals_and_one_sided_residuals() {
    for (left_texts, right_texts, expected) in [
        (
            vec!["file_1.docx"],
            vec!["file_1.docx"],
            vec![(Equal, "file_1.docx¶", "file_1.docx¶")],
        ),
        (
            vec!["file_1.docx", "alpha"],
            vec!["file_1.docx"],
            vec![
                (Equal, "file_1.docx¶", "file_1.docx¶"),
                (Deleted, "alpha¶", ""),
            ],
        ),
        (
            vec!["file_1.docx"],
            vec!["file_1.docx", "omega"],
            vec![
                (Equal, "file_1.docx¶", "file_1.docx¶"),
                (Inserted, "", "omega¶"),
            ],
        ),
    ] {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &left_texts);
        let right = paragraphs(&mut dom, &right_texts);
        let out =
            stamp_confetti_then_replace(&mut dom, &left, &right, &WmlComparerSettings::default())
                .unwrap();
        expect(&dom, &out, &expected);
        assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
    }
}

#[test]
fn stamp_confetti_unrelated_single_residual_is_inserted_then_deleted() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["file_1.docx", "alpha"]);
    let right = paragraphs(&mut dom, &["file_1.docx", "omega"]);
    let out = stamp_confetti_then_replace(&mut dom, &left, &right, &WmlComparerSettings::default())
        .unwrap();
    expect(
        &dom,
        &out,
        &[
            (Equal, "file_1.docx¶", "file_1.docx¶"),
            (Inserted, "", "omega¶"),
            (Deleted, "alpha¶", ""),
        ],
    );
}

#[test]
fn stamp_confetti_related_diagonal_preserves_both_residual_paragraphs() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["file_1.docx", "shared heading", "shared body"]);
    let right = paragraphs(&mut dom, &["file_1.docx", "shared heading", "shared body"]);
    let out = stamp_confetti_then_replace(&mut dom, &left, &right, &WmlComparerSettings::default())
        .unwrap();
    expect(
        &dom,
        &out,
        &[
            (Equal, "file_1.docx¶", "file_1.docx¶"),
            (Equal, "shared heading¶", "shared heading¶"),
            (Equal, "shared body¶", "shared body¶"),
        ],
    );
    assert_sides(
        &dom,
        &out,
        "file_1.docx¶shared heading¶shared body¶",
        "file_1.docx¶shared heading¶shared body¶",
    );
}

#[test]
fn stamp_confetti_declines_missing_content_without_mutating_other_side() {
    let mut dom = Dom::new();
    let empty = paragraphs(&mut dom, &[""]);
    let real = paragraphs(&mut dom, &["file_1.docx", "alpha"]);
    let node = real[0].first_atom().unwrap().content_element;
    dom.set_attribute_value(node, &PT::unid(), Some("preserved"));
    assert!(
        stamp_confetti_then_replace(&mut dom, &empty, &real, &WmlComparerSettings::default())
            .is_none()
    );
    assert!(
        stamp_confetti_then_replace(&mut dom, &real, &empty, &WmlComparerSettings::default())
            .is_none()
    );
    assert_eq!(text(&dom, &real), "file_1.docx¶alpha¶");
    assert_eq!(dom.attribute(node, &PT::unid()), Some("preserved"));
}
