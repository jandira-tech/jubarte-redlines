// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

// Coverage: lines and branches await the parent's instrumented run.
// Overall requested gates: 95% lines, 90% branches. No runtime IO here.
fn document(fragment: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\" xmlns:pt=\"{}\" xmlns:r=\"{}\"><w:body>{fragment}</w:body></w:document>",
            W::URI, PT::URI, R::URI
        ));
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (dom, root, body)
}

fn revision(kind: &str, text: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:{kind} w:id=\"7\" w:author=\"Author\" w:date=\"Pinned\"><w:r><w:{leaf}>{text}</w:{leaf}></w:r></w:{kind}>"
    )
}

fn pure(kind: &str, text: &str) -> String {
    format!("<w:p>{}</w:p>", revision(kind, text))
}

fn marked(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id=\"8\"/></w:rPr></w:pPr>{}</w:p>",
        revision(kind, text)
    )
}

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Fallback".into(),
        date_time_for_revisions: "2000-01-01T00:00:00Z".into(),
        ..WmlComparerSettings::default()
    }
}

// Compare the complete tree, including every attribute, text leaf and
// child position. Namespace declaration placement is serialization trivia.
fn same_tree(dom: &Dom, actual: NodeId, expected_dom: &Dom, expected: NodeId) {
    assert_eq!(dom.name(actual), expected_dom.name(expected));
    let attrs = |d: &Dom, n| {
        let mut a: Vec<_> = d
            .attributes(n)
            .into_iter()
            .filter(|(name, _)| !d.is_namespace_declaration(name))
            .map(|(name, value)| {
                (
                    name.namespace_name().to_string(),
                    name.local_name().to_string(),
                    value,
                )
            })
            .collect();
        a.sort();
        a
    };
    assert_eq!(attrs(dom, actual), attrs(expected_dom, expected));
    if !dom.is_element(actual) {
        assert_eq!(dom.text_value(actual), expected_dom.text_value(expected));
    }
    let a = dom.nodes(actual);
    let e = expected_dom.nodes(expected);
    assert_eq!(a.len(), e.len());
    for (a, e) in a.into_iter().zip(e) {
        same_tree(dom, a, expected_dom, e);
    }
}

fn expect_body(dom: &Dom, body: NodeId, expected: &str) {
    let (ed, _, eb) = document(expected);
    same_tree(dom, body, &ed, eb);
}

fn unchanged(fragment: &str, pass: fn(&mut Dom, NodeId)) {
    let (mut dom, root, _) = document(fragment);
    let before = dom.serialize_element(root);
    pass(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before);
}

#[test]
fn digit_title_split_moves_all_leading_fragments_and_keeps_original_properties() {
    for digits in ["24", " 2 4 ", "0"] {
        let leading = format!("{}{}", revision("ins", digits), revision("ins", "5"));
        let rest = format!("{}<w:r><w:t>.</w:t></w:r>", revision("del", "Title Demo"));
        let props = "<w:pPr><w:pStyle w:val=\"Title\"/><w:rPr><w:del/></w:rPr></w:pPr>";
        let (mut dom, root, body) =
            document(&format!("<w:p>{props}{leading}{rest}</w:p><w:sectPr/>"));
        split_digits_ins_from_mixed_title(&mut dom, root);
        expect_body(
            &dom,
            body,
            &format!("<w:p>{leading}</w:p><w:p>{props}{rest}</w:p><w:sectPr/>"),
        );
        let once = dom.serialize_element(root);
        split_digits_ins_from_mixed_title(&mut dom, root);
        assert_eq!(dom.serialize_element(root), once);
    }
}

#[test]
fn digit_split_rejects_nonascii_empty_and_later_insertions() {
    for text in ["", " ", "٢", "2a", "2.4"] {
        unchanged(
            &format!(
                "<w:p>{}{}</w:p>",
                revision("ins", text),
                revision("del", "Demo")
            ),
            split_digits_ins_from_mixed_title,
        );
    }
    unchanged(
        &format!(
            "<w:p>{}{}{}</w:p>",
            revision("ins", "24"),
            revision("del", "Demo"),
            revision("ins", "later")
        ),
        split_digits_ins_from_mixed_title,
    );
    // The deletion mark alone does not make the remaining body deleted.
    unchanged(
        &format!(
            "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr>{}<w:r><w:t>live</w:t></w:r></w:p>",
            revision("ins", "24")
        ),
        split_digits_ins_from_mixed_title,
    );
}

#[test]
fn trailing_insertion_peel_has_inclusive_token_bounds_and_keeps_punctuation() {
    for count in [1, 12] {
        let text = std::iter::repeat_n("shared", count)
            .collect::<Vec<_>>()
            .join(" ");
        let lead = revision("del", "old heading");
        let tail = revision("ins", &text);
        let punct = "<w:r><w:t>.</w:t></w:r>";
        let old = revision("del", "shared old body has six whole words");
        let props = "<w:pPr><w:jc w:val=\"right\"/></w:pPr>";
        let (mut dom, root, body) = document(&format!(
            "<w:p>{lead}{tail}{punct}</w:p><w:p>{props}{old}</w:p>"
        ));
        peel_trailing_ins_from_mix_into_following_pure_del(&mut dom, root);
        expect_body(
            &dom,
            body,
            &format!("<w:p>{lead}{punct}</w:p><w:p>{props}{tail}{old}</w:p>"),
        );
    }
}

#[test]
fn trailing_insertion_peel_rejects_exact_guard_boundaries() {
    let old = pure("del", "shared old body has six whole words");
    for text in [
        "",
        " ",
        "other",
        "sh",
        "shared shared shared shared shared shared shared shared shared shared shared shared shared",
    ] {
        unchanged(
            &format!(
                "<w:p>{}{}</w:p>{old}",
                revision("del", "old"),
                revision("ins", text)
            ),
            peel_trailing_ins_from_mix_into_following_pure_del,
        );
    }
    for rest in [
        "<w:r><w:t>live</w:t></w:r>",
        "<w:bookmarkStart w:id=\"1\"/>",
    ] {
        unchanged(
            &format!(
                "<w:p>{}{}{rest}</w:p>{old}",
                revision("del", "old"),
                revision("ins", "shared")
            ),
            peel_trailing_ins_from_mix_into_following_pure_del,
        );
    }
    unchanged(
        &format!("{}{}", pure("ins", "shared"), old),
        peel_trailing_ins_from_mix_into_following_pure_del,
    );
    unchanged(
        &format!(
            "<w:p>{}{}</w:p>{}",
            revision("del", "old"),
            revision("ins", "shared"),
            pure("del", "shared two three four five")
        ),
        peel_trailing_ins_from_mix_into_following_pure_del,
    );
}

