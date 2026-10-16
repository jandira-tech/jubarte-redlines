// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! The view prints a Word highlight as `<mark>…</mark>`, so `{==…==}` only
//! ever marks a comment's scope. With `==` for both, a highlight at the
//! edge of a scope printed `{====lit==…==}` and Word got the literal text
//! `==lit==`; text ending in `=` beside a highlight printed `a===b==`.
//! Hand-written `==x==` still reads as a highlight.

mod common;

use common::docx::docx;
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

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

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    common::validity::assert_word_valid_package(&docx);
    docx
}

fn document(docx: &[u8]) -> String {
    common::docx::part_string(docx, "word/document.xml").unwrap()
}

/// The text of `document`'s runs that hold a highlight, in order.
fn highlighted(document: &str) -> String {
    document
        .split("</w:r>")
        .filter_map(|run| {
            let run = &run[run.rfind("<w:r>").or_else(|| run.rfind("<w:r "))?..];
            run.contains("<w:highlight").then(|| {
                run.split("<w:t")
                    .skip(1)
                    .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
                    .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
                    .collect::<String>()
            })
        })
        .collect()
}

/// All of `document`'s text.
fn text(document: &str) -> String {
    document
        .split("<w:t")
        .skip(1)
        .filter_map(|t| t.split_once('>').map(|(_, rest)| rest))
        .filter_map(|t| t.split_once("</w:t>").map(|(text, _)| text))
        .collect()
}

#[test]
fn a_highlight_prints_as_mark() {
    let body = "<w:p><w:r><w:t xml:space=\"preserve\">Plain </w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>lit</w:t></w:r></w:p>";
    let first = view(&docx(body));
    assert!(first.contains("Plain <mark>lit</mark>"), "{first}");
    assert!(!first.contains("=="), "{first}");
}

#[test]
fn a_highlight_at_a_comment_scopes_edge_comes_back_highlighted() {
    let source = write("A {==<mark>lit</mark> and plain==}{>>note<<} b.\n");
    let first = view(&source);
    assert!(
        first.contains("{==<mark>lit</mark> and plain==}"),
        "{first}"
    );
    let again = document(&write(&first));
    assert_eq!(highlighted(&again), "lit", "{first}\n{again}");
    assert!(!text(&again).contains("=="), "{again}");
    assert_eq!(view(&write(&first)), first);
}

#[test]
fn an_equals_sign_beside_a_highlight_stays_text() {
    let body = "<w:p><w:r><w:t>a=</w:t></w:r>\
                <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr><w:t>b</w:t></w:r>\
                <w:r><w:t>=c</w:t></w:r></w:p>";
    let first = view(&docx(body));
    let again = document(&write(&first));
    assert_eq!(highlighted(&again), "b", "{first}\n{again}");
    assert_eq!(text(&again), "a=b=c", "{first}");
}

#[test]
fn hand_written_equals_marks_still_highlight() {
    let again = document(&write("Some ==marked== text.\n"));
    assert_eq!(highlighted(&again), "marked");
}
