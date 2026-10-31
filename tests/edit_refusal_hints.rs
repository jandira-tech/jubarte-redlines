// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1108: an edit refused for touching another author's deletion or move
//! names the revision, its author and its text, then accept, reject and a
//! comment, each with what it costs, in the syntax the caller wrote. No
//! refusal tells the caller to "settle it first".
mod common;

use common::docx::{Part, W_NS, docx, docx_with_sect, para, part_string};
use jubarte::edit::v2::PlanV2;
use jubarte::edit::{
    EditError, EditPlan, PlanSyntax, PrepareOptions, apply_plan, apply_plan_json_with, v2_apply,
};
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
fn deletion_hint_offers_restore_first_then_both_settles_then_comment() {
    let e = apply_plan(STEP2, &plan(replace("bankers", "brokers"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(e.operation.as_deref(), Some("op-1"));
    assert_eq!(
        e.message,
        [
            r#"the text is inside r4, "bankers" deleted by "Bob"."#,
            r#"  put it back visibly:                            {"kind":"restore","revision":"r4"}            (their deletion stays, your insertion shows)"#,
            r#"  let it go (erases "Bob"'s deletion):            "resolve_revisions":{"accept":{"ids":["r4"]}} (their words leave the document for good)"#,
            r#"  put it back silently (erases "Bob"'s deletion): "resolve_revisions":{"reject":{"ids":["r4"]}} (their deletion disappears, no trace)"#,
            r#"  or comment on it:                               {"kind":"comment","paragraph":"p0","find":"…","text":"…"}"#,
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

#[test]
fn following_the_restore_route_puts_their_words_back_visibly() {
    let e = apply_plan(STEP2, &plan(replace("bankers", "brokers"))).unwrap_err();
    let line = e.message.lines().nth(1).unwrap();
    let route: Value = serde_json::from_str(fragment(line)).unwrap_or_else(|x| panic!("{x}: {e}"));
    let out = apply_plan(STEP2, &plan(json!([route]))).unwrap_or_else(|x| panic!("{x}: {e}"));
    let accepted = jubarte::changes::accept_changes(&out.redline, &Default::default()).unwrap();
    let text = jubarte::inspect::paragraphs(&accepted).unwrap()[0]
        .text
        .clone();
    assert!(text.contains("pay the bankers"), "{text}");
    // Bob's deletion stays, beside ours.
    let bob = jubarte::changes::list_changes(&out.redline)
        .unwrap()
        .into_iter()
        .any(|c| c.id == "r4" && c.author.as_deref() == Some("Bob"));
    assert!(bob, "{e}");
}

#[test]
fn an_edit_of_words_the_same_command_restores_says_to_edit_them_after() {
    let restore = json!({"kind":"restore","revision":"r4"});
    let e = apply_plan(
        STEP2,
        &plan(json!([restore, replace("bankers", "brokers")[0].clone()])),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(
        e.message,
        r#"the text is inside r4, "bankers" deleted by "Bob", which this command restores: edit the restored words in a command of their own, after the restore."#,
        "{e}"
    );
}

#[test]
fn a_deletion_restore_cannot_take_offers_no_visible_route() {
    // r6 is split over two stretches: restore takes one.
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">alpha </w:t></w:r><w:del w:id="6" w:author="Bob" w:date="{OLD}"><w:r><w:delText>beta</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> mid </w:t></w:r><w:del w:id="6" w:author="Bob" w:date="{OLD}"><w:r><w:delText>gamma</w:delText></w:r></w:del></w:p>"#
    ));
    let e = apply_plan(&source, &plan(replace("beta", "delta"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(
        e.message
            .lines()
            .nth(1)
            .unwrap()
            .starts_with(r#"  let it go (erases "Bob"'s deletion):"#),
        "{e}"
    );
    assert!(!e.message.contains("restore"), "{e}");
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
        assert!(
            lines[1].starts_with(r#"  keep the move (erases "Alice"'s move):"#),
            "{e}"
        );
        assert!(lines[1].contains(r#"{"accept":{"ids":["r6"]}}"#), "{e}");
        assert!(
            lines[2].starts_with(r#"  undo it silently (erases "Alice"'s move):"#),
            "{e}"
        );
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
            r#"  keep their formatting (erases "Alice"'s formatting): "resolve_revisions":{"accept":{"ids":["r9"]}} (their formatting stays, unmarked)"#,
            r#"  undo it silently (erases "Alice"'s formatting):      "resolve_revisions":{"reject":{"ids":["r9"]}} (the old formatting comes back, no trace)"#,
            r#"  or comment on it:                                    {"kind":"comment","paragraph":"p0","find":"…","text":"…"}"#,
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
    // The route that keeps Bob's insertion comes first (#1953).
    assert!(lines[1].starts_with("  strike it:"), "{e}");
    assert!(
        lines[2].starts_with(r#"  keep it (erases "Bob"'s insertion):"#),
        "{e}"
    );
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
    // What the refusal says: stderr and each JSON row's message, not the
    // structured next actions (#1147), whose argv run `edit template`.
    let refusal = |out: &std::process::Output| {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let messages = stdout
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .flat_map(|row| [row["message"].clone(), row["error"]["message"].clone()])
            .filter_map(|m| m.as_str().map(str::to_string))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{messages}\n{}", String::from_utf8_lossy(&out.stderr))
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
    for want in [
        "-p r4 --restore",
        "-p r4 --accept",
        "-p r4 --reject",
        "add -p p0 --anchor",
    ] {
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
    let v1 = refusal(&out);
    assert!(!out.status.success(), "{v1}");
    assert!(
        v1.contains(r#"\"accept\":{\"ids\":[\"r4\"]}"#)
            || v1.contains(r#""accept":{"ids":["r4"]}"#),
        "{v1}"
    );
    assert!(
        v1.contains(r#"{\"kind\":\"restore\",\"revision\":\"r4\"}"#)
            || v1.contains(r#"{"kind":"restore","revision":"r4"}"#),
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
    let v2 = refusal(&out);
    assert!(!out.status.success(), "{v2}");
    assert!(
        v2.contains("short_id") && v2.contains("document.w:del.w:id.4"),
        "{v2}"
    );
    assert!(!v2.contains("--accept"), "{v2}");
    // A v2 plan has no restore yet.
    assert!(!v2.contains("restore"), "{v2}");
    assert!(
        !v2.contains(r#"{"ids":["r4"]}"#) && !v2.contains(r#"{\"ids\":[\"r4\"]}"#),
        "{v2}"
    );
    // A typed command (#1310): its own options, added to the command. Its
    // --find reads the final text, so Bob's moved words, not his deleted ones.
    std::fs::write(
        dir.path().join("moved.docx"),
        docx(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">pay the </w:t></w:r><w:moveTo w:id="4" w:author="Bob" w:date="{DATE}"><w:r><w:t>bankers</w:t></w:r></w:moveTo></w:p>"#
        )),
    )
    .unwrap();
    let out = cli(
        dir.path(),
        &[
            "edit",
            "replace",
            "moved.docx",
            "--at",
            "p0",
            "--find",
            "bankers",
            "--text",
            "brokers",
            "-a",
            "Carol Diaz",
            "-d",
            DATE,
            "-o",
            "out.docx",
        ],
    );
    let typed = refusal(&out);
    assert!(!out.status.success(), "{typed}");
    for want in [
        "--accept-change r4",
        "--reject-change r4",
        "add -p p0 --anchor",
    ] {
        assert!(typed.contains(want), "typed lacks {want}: {typed}");
    }
    assert!(!typed.contains("resolve_revisions"), "{typed}");
    assert!(!typed.contains("-p r4 --accept"), "{typed}");
}

/// A route line's fragment: what follows its label, without the cost.
fn fragment(line: &str) -> &str {
    let rest = line.split_once(':').map_or(line, |(_, r)| r).trim_start();
    rest.rsplit_once(" (").map_or(rest, |(f, _)| f.trim_end())
}

/// A settle route line as the `resolve_revisions` it quotes.
fn decisions(line: &str) -> Value {
    let route: Value = serde_json::from_str(&format!("{{{}}}", fragment(line)))
        .unwrap_or_else(|e| panic!("not JSON ({e}): {line}"));
    route["resolve_revisions"].clone()
}

/// The v2 template plan `edit template` writes for `source`.
fn v2_plan(source: &[u8]) -> Value {
    use jubarte::edit::{
        for_edit::ForEditRequest,
        template::{TemplateRequest, generate},
    };
    let generated = generate(
        source,
        &TemplateRequest {
            view: ForEditRequest::default(),
            author: "Carol Diaz".into(),
            date: DATE.into(),
        },
    )
    .unwrap();
    serde_json::from_str(&generated.plan.to_json()).unwrap()
}

fn apply_v2(source: &[u8], q: &Value) -> Result<v2_apply::V2EditResult, EditError> {
    v2_apply::apply_plan(source, &PlanV2::from_json(&q.to_string())?)
}

#[test]
fn every_v2_route_pastes_into_the_plan_it_refused() {
    let mut q = v2_plan(STEP2);
    let target = q["operations"][0]["target"].clone();
    q["operations"] = json!([{"kind":"replace","target":target,"find":"bankers","text":"brokers"}]);
    let e = apply_v2(STEP2, &q).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    let lines: Vec<&str> = e.message.lines().collect();
    assert_eq!(lines.len(), 4, "{e}");
    // Reject: their word comes back, and the replacement lands on it.
    let mut rejected = q.clone();
    rejected["resolve_revisions"] = decisions(lines[2]);
    let out = apply_v2(STEP2, &rejected).unwrap_or_else(|x| panic!("{x}: {e}"));
    let text = jubarte::inspect::paragraphs(&out.clean).unwrap()[0]
        .text
        .clone();
    assert!(text.contains("pay the brokers"), "{text}");
    // Accept, with the comment route on the operation's own target.
    let mut comment: Value = serde_json::from_str(fragment(lines[3]))
        .unwrap_or_else(|x| panic!("not JSON ({x}): {}", lines[3]));
    assert_eq!(comment["target"], target, "{e}");
    comment["find"] = json!("pay");
    comment["text"] = json!("Why?");
    let mut accepted = q.clone();
    accepted["resolve_revisions"] = decisions(lines[1]);
    accepted["operations"] = json!([comment]);
    apply_v2(STEP2, &accepted).unwrap_or_else(|x| panic!("{x}: {e}"));
}

#[test]
fn a_glyph_or_lookalike_before_their_deletion_still_names_it() {
    for (before, deleted, find) in [
        ("<w:t>alpha</w:t><w:tab/>", "gamma", "alpha\tgamma"),
        ("<w:t>alpha</w:t><w:noBreakHyphen/>", "gamma", "alpha-gamma"),
        ("<w:t>don</w:t>", "\u{2019}t", "don't"),
    ] {
        let source = docx(&format!(
            r#"<w:p><w:r>{before}</w:r><w:del w:id="4" w:author="Bob" w:date="{OLD}"><w:r><w:delText>{deleted}</w:delText></w:r></w:del></w:p>"#
        ));
        let e = apply_plan(&source, &plan(replace(find, "x"))).unwrap_err();
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{find:?}: {e}");
        assert!(
            e.message.starts_with(&format!(
                r#"the text is inside r4, {deleted:?} deleted by "Bob"."#
            )),
            "{find:?}: {e}"
        );
    }
}

#[test]
fn an_anchor_copied_with_its_change_marks_names_the_deletion() {
    let e = apply_plan(STEP2, &plan(replace("{--bankers--}", "brokers"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(
        e.message
            .starts_with(r#"the text is inside r4, "bankers" deleted by "Bob"."#),
        "{e}"
    );
}

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

#[test]
fn a_header_revision_routes_by_its_header_ids() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t xml:space="preserve">alpha </w:t></w:r><w:del w:id="6" w:author="Bob" w:date="{OLD}"><w:r><w:delText>gamma</w:delText></w:r></w:del></w:p></w:hdr>"#
    );
    let source = docx_with_sect(
        &para("Body"),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let request = plan(
        json!([{"kind":"replace","paragraph":"header1.p0","find":"gamma","replacement":"delta"}]),
    );
    let options = PrepareOptions {
        syntax: PlanSyntax::V2,
        ..PrepareOptions::default()
    };
    let e = apply_plan_json_with(&source, &request.to_json(), &options).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    let lines: Vec<&str> = e.message.lines().collect();
    let accept = decisions(lines[1]);
    assert_eq!(
        accept["accept"][0],
        json!({"short_id":"header1.r6","long_id":"header1.w:del.w:id.6"}),
        "{e}"
    );
    // A v2 plan takes the route as written.
    let mut q = v2_plan(&source);
    q["resolve_revisions"] = accept;
    q["operations"] = json!([]);
    let out = apply_v2(&source, &q).unwrap_or_else(|x| panic!("{x}: {e}"));
    let kept = part_string(&out.clean, "word/header1.xml").unwrap();
    assert!(!kept.contains("gamma"), "{kept}");
}