#[test]
fn trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths() {
    fn projected_body(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let copy = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(dom.element(copy, &W::body()).unwrap())
    }
    // These positive continuations already have B's joining boundary.
    // The previous fixture left A's retained boundary here and moved a
    // suffix past equal punctuation, overclaiming source conservation.
    let joining = "<w:pPr><w:rPr><w:ins w:id=\"8\" w:author=\"Author\" w:date=\"1970-01-01T00:00:00Z\"/></w:rPr></w:pPr>";
    let cases = [
        ("shared two three four", "shared alpha beta gamma", true),
        (
            "one two three four five six",
            "alpha beta gamma delta",
            true,
        ),
        (
            "is in red color",
            "alpha beta gamma delta epsilon zeta",
            true,
        ),
        (
            "are in red color",
            "alpha beta gamma delta epsilon zeta",
            true,
        ),
        (
            "in bright red color",
            "alpha beta gamma delta epsilon zeta",
            true,
        ),
        (
            "was in red color",
            "alpha beta gamma delta epsilon zeta",
            false,
        ),
        ("is in red color", "alpha beta gamma delta epsilon", false),
        ("one two three", "alpha beta gamma delta", false),
        ("one two three four five six", "alpha beta gamma", false),
    ];
    for (old, new, peel) in cases {
        let left = revision("ins", "lead");
        let tail = revision("del", old);
        let right = revision("ins", new);
        let punct = "<w:r><w:t>!</w:t></w:r>";
        let fragment = format!("<w:p>{joining}{left}{punct}{tail}</w:p><w:p>{right}</w:p>");
        let (mut dom, root, body) = document(&fragment);
        let accepted = projected_body(&mut dom, root, true);
        let rejected = projected_body(&mut dom, root, false);
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(projected_body(&mut dom, root, true), accepted);
        assert_eq!(projected_body(&mut dom, root, false), rejected);
        let expected = if peel {
            format!("<w:p>{joining}{left}{punct}</w:p><w:p>{right}{tail}</w:p>")
        } else {
            fragment
        };
        expect_body(&dom, body, &expected);
    }
    for list_word in ["bullet", "item", "point", "first", "second", "third"] {
        unchanged(
            &format!(
                "<w:p>{joining}{}{}</w:p>{}",
                revision("ins", "lead"),
                revision("del", "is in red color"),
                pure(
                    "ins",
                    &format!("{list_word} alpha beta gamma delta epsilon")
                )
            ),
            peel_trailing_del_from_mix_into_following_pure_ins,
        );
    }
    for count in [20, 21] {
        let text = std::iter::repeat_n("old", count)
            .collect::<Vec<_>>()
            .join(" ");
        let lead = revision("ins", "lead");
        let tail = revision("del", &text);
        let right = revision("ins", "alpha beta gamma delta");
        let original = format!("<w:p>{joining}{lead}{tail}</w:p><w:p>{right}</w:p>");
        let (mut dom, root, body) = document(&original);
        let accepted = projected_body(&mut dom, root, true);
        let rejected = projected_body(&mut dom, root, false);
        peel_trailing_del_from_mix_into_following_pure_ins(&mut dom, root);
        assert_eq!(projected_body(&mut dom, root, true), accepted);
        assert_eq!(projected_body(&mut dom, root, false), rejected);
        let expected = if count == 20 {
            format!("<w:p>{joining}{lead}</w:p><w:p>{right}{tail}</w:p>")
        } else {
            original
        };
        expect_body(&dom, body, &expected);
    }
}

#[test]
fn short_deletion_restore_checks_filename_and_alnum_length_boundaries() {
    for (anchor, old_len, new_len, swap) in [
        ("Title", 40, 41, true),
        ("Title", 41, 42, false),
        ("Title", 4, 4, false),
        ("Title", 0, 10, false),
        ("FILE_name", 4, 10, false),
        ("report.DOCX", 4, 10, false),
        ("report.doc", 4, 10, false),
    ] {
        let anchor = format!(
            "<w:p>{}{}</w:p>",
            revision("ins", anchor),
            revision("del", "previous")
        );
        let ins = pure("ins", &"n".repeat(new_len));
        let del = pure("del", &"o".repeat(old_len));
        let original = format!("{anchor}{ins}{del}<w:sectPr/>");
        let (mut dom, root, body) = document(&original);
        restore_short_del_before_long_ins(&mut dom, root);
        let expected = if swap {
            format!("{anchor}{del}{ins}<w:sectPr/>")
        } else {
            original
        };
        expect_body(&dom, body, &expected);
    }
    unchanged(
        &format!(
            "{}{}",
            pure("ins", "long inserted title"),
            pure("del", "old")
        ),
        restore_short_del_before_long_ins,
    );
}

#[test]
fn demo_fold_strips_only_revised_pilcrow_and_uses_nearest_numbered_heading() {
    for keep_jc in [false, true] {
        let jc = if keep_jc {
            "<w:jc w:val=\"center\"/>"
        } else {
            ""
        };
        let heading = marked("ins", "1. Latest heading", jc);
        let del = marked("del", "Title Demo", "<w:pStyle w:val=\"Title\"/>");
        let prefix = format!(
            "{}{}",
            pure("ins", "1. Earlier heading"),
            pure("ins", "intro")
        );
        let suffix = (0..4)
            .map(|n| pure("ins", &format!("Body {n}")))
            .collect::<String>();
        let fragment = format!("{prefix}{heading}{del}{suffix}");
        let (mut dom, root, body) = document(&fragment);
        fold_midstream_demo_title_into_numbered_heading(&mut dom, root);
        let configuration = WmlComparerSettings::default();
        let live_props = format!(
            "<w:pPr>{jc}<w:pPrChange w:id=\"9\" w:author=\"{}\" w:date=\"{}\"><w:pPr><w:pStyle w:val=\"Title\"/></w:pPr></w:pPrChange></w:pPr>",
            configuration.author_for_revisions, configuration.date_time_for_revisions,
        );
        expect_body(
            &dom,
            body,
            &format!(
                "{prefix}<w:p>{live_props}{}{}</w:p>{suffix}",
                revision("ins", "1. Latest heading"),
                revision("del", "Title Demo")
            ),
        );
    }
}

