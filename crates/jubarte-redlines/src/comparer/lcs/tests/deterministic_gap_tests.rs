// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};

fn word(dom: &mut Dom, text: &str, ancestors: &[NodeId]) -> ComparisonUnit {
    let leaf = dom.new_element(W::t());
    dom.add_text(leaf, text);
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        leaf,
        ancestors.to_vec(),
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

fn paragraph_in(
    dom: &mut Dom,
    parent: NodeId,
    text: &str,
    level: Option<&str>,
    style: Option<&str>,
) -> ComparisonUnit {
    let p = dom.new_element(W::p());
    dom.add(parent, p);
    let ppr = dom.new_element(W::p_pr());
    dom.add(p, ppr);
    if let Some(level) = level {
        let num = dom.new_element(W::num_pr());
        dom.add(ppr, num);
        if level != "default" {
            let ilvl = dom.new_element(W::name("ilvl"));
            dom.set_attribute_value(ilvl, &W::val(), Some(level));
            dom.add(num, ilvl);
        }
    }
    if let Some(style) = style {
        let ps = dom.new_element(W::p_style());
        dom.set_attribute_value(ps, &W::val(), Some(style));
        dom.add(ppr, ps);
    }
    let mut contents = Vec::new();
    if !text.is_empty() {
        let run = dom.new_element(W::r());
        dom.add(p, run);
        let w = word(dom, text, &[p, run]);
        dom.add(run, w.first_atom().unwrap().content_element);
        contents.push(w);
    }
    contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
        ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
    ])));
    group(
        ComparisonUnitGroupType::Paragraph,
        contents,
        &format!("paragraph:{text}"),
    )
}

fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    texts
        .iter()
        .map(|text| paragraph_in(dom, body, text, None, None))
        .collect()
}

