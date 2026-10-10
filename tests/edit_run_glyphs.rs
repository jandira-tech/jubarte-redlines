// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tabs, line breaks and non-breaking hyphens in edited text. A `\t` or
//! `\n` in new text writes a `w:tab` or a `w:br`, as the agent view prints
//! them, and an edit may cover the ones a paragraph already holds.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, EditResult, apply_plan};
use jubarte::inspect::paragraphs;

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn body(bytes: &[u8]) -> String {
    part_string(bytes, "word/document.xml").unwrap()
}

/// The clean copy reads `want`, the redline accepts to it and rejects back
/// to the source, and both are Word-valid.
fn check(source: &[u8], out: &EditResult, want: &[&str]) {
    assert_eq!(texts(&out.clean), want);
    assert_eq!(texts(&accept_revisions(&out.redline).unwrap()), want);
    assert_eq!(
        texts(&reject_revisions(&out.redline).unwrap()),
        texts(source)
    );
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
}

#[test]
fn tabs_and_breaks_in_new_text_are_written_as_a_tab_and_a_break() {
    let source = docx(&(para("Liability is capped at fees.") + &para("Name: Ann")));
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"replace","paragraph":"p0","find":"capped at fees","replacement":"capped:\n(1) fees;\tand\n(2) nothing more"},
                {"kind":"insert","paragraph":"p1","after":"Name:","text":"\tMs."},
                {"kind":"insert_paragraph","paragraph":"p1","position":"after","runs":[{"text":"Signed:\tAnn\nCounsel"}]}]"#,
        ),
    )
    .unwrap();
    check(
        &source,
        &out,
        &[
            "Liability is capped:\n(1) fees;\tand\n(2) nothing more.",
            "Name:\tMs. Ann",
            "Signed:\tAnn\nCounsel",
        ],
    );
    let clean = body(&out.clean).replace(" />", "/>");
    assert_eq!(clean.matches("<w:br/>").count(), 3, "{clean}");
    assert_eq!(clean.matches("<w:tab/>").count(), 3, "{clean}");
    assert!(!clean.contains('\t') && !clean.contains("\n("), "{clean}");
}

#[test]
fn rewrite_writes_the_tabs_and_breaks_it_is_given() {
    let source = docx(&para("Fees: ten dollars a month."));
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"rewrite","paragraph":"p0","text":"Fees:\tten dollars\na month."}]"#),
    )
    .unwrap();
    check(&source, &out, &["Fees:\tten dollars\na month."]);
}

#[test]
fn rewrite_keeps_tabs_written_as_spaces_and_says_when_nothing_changed() {
    let by = r#"<w:p><w:r><w:t xml:space="preserve">By: </w:t><w:tab/><w:tab/><w:t xml:space="preserve">By: </w:t><w:tab/></w:r></w:p>"#;
    let source = docx(by);
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"rewrite","paragraph":"p0","text":"By: By: "}]"#),
    )
    .unwrap();
    assert_eq!(texts(&out.clean), ["By: \t\tBy: \t"]);
    let op = &out.report.operations[0];
    assert_eq!(op.status, "ok");
    assert!(
        op.message
            .as_deref()
            .is_some_and(|m| m.contains("nothing changed")),
        "{op:?}"
    );
    assert_eq!(out.report.revisions.total, 0);
}

#[test]
fn rewrite_removes_the_tab_between_words_it_deletes() {
    let source = docx(
        r#"<w:p><w:r><w:t>Name:</w:t><w:tab/><w:t xml:space="preserve">John Smith</w:t></w:r></w:p>"#,
    );
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"rewrite","paragraph":"p0","text":"John Smith"}]"#),
    )
    .unwrap();
    check(&source, &out, &["John Smith"]);
}

#[test]
fn a_replace_or_a_delete_may_cover_non_breaking_hyphens_tabs_and_breaks() {
    // Word writes a non-breaking hyphen as `w:noBreakHyphen`, not as U+2011.
    let source = docx(&format!(
        "{}{}",
        r#"<w:p><w:r><w:t xml:space="preserve">the attorney</w:t><w:noBreakHyphen/><w:t>in</w:t><w:noBreakHyphen/><w:t xml:space="preserve">fact signs</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Name:</w:t><w:tab/><w:t xml:space="preserve">Ann</w:t><w:br/><w:t>Counsel</w:t></w:r></w:p>"#,
    ));
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"replace","paragraph":"p0","find":"attorney‑in‑fact","replacement":"agent"},
                {"kind":"delete","paragraph":"p1","find":"Name:\t"},
                {"kind":"replace","paragraph":"p1","find":"Ann\nCounsel","replacement":"Ann Counsel"}]"#,
        ),
    )
    .unwrap();
    check(&source, &out, &["the agent signs", "Ann Counsel"]);
}

#[test]
fn text_may_be_inserted_beside_a_tab() {
    let source =
        docx(r#"<w:p><w:r><w:t>Name:</w:t><w:tab/></w:r><w:r><w:tab/><w:t>Ann</w:t></w:r></w:p>"#);
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"insert","paragraph":"p0","after":"Name:\t","text":"Ms."}]"#),
    )
    .unwrap();
    check(&source, &out, &["Name:\tMs.\tAnn"]);
}

#[test]
fn a_carriage_return_or_another_control_character_is_refused_by_name() {
    let source = docx(&para("Fees apply."));
    for (text, says) in [
        ("a\rb", "U+000D"),
        ("a\u{1b}b", "U+001B"),
        ("a\u{ffff}b", "U+FFFF"),
    ] {
        for kind in ["replace", "rewrite"] {
            let op = match kind {
                "replace" => format!(
                    r#"[{{"kind":"replace","paragraph":"p0","find":"Fees","replacement":{}}}]"#,
                    serde_json::to_string(text).unwrap()
                ),
                _ => format!(
                    r#"[{{"kind":"rewrite","paragraph":"p0","text":{}}}]"#,
                    serde_json::to_string(text).unwrap()
                ),
            };
            let e = apply_plan(&source, &plan(&op)).unwrap_err();
            assert_eq!(e.code, "INVALID_EDIT", "{kind} {text:?}: {e}");
            assert!(e.message.contains(says), "{kind} {text:?}: {e}");
        }
    }
}
