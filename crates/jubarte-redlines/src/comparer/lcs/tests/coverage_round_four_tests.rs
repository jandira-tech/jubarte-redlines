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

// Besides exact markup, pin the source geometry: every emitted atom must
// retain its own element and full paragraph/run ancestor chain, in order.
// This catches a text-equal but incorrectly aligned repeated word or mark.
fn geometry(seqs: &[CorrelatedSequence], left: bool) -> Vec<(NodeId, Vec<NodeId>)> {
    seqs.iter()
        .flat_map(|s| {
            let units = if left { &s.com_units_1 } else { &s.com_units_2 };
            units
                .as_deref()
                .unwrap_or_default()
                .iter()
                .flat_map(ComparisonUnit::descendant_atoms)
                .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
                .collect::<Vec<_>>()
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
    expect(dom, out, expected);
    for (is_left, source) in [(true, left), (false, right)] {
        let original: Vec<_> = source
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
            .collect();
        assert_eq!(geometry(out, is_left), original);
    }
}

fn table(dom: &mut Dom, text: &str) -> ComparisonUnit {
    let tbl = dom.new_element(W::tbl());
    let tr = dom.new_element(W::tr());
    let tc = dom.new_element(W::tc());
    dom.add(tbl, tr);
    dom.add(tr, tc);
    let p = paragraph(dom, tc, &[text]);
    let mut p = group_contents(&p);
    for u in &mut p {
        if let ComparisonUnit::Word(w) = u {
            for a in &mut w.contents {
                let mut ancestors = vec![tbl, tr, tc];
                ancestors.extend(a.ancestor_elements.iter().copied());
                a.ancestor_elements = ancestors.into();
            }
        }
    }
    group(ComparisonUnitGroupType::Table, p, &format!("table:{text}"))
}

fn numbered(dom: &mut Dom, labels: &[(&str, u32)]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    labels
        .iter()
        .map(|&(label, level)| {
            let u = paragraph(dom, body, &[label]);
            let mark = u.descendant_atoms().last().unwrap().content_element;
            let num = dom.new_element(W::num_pr());
            let ilvl = dom.new_element(W::name("ilvl"));
            dom.set_attribute_value(ilvl, &W::val(), Some(&level.to_string()));
            dom.add(num, ilvl);
            dom.add(mark, num);
            u
        })
        .collect()
}

fn wholesale(dom: &mut Dom, left: &[ComparisonUnit], right: &[ComparisonUnit]) {
    let out =
        detect_unrelated_sources_word_mode_inner(dom, left, right, &WmlComparerSettings::default())
            .unwrap();
    let a = text(dom, left);
    let b = text(dom, right);
    check(
        dom,
        &out,
        left,
        right,
        &[(Inserted, "", &b), (Deleted, &a, "")],
    );
}

#[test]
fn weighted_move_keeps_the_long_word_and_revises_the_crossing_short_word() {
    // An in-order alignment cannot retain both crossing words. Character
    // weight must prefer the long anchor over the first/shorter match.
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &["brief", " ", "substantialanchor"]);
    let right = inline_paragraph(&mut dom, &["substantialanchor", " ", "brief"]);
    let out = resolve_paragraph_window(
        &mut dom,
        unknown(left.clone(), right.clone()),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Deleted, "brief ", ""),
            (Equal, "substantialanchor", "substantialanchor"),
            (Inserted, "", " brief"),
            (Equal, "¶", "¶"),
        ],
    );
    assert_eq!(
        out[1].com_units_1.as_ref().unwrap()[0]
            .first_atom()
            .unwrap()
            .content_element,
        left[2].first_atom().unwrap().content_element
    );
    assert_eq!(
        out[1].com_units_2.as_ref().unwrap()[0]
            .first_atom()
            .unwrap()
            .content_element,
        right[0].first_atom().unwrap().content_element
    );
}

