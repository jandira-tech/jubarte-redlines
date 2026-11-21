// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn xml(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\" xmlns:mc=\"{}\" xmlns:pt=\"{}\" xmlns:w14=\"{}\"><w:body>{fragment}</w:body></w:document>",
            W::URI, MC::URI, PT::URI, W14::URI
        ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn paragraphs(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.elements(dom.element(root, &W::body()).unwrap(), Some(&W::p()))
}

fn text(dom: &Dom, node: NodeId, name: &XName) -> String {
    dom.descendants(node, Some(name))
        .iter()
        .map(|&t| dom.value_str(t).into_owned())
        .collect()
}

fn revised(kind: &str, value: &str, props: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind}/></w:rPr></w:pPr><w:{kind} w:author=\"Reviewer\" w:date=\"2026-01-01T00:00:00Z\"><w:r><w:{leaf}>{value}</w:{leaf}></w:r></w:{kind}></w:p>"
    )
}

fn assert_unchanged(fragment: &str, transform: fn(&mut Dom, NodeId)) {
    let (mut dom, root) = xml(fragment);
    let before = dom.serialize_element(root);
    transform(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before, "{fragment}");
}

#[test]
fn replacement_classification_is_transparent_only_for_hyperlinks() {
    let cases = [
        ("", None, None),
        (
            "<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>",
            Some(true),
            Some(true),
        ),
        (
            "<w:pPr><w:rPr><w:del/></w:rPr></w:pPr>",
            Some(false),
            Some(false),
        ),
        ("<w:ins w:author=\"Reviewer\"/>", Some(true), Some(true)),
        ("<w:ins w:author=\"Other\"/>", Some(true), Some(false)),
        ("<w:ins/>", Some(true), Some(false)),
        ("<w:del/>", Some(false), Some(false)),
        ("<w:ins w:author=\"Other\"/><w:del/>", None, Some(false)),
        ("<w:ins w:author=\"Reviewer\"/><w:del/>", None, None),
        ("<w:r><w:t>kept</w:t></w:r><w:del/>", None, None),
        (
            "<w:bookmarkStart/><w:del/><w:bookmarkEnd/>",
            Some(false),
            Some(false),
        ),
        (
            "<w:hyperlink><w:del/></w:hyperlink>",
            Some(false),
            Some(false),
        ),
        (
            "<w:hyperlink><w:ins w:author=\"Reviewer\"/></w:hyperlink>",
            Some(true),
            Some(true),
        ),
        (
            "<w:hyperlink><w:ins w:author=\"Other\"/></w:hyperlink>",
            Some(true),
            Some(false),
        ),
        (
            "<w:hyperlink><w:ins/><w:del/></w:hyperlink>",
            None,
            Some(false),
        ),
        ("<w:hyperlink><w:r/></w:hyperlink>", None, None),
        (
            "<w:sdt><w:sdtContent><w:del/></w:sdtContent></w:sdt>",
            None,
            None,
        ),
    ];
    for (body, ordinary, carried) in cases {
        let (dom, root) = xml(&format!("<w:p>{body}</w:p>"));
        let p = paragraphs(&dom, root)[0];
        assert_eq!(para_replacement_class(&dom, p), ordinary, "{body}");
        assert_eq!(
            para_class_carried_aware(&dom, p, "Reviewer"),
            carried,
            "{body}"
        );
    }
    let (dom, root) = xml("<w:tbl/>");
    let table = dom
        .element(dom.element(root, &W::body()).unwrap(), &W::tbl())
        .unwrap();
    assert_eq!(para_replacement_class(&dom, table), None);
    assert_eq!(para_class_carried_aware(&dom, table, "Reviewer"), None);
    assert!(!para_is_mixed_revision(&dom, table));
}

#[test]
fn row_move_classification_requires_matching_structural_paragraph_marks() {
    for (direction, expected, same_mark, opposite_mark) in [
        ("moveTo", true, "ins", "del"),
        ("moveFrom", false, "del", "ins"),
    ] {
        for (mark, row_class) in [
            (Some(same_mark), Some(expected)),
            (None, None),
            (Some(opposite_mark), None),
        ] {
            let properties = mark.map_or_else(String::new, |mark| {
                format!("<w:pPr><w:rPr><w:{mark}/></w:rPr></w:pPr>")
            });
            let (dom, root) = xml(&format!(
                "<w:p>{properties}<w:{direction}RangeStart/><w:{direction}><w:r><w:t>alpha</w:t></w:r></w:{direction}><w:{direction}RangeEnd/></w:p>"
            ));
            let p = paragraphs(&dom, root)[0];
            assert_eq!(
                para_replacement_class_with_moves(&dom, p, true),
                row_class,
                "direction={direction}, mark={mark:?}"
            );
            assert_eq!(
                para_replacement_class(&dom, p),
                None,
                "general paragraph folding must keep moves distinct"
            );
        }
    }
}

#[test]
fn pure_and_mixed_paragraph_classification_preserves_live_content() {
    for (body, deleted, inserted, mixed) in [
        (
            "<w:del><w:r><w:delText>old</w:delText></w:r></w:del>",
            true,
            false,
            false,
        ),
        (
            "<w:del><w:r><w:t>old</w:t></w:r></w:del>",
            true,
            false,
            false,
        ),
        ("<w:del/><w:r><w:t> </w:t></w:r>", true, false, false),
        ("<w:del/><w:r><w:t>live</w:t></w:r>", false, false, false),
        ("<w:del/><w:ins/>", false, false, true),
        ("<w:ins/>", false, true, false),
        ("<w:pPr><w:rPr><w:del/></w:rPr></w:pPr>", true, false, false),
        ("<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>", false, true, false),
        (
            "<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr><w:del/>",
            false,
            false,
            true,
        ),
        ("<w:r><w:t>live</w:t></w:r>", false, false, false),
    ] {
        let (dom, root) = xml(&format!("<w:p>{body}</w:p>"));
        let p = paragraphs(&dom, root)[0];
        assert_eq!(para_is_pure_deleted(&dom, p), deleted, "{body}");
        assert_eq!(para_is_pure_inserted(&dom, p), inserted, "{body}");
        assert_eq!(para_is_mixed_revision(&dom, p), mixed, "{body}");
    }
}

