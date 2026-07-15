//! ACCEPT-SKIP-01 — when a subtree has no tracked-revision elements, the
//! accept pipeline must still strip rsids / empty-numPr / PT.UniqueId, but
//! must NOT run the multi-pass full-tree rebuild transforms (move / all-other
//! / field fixup / deleted-cells / merge).
//!
//! Gates:
//! 1. Clean tree: accept result equals remove_rsid-only content (serialize).
//! 2. Tree with w:ins/w:del: accept still unwraps ins and drops del.
//! 3. element_has_tracked_revisions is exact for clean vs dirty.

use jubarte::markup_simplifier::remove_rsid_transform;
use jubarte::namespaces::W;
use jubarte::revision_processor::{
    accept_revisions_document, accept_revisions_for_element, element_has_tracked_revisions,
};
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

fn clean_paragraph(dom: &mut Dom) -> jubarte::xmllinq::NodeId {
    let p = dom.new_element(w("p"));
    // Word-style rsid attrs — accept must strip them.
    dom.set_attribute_value(p, &w("rsidR"), Some("00AABBCC"));
    dom.set_attribute_value(p, &w("rsidRDefault"), Some("00AABBCC"));
    let r = dom.new_element(w("r"));
    let t = dom.new_element(w("t"));
    dom.add_text(t, "clean");
    dom.add(r, t);
    dom.add(p, r);
    p
}

fn dirty_paragraph(dom: &mut Dom) -> jubarte::xmllinq::NodeId {
    let p = dom.new_element(w("p"));
    // kept run
    let r1 = dom.new_element(w("r"));
    let t1 = dom.new_element(w("t"));
    dom.add_text(t1, "A");
    dom.add(r1, t1);
    dom.add(p, r1);
    // inserted
    let ins = dom.new_element(w("ins"));
    let r2 = dom.new_element(w("r"));
    let t2 = dom.new_element(w("t"));
    dom.add_text(t2, "B");
    dom.add(r2, t2);
    dom.add(ins, r2);
    dom.add(p, ins);
    // deleted
    let del = dom.new_element(w("del"));
    let r3 = dom.new_element(w("r"));
    let t3 = dom.new_element(w("delText"));
    dom.add_text(t3, "C");
    dom.add(r3, t3);
    dom.add(del, r3);
    dom.add(p, del);
    p
}

#[test]
fn scan_detects_clean_and_dirty() {
    let mut d = Dom::new();
    let clean = clean_paragraph(&mut d);
    assert!(!element_has_tracked_revisions(&d, clean));
    let dirty = dirty_paragraph(&mut d);
    assert!(element_has_tracked_revisions(&d, dirty));
}

#[test]
fn clean_element_accept_matches_rsid_strip_only() {
    let mut d = Dom::new();
    let p = clean_paragraph(&mut d);
    let rsid_only = remove_rsid_transform(&mut d, p).unwrap();
    let rsid_ser = d.serialize_element(rsid_only);

    let mut d2 = Dom::new();
    let p2 = clean_paragraph(&mut d2);
    let accepted = accept_revisions_for_element(&mut d2, p2);
    let acc_ser = d2.serialize_element(accepted);

    assert_eq!(
        acc_ser, rsid_ser,
        "clean accept must equal remove_rsid-only (no extra rebuild semantics)"
    );
    assert!(!acc_ser.contains("rsidR"), "rsids must be stripped");
    assert_eq!(d2.value(accepted), "clean");
}

#[test]
fn clean_document_accept_strips_rsid_keeps_text() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = clean_paragraph(&mut d);
    d.add(body, p);
    let out = accept_revisions_document(&mut d, body);
    assert!(!element_has_tracked_revisions(&d, out));
    assert_eq!(d.value(out), "clean");
    // no rsid attrs remain on descendants
    for id in d.descendants_and_self(out, None) {
        for (n, _) in d.attributes(id) {
            assert!(
                !n.local_name().starts_with("rsid"),
                "residual rsid attr {}",
                n.local_name()
            );
        }
    }
}

#[test]
fn dirty_accept_still_unwraps_ins_drops_del() {
    let mut d = Dom::new();
    let p = dirty_paragraph(&mut d);
    let out = accept_revisions_for_element(&mut d, p);
    assert!(d.descendants(out, Some(&w("ins"))).is_empty());
    assert!(d.descendants(out, Some(&w("del"))).is_empty());
    assert_eq!(d.value(out), "AB");
    assert!(!element_has_tracked_revisions(&d, out));
}

#[test]
fn dirty_document_accept_still_works() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = dirty_paragraph(&mut d);
    d.add(body, p);
    let out = accept_revisions_document(&mut d, body);
    assert_eq!(d.value(out), "AB");
    assert!(!element_has_tracked_revisions(&d, out));
}
