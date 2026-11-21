// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

// Pure in-memory WordprocessingML fixtures. Coverage is measured by the
// parent's serialized canonical run: required overall gates are 95% lines
// and 90% branches; this source-only round does not claim measured coverage.
fn document(fragment: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\" xmlns:pt=\"{}\"><w:body>{fragment}</w:body></w:document>",
        W::URI,
        PT::URI
    ));
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (dom, root, body)
}

fn rev(kind: &str, text: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:{kind} w:id=\"7\" w:author=\"Editor\" w:date=\"Pinned\"><w:r><w:{leaf}>{text}</w:{leaf}></w:r></w:{kind}>"
    )
}

fn para(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id=\"8\" w:author=\"Mark\" w:date=\"Old\"/></w:rPr></w:pPr>{}</w:p>",
        rev(kind, text)
    )
}

fn bare(kind: &str, text: &str) -> String {
    format!("<w:p>{}</w:p>", rev(kind, text))
}

fn paragraphs(dom: &Dom, parent: NodeId) -> Vec<NodeId> {
    dom.elements(parent, Some(&W::p()))
}

fn body_segments(dom: &Dom, p: NodeId) -> Vec<(String, String)> {
    dom.elements(p, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::p_pr()))
        .map(|n| {
            (
                dom.name(n).unwrap().local_name().to_string(),
                markup_text(dom, n),
            )
        })
        .collect()
}

fn expect_segments(dom: &Dom, p: NodeId, expected: &[(&str, &str)]) {
    assert_eq!(
        body_segments(dom, p),
        expected
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>()
    );
}

fn props_xml(dom: &Dom, p: NodeId) -> Option<String> {
    dom.element(p, &W::p_pr())
        .map(|pr| dom.serialize_element(pr))
}

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Editor".into(),
        date_time_for_revisions: "2026-01-02T03:04:05Z".into(),
        ..WmlComparerSettings::default()
    }
}

fn same_after_merge(fragment: &str) {
    let (mut dom, root, _) = document(fragment);
    let before = dom.serialize_element(root);
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    assert_eq!(dom.serialize_element(root), before, "{fragment}");
}

#[test]
fn paired_normalization_preserves_prior_pilcrow_provenance_and_only_unmarks_its_own_pair() {
    for (mark_author, mark_date, owned) in [
        ("Editor", "Pinned", true),
        ("Prior author", "Pinned", false),
        ("Editor", "Earlier", false),
    ] {
        for prior_on_inserted in [false, true] {
            for blank_anchor in [false, true] {
                for inserted_first in [false, true] {
                    if inserted_first && !blank_anchor {
                        continue;
                    }
                    let stamped = |kind, text, prior| {
                        let p = para(kind, text, "");
                        let (author, date) = if prior {
                            (mark_author, mark_date)
                        } else {
                            ("Editor", "Pinned")
                        };
                        p.replace("w:author=\"Mark\"", &format!("w:author=\"{author}\""))
                            .replace("w:date=\"Old\"", &format!("w:date=\"{date}\""))
                    };
                    let deleted = stamped("del", "shared original words", !prior_on_inserted);
                    let inserted = stamped("ins", "shared revised words", prior_on_inserted);
                    let anchor = if blank_anchor {
                        "<w:p/>"
                    } else {
                        "<w:p><w:r><w:t>Live tail</w:t></w:r></w:p>"
                    };
                    let (mut dom, root, body) = document(&if inserted_first {
                        format!("{inserted}{deleted}{anchor}")
                    } else {
                        format!("{deleted}{inserted}{anchor}")
                    });
                    let prior_paragraphs = paragraphs(&dom, body);
                    // Story-tail fusion keeps D's properties; a generic pair
                    // keeps I's. The ownership proof must inspect both marks.
                    let selected = prior_paragraphs[if blank_anchor {
                        usize::from(inserted_first)
                    } else {
                        1
                    }];
                    let expected_properties = props_xml(&dom, selected);
                    merge_replaced_paragraphs(&mut dom, root, "Editor");
                    let result = paragraphs(&dom, body);
                    assert_eq!(result.len(), 2);
                    assert_eq!(
                        body_segments(&dom, result[0])
                            .iter()
                            .map(|r| r.1.as_str())
                            .collect::<String>(),
                        "shared revised wordsshared original words"
                    );
                    if owned {
                        assert!(!para_mark_revision(&dom, result[0], &W::ins()));
                        assert!(!para_mark_revision(&dom, result[0], &W::del()));
                    } else {
                        assert_eq!(
                            props_xml(&dom, result[0]),
                            expected_properties,
                            "author={mark_author}, date={mark_date}, inserted={prior_on_inserted}, blank={blank_anchor}"
                        );
                    }
                    if blank_anchor {
                        assert!(body_segments(&dom, result[1]).is_empty());
                    } else {
                        expect_segments(&dom, result[1], &[("r", "Live tail")]);
                    }
                }
            }
        }
    }
}

