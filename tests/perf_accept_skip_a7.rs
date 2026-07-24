// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ACCEPT-SKIP-A7 — when no w:cellDel in the tree, accept_deleted_cells must
//! transfer the root NodeId (no full-tree rebuild).

use jubarte::namespaces::W;
use jubarte::revision_processor::accept_deleted_cells_transform;
use jubarte::xmllinq::Dom;

fn w(local: &str) -> jubarte::xmllinq::XName {
    W::name(local)
}

#[test]
fn skip_a7_no_celldel_preserves_root_id() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let tbl = d.new_element(w("tbl"));
    let tr = d.new_element(w("tr"));
    let tc = d.new_element(w("tc"));
    let p = d.new_element(w("p"));
    d.add(tc, p);
    d.add(tr, tc);
    d.add(tbl, tr);
    d.add(body, tbl);
    let body_id = body;
    let out = accept_deleted_cells_transform(&mut d, body);
    assert_eq!(out, body_id, "no cellDel → transfer same root");
}

#[test]
fn skip_a7_with_celldel_still_drops_cell() {
    let mut d = Dom::new();
    let tr = d.new_element(w("tr"));
    // kept cell
    let tc1 = d.new_element(w("tc"));
    let tcpr1 = d.new_element(w("tcPr"));
    d.add(tc1, tcpr1);
    let p1 = d.new_element(w("p"));
    d.add(tc1, p1);
    d.add(tr, tc1);
    // deleted cell
    let tc2 = d.new_element(w("tc"));
    let tcpr2 = d.new_element(w("tcPr"));
    let cell_del = d.new_element(w("cellDel"));
    d.add(tcpr2, cell_del);
    d.add(tc2, tcpr2);
    let p2 = d.new_element(w("p"));
    d.add(tc2, p2);
    d.add(tr, tc2);
    let out = accept_deleted_cells_transform(&mut d, tr);
    // only one tc remains
    assert_eq!(d.elements(out, Some(&w("tc"))).len(), 1);
}
