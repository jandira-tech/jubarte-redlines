// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The reader brings back what the agent view's id lines say about a
//! paragraph (`<!-- p2-p4 empty -->`, `<!-- p5 TitlePage center -->`), so a
//! view read into a docx and read again prints the same ids.

mod common;

use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

/// The view of the docx `markdown` reads into, without its YAML header.
fn round_trip(markdown: &str) -> String {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let view = read(
        &docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown;
    let body = view.split_once("\n---\n").map_or(view.as_str(), |(_, b)| b);
    body.trim().to_string()
}

#[test]
fn empty_paragraphs_come_back_from_their_id_lines() {
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 empty -->\n\n<!-- p2 -->\nTwo\n\n\
              <!-- p3-p5 empty -->\n\n<!-- p6 -->\nThree\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn an_id_line_alone_makes_no_paragraph() {
    // Only `empty` stands for paragraphs: a plain id line names the text
    // after it.
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 -->\nTwo\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn an_id_line_style_comes_back_on_its_paragraph() {
    let md = "<!-- p0 DocumentTitle -->\nLast Will\n\n<!-- p1 -->\nBody text.\n\n\
              <!-- p2 TitlePage -->\nOf Ann Counsel\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn id_line_alignment_and_indents_come_back() {
    let md = "<!-- p0 BodyText center, first-line 0.5in, left 0.25in, right 1in -->\nCentered\n\n\
              <!-- p1 justify, hanging 0.3in, left 0.3in -->\nJustified\n\n\
              <!-- p2 right -->\nRight\n\n\
              <!-- p3 left -->\nLeft\n\n\
              <!-- p4 left -0.25in -->\nOutdented\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_heading_keeps_its_level_under_the_style_its_id_line_names() {
    let md = "<!-- p0 DocumentTitle -->\n# Last Will\n\n<!-- p1 TextHeading2 center -->\n## Article One\n\n\
              <!-- p2 -->\nBody text.\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_paragraph_without_text_comes_back_from_its_id_line() {
    // (A textless paragraph with nothing but a style prints `empty`.)
    let md = "<!-- p0 -->\nOne\n\n<!-- p1 page-break -->\n\n<!-- p2 empty -->\n\n\
              <!-- p3 -->\nTwo\n\n<!-- p4 page-break -->\n";
    assert_eq!(round_trip(md), md.trim());
}

#[test]
fn a_page_break_comes_back_after_its_text() {
    let md = "<!-- p0 page-break -->\nBefore the break\n\n<!-- p1 -->\nAfter it\n";
    assert_eq!(round_trip(md), md.trim());
}

const AUTHORS: &str =
    "---\nauthors:\n  AC: Ann Counsel                  # 2026-10-01T09:00:00Z\n---\n\n";

#[test]
fn revisions_keep_the_numbers_their_tags_name() {
    let body = "<!-- p0 -->\nPay {~~thirty~>forty~~}{>>r7+9 @AC<<} days {++from today++}{>>r12 @AC<<}.\n\n\
                <!-- p1 -->\n{--Gone.--}{>>r3 @AC<<} Kept.\n";
    assert_eq!(round_trip(&format!("{AUTHORS}{body}")), body.trim());
}

#[test]
fn a_revision_number_is_written_once() {
    // A change written as several marks (two runs, or a view that names one
    // number for two changes) keeps its number on the first mark; the rest
    // take fresh ones, so no two marks share a `w:id`.
    let body = "<!-- p0 -->\n{++**Bold** plain++}{>>r4 @AC<<} and {++one++}{>>r5 @AC<<} \
                {++two++}{>>r5 @AC<<}.\n";
    let docx = markdown_to_docx(&format!("{AUTHORS}{body}"), &DocxOptions::default())
        .expect("the view reads")
        .docx;
    assert_word_valid_package(&docx);
    let document = common::docx::part_string(&docx, "word/document.xml").expect("document.xml");
    for id in ["4", "5"] {
        assert!(
            document.contains(&format!("w:ins w:id=\"{id}\"")),
            "w:id {id} kept: {document}"
        );
    }
}