fn flatten(units: &[ComparisonUnit]) -> Vec<ComparisonUnit> {
    units.iter().flat_map(group_contents).collect()
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

#[test]
fn alpha_labels_reject_english_bullets_and_overlong_numbers() {
    for label in [
        "ONE", "two", "ten", "A", "z", "II", "viii", "ix", "0", "12", "999",
    ] {
        assert!(is_alpha_list_label_token(label), "{label}");
    }
    for prose in [
        "First", "Second", "Third", "eleven", "1000", "1.", "a)", "é",
    ] {
        assert!(!is_alpha_list_label_token(prose), "{prose}");
    }
}

#[test]
fn alpha_list_counts_and_token_boundaries() {
    let mut dom = Dom::new();
    for (n, short, cluster) in [
        (0, false, false),
        (1, true, false),
        (4, true, false),
        (5, false, true),
        (20, false, true),
        (21, false, false),
    ] {
        let units = paragraphs(&mut dom, &vec!["a"; n]);
        assert_eq!(
            looks_like_short_alpha_list(&dom, &units),
            short,
            "count {n}"
        );
        assert_eq!(
            looks_like_short_alpha_list_cluster(&dom, &units),
            cluster,
            "count {n}"
        );
    }
    for (tokens, short, cluster) in [
        ("a abcdefgh", true, false),
        ("a abcdefghi", false, false),
        ("a b c", false, false),
        ("First Second", false, false),
    ] {
        let units = paragraphs(&mut dom, &["", tokens]);
        assert_eq!(looks_like_short_alpha_list(&dom, &units), short);
        assert_eq!(looks_like_short_alpha_list_cluster(&dom, &units), cluster);
    }
    let mixed = paragraphs(&mut dom, &["a", "b", "c", "ordinary", "ordinary"]);
    assert!(looks_like_short_alpha_list_cluster(&dom, &mixed));
    let long = paragraphs(&mut dom, &["a", "b", "c", "a abcdefghijklm", "d"]);
    assert!(!looks_like_short_alpha_list_cluster(&dom, &long));
}

#[test]
fn cover_markers_are_required_and_count_gated() {
    let mut dom = Dom::new();
    for marker in [
        "agreement",
        "prepared by",
        "memorandum",
        "apprenticeship",
        "march 1",
        "january 1",
        "february 1",
        "april 1",
        "may 1",
        "june 1",
        "july 1",
        "august 1",
        "september 1",
        "october 1",
        "november 1",
        "december 1",
        "to",
        "from",
        "date",
        "re",
    ] {
        let units = paragraphs(
            &mut dom,
            &["", marker, marker, "ordinary prose", "ordinary prose"],
        );
        assert!(looks_like_short_title_page(&dom, &units), "{marker}");
        let one = paragraphs(
            &mut dom,
            &[marker, "ordinary prose", "ordinary prose", "ordinary prose"],
        );
        assert!(
            !looks_like_short_title_page(&dom, &one),
            "one marker: {marker}"
        );
    }
    for (n, expected) in [
        (3, false),
        (4, true),
        (12, true),
        (13, false),
        (14, false),
        (15, false),
    ] {
        let units = paragraphs(&mut dom, &vec!["agreement"; n]);
        assert_eq!(
            looks_like_short_title_page(&dom, &units),
            expected,
            "count {n}"
        );
    }
}

#[test]
fn label_stubs_require_a_majority_of_short_paragraphs() {
    let mut dom = Dom::new();
    for (texts, expected) in [
        (
            vec!["a", "b", "ordinary prose words", "ordinary prose words"],
            true,
        ),
        (
            vec![
                "a",
                "ordinary prose words",
                "ordinary prose words",
                "ordinary prose words",
            ],
            false,
        ),
        (vec!["abcde abcde"; 4], true),
        (vec!["abcdef abcde"; 4], false),
        (vec!["a"; 3], false),
        (vec!["a"; 12], true),
        (vec!["a"; 13], false),
    ] {
        let units = paragraphs(&mut dom, &texts);
        assert_eq!(
            looks_like_short_label_stubs(&dom, &units),
            expected,
            "{texts:?}"
        );
    }
}

#[test]
fn memo_header_cuts_preserve_salutation_and_stop_before_body() {
    let mut dom = Dom::new();
    for (texts, expected) in [
        (vec![""], None),
        (vec!["ordinary prose"], None),
        (vec!["TO Alice", "FROM Bob"], Some(2)),
        (
            vec!["DATE today", "RE topic", "Dear colleague", "body"],
            Some(3),
        ),
        (
            vec!["memorandum", "one two three four five six seven eight"],
            Some(1),
        ),
        (
            vec![
                "to one two three four five six seven eight",
                "from one two three four five six seven eight",
                "date one two three four five six seven eight",
                "re one two three four five six seven eight",
            ],
            Some(4),
        ),
        (vec!["Dear reader"], Some(1)),
    ] {
        let units = paragraphs(&mut dom, &texts);
        assert_eq!(memo_header_cut(&dom, &units), expected, "{texts:?}");
    }
    let units = paragraphs(&mut dom, &["to recipient"; 13]);
    assert_eq!(memo_header_cut(&dom, &units), Some(12));
    for (texts, expected) in [
        (vec!["Memorandum to staff"], true),
        (vec!["To staff", "From manager"], true),
        (vec!["To staff"], false),
        (vec!["", "ordinary"], false),
    ] {
        let units = paragraphs(&mut dom, &texts);
        assert_eq!(looks_like_memo_doc(&dom, &units), expected);
    }
}

#[test]
fn legal_splice_ignores_document_titles_and_counts_three_body_sections() {
    let mut dom = Dom::new();
    let body = dom.new_element(W::body());
    let mut units = vec![
        word(&mut dom, "not a paragraph group", &[]),
        paragraph_in(&mut dom, body, "", None, None),
        paragraph_in(&mut dom, body, "Document title", None, Some("Title")),
        paragraph_in(&mut dom, body, "Major heading", None, Some("Heading1")),
        paragraph_in(&mut dom, body, "First section", None, Some("hEaDiNg2")),
        paragraph_in(&mut dom, body, "2) Rent", None, None),
    ];
    assert_eq!(legal_mid_splice_cut(&dom, &units), None);
    units.push(paragraph_in(&mut dom, body, "3. Term", None, None));
    units.push(paragraph_in(
        &mut dom,
        body,
        "preserved residual body",
        None,
        None,
    ));
    assert_eq!(legal_mid_splice_cut(&dom, &units), Some(7));
    for prefix in ["1", "1.", "1)"] {
        let u = paragraphs(&mut dom, &[prefix, prefix, prefix, "body"]);
        assert_eq!(legal_mid_splice_cut(&dom, &u), Some(3));
    }
    for invalid in [
        "1x section",
        "section 1",
        "1 one two three four five six seven eight nine ten",
    ] {
        let u = paragraphs(&mut dom, &[invalid, invalid, invalid]);
        assert_eq!(legal_mid_splice_cut(&dom, &u), None);
    }
}

#[test]
fn list_level_defaults_and_invalid_values_are_distinct() {
    let mut dom = Dom::new();
    let body = dom.new_element(W::body());
    for (level, expected) in [
        (None, None),
        (Some("default"), Some(0)),
        (Some("0"), Some(0)),
        (Some("2"), Some(2)),
        (Some("bad"), None),
        (Some("-1"), None),
    ] {
        let u = paragraph_in(&mut dom, body, "list item", level, None);
        assert_eq!(unit_para_has_numpr(&dom, &u), level.is_some());
        assert_eq!(unit_para_ilvl(&dom, &u), expected);
    }
    let orphan = word(&mut dom, "orphan", &[]);
    assert!(!unit_para_has_numpr(&dom, &orphan));
    assert_eq!(unit_para_ilvl(&dom, &orphan), None);
    let chain = vec![
        paragraph_in(&mut dom, body, "top", Some("0"), None),
        paragraph_in(&mut dom, body, "nested", Some("1"), None),
        paragraph_in(&mut dom, body, "", None, None),
        paragraph_in(&mut dom, body, "next top", Some("0"), None),
    ];
    assert_eq!(first_list_cluster_end(&dom, &chain), 3);
    assert_eq!(first_list_cluster_end(&dom, &chain[..1]), 1);
    assert_eq!(first_list_cluster_end(&dom, &[]), 0);
}

#[test]
fn short_list_limits_count_content_not_marks() {
    let mut dom = Dom::new();
    let units = paragraphs(&mut dom, &[""]);
    assert!(!mostly_list_paras(&dom, &[flatten(&units)]));
    assert!(!short_item_list_paras(&dom, &[flatten(&units)]));
    assert!(!short_item_list_groups(&dom, &[]));
    for n in [12, 13] {
        let words: Vec<_> = (0..n).map(|_| word(&mut dom, "item", &[])).collect();
        assert_eq!(short_item_list_paras(&dom, &[words]), n == 12);
        let sentence = vec!["item"; n].join(" ");
        let u = paragraphs(&mut dom, &[&sentence]);
        assert_eq!(short_item_list_groups(&dom, &[&u[0]]), n == 12);
    }
    let body = dom.new_element(W::body());
    let numbered = paragraph_in(&mut dom, body, "numbered", Some("0"), None);
    let plain = paragraph_in(&mut dom, body, "plain", None, None);
    assert!(mostly_list_paras(
        &dom,
        &[group_contents(&numbered), group_contents(&plain)]
    ));
    assert!(!mostly_list_paras(
        &dom,
        &[
            group_contents(&numbered),
            group_contents(&plain),
            group_contents(&plain)
        ]
    ));
}

#[test]
fn annotation_and_html_fingerprints_respect_limits() {
    let mut dom = Dom::new();
    for marker in ["suggest a revision", "leave a comment", "oftentimes"] {
        for (n, expected) in [(1, true), (6, true), (7, false)] {
            let units = paragraphs(&mut dom, &vec![marker; n]);
            assert_eq!(looks_like_short_annotation_doc(&dom, &units), expected);
        }
    }
    let plain = paragraphs(&mut dom, &["", "unrelated prose"]);
    assert!(!looks_like_short_annotation_doc(&dom, &plain));
    assert!(!looks_like_fields_html_doc(&dom, &plain));
    let mut visible = vec![""; 19];
    visible.push("HTML input type text");
    let units = paragraphs(&mut dom, &visible);
    assert!(looks_like_fields_html_doc(&dom, &units));
    visible.insert(0, "");
    let units = paragraphs(&mut dom, &visible);
    assert!(!looks_like_fields_html_doc(&dom, &units));
}

#[test]
fn math_borderbox_fingerprint_requires_a_matching_phrase() {
    let mut dom = Dom::new();
    for marker in ["borderbox", "m:borderbox", "math border box"] {
        let units = paragraphs(&mut dom, &["", marker]);
        assert!(looks_like_math_borderbox_doc(&dom, &units), "{marker}");
    }
    for plain in ["border", "box", "math box", "math border", "ordinary"] {
        let units = paragraphs(&mut dom, &[plain]);
        assert!(!looks_like_math_borderbox_doc(&dom, &units), "{plain}");
    }
}

#[test]
fn ooxml_property_titles_do_not_confuse_font_demos() {
    let mut dom = Dom::new();
    for marker in [
        "OOXML",
        "Tester",
        "ST_OnOff",
        "w:b",
        "w:i",
        "w:sz",
        "w:color",
        "w:strike",
        "w:highlight",
        "w:rfonts",
        "rfonts",
        "half-point",
    ] {
        let units = paragraphs(&mut dom, &["", marker]);
        assert!(short_ooxml_property_demo(&dom, &units), "{marker}");
        assert!(short_ooxml_property_demos(&dom, &units, &units));
    }
    for plain in ["Bold Underline Demo", "Font Size Demo", "color sample", ""] {
        let units = paragraphs(&mut dom, &[plain]);
        assert!(!short_ooxml_property_demo(&dom, &units), "{plain}");
    }
    let long = paragraphs(&mut dom, &["OOXML"; 51]);
    assert!(!short_ooxml_property_demo(&dom, &long));
}

#[test]
fn section_labels_require_uppercase_and_three_shared_sections() {
    let mut dom = Dom::new();
    let left = paragraphs(
        &mut dom,
        &["", " A) Alpha", "B) Beta", "C) Gamma", "d) lower", "E. dot"],
    );
    assert_eq!(
        section_letter_labels(&dom, &left),
        ['A', 'B', 'C'].into_iter().collect()
    );
    let same = paragraphs(&mut dom, &["A) other", "B) other", "C) other"]);
    let two = paragraphs(&mut dom, &["A) other", "B) other"]);
    let disjoint = paragraphs(&mut dom, &["D) other", "E) other", "F) other"]);
    assert!(parallel_sectioned_demos(&dom, &left, &same));
    assert!(!parallel_sectioned_demos(&dom, &left, &two));
    assert!(!parallel_sectioned_demos(&dom, &left, &disjoint));
}

#[test]
fn token_probes_join_split_atoms_and_ignore_properties() {
    let mut dom = Dom::new();
    let mut contents = vec![word(&mut dom, "Uni", &[]), word(&mut dom, "code ", &[])];
    contents.extend([
        word(&mut dom, "Δ", &[]),
        word(&mut dom, "42", &[]),
        word(&mut dom, "!\t", &[]),
    ]);
    let u = group(ComparisonUnitGroupType::Paragraph, contents, "unicode");
    assert_eq!(unit_text_token_count(&dom, &u), 2);
    assert_eq!(para_text_token_list(&dom, &u), vec!["unicode", "Δ42"]);
    assert!(unit_has_text_token(&dom, &u));
    let blank = paragraphs(&mut dom, &[" ,\t"]);
    assert_eq!(unit_text_token_count(&dom, &blank[0]), 0);
    assert!(!unit_has_text_token(&dom, &blank[0]));
    assert!(para_text_token_list(&dom, &blank[0]).is_empty());
}

#[test]
fn paragraph_mark_split_and_anchor_keep_boundaries() {
    let mut dom = Dom::new();
    let left = flatten(&paragraphs(&mut dom, &["tiny", "anchor"]));
    let right = flatten(&paragraphs(&mut dom, &["different", "anchor"]));
    assert_eq!(paragraph_final_anchor(&dom, &left, &right), Some((2, 2)));
    assert_eq!(find_index_of_next_para_mark(&dom, &left), 1);
    let chunks = split_at_paragraph_mark(&dom, &left);
    assert_eq!(text(&dom, &chunks[0]), "tiny");
    assert_eq!(text(&dom, &chunks[1]), "¶anchor¶");
    assert!(within_one_paragraph(&dom, &left[..2]));
    assert!(!within_one_paragraph(&dom, &left));
    assert!(within_one_paragraph(&dom, &[]));
    let no_mark = vec![word(&mut dom, "plain", &[])];
    assert_eq!(find_index_of_next_para_mark(&dom, &no_mark), 1);
    assert_eq!(split_at_paragraph_mark(&dom, &no_mark).len(), 1);
    for invalid in ["abc", "long-word", "123!"] {
        let stream = flatten(&paragraphs(&mut dom, &[invalid]));
        assert_eq!(paragraph_final_anchor(&dom, &stream, &stream), None);
    }
}

#[test]
fn paragraph_seams_emit_expected_revisions_and_preserve_all_text() {
    let mut dom = Dom::new();
    let a = flatten(&paragraphs(&mut dom, &["old"]));
    let b = flatten(&paragraphs(&mut dom, &["new"]));
    let mut out = Vec::new();
    seam_region(&dom, &a, &b, &mut out);
    assert_eq!(
        signature(&dom, &out),
        vec![
            (CorrelationStatus::Inserted, "".into(), "new".into()),
            (CorrelationStatus::Deleted, "old".into(), "".into()),
            (CorrelationStatus::Equal, "¶".into(), "¶".into()),
        ]
    );
    let a = flatten(&paragraphs(&mut dom, &["old", "tail"]));
    let b = flatten(&paragraphs(&mut dom, &["head", "new"]));
    out.clear();
    seam_region(&dom, &a, &b, &mut out);
    assert_eq!(
        signature(&dom, &out),
        vec![
            (CorrelationStatus::Inserted, "".into(), "head¶".into()),
            (CorrelationStatus::Inserted, "".into(), "new".into()),
            (CorrelationStatus::Deleted, "old".into(), "".into()),
            (CorrelationStatus::Deleted, "¶tail¶".into(), "".into()),
        ]
    );
    let unmarked = vec![word(&mut dom, "old", &[])];
    out.clear();
    seam_region(&dom, &unmarked, &b[2..], &mut out);
    assert_eq!(
        signature(&dom, &out),
        vec![
            (CorrelationStatus::Inserted, "".into(), "new".into()),
            (CorrelationStatus::Deleted, "old".into(), "".into()),
            (CorrelationStatus::Inserted, "".into(), "¶".into()),
        ]
    );
    for (left, right, status) in [
        (&a[..], &[][..], CorrelationStatus::Deleted),
        (&[][..], &b[..], CorrelationStatus::Inserted),
    ] {
        out.clear();
        seam_region(&dom, left, right, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].correlation_status, status);
        assert_eq!(
            text(&dom, out[0].com_units_1.as_deref().unwrap_or_default()),
            text(&dom, left)
        );
        assert_eq!(
            text(&dom, out[0].com_units_2.as_deref().unwrap_or_default()),
            text(&dom, right)
        );
    }
}

