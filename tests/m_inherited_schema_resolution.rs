// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The three shapes #1712 keeps out of redlines must not survive the three
//! paths that still carried them (#1717, option B): standalone accept,
//! standalone reject and self-compare, including the `tcPrChange`,
//! `tblGridChange` and `tblPrChange` snapshots Reject All of an older
//! redline restores. The library's `accept_revisions`/`reject_revisions`
//! are the paths behind `jubarte accept`/`jubarte reject`, the Python and
//! the WASM bindings; every one of them funnels into
//! `revision_processor::resolve_package`.
//!
//! Parts a resolution never writes (the glossary, #1996) stay as they came;
//! a clean source keeps its bytes.

mod common;

use common::docx::{Part, W_NS, docx_with, docx_with_sect, part_string};
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

const EMPTY_FILL: &str = r#"<w:shd w:val="clear" w:color="auto" w:fill=""/>"#;
const TYPED_GRID: &str = r#"<w:gridCol w:w="2400" w:type="dxa"/>"#;
const CLEAN_GRID: &str = r#"<w:gridCol w:w="2400"/>"#;

/// Out of order as the sources wrote them: right, bottom, left, top.
const SCRAMBLED: &str = r#"<w:right w:w="82" w:type="dxa"/><w:bottom w:w="83" w:type="dxa"/><w:left w:w="80" w:type="dxa"/><w:top w:w="81" w:type="dxa"/>"#;
/// Logically ordered but schema-invalid: end, top, start.
const LOGICAL: &str = r#"<w:end w:w="91" w:type="dxa"/><w:top w:w="92" w:type="dxa"/><w:start w:w="90" w:type="dxa"/>"#;

/// A one-cell table: `tbl_pr` and `grid` as given, the cell's `tc_pr`, and
/// one paragraph of `text`.
fn table(tbl_pr: &str, grid: &str, tc_pr: &str, text: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="2400" w:type="dxa"/>{tbl_pr}</w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/>{tc_pr}</w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#
    )
}

/// A tracked insertion beside the table, so a resolution has something to
/// settle and the package is a redline an older build could have made.
const INS: &str = r#"<w:p><w:ins w:id="1" w:author="Ann" w:date="2026-01-01T00:00:00Z"><w:r><w:t>New words.</w:t></w:r></w:ins></w:p>"#;

/// One of the three inherited shapes, placed live in the body, a header and
/// a style (a table style for the margins, a paragraph style for the fill).
fn shape_source(grid: &str, tbl_mar: &str, tc_mar: &str, shd: &str) -> Vec<u8> {
    let style = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Shaded"><w:name w:val="Shaded"/><w:pPr>{shd}</w:pPr></w:style><w:style w:type="table" w:styleId="Padded"><w:name w:val="Padded"/><w:tblPr><w:tblCellMar>{tbl_mar}</w:tblCellMar></w:tblPr></w:style></w:styles>"#
    );
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}">{}</w:hdr>"#,
        table("", grid, tc_mar, "Header cell.")
    );
    let body = format!(
        "{}{}",
        table(
            &format!("<w:tblCellMar>{tbl_mar}</w:tblCellMar>"),
            grid,
            &format!("{shd}<w:tcMar>{tc_mar}</w:tcMar>"),
            "Body cell."
        ),
        INS
    );
    docx_with_sect(
        &body,
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: &style,
            },
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
    )
}

/// The empty-fill source: the shape twice in the body cell and once in a
/// paragraph style.
fn empty_fill_source() -> Vec<u8> {
    shape_source(CLEAN_GRID, "", "", &format!("{EMPTY_FILL}{EMPTY_FILL}"))
}

/// The gridCol-type source: body grid and header grid.
fn grid_type_source() -> Vec<u8> {
    shape_source(TYPED_GRID, "", "", "")
}

/// The margins source: scrambled `tblCellMar` and logical `tcMar` in the
/// body, the header and a table style.
fn margins_source() -> Vec<u8> {
    shape_source(CLEAN_GRID, SCRAMBLED, LOGICAL, "")
}