#[test]
fn related_one_for_one_adopts_exact_structural_properties_and_inserted_attributes() {
    // A mid-story related replacement is a true MIX. New structural
    // formatting wins, otherwise the deleted structural layout wins.
    for (ins_props, del_props, choose_inserted) in [
        (
            "<w:jc w:val=\"right\"/>",
            "<w:spacing w:line=\"360\"/>",
            true,
        ),
        (
            "",
            "<w:numPr><w:ilvl w:val=\"2\"/><w:numId w:val=\"19\"/></w:numPr>",
            false,
        ),
        ("", "", true),
        (
            "<w:ind w:left=\"720\"/>",
            "<w:pStyle w:val=\"Heading2\"/>",
            true,
        ),
    ] {
        let inserted = para("ins", "shared revised words", ins_props)
            .replace("<w:p>", "<w:p w:rsidR=\"ABCD\">");
        let (mut dom, root, body) = document(&format!(
            "{}{}<w:p><w:r><w:t>Live tail</w:t></w:r></w:p>",
            para("del", "shared original words", del_props),
            inserted
        ));
        let old = paragraphs(&dom, body);
        let expected = props_xml(&dom, old[usize::from(choose_inserted)]);
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        let ps = paragraphs(&dom, body);
        assert_eq!(ps.len(), 2);
        assert_eq!(props_xml(&dom, ps[0]), expected);
        assert_eq!(dom.attribute(ps[0], &W::rsid_r()), Some("ABCD"));
        expect_segments(
            &dom,
            ps[0],
            &[
                ("ins", "shared revised words"),
                ("del", "shared original words"),
            ],
        );
        expect_segments(&dom, ps[1], &[("r", "Live tail")]);
        assert!(dom.parent(old[0]).is_none());
        assert!(dom.parent(old[1]).is_none());
    }
}

#[test]
fn field_replacement_keeps_bare_properties_instead_of_deleted_heading() {
    // A complete TOC field may merge, but its result must not inherit the
    // deleted heading style (M360); field geometry belongs to the field.
    let inserted = "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr><w:ins w:author=\"Editor\"><w:r><w:fldChar w:fldCharType=\"begin\"/><w:t> </w:t><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins></w:p>";
    let (mut dom, root, body) = document(&format!(
        "{}{}<w:p><w:r><w:t>Tail</w:t></w:r></w:p>",
        para("del", "", "<w:pStyle w:val=\"Heading1\"/>"),
        inserted
    ));
    let expected = props_xml(&dom, paragraphs(&dom, body)[1]);
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    let ps = paragraphs(&dom, body);
    assert_eq!(ps.len(), 2);
    assert_eq!(props_xml(&dom, ps[0]), expected);
    assert!(dom.descendants(ps[0], Some(&W::p_style())).is_empty());
    assert_eq!(dom.descendants(ps[0], Some(&W::fld_char())).len(), 2);
    expect_segments(&dom, ps[0], &[("ins", " "), ("del", "")]);
}

#[test]
fn bare_related_pair_does_not_invent_paragraph_properties() {
    // The deepest 1v1 merge also accepts paragraphs with no pPr at all.
    let (mut dom, root, body) = document(&format!(
        "{}{}<w:p><w:r><w:t>Tail</w:t></w:r></w:p>",
        bare("del", "common old"),
        bare("ins", "common new")
    ));
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    let ps = paragraphs(&dom, body);
    assert_eq!(ps.len(), 2);
    assert!(props_xml(&dom, ps[0]).is_none());
    expect_segments(&dom, ps[0], &[("ins", "common new"), ("del", "common old")]);
}

#[test]
fn deleted_then_multiple_inserted_folds_only_last_and_adopts_heading() {
    // A whole-story 1D/2I replacement preserves the leading inserted
    // paragraph and moves the sole old body into the final new paragraph.
    for heading in [false, true] {
        let props = if heading {
            "<w:pStyle w:val=\"Title\"/><w:jc w:val=\"center\"/>"
        } else {
            ""
        };
        let (mut dom, root, body) = document(&format!(
            "{}{}{}<w:sectPr/>",
            para("del", "Original heading words", props),
            para("ins", "Leading new words", ""),
            para("ins", "Revised ending words", "")
        ));
        let old = paragraphs(&dom, body);
        let deleted_props = props_xml(&dom, old[0]);
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        let ps = paragraphs(&dom, body);
        assert_eq!(ps, [old[1], old[2]]);
        expect_segments(&dom, ps[0], &[("ins", "Leading new words")]);
        expect_segments(
            &dom,
            ps[1],
            &[
                ("ins", "Revised ending words"),
                ("del", "Original heading words"),
            ],
        );
        assert_eq!(
            props_xml(&dom, ps[1]),
            if heading { deleted_props } else { None }
        );
        assert_eq!(dom.elements(body, Some(&W::sect_pr())).len(), 1);
    }
}

#[test]
fn last_inserted_list_adopts_deleted_listparagraph_and_numbering() {
    // M436: both paragraphs have live numbering, but the old ListParagraph
    // properties and deleted pilcrow must survive on the boundary MIX.
    let ip = "<w:numPr><w:numId w:val=\"22\"/></w:numPr>";
    let dp = "<w:pStyle w:val=\"ListParagraph\"/><w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"9\"/></w:numPr>";
    let (mut dom, root, body) = document(&format!(
        "{}{}{}{}",
        para("ins", "Earlier", ""),
        para("ins", "test", ip),
        para("del", "Num 1", dp),
        para("del", "Later old words", "")
    ));
    let old = paragraphs(&dom, body);
    let expected = props_xml(&dom, old[2]);
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    let ps = paragraphs(&dom, body);
    assert_eq!(ps, [old[0], old[1], old[3]]);
    assert_eq!(props_xml(&dom, ps[1]), expected);
    expect_segments(&dom, ps[1], &[("ins", "test"), ("del", "Num 1")]);
    assert!(para_mark_revision(&dom, ps[1], &W::del()));
    assert_eq!(
        dom.attribute(
            dom.descendants(ps[1], Some(&W::name("numId")))[0],
            &W::val()
        ),
        Some("9")
    );
}