#[test]
fn demo_title_fold_history_preserves_complete_old_layout_and_current_attribution() {
    for tracking in [false, true] {
        let old_properties = "<w:spacing w:line=\"276\"/><w:ind w:left=\"720\"/>";
        let new_properties = "<w:pBdr><w:bottom w:val=\"single\"/></w:pBdr><w:spacing w:before=\"300\" w:after=\"80\"/>";
        let fragment = format!(
            "{}{}{}{}",
            pure("ins", "Introduction").repeat(2),
            marked("ins", "1. What this is", new_properties),
            marked("del", "Old Demo", old_properties),
            pure("ins", "Revised body").repeat(4)
        );
        let (mut dom, root, body) = document(&fragment);
        let mut configuration = settings();
        configuration.detect_format_changes = tracking;
        let mut id = 501;
        fold_midstream_demo_title_into_numbered_heading_with_settings(
            &mut dom,
            root,
            &configuration,
            &mut id,
        );
        let heading = dom.elements(body, Some(&W::p()))[2];
        let properties = dom.element(heading, &W::p_pr()).unwrap();
        let history = dom.element(properties, &W::p_pr_change());
        assert_eq!(history.is_some(), tracking);
        assert_eq!(id, if tracking { 502 } else { 501 });
        if let Some(history) = history {
            assert_eq!(dom.attribute(history, &W::id()), Some("501"));
            assert_eq!(dom.attribute(history, &W::author()), Some("Fallback"));
            assert_eq!(
                dom.attribute(history, &W::date()),
                Some("2000-01-01T00:00:00Z")
            );
            let snapshot = dom.element(history, &W::p_pr()).unwrap();
            let (expected, expected_root, _) =
                document(&format!("<w:p><w:pPr>{old_properties}</w:pPr></w:p>"));
            let expected_snapshot = expected.descendants(expected_root, Some(&W::p_pr()))[0];
            same_tree(&dom, snapshot, &expected, expected_snapshot);
        }
        assert_eq!(dom.descendants(heading, Some(&W::del_text())).len(), 1);
        assert_eq!(markup_text(&dom, heading), "1. What this isOld Demo");
    }
}

#[test]
fn demo_fold_heading_text_limits_and_table_barriers_are_exact() {
    for (text, fold) in [
        ("1. short".to_string(), true),
        ("1 tiny".to_string(), false),
        (format!("1 {}", "a".repeat(78)), true),
        (format!("1 {}", "a".repeat(79)), false),
    ] {
        let head = pure("ins", &text);
        let del = pure("del", "Title Demo");
        let prefix = (0..2)
            .map(|n| pure("ins", &format!("Intro {n}")))
            .collect::<String>();
        let suffix = (0..4)
            .map(|n| pure("ins", &format!("Body {n}")))
            .collect::<String>();
        let fragment = format!("{prefix}{head}{del}{suffix}");
        let (mut dom, root, body) = document(&fragment);
        fold_midstream_demo_title_into_numbered_heading(&mut dom, root);
        let expected = if fold {
            format!(
                "{prefix}<w:p>{}{}</w:p>{suffix}",
                revision("ins", &text),
                revision("del", "Title Demo")
            )
        } else {
            fragment
        };
        expect_body(&dom, body, &expected);
    }
    let prefix = (0..5)
        .map(|n| pure("ins", &format!("Intro {n}")))
        .collect::<String>();
    unchanged(
        &format!(
            "{}<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>{prefix}{}{}",
            pure("ins", "1. Heading"),
            pure("del", "Title Demo"),
            pure("ins", "after")
        ),
        fold_midstream_demo_title_into_numbered_heading,
    );
    // A live equal token in the leading MIX anchors the replacement.
    let anchor = format!(
        "<w:p>{}<w:r><w:t>equal</w:t></w:r>{}</w:p>",
        revision("ins", "new"),
        revision("del", "old")
    );
    unchanged(
        &format!(
            "{anchor}{prefix}{}{}{}",
            pure("ins", "1. Heading"),
            pure("del", "Title Demo"),
            pure("ins", "after")
        ),
        fold_midstream_demo_title_into_numbered_heading,
    );
}

#[test]
fn title_spacers_are_kept_once_and_prose_spacers_are_removed() {
    for (text, props, empties, kept) in [
        ("Title Demo", "", 1, 1),
        ("Title Demo", "", 2, 1),
        ("Contract Review", "", 2, 1),
        (
            "one two three four five",
            "<w:pStyle w:val=\"Heading1\"/>",
            2,
            1,
        ),
        ("one two three four five", "", 2, 0),
        ("one two three four five", "", 3, 3),
    ] {
        let lead = pure("ins", "new content");
        let empty = "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr></w:p>";
        let del = marked("del", text, props);
        let (mut dom, root, body) =
            document(&format!("{lead}{}{del}<w:sectPr/>", empty.repeat(empties)));
        strip_empty_pure_ins_before_trailing_pure_dels(&mut dom, root);
        expect_body(
            &dom,
            body,
            &format!("{lead}{}{del}<w:sectPr/>", empty.repeat(kept)),
        );
    }
}

#[test]
fn list_contamination_cleanup_keeps_real_lists_and_alignment() {
    let spacing = "<w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>";
    for jc in ["both", "distribute", "center", "right"] {
        for list_props in ["", "<w:numPr/>", "<w:pStyle w:val=\"listparagraph\"/>"] {
            let donor = marked("ins", "a", spacing);
            let align = format!("<w:jc w:val=\"{jc}\"/>");
            let props = format!("{list_props}{spacing}{align}");
            let target = marked("del", "old", &props)
                .replace("<w:del w:id=\"8\"/>", "<w:del w:id=\"8\"/><w:b/>");
            let suffix = "<w:p><w:r><w:t>live tail</w:t></w:r></w:p>";
            let (mut dom, root, body) = document(&format!("{donor}{target}{suffix}"));
            strip_list_layout_from_mid_pure_del(&mut dom, root);
            let expected_target = if list_props.is_empty() {
                let align = if matches!(jc, "both" | "distribute") {
                    ""
                } else {
                    &align
                };
                marked("del", "old", align)
            } else {
                target
            };
            expect_body(&dom, body, &format!("{donor}{expected_target}{suffix}"));
        }
    }
}

#[test]
fn list_snug_spacing_requires_three_identical_short_tokens_and_respects_schema_tail() {
    for (text, count, inject) in [
        ("abcdefghijkl", 3, true),
        ("abcdefghijklm", 3, false),
        ("a", 2, false),
    ] {
        for tail in [
            "",
            "<w:pPrChange><w:pPr/></w:pPrChange>",
            "<w:rPr><w:ins/></w:rPr>",
        ] {
            let num = "<w:numPr><w:numId w:val=\"1\"/></w:numPr>";
            let para = format!(
                "<w:p><w:pPr>{num}{tail}</w:pPr>{}</w:p>",
                revision("ins", text)
            );
            let (mut dom, root, body) = document(&para.repeat(count));
            ensure_pure_i_list_snug_spacing(&mut dom, root);
            let expected = if inject {
                format!("<w:p><w:pPr>{num}<w:spacing w:before=\"0\" w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>{tail}</w:pPr>{}</w:p>", revision("ins", text)).repeat(count)
            } else {
                para.repeat(count)
            };
            expect_body(&dom, body, &expected);
        }
    }
}

