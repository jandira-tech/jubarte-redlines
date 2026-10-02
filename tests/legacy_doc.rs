// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A Word 97-2003 `.doc` (or an encrypted document, the same OLE container)
//! is `LEGACY_DOC` with a save-as hint on every entry point, and RTF is
//! `UNSUPPORTED_PACKAGE`: `admission::sniff` runs before the budgets.

use jubarte::admission::code_first;
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::comparer::WmlComparerSettings;
use jubarte::convert::docx_to_pdf;
use jubarte::document_comparer::{
    accept_revisions, compare_documents, get_revisions, reject_revisions,
};
use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};

fn ole() -> Vec<u8> {
    let mut bytes = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
    bytes.resize(4096, 0);
    bytes
}

fn docx() -> Vec<u8> {
    markdown_to_docx("Plain text.", &DocxOptions::default())
        .expect("docx")
        .docx
}

/// The message as the CLI and the bindings print it: code first.
fn printed(error: &impl std::fmt::Display) -> String {
    let message = error.to_string();
    code_first(&message).unwrap_or(message)
}

#[track_caller]
fn assert_legacy(what: &str, error: &impl std::fmt::Display) {
    let message = printed(error);
    assert!(message.starts_with("LEGACY_DOC: "), "{what}: {message}");
    assert!(message.contains("save it as .docx"), "{what}: {message}");
}

#[test]
fn legacy_doc_has_the_same_code_on_every_entry_point() {
    let (doc, ok) = (ole(), docx());
    let settings = WmlComparerSettings::default();
    assert_legacy("compare A", &compare_documents(&doc, &ok, "A").unwrap_err());
    assert_legacy("compare B", &compare_documents(&ok, &doc, "A").unwrap_err());
    assert_legacy("accept", &accept_revisions(&doc).unwrap_err());
    assert_legacy("reject", &reject_revisions(&doc).unwrap_err());
    assert_legacy("revisions", &get_revisions(&doc, &settings).unwrap_err());
    assert_legacy("changes", &list_changes(&doc).unwrap_err());
    let all = ChangeFilter::default();
    assert_legacy("accept changes", &accept_changes(&doc, &all).unwrap_err());
    assert_legacy("reject changes", &reject_changes(&doc, &all).unwrap_err());
    assert_legacy("convert", &docx_to_pdf(&doc).unwrap_err());
    let markdown = docx_to_markdown(&doc, &MarkdownOptions::default()).unwrap_err();
    assert_legacy("markdown", &markdown);
    assert_legacy(
        "inspect",
        &jubarte::inspect::inspect_json(&doc).unwrap_err(),
    );
    let comments = jubarte::comments::list_comments(&doc).unwrap_err();
    assert_eq!(comments.code(), "LEGACY_DOC", "{comments}");
    let appended = jubarte::append::append_documents(&ok, &doc, &Default::default()).unwrap_err();
    assert_eq!(appended.code(), "LEGACY_DOC", "{appended}");
    let plan = r#"{"schema_version":1,"author":"A","operations":[]}"#;
    let edit = jubarte::edit::apply_plan_json(&doc, plan).unwrap_err();
    assert_eq!(edit.code, "LEGACY_DOC", "{}", edit.message);
}

#[test]
fn rtf_is_unsupported_package_on_the_same_entry_points() {
    let rtf = b"{\\rtf1\\ansi Plain text.}".to_vec();
    let message = printed(&compare_documents(&rtf, &docx(), "A").unwrap_err());
    assert!(message.starts_with("UNSUPPORTED_PACKAGE: "), "{message}");
    assert!(message.contains("RTF"), "{message}");
    let message = printed(&docx_to_pdf(&rtf).unwrap_err());
    assert!(message.starts_with("UNSUPPORTED_PACKAGE: "), "{message}");
    let message = printed(&jubarte::inspect::inspect_json(&rtf).unwrap_err());
    assert!(message.starts_with("UNSUPPORTED_PACKAGE: "), "{message}");
}