#[test]
fn weighted_crossing_runs_keep_a_heavier_chain_in_both_directions() {
    // Three short matches must lose to one heavy match; reversing the
    // documents must preserve the same content anchor and shifted slots.
    assert_eq!(
        heckel_links(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
        vec![(3, 0)]
    );
    assert_eq!(
        weighted_lcs_pairs(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
        vec![(3, 0)]
    );
    assert_eq!(
        heckel_links(&[4, 1, 2, 3], &[1, 2, 3, 4], &[20, 2, 2, 2]),
        vec![(0, 3)]
    );
    assert_eq!(
        weighted_lcs_pairs(&[4, 1, 2, 3], &[1, 2, 3, 4], &[20, 2, 2, 2]),
        vec![(0, 3)]
    );
    assert_eq!(
        weighted_lcs_row(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
        vec![0, 20, 20, 20, 20]
    );
}

#[test]
fn duplicate_words_extend_from_a_unique_anchor_on_both_sides() {
    // Only key 9 is unique. Both backward and forward extension must
    // recover the repeated neighbours without matching zero separators.
    let a = [2, 2, 9, 3, 3, 0, 4];
    let b = [0, 2, 2, 9, 3, 3, 0];
    assert_eq!(
        heckel_links(&a, &b, &[2, 2, 9, 3, 3, 0, 4]),
        vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
    );
    assert_eq!(
        weighted_lcs_pairs(&a, &b, &[2, 2, 9, 3, 3, 0, 4]),
        vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
    );
}

#[test]
fn paragraph_gap_replaces_repeated_connectors_below_its_own_kept_ratio() {
    // Long outer anchors make the whole paragraph related, but the
    // middle rewrite shares only two repeated connectors. It must be
    // judged as its own gap rather than inherit the outer kept ratio.
    let mut dom = Dom::new();
    let left = inline_paragraph(
        &mut dom,
        &[
            "openingstableanchor",
            " ",
            "obsoletevocabularylong",
            "and",
            "discardedmateriallong",
            "and",
            "retiredphrasinglong",
            " ",
            "closingstableanchor",
        ],
    );
    let right = inline_paragraph(
        &mut dom,
        &[
            "openingstableanchor",
            " ",
            "replacementvocabularylong",
            "and",
            "brandnewmateriallong",
            "and",
            "freshphrasinglong",
            " ",
            "closingstableanchor",
        ],
    );
    let out = resolve_paragraph_window(
        &mut dom,
        unknown(left.clone(), right.clone()),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Equal, "openingstableanchor ", "openingstableanchor "),
            (
                Deleted,
                "obsoletevocabularylonganddiscardedmateriallongandretiredphrasinglong",
                "",
            ),
            (
                Inserted,
                "",
                "replacementvocabularylongandbrandnewmateriallongandfreshphrasinglong",
            ),
            (Equal, " closingstableanchor", " closingstableanchor"),
            (Equal, "¶", "¶"),
        ],
    );
}

#[test]
fn repeated_paragraph_without_unique_links_resolves_equal_copy_for_copy() {
    // No word or separator is unique, so the recursive fallback must
    // terminate and keep each copy aligned with its corresponding copy.
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &["echo", " ", "echo", " ", "echo"]);
    let right = inline_paragraph(&mut dom, &["echo", " ", "echo", " ", "echo"]);
    let out = resolve_paragraph_window(
        &mut dom,
        unknown(left.clone(), right.clone()),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[(Equal, "echo echo echo¶", "echo echo echo¶")],
    );
}

#[test]
fn kept_span_counts_boundary_blanks_only_when_both_sides_have_them() {
    // Blank geometry contributes characters, never independent anchors;
    // an asymmetric gap breaks a run and must not absorb intervening text.
    let weights = [0, 4, 0, 6, 0];
    let chars = [1, 4, 2, 6, 3];
    let blanks = [true, false, true, false, true];
    assert_eq!(
        kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blanks, &blanks),
        16
    );
    assert_eq!(
        kept_span(
            &[(1, 0), (3, 2)],
            &weights,
            &chars,
            &blanks,
            &[false, true, false]
        ),
        12
    );
    assert_eq!(
        kept_span(
            &[(1, 1), (3, 4)],
            &weights,
            &chars,
            &blanks,
            &[true, false, true, false, false]
        ),
        13
    );
}

#[test]
fn unmarked_seam_never_borrows_the_revised_paragraph_mark() {
    // Revised fragments without a mark cannot turn the original mark
    // into Equal, including an empty original carrier at the seam.
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &["old"]);
    let right = vec![word(&mut dom, "new")];
    let mut out = Vec::new();
    seam_region(&dom, &left, &right, &mut out);
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "new"),
            (Deleted, "old", ""),
            (Deleted, "¶", ""),
        ],
    );
    let left = inline_paragraph(&mut dom, &[]);
    let mut out = Vec::new();
    seam_region(&dom, &left, &right, &mut out);
    check(
        &dom,
        &out,
        &left,
        &right,
        &[(Inserted, "", "new"), (Deleted, "¶", "")],
    );
}