#[test]
fn trailing_blank_spacing_records_history_only_when_source_differs() {
    for tail in ["", "<w:rPr/>", "<w:rPr><w:b/></w:rPr>"] {
        for equal_source in [false, true] {
            let spacing = "<w:spacing w:before=\"80\" w:line=\"240\"/>";
            let fragment = format!("<w:p><w:pPr>{spacing}{tail}</w:pPr></w:p><w:sectPr/>");
            let (mut dom, root, body) = document(&fragment);
            let sp = dom.descendants(root, Some(&W::spacing_el()))[0];
            let source = sorted_attributes(&dom, sp);
            let mut id = 50;
            trailing_empty_spacing_to_pprchange(
                &mut dom,
                root,
                equal_source.then_some(source.as_slice()),
                &settings(),
                &mut id,
            );
            let expected = if equal_source {
                fragment
            } else {
                let kept = if tail == "<w:rPr/>" { "" } else { tail };
                format!(
                    "<w:p><w:pPr>{kept}<w:pPrChange w:id=\"50\" w:author=\"Fallback\" w:date=\"2000-01-01T00:00:00Z\"><w:pPr>{spacing}</w:pPr></w:pPrChange></w:pPr></w:p><w:sectPr/>"
                )
            };
            expect_body(&dom, body, &expected);
            assert_eq!(id, if equal_source { 50 } else { 51 });
        }
    }
    for extra in [
        "<w:jc w:val=\"right\"/>",
        "<w:pPrChange><w:pPr/></w:pPrChange>",
    ] {
        let fragment = format!("<w:p><w:pPr><w:spacing w:line=\"240\"/>{extra}</w:pPr></w:p>");
        let (mut dom, root, _) = document(&fragment);
        let before = dom.serialize_element(root);
        let mut id = 50;
        trailing_empty_spacing_to_pprchange(&mut dom, root, None, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 50);
    }
}

#[test]
fn table_chrome_adds_look_before_history_and_handles_all_row_positions() {
    let look = "<w:tblLook w:val=\"04A0\" w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"1\" w:lastColumn=\"0\" w:noHBand=\"0\" w:noVBand=\"1\"/>";
    let ex = "<w:tblPrEx><w:tblCellMar><w:top w:w=\"0\" w:type=\"dxa\"/><w:bottom w:w=\"0\" w:type=\"dxa\"/></w:tblCellMar></w:tblPrEx>";
    let history = "<w:tblPrChange><w:tblPr><w:tblW w:w=\"400\"/></w:tblPr></w:tblPrChange>";
    let fragment = format!(
        "<w:tbl><w:tblPr><w:tblCellMar/>{history}</w:tblPr><w:tr><w:trPr/><w:tc><w:p/></w:tc></w:tr><w:tr><w:tc><w:p/></w:tc></w:tr><w:tr/><w:tr><w:tblPrEx/><w:tc><w:p/></w:tc></w:tr></w:tbl>"
    );
    let (mut dom, root, body) = document(&fragment);
    align_word_table_and_comment_chrome(&mut dom, root);
    expect_body(
        &dom,
        body,
        &format!(
            "<w:tbl><w:tblPr><w:tblCellMar/>{look}{history}</w:tblPr><w:tr>{ex}<w:trPr/><w:tc><w:p/></w:tc></w:tr><w:tr>{ex}<w:tc><w:p/></w:tc></w:tr><w:tr>{ex}</w:tr><w:tr><w:tblPrEx/><w:tc><w:p/></w:tc></w:tr></w:tbl>"
        ),
    );
    let once = dom.serialize_element(root);
    align_word_table_and_comment_chrome(&mut dom, root);
    assert_eq!(dom.serialize_element(root), once);
}

#[test]
fn comment_chrome_fills_each_missing_property_independently_and_ignores_orphans() {
    for mask in 0u8..8 {
        let style = if mask & 1 != 0 {
            "<w:rStyle w:val=\"AnnotationReference\"/>"
        } else {
            ""
        };
        let sz = if mask & 2 != 0 {
            "<w:sz w:val=\"28\"/>"
        } else {
            ""
        };
        let cs = if mask & 4 != 0 {
            "<w:szCs w:val=\"30\"/>"
        } else {
            ""
        };
        let pr = format!("{style}{sz}{cs}");
        let fragment = format!(
            "<w:p><w:r><w:rPr>{pr}</w:rPr><w:commentReference w:id=\"1\"/></w:r><w:commentReference w:id=\"2\"/></w:p>"
        );
        let (mut dom, root, body) = document(&fragment);
        align_word_table_and_comment_chrome(&mut dom, root);
        let mut expected = pr;
        if mask & 1 == 0 {
            expected.push_str("<w:rStyle w:val=\"CommentReference\"/>");
        }
        if mask & 2 == 0 {
            expected.push_str("<w:sz w:val=\"20\"/>");
        }
        if mask & 4 == 0 {
            expected.push_str("<w:szCs w:val=\"20\"/>");
        }
        expect_body(
            &dom,
            body,
            &format!(
                "<w:p><w:r><w:rPr>{expected}</w:rPr><w:commentReference w:id=\"1\"/></w:r><w:commentReference w:id=\"2\"/></w:p>"
            ),
        );
    }
}

#[test]
fn deleted_textbox_attribution_is_inherited_per_attribute_and_runs_join_across_text() {
    for (author, date) in [(false, false), (true, false), (false, true), (true, true)] {
        let attribution = format!(
            "{}{}",
            if author { " w:author=\"Anchor\"" } else { "" },
            if date { " w:date=\"Earlier\"" } else { "" }
        );
        let fragment = format!(
            "<w:p><w:del{attribution}><w:r><w:drawing><w:txbxContent><w:p><w:pPr><w:sectPr/></w:pPr><w:r><w:instrText>FIELD</w:instrText></w:r>separator<w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p></w:txbxContent></w:drawing></w:r></w:del></w:p>"
        );
        let (mut dom, root, body) = document(&fragment);
        let mut id = 70;
        delete_stories_of_deleted_text_boxes(&mut dom, root, &settings(), &mut id);
        let author = if author { "Anchor" } else { "Fallback" };
        let date = if date {
            "Earlier"
        } else {
            "2000-01-01T00:00:00Z"
        };
        let stamp = format!("w:author=\"{author}\" w:date=\"{date}\"");
        expect_body(
            &dom,
            body,
            &format!(
                "<w:p><w:del{attribution}><w:r><w:drawing><w:txbxContent><w:p><w:pPr><w:rPr><w:del w:id=\"71\" {stamp}/></w:rPr><w:sectPr/></w:pPr><w:del w:id=\"70\" {stamp}><w:r><w:instrText>FIELD</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:del>separator</w:p></w:txbxContent></w:drawing></w:r></w:del></w:p>"
            ),
        );
        assert_eq!(id, 72);
    }
}

