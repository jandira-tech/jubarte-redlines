// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M-B — reference-driven footnote/endnote processing.
//!
//! B.1: `NotesContext` plumbing — `compare_bodies_faithful_with_notes` is a
//! sibling entry point taking the four notes-part roots; the existing
//! `compare_bodies_faithful` delegates with `None` and both are byte-identical
//! on note-free documents.

use jubarte::comparer::{
    NotesContext, WmlComparerSettings, compare_bodies_faithful, compare_bodies_faithful_with_notes,
};
use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::{Dom, NodeId};

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Test Author".to_string(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".to_string(),
        ..WmlComparerSettings::default()
    }
}

/// Parse a `<w:document><w:body>…</w:body></w:document>` and return
/// (document root, body).
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

/// B.1 — on note-free inputs the two entry points produce byte-identical
/// results (`with_notes(None)` ≡ the legacy signature).
#[test]
fn b1_entry_points_identical_on_note_free_docs() {
    let s = settings();

    let mut dom1 = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom1,
        "<w:p><w:r><w:t>alpha beta gamma</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom1,
        "<w:p><w:r><w:t>alpha BETA gamma</w:t></w:r></w:p>",
    );
    let out1 = compare_bodies_faithful(&mut dom1, r1, r2, b1, b2, &s);
    let x1 = dom1.serialize_element(out1);

    let mut dom2 = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom2,
        "<w:p><w:r><w:t>alpha beta gamma</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom2,
        "<w:p><w:r><w:t>alpha BETA gamma</w:t></w:r></w:p>",
    );
    let mut ctx = NotesContext::default();
    let out2 = compare_bodies_faithful_with_notes(&mut dom2, r1, r2, b1, b2, &s, Some(&mut ctx));
    let x2 = dom2.serialize_element(out2);

    assert_eq!(x1, x2, "entry points diverge on note-free input");
}

