// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A table's geometry rides on its line (#1472): the grid's column widths
//! (`widths 2.1424in 0.985in 1.049in`), the table's preferred width
//! (`width auto`, `width 50%`, `width 2.778in`), its indent (`indent
//! 0.25in`) and a fixed layout (`layout fixed`). Without them the writer
//! split the text width evenly over 100% of the page, a first column of
//! long labels wrapped, and the table took a page more than the source.
//! What the writer writes by itself (equal columns, 100%) prints nothing,
//! so a plain Markdown table reads back as it was written.

mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

fn view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
    // The source's hash differs for every docx written.
    .lines()
    .filter(|l| !l.starts_with("sha256: "))
    .collect::<Vec<_>>()
    .join("\n")
}

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    assert_word_valid_package(&docx);
    docx
}

fn document(docx: &[u8]) -> String {
    part_string(docx, "word/document.xml").expect("a main document")
}

/// The value of `attr` on the first `<w:{name} …/>` in `xml`.
fn attr(xml: &str, name: &str, attr: &str) -> Option<String> {
    let at = xml.find(&format!("<w:{name} "))?;
    let tag = &xml[at..at + xml[at..].find('>')?];
    let key = format!("w:{attr}=\"");
    let value = &tag[tag.find(&key)? + key.len()..];
    Some(value[..value.find('"')?].to_string())
}

/// Every `w:{name}`'s `w:{attr}` in `xml`, in order.
fn all(xml: &str, name: &str, attr: &str) -> Vec<String> {
    xml.split(&format!("<w:{name} "))
        .skip(1)
        .filter_map(|tail| {
            let tag = &tail[..tail.find('>')?];
            let key = format!("w:{attr}=\"");
            let value = &tag[tag.find(&key)? + key.len()..];
            Some(value[..value.find('"')?].to_string())
        })
        .collect()
}

/// A table's geometry as Word reads it: the grid, the preferred width and
/// its type, the indent, the layout, and each cell's width.
#[derive(Debug, PartialEq)]
struct Geometry {
    grid: Vec<String>,
    width: (Option<String>, Option<String>),
    indent: Option<String>,
    layout: Option<String>,
    cells: Vec<String>,
}

fn geometry(table: &str) -> Geometry {
    let properties = &table[..table.find("</w:tblPr>").expect("a tblPr")];
    Geometry {
        grid: all(table, "gridCol", "w"),
        width: (
            attr(properties, "tblW", "w"),
            attr(properties, "tblW", "type"),
        ),
        indent: attr(properties, "tblInd", "w"),
        layout: attr(properties, "tblLayout", "type"),
        cells: all(table, "tcW", "w"),
    }
}

/// Each table of `document`, as its XML.
fn tables(document: &str) -> Vec<&str> {
    document
        .split("<w:tbl>")
        .skip(1)
        .map(|t| &t[..t.find("</w:tbl>").expect("a closed table")])
        .collect()
}

fn cell(text: &str, width: u32, span: Option<u32>) -> String {
    let span = span.map_or(String::new(), |n| format!("<w:gridSpan w:val=\"{n}\"/>"));
    format!(
        "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/>{span}</w:tcPr>\
         <w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"
    )
}

const UNEQUAL: &str = "<w:tblGrid><w:gridCol w:w=\"3085\"/><w:gridCol w:w=\"1418\"/>\
                       <w:gridCol w:w=\"1511\"/></w:tblGrid>";

fn unequal_rows() -> String {
    format!(
        "<w:tr>{}{}{}</w:tr><w:tr>{}{}</w:tr>\
         <w:tr><w:trPr><w:gridBefore w:val=\"1\"/><w:wBefore w:w=\"3085\" w:type=\"dxa\"/></w:trPr>{}{}</w:tr>",
        cell("Label of the first row", 3085, None),
        cell("B", 1418, None),
        cell("C", 1511, None),
        cell("Across two", 4503, Some(2)),
        cell("F", 1511, None),
        cell("H", 1418, None),
        cell("I", 1511, None),
    )
}