#[test]
fn paragraph_deletion_mark_inserts_before_each_schema_tail_and_preserves_format() {
    for tail in ["<w:sectPr/>", "<w:pPrChange><w:pPr/></w:pPrChange>", ""] {
        let (mut dom, _, body) = document(&format!(
            "<w:p><w:pPr><w:jc w:val=\"right\"/>{tail}</w:pPr></w:p><w:del w:author=\"Owner\"/>"
        ));
        let p = dom.elements(body, Some(&W::p()))[0];
        let like = dom.element(body, &W::del()).unwrap();
        let mut id = 10;
        mark_paragraph_mark_deleted(&mut dom, p, like, &settings(), &mut id);
        expect_body(
            &dom,
            body,
            &format!(
                "<w:p><w:pPr><w:jc w:val=\"right\"/><w:rPr><w:del w:id=\"10\" w:author=\"Owner\" w:date=\"2000-01-01T00:00:00Z\"/></w:rPr>{tail}</w:pPr></w:p><w:del w:author=\"Owner\"/>"
            ),
        );
        assert_eq!(id, 11);
    }
}

#[test]
fn structural_property_classifiers_ignore_history_and_require_one_alignment() {
    for (props, structural, jc_only) in [
        ("", false, false),
        (
            "<w:rPr/><w:sectPr/><w:pPrChange/><pt:Scratch/>",
            false,
            false,
        ),
        (
            "<w:jc/><w:rPr/><w:sectPr/><w:pPrChange/><pt:Scratch/>",
            true,
            true,
        ),
        ("<w:jc/><w:jc/>", true, false),
        ("<w:spacing/><w:jc/>", true, false),
        ("<w:jc/><w:spacing/>", true, false),
    ] {
        let (dom, _, body) = document(&format!("<w:p><w:pPr>{props}</w:pPr></w:p>"));
        let pr = dom.descendants(body, Some(&W::p_pr()))[0];
        assert_eq!(ppr_has_structural_props(&dom, pr), structural, "{props}");
        assert_eq!(ppr_is_jc_only(&dom, pr), jc_only, "{props}");
    }
}

#[test]
fn conjoin_property_selection_retains_format_and_does_not_duplicate_history() {
    for (ins_props, del_props, inserted_wins) in [
        (
            "<w:jc w:val=\"center\"/>",
            "<w:spacing w:before=\"80\"/>",
            true,
        ),
        ("", "<w:spacing w:before=\"80\"/>", false),
        ("", "", true),
    ] {
        for has_history in [false, true] {
            let history = if has_history {
                "<w:pPrChange w:id=\"99\"><w:pPr/></w:pPrChange>"
            } else {
                ""
            };
            let ins = format!(
                "<w:pPr pt:Status=\"Inserted\">{ins_props}<w:rPr><w:ins/><w:del/><w:b/></w:rPr>{history}</w:pPr>"
            );
            let del = format!(
                "<w:pPr pt:Status=\"Deleted\">{del_props}<w:rPr><w:ins/><w:del/><w:i/></w:rPr></w:pPr>"
            );
            let text = revision("ins", "content");
            let (mut dom, root, _) =
                document(&format!("<w:p w:rsidR=\"123\">{ins}{del}{text}</w:p>"));
            let result = conjoin_paragraph_marks(&mut dom, root, &settings());
            let body = dom.element(result, &W::body()).unwrap();
            let props = if inserted_wins {
                let history = if has_history {
                    history.to_string()
                } else {
                    format!(
                        "<w:pPrChange w:id=\"0\" w:author=\"Fallback\" w:date=\"2000-01-01T00:00:00Z\"><w:pPr>{del_props}<w:rPr><w:i/></w:rPr></w:pPr></w:pPrChange>"
                    )
                };
                format!("<w:pPr>{ins_props}<w:rPr><w:b/></w:rPr>{history}</w:pPr>")
            } else {
                format!("<w:pPr>{del_props}<w:rPr><w:ins/><w:del/><w:i/></w:rPr></w:pPr>")
            };
            expect_body(
                &dom,
                body,
                &format!("<w:p w:rsidR=\"123\">{props}{text}</w:p>"),
            );
        }
    }
}

/// An empty field is a whole atom (no run), so the run-level status
/// pass never wrapped it: a deleted `PAGE` stayed live in its deleted
/// paragraph and Word 16.115 hung on the footer (b15, 0.12.0 sample).
/// A moved empty field travels as the moved runs do: range markers
/// around a `w:moveFrom` / `w:moveTo` holding the complex field (its
/// code stays `w:instrText`, as moved text stays `w:t`). Left live, the
/// field survived at both ends of every accept and reject.
#[test]
fn moved_empty_field_becomes_a_complex_field_in_the_move() {
    for (status, kind) in [("MovedSource", "moveFrom"), ("MovedDestination", "moveTo")] {
        let (mut dom, root, _) = document(&format!(
            "<w:p><w:fldSimple pt:Status=\"{status}\" pt:MoveName=\"move7\" w:instr=\"PAGE\"/></w:p>"
        ));
        let mut id = 1;
        let result = mark_content_as_deleted_or_inserted(&mut dom, root, &settings(), &mut id);
        let p = dom.descendants(result, Some(&W::p()))[0];
        let kids: Vec<String> = dom
            .elements(p, None)
            .into_iter()
            .map(|k| dom.name(k).unwrap().local_name().to_string())
            .collect();
        assert_eq!(
            kids,
            [
                format!("{kind}RangeStart"),
                kind.to_string(),
                format!("{kind}RangeEnd")
            ],
            "{status}"
        );
        let start = dom.elements(p, None)[0];
        assert_eq!(dom.attribute(start, &W::name("name")), Some("move7"));
        let moved = dom.elements(p, None)[1];
        assert_eq!(dom.elements(moved, Some(&W::r())).len(), 4);
        assert_eq!(
            dom.value(dom.descendants(moved, Some(&W::instr_text()))[0]),
            "PAGE"
        );
        assert!(
            dom.descendants(p, None)
                .iter()
                .all(|&n| dom.attribute(n, &PT::status()).is_none())
        );
    }
}

