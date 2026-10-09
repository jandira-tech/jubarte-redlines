// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration probes for command dispatch, report writes and field updates.

mod common;

use common::docx::{docx, para, part_string};
use std::path::Path;
use std::process::{Command, Output};

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn success(out: &Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

#[test]
fn fields_update_reports_the_written_cache_and_refuses_clobbering() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    let args = ["fields", "update", "input.docx", "-o", "updated.docx"];
    let out = run(dir.path(), &[args.as_slice(), &["--json"]].concat());
    let report: serde_json::Value = serde_json::from_str(&success(&out)).unwrap();
    assert_eq!(report["page_count"], 1);
    assert_eq!(report["fields"].as_array().unwrap().len(), 1);
    assert_eq!(report["fields"][0]["kind"], "NUMPAGES");
    assert_eq!(report["fields"][0]["old"], "99");
    assert_eq!(report["fields"][0]["new"], "1");
    let saved = std::fs::read(dir.path().join("updated.docx")).unwrap();
    let xml = part_string(&saved, "word/document.xml").unwrap();
    assert!(xml.contains("NUMPAGES") && xml.contains(">1</w:t>"));
    let refused = run(dir.path(), &args);
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("already exists"));
    assert_eq!(
        std::fs::read(dir.path().join("updated.docx")).unwrap(),
        saved
    );
    let out = run(dir.path(), &[args.as_slice(), &["--force"]].concat());
    let text = success(&out);
    assert!(text.contains("NUMPAGES") && text.contains("99") && text.contains('1'));
    assert!(String::from_utf8_lossy(&out.stderr).contains("1 field(s) written; 1 page(s)"));
    assert_eq!(
        std::fs::read(dir.path().join("input.docx")).unwrap(),
        original
    );
}

#[test]
fn output_directories_produce_write_errors_without_replacing_input() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(&para("Preserved source"));
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    std::fs::create_dir(dir.path().join("destination")).unwrap();
    for args in [
        vec![
            "convert",
            "input.docx",
            "-o",
            "destination",
            "--to",
            "pdf",
            "--force",
        ],
        vec![
            "fields",
            "update",
            "input.docx",
            "-o",
            "destination",
            "--force",
        ],
        vec![
            "validate",
            "input.docx",
            "--repair",
            "destination",
            "--force",
        ],
    ] {
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("destination"), "{err}");
        assert!(dir.path().join("destination").is_dir());
        assert_eq!(
            std::fs::read(dir.path().join("input.docx")).unwrap(),
            original
        );
    }
}

#[test]
fn debug_diff_disambiguates_duplicate_stems_and_duplicate_paths() {
    let dir = tempfile::tempdir().unwrap();
    for (folder, text) in [("old", "Original clause"), ("new", "Revised clause")] {
        std::fs::create_dir(dir.path().join(folder)).unwrap();
        std::fs::write(dir.path().join(folder).join("same.docx"), docx(&para(text))).unwrap();
    }
    for files in [
        vec!["old/same.docx", "new/same.docx"],
        vec!["old/same.docx", "old/same.docx", "new/same.docx"],
    ] {
        let args = [
            vec!["debug", "diff"],
            files,
            vec![
                "--full",
                "--raw",
                "-p",
                "document",
                "--para-text",
                "clause",
                "-n",
                "0",
            ],
        ]
        .concat();
        let text = success(&run(dir.path(), &args));
        assert!(
            text.contains("Original clause") && text.contains("Revised clause"),
            "{text}"
        );
    }
    let equal = success(&run(
        dir.path(),
        &["debug", "old/same.docx", "old/same.docx", "--list"],
    ));
    assert!(equal.contains("0 changed"), "{equal}");
}

#[test]
fn inspect_prose_exposes_paragraph_flags_and_long_unicode_previews() {
    let dir = tempfile::tempdir().unwrap();
    let long = "é".repeat(81);
    let body = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:pageBreakBefore/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:br w:type="page"/><w:t>{long}</w:t></w:r></w:p>"#
    );
    std::fs::write(dir.path().join("input.docx"), docx(&body)).unwrap();
    let text = success(&run(dir.path(), &["inspect", "input.docx"]));
    assert!(text.contains("Heading1,numbered,page-break"), "{text}");
    assert!(text.contains(&format!("{}…", "é".repeat(80))), "{text}");
    assert!(!text.contains(&long));
    assert_eq!(
        success(&run(dir.path(), &["inspect", "input.docx", "--tables"])),
        "no tables\n"
    );
}
