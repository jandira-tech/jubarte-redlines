// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Structural edit operations: tables built in the clean copy, and the
//! style definitions they name.

use std::collections::BTreeSet;

use crate::inspect::Opened;
use crate::markdown::{package, xml};
use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

use super::{RunSpec, build_paragraph, check_text, is_range_markup};

/// Word's column limit.
const MAX_COLUMNS: usize = 63;
/// The widest cell Word accepts: 22 inches, in twentieths of a point.
const MAX_WIDTH_DXA: u32 = 31_680;
/// Table styles a document may lack that the engine can add: `(id, name)`.
const BUILTIN_TABLE_STYLES: &[(&str, &str)] = &[("TableGrid", "Table Grid")];

/// The column count of `rows`, checked: at least one row, every row the
/// same nonzero length, plain cell text, and `widths` (when given) one
/// positive width per column.
pub(super) fn check_rows(rows: &[Vec<String>], widths: Option<&[u32]>) -> Result<usize, String> {
    let Some(first) = rows.first() else {
        return Err("rows must hold at least one row".into());
    };
    let columns = first.len();
    if columns == 0 {
        return Err("row 0 has no cells".into());
    }
    if columns > MAX_COLUMNS {
        return Err(format!(
            "row 0 has {columns} cells; Word allows at most {MAX_COLUMNS} columns"
        ));
    }
    for (i, row) in rows.iter().enumerate() {
        if row.len() != columns {
            return Err(format!(
                "row {i} has {} cells, row 0 has {columns}: rows must not be ragged",
                row.len()
            ));
        }
        for (j, cell) in row.iter().enumerate() {
            check_text(cell).map_err(|m| format!("row {i} cell {j}: {m}"))?;
        }
    }
    if let Some(widths) = widths {
        if widths.len() != columns {
            return Err(format!(
                "widths_dxa has {} widths for {columns} columns",
                widths.len()
            ));
        }
        if widths.iter().any(|&w| w == 0 || w > MAX_WIDTH_DXA) {
            return Err(format!(
                "widths_dxa must lie in 1..={MAX_WIDTH_DXA} (twentieths of a point)"
            ));
        }
    }
    Ok(columns)
}

/// The body's text width: the final section's page width less its side
/// margins (Letter with 1-inch margins when unset).
pub(super) fn text_width(dom: &Dom, body: NodeId) -> u32 {
    let section = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&n| dom.name_is(n, &W::sect_pr()));
    let read = |element: &str, attribute: &str, default: u32| {
        section
            .and_then(|s| dom.element(s, &W::name(element)))
            .and_then(|e| dom.attribute(e, &W::name(attribute)))
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(default)
    };
    let width = read("pgSz", "w", 12_240);
    let margins = read("pgMar", "left", 1_440) + read("pgMar", "right", 1_440);
    width.saturating_sub(margins).max(1_440)
}

/// `(styleId, name)` of every table style in the document's styles part.
pub(super) fn table_styles(opened: &Opened) -> Vec<(String, String)> {
    let Some(part) = opened.related("styles").into_iter().next() else {
        return Vec::new();
    };
    let Some(xml) = opened.pkg.part_string(&part) else {
        return Vec::new();
    };
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(document) else {
        return Vec::new();
    };
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| dom.attribute(s, &W::name("type")) == Some("table"))
        .filter_map(|s| {
            let id = dom.attribute(s, &W::name("styleId"))?.to_string();
            let name = dom
                .element(s, &W::name("name"))
                .and_then(|n| dom.attribute(n, &W::val()))
                .unwrap_or("")
                .to_string();
            Some((id, name))
        })
        .collect()
}

