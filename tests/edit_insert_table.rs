// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `insert_table`: a table is built in the clean copy next to an anchor
//! paragraph and the comparer tracks it as inserted rows.

mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with, docx_with_sect, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::document_comparer::reject_revisions;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","date":"2026-10-02T12:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

/// A part's XML with empty elements written `<x/>`, as Word writes them.
fn part(bytes: &[u8], name: &str) -> Option<String> {
    part_string(bytes, name).map(|xml| xml.replace(" />", "/>"))
}

fn texts(bytes: &[u8]) -> Vec<(String, bool)> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| (p.text, p.in_table))
        .collect()
}

#[test]
fn an_inserted_table_is_tracked_and_valid() {
    let source = docx(&(para("Intro.") + &para("Outro.")));
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"insert_table","paragraph":"body:p:0","position":"after","header_row":true,
                 "rows":[["Item","Qty"],["Bolt","40"]],"widths_dxa":[6000,3360]}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    assert!(xml.contains("<w:tblHeader/>"), "{xml}");
    assert!(xml.contains(r#"<w:tcW w:w="6000" w:type="dxa"/>"#), "{xml}");
    assert!(
        xml.contains(r#"<w:tblW w:w="9360" w:type="dxa"/>"#),
        "{xml}"
    );
    assert!(xml.contains(r#"<w:gridCol w:w="3360"/>"#), "{xml}");
    assert!(xml.contains(r#"<w:tblStyle w:val="TableGrid"/>"#), "{xml}");
    assert_eq!(xml.matches("<w:tblHeader/>").count(), 1, "first row only");
    assert_eq!(
        texts(&out.clean),
        vec![
            ("Intro.".to_string(), false),
            ("Item".to_string(), true),
            ("Qty".to_string(), true),
            ("Bolt".to_string(), true),
            ("40".to_string(), true),
            ("Outro.".to_string(), false),
        ]
    );
    // The style the table names exists in the clean copy, with its base.
    let styles = part(&out.clean, "word/styles.xml").expect("styles part created");
    assert!(styles.contains(r#"w:styleId="TableGrid""#), "{styles}");
    assert!(styles.contains(r#"w:styleId="TableNormal""#), "{styles}");
    let redline_styles = part(&out.redline, "word/styles.xml").expect("redline styles");
    assert!(
        redline_styles.contains(r#"w:styleId="TableGrid""#),
        "the redline's table names a style it defines: {redline_styles}"
    );
    let kinds: Vec<_> = list_changes(&out.redline)
        .unwrap()
        .into_iter()
        .map(|c| (c.kind, c.target))
        .collect();
    assert!(
        kinds
            .iter()
            .any(|(k, t)| *k == ChangeKind::Insertion && *t == "table_row"),
        "{kinds:?}"
    );
    // Rejecting everything gives the source text back.
    let rejected = reject_revisions(&out.redline).unwrap();
    assert_eq!(
        texts(&rejected),
        vec![("Intro.".to_string(), false), ("Outro.".to_string(), false)]
    );
    let report = &out.report.operations[0];
    assert_eq!(report.kind, "insert_table");
    assert_eq!(report.status, "ok");
    assert_eq!(report.paragraph.as_deref(), Some("body:p:0"));
}

#[test]
fn widths_default_to_the_text_width_split_evenly() {
    let source = docx(&para("Intro."));
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a","b","c"]]}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    // Letter page 12240 less two 1440 margins: 9360, three columns of 3120.
    assert_eq!(
        xml.matches(r#"<w:gridCol w:w="3120"/>"#).count(),
        3,
        "{xml}"
    );
    assert_eq!(
        xml.matches(r#"<w:tcW w:w="3120" w:type="dxa"/>"#).count(),
        3
    );
    assert!(
        xml.contains(r#"<w:tblW w:w="9360" w:type="dxa"/>"#),
        "{xml}"
    );
    assert!(!xml.contains("<w:tblHeader/>"), "no header unless asked");
}

#[test]
fn a_table_after_the_last_paragraph_is_closed_by_a_paragraph() {
    let source = docx(&para("Only."));
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["x"]]}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    let table_end = xml.find("</w:tbl>").unwrap();
    assert!(
        xml[table_end..].starts_with("</w:tbl><w:p"),
        "a paragraph follows the table: {xml}"
    );
}

#[test]
fn a_table_before_a_table_is_kept_apart_by_a_paragraph() {
    let existing = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="9360"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="9360" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Old cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let source = docx(&(existing.to_string() + &para("After.")));
    // body:p:0 is the old cell, body:p:1 is "After."; inserting before
    // "After." would put the new table right after the old one.
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"insert_table","paragraph":"body:p:1","position":"before","rows":[["New"]]}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    assert!(
        !xml.contains("</w:tbl><w:tbl>"),
        "tables must not touch: {xml}"
    );
    assert_eq!(xml.matches("<w:tbl>").count(), 2, "{xml}");
}

#[test]
fn an_existing_table_style_is_used_by_name() {
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="table" w:styleId="Fancy"><w:name w:val="Fancy Table"/></w:style></w:styles>"#;
    let source = docx_with(
        &para("Intro."),
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: styles,
        }],
    );
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["x"]],"style":"fancy table"}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:tblStyle w:val="Fancy"/>"#), "{xml}");
    let styles = part(&out.clean, "word/styles.xml").unwrap();
    assert!(!styles.contains("TableGrid"), "no style added: {styles}");
}

#[test]
fn cells_take_the_anchors_paragraph_style_only() {
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Body"><w:name w:val="Body"/></w:style></w:styles>"#;
    let anchor = r#"<w:p><w:pPr><w:pStyle w:val="Body"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Intro.</w:t></w:r></w:p>"#;
    let source = docx_with(
        anchor,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: styles,
        }],
    );
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["x"]]}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    let cell = &paragraphs(&out.clean).unwrap()[1];
    assert!(cell.in_table);
    assert_eq!(cell.style.as_deref(), Some("Body"));
    let xml = part(&out.clean, "word/document.xml").unwrap();
    assert_eq!(xml.matches(r#"<w:jc w:val="center"/>"#).count(), 1, "{xml}");
    // The paragraph after the table keeps its default style.
    let styles = part(&out.clean, "word/styles.xml").unwrap();
    assert!(styles.contains(r#"w:styleId="TableGrid""#));
}

#[test]
fn a_table_and_a_paragraph_after_one_anchor_keep_plan_order() {
    let source = docx(&(para("Intro.") + &para("Outro.")));
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["cell"]]},
                {"kind":"insert_paragraph","paragraph":"body:p:0","runs":[{"text":"Caption."}]}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let order: Vec<String> = texts(&out.clean).into_iter().map(|(t, _)| t).collect();
    assert_eq!(order, ["Intro.", "cell", "Caption.", "Outro."]);
}

fn refusal(source: &[u8], operations: &str) -> (String, String) {
    let error = apply_plan(source, &plan(operations)).unwrap_err();
    (error.code, error.message)
}

#[test]
fn ragged_rows_are_refused() {
    let source = docx(&para("Intro."));
    let (code, message) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a","b"],["c"]]}]"#,
    );
    assert_eq!(code, "INVALID_EDIT");
    assert!(message.contains("row 1"), "{message}");
}

#[test]
fn widths_must_match_the_column_count() {
    let source = docx(&para("Intro."));
    let (code, message) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a","b"]],"widths_dxa":[5000]}]"#,
    );
    assert_eq!(code, "INVALID_EDIT");
    assert!(message.contains("widths_dxa"), "{message}");
    let (code, _) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a","b"]],"widths_dxa":[5000,0]}]"#,
    );
    assert_eq!(code, "INVALID_EDIT");
}

#[test]
fn empty_tables_and_control_characters_are_refused() {
    let source = docx(&para("Intro."));
    for rows in [r#"[]"#, r#"[[]]"#, r#"[["a\tb"]]"#] {
        let (code, _) = refusal(
            &source,
            &format!(r#"[{{"kind":"insert_table","paragraph":"body:p:0","rows":{rows}}}]"#),
        );
        assert_eq!(code, "INVALID_EDIT", "{rows}");
    }
}

#[test]
fn unknown_table_styles_are_refused() {
    let source = docx(&para("Intro."));
    let (code, _) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a"]],"style":"Nope"}]"#,
    );
    assert_eq!(code, "UNKNOWN_STYLE");
}

#[test]
fn a_table_inside_a_table_cell_is_refused() {
    let table = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="9360"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="9360" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let source = docx(&(table.to_string() + &para("After.")));
    let (code, _) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a"]]}]"#,
    );
    assert_eq!(code, "UNSUPPORTED_STRUCTURE");
}

#[test]
fn a_table_on_a_deleted_anchor_conflicts() {
    let source = docx(&(para("Intro.") + &para("Outro.")));
    let (code, _) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a"]]},
            {"kind":"delete_paragraph","paragraph":"body:p:0"}]"#,
    );
    assert_eq!(code, "OVERLAPPING_EDITS");
}

