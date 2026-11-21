// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn paragraph(kind: &str, text: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:p><w:pPr><w:spacing w:after=\"60\"/><w:rPr><w:{kind} w:id=\"1\"/></w:rPr></w:pPr><w:{kind} w:id=\"2\"><w:r><w:rPr><w:b/></w:rPr><w:{leaf}>{text}</w:{leaf}></w:r></w:{kind}></w:p>"
    )
}

fn parse(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w=\"{}\"><w:body>{fragment}<w:sectPr/></w:body></w:document>",
        W::URI
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

// Namespace declarations can move when independently cloned revision
// projections are serialized. Preserve every semantic QName, attribute,
// text node and child position instead of comparing xmlns placement.
fn canonical(dom: &Dom, node: NodeId) -> String {
    let mut attrs = dom
        .attributes_iter(node)
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
        .map(|n| canonical(dom, n))
        .collect::<Vec<_>>();
    format!(
        "{:?}:{attrs:?}:{:?}:{children:?}",
        dom.name(node),
        dom.text_value(node)
    )
}

fn resolved(fragment: &str, accepting: bool) -> String {
    let (mut dom, root) = parse(fragment);
    let root = if accepting {
        crate::revision_processor::accept_revisions_document(&mut dom, root)
    } else {
        crate::revision_processor::reject_revisions_document(&mut dom, root)
    };
    canonical(&dom, root)
}

#[test]
fn adjacent_inserted_prefix_preserves_table_midsplice_and_both_complete_projections() {
    for nested in [false, true] {
        let prefix = paragraph("ins", "Lease opening");
        let deleted = paragraph("del", "Employment original");
        let payload = paragraph("ins", "Lease residual");
        let table = format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"2000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:trPr><w:ins w:id=\"3\"/></w:trPr><w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{payload}</w:tc></w:tr></w:tbl>"
        );
        let residual = if nested {
            format!(
                "<w:sdt><w:sdtPr><w:tag w:val=\"residual\"/></w:sdtPr><w:sdtContent>{table}</w:sdtContent></w:sdt>"
            )
        } else {
            table
        };
        let original = format!("{prefix}{deleted}{residual}{}", paragraph("ins", "Closing"));
        let (mut dom, root) = parse(&original);
        let before = dom.serialize_element(root);
        reorder_replaced_blocks(&mut dom, root);
        assert_eq!(dom.serialize_element(root), before, "nested={nested}");
        let body = dom.element(root, &W::body()).unwrap();
        let after = dom
            .elements(body, None)
            .into_iter()
            .filter(|&n| !dom.name_is(n, &W::sect_pr()))
            .map(|n| dom.serialize_element(n))
            .collect::<String>();
        for accepting in [false, true] {
            assert_eq!(
                resolved(&after, accepting),
                resolved(&original, accepting),
                "nested={nested}, accepting={accepting}"
            );
        }
    }
}

#[test]
fn live_boundary_prevents_an_earlier_insertion_from_claiming_the_splice() {
    let prefix = paragraph("ins", "Earlier insertion");
    let live = "<w:p><w:r><w:t>Unchanged boundary</w:t><w:tab/></w:r></w:p>";
    let deleted = paragraph("del", "Original");
    let inserted = format!(
        "<w:tbl><w:tr><w:trPr><w:ins/></w:trPr><w:tc>{}</w:tc></w:tr></w:tbl>",
        paragraph("ins", "Revised")
    );
    let (mut dom, root) = parse(&format!("{prefix}{live}{deleted}{inserted}"));
    reorder_replaced_blocks(&mut dom, root);
    let (expected, expected_root) = parse(&format!("{prefix}{live}{inserted}{deleted}"));
    assert_eq!(
        dom.serialize_element(root),
        expected.serialize_element(expected_root)
    );
}

#[test]
fn inserted_text_on_a_deleted_mark_stays_between_the_deleted_tables() {
    let table = |text| {
        format!(
            "<w:tbl><w:tr><w:trPr><w:del/></w:trPr><w:tc>{}</w:tc></w:tr></w:tbl>",
            paragraph("del", text)
        )
    };
    // The original's paragraph between its tables (#1826): its mark is
    // deleted and the revision's text is inserted on it.
    let mixed = "<w:p><w:pPr><w:rPr><w:del w:id=\"5\"/></w:rPr></w:pPr><w:ins w:id=\"6\"><w:r><w:t>Revised</w:t></w:r></w:ins></w:p>";
    let (mut dom, root) = parse(&format!("{}{mixed}{}", table("First"), table("Second")));
    let before = dom.serialize_element(root);
    reorder_replaced_blocks(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before);
    // The same text on its own inserted mark still goes ahead of the
    // deleted table.
    let inserted = paragraph("ins", "Revised");
    let (mut dom, root) = parse(&format!("{}{inserted}", table("First")));
    reorder_replaced_blocks(&mut dom, root);
    let (expected, expected_root) = parse(&format!("{inserted}{}", table("First")));
    assert_eq!(
        dom.serialize_element(root),
        expected.serialize_element(expected_root)
    );
    // One deleted table: the same inserted text on a deleted mark
    // takes its inserted class and moves ahead of the table (#1926).
    let (mut dom, root) = parse(&format!("{}{mixed}", table("Only")));
    reorder_replaced_blocks(&mut dom, root);
    let (expected, expected_root) = parse(&format!("{mixed}{}", table("Only")));
    assert_eq!(
        dom.serialize_element(root),
        expected.serialize_element(expected_root)
    );
}
