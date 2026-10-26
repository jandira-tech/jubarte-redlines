// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1870: a Word-mode compare first accepts the revisions its inputs
//! already hold, and the accept ends in the Word-save tidy, which joins
//! adjacent tables. The probe that decides it matched `<w:ins` as a prefix,
//! so a field's `<w:instrText>` and a table's `<w:insideH>` counted as
//! revisions: inputs with none were tidied, and two tables became one.
//!
//! Accept All and Reject All still join them, on purpose: Word does (R8,
//! `tests/m_accept_word_parity.rs`).

mod common;

use common::docx::{docx, para, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

/// A one-row table whose only property is its width.
const PLAIN: &str = r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Title</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

/// A two-row table with outer and inner borders.
const BORDERED: &str = r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/><w:tblBorders><w:top w:val="single" w:sz="4" w:color="000000"/><w:left w:val="single" w:sz="4" w:color="000000"/><w:bottom w:val="single" w:sz="4" w:color="000000"/><w:right w:val="single" w:sz="4" w:color="000000"/><w:insideH w:val="single" w:sz="4" w:color="000000"/><w:insideV w:val="single" w:sz="4" w:color="000000"/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>one</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>two</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

/// A paragraph holding a `PAGE` field.
const FIELD: &str = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;

/// Two adjacent tables, each source holding no revision: inner borders on
/// the second table, or two plain tables after a field.
fn sources() -> [(&'static str, String); 2] {
    [
        ("inner borders", format!("{PLAIN}{BORDERED}{}", para("End"))),
        ("field", format!("{FIELD}{PLAIN}{PLAIN}{}", para("End"))),
    ]
}

fn tables(docx: &[u8]) -> usize {
    part_string(docx, "word/document.xml")
        .unwrap()
        .matches("<w:tbl>")
        .count()
}

/// The text from the first `<w:tbl>` to the last `</w:tbl>`, empty
/// elements written `<x/>`.
fn tables_xml(docx: &[u8]) -> String {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let start = xml.find("<w:tbl>").unwrap();
    xml[start..xml.rfind("</w:tbl>").unwrap()].replace(" />", "/>")
}

/// The compare's redline keeps both tables. Accept All and Reject All of
/// it give the tables they give on the inputs themselves.
#[test]
fn a_word_compare_keeps_adjacent_tables_apart() {
    for (label, rest) in sources() {
        let original = docx(&format!("{}{rest}", para("Intro")));
        let revised = docx(&format!("{}{rest}", para("Intro changed")));
        assert_eq!(tables(&original), 2, "{label}");
        let redline =
            compare_documents_with_settings(&original, &revised, &WmlComparerSettings::default())
                .unwrap();
        let xml = part_string(&redline, "word/document.xml").unwrap();
        assert!(xml.contains("<w:ins "), "{label}: the edit is marked");
        assert_eq!(tables(&redline), 2, "{label}");
        assert_eq!(
            tables(&accept_revisions(&redline).unwrap()),
            tables(&accept_revisions(&revised).unwrap()),
            "{label}, accept"
        );
        assert_eq!(
            tables(&reject_revisions(&redline).unwrap()),
            tables(&reject_revisions(&original).unwrap()),
            "{label}, reject"
        );
    }
}

/// `add -p p0 --content "Probe paragraph."`.
fn add_p0(source: &[u8]) -> jubarte::edit::EditResult {
    apply_plan(
        source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap()
}

/// An `add` leaves the tables it did not touch as the source has them.
#[test]
fn an_add_keeps_adjacent_tables_apart() {
    for (label, rest) in sources() {
        let source = docx(&format!("{}{rest}", para("Intro")));
        let edited = add_p0(&source);
        assert_eq!(tables_xml(&edited.clean), tables_xml(&source), "{label}");
        assert_eq!(tables_xml(&edited.redline), tables_xml(&source), "{label}");
        assert_eq!(
            tables_xml(&accept_revisions(&edited.redline).unwrap()),
            tables_xml(&accept_revisions(&edited.clean).unwrap()),
            "{label}, accept"
        );
    }
}
