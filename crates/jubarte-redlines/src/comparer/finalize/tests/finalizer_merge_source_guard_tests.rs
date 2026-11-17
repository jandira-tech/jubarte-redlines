// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const PROPS: &str =
    "<w:pStyle w:val='BodyText'/><w:spacing w:before='120' w:after='240'/><w:ind w:left='360'/>";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

fn package(body: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:pt='{}' xmlns:r='http://schemas.openxmlformats.org/officeDocument/2006/relationships' xmlns:w14='http://schemas.microsoft.com/office/word/2010/wordml'><w:body>{body}</w:body></w:document>", W::URI, PT::URI));
    let root = dom.root(doc).expect("document");
    let body = dom.element(root, &W::body()).expect("body");
    (dom, root, body)
}
fn semantic(dom: &Dom, node: NodeId) -> String {
    if !dom.is_element(node) {
        return format!("text:{:?}", dom.text_value(node));
    }
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| {
            (
                name.namespace_name().to_owned(),
                name.local_name().to_owned(),
                value,
            )
        })
        .collect::<Vec<_>>();
    attrs.sort();
    format!(
        "{:?}{attrs:?}[{}]",
        dom.name(node),
        dom.nodes(node)
            .into_iter()
            .map(|child| semantic(dom, child))
            .collect::<String>()
    )
}
fn run(kind: &str, text: &str, author: &str, date: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    let run = format!("<w:r>{FORMAT}<w:{tag} xml:space='preserve'>{text}</w:{tag}></w:r>");
    if kind == "live" {
        run
    } else {
        format!("<w:{kind} w:id='17' w:author='{author}' w:date='{date}'>{run}</w:{kind}>")
    }
}
fn para(kind: &str, text: &str, props: &str) -> String {
    format!(
        "<w:p><w:pPr>{props}</w:pPr>{}</w:p>",
        run(kind, text, "Boundary editor", DATE)
    )
}
fn marked(
    kind: &str,
    text: &str,
    props: &str,
    mark_author: &str,
    mark_date: &str,
    body_author: &str,
    body_date: &str,
) -> String {
    format!(
        "<w:p><w:pPr>{props}<w:rPr><w:{kind} w:id='18' w:author='{mark_author}' w:date='{mark_date}'/></w:rPr></w:pPr>{}</w:p>",
        run(kind, text, body_author, body_date)
    )
}
fn assert_body(dom: &Dom, body: NodeId, expected: &str, label: &str) {
    let (want, _, wb) = package(expected);
    let actual = semantic(dom, body);
    let expected = semantic(&want, wb);
    if actual != expected {
        let first = actual
            .chars()
            .zip(expected.chars())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| actual.chars().count().min(expected.chars().count()));
        let context = |text: &str| {
            text.chars()
                .skip(first.saturating_sub(64))
                .take(240)
                .collect::<String>()
        };
        panic!(
            "{label}: complete source tree differs at character {first}; actual length {}, expected length {}; actual context {:?}; expected context {:?}",
            actual.len(),
            expected.len(),
            context(&actual),
            context(&expected)
        );
    }
}
fn words(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|n| format!("{prefix}{n}"))
        .collect::<Vec<_>>()
        .join(" ")
}

// M88 keeps original structural paragraph properties when the new mark
// is bare. A field with a textual result follows that same rule. These are phase
// contracts: complete authored source history is restored by later passes.
#[test]
fn related_replacement_selects_the_documented_structural_carrier_without_losing_run_sources() {
    for case in [
        "new-structural",
        "old-structural",
        "both-bare",
        "field-heading",
        "field-new-structural",
    ] {
        for fold_boundary in [false, true] {
            let old_props = match case {
                "old-structural" => {
                    "<w:numPr><w:ilvl w:val='1'/><w:numId w:val='7'/></w:numPr><w:spacing w:after='80'/>"
                }
                "field-heading" | "field-new-structural" => {
                    "<w:pStyle w:val='Heading1'/><w:spacing w:after='240'/>"
                }
                "new-structural" => "<w:pStyle w:val='BodyText'/><w:spacing w:after='80'/>",
                _ => "",
            };
            let new_props = if matches!(case, "new-structural" | "field-new-structural") {
                "<w:pStyle w:val='Heading2'/><w:spacing w:after='120'/><w:ind w:left='360'/>"
            } else {
                ""
            };
            let old = marked(
                "del",
                "Related original heading",
                old_props,
                "Boundary editor",
                DATE,
                "Boundary editor",
                DATE,
            )
            .replacen("<w:rPr>", "<w:rPr><w:i/>", 1);
            let mut new = marked(
                "ins",
                "Related revised heading",
                new_props,
                "Boundary editor",
                DATE,
                "Boundary editor",
                DATE,
            )
            .replacen("<w:rPr>", "<w:rPr><w:b/>", 1);
            if case.starts_with("field-") {
                let field = format!(
                    "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> REF AuthoredHeading </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r>{FORMAT}<w:t xml:space='preserve'>Related revised heading</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>"
                );
                new = new.replace(
                    &format!(
                        "<w:r>{FORMAT}<w:t xml:space='preserve'>Related revised heading</w:t></w:r>"
                    ),
                    &field,
                );
            }
            let anchor = para("live", "Independently live source paragraph", PROPS);
            let input = old.clone() + &new + &anchor + TABLE;
            let (mut dom, _, body) = package(&input);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", fold_boundary);
            let property_source = if matches!(case, "old-structural" | "field-heading") {
                &old
            } else {
                &new
            };
            let start = property_source.find("<w:pPr>").unwrap();
            let end = property_source.find("</w:pPr>").unwrap() + "</w:pPr>".len();
            let old_body = &old[old.find("</w:pPr>").unwrap() + "</w:pPr>".len()..old.len() - 6];
            let new_body = &new[new.find("</w:pPr>").unwrap() + "</w:pPr>".len()..new.len() - 6];
            let expected = format!(
                "<w:p>{}{new_body}{old_body}</w:p>{anchor}{TABLE}",
                &property_source[start..end]
            );
            assert_body(&dom, body, &expected, &format!("{case}/{fold_boundary}"));
            let once = semantic(&dom, body);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", fold_boundary);
            assert_eq!(semantic(&dom, body), once);
        }
    }
}

