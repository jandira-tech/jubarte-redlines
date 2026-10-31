// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word keeps rows of one table style and layout in one table (#1817).
//!
//! Word stores a table's properties on each of its rows. In a table Compare
//! pairs, the kept and inserted rows carry the revision's properties and the
//! deleted rows the original's; when Word saves, a new table starts wherever
//! two adjacent rows differ in table style (matched by name, as Compare maps
//! the two documents' styles) or in layout type. So where the two sides of a
//! paired table differ in either, each run of the original's deleted rows is
//! a table of its own, with the original's properties and its rows' grid, and each run
//! of live rows keeps the revision's. Word's probes (fix23 round 32): the
//! style and the layout split the rows; borders, width, indent, alignment,
//! look and cell margins do not. In Word's redlines of the 72 bench pairs
//! whose documents both hold two or more tables, no paired table whose sides
//! differ so keeps a deleted row.

use std::collections::{HashMap, HashSet};

use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// A styles part's style names by id, and its default table style's name.
#[derive(Debug, Default)]
pub struct TableStyleNames {
    by_id: HashMap<String, String>,
    default_table: Option<String>,
}

impl TableStyleNames {
    /// Reads the original's `styles_xml`, on which Word builds its redline:
    /// a style without a name keeps it, by its id (the 3x3 pair: the
    /// original's unnamed style "a" against the revision's default splits
    /// the rows in Word's redline). No part reads as no styles.
    pub fn original(styles_xml: Option<&str>) -> Self {
        Self::parse(styles_xml, true)
    }

    /// Reads the revision's `styles_xml`, whose styles Word maps into the
    /// redline by name: a style without a name has none to map by, and its
    /// tables take the default (70c30e926c: Word's redline drops the
    /// revision's unnamed "a0", and the rows stay together).
    pub fn revision(styles_xml: Option<&str>) -> Self {
        Self::parse(styles_xml, false)
    }

    fn parse(styles_xml: Option<&str>, keep_unnamed: bool) -> Self {
        let mut names = Self::default();
        let Some(xml) = styles_xml else {
            return names;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let Some(root) = dom.root(doc) else {
            return names;
        };
        for style in dom.elements(root, Some(&W::name("style"))) {
            let Some(id) = dom.attribute(style, &W::name("styleId")) else {
                continue;
            };
            let named = dom
                .element(style, &W::name("name"))
                .and_then(|n| dom.attribute(n, &W::val()));
            let Some(name) = named.or(keep_unnamed.then_some(id)) else {
                continue;
            };
            if dom.attribute(style, &W::name("type")) == Some("table")
                && matches!(
                    dom.attribute(style, &W::name("default")),
                    Some("1" | "true")
                )
            {
                names.default_table = Some(name.to_string());
            }
            names.by_id.insert(id.to_string(), name.to_string());
        }
        names
    }

    /// The name of the style `id` names. No id, or one the part lacks, names
    /// the default table style: the part's, else Word's built-in "Normal
    /// Table" (w23: a part without table styles against one that lists
    /// "Normal Table" keeps the rows together in Word's redline).
    fn name(&self, id: Option<&str>) -> String {
        id.and_then(|id| self.by_id.get(id))
            .or(self.default_table.as_ref())
            .map_or_else(|| "Normal Table".to_string(), String::clone)
    }
}

/// What keeps rows in one table for Word: the table style's name and
/// whether the layout is fixed.
fn identity(dom: &Dom, tbl_pr: Option<NodeId>, names: &TableStyleNames) -> (String, bool) {
    let child = |name: &str| tbl_pr.and_then(|pr| dom.element(pr, &W::name(name)));
    let style = child("tblStyle").and_then(|s| dom.attribute(s, &W::val()));
    let fixed =
        child("tblLayout").and_then(|l| dom.attribute(l, &W::name("type"))) == Some("fixed");
    (names.name(style), fixed)
}

/// Whether a table child is a deleted row (`Some(true)`), a live row
/// (`Some(false)`), or no row (a bookmark between rows).
fn deleted_row(dom: &Dom, child: NodeId) -> Option<bool> {
    let rows = if dom.name_is(child, &W::tr()) {
        vec![child]
    } else {
        dom.descendants(child, Some(&W::tr()))
    };
    if rows.is_empty() {
        return None;
    }
    let deleted = |row: NodeId| {
        dom.element(row, &W::tr_pr())
            .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
    };
    Some(rows.into_iter().all(deleted))
}

/// One past the largest `w:id` under `root`.
/// The `w:id`s under `root`, from which [`take_id`] hands out new ones.
fn taken_ids(dom: &Dom, root: NodeId) -> HashSet<u32> {
    dom.descendants(root, None)
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id())?.parse::<u32>().ok())
        .collect()
}

/// The first id after the largest taken one that no node holds: past
/// `u32::MAX` the search wraps instead of overflowing into a taken id.
fn take_id(taken: &mut HashSet<u32>) -> u32 {
    let mut next = taken
        .iter()
        .copied()
        .max()
        .map_or(0, |max| max.wrapping_add(1));
    while taken.contains(&next) {
        next = next.wrapping_add(1);
    }
    taken.insert(next);
    next
}