#[test]
fn classification_cache_hits_and_resets_agree_with_uncached_results() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            end_para_classification_cache();
        }
    }
    let _reset = Reset;
    let (dom, root) = xml("<w:p><w:del/></w:p><w:p><w:ins/><w:del/></w:p>");
    let ps = paragraphs(&dom, root);
    begin_para_classification_cache();
    for _ in 0..2 {
        assert!(para_is_pure_deleted(&dom, ps[0]));
        assert!(!para_is_mixed_revision(&dom, ps[0]));
        assert!(!para_is_pure_deleted(&dom, ps[1]));
        assert!(para_is_mixed_revision(&dom, ps[1]));
    }
    end_para_classification_cache();
    assert!(para_is_pure_deleted(&dom, ps[0]));
    assert!(para_is_mixed_revision(&dom, ps[1]));
}

#[test]
fn section_and_list_residual_label_boundaries() {
    for (s, section, residual) in [
        ("14.11Survival", true, true),
        (" 3.01 body", true, true),
        ("0.0", true, true),
        ("1. Item", false, false),
        ("1.", false, false),
        (".1", false, false),
        ("١.2", false, false),
        ("", false, false),
        ("Num 4", false, true),
        (" NUM 4. ", false, true),
        ("num 4 extra", false, true),
        ("num 4 extra words", false, false),
        ("num ", false, false),
        ("num A", false, false),
    ] {
        assert_eq!(body_looks_like_section_number(s), section, "{s}");
        assert_eq!(body_looks_like_list_residual_label(s), residual, "{s}");
    }
}

#[test]
fn token_splitting_preserves_separators_and_original_case() {
    for (s, label, expected) in [
        ("old RIGHT-aligned", "right", Some(("old ", "-aligned"))),
        ("right", "RIGHT", None),
        (" right ", "right", None),
        ("bright right end", "right", Some(("bright ", " end"))),
        ("élan RIGHT.", "right", Some(("élan ", "."))),
        ("abc", "absent", None),
        ("", "a", None),
    ] {
        assert_eq!(
            split_around_token(s, label),
            expected.map(|(a, b)| (a.to_string(), b.to_string()))
        );
    }
    for (s, label, expected) in [
        ("bright right end", "RIGHT", Some(("bright ", " end"))),
        ("right-aligned", "right", Some(("", "-aligned"))),
        ("right", "right", Some(("", ""))),
        ("brightly", "right", None),
        ("rightful right", "right", Some(("rightful ", ""))),
        ("", "right", None),
    ] {
        assert_eq!(
            split_around_whole_word(s, label),
            expected.map(|(a, b)| (a.to_string(), b.to_string()))
        );
    }
    for (s, token, prefix) in [
        ("Lvl 1 – A.", Some("A"), Some("Lvl 1 – ")),
        ("A", Some("A"), Some("")),
        ("élan a  ", Some("a"), Some("élan ")),
        ("---", None, None),
        ("Lvl 1 b", Some("b"), None),
    ] {
        assert_eq!(trailing_alnum_token(s).as_deref(), token);
        assert_eq!(strip_trailing_alnum_token(s, "a").as_deref(), prefix);
    }
}

#[test]
fn relatedness_and_short_title_gates_use_tokens_not_substrings() {
    for (a, b, expected) in [
        ("", "", 1.0),
        ("a", "", 0.0),
        ("A a", "a", 1.0),
        ("a b", "b c", 1.0 / 3.0),
    ] {
        assert!((body_text_jaccard(a, b) - expected).abs() < 1e-12);
    }
    for (a, b, expected) in [
        ("Heading Demo", "DEMO title", true),
        ("this with", "this with", false),
        ("abc", "abc", false),
        ("heading", "headings", false),
        ("", "Demo", false),
    ] {
        assert_eq!(short_title_shares_sig_token(a, b), expected);
    }
    for (value, digits, short, demo) in [
        ("", false, false, false),
        ("24", true, true, false),
        ("1 2", true, true, false),
        ("123", true, false, false),
        ("OK", false, true, false),
        ("12.", false, true, false),
        ("Title Demo 1", false, false, true),
        ("Demo Example", false, false, false),
        ("a b c d e f g h Demo", false, false, false),
    ] {
        let (dom, root) = xml(&revised("del", value, ""));
        let p = paragraphs(&dom, root)[0];
        assert_eq!(para_body_is_digits_only(&dom, p), digits, "{value}");
        assert_eq!(para_body_is_very_short(&dom, p), short, "{value}");
        assert_eq!(para_looks_like_demo_title(&dom, p), demo, "{value}");
    }
}

#[test]
fn deleted_text_conversion_stops_at_nested_revision_owners() {
    let (mut dom, root) = xml(
        "<w:p><w:r><w:t>old</w:t><w:instrText>FIELD</w:instrText><w:object><w:t>nested</w:t></w:object><w:ins><w:r><w:t>new</w:t></w:r></w:ins><w:del><w:r><w:t>owned</w:t></w:r></w:del><w:moveFrom><w:r><w:t>from</w:t></w:r></w:moveFrom><w:moveTo><w:r><w:t>to</w:t></w:r></w:moveTo></w:r></w:p>",
    );
    let run = dom.descendants(root, Some(&W::r()))[0];
    convert_run_text_to_del_text(&mut dom, run);
    assert_eq!(text(&dom, run, &W::del_text()), "oldnested");
    assert_eq!(text(&dom, run, &W::del_instr_text()), "FIELD");
    assert_eq!(text(&dom, run, &W::t()), "newownedfromto");
}

