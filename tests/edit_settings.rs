// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The `settings` edit operation: `trackRevisions`, `updateFields` and
//! `documentProtection` written in `CT_Settings` order, in the clean copy
//! and the redline alike.

mod common;

use common::docx::{Part, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::edit::{EditPlan, apply_plan, preview_plan};

const SETTINGS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";
const SETTINGS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";
const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn plan(ops: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":[{ops}]}}"#
    ))
    .unwrap()
}

fn with_settings(children: &str) -> Vec<u8> {
    let xml = format!(r#"<w:settings xmlns:w="{W}">{children}</w:settings>"#);
    docx_with(
        &para("Hello"),
        &[Part {
            name: "word/settings.xml",
            content_type: SETTINGS_CT,
            rel_type: SETTINGS_REL,
            xml: &xml,
        }],
    )
}

/// The settings part, self-closing tags written `<x/>` whatever the
/// serializer's spacing.
fn settings(doc: &[u8]) -> String {
    part_string(doc, "word/settings.xml")
        .unwrap()
        .replace(" />", "/>")
}

fn index(hay: &str, needle: &str) -> usize {
    hay.find(needle)
        .unwrap_or_else(|| panic!("{needle} not in {hay}"))
}

#[test]
fn settings_are_created_in_schema_order_in_both_outputs() {
    let source = docx(&para("Hello"));
    assert!(part_string(&source, "word/settings.xml").is_none());
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"settings","track_revisions":true,"update_fields":true,
            "protection":{"edit":"trackedChanges","enforcement":true}}"#,
        ),
    )
    .unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        let s = settings(doc);
        let track = index(&s, "<w:trackRevisions/>");
        let protect = index(
            &s,
            r#"<w:documentProtection w:edit="trackedChanges" w:enforcement="1"/>"#,
        );
        let update = index(&s, "<w:updateFields/>");
        assert!(track < protect && protect < update, "{s}");
        assert!(jubarte::inspect::summary(doc).unwrap().track_changes);
    }
    assert_eq!(out.report.operations[0].kind, "settings");
    assert_eq!(out.report.operations[0].status, "ok");
    assert_eq!(out.report.revisions.total, 0);
}

#[test]
fn an_existing_part_keeps_its_children_and_gets_each_in_its_place() {
    let source = with_settings(
        r#"<w:zoom w:percent="100"/><w:defaultTabStop w:val="720"/><w:compat/><w:rsids><w:rsidRoot w:val="00A1B2C3"/></w:rsids>"#,
    );
    let out = apply_plan(
        &source,
        &plan(r#"{"kind":"settings","track_revisions":true,"update_fields":true}"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    let s = settings(&out.clean);
    let order = [
        "<w:zoom",
        "<w:trackRevisions/>",
        "<w:defaultTabStop",
        "<w:updateFields/>",
        "<w:compat",
        "<w:rsids>",
    ];
    let at: Vec<usize> = order.iter().map(|n| index(&s, n)).collect();
    assert!(at.windows(2).all(|w| w[0] < w[1]), "{s}");
}

#[test]
fn false_removes_and_a_value_replaces() {
    let source = with_settings(
        r#"<w:trackRevisions/><w:documentProtection w:edit="readOnly" w:enforcement="1"/><w:updateFields w:val="true"/>"#,
    );
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"settings","track_revisions":false,"update_fields":false,
            "protection":{"edit":"comments","enforcement":false}}"#,
        ),
    )
    .unwrap();
    let s = settings(&out.clean);
    assert!(
        !s.contains("trackRevisions") && !s.contains("updateFields"),
        "{s}"
    );
    assert!(
        s.contains(r#"<w:documentProtection w:edit="comments" w:enforcement="0"/>"#),
        "{s}"
    );
    assert!(!jubarte::inspect::summary(&out.clean).unwrap().track_changes);
    let out = apply_plan(
        &source,
        &plan(r#"{"kind":"settings","protection":{"edit":"none"}}"#),
    )
    .unwrap();
    let s = settings(&out.clean);
    assert!(
        !s.contains("documentProtection") && s.contains("trackRevisions"),
        "{s}"
    );
}

#[test]
fn settings_ride_beside_a_text_edit() {
    let out = apply_plan(
        &docx(&para("Fee is 10.")),
        &plan(
            r#"{"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"},
            {"kind":"settings","track_revisions":true}"#,
        ),
    )
    .unwrap();
    assert!(out.report.revisions.total > 0);
    for doc in [&out.clean, &out.redline] {
        assert!(jubarte::inspect::summary(doc).unwrap().track_changes);
    }
}

#[test]
fn refusals() {
    let source = docx(&para("Hello"));
    let e = apply_plan(&source, &plan(r#"{"kind":"settings"}"#)).unwrap_err();
    assert_eq!(e.code, "INVALID_EDIT");
    let e = apply_plan(
        &source,
        &plan(r#"{"kind":"settings","protection":{"edit":"readOnly","password":"x"}}"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED");
    assert_eq!(e.outcomes[0].code.as_deref(), Some("UNSUPPORTED"));
    let e = apply_plan(
        &source,
        &plan(
            r#"{"kind":"settings","track_revisions":true},
            {"kind":"settings","update_fields":true}"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "OVERLAPPING_EDITS");
    let e = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"settings","protection":{"edit":"everything"}}]}"#,
    )
    .unwrap_err();
    assert_eq!(e.code, "INVALID_PLAN");
    let e = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"settings","track_revision":true}]}"#,
    )
    .unwrap_err();
    assert_eq!(e.code, "INVALID_PLAN");
    assert!(e.message.contains("track_revision"), "{}", e.message);
    // Settings name no paragraph; one given is a mistake, not ignored.
    let e = EditPlan::from_json(
        r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"settings","paragraph":"body:p:0","track_revisions":true}]}"#,
    )
    .unwrap_err();
    assert_eq!(e.code, "INVALID_PLAN");
    assert!(e.message.contains("paragraph"), "{}", e.message);
}

#[test]
fn preview_reports_the_operation() {
    let report = preview_plan(
        &docx(&para("Hello")),
        &plan(r#"{"kind":"settings","track_revisions":true}"#),
    )
    .unwrap();
    assert_eq!(report.operations[0].kind, "settings");
    assert_eq!(report.operations[0].status, "ok");
    assert!(report.operations[0].paragraph.is_none());
}
