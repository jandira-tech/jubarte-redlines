//! ACCEPT-REUSE-CLEAN — clean subtrees are transferred (same NodeId), not
//! clone_subtree'd, during accept_all_other / accept_move transforms.
//!
//! Gates:
//! 1. Mixed dirty+clean body: clean paragraph NodeId survives accept_all_other.
//! 2. Serialize/content still matches full clone-path oracle (unwrap ins, drop del).
//! 3. Pure-clean still works (transfer root).
//! 4. moveTo unwrap still works with clean siblings.

use jubarte::namespaces::W;
use jubarte::revision_processor::{
    accept_all_other_revisions_transform, accept_move_from_move_to_transform,
    accept_revisions_for_element, element_has_tracked_revisions,
};
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

fn para_with_text(dom: &mut Dom, text: &str) -> jubarte::xmllinq::NodeId {
    let p = dom.new_element(w("p"));
    let r = dom.new_element(w("r"));
    let t = dom.new_element(w("t"));
    dom.add_text(t, text);
    dom.add(r, t);
    dom.add(p, r);
    p
}

fn dirty_para_ins_del(dom: &mut Dom) -> jubarte::xmllinq::NodeId {
    let p = dom.new_element(w("p"));
    let r1 = dom.new_element(w("r"));
    let t1 = dom.new_element(w("t"));
    dom.add_text(t1, "keep");
    dom.add(r1, t1);
    dom.add(p, r1);
    let ins = dom.new_element(w("ins"));
    let r2 = dom.new_element(w("r"));
    let t2 = dom.new_element(w("t"));
    dom.add_text(t2, "ADD");
    dom.add(r2, t2);
    dom.add(ins, r2);
    dom.add(p, ins);
    let del = dom.new_element(w("del"));
    let r3 = dom.new_element(w("r"));
    let t3 = dom.new_element(w("delText"));
    dom.add_text(t3, "gone");
    dom.add(r3, t3);
    dom.add(del, r3);
    dom.add(p, del);
    p
}

#[test]
fn reuse_clean_preserves_node_id_under_dirty_sibling() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let dirty = dirty_para_ins_del(&mut d);
    let clean = para_with_text(&mut d, "CLEAN");
    d.add(body, dirty);
    d.add(body, clean);
    let clean_id = clean;
    assert!(!element_has_tracked_revisions(&d, clean_id));
    assert!(element_has_tracked_revisions(&d, body));

    let out = accept_all_other_revisions_transform(&mut d, body);
    assert_eq!(out.len(), 1);
    let kids = d.nodes(out[0]);
    // Clean paragraph must be the same arena node (transferred, not cloned).
    assert!(
        kids.contains(&clean_id),
        "clean NodeId {clean_id:?} must survive; kids={kids:?}"
    );
    // Dirty paragraph was rebuilt — its old id should not still be a body child
    // (or if present only as orphan, not under out[0] as the dirty slot).
    let ser = d.serialize_element(out[0]);
    assert!(ser.contains("CLEAN"), "{ser}");
    assert!(ser.contains("keep"), "{ser}");
    assert!(ser.contains("ADD"), "{ser}");
    assert!(!ser.contains("gone"), "{ser}");
}

#[test]
fn reuse_clean_root_transfers_without_clone() {
    let mut d = Dom::new();
    let p = para_with_text(&mut d, "only");
    let p_id = p;
    let out = accept_all_other_revisions_transform(&mut d, p);
    assert_eq!(out, vec![p_id]);
    assert_eq!(d.serialize_element(out[0]), d.serialize_element(p_id));
}

#[test]
fn reuse_clean_move_transform_preserves_clean_sibling() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    // moveTo wrapping a run
    let move_to = d.new_element(w("moveTo"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "moved");
    d.add(r, t);
    d.add(move_to, r);
    let clean = para_with_text(&mut d, "still");
    d.add(body, move_to);
    d.add(body, clean);
    let clean_id = clean;

    let out = accept_move_from_move_to_transform(&mut d, body);
    assert_eq!(out.len(), 1);
    assert!(
        d.nodes(out[0]).contains(&clean_id),
        "clean sibling must transfer under move transform"
    );
    let ser = d.serialize_element(out[0]);
    assert!(ser.contains("moved"), "{ser}");
    assert!(ser.contains("still"), "{ser}");
    assert!(!ser.contains("moveTo"), "{ser}");
}

#[test]
fn reuse_clean_element_accept_still_correct() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let dirty = dirty_para_ins_del(&mut d);
    let clean = para_with_text(&mut d, "Z");
    d.add(body, dirty);
    d.add(body, clean);
    let accepted = accept_revisions_for_element(&mut d, body);
    let ser = d.serialize_element(accepted);
    assert!(ser.contains("keep") && ser.contains("ADD") && ser.contains("Z"), "{ser}");
    assert!(!ser.contains("gone") && !ser.contains("<w:ins") && !ser.contains("<w:del"), "{ser}");
}