#[test]
fn final_mark_extraction_handles_front_back_and_absence() {
    let mut dom = Dom::new();
    let u = paragraphs(&mut dom, &["body"]);
    let flat = flatten(&u);
    for reverse in [false, true] {
        let mut units = flat.clone();
        if reverse {
            units.reverse();
        }
        let mark = take_paragraph_mark(&dom, &mut units).unwrap();
        assert!(unit_is_single_atom_ppr(&dom, &mark));
        assert_eq!(text(&dom, &units), "body");
        assert!(take_paragraph_mark(&dom, &mut units).is_none());
    }
    let mut grouped = u.clone();
    assert!(ends_with_mark(&dom, &grouped));
    assert!(unit_is_single_atom_ppr(
        &dom,
        &split_final_mark(&dom, &mut grouped).unwrap()
    ));
    assert_eq!(text(&dom, &grouped), "body");
    assert!(!ends_with_mark(&dom, &grouped));
    assert!(split_final_mark(&dom, &mut Vec::new()).is_none());
}

#[test]
fn replaced_story_pairs_final_marks_and_retains_both_contents() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["old"]);
    let right = paragraphs(&mut dom, &["new"]);
    for inserted_first in [false, true] {
        let mut seqs = vec![
            CorrelatedSequence::deleted(left.clone()),
            CorrelatedSequence::inserted(right.clone()),
        ];
        if inserted_first {
            seqs.reverse();
        }
        pair_story_final_marks(&dom, &mut seqs);
        assert_eq!(
            signature(&dom, &seqs),
            vec![
                (CorrelationStatus::Inserted, "".into(), "new".into()),
                (CorrelationStatus::Deleted, "old".into(), "".into()),
                (CorrelationStatus::Equal, "¶".into(), "¶".into()),
            ]
        );
    }
    let l = flatten(&paragraphs(&mut dom, &[""]));
    let r = flatten(&paragraphs(&mut dom, &[""]));
    let mut seqs = vec![
        CorrelatedSequence::inserted(r),
        CorrelatedSequence::deleted(l),
    ];
    assert!(pair_final_marks_behind_inserted_tail(&dom, &mut seqs));
    assert_eq!(
        signature(&dom, &seqs),
        vec![(CorrelationStatus::Equal, "¶".into(), "¶".into())]
    );
}

