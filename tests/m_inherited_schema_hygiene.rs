// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Three shapes the schema refuses come into a redline from its sources:
//! jubarte writes none of them, but carries them, live and in the property
//! snapshots its revisions keep (#1712). The redline normalizes them, as it
//! already supplies a missing `w:shd/@w:val`:
//!
//! - an empty `w:shd/@w:fill` (`ST_HexColor` has no empty value) goes, and
//!   the shading's other attributes stay;
//! - `w:gridCol/@w:type` (`CT_TblGridCol` declares only `w:w`) goes, and the
//!   width stays;
//! - `w:tblCellMar` and `w:tcMar` children take the schema's order: `top`,
//!   `left`, `start`, `bottom`, `right`, `end` (the Office 2019 validator
//!   refuses `start` before `left`).
//!
//! The text, the widths and the revisions do not change, and Accept All and
//! Reject All of the redline carry none of the three either.

mod common;

use common::docx::{Part, W_NS, docx_with, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

/// A one-cell table: `tbl_pr` and `grid` as given, the cell's `tc_pr`, and
/// one paragraph of `text` with `p_pr`/`r_pr`.
fn table(tbl_pr: &str, grid: &str, tc_pr: &str, p_pr: &str, r_pr: &str, text: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="2400" w:type="dxa"/>{tbl_pr}</w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/>{tc_pr}</w:tcPr><w:p><w:pPr>{p_pr}</w:pPr><w:r><w:rPr>{r_pr}</w:rPr><w:t>{text}</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#
    )
}

fn styles(inner: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{W_NS}">{inner}</w:styles>"#
    )
}

fn styled(body: &str, styles_xml: &str) -> Vec<u8> {
    docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: styles_xml,
        }],
    )
}

/// The redline of `a` against `b`, with its Accept All and Reject All, each
/// Word-valid: `(name, package)`.
fn resolutions(a: &[u8], b: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let redline = compare_documents(a, b, "Redline").expect("compare");
    assert_word_valid_package(&redline);
    let accepted = accept_revisions(&redline).expect("accept");
    let rejected = reject_revisions(&redline).expect("reject");
    assert_word_valid_package(&accepted);
    assert_word_valid_package(&rejected);
    vec![
        ("redline", redline),
        ("accept", accepted),
        ("reject", rejected),
    ]
}