#[test]
fn empty_revised_carrier_emits_no_empty_insertion() {
    // A blank final revised paragraph contributes its mark, not an
    // empty text sequence; the preceding revised paragraph stays inserted.
    let mut dom = Dom::new();
    let left = inline_paragraph(&mut dom, &[]);
    let right: Vec<_> = paragraphs(&mut dom, &["head", ""])
        .iter()
        .flat_map(group_contents)
        .collect();
    let mut out = Vec::new();
    seam_region(&dom, &left, &right, &mut out);
    check(
        &dom,
        &out,
        &left,
        &right,
        &[(Inserted, "", "head¶"), (Equal, "¶", "¶")],
    );
}

#[test]
fn junction_empty_original_carrier_pairs_only_the_two_final_marks() {
    // The original can be a blank carrier: no empty Deleted sequence
    // should be generated, while the revised preceding paragraph survives.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &[""]);
    let right = paragraphs(&mut dom, &["head", "tail"]);
    let out = junction_seam(&dom, &left, &right, 1, 2).unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "head¶"),
            (Inserted, "", "tail"),
            (Equal, "¶", "¶"),
        ],
    );
}

#[test]
fn junction_requires_real_text_and_both_carrier_marks() {
    // A whitespace carrier or missing pilcrow is not a complete
    // replacement seam; those windows must remain for the other resolvers.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["alpha"]);
    let blank = paragraphs(&mut dom, &["omega", " \t"]);
    assert!(junction_seam(&dom, &left, &blank, 1, 2).is_none());
    let mut right = paragraphs(&mut dom, &["omega", "sigma"]);
    if let ComparisonUnit::Group(g) = &mut right[1] {
        g.contents.pop();
    }
    assert!(junction_seam(&dom, &left, &right, 1, 2).is_none());
    let right = paragraphs(&mut dom, &["omega", "sigma"]);
    let mut unmarked = left.clone();
    if let ComparisonUnit::Group(g) = &mut unmarked[0] {
        g.contents.pop();
    }
    assert!(junction_seam(&dom, &unmarked, &right, 1, 2).is_none());
    assert_eq!(text(&dom, &left), "alpha¶");
}

#[test]
fn junction_keeps_original_tail_in_its_original_ancestors() {
    // Multiple original paragraphs delete the first carrier's mark
    // separately; the following original paragraph remains a block.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["alpha", "beta"]);
    let right = paragraphs(&mut dom, &["omega"]);
    let out = junction_seam(&dom, &left, &right, 2, 1).unwrap();
    // The revised carrier mark is consumed by Word's seam, rather than
    // emitted as an independent insertion. Pin that intentional geometry.
    let mut right_words = group_contents(&right[0]);
    let consumed_mark = right_words.pop().unwrap();
    assert!(unit_is_single_atom_ppr(&dom, &consumed_mark));
    assert!(
        !geometry(&out, false)
            .iter()
            .any(|(node, _)| *node == consumed_mark.first_atom().unwrap().content_element)
    );
    check(
        &dom,
        &out,
        &left,
        &right_words,
        &[
            (Inserted, "", "omega"),
            (Deleted, "alpha", ""),
            (Deleted, "¶", ""),
            (Deleted, "beta¶", ""),
        ],
    );
}

#[test]
fn nested_list_cut_preserves_the_four_way_revision_seam() {
    // A top-level item, nested item, blank, then top-level item form a
    // genuine cluster cut. The next list's first item precedes that cut.
    let mut dom = Dom::new();
    let left = numbered(
        &mut dom,
        &[
            ("alpha", 0),
            ("beta", 1),
            ("", 1),
            ("gamma", 0),
            ("delta", 0),
            ("epsilon", 0),
        ],
    );
    let right = numbered(
        &mut dom,
        &[("omega", 0), ("sigma", 0), ("tau", 0), ("upsilon", 0)],
    );
    assert_eq!(first_list_cluster_end(&dom, &left), 3);
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "omega¶"),
            (Deleted, "alpha¶beta¶¶", ""),
            (Inserted, "", "sigma¶tau¶upsilon¶"),
            (Deleted, "gamma¶delta¶epsilon¶", ""),
        ],
    );
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "omega¶"),
            (Deleted, "alpha¶beta¶¶", ""),
            (Inserted, "", "sigma¶tau¶upsilon¶"),
            (Deleted, "gamma¶delta¶epsilon¶", ""),
        ],
    );
}