#[test]
fn deleted_tail_moves_the_revised_closing_mark_to_original_close() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["kept", "tail"]);
    let right = paragraphs(&mut dom, &["kept"]);
    let mut seqs = vec![
        CorrelatedSequence::paired(CorrelationStatus::Equal, vec![left[0].clone()], right),
        CorrelatedSequence::deleted(vec![left[1].clone()]),
    ];
    assert!(pair_final_marks_past_deleted_tail(&dom, &mut seqs));
    assert_eq!(
        signature(&dom, &seqs),
        vec![
            (CorrelationStatus::Equal, "kept".into(), "kept".into()),
            (CorrelationStatus::Deleted, "¶tail".into(), "".into()),
            (CorrelationStatus::Equal, "¶".into(), "¶".into()),
        ]
    );
}

#[test]
fn story_closure_requires_a_story_parent_and_only_section_properties_after() {
    let mut dom = Dom::new();
    for story in [W::body(), W::tc(), W::txbx_content()] {
        let parent = dom.new_element(story);
        let first = paragraph_in(&mut dom, parent, "first", None, None);
        let last = paragraph_in(&mut dom, parent, "last", None, None);
        assert!(!unit_closes_story(&dom, &first));
        assert!(unit_closes_story(&dom, &last));
        let section = dom.new_element(W::sect_pr());
        dom.add(parent, section);
        assert!(unit_closes_story(&dom, &last));
        let table = dom.new_element(W::tbl());
        dom.add(parent, table);
        assert!(!unit_closes_story(&dom, &last));
    }
    let parent = dom.new_element(W::sdt_content());
    let p = paragraph_in(&mut dom, parent, "nested control", None, None);
    assert!(!unit_closes_story(&dom, &p));
    let orphan = word(&mut dom, "orphan", &[]);
    assert_eq!(story_closing_paragraph(&dom, &orphan), None);
}

