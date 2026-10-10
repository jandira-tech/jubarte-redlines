// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Anchors with Markdown marks fall back to the plain text.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn source() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Chapter 1</w:t></w:r></w:p>{}{}"#,
        para("keep it secret."),
        para("**stars** here")
    ))
}

#[test]
fn marks_are_dropped_when_the_literal_anchor_is_absent_and_the_report_says_so() {
    for (paragraph, find, read_as, replacement) in [
        ("p0", "# Chapter 1", "Chapter 1", "Chapter One"),
        ("p1", "**secret**", "secret", "confidential"),
        ("p1", "<u>secret</u>", "secret", "confidential"),
        (
            "p1",
            "{==keep it secret.==}{>>#c5 @AC: Cap?<<}",
            "keep it secret.",
            "keep it.",
        ),
        (
            "p1",
            "keep {++it++}{>>#0 @AC<<} secret",
            "keep it secret",
            "keep secret",
        ),
        (
            "p1",
            "keep {~~that~>it~~}{>>#0 @AC<<} secret",
            "keep it secret",
            "keep secret",
        ),
        (
            "p1",
            "keep it {--very --}{>>#1 @AC<<}secret",
            "keep it secret",
            "keep secret",
        ),
        ("p2", "\\*\\*stars\\*\\*", "**stars**", "asterisks"),
    ] {
        let out = apply_plan(
            &source(),
            &plan(&format!(
                r#"[{{"kind":"replace","paragraph":"{paragraph}","find":{},"replacement":"{replacement}"}}]"#,
                serde_json::to_string(find).unwrap()
            )),
        )
        .unwrap_or_else(|e| panic!("{find}: {e}"));
        let op = &out.report.operations[0];
        assert_eq!(op.anchor_given.as_deref(), Some(find), "{find}");
        assert_eq!(op.anchor_read_as.as_deref(), Some(read_as), "{find}");
        let line = out
            .report
            .to_jsonl()
            .lines()
            .find(|l| l.contains(r#""ev":"op""#))
            .unwrap()
            .to_string();
        assert!(
            line.contains(&format!(
                r#""anchor_given":{},"anchor_read_as":{}"#,
                serde_json::to_string(find).unwrap(),
                serde_json::to_string(read_as).unwrap()
            )),
            "{line}"
        );
    }
}

#[test]
fn a_literal_match_wins_and_carries_no_note() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p2","find":"**stars**","replacement":"asterisks"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.operations[0].anchor_read_as.is_none());
    assert!(!out.report.to_jsonl().contains("anchor_given"));
    let e = apply_plan(
        &source(),
        &plan(r##"[{"kind":"replace","paragraph":"p0","find":"# Missing","replacement":"x"}]"##),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert!(e.to_string().contains("\"# Missing\""), "{e}");
}

#[test]
fn a_lone_star_or_underscore_is_text_not_a_mark() {
    for (find, plain) in [
        ("a_b", "a_b"),
        ("*x*", "x"),
        ("2 * 3", "2 * 3"),
        ("\\\\*", "\\*"),
        ("snake_case and _x_", "snake_case and x"),
    ] {
        assert_eq!(jubarte::markdown::plain_anchor(find), plain, "{find}");
    }
}

/// pi review r392b tests F1: `format_run` and `insert_footnote` read a marked
/// anchor as the other kinds do, so `--anchor '**secret**' --style bold`
/// works on text the view shows bold.
#[test]
fn format_run_and_insert_footnote_drop_marks_too() {
    for op in [
        r#"{"kind":"format_run","paragraph":"p1","find":"**secret**","format":{"bold":true}}"#,
        r#"{"kind":"insert_footnote","paragraph":"p1","after":"**secret**","text":"Why."}"#,
    ] {
        let out = apply_plan(&source(), &plan(&format!("[{op}]")))
            .unwrap_or_else(|e| panic!("{op}: {e}"));
        let report = &out.report.operations[0];
        assert_eq!(report.anchor_given.as_deref(), Some("**secret**"), "{op}");
        assert_eq!(report.anchor_read_as.as_deref(), Some("secret"), "{op}");
    }
}
