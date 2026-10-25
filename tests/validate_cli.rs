// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte validate`: exit 0 clean, 2 on a word or edits finding (or an
//! error, or a warning under `--strict`), 1 unreadable; `--json`,
//! `--repair`, `--original --author`, `--rules`.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const DELETED: &str = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
/// Two insertions sharing an id: a lead, not Word-fatal.
const SAME_ID: &str = r#"<w:p><w:ins w:id="5" w:author="A"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="5" w:author="A"><w:r><w:t>b</w:t></w:r></w:ins></w:p>"#;

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
        out.starts_with(
            "* TEXT_INSIDE_DELETION\terror\tword/document.xml#\
             w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]\t"
        ),
        "{out}"
    );
    assert!(
        out.ends_with("1 finding(s): 1 error, 0 warning, 0 info\n"),
        "{out}"
    );

    let (code, out, _) = run(dir.path(), &["validate", "broken.docx", "--json"]);
    assert_eq!(code, 2);
    let row: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(row["code"], "TEXT_INSIDE_DELETION");
    assert_eq!(row["set"], "word");
    assert_eq!(row["severity"], "error");
    assert_eq!(row["location"], "w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]");
    assert_eq!(row["ref"], "w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]");
    assert_eq!(row["word_fatal"], true);
    assert!(row.get("path").is_none(), "{row}");

    let (code, out, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out, "repaired 1 finding(s)\nno findings\n");
    assert!(err.starts_with("wrote fixed.docx (docx, "), "{err}");
    let (_, out, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "quiet.docx", "-q"],
    );
    assert_eq!(
        (out.as_str(), err.as_str()),
        ("repaired 1 finding(s)\nno findings\n", "")
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
    assert_eq!(row["set"], "edits");
    assert_eq!(row["severity"], "error");
    assert_eq!(row["location"], "p0");
    assert_eq!(row["ref"], "p0");
    // The JSON location keeps the long id; the message prints the short one.
    assert!(
        row["message"].as_str().unwrap().starts_with("p0 differs"),
        "{row}"
    );

    let (code, _, _) = run(
        dir.path(),
        &[
            "a.docx",
            "b.docx",
            "-o",
            "r.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "--author",
            "Legal",
        ],
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

#[test]
fn a_word_lead_still_fails_but_audit_warnings_fail_only_under_strict() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("lead.docx"), docx(SAME_ID)).unwrap();
    std::fs::write(dir.path().join("bullets.docx"), docx(&para("• item"))).unwrap();

    let (code, out, _) = run(dir.path(), &["validate", "lead.docx"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.starts_with("  DUPLICATE_REVISION_ID\twarning\t"),
        "{out}"
    );

    let (code, out, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "style"],
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("  LITERAL_BULLET\twarning\tp0\t"), "{out}");
    assert!(
        out.ends_with("1 finding(s): 0 error, 1 warning, 0 info\n"),
        "{out}"
    );
    let (code, _, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "style", "--strict"],
    );
    assert_eq!(code, 2);
    // word,style: the Word check runs too and finds nothing.
    let (code, out, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "word,style"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out.lines().count(), 2, "{out}");
}

#[test]
fn rules_usage_errors_exit_two() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), docx(&para("a"))).unwrap();
    let (code, _, err) = run(dir.path(), &["validate", "a.docx", "--rules", "edits"]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("--rules edits needs --original and --author"),
        "{err}"
    );
    let (code, _, err) = run(dir.path(), &["validate", "a.docx", "--rules", "BOGUS"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("BOGUS"), "{err}");
    assert!(err.contains("word, edits, a11y, style, structure"), "{err}");
}