#[test]
fn final_mark_pairing_rejects_tables_nonclosing_and_incomplete_tails() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["old", "following"]);
    let right = paragraphs(&mut dom, &["new"]);
    let table = group(
        ComparisonUnitGroupType::Table,
        group_contents(&left[1]),
        "table",
    );
    for mut seqs in [
        vec![],
        vec![CorrelatedSequence::inserted(right.clone())],
        vec![
            CorrelatedSequence::inserted(right.clone()),
            CorrelatedSequence::deleted(vec![left[0].clone()]),
        ],
        vec![
            CorrelatedSequence::inserted(right.clone()),
            CorrelatedSequence::deleted(vec![table]),
        ],
        vec![
            CorrelatedSequence::deleted(left.clone()),
            CorrelatedSequence::deleted(left.clone()),
        ],
        vec![
            CorrelatedSequence::deleted(vec![word(&mut dom, "unmarked", &[])]),
            CorrelatedSequence::inserted(right.clone()),
        ],
    ] {
        let before = signature(&dom, &seqs);
        pair_story_final_marks(&dom, &mut seqs);
        assert_eq!(signature(&dom, &seqs), before);
    }
}

#[test]
fn visible_objects_are_not_blank_paragraph_matter() {
    let mut dom = Dom::new();
    for name in [
        W::drawing(),
        W::pict(),
        W::object(),
        W::name("sym"),
        M::name("oMath"),
        M::name("oMathPara"),
    ] {
        let leaf = dom.new_element(name.clone());
        let u = group(
            ComparisonUnitGroupType::Paragraph,
            vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(leaf, Vec::<NodeId>::new(), "visible"),
            ]))],
            name.local_name(),
        );
        assert!(!unit_is_textless(&dom, &u), "{}", name.local_name());
        assert!(!unit_is_textless_paragraph_matter(&dom, &u));
        assert_eq!(group_has_math(&dom, &u), name.namespace_name() == MATH_URI);
        assert_eq!(
            group_has_drawing_or_pict(&dom, &u),
            matches!(name.local_name(), "drawing" | "pict" | "object")
        );
    }
    let blank = paragraphs(&mut dom, &[" \t"]);
    assert!(unit_is_textless_paragraph_matter(&dom, &blank[0]));
    let table = group(
        ComparisonUnitGroupType::Table,
        group_contents(&blank[0]),
        "blank-table",
    );
    assert!(!unit_is_textless_paragraph_matter(&dom, &table));
}

#[test]
fn closing_paragraph_chain_counts_whole_paragraphs_and_stops_at_table() {
    let mut dom = Dom::new();
    let u = paragraphs(&mut dom, &["words", "", " "]);
    let flat = flatten(&u);
    assert_eq!(
        closing_paragraphs(&dom, &flat),
        vec![(3, true), (2, true), (0, false)]
    );
    assert_eq!(
        closing_paragraphs(&dom, &u),
        vec![(2, true), (1, true), (0, false)]
    );
    assert!(interior_blank_chain_holds(&dom, &flat, &flat));
    let blank = flatten(&paragraphs(&mut dom, &[""]));
    let words = flatten(&paragraphs(&mut dom, &["body"]));
    assert!(!interior_blank_chain_holds(&dom, &words, &blank));
    assert!(!interior_blank_chain_holds(&dom, &blank, &words));
    let mut blocked = vec![group(ComparisonUnitGroupType::Table, Vec::new(), "table")];
    blocked.extend(blank);
    assert_eq!(closing_paragraphs(&dom, &blocked), vec![(1, true)]);
}

#[test]
fn word_mode_scores_ignore_separators_and_count_unicode_characters() {
    let mut dom = Dom::new();
    let units = vec![
        word(&mut dom, " a-b ", &[]),
        word(&mut dom, "\t", &[]),
        word(&mut dom, "é猫", &[]),
    ];
    let settings = WmlComparerSettings {
        word_separators: vec!['-'],
        ..WmlComparerSettings::default()
    };
    assert_eq!(
        non_separator_prefix_sums(&dom, &units, &settings),
        vec![0, 2, 2, 4]
    );
    assert_eq!(run_non_separator_text_len(&dom, &units, &settings), 4);
    assert_eq!(run_real_text_len(&dom, &units), 5);
    assert_eq!(
        common_run_content_score(Some(&dom), &units, 0, 3, Some(&settings), None),
        4
    );
    assert_eq!(common_run_content_score(None, &units, 0, 3, None, None), 3);
    let prefix = non_separator_prefix_sums(&dom, &units, &settings);
    assert_eq!(
        common_run_content_score(Some(&dom), &units, 1, 2, Some(&settings), Some(&prefix)),
        2
    );
    let nested = group(ComparisonUnitGroupType::Paragraph, units, "nested");
    assert_eq!(unit_non_separator_text_len(&dom, &nested, &settings), 4);
}

#[test]
fn candidate_ranking_prefers_content_length_then_diagonal_with_stable_ties() {
    let incumbent = (4, 2, 0, 5);
    for (candidate, diagonal, replaces) in [
        ((5, 1, 9, 0), false, true),
        ((3, 9, 0, 0), true, false),
        ((4, 3, 9, 0), false, true),
        ((4, 1, 0, 0), true, false),
        ((4, 2, 2, 2), true, true),
        ((4, 2, 2, 2), false, false),
        ((4, 2, 6, 0), true, false),
        ((4, 2, 1, 6), true, false),
    ] {
        let mut best = Some(incumbent);
        consider_candidate(&mut best, candidate, diagonal);
        assert_eq!(best, Some(if replaces { candidate } else { incumbent }));
    }
    let mut best = None;
    consider_candidate(&mut best, incumbent, true);
    assert_eq!(best, Some(incumbent));
}