#[test]
fn field_completeness_requires_balanced_order_not_just_counts() {
    for (types, expected) in [
        (vec![], true),
        (vec!["begin", "separate", "end"], true),
        (vec!["begin", "begin", "end", "end"], true),
        (vec!["end", "begin"], false),
        (vec!["begin"], false),
        (vec!["end"], false),
        (vec!["separate", "unknown"], true),
    ] {
        let fields: String = types
            .iter()
            .map(|t| format!("<w:r><w:fldChar w:fldCharType=\"{t}\"/></w:r>"))
            .collect();
        let (dom, root) = xml(&format!("<w:p>{fields}</w:p>"));
        assert_eq!(
            fields_complete(&dom, root, &W::fld_char()),
            expected,
            "{types:?}"
        );
    }
}

#[test]
fn revision_wrapper_requires_consistent_hyperlink_stamps() {
    for (content, expected) in [
        (
            "<w:del w:author=\"A\" w:date=\"D\"/>",
            Some((false, "A", "D")),
        ),
        ("<w:ins/>", Some((true, "", ""))),
        (
            "<w:hyperlink><w:ins/><w:ins/></w:hyperlink>",
            Some((true, "", "")),
        ),
        ("<w:hyperlink><w:ins/><w:del/></w:hyperlink>", None),
        (
            "<w:hyperlink><w:ins w:author=\"A\"/><w:ins w:author=\"B\"/></w:hyperlink>",
            None,
        ),
        (
            "<w:hyperlink><w:ins w:date=\"A\"/><w:ins w:date=\"B\"/></w:hyperlink>",
            None,
        ),
        ("<w:hyperlink><w:r/></w:hyperlink>", None),
        ("<w:hyperlink/>", None),
        ("<w:r/>", None),
    ] {
        let (dom, root) = xml(&format!("<w:p>{content}</w:p>"));
        let child = dom.elements(paragraphs(&dom, root)[0], None)[0];
        assert_eq!(
            revision_wrapper(&dom, child),
            expected.map(|(k, a, d)| (k, a.to_string(), d.to_string())),
            "{content}"
        );
    }
    let mut dom = Dom::new();
    let leaf = dom.new_text("plain");
    assert_eq!(revision_wrapper(&dom, leaf), None);
}

#[test]
fn content_controls_unwrap_deepest_first_and_keep_text_order() {
    let (mut dom, root) = xml(
        "<w:p><w:sdt><w:sdtPr><w:tag w:val=\"outer\"/></w:sdtPr><w:sdtContent><w:r><w:t>A</w:t></w:r><w:sdt><w:sdtContent><w:r><w:t>B</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>C</w:t></w:r></w:sdtContent></w:sdt><w:sdt><w:sdtPr/></w:sdt></w:p>",
    );
    unwrap_content_controls(&mut dom, root);
    assert!(dom.descendants(root, Some(&W::sdt())).is_empty());
    assert!(dom.descendants(root, Some(&W::sdt_pr())).is_empty());
    assert_eq!(text(&dom, root, &W::t()), "ABC");
    assert_eq!(
        dom.elements(paragraphs(&dom, root)[0], Some(&W::r())).len(),
        3
    );
}

#[test]
fn residual_control_unwrap_keeps_equal_form_controls() {
    for marker in ["<w:ins/>", "<w:del/>", "<w:ins/><w:del/>"] {
        let control = "<w:sdt><w:sdtContent><w:r><w:t>form</w:t></w:r></w:sdtContent></w:sdt>";
        let (mut dom, root) = xml(&format!(
            "<w:p>{control}</w:p><w:p><w:pPr><w:rPr>{marker}</w:rPr></w:pPr>{control}</w:p>"
        ));
        unwrap_content_controls_in_pure_revisions(&mut dom, root);
        let ps = paragraphs(&dom, root);
        assert_eq!(dom.descendants(ps[0], Some(&W::sdt())).len(), 1);
        assert!(dom.descendants(ps[1], Some(&W::sdt())).is_empty());
        assert_eq!(text(&dom, root, &W::t()), "formform");
    }
}

#[test]
fn alternate_content_selects_first_bound_choice_or_fallback() {
    for (choices, expected, resolved) in [
        (
            "<mc:Choice Requires=\"missing\"><w:r><w:t>bad</w:t></w:r></mc:Choice><mc:Choice Requires=\"w w14\"><w:r><w:t>good</w:t></w:r></mc:Choice><mc:Fallback><w:r><w:t>fallback</w:t></w:r></mc:Fallback>",
            "good",
            true,
        ),
        (
            "<mc:Choice Requires=\"missing\"><w:r><w:t>bad</w:t></w:r></mc:Choice><mc:Fallback><w:r><w:t>fallback</w:t></w:r></mc:Fallback>",
            "fallback",
            true,
        ),
        // Declared, but not a namespace Word knows: Word 16 draws the
        // next Choice (#1876).
        (
            "<mc:Choice xmlns:x1=\"urn:jubarte:probe:unknown\" Requires=\"x1\"><w:r><w:t>unknown</w:t></w:r></mc:Choice><mc:Choice Requires=\"w14\"><w:r><w:t>known</w:t></w:r></mc:Choice><mc:Fallback><w:r><w:t>fallback</w:t></w:r></mc:Fallback>",
            "known",
            true,
        ),
        (
            "<mc:Choice xmlns:x1=\"urn:jubarte:probe:unknown\" Requires=\"x1\"><w:r><w:t>unknown</w:t></w:r></mc:Choice><mc:Fallback><w:r><w:t>fallback</w:t></w:r></mc:Fallback>",
            "fallback",
            true,
        ),
        (
            "<mc:Choice><w:r><w:t>lenient</w:t></w:r></mc:Choice>",
            "lenient",
            true,
        ),
        (
            "<mc:Choice Requires=\"\"><w:r><w:t>empty</w:t></w:r></mc:Choice>",
            "empty",
            true,
        ),
        (
            "<mc:Choice Requires=\"missing\"><w:r><w:t>unresolved</w:t></w:r></mc:Choice>",
            "unresolved",
            false,
        ),
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><mc:AlternateContent>{choices}</mc:AlternateContent></w:p>"
        ));
        resolve_alternate_content(&mut dom, root);
        assert_eq!(text(&dom, root, &W::t()), expected);
        assert_eq!(
            dom.descendants(root, Some(&MC::alternate_content()))
                .is_empty(),
            resolved
        );
    }
    for opaque in ["drawing", "pict"] {
        assert_unchanged(
            &format!(
                "<w:p><mc:AlternateContent><mc:Choice><w:{opaque}/></mc:Choice><mc:Fallback><w:r><w:t>fallback</w:t></w:r></mc:Fallback></mc:AlternateContent></w:p>"
            ),
            resolve_alternate_content,
        );
    }
    assert_unchanged(
        "<w:p><w:r><w:drawing><mc:AlternateContent><mc:Choice><w:r/></mc:Choice><mc:Fallback/></mc:AlternateContent></w:drawing></w:r></w:p>",
        resolve_alternate_content,
    );
}