/// Parse a `w:footnotes` part with realistic definitions (marker run +
/// text run) and return its root.
fn footnotes_root(dom: &mut Dom, defs: &[(&str, &str)]) -> NodeId {
    let mut inner = String::new();
    for (id, text) in defs {
        inner.push_str(&format!(
            "<w:footnote w:id=\"{id}\"><w:p>\
             <w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr>\
             <w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\"/></w:rPr><w:footnoteRef/></w:r>\
             <w:r><w:t xml:space=\"preserve\"> {text}</w:t></w:r>\
             </w:p></w:footnote>"
        ));
    }
    let xml = format!(
        "<w:footnotes xmlns:w=\"{w}\">{inner}</w:footnotes>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    dom.root(d).unwrap()
}

/// A separators-only `w:footnotes` root — the withRevisions part every real
/// output package carries (C.2 guarantees it exists).
fn separators_only_footnotes(dom: &mut Dom) -> NodeId {
    let xml = format!(
        "<w:footnotes xmlns:w=\"{w}\">\
         <w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>\
         <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>\
         </w:footnotes>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    dom.root(d).unwrap()
}

/// Find the note definition with the given id.
fn def_by_id(dom: &Dom, root: NodeId, id: &str) -> NodeId {
    dom.elements(root, Some(&W::footnote()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some(id))
        .unwrap_or_else(|| panic!("def {id} missing"))
}

/// Runs under `el` as (pt:Status, concatenated text) pairs. Produce writes
/// the status attribute on the leaf content element (`w:t`/`w:delText`), so
/// read it from the run's descendants.
fn run_statuses(dom: &Dom, el: NodeId) -> Vec<(String, String)> {
    dom.descendants(el, Some(&W::r()))
        .into_iter()
        .map(|r| {
            let status = dom
                .descendants(r, None)
                .into_iter()
                .find_map(|d| dom.attribute(d, &PT::status()))
                .unwrap_or("")
                .to_string();
            (status, dom.value(r))
        })
        .collect()
}

/// B.2a — Equal reference, edited definition: the nested mini-compare writes
/// pt:Status-marked mixed content into the AFTER definition, and the
/// reference-marker run survives (plan assert: "definition holds pt:Status-
/// marked mixed content + a footnoteRef run").
#[test]
fn b2a_equal_reference_definition_diffed_into_after() {
    let s = settings();
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r><w:r><w:footnoteReference w:id=\"1001\"/></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r><w:r><w:footnoteReference w:id=\"2001\"/></w:r></w:p>",
    );
    let fn_before = footnotes_root(&mut dom, &[("1001", "shared original tail")]);
    let fn_after = footnotes_root(&mut dom, &[("2001", "shared edited tail")]);
    let fn_wr = separators_only_footnotes(&mut dom);
    let mut ctx = NotesContext {
        fn_before: Some(fn_before),
        fn_after: Some(fn_after),
        en_before: None,
        en_after: None,
        fn_with_revisions: Some(fn_wr),
        ..Default::default()
    };
    compare_bodies_faithful_with_notes(&mut dom, r1, r2, b1, b2, &s, Some(&mut ctx));

    // the BEFORE definition is untouched by the Equal branch
    let before_def = def_by_id(&dom, fn_before, "1001");
    assert!(
        run_statuses(&dom, before_def)
            .iter()
            .all(|(st, _)| st.is_empty()),
        "before def must stay unprocessed"
    );

    // the AFTER definition holds the nested redline
    let after_def = def_by_id(&dom, fn_after, "2001");
    let runs = run_statuses(&dom, after_def);
    assert!(
        runs.iter()
            .any(|(st, tx)| st == "Deleted" && tx.contains("original")),
        "old text marked Deleted, got {runs:?}"
    );
    assert!(
        runs.iter()
            .any(|(st, tx)| st == "Inserted" && tx.contains("edited")),
        "new text marked Inserted, got {runs:?}"
    );
    assert!(
        !dom.descendants(after_def, Some(&W::name("footnoteRef")))
            .is_empty(),
        "reference-marker run guaranteed"
    );
}

/// B.2b — Inserted reference: the after-definition content is re-emitted
/// all-Inserted (every text-bearing run pt:Status="Inserted").
#[test]
fn b2b_inserted_reference_definition_all_inserted() {
    let s = settings();
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, "<w:p><w:r><w:t>hello</w:t></w:r></w:p>");
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r></w:p>\
         <w:p><w:r><w:t>brand new para</w:t></w:r><w:r><w:footnoteReference w:id=\"2001\"/></w:r></w:p>",
    );
    let fn_after = footnotes_root(&mut dom, &[("2001", "fresh note text")]);
    let fn_wr = separators_only_footnotes(&mut dom);
    let mut ctx = NotesContext {
        fn_before: None,
        fn_after: Some(fn_after),
        en_before: None,
        en_after: None,
        fn_with_revisions: Some(fn_wr),
        ..Default::default()
    };
    compare_bodies_faithful_with_notes(&mut dom, r1, r2, b1, b2, &s, Some(&mut ctx));

    let def = def_by_id(&dom, fn_after, "2001");
    let runs = run_statuses(&dom, def);
    let texty: Vec<&(String, String)> = runs.iter().filter(|(_, tx)| !tx.is_empty()).collect();
    assert!(!texty.is_empty(), "definition re-emitted, got {runs:?}");
    assert!(
        texty.iter().all(|(st, _)| st == "Inserted"),
        "all content Inserted, got {runs:?}"
    );
    assert!(
        runs.iter().any(|(_, tx)| tx.contains("fresh")),
        "definition text kept, got {runs:?}"
    );
    assert!(
        !dom.descendants(def, Some(&W::name("footnoteRef")))
            .is_empty(),
        "reference-marker run guaranteed"
    );
}

