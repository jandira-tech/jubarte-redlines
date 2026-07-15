//! HASH-STREAM-05 — simple-p stream allows empty run leaves (br/tab/…) with
//! fragment expansion + adjacent-t merge, matching clone oracle.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, block_sha1_from_source, clone_block_level_content_for_hashing, null_rel_resolver,
    try_stream_hash_simple_paragraph, try_stream_hash_simple_table_or_tr,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_p(xml: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    let p = d.elements(body, Some(&W::p()))[0];
    (d, p)
}

fn body_tbl(xml: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    let tbl = d.elements(body, Some(&W::tbl()))[0];
    (d, tbl)
}

fn oracle(dom: &mut Dom, p: jubarte::xmllinq::NodeId, s: &WmlComparerSettings) -> String {
    let clone = clone_block_level_content_for_hashing(dom, p, true, s, &null_rel_resolver);
    block_sha1(dom, clone)
}

fn s_space() -> WmlComparerSettings {
    WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    }
}

#[test]
fn hash_stream05_br_only_run() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:br/></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream05_t_then_br_same_run_fragments() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>a</w:t><w:br/></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_br_between_text_runs() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>a</w:t></w:r><w:r><w:br/></w:r><w:r><w:t>b</w:t></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_br_type_page_attr() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_tab_leaf() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:tab/></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_multi_t_merge_before_br() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>x</w:t><w:t>y</w:t><w:br/></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_table_cell_with_br() {
    let (mut dom, tbl) = body_tbl(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:br/></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let s = s_space();
    let (c, _) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false)
        .expect("br-only cell must stream after HASH-STREAM-05");
    let clone = clone_block_level_content_for_hashing(&mut dom, tbl, true, &s, &null_rel_resolver);
    assert_eq!(c, block_sha1(&dom, clone));
}

#[test]
fn hash_stream05_still_rejects_drawing() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:drawing/></w:r></w:p>"#);
    let s = s_space();
    // drawing is not an allowed empty leaf for stream (has complex clone path)
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_none());
    // still must match via fallback
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}

#[test]
fn hash_stream05_empty_run_skipped() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:rPr/></w:r><w:r><w:t>hi</w:t></w:r></w:p>"#);
    let s = s_space();
    assert!(try_stream_hash_simple_paragraph(&dom, p, &s, false).is_some());
    assert_eq!(
        block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false),
        oracle(&mut dom, p, &s)
    );
}
