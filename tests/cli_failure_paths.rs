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

fn success(out: &Output) -> &str {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::str::from_utf8(&out.stdout).unwrap()
}

#[test]
fn convert_update_fields_reports_the_written_cache_and_refuses_clobbering() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    let args = [
        "convert",
        "input.docx",
        "-o",
        "updated.docx",
        "--update-fields",
    ];
    let out = run(
        dir.path(),
        &[args.as_slice(), &["--report", "fields.json"]].concat(),
    );
    let text = success(&out);
    assert!(
        text.contains("NUMPAGES") && text.contains("99") && text.contains('1'),
        "{text}"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("1 field(s) written; 1 page(s)"));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("fields.json")).unwrap()).unwrap();
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
    success(&run(dir.path(), &[args.as_slice(), &["--force"]].concat()));
    // With --track-changes the document is resolved first.
    success(&run(
        dir.path(),
        &[args.as_slice(), &["--force", "--track-changes", "accept"]].concat(),
    ));
    // The fields are written into a .docx; a PDF lays them out anyway.
    let pdf = run(
        dir.path(),
        &["convert", "input.docx", "-o", "out.pdf", "--update-fields"],
    );
    assert_eq!(pdf.status.code(), Some(2), "clap refuses it");
    assert!(
        String::from_utf8_lossy(&pdf.stderr).contains("--update-fields writes a .docx"),
        "{}",
        String::from_utf8_lossy(&pdf.stderr)
    );
    std::fs::write(dir.path().join("note.md"), "# Note\n").unwrap();
    let markdown = run(
        dir.path(),
        &["convert", "note.md", "-o", "note.docx", "--update-fields"],
    );
    assert_eq!(markdown.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&markdown.stderr).contains("needs a Word document in"),
        "{}",
        String::from_utf8_lossy(&markdown.stderr)
    );
    assert!(!dir.path().join("note.docx").exists());
    // `fields update` left the CLI for this flag.
    assert!(
        !run(
            dir.path(),
            &["fields", "update", "input.docx", "-o", "x.docx"]
        )
        .status
        .success()
    );
    assert_eq!(
        std::fs::read(dir.path().join("input.docx")).unwrap(),
        original
    );
}

#[test]
fn convert_update_fields_resolves_tracked_page_breaks_before_counting_pages() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(concat!(
        "<w:p><w:r><w:t>First page</w:t></w:r></w:p>",
        "<w:p><w:ins w:id=\"1\" w:author=\"Reviewer\" w:date=\"2026-01-01T00:00:00Z\">",
        "<w:r><w:br w:type=\"page\"/></w:r></w:ins></w:p>",
        "<w:p><w:fldSimple w:instr=\" NUMPAGES \"><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>",
    ));
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    for (mode, pages) in [("accept", 2), ("reject", 1)] {
        let output = format!("{mode}.docx");
        let report = format!("{mode}.json");
        success(&run(
            dir.path(),
            &[
                "convert",
                "input.docx",
                "-o",
                &output,
                "--update-fields",
                "--track-changes",
                mode,
                "--report",
                &report,
            ],
        ));
        let written = std::fs::read(dir.path().join(&output)).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.path().join(&report)).unwrap()).unwrap();
        assert_eq!(json["page_count"], pages, "{mode}: {json}");
        assert_eq!(json["fields"].as_array().unwrap().len(), 1);
        assert_eq!(json["fields"][0]["old"], "99");
        assert_eq!(json["fields"][0]["new"], pages.to_string());
        let xml = part_string(&written, "word/document.xml").unwrap();
        assert!(!xml.contains("<w:ins"), "{mode}: {xml}");
        assert!(xml.contains(&format!(">{pages}</w:t>")), "{mode}: {xml}");
    }
    assert_eq!(
        std::fs::read(dir.path().join("input.docx")).unwrap(),
        original
    );
}