/// An older redline: live properties clean, the three table-property
/// snapshots dirty (`tcPrChange`, `tblGridChange`, `tblPrChange`), each
/// holding one of the shapes Reject All restores.
fn snapshot_source() -> Vec<u8> {
    let date = r#"w:date="2026-01-01T00:00:00Z""#;
    let body = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="2400" w:type="dxa"/><w:tblPrChange w:id="10" w:author="Old" {date}><w:tblPr><w:tblCellMar>{SCRAMBLED}</w:tblCellMar></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid>{CLEAN_GRID}<w:tblGridChange w:id="11" w:author="Old" {date}><w:gridCol w:w="2400" w:type="dxa"/></w:tblGridChange></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/><w:tcPrChange w:id="12" w:author="Old" {date}><w:tcPr>{EMPTY_FILL}<w:tcMar>{LOGICAL}</w:tcMar></w:tcPr></w:tcPrChange></w:tcPr><w:p><w:r><w:t>Cell.</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>{INS}"#
    );
    docx_with(&body, &[])
}

/// Every start tag named `tag` (`w:shd`) in `xml`, attributes included.
fn tags<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(&open) {
        let tail = &rest[at..];
        let next = tail.as_bytes().get(open.len()).copied();
        let end = tail.find('>').expect("tag closes") + 1;
        if matches!(next, Some(b' ' | b'/' | b'>')) {
            out.push(&tail[..end]);
        }
        rest = &tail[end..];
    }
    out
}

/// The children of each `<w:{container}>` in `xml`, by local name.
fn margin_orders(xml: &str, container: &str) -> Vec<Vec<String>> {
    let open = format!("<w:{container}>");
    let close = format!("</w:{container}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(&open) {
        let inner = &rest[at + open.len()..];
        let end = inner.find(&close).expect("margins close");
        out.push(
            inner[..end]
                .split("<w:")
                .skip(1)
                .map(|t| t.split([' ', '/', '>']).next().unwrap_or("").to_string())
                .collect(),
        );
        rest = &inner[end..];
    }
    out
}

fn rank(side: &str) -> Option<usize> {
    ["top", "left", "start", "bottom", "right", "end"]
        .iter()
        .position(|s| *s == side)
}

/// `(name, package)` for the three paths of #1717, each built from `source`.
fn paths(source: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("accept", accept_revisions(source).expect("accept")),
        ("reject", reject_revisions(source).expect("reject")),
        (
            "self-compare",
            compare_documents(source, source, "Me").expect("self"),
        ),
    ]
}

fn parts(docx: &[u8]) -> Vec<(&'static str, String)> {
    ["word/document.xml", "word/header1.xml", "word/styles.xml"]
        .into_iter()
        .filter_map(|n| part_string(docx, n).map(|x| (n, x)))
        .collect()
}

