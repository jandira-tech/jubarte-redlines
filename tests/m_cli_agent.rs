// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The agent-facing CLI surface: `inspect`, `text`, `edit`, `convert --png`,
//! `capabilities`. Every command here has a Python twin with the same names
//! and JSON shapes (`python -m jubarte_redlines`).
//!
//! Every test drives the CLI binary and some name `jubarte::cli` and
//! `same-file`, which only the `cli` feature builds (#1786).
#![cfg(feature = "cli")]

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
    run_in(Path::new("."), args)
}

fn run_in(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
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

#[test]
fn retired_names_work_once_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let file = file.to_str().unwrap();
    let (code, stdout, stderr) = run(&["text", file]);
    assert_eq!(code, 0);
    assert_eq!(stdout, run(&["read", file]).1);
    assert_eq!(stderr, "note: text is now read; text is removed in 0.14\n");
    let (code, stdout, stderr) = run(&["inspect", file, "--json"]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("{\"schema_version\":3"), "{stdout}");
    assert_eq!(
        stderr,
        "note: inspect is now read; inspect is removed in 0.14\n"
    );
    let bullets = dir.path().join("bullets.docx");
    std::fs::write(&bullets, common::docx::docx(&common::docx::para("• item"))).unwrap();
    let (code, stdout, stderr) = run(&["audit", bullets.to_str().unwrap()]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert!(
        stdout.contains("\n  LITERAL_BULLET\twarning\tp0\t"),
        "{stdout}"
    );
    assert_eq!(
        stderr,
        "note: audit is now validate --rules a11y,style,structure; \
         audit is removed in 0.14\n"
    );
    let (_, help, _) = run(&["--help"]);
    assert!(
        !help.contains("\n  inspect ") && !help.contains("\n  debug "),
        "{help}"
    );
    assert!(
        !help.contains("\n  text ") && !help.contains("\n  audit "),
        "{help}"
    );
    let tasks: Vec<&str> = help
        .split("\n\n")
        .find(|block| block.starts_with("Tasks:"))
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .collect();
    assert!(
        tasks.last().unwrap().starts_with("  self-update "),
        "{tasks:?}"
    );
}

/// A document with changes, comments and a table.
const RECEIVED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/agent-view/received.docx"
);

fn json(args: &[&str]) -> serde_json::Value {
    let (code, stdout, stderr) = run(args);
    assert_eq!(code, 0, "{args:?}: {stderr}");
    serde_json::from_str(&stdout).unwrap()
}

/// `read --json` prints the snapshot: the paragraphs, and the changes and
/// comments as `changes --json` and `comments --json` list them.
#[test]
fn read_json_prints_the_snapshot_with_changes_and_comments() {
    let snapshot = json(&["read", RECEIVED, "--json"]);
    assert_eq!(snapshot["schema_version"], 3);
    assert_eq!(snapshot["paragraphs"][1]["id"], "p1");
    assert_eq!(snapshot["changes"][0]["ref"], "r0");
    assert_eq!(snapshot["comments"][0]["ref"], "c5");
    let (_, inspect, _) = run(&["inspect", RECEIVED, "--json"]);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&inspect).unwrap(),
        snapshot
    );
    // Each record `changes`/`comments` lists is one of the snapshot's.
    for (task, key) in [("changes", "changes"), ("comments", "comments")] {
        let (code, stdout, stderr) = run(&[task, RECEIVED, "--json"]);
        assert_eq!(code, 0);
        let listed: Vec<serde_json::Value> = stdout
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let validity: serde_json::Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(
            validity["input_validity"], snapshot["summary"]["input_validity"],
            "{task}"
        );
        assert_eq!(
            listed.len(),
            snapshot[key].as_array().unwrap().len(),
            "{task}"
        );
        for record in &listed {
            assert!(
                snapshot[key].as_array().unwrap().contains(record),
                "{task}: {record}"
            );
        }
    }
}

/// The library's snapshot carries what `list_logical_changes` and
/// `list_comments` list, and `retain` keeps a paragraph's own records.
#[test]
fn the_snapshot_lists_changes_and_comments_and_retains_by_paragraph() {
    let bytes = std::fs::read(RECEIVED).unwrap();
    let mut snapshot = jubarte::inspect::snapshot(&bytes).unwrap();
    assert_eq!(
        snapshot.changes,
        jubarte::changes::list_logical_changes(&bytes).unwrap()
    );
    assert_eq!(
        snapshot.comments,
        jubarte::comments::list_comments(&bytes).unwrap()
    );
    let summary = snapshot.summary.clone();
    snapshot.retain(&[5]);
    assert_eq!(snapshot.summary, summary);
    let ids: Vec<&str> = snapshot.paragraphs.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids, ["p5"]);
    let refs: Vec<&str> = snapshot.comments.iter().map(|c| c.r#ref.as_str()).collect();
    assert_eq!(refs, ["c5", "c6"]);
    assert!(
        snapshot
            .changes
            .iter()
            .all(|c| c.paragraph.as_deref() == Some("p5"))
    );
}

