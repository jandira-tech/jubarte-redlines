//! RSID-INPLACE-01 — remove_rsid_transform mutates in place (no full-tree rebuild)
//! but must still strip all w:rsid* attrs and w:rsid elements with content intact.

use jubarte::markup_simplifier::remove_rsid_transform;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn inplace_preserves_node_id_and_strips_rsid() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    d.set_attribute_value(p, &w("rsidR"), Some("00AA11"));
    d.set_attribute_value(p, &w("rsidRDefault"), Some("00BB22"));
    let r = d.new_element(w("r"));
    d.set_attribute_value(r, &w("rsidRPr"), Some("00CC33"));
    let t = d.new_element(w("t"));
    d.add_text(t, "keep");
    d.add(r, t);
    d.add(p, r);

    let out = remove_rsid_transform(&mut d, p).unwrap();
    assert_eq!(out, p, "in-place path must return the same root NodeId");
    assert_eq!(d.value(out), "keep");
    for el in d.descendants_and_self(out, None) {
        for (name, _) in d.attributes(el) {
            assert!(
                !(name.namespace_name() == W::URI && name.local_name().starts_with("rsid")),
                "rsid attr leaked"
            );
        }
    }
}

#[test]
fn inplace_drops_rsid_element_children() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let ppr = d.new_element(w("pPr"));
    let rsid = d.new_element(w("rsid"));
    d.add(ppr, rsid);
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "x");
    d.add(r, t);
    d.add(p, ppr);
    d.add(p, r);

    let out = remove_rsid_transform(&mut d, p).unwrap();
    assert_eq!(out, p);
    assert_eq!(d.descendants(out, Some(&w("rsid"))).len(), 0);
    assert_eq!(d.value(out), "x");
    // pPr remains
    assert!(!d.descendants(out, Some(&w("pPr"))).is_empty());
}

#[test]
fn inplace_serialize_matches_clean_oracle() {
    // Build with rsids, strip, serialize; build without rsids, serialize — equal.
    let mut dirty = Dom::new();
    let p = dirty.new_element(w("p"));
    dirty.set_attribute_value(p, &w("rsidR"), Some("DEAD"));
    let r = dirty.new_element(w("r"));
    let t = dirty.new_element(w("t"));
    dirty.add_text(t, "hello");
    dirty.add(r, t);
    dirty.add(p, r);
    let stripped = remove_rsid_transform(&mut dirty, p).unwrap();
    let ser_stripped = dirty.serialize_element(stripped);

    let mut clean = Dom::new();
    let p2 = clean.new_element(w("p"));
    let r2 = clean.new_element(w("r"));
    let t2 = clean.new_element(w("t"));
    clean.add_text(t2, "hello");
    clean.add(r2, t2);
    clean.add(p2, r2);
    let ser_clean = clean.serialize_element(p2);

    assert_eq!(ser_stripped, ser_clean);
}
