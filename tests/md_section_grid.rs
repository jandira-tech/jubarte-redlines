// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A section's column line (`w:cols w:sep`) and line grid (`w:docGrid`)
//! come back. Demo.docx's second section draws a line between its two
//! columns and both sections snap their lines to a 15.6pt grid; the round
//! trip dropped both, so the columns lost their divider and every line moved.

mod common;

use common::docx::{docx_with_sect_pr, part_string};
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
        .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
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

const PAGE: &str = r#"<w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="851" w:footer="992" w:gutter="0"/>"#;

/// A two-section docx: section 1 (one paragraph) ends with `first`, the
/// body's final section is `last`; each is the tail after the page.
fn two_sections(first: &str, last: &str) -> Vec<u8> {
    let body = format!(
        "<w:p><w:pPr><w:sectPr>{PAGE}{first}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>"
    );
    docx_with_sect_pr(&body, &[], &format!("<w:sectPr>{PAGE}{last}</w:sectPr>"))
}

/// The `w:sectPr`s of `docx`'s document, in order.
fn sections(docx: &[u8]) -> Vec<String> {
    let document = part_string(docx, "word/document.xml").unwrap();
    document
        .split("<w:sectPr")
        .skip(1)
        .map(|s| s.split("</w:sectPr>").next().unwrap().to_string())
        .collect()
}

#[test]
fn a_column_line_comes_back() {
    let source = two_sections(
        r#"<w:cols w:space="720"/>"#,
        r#"<w:cols w:space="360" w:num="2" w:sep="1"/>"#,
    );
    let first = view(&source);
    assert!(
        first.contains("columns 2, column gap 0.25in, column line"),
        "{first}"
    );
    let again = write(&first);
    let sects = sections(&again);
    assert!(sects[1].contains(r#"w:num="2""#), "{sects:?}");
    assert!(sects[1].contains(r#"w:sep="1""#), "{sects:?}");
    assert!(!sects[0].contains("w:sep"), "{sects:?}");
    assert_eq!(view(&again), first);
}

#[test]
fn a_line_grid_comes_back_in_every_section() {
    let grid = r#"<w:docGrid w:linePitch="312" w:charSpace="0" w:type="lines"/>"#;
    let source = two_sections(
        &format!(r#"<w:cols w:space="720"/>{grid}"#),
        &format!(r#"<w:cols w:space="360" w:num="2"/>{grid}"#),
    );
    let first = view(&source);
    assert!(first.contains("line grid 15.6pt"), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(
            sect.contains(r#"<w:docGrid w:type="lines" w:linePitch="312"/>"#),
            "{sect}"
        );
    }
    assert_eq!(view(&again), first);
}

#[test]
fn a_grid_word_does_not_snap_to_is_not_printed() {
    // Word's own template writes `w:linePitch` with no type: no grid.
    let source = two_sections(
        r#"<w:cols w:space="720"/><w:docGrid w:linePitch="360"/>"#,
        r#"<w:cols w:space="720"/><w:docGrid w:linePitch="360"/>"#,
    );
    let first = view(&source);
    assert!(!first.contains("grid"), "{first}");
    assert!(!first.contains("column line"), "{first}");
}

#[test]
fn a_character_grid_keeps_its_pitch_and_spacing() {
    let grid = r#"<w:docGrid w:type="linesAndChars" w:linePitch="360" w:charSpace="4096"/>"#;
    let source = two_sections(grid, grid);
    let first = view(&source);
    assert!(
        first.contains("line and char grid 18pt, char space 4096"),
        "{first}"
    );
    let again = write(&first);
    for sect in sections(&again) {
        assert!(
            sect.contains(
                r#"<w:docGrid w:type="linesAndChars" w:linePitch="360" w:charSpace="4096"/>"#
            ),
            "{sect}"
        );
    }
}

#[test]
fn a_character_grid_without_a_line_pitch_comes_back() {
    // `tests/convert_docx_to_pdf.rs`'s character grid: `snapToChars` sets
    // only character spacing, so `w:linePitch` is absent.
    let grid = r#"<w:docGrid w:type="snapToChars" w:charSpace="40960"/>"#;
    let source = two_sections(grid, grid);
    let first = view(&source);
    assert!(first.contains("char grid, char space 40960"), "{first}");
    let again = write(&first);
    for sect in sections(&again) {
        assert!(sect.contains(grid), "{sect}");
    }
    assert_eq!(view(&again), first);
}