/// A table style id from its id or name (case-insensitive) among the
/// document's table styles, or a built-in one the engine adds; the flag is
/// true when the definition must be added.
pub(super) fn resolve_table_style(
    styles: &[(String, String)],
    requested: &str,
) -> Result<(String, bool), String> {
    if styles.iter().any(|(id, _)| id == requested) {
        return Ok((requested.to_string(), false));
    }
    if let Some((id, _)) = styles
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(requested))
    {
        return Ok((id.clone(), false));
    }
    if let Some((id, _)) = BUILTIN_TABLE_STYLES.iter().find(|(id, name)| {
        id.eq_ignore_ascii_case(requested) || name.eq_ignore_ascii_case(requested)
    }) {
        return Ok(((*id).to_string(), true));
    }
    let known: Vec<&str> = styles
        .iter()
        .map(|(id, _)| id.as_str())
        .chain(BUILTIN_TABLE_STYLES.iter().map(|(id, _)| *id))
        .take(12)
        .collect();
    Err(format!(
        "no table style has id or name {requested:?}; available: {}",
        known.join(", ")
    ))
}

/// A `w:tbl` of `rows`: `style`, a DXA width that is the sum of `widths`,
/// a grid column and a DXA `w:tcW` per column, `w:tblHeader` on the first
/// row when `header_row`, and one paragraph per cell that takes the
/// anchor's paragraph style and run formatting.
pub(super) fn build_table(
    dom: &mut Dom,
    anchor: NodeId,
    rows: &[Vec<String>],
    header_row: bool,
    widths: &[u32],
    style: &str,
) -> NodeId {
    let element = |dom: &mut Dom, parent: NodeId, local: &str, attributes: &[(&str, &str)]| {
        let node = dom.new_element(W::name(local));
        for (name, value) in attributes {
            dom.set_attribute_value(node, &W::name(name), Some(value));
        }
        dom.add(parent, node);
        node
    };
    let table = dom.new_element(W::tbl());
    let properties = element(dom, table, "tblPr", &[]);
    element(dom, properties, "tblStyle", &[("val", style)]);
    let total: u32 = widths.iter().sum();
    element(
        dom,
        properties,
        "tblW",
        &[("w", &total.to_string()), ("type", "dxa")],
    );
    element(
        dom,
        properties,
        "tblLook",
        &[
            ("val", "04A0"),
            ("firstRow", "1"),
            ("lastRow", "0"),
            ("firstColumn", "1"),
            ("lastColumn", "0"),
            ("noHBand", "0"),
            ("noVBand", "1"),
        ],
    );
    let grid = element(dom, table, "tblGrid", &[]);
    for width in widths {
        element(dom, grid, "gridCol", &[("w", &width.to_string())]);
    }
    for (i, row) in rows.iter().enumerate() {
        let tr = element(dom, table, "tr", &[]);
        if header_row && i == 0 {
            let tr_pr = element(dom, tr, "trPr", &[]);
            element(dom, tr_pr, "tblHeader", &[]);
        }
        for (cell, width) in row.iter().zip(widths) {
            let tc = element(dom, tr, "tc", &[]);
            let tc_pr = element(dom, tc, "tcPr", &[]);
            element(
                dom,
                tc_pr,
                "tcW",
                &[("w", &width.to_string()), ("type", "dxa")],
            );
            let p = cell_paragraph(dom, anchor, cell);
            dom.add(tc, p);
        }
    }
    table
}

/// A cell paragraph modeled on `anchor`, keeping only its paragraph style.
fn cell_paragraph(dom: &mut Dom, anchor: NodeId, text: &str) -> NodeId {
    let spec = RunSpec {
        text: text.to_string(),
        ..RunSpec::default()
    };
    let p = build_paragraph(dom, anchor, &[spec], None);
    if let Some(ppr) = dom.element(p, &W::p_pr()) {
        for child in dom.elements(ppr, None) {
            if !dom.name_is(child, &W::p_style()) {
                dom.remove(child);
            }
        }
        if dom.elements(ppr, None).is_empty() {
            dom.remove(ppr);
        }
    }
    p
}

/// Give `table` a paragraph after it unless one follows, and one before it
/// when a table precedes it: a story must not end in a table, and Word
/// joins two tables that touch.
pub(super) fn separate(dom: &mut Dom, table: NodeId) {
    let mut next = dom.next_element(table);
    while let Some(n) = next.filter(|&n| is_range_markup(dom, n)) {
        next = dom.next_element(n);
    }
    if !next.is_some_and(|n| dom.name_is(n, &W::p())) {
        let p = dom.new_element(W::p());
        dom.add_after_self(table, p);
    }
    let previous = dom
        .nodes_before_self(table)
        .into_iter()
        .rev()
        .filter(|&n| dom.is_element(n))
        .find(|&n| !is_range_markup(dom, n));
    if previous.is_some_and(|n| dom.name_is(n, &W::tbl())) {
        let p = dom.new_element(W::p());
        dom.add_before_self(table, p);
    }
}

