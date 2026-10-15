// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A table the view gives no style takes Word's thin grid (Table Grid:
//! single 0.5pt borders all round, as Word's Insert Table makes): Demo.docx's
//! table, which has its own thin borders and no style, came back with none
//! (Arthur, 2026-10-15). A table that has no borders says so in the view
//! (`TableNormal`), so it stays without.

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

/// The view of `docx`, the docx it writes, and that docx's view.
fn round_trip(docx: &[u8]) -> (String, Vec<u8>, String) {
    let first = view(docx);
    let again = write(&first);
    let second = view(&again);
    (first, again, second)
}

fn part(docx: &[u8], name: &str) -> String {
    common::docx::part_string(docx, name).unwrap_or_else(|| panic!("{name}"))
}

/// The `w:tblStyle` of each table in `document`, in order.
fn table_styles(document: &str) -> Vec<Option<String>> {
    document
        .split("<w:tbl>")
        .skip(1)
        .map(|table| {
            let props = &table[..table.find("</w:tblPr>").unwrap_or(0)];
            props
                .split("<w:tblStyle w:val=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .map(str::to_string)
        })
        .collect()
}

/// Whether `styles` defines Table Grid with Word's thin borders.
fn thin_grid(styles: &str) -> bool {
    let Some(at) = styles.find("w:styleId=\"TableGrid\"") else {
        return false;
    };
    let style = &styles[at..at + styles[at..].find("</w:style>").unwrap_or(0)];
    ["top", "left", "bottom", "right", "insideH", "insideV"]
        .iter()
        .all(|side| style.contains(&format!("<w:{side} w:val=\"single\" w:sz=\"4\"")))
}

fn table(props: &str) -> String {
    format!(
        "<w:tbl><w:tblPr>{props}</w:tblPr><w:tblGrid><w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:p/>"
    )
}

const DIRECT_BORDERS: &str = "<w:tblBorders>\
    <w:top w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:left w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    <w:bottom w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:right w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    <w:insideH w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:insideV w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    </w:tblBorders>";

#[test]
fn a_markdown_table_takes_words_thin_grid() {
    let written = write("| A | B |\n|---|---|\n| C | D |\n");
    assert_eq!(
        table_styles(&part(&written, "word/document.xml")),
        [Some("TableGrid".to_string())]
    );
    assert!(thin_grid(&part(&written, "word/styles.xml")));
}

#[test]
fn a_table_with_its_own_thin_borders_comes_back_with_borders() {
    // Demo.docx's table: no style, direct single 0.5pt borders.
    let (first, again, second) = round_trip(&docx(&table(DIRECT_BORDERS)));
    assert!(!first.contains("table:"), "{first}");
    assert_eq!(
        table_styles(&part(&again, "word/document.xml")),
        [Some("TableGrid".to_string())]
    );
    assert!(thin_grid(&part(&again, "word/styles.xml")));
    assert_eq!(second, first);
}

#[test]
fn a_table_without_borders_stays_without() {
    let (first, again, second) = round_trip(&docx(&table("")));
    assert!(first.contains("  table: TableNormal"), "{first}");
    // No tblStyle is Word's Normal Table, as the source had it.
    let document = part(&again, "word/document.xml");
    assert_eq!(table_styles(&document), [None]);
    assert!(!document.contains("tblBorders"), "{document}");
    assert_eq!(second, first);
}

#[test]
fn a_table_grid_table_needs_no_legend() {
    let (first, again, second) = round_trip(&docx(&table("<w:tblStyle w:val=\"TableGrid\"/>")));
    assert!(!first.contains("table:"), "{first}");
    assert_eq!(
        table_styles(&part(&again, "word/document.xml")),
        [Some("TableGrid".to_string())]
    );
    assert_eq!(second, first);
}

#[test]
fn a_grid_and_a_borderless_table_keep_their_own() {
    let body = format!(
        "{}<w:p/>{}",
        table("<w:tblStyle w:val=\"TableGrid\"/>"),
        table("")
    );
    let (first, again, second) = round_trip(&docx(&body));
    assert!(
        first.contains("<!-- t1 2x2, cells p6-p9 by row, style TableNormal -->"),
        "{first}"
    );
    assert!(
        first.contains("<!-- t0 2x2, cells p0-p3 by row -->"),
        "{first}"
    );
    assert_eq!(
        table_styles(&part(&again, "word/document.xml")),
        [Some("TableGrid".to_string()), None]
    );
    assert_eq!(second, first);
}
