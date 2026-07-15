//! HASH-STREAM-01 lite — streaming block SHA-1 == string-oracle digest.
//!
//! `block_sha1` / `serialize_element_sha1_hex` must match
//! `sha1_hex(block_hash_string(...))` on real hash-clone shapes.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_hash_string, block_sha1, clone_block_level_content_for_hashing, null_rel_resolver,
};
use jubarte::namespaces::W;
use jubarte::util::sha1::sha1_hex;
use jubarte::xmllinq::Dom;

fn body_p(xml_inner: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml_inner}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    let p = d.elements(body, Some(&W::p()))[0];
    (d, p)
}

#[test]
fn hash_stream_lite_matches_string_oracle_simple() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#);
    let s = WmlComparerSettings::default();
    let c = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let via_string = sha1_hex(&block_hash_string(&dom, c));
    let via_stream = block_sha1(&dom, c);
    assert_eq!(via_stream, via_string);
    assert_eq!(via_stream.len(), 40);
}

#[test]
fn hash_stream_lite_matches_multi_run_and_table() {
    let cases = [
        r#"<w:p><w:r><w:t>Hel</w:t></w:r><w:r><w:t>lo</w:t></w:r></w:p>"#,
        r#"<w:p w:rsidR="00AB"><w:r><w:t xml:space="preserve"> a&amp;b </w:t></w:r></w:p>"#,
    ];
    let s = WmlComparerSettings::default();
    for xml in cases {
        let (mut dom, p) = body_p(xml);
        let c = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
        assert_eq!(
            block_sha1(&dom, c),
            sha1_hex(&block_hash_string(&dom, c)),
            "mismatch for {xml}"
        );
    }
}

#[test]
fn hash_stream_lite_direct_serialize_api() {
    let (dom, p) = body_p(r#"<w:p><w:r><w:t>Z</w:t></w:r></w:p>"#);
    let mut s = dom.serialize_element(p);
    const WML: &str = " xmlns=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"";
    if let Some(i) = s.find(WML) {
        s.drain(i..i + WML.len());
    }
    assert_eq!(dom.serialize_element_sha1_hex(p), sha1_hex(&s));
}
