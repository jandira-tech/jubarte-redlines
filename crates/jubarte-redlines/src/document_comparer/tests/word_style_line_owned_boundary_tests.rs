// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
fn parse(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(xml);
    let root = dom.root(document).unwrap();
    (dom, root)
}
fn semantic(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes_iter(node)
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| (format!("{name:?}"), value))
        .collect::<Vec<_>>();
    attrs.sort();
    let children = dom
        .nodes(node)
        .into_iter()
        .map(|child| {
            let value = semantic(dom, child);
            format!("{}:{value}", value.len())
        })
        .collect::<String>();
    format!(
        "{:?}:{attrs:?}:{:?}:{children}",
        dom.name(node),
        dom.text_value(node)
    )
}
#[test]
fn word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties() {
    for line in [None, Some("240"), Some("276")] {
        for before in [None, Some("0"), Some("120")] {
            if line.is_none() && before.is_some() {
                continue;
            }
            let eligible = line == Some("240") && before.is_none_or(|before| before == "0");
            let styles = |normalized: bool| {
                let normal_spacing = match line {
                    Some(line) => format!(
                        "<w:spacing {}w:after='0' w:line='{line}' w:lineRule='auto'/>",
                        before
                            .map(|v| format!("w:before='{v}' "))
                            .unwrap_or_default()
                    ),
                    None => String::new(),
                };
                let normal_ppr = if line.is_some() {
                    format!("<w:pPr>{normal_spacing}</w:pPr>")
                } else {
                    String::new()
                };
                let line_attrs = if normalized {
                    " w:line='240' w:lineRule='auto'"
                } else {
                    ""
                };
                let generated_spacing = if normalized {
                    "<w:spacing w:after='0' w:line='240' w:lineRule='auto'/>"
                } else {
                    ""
                };
                let title_ppr = if normalized {
                    format!("<w:pPr>{generated_spacing}</w:pPr>")
                } else {
                    String::new()
                };
                format!(
                    "<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/>{normal_ppr}<w:rPr><w:rFonts w:ascii='Calibri'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Heading1'><w:name w:val='heading 1'/><w:basedOn w:val='Normal'/><w:pPr><w:spacing w:before='180' w:after='80'{line_attrs}/><w:ind w:left='120'/></w:pPr><w:rPr><w:b/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Title'><w:name w:val='Title'/>{title_ppr}<w:rPr><w:i/><w:color w:val='123456'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='ListParagraph'><w:name w:val='List Paragraph'/><w:basedOn w:val='Normal'/><w:pPr>{generated_spacing}<w:ind w:left='720'/><w:contextualSpacing/><w:pPrChange w:id='71' w:author='Original layout owner' w:date='2026-10-01T12:00:00Z'><w:pPr><w:spacing w:before='120' w:after='180' w:line='276' w:lineRule='auto'/><w:ind w:left='360'/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:lang w:val='en-US'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Heading2'><w:name w:val='heading 2'/><w:pPr><w:ind w:left='240'/></w:pPr></w:style><w:style w:type='paragraph' w:styleId='Heading3'><w:name w:val='heading 3'/><w:pPr><w:spacing w:after='80' w:line='276' w:lineRule='exact'/></w:pPr></w:style><w:style w:styleId='Heading4'><w:name w:val='heading 4'/><w:pPr><w:spacing w:after='80'{line_attrs}/></w:pPr></w:style><w:style w:type='paragraph' w:styleId='Heading5'><w:name w:val='heading 5'/><w:rPr><w:b/></w:rPr></w:style><w:style w:type='character' w:styleId='HighlightedStyle'><w:name w:val='Highlighted Style'/><w:rPr><w:highlight w:val='yellow'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Unrelated'><w:name w:val='Unrelated'/><w:pPr><w:spacing w:line='360' w:lineRule='exact'/></w:pPr></w:style></w:styles>",
                    W::URI
                )
            };
            let (mut dom, root) = parse(&styles(false));
            let history = dom.descendants(root, Some(&W::p_pr_change()))[0];
            let authored_history = semantic(&dom, history);
            let (oracle, expected) = parse(&styles(eligible));
            assert_eq!(
                normalize_word_paragraph_style_line(&mut dom, root),
                eligible,
                "line={line:?} before={before:?}"
            );
            assert_eq!(
                semantic(&dom, root),
                semantic(&oracle, expected),
                "line={line:?} before={before:?}"
            );
            assert_eq!(dom.descendants(root, Some(&W::p_pr_change())), [history]);
            assert_eq!(semantic(&dom, history), authored_history);
            assert!(!normalize_word_paragraph_style_line(&mut dom, root));
            assert_eq!(semantic(&dom, root), semantic(&oracle, expected));
        }
    }
}
