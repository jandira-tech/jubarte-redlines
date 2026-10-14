// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit --plan` and `add --plan` with operation flags record into the plan
//! instead of editing; `--plan` alone applies it, and `--consume` deletes it
//! once applied.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::docx::{docx, para};
use jubarte::inspect::source_sha256;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn letter(dir: &Path, name: &str, last: &str) -> PathBuf {
    let body = para("Heading")
        + &para("The individual signs in his or her individual capacity.")
        + &para(last);
    let path = dir.join(name);
    std::fs::write(&path, docx(&body)).unwrap();
    path
}

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

fn plan(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join("plan.json")).unwrap()).unwrap()
}

#[test]
fn edit_and_add_record_into_one_plan_that_edit_applies_and_consumes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let file = letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let before = std::fs::read(&file).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        out.trim(),
        "recorded 1 operation in plan.json: 1 operation, 0 revisions to accept, 0 to reject; \
         apply with: jubarte edit letter.docx --plan plan.json"
    );
    let (code, out, err) = run(
        dir,
        &[
            "add",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--comment",
            "--content",
            "Why 3?",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(
        out.starts_with("recorded 1 operation in plan.json: 2 operations"),
        "{out}"
    );
    // Recording edits nothing: FILE is as it was and no output bundle exists.
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert!(!dir.join("letter.edit").exists());
    // The plan reads as one written by hand: no null fields.
    let json = std::fs::read_to_string(dir.join("plan.json")).unwrap();
    assert!(!json.contains("null"), "{json}");
    let recorded = plan(dir);
    assert_eq!(recorded["operations"].as_array().unwrap().len(), 2);
    assert_eq!(recorded["source_sha256"], source_sha256(&before));
    assert_eq!(recorded["author"], "Modified User");

    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--out-dir",
            "out",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(
        !dir.join("plan.json").exists(),
        "--consume deletes the plan"
    );
    let (_, view, _) = run(dir, &["read", "out/redline.docx"]);
    assert!(view.contains("{~~his or her~>their~~}"), "{view}");
    assert!(view.contains("Why 3?"), "{view}");
}

#[test]
fn a_recording_that_does_not_apply_leaves_the_plan_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "Gone.",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let held = std::fs::read(dir.join("plan.json")).unwrap();
    let (code, out, _) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "nowhere",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 3, "{out}");
    assert!(out.contains(r#""status":"failed""#), "{out}");
    assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), held);
    let (code, out, _) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--dry-run",
            "-p",
            "p1",
            "--content",
            "Checked only.",
        ],
    );
    assert_eq!(code, 0);
    assert!(
        out.starts_with("would record 1 operation in plan.json: 2 operations"),
        "{out}"
    );
    assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), held);
}

#[test]
fn a_recording_refuses_what_applies_later_and_what_the_plan_names_otherwise() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    letter(dir, "other.docx", "Sections 4 and 5 survive.");
    let record = |extra: &[&str], file: &str| {
        let mut args = vec![
            "edit",
            file,
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ];
        args.extend_from_slice(extra);
        run(dir, &args)
    };
    for flag in [
        &["--out-dir", "d"][..],
        &["--pdf"],
        &["--consume"],
        &["--editing-mode"],
    ] {
        let (code, _, err) = record(flag, "letter.docx");
        assert_eq!(code, 2, "{flag:?}");
        assert!(
            err.contains(&format!(
                "{} takes effect when the plan is applied",
                flag[0]
            )),
            "{err}"
        );
        assert!(!dir.join("plan.json").exists(), "{flag:?}");
    }
    assert_eq!(record(&[], "letter.docx").0, 0);
    let (code, _, err) = record(&["--author", "Bob"], "letter.docx");
    assert_eq!(code, 2);
    assert!(
        err.contains(r#"the plan's author is "Modified User"; pass --author "Modified User""#),
        "{err}"
    );
    let (code, _, err) = record(&[], "other.docx");
    assert_eq!(code, 2);
    assert!(err.contains("recorded against another document"), "{err}");
    assert_eq!(plan(dir)["operations"].as_array().unwrap().len(), 1);
}

#[test]
fn consume_keeps_the_plan_until_it_has_been_applied() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    letter(dir, "other.docx", "Sections 4 and 5 survive.");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--dry-run",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("plan.json").exists(), "a dry run consumes nothing");
    // The plan is pinned to letter.docx: another file is refused as stale.
    let (code, _, _) = run(
        dir,
        &[
            "edit",
            "other.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--out-dir",
            "o",
        ],
    );
    assert_eq!(code, 3);
    assert!(dir.join("plan.json").exists(), "a refused plan is kept");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--author",
            "Bob",
        ],
    );
    assert_eq!(code, 2);
    assert!(
        err.contains("--author belongs to recorded operations"),
        "{err}"
    );
    let (code, _, err) = run(
        dir,
        &["add", "letter.docx", "--plan", "plan.json", "--consume"],
    );
    assert_eq!(code, 0, "{err}");
    assert!(!dir.join("plan.json").exists());
    assert!(dir.join("letter_modified.docx").exists());
    assert!(
        !dir.join("letter.edit").exists(),
        "only --out-dir writes the bundle"
    );
}

/// zcode review 06b F3, F4, F6, F8, F9.
#[test]
fn render_flags_explicit_defaults_links_and_overwritten_plans_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let record = |extra: &[&str]| {
        let mut args = vec![
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ];
        args.extend_from_slice(extra);
        run(dir, &args)
    };
    for flag in [&["--dpi", "300"][..], &["--revisions", "word"]] {
        let (code, _, err) = record(flag);
        assert_eq!(code, 2, "{flag:?}");
        assert!(
            err.contains(&format!(
                "{} takes effect when the plan is applied",
                flag[0]
            )),
            "{err}"
        );
    }
    assert_eq!(record(&[]).0, 0);
    // The default author spelled out is still a choice the plan names.
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--author",
            "Modified User",
        ],
    );
    assert_eq!(code, 2);
    assert!(
        err.contains("--author belongs to recorded operations"),
        "{err}"
    );
    // add without -p names -p, not --plan.
    let (code, _, err) = run(dir, &["add", "letter.docx"]);
    assert_eq!(code, 2);
    assert!(err.contains("--location <WHERE>"), "{err}");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("plan.json", dir.join("link.json")).unwrap();
        let (code, _, err) = run(
            dir,
            &["edit", "letter.docx", "--plan", "link.json", "--consume"],
        );
        assert_eq!(code, 2);
        assert!(
            err.contains("a symbolic link; pass the file it names"),
            "{err}"
        );
        // Recording through the link grows the file it names.
        let (code, _, err) = run(
            dir,
            &[
                "edit",
                "letter.docx",
                "--plan",
                "link.json",
                "-p",
                "p0",
                "--delete",
            ],
        );
        assert_eq!(code, 0, "{err}");
        assert!(
            dir.join("link.json")
                .symlink_metadata()
                .unwrap()
                .is_symlink()
        );
        assert_eq!(plan(dir)["operations"].as_array().unwrap().len(), 2);
    }
    // A plan an output overwrites is not deleted as if it were the plan.
    std::fs::create_dir(dir.join("out")).unwrap();
    std::fs::copy(dir.join("plan.json"), dir.join("out/report.jsonl")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "out/report.jsonl",
            "--out-dir",
            "out",
            "--force",
            "--consume",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("it no longer holds the plan"), "{err}");
    assert!(dir.join("out/redline.docx").exists());
    assert!(dir.join("out/report.jsonl").exists());
}