#[test]
fn alignment_only_carrier_adopts_deleted_spacing_exactly() {
    // M102b: center/right alignment alone cannot override the old spacing.
    for mark in [false, true] {
        let ins = if mark {
            para("ins", "shared new words", "<w:jc w:val=\"right\"/>")
        } else {
            format!(
                "<w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr>{}</w:p>",
                rev("ins", "shared new words")
            )
        };
        let (mut dom, root, body) = document(&format!(
            "{}{}{}",
            ins,
            para(
                "del",
                "shared old words",
                "<w:spacing w:before=\"240\" w:after=\"120\" w:line=\"360\"/>"
            ),
            para("del", "Remaining", "")
        ));
        let old = paragraphs(&dom, body);
        let expected = props_xml(&dom, old[1]);
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        let ps = paragraphs(&dom, body);
        assert_eq!(ps.len(), 2);
        assert_eq!(props_xml(&dom, ps[0]), expected);
        assert!(dom.descendants(ps[0], Some(&W::jc_el())).is_empty());
        expect_segments(
            &dom,
            ps[0],
            &[("ins", "shared new words"), ("del", "shared old words")],
        );
    }
}

#[test]
fn paired_body_tail_stays_separate_but_nested_stories_still_fold() {
    // Only the body's paired final marks disable boundary fusion. A cell,
    // text box, or content control is an independent story with its own end.
    for container in ["tc", "txbxContent", "sdtContent"] {
        let pair = format!(
            "{}{}",
            para("ins", "shared new", ""),
            para("del", "shared old", "")
        );
        let (mut dom, root, body) =
            document(&format!("{pair}<w:{container}>{pair}</w:{container}>"));
        let outer = paragraphs(&dom, body);
        let nested = dom.element(body, &W::name(container)).unwrap();
        merge_replaced_paragraphs_in(&mut dom, root, "Editor", true);
        assert_eq!(paragraphs(&dom, body), outer);
        expect_segments(&dom, outer[0], &[("ins", "shared new")]);
        expect_segments(&dom, outer[1], &[("del", "shared old")]);
        let ps = paragraphs(&dom, nested);
        assert_eq!(ps.len(), 1);
        expect_segments(&dom, ps[0], &[("ins", "shared new"), ("del", "shared old")]);
        assert!(props_xml(&dom, ps[0]).is_none());
    }
}

#[test]
fn shared_title_head_junction_precedes_remaining_inserted_stream() {
    // M322: TIFF's shared heading belongs to the first inserted paragraph,
    // even when the paired body boundary forbids a generic last-I fold.
    let ins = [
        "TIFF revised heading",
        "Second new paragraph",
        "Third new paragraph",
        "Fourth new paragraph",
        "Final unrelated prose",
    ];
    let fragment = ins.iter().map(|t| para("ins", t, "")).collect::<String>()
        + &para(
            "del",
            "TIFF original heading",
            "<w:pStyle w:val=\"Title\"/>",
        );
    let (mut dom, root, body) = document(&fragment);
    let old = paragraphs(&dom, body);
    let expected_props = props_xml(&dom, old[5]);
    merge_replaced_paragraphs_in(&mut dom, root, "Editor", true);
    assert_eq!(paragraphs(&dom, body), old[..5]);
    expect_segments(
        &dom,
        old[0],
        &[("ins", ins[0]), ("del", "TIFF original heading")],
    );
    for (&p, text) in old[1..5].iter().zip(&ins[1..]) {
        expect_segments(&dom, p, &[("ins", text)]);
    }
    assert_eq!(props_xml(&dom, old[0]), expected_props);
}

#[test]
fn story_tail_fusion_crosses_deleted_table_and_keeps_deleted_layout() {
    // A final live pilcrow after a deleted table is the original story's
    // tail; the revised tail's runs open the first deleted paragraph.
    for deleted_first in [false, true] {
        let d = para("del", "Base title", "<w:jc w:val=\"center\"/>");
        let i = format!(
            "{}{}",
            para("ins", "New leading paragraph", ""),
            para("ins", "Revised tail words", "<w:spacing w:line=\"240\"/>")
        );
        let table = format!(
            "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
            para("del", "Old cell", "")
        );
        let pair = if deleted_first {
            format!("{d}{i}")
        } else {
            format!("{i}{d}")
        };
        let (mut dom, root, body) = document(&format!("{pair}{table}<w:p/><w:sectPr/>"));
        let ps0 = paragraphs(&dom, body);
        let dnode = ps0[if deleted_first { 0 } else { 2 }];
        let expected_props = props_xml(&dom, dnode);
        let tbl = dom.element(body, &W::tbl()).unwrap();
        let expected_table = dom.serialize_element(tbl);
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        let ps = paragraphs(&dom, body);
        assert_eq!(ps.len(), 3);
        expect_segments(&dom, ps[0], &[("ins", "New leading paragraph")]);
        assert_eq!(ps[1], dnode);
        expect_segments(
            &dom,
            ps[1],
            &[("ins", "Revised tail words"), ("del", "Base title")],
        );
        assert_eq!(props_xml(&dom, ps[1]), expected_props);
        assert_eq!(dom.serialize_element(tbl), expected_table);
        assert!(body_segments(&dom, ps[2]).is_empty());
    }
}

#[test]
fn story_tail_fusion_stops_at_page_breaks_section_breaks_and_live_tables() {
    // These exact layout boundaries keep the tail paragraph whole.
    for (ip, dp, cell, paired) in [
        (
            "<w:pageBreakBefore/>",
            "",
            para("del", "Old cell", ""),
            false,
        ),
        (
            "",
            "<w:pageBreakBefore/>",
            para("del", "Old cell", ""),
            false,
        ),
        ("", "<w:sectPr/>", para("del", "Old cell", ""), false),
        (
            "",
            "",
            "<w:p><w:r><w:t>Live cell</w:t></w:r></w:p>".into(),
            false,
        ),
        ("", "", para("del", "Old cell", ""), true),
    ] {
        let (mut dom, root, _) = document(&format!(
            "{}{}<w:tbl><w:tr><w:tc>{cell}</w:tc></w:tr></w:tbl><w:p/>",
            para("ins", "Revised tail words", ip),
            para("del", "Base title", dp)
        ));
        let before = dom.serialize_element(root);
        merge_replaced_paragraphs_in(&mut dom, root, "Editor", paired);
        assert_eq!(dom.serialize_element(root), before);
    }
}

