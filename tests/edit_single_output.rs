// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit` and `add` write one document, `FILE_modified.docx` beside FILE
//! (`-o` names another); `--out-dir` writes the review bundle instead.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn letter(dir: &Path) {
    let body = para("Heading") + &para("The fee is thirty dollars.") + &para("Last line.");
    std::fs::write(dir.join("a.docx"), docx(&body)).unwrap();
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

fn document_xml(path: &Path) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("word/document.xml").unwrap(), &mut xml)
        .unwrap();
    xml
}

/// The files in `dir` besides `a.docx`, sorted.
fn written(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n != "a.docx")
        .collect();
    names.sort();
    names
}

const EDIT: [&str; 6] = [
    "edit",
    "a.docx",
    "-p",
    "p1",
    "--content",
    "The fee is forty dollars.",
];

#[test]
fn edit_writes_file_modified_docx_and_prints_its_view() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, out, err) = run(dir, &EDIT);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx"]);
    let xml = document_xml(&dir.join("a_modified.docx"));
    assert!(xml.contains("<w:ins "), "a redline by default: {xml}");
    assert!(out.contains("\nsource: a_modified.docx\n"), "{out}");
    assert!(err.contains("wrote a_modified.docx (docx, "), "{err}");

    let (code, _, err) = run(dir, &EDIT);
    assert_eq!(code, 2, "an existing output is refused: {err}");
    assert!(err.contains("--force"), "{err}");
    let mut forced = EDIT.to_vec();
    forced.push("--force");
    let (code, _, err) = run(dir, &forced);
    assert_eq!(code, 0, "{err}");
}

/// D15/Q14: a FILE whose stem ends `_modified` keeps its name, so editing
/// it again replaces it, with --force; `a.docx` is never touched.
#[test]
fn a_modified_file_is_edited_in_place_with_force() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let before = std::fs::read(dir.join("a.docx")).unwrap();
    assert_eq!(run(dir, &EDIT).0, 0);
    let again = [
        "add",
        "a_modified.docx",
        "-p",
        "p2",
        "--content",
        "New line.",
    ];
    let (code, _, err) = run(dir, &again);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("'a_modified.docx' already exists (use --force"),
        "{err}"
    );
    let mut forced = again.to_vec();
    forced.push("--force");
    let (code, out, err) = run(dir, &forced);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\nsource: a_modified.docx\n"), "{out}");
    assert_eq!(written(dir), ["a_modified.docx"]);
    assert!(document_xml(&dir.join("a_modified.docx")).contains("New line."));
    assert_eq!(std::fs::read(dir.join("a.docx")).unwrap(), before);
}

#[test]
fn editing_mode_writes_the_clean_document() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.push("--editing-mode");
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    let xml = document_xml(&dir.join("a_modified.docx"));
    assert!(!xml.contains("<w:ins "), "{xml}");
    assert!(xml.contains("forty"), "{xml}");
}

#[test]
fn add_writes_file_modified_docx_too() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &["add", "a.docx", "-p", "p2", "--content", "New line."],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx"]);
}

#[test]
fn out_dir_writes_the_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut bare = EDIT.to_vec();
    bare.push("--out-dir");
    let (code, out, err) = run(dir, &bare);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a.edit"]);
    for name in ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"] {
        assert!(
            dir.join("a.edit").join(name).is_file(),
            "a.edit holds {name}"
        );
    }
    assert!(out.contains("\nsource: a.edit/redline.docx\n"), "{out}");

    let mut named = EDIT.to_vec();
    named.extend(["--out-dir", "r"]);
    let (code, _, err) = run(dir, &named);
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("r/redline.docx").is_file());
}

#[test]
fn o_names_the_one_document_and_refuses_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    for bad in [
        &["-o", "b.docx", "--out-dir", "r"][..],
        &["-o", "b.pdf"][..],
        &["-o", "a.docx"][..],
        &["--out-dir", "x.docx"][..],
    ] {
        let mut args = EDIT.to_vec();
        args.extend(bad);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{bad:?}: {err}");
        assert!(written(dir).is_empty(), "{bad:?} wrote nothing");
    }
    let mut args = EDIT.to_vec();
    args.extend(["-o", "b.docx"]);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["b.docx"]);
    assert!(out.contains("\nsource: b.docx\n"), "{out}");
}

#[test]
fn pdf_lands_beside_the_document() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["--pdf", "-q"]);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx", "a_modified.pdf"]);
    assert!(out.is_empty() && err.is_empty(), "-q: {out}{err}");
}

#[test]
fn a_refused_plan_writes_nothing_and_prints_its_report() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "not in the paragraph",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 3, "{out}{err}");
    assert!(written(dir).is_empty(), "{:?}", written(dir));
    assert!(out.contains("not in the paragraph"), "{out}");
}
