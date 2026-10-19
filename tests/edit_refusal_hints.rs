// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1108: an edit refused for touching another author's deletion or move
//! names the revision, its author and its text, then accept, reject and a
//! comment, each with what it costs, in the syntax the caller wrote. No
//! refusal tells the caller to "settle it first".
mod common;

use common::docx::docx;
use jubarte::edit::{EditPlan, apply_plan};
use serde_json::{Value, json};
use std::process::Command;

const DATE: &str = "2026-10-18T10:00:00Z";
const OLD: &str = "2026-10-17T09:00:00Z";
/// Word 16: Bob deleted "bankers" (r4) inside Alma Sato Reyes's insertion.
const STEP2: &[u8] = include_bytes!("fixtures/re-dialect/step2-tracked.docx");

fn plan(operations: Value) -> EditPlan {
    let mut request = json!({"schema_version":1,"author":"Carol Diaz","date":DATE,
        "existing_revisions":"keep"});
    request["operations"] = operations;
    EditPlan::from_json(&request.to_string()).unwrap()
}

fn replace(find: &str, text: &str) -> Value {
    json!([{"kind":"replace","paragraph":"p0","find":find,"replacement":text}])
}

#[test]
fn deletion_hint_offers_both_settles_then_comment() {
    let e = apply_plan(STEP2, &plan(replace("bankers", "brokers"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(e.operation.as_deref(), Some("op-1"));
    assert_eq!(
        e.message,
        [
            r#"the text is inside r4, "bankers" deleted by "Bob"."#,
            r#"  let it go:            "resolve_revisions":{"accept":{"ids":["r4"]}} (their words leave the document for good)"#,
            r#"  put it back silently: "resolve_revisions":{"reject":{"ids":["r4"]}} (their deletion disappears, no trace)"#,
            r#"  or comment on it:     {"kind":"comment","paragraph":"p0","find":"…","text":"…"}"#,
        ]
        .join("\n")
    );
}

#[test]
fn an_anchor_reaching_into_their_deletion_names_it() {
    // "pay the bankers" runs from visible text into the deleted word.
    let e = apply_plan(STEP2, &plan(replace("the bankers", "the brokers"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(
        e.message
            .starts_with(r#"the text is inside r4, "bankers" deleted by "Bob"."#),
        "{e}"
    );
}

#[test]
fn text_that_is_nowhere_stays_anchor_not_found() {
    let e = apply_plan(STEP2, &plan(replace("lenders", "brokers"))).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{e}");
}

#[test]
fn following_the_reject_route_frees_the_text() {
    let mut request = json!({"schema_version":1,"author":"Carol Diaz","date":DATE,
        "existing_revisions":"keep","resolve_revisions":{"reject":{"ids":["r4"]}},
        "operations":[]});
    request["operations"] = replace("bankers", "brokers");
    let out = apply_plan(STEP2, &EditPlan::from_json(&request.to_string()).unwrap()).unwrap();
    let text = jubarte::inspect::paragraphs(&out.clean).unwrap()[0]
        .text
        .clone();
    assert!(text.contains("pay the brokers"), "{text}");
}

fn moved(kind: &str, t: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">alpha </w:t></w:r><w:{kind} w:id="6" w:author="Alice" w:date="{OLD}"><w:r><w:{t}>beta</w:{t}></w:r></w:{kind}><w:r><w:t xml:space="preserve"> gamma</w:t></w:r></w:p>"#
    ))
}

#[test]
fn move_hint_names_the_move_and_its_routes() {
    for (kind, t, verb) in [
        ("moveTo", "t", "moved here"),
        ("moveFrom", "delText", "moved away"),
    ] {
        let e = apply_plan(&moved(kind, t), &plan(replace("beta", "delta"))).unwrap_err();
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{kind}: {e}");
        let lines: Vec<&str> = e.message.lines().collect();
        assert_eq!(
            lines[0],
            format!(r#"the text is inside r6, "beta" {verb} by "Alice"."#),
            "{kind}: {e}"
        );
        assert!(lines[1].starts_with("  keep the move:"), "{e}");
        assert!(lines[1].contains(r#"{"accept":{"ids":["r6"]}}"#), "{e}");
        assert!(lines[2].starts_with("  undo it silently:"), "{e}");
        assert!(lines[2].contains(r#"{"reject":{"ids":["r6"]}}"#), "{e}");
        assert!(lines[3].starts_with("  or comment on it:"), "{e}");
    }
}

#[test]
fn a_field_inside_their_insertion_names_the_field_not_the_insertion() {
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD}"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TITLE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Master Agreement</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:ins></w:p>"#
    ));
    let e = apply_plan(&source, &plan(replace("Master", "Main"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(e.message.contains("the result of a field"), "{e}");
    assert!(!e.message.contains("resolve_revisions"), "{e}");
}

/// "alpha beta", centred by Alice as tracked change r9.
fn reformatted() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="9" w:author="Alice" w:date="{OLD}"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>alpha beta</w:t></w:r></w:p>"#
    ))
}

#[test]
fn a_tracked_formatting_change_is_named_with_its_routes() {
    let e = apply_plan(
        &reformatted(),
        &plan(json!([{"kind":"format_paragraph","paragraph":"p0","alignment":"right"}])),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(
        e.message,
        [
            r#"under keep, a paragraph whose formatting change is still tracked cannot be formatted again: that change is r9, "alpha beta" reformatted by "Alice"."#,
            r#"  keep their formatting: "resolve_revisions":{"accept":{"ids":["r9"]}} (their formatting stays, unmarked)"#,
            r#"  undo it silently:      "resolve_revisions":{"reject":{"ids":["r9"]}} (the old formatting comes back, no trace)"#,
            r#"  or comment on it:      {"kind":"comment","paragraph":"p0","find":"…","text":"…"}"#,
        ]
        .join("\n")
    );
}

#[test]
fn following_the_formatting_route_frees_the_paragraph() {
    let mut request = json!({"schema_version":1,"author":"Carol Diaz","date":DATE,
        "existing_revisions":"keep","resolve_revisions":{"accept":{"ids":["r9"]}}});
    request["operations"] =
        json!([{"kind":"format_paragraph","paragraph":"p0","alignment":"right"}]);
    apply_plan(
        &reformatted(),
        &EditPlan::from_json(&request.to_string()).unwrap(),
    )
    .unwrap();
}

#[test]
fn a_tracked_run_format_is_named_with_its_text() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">alpha </w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Alice" w:date="{OLD}"><w:rPr/></w:rPrChange></w:rPr><w:t>beta</w:t></w:r></w:p>"#
    ));
    let e = apply_plan(
        &source,
        &plan(
            json!([{"kind":"format_run","paragraph":"p0","find":"beta","format":{"italic":true}}]),
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(
        e.message.lines().next().unwrap(),
        r#"under keep, text whose formatting change is still tracked cannot be formatted again: that change is r5, "beta" reformatted by "Alice"."#
    );
}

#[test]
fn a_tracked_mark_is_named_without_text() {
    let source = docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="7" w:author="Bob" w:date="{OLD}"/></w:rPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p><w:p><w:r><w:t>two</w:t></w:r></w:p>"#
    ));
    let e = apply_plan(
        &source,
        &plan(json!([{"kind":"merge_paragraphs","paragraph":"p0","separator":" "}])),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    let lines: Vec<&str> = e.message.lines().collect();
    assert_eq!(
        lines[0],
        r#"under keep, a paragraph whose mark is already tracked cannot be merged: that mark is r7, inserted by "Bob"."#
    );
    assert!(lines[1].starts_with("  keep it:"), "{e}");
    assert!(!e.message.contains("first with resolve_revisions"), "{e}");
}

fn cli(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn hint_quotes_the_active_plan_syntax() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), STEP2).unwrap();
    let says = |out: &std::process::Output| {
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    };
    // Flags.
    let out = cli(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--anchor",
            "bankers",
            "--content",
            "brokers",
            "-a",
            "Carol Diaz",
            "-d",
            DATE,
            "-o",
            "out.docx",
        ],
    );
    let flags = says(&out);
    assert!(!out.status.success(), "{flags}");
    for want in ["-p r4 --accept", "-p r4 --reject", "add -p p0 --anchor"] {
        assert!(flags.contains(want), "flags lack {want}: {flags}");
    }
    assert!(!flags.contains("resolve_revisions"), "{flags}");
    // A v1 plan.
    std::fs::write(
        dir.path().join("plan.json"),
        plan(replace("bankers", "brokers")).to_json(),
    )
    .unwrap();
    let out = cli(
        dir.path(),
        &["edit", "in.docx", "--plan", "plan.json", "-o", "out.docx"],
    );
    let v1 = says(&out);
    assert!(
        v1.contains(r#"\"accept\":{\"ids\":[\"r4\"]}"#)
            || v1.contains(r#""accept":{"ids":["r4"]}"#),
        "{v1}"
    );
    assert!(!v1.contains("--accept"), "{v1}");
    // A v2 plan.
    let generated = cli(
        dir.path(),
        &[
            "edit",
            "template",
            "in.docx",
            "-a",
            "Carol Diaz",
            "-d",
            DATE,
            "-o",
            "-",
        ],
    );
    assert!(generated.status.success(), "{}", says(&generated));
    let mut q: Value = serde_json::from_slice(&generated.stdout).unwrap();
    q["operations"] = json!([{"kind":"replace","target":q["operations"][0]["target"],
        "find":"bankers","text":"brokers"}]);
    std::fs::write(dir.path().join("plan2.json"), q.to_string()).unwrap();
    let out = cli(
        dir.path(),
        &[
            "edit",
            "apply",
            "in.docx",
            "--plan",
            "plan2.json",
            "-o",
            "out.docx",
        ],
    );
    let v2 = says(&out);
    assert!(!out.status.success(), "{v2}");
    assert!(
        v2.contains("short_id") && v2.contains("document.w:del.w:id.4"),
        "{v2}"
    );
    assert!(!v2.contains("--accept"), "{v2}");
    assert!(
        !v2.contains(r#"{"ids":["r4"]}"#) && !v2.contains(r#"{\"ids\":[\"r4\"]}"#),
        "{v2}"
    );
}