/// Add the definitions of `wanted` styles the styles part lacks (making
/// the part when there is none). A definition's base style that is
/// missing is replaced by the document's default style of that type, or
/// added when the document has no default.
pub(super) fn add_styles(pkg: &mut PartFs, main: &str, wanted: &BTreeSet<String>) {
    let part = package::styles_part(pkg, main);
    if let Some(updated) = pkg
        .part_string(&part)
        .and_then(|source| with_definitions(&source, wanted))
    {
        pkg.set_part(&part, updated.into_bytes());
    }
}

/// `source` (a styles part) with the definitions [`add_styles`] adds, or
/// `None` when it adds none.
fn with_definitions(source: &str, wanted: &BTreeSet<String>) -> Option<String> {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(source);
    let root = dom.root(document)?;
    let style_name = W::name("style");
    let id_name = W::name("styleId");
    let type_name = W::name("type");
    let mut ids: BTreeSet<String> = dom
        .elements(root, Some(&style_name))
        .into_iter()
        .filter_map(|s| dom.attribute(s, &id_name).map(str::to_string))
        .collect();
    let defaults: Vec<(String, String)> = dom
        .elements(root, Some(&style_name))
        .into_iter()
        .filter(|&s| {
            matches!(
                dom.attribute(s, &W::name("default")),
                Some("1" | "true" | "on")
            )
        })
        .filter_map(|s| {
            Some((
                dom.attribute(s, &type_name)?.to_string(),
                dom.attribute(s, &id_name)?.to_string(),
            ))
        })
        .collect();
    let mut changed = false;
    for id in wanted {
        if ids.contains(id) {
            continue;
        }
        let Some(style) = definition(&mut dom, id) else {
            continue;
        };
        let based_on = dom.element(style, &W::name("basedOn"));
        let base = based_on.and_then(|b| dom.attribute(b, &W::val()).map(str::to_string));
        if let (Some(based_on), Some(base)) = (based_on, base)
            && !ids.contains(&base)
        {
            let kind = dom.attribute(style, &type_name).unwrap_or("paragraph");
            match defaults.iter().find(|(t, _)| t == kind) {
                Some((_, default)) => {
                    dom.set_attribute_value(based_on, &W::val(), Some(default));
                }
                None => {
                    if let Some(base_style) = definition(&mut dom, &base) {
                        dom.add(root, base_style);
                        ids.insert(base);
                    } else {
                        dom.remove(based_on);
                    }
                }
            }
        }
        dom.add(root, style);
        ids.insert(id.clone());
        changed = true;
    }
    changed.then(|| dom.serialize_document(document))
}

