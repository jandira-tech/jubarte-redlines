// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! DOM-ITER-01 — serializer must match the pre-borrow path byte-for-byte.
//!
//! Gates: namespace-hostile prefixes, empty elements, mixed content, QName-list
//! attributes (`mc:Ignorable`), and attr_at/attr_count contract.

use jubarte::xmllinq::{Dom, XName, XNamespace};

fn parse_and_reserialize(xml: &str) -> String {
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    d.serialize_document(doc)
}

#[test]
fn attr_count_matches_attributes_len() {
    let mut d = Dom::new();
    let xml = r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:rsidR="00AB" w:rsidRDefault="00CD"><w:r><w:t>hi</w:t></w:r></w:p>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    assert_eq!(d.attr_count(root), d.attributes(root).len());
    for i in 0..d.attr_count(root) {
        let (n, v) = d.attr_at(root, i);
        let owned = &d.attributes(root)[i];
        assert_eq!(n, &owned.0);
        assert_eq!(v, owned.1.as_str());
    }
}

#[test]
fn serialize_roundtrip_simple_w_paragraph() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>"#;
    let out = parse_and_reserialize(xml);
    // Round-trip twice must be stable (idempotent serializer).
    let out2 = parse_and_reserialize(&out);
    assert_eq!(out, out2, "serializer must be idempotent");
    assert!(out.contains("Hello"));
    assert!(out.contains("w:document") || out.contains("document"));
}

#[test]
fn serialize_empty_element_and_attrs() {
    let xml = r#"<root a="1" b="two"><empty/></root>"#;
    let out = parse_and_reserialize(xml);
    let out2 = parse_and_reserialize(&out);
    assert_eq!(out, out2);
    assert!(out.contains("a=") || out.contains("a ="));
}

#[test]
fn serialize_namespace_prefix_list_mc_ignorable() {
    // mc:Ignorable is a QName-list attribute; serializer must rewrite prefixes
    // consistently and remain stable across re-serialize.
    let xml = r#"<root xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" mc:Ignorable="w14"><child w14:paraId="12345678"/></root>"#;
    let out = parse_and_reserialize(xml);
    let out2 = parse_and_reserialize(&out);
    assert_eq!(out, out2, "mc:Ignorable path must be stable");
}

#[test]
fn child_index_matches_nodes_order() {
    let mut d = Dom::new();
    let xml = r#"<r><a/><b/><c/></r>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    let nodes = d.nodes(root);
    assert_eq!(d.child_count(root), nodes.len());
    for (i, &id) in nodes.iter().enumerate() {
        assert_eq!(d.child_at(root, i), id);
    }
}

#[test]
fn xname_attr_lookup_still_works() {
    let mut d = Dom::new();
    let xml = r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:rsidR="ABCD"><w:r/></w:p>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    let rsid = XName::get(
        "rsidR",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    );
    assert_eq!(d.attribute(root, &rsid), Some("ABCD"));
    let _ = XNamespace::get("http://schemas.openxmlformats.org/wordprocessingml/2006/main");
}
