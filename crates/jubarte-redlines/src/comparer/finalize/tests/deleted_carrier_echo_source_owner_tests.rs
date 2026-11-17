// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "1970-01-01T00:00:00Z";
const PROPS: &str = "<w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/>";
const RUN: &str = "<w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr>";

fn parse(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI));
    let root = dom.root(doc).unwrap();
    (dom, root)
}
fn paragraph(text: &str) -> String {
    format!(
        "<w:p><w:pPr>{PROPS}</w:pPr><w:r>{RUN}<w:t xml:space='preserve'>{text}</w:t></w:r></w:p>"
    )
}
fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .collect::<Vec<_>>();
    attrs.sort_by_key(|(name, value)| {
        (
            name.namespace_name().to_owned(),
            name.local_name().to_owned(),
            *value,
        )
    });
    if attrs.is_empty()
        && dom.nodes(node).is_empty()
        && [W::p_pr(), W::r_pr()]
            .iter()
            .any(|name| dom.name_is(node, name))
    {
        return String::new();
    }
    let mut result = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
    for child in dom.nodes(node) {
        let part = canonical(dom, child);
        if !part.is_empty() {
            result.push_str(&format!("{}:{part}", part.len()));
        }
    }
    result
}
fn events(dom: &Dom, root: NodeId) -> Vec<String> {
    let body = dom.element(root, &W::body()).unwrap();
    let mut events = Vec::new();
    for child in dom.elements(body, None) {
        if !dom.name_is(child, &W::p()) {
            events.push(canonical(dom, child));
            continue;
        }
        events.push("begin paragraph".into());
        if let Some(ppr) = dom.element(child, &W::p_pr()) {
            events.push(canonical(dom, ppr));
        }
        for run in dom.elements(child, Some(&W::r())) {
            let props = dom
                .element(run, &W::r_pr())
                .map(|p| canonical(dom, p))
                .unwrap_or_default();
            for payload in dom.elements(run, None) {
                if dom.name_is(payload, &W::r_pr()) {
                    continue;
                }
                if dom.name_is(payload, &W::t()) {
                    events.extend(
                        dom.value(payload)
                            .chars()
                            .map(|ch| format!("text:{ch}:{props}")),
                    );
                } else {
                    events.push(format!("payload:{}:{props}", canonical(dom, payload)));
                }
            }
        }
        assert!(
            dom.elements(child, None)
                .iter()
                .all(|&node| dom.name_is(node, &W::p_pr()) || dom.name_is(node, &W::r()))
        );
        events.push("end paragraph".into());
    }
    events
}
#[test]
fn deleted_original_carrier_keeps_echo_prefix_and_both_complete_source_projections() {
    for (inserted, deleted) in [
        ("alpha", "alpha-"),
        ("alpha", "alpha:"),
        ("Alpha", "Alpha "),
    ] {
        let body = format!(
            "<w:p><w:pPr>{PROPS}<w:rPr><w:ins w:id='1' w:author='Open-Xml-PowerTools' w:date='{DATE}'/></w:rPr></w:pPr><w:ins w:id='2' w:author='Open-Xml-PowerTools' w:date='{DATE}'><w:r>{RUN}<w:t>{inserted}</w:t></w:r></w:ins></w:p><w:p><w:pPr>{PROPS}<w:rPr><w:del w:id='3' w:author='Open-Xml-PowerTools' w:date='{DATE}'/></w:rPr></w:pPr><w:del w:id='4' w:author='Open-Xml-PowerTools' w:date='{DATE}'><w:r>{RUN}<w:delText xml:space='preserve'>{deleted}</w:delText></w:r></w:del><w:r>{RUN}<w:t>copper walnut</w:t></w:r><w:ins w:id='5' w:author='Open-Xml-PowerTools' w:date='{DATE}'><w:r>{RUN}<w:t xml:space='preserve'> </w:t></w:r></w:ins></w:p>{}",
            paragraph("violet glacier closing")
        );
        let (mut dom, root) = parse(&body);
        let before = canonical(&dom, root);
        strip_leading_del_echoing_prev_pure_i(&mut dom, root);
        assert_eq!(
            canonical(&dom, root),
            before,
            "original deleted carrier owns prefix {deleted:?}"
        );
        for accept in [false, true] {
            let copy = dom.clone_subtree(root);
            let projected = if accept {
                crate::revision_processor::accept_revisions_document(&mut dom, copy)
            } else {
                crate::revision_processor::reject_revisions_document(&mut dom, copy)
            };
            let expected = if accept {
                format!(
                    "{}{}",
                    paragraph(inserted),
                    paragraph("copper walnut violet glacier closing")
                )
            } else {
                format!(
                    "{}{}",
                    paragraph(&format!("{deleted}copper walnut")),
                    paragraph("violet glacier closing")
                )
            };
            let (expected_dom, expected_root) = parse(&expected);
            assert_eq!(
                events(&dom, projected),
                events(&expected_dom, expected_root),
                "complete source accept={accept} prefix={deleted:?}"
            );
        }
    }
}
