//! Word omits body `w:spacing line=276` that only restates pPrDefault.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

#[test]
fn default_line_276_spacing_stripped_from_body() {
    let mut dom = Dom::new();
    let base = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>Plain</w:t></w:r></w:p>"#;
    let next = r#"<w:p><w:pPr><w:spacing w:line="276"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Centered</w:t></w:r></w:p>"#;
    let (r1, b1) = doc_body(&mut dom, base);
    let (r2, b2) = doc_body(&mut dom, next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let ser = dom.serialize_element(out);
    assert!(
        !ser.contains(r#"w:line="276""#) && !ser.contains("line=\"276\""),
        "Word drops demo-default line=276 spacing: {ser}"
    );
    assert!(
        ser.contains("center") || ser.contains("Centered"),
        "center content retained: {ser}"
    );
}