#[test]
fn common_window_thresholds_include_zero_and_exact_boundary() {
    let mut dom = Dom::new();
    let a: Vec<_> = ["x", "a", "b", "y"]
        .iter()
        .map(|s| word(&mut dom, s, &[]))
        .collect();
    let b: Vec<_> = ["z", "a", "b", "q"]
        .iter()
        .map(|s| word(&mut dom, s, &[]))
        .collect();
    for (target, expected) in [
        (0, true),
        (1, true),
        (2, true),
        (3, false),
        (4, false),
        (5, false),
    ] {
        assert_eq!(
            has_common_run_ge(&a, &b, target),
            expected,
            "target {target}"
        );
    }
    assert!(has_common_run_ge(&[], &[], 0));
    assert!(!has_common_run_ge(&[], &b, 1));
}

#[test]
fn weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys() {
    type Case<'a> = (&'a [u32], &'a [u32], &'a [u32], Vec<(usize, usize)>);
    let cases: Vec<Case<'_>> = vec![
        (&[], &[], &[], vec![]),
        (&[0], &[0], &[10], vec![]),
        (&[1], &[2], &[10], vec![]),
        (&[1], &[2, 1, 3], &[10], vec![(0, 1)]),
        (&[1, 2], &[2, 1], &[9, 1], vec![(0, 1)]),
        (&[1, 2], &[2, 1], &[1, 9], vec![(1, 0)]),
        (
            &[1, 2, 3],
            &[1, 0, 2, 3],
            &[1, 2, 3],
            vec![(0, 0), (1, 2), (2, 3)],
        ),
        (
            &[0, 1, 0, 2],
            &[0, 1, 0, 2],
            &[99, 2, 99, 3],
            vec![(1, 1), (3, 3)],
        ),
    ];
    for (left, right, weights, expected) in cases {
        assert_eq!(weighted_lcs_pairs(left, right, weights), expected);
        assert_eq!(heckel_links(left, right, weights), expected);
        let expected_weight: u64 = expected.iter().map(|&(i, _)| u64::from(weights[i])).sum();
        assert_eq!(
            weighted_lcs_row(left, right, weights).last(),
            Some(&expected_weight)
        );
    }
    assert_eq!(
        heckel_links(&[1, 2, 2, 3], &[1, 2, 2, 3], &[1; 4]),
        vec![(0, 0), (1, 1), (2, 2), (3, 3)]
    );
    assert_eq!(
        heckel_links(&[2, 2, 3], &[2, 2, 3], &[1; 3]),
        vec![(0, 0), (1, 1), (2, 2)]
    );
    assert!(heckel_links(&[2, 2], &[2, 2], &[1; 2]).is_empty());
    assert_eq!(
        weighted_lcs_pairs(&[2, 2], &[2, 2], &[1; 2]),
        vec![(0, 0), (1, 1)]
    );
    assert_eq!(weighted_lcs_pairs(&[1], &[2, 1, 1], &[10]), vec![(0, 1)]);
    assert!(heckel_links(&[1], &[2, 1, 1], &[10]).is_empty());
}

#[test]
fn same_slot_pairs_use_content_evidence_not_function_words_or_numbers() {
    let mut dom = Dom::new();
    for (left, right, expected) in [
        (
            vec!["alpha old", "beta old"],
            vec!["alpha new", "beta new"],
            vec![(0, 0), (1, 1)],
        ),
        (vec!["the and 123"], vec!["the and 456"], vec![]),
        (vec!["alpha"], vec!["alpha one two three"], vec![]),
        (
            vec!["alpha beta"],
            vec!["alpha beta one two three four five"],
            vec![(0, 0)],
        ),
        (
            vec!["alpha"],
            vec!["alpha", "beta", "gamma", "delta"],
            vec![],
        ),
        (
            vec!["alpha beta", "alpha beta gamma"],
            vec!["alpha gamma", "unrelated"],
            vec![],
        ),
    ] {
        let l = paragraphs(&mut dom, &left);
        let r = paragraphs(&mut dom, &right);
        assert_eq!(
            same_slot_pairs(&dom, &l, &r),
            expected,
            "{left:?} vs {right:?}"
        );
    }
    assert!(same_slot_pairs(&dom, &[], &[]).is_empty());
}

