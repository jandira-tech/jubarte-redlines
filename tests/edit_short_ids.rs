// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Short paragraph ids (`p3`, `header1.p1`, `t0.row1.cell2`) in edit selectors.

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
        {"kind":"replace","paragraph":"t0.row1.cell1","find":"Day 10","replacement":"Day 12"},
        {"kind":"replace","paragraph":"t0.row1.cell1.p1","find":"later","replacement":"earlier"},
        {"kind":"replace","paragraph":"header1","find":"DRAFT","replacement":"FINAL"},
        {"kind":"replace","paragraph":{"id":"header1.p1"},"find":"Confidential","replacement":"Public"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "p6");
    assert_eq!(at(&out, 1), "p4");
    assert_eq!(at(&out, 2), "p5");
    assert_eq!(at(&out, 3), "header1");
    assert_eq!(at(&out, 4), "header1.p1");
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
            "header2 is not a story of this document (stories: body, header1)",
        ),
        (
            "footer1",
            "footer1 is not a story of this document (stories: body, header1)",
        ),
        (
            "header1.p2",
            "paragraph index 2 does not exist in header1 (2 paragraphs)",
        ),
        (
            "t1.row0.cell0",
            "t1 is not a table of this document (1 table)",
        ),
        ("t0.row2.cell0", "t0 has 2 rows, no row 2"),
        ("t0.row0.cell2", "t0.row0 has 2 cells, no cell 2"),
        // 0.12's cell form is expired: refused with its settled spelling.
        ("t0.r0.c2", "the settled spelling is t0.row0.cell2"),
        ("t0.row0.cell0.p1", "t0.row0.cell0 has 1 paragraph, no p1"),
        (
            "t0",
            "t0: a table id needs a row and a cell, as t0.row1.cell2",
        ),
        (
            "t0.row1",
            "t0.row1: a table id needs a row and a cell, as t0.row1.cell2",
        ),
        ("t0.r01.c1", "t0.r01.c1: expected a short id such as p12"),
        ("header", "header: expected a short id such as p12"),
        ("px", "px: expected a short id such as p12"),
        ("p012", "p012: expected a short id such as p12"),
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
        {"kind":"replace","paragraph":"t0.row0.cell0.p1","find":"ALPHA","replacement":"Alpha"},
        {"kind":"replace","paragraph":"t0.row0.cell0.p2","find":"BETA","replacement":"Beta"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "p2");
    assert_eq!(at(&out, 1), "p3");
    let refused = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"replace","paragraph":"t0.row0.cell0.p3","find":"x","replacement":"y"}]"#,
        ),
    );
    let message = match refused {
        Ok(result) => format!("{:?}", result.report.operations),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("has 3 paragraphs, no p3"), "{message}");
}

