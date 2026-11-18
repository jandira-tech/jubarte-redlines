// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

const MAIN: &str = "word/document.xml";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";

fn xml(local: &str, body: &str) -> String {
    format!(
        "<w:{local} xmlns:w=\"{}\" xmlns:r=\"{}\">{body}</w:{local}>",
        W::URI,
        R::URI
    )
}

fn root(dom: &mut Dom, source: &str) -> NodeId {
    let doc = dom.parse_xdocument(source);
    dom.root(doc).expect("well-formed fixture")
}

// Semantic properties include every attribute and child, independently of
// prefix declaration locations chosen when the OPC part is serialized.
fn signature(dom: &Dom, node: NodeId) -> String {
    let name = dom.name(node).unwrap();
    let mut attrs: Vec<_> = dom
        .attributes_iter(node)
        .map(|(n, v)| {
            (
                n.namespace_name().to_string(),
                n.local_name().to_string(),
                v.to_string(),
            )
        })
        .collect();
    attrs.sort();
    let children: Vec<_> = dom
        .elements(node, None)
        .into_iter()
        .map(|n| signature(dom, n))
        .collect();
    format!(
        "{}:{}:{attrs:?}:{children:?}",
        name.namespace_name(),
        name.local_name()
    )
}

#[derive(Debug, PartialEq, Eq)]
struct SourceView {
    text: Vec<(char, String, String)>,
    paragraphs: Vec<(String, String)>,
    payload: Vec<(String, String)>,
    geometry: Vec<String>,
}

fn view(pkg: &PartFs, part: &str) -> SourceView {
    let mut dom = Dom::new();
    let r = root(&mut dom, &pkg.part_string(part).expect(part));
    crate::comparer::finalize::remove_powertools_scratch_markup(&mut dom, r);
    // Empty property carriers have no formatting effect and may remain
    // after the revision processor removes their last change child.
    for local in ["rPr", "pPr"] {
        for n in dom.descendants(r, Some(&W::name(local))) {
            if dom.elements(n, None).is_empty() && !dom.has_attributes(n) {
                dom.remove(n);
            }
        }
    }
    let paragraphs = dom
        .descendants(r, Some(&W::p()))
        .into_iter()
        .map(|p| {
            (
                dom.value(p),
                dom.element(p, &W::p_pr())
                    .map(|n| signature(&dom, n))
                    .unwrap_or_default(),
            )
        })
        .collect();
    let mut text = Vec::new();
    for t in dom.descendants(r, Some(&W::t())) {
        let ancestors = dom.ancestors(t, None);
        let prop = |local: &str, property: &str| {
            ancestors
                .iter()
                .copied()
                .find(|&n| dom.name_is(n, &W::name(local)))
                .and_then(|n| dom.element(n, &W::name(property)))
                .filter(|&n| !dom.elements(n, None).is_empty() || dom.has_attributes(n))
                .map(|n| signature(&dom, n))
                .unwrap_or_default()
        };
        let ppr = prop("p", "pPr");
        let rpr = prop("r", "rPr");
        text.extend(dom.value(t).chars().map(|c| (c, ppr.clone(), rpr.clone())));
    }
    let payload = dom
        .descendants(r, None)
        .into_iter()
        .filter(|&n| {
            ["br", "tab", "fldChar", "instrText"]
                .iter()
                .any(|local| dom.name_is(n, &W::name(local)))
        })
        .map(|n| (signature(&dom, n), dom.value(n)))
        .collect();
    let geometry = dom
        .descendants(r, None)
        .into_iter()
        .filter(|&n| {
            ["tblPr", "tblGrid", "tcPr", "sectPr"]
                .iter()
                .any(|local| dom.name_is(n, &W::name(local)))
        })
        .map(|n| signature(&dom, n))
        .collect();
    SourceView {
        text,
        paragraphs,
        payload,
        geometry,
    }
}

fn add_part(pkg: &mut PartFs, local: &str, body: &str, kind: &str, ctype: &str) {
    let path = format!("word/{local}.xml");
    pkg.set_part(&path, xml(kind, body).into_bytes());
    pkg.add_content_type_override(&format!("/{path}"), ctype);
    pkg.add_document_relationship(MAIN, &format!("{REL}{local}"), &format!("{local}.xml"));
}