/// The selection flags keep paragraphs, and only their changes and
/// comments; the summary still counts the whole document.
#[test]
fn read_json_keeps_what_the_selection_flags_select() {
    let whole = json(&["read", RECEIVED, "--json"]);
    let one = json(&["read", RECEIVED, "--json", "-p", "p3"]);
    let ids: Vec<&str> = one["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["p3"]);
    assert_eq!(one["summary"], whole["summary"]);
    let changes = one["changes"].as_array().unwrap();
    assert!(!changes.is_empty());
    assert!(changes.iter().all(|c| c["paragraph"] == "p3"), "{one}");
    assert_eq!(one["comments"], serde_json::json!([]));
    assert_eq!(one["tables"], whole["tables"]);
    let changed = json(&["read", RECEIVED, "--json", "--changed"]);
    let kept = changed["paragraphs"].as_array().unwrap();
    assert!(kept.len() < whole["paragraphs"].as_array().unwrap().len());
    assert!(kept.iter().all(|p| p["id"] != "p0"), "{changed}");
    assert_eq!(changed["changes"], whole["changes"]);
}

/// A Strict package keeps its shapes' text in `wne:txbxContent`. The view,
/// the snapshot and `edit` all leave those paragraphs out, so `pN` names the
/// same paragraph everywhere (the view's `p131` used to be `p137`).
#[test]
fn strict_text_boxes_leave_the_paragraph_ids_alone() {
    const STRICT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/strict/Strict01.docx"
    );
    let whole = json(&["read", STRICT, "--json"]);
    let (code, view, stderr) = run(&["read", STRICT, "--no-page-markers"]);
    assert_eq!(code, 0, "{stderr}");
    let paragraphs = whole["paragraphs"].as_array().unwrap().len();
    assert!(
        view.contains(&format!("body: p0-p{}, ", paragraphs - 1)),
        "{paragraphs} paragraphs: {view}"
    );
    let one = json(&["read", STRICT, "--json", "-p", "p131"]);
    let kept = &one["paragraphs"][0];
    assert_eq!(kept["id"], "p131");
    assert!(
        kept["text"].as_str().unwrap().starts_with("Video provides"),
        "{kept}"
    );
    assert!(
        view.contains("<!-- p131 -->\n**_<u><mark>Video provides"),
        "{view}"
    );
    assert!(
        one["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == "r7"),
        "{one}"
    );
    let changed = json(&["read", STRICT, "--json", "--changed"]);
    assert_eq!(changed["changes"], whole["changes"]);
    let dir = tempfile::tempdir().unwrap();
    let (code, _, stderr) = run_in(
        dir.path(),
        &[
            "edit",
            STRICT,
            "-q",
            "-p",
            "p131",
            "--anchor",
            "Video provides",
            "--content",
            "Film provides",
            "-a",
            "Jubarte",
            "--out-dir",
            "out",
        ],
    );
    assert_eq!(code, 0, "edit -p p131 finds the view's p131: {stderr}");
}

/// `read --tables` prints the grids; --json and --tables read one document,
/// and the view-only flags do not go with --json.
#[test]
fn read_tables_and_the_json_refusals() {
    let (code, stdout, _) = run(&["read", RECEIVED, "--tables"]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("table 0: "), "{stdout}");
    // An -o beside them changes nothing: the read-only flag is what was meant.
    for args in [
        &[RECEIVED, RECEIVED, "--json"][..],
        &[RECEIVED, RECEIVED, "--json", "-o", "x.json"],
    ] {
        let (code, _, stderr) = run(args);
        assert_eq!(code, 2, "{stderr}");
        assert!(
            stderr.contains(
                "--json reads one document; `-o FILE.json` writes the redline's snapshot"
            ),
            "{args:?}: {stderr}"
        );
    }
    let (code, _, stderr) = run(&[RECEIVED, RECEIVED, "--tables", "-o", "x.json"]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("--tables reads one document"), "{stderr}");
    for args in [
        &["read", RECEIVED, "--tables", "--head", "2"][..],
        &["read", RECEIVED, "--tables", "-p", "p0"],
        &["read", RECEIVED, "--tables", "--json"],
        &[RECEIVED, RECEIVED, "--tables"],
    ] {
        assert_eq!(run(args).0, 2, "{args:?}");
    }
    // The view's own options change nothing in the snapshot (#1255).
    for args in [
        &["read", RECEIVED, "--json", "--dates"][..],
        &["read", RECEIVED, "--json", "--comments", "none"],
        &["read", RECEIVED, "--json", "--no-page-markers"],
    ] {
        assert_eq!(json(args), json(&["read", RECEIVED, "--json"]), "{args:?}");
    }
    // The one-file shorthand reads like `read`.
    assert_eq!(
        json(&[RECEIVED, "--json"]),
        json(&["read", RECEIVED, "--json"])
    );
}

#[test]
fn text_prints_markdown_with_paragraph_ids() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let (code, stdout, _) = run(&["text", file.to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("---\nsource: "), "{stdout}");
    assert!(stdout.contains("<!-- p0"), "{stdout}");
    assert!(stdout.contains("The individual"), "{stdout}");
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
    assert_eq!(code, 2);
    assert!(stderr.contains("OUTPUT_EXISTS"), "{stderr}");
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

/// Exit 3 does not always have an operation report: malformed plans fail
/// before execution, while a stale source is a plan-level refusal.
#[test]
fn edit_plan_refusal_docs_distinguish_parse_and_plan_level_errors() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let plan = dir.path().join("plan.json");
    let output = dir.path().join("output.docx");
    for invalid in [
        "{not JSON",
        r#"{"schema_version":1,"author":"Claude","operations":[{"kind":"unknown"}]}"#,
    ] {
        std::fs::write(&plan, invalid).unwrap();
        let (code, stdout, stderr) = run(&[
            "edit",
            file.to_str().unwrap(),
            "--plan",
            plan.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ]);
        assert_eq!(code, 3, "{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert!(stderr.contains("INVALID_PLAN"), "{stderr}");
        assert!(!output.exists());
    }
    let stale = serde_json::json!({
        "schema_version": 1, "author": "Claude", "source_sha256": "0".repeat(64),
        "operations": [{"kind":"delete", "paragraph":"p0", "find":"Heading"}]
    });
    std::fs::write(&plan, stale.to_string()).unwrap();
    let (code, stdout, stderr) = run(&[
        "edit",
        file.to_str().unwrap(),
        "--plan",
        plan.to_str().unwrap(),
        "-o",
        output.to_str().unwrap(),
    ]);
    assert_eq!(code, 3, "{stderr}");
    assert!(stderr.contains("STALE_SOURCE"), "{stderr}");
    assert!(stdout.contains("\"operation\":null"), "{stdout}");
    assert!(!output.exists());
    for text in [
        include_str!("../docs/rust.md"),
        include_str!("../skills/jubarte-documents/SKILL.md"),
    ] {
        assert!(
            text.contains("Parse/schema refusals may leave stdout empty"),
            "document the observed empty-stdout refusal"
        );
        assert!(
            text.contains("plan-level refusals may have no operation"),
            "do not promise an operation for every refusal"
        );
    }
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
    // One output names one format: the retired --pdf and --png together
    // name two.
    let (code, _, stderr) = run(&[
        "convert",
        file.to_str().unwrap(),
        "--pdf",
        "--png",
        "--dpi",
        "24",
        "--force",
    ]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("--pdf and --png name two formats"),
        "{stderr}"
    );
    assert!(!dir.path().join("letter.pdf").exists());
}

#[test]
fn convert_checks_every_png_path_before_writing_one() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    std::fs::write(dir.path().join("letter-page-01.png"), b"keep").unwrap();
    let report = dir.path().join("pages.json");
    let (code, _, stderr) = run(&[
        "convert",
        file.to_str().unwrap(),
        "-o",
        dir.path().join("letter.png").to_str().unwrap(),
        "--dpi",
        "24",
        "--report",
        report.to_str().unwrap(),
    ]);
    assert_eq!(code, 2);
    assert!(stderr.contains("letter-page-01.png"), "{stderr}");
    assert!(!report.exists(), "a refused PNG leaves no partial bundle");
    assert_eq!(
        std::fs::read(dir.path().join("letter-page-01.png")).unwrap(),
        b"keep"
    );
}

#[test]
fn edit_refuses_the_input_directory_when_the_input_has_no_directory() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let source = std::fs::read(&file).unwrap();
    let plan = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","operations":[
            {{"kind":"replace","paragraph":{{"index":1}},"find":"his or her","replacement":"an"}}]}}"#,
        source_sha256(&source)
    );
    std::fs::write(dir.path().join("plan.json"), plan).unwrap();
    let absolute = dir.path().to_str().unwrap();
    for out_dir in [".", "./", absolute] {
        let (code, _, stderr) = run_in(
            dir.path(),
            &[
                "edit",
                "letter.docx",
                "--plan",
                "plan.json",
                "--out-dir",
                out_dir,
                "--force",
            ],
        );
        assert_eq!(code, 1, "{out_dir}: {stderr}");
        assert!(
            stderr.contains("input's own directory"),
            "{out_dir}: {stderr}"
        );
        assert!(!dir.path().join("clean.docx").exists(), "{out_dir}");
    }
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
        "patch",
        "validate",
        "repair",
    ] {
        assert_eq!(v["operations"][op], true, "{op}");
    }
    assert_eq!(v["edit_plan_versions"], serde_json::json!([1, 2]));
    let kinds = v["edit_operations"].as_array().unwrap();
    assert!(kinds.iter().any(|k| k == "insert_paragraph"));
    assert_eq!(
        v["limits"]["stories"],
        serde_json::json!(["body", "header", "footer", "footnotes", "endnotes"])
    );
    assert_eq!(v["validate_rules"][0], "word");
    assert_eq!(v["formats"]["read"], serde_json::json!([".md", ".json"]));
    assert_eq!(
        v["formats"]["diff"],
        serde_json::json!([".md", ".markdown", ".txt", ".patch", ".diff"])
    );
    // Only the settled spellings are advertised (spec §2).
    assert_eq!(v["ids"]["comment"], serde_json::json!(["c27"]));
    assert_eq!(
        v["ids"]["change"],
        serde_json::json!(["r12", "r1+2", "header1.r1", "footnotes.r3"])
    );
    assert_eq!(v["ids"]["comment_text"], serde_json::json!(["c27.text"]));
    assert_eq!(v["ids"]["table"], serde_json::json!(["t0"]));
    assert_eq!(
        v["ids"]["control"],
        serde_json::json!(["cc0", "p12.cc0", "t0.row1.cell2.cc0"])
    );
    assert!(v["ids"]["note"].as_str().unwrap().contains("safe unquoted"));
}

