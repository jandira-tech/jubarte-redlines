// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! REJECT-SKIP-01 — clean trees skip the full reject rebuild chain.
//!
//! When there are no tracked-revision elements, reject must equal remove_rsid
//! (same serialized content). Dirty trees still invert ins/del correctly.

use jubarte::markup_simplifier::remove_rsid_transform;
use jubarte::namespaces::W;
use jubarte::revision_processor::{element_has_tracked_revisions, reject_revisions_document};
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

fn clean_para(dom: &mut Dom) -> jubarte::xmllinq::NodeId {
    let p = dom.new_element(w("p"));
    dom.set_attribute_value(p, &w("rsidR"), Some("00AA"));
    let r = dom.new_element(w("r"));
    let t = dom.new_element(w("t"));
    dom.add_text(t, "clean");
    dom.add(r, t);
    dom.add(p, r);
    p
}

fn dirty_para(dom: &mut Dom) -> jubarte::xmllinq::NodeId {
    // Original text "A", with insertion "B" — reject should yield "A" only.
    let p = dom.new_element(w("p"));
    let r1 = dom.new_element(w("r"));
    let t1 = dom.new_element(w("t"));
    dom.add_text(t1, "A");
    dom.add(r1, t1);
    dom.add(p, r1);
    let ins = dom.new_element(w("ins"));
    let r2 = dom.new_element(w("r"));
    let t2 = dom.new_element(w("t"));
    dom.add_text(t2, "B");
    dom.add(r2, t2);
    dom.add(ins, r2);
    dom.add(p, ins);
    p
}

#[test]
fn clean_reject_matches_rsid_strip_serialize() {
    let mut d = Dom::new();
    let p = clean_para(&mut d);
    let rsid = remove_rsid_transform(&mut d, p).unwrap();
    let ser_rsid = d.serialize_element(rsid);

    let mut d2 = Dom::new();
    let p2 = clean_para(&mut d2);
    let rej = reject_revisions_document(&mut d2, p2);
    let ser_rej = d2.serialize_element(rej);

    assert_eq!(ser_rej, ser_rsid);
    assert_eq!(d2.value(rej), "clean");
    assert!(!element_has_tracked_revisions(&d2, rej));
}

#[test]
fn dirty_reject_drops_insertions() {
    let mut d = Dom::new();
    let p = dirty_para(&mut d);
    assert!(element_has_tracked_revisions(&d, p));
    let rej = reject_revisions_document(&mut d, p);
    // reject undoes insert → only original "A"
    assert_eq!(d.value(rej), "A");
    assert!(!element_has_tracked_revisions(&d, rej));
}
