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
            ("--changed-only", None),
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
            &["--changed-only"],
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
            let mut args = vec!["diff", "original.docx", "revised.docx", "-o", &destination];
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
    let text = success(&run(dir.path(), &["changes", "revisions.docx"]));
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
    let plan = r#"{"schema_version":1,"author":"Page owner","date":"2026-10-01T00:00:00Z","operations":[{"id":"owned","kind":"replace","paragraph":"body:p:0","find":"original","replacement":"revised"}]}"#;
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