#[test]
fn convert_update_fields_refuses_an_existing_report_without_force() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), docx(&para("Unchanged"))).unwrap();
    std::fs::write(dir.path().join("fields.json"), b"keep this report").unwrap();
    let args = [
        "convert",
        "input.docx",
        "-o",
        "out.docx",
        "--update-fields",
        "--report",
        "fields.json",
    ];
    let refused = run(dir.path(), &args);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("already exists"));
    assert_eq!(
        std::fs::read(dir.path().join("fields.json")).unwrap(),
        b"keep this report"
    );
    assert!(!dir.path().join("out.docx").exists());
}

#[test]
fn convert_update_fields_force_replaces_both_outputs_for_a_field_free_document() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), docx(&para("Unchanged"))).unwrap();
    std::fs::write(dir.path().join("out.docx"), b"old document").unwrap();
    std::fs::write(dir.path().join("fields.json"), b"old report").unwrap();
    success(&run(
        dir.path(),
        &[
            "convert",
            "input.docx",
            "-o",
            "out.docx",
            "--update-fields",
            "--report",
            "fields.json",
            "--force",
        ],
    ));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("fields.json")).unwrap()).unwrap();
    assert_eq!(report, serde_json::json!({"page_count": 1, "fields": []}));
    let written = std::fs::read(dir.path().join("out.docx")).unwrap();
    assert_eq!(
        jubarte::inspect::paragraphs(&written).unwrap()[0].text,
        "Unchanged"
    );
}

#[test]
fn convert_update_fields_report_write_failure_leaves_no_document() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(&para("Preserved source"));
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    let out = run(
        dir.path(),
        &[
            "convert",
            "input.docx",
            "-o",
            "out.docx",
            "--update-fields",
            "--report",
            "missing/fields.json",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    let error = String::from_utf8_lossy(&out.stderr);
    assert!(error.contains("writing missing/fields.json"), "{error}");
    assert!(!dir.path().join("out.docx").exists());
    assert_eq!(
        std::fs::read(dir.path().join("input.docx")).unwrap(),
        original
    );
}

#[test]
fn convert_update_fields_without_a_report_preserves_unresolved_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(concat!(
        "<w:p><w:ins w:id=\"1\" w:author=\"Reviewer\" w:date=\"2026-01-01T00:00:00Z\">",
        "<w:r><w:t>Keep the insertion</w:t></w:r></w:ins></w:p>",
        "<w:p><w:fldSimple w:instr=\" NUMPAGES \"><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>",
    ));
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    success(&run(
        dir.path(),
        &[
            "convert",
            "input.docx",
            "--to",
            "docx",
            "-o",
            "out.bin",
            "--update-fields",
        ],
    ));
    let written = std::fs::read(dir.path().join("out.bin")).unwrap();
    let xml = part_string(&written, "word/document.xml").unwrap();
    assert!(
        xml.contains("<w:ins ") && xml.contains("Keep the insertion"),
        "{xml}"
    );
    assert!(
        xml.contains("NUMPAGES") && xml.contains(">1</w:t>"),
        "{xml}"
    );
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
            "convert",
            "input.docx",
            "-o",
            "destination",
            "--to",
            "docx",
            "--update-fields",
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
fn convert_update_fields_refuses_a_report_on_the_input_or_the_output() {
    // CodeRabbit on #393: the report is written apart from the .docx, so a
    // shared path would silently replace one with the other.
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    let update = [
        "convert",
        "input.docx",
        "-o",
        "updated.docx",
        "--update-fields",
    ];
    for (report, force, says) in [
        ("input.docx", false, "the same file as the input"),
        ("input.docx", true, "the same file as the input"),
        ("updated.docx", false, "the same file as the Word output"),
    ] {
        let args = [
            update.as_slice(),
            &["--report", report],
            if force { &["--force"][..] } else { &[] },
        ]
        .concat();
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(&format!("--report '{report}' is {says}")),
            "{err}"
        );
        assert_eq!(
            std::fs::read(dir.path().join("input.docx")).unwrap(),
            original
        );
        assert!(!dir.path().join("updated.docx").exists());
    }
    // An existing report is kept without --force, and replaced with it.
    std::fs::write(dir.path().join("fields.json"), "keep").unwrap();
    let kept = run(
        dir.path(),
        &[update.as_slice(), &["--report", "fields.json"]].concat(),
    );
    assert_eq!(kept.status.code(), Some(1), "{kept:?}");
    assert!(String::from_utf8_lossy(&kept.stderr).contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("fields.json")).unwrap(),
        "keep"
    );
    assert!(!dir.path().join("updated.docx").exists());
    success(&run(
        dir.path(),
        &[update.as_slice(), &["--report", "fields.json", "--force"]].concat(),
    ));
    assert!(
        std::fs::read_to_string(dir.path().join("fields.json"))
            .unwrap()
            .contains("NUMPAGES")
    );
}

