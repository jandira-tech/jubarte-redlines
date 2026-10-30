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
//!
//! #1885: a style's recorded formatting does not make the text tracked.
//! An edit skipped such changes by an id prefix no id has (`styles:`; ids
//! read `styles.r0`), so a plan was refused under `refuse`; and once let
//! through, the source's record set off the same pre-accept and tidy. A
//! tracked change inside a comment, which no story lists, went the same
//! way: the edit accepted it unasked.

#[path = "common/comment_revisions.rs"]
mod comment_revisions;
mod common;

use common::docx::{Part, W_NS, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::flags::editing_mode_conflict;
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

/// "Intro", the two tables of [`sources`]' first, "End", and a Normal
/// style whose bold is recorded as a change by "Prior": the source's only
/// revision.
fn styled_source() -> Vec<u8> {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:b/><w:rPrChange w:id="90" w:author="Prior" w:date="2001-02-03T04:05:06Z"><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    docx_with(
        &format!("{}{PLAIN}{BORDERED}{}", para("Intro"), para("End")),
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        }],
    )
}

/// A style's record is no tracked change of the text: a plan under
/// `refuse` applies, and `--editing-mode` under `keep` has nothing left to
/// conflict with.
#[test]
fn a_style_revision_does_not_make_the_text_tracked() {
    let source = styled_source();
    let changes = jubarte::changes::list_changes(&source).unwrap();
    assert!(
        !changes.is_empty() && changes.iter().all(|c| c.id.starts_with("styles.")),
        "{:?}",
        changes.iter().map(|c| &c.id).collect::<Vec<_>>()
    );
    for policy in ["refuse", "keep"] {
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Probe","existing_revisions":"{policy}","operations":[
            {{"kind":"insert_paragraph","paragraph":"p0","runs":[{{"text":"Probe paragraph."}}]}}]}}"#
        ))
        .unwrap();
        if let Err(e) = apply_plan(&source, &plan) {
            panic!("{policy}: {e}");
        }
        assert_eq!(editing_mode_conflict(&plan, &source), None, "{policy}");
    }
}

/// An `add` keeps the style's record in the clean copy and the redline,
/// and the redline keeps the untouched tables as the source has them.
#[test]
fn an_add_on_a_style_revision_keeps_adjacent_tables_apart() {
    let source = styled_source();
    let edited = add_p0(&source);
    assert_word_valid_package(&edited.redline);
    assert!(
        part_string(&edited.redline, "word/document.xml")
            .unwrap()
            .contains("<w:ins "),
        "the edit is marked"
    );
    assert_eq!(tables_xml(&edited.clean), tables_xml(&source));
    assert_eq!(tables_xml(&edited.redline), tables_xml(&source));
    assert_eq!(
        tables_xml(&accept_revisions(&edited.redline).unwrap()),
        tables_xml(&accept_revisions(&edited.clean).unwrap()),
        "accept"
    );
    for (view, docx) in [("clean", &edited.clean), ("redline", &edited.redline)] {
        let styles = part_string(docx, "word/styles.xml").unwrap();
        assert!(styles.contains("<w:rPrChange "), "{view}: {styles}");
    }
}

/// "Intro" under comment c0, the two tables of [`sources`]' first, "End";
/// c0 holds a tracked insertion by "Prior", the source's only revision.
fn commented_source() -> Vec<u8> {
    use comment_revisions::{anchor, comment, docx as with_comments};
    let note = r#"<w:p><w:r><w:t xml:space="preserve">Keep </w:t></w:r><w:ins w:id="5" w:author="Prior" w:date="2001-02-03T04:05:06Z"><w:r><w:t>this</w:t></w:r></w:ins></w:p>"#;
    with_comments(
        &format!(
            "<w:p>{}</w:p>{PLAIN}{BORDERED}{}",
            anchor(0, "Intro"),
            para("End")
        ),
        &[comment(0, "Prior", note)],
    )
}

/// A tracked insertion inside a comment, the source's only revision, is
/// still there after an `add`: the edit does not accept it on its own.
#[test]
fn an_add_keeps_a_tracked_change_inside_a_comment() {
    let source = commented_source();
    let edited = add_p0(&source);
    assert_word_valid_package(&edited.redline);
    for (view, docx) in [("clean", &edited.clean), ("redline", &edited.redline)] {
        let comments = part_string(docx, "word/comments.xml").unwrap();
        assert!(comments.contains("<w:ins "), "{view}: {comments}");
    }
    assert_eq!(tables_xml(&edited.redline), tables_xml(&source));
}

/// The comparer under an edit's settings leaves its inputs' revisions as
/// they are when the two inputs are the same bytes, as when they differ.
#[test]
fn an_edit_compare_of_identical_inputs_keeps_their_revisions() {
    let source = commented_source();
    let settings = WmlComparerSettings {
        preserve_untouched: true,
        ..WmlComparerSettings::default()
    };
    let redline = compare_documents_with_settings(&source, &source, &settings).unwrap();
    assert_word_valid_package(&redline);
    let comments = part_string(&redline, "word/comments.xml").unwrap();
    assert!(comments.contains("<w:ins "), "{comments}");
    assert_eq!(tables_xml(&redline), tables_xml(&source));
}
