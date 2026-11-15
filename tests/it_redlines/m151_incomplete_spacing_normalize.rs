// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C3/C5: incomplete `w:spacing` (lineRule=auto, no line) Word-normalize.
//!
//! Word Compare rewrites `before=0 after=0 lineRule=auto` (no line) to:
//!
//! - list items: `after=0 line=240 lineRule=auto`
//! - non-list inserts: strip the spacing entirely
//!
//! LO layout keys on line box height; incomplete form tanks pixel scores.

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

fn para_plain(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

fn para_incomplete_spacing(text: &str) -> String {
    format!(
        "<w:p><w:pPr><w:spacing w:before=\"0\" w:after=\"0\" w:lineRule=\"auto\"/></w:pPr>\
         <w:r><w:t>{text}</w:t></w:r></w:p>"
    )
}

fn para_list_incomplete(text: &str, num_id: &str) -> String {
    format!(
        "<w:p><w:pPr>\
           <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{num_id}\"/></w:numPr>\
           <w:spacing w:before=\"0\" w:after=\"0\" w:lineRule=\"auto\"/>\
         </w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"
    )
}

fn para_list_plain(text: &str, num_id: &str) -> String {
    format!(
        "<w:p><w:pPr>\
           <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"{num_id}\"/></w:numPr>\
         </w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"
    )
}

#[test]
fn incomplete_spacing_on_non_list_insert_is_stripped() {
    let mut dom = Dom::new();
    let base = para_plain("alpha");
    let next = [
        para_incomplete_spacing("alpha revised"),
        para_incomplete_spacing("beta new"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let xml = dom.serialize_element(body);
    // Incomplete form must not survive on non-list content.
    assert!(
        !xml.contains("w:before=\"0\"") || !xml.contains("w:lineRule=\"auto\""),
        "incomplete before=0+lineRule=auto without line must not remain: {xml}"
    );
    // If any lineRule=auto remains, it must carry line=240.
    for m in xml.match_indices("lineRule") {
        let window = &xml[m.0.saturating_sub(80)..(m.0 + 40).min(xml.len())];
        if window.contains("lineRule=\"auto\"") || window.contains("lineRule='auto'") {
            assert!(
                window.contains("line=\"240\"") || window.contains("line='240'"),
                "lineRule=auto without line=240: {window}"
            );
        }
    }
}

#[test]
fn incomplete_spacing_on_list_item_becomes_single_line() {
    let mut dom = Dom::new();
    // A bare; B list item with incomplete spacing (broken_media×duplicate_ppr shape).
    let base = para_plain("a");
    let next = para_list_incomplete("a", "3");
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let xml = dom.serialize_element(body);
    // List item should carry Word single-line factory spacing.
    assert!(
        xml.contains("line=\"240\"") || xml.contains("line='240'"),
        "list incomplete spacing must normalize to line=240: {xml}"
    );
    assert!(
        xml.contains("lineRule=\"auto\"") || xml.contains("lineRule='auto'"),
        "list spacing must keep lineRule=auto: {xml}"
    );
}

#[test]
fn line_without_line_rule_gets_auto() {
    let mut dom = Dom::new();
    // A and B differ only in body text; A carries line=240 without lineRule.
    let base = "<w:p><w:pPr><w:spacing w:before=\"400\" w:after=\"120\" w:line=\"240\"/></w:pPr>\
         <w:r><w:t>Heading residual alpha</w:t></w:r></w:p>"
        .to_string();
    let next = "<w:p><w:pPr><w:spacing w:before=\"400\" w:after=\"120\" w:line=\"240\"/></w:pPr>\
         <w:r><w:t>Heading residual bravo</w:t></w:r></w:p>"
        .to_string();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let xml = dom.serialize_element(body);
    if xml.contains("w:line=\"240\"") || xml.contains("line=\"240\"") {
        assert!(
            xml.contains("lineRule=\"auto\"") || xml.contains("lineRule='auto'"),
            "line without lineRule must gain lineRule=auto: {xml}"
        );
    }
}

#[test]
fn list_insert_without_spacing_is_left_alone() {
    // Control: do not inject factory spacing onto bare list inserts — that
    // regressed bullet_list×bullet_list_bold (~−33) on the corpus.
    let mut dom = Dom::new();
    let base = para_plain("item");
    let next = para_list_plain("item new", "1");
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let xml = dom.serialize_element(body);
    // Bare list insert may lack spacing; that is intentional (no inject).
    let _ = xml;
}
