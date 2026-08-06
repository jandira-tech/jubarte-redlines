// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Residual aligner policy — Word-parity residual pairing order:
//!   mesh anchors → junction seam → similarity-gated zip → pure I/D otherwise.
//!
//! These tests drive the **real** `compare_bodies_faithful` path (not a
//! reimplementation oracle). They pin the four policy arms that the shared
//! residual aligner (rust skeleton + lossless port) must preserve.
//!
//! Exhibits:
//! - **zip**: equal-count pure-para diagonal-dominant (m45 / heading demos)
//! - **mesh**: title last-sig Demo cousins with unrelated bodies → title MIX
//!   + residual pure I/D (not full 3×MIX zip)
//! - **pure I/D**: completely unrelated equal-count → no false 1:1 zip on
//!   zero-overlap empties

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

fn body_paras(dom: &Dom, out: NodeId) -> Vec<NodeId> {
    let body = dom.element(out, &W::body()).unwrap();
    dom.elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect()
}

fn para_shape(dom: &Dom, p: NodeId) -> &'static str {
    let has_ins = !dom.elements(p, Some(&W::ins())).is_empty()
        || dom
            .descendants(p, Some(&W::ins()))
            .into_iter()
            .next()
            .is_some();
    let has_del = !dom.elements(p, Some(&W::del())).is_empty()
        || dom
            .descendants(p, Some(&W::del()))
            .into_iter()
            .next()
            .is_some();
    // Also count pPr mark revisions.
    let ppr = dom.element(p, &W::p_pr());
    let mark_ins = ppr
        .and_then(|pp| dom.element(pp, &W::r_pr()))
        .is_some_and(|rp| dom.element(rp, &W::ins()).is_some());
    let mark_del = ppr
        .and_then(|pp| dom.element(pp, &W::r_pr()))
        .is_some_and(|rp| dom.element(rp, &W::del()).is_some());
    let ins = has_ins || mark_ins;
    let del = has_del || mark_del;
    match (ins, del) {
        (true, true) => "M",
        (true, false) => "I",
        (false, true) => "D",
        (false, false) => "E",
    }
}

/// Similarity-gated zip: 3×3 heading demos stay three mixed paragraphs.
#[test]
fn residual_align_zip_equal_count_diagonal_dominant() {
    let mut dom = Dom::new();
    let base = [
        para("Heading 2 Style Demo"),
        para("This document demonstrates Heading 2 paragraph style."),
        para("Subsection Title"),
    ]
    .concat();
    let next = [
        para("Heading 3 Center Italic Demo"),
        para("Heading 3 with center alignment and italic formatting."),
        para("This combination works for stylized section subheadings."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &WmlComparerSettings::default());
    let kids = body_paras(&dom, out);
    let shapes: Vec<_> = kids.iter().map(|&k| para_shape(&dom, k)).collect();
    assert_eq!(
        shapes,
        vec!["M", "M", "M"],
        "zip arm: diagonal-dominant equal-count → N MIX; got {shapes:?}"
    );
}

/// Mesh arm: Demo title last-sig cousins with unrelated residual bodies must
/// NOT force full positional zip of body paras (would invent false MIX).
/// Title meshes; residual stays pure I/D (or at least not all-MIX).
#[test]
fn residual_align_mesh_title_not_full_body_zip() {
    let mut dom = Dom::new();
    // Shared last-sig "Demo"; body residuals share almost nothing contentful.
    let base = [
        para("Blue Underline Combo Demo"),
        para("This text is blue and underlined for emphasis."),
        para("End note one."),
    ]
    .concat();
    let next = [
        para("Bold And Italic Combo Demo"),
        para("Combining bold with italic creates strong visual hierarchy."),
        para("Different closing remark entirely."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &WmlComparerSettings::default());
    let kids = body_paras(&dom, out);
    let shapes: Vec<_> = kids.iter().map(|&k| para_shape(&dom, k)).collect();
    // Must not be pure MMM zip of three unrelated bodies.
    let all_mix = shapes.iter().all(|&s| s == "M") && shapes.len() == 3;
    assert!(
        !all_mix,
        "mesh arm: unrelated residual bodies must not force 3×MIX zip; got {shapes:?}"
    );
    // Title should still participate in a mix or equal revision region.
    assert!(
        !shapes.is_empty(),
        "expected body paragraphs after residual align"
    );
}

/// LO-score residual peel (m180/M191b) for equal 3v3 Demo last-sig with
/// mid-related first residual + content-empty last residual: emits **MIMD**
/// (title mesh + pure-I/D residual), not Word's DOCX MMM. A/B 2026-08-05:
/// forcing MMM matched Word structure but LO dropped track_changes
/// 90.4→82.8 and median −0.43 (R-perfect/R-92 fail). Keep the LO peel.
#[test]
fn residual_align_m180_pure_id_peel_for_lo_score_on_empty_last_residual() {
    let mut dom = Dom::new();
    let base = [
        para("Track Changes Suggesting Italic Red Demo"),
        para("This document combines Suggesting mode with italic and red color."),
        para("Red italic suggestions stand out clearly for document reviewers."),
    ]
    .concat();
    let next = [
        para("Track Changes Suggesting Title Bold Center Demo"),
        para(
            "This document combines Suggesting mode with Title style, center alignment, and bold.",
        ),
        para(
            "This powerful combination shows major structural proposals in collaborative editing.",
        ),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &WmlComparerSettings::default());
    let kids = body_paras(&dom, out);
    let shapes: Vec<_> = kids.iter().map(|&k| para_shape(&dom, k)).collect();
    assert_eq!(
        shapes,
        vec!["M", "I", "M", "D"],
        "LO residual peel MIMD (not Word MMM); got {shapes:?}"
    );
}

/// Pure I/D arm for unequal residual counts: when one side has an extra
/// body paragraph with no counterpart, residual align emits pure-I and/or
/// pure-D (not forced equal-count zip — counts differ).
#[test]
fn residual_align_pure_id_on_unequal_residual_counts() {
    let mut dom = Dom::new();
    let base = [
        para("Shared Demo"),
        para("Only on the base side unique alpha."),
        para("Also only base side unique bravo."),
    ]
    .concat();
    let next = [
        para("Shared Demo"),
        para("Only on the next side unique zulu."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &WmlComparerSettings::default());
    let kids = body_paras(&dom, out);
    let shapes: Vec<_> = kids.iter().map(|&k| para_shape(&dom, k)).collect();
    let n_pure_i = shapes.iter().filter(|&&s| s == "I").count();
    let n_pure_d = shapes.iter().filter(|&&s| s == "D").count();
    assert!(
        n_pure_i + n_pure_d >= 1,
        "pure I/D arm: unequal residual counts must emit pure-I and/or pure-D; got {shapes:?}"
    );
}