fn assert_no_empty_fill(name: &str, docx: &[u8]) {
    for (part, xml) in parts(docx) {
        for shd in tags(&xml, "w:shd") {
            assert!(!shd.contains(r#"w:fill="""#), "{name} {part}: {shd}");
            assert!(shd.contains(r#"w:val="clear""#), "{name} {part}: {shd}");
            assert!(shd.contains(r#"w:color="auto""#), "{name} {part}: {shd}");
        }
    }
}

fn assert_no_grid_type(name: &str, docx: &[u8]) {
    for (part, xml) in parts(docx) {
        for gc in tags(&xml, "w:gridCol") {
            assert!(!gc.contains("w:type="), "{name} {part}: {gc}");
            assert!(gc.contains(r#"w:w="2400""#), "{name} {part}: {gc}");
        }
    }
}

fn assert_margins_ordered(name: &str, docx: &[u8]) {
    for (part, xml) in parts(docx) {
        for container in ["tblCellMar", "tcMar"] {
            for order in margin_orders(&xml, container) {
                let ranks: Vec<_> = order.iter().map(|s| rank(s)).collect();
                assert!(
                    ranks.iter().all(Option::is_some),
                    "{name} {part}: {order:?}"
                );
                assert!(
                    ranks.windows(2).all(|w| w[0] < w[1]),
                    "{name} {part} {container}: {order:?}"
                );
            }
        }
    }
}

#[test]
fn accept_loses_an_empty_fill() {
    for (name, out) in paths(&empty_fill_source()) {
        if name != "accept" {
            continue;
        }
        assert_no_empty_fill(name, &out);
    }
}

#[test]
fn reject_loses_an_empty_fill() {
    for (name, out) in paths(&empty_fill_source()) {
        if name != "reject" {
            continue;
        }
        assert_no_empty_fill(name, &out);
    }
}

#[test]
fn self_compare_loses_an_empty_fill() {
    for (name, out) in paths(&empty_fill_source()) {
        if name != "self-compare" {
            continue;
        }
        assert_no_empty_fill(name, &out);
    }
}

#[test]
fn accept_drops_a_grid_column_type() {
    for (name, out) in paths(&grid_type_source()) {
        if name != "accept" {
            continue;
        }
        assert_no_grid_type(name, &out);
    }
}

#[test]
fn reject_drops_a_grid_column_type() {
    for (name, out) in paths(&grid_type_source()) {
        if name != "reject" {
            continue;
        }
        assert_no_grid_type(name, &out);
    }
}

#[test]
fn self_compare_drops_a_grid_column_type() {
    for (name, out) in paths(&grid_type_source()) {
        if name != "self-compare" {
            continue;
        }
        assert_no_grid_type(name, &out);
    }
}

#[test]
fn accept_orders_cell_margins() {
    for (name, out) in paths(&margins_source()) {
        if name != "accept" {
            continue;
        }
        assert_margins_ordered(name, &out);
    }
}

#[test]
fn reject_orders_cell_margins() {
    for (name, out) in paths(&margins_source()) {
        if name != "reject" {
            continue;
        }
        assert_margins_ordered(name, &out);
    }
}

#[test]
fn self_compare_orders_cell_margins() {
    for (name, out) in paths(&margins_source()) {
        if name != "self-compare" {
            continue;
        }
        assert_margins_ordered(name, &out);
    }
}

/// Reject All of an older redline restores the dirty snapshots as live
/// properties; accept keeps the clean live ones and drops the snapshots.
/// Both outputs must be free of the three shapes.
#[test]
fn resolutions_repair_restored_table_property_snapshots() {
    let source = snapshot_source();
    let rejected = reject_revisions(&source).expect("reject");
    let document = part_string(&rejected, "word/document.xml").expect("document");
    assert!(
        !document.contains("tcPrChange") && !document.contains("tblPrChange"),
        "reject restores the snapshots: {document}"
    );
    assert_no_empty_fill("reject", &rejected);
    assert_no_grid_type("reject", &rejected);
    assert_margins_ordered("reject", &rejected);
    // Every restored value stays with its side.
    for (side, w) in [
        ("left", 80),
        ("top", 81),
        ("right", 82),
        ("bottom", 83),
        ("start", 90),
        ("end", 91),
    ] {
        let width = format!(r#"<w:{side} w:w="{w}" w:type="dxa""#);
        assert!(document.contains(&width), "reject {side}: {document}");
    }

    let accepted = accept_revisions(&source).expect("accept");
    let document = part_string(&accepted, "word/document.xml").expect("document");
    assert!(
        !document.contains("PrChange") && !document.contains("tblGridChange"),
        "accept drops the snapshots: {document}"
    );
    assert_no_empty_fill("accept", &accepted);
    assert_no_grid_type("accept", &accepted);
    assert_margins_ordered("accept", &accepted);

    for (name, out) in paths(&source) {
        if name != "self-compare" {
            continue;
        }
        assert_no_empty_fill(name, &out);
        assert_no_grid_type(name, &out);
        assert_margins_ordered(name, &out);
    }
}
