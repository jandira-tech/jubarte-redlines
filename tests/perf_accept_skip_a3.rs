//! ACCEPT-SKIP-A3 — when no w:moveFromRangeStart in the tree,
//! accept_paragraph_end_tags_in_move_from_transform transfers the root NodeId
//! (no full-tree identity rebuild).

use jubarte::namespaces::W;
use jubarte::revision_processor::accept_paragraph_end_tags_in_move_from_transform;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn skip_a3_no_mfrs_preserves_root_id() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p1 = d.new_element(w("p"));
    let r1 = d.new_element(w("r"));
    let t1 = d.new_element(w("t"));
    d.add_text(t1, "a");
    d.add(r1, t1);
    d.add(p1, r1);
    let p2 = d.new_element(w("p"));
    let r2 = d.new_element(w("r"));
    let t2 = d.new_element(w("t"));
    d.add_text(t2, "b");
    d.add(r2, t2);
    d.add(p2, r2);
    d.add(body, p1);
    d.add(body, p2);
    let body_id = body;
    let out = accept_paragraph_end_tags_in_move_from_transform(&mut d, body);
    assert_eq!(out, body_id, "no moveFromRangeStart → transfer same root");
    assert_eq!(d.elements(out, Some(&w("p"))).len(), 2);
    let texts: Vec<String> = d
        .descendants(out, Some(&w("t")))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a", "b"]);
}

#[test]
fn skip_a3_with_mfrs_still_preserves_structure() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p1 = d.new_element(w("p"));
    let mfrs = d.new_element(w("moveFromRangeStart"));
    d.set_attribute_value(mfrs, &w("id"), Some("1"));
    d.set_attribute_value(mfrs, &w("name"), Some("m1"));
    d.add(p1, mfrs);
    let r1 = d.new_element(w("r"));
    let t1 = d.new_element(w("t"));
    d.add_text(t1, "first");
    d.add(r1, t1);
    d.add(p1, r1);
    d.add(body, p1);
    let p2 = d.new_element(w("p"));
    let r2 = d.new_element(w("r"));
    let t2 = d.new_element(w("t"));
    d.add_text(t2, "second");
    d.add(r2, t2);
    d.add(p2, r2);
    d.add(body, p2);
    let out = accept_paragraph_end_tags_in_move_from_transform(&mut d, body);
    let ps = d.elements(out, Some(&w("p")));
    assert_eq!(ps.len(), 2, "faithful: no coalescing");
    let texts: Vec<String> = d
        .descendants(out, Some(&w("t")))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["first", "second"]);
    assert_eq!(
        d.descendants(out, Some(&w("moveFromRangeStart"))).len(),
        1,
        "marker preserved"
    );
}
