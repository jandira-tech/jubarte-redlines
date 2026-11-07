// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Another author's substitution nested in a change of the same kind keeps
//! both sides. The view prints that substitution, and writing it back lists
//! the same changes.

mod common;

use common::docx::{Part, docx, docx_with, docx_with_sect};
use jubarte::changes::list_logical_changes;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

const DATE: &str = "2026-10-01T09:00:00Z";

fn mark(kind: &str, id: u32, author: &str, inner: &str) -> String {
    format!(r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="{DATE}">{inner}</w:{kind}>"#)
}

fn text_run(kind: &str, text: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(r#"<w:r><w:{leaf}>{text}</w:{leaf}></w:r>"#)
}

/// `Keep `, then `outer` 7 holding `x`, A's deletion of `a` and A's
/// insertion of `b`, then `outer` 8 holding `y`, then ` tail.`.
fn paragraph(outer: &str) -> String {
    let nested = format!(
        "{}{}{}",
        text_run(outer, "x"),
        mark("del", 3, "A", &text_run("del", "a")),
        mark("ins", 4, "A", &text_run("ins", "b")),
    );
    format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Keep </w:t></w:r>{}{}<w:r><w:t xml:space="preserve"> tail.</w:t></w:r></w:p>"#,
        mark(outer, 7, "B", &nested),
        mark(outer, 8, "B", &text_run(outer, "y")),
    )
}

fn header_of(paragraph: &str) -> Vec<u8> {
    let xml = format!(
        r#"<w:hdr xmlns:w="{}">{paragraph}</w:hdr>"#,
        common::docx::W_NS
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &xml,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

fn footnote_of(paragraph: &str) -> Vec<u8> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="{}"><w:footnote w:id="1">{paragraph}</w:footnote></w:footnotes>"#,
        common::docx::W_NS
    );
    docx_with(
        r#"<w:p><w:r><w:t>Body</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &xml,
        }],
    )
}

fn view_of(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
    .lines()
    .filter(|line| !line.starts_with("sha256: ") && !line.starts_with("body: "))
    .collect::<Vec<_>>()
    .join("\n")
}

fn written(docx: &[u8]) -> (String, Vec<u8>, String) {
    let first = view_of(docx);
    let copy = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let second = view_of(&copy);
    (first, copy, second)
}

fn described(docx: &[u8]) -> Vec<String> {
    list_logical_changes(docx)
        .expect("the changes list")
        .into_iter()
        .map(|change| {
            let members: Vec<String> = change
                .members
                .iter()
                .map(|member| format!("{} {:?} {}", member.id, member.kind, member.text))
                .collect();
            format!(
                "{} {:?} {:?} inside {:?} [{members:?}]",
                change.id, change.kind, change.text, change.inside
            )
        })
        .collect()
}

/// The inserted side is the next piece, beside the outer mark, so that
/// mark's text is the text around it. The replacement itself is unchanged.
fn keeps_both_sides(docx: &[u8], span: &str, outer_kind: &str) {
    let (first, copy, second) = written(docx);
    assert!(
        first.contains(span),
        "the nested substitution keeps both sides:\n{first}"
    );
    assert_eq!(second, first, "the view comes back\n{first}\n---\n{second}");
    let source = described(docx);
    let back = described(&copy);
    assert_eq!(back.len(), source.len(), "{back:?}\n{source:?}");
    let replacement = |changes: &[String]| {
        changes
            .iter()
            .find(|change| change.contains("Replacement"))
            .cloned()
    };
    assert_eq!(replacement(&back), replacement(&source), "{first}");
    assert!(
        back.iter().all(|change| !change.contains("\"\"")),
        "no empty mark\n{back:?}"
    );
    let outer = |changes: &[String]| {
        changes
            .iter()
            .find(|change| !change.contains("Replacement"))
            .cloned()
            .expect("the outer change")
    };
    let written_outer = outer(&back);
    assert_eq!(
        written_outer.split_whitespace().next(),
        outer(&source).split_whitespace().next(),
        "the outer change keeps its id\n{written_outer}"
    );
    assert!(
        written_outer.contains(&format!("{outer_kind} \"xay\"")),
        "the outer text is the text beside the inserted side\n{written_outer}"
    );
    assert!(
        written_outer.contains(&format!("{outer_kind} xa\"")),
        "the first piece keeps the deleted side\n{written_outer}"
    );
    assert!(
        !written_outer.contains("xab"),
        "the inserted side stays beside the outer mark\n{written_outer}"
    );
    common::validity::assert_word_valid_package(&copy);
}

#[test]
fn a_substitution_nested_in_an_insertion_keeps_both_sides() {
    let paragraph = paragraph("ins");
    keeps_both_sides(
        &docx(&paragraph),
        "{~~a~>b~~}{r3+4 @A re r7+8 @B}",
        "Insertion",
    );
    keeps_both_sides(
        &header_of(&paragraph),
        "{~~a~>b~~}{header1.r3+4 @A re header1.r7+8 @B}",
        "Insertion",
    );
    keeps_both_sides(
        &footnote_of(&paragraph),
        "{~~a~>b~~}{footnotes.r3+4 @A re footnotes.r7+8 @B}",
        "Insertion",
    );
}

#[test]
fn a_substitution_nested_in_a_deletion_keeps_both_sides() {
    let paragraph = paragraph("del");
    keeps_both_sides(
        &docx(&paragraph),
        "{~~a~>b~~}{r3+4 @A re r7+8 @B}",
        "Deletion",
    );
    keeps_both_sides(
        &header_of(&paragraph),
        "{~~a~>b~~}{header1.r3+4 @A re header1.r7+8 @B}",
        "Deletion",
    );
    keeps_both_sides(
        &footnote_of(&paragraph),
        "{~~a~>b~~}{footnotes.r3+4 @A re footnotes.r7+8 @B}",
        "Deletion",
    );
}

/// A deletion alone inside another author's insertion stays that deletion.
/// It is not a substitution, and the outer change still names it.
#[test]
fn a_nested_deletion_of_another_author_stays_a_deletion() {
    let nested = format!(
        "{}{}",
        text_run("ins", "x"),
        mark("del", 3, "A", &text_run("del", "a")),
    );
    let paragraph = format!(
        r#"<w:p>{}{}</w:p>"#,
        mark("ins", 7, "B", &nested),
        mark("ins", 8, "B", &text_run("ins", "y")),
    );
    let (first, copy, second) = written(&docx(&paragraph));
    assert!(first.contains("{--a--}{r3 @A re r7+8 @B}"), "{first}");
    assert!(!first.contains("{~~"), "{first}");
    assert_eq!(second, first);
    assert_eq!(described(&copy), described(&docx(&paragraph)));
    common::validity::assert_word_valid_package(&copy);
}