#[test]
fn deleted_or_inserted_empty_field_becomes_a_complex_field_in_the_revision() {
    for (status, wrap, code) in [
        ("Deleted", "del", "delInstrText"),
        ("Inserted", "ins", "instrText"),
    ] {
        let (mut dom, root, _) = document(&format!(
            "<w:p><w:fldSimple pt:Status=\"{status}\" w:instr=\"NUMPAGES\"/></w:p>"
        ));
        let mut id = 1;
        let result = mark_content_as_deleted_or_inserted(&mut dom, root, &settings(), &mut id);
        assert!(dom.descendants(result, Some(&W::fld_simple())).is_empty());
        let p = dom.descendants(result, Some(&W::p()))[0];
        let kids = dom.elements(p, None);
        assert_eq!(kids.len(), 1, "{status}");
        assert!(dom.name_is(kids[0], &W::name(wrap)));
        assert_eq!(dom.elements(kids[0], Some(&W::r())).len(), 4);
        assert_eq!(
            dom.value(dom.descendants(p, Some(&W::name(code)))[0]),
            "NUMPAGES"
        );
        assert!(
            dom.descendants(p, None)
                .iter()
                .all(|&n| dom.attribute(n, &PT::status()).is_none())
        );
    }
}

#[test]
fn default_spacing_strips_only_the_selected_source_default_and_exact_shape() {
    for kind in ["ins", "del"] {
        for defaults in [
            (Some("240"), Some("276")),
            (Some("276"), Some("240")),
            (None, None),
        ] {
            for (attrs, shape) in [
                ("w:line=\"276\"", true),
                ("w:line=\"276\" w:after=\"200\" w:lineRule=\"auto\"", true),
                ("w:line=\"276\" w:after=\"0\"", false),
                ("w:line=\"276\" w:before=\"0\"", false),
                ("w:line=\"276\" w:lineRule=\"exact\"", false),
                ("w:line=\"240\"", false),
            ] {
                let spacing = format!("<w:spacing {attrs}/>");
                let (mut dom, root, body) = document(&marked(kind, "two words", &spacing));
                strip_redundant_demo_default_spacing(&mut dom, root, defaults);
                let selected = if kind == "del" {
                    defaults.0
                } else {
                    defaults.1
                };
                let strip = shape && selected.is_none_or(|s| s == "276");
                expect_body(
                    &dom,
                    body,
                    &marked(kind, "two words", if strip { "" } else { &spacing }),
                );
            }
        }
    }
}

#[test]
fn deleted_heading_spacing_threshold_keeps_missing_metrics_and_real_styles() {
    for (before, after, line, style, strip) in [
        ("359", "120", "240", "", false),
        ("360", "120", "240", "", true),
        ("360", "", "240", "", false),
        ("360", "120", "", "", false),
        ("360", "120", "240", "<w:pStyle w:val=\"Heading1\"/>", false),
        ("bad", "120", "240", "", false),
    ] {
        let sp =
            format!("<w:spacing w:before=\"{before}\" w:after=\"{after}\" w:line=\"{line}\"/>");
        let (mut dom, root, body) = document(&marked("del", "old body", &format!("{style}{sp}")));
        strip_redundant_demo_default_spacing(&mut dom, root, (None, None));
        expect_body(
            &dom,
            body,
            &marked(
                "del",
                "old body",
                &format!("{style}{}", if strip { "" } else { &sp }),
            ),
        );
    }
}

#[test]
fn unrecorded_spacing_guards_preserve_all_explicit_vertical_metrics() {
    for extra in [
        "before",
        "after",
        "beforeLines",
        "afterLines",
        "beforeAutospacing",
        "afterAutospacing",
    ] {
        let sp = format!("<w:spacing w:line=\"276\" w:{extra}=\"0\"/>");
        let fragment = marked("del", "old", &sp);
        let (mut dom, root, _) = document(&fragment);
        let before = dom.serialize_element(root);
        strip_unrecorded_word_defaults(&mut dom, root, Some("276"));
        assert_eq!(dom.serialize_element(root), before);
    }
    for (original, inserted_body, strip) in [
        (None, false, true),
        (Some("276"), false, true),
        (Some("240"), false, false),
        (None, true, false),
    ] {
        let ins = if inserted_body {
            revision("ins", "new")
        } else {
            String::new()
        };
        let sp = "<w:spacing w:line=\"276\"/>";
        let fragment = format!(
            "<w:p><w:pPr>{sp}<w:rPr><w:del/></w:rPr></w:pPr>{}{ins}</w:p>",
            revision("del", "old")
        );
        let (mut dom, root, body) = document(&fragment);
        strip_unrecorded_word_defaults(&mut dom, root, original);
        expect_body(
            &dom,
            body,
            &format!(
                "<w:p><w:pPr>{}<w:rPr><w:del/></w:rPr></w:pPr>{}{ins}</w:p>",
                if strip { "" } else { sp },
                revision("del", "old")
            ),
        );
    }
}

#[test]
fn deleted_history_flattening_uses_both_field_and_text_fingerprint() {
    for side in [FlattenSide::Original, FlattenSide::Revised] {
        for shared in [false, true] {
            for attributed in [false, true] {
                let attrs = if attributed {
                    " w:author=\"Old\" w:date=\"Earlier\""
                } else {
                    ""
                };
                let fragment = format!(
                    "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr><w:del{attrs}>separator<w:r><w:delText>old</w:delText><w:delInstrText>FIELD</w:delInstrText></w:r></w:del></w:p>"
                );
                let (mut dom, _, body) = document(&fragment);
                let other: HashSet<String> = if shared {
                    ["oldFIELD".to_string()].into()
                } else {
                    ["old".to_string()].into()
                };
                assert_eq!(
                    pending_deletion_texts(&dom, body),
                    HashSet::from(["oldFIELD".to_string()])
                );
                flatten_tracked_deletions(&mut dom, body, side, Some(&other));
                let mark = if side == FlattenSide::Original {
                    ""
                } else {
                    "<w:del/>"
                };
                let stamp = if side == FlattenSide::Original && !shared {
                    super::super::PREDELETE_STAMP_ORIG
                } else {
                    super::super::PREDELETE_STAMP_REV
                };
                let attribution = if attributed {
                    " pt:PreDelAuthor=\"Old\" pt:PreDelDate=\"Earlier\""
                } else {
                    ""
                };
                expect_body(
                    &dom,
                    body,
                    &format!(
                        "<w:p><w:pPr><w:rPr>{mark}</w:rPr></w:pPr>separator<w:r pt:PreDelete=\"{stamp}\"{attribution}><w:t>old</w:t><w:instrText>FIELD</w:instrText></w:r></w:p>"
                    ),
                );
            }
        }
    }
}