#[test]
fn an_unresolved_field_residue_preserves_m360s_bare_heading_carrier() {
    for style in ["Heading1", "Title", "BodyText"] {
        let field = "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> REF AuthoredHeading </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>";
        let inserted_body =
            format!("<w:ins w:id='17' w:author='Boundary editor' w:date='{DATE}'>{field}</w:ins>");
        let old = marked(
            "del",
            "Original authored heading",
            &format!("<w:pStyle w:val='{style}'/><w:spacing w:after='240'/>"),
            "Boundary editor",
            DATE,
            "Boundary editor",
            DATE,
        );
        let new = format!(
            "<w:p><w:pPr><w:rPr><w:ins w:id='18' w:author='Boundary editor' w:date='{DATE}'/></w:rPr></w:pPr>{inserted_body}</w:p>"
        );
        let section = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
        let (mut dom, _, body) = package(&(new + &old + section));
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        let end = old.find("</w:pPr>").unwrap() + "</w:pPr>".len();
        let original_body = &old[end..old.len() - 6];
        let expected = if style == "BodyText" {
            format!(
                "<w:p>{}{inserted_body}{original_body}</w:p>{section}",
                &old[old.find("<w:pPr>").unwrap()..end]
            )
        } else {
            format!("<w:p>{inserted_body}{original_body}</w:p>{section}")
        };
        assert_body(&dom, body, &expected, style);
        let once = semantic(&dom, body);
        merge_replaced_in_container(&mut dom, body, "Boundary editor", true);
        assert_eq!(semantic(&dom, body), once);
    }
}

#[test]
fn legal_and_memo_midstory_block_ownership_is_retained_at_exact_boundaries() {
    for (label, prefix_count, deleted_count, inserted_count, nested, memo, retains) in [
        ("legal lower", 3, 5, 5, false, false, true),
        ("legal upper", 20, 5, 5, false, false, true),
        ("legal prefix below", 2, 5, 5, false, false, false),
        ("legal prefix above", 21, 5, 5, false, false, false),
        ("legal deleted below", 3, 4, 5, false, false, false),
        ("legal inserted below", 3, 5, 4, false, false, false),
        ("nested first", 1, 2, 2, true, false, true),
        ("nested without prior owner", 0, 2, 2, true, false, false),
        ("nested only one deleted", 1, 1, 2, true, false, false),
        ("nested only one inserted", 1, 2, 1, true, false, false),
        ("memo lower", 0, 3, 5, false, true, true),
        ("memo upper", 0, 20, 5, false, true, true),
        ("memo below", 0, 2, 5, false, true, false),
        ("memo above", 0, 21, 5, false, true, false),
        ("memo new below", 0, 3, 4, false, true, false),
        ("ordinary wholesale", 0, 3, 5, false, false, false),
    ] {
        for intervening_live in [false, true] {
            let prefix = (0..prefix_count)
                .map(|n| para("ins", &words(&format!("lead{n}"), 3), PROPS))
                .collect::<String>();
            let barrier = if intervening_live {
                para("live", "Original live anchor", PROPS)
            } else {
                String::new()
            };
            let dels = (0..deleted_count)
                .map(|n| {
                    let text = if memo && n == 0 {
                        "Memorandum Original obligations".to_string()
                    } else {
                        format!("Original{n} authored{n} clause{n}")
                    };
                    para(
                        "del",
                        &text,
                        if nested && n == 0 {
                            "<w:numPr><w:ilvl w:val='1'/><w:numId w:val='7'/></w:numPr>"
                        } else {
                            PROPS
                        },
                    )
                })
                .collect::<String>();
            let inss = (0..inserted_count)
                .map(|n| para("ins", &words(&format!("revised{n}"), 3), PROPS))
                .collect::<String>();
            let input = prefix.clone() + &barrier + &dels + &inss + TABLE;
            let (mut dom, _, body) = package(&input);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            // A live predecessor is a boundary, not a leading insertion.
            // Memo's authored headers are first only with no predecessor.
            let keeps = retains && !intervening_live;
            let expected = if keeps {
                input
            } else {
                prefix + &barrier + &inss + &dels + TABLE
            };
            assert_body(
                &dom,
                body,
                &expected,
                &format!("{label}/live={intervening_live}"),
            );
            let once = semantic(&dom, body);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            assert_eq!(semantic(&dom, body), once, "repeat {label}");
        }
    }
}

