// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A published DOCX prints the before/after confirmation. Repair uses the
//! bytes it already read. A Word 97-2003 file uses the package it was
//! read into, not a second read of the original.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::docx;

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
fn repair_prints_the_success_block() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.docx"), docx(DELETED)).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("SUCCESS:\n~~~~diff\n") && !out.contains("cannot be shown"),
        "repair published without the confirmation block: {out}{err}"
    );
    assert!(dir.path().join("fixed.docx").is_file());
}

#[test]
fn converting_a_legacy_doc_prints_the_success_block() {
    let dir = tempfile::tempdir().unwrap();
    let doc = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/services.doc"),
    )
    .unwrap();
    std::fs::write(dir.path().join("old.doc"), &doc).unwrap();
    let (code, out, err) = run(dir.path(), &["convert", "old.doc", "-o", "out.docx"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("SUCCESS:\n~~~~diff\n") && !out.contains("cannot be shown"),
        "the confirmation reread the legacy file: {out}{err}"
    );
    assert!(dir.path().join("out.docx").is_file());
}
