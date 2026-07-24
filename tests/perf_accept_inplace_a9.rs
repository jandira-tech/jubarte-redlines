// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ACCEPT-INPLACE-A9 — empty cells filled without full-tree rebuild.
//!
//! `add_empty_paragraph_to_any_empty_cells` must return the same root NodeId
//! (in-place) and append `w:p` to empty `w:tc`s; non-empty cells stay intact.

use jubarte::namespaces::W;
use jubarte::revision_processor::add_empty_paragraph_to_any_empty_cells;
use jubarte::xmllinq::{Dom, XName};

fn w(local: &str) -> XName {
    W::name(local)
}

#[test]
fn accept_inplace_a9_returns_same_root_and_fills_empty_cell() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let tbl = d.new_element(w("tbl"));
    let tr = d.new_element(w("tr"));
    let tc = d.new_element(w("tc"));
    let tcpr = d.new_element(w("tcPr"));
    d.add(tc, tcpr);
    d.add(tr, tc);
    d.add(tbl, tr);
    d.add(body, tbl);

    let out = add_empty_paragraph_to_any_empty_cells(&mut d, body);
    assert_eq!(
        out, body,
        "ACCEPT-INPLACE-A9 must return the same root NodeId"
    );

    let cells = d.descendants(body, Some(&W::tc()));
    assert_eq!(cells.len(), 1);
    let kids: Vec<_> = d
        .elements(cells[0], None)
        .into_iter()
        .map(|e| d.name(e).unwrap().local_name().to_string())
        .collect();
    assert_eq!(kids, vec!["tcPr".to_string(), "p".to_string()]);
}

#[test]
fn accept_inplace_a9_leaves_nonempty_cell_alone() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let tbl = d.new_element(w("tbl"));
    let tr = d.new_element(w("tr"));
    let tc = d.new_element(w("tc"));
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "x");
    d.add(r, t);
    d.add(p, r);
    d.add(tc, p);
    d.add(tr, tc);
    d.add(tbl, tr);
    d.add(body, tbl);

    let before = d.child_count(tc);
    let out = add_empty_paragraph_to_any_empty_cells(&mut d, body);
    assert_eq!(out, body);
    assert_eq!(
        d.child_count(tc),
        before,
        "non-empty cell must not gain a spare p"
    );
    assert_eq!(d.value(tc), "x");
}

#[test]
fn accept_inplace_a9_fills_nested_empty_only() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let tbl = d.new_element(w("tbl"));
    let tr = d.new_element(w("tr"));
    // empty cell
    let tc_empty = d.new_element(w("tc"));
    // non-empty cell
    let tc_full = d.new_element(w("tc"));
    let p = d.new_element(w("p"));
    d.add(tc_full, p);
    d.add(tr, tc_empty);
    d.add(tr, tc_full);
    d.add(tbl, tr);
    d.add(body, tbl);

    add_empty_paragraph_to_any_empty_cells(&mut d, body);
    let empty_kids: Vec<_> = d
        .elements(tc_empty, None)
        .into_iter()
        .map(|e| d.name(e).unwrap().local_name().to_string())
        .collect();
    assert_eq!(empty_kids, vec!["p".to_string()]);
    let full_kids: Vec<_> = d
        .elements(tc_full, None)
        .into_iter()
        .map(|e| d.name(e).unwrap().local_name().to_string())
        .collect();
    assert_eq!(full_kids, vec!["p".to_string()]);
}