#[test]
fn convert_update_fields_refuses_a_report_that_names_the_input_another_way() {
    // A hard link, a symlink, and on a volume that ignores case a case
    // variant all name the input; writing the report through any of them
    // used to replace the input with JSON.
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    std::fs::hard_link(dir.path().join("input.docx"), dir.path().join("hard.json")).unwrap();
    let mut aliases = vec!["hard.json"];
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("input.docx", dir.path().join("link.json")).unwrap();
        aliases.push("link.json");
    }
    if dir.path().join("INPUT.DOCX").exists() {
        aliases.push("INPUT.DOCX");
    }
    for alias in aliases {
        let args = [
            "convert",
            "input.docx",
            "-o",
            "updated.docx",
            "--update-fields",
            "--report",
            alias,
            "--force",
        ];
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(&format!("--report '{alias}' is the same file as the input")),
            "{err}"
        );
        assert_eq!(
            std::fs::read(dir.path().join("input.docx")).unwrap(),
            original
        );
        assert!(!dir.path().join("updated.docx").exists());
    }
}

/// CodeRabbit on #398: an output that is a symlink to the report path, not
/// written yet, named the same file; the docx went through the link and the
/// report then replaced it.
#[cfg(unix)]
#[test]
fn convert_update_fields_refuses_a_report_a_dangling_output_link_names() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    std::os::unix::fs::symlink("fields.json", dir.path().join("updated.docx")).unwrap();
    std::os::unix::fs::symlink("../hop.docx", dir.path().join("sub/two.docx")).unwrap();
    std::os::unix::fs::symlink("fields.json", dir.path().join("hop.docx")).unwrap();
    for output in ["updated.docx", "sub/two.docx"] {
        let args = [
            "convert",
            "input.docx",
            "-o",
            output,
            "--update-fields",
            "--report",
            "fields.json",
            "--force",
        ];
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--report 'fields.json' is the same file as the Word output"),
            "{err}"
        );
        assert!(!dir.path().join("fields.json").exists());
    }
}

/// Neither file exists yet, so identity cannot tell `Out.docx` from
/// `out.docx`; on a volume that ignores case they are one file, and the
/// docx replaced the report written before it (pi review of #402).
#[test]
fn convert_update_fields_refuses_a_report_that_differs_from_the_output_by_case() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>99</w:t></w:r></w:fldSimple></w:p>"#,
    );
    std::fs::write(dir.path().join("input.docx"), &original).unwrap();
    let args = [
        "convert",
        "input.docx",
        "-o",
        "Out.docx",
        "--update-fields",
        "--report",
        "out.docx",
        "--force",
    ];
    let out = run(dir.path(), &args);
    assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains("--report 'out.docx' is the same file as the Word output"),
        "{out:?}"
    );
    assert!(!dir.path().join("Out.docx").exists());
    assert!(!dir.path().join("out.docx").exists());
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
        let out = run(dir.path(), &args);
        let text = success(&out);
        assert!(
            text.contains("Original clause") && text.contains("Revised clause"),
            "{text}"
        );
    }
    let out = run(
        dir.path(),
        &["debug", "old/same.docx", "old/same.docx", "--list"],
    );
    let equal = success(&out);
    assert!(equal.contains("0 changed"), "{equal}");
}

