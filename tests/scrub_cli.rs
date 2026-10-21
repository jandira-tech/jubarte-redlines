// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte scrub FILE -o OUT`: no flag scrubs everything with the alias
//! `Author`; flags pick what to scrub.

mod common;

use std::process::Command;

use common::docx::{docx, para, part_string};
use jubarte::document_comparer::compare_documents;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn no_flag_aliases_every_author_as_author_and_refuses_to_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let red = compare_documents(&docx(&para("a")), &docx(&para("b")), "Jane Secret").unwrap();
    let input = dir.path().join("red.docx");
    std::fs::write(&input, red).unwrap();
    let out = dir.path().join("out.docx");
    let (code, _, stderr) = run(&[
        "scrub",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let xml = part_string(&std::fs::read(&out).unwrap(), "word/document.xml").unwrap();
    assert!(xml.contains(r#"w:author="Author""#) && !xml.contains("Jane Secret"));
    let (code, _, stderr) = run(&[
        "scrub",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(code, 1);
    assert!(stderr.contains("already exists"), "{stderr}");
}

#[test]
fn flags_pick_what_to_scrub() {
    let dir = tempfile::tempdir().unwrap();
    let red = compare_documents(&docx(&para("a")), &docx(&para("b")), "Jane Secret").unwrap();
    let input = dir.path().join("red.docx");
    std::fs::write(&input, red).unwrap();
    // --rsids alone keeps the author.
    let out = dir.path().join("rsids.docx");
    let (code, _, stderr) = run(&[
        "scrub",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--rsids",
    ]);
    assert_eq!(code, 0, "{stderr}");
    let xml = part_string(&std::fs::read(&out).unwrap(), "word/document.xml").unwrap();
    assert!(xml.contains("Jane Secret"));
    // --author-alias alone renames.
    let out = dir.path().join("alias.docx");
    let (code, _, _) = run(&[
        "scrub",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--author-alias",
        "Counsel",
    ]);
    assert_eq!(code, 0);
    let xml = part_string(&std::fs::read(&out).unwrap(), "word/document.xml").unwrap();
    assert!(xml.contains(r#"w:author="Counsel""#), "{xml}");
}

#[test]
fn an_unreadable_input_exits_1() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("junk.docx");
    std::fs::write(&input, b"not a zip").unwrap();
    let out = dir.path().join("out.docx");
    let (code, _, stderr) = run(&[
        "scrub",
        input.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(code, 1);
    assert!(stderr.starts_with("error:"), "{stderr}");
    assert!(!out.exists());
}