#[test]
fn body_relatedness_requires_substantial_vocabulary() {
    let mut dom = Dom::new();
    assert_eq!(body_token_overlap_ratio(&dom, &[], &[]), 0.0);
    for (n, related) in [(39, false), (40, true)] {
        let sentence = (0..n)
            .map(|i| format!("token{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let left = paragraphs(&mut dom, &[&sentence]);
        let right = paragraphs(&mut dom, &[&sentence]);
        assert_eq!(body_token_overlap_ratio(&dom, &left, &right), 1.0);
        assert_eq!(is_related_stamped_variant(&dom, &left, &right), related);
        assert_eq!(should_stamp_confetti(&dom, &left, &right), !related);
    }
    let left = paragraphs(&mut dom, &["alpha beta gamma"]);
    let right = paragraphs(&mut dom, &["alpha delta epsilon"]);
    assert!((body_token_overlap_ratio(&dom, &left, &right) - 1.0 / 3.0).abs() < 1e-12);
}

#[test]
fn diagonal_and_residual_relatedness_keep_real_content_matches() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["Title Demo", "This alpha clause"]);
    let same = paragraphs(&mut dom, &["Title Demo", "This alpha clause"]);
    let swapped = vec![same[1].clone(), same[0].clone()];
    let unrelated = paragraphs(&mut dom, &["Other Demo", "zebra quartz"]);
    assert!(para_zip_diagonal_dominant(&dom, &left, &same));
    assert!(!para_zip_diagonal_dominant(&dom, &left, &swapped));
    assert!(!para_zip_diagonal_dominant(&dom, &[], &same));
    assert_eq!(m123_diagonal_stats(&dom, &left, &same), (1.0, 1.0, 1.0));
    assert_eq!(
        m123_diagonal_stats(&dom, &left[..1], &same),
        (0.0, 0.0, 0.0)
    );
    assert!(first_paras_share_last_sig(&dom, &left, &unrelated));
    assert!(!first_paras_share_last_sig(&dom, &[], &same));
    assert!(!body_residual_unrelated(&dom, &left, &same));
    assert!(body_residual_unrelated(&dom, &left, &unrelated));
    assert!(residual_bodies_this_cousins(&dom, &left, &same));
    assert!(!residual_bodies_this_cousins(&dom, &left[..1], &same));
    assert!(!residual_bodies_this_cousins(&dom, &left, &unrelated));
    assert!(residual_title_ends_demo(&dom, &left[0]));
    assert!(residual_first_body_starts_this(&dom, &left));
    assert!(!residual_has_this_body_after_non_this(&dom, &left));
    let subtitle = paragraphs(&mut dom, &["title", "subtitle", "This body"]);
    assert!(residual_has_this_body_after_non_this(&dom, &subtitle));
    assert_eq!(residual_first_this_body_index(&dom, &subtitle), Some(2));
    assert!(!residual_has_this_body_after_non_this(&dom, &unrelated));
    assert!(!residual_sets_weakly_related(&dom, &[], &same));
}

#[test]
fn atom_correlation_preserves_revision_order_and_modified_equal_atom() {
    let mut dom = Dom::new();
    let left =
        ["old", "keep", "tail"].map(|s| word(&mut dom, s, &[]).first_atom().unwrap().clone());
    let right =
        ["new", "keep", "extra"].map(|s| word(&mut dom, s, &[]).first_atom().unwrap().clone());
    let got = correlate_atoms(&left, &right);
    assert_eq!(
        got.iter()
            .map(|a| (a.status, dom.value_str(a.atom.content_element).into_owned()))
            .collect::<Vec<_>>(),
        vec![
            (CorrelationStatus::Deleted, "old".into()),
            (CorrelationStatus::Inserted, "new".into()),
            (CorrelationStatus::Equal, "keep".into()),
            (CorrelationStatus::Deleted, "tail".into()),
            (CorrelationStatus::Inserted, "extra".into()),
        ]
    );
    assert_eq!(got[2].atom.content_element, right[1].content_element);
    assert_ne!(got[2].atom.content_element, left[1].content_element);
    assert!(correlate_atoms(&[], &[]).is_empty());
    assert!(
        correlate_atoms(&left, &[])
            .iter()
            .all(|a| a.status == CorrelationStatus::Deleted)
    );
    assert!(
        correlate_atoms(&[], &right)
            .iter()
            .all(|a| a.status == CorrelationStatus::Inserted)
    );
}

#[test]
fn kept_span_counts_only_kept_words_and_shared_surrounding_blanks() {
    let weights = [0, 3, 0, 4, 0];
    let chars = [1, 3, 2, 4, 1];
    let blank = [true, false, true, false, true];
    assert_eq!(kept_span(&[], &weights, &chars, &blank, &blank), 0);
    assert_eq!(kept_span(&[(1, 1)], &weights, &chars, &blank, &blank), 6);
    assert_eq!(
        kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blank, &blank),
        11
    );
    let missing_blank = [false, false, false, false, false];
    assert_eq!(
        kept_span(&[(1, 1)], &weights, &chars, &blank, &missing_blank),
        3
    );
    assert_eq!(
        kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blank, &missing_blank),
        7
    );
    assert_eq!(kept_span(&[(0, 0)], &[3], &[3], &[false], &[false]), 3);
}

#[test]
fn peeling_single_revised_paragraph_preserves_words_and_pairs_only_marks() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["first", "last"]);
    let right = paragraphs(&mut dom, &["revised"]);
    let mut lg = vec![(0, group_contents(&left[0])), (1, group_contents(&left[1]))];
    let mut rg = vec![(0, group_contents(&right[0]))];
    let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
    assert_eq!((text(&dom, &a), text(&dom, &b)), ("¶".into(), "¶".into()));
    assert_eq!(lg.len(), 2);
    assert_eq!(text(&dom, &lg[0].1), "first¶");
    assert_eq!(text(&dom, &lg[1].1), "last");
    assert_eq!(text(&dom, &rg[0].1), "revised");
}

#[test]
fn peeling_empty_revised_close_after_table_keeps_original_words() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["original"]);
    let right = paragraphs(&mut dom, &[""]);
    let mut lg = vec![(0, group_contents(&left[0]))];
    let table = group(ComparisonUnitGroupType::Table, Vec::new(), "table");
    let mut rg = vec![(1, vec![table]), (0, group_contents(&right[0]))];
    let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
    assert_eq!((text(&dom, &a), text(&dom, &b)), ("¶".into(), "¶".into()));
    assert_eq!(text(&dom, &lg[0].1), "original");
    assert_eq!(rg.len(), 1);
    assert_eq!(
        as_group(&rg[0].1[0]).unwrap().group_type,
        ComparisonUnitGroupType::Table
    );
}