#[test]
fn read_json_exposes_paragraph_flags_and_long_unicode_text() {
    let dir = tempfile::tempdir().unwrap();
    let long = "é".repeat(81);
    let body = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:pageBreakBefore/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:br w:type="page"/><w:t>{long}</w:t></w:r></w:p>"#
    );
    std::fs::write(dir.path().join("input.docx"), docx(&body)).unwrap();
    let out = run(dir.path(), &["read", "input.docx", "--json"]);
    let json = success(&out);
    let snapshot: serde_json::Value = serde_json::from_str(json).unwrap();
    let paragraph = &snapshot["paragraphs"][0];
    assert_eq!(paragraph["style"], "Heading1", "{paragraph}");
    assert_eq!(paragraph["numbered"], true, "{paragraph}");
    assert_eq!(paragraph["page_break"], true, "{paragraph}");
    assert_eq!(paragraph["text"], long.as_str(), "{paragraph}");
    assert_eq!(
        success(&run(dir.path(), &["read", "input.docx", "--tables"])),
        "no tables\n"
    );
}

#[test]
fn layout_only_flags_refuse_word_and_markdown_before_any_output_write() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(&para("Owned original clause"));
    let revised = docx(&para("Owned revised clause"));
    std::fs::write(dir.path().join("original.docx"), &original).unwrap();
    std::fs::write(dir.path().join("revised.docx"), &revised).unwrap();
    for extension in ["docx", "md"] {
        for (flag, value) in [
            ("--report", Some("pages.json")),
            ("--font-report", Some("fonts.json")),
            ("--fail-on-substitution", None),
            ("--move-comments", None),
            ("--changed-pages", None),
            ("--pages", Some("1")),
            ("--dpi", Some("72")),
        ] {
            let destination = format!("converted.{extension}");
            let mut args = vec!["convert", "original.docx", "-o", &destination, flag];
            if let Some(value) = value {
                args.push(value);
            }
            let out = run(dir.path(), &args);
            assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
            assert!(
                String::from_utf8_lossy(&out.stderr)
                    .contains(&format!("{flag} applies to PDF or PNG output only")),
                "{out:?}"
            );
            assert!(!dir.path().join(&destination).exists());
            assert!(!dir.path().join("pages.json").exists());
            assert!(!dir.path().join("fonts.json").exists());
        }
        for extra in [
            &["--move-comments"][..],
            &["--changed-pages"],
            &["--pages", "1"],
            &["--revisions", "word"],
            &[
                "--revisions",
                "custom",
                "--revision-palette",
                "inserted=#0000FF",
            ],
        ] {
            let flag = extra[0];
            let destination = format!("compared.{extension}");
            let mut args = vec![
                "compare",
                "original.docx",
                "revised.docx",
                "-o",
                &destination,
            ];
            args.extend(extra);
            let out = run(dir.path(), &args);
            assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
            assert!(
                String::from_utf8_lossy(&out.stderr)
                    .contains(&format!("{flag} applies to PDF or PNG output only")),
                "{out:?}"
            );
            assert!(!dir.path().join(&destination).exists());
        }
    }
    // The retired spelling is refused under the canonical name.
    let out = run(
        dir.path(),
        &["convert", "original.docx", "-o", "c.docx", "--changed-only"],
    );
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains("--changed-pages applies to PDF or PNG output only"),
        "{out:?}"
    );
    for name in ["original.docx", "revised.docx"] {
        assert_eq!(
            &std::fs::read(dir.path().join(name)).unwrap(),
            if name == "original.docx" {
                &original
            } else {
                &revised
            }
        );
    }
}

#[test]
fn changes_json_lines_preserve_each_owned_revision_without_a_text_summary() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:r><w:t>Owned </w:t></w:r><w:del w:id="17" w:author="Original owner" w:date="2026-10-01T00:00:00Z"><w:r><w:delText>original</w:delText></w:r></w:del><w:ins w:id="23" w:author="Revised owner" w:date="2026-10-02T00:00:00Z"><w:r><w:t>revised</w:t></w:r></w:ins><w:r><w:t> clause</w:t></w:r></w:p>"#,
    );
    std::fs::write(dir.path().join("revisions.docx"), &original).unwrap();
    let expected = jubarte::changes::list_changes(&original).unwrap();
    assert_eq!(expected.len(), 2);
    let rows: Vec<serde_json::Value> =
        success(&run(dir.path(), &["changes", "revisions.docx", "--json"]))
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
    assert_eq!(rows[0]["author"], "Original owner");
    assert_eq!(rows[0]["text"], "original");
    assert_eq!(rows[1]["author"], "Revised owner");
    assert_eq!(rows[1]["text"], "revised");
    assert_eq!(
        rows,
        expected
            .iter()
            .map(|change| serde_json::to_value(change).unwrap())
            .collect::<Vec<_>>()
    );
    let out = run(dir.path(), &["changes", "revisions.docx"]);
    let text = success(&out);
    assert!(text.ends_with("2 change(s)\n"));
    assert!(text.contains("Original owner") && text.contains("Revised owner"));
    assert_eq!(
        std::fs::read(dir.path().join("revisions.docx")).unwrap(),
        original
    );
}