fn fold_gate(
    ins: &[&str],
    dels: &[&str],
    ins_num: bool,
    del_num: bool,
    extra_words: usize,
) -> bool {
    let num = "<w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"7\"/></w:numPr>";
    let mut fragment = String::new();
    for v in ins {
        fragment.push_str(&revised("ins", v, if ins_num { num } else { "" }));
    }
    for v in dels {
        fragment.push_str(&revised("del", v, if del_num { num } else { "" }));
    }
    if extra_words > 0 {
        fragment.push_str(&format!(
            "<w:p><w:r><w:t>{}</w:t></w:r></w:p>",
            vec!["unchanged"; extra_words].join(" ")
        ));
    }
    let (dom, root) = xml(&fragment);
    let ps = paragraphs(&dom, root);
    should_fold_multi_del_at_document_scale(
        &dom,
        dom.element(root, &W::body()).unwrap(),
        ps[ins.len() - 1],
        ps[ins.len()],
        &ps[..ins.len()],
        &ps[ins.len()..ins.len() + dels.len()],
    )
}

#[test]
fn document_scale_fold_respects_absolute_ratio_and_fraction_boundaries() {
    let short = ["alpha", "bravo", "charlie"];
    let long_line = vec!["unrelated"; 13].join(" ");
    let long = [long_line.as_str(); 3];
    // 42 gap atoms / 3 inserted atoms: a large unrelated replacement.
    assert!(!fold_gate(&short, &long, false, false, 0));
    assert!(!fold_gate(&short, &long, false, false, 27)); // 42/69 > .60
    assert!(fold_gate(&short, &long, false, false, 28)); // 42/70 == .60
    let below_floor = ["unrelated"; 12].join(" ");
    assert!(fold_gate(
        &short,
        &[below_floor.as_str(); 3],
        false,
        false,
        0
    )); // 39 atoms
    let equal = ["unrelated"; 4].join(" ");
    assert!(fold_gate(&short, &[equal.as_str(); 3], false, false, 0));
    let ins_line = ["alpha"; 3].join(" ");
    let ratio_below = ["unrelated"; 11].join(" ");
    let ratio_at = ["unrelated"; 12].join(" ");
    assert!(fold_gate(
        &[ins_line.as_str(); 3],
        &[ratio_below.as_str(); 3],
        false,
        false,
        0
    ));
    assert!(!fold_gate(
        &[ins_line.as_str(); 3],
        &[ratio_at.as_str(); 3],
        false,
        false,
        0
    ));
    assert!(fold_gate(&["one", "two"], &long, false, false, 0));
    assert!(fold_gate(&short, &["unrelated", "other"], false, false, 0));
    assert!(fold_gate(
        &short,
        &["unrelated", "", long_line.as_str()],
        false,
        false,
        0
    ));
    assert!(fold_gate(
        &short,
        &["alpha", long_line.as_str(), long_line.as_str()],
        false,
        false,
        0
    ));
    assert!(fold_gate(
        &short,
        &[long_line.as_str(), long_line.as_str(), "charlie"],
        false,
        false,
        0
    ));
}

