// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn xml(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\" xmlns:pt=\"{}\"><w:body>{fragment}</w:body></w:document>",
        W::URI,
        PT::URI
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn ps(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.elements(dom.element(root, &W::body()).unwrap(), Some(&W::p()))
}

fn revision(kind: &str, value: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:{kind} w:id=\"7\" w:author=\"Source\" w:date=\"Pinned\"><w:r><w:{leaf}>{value}</w:{leaf}></w:r></w:{kind}>"
    )
}

fn pure(kind: &str, value: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id=\"8\" w:author=\"Mark\" w:date=\"Old\"/></w:rPr></w:pPr>{}</w:p>",
        revision(kind, value)
    )
}

fn mixed(ins: &str, del: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}</w:pPr>{}{}</w:p>",
        revision("ins", ins),
        revision("del", del)
    )
}

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Editor".to_string(),
        date_time_for_revisions: "2026-01-02T03:04:05Z".to_string(),
        ..WmlComparerSettings::default()
    }
}

fn segments(dom: &Dom, p: NodeId, expected: &[(&str, &str)]) {
    let actual: Vec<_> = dom
        .elements(p, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::p_pr()))
        .map(|n| {
            (
                dom.name(n).unwrap().local_name().to_string(),
                markup_text(dom, n),
            )
        })
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|&(k, t)| (k.to_string(), t.to_string()))
        .collect();
    assert_eq!(actual, expected);
}

fn stamped(dom: &Dom, node: NodeId, id: &str) {
    assert_eq!(dom.attribute(node, &W::id()), Some(id));
    assert_eq!(dom.attribute(node, &W::author()), Some("Editor"));
    assert_eq!(
        dom.attribute(node, &W::date()),
        Some("2026-01-02T03:04:05Z")
    );
}

fn unchanged(fragment: &str, transform: fn(&mut Dom, NodeId)) {
    let (mut dom, root) = xml(fragment);
    let before = dom.serialize_element(root);
    transform(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before, "{fragment}");
}

#[test]
fn wholesale_mesh_emits_exact_shared_words_and_styled_residuals() {
    for styled in [false, true] {
        for period in [false, true] {
            let punctuation = if period { "." } else { "" };
            let rpr = if styled { "<w:rPr><w:b/></w:rPr>" } else { "" };
            let (mut dom, root) = xml(&format!(
                "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:ins w:author=\"Alice\" w:date=\"Fixed\"><w:r>{rpr}<w:t>Fresh is centered vivid orange document{punctuation}</w:t></w:r></w:ins><w:del><w:r><w:delText>Old is centered bold blue document</w:delText></w:r></w:del></w:p>"
            ));
            free_mesh_wholesale_body_mix(&mut dom, root);
            let p = ps(&dom, root)[0];
            let mut expected = vec![
                ("ins", "Fresh"),
                ("del", " Old"),
                ("r", " is"),
                ("r", " centered"),
                ("ins", " vivid orange"),
                ("del", " bold blue"),
                ("r", " document"),
            ];
            if period {
                expected.push(("r", "."));
            }
            segments(&dom, p, &expected);
            assert_eq!(
                dom.attribute(dom.descendants(p, Some(&W::jc_el()))[0], &W::val()),
                Some("center")
            );
            let wrappers: Vec<_> = dom
                .elements(p, None)
                .into_iter()
                .filter(|&n| dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del()))
                .collect();
            for (i, &wrapper) in wrappers.iter().enumerate() {
                assert_eq!(
                    dom.attribute(wrapper, &W::id()),
                    Some((i + 1).to_string().as_str())
                );
                assert_eq!(dom.attribute(wrapper, &W::author()), Some("Alice"));
                assert_eq!(dom.attribute(wrapper, &W::date()), Some("Fixed"));
            }
            for run in dom.descendants(p, Some(&W::r())) {
                assert_eq!(dom.element(run, &W::r_pr()).is_some(), styled);
                if styled {
                    assert_eq!(dom.descendants(run, Some(&W::name("b"))).len(), 1);
                }
            }
            for leaf in text_leaves(&dom, p) {
                if dom.value(leaf).starts_with(' ') {
                    assert_eq!(
                        dom.attribute(leaf, &XNamespace::xml().name("space")),
                        Some("preserve")
                    );
                }
            }
        }
    }
}

#[test]
fn wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format() {
    fn source_projection(dom: &mut Dom, root: NodeId, accept: bool) -> String {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.serialize_element(projected)
    }
    for terminal in [" ", "  ", "\t", "\n", "\u{a0}"] {
        for (old_terminal, new_terminal) in [(terminal, ""), ("", terminal), (terminal, terminal)] {
            let ins = format!("Fresh is centered vivid orange document{new_terminal}");
            let del = format!("Old is centered bold blue document{old_terminal}");
            let (mut dom, root) = xml(&format!(
                "<w:p><w:pPr><w:jc w:val='center'/><w:spacing w:before='120' w:after='80'/></w:pPr><w:ins w:id='17' w:author='Whitespace editor' w:date='2026-01-02T03:04:05Z'><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t xml:space='preserve'>{ins}</w:t></w:r></w:ins><w:del w:id='19' w:author='Whitespace editor' w:date='2026-01-02T03:04:05Z'><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:delText xml:space='preserve'>{del}</w:delText></w:r></w:del></w:p>"
            ));
            // These are valid XML text payloads. OOXML's xml:space
            // explicitly owns their terminal whitespace; no renderer or
            // filesystem fixture is needed to establish its lifetime.
            let expected = [
                source_projection(&mut dom, root, false),
                source_projection(&mut dom, root, true),
            ];
            let before = dom.serialize_element(root);
            free_mesh_wholesale_body_mix(&mut dom, root);
            assert_eq!(
                dom.serialize_element(root),
                before,
                "terminal={terminal:?}, old={old_terminal:?}, new={new_terminal:?}; attributed wrappers and complete properties survive"
            );
            for (side, accept) in [false, true].into_iter().enumerate() {
                assert_eq!(
                    source_projection(&mut dom, root, accept),
                    expected[side],
                    "complete independently owned source payload and formatting"
                );
            }
            free_mesh_wholesale_body_mix(&mut dom, root);
            assert_eq!(
                dom.serialize_element(root),
                before,
                "optional remesh is idempotent"
            );
        }
    }
}