#[test]
fn png_only_edit_writes_owned_clean_and_redline_pages_without_pdf_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(&para("Owned original clause"));
    std::fs::write(dir.path().join("original.docx"), &original).unwrap();
    let plan = r#"{"schema_version":1,"author":"Page owner","date":"2026-10-01T00:00:00Z","operations":[{"id":"owned","kind":"replace","paragraph":"p0","find":"original","replacement":"revised"}]}"#;
    std::fs::write(dir.path().join("plan.json"), plan).unwrap();
    let out = run(
        dir.path(),
        &[
            "edit",
            "original.docx",
            "--plan",
            "plan.json",
            "--out-dir",
            "pages",
            "--png",
            "--dpi",
            "24",
        ],
    );
    success(&out);
    let rows: Vec<serde_json::Value> =
        std::fs::read_to_string(dir.path().join("pages/report.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
    let render = rows.iter().find(|row| row["ev"] == "render").unwrap();
    assert_eq!(render["pages"]["clean"], 1);
    assert_eq!(render["pages"]["redline"], 1);
    assert_eq!(render["page_starts"]["clean"][0], "Owned revised clause");
    for stem in ["clean", "redline"] {
        assert!(!dir.path().join(format!("pages/{stem}.pdf")).exists());
        let png = std::fs::read(dir.path().join(format!("pages/{stem}-page-01.png"))).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(image::load_from_memory(&png).unwrap().width(), 204);
        let bytes = std::fs::read(dir.path().join(format!("pages/{stem}.docx"))).unwrap();
        let projected = jubarte::document_comparer::accept_revisions(&bytes).unwrap();
        let xml = part_string(&projected, "word/document.xml").unwrap();
        let mut dom = jubarte::xmllinq::Dom::new();
        let document = dom.parse_xdocument(&xml);
        let text: String = dom
            .descendants(
                dom.root(document).unwrap(),
                Some(&jubarte::namespaces::W::t()),
            )
            .into_iter()
            .map(|leaf| dom.value(leaf))
            .collect();
        assert_eq!(text, "Owned revised clause");
    }
    assert_eq!(
        std::fs::read(dir.path().join("original.docx")).unwrap(),
        original
    );
}

#[test]
fn png_conversion_projects_requested_revision_side_before_its_page_report() {
    let dir = tempfile::tempdir().unwrap();
    let original = docx(
        r#"<w:p><w:r><w:t xml:space="preserve">Owned </w:t></w:r><w:del w:id="17" w:author="Original owner" w:date="2026-10-01T00:00:00Z"><w:r><w:delText>original</w:delText></w:r></w:del><w:ins w:id="23" w:author="Revised owner" w:date="2026-10-02T00:00:00Z"><w:r><w:t>revised</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> clause</w:t></w:r></w:p>"#,
    );
    std::fs::write(dir.path().join("revisions.docx"), &original).unwrap();
    for (side, expected) in [
        ("accept", "Owned revised clause"),
        ("reject", "Owned original clause"),
    ] {
        let output = format!("{side}.png");
        let report = format!("{side}.json");
        let mut args = vec![
            "convert",
            "revisions.docx",
            "--to",
            "png",
            "--track-changes",
            side,
            "-o",
            &output,
            "--report",
            &report,
            "--dpi",
            "24",
        ];
        if side == "accept" {
            args.push("--png");
        }
        success(&run(dir.path(), &args));
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.path().join(&report)).unwrap()).unwrap();
        assert_eq!(report["page_count"], 1);
        assert_eq!(
            report["pages"][0]["text"].as_str().unwrap().trim(),
            expected
        );
        let png = std::fs::read(dir.path().join(format!("{side}-page-01.png"))).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(image::load_from_memory(&png).unwrap().width(), 204);
        assert!(!dir.path().join(format!("{side}.pdf")).exists());
    }
    assert_eq!(
        std::fs::read(dir.path().join("revisions.docx")).unwrap(),
        original
    );
}