#[test]
fn document_scale_list_and_cover_special_cases() {
    assert!(fold_gate(
        &["test", "TEST", "test"],
        &["Num 4", "other", "third"],
        true,
        true,
        0
    ));
    assert!(!fold_gate(
        &["alpha", "bravo"],
        &["old", "former", "prior", "earlier"],
        true,
        true,
        0
    ));
    assert!(fold_gate(
        &["alpha", "bravo"],
        &["old", "former", "prior", "earlier"],
        false,
        true,
        0
    ));
    assert!(fold_gate(
        &["alpha", "bravo"],
        &["old", "former", "prior", "earlier"],
        true,
        false,
        0
    ));
    assert!(fold_gate(
        &["alpha", "bravo"],
        &["alpha", "former", "prior", "earlier"],
        true,
        true,
        0
    ));
    let long_item = vec!["former"; 13].join(" ");
    assert!(fold_gate(
        &["alpha", "bravo"],
        &[long_item.as_str(), "prior", "earlier"],
        true,
        true,
        0
    ));
    for tail in [
        "prepared by reviewer",
        "mail@example.com",
        "March 2040",
        "January 2040",
        "February 2040",
        "April 2040",
        "June 2040",
        "July 2040",
        "August 2040",
        "September 2040",
        "October 2040",
        "November 2040",
        "December 2040",
        "agreement signed",
    ] {
        assert!(
            !fold_gate(
                &["cover", "author", "subtitle", tail],
                &["old clause has five words", "former provision"],
                false,
                false,
                0
            ),
            "{tail}"
        );
    }
    assert!(fold_gate(
        &["cover", "author", "subtitle", "ordinary ending"],
        &["old clause has five words", "former provision"],
        false,
        false,
        0
    ));
    assert!(fold_gate(
        &["cover", "author", "subtitle", "prepared by reviewer"],
        &["old stub", "former"],
        false,
        false,
        0
    ));
    assert!(!fold_gate(
        &["ONE", "A", "TWO", "C"],
        &["old label", "former", "prior", "earlier"],
        false,
        false,
        0
    ));
    assert!(fold_gate(
        &["ONE", "A", "TWO", "C"],
        &["Demo title", "former", "prior", "earlier"],
        false,
        false,
        0
    ));
    assert!(fold_gate(
        &["apple", "pear", "grapes"],
        &[
            "This document demonstrates several unrelated concepts",
            "former",
            "prior"
        ],
        false,
        false,
        0
    ));
    let repeated = ["Lets tightly wrap this one"; 4].join(" ");
    assert!(!fold_gate(
        &[repeated.as_str()],
        &["old", "former", "prior"],
        false,
        false,
        0
    ));
    let unique = (0..20)
        .map(|i| format!("word{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(fold_gate(
        &[unique.as_str()],
        &["old", "former", "prior"],
        false,
        false,
        0
    ));
    assert!(!fold_gate(
        &["alpha beta"; 10],
        &["unrelated paragraph"; 50],
        false,
        false,
        0
    ));
    let (dom, root) = xml(&format!(
        "{}{}",
        revised("ins", "citation", "<w:pStyle w:val=\"ListParagraph\"/>"),
        revised(
            "del",
            "MUTUAL NONDISCLOSURE AGREEMENT",
            "<w:pStyle w:val=\"Heading1\"/>"
        )
    ));
    let ps = paragraphs(&dom, root);
    assert!(should_fold_multi_del_at_document_scale(
        &dom,
        dom.element(root, &W::body()).unwrap(),
        ps[0],
        ps[1],
        &[ps[0]],
        &[ps[1]]
    ));
}

#[test]
fn sole_pair_relatedness_boundary_is_inclusive() {
    for (inserted, deleted, expected) in [
        ("a b c d e f", "a b g h i j k l m n o p q", false), // 2/17 is below .12
        ("a b c d e f g h", "a b c d e f g h", true),
        ("new", "old", false),
        ("new", "", true),
    ] {
        let (dom, root) = xml(&format!(
            "{}{}",
            revised("ins", inserted, ""),
            revised("del", deleted, "")
        ));
        let ps = paragraphs(&dom, root);
        assert_eq!(
            should_fold_ins_del_pair(&dom, ps[0], ps[1]),
            expected,
            "{inserted} / {deleted}"
        );
    }
}

#[test]
fn cleanup_spacing_removes_only_redundant_mixed_history() {
    let equal = "<w:spacing w:before=\"0\" w:after=\"120\" w:line=\"240\" w:lineRule=\"auto\"/>";
    for (live, old, other, preserve) in [
        (equal, equal, "", false),
        (
            equal,
            equal,
            "<w:rPr/><w:pStyle w:val=\"Heading1\"/>",
            false,
        ),
        (equal, equal, "<w:ind w:left=\"720\"/>", true),
        (equal, "<w:spacing w:line=\"276\"/>", "", true),
        (
            "<w:spacing w:after=\"120\"/>",
            "<w:spacing w:after=\"120\"/>",
            "",
            true,
        ),
        (equal, "<w:spacing w:after=\"120\"/>", "", true),
        ("", equal, "", true),
        (equal, "", "", true),
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr>{live}<w:pPrChange w:id=\"9\"><w:pPr>{old}{other}</w:pPr></w:pPrChange></w:pPr><w:ins><w:r><w:t>new</w:t></w:r></w:ins><w:del><w:r><w:delText>old</w:delText></w:r></w:del></w:p>"
        ));
        cleanup_spacing_and_default_jc(&mut dom, root);
        let p = paragraphs(&dom, root)[0];
        let ppr = dom.element(p, &W::p_pr()).unwrap();
        assert_eq!(
            dom.element(ppr, &W::p_pr_change()).is_some(),
            preserve,
            "{live} / {old} / {other}"
        );
        assert_eq!(text(&dom, p, &W::t()), "new");
        assert_eq!(text(&dom, p, &W::del_text()), "old");
    }
}

#[test]
fn cleanup_default_alignment_keeps_real_layout_history() {
    for (alignment, remove) in [
        ("left", true),
        ("start", true),
        ("right", false),
        ("center", false),
        ("", false),
    ] {
        for (other, keep_history) in [
            ("", false),
            ("<w:rPr><w:b/></w:rPr>", false),
            ("<w:ind w:left=\"720\"/>", true),
        ] {
            let (mut dom, root) = xml(&format!(
                "<w:p><w:pPr><w:jc w:val=\"{alignment}\"/><w:pPrChange><w:pPr><w:jc w:val=\"{alignment}\"/>{other}</w:pPr></w:pPrChange></w:pPr><w:r><w:t>kept</w:t></w:r></w:p>"
            ));
            cleanup_spacing_and_default_jc(&mut dom, root);
            let p = paragraphs(&dom, root)[0];
            let ppr = dom.element(p, &W::p_pr()).unwrap();
            assert_eq!(dom.element(ppr, &W::jc_el()).is_none(), remove);
            assert_eq!(
                dom.element(ppr, &W::p_pr_change()).is_some(),
                !remove || keep_history
            );
            assert_eq!(text(&dom, p, &W::t()), "kept");
        }
    }
    assert_unchanged(
        "<w:p/><w:tbl/><w:p><w:pPr><w:pPrChange/></w:pPr></w:p>",
        cleanup_spacing_and_default_jc,
    );
}