fn package(revised: bool, layout: usize, format: usize, numbered: bool, header: bool) -> PartFs {
    let mut pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/word_probes/tokens/cell_a.docx"
    ))
    .unwrap();
    for part in pkg.parts() {
        pkg.remove_part(&part);
    }
    let empty =
        b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>";
    pkg.set_part("_rels/.rels", empty.to_vec());
    pkg.set_part("word/_rels/document.xml.rels", empty.to_vec());
    pkg.add_package_relationship(&format!("{REL}officeDocument"), MAIN);
    pkg.add_content_type_override(
        "/word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
    );
    let ppr = [
        "",
        "<w:keepNext/><w:spacing w:before='120' w:after='80'/>",
        "<w:ind w:left='360' w:firstLine='120'/><w:jc w:val='both'/>",
        "<w:tabs><w:tab w:val='left' w:pos='720'/></w:tabs><w:spacing w:line='300' w:lineRule='auto'/>",
    ][format];
    let rpr = [
        "",
        "<w:b/><w:color w:val='234567'/>",
        "<w:i/><w:sz w:val='28'/><w:lang w:val='en-US'/>",
        "<w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:u w:val='single'/>",
    ][format];
    let num = if numbered {
        "<w:numPr><w:ilvl w:val='0'/><w:numId w:val='9'/></w:numPr>"
    } else {
        ""
    };
    let (before_num, after_num) = if format == 1 {
        ("<w:keepNext/>", "<w:spacing w:before='120' w:after='80'/>")
    } else {
        ("", ppr)
    };
    let mut body = String::new();
    for i in 0..7 {
        let word = if i == 3 && revised {
            "revised"
        } else {
            "original"
        };
        let text = format!("Clause {i} preserves the {word} obligation and delivery schedule.");
        let extra = if i == 5 {
            match layout {
                1 => "<w:tab/>",
                2 => "<w:br/>",
                3 => "<w:br w:type='page'/>",
                _ => "",
            }
        } else {
            ""
        };
        body.push_str(&format!("<w:p><w:pPr>{before_num}{num}{after_num}</w:pPr><w:r><w:rPr>{rpr}</w:rPr><w:t>{text}</w:t>{extra}</w:r></w:p>"));
    }
    if layout == 3 {
        body.push_str("<w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='3600'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='3600' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Unchanged table appendix</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>");
    }
    let href = if header {
        pkg.set_part("word/header1.xml", xml("hdr", "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Agreement heading</w:t></w:r></w:p>").into_bytes());
        pkg.add_content_type_override(
            "/word/header1.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        );
        let id = pkg.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
        format!("<w:headerReference w:type='default' r:id='{id}'/>")
    } else {
        String::new()
    };
    body.push_str(&format!("<w:sectPr>{href}<w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='1440' w:right='1440' w:bottom='1440' w:left='1440' w:header='720' w:footer='720' w:gutter='0'/></w:sectPr>"));
    pkg.set_part(
        MAIN,
        xml("document", &format!("<w:body>{body}</w:body>")).into_bytes(),
    );
    add_part(
        &mut pkg,
        "styles",
        "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style>",
        "styles",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
    );
    if numbered {
        add_part(
            &mut pkg,
            "numbering",
            "<w:abstractNum w:abstractNumId='0'><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='left'/></w:lvl></w:abstractNum><w:num w:numId='9'><w:abstractNumId w:val='0'/></w:num>",
            "numbering",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        );
    }
    pkg
}

