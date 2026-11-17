// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word table structure questions a DOM walk answers.
//!
//! `table_look` reads `w:tblLook` banding flags, `cell_is_deleted` the
//! `w:cellDel` mark, and `wrapped_children` the children of an element
//! through `w:sdt` / `w:customXml` wrappers, with the same prior-change
//! filtering the renderer applies.

use crate::namespaces::{A, R, W, W14, WP};
use crate::xmllinq::{Dom, NodeId, XName};

/// The table's own `w:tblPr` (a direct child). A descendant search would
/// return a nested table's properties when the outer table has none.
fn table_pr(dom: &Dom, table: NodeId) -> Option<NodeId> {
    direct_named(dom, table, "tblPr")
}

/// The cell's own `w:tcPr` (a direct child). A descendant search would
/// return the first nested cell's properties when this cell has none of
/// its own (#1729): a tcPr-less cell holding a nested table borrowed the
/// nested first cell's span, merge, borders, fill, margins, noWrap,
/// direction, alignment and hideMark. A `w:tcPrChange` sits inside the
/// cell's own tcPr, so lookups within `pr` keep their prior-change
/// filters.
fn cell_pr(dom: &Dom, cell: NodeId) -> Option<NodeId> {
    direct_named(dom, cell, "tcPr")
}

fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    // w:pPrChange / rPrChange / tblPrChange hold the *previous* pPr.
    // file_146 live pPr is pBdr+spacing; ListParagraph hanging lives
    // only in pPrChange. Stealing that ghost indented Hello at 90.
    dom.find_descendant_element(node, Some(&W::name(local)), |cand| {
        !under_prior_change(dom, node, cand)
    })
}

fn under_prior_change(dom: &Dom, root: NodeId, mut n: NodeId) -> bool {
    while n != root {
        let Some(parent) = dom.parent(n) else {
            break;
        };
        if parent == root {
            break;
        }
        if local_name_is(dom, parent, "pPrChange")
            || local_name_is(dom, parent, "rPrChange")
            || local_name_is(dom, parent, "tblPrChange")
            || local_name_is(dom, parent, "trPrChange")
            || local_name_is(dom, parent, "tcPrChange")
            || local_name_is(dom, parent, "sectPrChange")
            || local_name_is(dom, parent, "numPrChange")
        {
            return true;
        }
        n = parent;
    }
    false
}

fn direct_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    let want = W::name(local);
    (0..dom.child_count(node)).find_map(|i| {
        let child = dom.child_at(node, i);
        dom.name_is(child, &want).then_some(child)
    })
}

fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str> {
    for name in [
        XName::get(local, ""),
        W::name(local),
        W14::name(local),
        WP::name(local),
        A::name(local),
        R::name(local),
    ] {
        if let Some(v) = dom.attribute(node, &name) {
            return Some(v);
        }
    }
    None
}

fn local_name_is(dom: &Dom, node: NodeId, local: &str) -> bool {
    dom.name(node).is_some_and(|n| n.local_name() == local)
}

#[derive(Clone, Copy)]
#[doc(hidden)]
pub struct TblLook {
    pub first_row: bool,
    pub first_col: bool,
    pub no_h_band: bool,
}

#[doc(hidden)]
pub fn table_look(dom: &Dom, table: NodeId) -> TblLook {
    let mut look = TblLook {
        first_row: true,
        first_col: false,
        no_h_band: false,
    };
    let Some(pr) = table_pr(dom, table) else {
        return look;
    };
    let Some(el) = first_named(dom, pr, "tblLook") else {
        return look;
    };
    if let Some(v) = attr_any(dom, el, "firstRow") {
        look.first_row = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = attr_any(dom, el, "firstColumn") {
        look.first_col = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = attr_any(dom, el, "noHBand") {
        look.no_h_band = v == "1" || v.eq_ignore_ascii_case("true");
    }
    look
}

#[doc(hidden)]
pub fn cell_is_deleted(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = cell_pr(dom, cell) else {
        return false;
    };
    // Direct child only. tcPrChange also stores cellDel.
    direct_named(dom, pr, "cellDel").is_some()
}

/// `parent`'s `w:<local>` children, also those a content control or a
/// custom-XML wrapper holds (003c9ddd's tr > sdt > sdtContent > tc).
#[doc(hidden)]
pub fn wrapped_children(dom: &Dom, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut out = Vec::new();
    for i in 0..dom.child_count(parent) {
        let child = dom.child_at(parent, i);
        if local_name_is(dom, child, local) {
            out.push(child);
        } else if local_name_is(dom, child, "sdt")
            || local_name_is(dom, child, "sdtContent")
            || local_name_is(dom, child, "customXml")
        {
            out.extend(wrapped_children(dom, child, local));
        }
    }
    out
}
