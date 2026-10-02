// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte validate`: exit 0 clean, 2 findings, 1 unreadable; `--json`,
//! `--repair`, `--original --author`.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const DELETED: &str = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;

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

#[test]
fn validate_exit_codes_prose_json_and_repair() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.docx"), docx(DELETED)).unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("fine"))).unwrap();

    let (code, out, _) = run(dir.path(), &["validate", "clean.docx"]);
    assert_eq!((code, out.trim()), (0, "no findings"));

    let (code, out, _) = run(dir.path(), &["validate", "broken.docx"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.starts_with("* TEXT_INSIDE_DELETION\tword/document.xml#w:body[0]/w:p[0]"),
        "{out}"
    );
    assert!(out.contains("1 finding(s), 1 Word-fatal"), "{out}");

    let (code, out, _) = run(dir.path(), &["validate", "broken.docx", "--json"]);
    assert_eq!(code, 2);
    let row: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(row["code"], "TEXT_INSIDE_DELETION");
    assert_eq!(row["word_fatal"], true);

    let (code, out, _) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("repaired 1 finding(s) into fixed.docx"),
        "{out}"
    );
    let (code, _, _) = run(dir.path(), &["validate", "fixed.docx"]);
    assert_eq!(code, 0);

    let (code, _, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 1);
    assert!(err.contains("already exists"), "{err}");
    let (code, _, _) = run(
        dir.path(),
        &[
            "validate",
            "broken.docx",
            "--repair",
            "fixed.docx",
            "--force",
        ],
    );
    assert_eq!(code, 0);

    let (code, _, err) = run(dir.path(), &["validate", "missing.docx"]);
    assert_eq!(code, 1);
    assert!(err.starts_with("error:"), "{err}");

    let (code, _, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--original", "clean.docx"],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("--author"), "{err}");
}

#[test]
fn validate_audits_tracked_edits_against_an_original() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), docx(&para("Fee is 10."))).unwrap();
    std::fs::write(dir.path().join("b.docx"), docx(&para("Fee is 12."))).unwrap();
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "b.docx",
            "--original",
            "a.docx",
            "--author",
            "Legal",
            "--json",
        ],
    );
    assert_eq!(code, 2, "{out}");
    let row: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(row["code"], "UNTRACKED_EDIT");
    assert!(row["message"].as_str().unwrap().contains("body:p:0"));

    let (code, _, _) = run(
        dir.path(),
        &["a.docx", "b.docx", "-o", "r.docx", "--author", "Legal"],
    );
    assert_eq!(code, 0);
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "r.docx",
            "--original",
            "a.docx",
            "--author",
            "Legal",
        ],
    );
    assert_eq!((code, out.trim()), (0, "no findings"));
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "r.docx",
            "--original",
            "a.docx",
            "--author",
            "Other",
        ],
    );
    assert_eq!(code, 2);
    assert!(out.contains("FOREIGN_AUTHOR"), "{out}");
}
