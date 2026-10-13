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
