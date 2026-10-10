// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Short paragraph ids (`p3`, `header1.p1`, `t0.r1.c2`) in edit selectors.

mod common;

use common::docx::{Part, W_NS, docx_with_sect, para};
use jubarte::edit::{EditPlan, EditResult, apply_plan};

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
/// Two rows, two cells; the last cell holds two paragraphs. Body numbering:
/// p0 Zero, p1 Item, p2 Due, p3 Report, p4 Day 10, p5 or later, p6 Six.
const TABLE: &str = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Item</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Due</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Report</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Day 10</w:t></w:r></w:p><w:p><w:r><w:t>or later</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

fn source() -> Vec<u8> {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>DRAFT</w:t></w:r></w:p><w:p><w:r><w:t>Confidential</w:t></w:r></w:p></w:hdr>"#
    );
    docx_with_sect(
        &format!("{}{TABLE}{}", para("Zero"), para("Six")),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn at(result: &EditResult, i: usize) -> &str {
    result.report.operations[i].paragraph.as_deref().unwrap()
}

#[test]
fn short_ids_resolve_to_the_long_ids_the_report_prints() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[
        {"kind":"replace","paragraph":"p6","find":"Six","replacement":"Seven"},
        {"kind":"replace","paragraph":"t0.r1.c1","find":"Day 10","replacement":"Day 12"},
        {"kind":"replace","paragraph":"t0.r1.c1.p1","find":"later","replacement":"earlier"},
        {"kind":"replace","paragraph":"header1","find":"DRAFT","replacement":"FINAL"},
        {"kind":"replace","paragraph":{"id":"header1.p1"},"find":"Confidential","replacement":"Public"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "body:p:6");
    assert_eq!(at(&out, 1), "body:p:4");
    assert_eq!(at(&out, 2), "body:p:5");
    assert_eq!(at(&out, 3), "header1:p:0");
    assert_eq!(at(&out, 4), "header1:p:1");
}

#[test]
fn a_short_id_that_points_nowhere_is_refused_and_named() {
    for (short, message) in [
        (
            "p7",
            "paragraph index 7 does not exist in body (7 paragraphs)",
        ),
        (
            "header2",
            "header2 is not a part of this document (headers and footers: header1)",
        ),
        (
            "footer1",
            "footer1 is not a part of this document (headers and footers: header1)",
        ),
        (
            "header1.p2",
            "paragraph index 2 does not exist in header1 (2 paragraphs)",
        ),
        ("t1.r0.c0", "t1 is not a table of this document (1 table)"),
        ("t0.r2.c0", "t0 has 2 rows, no row 2"),
        ("t0.r0.c2", "t0.r0 has 2 cells, no cell 2"),
        ("t0.r0.c0.p1", "t0.r0.c0 has 1 paragraph, no p1"),
        ("t0", "t0: a table id needs a row and a cell, as t0.r1.c2"),
        ("px", "unknown paragraph id px"),
    ] {
        let e = apply_plan(
            &source(),
            &plan(&format!(
                r#"[{{"kind":"replace","paragraph":"{short}","find":"x","replacement":"y"}}]"#
            )),
        )
        .unwrap_err();
        assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{short}: {e}");
        assert!(e.to_string().contains(message), "{short}: {e}");
    }
}
