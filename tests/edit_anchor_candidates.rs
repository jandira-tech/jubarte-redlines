// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! An `ANCHOR_NOT_FOUND` names the paragraphs the anchor does occur in
//! (#1889): #1013's released message stays word for word as its prefix,
//! the candidates follow it, and the JSON `op` event carries them. The
//! edit is still refused — nothing is ever retargeted (#957).

#![cfg(feature = "cli")]

mod common;

use std::path::Path;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, PrepareOptions, apply_plan, prepare};

/// p53 holds the liability cap clause; p54 is the paragraph a misread id
/// names (the view prints the id above the text, so a grep that takes the
/// id below the words names the paragraph after the clause).
const CAP: &str = "10.2. Liability Cap. To the maximum extent permitted by applicable law, a provider's total liability is capped at the fees paid.";
/// `CAP`'s first 42 characters, cut with an ellipsis.
const CAP_EXCERPT: &str = "10.2. Liability Cap. To the maximum extent…";

fn contract() -> Vec<u8> {
    let mut body = String::new();
    for i in 0..55 {
        body.push_str(&para(match i {
            53 => CAP,
            54 => "The Provider gives no warranties.",
            _ => "Filler paragraph to position the clauses.",
        }));
    }
    docx(&body)
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","operations":{operations}}}"#
    ))
    .unwrap()
}

fn cli(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The plan form: the released refusal (#1013) stays word for word, and
/// the paragraph that holds the anchor follows it.
#[test]
fn the_refusal_names_the_paragraph_the_anchor_occurs_in() {
    let e = apply_plan(
        &contract(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p54","find":"Liability Cap","replacement":"Cap"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        format!(
            r#""Liability Cap" does not occur in the paragraph; it occurs in p53 ("{CAP_EXCERPT}")"#
        )
    );
    assert_eq!(
        e.outcomes[0].candidates,
        vec![jubarte::edit::AnchorCandidate {
            at: "p53".into(),
            ctx: CAP_EXCERPT.into()
        }]
    );
}

/// A candidate is exactly a paragraph the same anchor would match: the
/// Markdown-marked anchor that `locate` normalizes (`**Liability Cap**` →
/// `Liability Cap`) finds the same paragraph as a suggestion.
#[test]
fn a_marked_up_anchor_names_the_paragraph_it_would_match() {
    let e = apply_plan(
        &contract(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p54","find":"**Liability Cap**","replacement":"Cap"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        format!(
            r#""**Liability Cap**" does not occur in the paragraph; it occurs in p53 ("{CAP_EXCERPT}")"#
        ),
        "{}",
        e.message
    );
}

/// The legacy flags form (`edit FILE -p WHERE --anchor … --content …`)
/// says the same thing.
#[test]
fn the_flags_refusal_names_the_paragraph_too() {
    use jubarte::edit::flags::{FlagOp, Verb, plan_from_flags};
    let source = contract();
    let built = plan_from_flags(
        Verb::Edit,
        &[FlagOp {
            at: "p54".into(),
            anchor: Some("Liability Cap".into()),
            content: Some("Cap".into()),
            ..FlagOp::default()
        }],
        "Ann Counsel",
        None,
        None,
        &source,
    )
    .unwrap();
    let e = apply_plan(&source, &built.plan).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        format!(
            r#""Liability Cap" does not occur in the paragraph; it occurs in p53 ("{CAP_EXCERPT}")"#
        )
    );
}

/// Four other paragraphs hold it: three are named, then the count of the
/// rest.
#[test]
fn three_candidates_are_named_then_the_count_of_the_rest() {
    let texts = [
        "First Liability Cap here.",
        "Nothing to see in this paragraph.",
        "Second Liability Cap spot.",
        "Filler.",
        "Third Liability Cap line.",
        "Filler two.",
        "Fourth Liability Cap tail.",
        "End.",
    ];
    let body = texts.iter().map(|t| para(t)).collect::<String>();
    let e = apply_plan(
        &docx(&body),
        &plan(
            r#"[{"kind":"replace","paragraph":"p1","find":"Liability Cap","replacement":"Cap"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        r#""Liability Cap" does not occur in the paragraph; it occurs in p0 ("First Liability Cap here."), p2 ("Second Liability Cap spot."), p4 ("Third Liability Cap line."), and 1 more"#
    );
    let candidates = &e.outcomes[0].candidates;
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].at, "p0");
    assert_eq!(candidates[2].at, "p4");
}

