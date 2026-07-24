//! SER-01 — serializer writes tags/attrs/escapes directly into the final
//! buffer (no intermediate `attr_str` / `qname` / always-alloc `escape_*`).
//!
//! Gate: byte-identical XML vs the pre-SER-01 golden matrix (namespaces,
//! empty elements, QName-list attrs, entity escaping, mixed content).

use jubarte::xmllinq::Dom;

fn parse_and_reserialize(xml: &str) -> String {
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    d.serialize_document(doc)
}

/// Exact golden captured from pre-SER-01 serializer (MEASURED baseline).
#[test]
fn ser01_exact_simple_w_document() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>"#;
    let want = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_empty_element_and_attrs() {
    let xml = r#"<root a="1" b="two"><empty/></root>"#;
    let want = r#"<root a="1" b="two"><empty /></root>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_attr_and_text_escaping() {
    // Input entities are decoded by the parser; serializer re-escapes.
    let xml = r#"<e a="a&amp;b&lt;c&gt;d&quot;e"><t>x&amp;y&lt;z&gt;</t></e>"#;
    let want = r#"<e a="a&amp;b&lt;c&gt;d&quot;e"><t>x&amp;y&lt;z&gt;</t></e>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_mc_ignorable_qname_list() {
    let xml = r#"<root xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" mc:Ignorable="w14"><child w14:paraId="12345678"/></root>"#;
    let want = r#"<root xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" mc:Ignorable="w14"><child w14:paraId="12345678" /></root>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_nested_namespaces() {
    let xml = r#"<a:r xmlns:a="http://a" xmlns:b="http://b"><b:x b:y="1"/><a:z/></a:r>"#;
    let want = r#"<a:r xmlns:a="http://a" xmlns:b="http://b"><b:x b:y="1" /><a:z /></a:r>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_mixed_content_comment_pi() {
    let xml = r#"<p>hello<!--c--><?pi data?><c/>world</p>"#;
    let want = r#"<p>hello<!--c--><?pi data?><c />world</p>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_exact_w_attrs_and_xml_space() {
    let xml = r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:rsidR="00AB" w:rsidRDefault="00CD"><w:r><w:t xml:space="preserve"> hi </w:t></w:r></w:p>"#;
    let want = r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:rsidR="00AB" w:rsidRDefault="00CD"><w:r><w:t xml:space="preserve"> hi </w:t></w:r></w:p>"#;
    assert_eq!(parse_and_reserialize(xml), want);
}

#[test]
fn ser01_idempotent_reserialize() {
    let cases = [
        r#"<?xml version="1.0"?><root xmlns:x="urn:x" x:a="1"><x:c/></root>"#,
        r#"<e a="&quot;quoted&quot; &amp; more">t&lt;ag&gt;</e>"#,
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p><w:r><w:t/></w:r></w:p></w:body></w:document>"#,
    ];
    for xml in cases {
        let once = parse_and_reserialize(xml);
        let twice = parse_and_reserialize(&once);
        assert_eq!(once, twice, "idempotent fail for {xml}");
    }
}

/// Real package document.xml path: parse a body fragment twice and require
/// byte-identical serialize_element on the root element (not only document).
#[test]
fn ser01_exact_element_subtree_path() {
    let xml = r#"<w:body xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p w:rsidR="0011"><w:r><w:t>A&amp;B</w:t></w:r></w:p></w:body>"#;
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).expect("root");
    let once = d.serialize_element(root);
    let want = r#"<w:body xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p w:rsidR="0011"><w:r><w:t>A&amp;B</w:t></w:r></w:p></w:body>"#;
    assert_eq!(once, want);
    let mut d2 = Dom::new();
    let doc2 = d2.parse_xdocument(&once);
    let root2 = d2.root(doc2).expect("root2");
    assert_eq!(d2.serialize_element(root2), want);
}
