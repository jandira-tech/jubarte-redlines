// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A run's tracked bold survives `convert -t md` and back, and Reject All drops it.

mod common;

use common::docx::{docx, part_string};
use std::process::Command;

const AUTHOR: &str = "Ana Lee";
const DATE: &str = "2026-03-03T12:00:00Z";

fn revision(kind: &str, id: u32, text: &str) -> String {
    let node = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:{kind} w:id="{id}" w:author="{AUTHOR}" w:date="{DATE}"><w:r><w:{node} xml:space="preserve">{text}</w:{node}></w:r></w:{kind}>"#
    )
}

/// `See A **that** C today.`, with the bold on `that` recorded as a run change,
/// beside an insertion and a substitution.
fn source() -> Vec<u8> {
    let that = format!(
        r#"<w:r><w:rPr><w:b/><w:bCs/><w:rPrChange w:id="4" w:author="{AUTHOR}" w:date="{DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>that</w:t></w:r>"#
    );
    docx(&format!(
        "<w:p>{}{}{}{}{}{}</w:p>",
        r#"<w:r><w:t xml:space="preserve">See </w:t></w:r>"#,
        revision("ins", 1, "A "),
        that,
        revision("del", 2, "B"),
        revision("ins", 3, "C"),
        r#"<w:r><w:t xml:space="preserve"> today.</w:t></w:r>"#,
    ))
}

fn document(path: &std::path::Path) -> String {
    part_string(&std::fs::read(path).unwrap(), "word/document.xml").unwrap()
}

fn bold_marks(xml: &str) -> usize {
    xml.matches("<w:b/>").count() + xml.matches("<w:b ").count() + xml.matches("<w:bCs").count()
}

fn that_run_has_format_change(xml: &str) -> bool {
    let Some(text) = xml.find(">that<") else {
        return false;
    };
    let Some(run) = xml[..text].rfind("<w:r>") else {
        return false;
    };
    xml[run..=text].contains("<w:rPrChange")
}

fn run(args: &[&str], dir: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn convert_round_trip_keeps_a_run_formatting_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("anchor.docx"), source()).unwrap();
    let original = document(&dir.path().join("anchor.docx"));
    assert_eq!(original.matches("<w:rPrChange").count(), 1, "{original}");
    assert_eq!(original.matches("<w:pPrChange").count(), 0, "{original}");
    assert!(that_run_has_format_change(&original), "{original}");

    run(
        &["reject", "anchor.docx", "-o", "source-rejected.docx"],
        dir.path(),
    );
    let source_rejected = document(&dir.path().join("source-rejected.docx"));
    assert_eq!(bold_marks(&source_rejected), 0, "{source_rejected}");

    run(
        &[
            "convert",
            "anchor.docx",
            "-t",
            "md",
            "--no-page-markers",
            "-o",
            "anchor.md",
        ],
        dir.path(),
    );
    let markdown = std::fs::read_to_string(dir.path().join("anchor.md")).unwrap();
    assert!(markdown.contains("**that**"), "{markdown}");
    assert!(markdown.contains("fmt "), "{markdown}");

    run(
        &["convert", "anchor.md", "-t", "docx", "-o", "back.docx"],
        dir.path(),
    );
    let back = document(&dir.path().join("back.docx"));
    assert_eq!(
        back.matches("<w:rPrChange").count(),
        1,
        "the run change was dropped: {back}\n{markdown}"
    );
    assert_eq!(
        back.matches("<w:pPrChange").count(),
        0,
        "the run change became a paragraph change: {back}\n{markdown}"
    );
    assert!(
        that_run_has_format_change(&back),
        "the change is not on the bold run: {back}\n{markdown}"
    );

    run(
        &["reject", "back.docx", "-o", "back-rejected.docx"],
        dir.path(),
    );
    let rejected = document(&dir.path().join("back-rejected.docx"));
    assert_eq!(
        bold_marks(&rejected),
        0,
        "Reject All kept the bold: {rejected}\n{back}"
    );
}
