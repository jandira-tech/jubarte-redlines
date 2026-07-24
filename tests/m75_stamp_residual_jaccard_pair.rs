// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M75 — after stamp confetti, short base residual paragraphs that share
//! body tokens with a next residual (Jaccard ≥ 0.25) must pair for word-level
//! LCS, not pure-delete after insert-all. file_33×file_34: Word pairs
//! "This document demonstrates …" cousins; pure replace left 3 pages vs 2.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn doc_body(dom: &mut Dom, inner: &str) -> (jubarte::xmllinq::NodeId, jubarte::xmllinq::NodeId) {
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

#[test]
fn m75_short_residual_pairs_related_demonstrates_sentences() {
    let mut dom = Dom::new();
    // Base: stamp + short heading-demo residual (file_33 shape).
    let base = [
        para("file_33.docx"),
        para("Heading 1 Style Demo"),
        para("This document demonstrates Heading 1 paragraph style."),
        para("Main Title Section"),
    ]
    .concat();
    // Next: stamp + long unrelated demo with one related summary sentence.
    let next = [
        para("file_34.docx"),
        para("Comprehensive DOCX Features Demonstration"),
        para("1. Inline Text Formatting"),
        para("Normal text bold italic unique next body alpha"),
        para("Summary"),
        para("This document demonstrates all major DOCX features:"),
        para("Inline formatting unique bullet one"),
        para("Text alignment options"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let ser = dom.serialize_element(body);
    // Word-style: shared "This document demonstrates" survives as live text
    // with del of A-only tail / ins of B-only tail inside one MIX para — not
    // pure-del of the whole A sentence after all B inserts.
    assert!(
        ser.contains("This document demonstrates"),
        "shared prefix should appear: {ser}"
    );
    // Must not leave A sentence only as pure delText without any live sibling
    // in a mixed paragraph — at least one para should hold both ins and del
    // around the demonstrates cousins.
    let mut found_mix = false;
    for p in dom.elements(body, Some(&W::p())) {
        let ps = dom.serialize_element(p);
        let has_demo = ps.contains("demonstrates");
        let has_del = ps.contains("delText");
        let has_ins = ps.contains("<w:ins") || ps.contains("w:ins ");
        let has_live = ps.contains("<w:t") && !ps.contains("delText");
        // MIX: del of heading/paragraph style bits + ins of major/features
        if has_demo
            && has_del
            && (has_ins || has_live)
            && (ps.contains("paragraph style") || ps.contains("major") || ps.contains("features"))
        {
            found_mix = true;
            break;
        }
    }
    assert!(
        found_mix,
        "expected MIX para pairing related demonstrates sentences, got: {ser}"
    );
}

#[test]
fn m75_unrelated_stamp_still_confettis_without_false_pairs() {
    // file_134 class: long unique base residual must stay insert-all / delete-all
    // (no jaccard pair across disjoint demos).
    let mut dom = Dom::new();
    let base = [
        para("file_134.docx"),
        para("eigenpal docx editor project charter unique base alpha zeta"),
        para("npm package github contributor agreement unique base beta"),
        para("more unique base body gamma delta epsilon theta"),
        para("still more unique base body iota kappa lambda"),
        para("final unique base residual mu nu xi omicron"),
        para("extra unique base para pi rho sigma tau"),
        para("seventh unique base so residual longer than six"),
    ]
    .concat();
    let next = [
        para("file_135.docx"),
        para("Track Changes Editing Strikethrough Blue Demo unique next"),
        para("This document uses Editing mode with blue unique next two"),
        para("Blue strikethrough marks deleted content unique next three"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first = dom.serialize_element(dom.elements(body, Some(&W::p()))[0]);
    assert!(
        first.contains("135") && first.contains("delText") && first.contains("134"),
        "stamp confetti still required: {first}"
    );
}