#[test]
fn uniform_next_list_bypasses_nested_cut_and_replaces_whole() {
    // M428: repeated single-word next items are a wholesale list
    // replacement, despite a valid nested cut in the original list.
    let mut dom = Dom::new();
    let left = numbered(
        &mut dom,
        &[
            ("alpha", 0),
            ("beta", 1),
            ("gamma", 0),
            ("delta", 0),
            ("epsilon", 0),
        ],
    );
    let right = numbered(
        &mut dom,
        &[("test", 0), ("test", 0), ("test", 0), ("test", 0)],
    );
    wholesale(&mut dom, &left, &right);
}

#[test]
fn flat_short_list_replacement_emits_whole_paragraphs_in_both_directions() {
    // No nested cut exists. Unequal short list counts must retain
    // each list item's pilcrow instead of making a prose carrier seam.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let mut left = numbered(&mut dom, &[("alpha", 0), ("beta", 0)]);
        let mut right = numbered(&mut dom, &[("omega", 0), ("sigma", 0), ("tau", 0)]);
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        let mut expected = Vec::new();
        let a: Vec<_> = left
            .iter()
            .map(|u| text(&dom, std::slice::from_ref(u)))
            .collect();
        let b: Vec<_> = right
            .iter()
            .map(|u| text(&dom, std::slice::from_ref(u)))
            .collect();
        for s in &b {
            expected.push((Inserted, "", s.as_str()));
        }
        for s in &a {
            expected.push((Deleted, s.as_str(), ""));
        }
        check(&dom, &out, &left, &right, &expected);
        wholesale(&mut dom, &left, &right);
    }
}

#[test]
fn equal_title_prose_table_seam_peels_title_and_replaces_only_residual() {
    // M207: matching titles do not license free matching prose against
    // cells. The title remains paired and the residual retains I-before-D.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["Stable heading", "prose body"]);
        let mut right = paragraphs(&mut dom, &["Stable heading", ""]);
        right.push(table(&mut dom, "cell labels"));
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        let expected = if reverse {
            vec![
                (Unknown, "Stable heading¶", "Stable heading¶"),
                (Inserted, "", "prose body¶"),
                (Deleted, "¶", ""),
                (Deleted, "cell labels¶", ""),
            ]
        } else {
            vec![
                (Unknown, "Stable heading¶", "Stable heading¶"),
                (Inserted, "", "¶"),
                (Inserted, "", "cell labels¶"),
                (Deleted, "prose body¶", ""),
            ]
        };
        check(&dom, &out, &left, &right, &expected);
    }
}

#[test]
fn table_title_carrier_pairs_last_prose_and_deletes_empty_table_prefix() {
    // M208: unrelated table-left versus at least four prose paragraphs
    // inserts early prose, meshes only its last paragraph with the title.
    let mut dom = Dom::new();
    let mut left = paragraphs(&mut dom, &["catalog", ""]);
    left.push(table(&mut dom, "entries"));
    let right = paragraphs(&mut dom, &["omega", "sigma", "tau", "upsilon"]);
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "omega¶"),
            (Inserted, "", "sigma¶"),
            (Inserted, "", "tau¶"),
            (Unknown, "catalog¶", "upsilon¶"),
            (Deleted, "¶", ""),
            (Deleted, "entries¶", ""),
        ],
    );
}

#[test]
fn shared_project_title_token_pairs_titles_and_replaces_disjoint_bodies() {
    // M168: Project Plan vs Project Proposal shares its first title
    // token, not the final token; unequal body counts still stay separate.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["Project Plan", "alpha", "beta"]);
    let right = paragraphs(&mut dom, &["Project Proposal", "omega", "sigma", "tau"]);
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Unknown, "Project Plan¶", "Project Proposal¶"),
            (Inserted, "", "omega¶"),
            (Inserted, "", "sigma¶"),
            (Inserted, "", "tau¶"),
            (Deleted, "alpha¶", ""),
            (Deleted, "beta¶", ""),
        ],
    );
}

#[test]
fn this_text_document_residual_realignment_keeps_the_crossed_carrier() {
    // M151 on an already peeled 2x2 residual: revised first body is
    // inserted, original first meshes with revised second, old last deletes.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["This text describes alpha", "legacy ending"]);
    let right = paragraphs(&mut dom, &["This document describes omega", "fresh ending"]);
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "This document describes omega¶"),
            (Unknown, "This text describes alpha¶", "fresh ending¶"),
            (Deleted, "legacy ending¶", ""),
        ],
    );
}

