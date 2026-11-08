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

/// A view's YAML header: one author, one page.
const HEADER: &str = "---
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 1                       # 1 formatting
authors:
  document_owner: none             # no docProps/core.xml
  AL: Ann Lee                      # 1 revision, 2026-03-02T10:00:00Z
body: p0-p0, 0 tables, 1 page
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
---

";

/// `body` (id lines and text) under [`HEADER`], converted to Word.
fn markdown_to_document(body: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.md"), format!("{HEADER}{body}")).unwrap();
    run(
        &["convert", "in.md", "-t", "docx", "-o", "out.docx"],
        dir.path(),
    );
    document(&dir.path().join("out.docx"))
}

#[test]
fn a_list_label_keeps_its_fmt_on_the_paragraph() {
    // `num "(a)"` is the paragraph's own numbering: its `fmt` is that
    // property's change, not the bold word's (#2116 review).
    let xml = markdown_to_document("<!-- p0 num \"(a)\", fmt r10 @AL -->\nItem **bold** here.\n");
    assert_eq!(xml.matches("<w:rPrChange").count(), 0, "{xml}");
    assert_eq!(xml.matches("<w:pPrChange").count(), 1, "{xml}");
}

#[test]
fn bold_math_does_not_swallow_the_paragraph_fmt() {
    // Math is written outside any run: it cannot hold a run change, so
    // the `fmt` stays the paragraph's (#2116 review).
    let xml = markdown_to_document("<!-- p0 fmt r10 @AL -->\nSee **$x$** here.\n");
    assert_eq!(
        xml.matches("<w:rPrChange").count() + xml.matches("<w:pPrChange").count(),
        1,
        "the fmt was dropped: {xml}"
    );
}

#[test]
fn a_bold_field_keeps_one_run_change_on_its_result() {
    // A field's begin, code and separate runs share one property string:
    // the change goes on the result text, once (#2116 review).
    let xml = markdown_to_document("<!-- p0 fmt r10 @AL -->\nPage **{PAGE}1{/}** here.\n");
    assert_eq!(xml.matches("<w:rPrChange").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:pPrChange").count(), 0, "{xml}");
    let result = xml.find(">1</w:t>").expect("field result");
    let run = xml[..result].rfind("<w:r>").unwrap();
    assert!(xml[run..result].contains("<w:rPrChange"), "{xml}");
}

#[test]
fn a_text_box_before_the_bold_run_does_not_take_its_change() {
    // The box's own paragraphs open while the host's is open: the host's
    // pending change must survive them (#2116 review).
    let dir = tempfile::tempdir().unwrap();
    let that = format!(
        r#"<w:r><w:rPr><w:b/><w:bCs/><w:rPrChange w:id="4" w:author="{AUTHOR}" w:date="{DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>that</w:t></w:r>"#
    );
    let source = docx(&format!(
        "<w:p>{}{}{}{}</w:p>",
        r#"<w:r><w:t xml:space="preserve">See </w:t></w:r>"#,
        r#"<w:r><w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shape style="width:100pt;height:40pt"><v:textbox><w:txbxContent><w:p><w:r><w:t>Box</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r>"#,
        that,
        r#"<w:r><w:t xml:space="preserve"> today.</w:t></w:r>"#,
    ));
    std::fs::write(dir.path().join("boxed.docx"), source).unwrap();
    run(
        &["convert", "boxed.docx", "-t", "md", "-o", "boxed.md"],
        dir.path(),
    );
    let markdown = std::fs::read_to_string(dir.path().join("boxed.md")).unwrap();
    run(
        &["convert", "boxed.md", "-t", "docx", "-o", "back.docx"],
        dir.path(),
    );
    let back = document(&dir.path().join("back.docx"));
    assert_eq!(
        back.matches("<w:rPrChange").count() + back.matches("<w:pPrChange").count(),
        1,
        "the change was dropped: {back}\n{markdown}"
    );
    assert!(
        that_run_has_format_change(&back),
        "the change is not on the bold run: {back}\n{markdown}"
    );
}