#[test]
fn wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams() {
    fn owned(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<(char, String)> {
        let copy = dom.clone_subtree(root);
        let projected = if accept {
            crate::revision_processor::accept_revisions_document(dom, copy)
        } else {
            crate::revision_processor::reject_revisions_document(dom, copy)
        };
        dom.descendants(projected, Some(&W::t()))
            .into_iter()
            .flat_map(|leaf| {
                let rpr = dom
                    .parent(leaf)
                    .and_then(|run| dom.element(run, &W::r_pr()))
                    .map(|properties| dom.serialize_element(properties))
                    .unwrap_or_default();
                dom.value(leaf)
                    .chars()
                    .map(|character| (character, rpr.clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
    // M462's recorded leading delimiter is authored on the original
    // here, so both supported Word streams have a strict source oracle.
    let (mut dom, root) = xml(
        "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><w:ins w:id='17' w:author='Mesh editor' w:date='2026-01-02T03:04:05Z'><w:r><w:rPr><w:b/></w:rPr><w:t>Fresh is centered vivid orange document</w:t></w:r></w:ins><w:del w:id='19' w:author='Mesh editor' w:date='2026-01-02T03:04:05Z'><w:r><w:rPr><w:b/></w:rPr><w:delText xml:space='preserve'> Old is centered bold blue document</w:delText></w:r></w:del></w:p>",
    );
    let expected = [owned(&mut dom, root, false), owned(&mut dom, root, true)];
    free_mesh_wholesale_body_mix(&mut dom, root);
    let paragraph = ps(&dom, root)[0];
    segments(
        &dom,
        paragraph,
        &[
            ("ins", "Fresh"),
            ("del", " Old"),
            ("r", " is"),
            ("r", " centered"),
            ("ins", " vivid orange"),
            ("del", " bold blue"),
            ("r", " document"),
        ],
    );
    for (side, accept) in [false, true].into_iter().enumerate() {
        assert_eq!(
            owned(&mut dom, root, accept),
            expected[side],
            "exact characters and run formatting retain independent source ownership"
        );
    }
    for name in [W::ins(), W::del()] {
        for wrapper in dom.descendants(root, Some(&name)) {
            assert_eq!(dom.attribute(wrapper, &W::author()), Some("Mesh editor"));
            assert_eq!(
                dom.attribute(wrapper, &W::date()),
                Some("2026-01-02T03:04:05Z")
            );
        }
    }
    let once = dom.serialize_element(root);
    free_mesh_wholesale_body_mix(&mut dom, root);
    assert_eq!(dom.serialize_element(root), once);
}

#[test]
fn wholesale_mesh_keeps_low_overlap_and_nontext_changes_verbatim() {
    let too_many = "centered ".repeat(41);
    for (ins, del, extra) in [
        ("is centered fresh orange", "is centered old blue", ""),
        (too_many.as_str(), "is centered old blue document", ""),
        ("this that with from have", "this that with from have", ""),
        (
            "is alpha bravo charlie delta",
            "is echo foxtrot golf hotel",
            "",
        ),
        (
            "alpha bravo charlie delta echo",
            "alpha foxtrot golf hotel india",
            "",
        ),
        (
            "alpha beta gamma delta epsilon",
            "alpha red blue green yellow",
            "",
        ),
        (
            "Fresh is centered vivid orange document",
            "Old is centered bold blue document",
            "<w:r><w:t>equal prose</w:t></w:r>",
        ),
        (
            "Fresh is centered vivid orange document",
            "Old is centered bold blue document",
            "<w:bookmarkStart/>",
        ),
    ] {
        unchanged(
            &mixed(ins, del, "").replace("</w:p>", &format!("{extra}</w:p>")),
            free_mesh_wholesale_body_mix,
        );
    }
    unchanged(
        "<w:p><w:ins><w:r><w:t>Fresh is centered vivid orange document</w:t><w:tab/></w:r></w:ins><w:del><w:r><w:delText>Old is centered bold blue document</w:delText></w:r></w:del></w:p>",
        free_mesh_wholesale_body_mix,
    );
}

#[test]
fn lcs_rebuild_covers_equal_prefixes_and_both_unmatched_tails() {
    for (i, d, lcs, expected) in [
        (
            vec!["Anchor", "New"],
            vec!["ANCHOR", "Old"],
            vec![(0, 0)],
            vec![("r", "ANCHOR"), ("del", " Old"), ("ins", " New")],
        ),
        (
            vec!["New"],
            vec!["Old"],
            vec![],
            vec![("del", "Old"), ("ins", " New")],
        ),
        (vec!["Only"], vec![], vec![], vec![("ins", "Only")]),
        (vec![], vec!["Only"], vec![], vec![("del", "Only")]),
        (vec![], vec![], vec![], vec![]),
    ] {
        let (mut dom, root) = xml("<w:p/>");
        let p = ps(&dom, root)[0];
        let i: Vec<_> = i.into_iter().map(str::to_string).collect();
        let d: Vec<_> = d.into_iter().map(str::to_string).collect();
        rebuild_body_free_mesh_lcs(
            &mut dom,
            p,
            &i,
            &d,
            &lcs,
            RevStamp {
                author: "A",
                date: "D",
                sample_rpr: None,
            },
            false,
        );
        segments(&dom, p, &expected);
    }
}

#[test]
fn title_page_relocation_preserves_six_spacers_and_moves_only_last() {
    for title in [
        "prepared",
        "a@b",
        "2040",
        "2026",
        "march day",
        "january day",
        "february day",
        "april day",
        "june day",
        "july day",
        "august day",
        "september day",
        "october day",
        "november day",
        "december day",
        "1234",
    ] {
        for tail in ["", "<w:sectPr/>", "<w:p/>"] {
            let fragment = format!(
                "{}{}{}{tail}",
                pure("ins", title, ""),
                pure("ins", " ", "").repeat(6),
                pure("del", "Original", "")
            );
            let (mut dom, root) = xml(&fragment);
            let before = ps(&dom, root);
            let empty = before[6];
            let del = before[7];
            relocate_title_page_last_empty_after_pure_dels(&mut dom, root);
            let after = ps(&dom, root);
            assert_eq!(&after[..6], &before[..6]);
            assert_eq!(after[6], del);
            assert_eq!(after[7], empty);
            assert_eq!(markup_text(&dom, empty), " ");
            assert!(dom.descendants(empty, Some(&W::ins())).is_empty());
            assert!(dom.descendants(empty, Some(&W::del())).is_empty());
            assert_eq!(markup_text(&dom, del), "Original");
            assert_eq!(markup_text(&dom, after[0]), title);
            assert_eq!(
                dom.descendants(root, Some(&W::sect_pr())).len(),
                usize::from(tail == "<w:sectPr/>")
            );
        }
    }
}

#[test]
fn title_page_relocation_respects_cover_and_layout_boundaries() {
    for (title, count, last_empty_body, tail) in [
        ("Prepared", 5, "", ""),
        ("ordinary title", 6, "", ""),
        ("Prepared", 6, "<w:r><w:br/></w:r>", ""),
        ("Prepared", 6, "<w:r><w:drawing/></w:r>", ""),
        ("Prepared", 6, "", "<w:p><w:pPr/></w:p>"),
        ("Prepared", 6, "", "<w:p><w:ins/></w:p>"),
    ] {
        let empties = format!(
            "{}<w:p><w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>{last_empty_body}</w:p>",
            pure("ins", "", "").repeat(count - 1)
        );
        unchanged(
            &format!(
                "{}{empties}{}{}{}",
                pure("ins", title, ""),
                pure("del", "Original", ""),
                pure("del", "Second", ""),
                tail
            ),
            relocate_title_page_last_empty_after_pure_dels,
        );
    }
    unchanged(
        &format!(
            "{}{}",
            pure("ins", "", "").repeat(7),
            pure("del", "Original", "")
        ),
        relocate_title_page_last_empty_after_pure_dels,
    );
}

#[test]
fn short_head_split_preserves_title_style_and_deleted_wrapper_identity() {
    for props in ["", "<w:pStyle w:val=\"Title\"/>"] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p><w:bookmarkStart w:id=\"42\"/></w:p>",
            mixed(
                "Fresh Title",
                "one two three four five six seven eight nine ten",
                props
            )
        ));
        let old = ps(&dom, root)[0];
        let deleted = dom.element(old, &W::del()).unwrap();
        let deleted_xml = dom.serialize_element(deleted);
        split_head_short_title_long_del_mix(&mut dom, root);
        let paragraphs = ps(&dom, root);
        assert_eq!(paragraphs.len(), 3);
        segments(&dom, paragraphs[0], &[("ins", "Fresh Title")]);
        segments(
            &dom,
            paragraphs[1],
            &[("del", "one two three four five six seven eight nine ten")],
        );
        assert_eq!(dom.serialize_element(deleted), deleted_xml);
        let ppr = dom.element(paragraphs[1], &W::p_pr()).unwrap();
        assert!(dom.element(ppr, &W::p_style()).is_none());
        let mark = dom
            .element(dom.element(ppr, &W::r_pr()).unwrap(), &W::del())
            .unwrap();
        assert_eq!(dom.attribute(mark, &W::id()), Some("43"));
        assert_eq!(dom.attribute(mark, &W::author()), Some("Source"));
        assert_eq!(dom.attribute(mark, &W::date()), Some("Pinned"));
        assert_eq!(
            dom.descendants(paragraphs[0], Some(&W::p_style())).len(),
            usize::from(!props.is_empty())
        );
    }
}

#[test]
fn short_head_split_rejects_shared_words_marks_and_equal_interleaving() {
    let long = "one two three four five six seven eight nine ten";
    for (ins, del, props) in [
        ("one two three four five", long, ""),
        ("Fresh", "one two three", ""),
        ("three", long, ""),
        ("", long, ""),
        ("Fresh", long, "<w:rPr><w:ins/></w:rPr>"),
        ("Fresh", long, "<w:rPr><w:del/></w:rPr>"),
    ] {
        unchanged(&mixed(ins, del, props), split_head_short_title_long_del_mix);
    }
    unchanged(
        &format!(
            "<w:p>{}{}{}<w:r><w:t>equal</w:t></w:r></w:p>",
            revision("ins", "Fresh"),
            revision("del", long),
            revision("ins", "Again")
        ),
        split_head_short_title_long_del_mix,
    );
}

#[test]
fn stranded_mark_is_cloned_without_removing_empty_shell() {
    for bare in ["", "<w:pPr/>"] {
        let (mut dom, root) = xml(&format!(
            "<w:p>{bare}{}</w:p><w:p><w:pPr><w:rPr><w:del w:id=\"20\" w:author=\"A\" w:date=\"D\"/></w:rPr></w:pPr></w:p>",
            revision("del", "Lost text")
        ));
        let paragraphs = ps(&dom, root);
        let shell = dom.serialize_element(paragraphs[1]);
        restamp_stranded_del_mark_onto_del_only_paragraph(&mut dom, root);
        assert_eq!(ps(&dom, root), paragraphs);
        assert_eq!(dom.serialize_element(paragraphs[1]), shell);
        assert_eq!(markup_text(&dom, paragraphs[0]), "Lost text");
        assert_eq!(
            dom.serialize_element(dom.element(paragraphs[0], &W::p_pr()).unwrap()),
            dom.serialize_element(dom.element(paragraphs[1], &W::p_pr()).unwrap())
        );
        let once = dom.serialize_element(root);
        restamp_stranded_del_mark_onto_del_only_paragraph(&mut dom, root);
        assert_eq!(dom.serialize_element(root), once);
    }
}

#[test]
fn stranded_mark_rejects_layout_or_body_content_in_shell() {
    for (first_props, shell_props, shell_body) in [
        ("<w:jc w:val=\"center\"/>", "", ""),
        ("", "<w:spacing/>", ""),
        ("", "", "<w:r/>"),
        ("", "<w:rPr><w:b/></w:rPr>", ""),
    ] {
        unchanged(
            &format!(
                "<w:p><w:pPr>{first_props}</w:pPr>{}</w:p><w:p><w:pPr>{shell_props}<w:rPr><w:del/></w:rPr></w:pPr>{shell_body}</w:p>",
                revision("del", "Lost text")
            ),
            restamp_stranded_del_mark_onto_del_only_paragraph,
        );
    }
}

#[test]
fn mixed_spacing_moves_to_blank_and_preserves_existing_history() {
    let spacing = "<w:spacing w:before=\"240\" w:after=\"100\" w:line=\"300\"/>";
    for (empty_props, mark, after, expected_before, expected_ids) in [
        ("", "", "", "240", 2),
        ("<w:pPr/>", "", "", "240", 2),
        (
            "<w:pPr><w:spacing w:before=\"999\"/></w:pPr>",
            "",
            "",
            "999",
            2,
        ),
        (
            "<w:pPr><w:pPrChange w:id=\"9\"><w:pPr/></w:pPrChange></w:pPr>",
            "<w:rPr><w:ins w:id=\"10\"/></w:rPr>",
            "",
            "240",
            0,
        ),
        (
            "",
            "",
            "<w:tbl><w:tr><w:tc><w:p><w:del/></w:p></w:tc></w:tr></w:tbl>",
            "240",
            2,
        ),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p>{empty_props}<w:r/></w:p>{after}<w:sectPr/>",
            mixed("New", "Old", &format!("{spacing}{mark}"))
        ));
        let paragraphs = ps(&dom, root);
        let mut id = 40;
        mixed_spacing_to_following_empty(&mut dom, root, &settings(), &mut id);
        let mppr = dom.element(paragraphs[0], &W::p_pr()).unwrap();
        let eppr = dom.element(paragraphs[1], &W::p_pr()).unwrap();
        assert!(dom.element(mppr, &W::spacing_el()).is_none());
        let sp = dom.element(eppr, &W::spacing_el()).unwrap();
        assert_eq!(dom.attribute(sp, &W::name("before")), Some(expected_before));
        let chg = dom.element(eppr, &W::p_pr_change()).unwrap();
        assert!(change_is_empty(&dom, chg));
        if expected_ids > 0 {
            stamped(&dom, chg, "40");
            stamped(
                &dom,
                dom.element(dom.element(mppr, &W::r_pr()).unwrap(), &W::del())
                    .unwrap(),
                "41",
            );
        } else {
            assert_eq!(dom.attribute(chg, &W::id()), Some("9"));
            assert!(para_mark_revision(&dom, paragraphs[0], &W::ins()));
        }
        assert_eq!(id, 40 + expected_ids);
        segments(&dom, paragraphs[0], &[("ins", "New"), ("del", "Old")]);
        segments(&dom, paragraphs[1], &[("r", "")]);
    }
}

#[test]
fn mixed_spacing_does_not_cross_live_blocks_or_fill_textual_paragraphs() {
    for tail in [
        "<w:p><w:r><w:t>text</w:t></w:r></w:p>",
        "<w:p/><w:p/>",
        "<w:p/><w:tbl><w:tr><w:tc><w:p><w:ins/><w:del/></w:p></w:tc></w:tr></w:tbl>",
    ] {
        let (mut dom, root) = xml(&format!(
            "{}{tail}",
            mixed("New", "Old", "<w:spacing w:before=\"300\"/>")
        ));
        let before = dom.serialize_element(root);
        let mut id = 30;
        mixed_spacing_to_following_empty(&mut dom, root, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 30);
    }
}

#[test]
fn heading_spacing_parks_on_last_short_deleted_residual() {
    for count in [2, 10] {
        for last_props in [
            "",
            "<w:pPr><w:rPr><w:del/></w:rPr></w:pPr>",
            "<w:pPr><w:pPrChange w:id=\"8\"><w:pPr/></w:pPrChange></w:pPr>",
        ] {
            let (mut dom, root) = xml(&format!(
                "{}{}<w:p>{last_props}{}</w:p><w:sectPr/>",
                mixed("New", "Old", "<w:spacing w:before=\"200\" w:after=\"80\"/>"),
                pure("del", "Earlier", "").repeat(count - 1),
                revision("del", "Last")
            ));
            let paragraphs = ps(&dom, root);
            let mut id = 60;
            park_mixed_spacing_onto_trailing_pure_del(&mut dom, root, &settings(), &mut id);
            let mppr = dom.element(paragraphs[0], &W::p_pr()).unwrap();
            let lppr = dom
                .element(*paragraphs.last().unwrap(), &W::p_pr())
                .unwrap();
            assert!(dom.element(mppr, &W::spacing_el()).is_none());
            let sp = dom.element(lppr, &W::spacing_el()).unwrap();
            assert_eq!(dom.attribute(sp, &W::name("before")), Some("200"));
            assert_eq!(dom.attribute(sp, &W::name("after")), Some("80"));
            let old_history = last_props.contains("pPrChange");
            let chg = dom.element(lppr, &W::p_pr_change()).unwrap();
            if old_history {
                assert_eq!(dom.attribute(chg, &W::id()), Some("8"));
            } else {
                stamped(&dom, chg, "60");
            }
            stamped(
                &dom,
                dom.element(dom.element(mppr, &W::r_pr()).unwrap(), &W::del())
                    .unwrap(),
                if old_history { "60" } else { "61" },
            );
            assert_eq!(id, if old_history { 61 } else { 62 });
            assert_eq!(markup_text(&dom, *paragraphs.last().unwrap()), "Last");
            assert_eq!(ps(&dom, root), paragraphs);
        }
    }
}

#[test]
fn spacing_park_rejects_count_length_and_heading_thresholds() {
    for (count, before_value, after_value, last_text, last_props) in [
        (1, "200", "80", "Last".to_string(), ""),
        (11, "200", "80", "Last".to_string(), ""),
        (2, "199", "80", "Last".to_string(), ""),
        (2, "200", "79", "Last".to_string(), ""),
        (2, "bad", "80", "Last".to_string(), ""),
        (2, "200", "80", "".to_string(), ""),
        (2, "200", "80", "x".repeat(81), ""),
        (2, "200", "80", "a b c d e f g h i j k l m".to_string(), ""),
        (2, "200", "80", "Last".to_string(), "<w:spacing/>"),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}{}{}",
            mixed(
                "New",
                "Old",
                &format!("<w:spacing w:before=\"{before_value}\" w:after=\"{after_value}\"/>")
            ),
            pure("del", "Earlier", "").repeat(count - 1),
            if last_text.is_empty() {
                // A mark-only paragraph has no text node; <delText/> is
                // a text carrier even when its value is empty.
                "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr></w:p>".to_string()
            } else {
                pure("del", &last_text, last_props)
            }
        ));
        let before = dom.serialize_element(root);
        let mut id = 60;
        park_mixed_spacing_onto_trailing_pure_del(&mut dom, root, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 60);
    }
}

