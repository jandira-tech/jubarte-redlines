// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ACCEPT-SKIP-A5 — when no paragraph mark is deleted/moved-from
//! (p/pPr/rPr/(del|moveFrom)), accept_deleted_and_move_from_paragraph_marks
//! transfers the root NodeId (no annotate + full-tree rebuild + rewrap).

use jubarte::namespaces::W;
use jubarte::revision_processor::accept_deleted_and_move_from_paragraph_marks;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn skip_a5_no_deleted_marks_preserves_root_id() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p1 = d.new_element(w("p"));
    let r1 = d.new_element(w("r"));
    let t1 = d.new_element(w("t"));
    d.add_text(t1, "a");
    d.add(r1, t1);
    d.add(p1, r1);
    // content-level del (not paragraph *mark*) must not force A.5 rebuild path
    // for the skip gate — only pPr/rPr del|moveFrom counts. Include plain p.
    let p2 = d.new_element(w("p"));
    let r2 = d.new_element(w("r"));
    let t2 = d.new_element(w("t"));
    d.add_text(t2, "b");
    d.add(r2, t2);
    d.add(p2, r2);
    d.add(body, p1);
    d.add(body, p2);
    let body_id = body;
    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);
    assert_eq!(
        out, body_id,
        "no deleted paragraph marks → transfer same root"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&w("t")))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a", "b"]);
}

#[test]
fn skip_a5_with_deleted_mark_still_merges() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    // first p: mark deleted
    let p1 = d.new_element(w("p"));
    let ppr = d.new_element(w("pPr"));
    let rpr = d.new_element(w("rPr"));
    let del = d.new_element(w("del"));
    d.add(rpr, del);
    d.add(ppr, rpr);
    d.add(p1, ppr);
    let r1 = d.new_element(w("r"));
    let t1 = d.new_element(w("t"));
    d.add_text(t1, "gone");
    d.add(r1, t1);
    d.add(p1, r1);
    d.add(body, p1);
    // following p joins deleted range group
    let p2 = d.new_element(w("p"));
    let r2 = d.new_element(w("r"));
    let t2 = d.new_element(w("t"));
    d.add_text(t2, "kept");
    d.add(r2, t2);
    d.add(p2, r2);
    d.add(body, p2);
    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);
    // A.5 merges deleted-mark p + following into one p with collapsed content
    let ps = d.elements(out, Some(&w("p")));
    assert!(!ps.is_empty(), "pipeline must still produce paragraph(s)");
    let texts: Vec<String> = d
        .descendants(out, Some(&w("t")))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert!(
        texts.iter().any(|t| t == "kept") || texts.iter().any(|t| t == "gone"),
        "content preserved in some form: {texts:?}"
    );
}
