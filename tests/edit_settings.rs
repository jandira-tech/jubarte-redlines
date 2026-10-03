// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! S5: the `settings` plan operation writes `w:trackRevisions`,
//! `w:updateFields` and `w:documentProtection` in `CT_Settings` order, into
//! the clean copy and the redline alike (they are not revisions).

mod common;

use common::docx::{Part, W_NS, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::compare_documents;
use jubarte::edit::{EditPlan, apply_plan, preview_plan};
use jubarte::inspect::summary;

const SETTINGS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml";
const SETTINGS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings";

/// `word/settings.xml` with empty elements spelled `<x/>`: the engine's
/// serializer writes `<x />`, as .NET's does; Word reads both.
fn settings_xml(docx: &[u8]) -> String {
    part_string(docx, "word/settings.xml")
        .unwrap()
        .replace(" />", "/>")
}

fn plan(ops: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":[{ops}]}}"#
    ))
    .unwrap()
}

fn with_settings(children: &str) -> Vec<u8> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:settings xmlns:w="{W_NS}">{children}</w:settings>"#
    );
    docx_with(
        &para("x"),
        &[Part {
            name: "word/settings.xml",
            content_type: SETTINGS_CT,
            rel_type: SETTINGS_REL,
            xml: &xml,
        }],
    )
}

#[test]
fn settings_are_written_in_schema_order_into_a_new_settings_part() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"A","operations":[
        {"kind":"settings","track_revisions":true,"update_fields":true,"protection":{"edit":"trackedChanges"}}]}"#).unwrap();
    let out = apply_plan(&docx(&para("x")), &plan).unwrap();
    assert_word_valid_package(&out.clean);
    let xml = settings_xml(&out.clean);
    let track = xml.find("<w:trackRevisions/>").unwrap();
    let prot = xml
        .find(r#"<w:documentProtection w:edit="trackedChanges" w:enforcement="1"/>"#)
        .unwrap();
    let update = xml.find("<w:updateFields/>").unwrap();
    assert!(track < prot && prot < update, "{xml}");
    assert!(summary(&out.clean).unwrap().track_changes);
}

#[test]
fn an_existing_settings_part_keeps_its_children_and_takes_the_new_ones_in_order() {
    let source = with_settings(
        r#"<w:zoom w:percent="100"/><w:defaultTabStop w:val="720"/><w:characterSpacingControl w:val="doNotCompress"/><w:compat/>"#,
    );
    let out = apply_plan(
        &source,
        &plan(
            r#"{"kind":"settings","track_revisions":true,"update_fields":true,"protection":{"edit":"readOnly","enforcement":false}}"#,
        ),
    )
    .unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        let xml = settings_xml(doc);
        let at = |needle: &str| {
            xml.find(needle)
                .unwrap_or_else(|| panic!("{needle} missing: {xml}"))
        };
        let order = [
            at("<w:zoom "),
            at("<w:trackRevisions/>"),
            at(r#"<w:documentProtection w:edit="readOnly" w:enforcement="0"/>"#),
            at("<w:defaultTabStop "),
            at("<w:characterSpacingControl "),
            at("<w:updateFields/>"),
            at("<w:compat/>"),
        ];
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{xml}");
    }
}

#[test]
fn false_and_none_remove_what_is_there_and_a_second_write_replaces() {
    let source = with_settings(
        r#"<w:zoom w:percent="100"/><w:trackRevisions/><w:documentProtection w:edit="forms" w:enforcement="1"/><w:updateFields/>"#,
    );
    let off = apply_plan(
        &source,
        &plan(
            r#"{"kind":"settings","track_revisions":false,"update_fields":false,"protection":{"edit":"none"}}"#,
        ),
    )
    .unwrap();
    let xml = settings_xml(&off.clean);
    for gone in ["trackRevisions", "updateFields", "documentProtection"] {
        assert!(!xml.contains(gone), "{gone}: {xml}");
    }
    assert!(xml.contains("<w:zoom "), "{xml}");
    assert!(!summary(&off.clean).unwrap().track_changes);
    let replaced = apply_plan(
        &source,
        &plan(r#"{"kind":"settings","protection":{"edit":"comments"}}"#),
    )
    .unwrap();
    let xml = settings_xml(&replaced.clean);
    assert_eq!(xml.matches("<w:documentProtection ").count(), 1, "{xml}");
    assert!(
        xml.contains(r#"<w:documentProtection w:edit="comments" w:enforcement="1"/>"#),
        "{xml}"
    );
    // Untouched settings stay.
    assert!(xml.contains("<w:trackRevisions/>") && xml.contains("<w:updateFields/>"));
}

#[test]
fn the_redline_carries_the_settings_and_no_change_for_them() {
    let out = apply_plan(
        &docx(&para("Payment is due in 30 days.")),
        &plan(
            r#"{"kind":"settings","track_revisions":true},
               {"kind":"replace","paragraph":"body:p:0","find":"30","replacement":"45"}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.redline);
    assert!(summary(&out.redline).unwrap().track_changes);
    let texts: Vec<String> = jubarte::changes::list_changes(&out.redline)
        .unwrap()
        .into_iter()
        .map(|c| c.text)
        .collect();
    let mut sorted = texts.clone();
    sorted.sort();
    assert_eq!(sorted, ["30", "45"], "{texts:?}");
    assert_eq!(out.report.operations[0].kind, "settings");
    assert_eq!(out.report.operations[0].status, "ok");
}

#[test]
fn settings_apply_under_keep() {
    let theirs = compare_documents(
        &docx(&para("Payment is due in 30 days.")),
        &docx(&para("Payment is due in 45 days.")),
        "Them",
    )
    .unwrap();
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"settings","track_revisions":true,"protection":{"edit":"trackedChanges"}}]}"#,
    )
    .unwrap();
    let out = apply_plan(&theirs, &plan).unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        let xml = settings_xml(doc);
        assert!(xml.contains("<w:trackRevisions/>"), "{xml}");
        assert!(xml.contains(r#"w:edit="trackedChanges""#), "{xml}");
    }
    let changes = jubarte::changes::list_changes(&out.redline).unwrap();
    assert!(changes.iter().all(|c| c.author.as_deref() == Some("Them")));
}

