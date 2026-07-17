// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M101: digits-only pure-I ("24") must not fold into multi pure-D demo body
//! (1_5_line_spacing × 24).

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

#[test]
fn digits_pure_i_does_not_mix_with_demo_title() {
    let mut dom = Dom::new();
    let base = [
        para("1.5 Line Spacing Demo"),
        para("This document demonstrates 1.5 line spacing."),
        para("The space between lines is one and a half times normal."),
        para("This improves readability of body text in long documents."),
    ]
    .concat();
    // Next is digits-only + empty (matches 24_id_paraid_overflow shape).
    let next = [para("24"), para("")].concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        let ser = dom.serialize_element(k);
        if has_ins && has_del && ser.contains("24") && ser.contains("1.5") {
            panic!("Word keeps pure-I 24 and pure-D title separate; got MIX: {ser}");
        }
    }
    // Expect pure-I 24 somewhere
    let mut found_pure_i24 = false;
    let mut found_pure_d_title = false;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        let ser = dom.serialize_element(k);
        if has_ins && !has_del && ser.contains(">24<") {
            found_pure_i24 = true;
        }
        if has_del && !has_ins && ser.contains("1.5") {
            found_pure_d_title = true;
        }
    }
    assert!(found_pure_i24, "pure-I 24 missing; kids={}", kids.len());
    assert!(found_pure_d_title, "pure-D title missing");
}
