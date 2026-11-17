// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

const DATE: &str = "2001-02-03T04:05:06Z";
const NUMBERED: &str = "<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"9\"/></w:numPr>";
const NESTED: &str = "<w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"9\"/></w:numPr>";

fn document(fragment: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><w:body>{fragment}</w:body></w:document>"
        ));
    let root = dom.root(doc).expect("in-memory document");
    let body = dom.element(root, &W::body()).expect("body");
    (dom, root, body)
}

fn revision(kind: &str, text: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:{kind} w:id=\"17\" w:author=\"Editor\" w:date=\"{DATE}\"><w:r><w:{tag} xml:space=\"preserve\">{text}</w:{tag}></w:r></w:{kind}>"
    )
}

fn para(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p w:rsidR=\"ABCD\"><w:pPr>{props}<w:rPr><w:{kind} w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}</w:p>",
        revision(kind, text)
    )
}

fn empty(kind: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr></w:p>"
    )
}

fn live(text: &str) -> String {
    format!("<w:p><w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>")
}

fn blocks(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.elements(body, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::sect_pr()))
        .collect()
}

// Independent source projections: paragraph properties are not body text,
// and a rejected/accepted revision excludes its entire nested subtree.
fn projection(dom: &Dom, node: NodeId, accept: bool) -> String {
    let omitted = if accept { W::del() } else { W::ins() };
    if dom.name_is(node, &W::p_pr()) || dom.name_is(node, &omitted) {
        return String::new();
    }
    if dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()) {
        return dom.value(node);
    }
    dom.elements(node, None)
        .into_iter()
        .map(|n| projection(dom, n, accept))
        .collect()
}

fn views(dom: &Dom, root: NodeId) -> (String, String) {
    (projection(dom, root, true), projection(dom, root, false))
}

fn body_revisions(dom: &Dom, root: NodeId) -> Vec<String> {
    let mut xml: Vec<String> = [W::ins(), W::del()]
        .iter()
        .flat_map(|name| dom.descendants(root, Some(name)))
        .filter(|&n| {
            !dom.ancestors_and_self(n, None)
                .iter()
                .any(|&a| dom.name_is(a, &W::p_pr()))
        })
        .map(|n| dom.serialize_element(n))
        .collect();
    xml.sort();
    xml
}

fn properties(dom: &Dom, p: NodeId) -> Option<String> {
    dom.element(p, &W::p_pr()).map(|n| dom.serialize_element(n))
}

fn segments(dom: &Dom, p: NodeId) -> Vec<(String, String)> {
    dom.elements(p, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::p_pr()))
        .map(|n| {
            (
                dom.name(n).expect("body element").local_name().to_string(),
                markup_text(dom, n),
            )
        })
        .collect()
}

fn expect_segments(dom: &Dom, p: NodeId, expected: &[(&str, &str)]) {
    assert_eq!(
        segments(dom, p),
        expected
            .iter()
            .map(|&(k, t)| (k.to_string(), t.to_string()))
            .collect::<Vec<_>>()
    );
}

fn merge(fragment: &str, fold: bool) -> (Dom, NodeId, NodeId, Vec<NodeId>) {
    let (mut dom, root, body) = document(fragment);
    let original = blocks(&dom, body);
    let text = views(&dom, root);
    let revisions = body_revisions(&dom, root);
    merge_replaced_in_container(&mut dom, body, "Editor", fold);
    assert_eq!(
        views(&dom, root),
        text,
        "both source text streams survive merging"
    );
    assert_eq!(
        body_revisions(&dom, root),
        revisions,
        "body revisions retain content, format and attribution"
    );
    let once = dom.serialize_element(root);
    merge_replaced_in_container(&mut dom, body, "Editor", fold);
    assert_eq!(
        dom.serialize_element(root),
        once,
        "merging reaches a fixed point"
    );
    (dom, root, body, original)
}

fn unchanged(fragment: &str, pass: fn(&mut Dom, NodeId)) {
    let (mut dom, root, _) = document(fragment);
    let original = dom.serialize_element(root);
    let text = views(&dom, root);
    pass(&mut dom, root);
    assert_eq!(dom.serialize_element(root), original);
    assert_eq!(views(&dom, root), text);
}

#[test]
fn legal_and_memo_midsplices_keep_each_source_paragraph_in_order() {
    // Exercise both alternatives of M393/M394, plus each memo fingerprint.
    for (leading, deleted, inserted, nested, memo) in [
        (1, 2, 2, true, ""),
        (3, 5, 5, false, ""),
        (20, 5, 5, false, ""),
        (0, 3, 5, false, "Memorandum"),
        (0, 3, 5, false, "To counsel"),
        (0, 20, 5, false, "From counsel"),
    ] {
        let mut fragment = para("ins", "Lead", "").repeat(leading);
        for i in 0..deleted {
            fragment.push_str(&para(
                "del",
                if i == 0 && !memo.is_empty() {
                    memo
                } else {
                    "Base clause"
                },
                if nested { NESTED } else { "" },
            ));
        }
        fragment.push_str(&para("ins", "New clause", "").repeat(inserted));
        let (dom, root, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body),
            old,
            "midsplice {leading}/{deleted}/{inserted}/{memo}"
        );
        assert_eq!(
            dom.descendants(root, Some(&W::p())).len(),
            leading + deleted + inserted
        );
    }
}

#[test]
fn nested_level_parsing_controls_interleave_without_changing_markup() {
    for (level, nested) in [
        ("1", true),
        ("4294967295", true),
        ("0", false),
        ("-1", false),
        ("bad", false),
        ("4294967296", false),
    ] {
        let props = format!("<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"9\"/></w:numPr>");
        let fragment = format!(
            "{}{}{}{}{}",
            para("ins", "Lead", ""),
            para("del", "Old top", &props),
            para("del", "Old sub", ""),
            para("ins", "New alpha", ""),
            para("ins", "New beta", "")
        );
        let (dom, _, body, old) = merge(&fragment, false);
        let expected = if nested {
            old.clone()
        } else {
            vec![old[0], old[3], old[4], old[1], old[2]]
        };
        assert_eq!(blocks(&dom, body), expected, "level {level}");
        assert_eq!(
            dom.attribute(
                dom.descendants(old[1], Some(&W::name("ilvl")))[0],
                &W::val()
            ),
            Some(level)
        );
    }
}

#[test]
fn legal_midsplice_count_boundaries_reorder_outside_the_guard() {
    for (leading, deleted, inserted) in [(2, 5, 5), (21, 5, 5), (3, 4, 5), (3, 5, 4)] {
        let fragment = format!(
            "{}{}{}",
            para("ins", "Revised leading words", "").repeat(leading),
            para("del", "Former contract text", "").repeat(deleted),
            para("ins", "Fresh changed provision", "").repeat(inserted)
        );
        let (dom, _, body, old) = merge(&fragment, false);
        let expected: Vec<_> = old[..leading]
            .iter()
            .chain(&old[leading + deleted..])
            .chain(&old[leading..leading + deleted])
            .copied()
            .collect();
        assert_eq!(
            blocks(&dom, body),
            expected,
            "counts {leading}/{deleted}/{inserted}"
        );
    }
}

#[test]
fn prior_nested_cluster_keeps_short_labels_but_allows_a_longer_carrier() {
    for (last, folds) in [
        ("New gamma", false),
        ("Revised final clause continues", true),
    ] {
        let fragment = format!(
            "{}{}{}{}{}{}{}",
            para("ins", "Lead", ""),
            para("del", "Old top", NUMBERED),
            para("del", "Old nested", NESTED),
            para("ins", "New alpha", ""),
            para("ins", "New beta", ""),
            para("ins", last, ""),
            para("del", "Residual item", NUMBERED)
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body),
            if folds {
                old[..6].to_vec()
            } else {
                old.clone()
            }
        );
        if folds {
            expect_segments(&dom, old[5], &[("ins", last), ("del", "Residual item")]);
            assert!(para_mark_revision(&dom, old[5], &W::del()));
            assert!(para_has_live_numpr(&dom, old[5]));
        }
    }
}

#[test]
fn long_content_cannot_reach_back_across_an_empty_tail_without_a_head_match() {
    for (spacer, folds_spacer) in [
        (empty("ins", ""), false),
        (empty("ins", "<w:spacing w:before=\"120\"/>"), false),
        (para("ins", " ", ""), true),
    ] {
        let fragment = format!(
            "{}{spacer}{}",
            para(
                "ins",
                "Revised body contains seven entirely distinct words",
                ""
            ),
            para("del", "Base title", "<w:pStyle w:val=\"Title\"/>")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body),
            if folds_spacer {
                old[..2].to_vec()
            } else {
                old.clone()
            }
        );
        expect_segments(
            &dom,
            old[0],
            &[("ins", "Revised body contains seven entirely distinct words")],
        );
        if folds_spacer {
            expect_segments(&dom, old[1], &[("ins", " "), ("del", "Base title")]);
            assert!(para_has_heading_or_title_style(&dom, old[1]));
            assert!(para_mark_revision(&dom, old[1], &W::del()));
        }
    }
    // The significant head match overrides M468, even with paired marks.
    let fragment = format!(
        "{}{}{}{}{}{}",
        para("ins", "TIFF revised heading", ""),
        para("ins", "Second revised paragraph", ""),
        para("ins", "Third revised paragraph", ""),
        para(
            "ins",
            "Final body contains seven entirely distinct words",
            ""
        ),
        empty("ins", ""),
        para(
            "del",
            "TIFF original heading",
            "<w:pStyle w:val=\"Title\"/>"
        )
    );
    let (dom, _, body, old) = merge(&fragment, false);
    assert_eq!(blocks(&dom, body), old[..5]);
    expect_segments(
        &dom,
        old[0],
        &[
            ("ins", "TIFF revised heading"),
            ("del", "TIFF original heading"),
        ],
    );
    assert!(para_has_heading_or_title_style(&dom, old[0]));
    assert!(para_mark_revision(&dom, old[4], &W::ins()));
}

#[test]
fn math_shells_are_fold_carriers_and_plain_layout_empties_are_preserved() {
    for math in [false, true] {
        let shell = if math {
            "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr><w:ins w:author=\"Editor\"><m:oMath><m:r><m:t>x</m:t></m:r></m:oMath></w:ins></w:p>".to_string()
        } else {
            empty("ins", "")
        };
        let fragment = format!(
            "{}{shell}{}{}",
            empty("ins", "").repeat(2),
            para("del", "Original equation title words", ""),
            para("del", "Base continuation", "")
        );
        let (dom, root, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body),
            if math {
                vec![old[0], old[1], old[2], old[4]]
            } else {
                old.clone()
            }
        );
        if math {
            assert_eq!(dom.descendants(root, Some(&M::name("oMath"))).len(), 1);
            assert_eq!(
                dom.value(dom.descendants(root, Some(&M::name("t")))[0]),
                "x"
            );
            assert!(!para_mark_revision(&dom, old[2], &W::ins()));
            assert!(dom.parent(old[3]).is_none());
        }
    }
}

