// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! HASH-STREAM-04 — simple-table / simple-row stream hash == clone oracle
//! (content + structure digests, no hash-clone DOM on the stream path).

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::preprocess::{
    block_sha1, block_sha1_from_source, clone_block_level_content_for_hashing,
    clone_for_structure_hash, null_rel_resolver, structure_sha1,
    try_stream_hash_simple_table_or_tr,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_block(xml: &str, local: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{xml}</w:body></w:document>"#
    );
    let doc = d.parse_xdocument(&full);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    let name = W::name(local);
    let n = d.elements(body, Some(&name))[0];
    (d, n)
}

fn oracle_content(
    dom: &mut Dom,
    node: jubarte::xmllinq::NodeId,
    settings: &WmlComparerSettings,
    correlated_ws: bool,
) -> String {
    let clone =
        clone_block_level_content_for_hashing(dom, node, true, settings, &null_rel_resolver);
    if correlated_ws {
        // mirror production strip after clone
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
        strip_ws(dom, clone);
    }
    block_sha1(dom, clone)
}

fn oracle_structure(
    dom: &mut Dom,
    node: jubarte::xmllinq::NodeId,
    settings: &WmlComparerSettings,
) -> String {
    let clone =
        clone_block_level_content_for_hashing(dom, node, true, settings, &null_rel_resolver);
    structure_sha1(dom, clone)
}

fn settings_space_sensitive() -> WmlComparerSettings {
    WmlComparerSettings {
        conflate_breaking_and_nonbreaking_spaces: false,
        ..Default::default()
    }
}

#[test]
fn hash_stream04_simple_1x1_matches_clone() {
    let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let (mut dom, tbl) = body_block(xml, "tbl");
    let s = settings_space_sensitive();
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, false));
    let (c, st) =
        try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).expect("simple table must stream");
    assert_eq!(c, stream);
    assert_eq!(st, oracle_structure(&mut dom, tbl, &s));
}

#[test]
fn hash_stream04_tblpr_dropped_same_as_without() {
    // tblPr is dropped by clone; stream must match
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tblPr/><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, false));
    assert!(try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).is_some());
}

#[test]
fn hash_stream04_merged_runs_in_cell() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Hel</w:t></w:r><w:r><w:t>lo</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, false));
    let (_, st) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).unwrap();
    assert_eq!(st, oracle_structure(&mut dom, tbl, &s));
}

#[test]
fn hash_stream04_empty_tcpr_and_two_cells() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:trPr/><w:tc><w:tcPr/><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, false));
    let (c, st) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).unwrap();
    assert_eq!(c, stream);
    assert_eq!(st, oracle_structure(&mut dom, tbl, &s));
}

#[test]
fn hash_stream04_gridspan_val_no_prefix() {
    // Clone rewrites w:val → bare val=""
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, false));
    let (_, st) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).unwrap();
    assert_eq!(st, oracle_structure(&mut dom, tbl, &s));
}

#[test]
fn hash_stream04_multi_p_cell() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>a</w:t></w:r></w:p><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    assert_eq!(
        block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false),
        oracle_content(&mut dom, tbl, &s, false)
    );
}

#[test]
fn hash_stream04_empty_p_cell() {
    let (mut dom, tbl) = body_block(r#"<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#, "tbl");
    let s = settings_space_sensitive();
    assert_eq!(
        block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false),
        oracle_content(&mut dom, tbl, &s, false)
    );
    assert!(try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).is_some());
}

#[test]
fn hash_stream04_tr_root_matches() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>row</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let tr = dom.elements(tbl, Some(&W::tr()))[0];
    let stream = block_sha1_from_source(&mut dom, tr, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, oracle_content(&mut dom, tr, &s, false));
    let (c, st) = try_stream_hash_simple_table_or_tr(&dom, tr, &s, false).unwrap();
    assert_eq!(c, stream);
    assert_eq!(st, oracle_structure(&mut dom, tr, &s));
}

#[test]
fn hash_stream04_br_in_cell_streams_after_05() {
    // HASH-STREAM-05: empty leaf `w:br` is streamable; table cell streams too.
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:br/></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let (c, st) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false)
        .expect("br-only cell streams after HASH-STREAM-05");
    assert_eq!(c, oracle_content(&mut dom, tbl, &s, false));
    assert_eq!(st, oracle_structure(&mut dom, tbl, &s));
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false);
    assert_eq!(stream, c);
}

#[test]
fn hash_stream04_ppr_rpr_dropped() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:pPr/><w:r><w:rPr/><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    assert_eq!(
        block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, false),
        oracle_content(&mut dom, tbl, &s, false)
    );
}

#[test]
fn hash_stream04_correlated_ws() {
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>a b</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = WmlComparerSettings::default(); // conflate on; correlated strips ws
    let stream = block_sha1_from_source(&mut dom, tbl, true, &s, &null_rel_resolver, true);
    assert_eq!(stream, oracle_content(&mut dom, tbl, &s, true));
}

#[test]
fn hash_stream04_structure_matches_structure_clone_oracle() {
    // structure_sha1(content_clone) == stream structure, and equals
    // serialize of clone_for_structure_hash(content_clone)
    let (mut dom, tbl) = body_block(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        "tbl",
    );
    let s = settings_space_sensitive();
    let (_, st) = try_stream_hash_simple_table_or_tr(&dom, tbl, &s, false).unwrap();
    let clone = clone_block_level_content_for_hashing(&mut dom, tbl, true, &s, &null_rel_resolver);
    let sc = clone_for_structure_hash(&mut dom, clone).unwrap();
    assert_eq!(st, block_sha1(&dom, sc));
}
