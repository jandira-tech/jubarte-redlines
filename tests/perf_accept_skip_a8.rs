//! ACCEPT-SKIP-A8 — when no mergeable adjacent revision-bearing tables exist,
//! merge_adjacent_tables must transfer the root NodeId.

use jubarte::namespaces::W;
use jubarte::revision_processor::merge_adjacent_tables_transform;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn skip_a8_no_tables_preserves_root() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = d.new_element(w("p"));
    d.add(body, p);
    let id = body;
    assert_eq!(merge_adjacent_tables_transform(&mut d, body), id);
}

#[test]
fn skip_a8_single_clean_table_preserves_root() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let tbl = d.new_element(w("tbl"));
    let tr = d.new_element(w("tr"));
    let tc = d.new_element(w("tc"));
    d.add(tr, tc);
    d.add(tbl, tr);
    d.add(body, tbl);
    let id = body;
    assert_eq!(merge_adjacent_tables_transform(&mut d, body), id);
}

#[test]
fn skip_a8_two_clean_adjacent_tables_preserves_root() {
    // M112: clean adjacent tables must NOT merge — and we should not rebuild.
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    for _ in 0..2 {
        let tbl = d.new_element(w("tbl"));
        let tr = d.new_element(w("tr"));
        let tc = d.new_element(w("tc"));
        d.add(tr, tc);
        d.add(tbl, tr);
        d.add(body, tbl);
    }
    let id = body;
    let out = merge_adjacent_tables_transform(&mut d, body);
    assert_eq!(out, id);
    assert_eq!(d.elements(out, Some(&w("tbl"))).len(), 2);
}