/// pi review r392b F5: a table nested in a cell is part of the cell in the
/// view (`cells t0.row0 p2-p4`), so `tN.rR.cC.pK` counts its paragraphs too.
#[test]
fn a_cell_paragraph_id_counts_the_nested_table_as_the_view_does() {
    let inner = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>"#,
        para("INNER")
    );
    let table = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{}{inner}{}</w:tc></w:tr></w:tbl>"#,
        para("HEAD"),
        para("TAIL")
    );
    let source = docx_with_sect(&format!("{}{table}{}", para("Zero"), para("Six")), &[], "");
    let view = jubarte::markdown::docx_to_markdown(
        &source,
        &jubarte::markdown::MarkdownOptions {
            ids: true,
            page_markers: false,
            ..Default::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(view.contains("cells t0.row0 p1-p3"), "{view}");
    let out = apply_plan(
        &source,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.row0.cell0.p1","find":"INNER","replacement":"Inner"},
        {"kind":"replace","paragraph":"t0.row0.cell0.p2","find":"TAIL","replacement":"Tail"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.ok, "{:?}", out.report.operations);
    assert_eq!(at(&out, 0), "p2");
    assert_eq!(at(&out, 1), "p3");
}

#[test]
fn pr392_nested_cell_paragraphs_skip_text_boxes_and_stop_at_the_outer_cell() {
    let boxed = r#"<w:r><w:pict><w:shape><w:txbxContent><w:p><w:r><w:t>BOX</w:t></w:r></w:p></w:txbxContent></w:shape></w:pict></w:r>"#;
    let table = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{}<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p>{boxed}<w:r><w:t>INNER</w:t></w:r></w:p></w:tc></w:tr></w:tbl>{}</w:tc><w:tc>{}</w:tc></w:tr></w:tbl>"#,
        para("HEAD"),
        para("TAIL"),
        para("NEIGHBOR")
    );
    let source = docx_with_sect(&table, &[], "");
    for (short, find, paragraph) in [
        ("t0.row0.cell0.p1", "INNER", "p1"),
        ("t0.row0.cell0.p2", "TAIL", "p2"),
        ("t0.row0.cell1", "NEIGHBOR", "p3"),
    ] {
        let out = apply_plan(
            &source,
            &plan(
                &serde_json::json!([{
                    "kind":"replace", "paragraph":short, "find":find, "replacement":"Edited"
                }])
                .to_string(),
            ),
        )
        .unwrap();
        assert_eq!(at(&out, 0), paragraph);
        let clean = jubarte::markdown::docx_to_markdown(&out.clean, &Default::default())
            .unwrap()
            .markdown;
        assert!(clean.contains("BOX"), "{short}: {clean}");
        assert!(clean.contains("Edited"), "{short}: {clean}");
        assert!(!clean.contains(find), "{short}: {clean}");
    }
    let error = apply_plan(&source, &plan(r#"[{"kind":"replace","paragraph":"t0.row0.cell0.p3","find":"NEIGHBOR","replacement":"Wrong"}]"#)).unwrap_err();
    assert_eq!(error.code, "ANCHOR_NOT_FOUND");
    assert!(
        error.to_string().contains("has 3 paragraphs, no p3"),
        "{error}"
    );
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
            "t0.row1.cell1.p1",
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
        "t0.row0",
        "t0.row-1.cell0",
        "t0.row0.cell+1",
        "t0.row0.cell0.p",
        "t0.row0.cell0.p0.extra",
        "t0.row0.cell18446744073709551616",
        "p012",
        "t0.row01.cell0",
        "r1.c2",
        "row0.cell0",
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
        ("p0", "p0", "Zero"),
        ("p6", "p6", "Six"),
        ("t0.row1.cell1", "p4", "Day 10"),
        ("header1.p0", "header1", "DRAFT"),
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
    assert_eq!(at(&out, 0), "footer2");
    assert_eq!(at(&out, 1), "footer2.p1");
    let xml = common::docx::part_string(&out.clean, "word/footer2.xml").unwrap();
    assert!(xml.contains("One") && xml.contains("Two"), "{xml}");
    assert!(!xml.contains("First") && !xml.contains("Second"), "{xml}");
}

/// `tN` counts top-level tables only, while `cC.pK` counts the nested
/// table's paragraphs as the view does (ba9798e9): "After" is the cell's p2.
#[test]
fn table_ids_skip_nested_tables_and_cell_indices_count_nested_paragraphs() {
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
        {"kind":"replace","paragraph":"t0.row0.cell0.p2","find":"After","replacement":"Later"},
        {"kind":"replace","paragraph":"t1.row0.cell0","find":"Next","replacement":"Last"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(at(&out, 0), "p2");
    assert_eq!(at(&out, 1), "p3");
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
        {"kind":"replace","paragraph":"t0.row0.cell1","find":"Target","replacement":"Changed"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(at(&out, 0), "p1");
    let error = apply_plan(
        &bytes,
        &plan(
            r#"[
        {"kind":"replace","paragraph":"t0.row0.cell2","find":"Target","replacement":"Changed"}
    ]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(error.code, "ANCHOR_NOT_FOUND");
    assert!(
        error.to_string().contains("t0.row0 has 2 cells, no cell 2"),
        "{error}"
    );
}

#[test]
fn missing_parts_and_tables_report_empty_inventories() {
    let bytes = common::docx::docx(&para("Only body"));
    for (id, message) in [
        (
            "header1",
            "header1 is not a story of this document (stories: body)",
        ),
        (
            "t0.row0.cell0",
            "t0 is not a table of this document (0 tables)",
        ),
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

fn received() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/agent-view/received.docx"
    ))
    .unwrap()
}

/// A plan's `comment_id` takes only the view's `c5` and the number;
/// both write the same reply, while `#c5` and `"5"` are refused with
/// the settled spelling.
#[test]
fn a_plan_comment_id_takes_every_comment_spelling() {
    let reply = |id: serde_json::Value| {
        let operations = serde_json::json!([{
            "kind": "reply_comment", "comment_id": id, "text": "Yes."
        }]);
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","existing_revisions":"keep","operations":{operations}}}"#
        ))
        .unwrap();
        let out = apply_plan(&received(), &plan).unwrap();
        assert!(out.report.ok, "{id}: {:?}", out.report.operations);
        common::docx::part_string(&out.redline, "word/comments.xml").unwrap()
    };
    // A JSON number is a typed value; a string is an id, settled only.
    assert_eq!(reply(serde_json::json!("c5")), reply(serde_json::json!(5)));
    for (id, message) in [
        ("p5", "p5: expected a comment id such as c27"),
        (
            "#c5",
            "#c5: expected a comment id such as c27; the settled spelling is c5",
        ),
        (
            "5",
            "5: expected a comment id such as c27; the settled spelling is c5",
        ),
    ] {
        let operations = serde_json::json!([{
            "kind": "reply_comment", "comment_id": id, "text": "Yes."
        }]);
        let error = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"A","operations":{operations}}}"#
        ))
        .unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}

