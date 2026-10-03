// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Body tables as grids of cells, each cell with the ids of the paragraphs
//! it holds.

use std::collections::HashMap;

use serde::Serialize;

use super::{body_paragraph_nodes, project_paragraph};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XName};

/// A body table, nested tables included (each is its own entry).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Table {
    /// Position among the body's tables in document order, nested ones
    /// counted where they start.
    pub index: usize,
    /// Cells row by row, as they appear in the XML: a merged cell is one
    /// cell, so rows can differ in length.
    pub rows: Vec<Vec<TableCell>>,
    /// Leading rows marked to repeat as a header (`w:tblHeader`).
    pub header_rows: usize,
    /// Grid column widths (`w:gridCol`) in twentieths of a point; 0 when a
    /// width is missing or unreadable.
    pub widths_dxa: Vec<u32>,
}

/// A table cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TableCell {
    /// Ids (`body:p:N`) of the cell's own paragraphs; a nested table's
    /// paragraphs belong to that table's cells.
    pub paragraph_ids: Vec<String>,
    /// Those paragraphs' text, joined with `\n`.
    pub text: String,
}

/// The nearest ancestor named `name`.
fn nearest(dom: &Dom, node: NodeId, name: &XName) -> Option<NodeId> {
    dom.ancestors(node, Some(name)).first().copied()
}

/// Every table under `body`, in document order.
pub(super) fn body_tables(dom: &Dom, body: NodeId) -> Vec<Table> {
    let tc = W::tc();
    let tr = W::name("tr");
    let tbl = W::tbl();
    // Each addressable paragraph, grouped under the cell that holds it.
    let mut by_cell: HashMap<NodeId, Vec<(usize, NodeId)>> = HashMap::new();
    for (index, p) in body_paragraph_nodes(dom, body).into_iter().enumerate() {
        if let Some(cell) = nearest(dom, p, &tc) {
            by_cell.entry(cell).or_default().push((index, p));
        }
    }
    dom.descendants(body, Some(&tbl))
        .into_iter()
        .enumerate()
        .map(|(index, table)| {
            let rows: Vec<NodeId> = dom
                .descendants(table, Some(&tr))
                .into_iter()
                .filter(|&row| nearest(dom, row, &tbl) == Some(table))
                .collect();
            let header_rows = rows
                .iter()
                .take_while(|&&row| {
                    dom.element(row, &W::tr_pr())
                        .and_then(|pr| dom.element(pr, &W::name("tblHeader")))
                        .is_some_and(|h| {
                            !matches!(dom.attribute(h, &W::val()), Some("0" | "false" | "off"))
                        })
                })
                .count();
            let widths_dxa = dom
                .element(table, &W::name("tblGrid"))
                .map(|grid| {
                    dom.elements(grid, Some(&W::name("gridCol")))
                        .into_iter()
                        .map(|col| {
                            dom.attribute(col, &W::name("w"))
                                .and_then(|w| w.parse::<u32>().ok())
                                .unwrap_or(0)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let rows = rows
                .into_iter()
                .map(|row| {
                    dom.descendants(row, Some(&tc))
                        .into_iter()
                        .filter(|&cell| nearest(dom, cell, &tr) == Some(row))
                        .map(|cell| {
                            let own = by_cell.get(&cell).map(Vec::as_slice).unwrap_or_default();
                            TableCell {
                                paragraph_ids: own
                                    .iter()
                                    .map(|(i, _)| format!("body:p:{i}"))
                                    .collect(),
                                text: own
                                    .iter()
                                    .map(|&(_, p)| project_paragraph(dom, p).text)
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                            }
                        })
                        .collect()
                })
                .collect();
            Table {
                index,
                rows,
                header_rows,
                widths_dxa,
            }
        })
        .collect()
}