#[test]
fn inserted_cover_tail_and_short_residual_exceptions_are_distinct() {
    for tail in [
        "prepared by reviewer",
        "mail@example.com",
        "year 2040",
        "agreement signed",
        "xy",
    ] {
        let fragment = format!(
            "{}{}{}{}{}",
            para("ins", "Cover", ""),
            para("ins", "Author", ""),
            para("ins", "Subtitle", ""),
            para("ins", tail, ""),
            para("del", "Old clause has five distinct words", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(blocks(&dom, body), old, "cover tail {tail}");
    }
    // Three new paragraphs and a substantial last one keep a one-token
    // old comment target. Two new paragraphs still form the final pair.
    for (n, folds) in [(2, true), (3, false)] {
        let fragment = format!(
            "{}{}{}",
            para("ins", "Earlier revised body", "").repeat(n - 1),
            para("ins", "Substantial revised final paragraph", ""),
            para("del", "Ouch", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(blocks(&dom, body).len(), if folds { n } else { n + 1 });
        assert_eq!(dom.parent(old[n]).is_none(), folds);
    }
}

#[test]
fn deletion_first_short_residual_and_field_carrier_keep_source_properties() {
    for field in [false, true] {
        let deleted = para(
            "del",
            if field { "Original heading" } else { "Ouch" },
            "<w:pStyle w:val=\"Title\"/>",
        );
        let mut inserted = para(
            "ins",
            if field {
                " "
            } else {
                "Substantial revised final paragraph"
            },
            "<w:jc w:val=\"right\"/>",
        );
        if field {
            inserted = inserted.replace("<w:t xml:space=", "<w:fldChar w:fldCharType=\"begin\"/><w:fldChar w:fldCharType=\"end\"/><w:t xml:space=");
        }
        let fragment = format!(
            "{deleted}{}{inserted}",
            para("ins", "Leading revised body", "").repeat(2)
        );
        let (dom, _, body, old) = merge(&fragment, true);
        if field {
            assert_eq!(blocks(&dom, body), old[1..]);
            assert!(!para_has_heading_or_title_style(&dom, old[3]));
            assert_eq!(
                dom.attribute(dom.descendants(old[3], Some(&W::jc_el()))[0], &W::val()),
                Some("right")
            );
            assert!(!para_mark_revision(&dom, old[3], &W::ins()));
            assert_eq!(dom.descendants(old[3], Some(&W::fld_char())).len(), 2);
        } else {
            assert_eq!(blocks(&dom, body), [old[1], old[2], old[3], old[0]]);
            assert!(para_mark_revision(&dom, old[0], &W::del()));
        }
    }
}

#[test]
fn empty_deleted_tail_is_not_consumed_after_long_inserted_streams() {
    for count in [2, 10] {
        let fragment = format!(
            "{}{}",
            para("ins", "New substantive paragraph", "").repeat(count),
            para("del", "", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(blocks(&dom, body), old);
    }
}

#[test]
fn short_label_long_prose_gate_distinguishes_plain_demo_and_ooxml_intros() {
    for (intro, count, folds) in [
        (
            "Original prose has eight entirely unrelated distinct separate words",
            3,
            false,
        ),
        (
            "This document demonstrates eight entirely unrelated distinct separate words",
            3,
            true,
        ),
        (
            "This document demonstrates sample OOXML properties with many distinct terms",
            5,
            false,
        ),
        (
            "This document demonstrates sample ST_OnOff properties with many distinct terms",
            5,
            false,
        ),
        (
            "This document demonstrates sample w:b properties with many distinct terms",
            5,
            false,
        ),
        (
            "This document demonstrates sample w:i properties with many distinct terms",
            5,
            false,
        ),
        (
            "This document demonstrates ordinary properties with many distinct lengthy terms",
            5,
            true,
        ),
    ] {
        let fragment = format!(
            "{}{}{}",
            para("ins", "Apple", "").repeat(count),
            para("del", intro, ""),
            para("del", "Residual", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body).len(),
            count + 2 - usize::from(folds),
            "{intro}"
        );
        assert_eq!(dom.parent(old[count]).is_none(), folds, "{intro}");
        if folds {
            expect_segments(&dom, old[count - 1], &[("ins", "Apple"), ("del", intro)]);
        }
    }
}

#[test]
fn styled_inserted_titles_keep_unrelated_short_base_titles_except_demos() {
    for (old_title, folds) in [
        ("Base title", false),
        ("Base Demo", true),
        ("Base formatting Demo", true),
    ] {
        let fragment = format!(
            "{}{}{}{}",
            para("ins", "Earlier new body", ""),
            para(
                "ins",
                "Revised heading contains words",
                "<w:pStyle w:val=\"Heading2\"/>"
            ),
            para("del", old_title, ""),
            para("del", "Residual", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body).len(),
            4 - usize::from(folds),
            "{old_title}"
        );
        assert_eq!(dom.parent(old[2]).is_none(), folds);
        assert!(para_has_heading_style(&dom, old[1]));
    }
}

#[test]
fn long_demo_stream_gate_has_an_exact_ten_paragraph_boundary() {
    for count in [9, 10] {
        let fragment = format!(
            "{}{}{}",
            para("ins", "Lengthy revised body", "").repeat(count),
            para("del", "Original formatting Demo", ""),
            para("del", "Residual", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        let folds = count == 9;
        assert_eq!(blocks(&dom, body).len(), count + 2 - usize::from(folds));
        assert_eq!(dom.parent(old[count]).is_none(), folds);
    }
}

#[test]
fn heading_skip_is_limited_to_short_multitoken_headings_and_nonlist_carriers() {
    for (style, title, ins_props, folds) in [
        ("Heading1", "Mixed Formatting Test", "", false),
        ("Heading1", "Images", "", true),
        ("Title", "Mixed Formatting Test", "", true),
        (
            "Heading1",
            "Old heading contains seven wholly unrelated distinct words",
            "",
            true,
        ),
        (
            "Heading1",
            "Mixed Formatting Test",
            "<w:pStyle w:val=\"ListNumber\"/>",
            true,
        ),
    ] {
        let dp = format!("<w:pStyle w:val=\"{style}\"/>");
        let fragment = format!(
            "{}{}{}{}{}",
            para("ins", "First revised paragraph", ""),
            para("ins", "Second revised paragraph", ""),
            para("ins", "Substantial final revised paragraph", ins_props),
            para("del", title, &dp),
            para("del", "Residual", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(
            blocks(&dom, body).len(),
            5 - usize::from(folds),
            "{style}/{title}/{ins_props}"
        );
        assert_eq!(dom.parent(old[3]).is_none(), folds);
        if folds {
            assert!(para_has_heading_or_title_style(&dom, old[2]));
            assert!(para_mark_revision(&dom, old[2], &W::del()));
            assert_eq!(
                dom.attribute(dom.descendants(old[2], Some(&W::p_style()))[0], &W::val()),
                Some(style)
            );
        }
    }
}

#[test]
fn list_carriers_override_the_long_prose_list_skip_and_retain_their_geometry() {
    for ins_props in [
        NUMBERED,
        "<w:pStyle w:val=\"ListNumber\"/>",
        "<w:pStyle w:val=\"StandardL1\"/>",
    ] {
        let fragment = format!(
            "{}{}{}",
            para("ins", "Lengthy revised legal clause", ins_props),
            para(
                "del",
                "Original list clause words",
                &format!("<w:pStyle w:val=\"ListParagraph\"/>{NUMBERED}")
            ),
            para("del", "Residual", "")
        );
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(blocks(&dom, body), [old[0], old[2]]);
        // Structural new formatting wins for a long list carrier; M436's
        // short-list adoption exception intentionally does not apply.
        assert!(!para_mark_revision(&dom, old[0], &W::ins()));
        if ins_props == NUMBERED {
            assert!(para_has_live_numpr(&dom, old[0]));
        } else {
            assert_eq!(
                dom.attribute(dom.descendants(old[0], Some(&W::p_style()))[0], &W::val()),
                Some(if ins_props.contains("StandardL1") {
                    "StandardL1"
                } else {
                    "ListNumber"
                })
            );
        }
        expect_segments(
            &dom,
            old[0],
            &[
                ("ins", "Lengthy revised legal clause"),
                ("del", "Original list clause words"),
            ],
        );
    }
}

#[test]
fn wholesale_scale_gate_preserves_large_math_and_drawing_replacements() {
    for payload in [
        "<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>",
        "<w:r><w:drawing/></w:r>",
        "<w:r><w:pict/></w:r>",
        "<w:r><w:object/></w:r>",
    ] {
        let mut fragment = para("ins", "Fresh alpha beta gamma delta", "").repeat(9);
        fragment.push_str(&format!("<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr><w:ins w:author=\"Editor\">{payload}</w:ins></w:p>"));
        fragment.push_str(
            &para(
                "del",
                "Stale red orange purple green blue black white silver gold",
                "",
            )
            .repeat(151),
        );
        // Disjoint vocab is essential: any related prose pair overrides
        // the wholesale gate, even when the final carrier is a payload.
        let (dom, root, body, old) = merge(&fragment, true);
        let payload_name = if payload.starts_with("<m:") {
            M::name("oMath")
        } else if payload.contains("drawing") {
            W::drawing()
        } else if payload.contains("pict") {
            W::pict()
        } else {
            W::object()
        };
        let payloads = dom.descendants(root, Some(&payload_name));
        assert_eq!(
            payloads.len(),
            1,
            "payload survives wholesale gate: {payload}"
        );
        assert_eq!(
            dom.serialize_element(payloads[0]),
            {
                let (expected, expected_root, _) = document(&format!("<w:p>{payload}</w:p>"));
                let expected_payload = expected.descendants(expected_root, Some(&payload_name))[0];
                expected.serialize_element(expected_payload)
            },
            "literal payload is preserved"
        );
        assert_eq!(blocks(&dom, body), old, "large replacement {payload}");
    }
}

#[test]
fn midstory_checklist_cell_guard_distinguishes_titles_at_a_weak_head_junction() {
    // A head junction bypasses the earlier story-end gate, allowing the
    // checklist-cell test to decide whether the old label is a heading.
    for (label, shared, props, folds) in [
        ("Sign NDA", "Sign", "", false),
        ("Sign NDA", "Sign", "<w:pStyle w:val=\"Title\"/>", true),
        ("Quarterly Performance Report", "Quarterly", "", true),
    ] {
        let revised = format!(
            "{shared} fresh changed cover contains many wholly distinct unusual tokens beyond all ordinary former legal text clauses provisions"
        );
        assert!(body_text_jaccard(&revised, label) < SOLE_DEL_FOLD_MIN_JACCARD);
        let mut fragment = para("ins", &revised, "");
        fragment.push_str(&para("ins", "New substantive body paragraph", "").repeat(4));
        fragment.push_str(&para("del", label, props));
        fragment.push_str(&para("del", "Base residual", ""));
        fragment.push_str(&live("Live continuation"));
        let (dom, _, body, old) = merge(&fragment, true);
        assert_eq!(blocks(&dom, body).len(), 8 - usize::from(folds));
        assert_eq!(dom.parent(old[5]).is_none(), folds);
        if folds {
            expect_segments(&dom, old[0], &[("ins", &revised), ("del", label)]);
        }
    }
}

#[test]
fn whitespace_fold_keeps_a_whitespace_only_deleted_source_paragraph() {
    unchanged(
        &format!("{}{}", para("ins", " ", ""), para("del", " \t", "")),
        fold_whitespace_pure_ins_into_following_pure_del,
    );
}

#[test]
fn whitespace_mark_shell_variants_keep_existing_format_without_inventing_a_mark() {
    for (carrier, deleted, expected_mark, expected_rpr) in [
        ("<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>", "", false, false),
        (
            "<w:pPr><w:rPr><w:b/><w:ins/></w:rPr></w:pPr>",
            "",
            false,
            true,
        ),
        (
            "<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>",
            "<w:pPr/>",
            false,
            false,
        ),
        (
            "<w:pPr/>",
            "<w:pPr><w:rPr><w:b/></w:rPr></w:pPr>",
            false,
            false,
        ),
        ("", "<w:pPr><w:rPr><w:b/></w:rPr></w:pPr>", false, false),
    ] {
        let fragment = format!(
            "<w:p>{carrier}{}</w:p><w:p>{deleted}{}</w:p>",
            revision("ins", " "),
            revision("del", "Old substantive clause has five words")
        );
        let (mut dom, root, body) = document(&fragment);
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        let revisions = body_revisions(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(blocks(&dom, body), [old[0]]);
        assert_eq!(views(&dom, root), text);
        assert_eq!(body_revisions(&dom, root), revisions);
        assert_eq!(para_mark_revision(&dom, old[0], &W::del()), expected_mark);
        assert!(!para_mark_revision(&dom, old[0], &W::ins()));
        assert_eq!(
            dom.element(old[0], &W::p_pr())
                .and_then(|p| dom.element(p, &W::r_pr()))
                .is_some(),
            expected_rpr
        );
    }
}

#[test]
fn whitespace_spacing_guard_searches_past_noninserted_predecessors() {
    for (prior, folds) in [
        (live("Equal predecessor"), false),
        (para("ins", "Earlier revised text", ""), true),
    ] {
        let fragment = format!(
            "{prior}{}{}",
            para("ins", " ", "<w:spacing w:line=\"240\"/>"),
            para("del", "Old substantive clause has five words", "").repeat(3)
        );
        let (mut dom, root, body) = document(&fragment);
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(blocks(&dom, body).len(), 5 - usize::from(folds));
        assert_eq!(views(&dom, root), text);
        assert_eq!(dom.parent(old[2]).is_none(), folds);
        assert_eq!(
            dom.attribute(
                dom.descendants(old[1], Some(&W::spacing_el()))[0],
                &W::name("line")
            ),
            Some("240")
        );
    }
}

fn spacer_context(dom: &mut Dom, fragment: &str) -> HashMap<String, [NodeId; 2]> {
    let source = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{fragment}</w:body></w:document>"
        ));
    let root = dom.root(source).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    revised_two_blank_table_titles(dom, body)
}

#[test]
fn table_title_spacers_stamp_only_new_marks_and_keep_existing_empty_properties() {
    let settings = WmlComparerSettings {
        author_for_revisions: "Spacer author".into(),
        date_time_for_revisions: DATE.into(),
        ..WmlComparerSettings::default()
    };
    for (blank, existing_mark) in [
        ("<w:p/>", false),
        (
            "<w:p><w:pPr><w:spacing w:after=\"120\"/></w:pPr></w:p>",
            false,
        ),
        (
            "<w:p><w:pPr><w:rPr><w:b/></w:rPr></w:pPr><w:r/></w:p>",
            false,
        ),
        (
            "<w:p><w:pPr><w:rPr><w:del w:id=\"88\"/></w:rPr></w:pPr></w:p>",
            true,
        ),
        (
            "<w:p><w:pPr><w:rPr><w:ins w:id=\"89\"/></w:rPr></w:pPr></w:p>",
            true,
        ),
    ] {
        let fragment = format!(
            "{}{}{blank}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl><w:sectPr/>",
            para("ins", "New body", ""),
            para("del", "Contract Review", ""),
            live("Live cell")
        );
        let (mut dom, root, body) = document(&fragment);
        let revised_titles = spacer_context(
            &mut dom,
            &format!("{}<w:p/><w:p/><w:tbl/>", live("New body")),
        );
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        let table = dom.serialize_element(old[3]);
        let blank_props = properties(&dom, old[2]);
        let before = dom.serialize_element(root);
        let mut id = 100;
        ensure_empty_pure_i_before_short_title_del(
            &mut dom,
            root,
            &revised_titles,
            &settings,
            &mut id,
        );
        // This fixture previously asserted two fabricated spacers even
        // when an inserted separator already survived acceptance.
        if !dom.descendants(old[2], Some(&W::ins())).is_empty() {
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(blocks(&dom, body), old);
            assert_eq!(id, 100);
            continue;
        }
        let after = blocks(&dom, body);
        assert_eq!(after.len(), 6);
        assert_eq!(
            [after[0], after[3], after[4], after[5]],
            [old[0], old[1], old[2], old[3]]
        );
        assert_eq!(views(&dom, root), text);
        assert_eq!(dom.serialize_element(old[3]), table);
        for (offset, &p) in after[1..3].iter().enumerate() {
            let mark = dom.descendants(p, Some(&W::ins()))[0];
            assert_eq!(
                dom.attribute(mark, &W::id()),
                Some((100 + offset).to_string().as_str())
            );
            assert_eq!(dom.attribute(mark, &W::author()), Some("Spacer author"));
            assert_eq!(dom.attribute(mark, &W::date()), Some(DATE));
            assert!(dom.elements(p, Some(&W::r())).is_empty());
        }
        assert_eq!(id, if existing_mark { 102 } else { 103 });
        if existing_mark {
            assert_eq!(properties(&dom, old[2]), blank_props);
        } else {
            assert!(para_mark_revision(&dom, old[2], &W::del()));
            let mark = dom.descendants(old[2], Some(&W::del()))[0];
            assert_eq!(dom.attribute(mark, &W::id()), Some("102"));
            assert_eq!(dom.attribute(mark, &W::author()), Some("Spacer author"));
            assert_eq!(dom.attribute(mark, &W::date()), Some(DATE));
        }
        let once = dom.serialize_element(root);
        ensure_empty_pure_i_before_short_title_del(
            &mut dom,
            root,
            &revised_titles,
            &settings,
            &mut id,
        );
        assert_eq!(dom.serialize_element(root), once);
    }
}

#[test]
fn title_spacers_require_unique_revised_two_blank_pretable_context() {
    let settings = WmlComparerSettings::default();
    for (blanks, duplicate, title_matches, restore) in [
        (2, false, true, true),
        (1, false, true, false),
        (3, false, true, false),
        (2, true, true, false),
        (2, false, false, false),
    ] {
        let source_title = if title_matches {
            "HR Onboarding Checklist"
        } else {
            "Other revised title"
        };
        let repeated = if duplicate {
            live(source_title)
        } else {
            String::new()
        };
        let source_fragment = format!(
            "<w:p/>{repeated}{}{}<w:tbl/><w:p/>",
            live(source_title),
            "<w:p/>".repeat(blanks)
        );

        let (mut dom, root, body) = document(&format!(
            "{}{}<w:p/><w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl><w:p/>",
            para("ins", "HR Onboarding Checklist", ""),
            para("del", "Contract Review", ""),
            live("Live cell")
        ));
        let revised_titles = spacer_context(&mut dom, &source_fragment);
        assert_eq!(
            revised_titles.contains_key(source_title),
            blanks == 2 && !duplicate
        );
        let old = blocks(&dom, body);
        let before = dom.serialize_element(root);
        let mut id = 100;
        ensure_empty_pure_i_before_short_title_del(
            &mut dom,
            root,
            &revised_titles,
            &settings,
            &mut id,
        );
        if !restore {
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(blocks(&dom, body), old);
            assert_eq!(id, 100);
            continue;
        }
        assert_eq!(blocks(&dom, body).len(), old.len() + 2);
        for (accept, expected) in [
            (
                true,
                vec![
                    "p:HR Onboarding Checklist",
                    "p:",
                    "p:",
                    "tbl:Live cell",
                    "p:",
                ],
            ),
            (
                false,
                vec!["p:Contract Review", "p:", "tbl:Live cell", "p:"],
            ),
        ] {
            let copy = dom.clone_subtree(root);
            let copy = if accept {
                crate::revision_processor::accept_revisions_document(&mut dom, copy)
            } else {
                crate::revision_processor::reject_revisions_document(&mut dom, copy)
            };
            let copy_body = dom.element(copy, &W::body()).unwrap();
            let layout: Vec<String> = blocks(&dom, copy_body)
                .into_iter()
                .map(|p| {
                    format!(
                        "{}:{}",
                        if dom.name_is(p, &W::p()) { "p" } else { "tbl" },
                        projection(&dom, p, accept)
                    )
                })
                .collect();
            assert_eq!(layout, expected, "accept={accept}");
        }
    }
}

#[test]
fn restored_title_spacers_preserve_each_revised_blank_template() {
    let first = "<w:p><w:pPr><w:pStyle w:val=\"SpacerOne\"/><w:spacing w:before=\"120\" w:after=\"80\"/><w:ind w:left=\"240\"/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:rPr><w:i/></w:rPr></w:r></w:p>";
    let second = "<w:p><w:pPr><w:pStyle w:val=\"SpacerTwo\"/><w:spacing w:before=\"300\" w:after=\"40\"/><w:ind w:right=\"360\"/><w:rPr><w:color w:val=\"123456\"/></w:rPr></w:pPr></w:p>";
    let (mut dom, root, body) = document(&format!(
        "{}{}<w:p/><w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
        para("ins", "HR Onboarding Checklist", ""),
        para("del", "Contract Review", ""),
        live("Cell")
    ));
    let context = spacer_context(
        &mut dom,
        &format!("{}{first}{second}<w:tbl/>", live("HR Onboarding Checklist")),
    );
    let templates = context["HR Onboarding Checklist"];
    let mut expected = Vec::new();
    for template in templates {
        let copy = dom.clone_subtree(template);
        let copy = crate::revision_processor::accept_revisions_document(&mut dom, copy);
        expected.push(dom.serialize_element(copy));
        dom.set_attribute_value(template, &W14::name("paraId"), Some("01020304"));
        dom.set_attribute_value(template, &W14::name("textId"), Some("05060708"));
        dom.set_attribute_value(template, &PT::unid(), Some("source-identity"));
    }
    let mut id = 100;
    ensure_empty_pure_i_before_short_title_del(
        &mut dom,
        root,
        &context,
        &WmlComparerSettings::default(),
        &mut id,
    );
    let restored = blocks(&dom, body)[1..3].to_vec();
    for (i, p) in restored.into_iter().enumerate() {
        assert_eq!(dom.attribute(p, &W14::name("paraId")), None);
        assert_eq!(dom.attribute(p, &W14::name("textId")), None);
        assert_eq!(dom.attribute(p, &PT::unid()), None);
        assert!(para_mark_revision(&dom, p, &W::ins()));
        let copy = dom.clone_subtree(p);
        let copy = crate::revision_processor::accept_revisions_document(&mut dom, copy);
        assert_eq!(dom.serialize_element(copy), expected[i], "blank {i}");
    }
}

#[test]
fn title_spacer_context_declines_owned_nonlayout_payload() {
    for payload in [
        "<w:bookmarkStart w:id=\"1\" w:name=\"kept\"/><w:bookmarkEnd w:id=\"1\"/>",
        "<w:commentRangeStart w:id=\"2\"/><w:commentRangeEnd w:id=\"2\"/>",
        "<w:fldSimple w:instr=\"PAGE\"/>",
        "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>",
        "<w:r><w:instrText>PAGE</w:instrText></w:r>",
        "<w:r><w:tab/></w:r>",
        "<w:r><w:br/></w:r>",
        "<w:r><w:footnoteReference w:id=\"3\"/></w:r>",
        "<w:r><w:endnoteReference w:id=\"4\"/></w:r>",
        "<w:r><w:drawing/></w:r>",
        "<w:sdt><w:sdtContent/></w:sdt>",
        "<w:pPr><w:sectPr/></w:pPr>",
    ] {
        let (mut dom, root, body) = document(&format!(
            "{}{}<w:p/><w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
            para("ins", "HR Onboarding Checklist", ""),
            para("del", "Contract Review", ""),
            live("Cell")
        ));
        let context = spacer_context(
            &mut dom,
            &format!(
                "{}<w:p>{payload}</w:p><w:p/><w:tbl/>",
                live("HR Onboarding Checklist")
            ),
        );
        assert!(context.is_empty(), "payload {payload}");
        let old = blocks(&dom, body);
        let before = dom.serialize_element(root);
        let mut id = 100;
        ensure_empty_pure_i_before_short_title_del(
            &mut dom,
            root,
            &context,
            &WmlComparerSettings::default(),
            &mut id,
        );
        assert_eq!(dom.serialize_element(root), before, "payload {payload}");
        assert_eq!(blocks(&dom, body), old);
        assert_eq!(id, 100);
    }
}

#[test]
fn title_spacer_injection_requires_a_paired_table_and_a_truly_blank_separator() {
    let settings = WmlComparerSettings {
        author_for_revisions: "Editor".into(),
        date_time_for_revisions: DATE.into(),
        ..WmlComparerSettings::default()
    };
    for (title, blank, cell) in [
        ("Contract Review", "<w:p/>", para("del", "Deleted cell", "")),
        ("Contract Review", "<w:p/>", String::new()),
        (
            "Contract Review",
            "<w:p><w:r><w:t>Live separator</w:t></w:r></w:p>",
            live("Cell"),
        ),
        ("One two three four five", "<w:p/>", live("Cell")),
    ] {
        let (mut dom, root, _) = document(&format!(
            "{}{}{blank}<w:tbl><w:tr><w:tc>{cell}</w:tc></w:tr></w:tbl>",
            para("ins", "New body", ""),
            para("del", title, "")
        ));
        let revised_titles = spacer_context(
            &mut dom,
            &format!("{}<w:p/><w:p/><w:tbl/>", live("New body")),
        );
        let before = dom.serialize_element(root);
        let mut id = 100;
        ensure_empty_pure_i_before_short_title_del(
            &mut dom,
            root,
            &revised_titles,
            &settings,
            &mut id,
        );
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 100);
    }
}

#[test]
fn three_paragraph_rotation_trades_full_properties_and_preserves_each_revision() {
    let p0 = format!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/><w:rPr><w:del w:id=\"71\"/></w:rPr></w:pPr>{}{}{}{}</w:p>",
        revision("ins", "New first "),
        revision("ins", "title"),
        revision("del", "Old first "),
        revision("del", "title")
    );
    let p1 = format!(
        "<w:p><w:pPr><w:spacing w:before=\"240\"/><w:rPr><w:ins w:id=\"72\"/></w:rPr></w:pPr>{}{}</w:p>",
        revision("del", "Old second "),
        revision("del", "body")
    );
    let p2 = format!(
        "<w:p><w:pPr><w:ind w:left=\"360\"/></w:pPr>{}{}<w:r><w:t> equal </w:t></w:r>{}</w:p>",
        revision("ins", "New second "),
        revision("ins", "body"),
        revision("ins", "new tail")
    );
    let (mut dom, root, body) = document(&format!("{p0}{p1}{p2}<w:sectPr/>"));
    let old = blocks(&dom, body);
    let props = old.iter().map(|&p| properties(&dom, p)).collect::<Vec<_>>();
    let text = views(&dom, root);
    let revisions = body_revisions(&dom, root);
    rotate_ins_mark_del_only_paragraph(&mut dom, root);
    assert_eq!(blocks(&dom, body), old);
    assert_eq!(views(&dom, root), text);
    assert_eq!(body_revisions(&dom, root), revisions);
    assert_eq!(properties(&dom, old[0]), props[1]);
    assert_eq!(properties(&dom, old[1]), props[0]);
    assert_eq!(properties(&dom, old[2]), props[2]);
    expect_segments(&dom, old[0], &[("ins", "New first "), ("ins", "title")]);
    expect_segments(
        &dom,
        old[1],
        &[
            ("ins", "New second "),
            ("ins", "body"),
            ("del", "Old first "),
            ("del", "title"),
        ],
    );
    expect_segments(
        &dom,
        old[2],
        &[
            ("del", "Old second "),
            ("del", "body"),
            ("r", " equal "),
            ("ins", "new tail"),
        ],
    );
    let once = dom.serialize_element(root);
    rotate_ins_mark_del_only_paragraph(&mut dom, root);
    assert_eq!(dom.serialize_element(root), once);
}

#[test]
fn rotation_rejects_interleaved_first_runs_and_nonfinal_middle_deletions() {
    let good0 = format!(
        "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr>{}{}</w:p>",
        revision("ins", "new"),
        revision("del", "old")
    );
    let good1 = format!(
        "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>{}</w:p>",
        revision("del", "old middle")
    );
    let good2 = format!(
        "<w:p>{}<w:r><w:t>equal</w:t></w:r></w:p>",
        revision("ins", "new middle")
    );
    let bad0 = [
        format!(
            "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr>{}{}{}</w:p>",
            revision("ins", "new"),
            revision("del", "old"),
            revision("ins", "later")
        ),
        good0.replace("</w:p>", "<w:r><w:t>live</w:t></w:r></w:p>"),
    ];
    for p in bad0 {
        unchanged(
            &format!("{p}{good1}{good2}"),
            rotate_ins_mark_del_only_paragraph,
        );
    }
    for middle in [
        good1.replace("</w:p>", "<w:r><w:t>live</w:t></w:r></w:p>"),
        empty("ins", ""),
        good1.replace("<w:ins/>", "<w:del/>"),
    ] {
        unchanged(
            &format!("{good0}{middle}{good2}"),
            rotate_ins_mark_del_only_paragraph,
        );
    }
    for tail in [
        para("ins", "new middle", ""),
        format!("<w:p>{}</w:p>", revision("ins", "new middle")),
        live("equal"),
    ] {
        unchanged(
            &format!("{good0}{good1}{tail}"),
            rotate_ins_mark_del_only_paragraph,
        );
    }
}

#[test]
fn leading_subtitle_fold_moves_multiple_wrappers_and_retains_deleted_layout() {
    for props in [
        "",
        "<w:pPr/>",
        "<w:pPr><w:rPr/></w:pPr>",
        "<w:pPr><w:jc w:val=\"center\"/><w:rPr><w:del w:id=\"44\"/></w:rPr></w:pPr>",
    ] {
        let d = format!("<w:p>{props}{}</w:p>", revision("del", "Original title"));
        let m = format!(
            "<w:p><w:pPr><w:spacing w:line=\"360\"/></w:pPr>{}{}{}<w:r><w:t> equal suffix</w:t></w:r></w:p>",
            revision("ins", "Fresh "),
            revision("ins", "subtitle"),
            revision("del", "Old body")
        );
        let (mut dom, root, body) = document(&format!("{d}{m}"));
        let old = blocks(&dom, body);
        let pp = old.iter().map(|&p| properties(&dom, p)).collect::<Vec<_>>();
        let text = views(&dom, root);
        let revisions = body_revisions(&dom, root);
        fold_leading_ins_from_mix_into_preceding_pure_del(&mut dom, root);
        assert_eq!(blocks(&dom, body), old);
        assert_eq!(views(&dom, root), text);
        assert_eq!(body_revisions(&dom, root), revisions);
        assert_eq!(properties(&dom, old[0]), pp[0]);
        assert_eq!(properties(&dom, old[1]), pp[1]);
        expect_segments(
            &dom,
            old[0],
            &[
                ("ins", "Fresh "),
                ("ins", "subtitle"),
                ("del", "Original title"),
            ],
        );
        expect_segments(&dom, old[1], &[("del", "Old body"), ("r", " equal suffix")]);
    }
}

#[test]
fn leading_subtitle_gate_requires_disjoint_significant_tokens_and_eight_token_cap() {
    for (title, ins, folds) in [
        (
            "one two three four five six seven eight",
            "Fresh subtitle",
            true,
        ),
        (
            "one two three four five six seven eight nine",
            "Fresh subtitle",
            false,
        ),
        ("Original title", "Original subtitle", false),
        ("Original title", " \t", false),
    ] {
        let fragment = format!(
            "{}<w:p>{}{}</w:p>",
            para("del", title, ""),
            revision("ins", ins),
            revision("del", "Old body")
        );
        let (mut dom, root, body) = document(&fragment);
        let old = blocks(&dom, body);
        let before = dom.serialize_element(root);
        let text = views(&dom, root);
        fold_leading_ins_from_mix_into_preceding_pure_del(&mut dom, root);
        assert_eq!(views(&dom, root), text);
        assert_eq!(
            dom.elements(old[0], Some(&W::ins())).len(),
            usize::from(folds)
        );
        if !folds {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn shared_title_mesh_executes_wholesale_and_confetti_rebuilds_with_exact_views() {
    for (split, attribution, suffix) in [
        (false, true, " with examples"),
        (true, true, ""),
        (true, false, " with examples"),
    ] {
        let mut i = if split {
            format!(
                "{}{}",
                revision("ins", "New "),
                revision("ins", &format!("document{suffix}"))
            )
        } else {
            revision("ins", &format!("New document{suffix}"))
        };
        let mut d = if split {
            format!("{}{}", revision("del", "Old "), revision("del", "document"))
        } else {
            revision("del", "Old document")
        };
        if !attribution {
            for attr in [
                " w:author=\"Editor\"".to_string(),
                format!(" w:date=\"{DATE}\""),
                " w:id=\"17\"".into(),
            ] {
                i = i.replace(&attr, "");
                d = d.replace(&attr, "");
            }
        }
        let (mut dom, root, body) = document(&format!(
            "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>{i}{d}</w:p>"
        ));
        let p = blocks(&dom, body)[0];
        let props = properties(&dom, p);
        let text = views(&dom, root);
        free_mesh_shared_title_token_in_mix(&mut dom, root);
        assert_eq!(views(&dom, root), text);
        assert_eq!(properties(&dom, p), props);
        assert_eq!(projection(&dom, p, true), format!("New document{suffix}"));
        assert_eq!(projection(&dom, p, false), "Old document");
        let body = dom.elements(p, None);
        assert_eq!(dom.name(body[3]), Some(W::r()));
        assert_eq!(
            dom.value(dom.element(body[3], &W::t()).unwrap()),
            " document"
        );
        let revisions: Vec<_> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del()))
            .collect();
        assert_eq!(revisions.len(), if suffix.is_empty() { 2 } else { 3 });
        for (index, &rev) in revisions.iter().enumerate() {
            assert_eq!(
                dom.attribute(rev, &W::id()),
                Some((index + 1).to_string().as_str())
            );
            assert_eq!(
                dom.attribute(rev, &W::author()),
                Some(if attribution { "Editor" } else { "Redline" })
            );
            assert_eq!(
                dom.attribute(rev, &W::date()),
                Some(if attribution {
                    DATE
                } else {
                    crate::document_comparer::DEFAULT_DATE
                })
            );
        }
    }
}

#[test]
fn bookended_mesh_handles_boundary_anchors_and_the_forty_token_limit() {
    for (ins, del) in [
        ("right fresh aligned sentence", "right old justified prose"),
        ("fresh aligned sentence right", "old justified prose right"),
        ("fresh right aligned sentence", "old justified prose right"),
        ("fresh right aligned sentence", "right old justified prose"),
    ] {
        let (mut dom, root, body) = document(&format!(
            "<w:p><w:r><w:t>Prefix </w:t></w:r>{}{}<w:r><w:t> suffix</w:t></w:r></w:p>",
            revision("ins", ins),
            revision("del", del)
        ));
        let p = blocks(&dom, body)[0];
        let text = views(&dom, root);
        free_mesh_bookended_ins_del(&mut dom, root);
        assert_eq!(views(&dom, root), text);
        assert_eq!(dom.elements(p, Some(&W::r())).len(), 3);
        assert_eq!(
            dom.value(dom.descendants(dom.elements(p, Some(&W::r()))[1], Some(&W::t()))[0]),
            "right"
        );
        assert_eq!(projection(&dom, p, true), format!("Prefix {ins} suffix"));
        assert_eq!(projection(&dom, p, false), format!("Prefix {del} suffix"));
    }
    for (ins_count, del_count) in [(40, 40), (41, 40), (40, 41)] {
        let ins = format!("{} right", vec!["fresh"; ins_count - 1].join(" "));
        let del = format!("{} right", vec!["old"; del_count - 1].join(" "));
        let (mut dom, root, body) = document(&format!(
            "<w:p><w:r><w:t>Prefix </w:t></w:r>{}{}</w:p>",
            revision("ins", &ins),
            revision("del", &del)
        ));
        let p = blocks(&dom, body)[0];
        let before = dom.serialize_element(root);
        let text = views(&dom, root);
        free_mesh_bookended_ins_del(&mut dom, root);
        assert_eq!(views(&dom, root), text);
        assert_eq!(
            dom.elements(p, Some(&W::r())).len(),
            if ins_count == 40 && del_count == 40 {
                2
            } else {
                1
            }
        );
        if ins_count == 41 || del_count == 41 {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn connector_peel_preserves_both_sources_and_falls_back_to_inserted_run_format() {
    // A single trailing space on each source side is the connector's
    // actual separator. Assert literal views so trimming cannot hide loss.
    for bare in ["", "<w:r><w:t>equal</w:t></w:r>"] {
        let i = format!(
            "<w:ins w:id=\"77\" w:author=\"Editor\" w:date=\"{DATE}\"><w:r><w:rPr><w:b/></w:rPr><w:t>new </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>for next </w:t></w:r></w:ins>"
        );
        let d = format!(
            "<w:del w:id=\"78\" w:author=\"Editor\" w:date=\"{DATE}\"><w:r><w:delText>previous </w:delText></w:r><w:r><w:delText>for </w:delText></w:r></w:del>"
        );
        let (mut dom, root, body) = document(&format!(
            "<w:p>{}{i}</w:p><w:p>{d}{bare}{}</w:p>",
            revision("del", "old"),
            revision("ins", "suffix")
        ));
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        peel_trailing_for_word_onto_next_mix(&mut dom, root);
        assert_eq!(views(&dom, root), text);
        assert_eq!(
            dom.attribute(dom.element(old[0], &W::ins()).unwrap(), &W::id()),
            Some("77")
        );
        assert_eq!(
            dom.attribute(dom.element(old[1], &W::del()).unwrap(), &W::id()),
            Some("78")
        );
        assert_eq!(
            dom.elements(dom.element(old[0], &W::ins()).unwrap(), Some(&W::r()))
                .len(),
            1
        );
        assert_eq!(
            dom.elements(dom.element(old[1], &W::del()).unwrap(), Some(&W::r()))
                .len(),
            1
        );
        let new = dom.elements(old[1], Some(&W::ins()))[0];
        assert_eq!(dom.attribute(new, &W::author()), Some("Editor"));
        assert_eq!(dom.attribute(new, &W::date()), Some(DATE));
        assert_eq!(markup_text(&dom, new), "next ");
        assert_eq!(dom.descendants(new, Some(&W::name("b"))).len(), 1);
        let connector = dom.elements(old[1], Some(&W::r()))[0];
        assert_eq!(markup_text(&dom, connector), " for ");
        assert_eq!(dom.descendants(connector, Some(&W::name("b"))).len(), 1);
        assert_eq!(
            dom.attribute(
                dom.element(connector, &W::t()).unwrap(),
                &XNamespace::xml().name("space")
            ),
            Some("preserve")
        );
    }
}

fn splice_fragment(anchor: &str, inserted: usize, count: usize, del_texts: &[&str]) -> String {
    let mut s = String::new();
    for i in 0..count {
        let text = if i == 4 { anchor } else { "New body" };
        s.push_str(&if i < inserted {
            para("ins", text, "")
        } else {
            live(text)
        });
    }
    for text in del_texts {
        s.push_str(&para("del", text, ""));
    }
    s.push_str("<w:sectPr/>");
    s
}

#[test]
fn short_residual_splice_prefers_tip_then_toc_and_moves_only_the_short_numbered_title() {
    for (anchor, dels, moved) in [
        (
            "TIP: comparison",
            vec!["Old title", "Second base", "Third base"],
            3,
        ),
        ("Table of Contents", vec!["Old title", "Second base"], 2),
        (
            " 1. First section",
            vec!["Old", "Substantially longer original body paragraph"],
            1,
        ),
        ("1 First section", vec!["Old"], 1),
    ] {
        let (mut dom, root, body) = document(&splice_fragment(anchor, 40, 40, &dels));
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        let revisions = body_revisions(&dom, root);
        splice_trailing_short_pure_dels_midstream(&mut dom, root);
        let expected: Vec<_> = old[..5]
            .iter()
            .chain(&old[40..40 + moved])
            .chain(&old[5..40])
            .chain(&old[40 + moved..])
            .copied()
            .collect();
        assert_eq!(blocks(&dom, body), expected, "{anchor}");
        assert_eq!(views(&dom, root), text);
        assert_eq!(body_revisions(&dom, root), revisions);
        assert_eq!(dom.elements(body, Some(&W::sect_pr())).len(), 1);
        let once = dom.serialize_element(root);
        splice_trailing_short_pure_dels_midstream(&mut dom, root);
        assert_eq!(dom.serialize_element(root), once);
    }
}

#[test]
fn short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted() {
    fn projected(dom: &mut Dom, root: NodeId, accepting: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accepting {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    for (marks, second_marks, nested_mark, moved) in [
        ("<w:ins/>", "<w:ins/>", "", true),
        ("", "<w:ins/>", "<w:ins/>", false),
        ("<w:ins/>", "", "<w:ins/>", false),
        ("<w:ins/><w:del/>", "<w:ins/>", "", false),
        ("<w:del/>", "<w:ins/>", "", false),
    ] {
        let mut fragment = String::new();
        for index in 0..40 {
            if index == 10 {
                fragment.push_str(&format!(
                        "<w:tbl><w:tr><w:trPr>{marks}</w:trPr><w:tc><w:p><w:r><w:tab/></w:r></w:p><w:tbl><w:tr><w:trPr>{nested_mark}</w:trPr><w:tc><w:p><w:r><w:t>Nested owned payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr><w:tr><w:trPr>{second_marks}</w:trPr><w:tc>{}</w:tc></w:tr></w:tbl>",
                        para("ins", "New cell", "")
                    ));
            } else {
                fragment.push_str(&para(
                    "ins",
                    if index == 4 {
                        "1. First section"
                    } else {
                        "New body"
                    },
                    "",
                ));
            }
        }
        fragment.push_str(&para("del", "Old Demo", ""));
        fragment.push_str(&para(
            "del",
            "Substantially longer original body paragraph",
            "",
        ));
        let (mut dom, root, body) = document(&fragment);
        let before = dom.serialize_element(root);
        let old = blocks(&dom, body);
        let accepted = projected(&mut dom, root, true);
        let rejected = projected(&mut dom, root, false);
        splice_trailing_short_pure_dels_midstream(&mut dom, root);
        if moved {
            assert_eq!(blocks(&dom, body)[5], old[40]);
            let rejected_copy = dom.clone_subtree(root);
            let rejected_copy =
                crate::revision_processor::reject_revisions_document(&mut dom, rejected_copy);
            assert!(dom.descendants(rejected_copy, Some(&W::tbl())).is_empty());
            assert!(
                dom.descendants(rejected_copy, Some(&W::name("tab")))
                    .is_empty()
            );
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
        assert_eq!(projected(&mut dom, root, true), accepted);
        assert_eq!(projected(&mut dom, root, false), rejected);
    }
}

#[test]
fn short_residual_splice_majority_and_length_boundaries_do_not_invent_reordering() {
    for (inserted, count, title, tail) in [
        (19, 40, "Old", "Much longer original deleted body paragraph"),
        (40, 39, "Old", "Much longer original deleted body paragraph"),
        (40, 40, "", "Much longer original deleted body paragraph"),
        (40, 40, "Old", "Short"),
        (
            40,
            40,
            "12345678901234567890123456789012345678901",
            "Much longer original deleted body paragraph",
        ),
    ] {
        unchanged(
            &splice_fragment("1. First section", inserted, count, &[title, tail]),
            splice_trailing_short_pure_dels_midstream,
        );
    }
    // A majority alone cannot authorize crossing unrevised source text:
    // the original order is twenty live paragraphs, then "Old".
    unchanged(
        &splice_fragment("TIP: body", 20, 40, &["Old"]),
        splice_trailing_short_pure_dels_midstream,
    );
    // Exactly half inserted is still sufficient when ALL crossed blocks
    // are insertions. Put the original live prefix before the TIP anchor.
    let mut fragment = live("Earlier original text").repeat(20);
    for i in 20..40 {
        fragment.push_str(&para(
            "ins",
            if i == 24 { "TIP: body" } else { "New body" },
            "",
        ));
    }
    fragment.push_str(&para("del", "Old", ""));
    let (mut dom, root, body) = document(&fragment);
    let old = blocks(&dom, body);
    let text = views(&dom, root);
    splice_trailing_short_pure_dels_midstream(&mut dom, root);
    assert_eq!(
        blocks(&dom, body)[25],
        old[40],
        "exactly half inserted is sufficient across a pure-insertion gap"
    );
    assert_eq!(views(&dom, root), text);
    let once = dom.serialize_element(root);
    splice_trailing_short_pure_dels_midstream(&mut dom, root);
    assert_eq!(dom.serialize_element(root), once);
}

#[test]
fn revised_internal_hyperlink_fields_preserve_result_text_and_revision_attribution() {
    for kind in ["ins", "del"] {
        for outer in [false, true] {
            let result = if kind == "del" { "delText" } else { "t" };
            let hl = if outer {
                format!(
                    "<w:{kind} w:id=\"55\" w:author=\"Editor\" w:date=\"{DATE}\"><w:hyperlink w:anchor=\"target\" w:tooltip=\"Jump there\"><w:r><w:{result}>Link result</w:{result}></w:r></w:hyperlink></w:{kind}>"
                )
            } else {
                format!(
                    "<w:hyperlink w:anchor=\"target\" w:tooltip=\"Jump there\">{}{}</w:hyperlink>",
                    revision(kind, "Link "),
                    revision(kind, "result")
                )
            };
            let (mut dom, root, _) = document(&format!("<w:p>{hl}</w:p>"));
            let text = views(&dom, root);
            let link = dom.descendants(root, Some(&W::hyperlink()))[0];
            wrap_revised_hyperlink_as_field(&mut dom, link);
            assert_eq!(views(&dom, root), text);
            let fields = dom.descendants(root, Some(&W::fld_char()));
            assert_eq!(
                fields
                    .iter()
                    .map(|&n| dom.attribute(n, &W::name("fldCharType")).unwrap())
                    .collect::<Vec<_>>(),
                ["begin", "separate", "end"]
            );
            let instr = dom.descendants(
                root,
                Some(&W::name(if kind == "del" {
                    "delInstrText"
                } else {
                    "instrText"
                })),
            )[0];
            assert_eq!(
                dom.value(instr),
                "HYPERLINK \\l \"target\" \\o \"Jump there\""
            );
            for rev in dom.descendants(root, Some(&W::name(kind))) {
                assert_eq!(dom.attribute(rev, &W::author()), Some("Editor"));
                assert_eq!(dom.attribute(rev, &W::date()), Some(DATE));
            }
            let field_parent = dom.parent(dom.parent(fields[0]).unwrap()).unwrap();
            assert!(dom.name_is(field_parent, &W::name(kind)));
            if !outer {
                let revs = dom.elements(link, Some(&W::name(kind)));
                assert_eq!(dom.parent(dom.parent(fields[0]).unwrap()), Some(revs[0]));
                assert_eq!(dom.parent(dom.parent(fields[2]).unwrap()), Some(revs[1]));
            }
        }
    }
}

#[test]
fn deletion_peel_keeps_retained_source_boundary_and_inserted_continuations() {
    fn paragraphs(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<String> {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        let body = dom.element(projected, &W::body()).unwrap();
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
    for inserted_boundary in [false, true] {
        let mark = if inserted_boundary {
            "<w:pPr><w:rPr><w:ins w:id=\"31\"/></w:rPr></w:pPr>"
        } else {
            "<w:pPr><w:spacing w:before=\"120\"/></w:pPr>"
        };
        let fragment = format!(
            "<w:p>{mark}<w:r><w:t>Retained source anchor</w:t></w:r>{}{}</w:p><w:p>{}</w:p>",
            revision("ins", " revised"),
            revision(
                "del",
                " within sixty days after receiving the signed request"
            ),
            revision(
                "ins",
                "within thirty days after receiving the written request"
            ),
        );
        let (mut dom, root, body) = document(&fragment);
        let expected_accept = paragraphs(&mut dom, root, true);
        let expected_reject = paragraphs(&mut dom, root, false);
        assert_eq!(
            expected_accept,
            vec![
                "Retained source anchor revised".to_string(),
                "within thirty days after receiving the written request".to_string()
            ]
        );
        assert_eq!(
            expected_reject[0],
            "Retained source anchor within sixty days after receiving the signed request"
        );
        let before = dom.serialize_element(root);
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(paragraphs(&mut dom, root, true), expected_accept);
        assert_eq!(paragraphs(&mut dom, root, false), expected_reject);
        if inserted_boundary {
            let ps = dom.elements(body, Some(&W::p()));
            assert!(dom.elements(ps[0], Some(&W::del())).is_empty());
            assert_eq!(dom.elements(ps[1], Some(&W::del())).len(), 1);
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload() {
    fn projected(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<(String, String)> {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        let body = dom.element(copy, &W::body()).unwrap();
        dom.elements(body, Some(&W::p()))
            .into_iter()
            .map(|p| {
                let text = dom
                    .descendants(p, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect();
                let props = dom
                    .element(p, &W::p_pr())
                    .map(|pr| dom.serialize_element(pr))
                    .unwrap_or_default();
                (text, props)
            })
            .collect()
    }
    let props = "<w:spacing w:before=\"120\"/><w:ind w:left=\"720\"/>";
    for (source_props, receiving_props, live_payload, peel) in [
        (props, props, "", true),
        (
            props,
            "<w:spacing w:before=\"240\"/><w:ind w:left=\"720\"/>",
            "",
            false,
        ),
        (props, props, "<w:r><w:tab/></w:r>", false),
        (
            props,
            props,
            "<w:bookmarkStart w:id=\"21\" w:name=\"owned\"/><w:bookmarkEnd w:id=\"21\"/>",
            false,
        ),
    ] {
        let source = format!(
            "<w:p><w:pPr>{source_props}<w:rPr><w:b/></w:rPr></w:pPr><w:r><w:t>Retained source anchor</w:t></w:r>{live_payload}{}{}</w:p>",
            revision("ins", " revised"),
            revision(
                "del",
                " within sixty days after receiving the signed request"
            )
        );
        let receiving = format!(
            "<w:p><w:pPr>{receiving_props}<w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/><w:b/></w:rPr></w:pPr>{}</w:p>",
            revision(
                "ins",
                "within thirty days after receiving the written request"
            )
        );
        let (mut dom, root, body) = document(&(source + &receiving));
        let accepted = projected(&mut dom, root, true);
        let rejected = projected(&mut dom, root, false);
        let before = dom.serialize_element(root);
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(
            projected(&mut dom, root, true),
            accepted,
            "accepted paragraphs and complete properties"
        );
        assert_eq!(
            projected(&mut dom, root, false),
            rejected,
            "rejected paragraphs and complete properties"
        );
        if peel {
            let ps = dom.elements(body, Some(&W::p()));
            assert!(dom.elements(ps[0], Some(&W::del())).is_empty());
            assert_eq!(dom.elements(ps[1], Some(&W::del())).len(), 1);
            let marker = dom
                .element(ps[0], &W::p_pr())
                .and_then(|pr| dom.element(pr, &W::r_pr()))
                .and_then(|pr| dom.element(pr, &W::ins()))
                .unwrap();
            assert_eq!(dom.attribute(marker, &W::author()), Some("Editor"));
            assert_eq!(dom.attribute(marker, &W::date()), Some(DATE));
            assert_eq!(dom.attribute(marker, &W::id()), Some("32"));
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph() {
    fn projected(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        let body = dom.element(projected, &W::body()).unwrap();
        dom.serialize_element(body)
    }
    for payload in [
        "<w:r><w:tab/></w:r>",
        "<w:r><w:br/></w:r>",
        "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r>",
    ] {
        // The unchanged payload owns A's first paragraph even though its
        // equal textual stream has no alphanumeric anchor. The following
        // pure-I paragraph disappears when revisions are rejected.
        let fragment = format!(
            "<w:p><w:pPr><w:spacing w:before=\"120\"/></w:pPr>{payload}{}{}</w:p><w:p><w:pPr><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}</w:p>",
            revision("ins", "Revised opening words"),
            revision(
                "del",
                " within sixty days after receiving the signed request"
            ),
            revision(
                "ins",
                "within thirty days after receiving the written request"
            ),
        );
        let (mut dom, root, _) = document(&fragment);
        let expected_accept = projected(&mut dom, root, true);
        let expected_reject = projected(&mut dom, root, false);
        let rejected_doc = dom.parse_xdocument(&expected_reject);
        let rejected_body = dom.root(rejected_doc).unwrap();
        assert_eq!(dom.elements(rejected_body, Some(&W::p())).len(), 1);
        assert_eq!(
            dom.descendants(rejected_body, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>(),
            " within sixty days after receiving the signed request"
        );
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(
            projected(&mut dom, root, true),
            expected_accept,
            "accepted exact body preserves nontext payload order"
        );
        assert_eq!(
            projected(&mut dom, root, false),
            expected_reject,
            "rejected exact body preserves payload, paragraph boundary and source properties"
        );
    }
}

#[test]
fn deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    for anchor in ["!", "🙂", " "] {
        let fragment = format!(
            "<w:p><w:pPr><w:spacing w:before=\"120\"/></w:pPr><w:r><w:t xml:space=\"preserve\">{anchor}</w:t></w:r>{}{}</w:p><w:p><w:pPr><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}</w:p>",
            revision("ins", "Revised opening words"),
            revision(
                "del",
                " within sixty days after receiving the signed request"
            ),
            revision(
                "ins",
                "within thirty days after receiving the written request"
            ),
        );
        let (mut dom, root, _) = document(&fragment);
        let accepted = projection(&mut dom, root, true);
        let rejected = projection(&mut dom, root, false);
        let before_doc = dom.parse_xdocument(&rejected);
        let before_body = dom.root(before_doc).unwrap();
        assert_eq!(dom.elements(before_body, Some(&W::p())).len(), 1);
        assert_eq!(
            dom.descendants(before_body, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>(),
            format!("{anchor} within sixty days after receiving the signed request")
        );
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(projection(&mut dom, root, true), accepted);
        assert_eq!(
            projection(&mut dom, root, false),
            rejected,
            "literal unchanged anchor {anchor:?} must not split the original paragraph"
        );
    }
}

#[test]
fn deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    let fragment = format!(
        "<w:p><w:pPr><w:spacing w:before=\"120\"/></w:pPr>{}{}</w:p><w:p><w:pPr><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}</w:p>",
        revision("ins", "Revised opening words"),
        revision(
            "del",
            "within sixty days after receiving the signed request"
        ),
        revision(
            "ins",
            "within thirty days after receiving the written request"
        ),
    );
    let (mut dom, root, _) = document(&fragment);
    let accepted = projection(&mut dom, root, true);
    let rejected = projection(&mut dom, root, false);
    let before_doc = dom.parse_xdocument(&rejected);
    let before_body = dom.root(before_doc).unwrap();
    assert_eq!(dom.elements(before_body, Some(&W::p())).len(), 1);
    assert_eq!(
        dom.descendants(before_body, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect::<String>(),
        "within sixty days after receiving the signed request"
    );
    peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
    assert_eq!(projection(&mut dom, root, true), accepted);
    assert_eq!(
        projection(&mut dom, root, false),
        rejected,
        "the retained original closing paragraph must not become an empty extra paragraph"
    );
}

#[test]
fn wholesale_deleted_body_peel_transfers_its_retained_closing_boundary() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    let props = "<w:spacing w:before=\"120\"/><w:ind w:left=\"720\"/>";
    let fragment = format!(
        "<w:p><w:pPr>{props}<w:rPr><w:b/></w:rPr></w:pPr>{}{}</w:p><w:p><w:pPr>{props}<w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/><w:b/></w:rPr></w:pPr>{}</w:p>",
        revision("ins", "Revised opening words"),
        revision(
            "del",
            "within sixty days after receiving the signed request"
        ),
        revision(
            "ins",
            "within thirty days after receiving the written request"
        ),
    );
    let (mut dom, root, body) = document(&fragment);
    let accepted = projection(&mut dom, root, true);
    let rejected = projection(&mut dom, root, false);
    let before_doc = dom.parse_xdocument(&rejected);
    let before_body = dom.root(before_doc).unwrap();
    assert_eq!(dom.elements(before_body, Some(&W::p())).len(), 1);
    assert_eq!(
        dom.descendants(before_body, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect::<String>(),
        "within sixty days after receiving the signed request"
    );
    peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
    assert_eq!(projection(&mut dom, root, true), accepted);
    assert_eq!(projection(&mut dom, root, false), rejected);
    let paragraphs = dom.elements(body, Some(&W::p()));
    assert!(dom.elements(paragraphs[0], Some(&W::del())).is_empty());
    assert_eq!(dom.elements(paragraphs[1], Some(&W::del())).len(), 1);
    let marker = dom
        .element(paragraphs[0], &W::p_pr())
        .and_then(|pr| dom.element(pr, &W::r_pr()))
        .and_then(|pr| dom.element(pr, &W::ins()))
        .unwrap();
    assert_eq!(dom.attribute(marker, &W::id()), Some("32"));
    assert_eq!(dom.attribute(marker, &W::author()), Some("Editor"));
    assert_eq!(dom.attribute(marker, &W::date()), Some(DATE));
}

#[test]
fn deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    for suffix in ["!", "🙂", " ", "..", ". "] {
        let fragment = format!(
            "<w:p><w:pPr><w:spacing w:before=\"120\"/><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}{}<w:r><w:t xml:space=\"preserve\">{suffix}</w:t></w:r></w:p><w:p><w:pPr><w:spacing w:before=\"120\"/></w:pPr>{}</w:p>",
            revision("ins", "Revised opening words"),
            revision(
                "del",
                "within sixty days after receiving the signed request"
            ),
            revision(
                "ins",
                "within thirty days after receiving the written request."
            ),
        );
        let (mut dom, root, _) = document(&fragment);
        let accepted = projection(&mut dom, root, true);
        let rejected = projection(&mut dom, root, false);
        let before_doc = dom.parse_xdocument(&rejected);
        let before_body = dom.root(before_doc).unwrap();
        assert_eq!(dom.elements(before_body, Some(&W::p())).len(), 1);
        assert_eq!(
            dom.descendants(before_body, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>(),
            format!("within sixty days after receiving the signed request{suffix}")
        );
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(projection(&mut dom, root, true), accepted);
        assert_eq!(
            projection(&mut dom, root, false),
            rejected,
            "equal suffix {suffix:?} must not be reordered ahead of original words"
        );
    }
}

#[test]
fn borrowed_period_peel_keeps_the_original_period_run_formatting() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    let deleted = revision(
        "del",
        "within sixty days after receiving the signed request",
    )
    .replace("<w:r>", "<w:r><w:rPr><w:i/></w:rPr>");
    let fragment = format!(
        "<w:p><w:pPr><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}{deleted}<w:r><w:rPr><w:b/></w:rPr><w:t>.</w:t></w:r></w:p><w:p>{}</w:p>",
        revision("ins", "Revised opening words"),
        revision(
            "ins",
            "within thirty days after receiving the written request."
        ),
    );
    let (mut dom, root, _) = document(&fragment);
    let accepted = projection(&mut dom, root, true);
    let rejected = projection(&mut dom, root, false);
    peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
    repair_borrowed_sentence_period(&mut dom, root, &WmlComparerSettings::default(), &mut 32);
    assert_eq!(projection(&mut dom, root, true), accepted);
    assert_eq!(
        projection(&mut dom, root, false),
        rejected,
        "A's shared bold period must not inherit the italic deleted word's formatting"
    );
}

#[test]
fn same_format_borrowed_period_peel_keeps_literal_text_and_character_properties() {
    fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<Vec<(char, String)>> {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        let body = dom.element(copy, &W::body()).unwrap();
        dom.elements(body, Some(&W::p()))
            .into_iter()
            .map(|p| {
                dom.descendants(p, Some(&W::t()))
                    .into_iter()
                    .flat_map(|t| {
                        let props = dom
                            .parent(t)
                            .and_then(|run| dom.element(run, &W::r_pr()))
                            .map(|pr| dom.serialize_element(pr))
                            .unwrap_or_default();
                        dom.value(t)
                            .chars()
                            .map(|ch| (ch, props.clone()))
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .collect()
    }
    let deleted = revision(
        "del",
        "within sixty days after receiving the signed request",
    )
    .replace("<w:r>", "<w:r><w:rPr><w:b/></w:rPr>");
    let fragment = format!(
        "<w:p><w:pPr><w:rPr><w:ins w:id=\"31\" w:author=\"Editor\" w:date=\"{DATE}\"/></w:rPr></w:pPr>{}{deleted}<w:r><w:rPr><w:b/></w:rPr><w:t>.</w:t></w:r></w:p><w:p>{}</w:p>",
        revision("ins", "Revised opening words"),
        revision(
            "ins",
            "within thirty days after receiving the written request."
        ),
    );
    let (mut dom, root, body) = document(&fragment);
    let accepted = projection(&mut dom, root, true);
    let rejected = projection(&mut dom, root, false);
    assert_eq!(rejected.len(), 1);
    assert_eq!(
        rejected[0].iter().map(|(ch, _)| *ch).collect::<String>(),
        "within sixty days after receiving the signed request."
    );
    peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
    let paragraphs = dom.elements(body, Some(&W::p()));
    assert!(dom.elements(paragraphs[0], Some(&W::del())).is_empty());
    assert_eq!(dom.elements(paragraphs[1], Some(&W::del())).len(), 1);
    repair_borrowed_sentence_period(&mut dom, root, &WmlComparerSettings::default(), &mut 32);
    assert_eq!(projection(&mut dom, root, true), accepted);
    assert_eq!(projection(&mut dom, root, false), rejected);
}

#[test]
fn nested_interleave_keeps_mark_only_items_and_defaults_invalid_terminal_levels() {
    for level in ["0", "invalid", "-1"] {
        for section in [false, true] {
            let last_props = format!("<w:numPr><w:ilvl w:val=\"{level}\"/></w:numPr>");
            let fragment = format!(
                "{}{}{}{}{}{}{}{}",
                para("ins", "First header", ""),
                para("ins", "A", ""),
                para("del", "Old top", NUMBERED),
                empty("del", ""),
                para("del", "Old nested", NESTED),
                empty("del", ""),
                para("del", "Old next", &last_props),
                if section { "<w:sectPr/>" } else { "" }
            );
            let (mut dom, root, body) = document(&fragment);
            let old = blocks(&dom, body);
            let xml = old
                .iter()
                .map(|&n| dom.serialize_element(n))
                .collect::<Vec<_>>();
            let text = views(&dom, root);
            interleave_list_cluster_after_coalesce(&mut dom, root);
            assert_eq!(
                blocks(&dom, body),
                [old[0], old[2], old[3], old[4], old[5], old[1], old[6]]
            );
            assert_eq!(views(&dom, root), text);
            for (&n, expected) in old.iter().zip(&xml) {
                assert_eq!(&dom.serialize_element(n), expected);
            }
            assert_eq!(
                dom.elements(body, Some(&W::sect_pr())).len(),
                usize::from(section)
            );
            let once = dom.serialize_element(root);
            interleave_list_cluster_after_coalesce(&mut dom, root);
            assert_eq!(dom.serialize_element(root), once);
        }
    }
}

#[test]
fn nested_interleave_uniform_tokens_are_case_insensitive_and_late_mismatches_reorder() {
    for (third, rewrites) in [("TeSt", false), ("other", true)] {
        let fragment = format!(
            "{}{}{}{}{}{}",
            para("ins", "test", ""),
            para("ins", "TEST", ""),
            para("ins", third, ""),
            para("del", "Old top", NUMBERED),
            para("del", "Old nested", NESTED),
            para("del", "Old next", NUMBERED)
        );
        let (mut dom, root, body) = document(&fragment);
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        let revisions = body_revisions(&dom, root);
        interleave_list_cluster_after_coalesce(&mut dom, root);
        assert_eq!(
            blocks(&dom, body),
            if rewrites {
                vec![old[0], old[3], old[4], old[1], old[2], old[5]]
            } else {
                old.clone()
            }
        );
        assert_eq!(views(&dom, root), text);
        assert_eq!(body_revisions(&dom, root), revisions);
    }
}

#[test]
fn nested_interleave_needs_a_complete_nested_cluster_followed_by_a_top_level_item() {
    for levels in [
        ["1", "0", "0"],
        ["0", "0", "1"],
        ["bad", "-1", "4294967296"],
    ] {
        let mut fragment = para("ins", "First header", "");
        fragment.push_str(&para("ins", "A", ""));
        fragment.push_str(&para("ins", "B", ""));
        for level in levels {
            fragment.push_str(&para(
                "del",
                "Old item",
                &format!("<w:numPr><w:ilvl w:val=\"{level}\"/></w:numPr>"),
            ));
        }
        unchanged(&fragment, interleave_list_cluster_after_coalesce);
    }
}

#[test]
fn final_story_fusion_keeps_the_original_deleted_tail_and_its_metadata() {
    for deletion_first in [false, true] {
        let d = para("del", "Base title", "<w:spacing w:line=\"360\"/>");
        let i = format!(
            "{}{}",
            para("ins", "New leading paragraph", ""),
            para("ins", "Revised tail words", "")
        );
        let pair = if deletion_first {
            format!("{d}{i}")
        } else {
            format!("{i}{d}")
        };
        let table = format!(
            "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
            para("del", "Old cell", "")
        );
        let final_p = format!(
            "<w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr>{}</w:p>",
            revision("del", "Original final words")
        );
        let (dom, _, body, old) = merge(&format!("{pair}{table}{final_p}<w:sectPr/>"), true);
        let (first_i, d, tail_i) = if deletion_first {
            (old[1], old[0], old[2])
        } else {
            (old[0], old[2], old[1])
        };
        assert_eq!(blocks(&dom, body), [first_i, d, old[3], old[4]]);
        assert!(dom.parent(tail_i).is_none());
        assert!(para_mark_revision(&dom, d, &W::del()));
        expect_segments(
            &dom,
            d,
            &[("ins", "Revised tail words"), ("del", "Base title")],
        );
        expect_segments(&dom, old[4], &[("del", "Original final words")]);
        assert_eq!(
            dom.attribute(dom.descendants(old[4], Some(&W::del()))[0], &W::date()),
            Some(DATE)
        );
        assert_eq!(
            dom.attribute(
                dom.descendants(d, Some(&W::spacing_el()))[0],
                &W::name("line")
            ),
            Some("360")
        );
    }
}

#[test]
fn explicit_page_breaks_block_final_story_fusion_on_either_source_side() {
    for deletion_first in [false, true] {
        for inserted_break in [false, true] {
            let mut d = para("del", "Base title", "");
            let mut i = para("ins", "Revised tail words", "");
            if inserted_break {
                i = i.replace("<w:t xml:space=", "<w:br w:type=\"page\"/><w:t xml:space=");
            } else {
                d = d.replace(
                    "<w:delText xml:space=",
                    "<w:br w:type=\"page\"/><w:delText xml:space=",
                );
            }
            let pair = if deletion_first {
                format!("{d}{i}")
            } else {
                format!("{i}{d}")
            };
            let table = format!(
                "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
                para("del", "Old cell", "")
            );
            let (dom, root, body, old) = merge(&format!("{pair}{table}<w:p/>"), true);
            assert_eq!(
                blocks(&dom, body),
                if deletion_first {
                    vec![old[1], old[0], old[2], old[3]]
                } else {
                    old.clone()
                }
            );
            assert_eq!(dom.descendants(root, Some(&W::name("br"))).len(), 1);
            assert!(para_carries_page_break(
                &dom,
                old[usize::from(deletion_first == inserted_break)]
            ));
        }
    }
}

#[test]
fn numbered_wholesale_gate_requires_the_last_new_item_and_half_of_old_items_numbered() {
    for (last_numbered, old_numbered, fold) in [(true, 2, false), (false, 2, true), (true, 1, true)]
    {
        let fragment = format!(
            "{}{}{}{}{}",
            para("ins", "Alpha", if last_numbered { "" } else { NUMBERED }),
            para("ins", "Beta", if last_numbered { NUMBERED } else { "" }),
            para(
                "del",
                "Old first item",
                if old_numbered >= 1 { NUMBERED } else { "" }
            ),
            para(
                "del",
                "Former second item",
                if old_numbered >= 2 { NUMBERED } else { "" }
            ),
            para("del", "Prior third item", "")
        );
        let (dom, _, body) = document(&fragment);
        let ps = blocks(&dom, body);
        assert_eq!(
            should_fold_multi_del_at_document_scale(&dom, body, ps[1], ps[2], &ps[..2], &ps[2..]),
            fold
        );
    }
}

#[test]
fn punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness() {
    for first in ["???", "Old item"] {
        let fragment = format!(
            "{}{}{}{}{}",
            para("ins", "!!!", NUMBERED),
            para("ins", "Fresh", NUMBERED),
            para("del", first, NUMBERED),
            para("del", "???", NUMBERED),
            para("del", "Prior item", NUMBERED)
        );
        let (dom, _, body) = document(&fragment);
        let ps = blocks(&dom, body);
        assert!(!para_revision_body_text(&dom, ps[0]).trim().is_empty());
        assert!(body_token_set(&para_revision_body_text(&dom, ps[0])).is_empty());
        assert!(!should_fold_ins_del_pair(&dom, ps[1], ps[2]));
        assert!(should_fold_multi_del_at_document_scale(
            &dom,
            body,
            ps[1],
            ps[2],
            &ps[..2],
            &ps[2..]
        ));
    }
}

#[test]
fn late_tip_overrides_earlier_toc_and_numbered_splice_anchors() {
    let mut fragment = String::new();
    for i in 0..40 {
        if i == 2 {
            fragment.push_str(&format!(
                "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
                para("ins", "New cell", "")
            ));
        } else {
            let text = match i {
                1 => "1. First section",
                3 => "Table of Contents",
                7 => "TIP: final anchor",
                _ => "New body",
            };
            fragment.push_str(&para("ins", text, ""));
        }
    }
    fragment.push_str(&para("del", "Base title", ""));
    fragment.push_str(&para("del", "Other base title", ""));
    let (mut dom, root, body) = document(&fragment);
    let old = blocks(&dom, body);
    let text = views(&dom, root);
    let table = dom.serialize_element(old[2]);
    splice_trailing_short_pure_dels_midstream(&mut dom, root);
    let expected: Vec<_> = old[..8]
        .iter()
        .chain(&old[40..])
        .chain(&old[8..40])
        .copied()
        .collect();
    assert_eq!(blocks(&dom, body), expected);
    assert_eq!(views(&dom, root), text);
    assert_eq!(dom.serialize_element(old[2]), table);
}

#[test]
fn plain_label_mesh_may_share_across_multiple_contiguous_revision_wrappers() {
    let fragment = format!(
        "<w:p>{}{}{}{}</w:p>",
        revision("ins", "1"),
        revision("ins", "2 "),
        revision("del", "Lvl 1 1"),
        revision("del", "2")
    );
    let (mut dom, root, body) = document(&fragment);
    let p = blocks(&dom, body)[0];
    let text = views(&dom, root);
    let old = dom.elements(p, None);
    mesh_short_label_shared_eq(&mut dom, p);
    assert_eq!(views(&dom, root), text);
    assert_eq!(projection(&dom, p, true), "12 ");
    assert_eq!(projection(&dom, p, false), "Lvl 1 12");
    assert_eq!(dom.elements(p, Some(&W::r())).len(), 1);
    let eq = dom.elements(p, Some(&W::r()))[0];
    assert_eq!(markup_text(&dom, eq), "12");
    assert!(
        dom.parent(old[0]).is_none(),
        "the fully shared insertion wrapper is removed"
    );
    assert_eq!(dom.attribute(old[1], &W::author()), Some("Editor"));
    assert_eq!(dom.attribute(old[1], &W::date()), Some(DATE));
    assert_eq!(markup_text(&dom, old[1]), " ");
    for &n in &old[2..] {
        assert_eq!(dom.attribute(n, &W::author()), Some("Editor"));
        assert_eq!(dom.attribute(n, &W::date()), Some(DATE));
    }
}

#[test]
fn suffix_only_bookend_mesh_uses_pinned_defaults_and_keeps_the_original_suffix() {
    let (mut dom, root, body) = document(
        "<w:p><w:ins><w:r><w:t>fresh right aligned sentence</w:t></w:r></w:ins><w:del><w:r><w:delText>old right justified prose</w:delText></w:r></w:del><w:r><w:t> suffix</w:t></w:r></w:p>",
    );
    let p = blocks(&dom, body)[0];
    let text = views(&dom, root);
    free_mesh_bookended_ins_del(&mut dom, root);
    assert_eq!(views(&dom, root), text);
    expect_segments(
        &dom,
        p,
        &[
            ("ins", "fresh "),
            ("del", "old "),
            ("r", "right"),
            ("ins", " aligned sentence"),
            ("del", " justified prose"),
            ("r", " suffix"),
        ],
    );
    let revisions: Vec<_> = dom
        .elements(p, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del()))
        .collect();
    for (index, &revision) in revisions.iter().enumerate() {
        assert_eq!(dom.attribute(revision, &W::author()), Some("Redline"));
        assert_eq!(
            dom.attribute(revision, &W::date()),
            Some(crate::document_comparer::DEFAULT_DATE)
        );
        assert_eq!(
            dom.attribute(revision, &W::id()),
            Some((index + 1).to_string().as_str())
        );
    }
}

#[test]
fn complete_field_carriers_keep_title_deletions_as_mark_only_properties() {
    for ppr in ["", "<w:pPr/>", "<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>"] {
        let field = format!(
            "<w:p>{ppr}<w:ins w:id=\"77\" w:author=\"Editor\" w:date=\"{DATE}\"><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/><w:t xml:space=\"preserve\"> </w:t></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins></w:p>"
        );
        let deleted = para(
            "del",
            "Original title",
            "<w:pStyle w:val=\"Title\"/><w:jc w:val=\"center\"/>",
        );
        let (mut dom, root, body) = document(&format!("{field}{deleted}"));
        let old = blocks(&dom, body);
        let text = views(&dom, root);
        let revisions = body_revisions(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(blocks(&dom, body), [old[0]]);
        assert_eq!(views(&dom, root), text);
        assert_eq!(body_revisions(&dom, root), revisions);
        assert!(dom.descendants(old[0], Some(&W::p_style())).is_empty());
        assert!(dom.descendants(old[0], Some(&W::jc_el())).is_empty());
        assert!(para_mark_revision(&dom, old[0], &W::del()));
        assert!(!para_mark_revision(&dom, old[0], &W::ins()));
        assert!(fields_complete(&dom, old[0], &W::fld_char()));
        let ppr = dom.element(old[0], &W::p_pr()).unwrap();
        let mark = dom.descendants(ppr, Some(&W::del()))[0];
        assert_eq!(dom.attribute(mark, &W::id()), Some("31"));
        assert_eq!(dom.attribute(mark, &W::author()), Some("Editor"));
        assert_eq!(dom.attribute(mark, &W::date()), Some(DATE));
    }
}