#[test]
fn mixed_numbering_moves_before_last_deletion_mark_or_history() {
    let num = "<w:numPr><w:ilvl w:val=\"2\"/><w:numId w:val=\"19\"/></w:numPr>";
    for count in [2, 6] {
        for props in [
            "",
            "<w:pPr><w:rPr><w:del/></w:rPr></w:pPr>",
            "<w:pPr><w:pPrChange w:id=\"5\"><w:pPr/></w:pPrChange></w:pPr>",
        ] {
            let (mut dom, root) = xml(&format!(
                "{}{}<w:p>{props}{}</w:p>",
                mixed("New", "Old", num),
                pure("del", "Earlier", "").repeat(count - 1),
                revision("del", "Last")
            ));
            let paragraphs = ps(&dom, root);
            let mut id = 70;
            park_mixed_numpr_onto_trailing_empty_pure_del(&mut dom, root, &settings(), &mut id);
            let mppr = dom.element(paragraphs[0], &W::p_pr()).unwrap();
            let lppr = dom
                .element(*paragraphs.last().unwrap(), &W::p_pr())
                .unwrap();
            assert!(dom.element(mppr, &W::num_pr()).is_none());
            let numbering = dom.element(lppr, &W::num_pr()).unwrap();
            assert_eq!(
                dom.attribute(
                    dom.element(numbering, &W::name("numId")).unwrap(),
                    &W::val()
                ),
                Some("19")
            );
            assert_eq!(
                dom.attribute(dom.element(numbering, &W::name("ilvl")).unwrap(), &W::val()),
                Some("2")
            );
            assert_eq!(dom.elements(lppr, None)[0], numbering);
            let chg = dom.element(lppr, &W::p_pr_change()).unwrap();
            assert!(change_is_empty(&dom, chg));
            if props.contains("pPrChange") {
                assert_eq!(dom.attribute(chg, &W::id()), Some("5"));
                assert_eq!(id, 70);
            } else {
                stamped(&dom, chg, "70");
                assert_eq!(id, 71);
            }
            assert_eq!(markup_text(&dom, *paragraphs.last().unwrap()), "Last");
            assert_eq!(ps(&dom, root), paragraphs);
        }
    }
}