#[test]
fn cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history() {
    for (spacing, other, last, live, keep) in [
        ("<w:spacing w:line=\"276\"/>", "", false, false, false),
        ("<w:spacing w:line=\"276\"/>", "", true, false, false),
        ("<w:spacing w:line=\"240\"/>", "", false, true, false),
        ("<w:spacing w:line=\"240\"/>", "", true, false, true),
        (
            "<w:spacing w:line=\"276\" w:after=\"0\"/>",
            "",
            false,
            true,
            false,
        ),
        (
            "<w:spacing w:line=\"276\" w:before=\"0\"/>",
            "",
            false,
            true,
            false,
        ),
        (
            "<w:spacing w:line=\"276\" w:lineRule=\"auto\"/>",
            "",
            false,
            true,
            false,
        ),
        (
            "<w:spacing w:line=\"240\"/>",
            "<w:ind w:left=\"720\"/>",
            false,
            false,
            true,
        ),
        (
            "<w:spacing w:line=\"240\"/>",
            "<w:pStyle w:val=\"Title\"/><w:rPr/>",
            false,
            true,
            false,
        ),
    ] {
        let props = format!("<w:pPrChange><w:pPr>{spacing}{other}</w:pPr></w:pPrChange>");
        let tail = if last {
            ""
        } else {
            "<w:p><w:r><w:t>tail</w:t></w:r></w:p>"
        };
        let (mut dom, root) = xml(&format!("{}{tail}", revised("del", "old", &props)));
        cleanup_spacing_and_default_jc(&mut dom, root);
        let ppr = dom.element(paragraphs(&dom, root)[0], &W::p_pr()).unwrap();
        assert_eq!(
            dom.element(ppr, &W::spacing_el()).is_some(),
            live,
            "{spacing}/{other}/{last}"
        );
        assert_eq!(dom.element(ppr, &W::p_pr_change()).is_some(), keep);
        assert_eq!(text(&dom, root, &W::del_text()), "old");
    }
}

#[test]
fn short_label_free_mesh_preserves_deleted_prefix_and_inserted_whitespace() {
    for (inserted, deleted, expected_del, expected_live) in [
        ("A", "Lvl 1 – A", "Lvl 1 – ", "A"),
        ("A ", "Lvl 1 – A", "Lvl 1 – ", "A "),
        ("A.", "Lvl 1 – A", "Lvl 1 – ", "A."),
        ("AB ", "Lvl 1 – AB", "Lvl 1 – ", "AB "),
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr/><w:ins><w:r><w:t>{inserted}</w:t></w:r></w:ins><w:del><w:r><w:delText>{deleted}</w:delText></w:r></w:del></w:p>"
        ));
        let p = paragraphs(&dom, root)[0];
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(text(&dom, p, &W::del_text()), expected_del);
        assert_eq!(text(&dom, p, &W::t()), expected_live);
        let kids = dom.elements(p, None);
        assert!(dom.name_is(kids[1], &W::del()));
        assert!(dom.name_is(kids[2], &W::r()));
        assert_eq!(
            text(&dom, kids[2], &W::t()),
            if inserted.starts_with("AB") {
                "AB"
            } else {
                "A"
            }
        );
        assert_eq!(
            dom.elements(p, Some(&W::ins())).len(),
            usize::from(inserted != "A")
        );
    }
    for (i, d) in [
        ("", "Lvl 1 a"),
        ("abc", "Lvl 1 abc"),
        ("---", "Lvl 1 a"),
        ("a", "Item a"),
        ("a", "Lvl a 1"),
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:ins><w:r><w:t>{i}</w:t></w:r></w:ins><w:del><w:r><w:delText>{d}</w:delText></w:r></w:del></w:p>"
        ));
        let before = dom.serialize_element(root);
        let p = paragraphs(&dom, root)[0];
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(dom.serialize_element(root), before, "{i}/{d}");
    }
}

#[test]
fn residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer() {
    let num = "<w:numPr><w:numId w:val=\"7\"/></w:numPr>";
    let source = "<w:spacing w:line=\"276\"/><w:ind w:left=\"720\"/>";
    let fragment = format!(
        "{}{}{}<w:p><w:r><w:t>tail</w:t></w:r></w:p>",
        revised("ins", "a ", source),
        revised("ins", "", ""),
        revised(
            "del",
            "Lvl 1 – a",
            &format!("<w:pStyle w:val=\"ListParagraph\"/>{num}")
        )
    );
    let (mut dom, root) = xml(&fragment);
    residual_short_label_zip(&mut dom, root);
    let ps = paragraphs(&dom, root);
    assert_eq!(ps.len(), 3);
    let ppr = dom.element(ps[0], &W::p_pr()).unwrap();
    assert!(dom.element(ppr, &W::num_pr()).is_some());
    assert!(dom.element(ppr, &W::spacing_el()).is_some());
    assert!(dom.element(ppr, &W::name("ind")).is_some());
    assert_eq!(text(&dom, ps[0], &W::del_text()), "Lvl 1 – ");
    assert_eq!(text(&dom, ps[0], &W::t()), "a ");
    assert!(para_is_pure_inserted(&dom, ps[1]));
    assert_eq!(text(&dom, ps[1], &W::t()), "");
    assert!(para_body_text_is_whitespace_only(&dom, ps[1]));
    assert_eq!(text(&dom, ps[2], &W::t()), "tail");
}

