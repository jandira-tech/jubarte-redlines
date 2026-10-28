// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1914 — a kept row that pairs the revision's one cell with the
//! original's two keeps the original's surplus cell, and the grid rebuilt
//! for that extra column keeps the width the revision's table declares, as
//! in Word's redline.
//!
//! Word 16.115's Compare of the pairs below (jubarte-loop/fix23/r55/syn,
//! 2026-10-27), the shape of 2fa06a6cb0's four question boxes: the
//! original's table holds two columns, the revision's one fixed column
//! 9809 twips wide.
//! - The revision's `tblW` stays, a width or a percentage. Only a table
//!   without one gets `tblW 0 auto`.
//! - The rebuilt grid splits a width in twips evenly: 9809 → 4905 + 4904.
//! - The original's surplus cell takes the cell properties of the
//!   revision's cell before it, all but its span, and records its own
//!   (`tcW 7422`) in a `tcPrChange`.
//!
//! Before, the grid took the text width and the table `tblW 0 auto`, and
//! the surplus cell kept its 7422. The first cell's 9809 then pushed it
//! past the page edge in our PDF.

mod common;

use common::docx::{docx_with_sect_pr, esc};
use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};

/// A4 with 850-twip side margins: a text width of 10206.
const SECT: &str = r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="850" w:bottom="1440" w:left="850" w:header="708" w:footer="708" w:gutter="0"/></w:sectPr>"#;

fn cell(props: &str, text: &str) -> String {
    let p = if text.is_empty() {
        "<w:p/>".to_string()
    } else {
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
            esc(text)
        )
    };
    format!("<w:tc><w:tcPr>{props}</w:tcPr>{p}</w:tc>")
}

fn tcw(w: u32) -> String {
    format!(r#"<w:tcW w:w="{w}" w:type="dxa"/>"#)
}

fn document(table: &str) -> Vec<u8> {
    let p = |t: &str| format!(r#"<w:p><w:r><w:t>{t}</w:t></w:r></w:p>"#);
    docx_with_sect_pr(
        &format!(
            "{}{table}{}",
            p("Which document are you commenting on?"),
            p("Thank you.")
        ),
        &[],
        SECT,
    )
}

/// The original: an autofit table of two columns, 1101 + 7422.
fn original() -> Vec<u8> {
    let rows: String = ["Submission Plan", "Consultation Statement", "Other"]
        .iter()
        .map(|t| {
            format!(
                "<w:tr>{}{}</w:tr>",
                cell(&tcw(1101), ""),
                cell(&tcw(7422), t)
            )
        })
        .collect();
    document(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="1101"/><w:gridCol w:w="7422"/></w:tblGrid>{rows}</w:tbl>"#
    ))
}

/// The revision: one empty cell 9809 wide under the table properties `pr`.
fn revision(pr: &str) -> Vec<u8> {
    document(&format!(
        r#"<w:tbl><w:tblPr>{pr}</w:tblPr><w:tblGrid><w:gridCol w:w="9809"/></w:tblGrid><w:tr>{}</w:tr></w:tbl>"#,
        cell(&tcw(9809), "")
    ))
}

/// The first table of a redline's body.
struct Table {
    /// `tblW`'s `w` and `type`, outside the change record.
    width: Option<(String, String)>,
    grid: Vec<u32>,
    /// The first row's cells: live `tcW`, the `tcPrChange`'s `tcW`, the
    /// live `gridSpan` and `vAlign`.
    cells: Vec<Cell>,
}

#[derive(Debug, PartialEq)]
struct Cell {
    width: Option<u32>,
    recorded: Option<u32>,
    span: Option<u32>,
    v_align: Option<String>,
}

fn first_table(docx: &[u8]) -> Table {
    let pkg = PartFs::open(docx).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let tbl = dom.element(body, &W::tbl()).expect("a table");
    let val = |dom: &Dom, parent: NodeId, name: &str, attr: &str| -> Option<String> {
        let el = dom.element(parent, &W::name(name))?;
        dom.attribute(el, &W::name(attr)).map(|v| v.to_string())
    };
    let number = |v: Option<String>| v.and_then(|v| v.parse::<u32>().ok());
    let pr = dom.element(tbl, &W::tbl_pr()).unwrap();
    let width = dom.element(pr, &W::name("tblW")).map(|w| {
        let a = |n: &str| {
            dom.attribute(w, &W::name(n))
                .unwrap_or_default()
                .to_string()
        };
        (a("w"), a("type"))
    });
    let grid_el = dom.element(tbl, &W::name("tblGrid")).unwrap();
    let grid = dom
        .elements(grid_el, Some(&W::grid_col()))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("w"))?.parse().ok())
        .collect();
    let tr = dom.element(tbl, &W::tr()).unwrap();
    let cells = dom
        .elements(tr, Some(&W::tc()))
        .into_iter()
        .map(|tc| {
            let pr = dom.element(tc, &W::tc_pr()).unwrap();
            let record = dom
                .element(pr, &W::name("tcPrChange"))
                .and_then(|c| dom.element(c, &W::tc_pr()));
            Cell {
                width: number(val(&dom, pr, "tcW", "w")),
                recorded: record.and_then(|r| number(val(&dom, r, "tcW", "w"))),
                span: number(val(&dom, pr, "gridSpan", "val")),
                v_align: val(&dom, pr, "vAlign", "val"),
            }
        })
        .collect();
    Table { width, grid, cells }
}