#[test]
fn predelete_conversion_missing_attribution_defaults_without_changing_neighbor_runs() {
    for (author, date) in [(false, false), (true, false), (false, true), (true, true)] {
        let attrs = format!(
            "{}{}",
            if author {
                " pt:PreDelAuthor=\"Old\""
            } else {
                ""
            },
            if date {
                " pt:PreDelDate=\"Earlier\""
            } else {
                ""
            }
        );
        let (mut dom, root, body) = document(&format!(
            "<w:p><w:r><w:t>before</w:t></w:r><w:r pt:PreDelete=\"1\"{attrs}><w:t>old</w:t><w:instrText>FIELD</w:instrText></w:r><w:r><w:t>after</w:t></w:r></w:p>"
        ));
        let mut id = 90;
        convert_stamped_predeletes(&mut dom, root, &settings(), &mut id);
        let a = if author { "Old" } else { "Fallback" };
        let d = if date {
            "Earlier"
        } else {
            "2000-01-01T00:00:00Z"
        };
        expect_body(
            &dom,
            body,
            &format!(
                "<w:p><w:r><w:t>before</w:t></w:r><w:del w:id=\"90\" w:author=\"{a}\" w:date=\"{d}\"><w:r><w:delText>old</w:delText><w:delInstrText>FIELD</w:delInstrText></w:r></w:del><w:r><w:t>after</w:t></w:r></w:p>"
            ),
        );
        assert_eq!(id, 91);
    }
}

#[test]
fn short_label_mesh_rejects_empty_oversized_and_mismatching_labels_verbatim() {
    for (ins, del) in [
        ("", "Lvl 1 a"),
        ("a", ""),
        ("!", "Lvl 1 a"),
        ("abc", "Lvl 1 abc"),
        ("a", "1 a"),
        ("a", "Lvl 1 b"),
    ] {
        let fragment = format!(
            "<w:p>{}{}</w:p>",
            revision("ins", ins),
            revision("del", del)
        );
        let (mut dom, _, body) = document(&fragment);
        let p = dom.element(body, &W::p()).unwrap();
        let before = dom.serialize_element(p);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(dom.serialize_element(p), before, "{ins:?} / {del:?}");
    }
}

#[test]
fn label_fold_creates_missing_properties_without_inventing_source_geometry() {
    for source_props in [
        "",
        "<w:pPr><w:spacing w:line=\"240\"/><w:ind w:left=\"80\"/></w:pPr>",
    ] {
        for target_has_body in [false, true] {
            let inserted = revision("ins", "x");
            let deleted = if target_has_body {
                revision("del", "old unrelated body")
            } else {
                String::new()
            };
            let (mut dom, _, body) = document(&format!(
                "<w:p>{source_props}{inserted}</w:p><w:p>{deleted}</w:p>"
            ));
            let ps = dom.elements(body, Some(&W::p()));
            fold_short_label_ins_into_del(&mut dom, ps[0], ps[1]);
            expect_body(
                &dom,
                body,
                &format!("<w:p>{source_props}{inserted}{deleted}</w:p>"),
            );
        }
    }
}

#[test]
fn for_word_peel_preserves_run_format_and_uses_equal_format_for_shared_connector() {
    let left = format!(
        "<w:p>{}<w:ins w:id=\"7\" w:author=\"Author\" w:date=\"Pinned\"><w:r><w:rPr><w:b/></w:rPr><w:t>new for next</w:t></w:r></w:ins></w:p>",
        revision("del", "old")
    );
    let right = format!(
        "<w:p>{}<w:r><w:rPr><w:i/></w:rPr><w:t>equal</w:t></w:r>{}</w:p>",
        revision("del", "previous for"),
        revision("ins", "suffix")
    );
    let (mut dom, root, body) = document(&format!("{left}{right}"));
    peel_trailing_for_word_onto_next_mix(&mut dom, root);
    let left = format!(
        "<w:p>{}<w:ins w:id=\"7\" w:author=\"Author\" w:date=\"Pinned\"><w:r><w:rPr><w:b/></w:rPr><w:t>new</w:t></w:r></w:ins></w:p>",
        revision("del", "old")
    );
    let right = format!(
        "<w:p>{}<w:r><w:rPr><w:i/></w:rPr><w:t xml:space=\"preserve\"> for </w:t></w:r><w:ins w:id=\"1\" w:author=\"Author\" w:date=\"Pinned\"><w:r><w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">next </w:t></w:r></w:ins><w:r><w:rPr><w:i/></w:rPr><w:t>equal</w:t></w:r>{}</w:p>",
        revision("del", "previous"),
        revision("ins", "suffix")
    );
    expect_body(&dom, body, &format!("{left}{right}"));
}

#[test]
fn for_word_guard_matrix_requires_residual_and_leading_deleted_carrier() {
    for (ins, del) in [
        ("new for two words", "old for"),
        ("new for next!", "old for"),
        ("new for next", "old for later"),
        ("new for next", " for"),
    ] {
        unchanged(
            &format!(
                "<w:p>{}{}</w:p><w:p>{}{}</w:p>",
                revision("ins", ins),
                revision("del", "left"),
                revision("del", del),
                revision("ins", "right")
            ),
            peel_trailing_for_word_onto_next_mix,
        );
    }
    unchanged(
        &format!(
            "<w:p>{}{}</w:p><w:p>{}{}</w:p>",
            revision("ins", "new for next"),
            revision("del", "left"),
            revision("ins", "right"),
            revision("del", "old for")
        ),
        peel_trailing_for_word_onto_next_mix,
    );
}

#[test]
fn borrowed_period_moves_to_finished_side_and_finishes_the_continuation() {
    for inserted_continues in [false, true] {
        let ins = if inserted_continues {
            " lower"
        } else {
            "Complete"
        };
        let del = if inserted_continues {
            "Complete"
        } else {
            " lower"
        };
        let left = "<w:p><w:r><w:t>Prefix</w:t></w:r><w:r><w:t>.</w:t></w:r></w:p>";
        let (mut dom, root, body) = document(&format!(
            "{left}<w:p>{}{}</w:p>",
            revision("ins", ins),
            revision("del", del)
        ));
        let mut id = 12;
        repair_borrowed_sentence_period(&mut dom, root, &settings(), &mut id);
        let wrapper = if inserted_continues { "del" } else { "ins" };
        // The period leaf retains w:t even when its run is wrapped in del.
        let expected_left = format!(
            "<w:p><w:r><w:t>Prefix</w:t></w:r><w:{wrapper} w:id=\"12\" w:author=\"Fallback\" w:date=\"2000-01-01T00:00:00Z\"><w:r><w:t>.</w:t></w:r></w:{wrapper}></w:p>"
        );
        let ins = if inserted_continues { " lower." } else { ins };
        let del = if inserted_continues { del } else { " lower." };
        expect_body(
            &dom,
            body,
            &format!(
                "{expected_left}<w:p>{}{}</w:p>",
                revision("ins", ins),
                revision("del", del)
            ),
        );
        assert_eq!(id, 13);
    }
}