#[test]
fn public_comparison_preserves_complete_sources_across_package_features() {
    for layout in 0..4 {
        for format in 0..4 {
            for numbered in [false, true] {
                for header in [false, true] {
                    let a = package(false, layout, format, numbered, header);
                    let b = package(true, layout, format, numbered, header);
                    let settings = WmlComparerSettings {
                        author_for_revisions: "Package matrix editor".into(),
                        date_time_for_revisions: "2001-02-03T04:05:06Z".into(),
                        merge_replaced_paragraphs: false,
                        ..WmlComparerSettings::default()
                    };
                    let compared = compare_documents_with_settings(
                        &a.to_zip().unwrap(),
                        &b.to_zip().unwrap(),
                        &settings,
                    )
                    .unwrap();
                    assert!(docx_has_tracked_changes(
                        &compared,
                        crate::admission::InputLimits::compare()
                    ));
                    let accepted = PartFs::open(&accept_revisions(&compared).unwrap()).unwrap();
                    let rejected = PartFs::open(&reject_revisions(&compared).unwrap()).unwrap();
                    let context = format!(
                        "layout={layout} format={format} numbered={numbered} header={header}"
                    );
                    assert_eq!(view(&accepted, MAIN), view(&b, MAIN), "accept {context}");
                    assert_eq!(view(&rejected, MAIN), view(&a, MAIN), "reject {context}");
                    if header {
                        assert_eq!(
                            view(&accepted, "word/header1.xml"),
                            view(&b, "word/header1.xml"),
                            "header accept {context}"
                        );
                        assert_eq!(
                            view(&rejected, "word/header1.xml"),
                            view(&a, "word/header1.xml"),
                            "header reject {context}"
                        );
                    }
                    for projected in [&accepted, &rejected] {
                        let rels = projected.read_rels_for(MAIN).unwrap();
                        for rel in rels
                            .items
                            .iter()
                            .filter(|r| r.target_mode.as_deref() != Some("External"))
                        {
                            assert!(
                                projected
                                    .part_bytes(&projected.resolve_rel_target(MAIN, &rel.target))
                                    .is_some(),
                                "dangling {} {context}",
                                rel.target
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides() {
    for (local, default_on) in [
        ("widowControl", true),
        ("autoSpaceDE", true),
        ("autoSpaceDN", true),
        ("keepNext", false),
        ("keepLines", false),
        ("pageBreakBefore", false),
        ("suppressLineNumbers", false),
        ("suppressAutoHyphens", false),
    ] {
        for a_on in [false, true] {
            for b in [None, Some(false), Some(true)] {
                for own in [false, true] {
                    let mut dom = Dom::new();
                    let a_xml = xml(
                        "styles",
                        &format!(
                            "<w:docDefaults><w:pPrDefault><w:pPr><w:{local} w:val='{}'/><w:spacing w:beforeLines='100' w:afterLines='200'/></w:pPr></w:pPrDefault></w:docDefaults>",
                            usize::from(a_on)
                        ),
                    );
                    let a = root(&mut dom, &a_xml);
                    let declared = b
                        .map(|v| format!("<w:{local} w:val='{}'/>", usize::from(v)))
                        .unwrap_or_default();
                    let b_xml = if own {
                        xml(
                            "styles",
                            &format!(
                                "<w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:beforeLines='100'/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type='paragraph' w:styleId='Normal'><w:name w:val='Normal'/><w:pPr>{declared}</w:pPr></w:style>"
                            ),
                        )
                    } else {
                        xml(
                            "styles",
                            &format!(
                                "<w:docDefaults><w:pPrDefault><w:pPr>{declared}<w:spacing w:beforeLines='100'/></w:pPr></w:pPrDefault></w:docDefaults>"
                            ),
                        )
                    };
                    let br = root(&mut dom, &b_xml);
                    let bs = dom.element(br, &W::name("style"));
                    let before = dom.serialize_element(a);
                    let delta = doc_default_ppr_delta(&mut dom, a, br, bs);
                    let expected = match b {
                        Some(v) if v == a_on => None,
                        Some(v) => Some(v),
                        None if a_on == default_on => None,
                        None => Some(default_on),
                    };
                    assert_eq!(
                        delta.elements.len(),
                        usize::from(expected.is_some()),
                        "{local} A={a_on} B={b:?} own={own}"
                    );
                    if let Some(value) = expected {
                        assert_eq!(delta.elements[0].0, local);
                        assert_eq!(!on_off_is_off(&dom, delta.elements[0].1), value);
                    }
                    assert_eq!(delta.spacing, [("afterLines".to_string(), "0".to_string())]);
                    assert_eq!(
                        dom.serialize_element(a),
                        before,
                        "source defaults must remain immutable"
                    );
                }
            }
        }
    }
}
#[test]
fn normal_sync_preserves_history_and_replaces_indent_in_schema_order() {
    for spacing in [false, true] {
        for history in [false, true] {
            for revised_indent in [false, true] {
                let mut dom = Dom::new();
                let sp = if spacing {
                    "<w:spacing w:after='80'/>"
                } else {
                    ""
                };
                let old = if history {
                    "<w:pPrChange w:id='12' w:author='Original editor' w:date='2001-02-03T04:05:06Z'><w:pPr><w:ind w:left='240'/></w:pPr></w:pPrChange>"
                } else {
                    ""
                };
                let r = root(
                    &mut dom,
                    &xml(
                        "styles",
                        &format!(
                            "<w:style w:type='paragraph' w:styleId='Normal'><w:name w:val='Normal'/><w:pPr><w:keepLines/>{sp}<w:ind w:left='240'/>{old}</w:pPr></w:style>"
                        ),
                    ),
                );
                let ppr = dom.descendants(r, Some(&W::p_pr()))[0];
                let history_before = dom
                    .element(ppr, &W::p_pr_change())
                    .map(|n| signature(&dom, n));
                let indent = if revised_indent {
                    "<w:ind w:left='720' w:hanging='360'/>"
                } else {
                    ""
                };
                let b = root(
                    &mut dom,
                    &xml(
                        "style",
                        &format!(
                            "<w:name w:val='Normal'/><w:pPr><w:keepNext/><w:tabs><w:tab w:val='left' w:pos='720'/></w:tabs>{indent}<w:jc w:val='both'/></w:pPr>"
                        ),
                    ),
                );
                let b_before = dom.serialize_element(b);
                sync_normal_ppr_with_revised(&mut dom, ppr, Some(b));
                let children: Vec<_> = dom
                    .elements(ppr, None)
                    .into_iter()
                    .map(|n| dom.name(n).unwrap().local_name().to_string())
                    .collect();
                let mut expected = vec!["keepNext", "tabs"];
                if spacing {
                    expected.push("spacing");
                }
                if revised_indent {
                    expected.push("ind");
                }
                expected.push("jc");
                if history {
                    expected.push("pPrChange");
                }
                assert_eq!(children, expected);
                assert_eq!(
                    dom.element(ppr, &W::p_pr_change())
                        .map(|n| signature(&dom, n)),
                    history_before
                );
                if revised_indent {
                    let ind = dom.element(ppr, &W::name("ind")).unwrap();
                    assert_eq!(dom.attribute(ind, &W::name("left")), Some("720"));
                    assert_eq!(dom.attribute(ind, &W::name("hanging")), Some("360"));
                }
                assert_eq!(dom.serialize_element(b), b_before);
                let once = signature(&dom, ppr);
                sync_normal_ppr_with_revised(&mut dom, ppr, Some(b));
                assert_eq!(signature(&dom, ppr), once);
            }
        }
    }
}
#[test]
fn accepted_source_revision_converges_to_identical_final_before_comparison() {
    for format in 0..4 {
        let mut a = package(false, 0, format, true, true);
        let source = a
            .part_string(MAIN)
            .unwrap()
            .replacen(
                "<w:r>",
                "<w:ins w:id='17' w:author='Prior editor' w:date='2001-02-03T04:05:06Z'><w:r>",
                1,
            )
            .replacen("</w:r>", "</w:r></w:ins>", 1);
        a.set_part(MAIN, source.into_bytes());
        let a_bytes = a.to_zip().unwrap();
        assert!(docx_has_tracked_changes(
            &a_bytes,
            crate::admission::InputLimits::compare()
        ));
        let final_bytes =
            accept_source_revisions(&a_bytes, crate::admission::InputLimits::compare()).unwrap();
        assert!(!docx_has_tracked_changes(
            &final_bytes,
            crate::admission::InputLimits::compare()
        ));
        let settings = WmlComparerSettings {
            author_for_revisions: "Package matrix editor".into(),
            date_time_for_revisions: "2001-02-03T04:05:06Z".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        };
        let result = compare_documents_with_settings(&a_bytes, &final_bytes, &settings).unwrap();
        assert!(
            !docx_has_tracked_changes(&result, crate::admission::InputLimits::compare()),
            "an accepted pre-existing revision is not a new edit"
        );
        let final_pkg = PartFs::open(&final_bytes).unwrap();
        let accepted = PartFs::open(&accept_revisions(&result).unwrap()).unwrap();
        let rejected = PartFs::open(&reject_revisions(&result).unwrap()).unwrap();
        for part in [MAIN, "word/header1.xml"] {
            assert_eq!(view(&accepted, part), view(&final_pkg, part));
            assert_eq!(view(&rejected, part), view(&final_pkg, part));
        }
        assert_eq!(
            accepted.part_string("word/numbering.xml"),
            final_pkg.part_string("word/numbering.xml")
        );
        assert_eq!(
            rejected.part_string("word/styles.xml"),
            final_pkg.part_string("word/styles.xml")
        );
    }
}
