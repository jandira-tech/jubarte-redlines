// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M36 — pt:PreDelete-stamped content must never correlate Equal with
//! unstamped content (M-MOVE S1, fresh-p4 forensics).
//!
//! Word-mode flattening resurfaces doc A's pre-existing tracked deletions as
//! live stamped text. When doc B carries the IDENTICAL text live, the stamped
//! A content used to hash equal to B's runs at both block and atom level, so
//! LCS correlated them Equal — annihilating BOTH the deletion history and
//! B's real insertions (our fresh-p4 output: 5 ins/5 del vs GT's full
//! del+ins set; 11 rendered pages vs GT's 12).
//!
//! The fix salts every SHA1 input (block clone hash + atom hash) with the
//! PreDelete stamp so stamped content can only match stamped content.
//! Scope guard: pt:PreIns is NOT salted — carried insertions REQUIRE Equal
//! correlation with B's live copy (D1 semantics, m32 w18).

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::revision_processor::accept_revisions_document;
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

const A: &str = "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>\
     <w:p><w:del w:id=\"7\" w:author=\"Online User\" w:date=\"2025-05-14T00:00:00Z\">\
     <w:r><w:delText>the removed capability matrix</w:delText></w:r></w:del></w:p>";

const B: &str = "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>\
     <w:p><w:r><w:t>the removed capability matrix</w:t></w:r></w:p>\
     <w:p><w:r><w:t>brand new evidence base</w:t></w:r></w:p>";

/// A's pre-existing deletion text also lives (unmarked) in B: the deletion
/// history must survive as struck-through w:del with the original author,
/// B's live copy must be present too, and accept(redline) ≡ accept(B).
#[test]
fn s1a_predel_never_correlates_equal_with_live_b_text() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, A);
    let (r2, b2) = doc_body(&mut dom, B);
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);

    // deletion history visible: a w:del whose delText covers the text,
    // attributed to the ORIGINAL author
    let del = dom
        .descendants(out, Some(&W::del()))
        .into_iter()
        .find(|&d| {
            dom.descendants(d, Some(&W::name("delText")))
                .iter()
                .map(|&t| dom.value(t))
                .collect::<String>()
                .contains("the removed capability matrix")
        })
        .unwrap_or_else(|| panic!("pre-deletion history vanished (correlated Equal): {x}"));
    assert_eq!(
        dom.attribute(del, &W::author()),
        Some("Online User"),
        "original author preserved: {x}"
    );

    // B's live copy is ALSO present (as live or inserted w:t)
    let live: String = dom
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(
        live.contains("the removed capability matrix"),
        "B's live copy present: {x}"
    );

    // reconstruction invariant: accept(redline) ≡ accept(B) — the text
    // survives exactly once, plus B's new paragraph. Assert the FULL accepted
    // text, not just substrings: a substring check would still pass with extra
    // or reordered content, masking a reconstruction regression.
    let accepted = accept_revisions_document(&mut dom, out);
    let atext: String = dom
        .descendants(accepted, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert_eq!(
        atext, "shared heading textthe removed capability matrixbrand new evidence base",
        "accepted redline text should match accepted B exactly"
    );
}

/// Faithful preset guard: same inputs keep PowerTools' accept-first behavior
/// (A's deletion is accepted away pre-diff; no flatten, no stamp, so the
/// salt has no effect — pinned so the fix cannot drift the faithful path).
#[test]
fn s1b_faithful_preset_unchanged() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, A);
    let (r2, b2) = doc_body(&mut dom, B);
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);

    // accept-first: no pre-deletion history in the output…
    assert!(
        !x.contains("delText"),
        "faithful keeps accept-first (no carried deletion): {x}"
    );
    // …A-accepted lacks the matrix text, so B's copy comes out INSERTED,
    // along with the genuinely new paragraph
    let mut ins = String::new();
    for i in dom.descendants(out, Some(&W::ins())) {
        for t in dom.descendants(i, Some(&W::t())) {
            ins.push_str(&dom.value(t));
        }
    }
    assert!(
        ins.contains("the removed capability matrix"),
        "faithful re-inserts the text A had deleted: {x}"
    );
    assert!(ins.contains("brand new evidence base"), "{x}");
}