#[test]
fn demo_document_title_residual_remains_a_wholesale_short_title() {
    // M161: a two-token Document Title is not a body anchor into a
    // long residual. The first residual still pairs positionally.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let mut left = paragraphs(
            &mut dom,
            &["Shared Demo", "matching body", "Document Title"],
        );
        let mut right = paragraphs(
            &mut dom,
            &[
                "Shared Demo",
                "matching body",
                "long residual explains entirely unrelated detailed material here",
            ],
        );
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        let a = text(&dom, &left[2..]);
        let b = text(&dom, &right[2..]);
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "Shared Demo¶", "Shared Demo¶"),
                (Unknown, "matching body¶", "matching body¶"),
                (Inserted, "", &b),
                (Deleted, &a, ""),
            ],
        );
    }
}

#[test]
fn long_unrelated_demo_residuals_pair_crosswise_after_inserting_next_first() {
    // M153: long, disjoint first residuals must bypass diagonal zip;
    // the revised last is the carrier for the original first body.
    let mut dom = Dom::new();
    let left = paragraphs(
        &mut dom,
        &[
            "Shared Demo",
            "alpha bravo charlie delta echo foxtrot golf hotel",
            "old appendix",
        ],
    );
    let right = paragraphs(
        &mut dom,
        &[
            "Shared Demo",
            "india juliet kilo lima mike november oscar papa",
            "new conclusion",
        ],
    );
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Unknown, "Shared Demo¶", "Shared Demo¶"),
            (
                Inserted,
                "",
                "india juliet kilo lima mike november oscar papa¶",
            ),
            (
                Unknown,
                "alpha bravo charlie delta echo foxtrot golf hotel¶",
                "new conclusion¶",
            ),
            (Deleted, "old appendix¶", ""),
        ],
    );
}

#[test]
fn short_first_demo_residual_orders_deletion_before_insertion() {
    // M149: a short original first residual leads the revision pair;
    // the final residuals remain one Unknown for their own resolver.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["Shared Demo", "alpha beta", "old appendix"]);
    let right = paragraphs(
        &mut dom,
        &[
            "Shared Demo",
            "omega sigma tau upsilon phi chi psi",
            "new conclusion",
        ],
    );
    let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Unknown, "Shared Demo¶", "Shared Demo¶"),
            (Deleted, "alpha beta¶", ""),
            (Inserted, "", "omega sigma tau upsilon phi chi psi¶"),
            (Unknown, "old appendix¶", "new conclusion¶"),
        ],
    );
}

#[test]
fn short_prose_to_table_demo_classifies_the_entire_next_stream_first() {
    // M426/M427: tables pack many cells into one group. Count the
    // contentful blocks, not cell atoms, and keep a short base at the end.
    for n in [1, 4] {
        let mut dom = Dom::new();
        let labels: Vec<_> = (0..n).map(|i| format!("legacy{i}")).collect();
        let refs: Vec<_> = labels.iter().map(String::as_str).collect();
        let left = paragraphs(&mut dom, &refs);
        let mut right = paragraphs(&mut dom, &["new report"]);
        right.push(table(&mut dom, "fresh metrics"));
        wholesale(&mut dom, &left, &right);
    }
}

#[test]
fn single_short_base_against_long_prose_is_not_a_last_paragraph_carrier() {
    // M315: the single-base count falls below the classic unrelated
    // gate; nevertheless the long next stream precedes the base deletion.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["hummingbird"]);
    let right = paragraphs(&mut dom, &["alpha", "bravo", "charlie", "delta", "echo"]);
    wholesale(&mut dom, &left, &right);
}

#[test]
fn math_borderbox_medium_and_long_prose_classify_in_both_directions() {
    // M417: medium math versus long prose must survive shared empty
    // structural hashes; real content counts, not the total group count.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["unrelated legal clause"; 30]);
        let mut right = paragraphs(
            &mut dom,
            &[
                "math border box",
                "equation",
                "fraction",
                "radical",
                "integral",
            ],
        );
        left.extend(paragraphs(&mut dom, &[""]));
        right.extend(paragraphs(&mut dom, &[""]));
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        wholesale(&mut dom, &left, &right);
    }
}

