// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

mod common;
use common::docx::{docx, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::{M, W};
use jubarte::xmllinq::{Dom, NodeId};

fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes(node)
        .into_iter()
        .filter(|(name, _)| !dom.is_namespace_declaration(name))
        .map(|(name, value)| {
            (
                name.namespace_name().to_string(),
                name.local_name().to_string(),
                value,
            )
        })
        .collect::<Vec<_>>();
    attrs.sort();
    let children = dom
        .nodes(node)
        .into_iter()
        .filter(|&n| {
            !(dom.name_is(n, &W::r_pr()) && dom.attributes(n).is_empty() && dom.nodes(n).is_empty())
        })
        .map(|n| canonical(dom, n))
        .collect::<Vec<_>>();
    format!(
        "{:?}:{attrs:?}:{:?}:{children:?}",
        dom.name(node),
        dom.text_value(node)
    )
}

fn source_view(bytes: &[u8], faithful: bool) -> Vec<String> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let paras = dom.descendants(root, Some(&W::p()));
    let mut output = Vec::new();
    for p in paras {
        if faithful {
            let props = dom
                .element(p, &W::p_pr())
                .map(|n| canonical(&dom, n))
                .unwrap_or_default();
            output.push(format!("paragraph:{props}"));
        }
        for node in dom.descendants(p, None) {
            if dom.name_is(node, &W::t()) {
                let format = if faithful {
                    let run = dom
                        .ancestors(node, None)
                        .into_iter()
                        .find(|&n| dom.name_is(n, &W::r()))
                        .unwrap();
                    dom.element(run, &W::r_pr())
                        .map(|n| canonical(&dom, n))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                output.extend(
                    dom.value(node)
                        .chars()
                        .map(|ch| format!("text:{ch}:{format}")),
                );
            } else if dom.name_is(node, &M::name("oMath"))
                || dom.name_is(node, &W::name("tab"))
                || dom.name_is(node, &W::name("br"))
            {
                output.push(canonical(&dom, node));
            }
        }
    }
    output
}

#[test]
fn interior_math_and_layout_gaps_preserve_original_and_revised_source_order() {
    let paragraph = |text: &str| {
        format!(
            "<w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>{text}</w:t></w:r></w:p>"
        )
    };
    // Internal math revision marks materialize Word's implicit math font.
    // Supply that font explicitly so this strict source oracle measures
    // payload ownership, formatting and order without a default-font rewrite.
    let math = format!(
        "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><m:oMath xmlns:m='{}'><m:r><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/></w:rPr><m:t>x+y</m:t></m:r></m:oMath></w:p>",
        M::URI
    );
    let gap = "<w:p><w:pPr><w:spacing w:after='240'/></w:pPr></w:p>";
    let a = docx(&format!(
        "{}{math}{}{gap}{}",
        paragraph("Shared alpha title"),
        paragraph("Shared original middle"),
        paragraph("Shared original end")
    ));
    let b = docx(&format!(
        "{gap}{}{}{}{}{}",
        paragraph("Shared beta title"),
        paragraph("Shared revised middle"),
        paragraph("Shared revised third"),
        paragraph("New revised fourth"),
        paragraph("New revised end")
    ));
    for word in [false, true] {
        let out = compare_documents_with_settings(
            &a,
            &b,
            &WmlComparerSettings {
                merge_replaced_paragraphs: word,
                ..WmlComparerSettings::default()
            },
        )
        .unwrap();
        assert_eq!(
            source_view(&reject_revisions(&out).unwrap(), !word),
            source_view(&a, !word),
            "original: Word={word}"
        );
        assert_eq!(
            source_view(&accept_revisions(&out).unwrap(), !word),
            source_view(&b, !word),
            "revised: Word={word}"
        );
    }
}
