//! HASH-STREAM-02 — structure_sha1 == block_sha1(structure_clone).
//!
//! Production stamps `pt:StructureSHA1Hash` via streaming structure serialize;
//! this gate proves equality against the clone_for_structure_hash oracle.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, clone_block_level_content_for_hashing, clone_for_structure_hash, null_rel_resolver,
    structure_sha1,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn parse_body(xml: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    (d, body)
}

#[test]
fn hash_stream02_structure_matches_clone_oracle_on_table() {
    let (mut dom, body) = parse_body(
        r#"<w:tbl>
          <w:tr>
            <w:tc><w:p><w:r><w:t>cell-a</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>cell-b</w:t></w:r></w:p></w:tc>
          </w:tr>
        </w:tbl>"#,
    );
    let tbl = dom.elements(body, Some(&W::tbl()))[0];
    let s = WmlComparerSettings::default();
    let clone = clone_block_level_content_for_hashing(&mut dom, tbl, true, &s, &null_rel_resolver);
    let via_stream = structure_sha1(&dom, clone);
    let sc = clone_for_structure_hash(&mut dom, clone).expect("structure clone");
    let via_oracle = block_sha1(&dom, sc);
    assert_eq!(via_stream, via_oracle);
    assert_eq!(via_stream.len(), 40);
}

#[test]
fn hash_stream02_structure_differs_from_content_when_text_present() {
    let (mut dom, body) = parse_body(r#"<w:p><w:r><w:t>unique-text-xyz</w:t></w:r></w:p>"#);
    let p = dom.elements(body, Some(&W::p()))[0];
    let s = WmlComparerSettings::default();
    let clone = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let content = block_sha1(&dom, clone);
    let structure = structure_sha1(&dom, clone);
    assert_ne!(
        content, structure,
        "text should change content hash but not structure skeleton alone"
    );
}

#[test]
fn hash_stream02_empty_element_and_nested() {
    let (mut dom, body) = parse_body(r#"<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#);
    let tbl = dom.elements(body, Some(&W::tbl()))[0];
    let s = WmlComparerSettings::default();
    let clone = clone_block_level_content_for_hashing(&mut dom, tbl, true, &s, &null_rel_resolver);
    let sc = clone_for_structure_hash(&mut dom, clone).unwrap();
    assert_eq!(structure_sha1(&dom, clone), block_sha1(&dom, sc));
}