#[test]
fn short_alpha_clusters_and_ooxml_property_demos_replace_in_both_directions() {
    // M410: both the two-item and five-item cluster fingerprints route
    // to wholesale replacement, even if the OOXML demo carries a table.
    for cluster in [false, true] {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["OOXML property tester", "sample settings"]);
            left.push(table(&mut dom, "property values"));
            let mut right = paragraphs(
                &mut dom,
                if cluster {
                    &["ONE", "a", "b", "TWO", "c"][..]
                } else {
                    &["ONE", "a"][..]
                },
            );
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            wholesale(&mut dom, &left, &right);
        }
    }
}

#[test]
fn textless_multi_paragraph_replacement_retains_every_layout_mark() {
    // M311b: no contentful next group does not mean an empty next
    // document; each blank paragraph's layout mark must survive insertion.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["visible text"]);
        let mut right = paragraphs(&mut dom, &["", "", ""]);
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        wholesale(&mut dom, &left, &right);
    }
}

#[test]
fn three_num_statistics_with_no_residual_insert_before_table_deletion() {
    // M425: three leading stats are an indivisible inserted prefix;
    // with no residual there is nothing to mesh against original cells.
    let mut dom = Dom::new();
    let left = vec![table(&mut dom, "old cells")];
    let right = paragraphs(&mut dom, &["Num words 10", "Num chars 20", "Num pages 3"]);
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "Num words 10¶Num chars 20¶Num pages 3¶"),
            (Deleted, "old cells¶", ""),
        ],
    );
}

#[test]
fn num_statistics_keep_textless_residual_before_original_layout_deletions() {
    // M431: blank residual layout must be inserted before deleting
    // original blanks; the remaining table is deleted as one source block.
    let mut dom = Dom::new();
    let mut left = paragraphs(&mut dom, &[""]);
    left.push(table(&mut dom, "old cells"));
    let right = paragraphs(
        &mut dom,
        &["Num words 10", "Num characters 20", "Num paragraphs 3", ""],
    );
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (
                Inserted,
                "",
                "Num words 10¶Num characters 20¶Num paragraphs 3¶",
            ),
            (Inserted, "", "¶"),
            (Deleted, "¶", ""),
            (Deleted, "old cells¶", ""),
        ],
    );
}
#[test]
fn positional_zip_preserves_each_mark_shape_and_blank_layout_residual() {
    // Missing marks belong only to their source side. Positional text
    // replacement must not invent a paired mark or swallow blank blocks.
    for (mark_a, mark_b) in [(true, true), (true, false), (false, true), (false, false)] {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["alpha", ""]);
        let mut right = paragraphs(&mut dom, &["omega", ""]);
        if !mark_a && let ComparisonUnit::Group(g) = &mut left[0] {
            g.contents.pop();
        }
        if !mark_b && let ComparisonUnit::Group(g) = &mut right[0] {
            g.contents.pop();
        }
        let out =
            positional_title_token_zip(&mut dom, &left, &right, &WmlComparerSettings::default())
                .unwrap();
        let mut expected = vec![(Deleted, "alpha", ""), (Inserted, "", "omega")];
        match (mark_a, mark_b) {
            (true, true) => expected.push((Equal, "¶", "¶")),
            (true, false) => expected.push((Deleted, "¶", "")),
            (false, true) => expected.push((Inserted, "", "¶")),
            (false, false) => {}
        }
        expected.extend([(Inserted, "", "¶"), (Deleted, "¶", "")]);
        check(&dom, &out, &left, &right, &expected);
    }
}

#[test]
fn positional_zip_declines_a_moved_exact_paragraph_before_changing_ancestors() {
    // Identical paragraph copies in different slots are move anchors,
    // not replacements against the unrelated same-position paragraph.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["stable anchor one", "stable anchor two"]);
    let right = paragraphs(
        &mut dom,
        &["new clause", "stable anchor one", "stable anchor two"],
    );
    let before = left
        .iter()
        .chain(&right)
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
        .collect::<Vec<_>>();
    assert!(
        positional_title_token_zip(&mut dom, &left, &right, &WmlComparerSettings::default())
            .is_none()
    );
    let after = left
        .iter()
        .chain(&right)
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
        .collect::<Vec<_>>();
    assert_eq!(before, after);
}

