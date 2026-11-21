// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn document(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:document xmlns:w="{}" xmlns:m="{}"><w:body>{body}<w:sectPr/></w:body></w:document>"#,
        W::URI,
        M::URI
    ));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

#[test]
fn unchanged_nontext_payload_prevents_a_moved_neighbor_from_owning_its_row() {
    for payload in [
        "<w:r><w:tab/></w:r>",
        "<w:r><w:br/></w:r>",
        "<w:r><w:cr/></w:r>",
        "<w:r><w:noBreakHyphen/></w:r>",
        "<w:r><w:footnoteReference w:id=\"1\"/></w:r>",
        "<w:r><w:fldChar w:fldCharType=\"begin\"/><w:instrText>DATE</w:instrText><w:fldChar w:fldCharType=\"end\"/></w:r>",
        "<w:fldSimple w:instr=\"DATE\"/>",
        "<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath>",
    ] {
        for (direction, mark) in [("moveTo", "ins"), ("moveFrom", "del")] {
            let (mut dom, root) = document(&format!(
                "<w:tbl><w:tr><w:tc><w:p><w:pPr><w:rPr><w:{mark} w:id=\"1\"/></w:rPr></w:pPr><w:{direction} w:id=\"2\"><w:r><w:t>Moved</w:t></w:r></w:{direction}></w:p></w:tc><w:tc><w:p>{payload}</w:p></w:tc></w:tr></w:tbl>"
            ));
            let expected = dom.serialize_element(dom.descendants(root, Some(&W::tc()))[1]);
            mark_fully_revised_rows(&mut dom, root, &WmlComparerSettings::default(), &mut 10);
            assert!(
                dom.descendants(root, Some(&W::tr_pr())).is_empty(),
                "{direction} {payload}"
            );
            for accept in [false, true] {
                let copy = dom.clone_subtree(root);
                let projected = if accept {
                    crate::revision_processor::accept_revisions_document(&mut dom, copy)
                } else {
                    crate::revision_processor::reject_revisions_document(&mut dom, copy)
                };
                assert_eq!(dom.descendants(projected, Some(&W::tr())).len(), 1);
                let cells = dom.descendants(projected, Some(&W::tc()));
                assert_eq!(cells.len(), 2);
                assert_eq!(
                    dom.serialize_element(cells[1]),
                    expected,
                    "{direction} {payload} accept={accept}"
                );
            }
        }
    }
}

fn package(body: &str) -> Vec<u8> {
    let mut pkg = crate::opc::PartFs::open(include_bytes!(
        "../../../../../../tests/fixtures/relids/image_doc.docx"
    ))
    .unwrap();
    pkg.set_part(
        "word/document.xml",
        format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:p/><w:sectPr/></w:body></w:document>"#,
            W::URI
        )
        .into_bytes(),
    );
    pkg.to_zip().unwrap()
}

fn row_payload(bytes: &[u8], name: &str) -> (usize, usize, usize) {
    let pkg = crate::opc::PartFs::open(bytes).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    (
        dom.descendants(root, Some(&W::tr())).len(),
        dom.descendants(root, Some(&W::tc())).len(),
        dom.descendants(root, Some(&W::name(name))).len(),
    )
}

#[test]
fn public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row() {
    let text = "The complete signed report shall be delivered to the other party within thirty days after the written request.";
    for name in ["tab", "br", "cr", "noBreakHyphen"] {
        let table = format!(
            "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc><w:tc><w:p><w:r><w:{name}/></w:r></w:p></w:tc></w:tr></w:tbl>"
        );
        let a = package(&format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>{table}"));
        let populated = table.replacen(
            "<w:tc><w:p/></w:tc>",
            &format!("<w:tc><w:p/><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"),
            1,
        );
        let b = package(&format!("<w:p/>{populated}"));
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            for reverse in [false, true] {
                let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                let compared =
                    crate::document_comparer::compare_documents_with_settings(a, b, &settings)
                        .unwrap();
                for (projected, source) in [
                    (
                        crate::document_comparer::accept_revisions(&compared).unwrap(),
                        b,
                    ),
                    (
                        crate::document_comparer::reject_revisions(&compared).unwrap(),
                        a,
                    ),
                ] {
                    assert_eq!(
                        row_payload(&projected, name),
                        row_payload(source, name),
                        "{name} reverse={reverse}"
                    );
                    assert_eq!(row_payload(&projected, name), (1, 2, 1));
                    let pkg = crate::opc::PartFs::open(&projected).unwrap();
                    let mut projected_dom = Dom::new();
                    let doc = projected_dom
                        .parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                    let root = projected_dom.root(doc).unwrap();
                    let recovered: String = projected_dom
                        .descendants(root, Some(&W::t()))
                        .into_iter()
                        .map(|t| projected_dom.value(t))
                        .collect();
                    assert_eq!(
                        recovered, text,
                        "moved text must survive each source projection"
                    );
                }
            }
        }
    }
}