#[test]
fn numbering_park_preserves_short_long_and_already_numbered_runs() {
    for (count, props) in [(1, ""), (7, ""), (2, "<w:numPr/>")] {
        let (mut dom, root) = xml(&format!(
            "{}{}{}",
            mixed("New", "Old", "<w:numPr><w:numId w:val=\"9\"/></w:numPr>"),
            pure("del", "Old", props),
            pure("del", "Last", "").repeat(count - 1)
        ));
        let before = dom.serialize_element(root);
        let mut id = 8;
        park_mixed_numpr_onto_trailing_empty_pure_del(&mut dom, root, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 8);
    }
}

#[test]
fn whitespace_fold_adopts_list_layout_and_replaces_inserted_mark() {
    for (carrier_props, deleted_props) in [
        ("", ""),
        ("<w:spacing w:line=\"240\"/>", ""),
        (
            "",
            "<w:pStyle w:val=\"ListParagraph\"/><w:numPr><w:numId w:val=\"2\"/></w:numPr>",
        ),
        ("<w:jc w:val=\"right\"/>", "<w:spacing w:before=\"300\"/>"),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}{}<w:sectPr/>",
            pure("ins", " ", carrier_props),
            pure("del", "one two three four five", deleted_props)
        ));
        let paragraphs = ps(&dom, root);
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        assert_eq!(ps(&dom, root), [paragraphs[0]]);
        assert!(dom.parent(paragraphs[1]).is_none());
        segments(
            &dom,
            paragraphs[0],
            &[("ins", " "), ("del", "one two three four five")],
        );
        assert!(para_mark_revision(&dom, paragraphs[0], &W::del()));
        assert!(!para_mark_revision(&dom, paragraphs[0], &W::ins()));
        let ppr = dom.element(paragraphs[0], &W::p_pr()).unwrap();
        assert_eq!(
            dom.element(ppr, &W::num_pr()).is_some(),
            deleted_props.contains("numPr")
        );
        if deleted_props.contains("spacing") {
            assert_eq!(
                dom.attribute(
                    dom.element(ppr, &W::spacing_el()).unwrap(),
                    &W::name("before")
                ),
                Some("300")
            );
        }
        assert_eq!(dom.descendants(root, Some(&W::sect_pr())).len(), 1);
    }
}