#[test]
fn refusals_name_the_problem() {
    let source = docx(&para("x"));
    let refused = |ops: &str| apply_plan(&source, &plan(ops)).unwrap_err();
    let e = refused(r#"{"kind":"settings"}"#);
    assert_eq!(e.code, "INVALID_EDIT", "{e:?}");
    let e = refused(r#"{"kind":"settings","protection":{"edit":"readOnly","password":"hunter2"}}"#);
    assert_eq!(e.code, "UNSUPPORTED", "{e:?}");
    assert!(!format!("{e:?}").contains("hunter2"), "{e:?}");
    let e = refused(
        r#"{"kind":"settings","track_revisions":true},{"kind":"settings","update_fields":true}"#,
    );
    assert_eq!(e.code, "OVERLAPPING_EDITS", "{e:?}");
    assert_eq!(e.operation.as_deref(), Some("op-2"));
    // The wire form is checked before anything resolves.
    for bad in [
        r#"{"kind":"settings","paragraph":"body:p:0","track_revisions":true}"#,
        r#"{"kind":"settings","track_revision":true}"#,
        r#"{"kind":"settings","protection":{"edit":"everything"}}"#,
        r#"{"kind":"settings","protection":{"edit":"readOnly","enforce":true}}"#,
    ] {
        let e = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"A","operations":[{bad}]}}"#
        ))
        .unwrap_err();
        assert_eq!(e.code, "INVALID_PLAN", "{bad}: {e:?}");
    }
}

#[test]
fn preview_reports_the_settings_operation() {
    let report = preview_plan(
        &docx(&para("x")),
        &plan(r#"{"kind":"settings","track_revisions":true,"update_fields":false}"#),
    )
    .unwrap();
    let op = &report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("settings", "ok"));
    let context = op.context.as_deref().unwrap();
    assert!(
        context.contains("track_revisions=true") && context.contains("update_fields=false"),
        "{context}"
    );
}

#[test]
fn settings_ride_with_a_footnote_and_a_page_setup() {
    let out = apply_plan(
        &docx(&para("Payment is due in 30 days.")),
        &plan(
            r#"{"kind":"insert_footnote","paragraph":"body:p:0","after":"30 days","text":"Calendar days."},
               {"kind":"page_setup","orientation":"landscape"},
               {"kind":"settings","track_revisions":true,"protection":{"edit":"trackedChanges"}}"#,
        ),
    )
    .unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        let xml = settings_xml(doc);
        assert!(xml.contains("<w:trackRevisions/>"), "{xml}");
        assert!(xml.contains(r#"w:edit="trackedChanges""#), "{xml}");
        assert!(
            part_string(doc, "word/footnotes.xml")
                .unwrap()
                .contains("Calendar days.")
        );
    }
    assert!(
        out.report.operations.iter().all(|op| op.status == "ok"),
        "{:?}",
        out.report.operations
    );
}

#[test]
fn the_settings_update_fields_and_the_plans_are_two_things_and_both_hold() {
    let source = docx(&format!(
        "{}{}",
        para("Cover"),
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Scope</w:t></w:r></w:p>"#
    ));
    // The plan's update_fields writes the TOC entries now; the settings
    // op's asks Word to recompute them on open.
    let out = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"A","update_fields":true,"operations":[
            {"kind":"insert_toc","paragraph":"body:p:0","levels":1},
            {"kind":"settings","update_fields":true}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    for doc in [&out.clean, &out.redline] {
        assert_word_valid_package(doc);
        assert!(settings_xml(doc).contains("<w:updateFields/>"));
    }
    let texts: Vec<String> = jubarte::inspect::paragraphs(&out.clean)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert!(texts.iter().any(|t| t == "Scope\t1"), "{texts:?}");
}