fn parts(docx: &[u8], names: &[&str]) -> Vec<(String, String)> {
    names
        .iter()
        .filter_map(|&n| part_string(docx, n).map(|x| (n.to_string(), x)))
        .collect()
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

#[test]
fn an_empty_fill_goes_and_the_rest_of_the_shading_stays() {
    let dirty = r#"<w:shd w:val="clear" w:color="auto" w:fill=""/>"#;
    let themed = r#"<w:shd w:val="clear" w:color="auto" w:fill="" w:themeFill="accent1"/>"#;
    let style = styles(&format!(
        r#"<w:style w:type="paragraph" w:styleId="Shaded"><w:name w:val="Shaded"/><w:pPr>{dirty}</w:pPr></w:style>"#
    ));
    // Live in both: in the cell, the paragraph, the run and a style.
    let a = styled(
        &table(
            "",
            r#"<w:gridCol w:w="2400"/>"#,
            dirty,
            dirty,
            themed,
            "Old words.",
        ),
        &style,
    );
    let b = styled(
        &table(
            "",
            r#"<w:gridCol w:w="2400"/>"#,
            dirty,
            dirty,
            themed,
            "New words.",
        ),
        &style,
    );
    // Only in A's cell, whose shading B changes: the redline keeps it in the
    // tcPrChange snapshot, and Reject All brings that snapshot back.
    let filled = r#"<w:shd w:val="clear" w:color="auto" w:fill="FF0000"/>"#;
    let snap_a = docx_with(
        &table("", r#"<w:gridCol w:w="2400"/>"#, dirty, "", "", "Words."),
        &[],
    );
    let snap_b = docx_with(
        &table("", r#"<w:gridCol w:w="2400"/>"#, filled, "", "", "Words."),
        &[],
    );
    for (a, b) in [(&a, &b), (&snap_a, &snap_b)] {
        for (name, out) in resolutions(a, b) {
            for (part, xml) in parts(&out, &["word/document.xml", "word/styles.xml"]) {
                for shd in tags(&xml, "w:shd") {
                    assert!(!shd.contains(r#"w:fill="""#), "{name} {part}: {shd}");
                    assert!(shd.contains(r#"w:val="clear""#), "{name} {part}: {shd}");
                }
            }
        }
    }
    // The other attributes stay, the theme fill included.
    let redline = compare_documents(&a, &b, "Redline").expect("compare");
    let document = part_string(&redline, "word/document.xml").expect("document");
    assert!(
        tags(&document, "w:shd")
            .iter()
            .any(|s| s.contains(r#"w:themeFill="accent1""#)),
        "{document}"
    );
    assert!(
        tags(&document, "w:shd")
            .iter()
            .all(|s| s.contains(r#"w:color="auto""#)),
        "{document}"
    );
    let snapshot = compare_documents(&snap_a, &snap_b, "Redline").expect("compare");
    let document = part_string(&snapshot, "word/document.xml").expect("document");
    assert!(document.contains("<w:tcPrChange"), "{document}");
    assert!(document.contains(r#"w:fill="FF0000""#), "{document}");
}

#[test]
fn a_grid_column_keeps_only_its_width() {
    let col = |w: u32| format!(r#"<w:gridCol w:w="{w}" w:type="dxa"/>"#);
    let header = |w: u32| {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}">{}</w:hdr>"#,
            table("", &col(w), "", "", "", "Header cell.")
        )
    };
    let (ha, hb) = (header(2400), header(2400));
    let doc = |w: u32, hdr: &str, text: &str| {
        docx_with_sect(
            &table("", &col(w), "", "", "", text),
            &[Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: hdr,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        )
    };
    // B widens the column: the redline keeps A's grid in a tblGridChange.
    let a = doc(2400, &ha, "Old words.");
    let b = doc(3000, &hb, "New words.");
    for (name, out) in resolutions(&a, &b) {
        for (part, xml) in parts(&out, &["word/document.xml", "word/header1.xml"]) {
            for gc in tags(&xml, "w:gridCol") {
                assert!(!gc.contains("w:type="), "{name} {part}: {gc}");
            }
        }
    }
    let redline = compare_documents(&a, &b, "Redline").expect("compare");
    let document = part_string(&redline, "word/document.xml").expect("document");
    let widths: Vec<&str> = tags(&document, "w:gridCol");
    assert!(
        widths.iter().any(|g| g.contains(r#"w:w="3000""#)),
        "{document}"
    );
    assert!(document.contains("<w:tblGridChange"), "{document}");
    assert!(
        widths.iter().any(|g| g.contains(r#"w:w="2400""#)),
        "{document}"
    );
    let header = part_string(&redline, "word/header1.xml").expect("header");
    assert!(
        tags(&header, "w:gridCol")
            .iter()
            .all(|g| g.contains(r#"w:w="2400""#)),
        "{header}"
    );
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

#[test]
fn cell_margins_follow_the_schema_order() {
    let m = |side: &str, w: u32| format!(r#"<w:{side} w:w="{w}" w:type="dxa"/>"#);
    // Out of order, as the sources wrote them: right before left, start and
    // end after the physical sides.
    let scrambled = format!(
        "{}{}{}{}",
        m("right", 82),
        m("bottom", 83),
        m("left", 80),
        m("top", 81)
    );
    let logical = format!("{}{}{}", m("end", 91), m("top", 92), m("start", 90));
    let tbl_mar = format!("<w:tblCellMar>{scrambled}</w:tblCellMar>");
    let tc_mar = format!("<w:tcMar>{logical}</w:tcMar>");
    let style = styles(&format!(
        r#"<w:style w:type="table" w:styleId="Padded"><w:name w:val="Padded"/><w:tblPr><w:tblCellMar>{logical}{}</w:tblCellMar></w:tblPr></w:style>"#,
        m("left", 70)
    ));
    let grid = r#"<w:gridCol w:w="2400"/>"#;
    let a = styled(
        &table(&tbl_mar, grid, &tc_mar, "", "", "Old words."),
        &style,
    );
    let b = styled(
        &table(&tbl_mar, grid, &tc_mar, "", "", "New words."),
        &style,
    );
    // A's table margins only: the redline keeps them in a tblPrChange.
    let c = docx_with(&table(&tbl_mar, grid, "", "", "", "Words."), &[]);
    let d = docx_with(&table("", grid, "", "", "", "Words."), &[]);
    let rank = |side: &str| {
        ["top", "left", "start", "bottom", "right", "end"]
            .iter()
            .position(|s| *s == side)
    };
    for (a, b) in [(&a, &b), (&c, &d)] {
        for (name, out) in resolutions(a, b) {
            for (part, xml) in parts(&out, &["word/document.xml", "word/styles.xml"]) {
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
    }
    // Every width stays with its side.
    let redline = compare_documents(&a, &b, "Redline").expect("compare");
    let document = part_string(&redline, "word/document.xml").expect("document");
    for (side, w) in [
        ("left", 80),
        ("top", 81),
        ("right", 82),
        ("bottom", 83),
        ("start", 90),
        ("end", 91),
    ] {
        let width = format!(r#"<w:{side} w:w="{w}" w:type="dxa""#);
        assert!(document.contains(&width), "{side}: {document}");
    }
    let snapshot = compare_documents(&c, &d, "Redline").expect("compare");
    let document = part_string(&snapshot, "word/document.xml").expect("document");
    assert!(document.contains("<w:tblPrChange"), "{document}");
}

#[test]
fn cell_and_table_borders_keep_the_schema_order() {
    // Valid as the sources wrote them: `left` before `start`, `right` before
    // `end`. jubarte must not swap them (its cell-border ranks once put
    // `start` first, which the schema refuses).
    let b =
        |side: &str| format!(r#"<w:{side} w:val="single" w:sz="4" w:space="0" w:color="auto"/>"#);
    let sides = ["top", "left", "start", "bottom", "right", "end"]
        .map(b)
        .concat();
    let tc = format!("<w:tcBorders>{sides}</w:tcBorders>");
    let tbl = format!("<w:tblBorders>{sides}</w:tblBorders>");
    let grid = r#"<w:gridCol w:w="2400"/>"#;
    let a = docx_with(&table(&tbl, grid, &tc, "", "", "Old words."), &[]);
    let z = docx_with(&table(&tbl, grid, &tc, "", "", "New words."), &[]);
    let rank = |side: &str| {
        ["top", "left", "start", "bottom", "right", "end"]
            .iter()
            .position(|s| *s == side)
    };
    for (name, out) in resolutions(&a, &z) {
        let xml = part_string(&out, "word/document.xml").expect("document");
        for container in ["tcBorders", "tblBorders"] {
            let orders = margin_orders(&xml, container);
            assert!(!orders.is_empty(), "{name} {container}: {xml}");
            for order in orders {
                let ranks: Vec<_> = order.iter().map(|s| rank(s)).collect();
                assert!(
                    ranks.windows(2).all(|w| w[0] < w[1]),
                    "{name} {container}: {order:?}"
                );
            }
        }
    }
}
