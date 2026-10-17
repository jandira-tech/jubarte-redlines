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

/// `styles.xml` defining Table Grid with `sz` on all six sides.
fn table_grid_styles(sz: u32) -> String {
    let borders = sides(
        &ALL,
        &format!("w:val=\"single\" w:sz=\"{sz}\" w:space=\"0\" w:color=\"auto\""),
    );
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"{}\"><w:style w:type=\"table\" w:styleId=\"TableGrid\">\
         <w:name w:val=\"Table Grid\"/><w:basedOn w:val=\"TableNormal\"/>\
         <w:tblPr><w:tblBorders>{borders}</w:tblBorders></w:tblPr></w:style></w:styles>",
        common::docx::W_NS
    )
}

fn with_styles(body: &str, styles: &str) -> Vec<u8> {
    common::docx::docx_with(
        body,
        &[common::docx::Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: styles,
        }],
    )
}

#[test]
fn theme_border_colours_come_back() {
    // A border in a theme colour keeps it, beside its fallback colour.
    let border = "w:val=\"single\" w:sz=\"8\" w:space=\"0\" w:color=\"2F5496\" \
                  w:themeColor=\"accent1\" w:themeTint=\"99\" w:themeShade=\"BF\"";
    let props = format!("<w:tblBorders>{}</w:tblBorders>", sides(&ALL, border));
    let (first, again, second) = round_trip(&docx(&table(&props)));
    assert!(
        first.contains(", borders single 1pt 2F5496 theme accent1 tint 99 shade BF"),
        "{first}"
    );
    let document = part(&again, "word/document.xml");
    assert_eq!(table_borders(&document), [Some(sides(&ALL, border))]);
    assert_eq!(second, first);
}

#[test]
fn a_rows_own_table_borders_come_back() {
    // `w:tblPrEx/w:tblBorders`: the table's borders as one row overrides
    // them.
    let ex = "<w:tblPrEx><w:tblBorders><w:top w:val=\"double\" w:sz=\"4\" w:space=\"0\" \
              w:color=\"FF0000\"/></w:tblBorders></w:tblPrEx>";
    let cell = |text: &str| {
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"4000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"
        )
    };
    let body = format!(
        "<w:tbl><w:tblPr>{DEMO_BORDERS}</w:tblPr><w:tblGrid><w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>{}{}</w:tr><w:tr>{ex}{}{}</w:tr></w:tbl><w:p/>",
        cell("A"),
        cell("B"),
        cell("C"),
        cell("D")
    );
    let (first, again, second) = round_trip(&docx(&body));
    assert!(
        first.contains("row-borders top double 0.5pt FF0000 in t0.row1"),
        "{first}"
    );
    let document = part(&again, "word/document.xml");
    let rows: Vec<&str> = document.split("<w:tr>").skip(1).collect();
    assert!(!rows[0].starts_with("<w:tblPrEx>"), "{document}");
    assert!(
        rows[1].starts_with(
            "<w:tblPrEx><w:tblBorders><w:top w:val=\"double\" w:sz=\"4\" w:space=\"0\" \
             w:color=\"FF0000\"/></w:tblBorders></w:tblPrEx>"
        ),
        "{document}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_source_table_grid_of_its_own_comes_back() {
    // A Table Grid the source defines with 1.5pt borders is not Word's thin
    // one: the legend names it and the writer defines it so.
    let thick = with_styles(
        &table("<w:tblStyle w:val=\"TableGrid\"/>"),
        &table_grid_styles(12),
    );
    let (first, again, second) = round_trip(&thick);
    assert!(
        first.contains("  table: TableGrid, all borders 1.5pt\n"),
        "{first}"
    );
    assert_eq!(
        table_styles(&part(&again, "word/document.xml")),
        [Some("TableGrid".to_string())]
    );
    let styles = part(&again, "word/styles.xml");
    assert!(!thin_grid(&styles), "{styles}");
    assert_eq!(
        styles.matches("w:styleId=\"TableGrid\"").count(),
        1,
        "{styles}"
    );
    assert!(
        styles.contains(&sides(
            &ALL,
            "w:val=\"single\" w:sz=\"12\" w:space=\"0\" w:color=\"auto\""
        )),
        "{styles}"
    );
    assert_eq!(second, first);
    // Word's own thin definition still needs no legend.
    let thin = with_styles(
        &table("<w:tblStyle w:val=\"TableGrid\"/>"),
        &table_grid_styles(4),
    );
    let (first, again, _) = round_trip(&thin);
    assert!(!first.contains("table:"), "{first}");
    assert!(thin_grid(&part(&again, "word/styles.xml")));
}

