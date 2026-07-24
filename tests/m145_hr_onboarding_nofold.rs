// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M145: long pure-I subtitle + short unrelated pure-D title must not fold
//! (hr_onboarding × long next doc).

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
fn long_subtitle_does_not_fold_short_unrelated_checklist_title() {
    let mut dom = Dom::new();
    // Short base checklist
    let base = [
        para("HR Onboarding Checklist"),
        para(""),
        para("Step"),
        para("Task"),
        para("Done"),
    ]
    .concat();
    // Long next starts with title + long subtitle then body
    let next = [
        para("Microsoft Word vs. Google Docs"),
        para("A comprehensive, evidence-backed demonstration document"),
        para("Positioning thesis Word provides the real-time collaboration people expect from modern cloud editors while adding deeper professional document production."),
        para("Prepared for"),
        para("Executive decision-makers"),
    ]
    .concat();
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
    // Find pure-I "comprehensive" and pure-D "HR Onboarding" as separate paras
    let mut found_pure_i_sub = false;
    let mut found_pure_d_hr = false;
    let mut mixed_both = false;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        let ser = dom.serialize_element(k);
        let has_comp = ser.contains("comprehensive");
        let has_hr = ser.contains("HR Onboarding") || ser.contains("Checklist");
        if has_comp && has_hr && has_ins && has_del {
            mixed_both = true;
        }
        if has_comp && has_ins && !has_del {
            found_pure_i_sub = true;
        }
        if has_hr && has_del && !has_ins {
            found_pure_d_hr = true;
        }
    }
    assert!(
        !mixed_both,
        "Word keeps subtitle pure-I and HR title pure-D separate"
    );
    assert!(
        found_pure_i_sub || found_pure_d_hr,
        "expect pure-I subtitle and/or pure-D HR; kids={}",
        kids.len()
    );
}

#[test]
fn prepared_for_does_not_fold_digit_checklist_cells() {
    let mut dom = Dom::new();
    let base = [
        para("HR Onboarding Checklist"),
        para("1"),
        para("Sign NDA"),
        para("Yes"),
        para("2"),
        para("Setup laptop"),
        para("No"),
    ]
    .concat();
    let next = [
        para("Microsoft Word vs. Google Docs"),
        para("A comprehensive, evidence-backed demonstration document"),
        para("Positioning thesis Word provides real-time collaboration people expect from modern cloud editors while adding deeper professional document production."),
        para("Prepared for"),
        para("Executive / Sales / IT decision-makers"),
        para("Prepared by"),
        para("Microsoft Word capability team"),
        para("Date"),
        para("2026-07-03"),
    ]
    .concat();
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
        let ser = dom.serialize_element(k);
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        if ser.contains("Prepared for") && has_ins && has_del {
            panic!("Prepared for must not MIX with checklist cells: {ser}");
        }
        if ser.contains("capability team") && has_ins && has_del && ser.contains("Setup") {
            panic!("capability team must not MIX with Setup laptop: {ser}");
        }
    }
}