#[test]
fn whitespace_field_residue_never_inherits_heading_style() {
    for carrier_props in ["", "<w:pPr><w:rPr><w:ins/></w:rPr></w:pPr>"] {
        let (mut dom, root) = xml(&format!(
            "<w:p>{carrier_props}<w:ins><w:r><w:fldChar w:fldCharType=\"end\"/><w:t> </w:t></w:r></w:ins></w:p>{}",
            pure("del", "Heading words", "<w:pStyle w:val=\"Heading1\"/>")
        ));
        fold_whitespace_pure_ins_into_following_pure_del(&mut dom, root);
        let paragraphs = ps(&dom, root);
        assert_eq!(paragraphs.len(), 1);
        assert!(
            dom.descendants(paragraphs[0], Some(&W::p_style()))
                .is_empty()
        );
        assert_eq!(
            dom.descendants(paragraphs[0], Some(&W::fld_char())).len(),
            1
        );
        assert!(para_mark_revision(&dom, paragraphs[0], &W::del()));
        segments(
            &dom,
            paragraphs[0],
            &[("ins", " "), ("del", "Heading words")],
        );
    }
}

#[test]
fn whitespace_fold_keeps_layout_payloads_long_prose_and_cover_spacers() {
    for payload in ["<w:br/>", "<w:drawing/>", "<w:pict/>", "<w:object/>"] {
        unchanged(
            &format!(
                "<w:p><w:ins><w:r><w:t> </w:t>{payload}</w:r></w:ins></w:p>{}",
                pure("del", "Old words", "")
            ),
            fold_whitespace_pure_ins_into_following_pure_del,
        );
    }
    unchanged(
        &format!(
            "{}{}",
            pure("ins", " ", ""),
            pure(
                "del",
                "one two three four five six seven eight nine ten eleven twelve thirteen",
                ""
            )
        ),
        fold_whitespace_pure_ins_into_following_pure_del,
    );
    unchanged(
        &format!(
            "{}{}{}",
            pure("ins", "Cover", ""),
            pure("ins", "", "").repeat(3),
            pure("del", "Original words", "")
        ),
        fold_whitespace_pure_ins_into_following_pure_del,
    );
    unchanged(
        &format!(
            "{}{}",
            pure("ins", "", "").repeat(3),
            pure("del", "Original words", "")
        ),
        fold_whitespace_pure_ins_into_following_pure_del,
    );
    unchanged(
        &format!(
            "{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
            pure("ins", " ", ""),
            pure("del", "Original", "")
        ),
        fold_whitespace_pure_ins_into_following_pure_del,
    );
}

#[test]
fn comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts() {
    for comment in [
        "<w:commentRangeStart w:id=\"3\"/>",
        "<w:r><w:commentReference w:id=\"3\"/></w:r>",
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr><w:numPr><w:numId w:val=\"4\"/></w:numPr><w:pPrChange><w:pPr><w:spacing w:after=\"160\"/></w:pPr></w:pPrChange></w:pPr>{}<w:del>{comment}<w:r><w:delText>Old item</w:delText></w:r></w:del></w:p>",
            revision("ins", "New item")
        ));
        let mut id = 55;
        unmerge_comment_list_item(&mut dom, root, &settings(), &mut id);
        let paragraphs = ps(&dom, root);
        assert_eq!(paragraphs.len(), 2);
        segments(&dom, paragraphs[0], &[("ins", "New item")]);
        segments(&dom, paragraphs[1], &[("del", "Old item")]);
        let first = dom.element(paragraphs[0], &W::p_pr()).unwrap();
        let second = dom.element(paragraphs[1], &W::p_pr()).unwrap();
        assert!(dom.element(first, &W::num_pr()).is_some());
        assert!(dom.element(first, &W::p_pr_change()).is_none());
        assert!(dom.element(second, &W::num_pr()).is_none());
        for ppr in [first, second] {
            assert_eq!(
                dom.attribute(
                    dom.element(ppr, &W::spacing_el()).unwrap(),
                    &W::name("after")
                ),
                Some("160")
            );
        }
        stamped(
            &dom,
            dom.element(dom.element(first, &W::r_pr()).unwrap(), &W::ins())
                .unwrap(),
            "55",
        );
        let chg = dom.element(second, &W::p_pr_change()).unwrap();
        stamped(&dom, chg, "56");
        assert!(change_is_empty(&dom, chg));
        assert_eq!(id, 57);
        assert_eq!(
            dom.descendants(
                paragraphs[1],
                Some(&W::name(if comment.contains("commentRangeStart") {
                    "commentRangeStart"
                } else {
                    "commentReference"
                }))
            )
            .len(),
            1
        );
    }
}