/// Splits every body table whose `tblPrChange` records an original of
/// another table style or layout into runs of deleted and live rows, each a
/// table: the deleted runs with the original's properties and grid, the
/// live runs with the table's own. `old_names` resolves the original's style
/// ids, `new_names` the revision's.
pub fn split_rows_by_table_identity(
    dom: &mut Dom,
    root: NodeId,
    old_names: &TableStyleNames,
    new_names: &TableStyleNames,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let mut taken = taken_ids(dom, root);
    for tbl in dom.descendants(body, Some(&W::tbl())) {
        let Some(pr) = dom.element(tbl, &W::tbl_pr()) else {
            continue;
        };
        let Some(change) = dom.element(pr, &W::name("tblPrChange")) else {
            continue;
        };
        let old_pr = dom.element(change, &W::tbl_pr());
        if identity(dom, old_pr, old_names) == identity(dom, Some(pr), new_names) {
            continue;
        }
        let grid = dom.element(tbl, &W::name("tblGrid"));
        let children: Vec<NodeId> = dom
            .elements(tbl, None)
            .into_iter()
            .filter(|&c| Some(c) != grid && c != pr)
            .collect();
        // A child that is no row goes with the row before it, or the first.
        let kinds: Vec<Option<bool>> = children.iter().map(|&c| deleted_row(dom, c)).collect();
        let Some(first) = kinds.iter().flatten().next().copied() else {
            continue;
        };
        let mut runs: Vec<(bool, Vec<NodeId>)> = vec![(first, Vec::new())];
        for (&child, kind) in children.iter().zip(&kinds) {
            match kind {
                Some(deleted) if *deleted != runs.last().map_or(first, |r| r.0) => {
                    runs.push((*deleted, vec![child]));
                }
                _ => {
                    if let Some(run) = runs.last_mut() {
                        run.1.push(child);
                    }
                }
            }
        }
        if runs.len() < 2 {
            continue;
        }
        let mut live_seen = false;
        for (deleted, rows) in runs {
            let table = dom.new_element(W::tbl());
            let (props, layout) = if deleted {
                let props = match old_pr {
                    Some(old) => dom.clone_subtree(old),
                    None => dom.new_element(W::tbl_pr()),
                };
                let old_grid = grid
                    .and_then(|g| dom.element(g, &W::name("tblGridChange")))
                    .and_then(|c| dom.element(c, &W::name("tblGrid")));
                let layout = match old_grid.or(grid) {
                    Some(g) => dom.clone_subtree(g),
                    None => dom.new_element(W::name("tblGrid")),
                };
                for stale in dom.elements(layout, Some(&W::name("tblGridChange"))) {
                    dom.remove(stale);
                }
                (props, layout)
            } else {
                let props = dom.clone_subtree(pr);
                let layout = match grid {
                    Some(g) => dom.clone_subtree(g),
                    None => dom.new_element(W::name("tblGrid")),
                };
                // Each copy of the change records after the first needs ids
                // of its own.
                if live_seen {
                    let records: Vec<NodeId> = [(props, "tblPrChange"), (layout, "tblGridChange")]
                        .into_iter()
                        .flat_map(|(el, name)| dom.elements(el, Some(&W::name(name))))
                        .collect();
                    for record in records {
                        let id = take_id(&mut taken);
                        dom.set_attribute_value(record, &W::id(), Some(&id.to_string()));
                    }
                }
                live_seen = true;
                (props, layout)
            };
            dom.add(table, props);
            dom.add(table, layout);
            for row in rows {
                dom.remove(row);
                dom.add(table, row);
            }
            if deleted {
                rebuild_deleted_grid(dom, table, layout);
            }
            dom.add_before_self(tbl, table);
            if deleted {
                resolve_deleted_preferences(dom, table, layout);
            }
        }
        dom.remove(tbl);
    }
}

/// A row's cell edges, expressed in the original grid's column indices.
struct RowEdges {
    row: NodeId,
    cells: Vec<NodeId>,
    edges: Vec<usize>,
    after: usize,
}

fn grid_count(dom: &Dom, pr: Option<NodeId>, name: &str, default: usize) -> Option<usize> {
    match pr.and_then(|pr| dom.element(pr, &W::name(name))) {
        Some(el) => dom.attribute(el, &W::val())?.parse().ok(),
        None => Some(default),
    }
}