#[test]
fn multiple_mark_only_deletions_fold_once_and_preserve_remaining_empty() {
    // M218: adopt one deleted pilcrow, then the carrier becomes mixed and
    // a second empty paragraph cannot be swallowed on the next rescan.
    for mark_count in [1, 2] {
        let empty = "<w:p><w:pPr><w:rPr><w:del w:id=\"41\"/></w:rPr></w:pPr></w:p>";
        let (mut dom, root, body) = document(&format!(
            "{}{}",
            para("ins", "New prose", ""),
            empty.repeat(mark_count)
        ));
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        let ps = paragraphs(&dom, body);
        assert_eq!(ps.len(), mark_count);
        expect_segments(&dom, ps[0], &[("ins", "New prose")]);
        assert!(para_mark_revision(&dom, ps[0], &W::del()));
        assert!(!para_mark_revision(&dom, ps[0], &W::ins()));
        assert_eq!(
            dom.attribute(
                dom.descendants(dom.element(ps[0], &W::p_pr()).unwrap(), Some(&W::del()))[0],
                &W::id()
            ),
            Some("41")
        );
        if mark_count == 2 {
            assert!(body_segments(&dom, ps[1]).is_empty());
        }
    }
}

#[test]
fn empty_heading_residual_adopts_heading_for_last_list_carrier() {
    // M385 permits an empty Heading1 deleted mark after multiple list
    // insertions, while an ordinary empty mark remains a separate spacer.
    let heading_empty = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/><w:rPr><w:del w:id=\"45\"/></w:rPr></w:pPr></w:p>";
    let (mut dom, root, body) = document(&format!(
        "{}{}{}{}<w:p><w:r><w:t>Live tail</w:t></w:r></w:p>",
        para("ins", "Earlier reference", ""),
        para("ins", "Final citation", "<w:pStyle w:val=\"ListNumber\"/>"),
        heading_empty,
        para("del", "Old remaining prose", "")
    ));
    let old = paragraphs(&dom, body);
    let expected = props_xml(&dom, old[2]);
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    assert_eq!(paragraphs(&dom, body), [old[0], old[1], old[3], old[4]]);
    assert_eq!(props_xml(&dom, old[1]), expected);
    expect_segments(&dom, old[1], &[("ins", "Final citation")]);
}

#[test]
fn carried_foreign_insertions_reorder_without_becoming_replacement_deletions() {
    // A carried insertion counts as old-side structure but contains no
    // real deletion; moving it must retain its pending insertion markup.
    let carried = bare("ins", "Carried words").replace("Editor", "Prior");
    let (mut dom, root, body) = document(&format!(
        "{carried}{}<w:p><w:r><w:t>Tail</w:t></w:r></w:p>",
        para("ins", "New words", "")
    ));
    let old = paragraphs(&dom, body);
    let carried_xml = dom.serialize_element(old[0]);
    merge_replaced_paragraphs(&mut dom, root, "Editor");
    assert_eq!(paragraphs(&dom, body), [old[1], old[0], old[2]]);
    assert_eq!(dom.serialize_element(old[0]), carried_xml);
    assert!(dom.descendants(root, Some(&W::del())).is_empty());
}

#[test]
fn unmatched_field_blocks_shared_title_head_junction() {
    // An unmatched old TOC field cannot move ahead of an inserted field.
    // Paired final marks also forbid falling back to a generic tail fold.
    for field in ["begin", "end"] {
        let mut stream = para("ins", "TIFF revised heading", "");
        stream.push_str(
            &para("ins", "Second inserted paragraph", "")
                .replace("<w:r><w:t>", "<w:r><w:fldChar w:fldCharType=\"end\"/><w:t>"),
        );
        for text in [
            "Third inserted paragraph",
            "Fourth inserted paragraph",
            "Final unrelated prose",
        ] {
            stream.push_str(&para("ins", text, ""));
        }
        let deleted = para("del", "TIFF original heading", "").replace(
            "<w:r><w:delText>",
            &format!("<w:r><w:fldChar w:fldCharType=\"{field}\"/><w:delText>"),
        );
        let (mut dom, root, _) = document(&format!("{stream}{deleted}"));
        let before = dom.serialize_element(root);
        merge_replaced_paragraphs_in(&mut dom, root, "Editor", true);
        assert_eq!(dom.serialize_element(root), before);
    }
}

#[test]
fn digits_and_short_list_residuals_keep_their_own_inserted_paragraph() {
    // M101 and M124 distinguish digits/one-letter residues from prose.
    for label in ["24", "a", "ab"] {
        same_after_merge(&format!(
            "{}{}{}",
            para("ins", label, ""),
            para("del", "Original long heading", ""),
            para("del", "Other old content", "")
        ));
    }
}