#[test]
fn comment_unmerge_rejects_incomplete_or_substantive_property_history() {
    for props in [
        "<w:numPr/>",
        "<w:pPrChange><w:pPr><w:spacing/></w:pPr></w:pPrChange>",
        "<w:numPr/><w:pPrChange/>",
        "<w:numPr/><w:pPrChange><w:pPr/><w:pPr/></w:pPrChange>",
        "<w:numPr/><w:pPrChange><w:pPr><w:spacing/><w:jc/></w:pPr></w:pPrChange>",
        "<w:numPr/><w:pPrChange><w:pPr><w:jc/></w:pPr></w:pPrChange>",
        "<w:numPr/><w:rPr/><w:pPrChange><w:pPr><w:spacing/></w:pPr></w:pPrChange>",
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr>{props}</w:pPr>{}<w:del><w:commentRangeStart/><w:r><w:delText>Old</w:delText></w:r></w:del></w:p>",
            revision("ins", "New")
        ));
        let before = dom.serialize_element(root);
        let mut id = 2;
        unmerge_comment_list_item(&mut dom, root, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 2);
    }
}

fn source(dom: &mut Dom, fragment: &str) -> NodeId {
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\"><w:body>{fragment}</w:body></w:document>",
        W::URI
    ));
    dom.root(doc).unwrap()
}

#[test]
fn bare_deletion_receives_unique_revised_spacing_with_auto_line_rule() {
    for rule in ["", " w:lineRule=\"exact\""] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p>{}</w:p>",
            mixed("Match", "Changed", ""),
            revision("del", "Bare")
        ));
        let revised = source(
            &mut dom,
            &format!(
                "<w:p><w:pPr><w:spacing w:before=\"300\" w:after=\"120\" w:line=\"360\"{rule}/></w:pPr><w:r><w:t>Match</w:t></w:r></w:p>"
            ),
        );
        let original_source = dom.serialize_element(revised);
        let paragraphs = ps(&dom, root);
        let donor = dom.serialize_element(paragraphs[0]);
        let mut id = 80;
        move_spacing_onto_bare_deletion(&mut dom, root, revised, &settings(), &mut id);
        assert_eq!(dom.serialize_element(paragraphs[0]), donor);
        assert_eq!(dom.serialize_element(revised), original_source);
        let ppr = dom.element(paragraphs[1], &W::p_pr()).unwrap();
        let spacing = dom.element(ppr, &W::spacing_el()).unwrap();
        assert_eq!(dom.attribute(spacing, &W::name("before")), Some("300"));
        assert_eq!(dom.attribute(spacing, &W::name("after")), Some("120"));
        assert_eq!(dom.attribute(spacing, &W::name("line")), Some("360"));
        assert_eq!(
            dom.attribute(spacing, &W::name("lineRule")),
            Some(if rule.is_empty() { "auto" } else { "exact" })
        );
        let change = dom.element(ppr, &W::p_pr_change()).unwrap();
        stamped(&dom, change, "80");
        assert!(change_is_empty(&dom, change));
        assert_eq!(markup_text(&dom, paragraphs[1]), "Bare");
        assert_eq!(id, 81);
    }
}

#[test]
fn bare_deletion_spacing_rejects_ambiguity_reverse_order_and_equal_spacing() {
    let candidate =
        "<w:p><w:pPr><w:spacing w:before=\"300\"/></w:pPr><w:r><w:t>Match</w:t></w:r></w:p>";
    for (reverse, duplicate, props) in [
        (true, false, ""),
        (false, true, ""),
        (false, false, "<w:spacing w:before=\"300\"/>"),
    ] {
        let mix = mixed("Match", "Changed", props);
        let bare = format!("<w:p>{}</w:p>", revision("del", "Bare"));
        let (mut dom, root) = xml(&if reverse {
            format!("{bare}{mix}")
        } else {
            format!("{mix}{bare}")
        });
        let revised = source(&mut dom, &candidate.repeat(if duplicate { 2 } else { 1 }));
        let before = dom.serialize_element(root);
        let mut id = 2;
        move_spacing_onto_bare_deletion(&mut dom, root, revised, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 2);
    }
}

#[test]
fn trailing_normal_empty_adopts_source_numbering_and_empty_history() {
    for style in ["Normal", "style0"] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"{style}\"/></w:pPr></w:p>"
        ));
        let revised = source(
            &mut dom,
            "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"24\"/></w:numPr></w:pPr><w:r><w:t>Last item</w:t></w:r></w:p>",
        );
        let before = dom.serialize_element(revised);
        let mut id = 90;
        stamp_trailing_numbering(&mut dom, root, revised, &settings(), &mut id);
        let ppr = dom.element(ps(&dom, root)[0], &W::p_pr()).unwrap();
        assert!(dom.element(ppr, &W::p_style()).is_none());
        let num = dom.element(ppr, &W::num_pr()).unwrap();
        assert_eq!(
            dom.attribute(dom.element(num, &W::name("numId")).unwrap(), &W::val()),
            Some("24")
        );
        assert_eq!(
            dom.attribute(dom.element(num, &W::name("ilvl")).unwrap(), &W::val()),
            Some("1")
        );
        let change = dom.element(ppr, &W::p_pr_change()).unwrap();
        stamped(&dom, change, "90");
        assert!(change_is_empty(&dom, change));
        assert_eq!(markup_text(&dom, root), "");
        assert_eq!(dom.serialize_element(revised), before);
        assert_eq!(id, 91);
    }
}

#[test]
fn trailing_numbering_preserves_nonempty_custom_and_source_final_blanks() {
    for (props, text, source_tail) in [
        ("<w:pStyle w:val=\"Custom\"/>", "", ""),
        ("<w:pStyle w:val=\"Normal\"/>", "text", ""),
        ("<w:pStyle w:val=\"Normal\"/><w:jc/>", "", ""),
        ("<w:pStyle w:val=\"Normal\"/>", "", "<w:p/>"),
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr>{props}</w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"
        ));
        let revised = source(
            &mut dom,
            &format!("<w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>Item</w:t></w:r></w:p>{source_tail}"),
        );
        let before = dom.serialize_element(root);
        let mut id = 3;
        stamp_trailing_numbering(&mut dom, root, revised, &settings(), &mut id);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 3);
    }
}

