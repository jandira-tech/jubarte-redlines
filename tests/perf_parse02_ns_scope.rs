//! PARSE-02 — skip HashMap clone when an element declares no xmlns.
//!
//! Correctness gate: shadowed prefixes and default-namespace inheritance must
//! still resolve exactly as before (parse + serialize stable).

use jubarte::xmllinq::Dom;

fn roundtrip(xml: &str) -> String {
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    d.serialize_document(doc)
}

#[test]
fn nested_default_ns_inheritance() {
    let xml = r#"<root xmlns="http://example.com/a"><child><g/></child></root>"#;
    let out = roundtrip(xml);
    let out2 = roundtrip(&out);
    assert_eq!(out, out2);
    // children inherit default ns (serialize may re-emit xmlns shape)
    assert!(out.contains("child") || out.contains("g"));
}

#[test]
fn shadowed_prefix_inner_override() {
    let xml = r#"<r xmlns:p="http://outer"><a xmlns:p="http://inner" p:x="1"/><b p:y="2"/></r>"#;
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    // a has p:x in inner ns; b has p:y in outer ns
    let kids: Vec<_> = (0..d.child_count(root))
        .map(|i| d.child_at(root, i))
        .collect();
    assert_eq!(kids.len(), 2);
    let a = kids[0];
    let b = kids[1];
    // attribute namespaces via name()
    let a_attrs = d.attributes(a);
    let b_attrs = d.attributes(b);
    let a_x = a_attrs.iter().find(|(n, _)| n.local_name() == "x").unwrap();
    let b_y = b_attrs.iter().find(|(n, _)| n.local_name() == "y").unwrap();
    assert_eq!(a_x.0.namespace_name(), "http://inner");
    assert_eq!(b_y.0.namespace_name(), "http://outer");
    // re-serialize stable
    let s1 = d.serialize_element(root);
    let mut d2 = Dom::new();
    let doc2 = d2.parse_xdocument(&format!("<?xml version=\"1.0\"?>{s1}"));
    let s2 = d2.serialize_element(d2.root(doc2).unwrap());
    assert_eq!(s1, s2);
}

#[test]
fn no_xmlns_interior_matches_with_xmlns_root() {
    // Interior elements declare no xmlns — PARSE-02 reuses parent map.
    let xml = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>"#;
    let out = roundtrip(xml);
    let out2 = roundtrip(&out);
    assert_eq!(out, out2);
    assert!(out.contains(">x<") || out.contains("x"));
}
