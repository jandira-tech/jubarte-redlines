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
            {"kind":"comment","paragraph":"body:p:0","find":"cap","text":"Too low"},
            {"kind":"comment","paragraph":"body:p:1","text":"Fine"}]}"#,
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
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["text"], "Too low");
    assert_eq!(rows[0]["anchor_text"], "cap");
    assert_eq!(rows[0]["paragraph"], "body:p:0");
    assert_eq!(rows[1]["parent"], 0);
    assert_eq!(rows[1]["done"], false);

    let (code, stdout, _) = run(&["comments", file, "--json", "--author", "Bob"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("Agreed"), "{stdout}");

    let (code, stdout, _) = run(&["comments", file, "--json", "--latest"]);
    assert_eq!(code, 0);
    let texts: Vec<String> = stdout
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["text"].to_string())
        .collect();
    assert_eq!(texts, ["\"Agreed\"", "\"Fine\""]);

    let (code, stdout, _) = run(&["comments", file]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Too low") && stdout.contains("body:p:0"),
        "{stdout}"
    );
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