#[test]
fn insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs() {
    for (cut, prefix, suffix) in [
        (2, "é ", "Aβ tail"),
        (3, "é A", "β tail"),
        (4, "é Aβ", " tail"),
    ] {
        let (mut dom, root) = xml(
            "<w:p><w:ins w:id=\"4\" w:author=\"Original\"><w:r><w:rPr><w:i/></w:rPr><w:t>é A</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>β</w:t><w:t> tail</w:t></w:r></w:ins></w:p>",
        );
        let p = ps(&dom, root)[0];
        let ins = dom.element(p, &W::ins()).unwrap();
        let mut id = 100;
        assert!(split_insertion_at(
            &mut dom,
            ins,
            cut,
            &Some("Carried".to_string()),
            &Some("Fixed".to_string()),
            &mut id
        ));
        segments(&dom, p, &[("ins", prefix), ("ins", suffix)]);
        let insertions = dom.elements(p, Some(&W::ins()));
        assert_eq!(dom.attribute(insertions[0], &W::id()), Some("4"));
        assert_eq!(dom.attribute(insertions[0], &W::author()), Some("Original"));
        assert_eq!(dom.attribute(insertions[1], &W::id()), Some("100"));
        assert_eq!(dom.attribute(insertions[1], &W::author()), Some("Carried"));
        assert_eq!(dom.attribute(insertions[1], &W::date()), Some("Fixed"));
        assert_eq!(id, 101);
        assert_eq!(markup_text(&dom, p), "é Aβ tail");
        assert_eq!(dom.descendants(insertions[1], Some(&W::name("b"))).len(), 1);
        if cut == 2 {
            assert_eq!(dom.descendants(insertions[1], Some(&W::name("i"))).len(), 1);
            let leaf = text_leaves(&dom, insertions[0])[0];
            assert_eq!(
                dom.attribute(leaf, &XNamespace::xml().name("space")),
                Some("preserve")
            );
        }
    }
    for cut in [0, 3, 10] {
        let (mut dom, root) = xml("<w:p><w:ins><w:r><w:t>abc</w:t></w:r></w:ins></w:p>");
        let ins = dom.element(ps(&dom, root)[0], &W::ins()).unwrap();
        let before = dom.serialize_element(root);
        let mut id = 100;
        assert!(!split_insertion_at(
            &mut dom, ins, cut, &None, &None, &mut id
        ));
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 100);
    }
}

#[test]
fn carried_insert_and_delete_split_preserves_neighbors_and_attribution() {
    for deletion in [false, true] {
        for index in 0..3 {
            let parent = if deletion { "ins" } else { "del" };
            let mut runs = String::new();
            for (n, value) in ["Before", "Carried", "After"].iter().enumerate() {
                let marker = if n == index {
                    if deletion {
                        " pt:PreDelete=\"1\" pt:PreDelAuthor=\"Prior\" pt:PreDelDate=\"Earlier\""
                    } else {
                        " pt:PreIns=\"1\" pt:PreInsAuthor=\"Prior\" pt:PreInsDate=\"Earlier\""
                    }
                } else {
                    ""
                };
                let leaf = if deletion { "t" } else { "delText" };
                runs.push_str(&format!("<w:r{marker}><w:{leaf}>{value}</w:{leaf}></w:r>"));
            }
            let (mut dom, root) = xml(&format!(
                "<w:p><w:{parent} w:id=\"4\" w:author=\"Wrapper\" w:date=\"Old\">{runs}</w:{parent}></w:p>"
            ));
            let mut id = 200;
            if deletion {
                convert_stamped_predeletes(&mut dom, root, &settings(), &mut id);
            } else {
                convert_stamped_preins(&mut dom, root, &settings(), &mut id);
            }
            let p = ps(&dom, root)[0];
            let changed = if deletion { "del" } else { "ins" };
            let values = ["Before", "Carried", "After"];
            let mut expected = Vec::new();
            if index > 0 {
                expected.push((
                    parent,
                    if index == 1 {
                        "Before"
                    } else {
                        "BeforeCarried"
                    },
                ));
            }
            expected.push((changed, values[index]));
            if index < 2 {
                expected.push((parent, if index == 0 { "CarriedAfter" } else { "After" }));
            }
            segments(&dom, p, &expected);
            let wrapper = dom.element(p, &W::name(changed)).unwrap();
            assert_eq!(dom.attribute(wrapper, &W::id()), Some("200"));
            assert_eq!(dom.attribute(wrapper, &W::author()), Some("Prior"));
            assert_eq!(dom.attribute(wrapper, &W::date()), Some("Earlier"));
            assert_eq!(markup_text(&dom, p), "BeforeCarriedAfter");
            let leaf_kind = if deletion { W::del_text() } else { W::t() };
            assert_eq!(dom.descendants(wrapper, Some(&leaf_kind)).len(), 1);
            assert_eq!(id, if index < 2 { 202 } else { 201 });
            for run in dom.descendants(p, Some(&W::r())) {
                assert!(
                    dom.attributes(run)
                        .iter()
                        .all(|(name, _)| name.namespace_name() != PT::URI)
                );
            }
            for sibling in dom.elements(p, Some(&W::name(parent))) {
                assert_eq!(dom.attribute(sibling, &W::author()), Some("Wrapper"));
                assert_eq!(dom.attribute(sibling, &W::date()), Some("Old"));
            }
        }
    }
}

#[test]
fn heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell() {
    for prior in [
        "<w:spacing w:before=\"200\" w:line=\"240\"/>",
        "<w:spacing w:before=\"199\" w:line=\"240\"/>",
        "<w:spacing w:before=\"200\"/>",
    ] {
        let (mut dom, root) = xml(&format!(
            "<w:p><w:pPr>{prior}</w:pPr></w:p>{}",
            mixed(
                "New",
                "Old",
                "<w:pPrChange w:id=\"3\"><w:pPr><w:spacing w:before=\"300\" w:line=\"360\"/></w:pPr></w:pPrChange>"
            )
        ));
        let before = dom.serialize_element(root);
        promote_heading_spacing_from_pprchange_on_last_mix(&mut dom, root);
        let last = *ps(&dom, root).last().unwrap();
        let ppr = dom.element(last, &W::p_pr()).unwrap();
        if prior.contains("199") || !prior.contains("line") {
            assert_eq!(dom.serialize_element(root), before);
        } else {
            let sp = dom.element(ppr, &W::spacing_el()).unwrap();
            assert_eq!(dom.attribute(sp, &W::name("before")), Some("300"));
            assert_eq!(dom.attribute(sp, &W::name("line")), Some("360"));
            let chg = dom.element(ppr, &W::p_pr_change()).unwrap();
            assert!(change_is_empty(&dom, chg));
            assert_eq!(dom.attribute(chg, &W::id()), Some("3"));
            segments(&dom, last, &[("ins", "New"), ("del", "Old")]);
        }
    }
}

