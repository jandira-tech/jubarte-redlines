// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A stale-plan recovery keeps the authored row, runs with the plan's own
//! attribution, and writes somewhere the refused command is not already using.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::docx::docx;
use jubarte::inspect::{paragraphs, source_sha256};
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
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

fn same_place(suggested: &str, other: &Path) -> bool {
    let suggested = Path::new(suggested);
    match (suggested.canonicalize(), other.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => {
            suggested.file_name() == other.file_name()
                && suggested.parent().and_then(|path| path.canonicalize().ok())
                    == other.parent().and_then(|path| path.canonicalize().ok())
        }
    }
}

/// A one-paragraph document and a plan whose rewrite no longer matches it.
fn stale_plan(dir: &Path, author: &str, date: &str) -> Value {
    std::fs::write(
        dir.join("source.docx"),
        docx(r#"<w:p><w:r><w:t>Current terms.</w:t></w:r></w:p>"#),
    )
    .unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "template",
            "source.docx",
            "-o",
            "plan.json",
            "--author",
            "Keeper",
            "--datetime",
            "2026-10-17T12:00:00Z",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    let mut plan: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("plan.json")).unwrap()).unwrap();
    plan["author"] = json!(author);
    plan["date"] = json!(date);
    plan["operations"][0]["text"] = json!("Authored terms");
    plan["operations"][0]["attached_comment"] = json!({"text": "Why this change"});
    plan["operations"][0]["before_sha256"] = json!(source_sha256(b"stale baseline"));
    std::fs::write(
        dir.join("plan.json"),
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    plan
}

fn recovery(dir: &Path, extra: &[&str]) -> Value {
    let mut args = vec![
        "edit",
        "apply",
        "source.docx",
        "--plan",
        "plan.json",
        "-o",
        "revised.docx",
        "--json",
    ];
    args.extend_from_slice(extra);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 3, "{out}{err}");
    let receipt: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(receipt["error"]["code"], "WORKING_BASE_CHANGED");
    receipt["error"]["next_actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| {
            action["argv"]
                .as_array()
                .is_some_and(|argv| argv.len() >= 3 && argv[1] == "edit" && argv[2] == "template")
        })
        .cloned()
        .unwrap_or_else(|| panic!("no template recovery in {receipt}"))
}

fn argv_of(action: &Value) -> Vec<String> {
    action["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| part.as_str().unwrap().to_string())
        .collect()
}

fn output_of(argv: &[String]) -> PathBuf {
    let index = argv.iter().position(|arg| arg == "-o").unwrap();
    PathBuf::from(&argv[index + 1])
}

#[test]
fn recovery_keeps_authored_text_and_comment() {
    let dir = tempfile::tempdir().unwrap();
    stale_plan(dir.path(), "Keeper", "2026-10-17T12:00:00Z");
    let action = recovery(dir.path(), &[]);
    let argv = argv_of(&action);
    let regenerated = output_of(&argv);
    let followed: Vec<_> = argv.iter().skip(1).map(String::as_str).collect();
    let (code, out, err) = run(dir.path(), &followed);
    assert_eq!(code, 0, "{out}{err}");
    let plan: Value =
        serde_json::from_str(&std::fs::read_to_string(&regenerated).unwrap()).unwrap();
    let row = &plan["operations"][0];
    assert_eq!(row["text"], "Authored terms");
    assert_eq!(row["attached_comment"]["text"], "Why this change");
    assert_eq!(row["before_sha256"], source_sha256(b"Current terms."));
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "source.docx",
            "--plan",
            regenerated.to_str().unwrap(),
            "-o",
            "landed.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    let landed = std::fs::read(dir.path().join("landed.docx")).unwrap();
    assert_eq!(paragraphs(&landed).unwrap()[0].text, "Authored terms");
}

#[test]
fn recovery_runs_for_a_wide_offset_and_keeps_a_padded_author() {
    let dir = tempfile::tempdir().unwrap();
    let author = "  Padded  ";
    let date = "2026-10-17T12:00:00+15:00";
    stale_plan(dir.path(), author, date);
    let argv = argv_of(&recovery(dir.path(), &[]));
    let regenerated = output_of(&argv);
    let followed: Vec<_> = argv.iter().skip(1).map(String::as_str).collect();
    let (code, out, err) = run(dir.path(), &followed);
    assert_eq!(code, 0, "{out}{err}");
    let plan: Value =
        serde_json::from_str(&std::fs::read_to_string(&regenerated).unwrap()).unwrap();
    assert_eq!(plan["author"], author);
    assert_eq!(plan["date"], date);
    assert_eq!(plan["operations"][0]["text"], "Authored terms");
}

#[test]
fn recovery_output_avoids_the_refused_commands_paths() {
    let dir = tempfile::tempdir().unwrap();
    stale_plan(dir.path(), "Keeper", "2026-10-17T12:00:00Z");
    let report = dir.path().join("plan.regenerated.json");
    assert!(!report.exists());
    let argv = argv_of(&recovery(
        dir.path(),
        &["--report", "plan.regenerated.json"],
    ));
    let suggested = output_of(&argv);
    let suggested = suggested.to_str().unwrap();
    for reserved in [
        dir.path().join("source.docx"),
        dir.path().join("plan.json"),
        dir.path().join("revised.docx"),
        report,
    ] {
        assert!(
            !same_place(suggested, &reserved),
            "suggested {suggested} collides with {}",
            reserved.display()
        );
    }
}