#[test]
fn peeling_blank_final_group_requires_compatible_unequal_runs() {
    let mut dom = Dom::new();
    let left = paragraphs(&mut dom, &["first", ""]);
    let right = paragraphs(&mut dom, &["first", "middle", "last"]);
    let mut lg = vec![(0, vec![left[0].clone()]), (0, vec![left[1].clone()])];
    let mut rg = vec![
        (0, vec![right[0].clone()]),
        (1, vec![right[1].clone()]),
        (0, vec![right[2].clone()]),
    ];
    let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
    assert_eq!(
        (text(&dom, &a), text(&dom, &b)),
        ("¶".into(), "last¶".into())
    );
    assert_eq!((lg.len(), rg.len()), (1, 2));
    let mut lg = vec![(0, vec![left[0].clone()]), (0, vec![left[1].clone()])];
    let mut rg = lg.clone();
    assert!(peel_story_final_groups(&dom, &mut lg, &mut rg).is_none());
    assert_eq!((lg.len(), rg.len()), (2, 2));
}

#[test]
fn ancestor_unids_align_by_group_boundary_and_preserve_missing_ids() {
    let mut dom = Dom::new();
    for (kind, boundary) in [
        (ComparisonUnitGroupType::Paragraph, W::p()),
        (ComparisonUnitGroupType::Table, W::tbl()),
        (ComparisonUnitGroupType::Row, W::tr()),
        (ComparisonUnitGroupType::Cell, W::tc()),
        (ComparisonUnitGroupType::Textbox, W::txbx_content()),
    ] {
        let outer_a = dom.new_element(W::sdt());
        let a = dom.new_element(boundary.clone());
        let b = dom.new_element(boundary);
        dom.set_attribute_value(outer_a, &PT::unid(), Some("outer"));
        dom.set_attribute_value(a, &PT::unid(), Some("original"));
        dom.set_attribute_value(b, &PT::unid(), Some("revised"));
        let aw = word(&mut dom, "old", &[outer_a, a]);
        let bw = word(&mut dom, "new", &[b]);
        let left = group(kind, vec![aw], "old");
        let right = group(kind, vec![bw], "new");
        let seq = CorrelatedSequence::paired(CorrelationStatus::Unknown, vec![left], vec![right]);
        set_after_unids(&mut dom, &seq);
        assert_eq!(dom.attribute(b, &PT::unid()), Some("original"));
        assert_eq!(dom.attribute(outer_a, &PT::unid()), Some("outer"));
        dom.set_attribute_value(b, &PT::unid(), None);
        set_after_unids(&mut dom, &seq);
        assert_eq!(dom.attribute(b, &PT::unid()), None);
    }
    let a = paragraphs(&mut dom, &["old"]);
    let b = paragraphs(&mut dom, &["new"]);
    let p = b[0].first_atom().unwrap().ancestor_elements[0];
    dom.set_attribute_value(p, &PT::unid(), Some("untouched"));
    for seq in [
        CorrelatedSequence::deleted(a.clone()),
        CorrelatedSequence::paired(CorrelationStatus::Unknown, a.clone(), vec![]),
        CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![word(&mut dom, "word", &[])],
            b.clone(),
        ),
        CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            a,
            vec![group(
                ComparisonUnitGroupType::Table,
                group_contents(&b[0]),
                "table",
            )],
        ),
    ] {
        set_after_unids(&mut dom, &seq);
        assert_eq!(dom.attribute(p, &PT::unid()), Some("untouched"));
    }
}

#[test]
fn unrelated_source_shortcut_requires_four_disjoint_block_groups() {
    let mut dom = Dom::new();
    let a = paragraphs(&mut dom, &["alpha", "beta", "gamma", "delta"]);
    let b = paragraphs(&mut dom, &["epsilon", "zeta", "eta", "theta"]);
    let seqs = detect_unrelated_sources(&a, &b).unwrap();
    assert_eq!(
        signature(&dom, &seqs),
        vec![
            (
                CorrelationStatus::Deleted,
                "alpha¶beta¶gamma¶delta¶".into(),
                "".into()
            ),
            (
                CorrelationStatus::Inserted,
                "".into(),
                "epsilon¶zeta¶eta¶theta¶".into()
            ),
        ]
    );
    assert!(detect_unrelated_sources(&a[..3], &b).is_none());
    assert!(detect_unrelated_sources(&a, &b[..3]).is_none());
    let mut shared = b;
    shared[3] = a[0].clone();
    assert!(detect_unrelated_sources(&a, &shared).is_none());
}

#[test]
fn math_detection_finds_direct_and_ancestor_equations_with_real_text() {
    let mut dom = Dom::new();
    for local in ["oMath", "oMathPara"] {
        let equation = dom.new_element(M::name(local));
        let math_atom =
            ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
                equation,
                Vec::<NodeId>::new(),
                "equation",
            )]));
        let ordinary = word(&mut dom, "formula", &[]);
        let u = group(
            ComparisonUnitGroupType::Paragraph,
            vec![ordinary, math_atom],
            "direct-math",
        );
        assert!(looks_like_math_doc(&dom, &[u]));
        let ancestor_text = word(&mut dom, "formula", &[equation]);
        assert!(looks_like_math_doc(&dom, &[ancestor_text]));
    }
    let plain = paragraphs(&mut dom, &["ordinary prose", ""]);
    assert!(!looks_like_math_doc(&dom, &plain));
}

/// One key, two weights: under case_insensitive the key is the uppercased
/// text, the weight the raw characters, so `SS` (2) and `ß` (1) share a
/// key. The heaviest common subsequence of [SS, ß] against [ß] keeps the
/// SS match (2); the row must not trade it for the later, lighter one
/// (#2454).
#[test]
fn weighted_lcs_keeps_the_heavier_match_when_one_key_carries_two_weights() {
    assert_eq!(weighted_lcs_row(&[7, 7], &[7], &[2, 1]).last(), Some(&2));
    let pairs = weighted_lcs_pairs(&[7, 7], &[7], &[2, 1]);
    assert_eq!(pairs, vec![(0, 0)], "the pairs keep the weight-2 match");
}