/// Without `--json`, `capabilities` prints the text summary.
#[test]
fn capabilities_print_a_text_summary_by_default() {
    let (code, text, stderr) = run(&["capabilities"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines[0],
        format!("engine: jubarte {} (cli)", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(lines[1], "compare: yes");
    assert!(lines.contains(&"png: yes"), "{text}");
    assert!(lines.contains(&"edit plan versions: 1, 2"), "{text}");
    for prefix in [
        "edit operations: replace, insert, ",
        "stories: body, header, footer, footnotes, endnotes",
        "limits: plain text runs no, ",
        "validate rules: word (default), edits, a11y, style, structure",
        "audit codes: HEADING_SKIP, ",
        "formats: compare .docx .md .pdf .png .json; convert ",
        "ids: revisions r12, r1+2, header1.r1, footnotes.r3; ",
        "ids note: body ids print without a story",
    ] {
        assert!(
            lines.iter().any(|line| line.starts_with(prefix)),
            "{prefix}: {text}"
        );
    }
    assert!(text.contains("; read .md .json; diff .md .markdown .txt .patch .diff;"));
    assert!(
        text.contains("; comments c27; comment text c27.text; "),
        "{text}"
    );
    assert_eq!(lines.last(), Some(&"run with --json for the manifest"));

    // The text says what the manifest says: every operation with its value,
    // the limits and every task's formats.
    let (code, json, _) = run(&["capabilities", "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let yes = |value: &serde_json::Value| {
        if value.as_bool().unwrap() {
            "yes"
        } else {
            "no"
        }
    };
    let operations = v["operations"].as_object().unwrap();
    let mut expected: Vec<String> = operations
        .iter()
        .map(|(name, on)| format!("{name}: {}", yes(on)))
        .collect();
    let mut printed: Vec<String> = lines[1..=operations.len()]
        .iter()
        .map(|line| line.to_string())
        .collect();
    expected.sort();
    printed.sort();
    assert_eq!(printed, expected);
    assert_eq!(lines[operations.len() + 1], "edit plan versions: 1, 2");
    let (limits, input) = (&v["limits"], &v["limits"]["input"]);
    let limits_line = format!(
        "limits: plain text runs {}, refuses opaque ranges {}, reads legacy .doc {}, \
         {} bytes compressed, {} entries, {} bytes per part, {} bytes uncompressed, \
         XML depth {}, {} XML elements",
        yes(&limits["plain_text_runs"]),
        yes(&limits["refuses_opaque_ranges"]),
        yes(&limits["reads_legacy_doc"]),
        input["max_compressed_bytes"],
        input["max_entries"],
        input["max_part_bytes"],
        input["max_uncompressed_bytes"],
        input["max_xml_depth"],
        input["max_xml_elements"]
    );
    // The element budget is part of the advertised input limits (#2014).
    assert_eq!(input["max_xml_elements"], 2_000_000);
    assert!(
        lines.contains(&limits_line.as_str()),
        "{limits_line}\n{text}"
    );
    let formats_line = lines
        .iter()
        .find_map(|line| line.strip_prefix("formats: "))
        .unwrap();
    let mut printed: Vec<&str> = formats_line.split("; ").collect();
    let mut expected: Vec<String> = v["formats"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(task, extensions)| {
            let extensions: Vec<&str> = extensions
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e.as_str().unwrap())
                .collect();
            format!("{task} {}", extensions.join(" "))
        })
        .collect();
    printed.sort();
    expected.sort();
    assert_eq!(printed, expected);
}

#[test]
fn convert_refuses_report_aliases_before_writing_any_output() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let original = std::fs::read(&file).unwrap();
    let output = dir.path().join("result.pdf");
    for flag in ["--report", "--font-report"] {
        for target in [&file, &output] {
            let (code, _, stderr) = run(&[
                "convert",
                file.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
                flag,
                target.to_str().unwrap(),
                "--force",
            ]);
            assert_eq!(code, 1, "{flag}: {stderr}");
            assert!(stderr.contains("same file"), "{stderr}");
            assert_eq!(std::fs::read(&file).unwrap(), original);
            assert!(!output.exists());
        }
    }
}

#[test]
fn invalid_png_dpi_leaves_no_partial_pdf_or_report() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let output = dir.path().join("result.pdf");
    let report = dir.path().join("report.json");
    let (code, stdout, stderr) = run(&[
        "convert",
        file.to_str().unwrap(),
        "--pdf",
        "--png",
        "--dpi",
        "0",
        "-o",
        output.to_str().unwrap(),
        "--report",
        report.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stdout.is_empty());
    assert!(stderr.contains("invalid value '0'"), "{stderr}");
    assert!(stderr.contains("dpi"), "{stderr}");
    assert!(!output.exists());
    assert!(!report.exists());
    assert!(!dir.path().join("result-page-01.png").exists());
}

#[test]
fn edit_writes_the_patch_and_confirms_full_views_unless_quiet() {
    let dir = tempfile::tempdir().unwrap();
    let file = write_fixture(dir.path());
    let plan = r#"{"schema_version":1,"author":"Claude","date":"2026-09-25T12:00:00Z","operations":[
        {"id":"pronoun","kind":"replace","paragraph":{"index":1},"find":"his or her","replacement":"an"},
        {"id":"survival","kind":"insert","paragraph":{"starts_with":"Sections 1(g), "},"after":"1(g), ","text":"2(c), ","comment":"post-disclosure duty"}]}"#;
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, plan).unwrap();
    let edit = |out: &str, quiet: bool| {
        let out_dir = dir.path().join(out);
        let mut args = vec![
            "edit",
            file.to_str().unwrap(),
            "--plan",
            plan_path.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ];
        if quiet {
            args.push("-q");
        }
        let (code, stdout, stderr) = run(&args);
        assert_eq!(code, 0, "stdout={stdout}\nstderr={stderr}");
        let patch = std::fs::read_to_string(out_dir.join("patch.diff")).unwrap();
        (stdout, patch)
    };
    let (stdout, patch) = edit("review", false);
    assert!(
        patch.starts_with(
            "--- a/letter.docx\n+++ b/letter.docx\tClaude\t2026-09-25T12:00:00Z\n@@ [p1] @@\n"
        ),
        "{patch}"
    );
    assert!(patch.contains("[-his or her-]{+an+}"), "{patch}");
    // The comparer aligns the insertion as "c), 2(" after the old "2(";
    // the comment on it follows the change, never inside it.
    let unwrapped = patch.replace('\n', " ");
    assert!(
        patch.contains("@@ [p2] @@\n")
            && unwrapped.contains("{>>Claude (2026-09-25T12:00:00Z): post-disclosure duty<<}")
            && !unwrapped.contains("{+{>>"),
        "{patch}"
    );
    let accepted: String = unwrapped
        .replace(
            "{>>Claude (2026-09-25T12:00:00Z): post-disclosure duty<<}",
            "",
        )
        .replace(['{', '}', '+', '='], "");
    assert!(accepted.contains("1(g), 2(c), 2(e), 3"), "{accepted}");
    // The patch stays on disk; stdout compares the full published views.
    assert!(stdout.contains("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(stdout.contains("\n@@ "), "{stdout}");
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('+') && line.contains("{~~his or her~>an~~}")),
        "{stdout}"
    );
    assert!(
        stdout.lines().any(|line| line.starts_with('+')
            && regex::Regex::new(r"\{~~his or her~>an~~\}\{r[0-9]+\+[0-9]+ @C\}")
                .unwrap()
                .is_match(line))
            && stdout
                .lines()
                .any(|line| line.starts_with('+') && line.contains("c0 @C: post-disclosure duty")),
        "{stdout}"
    );
    assert!(stdout.contains("\"ev\":\"summary\""), "{stdout}");
    // Quiet: the same files, nothing printed.
    let (stdout, quiet_patch) = edit("quiet", true);
    assert_eq!(stdout, "");
    assert_eq!(quiet_patch, patch);
}