/// Coalesce columns at edges no cell in this deleted run uses (#1913).
/// Coalescing starts from the original grid. The separate preference pass
/// resolves supported layouts after the split is attached to its section.
///
/// Word's saved redlines have no tblGridChange on any of the 108 adjacent
/// all-deleted tables in fix23/r47. This is the split table's own grid, not
/// another tracked edit. In be486e3b25 Word also drops the trailing columns
/// every row explicitly omits; retained cell widths do not change.
fn rebuild_deleted_grid(dom: &mut Dom, table: NodeId, grid: NodeId) {
    let columns = dom.elements(grid, Some(&W::grid_col()));
    let Some(widths): Option<Vec<u32>> = columns
        .iter()
        .map(|&col| dom.attribute(col, &W::name("w"))?.parse().ok())
        .collect()
    else {
        return;
    };
    if widths.is_empty() || widths.contains(&0) {
        return;
    }
    let mut rows = Vec::new();
    // Include sdt/customXml wrappers, but never the rows of a nested table.
    for row in dom.descendants(table, Some(&W::tr())) {
        if dom.ancestors(row, Some(&W::tbl())).first() != Some(&table) {
            continue;
        }
        let pr = dom.element(row, &W::tr_pr());
        let (Some(before), Some(after)) = (
            grid_count(dom, pr, "gridBefore", 0),
            grid_count(dom, pr, "gridAfter", 0),
        ) else {
            return;
        };
        let cells: Vec<_> = dom
            .descendants(row, Some(&W::tc()))
            .into_iter()
            .filter(|&cell| dom.ancestors(cell, Some(&W::tr())).first() == Some(&row))
            .collect();
        if cells.is_empty() {
            return;
        }
        let mut edges = vec![before];
        let mut end = before;
        for &cell in &cells {
            let Some(span) = grid_count(dom, dom.element(cell, &W::tc_pr()), "gridSpan", 1)
                .filter(|&span| span > 0)
            else {
                return;
            };
            let Some(next) = end.checked_add(span).filter(|&n| n <= widths.len()) else {
                return;
            };
            end = next;
            edges.push(end);
        }
        if end.checked_add(after) != Some(widths.len()) {
            // An incomplete/malformed row supplies no reliable geometry.
            // Leave the whole run alone rather than guess at missing cells.
            return;
        }
        rows.push(RowEdges {
            row,
            cells,
            edges,
            after,
        });
    }
    let Some(trailing) = rows.iter().map(|r| r.after).min() else {
        return;
    };
    let end = widths.len() - trailing;
    let mut edges = vec![0, end];
    edges.extend(rows.iter().flat_map(|r| r.edges.iter().copied()));
    edges.sort_unstable();
    edges.dedup();
    if edges == (0..=widths.len()).collect::<Vec<_>>() {
        return;
    }
    for col in columns {
        dom.remove(col);
    }
    for pair in edges.windows(2) {
        let width: u64 = widths[pair[0]..pair[1]].iter().map(|&w| u64::from(w)).sum();
        let col = dom.new_element(W::grid_col());
        dom.set_attribute_value(col, &W::name("w"), Some(&width.to_string()));
        dom.add(grid, col);
    }
    for row in rows {
        // Every row edge was inserted above, so each binary search succeeds.
        let indices: Vec<_> = row
            .edges
            .iter()
            .map(|e| edges.binary_search(e).unwrap())
            .collect();
        for (&cell, pair) in row.cells.iter().zip(indices.windows(2)) {
            set_grid_count(dom, cell, "tcPr", "gridSpan", pair[1] - pair[0], 1);
        }
        set_grid_count(dom, row.row, "trPr", "gridBefore", indices[0], 0);
        let after = edges.len() - 1 - indices[indices.len() - 1];
        set_grid_count(dom, row.row, "trPr", "gridAfter", after, 0);
        if let Some(pr) = dom.element(row.row, &W::tr_pr()) {
            for (name, count) in [("wBefore", indices[0]), ("wAfter", after)] {
                if let Some(width) = dom.element(pr, &W::name(name)) {
                    if count == 0 {
                        dom.remove(width);
                    } else if name == "wAfter" && trailing > 0 {
                        let last = row.edges[row.edges.len() - 1];
                        let remaining: u64 = widths[last..end].iter().map(|&w| u64::from(w)).sum();
                        dom.set_attribute_value(width, &W::name("w"), Some(&remaining.to_string()));
                        dom.set_attribute_value(width, &W::name("type"), Some("dxa"));
                    }
                }
            }
        }
    }
}

/// Resolve the part of Word's layout normalization established by #1913's
/// twelve controlled probes: agreeing positive dxa cell preferences, with
/// an auto total or a larger declared total. This also applies to fixed
/// tables. Keep content-dependent or underspecified layouts unchanged;
/// notably, tcW is not a universal substitute for the stored grid.
fn resolve_deleted_preferences(dom: &mut Dom, table: NodeId, grid: NodeId) {
    let Some(widths) = deleted_preferred_widths(dom, table, grid) else {
        return;
    };
    for (col, width) in dom
        .elements(grid, Some(&W::grid_col()))
        .into_iter()
        .zip(widths)
    {
        dom.set_attribute_value(col, &W::w(), Some(&width.to_string()));
    }
}

/// Explicit horizontal margins avoid guessing at style/default resolution.
/// Spacing is a separate addition to cell widths (435d2c2474), not a margin.
fn explicit_margins(dom: &Dom, pr: NodeId, name: &str) -> Option<(u32, u32)> {
    let mar = dom.element(pr, &W::name(name))?;
    let width = |side: &str| {
        let el = dom.element(mar, &W::name(side))?;
        (dom.attribute(el, &W::type_()) == Some("dxa"))
            .then(|| dom.attribute(el, &W::w())?.parse().ok())?
    };
    Some((width("left")?, width("right")?))
}