#[test]
fn short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail() {
    let num = "<w:numPr><w:numId w:val=\"7\"/></w:numPr>";
    let empty = "<w:p><w:pPr><w:rPr><w:del w:id=\"44\"/></w:rPr></w:pPr></w:p>";
    let (mut dom, root) = xml(&format!(
        "{}{empty}{}",
        revised("ins", "a", num),
        revised("del", "old list item", "")
    ));
    fold_short_list_label_into_empty_pure_del(&mut dom, root);
    let ps = paragraphs(&dom, root);
    assert_eq!(ps.len(), 2);
    let ppr = dom.element(ps[0], &W::p_pr()).unwrap();
    assert!(dom.element(ppr, &W::num_pr()).is_none());
    let mark = dom
        .element(dom.element(ppr, &W::r_pr()).unwrap(), &W::del())
        .unwrap();
    assert_eq!(dom.attribute(mark, &W::id()), Some("44"));
    assert_eq!(text(&dom, ps[0], &W::t()), "a");
    assert_eq!(text(&dom, ps[1], &W::del_text()), "old list item");
    for (label, props, deleted, tail) in [
        ("a", "", empty, true),
        ("one two three", num, empty, true),
        ("", num, empty, true),
        ("a", num, empty, false),
        (
            "a",
            num,
            "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr><w:del/></w:p>",
            true,
        ),
        (
            "a",
            num,
            "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr><w:r><w:t>live</w:t></w:r></w:p>",
            true,
        ),
    ] {
        let tail = if tail {
            revised("del", "old list item", "")
        } else {
            String::new()
        };
        assert_unchanged(
            &format!("{}{deleted}{tail}", revised("ins", label, props)),
            fold_short_list_label_into_empty_pure_del,
        );
    }
}

#[test]
fn bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties() {
    let (mut dom, root) = xml(
        "<w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>This </w:t></w:r><w:ins w:author=\"Reviewer\" w:date=\"Pinned\"><w:r><w:t>fresh right-aligned clever sentence</w:t></w:r></w:ins><w:del><w:r><w:rPr><w:b/></w:rPr><w:delText>old right justified prose</w:delText></w:r></w:del><w:r><w:t>.</w:t></w:r></w:p>",
    );
    free_mesh_bookended_ins_del(&mut dom, root);
    let p = paragraphs(&dom, root)[0];
    let kids = dom.elements(p, None);
    let names: Vec<_> = kids
        .iter()
        .map(|&c| dom.name(c).unwrap().local_name().to_string())
        .collect();
    assert_eq!(names, ["pPr", "r", "ins", "del", "r", "ins", "del", "r"]);
    assert_eq!(text(&dom, kids[1], &W::t()), "This ");
    assert_eq!(text(&dom, kids[2], &W::t()), "fresh ");
    assert_eq!(text(&dom, kids[3], &W::del_text()), "old ");
    assert_eq!(text(&dom, kids[4], &W::t()), "right");
    assert_eq!(text(&dom, kids[5], &W::t()), "-aligned clever sentence");
    assert_eq!(text(&dom, kids[6], &W::del_text()), " justified prose");
    assert_eq!(text(&dom, kids[7], &W::t()), ".");
    assert_eq!(dom.descendants(kids[4], Some(&W::name("i"))).len(), 1);
    for &c in &[kids[3], kids[6]] {
        assert_eq!(dom.descendants(c, Some(&W::name("b"))).len(), 1);
    }
    for (id, &c) in [kids[2], kids[3], kids[5], kids[6]].iter().enumerate() {
        assert_eq!(dom.attribute(c, &W::author()), Some("Reviewer"));
        assert_eq!(dom.attribute(c, &W::date()), Some("Pinned"));
        assert_eq!(
            dom.attribute(c, &W::id()),
            Some((id + 1).to_string().as_str())
        );
    }
}

#[test]
fn bookended_free_mesh_rejects_ambiguous_and_nontext_anchors() {
    for (i, d, extra) in [
        ("fresh right aligned", "old right justified prose", ""),
        (
            "fresh right aligned sentence",
            "old left justified prose",
            "",
        ),
        ("fresh right shared sentence", "old right shared prose", ""),
        (
            "fresh right right sentence",
            "old right justified prose",
            "",
        ),
        (
            "fresh this aligned sentence",
            "old this justified prose",
            "",
        ),
        (
            "fresh right aligned sentence",
            "old right justified prose",
            "<w:bookmarkStart/>",
        ),
        (
            "fresh right aligned sentence",
            "old right justified prose",
            "<w:ins/>",
        ),
    ] {
        assert_unchanged(
            &format!(
                "<w:p><w:r><w:t>This </w:t></w:r><w:ins><w:r><w:t>{i}</w:t></w:r></w:ins><w:del><w:r><w:delText>{d}</w:delText></w:r></w:del>{extra}</w:p>"
            ),
            free_mesh_bookended_ins_del,
        );
    }
    assert_unchanged(
        "<w:p><w:r><w:t>This </w:t></w:r><w:ins><w:r><w:t>fresh right aligned sentence</w:t><w:tab/></w:r></w:ins><w:del><w:r><w:delText>old right justified prose</w:delText></w:r></w:del></w:p>",
        free_mesh_bookended_ins_del,
    );
    assert_unchanged(
        "<w:p><w:r><w:t>.</w:t></w:r><w:ins><w:r><w:t>fresh right aligned sentence</w:t></w:r></w:ins><w:del><w:r><w:delText>old right justified prose</w:delText></w:r></w:del></w:p>",
        free_mesh_bookended_ins_del,
    );
}

