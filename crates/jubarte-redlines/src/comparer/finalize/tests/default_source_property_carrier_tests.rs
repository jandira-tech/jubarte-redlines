// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:lang w:val='en-US'/></w:rPr>";
const TABLE: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent table payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

fn package(body: &str) -> (Dom, NodeId, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}' xmlns:pt='{}'><w:body>{body}</w:body></w:document>",
        W::URI,
        PT::URI
    ));
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
fn assert_body(dom: &Dom, body: NodeId, expected: &str, label: &str) {
    let (want, _, wb) = package(expected);
    assert_eq!(semantic(dom, body), semantic(&want, wb), "{label}");
}
fn properties(kind: &str, lifetime: &str) -> String {
    match kind {
        "absent" => String::new(),
        "empty" => "<w:pPr/>".into(),
        "format" => "<w:pPr><w:rPr><w:b/><w:color w:val='123456'/></w:rPr></w:pPr>".into(),
        "spacing" => "<w:pPr><w:spacing w:before='120' w:after='240'/></w:pPr>".into(),
        "heading" => format!(
            "<w:pPr><w:pStyle w:val='Heading1'/><w:spacing w:after='240'/><w:rPr><w:{lifetime} w:id='18' w:author='Boundary editor' w:date='{DATE}'/></w:rPr></w:pPr>"
        ),
        "ins-mark" | "del-mark" => format!(
            "<w:pPr><w:rPr><w:{} w:id='18' w:author='Boundary editor' w:date='{DATE}'/></w:rPr></w:pPr>",
            if kind == "ins-mark" { "ins" } else { "del" }
        ),
        _ => panic!("unknown authored property fixture"),
    }
}

// Clean Word paragraphs may omit pPr entirely. A source's own rPr-only
// closing carrier likewise has no structural paragraph property. These
// are valid inputs, not simulated missing DOM children after detachment.
#[test]
fn related_one_to_one_pairs_keep_the_exact_default_or_structural_source_carrier() {
    for old_kind in ["absent", "empty", "format", "spacing", "heading"] {
        for new_kind in ["absent", "empty", "format", "spacing", "ins-mark"] {
            let old_props = properties(old_kind, "del");
            let new_props = properties(new_kind, "ins");
            let original_body = run(
                "del",
                "Related original source words",
                "Boundary editor",
                DATE,
            );
            let revised_body = run(
                "ins",
                "Related revised source words",
                "Boundary editor",
                DATE,
            );
            let old = format!("<w:p>{old_props}{original_body}</w:p>");
            let new = format!("<w:p>{new_props}{revised_body}</w:p>");
            let (mut dom, _, body) = package(&(old + &new + TABLE));
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            // M88's independent carrier cases: revised structural spacing
            // owns the mixed p; otherwise old structural spacing/heading
            // owns it; otherwise the revised default carrier is retained.
            let expected_properties = match (old_kind, new_kind) {
                (_, "spacing") => new_props,
                ("spacing" | "heading", _) => old_props,
                _ => new_props,
            };
            let expected =
                format!("<w:p>{expected_properties}{revised_body}{original_body}</w:p>{TABLE}");
            assert_body(
                &dom,
                body,
                &expected,
                &format!("default D/I {old_kind}/{new_kind}"),
            );
            let once = semantic(&dom, body);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            assert_eq!(semantic(&dom, body), once);
        }
    }
}

// Single-paragraph Word GT closes a wholesale replacement in the last
// revised paragraph. Nonheading originals leave that closing mark bare;
// M435 retains an authored old heading and its deleted pilcrow instead.
#[test]
fn a_sole_original_closes_multi_inserted_default_paragraphs_without_inventing_properties() {
    for original_kind in ["absent", "empty", "spacing", "heading"] {
        for closing_kind in [
            "absent", "empty", "format", "spacing", "ins-mark", "del-mark",
        ] {
            let old_props = properties(original_kind, "del");
            let closing_props = properties(closing_kind, "ins");
            let revised_open = run(
                "ins",
                "Independently revised opening paragraph",
                "Boundary editor",
                DATE,
            );
            let revised_close = run(
                "ins",
                "Independently revised final paragraph",
                "Boundary editor",
                DATE,
            );
            let original_body = run(
                "del",
                "Original independently authored complete paragraph",
                "Boundary editor",
                DATE,
            );
            let lead = format!("<w:p>{}{revised_open}</w:p>", properties("ins-mark", "ins"));
            let new = format!("<w:p>{closing_props}{revised_close}</w:p>");
            let old = format!("<w:p>{old_props}{original_body}</w:p>");
            let section = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
            let (mut dom, _, body) = package(&(old + &lead + &new + section));
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            let expected_properties = if original_kind == "heading" {
                old_props
            } else {
                match closing_kind {
                    "absent" | "empty" | "ins-mark" | "del-mark" => String::new(),
                    _ => closing_props,
                }
            };
            let expected = format!(
                "{lead}<w:p>{expected_properties}{revised_close}{original_body}</w:p>{section}"
            );
            assert_body(
                &dom,
                body,
                &expected,
                &format!("sole original {original_kind}/{closing_kind}"),
            );
            let once = semantic(&dom, body);
            merge_replaced_in_container(&mut dom, body, "Boundary editor", false);
            assert_eq!(semantic(&dom, body), once);
        }
    }
}
