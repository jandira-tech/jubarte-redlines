// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A header's or footer's empty paragraphs after its text are lines Word
//! lays out (the header grows by each), so the view keeps them:
//! `text: [My header, ""]`. ids-first-page.md (file_202.docx) lost one.

mod common;

use common::docx::{Part, docx_with_sect};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const W: &str = common::docx::W_NS;

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        .filter(|l| !l.starts_with("sha256: ") && !l.starts_with("body: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A docx whose default header holds `paragraphs`, each in Header style.
fn with_header(paragraphs: &[&str]) -> Vec<u8> {
    let body: String = paragraphs
        .iter()
        .map(|text| {
            let run = if text.is_empty() {
                String::new()
            } else {
                format!("<w:r><w:t>{text}</w:t></w:r>")
            };
            format!(r#"<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr>{run}</w:p>"#)
        })
        .collect();
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W}">{body}</w:hdr>"#
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// The view of `docx`, the paragraphs of the header it writes back, and
/// that docx's view.
fn round_trip(docx: &[u8]) -> (String, usize, String) {
    let first = view(docx);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let header = common::docx::part_string(&again, "word/header1.xml").expect("header1.xml");
    let paragraphs = header.matches("<w:p>").count() + header.matches("<w:p ").count();
    (first, paragraphs, view(&again))
}

fn header_line(view: &str) -> &str {
    view.lines()
        .find(|l| l.contains("id: header1"))
        .unwrap_or_else(|| panic!("no header line: {view}"))
}

#[test]
fn an_empty_paragraph_after_the_text_comes_back() {
    let (first, paragraphs, second) = round_trip(&with_header(&["My header", ""]));
    assert!(
        header_line(&first).contains(r#"text: [My header, """#),
        "{first}"
    );
    assert_eq!(paragraphs, 2, "{first}");
    assert_eq!(second, first);
}

#[test]
fn empty_paragraphs_around_the_text_come_back() {
    let (first, paragraphs, second) = round_trip(&with_header(&["", "Middle", "", ""]));
    assert_eq!(paragraphs, 4, "{first}");
    assert_eq!(second, first);
}

#[test]
fn a_header_of_empty_paragraphs_keeps_them() {
    let (first, paragraphs, second) = round_trip(&with_header(&["", ""]));
    assert_eq!(paragraphs, 2, "{first}");
    assert_eq!(second, first);
}

#[test]
fn a_one_paragraph_header_stays_a_scalar() {
    let (first, paragraphs, second) = round_trip(&with_header(&["My header"]));
    assert!(header_line(&first).contains("text: My header"), "{first}");
    assert_eq!(paragraphs, 1, "{first}");
    assert_eq!(second, first);
}
