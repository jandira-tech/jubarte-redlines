//! Word Compare pairs equal-count pure-paragraph docs positionally
//! (heading_2_style × heading_3_center_italic: 3 mixed paras, not 4
//! cross-stitched). Flattening into one word-LCS window lets shared tokens
//! ("Heading") bridge the wrong paragraphs.

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
fn three_vs_three_heading_style_stays_three_mixed() {
    let mut dom = Dom::new();
    // Shared "Heading" tokens would otherwise cross-stitch p1↔p2 under flat LCS.
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
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert_eq!(
        kids.len(),
        3,
        "Word: 3 positionally mixed paras, not 4 cross-stitched; kids={}",
        kids.len()
    );
    for (i, &k) in kids.iter().enumerate() {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        assert!(
            has_ins && has_del,
            "para {i} must be mixed ins+del (positional 1:1); ser={}",
            dom.serialize_element(k)
        );
    }
    // p0 should still show the classic "2 Style" vs "3 Center Italic" split shape
    let ser0 = dom.serialize_element(kids[0]);
    assert!(
        ser0.contains("3 Center Italic") || ser0.contains("Heading"),
        "p0 carries next heading text: {ser0}"
    );
    assert!(
        ser0.contains("2 Style") || ser0.contains("delText"),
        "p0 carries base '2 Style' del: {ser0}"
    );
}

#[test]
fn numbered_list_role_shift_does_not_force_positional_zip() {
    // Equal count (5 vs 5) but roles shift: Demo+4 items vs Demo+intro+3 items.
    // Diagonal is NOT dominant — flat LCS must win (forced zip regressed ~7 pts).
    let mut dom = Dom::new();
    let base = [
        para("Numbered List Demo"),
        para("First item"),
        para("Second item"),
        para("Third item"),
        para("Fourth item"),
    ]
    .concat();
    let next = [
        para("Numbered List Italic Demo"),
        para("This document shows numbered lists with italic formatting:"),
        para("First italic numbered item"),
        para("Second italic numbered item"),
        para("Third italic numbered item"),
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
    // With zip: all 5 would be mixed. Without zip, at least one pure-ins or
    // pure-del appears (role-shift leftover), or para count ≠ 5 mixed.
    let mut n_mixed = 0;
    let mut n_pure = 0;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        match (has_ins, has_del) {
            (true, true) => n_mixed += 1,
            (true, false) | (false, true) => n_pure += 1,
            _ => {}
        }
    }
    assert!(
        n_pure > 0 || kids.len() != 5 || n_mixed < 5,
        "must not force 5 mixed positional pairs on role-shifted lists; kids={} mixed={} pure={}",
        kids.len(),
        n_mixed,
        n_pure
    );
}