#[test]
fn old_body_alignment_promotes_only_for_aligned_revised_paragraphs() {
    for (extra, excluded) in [
        ("", false),
        ("<w:rPr><w:b/></w:rPr>", false),
        ("<w:ind w:left=\"100\"/>", false),
        ("", true),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p/>",
            mixed(
                "one two three",
                "four five six",
                &format!(
                    "<w:pPrChange><w:pPr><w:jc w:val=\"right\"/>{extra}</w:pPr></w:pPrChange>"
                )
            )
        ));
        let before = dom.serialize_element(root);
        let unaligned = if excluded {
            HashSet::from(["one two three".to_string()])
        } else {
            HashSet::new()
        };
        promote_live_jc_from_pprchange_on_body_mix(&mut dom, root, &unaligned);
        if excluded {
            assert_eq!(dom.serialize_element(root), before);
        } else {
            let ppr = dom.element(ps(&dom, root)[0], &W::p_pr()).unwrap();
            assert_eq!(
                dom.attribute(dom.element(ppr, &W::jc_el()).unwrap(), &W::val()),
                Some("right")
            );
            assert_eq!(
                dom.element(ppr, &W::p_pr_change()).is_some(),
                extra.contains("ind")
            );
            segments(
                &dom,
                ps(&dom, root)[0],
                &[("ins", "one two three"), ("del", "four five six")],
            );
        }
    }
}

#[test]
fn leading_deleted_echo_is_removed_only_with_one_or_two_tokens() {
    for (echo, remove) in [
        ("Alpha", true),
        ("--ALPHA beta!", true),
        ("Alpha beta gamma", false),
        ("Beta", false),
        (" ", false),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p>{}{}</w:p>",
            pure("ins", "Alpha heading", ""),
            revision("del", echo),
            revision("ins", "Remaining")
        ));
        let paragraphs = ps(&dom, root);
        let previous = dom.serialize_element(paragraphs[0]);
        strip_leading_del_echoing_prev_pure_i(&mut dom, root);
        assert_eq!(dom.serialize_element(paragraphs[0]), previous);
        if remove {
            segments(&dom, paragraphs[1], &[("ins", "Remaining")]);
        } else {
            segments(&dom, paragraphs[1], &[("del", echo), ("ins", "Remaining")]);
        }
    }
}

#[test]
fn tabbed_title_mark_matches_original_body_and_excludes_table_only_titles() {
    for in_table in [false, true] {
        let (mut dom, root) = xml(
            "<w:p><w:ins><w:r><w:tab/><w:t>New title</w:t></w:r></w:ins><w:del><w:r><w:delText>Old title</w:delText></w:r></w:del></w:p>",
        );
        let original_paragraph = "<w:p><w:r><w:t>Old title</w:t></w:r></w:p>";
        let original = source(
            &mut dom,
            &if in_table {
                format!("<w:tbl><w:tr><w:tc>{original_paragraph}</w:tc></w:tr></w:tbl>")
            } else {
                original_paragraph.to_string()
            },
        );
        let before = dom.serialize_element(root);
        let mut id = 300;
        mark_tabbed_deleted_titles(&mut dom, root, original, &settings(), &mut id);
        if in_table {
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(id, 300);
        } else {
            let p = ps(&dom, root)[0];
            let ppr = dom.element(p, &W::p_pr()).unwrap();
            assert_eq!(dom.elements(p, None)[0], ppr);
            stamped(
                &dom,
                dom.element(dom.element(ppr, &W::r_pr()).unwrap(), &W::del())
                    .unwrap(),
                "300",
            );
            segments(&dom, p, &[("ins", "New title"), ("del", "Old title")]);
            assert_eq!(dom.descendants(p, Some(&W::name("tab"))).len(), 1);
            assert_eq!(id, 301);
        }
    }
}

#[test]
fn duplicate_deleted_default_style_drops_only_after_matching_marked_text() {
    for (earlier_text, earlier_props, remove) in [
        ("Same", "", true),
        ("Different", "", false),
        ("Same", "<w:pStyle w:val=\"Normal\"/>", false),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}<w:p><w:pPr><w:pStyle w:val=\"Normal\"/><w:pPrChange><w:pPr/></w:pPrChange></w:pPr>{}</w:p>",
            pure("del", earlier_text, earlier_props),
            revision("del", "Same")
        ));
        let paragraphs = ps(&dom, root);
        let previous = dom.serialize_element(paragraphs[0]);
        drop_duplicate_normal_style(&mut dom, root);
        assert_eq!(dom.serialize_element(paragraphs[0]), previous);
        assert_eq!(dom.element(paragraphs[1], &W::p_pr()).is_none(), remove);
        segments(&dom, paragraphs[1], &[("del", "Same")]);
        if !remove {
            assert_eq!(dom.descendants(paragraphs[1], Some(&W::p_style())).len(), 1);
        }
    }
}

#[test]
fn live_heading_history_is_added_only_when_last_history_is_empty() {
    for (before_value, line, old_props, expected) in [
        ("200", " w:line=\"300\"", "", true),
        ("199", " w:line=\"300\"", "", false),
        ("200", "", "", false),
        ("200", " w:line=\"300\"", "<w:jc/>", false),
        ("200", " w:line=\"300\"", "<w:rPr><w:b/></w:rPr>", true),
    ] {
        let (mut dom, root) = xml(&format!(
            "{}{}",
            mixed(
                "New",
                "Old",
                &format!("<w:spacing w:before=\"{before_value}\"{line}/>")
            ),
            mixed(
                "Last",
                "End",
                &format!("<w:pPrChange w:id=\"4\"><w:pPr>{old_props}</w:pPr></w:pPrChange>")
            )
        ));
        let before = dom.serialize_element(root);
        let mut id = 400;
        ensure_empty_pprchange_on_live_heading_spacing(&mut dom, root, &settings(), &mut id);
        if expected {
            let p = ps(&dom, root)[0];
            let ppr = dom.element(p, &W::p_pr()).unwrap();
            let chg = dom.element(ppr, &W::p_pr_change()).unwrap();
            stamped(&dom, chg, "400");
            assert!(change_is_empty(&dom, chg));
            let sp = dom.element(ppr, &W::spacing_el()).unwrap();
            assert_eq!(dom.attribute(sp, &W::name("before")), Some("200"));
            assert_eq!(dom.attribute(sp, &W::name("line")), Some("300"));
            segments(&dom, p, &[("ins", "New"), ("del", "Old")]);
            assert_eq!(id, 401);
        } else {
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(id, 400);
        }
    }
}