fn compare(a: &[u8], b: &[u8]) -> Table {
    let redline =
        compare_documents_with_settings(a, b, &WmlComparerSettings::default()).expect("compare");
    assert_word_valid_package(&redline);
    first_table(&redline)
}

fn cell_of(width: u32, recorded: u32) -> Cell {
    Cell {
        width: Some(width),
        recorded: Some(recorded),
        span: None,
        v_align: None,
    }
}

const FIXED: &str = r#"<w:tblInd w:w="497" w:type="dxa"/><w:tblLayout w:type="fixed"/>"#;

#[test]
fn a_width_in_twips_stays_and_the_grid_splits_it() {
    let t = compare(
        &original(),
        &revision(&format!(r#"<w:tblW w:w="9809" w:type="dxa"/>{FIXED}"#)),
    );
    assert_eq!(t.width, Some(("9809".into(), "dxa".into())));
    assert_eq!(t.grid, [4905, 4904]);
    assert_eq!(t.cells, [cell_of(9809, 1101), cell_of(9809, 7422)]);
}

#[test]
fn an_autofit_table_keeps_its_width_too() {
    // Word lays the two columns out by their content (4544 + 5265); the
    // total is the declared width either way.
    let t = compare(
        &original(),
        &revision(r#"<w:tblW w:w="9809" w:type="dxa"/><w:tblInd w:w="497" w:type="dxa"/>"#),
    );
    assert_eq!(t.width, Some(("9809".into(), "dxa".into())));
    assert_eq!(t.grid.iter().sum::<u32>(), 9809);
    assert_eq!(t.cells, [cell_of(9809, 1101), cell_of(9809, 7422)]);
}

#[test]
fn a_percentage_stays() {
    // 80% of the text width; Word's grid is 4091 + 4090, as it counts the
    // cells' side margins into the width the percentage takes.
    let t = compare(
        &original(),
        &revision(&format!(r#"<w:tblW w:w="4000" w:type="pct"/>{FIXED}"#)),
    );
    assert_eq!(t.width, Some(("4000".into(), "pct".into())));
    assert_eq!(t.grid, [4083, 4082]);
    assert_eq!(t.cells, [cell_of(9809, 1101), cell_of(9809, 7422)]);
}

#[test]
fn a_table_without_a_width_gets_auto() {
    // Word's grid here is the cells' own widths, 9809 + 9809; ours splits
    // the text width.
    let t = compare(&original(), &revision(FIXED));
    assert_eq!(t.width, Some(("0".into(), "auto".into())));
    assert_eq!(t.cells, [cell_of(9809, 1101), cell_of(9809, 7422)]);
}

#[test]
fn the_copied_properties_leave_the_span() {
    // The revision's one cell spans its two columns and is centred. Word
    // writes both cells 9809 wide and centred, neither spanning. Its grid
    // is 4905 + 4904; ours keeps the revision's 4000 + 5809.
    let spanning = cell(
        &format!(
            r#"{}<w:gridSpan w:val="2"/><w:vAlign w:val="center"/>"#,
            tcw(9809)
        ),
        "",
    );
    let b = document(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="9809" w:type="dxa"/>{FIXED}</w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="5809"/></w:tblGrid><w:tr>{spanning}</w:tr></w:tbl>"#
    ));
    let t = compare(&original(), &b);
    assert_eq!(t.width, Some(("9809".into(), "dxa".into())));
    let centred = |recorded| Cell {
        width: Some(9809),
        recorded: Some(recorded),
        span: None,
        v_align: Some("center".into()),
    };
    assert_eq!(t.cells, [centred(1101), centred(7422)]);
}