/// `jubarte edit ARGS` under a pinned timestamp, so two runs compared byte
/// for byte cannot straddle a second.
fn edit_cli(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .args(args)
        .args(["--datetime", "2026-10-11T00:00:00Z"])
        .current_dir(dir)
        .output()
        .unwrap()
}

/// `-p 6` is `-p p6`; `-p '#c5'` (0.12's spelling, passed as argv) is
/// `-p c5`; `-p footnotes.p0` edits the note.
#[test]
fn the_cli_p_flag_takes_settled_ids_and_refuses_expired_ones() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("src.docx"), source()).unwrap();
    // Expired spellings (Arthur, 2026-10-13) exit 2 with the settled one.
    // `--resolve` names a comment: only `c6` is suggested.
    for (at, settled) in [("6", "c6"), ("#c5", "c5")] {
        let out = edit_cli(dir.path(), &["src.docx", "-p", at, "--resolve"]);
        assert_eq!(out.status.code(), Some(2), "{at}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(&format!("the settled spelling is {settled}")),
            "{at}: {stderr}"
        );
    }
    // A group a comment cannot take suggests only the paragraph.
    let out = edit_cli(dir.path(), &["src.docx", "-p", "5", "--style", "bold"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the settled spelling is p5"), "{stderr}");
    assert!(!stderr.contains("p5 or c5"), "{stderr}");
    for (n, at) in ["p6", "p6"].iter().enumerate() {
        let out_dir = format!("out{n}");
        let out = edit_cli(
            dir.path(),
            &[
                "src.docx",
                "-p",
                at,
                "--anchor",
                "Six",
                "--content",
                "Seven",
                "--out-dir",
                &out_dir,
            ],
        );
        assert!(
            out.status.success(),
            "{at}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let a = std::fs::read(dir.path().join("out0/redline.docx")).unwrap();
    let b = std::fs::read(dir.path().join("out1/redline.docx")).unwrap();
    assert_eq!(
        common::docx::part_string(&a, "word/document.xml"),
        common::docx::part_string(&b, "word/document.xml")
    );

    std::fs::write(dir.path().join("received.docx"), received()).unwrap();
    for (n, at) in ["c5", "c5"].iter().enumerate() {
        let out_dir = format!("reply{n}");
        let out = edit_cli(
            dir.path(),
            &[
                "received.docx",
                "-p",
                at,
                "--resolve",
                // c5 is another author's: resolved on request (#1846).
                "--rewrite-others-comments",
                "--out-dir",
                &out_dir,
            ],
        );
        assert!(
            out.status.success(),
            "{at}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let resolved = |out: &str| {
        let docx = std::fs::read(dir.path().join(out).join("redline.docx")).unwrap();
        common::docx::part_string(&docx, "word/commentsExtended.xml").unwrap()
    };
    let done = resolved("reply0");
    assert!(done.contains(r#"w15:done="1""#), "{done}");
    assert_eq!(resolved("reply1"), done);

    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:r><w:t>Source note</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let noted = docx_with_sect(
        r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &footnotes,
        }],
        "",
    );
    std::fs::write(dir.path().join("noted.docx"), noted).unwrap();
    let out = edit_cli(
        dir.path(),
        &[
            "noted.docx",
            "-p",
            "footnotes.p0",
            "--anchor",
            "Source",
            "--content",
            "Cited",
            "--out-dir",
            "notes",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let redline = std::fs::read(dir.path().join("notes/redline.docx")).unwrap();
    let notes = common::docx::part_string(&redline, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains(">Cited<") && notes.contains(">Source<"),
        "{notes}"
    );
}

/// 0.12's cell form `t0.r1.c1.p1` is expired: refused with exit 2 and the
/// settled `t0.row1.cell1.p1`, which edits the cell.
#[test]
fn the_expired_cell_form_is_refused_with_its_settled_spelling() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("src.docx"), source()).unwrap();
    let args = |at: &'static str, out_dir: &'static str| {
        [
            "src.docx",
            "-p",
            at,
            "--anchor",
            "later",
            "--content",
            "earlier",
            "--out-dir",
            out_dir,
        ]
    };
    let out = edit_cli(dir.path(), &args("t0.row1.cell1.p1", "out0"));
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let a = std::fs::read(dir.path().join("out0/redline.docx")).unwrap();
    let body = common::docx::part_string(&a, "word/document.xml").unwrap();
    assert!(body.contains(">earlier<"), "{body}");
    let out = edit_cli(dir.path(), &args("t0.r1.c1.p1", "out1"));
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("the settled spelling is t0.row1.cell1.p1"),
        "{stderr}"
    );
    assert!(!dir.path().join("out1/redline.docx").exists());
}

fn listed(docx: &[u8]) -> Vec<String> {
    jubarte::changes::list_changes(docx)
        .unwrap()
        .into_iter()
        .map(|c| format!("{:?} {:?}", c.kind, c.text))
        .collect()
}

/// `edit -p r0 --accept -p r1 --reject` resolves those two revisions and
/// keeps the others; `-p c5 --reply` beside them writes the reply
/// `add -p c5 --content` writes.
#[test]
fn the_cli_resolves_revisions_and_replies_by_their_ids() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("received.docx"), received()).unwrap();
    let run = |verb: &str, args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .arg(verb)
            .arg("received.docx")
            .args(args)
            .args([
                "--author",
                "Ann Counsel",
                "--datetime",
                "2026-10-01T09:00:00Z",
            ])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(
        "edit",
        &[
            "-p",
            "r0",
            "--accept",
            "-p",
            "r1",
            "--reject",
            "--out-dir",
            "resolved",
        ],
    );
    let resolved = std::fs::read(dir.path().join("resolved/redline.docx")).unwrap();
    let before = listed(&received());
    let after = listed(&resolved);
    assert_eq!(after, before[2..], "{after:?}");
    let text = common::docx::part_string(&resolved, "word/document.xml").unwrap();
    assert!(
        text.contains(">quarterly<") && text.contains(">weekly<"),
        "the accepted insertion and the rejected deletion stay as text"
    );

    run(
        "edit",
        &["-p", "c5", "--reply", "Agreed.", "--out-dir", "reply"],
    );
    run(
        "add",
        &["-p", "c5", "--reply", "Agreed.", "--out-dir", "add-reply"],
    );
    run(
        "add",
        &["-p", "c5", "--content", "Agreed.", "--out-dir", "content"],
    );
    let comments = |out: &str| {
        let docx = std::fs::read(dir.path().join(out).join("redline.docx")).unwrap();
        common::docx::part_string(&docx, "word/comments.xml").unwrap()
    };
    let want = comments("content");
    assert!(want.contains(">Agreed.<"), "{want}");
    assert_eq!(comments("reply"), want);
    assert_eq!(comments("add-reply"), want);
}

/// A revision flag on a paragraph, a reply on a revision or a lone
/// revision is a usage error (exit 2) naming the -p.
#[test]
fn the_cli_refuses_revision_and_reply_flags_on_the_wrong_location() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("received.docx"), received()).unwrap();
    for (args, message) in [
        (
            &["-p", "p3", "--accept"][..],
            "-p p3: --accept and --reject take a revision (-p r12)",
        ),
        (
            &["-p", "r0"][..],
            "-p r0: a revision takes --accept, --reject, --strike or --restore",
        ),
        (
            &["-p", "r0", "--accept", "--reject"][..],
            "-p r0: --accept and --reject exclude each other",
        ),
        (
            &["-p", "p3", "--reply", "x"][..],
            "-p p3: --reply is for a comment (-p c5)",
        ),
        (
            &["-p", "c5", "--reply", "x", "--resolve"][..],
            "-p c5: --reply takes no other flag",
        ),
        // A bare number is expired (Arthur, 2026-10-13): refused whatever
        // flag sits beside it, naming the settled spellings when they parse.
        (
            &["-p", "1+2", "--accept"][..],
            "-p 1+2: -p takes a paragraph",
        ),
        (
            &["-p", "1+2", "--accept"][..],
            "; the settled spelling is r1+2",
        ),
        (
            &["-p", "1", "--reject"][..],
            "; the settled spelling is r1\n",
        ),
        (
            &["-p", "5", "--resolve"][..],
            "; the settled spelling is c5\n",
        ),
        (
            &["-p", "5", "--reply", "x"][..],
            "; the settled spelling is c5\n",
        ),
        (
            // Only a spelling the group takes: no comment is spelled `1+2`.
            &["-p", "1+2", "--resolve"][..],
            "-p 1+2: -p takes a paragraph",
        ),
        // A settled id -p does not edit yet: refused before any I/O.
        (
            &["-p", "s0", "--content", "x"][..],
            "-p s0: a settled id, but -p does not edit",
        ),
        (
            &["-p", "p3-p7", "--content", "x"][..],
            "-p p3-p7: a settled id, but -p does not edit",
        ),
        (
            &["-p", "toc0", "--accept"][..],
            "-p toc0: a settled id, but -p does not edit",
        ),
        // No suggestion that would not parse: the flag's own message.
        (
            &["-p", "01", "--accept"][..],
            "-p 01: --accept and --reject take a revision (-p r12)",
        ),
        (
            &["-p", "1+02", "--reject"][..],
            "-p 1+02: --accept and --reject take a revision (-p r12)",
        ),
        (
            &["-p", "05", "--reply", "x"][..],
            "-p 05: --reply is for a comment (-p c5)",
        ),
        (
            &["-p", "4294967296", "--accept"][..],
            "-p 4294967296: --accept and --reject take a revision (-p r12)",
        ),
    ] {
        let out = edit_cli(dir.path(), &[&["received.docx"][..], args].concat());
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }
}

/// `-p r1 --reject -p p3 --anchor weekly --content biweekly` rejects first:
/// Ann's deletion "weekly" comes back as text, and the edit then finds it.
#[test]
fn a_rejection_lands_before_the_edit_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("received.docx"), received()).unwrap();
    let out = edit_cli(
        dir.path(),
        &[
            "received.docx",
            "-p",
            "r1",
            "--reject",
            "-p",
            "p3",
            "--anchor",
            "weekly",
            "--content",
            "biweekly",
            "--existing-revisions",
            "keep",
            "--author",
            "Editor",
            "--out-dir",
            "both",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let redline = std::fs::read(dir.path().join("both/redline.docx")).unwrap();
    let texts: Vec<String> = jubarte::changes::list_changes(&redline)
        .unwrap()
        .into_iter()
        .map(|c| format!("{:?} {} {}", c.kind, c.author.unwrap_or_default(), c.text))
        .collect();
    // The edit tracks the smallest change: "bi" before the live "weekly".
    assert!(
        texts.iter().any(|t| t == "Insertion Editor bi"),
        "{texts:?}"
    );
    assert!(
        !texts.iter().any(|t| t == "Deletion Ann Counsel weekly"),
        "{texts:?}"
    );
}