/// codex 4249037697: a comment whose range starts in p4 and spans p5 is
/// kept by `-p p5`, as the view marks p5 `in c0`.
#[test]
fn read_json_keeps_a_comment_spanning_a_kept_paragraph() {
    const CLAUSES: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/md-roundtrip/edges/edge-clauses.docx"
    );
    let (code, view, stderr) = run(&["read", CLAUSES, "-p", "p5", "--no-page-markers"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(view.contains("p5 in c0"), "{view}");
    let one = json(&["read", CLAUSES, "--json", "-p", "p5"]);
    let refs: Vec<&str> = one["comments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["ref"].as_str().unwrap())
        .collect();
    assert_eq!(refs, ["c0"], "{one}");
    let comment = &one["comments"][0];
    assert_eq!(comment["paragraph"], "p4");
    // c0 runs on through the table to p12.
    assert_eq!(comment["last_paragraph"], "p12");
}

/// codex 4242942130: `--changed` keeps a header's changes: the view notes
/// them on its headers: line, and the snapshot lists them.
#[test]
fn read_json_changed_keeps_a_headers_changes() {
    let header = format!(
        r#"<w:hdr xmlns:w="{}"><w:p><w:ins w:id="7" w:author="Ann Counsel"><w:r><w:t>Head</w:t></w:r></w:ins></w:p></w:hdr>"#,
        common::docx::W_NS
    );
    let bytes = common::docx::docx_with_sect(
        &format!(
            r#"{}<w:p><w:ins w:id="1" w:author="Bo Other"><w:r><w:t>Body</w:t></w:r></w:ins></w:p>"#,
            para("Plain")
        ),
        &[common::docx::Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("head.docx");
    std::fs::write(&path, bytes).unwrap();
    let path = path.to_str().unwrap();
    let ids = |value: &serde_json::Value| -> Vec<String> {
        value["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap().to_string())
            .collect()
    };
    let changed = json(&["read", path, "--json", "--changed"]);
    assert_eq!(ids(&changed), ["r1", "header1.r7"], "{changed}");
    let by_ann = json(&["read", path, "--json", "--changed", "--by", "AC"]);
    assert_eq!(ids(&by_ann), ["header1.r7"], "{by_ann}");
    // A paragraph pick names body paragraphs only.
    let one = json(&["read", path, "--json", "-p", "p1"]);
    assert_eq!(ids(&one), ["r1"], "{one}");
}

// These are process-level integration tests: the filesystem and entrypoint
// are the boundaries under test, using only a synthetic DOCX.
fn read_json_fixture(dir: &Path) {
    let body = para("Alice agrees with Bob.")
        + "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"2400\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>Carol Diaz</w:t></w:r></w:p>\
           </w:tc></w:tr></w:tbl>";
    std::fs::write(dir.join("source.docx"), docx(&body)).unwrap();
}

/// `--no-page-markers` with an output that has no page markers: taken,
/// and nothing changes, stdout, stderr or the file written (#1255).
fn no_page_markers_changes_nothing(shorthand: bool, output: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    read_json_fixture(dir.path());
    let prefix = if shorthand {
        vec!["source.docx"]
    } else {
        vec!["read", "source.docx"]
    };
    let args = [prefix.as_slice(), output].concat();
    let plain = run_in(dir.path(), &args);
    assert_eq!(plain.0, 0, "{args:?}: {}", plain.2);
    let file = output
        .contains(&"-o")
        .then(|| std::fs::read(dir.path().join("snap.json")).unwrap());
    let fast_args = [args.as_slice(), &["--no-page-markers", "--force"]].concat();
    let fast = run_in(dir.path(), &fast_args);
    assert_eq!(fast, plain, "{fast_args:?}");
    if let Some(file) = file {
        assert_eq!(
            std::fs::read(dir.path().join("snap.json")).unwrap(),
            file,
            "{fast_args:?}"
        );
    }
}

macro_rules! no_page_markers_case {
    ($name:ident, $shorthand:expr, [$($output:expr),*]) => {
        #[test]
        fn $name() {
            no_page_markers_changes_nothing($shorthand, &[$($output),*]);
        }
    };
}

no_page_markers_case!(no_page_markers_json, false, ["--json"]);
no_page_markers_case!(no_page_markers_tables, false, ["--tables"]);
no_page_markers_case!(no_page_markers_for_edit, false, ["--for-edit"]);
no_page_markers_case!(no_page_markers_json_shorthand, true, ["--json"]);
no_page_markers_case!(no_page_markers_tables_shorthand, true, ["--tables"]);
no_page_markers_case!(no_page_markers_for_edit_shorthand, true, ["--for-edit"]);
no_page_markers_case!(no_page_markers_json_file, false, ["-o", "snap.json", "-q"]);
no_page_markers_case!(
    no_page_markers_json_file_shorthand,
    true,
    ["-o", "snap.json", "-q"]
);
no_page_markers_case!(
    no_page_markers_explicit_json_file,
    false,
    ["--json", "-o", "snap.json", "-q"]
);
no_page_markers_case!(
    no_page_markers_explicit_json_file_shorthand,
    true,
    ["--json", "-o", "snap.json", "-q"]
);
no_page_markers_case!(
    no_page_markers_for_edit_file,
    false,
    ["--for-edit", "-o", "snap.json", "-q"]
);
no_page_markers_case!(
    no_page_markers_for_edit_file_shorthand,
    true,
    ["--for-edit", "-o", "snap.json", "-q"]
);

fn read_view_conflict_teaches(flags: &[&str], output: &[&str], output_name: &str) {
    let dir = tempfile::tempdir().unwrap();
    read_json_fixture(dir.path());
    for prefix in [
        &["read", "source.docx"][..],
        &["source.docx"],
        &["text", "source.docx"],
    ] {
        let args = [prefix, flags, output].concat();
        let (code, stdout, stderr) = run_in(dir.path(), &args);
        assert_eq!(code, 2, "{args:?}: {stderr}");
        assert!(stdout.is_empty(), "{args:?}: {stdout}");
        assert!(stderr.contains("error: "), "{args:?}: {stderr}");
        for expected in [
            flags[0],
            output_name,
            "shapes the Markdown view",
            "has no view",
            "drop",
        ] {
            assert!(
                stderr.contains(expected),
                "missing {expected:?}: {args:?}: {stderr}"
            );
        }
        let usage = stderr
            .lines()
            .find(|line| line.starts_with("Usage:"))
            .unwrap();
        assert_eq!(usage, "Usage: jubarte read [OPTIONS] <FILE>", "{args:?}");
        assert!(!usage.contains(flags[0]), "{args:?}: {usage}");
        assert!(!dir.path().join("snap.json").exists());
    }
}

macro_rules! read_view_conflict_case {
    ($name:ident, [$($flag:expr),*], [$($output:expr),*], $label:expr) => {
        #[test]
        fn $name() {
            read_view_conflict_teaches(&[$($flag),*], &[$($output),*], $label);
        }
    };
}

read_view_conflict_case!(
    read_view_conflict_track_changes_for_edit,
    ["--track-changes", "all"],
    ["--for-edit"],
    "--for-edit"
);

/// A view option beside an output without a view: taken, under every
/// spelling of `read`, and `--comments`/`--dates` change nothing (#1255).
fn read_view_option_is_taken(flags: &[&str], output: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    read_json_fixture(dir.path());
    for prefix in [
        &["read", "source.docx"][..],
        &["source.docx"],
        &["text", "source.docx"],
    ] {
        let plain = [prefix, output, &["--force"]].concat();
        let plain = (
            run_in(dir.path(), &plain),
            std::fs::read(dir.path().join("snap.json")).ok(),
        );
        let args = [prefix, flags, output, &["--force"]].concat();
        let (code, stdout, stderr) = run_in(dir.path(), &args);
        assert_eq!(code, 0, "{args:?}: {stderr}");
        assert!(
            !stderr.contains("error") && !stderr.contains("warning"),
            "{args:?}: {stderr}"
        );
        if flags[0] != "--track-changes" {
            assert_eq!((code, stdout, stderr), plain.0, "{args:?}");
            assert_eq!(
                std::fs::read(dir.path().join("snap.json")).ok(),
                plain.1,
                "{args:?}"
            );
        }
    }
}

macro_rules! read_view_option_case {
    ($name:ident, [$($flag:expr),*], [$($output:expr),*]) => {
        #[test]
        fn $name() {
            read_view_option_is_taken(&[$($flag),*], &[$($output),*]);
        }
    };
}

read_view_option_case!(
    read_view_option_comments_json,
    ["--comments", "inline"],
    ["--json"]
);
read_view_option_case!(
    read_view_option_comments_tables,
    ["--comments", "inline"],
    ["--tables"]
);
read_view_option_case!(
    read_view_option_comments_for_edit,
    ["--comments", "inline"],
    ["--for-edit"]
);
read_view_option_case!(
    read_view_option_comments_file,
    ["--comments", "inline"],
    ["-o", "snap.json"]
);
read_view_option_case!(
    read_view_option_track_changes_json,
    ["--track-changes", "all"],
    ["--json"]
);
read_view_option_case!(
    read_view_option_track_changes_tables,
    ["--track-changes", "all"],
    ["--tables"]
);
read_view_option_case!(
    read_view_option_track_changes_file,
    ["--track-changes", "all"],
    ["-o", "snap.json"]
);
read_view_option_case!(read_view_option_dates_json, ["--dates"], ["--json"]);
read_view_option_case!(read_view_option_dates_tables, ["--dates"], ["--tables"]);
read_view_option_case!(read_view_option_dates_for_edit, ["--dates"], ["--for-edit"]);
read_view_option_case!(
    read_view_option_dates_file,
    ["--dates"],
    ["-o", "snap.json"]
);

fn read_option_help(option: &str, clause: &str) {
    let (code, stdout, stderr) = run(&["read", "--help"]);
    assert_eq!(code, 0, "{stderr}");
    let block = stdout
        .split(&format!("      {option}"))
        .nth(1)
        .unwrap()
        .split("\n      --")
        .next()
        .unwrap();
    let words = block.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(words.contains(clause), "{option}: {words}");
}

#[test]
fn read_help_track_changes_exclusion() {
    read_option_help(
        "--track-changes",
        "--json and --tables read the accepted text",
    );
}

#[test]
fn read_help_comments_exclusion() {
    read_option_help(
        "--comments",
        "ignored by --json, --for-edit and the --tables grids",
    );
}

#[test]
fn read_help_dates_exclusion() {
    read_option_help(
        "--dates",
        "ignored by --json, --for-edit and the --tables grids",
    );
}

#[test]
fn read_help_no_page_markers_structured_outputs() {
    read_option_help(
        "--no-page-markers",
        "ignored by --json, --for-edit and the --tables grids",
    );
}

/// Repeated `-p` selectors read as one comma list (#1690): the view and
/// the snapshot alike.
#[test]
fn repeated_paragraph_selectors_read_as_one_list() {
    for extra in [&[][..], &["--json"]] {
        let joined: Vec<&str> = [&["read", RECEIVED, "-p", "p3,p8"][..], extra].concat();
        let repeated: Vec<&str> = [&["read", RECEIVED, "-p", "p3", "-p", "p8"][..], extra].concat();
        let (code, want, stderr) = run(&joined);
        assert_eq!(code, 0, "{stderr}");
        let (code, got, stderr) = run(&repeated);
        assert_eq!(code, 0, "{repeated:?}: {stderr}");
        assert_eq!(got, want, "{repeated:?}");
    }
    // `--select`, the alias, repeats the same way.
    assert_eq!(
        run(&["read", RECEIVED, "--select", "p3", "-p", "p8"]).1,
        run(&["read", RECEIVED, "-p", "p3,p8"]).1
    );
}

/// `read --tables -o FILE` writes the grids stdout gets (#1690); a .json
/// name and an existing file without --force are refused, and the input
/// is never overwritten.
#[test]
fn read_tables_writes_its_grids_to_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let (code, grids, stderr) = run(&["read", RECEIVED, "--tables"]);
    assert_eq!(code, 0, "{stderr}");
    for name in ["tables.txt", "tables.md"] {
        let path = dir.path().join(name);
        let path = path.to_str().unwrap();
        let (code, stdout, stderr) = run(&["read", RECEIVED, "--tables", "-o", path]);
        assert_eq!(code, 0, "{name}: {stderr}");
        assert_eq!(stdout, "", "{name}: the grids go to the file");
        assert_eq!(std::fs::read_to_string(path).unwrap(), grids, "{name}");
        // The file exists now: refused as a usage error, and --force
        // overwrites it.
        let (code, _, stderr) = run(&["read", RECEIVED, "--tables", "-o", path]);
        assert_eq!(code, 2, "{name}: {stderr}");
        assert!(stderr.contains("OUTPUT_EXISTS"), "{name}: {stderr}");
        let (code, _, stderr) = run(&["read", RECEIVED, "--tables", "-o", path, "--force"]);
        assert_eq!(code, 0, "{name}: {stderr}");
    }
    let json = dir.path().join("tables.json");
    let (code, _, stderr) = run(&["read", RECEIVED, "--tables", "-o", json.to_str().unwrap()]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("--tables"), "{stderr}");
    // The input is never overwritten, including through a hard link.
    let copy = dir.path().join("received.docx");
    std::fs::copy(RECEIVED, &copy).unwrap();
    let alias = dir.path().join("alias.md");
    std::fs::hard_link(&copy, &alias).unwrap();
    let (copy, alias) = (copy.to_str().unwrap(), alias.to_str().unwrap());
    let (code, _, stderr) = run(&["read", copy, "--tables", "-o", copy, "--force"]);
    assert_eq!(code, 2, "{stderr}");
    let (code, _, stderr) = run(&["read", copy, "--tables", "-o", alias, "--force"]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("OUTPUT_ALIAS"), "{stderr}");
    assert_eq!(
        std::fs::read(copy).unwrap(),
        std::fs::read(RECEIVED).unwrap()
    );
    assert_eq!(std::fs::read(alias).unwrap(), std::fs::read(copy).unwrap());
}

#[test]
fn read_tables_refuses_input_aliases_before_reading() {
    let dir = tempfile::tempdir().unwrap();
    let original = std::fs::read(RECEIVED).unwrap();
    for name in ["source", "source.md"] {
        let source = dir.path().join(name);
        let hardlink = dir.path().join(format!("{name}-alias.md"));
        std::fs::write(&source, &original).unwrap();
        std::fs::hard_link(&source, &hardlink).unwrap();
        for output in [source.clone(), dir.path().join(".").join(name), hardlink] {
            for force in [false, true] {
                let mut args = vec![
                    "read",
                    source.to_str().unwrap(),
                    "--tables",
                    "-o",
                    output.to_str().unwrap(),
                ];
                if force {
                    args.push("--force");
                }
                let (code, stdout, stderr) = run(&args);
                assert_eq!(code, 2, "{args:?}: {stderr}");
                assert!(stdout.is_empty(), "{stdout}");
                // The CLI names paths with forward slashes on every
                // platform (`slash_path`); `display()` keeps Windows'
                // backslashes.
                assert_eq!(
                    stderr,
                    format!(
                        "error: OUTPUT_ALIAS: the output '{}' is the same file as the input '{}'\n",
                        jubarte::cli::slash_path(&output),
                        jubarte::cli::slash_path(&source)
                    )
                );
                assert_eq!(std::fs::read(&source).unwrap(), original);
                assert_eq!(std::fs::read(&output).unwrap(), original);
            }
        }
    }
    // Prove the refusal precedes parsing the input, even with --force.
    let invalid = dir.path().join("not-a-package.md");
    std::fs::write(&invalid, "preserve me").unwrap();
    let (code, _, stderr) = run(&[
        "read",
        invalid.to_str().unwrap(),
        "--tables",
        "-o",
        invalid.to_str().unwrap(),
        "--force",
    ]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("OUTPUT_ALIAS"), "{stderr}");
    assert_eq!(std::fs::read_to_string(invalid).unwrap(), "preserve me");
}

#[cfg(unix)]
#[test]
fn read_tables_refuses_symlink_to_input() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.md");
    let alias = dir.path().join("alias.md");
    let original = std::fs::read(RECEIVED).unwrap();
    std::fs::write(&source, &original).unwrap();
    std::os::unix::fs::symlink(&source, &alias).unwrap();
    let (code, _, stderr) = run(&[
        "read",
        source.to_str().unwrap(),
        "--tables",
        "-o",
        alias.to_str().unwrap(),
        "--force",
    ]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("OUTPUT_ALIAS"), "{stderr}");
    assert!(std::fs::symlink_metadata(alias).unwrap().is_symlink());
    assert_eq!(std::fs::read(source).unwrap(), original);
}

/// --force replaces a separate output's directory entry, preserving both
/// the input and any other file linked to that old output inode.
#[test]
fn read_tables_force_replaces_output_entry() {
    let dir = tempfile::tempdir().unwrap();
    let original = std::fs::read(RECEIVED).unwrap();
    let (code, grids, stderr) = run(&["read", RECEIVED, "--tables"]);
    assert_eq!(code, 0, "{stderr}");
    for symlink in [false, true] {
        if symlink && !cfg!(unix) {
            continue;
        }
        let target = dir.path().join(if symlink {
            "symbol-target.md"
        } else {
            "hard-target.md"
        });
        let output = target.with_extension("txt");
        std::fs::write(&target, "keep this output inode").unwrap();
        if symlink {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &output).unwrap();
        } else {
            std::fs::hard_link(&target, &output).unwrap();
        }
        let (code, stdout, stderr) = run(&[
            "read",
            RECEIVED,
            "--tables",
            "-o",
            output.to_str().unwrap(),
            "--force",
        ]);
        assert_eq!(code, 0, "{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert_eq!(std::fs::read_to_string(&output).unwrap(), grids);
        assert!(!std::fs::symlink_metadata(&output).unwrap().is_symlink());
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "keep this output inode"
        );
        assert!(!same_file::is_same_file(&target, &output).unwrap());
        assert_eq!(std::fs::read(RECEIVED).unwrap(), original);
    }
}

/// `changes --author NAME` filters listed changes by exact, repeatable names
/// (#1690), comment-body revisions included (#1706).
#[test]
fn changes_filter_by_author() {
    let dir = tempfile::tempdir().unwrap();
    let mark = |kind: &str, id: u32, author: &str, text: &str| {
        let t = if kind == "ins" { "t" } else { "delText" };
        format!(
            r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="2026-10-24T00:00:00Z"><w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>"#
        )
    };
    let body = format!(
        "<w:p><w:r><w:t>Kept </w:t></w:r>{}</w:p><w:p><w:r><w:t>Also </w:t></w:r>{}</w:p><w:p>{}</w:p>",
        mark("ins", 1, "Ana", "new"),
        mark("del", 2, "Bo", "old"),
        mark("ins", 3, "Ana Maria", "more")
    );
    let path = dir.path().join("two.docx");
    std::fs::write(&path, docx(&body)).unwrap();
    let path = path.to_str().unwrap();
    let lines = |args: &[&str]| -> Vec<String> {
        let (code, stdout, stderr) = run(args);
        assert_eq!(code, 0, "{args:?}: {stderr}");
        stdout.lines().map(str::to_string).collect()
    };
    let all = lines(&["changes", path]);
    let ana = lines(&["changes", path, "--author", "Ana"]);
    assert_eq!(ana.len(), 2, "one change and the count: {ana:?}");
    assert!(
        ana[0].contains("\tAna\t") && ana[0].contains("new"),
        "{ana:?}"
    );
    assert_eq!(ana[1], "1 change(s)");
    let both = lines(&["changes", path, "--author", "Ana", "--author", "Bo"]);
    assert_eq!(both.len(), 3, "{both:?}");
    assert_eq!(all.len(), 4, "{all:?}");
    let none = lines(&["changes", path, "--author", "Nobody"]);
    assert_eq!(none, ["0 change(s)"]);
    // JSONL: the same selection, one object per line.
    let json: Vec<serde_json::Value> = lines(&["changes", path, "--json", "--author", "Bo"])
        .iter()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|v: &serde_json::Value| v.get("author").is_some())
        .collect();
    assert_eq!(json.len(), 1, "{json:?}");
    assert_eq!(json[0]["author"], "Bo");
}

/// #1706: the issue's document — Cara's insertion inside comment 0, and no
/// body revision — is one change, `comments.r9`, in the text listing, in
/// `--json`, and under `--author Cara`. `accept`/`reject --id` settle that
/// id. `c0.r9` is refused: it is not a change id.
#[test]
fn changes_lists_a_revision_inside_a_comment_body() {
    use common::docx::{Part, W_NS, docx_with, part_string};
    let dir = tempfile::tempdir().unwrap();
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Rev"><w:p><w:r><w:t>plain </w:t></w:r><w:ins w:id="9" w:author="Cara" w:date="2026-10-01T00:00:00Z"><w:r><w:t>revised</w:t></w:r></w:ins></w:p></w:comment></w:comments>"#
    );
    let bytes = docx_with(
        r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>anchored</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    let source = dir.path().join("source.docx");
    std::fs::write(&source, &bytes).unwrap();
    let source = source.to_str().unwrap();
    let (code, stdout, stderr) = run(&["changes", source]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("comments.r9\tinsertion\ttext\tCara\t"),
        "{stdout}"
    );
    assert!(stdout.trim_end().ends_with("1 change(s)"), "{stdout}");
    let (code, stdout, stderr) = run(&["changes", source, "--author", "Cara"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("comments.r9"), "{stdout}");
    let (code, stdout, stderr) = run(&["changes", source, "--author", "Rev"]);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout.trim(), "0 change(s)");
    let (code, stdout, stderr) = run(&["changes", source, "--json"]);
    assert_eq!(code, 0, "{stderr}");
    let record: serde_json::Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    assert_eq!(record["id"], "comments.r9");
    assert_eq!(record["author"], "Cara");
    assert_eq!(record["kind"], "insertion");

    let accepted = dir.path().join("accepted.docx");
    let (code, _, stderr) = run(&[
        "accept",
        source,
        "--id",
        "comments.r9",
        "-o",
        accepted.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let comments = part_string(&std::fs::read(&accepted).unwrap(), "word/comments.xml").unwrap();
    assert!(!comments.contains("<w:ins"), "{comments}");
    assert!(comments.contains("revised"), "{comments}");

    let rejected = dir.path().join("rejected.docx");
    let (code, _, stderr) = run(&[
        "reject",
        source,
        "--id",
        "comments.r9",
        "-o",
        rejected.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let comments = part_string(&std::fs::read(&rejected).unwrap(), "word/comments.xml").unwrap();
    assert!(
        !comments.contains("<w:ins") && !comments.contains("revised"),
        "{comments}"
    );

    let missed = dir.path().join("miss.docx");
    let (code, _, stderr) = run(&[
        "accept",
        source,
        "--id",
        "r9",
        "-o",
        missed.to_str().unwrap(),
    ]);
    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("no tracked change r9"),
        "a body id does not name the comment revision: {stderr}"
    );
    let (code, _, stderr) = run(&[
        "accept",
        source,
        "--id",
        "c0.r9",
        "-o",
        missed.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("c0.r9"), "{stderr}");

    let (code, _, stderr) = run(&[
        "accept",
        source,
        "--author",
        "Cara",
        "-o",
        accepted.to_str().unwrap(),
        "--force",
    ]);
    assert_eq!(code, 0, "{stderr}");
    let comments = part_string(&std::fs::read(&accepted).unwrap(), "word/comments.xml").unwrap();
    assert!(
        !comments.contains("<w:ins") && comments.contains("revised"),
        "{comments}"
    );
    let (code, help, stderr) = run(&["changes", "--help"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        help.contains("comments.r9"),
        "the listing names the comments story: {help}"
    );
    assert!(
        !help.contains("Comment-body revisions are not listed"),
        "{help}"
    );
    let (code, help, stderr) = run(&["accept", "--help"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        !help.contains("no ID names a comment-body change"),
        "{help}"
    );
    assert!(help.contains("comments.r9"), "{help}");
}

/// `edit --help` names a plan's contents as operations, shows one
/// complete plan that applies as printed (once its hash is filled in),
/// and the separate template/apply workflow (#1690).
#[test]
fn edit_help_shows_a_complete_plan_and_the_template_workflow() {
    let (code, help, stderr) = run(&["edit", "--help"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(!help.contains("batches"), "{help}");
    assert!(help.contains("operations"), "{help}");
    assert!(
        help.contains("jubarte edit template FILE") && help.contains("jubarte edit apply FILE"),
        "{help}"
    );
    let start = help
        .find("{\"schema_version\"")
        .expect("a plan in the help");
    let end = start + help[start..].find("]}").expect("the plan's end") + 2;
    let dir = tempfile::tempdir().unwrap();
    let doc = write_fixture(dir.path());
    let sha = source_sha256(&std::fs::read(&doc).unwrap());
    let plan = help[start..end].replace("<sha256 from jubarte read>", &sha);
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, &plan).unwrap();
    let out = dir.path().join("out.docx");
    let (code, stdout, stderr) = run(&[
        "edit",
        doc.to_str().unwrap(),
        "--plan",
        plan_path.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{plan}\n{stdout}\n{stderr}");
    assert!(out.exists());
}

#[test]
fn skill_distinguishes_legacy_and_typed_edit_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let source = write_fixture(dir.path());
    let (code, stdout, stderr) = run(&[
        "edit",
        "replace",
        source.to_str().unwrap(),
        "--find",
        "his or her",
        "--text",
        "their",
    ]);
    assert_eq!(code, 2, "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("writes require -o FILE or --out-dir NEW_DIR"),
        "{stderr}"
    );
    assert!(!dir.path().join("letter_modified.docx").exists());
    let (code, _, stderr) = run(&[
        "edit",
        source.to_str().unwrap(),
        "-p",
        "p1",
        "--anchor",
        "his or her",
        "--content",
        "their",
        "-a",
        "Jubarte",
    ]);
    assert_eq!(code, 0, "{stderr}");
    assert!(dir.path().join("letter_modified.docx").exists());
    let skill = include_str!("../skills/jubarte-documents/SKILL.md");
    assert!(
        skill.contains("legacy `edit FILE` and `add FILE`"),
        "qualify the implicit output claim"
    );
    assert!(
        skill.contains("Typed edit actions require `-o` or `--out-dir` to write"),
        "document typed edit outputs"
    );
}
