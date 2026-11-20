// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn package(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}'><w:body>{body}</w:body></w:document>",
        W::URI
    ));
    let root = dom.root(document).expect("document");
    (dom, root)
}
const DATE: &str = "2026-01-02T03:04:05Z";
const PROPS: &str =
    "<w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:after='80'/><w:rPr><w:i/></w:rPr></w:pPr>";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/><w:rPrChange w:id='42' w:author='Earlier format owner' w:date='2025-01-01T00:00:00Z'><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr>";
const TAIL: &str = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Independent live cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
fn revision(kind: &str, id: u32, body: &str) -> String {
    format!(
        "<w:{kind} w:id='{id}' w:author='Source {kind} owner' w:date='{DATE}'>{body}</w:{kind}>"
    )
}
fn field(deleted: bool) -> String {
    let instruction = if deleted { "delInstrText" } else { "instrText" };
    let text = if deleted { "delText" } else { "t" };
    format!(
        "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r>{FORMAT}<w:{instruction} xml:space='preserve'> PAGE </w:{instruction}></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r>{FORMAT}<w:{text}>7</w:{text}></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>"
    )
}
#[test]
fn outer_deletion_preserves_independently_owned_nested_revision_text_and_fields() {
    for kind in ["ins", "moveFrom", "moveTo"] {
        for nested_first in [false, true] {
            for with_field in [false, true] {
                for with_xml_metadata in [false, true] {
                    let plain =
                        format!("<w:r>{FORMAT}<w:t xml:space='preserve'>  原é </w:t></w:r>");
                    let deleted = format!(
                        "<w:r>{FORMAT}<w:delText xml:space='preserve'>  原é </w:delText></w:r>"
                    );
                    let nested_body = format!(
                        "<w:r>{FORMAT}<w:t xml:space='preserve'> Nested source </w:t></w:r>{}",
                        if with_field {
                            field(false)
                        } else {
                            String::new()
                        }
                    );
                    let nested_revision = revision(kind, 31, &nested_body);
                    let nested = if kind == "ins" {
                        nested_revision
                    } else {
                        format!(
                            "<w:{kind}RangeStart w:id='41' w:name='OwnedMove' w:author='Move range owner' w:date='{DATE}'/>{nested_revision}<w:{kind}RangeEnd w:id='41'/>"
                        )
                    };
                    let metadata = if with_xml_metadata {
                        "<!--Authored source comment--><?owned source?>"
                    } else {
                        ""
                    };
                    let own_field = if with_field {
                        field(false)
                    } else {
                        String::new()
                    };
                    let renamed_field = if with_field {
                        field(true)
                    } else {
                        String::new()
                    };
                    let body = if nested_first {
                        format!("{metadata}{nested}{plain}{own_field}")
                    } else {
                        format!("{plain}{own_field}{metadata}{nested}")
                    };
                    let expected_body = if nested_first {
                        format!("{metadata}{nested}{deleted}{renamed_field}")
                    } else {
                        format!("{deleted}{renamed_field}{metadata}{nested}")
                    };
                    let input = format!("<w:p>{PROPS}{}</w:p>{TAIL}", revision("del", 30, &body));
                    let expected = format!(
                        "<w:p>{PROPS}{}</w:p>{TAIL}",
                        revision("del", 30, &expected_body)
                    );
                    let (mut dom, root) = package(&input);
                    let (expected_dom, expected_root) = package(&expected);
                    let nodes = dom.node_count();
                    enforce_deleted_text_kinds(&mut dom, root);
                    let coordinate = format!(
                        "nested={kind}/first={nested_first}/field={with_field}/xml-metadata={with_xml_metadata}"
                    );
                    assert_eq!(
                        dom.serialize_element(root),
                        expected_dom.serialize_element(expected_root),
                        "{coordinate}: complete source ownership"
                    );
                    assert_eq!(dom.node_count(), nodes, "{coordinate}: renaming only");
                    let once = dom.serialize_element(root);
                    enforce_deleted_text_kinds(&mut dom, root);
                    assert_eq!(dom.serialize_element(root), once, "{coordinate}: repeat");
                    assert_eq!(dom.node_count(), nodes);
                }
            }
        }
    }
}
