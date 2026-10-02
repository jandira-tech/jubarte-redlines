// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `insert_toc` operation and a plan's `"update_fields": true`: the
//! clean copy carries a TOC filled from jubarte's layout, the redline
//! inserts it, and accepting the redline gives the clean copy back.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::accept_revisions;
use jubarte::edit::{EditPlan, apply_plan, preview_plan};
use jubarte::inspect::{paragraphs, source_sha256};

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn heading(text: &str) -> String {
    format!(r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
}

fn source() -> Vec<u8> {
    docx(
        &(para("Cover")
            + &heading("Scope")
            + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#
            + &heading("Terms")),
    )
}

fn plan(source: &[u8], update_fields: bool, operations: &str) -> EditPlan {
    let json = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-10-02T12:00:00Z","update_fields":{update_fields},"operations":{operations}}}"#,
        source_sha256(source)
    );
    EditPlan::from_json(&json).unwrap()
}

#[test]
fn insert_toc_with_update_fields_writes_the_entries() {
    let source = source();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            true,
            r#"[{"kind":"insert_toc","paragraph":{"index":0},"position":"after","levels":2,"title":"Contents"}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    let clean = texts(&result.clean);
    assert_eq!(
        &clean[..4],
        ["Cover", "Contents", "Scope\t1", "Terms\t2"],
        "{clean:?}"
    );
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert!(xml.contains(r#" TOC \o "1-2" \h \z \u "#), "{xml}");
    assert!(xml.contains("w:val=\"TOCHeading\""), "{xml}");
    let op = &result.report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("insert_toc", "ok"));
    assert_eq!(op.paragraph.as_deref(), Some("body:p:0"));
    let fields = &result.report.fields;
    assert_eq!(fields.iter().filter(|f| f.kind == "PAGEREF").count(), 2);
    assert!(fields.iter().any(|f| f.kind == "TOC"));
    // The redline inserts the TOC; accepting it gives the clean text.
    assert!(result.report.revisions.inserted > 0);
    assert_eq!(texts(&accept_revisions(&result.redline).unwrap()), clean);
}

#[test]
fn insert_toc_without_update_fields_leaves_an_empty_field_and_says_so() {
    let source = source();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            false,
            r#"[{"kind":"insert_toc","paragraph":{"index":1},"position":"before"}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&result.clean);
    assert_eq!(texts(&result.clean)[..3], ["Cover", "", "Scope"]);
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert!(xml.contains(r#" TOC \o "1-3" \h \z \u "#), "{xml}");
    assert!(result.report.fields.is_empty());
    let message = result.report.operations[0].message.as_deref().unwrap();
    assert!(message.contains("update_fields"), "{message}");
}

#[test]
fn update_fields_alone_refreshes_existing_fields() {
    let source = docx(
        &(para("One")
            + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#
            + r#"<w:p><w:r><w:t xml:space="preserve">Pages: </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>9</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#),
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            true,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"One","replacement":"Uno"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), ["Uno", "", "Pages: 2"]);
    assert_eq!(result.report.fields.len(), 1);
}

#[test]
fn insert_toc_refuses_bad_levels_titles_and_stories() {
    let source = source();
    let refuse = |operations: &str| {
        apply_plan(&source, &plan(&source, true, operations))
            .unwrap_err()
            .code
    };
    assert_eq!(
        refuse(r#"[{"kind":"insert_toc","paragraph":{"index":0},"levels":0}]"#),
        "INVALID_EDIT"
    );
    assert_eq!(
        refuse(r#"[{"kind":"insert_toc","paragraph":{"index":0},"levels":10}]"#),
        "INVALID_EDIT"
    );
    assert_eq!(
        refuse(r#"[{"kind":"insert_toc","paragraph":{"index":0},"title":"a\tb"}]"#),
        "INVALID_EDIT"
    );
    let err = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[{"kind":"insert_toc","paragraph":{"index":0},"depth":2}]}"#,
    )
    .unwrap_err();
    assert_eq!(err.code, "INVALID_PLAN");
}

#[test]
fn a_plan_without_update_fields_serializes_without_the_key() {
    let source = source();
    let json = plan(&source, false, "[]").to_json();
    assert!(!json.contains("update_fields"), "{json}");
    let json = plan(&source, true, "[]").to_json();
    assert!(json.contains("\"update_fields\": true"), "{json}");
}

#[test]
fn preview_resolves_insert_toc() {
    let source = source();
    let report = preview_plan(
        &source,
        &plan(
            &source,
            true,
            r#"[{"kind":"insert_toc","paragraph":{"starts_with":"Cover"}}]"#,
        ),
    )
    .unwrap();
    assert_eq!(report.operations[0].status, "ok");
    assert!(report.fields.is_empty());
}