#[test]
fn trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor() {
    for (ins_count, del_count, empty, remove) in [
        (
            10,
            1,
            "<w:p><w:pPr><w:spacing w:after=\"120\"/></w:pPr></w:p>",
            true,
        ),
        (10, 4, "<w:p><w:pPr><w:spacing/></w:pPr></w:p>", true),
        (9, 2, "<w:p><w:pPr><w:spacing/></w:pPr></w:p>", false),
        (10, 5, "<w:p><w:pPr><w:spacing/></w:pPr></w:p>", false),
        (10, 1, "<w:p/>", false),
        (
            10,
            1,
            "<w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr></w:p>",
            false,
        ),
        (
            10,
            1,
            "<w:p><w:pPr><w:spacing/><w:rPr><w:ins/></w:rPr></w:pPr></w:p>",
            false,
        ),
        (
            10,
            1,
            "<w:p><w:pPr><w:spacing/></w:pPr><w:r><w:t>tail</w:t></w:r></w:p>",
            false,
        ),
    ] {
        let fragment = format!(
            "{}{}{empty}<w:sectPr/>",
            revised("ins", "new", "").repeat(ins_count),
            revised("del", "old", "").repeat(del_count)
        );
        let (mut dom, root) = xml(&fragment);
        let ps = paragraphs(&dom, root);
        let last = *ps.last().unwrap();
        strip_trailing_bare_empty_after_pure_i_dominant(&mut dom, root);
        assert_eq!(
            dom.parent(last).is_none(),
            remove,
            "{ins_count}/{del_count}/{empty}"
        );
        assert_eq!(text(&dom, root, &W::del_text()), "old".repeat(del_count));
        assert_eq!(dom.descendants(root, Some(&W::sect_pr())).len(), 1);
    }
}

#[test]
fn promoted_list_numbering_uses_inserted_id_and_deleted_level() {
    for level in [None, Some("2")] {
        let ilvl = level
            .map(|v| format!("<w:ilvl w:val=\"{v}\"/>"))
            .unwrap_or_default();
        let parked = format!(
            "<w:pPrChange w:id=\"9\"><w:pPr><w:numPr>{ilvl}<w:numId w:val=\"0\"/></w:numPr></w:pPr></w:pPrChange>"
        );
        let fragment = format!(
            "{}{}",
            revised("ins", "a", "<w:numPr><w:numId w:val=\"7\"/></w:numPr>"),
            revised("del", "14.11Survival", &parked)
        );
        let (mut dom, root) = xml(&fragment);
        promote_live_numpr_on_pure_d_from_pprchange(&mut dom, root);
        let p = paragraphs(&dom, root)[1];
        let ppr = dom.element(p, &W::p_pr()).unwrap();
        let num = dom.element(ppr, &W::num_pr()).unwrap();
        assert_eq!(
            dom.attribute(dom.element(num, &W::name("ilvl")).unwrap(), &W::val()),
            Some(level.unwrap_or("0"))
        );
        assert_eq!(
            dom.attribute(dom.element(num, &W::name("numId")).unwrap(), &W::val()),
            Some("7")
        );
        assert!(dom.element(ppr, &W::p_pr_change()).is_some());
        assert_eq!(text(&dom, p, &W::del_text()), "14.11Survival");
        let once = dom.serialize_element(root);
        promote_live_numpr_on_pure_d_from_pprchange(&mut dom, root);
        assert_eq!(dom.serialize_element(root), once);
    }
    for (label, source, deleted, parked) in [
        (
            "one two three",
            "<w:numPr><w:numId w:val=\"7\"/></w:numPr>",
            "14.11Survival",
            "<w:pPrChange><w:pPr><w:numPr/></w:pPr></w:pPrChange>",
        ),
        (
            "a",
            "<w:numPr/>",
            "14.11Survival",
            "<w:pPrChange><w:pPr><w:numPr/></w:pPr></w:pPrChange>",
        ),
        (
            "a",
            "<w:numPr><w:numId w:val=\"7\"/></w:numPr>",
            "ordinary prose",
            "<w:pPrChange><w:pPr><w:numPr/></w:pPr></w:pPrChange>",
        ),
        (
            "a",
            "<w:numPr><w:numId w:val=\"7\"/></w:numPr>",
            "Num 4",
            "<w:pStyle w:val=\"ListParagraph\"/><w:pPrChange><w:pPr><w:numPr/></w:pPr></w:pPrChange>",
        ),
        (
            "a",
            "<w:numPr><w:numId w:val=\"7\"/></w:numPr>",
            "Num 4",
            "<w:pPrChange><w:pPr/></w:pPrChange>",
        ),
    ] {
        assert_unchanged(
            &format!(
                "{}{}",
                revised("ins", label, source),
                revised("del", deleted, parked)
            ),
            promote_live_numpr_on_pure_d_from_pprchange,
        );
    }
}

#[test]
fn field_fold_rejects_crossing_but_allows_balanced_deleted_fields() {
    for (types, inserted_field, expected) in [
        (vec!["begin", "end"], true, false),
        (vec!["end", "begin"], true, true),
        (vec!["begin"], true, true),
        (vec!["end"], false, false),
        (vec!["separate"], true, false),
    ] {
        let fc: String = types
            .iter()
            .map(|v| format!("<w:r><w:fldChar w:fldCharType=\"{v}\"/></w:r>"))
            .collect();
        let later = if inserted_field {
            "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>"
        } else {
            "<w:r><w:t>plain</w:t></w:r>"
        };
        let (dom, root) = xml(&format!("<w:p/><w:p>{later}</w:p><w:p>{fc}</w:p>"));
        let ps = paragraphs(&dom, root);
        assert_eq!(
            fold_crosses_a_field(&dom, ps[2], ps[0], &ps[..2]),
            expected,
            "{types:?}"
        );
    }
    for (body, residue, page) in [
        ("<w:r><w:fldChar/></w:r>", true, false),
        ("<w:r><w:fldChar/><w:t> </w:t></w:r>", true, false),
        ("<w:r><w:fldChar/><w:t>link</w:t></w:r>", false, false),
        ("<w:r><w:br w:type=\"page\"/></w:r>", false, true),
        ("<w:pPr><w:pageBreakBefore/></w:pPr>", false, true),
        ("<w:r><w:br w:type=\"textWrapping\"/></w:r>", false, false),
    ] {
        let (dom, root) = xml(&format!("<w:p>{body}</w:p>"));
        let p = paragraphs(&dom, root)[0];
        assert_eq!(para_is_field_residue(&dom, p), residue);
        assert_eq!(para_carries_page_break(&dom, p), page);
    }
}