#[test]
fn long_prose_does_not_inherit_multiword_list_geometry() {
    // M363 rejects prose × a three-word list item, while M382 permits a
    // two-word label and adopts its old numbering and deleted mark.
    for (old_text, folds) in [("First item", true), ("First item continued", false)] {
        let text = "Revised continuous prose with enough letters";
        let (mut dom, root, body) = document(&format!(
            "{}{}{}{}",
            para("ins", "Earlier new words", ""),
            para("ins", text, ""),
            para("del", old_text, "<w:numPr><w:numId w:val=\"5\"/></w:numPr>"),
            para("del", "Remaining old prose", "")
        ));
        let before = dom.serialize_element(root);
        let old = paragraphs(&dom, body);
        let expected = props_xml(&dom, old[2]);
        merge_replaced_paragraphs(&mut dom, root, "Editor");
        if folds {
            assert_eq!(paragraphs(&dom, body), [old[0], old[1], old[3]]);
            expect_segments(&dom, old[1], &[("ins", text), ("del", old_text)]);
            assert_eq!(props_xml(&dom, old[1]), expected);
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn runless_empty_carrier_preserves_cover_spacers_and_long_prose() {
    // At 12 words a whitespace run can fold; at 13 it must preserve the
    // prose paragraph. A following inserted table preserves a runless blank.
    for count in [12, 13] {
        let text = (0..count)
            .map(|i| format!("word{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let (mut dom, root, body) = document(&format!(
            "{}{}",
            para("ins", " ", ""),
            para("del", &text, "")
        ));
        let before = dom.serialize_element(root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        if count == 12 {
            let ps = paragraphs(&dom, body);
            assert_eq!(ps.len(), 1);
            expect_segments(&dom, ps[0], &[("ins", " "), ("del", &text)]);
            assert!(para_mark_revision(&dom, ps[0], &W::del()));
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
    let blank = "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr></w:p>";
    let (mut dom, root, _) = document(&format!(
        "{blank}{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
        para("del", "Original body contains five distinct words", ""),
        para("ins", "Later cell", "")
    ));
    let before = dom.serialize_element(root);
    fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before);
}

#[test]
fn trailing_whitespace_spacing_folds_only_after_contentful_insertions() {
    // M366 removes a true trailing residual spacer. M345 retains the
    // same spacing-only empty paragraph when it is the leading layout.
    for preceding in [false, true] {
        let prefix = if preceding {
            para("ins", "Cover prose", "")
        } else {
            String::new()
        };
        let (mut dom, root, body) = document(&format!(
            "{prefix}{}{}{}{}",
            para("ins", " ", "<w:spacing w:line=\"240\"/>"),
            para("del", "Old first five distinct words", ""),
            para("del", "Old second", ""),
            para("del", "Old third", "")
        ));
        let before = dom.serialize_element(root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        if preceding {
            let ps = paragraphs(&dom, body);
            assert_eq!(ps.len(), 4);
            expect_segments(
                &dom,
                ps[1],
                &[("ins", " "), ("del", "Old first five distinct words")],
            );
            let sp = dom
                .element(dom.element(ps[1], &W::p_pr()).unwrap(), &W::spacing_el())
                .unwrap();
            assert_eq!(dom.attribute(sp, &W::name("line")), Some("240"));
            assert!(para_mark_revision(&dom, ps[1], &W::del()));
        } else {
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn whitespace_carriers_without_mark_shell_adopt_deleted_pilcrow() {
    // Body revisions alone can classify a paragraph pure-I. Cover all
    // pPr states without inventing layout or losing the deleted mark.
    for props in ["", "<w:pPr/>", "<w:pPr><w:rPr><w:b/></w:rPr></w:pPr>"] {
        let (mut dom, root, body) = document(&format!(
            "<w:p>{props}{}</w:p>{}",
            rev("ins", " "),
            para("del", "Old prose words", "")
        ));
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        let ps = paragraphs(&dom, body);
        assert_eq!(ps.len(), 1);
        assert!(para_mark_revision(&dom, ps[0], &W::del()));
        assert_eq!(
            dom.descendants(ps[0], Some(&W::name("b"))).len(),
            usize::from(props.contains("w:b"))
        );
        expect_segments(&dom, ps[0], &[("ins", " "), ("del", "Old prose words")]);
    }
}

#[test]
fn fragmented_two_character_label_meshes_and_retains_whitespace_and_punctuation() {
    // Case and punctuation belong to the source streams. An ambiguous
    // label must remain a replacement instead of changing either view.
    for inserted in [
        "<w:r><w:t> a</w:t></w:r><w:r><w:t>b!</w:t></w:r>",
        "<w:r><w:t>a</w:t><w:t>b </w:t></w:r>",
    ] {
        let (mut dom, root, body) = document(&format!(
            "<w:p><w:ins>{inserted}</w:ins><w:del><w:r><w:delText>Lvl 1 – </w:delText><w:delText>A</w:delText><w:delText>B</w:delText></w:r></w:del><w:r><w:t>Tail</w:t></w:r></w:p>"
        ));
        let p = paragraphs(&dom, body)[0];
        let before = dom.serialize_element(p);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(dom.serialize_element(p), before);
        assert_eq!(dom.descendants(root, Some(&W::p())).len(), 1);
    }
}

#[test]
fn label_mesh_removes_empty_inserted_wrapper_and_keeps_deleted_list_layout() {
    // Shared label with no residual insertion becomes exactly del-prefix
    // plus EQ; source list geometry stays on the deleted carrier.
    let (mut dom, root, body) = document(&format!(
        "{}{}{}<w:p/>",
        para(
            "ins",
            "a",
            "<w:spacing w:line=\"276\"/><w:ind w:left=\"1440\"/>"
        ),
        para("ins", "", ""),
        para(
            "del",
            "Lvl 1 a",
            "<w:numPr><w:numId w:val=\"3\"/></w:numPr>"
        )
    ));
    let old = paragraphs(&dom, body);
    residual_short_label_zip(&mut dom, root);
    assert_eq!(paragraphs(&dom, body), [old[2], old[1], old[3]]);
    expect_segments(&dom, old[2], &[("del", "Lvl 1 "), ("r", "a")]);
    assert!(dom.elements(old[2], Some(&W::ins())).is_empty());
    let pr = dom.element(old[2], &W::p_pr()).unwrap();
    assert_eq!(
        dom.attribute(dom.element(pr, &W::spacing_el()).unwrap(), &W::name("line")),
        Some("276")
    );
    assert_eq!(
        dom.attribute(dom.element(pr, &W::name("ind")).unwrap(), &W::name("left")),
        Some("1440")
    );
    assert_eq!(
        dom.attribute(dom.descendants(pr, Some(&W::name("numId")))[0], &W::val()),
        Some("3")
    );
    assert!(para_mark_revision(&dom, old[2], &W::del()));
}

#[test]
fn positional_label_zip_keeps_distinct_labels_and_existing_geometry() {
    // A mismatching short label zips into a Lvl item positionally, retaining
    // full I+D and the old spacing/indent rather than overwriting them.
    let (mut dom, root, body) = document(&format!(
        "{}{}",
        para(
            "ins",
            "3",
            "<w:spacing w:line=\"276\"/><w:ind w:left=\"1440\"/>"
        ),
        para(
            "del",
            "Lvl 2 i",
            "<w:spacing w:line=\"360\"/><w:ind w:left=\"720\"/>"
        )
    ));
    let old = paragraphs(&dom, body);
    let expected = props_xml(&dom, old[1]);
    residual_short_label_zip(&mut dom, root);
    assert_eq!(paragraphs(&dom, body), [old[1]]);
    expect_segments(&dom, old[1], &[("ins", "3"), ("del", "Lvl 2 i")]);
    assert_eq!(props_xml(&dom, old[1]), expected);
}

#[test]
fn nested_list_interleave_preserves_exact_node_order_and_trailing_anchors() {
    // Move the remaining new labels after the old nested cluster, stopping
    // at the next top-level item. Empty old items stay inside the cluster.
    for section in [false, true] {
        let fragment = format!(
            "{}{}{}{}{}{}<w:p/>{}",
            para("ins", "ONE", ""),
            para("ins", "a", ""),
            para("ins", "TWO", ""),
            para("del", "Old top", "<w:numPr><w:ilvl w:val=\"0\"/></w:numPr>"),
            para(
                "del",
                "Old nested",
                "<w:numPr><w:ilvl w:val=\"1\"/></w:numPr>"
            ),
            para(
                "del",
                "Old following",
                "<w:numPr><w:ilvl w:val=\"0\"/></w:numPr>"
            ),
            if section { "<w:sectPr/>" } else { "" }
        );
        let (mut dom, root, body) = document(&fragment);
        let old = paragraphs(&dom, body);
        let originals = old
            .iter()
            .map(|&p| dom.serialize_element(p))
            .collect::<Vec<_>>();
        interleave_list_cluster_after_coalesce(&mut dom, root);
        assert_eq!(
            paragraphs(&dom, body),
            [old[0], old[3], old[4], old[1], old[2], old[5], old[6]]
        );
        for (&p, expected) in old.iter().zip(&originals) {
            assert_eq!(&dom.serialize_element(p), expected);
        }
        let once = dom.serialize_element(root);
        interleave_list_cluster_after_coalesce(&mut dom, root);
        assert_eq!(dom.serialize_element(root), once);
        assert_eq!(
            dom.elements(body, Some(&W::sect_pr())).len(),
            usize::from(section)
        );
    }
}

#[test]
fn deleted_textbox_story_marks_runs_and_pilcrows_in_schema_order() {
    // A deleted anchor's revision does not reach its text-box story.
    // Bare field/text runs share a deletion; a bookmark starts a new span.
    let (mut dom, root, _) = document(
        "<w:p><w:del w:id=\"2\" w:author=\"Anchor\" w:date=\"Earlier\"><w:r><w:drawing><w:txbxContent><w:p><w:pPr><w:spacing w:line=\"240\"/><w:pPrChange><w:pPr/></w:pPrChange></w:pPr><w:r><w:instrText>FILENAME</w:instrText></w:r><w:r><w:t>Shown</w:t></w:r><w:bookmarkStart w:id=\"1\"/><w:r><w:t>After</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:b/></w:rPr><w:sectPr/></w:pPr><w:r><w:t>Second</w:t></w:r></w:p></w:txbxContent></w:drawing></w:r></w:del></w:p>",
    );
    let mut id = 100;
    wrap_bare_del_text_runs(&mut dom, root, &settings(), &mut id);
    let tb = dom.descendants(root, Some(&W::txbx_content()))[0];
    let ps = paragraphs(&dom, tb);
    expect_segments(
        &dom,
        ps[0],
        &[("del", "Shown"), ("bookmarkStart", ""), ("del", "After")],
    );
    assert_eq!(
        dom.elements(dom.element(ps[0], &W::del()).unwrap(), Some(&W::r()))
            .len(),
        2
    );
    assert_eq!(dom.descendants(tb, Some(&W::instr_text())).len(), 1);
    expect_segments(&dom, ps[1], &[("del", "Second")]);
    for p in ps {
        let pr = dom.element(p, &W::p_pr()).unwrap();
        let names = dom
            .elements(pr, None)
            .iter()
            .map(|&n| dom.name(n).unwrap().local_name().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            if p == paragraphs(&dom, tb)[0] {
                vec!["spacing", "rPr", "pPrChange"]
            } else {
                vec!["rPr", "sectPr"]
            }
        );
        let rp = dom.element(pr, &W::r_pr()).unwrap();
        assert!(dom.name_is(dom.elements(rp, None)[0], &W::del()));
    }
    for d in dom.descendants(tb, Some(&W::del())) {
        assert_eq!(dom.attribute(d, &W::author()), Some("Anchor"));
        assert_eq!(dom.attribute(d, &W::date()), Some("Earlier"));
        assert!(
            dom.attributes(d)
                .iter()
                .all(|(n, _)| n.namespace_name() != PT::URI)
        );
    }
    assert_eq!(id, 105);
    let once = dom.serialize_element(root);
    wrap_bare_del_text_runs(&mut dom, root, &settings(), &mut id);
    assert_eq!(dom.serialize_element(root), once);
    assert_eq!(id, 105);
}

#[test]
fn nested_deleted_textboxes_use_their_own_anchor_and_preserve_existing_marks() {
    // An outer deletion does not reach through a live nested anchor.
    // An explicitly deleted nested anchor supplies its own attribution.
    for nested_deleted in [false, true] {
        let nested = if nested_deleted {
            "<w:del w:author=\"Nested\" w:date=\"NestedDate\"><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>Inner</w:t></w:r></w:p></w:txbxContent></w:pict></w:r></w:del>"
        } else {
            "<w:ins><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>Inner</w:t></w:r></w:p></w:txbxContent></w:pict></w:r></w:ins>"
        };
        let (mut dom, root, _) = document(&format!(
            "<w:p><w:del><w:r><w:pict><w:txbxContent><w:p><w:pPr><w:rPr><w:moveTo w:id=\"77\"/></w:rPr></w:pPr>{nested}</w:p></w:txbxContent></w:pict></w:r></w:del></w:p>"
        ));
        let boxes = dom.descendants(root, Some(&W::txbx_content()));
        let outer_p = paragraphs(&dom, boxes[0])[0];
        let inner_p = paragraphs(&dom, boxes[1])[0];
        let outer_props = props_xml(&dom, outer_p);
        let inner_before = dom.serialize_element(inner_p);
        let mut id = 300;
        wrap_bare_del_text_runs(&mut dom, root, &settings(), &mut id);
        assert_eq!(props_xml(&dom, outer_p), outer_props);
        if nested_deleted {
            expect_segments(&dom, inner_p, &[("del", "Inner")]);
            for d in dom.descendants(inner_p, Some(&W::del())) {
                assert_eq!(dom.attribute(d, &W::author()), Some("Nested"));
                assert_eq!(dom.attribute(d, &W::date()), Some("NestedDate"));
            }
            assert_eq!(id, 302);
        } else {
            assert_eq!(dom.serialize_element(inner_p), inner_before);
            assert_eq!(id, 300);
        }
    }
}

#[test]
fn deleted_textbox_missing_attribution_uses_pinned_settings() {
    // Missing anchor attributes default to settings; existing paragraph
    // revisions are retained and do not consume IDs.
    for existing in ["ins", "del", "moveFrom", "moveTo"] {
        let (mut dom, root, _) = document(&format!(
            "<w:p><w:del><w:r><w:drawing><w:txbxContent><w:p><w:pPr><w:rPr><w:{existing} w:id=\"88\"/></w:rPr></w:pPr><w:r><w:t>Story</w:t></w:r></w:p></w:txbxContent></w:drawing></w:r></w:del></w:p>"
        ));
        let tb = dom.descendants(root, Some(&W::txbx_content()))[0];
        let p = paragraphs(&dom, tb)[0];
        let expected_props = props_xml(&dom, p);
        let mut id = 400;
        wrap_bare_del_text_runs(&mut dom, root, &settings(), &mut id);
        assert_eq!(props_xml(&dom, p), expected_props);
        let d = dom.element(p, &W::del()).unwrap();
        assert_eq!(dom.attribute(d, &W::id()), Some("400"));
        assert_eq!(dom.attribute(d, &W::author()), Some("Editor"));
        assert_eq!(dom.attribute(d, &W::date()), Some("2026-01-02T03:04:05Z"));
        assert_eq!(id, 401);
    }
}

#[test]
fn bordered_table_geometry_uses_row_revision_source_and_schema_order() {
    // Deleted rows select A's default table style; live/mixed/empty tables
    // select B's. Only the selected source can suppress Word's mar10/ind10.
    for deleted in [false, true] {
        for defaults in [(false, false), (true, false), (false, true), (true, true)] {
            let trpr = if deleted {
                "<w:trPr><w:del/></w:trPr>"
            } else {
                ""
            };
            let (mut dom, root, _) = document(&format!(
                "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/><w:tblBorders/><w:tblLook w:val=\"04A0\"/></w:tblPr><w:tr>{trpr}<w:tc><w:p/></w:tc></w:tr></w:tbl>"
            ));
            let before = dom.serialize_element(root);
            synthesize_table_cell_margins(&mut dom, root, defaults);
            let suppressed = if deleted { defaults.0 } else { defaults.1 };
            if suppressed {
                assert_eq!(dom.serialize_element(root), before);
            } else {
                let pr = dom.descendants(root, Some(&W::tbl_pr()))[0];
                let names = dom
                    .elements(pr, None)
                    .iter()
                    .map(|&n| dom.name(n).unwrap().local_name().to_string())
                    .collect::<Vec<_>>();
                assert_eq!(
                    names,
                    ["tblW", "tblInd", "tblBorders", "tblCellMar", "tblLook"]
                );
                let ind = dom.element(pr, &W::tbl_ind()).unwrap();
                assert_eq!(dom.attribute(ind, &W::w()), Some("10"));
                assert_eq!(dom.attribute(ind, &W::type_()), Some("dxa"));
                let mar = dom.element(pr, &W::name("tblCellMar")).unwrap();
                for side in ["left", "right"] {
                    let n = dom.element(mar, &W::name(side)).unwrap();
                    assert_eq!(dom.attribute(n, &W::w()), Some("10"));
                    assert_eq!(dom.attribute(n, &W::type_()), Some("dxa"));
                }
                let once = dom.serialize_element(root);
                synthesize_table_cell_margins(&mut dom, root, defaults);
                assert_eq!(dom.serialize_element(root), once);
            }
        }
    }
}

#[test]
fn table_geometry_preserves_explicit_values_and_fills_only_missing_properties() {
    // Existing explicit geometry is authoritative, including one-sided
    // margins; synthesis fills whole absent properties only.
    for (ind, mar) in [(false, false), (false, true), (true, false), (true, true)] {
        let explicit_ind = if ind {
            "<w:tblInd w:w=\"720\" w:type=\"dxa\"/>"
        } else {
            ""
        };
        let explicit_mar = if mar {
            "<w:tblCellMar><w:left w:w=\"108\" w:type=\"dxa\"/></w:tblCellMar>"
        } else {
            ""
        };
        let (mut dom, root, _) = document(&format!(
            "<w:tbl><w:tblPr>{explicit_ind}<w:tblBorders/>{explicit_mar}</w:tblPr><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"
        ));
        synthesize_table_cell_margins(&mut dom, root, (false, false));
        let pr = dom.descendants(root, Some(&W::tbl_pr()))[0];
        assert_eq!(
            dom.attribute(dom.element(pr, &W::tbl_ind()).unwrap(), &W::w()),
            Some(if ind { "720" } else { "10" })
        );
        let margins = dom.element(pr, &W::name("tblCellMar")).unwrap();
        assert_eq!(
            dom.attribute(dom.element(margins, &W::name("left")).unwrap(), &W::w()),
            Some(if mar { "108" } else { "10" })
        );
        assert_eq!(dom.element(margins, &W::name("right")).is_some(), !mar);
    }
}

#[test]
fn large_wholesale_replacement_keeps_all_paragraphs_at_scale_boundary() {
    // Above 150 deleted paragraphs the explicit large-document gate
    // precedes the ordinary scale gate; both must retain a clean I/D gap.
    for count in [150, 151] {
        let inserted = para("ins", "Fresh alpha beta gamma delta", "").repeat(10);
        let deleted = para(
            "del",
            "Stale red orange purple green blue black white silver gold",
            "",
        )
        .repeat(count);
        same_after_merge(&format!("{inserted}{deleted}"));
    }
}

#[test]
fn short_wholesale_lists_keep_both_sides_when_old_count_is_twice_new() {
    // M308d's short-list scale gate must reject a forced boundary MIX
    // even though this local two-I/four-D gap is below 40 words.
    let num = "<w:numPr><w:numId w:val=\"5\"/></w:numPr>";
    let fragment = format!(
        "{}{}{}",
        para("ins", "Fresh alpha", num),
        para("ins", "New bravo", num),
        para("del", "Stale red green blue", num).repeat(4)
    );
    same_after_merge(&fragment);
}

#[test]
fn inserted_break_never_folds_into_deleted_drawing_pict_or_object() {
    // M431 regards non-text page content as layout, rather than an empty
    // shell eligible for the ordinary small-gap override.
    for payload in ["drawing", "pict", "object"] {
        let inserted = "<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr><w:ins w:author=\"Editor\"><w:r><w:br/></w:r></w:ins></w:p>";
        let deleted = format!(
            "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr><w:del><w:r><w:{payload}/></w:r></w:del></w:p>"
        );
        same_after_merge(&format!(
            "{inserted}{deleted}{}",
            para("del", "Other old content", "")
        ));
    }
}

#[test]
fn revision_rows_preserve_cell_geometry_and_are_stamped_once() {
    // A fully revised empty tail owns a revised pilcrow. A live empty
    // tail is an independent source owner and forbids wholesale row
    // deletion/insertion, even when every visible run is revised.
    for kind in ["ins", "del"] {
        for live_tail in [false, true] {
            let tail = if live_tail {
                "<w:p/>".to_string()
            } else {
                para(kind, "", "")
            };
            let (mut dom, root, _) = document(&format!(
                "<w:tbl><w:tr><w:trPr><w:trHeight w:val=\"320\"/></w:trPr><w:tc><w:tcPr><w:tcW w:w=\"1440\" w:type=\"dxa\"/></w:tcPr>{}</w:tc><w:tc><w:tcPr><w:vAlign w:val=\"center\"/></w:tcPr>{}{tail}</w:tc></w:tr></w:tbl>",
                para(kind, "First cell", ""),
                para(kind, "Second cell", "")
            ));
            let row = dom.descendants(root, Some(&W::tr()))[0];
            let cells = dom.elements(row, Some(&W::tc()));
            let cell_xml = cells
                .iter()
                .map(|&c| dom.serialize_element(c))
                .collect::<Vec<_>>();
            let mut id = 500;
            mark_fully_revised_rows(&mut dom, root, &settings(), &mut id);
            let pr = dom.element(row, &W::tr_pr()).unwrap();
            if live_tail {
                assert!(dom.element(pr, &W::ins()).is_none());
                assert!(dom.element(pr, &W::del()).is_none());
                assert_eq!(id, 500, "live ownership creates no revision stamp");
            } else {
                let mark = dom.element(pr, &W::name(kind)).unwrap();
                assert_eq!(dom.attribute(mark, &W::id()), Some("500"));
                assert_eq!(dom.attribute(mark, &W::author()), Some("Editor"));
                assert_eq!(
                    dom.attribute(mark, &W::date()),
                    Some("2026-01-02T03:04:05Z")
                );
                assert_eq!(dom.elements(pr, Some(&W::name(kind))).len(), 1);
                assert_eq!(id, 501);
            }
            assert_eq!(
                dom.attribute(dom.element(pr, &W::name("trHeight")).unwrap(), &W::val()),
                Some("320")
            );
            for (&cell, xml) in cells.iter().zip(&cell_xml) {
                assert_eq!(
                    &dom.serialize_element(cell),
                    xml,
                    "{kind}, live tail={live_tail}: complete cell geometry and payload remain unchanged"
                );
            }
            let once = dom.serialize_element(root);
            mark_fully_revised_rows(&mut dom, root, &settings(), &mut id);
            assert_eq!(dom.serialize_element(root), once);
            assert_eq!(id, if live_tail { 500 } else { 501 });
        }
    }
}
