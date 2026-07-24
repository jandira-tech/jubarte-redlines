// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Fallible-input surface must return `Err` / non-zero exit — never panic —
//! on malformed zip, empty packages, or missing main document parts (plan A2).

use std::io::Cursor;
use std::io::Write;
use std::process::Command;

use jubarte::WmlDocument;
use jubarte::document_comparer::{compare_documents, get_revisions};
use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

/// Minimal OPC zip with [Content_Types] + package rels but NO main document.
fn empty_package_bytes() -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
        )
        .unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

#[test]
fn library_malformed_zip_returns_err() {
    let bad = b"this is not a zip at all";
    let err = compare_documents(bad, bad, "Author").expect_err("must Err not panic");
    let _ = format!("{err:?}"); // ensure Display/Debug usable on CLI path
}

#[test]
fn library_empty_package_missing_main_returns_err() {
    let pkg = empty_package_bytes();
    // open may succeed (valid zip + content types); compare must still Err
    assert!(
        PartFs::open(&pkg).is_ok(),
        "empty package is a valid zip shell"
    );
    // Identical-byte fast path skips open; force the full path with a real
    // modified doc so the missing main on `pkg` is reached.
    let modified = std::fs::read("tests/fixtures/redline/modified.docx")
        .expect("fixture present for non-identical pair");
    let err = compare_documents(&pkg, &modified, "Author").expect_err("missing main → Err");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("not found") || msg.contains("PartNotFound") || msg.contains("document"),
        "error should name the missing part: {msg}"
    );
}

#[test]
fn wml_document_missing_main_returns_err() {
    let pkg = empty_package_bytes();
    let mut wml = WmlDocument::from_bytes(&pkg).expect("open shell");
    let err = wml.main_document().expect_err("missing main → Err");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("not found") || msg.contains("PartNotFound") || msg.contains("document"),
        "error should name the missing part: {msg}"
    );
}

#[test]
fn get_revisions_malformed_zip_returns_err() {
    let bad = b"not a zip";
    let settings = jubarte::comparer::WmlComparerSettings::default();
    let _ = get_revisions(bad, &settings).expect_err("must Err not panic");
}

#[test]
fn cli_compare_malformed_zip_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.docx");
    let b = dir.path().join("b.docx");
    std::fs::write(&a, b"not a zip").unwrap();
    std::fs::write(&b, b"also not a zip").unwrap();
    let out = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(dir.path().join("out.docx"))
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "CLI must exit non-zero on bad zip; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("error:"),
        "CLI must print one-line error, got: {err}"
    );
}

#[test]
fn cli_compare_empty_package_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let pkg = empty_package_bytes();
    let a = dir.path().join("a.docx");
    let b = dir.path().join("b.docx");
    std::fs::write(&a, &pkg).unwrap();
    // non-identical pair so the CLI reaches PartFs open + main-part lookup
    std::fs::copy("tests/fixtures/redline/modified.docx", &b).unwrap();
    let out = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(dir.path().join("out.docx"))
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "CLI must exit non-zero on empty package; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}