#[test]
fn num_statistics_then_identical_content_keep_only_the_residual_equal() {
    // Stats peel before cell/prose flattening; equal residual text is
    // retained with the table-side ancestor chain, not assigned Num's chain.
    let mut dom = Dom::new();
    let mut left = paragraphs(&mut dom, &[""]);
    left.push(table(&mut dom, "stable cells"));
    let right = paragraphs(
        &mut dom,
        &[
            "Num words 10",
            "Num chars 20",
            "Num pages 3",
            "",
            "stable cells",
        ],
    );
    let out = detect_unrelated_sources_word_mode_inner(
        &mut dom,
        &left,
        &right,
        &WmlComparerSettings::default(),
    )
    .unwrap();
    check(
        &dom,
        &out,
        &left,
        &right,
        &[
            (Inserted, "", "Num words 10¶Num chars 20¶Num pages 3¶"),
            (Inserted, "", "¶"),
            (Deleted, "¶", ""),
            (Equal, "stable cells", "stable cells"),
            (Equal, "¶", "¶"),
        ],
    );
}

#[test]
fn short_property_prose_gate_accepts_comment_prose_in_either_orientation() {
    // M351's exception is real short prose, including leading blanks;
    // style demos and document-shows boilerplate must stay outside it.
    let mut dom = Dom::new();
    let property = paragraphs(&mut dom, &["OOXML property tester", "sample"]);
    let prose = paragraphs(&mut dom, &["", "Here is a comment", "replacement passage"]);
    assert!(ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 2));
    assert!(ooxml_x_short_prose_demo(&dom, &prose, &property, 2, 2));
    for title in [
        "Font Demo",
        "property tester",
        "This document demonstrates samples",
        "This document shows samples",
    ] {
        let demo = paragraphs(&mut dom, &[title]);
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &demo, 2, 1));
    }
    let too_many = paragraphs(&mut dom, &["one phrase", "two phrase", "three phrase"]);
    assert!(!ooxml_x_short_prose_demo(&dom, &property, &too_many, 2, 3));
    assert!(!ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 5));
    assert_eq!(
        text(&dom, &prose),
        "¶Here is a comment¶replacement passage¶"
    );
}

#[test]
fn cell_only_table_gate_preserves_short_labels_and_rejects_demo_titles() {
    // M338 allows a tiny cell table against a long table report, but
    // never a prose-rich table demo. Pin both orientations and the title
    // exclusions that prevent a wholesale table-title seam regression.
    let mut dom = Dom::new();
    let mut long = vec![table(&mut dom, "unrelated detailed report vocabulary")];
    long.extend(paragraphs(&mut dom, &["report prose"; 14]));
    let cells = vec![table(&mut dom, "name role dept team")];
    assert!(short_cell_table_x_long_table_doc(
        &dom, &cells, &long, 1, 15
    ));
    assert!(short_cell_table_x_long_table_doc(
        &dom, &long, &cells, 15, 1
    ));
    for title in [
        "table labels",
        "SD title",
        "demo labels",
        "three word heading",
    ] {
        let mut short = paragraphs(&mut dom, &[title]);
        short.extend(cells.clone());
        assert!(!short_cell_table_x_long_table_doc(
            &dom, &short, &long, 2, 15
        ));
    }
    let mut two_titles = paragraphs(&mut dom, &["short heading", "tiny caption"]);
    two_titles.extend(cells.clone());
    assert!(!short_cell_table_x_long_table_doc(
        &dom,
        &two_titles,
        &long,
        3,
        15
    ));
    let oversized = vec![table(
        &mut dom,
        "name role dept extraordinarilylongvocabulary",
    )];
    assert!(!short_cell_table_x_long_table_doc(
        &dom, &oversized, &long, 1, 15
    ));
    assert_eq!(text(&dom, &cells), "name role dept team¶");
}