#[test]
fn read_refuses_an_expired_selection_as_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("input.docx"),
        docx("<w:p><w:r><w:t>One</w:t></w:r></w:p>"),
    )
    .unwrap();
    let out = run(dir.path(), &["read", "input.docx", "-p", "12"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the settled spelling is p12"), "{stderr}");
}

/// CodeRabbit on #438: a side file named like one of the PNG pages would
/// replace that page; it is refused before anything is written.
#[test]
fn a_report_named_like_a_png_page_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), docx(&para("One page."))).unwrap();
    for flag in ["--report", "--font-report"] {
        let args = [
            "convert",
            "input.docx",
            "-o",
            "result.png",
            flag,
            "result-page-01.png",
        ];
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(&format!(
                "{flag} 'result-page-01.png' is the same file as the PNG output"
            )),
            "{err}"
        );
        assert!(!dir.path().join("result-page-01.png").exists());
    }
}

/// Codex on #442, decided in its thread: an -o extension jubarte does not
/// know is written as a .docx, as compare's writer does; only the known
/// non-Word ones exit 2.
#[test]
fn edit_writes_a_docx_under_an_unknown_extension() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.docx"), docx(&para("One."))).unwrap();
    let out = run(
        dir.path(),
        &[
            "add",
            "input.docx",
            "-p",
            "p0",
            "--comment",
            "--content",
            "Note.",
            "-o",
            "result.zip",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let written = std::fs::read(dir.path().join("result.zip")).unwrap();
    assert!(part_string(&written, "word/document.xml").is_some());
    let refused = run(
        dir.path(),
        &[
            "add",
            "input.docx",
            "-p",
            "p0",
            "--comment",
            "--content",
            "Note.",
            "-o",
            "result.pdf",
        ],
    );
    assert_eq!(refused.status.code(), Some(2), "{refused:?}");
}

/// A Word input that is not a ZIP package reads the same on every task: the
/// code, the file, and what it is instead (ledger review of #454, finding 3).
#[test]
fn a_word_input_that_is_not_a_zip_reads_the_same_on_every_task() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fake.docx"), b"hello, not a zip").unwrap();
    std::fs::write(dir.path().join("good.docx"), docx(&para("Plain"))).unwrap();
    let expected =
        "error: INVALID_PACKAGE: fake.docx is not a ZIP file: no end-of-central-directory record\n";
    for args in [
        &["read", "fake.docx"][..],
        &["read", "fake.docx", "--json"],
        &["read", "fake.docx", "--tables"],
        &["read", "fake.docx", "-o", "v.json"],
        &["changes", "fake.docx"],
        &["comments", "fake.docx"],
        &["accept", "fake.docx", "-o", "a.docx"],
        &["reject", "fake.docx", "-o", "a.docx"],
        &["scrub", "fake.docx", "-o", "a.docx"],
        &["convert", "fake.docx", "-o", "a.pdf"],
        &["convert", "fake.docx", "-o", "a.md"],
        &["compare", "fake.docx", "good.docx"],
        &["compare", "good.docx", "fake.docx", "-o", "c.docx"],
        &["diff", "fake.docx", "good.docx"],
        &["edit", "fake.docx", "-p", "p0", "--delete"],
        &["add", "fake.docx", "-p", "p0", "--content", "x"],
        &["append", "good.docx", "fake.docx", "-o", "j.docx"],
        &["visual-diff", "fake.docx", "good.docx"],
    ] {
        let out = run(dir.path(), args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), expected, "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
    }
}
