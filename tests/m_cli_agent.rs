// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The agent-facing CLI surface: `inspect`, `text`, `edit`, `convert --png`,
//! `capabilities`. Every command here has a Python twin with the same names
//! and JSON shapes (`python -m jubarte_redlines`).

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};
use jubarte::inspect::source_sha256;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn write_fixture(dir: &Path) -> std::path::PathBuf {
    let body = para("Heading")
        + &para("The individual signs in his or her individual capacity.")
        + &para("Sections 1(g), 2(e), 3 survive.");
    let path = dir.join("letter.docx");
    std::fs::write(&path, docx(&body)).unwrap();
    path
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn inspect_prints_json_snapshot_or_a_human_table() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let (code, stdout, _) = run(&["inspect", file.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["summary"]["paragraphs"], 3);
    assert_eq!(v["paragraphs"][1]["id"], "body:p:1");
    let (code, stdout, _) = run(&["inspect", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains("body:p:2"), "{stdout}");
    assert!(stdout.contains("paragraphs: 3"), "{stdout}");
}

#[test]
fn text_prints_markdown_with_paragraph_ids() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let (code, stdout, _) = run(&["text", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stdout.starts_with("[body:p:0] Heading\n\n[body:p:1] The individual"),
        "{stdout}"
    );
}

#[test]
fn edit_writes_clean_redline_and_report_and_refuses_an_existing_output_dir() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let source = std::fs::read(&file).unwrap();
    let plan = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-09-25T12:00:00Z","operations":[
            {{"id":"pronoun","kind":"replace","paragraph":{{"index":1}},"find":"his or her","replacement":"an"}},
            {{"id":"survival","kind":"insert","paragraph":{{"starts_with":"Sections 1(g), "}},"after":"1(g), ","text":"2(c), ","comment":"post-disclosure duty"}}]}}"#,
        source_sha256(&source)
    );
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, plan).unwrap();
    let out_dir = dir.path().join("review");
    let (code, stdout, stderr) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--pdf",
        "--png",
        "--dpi",
        "24",
    ]);
    assert_eq!(code, 0, "stdout={stdout}\nstderr={stderr}");
    for name in [
        "clean.docx",
        "redline.docx",
        "report.jsonl",
        "redline.pdf",
        "clean.pdf",
        "redline-page-01.png",
        "clean-page-01.png",
    ] {
        assert!(out_dir.join(name).is_file(), "{name} written");
    }
    let report = std::fs::read_to_string(out_dir.join("report.jsonl")).unwrap();
    let lines: Vec<serde_json::Value> = report
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["ev"], "load");
    assert_eq!(lines[1]["ev"], "op");
    assert_eq!(lines[1]["id"], "pronoun");
    assert_eq!(lines[2]["comment_id"], 0);
    let render = lines
        .iter()
        .find(|l| l["ev"] == "render")
        .expect("render event");
    assert_eq!(render["pages"]["redline"], 1);
    assert_eq!(render["pages"]["clean"], 1);
    let save = lines
        .iter()
        .find(|l| l["ev"] == "save")
        .expect("save event");
    assert!(
        save["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["f"] == "redline.docx")
    );
    assert_eq!(lines.last().unwrap()["ev"], "summary");
    assert_eq!(lines.last().unwrap()["status"], "ok");
    // The summary line also goes to stdout.
    assert!(stdout.contains("\"ev\":\"summary\""), "{stdout}");
    // Existing destination is refused without --force.
    let (code, _, stderr) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 1);
    assert!(stderr.contains("already exists"), "{stderr}");
    let (code, _, _) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--force",
    ]);
    assert_eq!(code, 0);
}

#[test]
fn edit_failure_writes_nothing_and_reports_every_operation() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let plan = r#"{"schema_version":1,"author":"Claude","operations":[
        {"id":"ok","kind":"replace","paragraph":{"index":1},"find":"his or her","replacement":"an"},
        {"id":"bad","kind":"replace","paragraph":{"index":2},"find":"nowhere","replacement":"x"}]}"#;
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, plan).unwrap();
    let out_dir = dir.path().join("review");
    let (code, stdout, stderr) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 3, "stdout={stdout}\nstderr={stderr}");
    assert!(!out_dir.exists(), "no output directory on failure");
    assert!(stderr.contains("ANCHOR_NOT_FOUND"), "{stderr}");
    let lines: Vec<serde_json::Value> = stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let ops: Vec<(&str, &str)> = lines
        .iter()
        .filter(|l| l["ev"] == "op")
        .map(|l| (l["id"].as_str().unwrap(), l["status"].as_str().unwrap()))
        .collect();
    assert_eq!(ops, [("ok", "ok"), ("bad", "failed")]);
    assert_eq!(lines.last().unwrap()["status"], "failed");
}

#[test]
fn edit_dry_run_prints_the_report_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let plan = r#"{"schema_version":1,"author":"Claude","operations":[{"kind":"delete","paragraph":{"index":0},"find":"Heading"}]}"#;
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, plan).unwrap();
    let out_dir = dir.path().join("review");
    let (code, stdout, _) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "--out-dir",
        out_dir.to_str().unwrap(),
        "--dry-run",
    ]);
    assert_eq!(code, 0);
    assert!(!out_dir.exists());
    assert!(stdout.contains(r#""ctx":"{-Heading}""#), "{stdout}");
}

#[test]
fn convert_can_write_png_pages_and_a_page_report() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let report = dir.path().join("pages.json");
    let (code, stdout, stderr) = run(&[
        "convert",
        file.to_str().unwrap(),
        "--png",
        "--dpi",
        "24",
        "--report",
        report.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "stdout={stdout}\nstderr={stderr}");
    assert!(dir.path().join("letter-page-01.png").is_file());
    assert!(
        !dir.path().join("letter.pdf").exists(),
        "--png alone writes no PDF"
    );
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(v["page_count"], 1);
    assert!(v["pages"][0]["text"].as_str().unwrap().contains("Heading"));
    // PDF and PNG together, from one layout.
    let (code, _, _) = run(&[
        "convert",
        file.to_str().unwrap(),
        "--pdf",
        "--png",
        "--dpi",
        "24",
        "--force",
    ]);
    assert_eq!(code, 0);
    assert!(dir.path().join("letter.pdf").is_file());
}

#[test]
fn capabilities_describe_the_built_binary() {
    let (code, stdout, _) = run(&["capabilities", "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["engine_version"], env!("CARGO_PKG_VERSION"));
    for op in [
        "compare",
        "accept_revisions",
        "reject_revisions",
        "revision_records",
        "pdf",
        "png",
        "inspect_body",
        "markdown",
        "edit",
    ] {
        assert_eq!(v["operations"][op], true, "{op}");
    }
    assert_eq!(v["edit_plan_versions"], serde_json::json!([1]));
    let kinds = v["edit_operations"].as_array().unwrap();
    assert!(kinds.iter().any(|k| k == "insert_paragraph"));
    assert_eq!(v["limits"]["stories"], serde_json::json!(["body"]));
}