fn deleted_preferred_widths(dom: &Dom, table: NodeId, grid: NodeId) -> Option<Vec<u32>> {
    let columns = dom.elements(grid, Some(&W::grid_col()));
    if columns.is_empty()
        || columns.iter().any(|&c| {
            dom.attribute(c, &W::w())
                .and_then(|v| v.parse::<u32>().ok())
                .is_none_or(|w| w == 0)
        })
    {
        return None;
    }
    let pr = dom.element(table, &W::tbl_pr())?;
    if dom.element(pr, &W::name("tblCellSpacing")).is_some() {
        return None;
    }
    let margins = explicit_margins(dom, pr, "tblCellMar");
    let mut preferred = None;
    let rows = dom.elements(table, Some(&W::tr()));
    // Wrapped or nested rows and competing preferences require a fuller
    // layout solver. Do not silently size only a subset of the run.
    if rows.len() != dom.descendants(table, Some(&W::tr())).len() {
        return None;
    }
    for row in rows {
        let row_pr = dom.element(row, &W::tr_pr());
        if grid_count(dom, row_pr, "gridBefore", 0)? != 0
            || grid_count(dom, row_pr, "gridAfter", 0)? != 0
            || dom.element(row, &W::name("tblPrEx")).is_some_and(|ex| {
                dom.element(ex, &W::name("tblCellSpacing")).is_some()
                    || dom.element(ex, &W::name("tblCellMar")).is_some_and(|mar| {
                        ["left", "right", "start", "end"]
                            .iter()
                            .any(|side| dom.element(mar, &W::name(side)).is_some())
                    })
            })
            || row_pr.is_some_and(|rp| dom.element(rp, &W::name("tblCellSpacing")).is_some())
        {
            return None;
        }
        let cells = dom.elements(row, Some(&W::tc()));
        if cells.len() != columns.len() {
            return None;
        }
        let mut widths = Vec::new();
        for cell in cells {
            let cp = dom.element(cell, &W::tc_pr())?;
            if grid_count(dom, Some(cp), "gridSpan", 1)? != 1 {
                return None;
            }
            if dom.element(cp, &W::name("tcMar")).is_some() {
                explicit_margins(dom, cp, "tcMar")?;
            } else {
                margins?;
            }
            let width = dom.element(cp, &W::name("tcW"))?;
            if dom.attribute(width, &W::type_()) != Some("dxa") {
                return None;
            }
            let width: u32 = dom.attribute(width, &W::w())?.parse().ok()?;
            if width == 0 || width > 31_680 {
                return None;
            }
            widths.push(width);
        }
        if preferred.as_ref().is_some_and(|p| p != &widths) {
            return None;
        }
        preferred = Some(widths);
    }
    let preferred = preferred?;
    let sum: u64 = preferred.iter().map(|&w| u64::from(w)).sum();
    let tw = dom.element(pr, &W::tbl_w())?;
    let total = match dom.attribute(tw, &W::type_())? {
        "auto" => sum,
        "dxa" => dom.attribute(tw, &W::w())?.parse().ok()?,
        "pct" => {
            // The controlled percent probes have zero margins and indent;
            // inherited edge allowances and multi-column sections are not
            // established by them. Nor is a nested table's available width.
            if margins != Some((0, 0)) || dom.element(pr, &W::name("tblInd")).is_some() {
                return None;
            }
            let area = split_section_width(dom, table)?;
            let percent: u64 = dom.attribute(tw, &W::w())?.parse().ok()?;
            let product = area.checked_mul(percent)?;
            if product % 5000 != 0 {
                return None;
            }
            product / 5000
        }
        _ => return None,
    };
    // Shrinking below preferred widths and fractional allocation depend on
    // content/rounding not isolated by these probes. Preserve the old grid.
    if total < sum || total > 31_680 {
        return None;
    }
    preferred
        .into_iter()
        .map(|w| {
            let product = u64::from(w) * total;
            (product % sum == 0).then(|| (product / sum) as u32)
        })
        .collect()
}

/// The section ending after this body table, excluding revision histories
/// and section properties inside cells. No fallback to the document's first
/// section: the existing corpus includes changes of section width.
fn split_section_width(dom: &Dom, table: NodeId) -> Option<u64> {
    if !dom
        .parent(table)
        .is_some_and(|p| dom.name_is(p, &W::body()))
    {
        return None;
    }
    let section = dom.nodes_after_self(table).into_iter().find_map(|node| {
        if dom.name_is(node, &W::sect_pr()) {
            Some(node)
        } else if dom.name_is(node, &W::p()) {
            dom.element(node, &W::p_pr())
                .and_then(|p| dom.element(p, &W::sect_pr()))
        } else {
            None
        }
    })?;
    if dom.element(section, &W::name("cols")).is_some_and(|c| {
        dom.attribute(c, &W::name("num")).is_some_and(|n| n != "1")
            || !dom.elements(c, Some(&W::name("col"))).is_empty()
    }) {
        return None;
    }
    let size = dom.element(section, &W::name("pgSz"))?;
    let margins = dom.element(section, &W::name("pgMar"))?;
    let page: u64 = dom.attribute(size, &W::w())?.parse().ok()?;
    let left: u64 = dom.attribute(margins, &W::name("left"))?.parse().ok()?;
    let right: u64 = dom.attribute(margins, &W::name("right"))?.parse().ok()?;
    let gutter: u64 = dom
        .attribute(margins, &W::name("gutter"))
        .unwrap_or("0")
        .parse()
        .ok()?;
    if gutter != 0 {
        return None;
    }
    page.checked_sub(left)?
        .checked_sub(right)
        .filter(|&w| w > 0)
}