#[test]
fn a_tables_grid_width_indent_and_layout_survive_docx_to_markdown_and_back() {
    let body = format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/><w:tblInd w:w=\"360\" w:type=\"dxa\"/>\
         <w:tblLayout w:type=\"fixed\"/></w:tblPr>{UNEQUAL}{}</w:tbl><w:p/>",
        unequal_rows()
    );
    let source = docx(&body);
    let shown = view(&source);
    for clause in [
        "widths 2.1424in 0.985in 1.049in",
        "width auto",
        "indent 0.25in",
        "layout fixed",
    ] {
        assert!(shown.contains(clause), "no `{clause}` in:\n{shown}");
    }
    let written = write(&shown);
    let want = geometry(tables(&document(&source))[0]);
    let got = geometry(tables(&document(&written))[0]);
    assert_eq!(got, want, "the view:\n{shown}");
    assert_eq!(
        all(&document(&written), "wBefore", "w"),
        ["3085"],
        "a row's gap is as wide as the grid columns it leaves out"
    );
    assert_eq!(view(&written), shown, "the view reads back as it was");
}

#[test]
fn a_preferred_width_in_percent_or_twips_comes_back() {
    let equal = "<w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid>";
    let rows = format!(
        "<w:tr>{}{}</w:tr>",
        cell("A", 2000, None),
        cell("B", 2000, None)
    );
    let body = format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"2500\" w:type=\"pct\"/></w:tblPr>{equal}{rows}</w:tbl><w:p/>\
         <w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr>{equal}{rows}</w:tbl><w:p/>\
         <w:tbl><w:tblPr><w:tblW w:w=\"25%\" w:type=\"pct\"/></w:tblPr>{equal}{rows}</w:tbl><w:p/>"
    );
    let source = docx(&body);
    let shown = view(&source);
    assert!(shown.contains("width 50%"), "{shown}");
    assert!(shown.contains("width 2.778in"), "{shown}");
    assert!(shown.contains("width 25%"), "{shown}");
    let written = document(&write(&shown));
    let widths: Vec<_> = tables(&written).iter().map(|t| geometry(t).width).collect();
    let pair = |w: &str, t: &str| (Some(w.to_string()), Some(t.to_string()));
    assert_eq!(
        widths,
        [
            pair("2500", "pct"),
            pair("4000", "dxa"),
            pair("1250", "pct")
        ],
        "the view:\n{shown}"
    );
}

#[test]
fn a_markdown_tables_own_geometry_prints_nothing() {
    let markdown = "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n";
    let first = view(&write(markdown));
    for word in ["widths ", "width ", "indent ", "layout "] {
        let line = first
            .lines()
            .find(|l| l.starts_with("<!-- t0"))
            .expect("a table line");
        assert!(!line.contains(word), "`{word}` on {line}");
    }
    assert_eq!(view(&write(&first)), first, "the view is a fixed point");
}

#[test]
fn widths_that_do_not_count_the_columns_are_not_written() {
    // An edit that adds a column leaves the widths one short: the writer
    // splits the text width evenly, as with no widths, and says why.
    let shown = view(&write(
        "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n",
    ));
    let edited: String = shown
        .lines()
        .map(|l| {
            match l
                .strip_prefix("<!-- t0 ")
                .and_then(|l| l.strip_suffix(" -->"))
            {
                Some(clauses) => format!("<!-- t0 {clauses}, widths 1in 2in -->"),
                None => l.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(edited.contains(", widths 1in 2in -->"), "{edited}");
    let out = markdown_to_docx(&edited, &DocxOptions::default()).expect("the Markdown reads");
    assert_word_valid_package(&out.docx);
    let grid = geometry(tables(&document(&out.docx))[0]).grid;
    assert_eq!(grid.len(), 3);
    assert!(grid.iter().all(|w| *w == grid[0]), "{grid:?}");
    assert!(
        out.warnings
            .iter()
            .any(|w| w.contains("t0") && w.contains("widths")),
        "{:?}",
        out.warnings
    );
}
