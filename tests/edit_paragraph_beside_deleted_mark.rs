// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Inserting a paragraph after one whose mark is a pending deletion
//! reports that accepting the mark merges the two.

mod common;

use common::docx::{docx, part_string};
use jubarte::changes::{ChangeFilter, accept_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const DATE: &str = "2026-10-18T12:00:00Z";

fn marked(text: &str) -> String {
    format!(r#"<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"#)
}

fn source() -> Vec<u8> {
    let mark = r#"<w:p><w:pPr><w:rPr><w:del w:id="88" w:author="Trainer 1" w:date="2026-10-17T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>three</w:t></w:r></w:p>"#;
    docx(&format!(
        "{}{}{}{mark}{}",
        marked("zero"),
        marked("one"),
        marked("two"),
        marked("four")
    ))
}

fn plan(paragraph: &str, position: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"{DATE}","existing_revisions":"keep","operations":[{{"kind":"insert_paragraph","paragraph":"{paragraph}","position":"{position}","runs":[{{"text":"Added."}}]}}]}}"#
    ))
    .unwrap()
}

fn message_of(paragraph: &str, position: &str) -> Option<String> {
    let out = apply_plan(&source(), &plan(paragraph, position)).expect("the insert applies");
    assert_eq!(out.report.operations[0].status, "ok");
    out.report.operations[0].message.clone()
}

#[test]
fn insert_after_a_pending_deleted_mark_reports_the_merge() {
    let expected = "p3's paragraph mark is a pending deletion by Trainer 1 (r88); if it is accepted, p3 and the new paragraph merge; restore r88 to keep them apart";
    let out = apply_plan(&source(), &plan("p3", "after")).expect("the insert applies");
    assert_eq!(out.report.operations[0].status, "ok");
    assert_eq!(out.report.operations[0].message.as_deref(), Some(expected));
    let xml = part_string(&out.redline, "word/document.xml").expect("document");
    assert!(
        xml.contains(r#"<w:del w:id="88" w:author="Trainer 1""#),
        "the mark stays deleted:\n{xml}"
    );
    let accepted = accept_changes(&out.redline, &ChangeFilter::default()).expect("accept");
    let texts: Vec<String> = paragraphs(&accepted)
        .expect("paragraphs")
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("three") && t.contains("Added.")),
        "{texts:?}"
    );
}

#[test]
fn insert_elsewhere_stays_silent() {
    assert_eq!(message_of("p3", "before"), None);
    assert_eq!(message_of("p2", "after"), None);
}
