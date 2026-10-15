// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Tables keep the borders Word passes (Arthur, 2026-10-15: "whatever word
//! passes, we render"). A table with no style is Word's Normal Table
//! (`style TableNormal`, no `tblStyle`), and the borders it carries itself
//! (`tblBorders`, each cell's `tcBorders`) print on its line and come back
//! as they were: Demo.docx's thin black grid, 25f1d6ecc3's in automatic
//! colour, 5331b981b7's 0.25pt table with 1pt grey cells. Only a table that
//! names nothing, a plain Markdown table, takes Word's thin grid (Table
//! Grid, as Insert Table makes it).

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
    table_with(props, &["", "", "", ""])
}

/// A 2x2 table with `props` in its `tblPr` and each cell's `tcPr` content
/// from `cells`, row by row.
fn table_with(props: &str, cells: &[&str; 4]) -> String {
    let cell = |i: usize, text: &str| {
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"4000\" w:type=\"dxa\"/>{}</w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>",
            cells[i]
        )
    };
    format!(
        "<w:tbl><w:tblPr>{props}</w:tblPr><w:tblGrid><w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl><w:p/>",
        cell(0, "A"),
        cell(1, "B"),
        cell(2, "C"),
        cell(3, "D")
    )
}

/// The `tblBorders` of each table in `document`, in order.
fn table_borders(document: &str) -> Vec<Option<String>> {
    document
        .split("<w:tbl>")
        .skip(1)
        .map(|table| {
            let props = &table[..table.find("</w:tblPr>").unwrap_or(0)];
            props
                .split_once("<w:tblBorders>")
                .and_then(|(_, rest)| rest.split_once("</w:tblBorders>"))
                .map(|(inner, _)| inner.to_string())
        })
        .collect()
}

/// Each cell's `tcBorders` in `document`, in order.
fn cell_borders(document: &str) -> Vec<Option<String>> {
    document
        .split("<w:tcPr>")
        .skip(1)
        .map(|props| {
            let props = &props[..props.find("</w:tcPr>").unwrap_or(0)];
            props
                .split_once("<w:tcBorders>")
                .and_then(|(_, rest)| rest.split_once("</w:tcBorders>"))
                .map(|(inner, _)| inner.to_string())
        })
        .collect()
}

/// `<w:{side} …/>` for each side, as the writer writes a border.
fn sides(sides: &[&str], border: &str) -> String {
    sides
        .iter()
        .map(|side| format!("<w:{side} {border}/>"))
        .collect()
}

const ALL: [&str; 6] = ["top", "left", "bottom", "right", "insideH", "insideV"];

/// Demo.docx's table: no style, its own thin black grid.
const DEMO_BORDERS: &str = "<w:tblBorders>\
    <w:top w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:left w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    <w:bottom w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:right w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    <w:insideH w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/><w:insideV w:val=\"single\" w:color=\"000000\" w:sz=\"4\"/>\
    </w:tblBorders>";

#[test]
fn a_markdown_table_takes_words_thin_grid() {
    let written = write("| A | B |\n|---|---|\n| C | D |\n");
    let document = part(&written, "word/document.xml");
    assert_eq!(table_styles(&document), [Some("TableGrid".to_string())]);
    assert_eq!(table_borders(&document), [None]);
    assert!(thin_grid(&part(&written, "word/styles.xml")));
}

#[test]
fn a_table_keeps_its_own_borders_and_no_style() {
    let (first, again, second) = round_trip(&docx(&table(DEMO_BORDERS)));
    assert!(first.contains("  table: TableNormal\n"), "{first}");
    assert!(first.contains(", borders single 0.5pt 000000"), "{first}");
    let document = part(&again, "word/document.xml");
    assert_eq!(table_styles(&document), [None]);
    assert_eq!(
        table_borders(&document),
        [Some(sides(
            &ALL,
            "w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\""
        ))]
    );
    assert_eq!(second, first);
}

#[test]
fn borders_in_automatic_colour_name_no_colour() {
    // 25f1d6ecc3's tables.
    let auto = ALL
        .iter()
        .map(|side| {
            format!("<w:{side} w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>")
        })
        .collect::<String>();
    let (first, again, second) = round_trip(&docx(&table(&format!(
        "<w:tblBorders>{auto}</w:tblBorders>"
    ))));
    assert!(first.contains(", borders single 0.5pt -->"), "{first}");
    assert_eq!(
        table_borders(&part(&again, "word/document.xml")),
        [Some(sides(
            &ALL,
            "w:val=\"single\" w:sz=\"4\" w:space=\"0\""
        ))]
    );
    assert_eq!(second, first);
}

