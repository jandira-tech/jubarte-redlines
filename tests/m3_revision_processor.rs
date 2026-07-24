// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::namespaces::W;
use jubarte::revision_processor::{
    accept_revisions_document, accept_revisions_for_element, element_has_tracked_revisions,
};
use jubarte::xmllinq::{Dom, XName};

fn w(local: &str) -> XName {
    XName::get(local, W::URI)
}

/// Build `<w:p>` with: run "A", w:ins>run "B", w:del>run(delText "C").
/// Returns (dom, p).
fn build_revision_paragraph() -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));

    let r_a = d.new_element(w("r"));
    let t_a = d.new_element(w("t"));
    d.add_text(t_a, "A");
    d.add(r_a, t_a);
    d.add(p, r_a);

    let ins = d.new_element(w("ins"));
    let r_b = d.new_element(w("r"));
    let t_b = d.new_element(w("t"));
    d.add_text(t_b, "B");
    d.add(r_b, t_b);
    d.add(ins, r_b);
    d.add(p, ins);

    let del = d.new_element(w("del"));
    let r_c = d.new_element(w("r"));
    let dt_c = d.new_element(w("delText"));
    d.add_text(dt_c, "C");
    d.add(r_c, dt_c);
    d.add(del, r_c);
    d.add(p, del);

    (d, p)
}

#[test]
fn has_tracked_revisions_detects_ins_and_clean() {
    let (d, p) = build_revision_paragraph();
    assert!(element_has_tracked_revisions(&d, p));

    let mut d2 = Dom::new();
    let p2 = d2.new_element(w("p"));
    let r = d2.new_element(w("r"));
    let t = d2.new_element(w("t"));
    d2.add_text(t, "clean");
    d2.add(r, t);
    d2.add(p2, r);
    assert!(!element_has_tracked_revisions(&d2, p2));
}

#[test]
fn accept_unwraps_ins_and_drops_del() {
    let (mut d, p) = build_revision_paragraph();
    let accepted = accept_revisions_for_element(&mut d, p);

    // ins unwrapped → its run survives; del dropped entirely.
    assert!(d.descendants(accepted, Some(&w("ins"))).is_empty());
    assert!(d.descendants(accepted, Some(&w("del"))).is_empty());
    // surviving text is "A" (kept) + "B" (was inserted); "C" (deleted) gone.
    assert_eq!(d.value(accepted), "AB");
    // no tracked revisions remain
    assert!(!element_has_tracked_revisions(&d, accepted));
    // three runs collapsed to two (A, B)
    assert_eq!(d.elements(accepted, Some(&w("r"))).len(), 2);
}

#[test]
fn accept_removes_format_change_markers() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let ppr = d.new_element(w("pPr"));
    let ppr_change = d.new_element(w("pPrChange"));
    d.add(ppr, ppr_change);
    d.add(p, ppr);
    let r = d.new_element(w("r"));
    let rpr = d.new_element(w("rPr"));
    let rpr_change = d.new_element(w("rPrChange"));
    d.add(rpr, rpr_change);
    d.add(r, rpr);
    let t = d.new_element(w("t"));
    d.add_text(t, "x");
    d.add(r, t);
    d.add(p, r);

    let accepted = accept_revisions_for_element(&mut d, p);
    assert!(d.descendants(accepted, Some(&w("pPrChange"))).is_empty());
    assert!(d.descendants(accepted, Some(&w("rPrChange"))).is_empty());
    // pPr and rPr themselves are preserved
    assert!(!d.descendants(accepted, Some(&w("pPr"))).is_empty());
    assert_eq!(d.value(accepted), "x");
}

/// Regression test for the accept-both-inputs path in
/// `compare_bodies_faithful` (PR #13): a paragraph containing `w:ins` and
/// `w:del` is passed through `accept_revisions_document`, and we verify
/// the wrapper unwraps the ins (preserving its inserted run), drops the
/// del entirely, and yields an accepted tree with no tracked-revision
/// markers. This is wrapper-level coverage for `accept_revisions_document`
/// itself; the call-site behavior inside `compare_bodies_faithful` is
/// guarded by `compare_bodies_faithful_accepts_tracked_revisions_in_inputs`
/// in `m4_comparer.rs`.
#[test]
fn accept_revisions_document_unwraps_ins_and_drops_del() {
    let (mut d, p) = build_revision_paragraph();
    let accepted = accept_revisions_document(&mut d, p);

    assert!(d.descendants(accepted, Some(&w("ins"))).is_empty());
    assert!(d.descendants(accepted, Some(&w("del"))).is_empty());
    assert_eq!(d.value(accepted), "AB");
    assert!(!element_has_tracked_revisions(&d, accepted));
    assert_eq!(d.elements(accepted, Some(&w("r"))).len(), 2);
}

/// Idempotency guard for the accept-both-inputs call site in
/// `compare_bodies_faithful` (where the same body1/body2 are not currently
/// re-accepted, but downstream reuse must remain safe): running
/// `accept_revisions_document` twice on the same tree must produce the
/// same final content as running it once, and must not introduce phantom
/// tracked-revision markers.
///
/// The first-pass result (value, tracked-revision flag, run count) is
/// snapshotted before the second call so the equality check compares the
/// preserved first-pass state against the second-pass state — not the
/// same mutated tree seen twice.
#[test]
fn accept_revisions_document_is_idempotent() {
    let (mut d, p) = build_revision_paragraph();
    let once = accept_revisions_document(&mut d, p);
    let once_value = d.value(once);
    let once_has_revisions = element_has_tracked_revisions(&d, once);
    let once_run_count = d.elements(once, Some(&w("r"))).len();

    let twice = accept_revisions_document(&mut d, once);

    assert_eq!(once_value, d.value(twice));
    assert!(!once_has_revisions);
    assert!(!element_has_tracked_revisions(&d, twice));
    assert_eq!(once_run_count, d.elements(twice, Some(&w("r"))).len());
    assert_eq!(once_run_count, 2);
}