/// B.3 — RectifyFootnoteEndnoteIds runs inside the pipeline after B.2: the
/// withRevisions notes part ends up holding ONLY the separators plus the
/// referenced definitions renumbered 1..n in reference document order, with
/// REAL `w:ins`/`w:del` markup (pt:Status finalized); the body references are
/// renumbered to match.
#[test]
fn b3_rectify_renumbers_and_finalizes_notes_part() {
    let s = settings();
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r><w:r><w:footnoteReference w:id=\"1001\"/></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r><w:r><w:footnoteReference w:id=\"2001\"/></w:r></w:p>\
         <w:p><w:r><w:t>brand new</w:t></w:r><w:r><w:footnoteReference w:id=\"2002\"/></w:r></w:p>",
    );
    let fn_before = footnotes_root(&mut dom, &[("1001", "shared original tail")]);
    let fn_after = footnotes_root(
        &mut dom,
        &[("2001", "shared edited tail"), ("2002", "fresh note")],
    );
    // withRevisions part: separators + a stale definition that must be stripped
    let wr_xml = format!(
        "<w:footnotes xmlns:w=\"{w}\">\
         <w:footnote w:type=\"separator\" w:id=\"-1\"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>\
         <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>\
         <w:footnote w:id=\"999\"><w:p><w:r><w:t>stale</w:t></w:r></w:p></w:footnote>\
         </w:footnotes>",
        w = W::URI
    );
    let wr_doc = dom.parse_xdocument(&wr_xml);
    let fn_wr = dom.root(wr_doc).unwrap();

    let mut ctx = NotesContext {
        fn_before: Some(fn_before),
        fn_after: Some(fn_after),
        en_before: None,
        en_after: None,
        fn_with_revisions: Some(fn_wr),
        en_with_revisions: None,
    };
    let out = compare_bodies_faithful_with_notes(&mut dom, r1, r2, b1, b2, &s, Some(&mut ctx));

    // body references renumbered 1..n in document order
    let refs: Vec<String> = dom
        .descendants(out, Some(&W::name("footnoteReference")))
        .into_iter()
        .filter_map(|r| dom.attribute(r, &W::id()).map(str::to_string))
        .collect();
    assert_eq!(refs, vec!["1", "2"], "refs renumbered in doc order");

    // withRevisions part: separators + defs 1 and 2, stale def gone
    let ids: Vec<String> = dom
        .elements(fn_wr, Some(&W::footnote()))
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
        .collect();
    assert_eq!(
        ids,
        vec!["-1", "0", "1", "2"],
        "separators + renumbered defs"
    );

    // finalized markup: def 1 carries REAL w:del + w:ins (not pt:Status)
    let def1 = def_by_id(&dom, fn_wr, "1");
    let dx = dom.serialize_element(def1);
    assert!(
        !dom.descendants(def1, Some(&W::del())).is_empty(),
        "real w:del in def 1: {dx}"
    );
    assert!(
        !dom.descendants(def1, Some(&W::ins())).is_empty(),
        "real w:ins in def 1: {dx}"
    );
    // NOTE: C# keeps the (mc:Ignorable-declared) pt:Status attributes in the
    // notes part — its finalization (:3433) has no pt-attr strip — so we
    // don't assert their absence, only the real revision wrappers above.

    // def 2 (inserted note) is fully inside w:ins
    let def2 = def_by_id(&dom, fn_wr, "2");
    let d2x = dom.serialize_element(def2);
    assert!(
        !dom.descendants(def2, Some(&W::ins())).is_empty(),
        "inserted def wrapped in w:ins: {d2x}"
    );
    assert!(d2x.contains("fresh"), "inserted def text kept: {d2x}");
}

/// B.2c — Deleted reference: the BEFORE definition content is re-emitted
/// all-Deleted into the before definition (the C# branch keys the before-part
/// lookup by the misnamed `afterId` local — correct for Deleted — and leaves
/// the footnote partToUseBefore null, inert with the null rel resolver).
#[test]
fn b2c_deleted_reference_definition_all_deleted() {
    let s = settings();
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>hello</w:t></w:r></w:p>\
         <w:p><w:r><w:t>old para</w:t></w:r><w:r><w:footnoteReference w:id=\"1001\"/></w:r></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>hello</w:t></w:r></w:p>");
    let fn_before = footnotes_root(&mut dom, &[("1001", "vanishing note text")]);
    let fn_wr = separators_only_footnotes(&mut dom);
    let mut ctx = NotesContext {
        fn_before: Some(fn_before),
        fn_after: None,
        en_before: None,
        en_after: None,
        fn_with_revisions: Some(fn_wr),
        ..Default::default()
    };
    compare_bodies_faithful_with_notes(&mut dom, r1, r2, b1, b2, &s, Some(&mut ctx));

    let def = def_by_id(&dom, fn_before, "1001");
    let runs = run_statuses(&dom, def);
    let texty: Vec<&(String, String)> = runs.iter().filter(|(_, tx)| !tx.is_empty()).collect();
    assert!(!texty.is_empty(), "definition re-emitted, got {runs:?}");
    assert!(
        texty.iter().all(|(st, _)| st == "Deleted"),
        "all content Deleted, got {runs:?}"
    );
    assert!(
        runs.iter().any(|(_, tx)| tx.contains("vanishing")),
        "definition text kept, got {runs:?}"
    );
    assert!(
        !dom.descendants(def, Some(&W::name("footnoteRef")))
            .is_empty(),
        "reference-marker run guaranteed"
    );
}
