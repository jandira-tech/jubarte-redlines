// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A bound `mc:Choice` whose Requires names a namespace Word does not know
//! is not the active branch. The fallback is.

mod common;

use jubarte::markdown::{ReadOptions, read};
use jubarte::opc::PartFs;

const WPS: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";
const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
const UNKNOWN: &str = "urn:example:unknown";

fn box_drawing(story: &str) -> String {
    format!(
        r#"<w:drawing><wp:anchor xmlns:wp="{WP}"><wps:wsp xmlns:wps="{WPS}"><wps:txbx><w:txbxContent>{story}</w:txbxContent></wps:txbx></wps:wsp></wp:anchor></w:drawing>"#
    )
}

/// Outer text box. `choice` and `fallback` are the branches inside its story.
/// `prefix` is bound on the alternate to `xmlns`.
fn nested_choice(prefix: &str, xmlns: &str, choice: &str, fallback: &str) -> String {
    let story = format!(
        r#"<w:p><w:r><mc:AlternateContent xmlns:{prefix}="{xmlns}"><mc:Choice Requires="{prefix}">{choice}</mc:Choice><mc:Fallback>{fallback}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#
    );
    format!("<w:p><w:r>{}</w:r></w:p>", box_drawing(&story))
}

fn package(body: &str) -> Vec<u8> {
    common::docx::docx_with(body, &[])
}

fn nested_findings(body: &str) -> Vec<jubarte::validate::Finding> {
    let docx = package(body);
    jubarte::validate::ring1(&PartFs::open(&docx).expect("package opens"))
        .into_iter()
        .filter(|finding| finding.code == "NESTED_TEXT_BOX")
        .collect()
}

fn markdown(body: &str) -> String {
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    read(&package(body), &options)
        .expect("the docx reads")
        .markdown
}

/// Visible text-box stories, not the drawing payload that keeps both branches.
fn shown_stories(markdown: &str) -> String {
    let mut out = String::new();
    for part in markdown.split("<!-- text-box").skip(1) {
        let body = part.split("<!-- /text-box -->").next().unwrap_or("");
        out.push_str(body);
    }
    out
}

#[test]
fn an_unknown_requires_choice_is_not_a_nested_text_box() {
    let nested = box_drawing("<w:p><w:r><w:t>Choice words</w:t></w:r></w:p>");
    let body = nested_choice("cx", UNKNOWN, &nested, "<w:t>Fallback words</w:t>");
    let findings = nested_findings(&body);
    assert!(
        findings.is_empty(),
        "unknown choice nested text box was reported: {findings:?}"
    );
}

#[test]
fn a_nested_text_box_in_the_fallback_of_an_unknown_choice_is_reported() {
    let nested = box_drawing("<w:p><w:r><w:t>Fallback words</w:t></w:r></w:p>");
    let body = nested_choice("cx", UNKNOWN, "<w:t>Choice words</w:t>", &nested);
    let findings = nested_findings(&body);
    assert_eq!(
        findings.len(),
        1,
        "a nested text box in the fallback was missed: {findings:?}"
    );
}

#[test]
fn a_requires_namespace_word_knows_still_reports_the_choice() {
    let nested = box_drawing("<w:p><w:r><w:t>Choice words</w:t></w:r></w:p>");
    let body = nested_choice("wps", WPS, &nested, "<w:t>Fallback words</w:t>");
    let findings = nested_findings(&body);
    assert_eq!(
        findings.len(),
        1,
        "a choice Word understands dropped its nested text box: {findings:?}"
    );
}

#[test]
fn the_reader_shows_the_fallback_when_requires_is_unknown() {
    let nested = box_drawing("<w:p><w:r><w:t>Choice words</w:t></w:r></w:p>");
    let fallback = box_drawing("<w:p><w:r><w:t>Fallback words</w:t></w:r></w:p>");
    let body = nested_choice("cx", UNKNOWN, &nested, &fallback);
    let view = markdown(&body);
    let shown = shown_stories(&view);
    assert!(
        shown.contains("Fallback words"),
        "the reader showed the choice instead of the fallback: {shown}"
    );
    assert!(
        !shown.contains("Choice words"),
        "the reader kept the unknown choice: {shown}"
    );
}
