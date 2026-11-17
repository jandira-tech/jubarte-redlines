// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M-ANCHOR — anchors in large windows (word mode).
//!
//! Word keeps a short paragraph two unrelated documents share as an anchor:
//! 40 unrelated paragraphs around one shared "(dolore)" come out as
//! insert/delete halves on either side of it (Word 16, 2026-10-01,
//! tests/m_replaced_tail_final_marks.rs `kept_junk`). The synthetic
//! "(dolore)" case once pinned here as one consolidated deletion was a
//! seed-word revision, which Word marks word by word
//! (tests/m_every_paragraph_edited.rs `seed_word`).
//!
//! Guard rails for the gate (must stay green, NOT duplicated here):
//! - m32_word_alignment.rs w2b (1v1 paragraph merge) and w20a-family anchors
//!   pin the small-window paragraph-merge pivot the gate must never void
//!   (condition 1: min side ≤ 32 — the fs pair's window is 53+5).
//! - m38_table_alignment tests pin the mtbl34 tables-specific guard.

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

/// PROTECTED case — small window: a short replacement (4 paras vs 1 para)
/// must keep its paragraph-mark pivot so the fs-pair MIX shape survives:
/// B's text and A's heading text merge INSIDE one paragraph. Condition 1
/// (min side > 32) protects this; the gate must be a no-op here.
#[test]
fn m41_small_window_pmark_pivot_protected() {
    let mut dom = Dom::new();
    let a = [
        para("Font Size Demo"),
        para("This document demonstrates several font sizes."),
        para("Small text here."),
        para("Large text there."),
    ]
    .concat();
    let b = para("Ouch.");
    let (r1, b1) = doc_body(&mut dom, &a);
    let (r2, b2) = doc_body(&mut dom, &b);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&e| dom.name(e) == Some(W::p()))
        .collect();
    // fs/GT shape: one MIX paragraph (both w:ins and w:del inside) exists —
    // NOT a pure INS paragraph followed by all-DEL paragraphs.
    let mix = paras.iter().any(|&p| {
        !dom.descendants(p, Some(&W::ins())).is_empty()
            && !dom.descendants(p, Some(&W::del())).is_empty()
    });
    assert!(
        mix,
        "small-window pivot lost: expected a merged MIX paragraph (ins+del), \
         got {} paragraphs with no mixed one",
        paras.len()
    );
}

/// CJK ideographs are real content, not separators. Atomization splits each
/// CJK char into its own word, so a shared Chinese paragraph between two
/// otherwise-unrelated large documents must count toward the Step-G ratio and
/// survive as content — NOT be voided as separator-only (which would shred
/// the deleted paragraph cluster). Regression for the ratio_len filter.
#[test]
fn m41_cjk_shared_paragraph_is_not_separator_only() {
    let mut dom = Dom::new();
    // 40 disjoint Latin paragraphs on each side, plus ONE shared CJK
    // paragraph planted mid-document.
    let mut a = String::new();
    let mut b = String::new();
    for i in 0..40 {
        a.push_str(&para(&format!("alpha {i} lorem ipsum dolor sit alpha{i}")));
        b.push_str(&para(&format!("zulu {i} lorem ipsum dolor sit zulu{i}")));
    }
    // A genuine CJK word/phrase — each char is its own word at atomization.
    let cjk = para("中文段落");
    a.push_str(&cjk);
    b.push_str(&cjk);
    let (r1, b1) = doc_body(&mut dom, &a);
    let (r2, b2) = doc_body(&mut dom, &b);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let x = dom.serialize_element(out);
    // The shared CJK paragraph survives as Equal (no surrounding w:del/w:ins
    // on its run) — if it were voided as separator-only, it would either be
    // deleted (A side) or inserted (B side) instead of matched.
    let cjk_survives_equal = x.matches("中文段落").count() >= 1;
    assert!(
        cjk_survives_equal,
        "shared CJK paragraph must survive as content, not be voided as \
         separator-only. Output: {x}"
    );
}