#[test]
fn cell_borders_come_back_on_every_cell() {
    // 5331b981b7's tables: a 0.25pt grid written with start/end, and each
    // cell's own 1pt grey box over it.
    let grid = ["top", "start", "bottom", "end", "insideH", "insideV"]
        .iter()
        .map(|side| {
            format!("<w:{side} w:val=\"single\" w:sz=\"2\" w:space=\"0\" w:color=\"000000\"/>")
        })
        .collect::<String>();
    let boxed = ["top", "start", "bottom", "end"]
        .iter()
        .map(|side| {
            format!("<w:{side} w:val=\"single\" w:sz=\"8\" w:space=\"0\" w:color=\"CCCCCC\"/>")
        })
        .collect::<String>();
    let boxed = format!("<w:tcBorders>{boxed}</w:tcBorders>");
    let source = table_with(
        &format!("<w:tblBorders>{grid}</w:tblBorders>"),
        &[&boxed, &boxed, &boxed, &boxed],
    );
    let (first, again, second) = round_trip(&docx(&source));
    assert!(first.contains(", borders single 0.25pt 000000"), "{first}");
    assert!(
        first.contains(", cell-borders top,left,bottom,right single 1pt CCCCCC in all"),
        "{first}"
    );
    let document = part(&again, "word/document.xml");
    assert_eq!(
        table_borders(&document),
        [Some(sides(
            &ALL,
            "w:val=\"single\" w:sz=\"2\" w:space=\"0\" w:color=\"000000\""
        ))]
    );
    let cell = sides(
        &["top", "left", "bottom", "right"],
        "w:val=\"single\" w:sz=\"8\" w:space=\"0\" w:color=\"CCCCCC\"",
    );
    assert_eq!(cell_borders(&document), vec![Some(cell); 4]);
    assert_eq!(second, first);
}

#[test]
fn sides_that_differ_print_in_groups() {
    let props = "<w:tblBorders>\
        <w:top w:val=\"single\" w:sz=\"12\"/><w:left w:val=\"nil\"/><w:bottom w:val=\"single\" w:sz=\"12\"/>\
        <w:right w:val=\"nil\"/><w:insideH w:val=\"dashed\" w:sz=\"4\" w:color=\"FF0000\"/><w:insideV w:val=\"none\"/>\
        </w:tblBorders>";
    let (first, again, second) = round_trip(&docx(&table(props)));
    assert!(
        first.contains(", borders top,bottom single 1.5pt + left,right,insideV none + insideH dashed 0.5pt FF0000"),
        "{first}"
    );
    let written = table_borders(&part(&again, "word/document.xml"));
    let written = written[0].as_deref().unwrap();
    assert!(
        written.starts_with(
            "<w:top w:val=\"single\" w:sz=\"12\" w:space=\"0\"/><w:left w:val=\"nil\"/>"
        ),
        "{written}"
    );
    assert!(written.contains("<w:insideH w:val=\"dashed\" w:sz=\"4\" w:space=\"0\" w:color=\"FF0000\"/><w:insideV w:val=\"nil\"/>"), "{written}");
    assert_eq!(second, first);
}

#[test]
fn a_rows_cell_borders_name_the_row() {
    let rule = "<w:tcBorders><w:bottom w:val=\"single\" w:sz=\"8\"/></w:tcBorders>";
    let (first, again, second) = round_trip(&docx(&table_with("", &[rule, rule, "", ""])));
    assert!(
        first.contains(", cell-borders bottom single 1pt in t0.row0"),
        "{first}"
    );
    let cells = cell_borders(&part(&again, "word/document.xml"));
    let bottom = Some("<w:bottom w:val=\"single\" w:sz=\"8\" w:space=\"0\"/>".to_string());
    assert_eq!(cells, [bottom.clone(), bottom, None, None]);
    assert_eq!(second, first);
}

#[test]
fn single_cells_name_themselves() {
    let rule = "<w:tcBorders><w:top w:val=\"double\" w:sz=\"4\"/></w:tcBorders>";
    let (first, again, second) = round_trip(&docx(&table_with("", &["", rule, "", rule])));
    assert!(
        first.contains(", cell-borders top double 0.5pt in t0.row0.cell1 t0.row1.cell1"),
        "{first}"
    );
    let cells = cell_borders(&part(&again, "word/document.xml"));
    assert_eq!(cells.iter().filter(|c| c.is_some()).count(), 2, "{cells:?}");
    assert_eq!(second, first);
}

#[test]
fn a_table_without_borders_stays_without() {
    let (first, again, second) = round_trip(&docx(&table("")));
    assert!(first.contains("  table: TableNormal\n"), "{first}");
    assert!(!first.contains("borders"), "{first}");
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
    assert!(thin_grid(&part(&again, "word/styles.xml")));
    assert_eq!(second, first);
}

#[test]
fn a_grid_and_a_borderless_table_keep_their_own() {
    let body = format!(
        "{}<w:p/>{}",
        table("<w:tblStyle w:val=\"TableGrid\"/>"),
        table(DEMO_BORDERS)
    );
    let (first, again, second) = round_trip(&docx(&body));
    assert!(
        first.contains(
            "<!-- t1 2x2, cells p6-p9 by row, style TableNormal, borders single 0.5pt 000000 -->"
        ),
        "{first}"
    );
    assert!(
        first.contains("<!-- t0 2x2, cells p0-p3 by row -->"),
        "{first}"
    );
    let document = part(&again, "word/document.xml");
    assert_eq!(
        table_styles(&document),
        [Some("TableGrid".to_string()), None]
    );
    assert_eq!(table_borders(&document)[0], None);
    assert_eq!(second, first);
}
