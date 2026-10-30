// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte comments FILE [--json] [--author NAME] [--latest]`: the CLI
//! listing of comment threads, with the Python twin's names and shapes.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A document with one thread (Ann, then Bob's reply) and one lone comment.
fn threaded(dir: &Path) -> std::path::PathBuf {
    let source = docx(&(para("The cap is 10.") + &para("Other.")));
    let first = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"},
            {"kind":"comment","paragraph":"p1","text":"Fine"}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let second = apply_plan(
        &first.clean,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"reply_comment","comment_id":0,"text":"Agreed"}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let path = dir.join("threaded.docx");
    std::fs::write(&path, second.clean).unwrap();
    path
}

#[test]
fn comments_lists_json_lines_and_filters() {
    let dir = tempfile::tempdir().unwrap();
    let file = threaded(dir.path());
    let file = file.to_str().unwrap();

    let (code, stdout, stderr) = run(&["comments", file, "--json"]);
    assert_eq!(code, 0, "{stderr}");
    let rows: Vec<serde_json::Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    for row in &rows {
        for field in ["text", "ref", "author"] {
            assert!(row.get(field).is_some(), "missing {field}: {row}");
        }
    }
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["text"], "Too low");
    assert_eq!(rows[0]["anchor_text"], "cap");
    assert_eq!(rows[0]["paragraph"], "p0");
    assert_eq!(rows[0]["ref"], "c0");
    assert_eq!(rows[1]["parent"], 0);
    assert_eq!(rows[1]["done"], false);

    let (code, stdout, _) = run(&["comments", file, "--json", "--author", "Bob"]);
    assert_eq!(code, 0);
    // One comment on stdout.
    assert_eq!(stdout.lines().count(), 1);
    assert!(
        stdout.lines().next().unwrap().contains("Agreed"),
        "{stdout}"
    );

    let (code, stdout, _) = run(&["comments", file, "--json", "--latest"]);
    assert_eq!(code, 0);
    let texts: Vec<String> = stdout
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["text"].to_string())
        .collect();
    assert_eq!(texts, ["\"Agreed\"", "\"Fine\""]);

    let (code, stdout, _) = run(&["comments", file]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines[0].starts_with("c0\tp0\t") && lines[0].contains("\"Too low\""),
        "{stdout}"
    );
    assert!(!stdout.contains("body:p:"), "{stdout}");
    assert!(lines[1].contains("\treply to c0"), "{stdout}");
    assert!(stdout.contains("3 comment(s)"), "{stdout}");
}

#[test]
fn comments_refuses_a_file_that_is_not_a_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.docx");
    std::fs::write(&path, b"not a zip").unwrap();
    let (code, _, stderr) = run(&["comments", path.to_str().unwrap()]);
    assert_ne!(code, 0);
    assert!(!stderr.is_empty());
}

#[path = "common/dead_comments.rs"]
mod dead_comments;

fn resolve_probe(case: char, what: &str) -> (Vec<u8>, String) {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let output = dir.path().join("resolved.docx");
    std::fs::write(&input, dead_comments::probe(case)).unwrap();
    let (code, stdout, stderr) = run(&[
        what,
        input.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
        "--quiet",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "");
    (std::fs::read(&output).unwrap(), stderr)
}

#[test]
fn accept_warns_for_a_dead_range_with_a_surviving_reference() {
    let (output, stderr) = resolve_probe('a', "accept");
    assert_eq!(
        stderr,
        "warning: accept removed comment c0 (Alice: \"Existing range\"): anchor removed by accepting the changes that held all its text\n"
    );
    dead_comments::assert_state(&output, false, &[]);
}

#[test]
fn accept_warns_when_the_reference_is_deleted_too() {
    let (output, stderr) = resolve_probe('b', "accept");
    // The deletion r7 held the mark itself.
    assert_eq!(
        stderr,
        "warning: accept removed comment c0 (Alice: \"Existing range\"): anchor removed by accepting r7\n"
    );
    dead_comments::assert_state(&output, false, &[]);
}

#[test]
fn reject_warns_for_a_dead_inserted_range() {
    let (output, stderr) = resolve_probe('d', "reject");
    assert_eq!(
        stderr,
        "warning: reject removed comment c0 (Alice: \"Existing range\"): anchor removed by rejecting the changes that held all its text\n"
    );
    dead_comments::assert_state(&output, false, &[]);
}

#[test]
fn accept_warns_once_per_thread_member_using_original_ids() {
    let (output, stderr) = resolve_probe('e', "accept");
    assert_eq!(
        stderr,
        "warning: accept removed comment c0 (Alice: \"Existing range\"): anchor removed by accepting the changes that held all its text\nwarning: accept removed comment c1 (Bob: \"Reply to range\"): anchor removed by accepting the changes that held all its text\n"
    );
    // The comment that stays keeps its id (#1264).
    dead_comments::assert_state(&output, true, &[(2, "Kept range", None)]);
}

#[test]
fn resolution_does_not_warn_about_surviving_comments() {
    for (case, what) in [('c', "accept"), ('b', "reject")] {
        let (output, stderr) = resolve_probe(case, what);
        assert_eq!(stderr, "");
        dead_comments::assert_state(&output, false, &[(0, "Existing range", None)]);
    }
}