#[test]
fn compatibility_defaults_keep_nonzero_vertical_margins_and_existing_look() {
    for side in ["top", "bottom"] {
        for width in ["0", "80"] {
            let look = "<w:tblLook w:val=\"custom\"/>";
            let mar = format!("<w:tblCellMar><w:{side} w:w=\"{width}\"/></w:tblCellMar>");
            let fragment = format!("<w:tbl><w:tblPr>{mar}{look}</w:tblPr><w:tr/></w:tbl>");
            let (mut dom, root, body) = document(&fragment);
            align_word_table_and_comment_chrome(&mut dom, root);
            let expected = if width == "80" {
                fragment
            } else {
                format!(
                    "<w:tbl><w:tblPr>{mar}{look}</w:tblPr><w:tr><w:tblPrEx><w:tblCellMar><w:top w:w=\"0\" w:type=\"dxa\"/><w:bottom w:w=\"0\" w:type=\"dxa\"/></w:tblCellMar></w:tblPrEx></w:tr></w:tbl>"
                )
            };
            expect_body(&dom, body, &expected);
        }
    }
}

#[test]
fn shared_title_mesh_rejects_author_mismatch_repeats_and_nonfinal_shared_tokens() {
    for (ins, del) in [
        ("new document document", "old document"),
        ("new document", "old document document"),
        ("new document", "document old"),
        ("new document second", "old document second"),
        ("one two three four five six document", "old document"),
        ("new document", "one two three four five six document"),
        ("new simple", "old simple"),
        ("new with", "old with"),
        ("", "old document"),
        ("new document", ""),
    ] {
        unchanged(
            &format!(
                "<w:p>{}{}</w:p>",
                revision("ins", ins),
                revision("del", del)
            ),
            free_mesh_shared_title_token_in_mix,
        );
    }
    for side in ["ins", "del"] {
        let i = revision("ins", "new document");
        let d = revision("del", "old document");
        let (i, d) = if side == "ins" {
            (i.replace("Author", "Other"), d)
        } else {
            (i, d.replace("Author", "Other"))
        };
        unchanged(
            &format!("<w:p>{i}{d}</w:p>"),
            free_mesh_shared_title_token_in_mix,
        );
    }
    for side in ["ins", "del"] {
        let extra = revision(side, " ").repeat(2);
        unchanged(
            &format!(
                "<w:p>{}{}{extra}</w:p>",
                revision("ins", "new document"),
                revision("del", "old document")
            ),
            free_mesh_shared_title_token_in_mix,
        );
    }
}

#[test]
fn revised_complex_deletion_stays_pending_while_original_restores_nested_text() {
    let fragment = "<w:p><w:del><w:hyperlink w:anchor=\"here\"><w:r><w:delText>nested</w:delText></w:r></w:hyperlink></w:del></w:p>";
    for side in [FlattenSide::Original, FlattenSide::Revised] {
        let (mut dom, _, body) = document(fragment);
        flatten_tracked_deletions(&mut dom, body, side, None);
        let expected = if side == FlattenSide::Revised {
            fragment
        } else {
            "<w:p><w:hyperlink w:anchor=\"here\"><w:r><w:t>nested</w:t></w:r></w:hyperlink></w:p>"
        };
        expect_body(&dom, body, expected);
    }
}

#[test]
fn strict_shape_repair_stops_at_nested_drawing_and_keeps_bad_percent_text() {
    let wp = crate::namespaces::WP::URI;
    let a = crate::namespaces::A::URI;
    let target = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";
    let fragment = format!(
        "<w:p><w:r><w:drawing><a:graphicData xmlns:a=\"{a}\" xmlns:wp=\"{wp}\" uri=\"{target}\"><wp:wsp>text<wp:spPr/><w:drawing><wp:anchor/></w:drawing></wp:wsp></a:graphicData><a:graphicData xmlns:a=\"{a}\"><wp:anchor xmlns:wp=\"{wp}\"/></a:graphicData><wp14:pctWidth xmlns:wp14=\"{}\">bad%</wp14:pctWidth></w:drawing></w:r></w:p>",
        WP14::URI
    );
    let (mut dom, root, body) = document(&fragment);
    fix_strict_validity_artifacts(&mut dom, root);
    let expected = format!(
        "<w:p><w:r><w:drawing><a:graphicData xmlns:a=\"{a}\" xmlns:wp=\"{wp}\" xmlns:s=\"{target}\" uri=\"{target}\"><s:wsp>text<s:spPr/><w:drawing><wp:anchor/></w:drawing></s:wsp></a:graphicData><a:graphicData xmlns:a=\"{a}\"><wp:anchor xmlns:wp=\"{wp}\"/></a:graphicData><wp14:pctWidth xmlns:wp14=\"{}\">bad%</wp14:pctWidth></w:drawing></w:r></w:p>",
        WP14::URI
    );
    expect_body(&dom, body, &expected);
}

#[test]
fn block_reordering_requires_uniform_revision_rows_and_nonempty_sdt_content() {
    let inserted = pure("ins", "new");
    for (block, reorder) in [
        (
            "<w:tbl><w:tr><w:trPr><w:del/></w:trPr></w:tr></w:tbl>",
            true,
        ),
        ("<w:tbl/>", false),
        ("<w:tbl><w:tr/></w:tbl>", false),
        ("<w:tbl><w:tr><w:trPr/></w:tr></w:tbl>", false),
        (
            "<w:tbl><w:tr><w:trPr><w:del/></w:trPr></w:tr><w:tr><w:trPr><w:ins/></w:trPr></w:tr></w:tbl>",
            false,
        ),
        ("<w:sdt/>", false),
        ("<w:sdt><w:sdtContent/></w:sdt>", false),
        (
            "<w:sdt><w:sdtContent><w:p><w:del/></w:p></w:sdtContent></w:sdt>",
            true,
        ),
        (
            "<w:sdt><w:sdtContent><w:p><w:del/></w:p><w:p><w:ins/></w:p></w:sdtContent></w:sdt>",
            false,
        ),
        (
            "<w:sdt><w:sdtContent><w:p><w:del/></w:p><w:p><w:r><w:t>live</w:t></w:r></w:p></w:sdtContent></w:sdt>",
            false,
        ),
    ] {
        let original = format!("{block}{inserted}<w:sectPr/>");
        let (mut dom, root, body) = document(&original);
        reorder_replaced_blocks(&mut dom, root);
        let expected = if reorder {
            format!("{inserted}{block}<w:sectPr/>")
        } else {
            original
        };
        expect_body(&dom, body, &expected);
    }
}
