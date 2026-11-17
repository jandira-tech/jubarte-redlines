// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! broken_ones_two file_137×file_138 shape:
//! - base: stamped `file_137.docx` + long bold body paras
//! - next: stamped `file_138.docx` + short plain title
//!
//! Word Compare keeps 4 paragraphs: word-level confetti on the filename
//! (`file_` equal, number ins/del, `.docx` equal), pairwise replace of the
//! second para, then pure dels of trailing base body. Ours used to treat the
//! whole `file_N.docx` as one word, fire the short-vs-long unrelated
//! short-circuit, and emit insert-all-next then delete-all-base (6 pure
//! ins/del paras, pixel ~43).

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

/// Word confetti on stamped filenames: shared `file_` / `.docx`, digit run
/// inserted/deleted — not whole-string replace, not insert-all/delete-all.
#[test]
fn m49_filename_digit_boundary_confetti() {
    let mut dom = Dom::new();
    let base = [
        para("file_137.docx"),
        para("Track Changes Suggesting Title Bold Center Demo"),
        para("This document combines Suggesting mode with Title style."),
        para("This powerful combination shows major structural proposals."),
    ]
    .concat();
    let next = [para("file_138.docx"), para("Walking on imported air")].concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<_> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&e| dom.name(e) == Some(W::p()))
        .collect();
    assert!(
        paras.len() <= 5,
        "Word keeps ~4 paras (not insert-all then delete-all → 6); got {}",
        paras.len()
    );

    let first = dom.serialize_element(paras[0]);
    assert!(
        first.contains("file_") || first.contains(">file<") || first.contains("file"),
        "filename stem should survive: {first}"
    );
    // Must be MIX: both ins of 138 and del of 137, not pure whole-string ins/del of two paras
    let has_ins_138 = first.contains("138") && first.contains("<w:ins");
    let has_del_137 = first.contains("137") && first.contains("delText");
    assert!(
        has_ins_138 && has_del_137,
        "Word confetti on number run (ins 138 + del 137) in one para, got: {first}"
    );
    // Shared suffix/prefix should be Equal (not wrapped in ins/del as whole filename)
    assert!(
        !first.contains(">file_137.docx</w:delText>")
            && !first.contains(">file_138.docx</w:t></w:r></w:ins>"),
        "must not treat whole filename as single ins/del word: {first}"
    );
}

/// Same shape must not collapse to pure insert-all-next leading.
#[test]
fn m49_short_related_stamp_not_unrelated_insert_first() {
    let mut dom = Dom::new();
    let base = [
        para("file_137.docx"),
        para("Alpha unique base body one"),
        para("Beta unique base body two"),
        para("Gamma unique base body three"),
    ]
    .concat();
    let next = [para("file_138.docx"), para("Walking on imported air")].concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first = dom.serialize_element(dom.elements(body, Some(&W::p()))[0]);
    // Leading pure-ins of the whole next filename is the unrelated short-circuit
    let pure_ins_filename = first.contains("file_138")
        && first.contains("<w:ins")
        && !first.contains("delText")
        && !first.contains("137");
    assert!(
        !pure_ins_filename,
        "must not short-circuit to pure INS of next filename first: {first}"
    );
}