#[test]
fn long_multitable_gate_uses_content_and_direction_not_a_short_table_title_seam() {
    // M348 permits long multi-table families in either orientation;
    // sharing the first title token instead requires structural matching.
    let mut dom = Dom::new();
    let mut long = paragraphs(&mut dom, &["clinical report"]);
    for i in 0..4 {
        long.push(table(&mut dom, &format!("metric{i}")));
    }
    long.extend(paragraphs(&mut dom, &["additional clinical prose"; 25]));
    let mut short = paragraphs(&mut dom, &["directory"]);
    short.push(table(&mut dom, "names"));
    assert!(long_multitable_x_short_table_free_mesh(
        &dom, &long, &short, 30, 2
    ));
    assert!(long_multitable_x_short_table_free_mesh(
        &dom, &short, &long, 2, 30
    ));
    assert!(!long_multitable_x_short_table_free_mesh(
        &dom, &long, &short, 29, 2
    ));
    let mut related = paragraphs(&mut dom, &["clinical directory"]);
    related.push(table(&mut dom, "names"));
    assert!(!long_multitable_x_short_table_free_mesh(
        &dom, &long, &related, 30, 2
    ));
    let mut medium = short.clone();
    medium.extend(paragraphs(&mut dom, &["directory entries"; 9]));
    assert!(both_tables_unrelated_free_mesh(
        &dom, &long, &medium, 30, 11
    ));
    related.extend(paragraphs(&mut dom, &["directory entries"; 9]));
    assert!(!both_tables_unrelated_free_mesh(
        &dom, &long, &related, 30, 11
    ));
    assert_eq!(text(&dom, &short), "directory¶names¶");
}

#[test]
fn paragraph_unids_align_from_the_paragraph_through_a_different_wrapper() {
    // A before paragraph inside SDT and a bare after paragraph share
    // paragraph identity, never the wrapper's identity or the run's.
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["alpha"]);
    let right = paragraphs(&mut dom, &["omega"]);
    let wrapper = dom.new_element(W::sdt());
    dom.set_attribute_value(wrapper, &PT::unid(), Some("wrapper"));
    let mut left = left;
    let source_p = left[0].first_atom().unwrap().ancestor_elements[0];
    let source_r = left[0].first_atom().unwrap().ancestor_elements[1];
    let target_p = right[0].first_atom().unwrap().ancestor_elements[0];
    let target_r = right[0].first_atom().unwrap().ancestor_elements[1];
    for (n, id) in [
        (source_p, "before-p"),
        (source_r, "before-r"),
        (target_p, "after-p"),
        (target_r, "after-r"),
    ] {
        dom.set_attribute_value(n, &PT::unid(), Some(id));
    }
    if let ComparisonUnit::Group(g) = &mut left[0] {
        for u in &mut g.contents {
            if let ComparisonUnit::Word(w) = u {
                for a in &mut w.contents {
                    let mut chain = vec![wrapper];
                    chain.extend(a.ancestor_elements.iter().copied());
                    a.ancestor_elements = chain.into();
                }
            }
        }
    }
    set_after_unids(&mut dom, &unknown(left.clone(), right.clone()));
    assert_eq!(dom.attribute(target_p, &PT::unid()), Some("before-p"));
    assert_eq!(dom.attribute(target_r, &PT::unid()), Some("after-r"));
    assert_eq!(dom.attribute(wrapper, &PT::unid()), Some("wrapper"));
    assert_eq!(
        right[0].first_atom().unwrap().ancestor_elements.as_ref(),
        &[target_p, target_r]
    );
}
#[test]
fn long_prefix_realigns_two_original_paragraphs_into_one_revised_body() {
    // M152: a long rehashed prefix does not justify pairing the second
    // original paragraph with a bare mark. Split the revised body at
    // the prefix, extending through its next word only for a short tail.
    for short_original_tail in [true, false] {
        let mut dom = Dom::new();
        let body_a = dom.new_element(W::body());
        let body_b = dom.new_element(W::body());
        let mut tokens = vec![
            "This",
            " ",
            "document",
            " ",
            "demonstrates",
            " ",
            "justified",
            " ",
            "paragraph",
        ];
        if !short_original_tail {
            tokens.push(".");
        }
        let left = vec![
            paragraph(&mut dom, body_a, &tokens),
            paragraph(&mut dom, body_a, &["tail", " ", "content"]),
        ];
        let right = vec![paragraph(
            &mut dom,
            body_b,
            &[
                "This",
                " ",
                "document",
                " ",
                "demonstrates",
                " ",
                "justified",
                " ",
                "text",
                " ",
                "reflows",
            ],
        )];
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        let expected = if short_original_tail {
            vec![
                (
                    Unknown,
                    "This document demonstrates justified paragraph¶",
                    "This document demonstrates justified text",
                ),
                (Unknown, "tail content¶", " reflows¶"),
            ]
        } else {
            vec![
                (
                    Unknown,
                    "This document demonstrates justified paragraph.¶",
                    "This document demonstrates justified ",
                ),
                (Unknown, "tail content¶", "text reflows¶"),
            ]
        };
        check(&dom, &out, &left, &right, &expected);
    }
}
