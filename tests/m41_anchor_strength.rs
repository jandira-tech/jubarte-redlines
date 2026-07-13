//! M-ANCHOR attempt 3 — strength+relatedness anchor gate (word mode).
//!
//! A whole-document replacement (two UNRELATED large documents) must not let
//! a single short junk paragraph shared by coincidence ("(dolore)") anchor
//! the LCS: Word collapses such windows to insert-all + delete-all, keeping
//! A's deleted paragraphs as ONE consolidated cluster (sd2517b GT shape).
//! Evidence: parity/_scratch/anchor_sensitivity.md, sd2517b_physics.md.
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

/// 40 unique paragraphs, lorem-ish, with a single short "(dolore)" paragraph
/// planted mid-document. `seed` makes the two sides fully disjoint except for
/// that one junk paragraph.
fn forty_paras(seed: &str) -> String {
    let mut s = String::new();
    for i in 0..40 {
        if i == 20 {
            s.push_str(&para("(dolore)"));
        } else {
            s.push_str(&para(&format!(
                "{seed} paragraph {i} consectetur adipiscing elit {seed}{i} sed do eiusmod \
                 tempor incididunt ut labore {seed} magna aliqua {i}"
            )));
        }
    }
    s
}

/// Whole-doc replacement: the "(dolore)" coincidence paragraph must NOT
/// anchor — A's deleted paragraphs come out as one contiguous cluster.
#[test]
fn m41_junk_anchor_voided_in_large_unrelated_window() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, &forty_paras("alpha"));
    let (r2, b2) = doc_body(&mut dom, &forty_paras("zulu"));
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    // Classify each top-level output paragraph: does it carry a deletion?
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&e| dom.name(e) == Some(W::p()))
        .collect();
    let del_flags: Vec<bool> = paras
        .iter()
        .map(|&p| !dom.descendants(p, Some(&W::del())).is_empty())
        .collect();

    let first = del_flags.iter().position(|&d| d);
    let last = del_flags.iter().rposition(|&d| d);
    let (Some(first), Some(last)) = (first, last) else {
        panic!("no deleted paragraphs in output");
    };
    let holes: Vec<usize> = (first..=last).filter(|&i| !del_flags[i]).collect();
    assert!(
        holes.is_empty(),
        "deleted paragraphs scattered: DEL paras span [{first}..={last}] \
         with non-DEL holes at {holes:?} (junk '(dolore)' anchor split the cluster)"
    );
    // All 40 A paragraphs must be deleted (nothing survives as Equal).
    let del_count = del_flags.iter().filter(|&&d| d).count();
    assert_eq!(
        del_count, 40,
        "expected all 40 A paragraphs deleted, got {del_count}"
    );
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
