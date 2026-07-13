//! Word/jubarte: N inserted paragraphs + exactly one deleted paragraph folds
//! the deleted body into the last inserted paragraph (mixed I+D).
//! Evidence: single_paragraph × small_font_size_demo (JS 100, ours ~68 with
//! separate trailing pure-del para).

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
fn three_ins_one_del_folds_into_last_ins() {
    let mut dom = Dom::new();
    // base: single unrelated paragraph
    let base = para("Walking on imported air");
    // next: three small-font demo paragraphs (no shared text)
    let next = [
        para("Small Font Size Demo"),
        para("This document demonstrates very small font size of 8pt."),
        para("Small fonts are used in footnotes and disclaimers."),
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
        "Word/jubarte: 3 paras (last is mixed), not 4 with trailing pure-del"
    );
    // last para must contain both ins and del
    let last = kids[2];
    let has_ins = !dom.elements(last, Some(&W::ins())).is_empty();
    let has_del = !dom.elements(last, Some(&W::del())).is_empty();
    assert!(has_ins && has_del, "last para must be mixed ins+del");
    let ser = dom.serialize_element(last);
    assert!(
        ser.contains("Walking on imported air") || ser.contains("delText"),
        "deleted base text in last para: {ser}"
    );
    assert!(
        ser.contains("Small fonts") || ser.contains("disclaimers"),
        "inserted last-line text in last para: {ser}"
    );
    // Word: mixed last para has no pPr/rPr ins|del mark (only body ins+del runs).
    assert!(
        !para_mark_revision_present(&dom, last),
        "Word: mixed folded last para has no paragraph-mark revision: {ser}"
    );
}

fn para_mark_revision_present(dom: &Dom, p: NodeId) -> bool {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.element(rpr, &W::ins()).is_some() || dom.element(rpr, &W::del()).is_some()
        })
}

#[test]
fn multi_del_boundary_folds_last_ins_first_del() {
    // M90: 2 ins + 3 del → fold last pure-I with first pure-D (Word file_38/62/11).
    // Prior green-underline GT expected no fold; real Word Compare folds the
    // boundary pair on stamped residual demos. Use fully disjoint words so LCS
    // does not anchor mid-token (fold is merge_replaced boundary, not LCS).
    let mut dom = Dom::new();
    let base = [
        para("Alpha one unique"),
        para("Bravo two unique"),
        para("Charlie three unique"),
    ]
    .concat();
    let next = [para("Xray insert only"), para("Yankee insert only")].concat();
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
    // Classify: pure I / pure D / mixed
    let mut n_i = 0;
    let mut n_d = 0;
    let mut n_m = 0;
    for &k in &kids {
        let has_ins = !dom.elements(k, Some(&W::ins())).is_empty();
        let has_del = !dom.elements(k, Some(&W::del())).is_empty();
        match (has_ins, has_del) {
            (true, true) => n_m += 1,
            (true, false) => n_i += 1,
            (false, true) => n_d += 1,
            _ => {}
        }
    }
    assert_eq!(
        (n_i, n_d, n_m),
        (1, 2, 1),
        "2I+3D → 1 pure-I + 1 mixed + 2 pure-D; kids={} i={} d={} m={}",
        kids.len(),
        n_i,
        n_d,
        n_m
    );
}
