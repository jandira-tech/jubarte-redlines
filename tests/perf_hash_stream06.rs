//! HASH-STREAM-06 — simple `w:tc` stream hash == clone oracle (no hash-clone DOM).

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, block_sha1_from_source, clone_block_level_content_for_hashing, null_rel_resolver,
    try_stream_hash_simple_table_or_tr,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_tc(xml: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr>{xml}</w:tr></w:tbl></w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    let tbl = d.elements(body, Some(&W::tbl()))[0];
    let tr = d.elements(tbl, Some(&W::tr()))[0];
    let tc = d.elements(tr, Some(&W::tc()))[0];
    (d, tc)
}

fn s_space() -> WmlComparerSettings {
    WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    }
}

fn oracle(dom: &mut Dom, tc: jubarte::xmllinq::NodeId, s: &WmlComparerSettings) -> String {
    let clone = clone_block_level_content_for_hashing(dom, tc, true, s, &null_rel_resolver);
    block_sha1(dom, clone)
}

#[test]
fn hash_stream06_simple_tc_matches() {
    let (mut dom, tc) = body_tc(r#"<w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc>"#);
    let s = s_space();
    let (c, _) = try_stream_hash_simple_table_or_tr(&dom, tc, &s, false).expect("stream tc");
    assert_eq!(c, oracle(&mut dom, tc, &s));
    assert_eq!(
        block_sha1_from_source(&mut dom, tc, true, &s, &null_rel_resolver, false),
        c
    );
}

#[test]
fn hash_stream06_tc_with_tcpr_and_br() {
    let (mut dom, tc) = body_tc(
        r#"<w:tc><w:tcPr/><w:p><w:r><w:t>a</w:t><w:br/></w:r></w:p></w:tc>"#,
    );
    let s = s_space();
    let (c, _) = try_stream_hash_simple_table_or_tr(&dom, tc, &s, false).expect("stream");
    assert_eq!(c, oracle(&mut dom, tc, &s));
}

#[test]
fn hash_stream06_gridspan_tc() {
    let (mut dom, tc) = body_tc(
        r#"<w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc>"#,
    );
    let s = s_space();
    let (c, _) = try_stream_hash_simple_table_or_tr(&dom, tc, &s, false).unwrap();
    assert_eq!(c, oracle(&mut dom, tc, &s));
}

#[test]
fn hash_stream06_drawing_falls_back() {
    let (mut dom, tc) = body_tc(r#"<w:tc><w:p><w:r><w:drawing/></w:r></w:p></w:tc>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_table_or_tr(&dom, tc, &s, false).is_none());
    assert_eq!(
        block_sha1_from_source(&mut dom, tc, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, tc, &s)
    );
}