/// Coalescing only reduces counts, so a non-default result already has a
/// property. Update it in place to retain schema order and revision records.
fn set_grid_count(
    dom: &mut Dom,
    owner: NodeId,
    props: &str,
    name: &str,
    count: usize,
    default: usize,
) {
    let Some(el) = dom
        .element(owner, &W::name(props))
        .and_then(|pr| dom.element(pr, &W::name(name)))
    else {
        return;
    };
    if count == default {
        dom.remove(el);
    } else {
        dom.set_attribute_value(el, &W::val(), Some(&count.to_string()));
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    fn preferred_xml() -> String {
        format!(
            r#"<w:document xmlns:w="{}"><w:body><w:tbl><w:tblPr><w:tblW w:w="9000" w:type="dxa"/><w:tblCellMar><w:left w:w="0" w:type="dxa"/><w:right w:w="0" w:type="dxa"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="3000"/><w:gridCol w:w="3000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p/></w:tc><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl><w:p/><w:sectPr><w:pgSz w:w="11880"/><w:pgMar w:left="1440" w:right="1440"/></w:sectPr></w:body></w:document>"#,
            W::URI
        )
    }

    fn preferred_from_xml(xml: &str) -> Option<Vec<u32>> {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let table = dom.descendants(dom.root(doc).unwrap(), Some(&W::tbl()))[0];
        let grid = dom.element(table, &W::tbl_grid()).unwrap();
        deleted_preferred_widths(&dom, table, grid)
    }

    #[test]
    fn split_preferences_keep_ambiguous_or_invalid_geometry() {
        let xml = preferred_xml();
        assert_eq!(preferred_from_xml(&xml), Some(vec![3000, 6000]));
        for (from, to) in [
            (r#"<w:gridCol w:w="3000"/>"#, ""),
            (r#"w:w="3000""#, r#"w:w="0""#),
            (r#"w:w="3000""#, r#"w:w="bad""#),
            (
                "<w:tblPr>",
                r#"<w:tblPr><w:tblCellSpacing w:w="14" w:type="dxa"/>"#,
            ),
            (
                "<w:tr>",
                r#"<w:tr><w:trPr><w:gridBefore w:val="1"/></w:trPr>"#,
            ),
            (
                "<w:tr>",
                r#"<w:tr><w:trPr><w:gridAfter w:val="1"/></w:trPr>"#,
            ),
            (
                "<w:tr>",
                r#"<w:tr><w:tblPrEx><w:tblCellSpacing w:w="14" w:type="dxa"/></w:tblPrEx>"#,
            ),
            (
                "<w:tr>",
                r#"<w:tr><w:trPr><w:tblCellSpacing w:w="14" w:type="dxa"/></w:trPr>"#,
            ),
            ("<w:tr>", "<w:sdt><w:sdtContent><w:tr>"),
            ("<w:tcPr>", r#"<w:tcPr><w:gridSpan w:val="2"/>"#),
            (r#"<w:tcW w:w="2000" w:type="dxa"/>"#, ""),
            (
                r#"<w:tcW w:w="2000" w:type="dxa"/>"#,
                r#"<w:tcW w:w="0" w:type="auto"/>"#,
            ),
            (r#"w:w="2000""#, r#"w:w="0""#),
            (r#"w:w="2000""#, r#"w:w="40000""#),
            (r#"w:w="2000""#, r#"w:w="bad""#),
            (r#"<w:tblW w:w="9000" w:type="dxa"/>"#, ""),
            (
                r#"<w:tblW w:w="9000" w:type="dxa"/>"#,
                r#"<w:tblW w:w="0" w:type="nil"/>"#,
            ),
            (r#"w:w="9000""#, r#"w:w="5000""#),
            (r#"w:w="9000""#, r#"w:w="40000""#),
            (r#"w:w="9000""#, r#"w:w="9001""#),
        ] {
            // Close the one intentionally wrapped row before parsing.
            let changed = xml.replace(from, to).replace(
                "</w:tr></w:tbl>",
                if to.contains("<w:sdt>") {
                    "</w:tr></w:sdtContent></w:sdt></w:tbl>"
                } else {
                    "</w:tr></w:tbl>"
                },
            );
            assert_eq!(preferred_from_xml(&changed), None, "{from} -> {to}");
        }
        let row = xml
            .split("<w:tr>")
            .nth(1)
            .unwrap()
            .split("</w:tr>")
            .next()
            .unwrap();
        let conflicting = xml.replace(
            "</w:tr>",
            &format!("</w:tr><w:tr>{}</w:tr>", row.replace("2000", "2100")),
        );
        assert_eq!(preferred_from_xml(&conflicting), None, "conflicting rows");
        let incomplete = xml.replace(
            r#"<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p/></w:tc>"#,
            "",
        );
        assert_eq!(preferred_from_xml(&incomplete), None, "incomplete row");
    }

    #[test]
    fn split_preferences_do_not_ignore_cell_or_row_margin_overrides() {
        let xml = preferred_xml();
        for changed in [
            xml.replace("</w:tcPr>", r#"<w:tcMar><w:left w:w="100" w:type="dxa"/></w:tcMar></w:tcPr>"#),
            xml.replace("<w:tr>", r#"<w:tr><w:tblPrEx><w:tblCellMar><w:left w:w="100" w:type="dxa"/></w:tblCellMar></w:tblPrEx>"#),
        ] {
            assert_eq!(preferred_from_xml(&changed), None, "unresolved margin overrides");
        }
    }

    #[test]
    fn split_preferences_percent_requires_unambiguous_section_geometry() {
        let xml = preferred_xml().replace(
            r#"<w:tblW w:w="9000" w:type="dxa"/>"#,
            r#"<w:tblW w:w="5000" w:type="pct"/>"#,
        );
        assert_eq!(preferred_from_xml(&xml), Some(vec![3000, 6000]));
        for (from, to) in [
            (r#"w:w="0""#, r#"w:w="108""#),
            (
                "</w:tblPr>",
                r#"<w:tblInd w:w="10" w:type="dxa"/></w:tblPr>"#,
            ),
            ("<w:sectPr>", r#"<w:sectPr><w:cols w:num="2"/>"#),
            (
                "<w:sectPr>",
                r#"<w:sectPr><w:cols><w:col w:w="4500"/></w:cols>"#,
            ),
            (r#"w:left="1440""#, r#"w:gutter="100" w:left="1440""#),
            (r#"w:w="11880""#, r#"w:w="1000""#),
            (r#"w:w="11880""#, r#"w:w="2880""#),
            (r#"w:w="5000""#, r#"w:w="5001""#),
            ("<w:pgSz w:w=", "<w:unavailable w:w="),
            ("<w:pgMar w:left=", "<w:unavailable w:left="),
        ] {
            assert_eq!(
                preferred_from_xml(&xml.replace(from, to)),
                None,
                "{from} -> {to}"
            );
        }
        let preceding = xml.replace("<w:body>", r#"<w:body><w:p><w:pPr><w:sectPr><w:pgSz w:w="20000"/><w:pgMar w:left="1000" w:right="1000"/></w:sectPr></w:pPr></w:p>"#);
        assert_eq!(
            preferred_from_xml(&preceding),
            Some(vec![3000, 6000]),
            "ignore preceding section"
        );
        let in_paragraph = xml
            .replace("<w:sectPr>", "<w:p><w:pPr><w:sectPr>")
            .replace("</w:sectPr>", "</w:sectPr></w:pPr></w:p>");
        assert_eq!(
            preferred_from_xml(&in_paragraph),
            Some(vec![3000, 6000]),
            "section-ending paragraph"
        );
        let detached = xml
            .replace("<w:body>", "<w:body><w:sdt><w:sdtContent>")
            .replace("</w:body>", "</w:sdtContent></w:sdt></w:body>");
        assert_eq!(
            preferred_from_xml(&detached),
            None,
            "unknown container width"
        );
    }

    #[test]
    fn split_grid_wrapped_cells_and_rows_do_not_absorb_nested_table_edges() {
        let mut dom = Dom::new();
        let xml = format!(
            r#"<w:tbl xmlns:w="{}"><w:tblGrid><w:gridCol w:w="100"/><w:gridCol w:w="200"/><w:gridCol w:w="300"/></w:tblGrid><w:sdt><w:sdtContent><w:tr><w:sdt><w:sdtContent><w:tc><w:tcPr><w:tcW w:w="2500" w:type="pct"/><w:gridSpan w:val="2"/></w:tcPr><w:tbl><w:tblGrid><w:gridCol w:w="99"/><w:gridCol w:w="1"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl><w:p/></w:tc></w:sdtContent></w:sdt><w:tc><w:p/></w:tc></w:tr></w:sdtContent></w:sdt></w:tbl>"#,
            W::URI
        );
        let doc = dom.parse_xdocument(&xml);
        let table = dom.root(doc).unwrap();
        let grid = dom.element(table, &W::tbl_grid()).unwrap();
        let nested = dom.descendants(table, Some(&W::tbl()))[0];
        let before = dom.serialize_element(nested);
        rebuild_deleted_grid(&mut dom, table, grid);
        let widths: Vec<_> = dom
            .elements(grid, Some(&W::grid_col()))
            .iter()
            .map(|&col| dom.attribute(col, &W::name("w")).unwrap())
            .collect();
        assert_eq!(widths, ["300", "300"]);
        assert_eq!(dom.serialize_element(nested), before);
        let cells = dom.descendants(table, Some(&W::tc()));
        let pr = dom.element(cells[0], &W::tc_pr()).unwrap();
        assert!(dom.element(pr, &W::grid_span()).is_none());
        let width = dom.element(pr, &W::name("tcW")).unwrap();
        assert_eq!(dom.attribute(width, &W::name("w")), Some("2500"));
        assert_eq!(dom.attribute(width, &W::name("type")), Some("pct"));
    }

    #[test]
    fn split_grid_leaves_unusable_geometry_untouched() {
        let good = r#"<w:gridCol w:w="100"/><w:gridCol w:w="200"/>"#;
        for (grid, props, cells) in [
            ("", "", "<w:tc><w:p/></w:tc>"),
            (r#"<w:gridCol w:w="bad"/>"#, "", "<w:tc><w:p/></w:tc>"),
            (r#"<w:gridCol/>"#, "", "<w:tc><w:p/></w:tc>"),
            (r#"<w:gridCol w:w="0"/>"#, "", "<w:tc><w:p/></w:tc>"),
            (
                good,
                r#"<w:gridBefore w:val="bad"/>"#,
                "<w:tc><w:p/></w:tc>",
            ),
            (good, r#"<w:gridAfter w:val="bad"/>"#, "<w:tc><w:p/></w:tc>"),
            (good, "", ""),
            (
                good,
                "",
                r#"<w:tc><w:tcPr><w:gridSpan w:val="0"/></w:tcPr><w:p/></w:tc>"#,
            ),
            (
                good,
                "",
                r#"<w:tc><w:tcPr><w:gridSpan w:val="bad"/></w:tcPr><w:p/></w:tc>"#,
            ),
            (
                good,
                "",
                r#"<w:tc><w:tcPr><w:gridSpan w:val="3"/></w:tcPr><w:p/></w:tc>"#,
            ),
            (good, "", "<w:tc><w:p/></w:tc>"),
        ] {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&format!(r#"<w:tbl xmlns:w="{}"><w:tblGrid>{grid}</w:tblGrid><w:tr><w:trPr>{props}</w:trPr>{cells}</w:tr></w:tbl>"#, W::URI));
            let table = dom.root(doc).unwrap();
            let grid = dom.element(table, &W::tbl_grid()).unwrap();
            let before = dom.serialize_element(table);
            rebuild_deleted_grid(&mut dom, table, grid);
            assert_eq!(dom.serialize_element(table), before);
        }
    }

    /// A styles part whose "Normal Table" (default) and "Table Grid" styles
    /// go by the ids given.
    fn names(normal: &str, grid: &str) -> TableStyleNames {
        TableStyleNames::original(Some(&format!(
            r#"<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="{normal}"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="{grid}"><w:name w:val="Table Grid"/></w:style></w:styles>"#,
            W::URI
        )))
    }

    /// A paired table: the revision's `new` and the original's `old` table
    /// properties, a grid with its change, and rows `=` kept or `D` deleted.
    fn table(new: &str, old: &str, rows: &str) -> String {
        let rows: String = rows
            .chars()
            .map(|r| {
                let del = if r == 'D' {
                    r#"<w:trPr><w:del w:id="9" w:author="A" w:date="2026-10-27T00:00:00Z"/></w:trPr>"#
                } else {
                    ""
                };
                format!("<w:tr>{del}<w:tc><w:p/></w:tc></w:tr>")
            })
            .collect();
        format!(
            r#"<w:tbl><w:tblPr>{new}<w:tblPrChange w:id="1" w:author="A" w:date="2026-10-27T00:00:00Z"><w:tblPr>{old}</w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w="300"/><w:tblGridChange w:id="2"><w:tblGrid><w:gridCol w:w="900"/></w:tblGrid></w:tblGridChange></w:tblGrid>{rows}</w:tbl>"#
        )
    }

    /// Each body table after the pass: its rows' marks, its grid width, and
    /// its change records' ids.
    fn split(
        body: &str,
        old: &TableStyleNames,
        new: &TableStyleNames,
    ) -> Vec<(String, String, Vec<String>)> {
        let xml = format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:p/><w:sectPr/></w:body></w:document>"#,
            W::URI
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        split_rows_by_table_identity(&mut dom, root, old, new);
        let body = dom.element(root, &W::body()).unwrap();
        dom.elements(body, Some(&W::tbl()))
            .into_iter()
            .map(|t| {
                let marks = dom
                    .elements(t, Some(&W::tr()))
                    .into_iter()
                    .map(|r| {
                        if deleted_row(&dom, r) == Some(true) {
                            'D'
                        } else {
                            '='
                        }
                    })
                    .collect();
                let grid = dom.element(t, &W::name("tblGrid")).unwrap();
                let width = dom
                    .element(grid, &W::name("gridCol"))
                    .and_then(|c| dom.attribute(c, &W::name("w")))
                    .unwrap_or("")
                    .to_string();
                let ids = ["tblPrChange", "tblGridChange"]
                    .into_iter()
                    .flat_map(|n| dom.descendants(t, Some(&W::name(n))))
                    .filter_map(|c| dom.attribute(c, &W::id()).map(str::to_string))
                    .collect();
                (marks, width, ids)
            })
            .collect()
    }

    #[test]
    fn a_table_style_of_another_name_splits_the_deleted_rows_off() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let tables = split(&table(r#"<w:tblStyle w:val="Racs"/>"#, "", "==DD"), &a, &b);
        assert_eq!(
            tables,
            [
                ("==".into(), "300".into(), vec!["1".into(), "2".into()]),
                ("DD".into(), "900".into(), vec![]),
            ],
            "the deleted rows take the original's properties and grid"
        );
    }

    #[test]
    fn one_style_name_under_two_ids_keeps_the_rows_together() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let body = table(
            r#"<w:tblStyle w:val="Racs"/>"#,
            r#"<w:tblStyle w:val="Grille"/><w:tblInd w:w="284" w:type="dxa"/>"#,
            "==DD",
        );
        assert_eq!(
            split(&body, &a, &b),
            [("==DD".into(), "300".into(), vec!["1".into(), "2".into()])]
        );
    }

    #[test]
    fn a_fixed_layout_splits_each_run_and_the_copies_take_new_ids() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let body = table(r#"<w:tblLayout w:type="fixed"/>"#, "", "=D=");
        let tables = split(&body, &a, &b);
        assert_eq!(
            tables.iter().map(|t| t.0.as_str()).collect::<Vec<_>>(),
            ["=", "D", "="]
        );
        assert_eq!(tables[0].2, ["1", "2"]);
        assert!(tables[1].2.is_empty());
        assert_eq!(tables[2].2.len(), 2);
        assert!(
            tables[2]
                .2
                .iter()
                .all(|id| !["1", "2", "9"].contains(&id.as_str())),
            "fresh ids: {:?}",
            tables[2].2
        );
    }

    /// An id next to `u32::MAX` already in the part: the copies' ids wrap
    /// past it to unused ones instead of overflowing (Codex on #1921).
    #[test]
    fn fresh_ids_past_u32_max_wrap_to_unused_ones() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let body = table(r#"<w:tblLayout w:type="fixed"/>"#, "", "=D=D=")
            .replace(r#"w:id="1""#, r#"w:id="4294967294""#)
            .replace(r#"w:id="2""#, r#"w:id="0""#);
        let tables = split(&body, &a, &b);
        let ids: Vec<&String> = tables.iter().flat_map(|t| &t.2).collect();
        assert_eq!(ids.len(), 6, "{ids:?}");
        let unique: HashSet<&String> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert!(
            ids[2..].iter().all(|id| *id != "9"),
            "the deleted rows' id stays theirs: {ids:?}"
        );
    }

    #[test]
    fn a_part_without_table_styles_defaults_to_normal_table() {
        let a = TableStyleNames::original(Some(&format!(r#"<w:styles xmlns:w="{}"/>"#, W::URI)));
        let b = names("TableNormal", "TableGrid");
        let body = table(r#"<w:jc w:val="center"/>"#, "", "=DD");
        assert_eq!(
            split(&body, &a, &b),
            [("=DD".into(), "300".into(), vec!["1".into(), "2".into()])]
        );
        assert_eq!(TableStyleNames::revision(None).name(None), "Normal Table");
    }

    #[test]
    fn an_original_style_without_a_name_goes_by_its_id() {
        let a = TableStyleNames::original(Some(&format!(
            r#"<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:customStyle="1" w:styleId="a"/></w:styles>"#,
            W::URI
        )));
        assert_eq!(a.name(Some("a")), "a");
        assert_eq!(a.name(Some("missing")), "Normal Table");
        let b = names("TableNormal", "TableGrid");
        let body = table("", r#"<w:tblStyle w:val="a"/>"#, "=DD");
        let tables = split(&body, &a, &b);
        assert_eq!(
            tables.iter().map(|t| t.0.as_str()).collect::<Vec<_>>(),
            ["=", "DD"]
        );
    }

    #[test]
    fn a_revision_style_without_a_name_is_the_default() {
        // 70c30e926c: the revision's tables use unnamed styles "a0"..; Word
        // drops them from its redline and keeps the deleted rows of the
        // original's unstyled table in the paired table.
        let unnamed = format!(
            r#"<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:customStyle="1" w:styleId="a0"/></w:styles>"#,
            W::URI
        );
        let a = names("TableNormal", "TableGrid");
        let b = TableStyleNames::revision(Some(&unnamed));
        assert_eq!(b.name(Some("a0")), "Normal Table");
        let body = table(r#"<w:tblStyle w:val="a0"/>"#, "", "=DD");
        assert_eq!(
            split(&body, &a, &b),
            [("=DD".into(), "300".into(), vec!["1".into(), "2".into()])]
        );
    }

    #[test]
    fn a_bookmark_stays_with_the_row_before_it_and_a_wrapped_row_counts() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let del =
            r#"<w:trPr><w:del w:id="9" w:author="A" w:date="2026-10-27T00:00:00Z"/></w:trPr>"#;
        let body = format!(
            r#"<w:tbl><w:tblPr><w:tblStyle w:val="Racs"/><w:tblPrChange w:id="1" w:author="A" w:date="2026-10-27T00:00:00Z"><w:tblPr/></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w="300"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr><w:bookmarkStart w:id="5" w:name="b"/><w:sdt><w:sdtContent><w:tr>{del}<w:tc><w:p/></w:tc></w:tr></w:sdtContent></w:sdt><w:tr>{del}<w:tc><w:p/></w:tc></w:tr></w:tbl>"#
        );
        let xml = format!(
            r#"<w:document xmlns:w="{}"><w:body>{body}<w:p/><w:sectPr/></w:body></w:document>"#,
            W::URI
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        split_rows_by_table_identity(&mut dom, root, &a, &b);
        let body = dom.element(root, &W::body()).unwrap();
        let tables = dom.elements(body, Some(&W::tbl()));
        let kids = |t: NodeId| -> Vec<String> {
            dom.elements(t, None)
                .into_iter()
                .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
                .collect()
        };
        assert_eq!(tables.len(), 2);
        assert_eq!(kids(tables[0]), ["tblPr", "tblGrid", "tr", "bookmarkStart"]);
        assert_eq!(kids(tables[1]), ["tblPr", "tblGrid", "sdt", "tr"]);
        let grid = dom.element(tables[1], &W::name("tblGrid")).unwrap();
        assert!(
            dom.element(grid, &W::name("gridCol")).is_some(),
            "without a grid change the deleted rows keep the table's grid"
        );
    }

    #[test]
    fn a_table_without_a_change_record_is_left_as_it_is() {
        let (a, b) = (names("TableNormal", "Grille"), names("Normal", "Racs"));
        let body = r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid><w:gridCol w:w="300"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr><w:tr><w:trPr><w:del w:id="9" w:author="A" w:date="2026-10-27T00:00:00Z"/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#;
        assert_eq!(split(body, &a, &b), [("=D".into(), "300".into(), vec![])]);
    }
}