/// A table style the view names keeps the name the document gives it: a
/// style id is not its name (`9` is named `Normal Table`, ids-fonts,
/// #528), so the legend says both and the writer defines both.
#[test]
fn a_table_style_keeps_its_name() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="9"><w:name w:val="Normal Table"/><w:tblPr><w:tblCellMar><w:top w:w="0" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style></w:styles>"#,
        jubarte::namespaces::W::URI
    );
    let parts = [common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &styles,
    }];
    let body = table(r#"<w:tblStyle w:val="9"/>"#);
    let docx = common::docx::docx_with(&body, &parts);
    let (first, again, second) = round_trip(&docx);
    assert!(first.contains("table: 9, named Normal Table"), "{first}");
    let document = part(&again, "word/document.xml");
    assert_eq!(table_styles(&document), [Some("9".to_string())]);
    let written = part(&again, "word/styles.xml");
    let definition = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"9\""))
        .unwrap_or_else(|| panic!("no style 9: {written}"));
    assert!(
        definition.contains(r#"<w:name w:val="Normal Table"/>"#),
        "{definition}"
    );
    assert_eq!(second, first);
}

/// Word's older table styles (Light Shading Accent 1, Medium List 2) keep
/// Word's names without the view naming them (docx_lots_of_comments,
/// #528): the writer derives them from the id, as Word spells them.
#[test]
fn words_older_table_styles_get_their_own_names() {
    let w = jubarte::namespaces::W::URI;
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="table" w:styleId="LightShading-Accent1"><w:name w:val="Light Shading Accent 1"/><w:basedOn w:val="TableNormal"/></w:style><w:style w:type="table" w:styleId="MediumList2"><w:name w:val="Medium List 2"/><w:basedOn w:val="TableNormal"/></w:style></w:styles>"#
    );
    let parts = [common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &styles,
    }];
    let body = format!(
        "{}{}",
        table(r#"<w:tblStyle w:val="MediumList2"/>"#),
        table(r#"<w:tblStyle w:val="LightShading-Accent1"/>"#)
    );
    let docx = common::docx::docx_with(&body, &parts);
    let (first, again, second) = round_trip(&docx);
    assert!(!first.contains("named "), "{first}");
    let written = part(&again, "word/styles.xml");
    // Word's own definitions, band shading and all, not a bare name.
    let shading = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"MediumList2\""))
        .unwrap_or_else(|| panic!("no MediumList2: {written}"));
    assert!(
        shading.contains(r#"<w:tblStylePr w:type="band1Horz">"#) || shading.contains("band1Vert"),
        "{shading}"
    );
    for (id, name) in [
        ("LightShading-Accent1", "Light Shading Accent 1"),
        ("MediumList2", "Medium List 2"),
    ] {
        let definition = written
            .split("<w:style ")
            .find(|s| s.contains(&format!("w:styleId=\"{id}\"")))
            .unwrap_or_else(|| panic!("no {id}: {written}"));
        assert!(
            definition.contains(&format!(r#"<w:name w:val="{name}"/>"#)),
            "{definition}"
        );
    }
    // The written styles are Word's own definitions, borders and all, so
    // the second legend says them (`borders vary`) where the document's
    // bare definitions said nothing: the plural legend tells the truth
    // about the docx it describes (#549).
    assert!(
        second.contains("tables: MediumList2, borders vary, LightShading-Accent1, borders vary"),
        "{second}"
    );
}

/// The plural `tables:` legend carries each style's name and borders as
/// the singular `table:` line does: a styled table beside an unstyled one
/// (or any two styles) must not drop them, for the writer would name the
/// style by its id (#549).
#[test]
fn the_plural_tables_legend_carries_each_styles_name() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="X"><w:name w:val="X Name"/><w:basedOn w:val="TableNormal"/></w:style></w:styles>"#,
        jubarte::namespaces::W::URI
    );
    let parts = [common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &styles,
    }];
    let body = format!(
        "{}{}",
        table(r#"<w:tblStyle w:val="X"/>"#),
        table(r#"<w:tblStyle w:val="TableNormal"/>"#)
    );
    let docx = common::docx::docx_with(&body, &parts);
    let (first, again, second) = round_trip(&docx);
    assert!(
        first.contains("tables: X, named X Name, TableNormal"),
        "{first}"
    );
    let written = part(&again, "word/styles.xml");
    let definition = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"X\""))
        .unwrap_or_else(|| panic!("no style X: {written}"));
    assert!(
        definition.contains(r#"<w:name w:val="X Name"/>"#),
        "{definition}"
    );
    assert_eq!(second, first);
}

