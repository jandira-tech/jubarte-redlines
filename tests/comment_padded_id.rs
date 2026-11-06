// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment id written with leading zeros is still named when accept removes it.

mod common;

use std::io::{Cursor, Read, Write};
use std::process::Command;

#[path = "common/dead_comments.rs"]
mod dead_comments;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

/// The probe's comment is `w:id="0"`. Spell that same comment `0005`.
fn padded(docx: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().expect("zip name").to_string();
        let mut data = Vec::new();
        Read::read_to_end(&mut file, &mut data).unwrap();
        if name.ends_with(".xml") {
            let text = String::from_utf8(data).unwrap();
            data = text.replace(r#"w:id="0""#, r#"w:id="0005""#).into_bytes();
        }
        out.start_file(name, opts).unwrap();
        out.write_all(&data).unwrap();
    }
    out.finish().unwrap().into_inner()
}

#[test]
fn a_comment_id_with_leading_zeros_is_named_when_accept_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let output = dir.path().join("resolved.docx");
    std::fs::write(&input, padded(&dead_comments::probe('b'))).unwrap();
    let ran = Command::new(BIN)
        .args([
            "accept",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            "--quiet",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(
        stderr.contains("warning: accept removed comment c5"),
        "padded comment id missing from the warning: {stderr}"
    );
}
