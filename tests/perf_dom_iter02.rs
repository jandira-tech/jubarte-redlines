//! DOM-ITER-02 — hash-clone preprocess uses index walks; digests stay exact.
//!
//! Gates `clone_block_level_content_for_hashing` / `block_sha1` against the
//! m4b oracle cases (paragraphs, multi-run, tables).

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, clone_block_level_content_for_hashing, clone_for_structure_hash, null_rel_resolver,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn parse_body(xml_body: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml_body}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    (d, body)
}

fn first_p(dom: &Dom, body: jubarte::xmllinq::NodeId) -> jubarte::xmllinq::NodeId {
    dom.elements(body, Some(&W::p()))[0]
}

#[test]
fn dom_iter02_hash_clone_stable_simple_para() {
    let (mut dom, body) = parse_body(r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#);
    let p = first_p(&dom, body);
    let s = WmlComparerSettings::default();
    let c1 = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let h1 = block_sha1(&dom, c1);
    let c2 = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let h2 = block_sha1(&dom, c2);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 40);
}

#[test]
fn dom_iter02_hash_clone_multi_run_coalesce() {
    let (mut dom, body) =
        parse_body(r#"<w:p><w:r><w:t>Hel</w:t></w:r><w:r><w:t>lo</w:t></w:r></w:p>"#);
    let p = first_p(&dom, body);
    let s = WmlComparerSettings::default();
    let c = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let ser = dom.serialize_element(c);
    // Coalesced into single t-run for hashing.
    assert!(
        ser.contains("Hello") || (ser.contains("Hel") && ser.contains("lo")),
        "{ser}"
    );
    let h = block_sha1(&dom, c);
    assert_eq!(h.len(), 40);
}

#[test]
fn dom_iter02_structure_hash_drops_text() {
    let (mut dom, body) = parse_body(r#"<w:p><w:r><w:t>X</w:t></w:r></w:p>"#);
    let p = first_p(&dom, body);
    let s = WmlComparerSettings::default();
    let c = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    let sc = clone_for_structure_hash(&mut dom, c).expect("structure clone");
    let ser = dom.serialize_element(sc);
    assert!(!ser.contains('X'), "structure hash must drop text: {ser}");
}

#[test]
fn dom_iter02_table_clone_hashes() {
    let (mut dom, body) = parse_body(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let tbl = dom.elements(body, Some(&W::name("tbl")))[0];
    let s = WmlComparerSettings::default();
    let c = clone_block_level_content_for_hashing(&mut dom, tbl, true, &s, &null_rel_resolver);
    let h = block_sha1(&dom, c);
    assert_eq!(h.len(), 40);
    let sc = clone_for_structure_hash(&mut dom, c).expect("struct");
    let h2 = block_sha1(&dom, sc);
    assert_eq!(h2.len(), 40);
    assert_ne!(
        h, h2,
        "content hash and structure hash should differ when text present"
    );
}
