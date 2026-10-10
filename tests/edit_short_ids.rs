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

#[test]
fn malformed_short_ids_never_fall_back_to_a_valid_paragraph() {
    for id in [
        "p",
        "p-1",
        "p+1",
        "p１",
        "p1.p0",
        "p1.",
        "P1",
        "p18446744073709551616",
        "header",
        "header1.p-1",
        "header1.p0.extra",
        "t0.r0",
        "t0.r-1.c0",
        "t0.r0.c+1",
        "t0.r0.c0.p",
        "t0.r0.c0.p0.extra",
        "t0.r0.c18446744073709551616",
    ] {
        for selector in [serde_json::json!(id), serde_json::json!({ "id": id })] {
            let operations = serde_json::json!([{
                "kind": "replace", "paragraph": selector,
                "find": "Zero", "replacement": "Changed"
            }]);
            let error = apply_plan(&source(), &plan(&operations.to_string())).unwrap_err();
            assert_eq!(error.code, "ANCHOR_NOT_FOUND", "{id}: {error}");
        }
    }
}

#[test]
fn equivalent_short_and_long_selectors_edit_the_same_content() {
    for (short, long, find) in [
        ("p0", "body:p:0", "Zero"),
        ("p6", "body:p:6", "Six"),
        ("t0.r1.c1.p0", "body:p:4", "Day 10"),
        ("header1.p0", "header1:p:0", "DRAFT"),
    ] {
        let edit = |selector| {
            apply_plan(
                &source(),
                &plan(
                    &serde_json::json!([{
                        "kind": "replace", "paragraph": selector,
                        "find": find, "replacement": "Changed"
                    }])
                    .to_string(),
                ),
            )
            .unwrap()
        };
        let expected = edit(serde_json::json!(long));
        for selector in [serde_json::json!(short), serde_json::json!({ "id": short })] {
            let actual = edit(selector);
            assert_eq!(at(&actual, 0), long);
            for part in ["word/document.xml", "word/header1.xml"] {
                assert_eq!(
                    common::docx::part_string(&actual.clean, part),
                    common::docx::part_string(&expected.clean, part),
                    "{short}: {part}"
                );
            }
        }
    }
}

#[test]
fn footer_ids_use_the_part_number_and_default_to_its_first_paragraph() {
    let footer = format!(
        r#"<w:ftr xmlns:w="{W_NS}">{}{}</w:ftr>"#,
        para("First"),
        para("Second")
    );
    let bytes = docx_with_sect(
        &para("Body"),
        &[Part {
            name: "word/footer2.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
            xml: &footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = apply_plan(
        &bytes,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"footer2","find":"First","replacement":"One"},
        {"kind":"replace","paragraph":{"id":"footer2.p1"},"find":"Second","replacement":"Two"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(at(&out, 0), "footer2:p:0");
    assert_eq!(at(&out, 1), "footer2:p:1");
    let xml = common::docx::part_string(&out.clean, "word/footer2.xml").unwrap();
    assert!(xml.contains("One") && xml.contains("Two"), "{xml}");
    assert!(!xml.contains("First") && !xml.contains("Second"), "{xml}");
}

#[test]
fn table_ids_skip_nested_tables_and_cell_indices_skip_nested_paragraphs() {
    let nested = format!(
        "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
        para("Nested")
    );
    let outer = format!(
        "<w:tbl><w:tr><w:tc>{}{nested}{}</w:tc></w:tr></w:tbl>",
        para("Before"),
        para("After")
    );
    let next = format!("<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>", para("Next"));
    let bytes = common::docx::docx(&format!("{outer}{next}"));
    let out = apply_plan(
        &bytes,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.r0.c0.p1","find":"After","replacement":"Later"},
        {"kind":"replace","paragraph":"t1.r0.c0","find":"Next","replacement":"Last"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(at(&out, 0), "body:p:2");
    assert_eq!(at(&out, 1), "body:p:3");
    let xml = common::docx::part_string(&out.clean, "word/document.xml").unwrap();
    assert!(
        xml.contains("Nested") && xml.contains("Later") && xml.contains("Last"),
        "{xml}"
    );
}

#[test]
fn content_controls_and_merged_cells_keep_physical_cell_indices() {
    let bytes = common::docx::docx(&format!(
        r#"<w:sdt><w:sdtContent><w:tbl><w:tblGrid><w:gridCol/><w:gridCol/><w:gridCol/></w:tblGrid><w:sdt><w:sdtContent><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr>{}</w:tc><w:sdt><w:sdtContent><w:tc>{}</w:tc></w:sdtContent></w:sdt></w:tr></w:sdtContent></w:sdt></w:tbl></w:sdtContent></w:sdt>"#,
        para("Merged"),
        para("Target")
    ));
    let out = apply_plan(
        &bytes,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.r0.c1","find":"Target","replacement":"Changed"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(at(&out, 0), "body:p:1");
    let error = apply_plan(
        &bytes,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.r0.c2","find":"Target","replacement":"Changed"}
    ]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(error.code, "ANCHOR_NOT_FOUND");
    assert!(
        error.to_string().contains("t0.r0 has 2 cells, no cell 2"),
        "{error}"
    );
}

#[test]
fn missing_parts_and_tables_report_empty_inventories() {
    let bytes = common::docx::docx(&para("Only body"));
    for (id, message) in [
        (
            "header1",
            "header1 is not a part of this document (headers and footers: none)",
        ),
        ("t0.r0.c0", "t0 is not a table of this document (0 tables)"),
    ] {
        let error = apply_plan(
            &bytes,
            &plan(&format!(
                r#"[{{"kind":"replace","paragraph":"{id}","find":"Only","replacement":"New"}}]"#
            )),
        )
        .unwrap_err();
        assert_eq!(error.code, "ANCHOR_NOT_FOUND");
        assert!(error.to_string().contains(message), "{error}");
    }
}
