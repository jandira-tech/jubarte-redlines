// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Body tables as grids of cells, each cell with the ids of the paragraphs
//! it holds.

use std::collections::HashMap;

use serde::Serialize;

use super::{body_paragraph_nodes, project_paragraph};
use crate::namespaces::{MC, W};
use crate::xmllinq::{Dom, NodeId, XName};

/// A top-level body table, numbered as the view's `tN`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Table {
    /// Position among top-level body tables in document order.
    pub index: usize,
    /// Cells row by row, as they appear in the XML: a merged cell is one
    /// cell, so rows can differ in length.
    pub rows: Vec<Vec<TableCell>>,
    /// Leading rows marked to repeat as a header (`w:tblHeader`).
    pub header_rows: usize,
    /// Grid column widths (`w:gridCol`) in twentieths of a point; 0 when a
    /// width is missing or unreadable.
    pub widths_dxa: Vec<u32>,
    /// Tables nested directly in this table's cells, in document order.
    pub nested: Vec<NestedTable>,
}

/// A table nested in a cell. It has a parent cell path, rather than a `tN`
/// index in the body. Its own nested tables carry the full cell path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NestedTable {
    /// Containing cell, for example `t0.row2.cell0` or
    /// `t0.row2.cell0.t0.row0.cell1`.
    pub parent: String,
    /// The cell grid, with the same own-paragraph rule as [`Table::rows`].
    pub rows: Vec<Vec<TableCell>>,
    /// Leading rows marked to repeat as a header.
    pub header_rows: usize,
    /// Grid column widths in twentieths of a point.
    pub widths_dxa: Vec<u32>,
    /// Tables nested directly in this table's cells, in document order.
    pub nested: Vec<NestedTable>,
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

/// Whether a table is hidden from tN addressing by an opaque wrapper.
pub(crate) fn opaque_table(dom: &Dom, table: NodeId) -> bool {
    super::in_text_box(dom, table) || nearest(dom, table, &MC::name("AlternateContent")).is_some()
}

/// The tables numbered by the view: outside cells, text boxes and AC.
pub(crate) fn top_level_table_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId> {
    dom.descendants(body, Some(&W::tbl()))
        .into_iter()
        .filter(|&table| nearest(dom, table, &W::tc()).is_none() && !opaque_table(dom, table))
        .collect()
}

/// Top-level body tables, with their nested tables under their cells.
pub(super) fn body_tables(dom: &Dom, body: NodeId) -> Vec<Table> {
    let tc = W::tc();
    // Each addressable paragraph, grouped under the cell that holds it.
    let mut by_cell: HashMap<NodeId, Vec<(usize, NodeId)>> = HashMap::new();
    for (index, p) in body_paragraph_nodes(dom, body).into_iter().enumerate() {
        if let Some(cell) = nearest(dom, p, &tc) {
            by_cell.entry(cell).or_default().push((index, p));
        }
    }
    top_level_table_nodes(dom, body)
        .into_iter()
        .enumerate()
        .map(|(index, table)| {
            let grid = table_grid(dom, table, &format!("t{index}"), &by_cell);
            Table {
                index,
                rows: grid.rows,
                header_rows: grid.header_rows,
                widths_dxa: grid.widths_dxa,
                nested: grid.nested,
            }
        })
        .collect()
}

struct Grid {
    rows: Vec<Vec<TableCell>>,
    header_rows: usize,
    widths_dxa: Vec<u32>,
    nested: Vec<NestedTable>,
}

fn table_grid(
    dom: &Dom,
    table: NodeId,
    path: &str,
    by_cell: &HashMap<NodeId, Vec<(usize, NodeId)>>,
) -> Grid {
    let tc = W::tc();
    let tr = W::tr();
    let tbl = W::tbl();
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
            dom.elements(grid, Some(&W::grid_col()))
                .into_iter()
                .map(|col| {
                    dom.attribute(col, &W::w())
                        .and_then(|w| w.parse::<u32>().ok())
                        .unwrap_or(0)
                })
                .collect()
        })
        .unwrap_or_default();
    let mut nested = Vec::new();
    let rows = rows
        .into_iter()
        .enumerate()
        .map(|(r, row)| {
            dom.descendants(row, Some(&tc))
                .into_iter()
                .filter(|&cell| nearest(dom, cell, &tr) == Some(row))
                .enumerate()
                .map(|(c, cell)| {
                    let parent = format!("{path}.row{r}.cell{c}");
                    for (index, child) in dom
                        .descendants(cell, Some(&tbl))
                        .into_iter()
                        .filter(|&child| {
                            nearest(dom, child, &tc) == Some(cell) && !opaque_table(dom, child)
                        })
                        .enumerate()
                    {
                        let grid = table_grid(dom, child, &format!("{parent}.t{index}"), by_cell);
                        nested.push(NestedTable {
                            parent: parent.clone(),
                            rows: grid.rows,
                            header_rows: grid.header_rows,
                            widths_dxa: grid.widths_dxa,
                            nested: grid.nested,
                        });
                    }
                    let own = by_cell.get(&cell).map(Vec::as_slice).unwrap_or_default();
                    TableCell {
                        paragraph_ids: own
                            .iter()
                            .map(|&(i, _)| crate::ids::paragraph_id("body", i))
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
    Grid {
        rows,
        header_rows,
        widths_dxa,
        nested,
    }
}