#[test]
fn misspelled_insert_table_keys_are_refused() {
    let error = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[{"kind":"insert_table","paragraph":"body:p:0","rows":[["a"]],"header":true}]}"#,
    )
    .unwrap_err();
    assert_eq!(error.code, "INVALID_PLAN");
}

#[test]
fn a_table_between_two_paragraphs_a_merge_joins_conflicts() {
    let source = docx(&(para("One.") + &para("Two.")));
    let (code, message) = refusal(
        &source,
        r#"[{"kind":"merge_paragraphs","paragraph":"body:p:0"},
            {"kind":"insert_table","paragraph":"body:p:0","rows":[["a"]]}]"#,
    );
    assert_eq!(code, "OVERLAPPING_EDITS");
    assert!(message.contains("merge"), "{message}");
}

#[test]
fn a_table_in_a_header_is_refused() {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>Header</w:t></w:r></w:p></w:hdr>"#
    );
    let rel = format!("{R_NS}/header");
    let source = docx_with_sect(
        &para("Body."),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: &rel,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let (code, message) = refusal(
        &source,
        r#"[{"kind":"insert_table","paragraph":"header1:p:0","rows":[["a"]]}]"#,
    );
    assert_eq!(code, "UNSUPPORTED_STRUCTURE");
    assert!(message.contains("body only"), "{message}");
}

#[test]
fn cells_of_an_unstyled_anchor_have_no_paragraph_properties() {
    let anchor = r#"<w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>Intro.</w:t></w:r></w:p>"#;
    let source = docx(anchor);
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"insert_table","paragraph":"body:p:0","rows":[["x"]]}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    let xml = part(&out.clean, "word/document.xml").unwrap();
    let cell = &xml[xml.find("<w:tc>").unwrap()..xml.find("</w:tc>").unwrap()];
    assert!(!cell.contains("<w:pPr>"), "{cell}");
}