/// An anchor that occurs nowhere changes nothing: the released message,
/// byte for byte, and no candidates anywhere.
#[test]
fn an_anchor_that_occurs_nowhere_leaves_the_message_byte_for_byte() {
    let e = apply_plan(
        &contract(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p54","find":"No Such Anchor","replacement":"Cap"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        r#""No Such Anchor" does not occur in the paragraph"#
    );
    assert!(e.outcomes[0].candidates.is_empty());
}

/// A paragraph index past the end names the candidates when an anchor was
/// given; without one the released message stands alone.
#[test]
fn a_paragraph_index_past_the_end_names_candidates_when_an_anchor_was_given() {
    let e = apply_plan(
        &contract(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p182","find":"Liability Cap","replacement":"Cap"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        format!(
            r#"paragraph index 182 does not exist in body (55 paragraphs); it occurs in p53 ("{CAP_EXCERPT}")"#
        )
    );
    let e = apply_plan(
        &contract(),
        &plan(r#"[{"kind":"rewrite","paragraph":"p182","text":"x"}]"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(
        e.message,
        "paragraph index 182 does not exist in body (55 paragraphs)"
    );
}

/// The JSON `op` event of a refused plan carries the candidates, and has
/// no `candidates` key when there are none.
#[test]
fn the_op_event_carries_the_candidates() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("contract.docx"), contract()).unwrap();
    let (code, out, err) = cli(
        dir.path(),
        &[
            "edit",
            "contract.docx",
            "-p",
            "p54",
            "--anchor",
            "Liability Cap",
            "--content",
            "Cap",
        ],
    );
    assert_eq!(code, 3, "{out}{err}");
    let op: serde_json::Value =
        serde_json::from_str(out.lines().next().unwrap_or_default()).unwrap();
    assert_eq!(op["ev"], "op", "{out}");
    assert_eq!(op["status"], "failed");
    assert_eq!(op["code"], "ANCHOR_NOT_FOUND");
    assert_eq!(op["at"], "p54");
    assert_eq!(
        op["candidates"],
        serde_json::json!([{"at": "p53", "ctx": CAP_EXCERPT}]),
        "{out}"
    );
    let (code, out, err) = cli(
        dir.path(),
        &[
            "edit",
            "contract.docx",
            "-p",
            "p54",
            "--anchor",
            "No Such Anchor",
            "--content",
            "Cap",
        ],
    );
    assert_eq!(code, 3, "{out}{err}");
    let op: serde_json::Value =
        serde_json::from_str(out.lines().next().unwrap_or_default()).unwrap();
    assert_eq!(op["code"], "ANCHOR_NOT_FOUND");
    assert!(op.get("candidates").is_none(), "{op}");
}

/// `--skip-failed` keeps the skipped operation's candidates in the report
/// the success path prints.
#[test]
fn a_skipped_operation_keeps_its_candidates_in_the_report() {
    let out = prepare(
        &contract(),
        &plan(
            r#"[
            {"kind":"replace","paragraph":"p54","find":"Liability Cap","replacement":"Cap"},
            {"kind":"replace","paragraph":"p54","find":"warranties","replacement":"guarantees"}]"#,
        ),
        &PrepareOptions {
            skip_failed: true,
            ..PrepareOptions::default()
        },
    )
    .unwrap();
    assert_eq!(out.result.report.skipped, 1);
    let jsonl = out.result.report.to_jsonl();
    let line = jsonl
        .lines()
        .find(|l| l.contains(r#""ev":"op""#) && l.contains("ANCHOR_NOT_FOUND"))
        .unwrap_or_default();
    assert!(
        line.contains(&format!(
            r#""candidates":[{{"at":"p53","ctx":"{CAP_EXCERPT}"}}]"#
        )),
        "{jsonl}"
    );
}

#[test]
fn candidate_excerpt_shows_a_late_anchor_and_preserves_graphemes() {
    for (text, anchor, expected) in [
        (
            format!("{}Liability Cap{}", "a".repeat(80), "z".repeat(80)),
            "Liability Cap",
            format!("…{}Liability Cap{}…", "a".repeat(14), "z".repeat(15)),
        ),
        (
            format!("{}Liability Cap{}", "e\u{301}".repeat(80), "👩‍💻".repeat(80)),
            "Liability Cap",
            format!(
                "…{}Liability Cap{}…",
                "e\u{301}".repeat(14),
                "👩‍💻".repeat(15)
            ),
        ),
        (
            format!("{}curly’s{}", "a".repeat(80), "z".repeat(80)),
            "curly's",
            format!("…{}curly’s{}…", "a".repeat(17), "z".repeat(18)),
        ),
    ] {
        let source = docx(&format!("{}{}", para("Other."), para(&text)));
        let operations = serde_json::json!([{
            "kind": "replace", "paragraph": "p0", "find": anchor, "replacement": "Cap"
        }]);
        let error = apply_plan(&source, &plan(&operations.to_string())).unwrap_err();
        assert_eq!(error.code, "ANCHOR_NOT_FOUND");
        assert_eq!(error.outcomes[0].candidates[0].ctx, expected);
    }
}

#[test]
fn missing_comment_text_never_suggests_document_candidates() {
    for selector in [
        serde_json::json!("c999.text"),
        serde_json::json!({"id":"c999.text"}),
    ] {
        let operations = serde_json::json!([{
            "kind": "replace", "paragraph": selector, "find": "Liability Cap", "replacement": "Cap"
        }]);
        let error = apply_plan(&contract(), &plan(&operations.to_string())).unwrap_err();
        assert_eq!(error.code, "ANCHOR_NOT_FOUND");
        assert_eq!(
            error.message,
            "no comment c999 covers text in this document"
        );
        assert!(error.outcomes[0].candidates.is_empty());
    }
}
