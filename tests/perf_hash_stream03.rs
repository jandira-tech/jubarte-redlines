//! HASH-STREAM-03 — simple-paragraph stream hash == clone oracle.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, block_sha1_from_source, clone_block_level_content_for_hashing, null_rel_resolver,
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

fn oracle(dom: &mut Dom, p: jubarte::xmllinq::NodeId, settings: &WmlComparerSettings) -> String {
    let clone = clone_block_level_content_for_hashing(dom, p, true, settings, &null_rel_resolver);
    block_sha1(dom, clone)
}

#[test]
fn hash_stream03_simple_hello_matches_clone() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#);
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03_merged_runs_match_clone() {
    let (mut dom, p) = body_p(
        r#"<w:p><w:pPr><w:jc/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Hel</w:t></w:r><w:r><w:t>lo</w:t></w:r></w:p>"#,
    );
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03_conflate_nbsp_matches() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>a b</w:t></w:r></w:p>"#);
    let s = WmlComparerSettings::default(); // conflate on
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03_complex_falls_back_still_matches() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:footnoteReference w:id="7"/></w:r></w:p>"#);
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03_empty_p_matches() {
    let (mut dom, p) = body_p(r#"<w:p><w:pPr/></w:p>"#);
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03b_multi_t_run_matches_clone() {
    // One run with two w:t children → clone fragments then merges to "ab"
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>a</w:t><w:t>b</w:t></w:r></w:p>"#);
    let s = WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    };
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle(&mut dom, p, &s));
}

#[test]
fn hash_stream03b_correlated_ws_matches_clone_path() {
    let (mut dom, p) = body_p(r#"<w:p><w:r><w:t>a b</w:t></w:r><w:r><w:t> c</w:t></w:r></w:p>"#);
    let s = WmlComparerSettings::default();
    let stream = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, true);
    let oracle = block_sha1_from_source(&mut dom, p, true, &s, &null_rel_resolver, true);
    // Force oracle via clone by using a complex marker would be different;
    // stream twice must be stable, and equal clone path:
    let clone = clone_block_level_content_for_hashing(&mut dom, p, true, &s, &null_rel_resolver);
    // strip whitespace on clone text like production correlated path
    fn strip_ws(dom: &mut Dom, id: jubarte::xmllinq::NodeId) {
        if dom.is_text(id) {
            let raw = dom.text_value(id).unwrap_or("").to_string();
            let stripped: String = raw.chars().filter(|ch| !ch.is_whitespace()).collect();
            if stripped != raw {
                dom.set_text_value(id, &stripped);
            }
        }
        let n = dom.child_count(id);
        for i in 0..n {
            let c = dom.child_at(id, i);
            strip_ws(dom, c);
        }
    }
    strip_ws(&mut dom, clone);
    assert_eq!(stream, block_sha1(&dom, clone));
    assert_eq!(stream, oracle);
}