/// A `named` clause on one of Word's own table-style ids renames Word's
/// template: the document renamed the built-in (`GridTable2` named
/// `My Grid`), and the writer writes the document's name, not Word's
/// (#549).
#[test]
fn a_named_built_in_table_style_keeps_the_documents_name() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="GridTable2"><w:name w:val="My Grid"/><w:basedOn w:val="TableNormal"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="8"/><w:left w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="8"/><w:right w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="8"/></w:tblBorders></w:tblPr></w:style></w:styles>"#,
        jubarte::namespaces::W::URI
    );
    let parts = [common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &styles,
    }];
    let docx = common::docx::docx_with(&table(r#"<w:tblStyle w:val="GridTable2"/>"#), &parts);
    let (first, again, second) = round_trip(&docx);
    assert!(
        first.contains("table: GridTable2, borders vary, named My Grid"),
        "{first}"
    );
    let written = part(&again, "word/styles.xml");
    let definition = written
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"GridTable2\""))
        .unwrap_or_else(|| panic!("no GridTable2: {written}"));
    assert!(
        definition.contains(r#"<w:name w:val="My Grid"/>"#),
        "{definition}"
    );
    assert!(!definition.contains("Grid Table 2"), "{definition}");
    assert_eq!(second, first);
}

/// A table style's name that holds the `named` separator or a comma
/// quotes in the legend (`named "Foo, named Bar"`), so the clause's
/// grammar stays unambiguous, and the writer writes no comma into
/// `w:name` — Word forbids one there — replacing it with a space and
/// saying so in a warning (#549).
#[test]
fn table_style_names_with_commas_quote_and_sanitize() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{}"><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="X"><w:name w:val="Foo, named Bar"/><w:basedOn w:val="TableNormal"/></w:style><w:style w:type="table" w:styleId="Y"><w:name w:val="A, Comma"/><w:basedOn w:val="TableNormal"/></w:style></w:styles>"#,
        jubarte::namespaces::W::URI
    );
    let parts = [common::docx::Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &styles,
    }];
    let body = format!(
        "{}{}",
        table(r#"<w:tblStyle w:val="X"/>"#),
        table(r#"<w:tblStyle w:val="Y"/>"#)
    );
    let docx = common::docx::docx_with(&body, &parts);
    let first_view = view(&docx);
    assert!(
        first_view.contains(r#"tables: X, named "Foo, named Bar", Y, named "A, Comma""#),
        "{first_view}"
    );
    let (first, again, second) = round_trip(&docx);
    let written = part(&again, "word/styles.xml");
    assert!(
        written.contains(r#"<w:name w:val="Foo  named Bar"/>"#),
        "{written}"
    );
    assert!(
        written.contains(r#"<w:name w:val="A  Comma"/>"#),
        "{written}"
    );
    assert!(!written.contains("A, Comma"), "{written}");
    assert!(second.contains("named Foo  named Bar"), "{second}");
    assert!(second.contains("named A  Comma"), "{second}");
    // The writer says what it did to each name.
    let converted = markdown_to_docx(&first, &DocxOptions::default()).expect("the view reads");
    for name in ["Foo, named Bar", "A, Comma"] {
        assert!(
            converted.warnings.iter().any(|w| w.contains(name)),
            "no warning names {name}: {:?}",
            converted.warnings
        );
    }
}