#[test]
fn a_carried_original_insertion_keeps_its_author_and_never_becomes_a_real_deletion() {
    for hyperlink in [false, true] {
        for with_bookmarks in [false, true] {
            for tail in ["live", "table"] {
                let old_run = run(
                    "ins",
                    "Original pending authored insertion",
                    "Legacy editor",
                    "2025-06-07T08:09:10Z",
                );
                let old_body = if hyperlink {
                    format!("<w:hyperlink w:anchor='OriginalBookmark'>{old_run}</w:hyperlink>")
                } else {
                    old_run
                };
                let bookmarks = if with_bookmarks {
                    (
                        "<w:bookmarkStart w:id='81' w:name='OriginalBookmark'/>",
                        "<w:bookmarkEnd w:id='81'/>",
                    )
                } else {
                    ("", "")
                };
                let old = format!(
                    "<w:p><w:pPr>{PROPS}</w:pPr>{}{old_body}{}</w:p>",
                    bookmarks.0, bookmarks.1
                );
                let new = marked(
                    "ins",
                    "Revised independently owned text",
                    PROPS,
                    "Boundary editor",
                    DATE,
                    "Boundary editor",
                    DATE,
                );
                let after = if tail == "table" {
                    TABLE.to_string()
                } else {
                    para("live", "Surviving final anchor", PROPS)
                };
                let input = old.clone() + &new + &after;
                let (mut dom, _, body) = package(&input);
                merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
                assert_body(
                    &dom,
                    body,
                    &(new + &old + &after),
                    &format!("hyperlink={hyperlink}/bookmarks={with_bookmarks}/{tail}"),
                );
                let once = semantic(&dom, body);
                merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
                assert_eq!(semantic(&dom, body), once);
            }
        }
    }
}

#[test]
fn pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch() {
    let fill = |colour: &str| {
        format!(
            "<w14:textFill><w14:solidFill><w14:srgbClr w14:val='{colour}'/></w14:solidFill></w14:textFill>"
        )
    };
    for (name, old, new, want) in [
        ("identical", fill("123456"), fill("123456"), true),
        ("changed fill", fill("123456"), fill("654321"), false),
        ("removed fill", fill("123456"), String::new(), false),
        ("added fill", String::new(), fill("123456"), false),
        (
            "reordered formats",
            "<w:b/><w:i/>".into(),
            "<w:i/><w:b/>".into(),
            true,
        ),
        (
            "changed scalar",
            "<w:color w:val='123456'/>".into(),
            "<w:color w:val='654321'/>".into(),
            false,
        ),
        (
            "history is source metadata",
            format!(
                "<w:b/><w:rPrChange w:id='19' w:author='Original editor' w:date='{DATE}'><w:rPr><w:i/></w:rPr></w:rPrChange>"
            ),
            "<w:b/>".into(),
            true,
        ),
        (
            "nested history remains historical",
            format!(
                "{}<w:rPrChange w:id='19' w:author='Original editor' w:date='{DATE}'><w:rPr>{}</w:rPr></w:rPrChange>",
                fill("123456"),
                fill("654321")
            ),
            fill("123456"),
            true,
        ),
    ] {
        let xml = format!(
            "<w:p><w:pPr><w:rPr pt:Unid='original'>{old}</w:rPr></w:pPr>{}</w:p><w:p><w:pPr><w:rPr pt:Unid='revised'>{new}</w:rPr></w:pPr>{}</w:p>{TABLE}",
            run(
                "live",
                "Original full source format",
                "Boundary editor",
                DATE
            ),
            run(
                "live",
                "Revised full source format",
                "Boundary editor",
                DATE
            )
        );
        let (mut dom, root, body) = package(&xml);
        let p = dom.elements(body, Some(&W::p()));
        let old = dom.element(dom.element(p[0], &W::p_pr()).unwrap(), &W::r_pr());
        let new = dom.element(dom.element(p[1], &W::p_pr()).unwrap(), &W::r_pr());
        let before = semantic(&dom, root);
        assert_eq!(
            paragraph_mark_properties_equal(&mut dom, old, new),
            want,
            "{name}"
        );
        assert_eq!(
            semantic(&dom, root),
            before,
            "format equality is source preserving"
        );
    }
}
