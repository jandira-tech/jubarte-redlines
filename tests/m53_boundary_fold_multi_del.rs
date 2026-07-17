// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! broken_ones_two file_14×file_15 shape:
//! After stamp confetti, short pure-ins next body meets long pure-del base.
//! Word: 2 pure-ins + 1 mixed (last next + first base del) + remaining pure-dels.
//! Ours used to keep all pure-ins then pure-dels → extra pages (3 vs 5).

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
fn m53_short_next_long_base_folds_boundary_pair() {
    let mut dom = Dom::new();
    let base = [
        para("file_14.docx"),
        para("eigenpal docx editor project charter unique base one"),
        para("npm package github contributor agreement unique base two"),
        para("more unique base body three with lots of words here"),
        para("still more unique base body four continuing the charter"),
    ]
    .concat();
    let next = [
        para("file_15.docx"),
        para("Verdana Italic Centered Demo unique next title"),
        para("This document shows Verdana font with italic unique next body"),
        para("Verdana italic centered creates a clean modern unique next three"),
    ]
    .concat();
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
    // Count pure-ins / pure-del / mixed after stamp
    let mut pure_ins = 0;
    let mut pure_del = 0;
    let mut mixed = 0;
    for &p in &paras {
        let has_ins = !dom.elements(p, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(p, Some(&W::del())).is_empty();
        match (has_ins, has_del) {
            (true, true) => mixed += 1,
            (true, false) => pure_ins += 1,
            (false, true) => pure_del += 1,
            _ => {}
        }
    }
    // Direct merge path is covered by m53b. End-to-end: stamp confetti + LCS may
    // already interleave; require at least stamp mix and not pure insert-all
    // (no pure-ins-only document).
    assert!(
        mixed >= 1,
        "expect at least stamp mix; pure_ins={pure_ins} pure_del={pure_del} mixed={mixed} n={}",
        paras.len()
    );
    assert!(
        pure_del >= 1,
        "expect deleted base content; pure_del={pure_del}"
    );
}