/// The engine's definition of style `id`, parsed into `dom` and detached.
fn definition(dom: &mut Dom, id: &str) -> Option<NodeId> {
    let xml = xml::style_definition(id)?;
    let document = dom.parse_xdocument(&format!(
        "<w:styles xmlns:w=\"{}\">{xml}</w:styles>",
        W::URI
    ));
    let root = dom.root(document)?;
    let style = dom.element(root, &W::name("style"))?;
    dom.remove(style);
    Some(style)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(cells: &[&[&str]]) -> Vec<Vec<String>> {
        cells
            .iter()
            .map(|r| r.iter().map(|c| (*c).to_string()).collect())
            .collect()
    }

    #[test]
    fn rows_are_checked_for_shape_and_widths() {
        assert_eq!(check_rows(&rows(&[&["a", "b"], &["c", "d"]]), None), Ok(2));
        assert!(
            check_rows(&[], None)
                .unwrap_err()
                .contains("at least one row")
        );
        assert!(
            check_rows(&rows(&[&[]]), None)
                .unwrap_err()
                .contains("no cells")
        );
        assert!(
            check_rows(&rows(&[&["a"], &["b", "c"]]), None)
                .unwrap_err()
                .contains("row 1 has 2 cells")
        );
        assert!(
            check_rows(&rows(&[&["a\u{7}"]]), None)
                .unwrap_err()
                .contains("row 0 cell 0")
        );
        let wide = vec![vec![String::new(); MAX_COLUMNS + 1]];
        assert!(check_rows(&wide, None).unwrap_err().contains("at most 63"));
        assert!(
            check_rows(&rows(&[&["a", "b"]]), Some(&[1]))
                .unwrap_err()
                .contains("1 widths for 2 columns")
        );
        assert!(check_rows(&rows(&[&["a"]]), Some(&[MAX_WIDTH_DXA + 1])).is_err());
        assert_eq!(check_rows(&rows(&[&["a"]]), Some(&[MAX_WIDTH_DXA])), Ok(1));
    }

    #[test]
    fn table_styles_resolve_by_id_name_or_builtin() {
        let styles = vec![("Fancy".to_string(), "Fancy Table".to_string())];
        assert_eq!(
            resolve_table_style(&styles, "Fancy"),
            Ok(("Fancy".into(), false))
        );
        assert_eq!(
            resolve_table_style(&styles, "FANCY TABLE"),
            Ok(("Fancy".into(), false))
        );
        assert_eq!(
            resolve_table_style(&styles, "table grid"),
            Ok(("TableGrid".into(), true))
        );
        let refused = resolve_table_style(&styles, "Nope").unwrap_err();
        assert!(refused.contains("Fancy, TableGrid"), "{refused}");
    }

    #[test]
    fn text_width_reads_the_final_section() {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            "<w:body xmlns:w=\"{}\"><w:p/><w:sectPr><w:pgSz w:w=\"11906\"/>\
             <w:pgMar w:left=\"1000\" w:right=\"906\"/></w:sectPr></w:body>",
            W::URI
        ));
        let body = dom.root(document).unwrap();
        assert_eq!(text_width(&dom, body), 10_000);
        let bare = dom.parse_xdocument(&format!("<w:body xmlns:w=\"{}\"/>", W::URI));
        let bare = dom.root(bare).unwrap();
        assert_eq!(text_width(&dom, bare), 9_360);
    }

    #[test]
    fn a_missing_base_follows_the_documents_default_style() {
        let source = format!(
            "<w:styles xmlns:w=\"{}\"><w:style w:type=\"table\" w:default=\"1\" \
             w:styleId=\"Plain\"><w:name w:val=\"Normal Table\"/></w:style></w:styles>",
            W::URI
        );
        let xml = with_definitions(&source, &BTreeSet::from(["TableGrid".to_string()])).unwrap();
        assert!(xml.contains("w:styleId=\"TableGrid\""), "{xml}");
        assert!(xml.contains("<w:basedOn w:val=\"Plain\""), "{xml}");
        assert!(!xml.contains("TableNormal"), "{xml}");
    }

    #[test]
    fn present_styles_are_left_alone() {
        let source = format!(
            "<w:styles xmlns:w=\"{}\"><w:style w:type=\"table\" w:styleId=\"TableGrid\">\
             <w:name w:val=\"Mine\"/></w:style></w:styles>",
            W::URI
        );
        let wanted = BTreeSet::from(["TableGrid".to_string(), "NotAStyle".to_string()]);
        assert_eq!(with_definitions(&source, &wanted), None);
    }

    #[test]
    fn a_missing_base_without_a_default_is_added() {
        let source = format!("<w:styles xmlns:w=\"{}\"/>", W::URI);
        let wanted = BTreeSet::from(["TableGrid".to_string(), "ListParagraph".to_string()]);
        let xml = with_definitions(&source, &wanted).unwrap();
        for id in ["TableGrid", "TableNormal", "ListParagraph", "Normal"] {
            assert_eq!(
                xml.matches(&format!("w:styleId=\"{id}\"")).count(),
                1,
                "{id}: {xml}"
            );
        }
        // Every base precedes the style built on it.
        assert!(xml.find("\"TableNormal\"").unwrap() < xml.find("\"TableGrid\"").unwrap());
    }
}
