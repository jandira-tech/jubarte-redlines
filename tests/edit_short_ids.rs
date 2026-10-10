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

/// A text box inside a cell is not one of the cell's paragraphs: the view
/// numbers the cell `host, ALPHA, BETA` (the box's own paragraph has no id),
/// and `tN.rR.cC.pK` counts the same way.
#[test]
fn a_cell_paragraph_id_skips_the_text_box_inside_the_cell() {
    let host = r#"<w:p><w:r><w:pict><w:shape><w:txbxContent><w:p><w:r><w:t>Boxed</w:t></w:r></w:p></w:txbxContent></w:shape></w:pict></w:r></w:p>"#;
    let table = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{host}{}{}</w:tc></w:tr></w:tbl>"#,
        para("ALPHA"),
        para("BETA")
    );
    let source = docx_with_sect(&format!("{}{table}{}", para("Zero"), para("Six")), &[], "");
    let out = apply_plan(
        &source,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.r0.c0.p1","find":"ALPHA","replacement":"Alpha"},
        {"kind":"replace","paragraph":"t0.r0.c0.p2","find":"BETA","replacement":"Beta"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "body:p:2");
    assert_eq!(at(&out, 1), "body:p:3");
    let refused = apply_plan(
        &source,
        &plan(r#"[{"kind":"replace","paragraph":"t0.r0.c0.p3","find":"x","replacement":"y"}]"#),
    );
    let message = match refused {
        Ok(result) => format!("{:?}", result.report.operations),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("has 3 paragraphs, no p3"), "{message}");
}

/// pi review av4 F14: the CLI's `-p` takes the same cell and story ids as a
/// plan, one group per operation.
#[test]
fn the_cli_p_flag_takes_cell_and_story_ids() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("src.docx"), source()).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "edit",
            "src.docx",
            "-p",
            "t0.r1.c1.p1",
            "--anchor",
            "later",
            "--content",
            "earlier",
            "-p",
            "header1",
            "--anchor",
            "DRAFT",
            "--content",
            "FINAL",
            "--out-dir",
            "out",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let redline = std::fs::read(dir.path().join("out/redline.docx")).unwrap();
    let body = common::docx::part_string(&redline, "word/document.xml").unwrap();
    assert!(
        body.contains(">earlier<") && body.contains(">later<"),
        "{body}"
    );
    let header = common::docx::part_string(&redline, "word/header1.xml").unwrap();
    assert!(
        header.contains(">FINAL<") && header.contains(">DRAFT<"),
        "{header}"
    );
}
