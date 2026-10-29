// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1869: a table row's exceptions (`w:tblPrEx`, the table properties one
//! row overrides) survive a redline. The comparer rebuilt each row from its
//! `w:trPr` alone, so a row that drew its own borders or margins lost them
//! in `compare`, in both modes, and in the redline of every edit; a change
//! to them was not recorded either.

mod common;

use common::docx::{docx, para, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

/// Row exceptions that draw a single border around the row's cells.
const BORDERS: &str = r#"<w:tblPrEx><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPrEx>"#;

/// Row exceptions that set the row's cell margins.
const MARGINS: &str = r#"<w:tblPrEx><w:tblCellMar><w:left w:w="0" w:type="dxa"/><w:right w:w="0" w:type="dxa"/></w:tblCellMar></w:tblPrEx>"#;

/// A row of one cell holding `text`, its exceptions `ex` first.
fn row(ex: &str, text: &str) -> String {
    format!(
        r#"<w:tr>{ex}<w:trPr><w:cantSplit/></w:trPr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc></w:tr>"#
    )
}

/// A one-column table of `rows`.
fn table(rows: &[String]) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>{}</w:tbl>"#,
        rows.concat()
    )
}

/// Every row's exceptions in `docx`'s body, in order; `""` for a row
/// without any.
fn exceptions(docx: &[u8]) -> Vec<String> {
    let xml = part_string(docx, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    xml.match_indices("<w:tr")
        .filter(|&(at, _)| matches!(xml.as_bytes().get(at + 5), Some(b'>' | b' ')))
        .map(|(at, _)| {
            let row = &xml[at..];
            let cell = [row.find("<w:tc>"), row.find("<w:tc ")]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(row.len());
            let head = &row[..cell];
            head.find("<w:tblPrEx>").map_or_else(String::new, |open| {
                head[open..head.find("</w:tblPrEx>").unwrap() + "</w:tblPrEx>".len()].to_string()
            })
        })
        .collect()
}

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// The body changes outside the table: every row keeps its exceptions in the
/// redline, its Accept All and its Reject All.
#[test]
fn an_untouched_table_keeps_its_row_exceptions() {
    let rows = [
        row(BORDERS, "First row"),
        row("", "Second row"),
        row(MARGINS, "Third row"),
    ];
    let original = docx(&format!(
        "{}{}{}",
        para("Intro text."),
        table(&rows),
        para("Omega.")
    ));
    let modified = docx(&format!(
        "{}{}{}",
        para("Intro words."),
        table(&rows),
        para("Omega.")
    ));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
        for (label, docx) in [
            ("redline", redline.clone()),
            ("accepted", accept_revisions(&redline).unwrap()),
            ("rejected", reject_revisions(&redline).unwrap()),
        ] {
            assert_eq!(exceptions(&docx), [BORDERS, "", MARGINS], "{mode} {label}");
        }
    }
}

/// A row whose text and exceptions both change is one row in the redline
/// (the comparer pairs it): it shows the revised exceptions and records the
/// original's in `w:tblPrExChange`, as Word's compare does, so Accept All
/// keeps the revised ones and Reject All gives the original's back.
#[test]
fn a_row_whose_exceptions_change_records_the_original_ones() {
    let original = docx(&table(&[
        row("", "Kept row"),
        row(BORDERS, "An old row that goes away"),
    ]));
    let modified = docx(&table(&[
        row("", "Kept row"),
        row(MARGINS, "Quite another row arrives"),
    ]));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
        let xml = part_string(&redline, "word/document.xml").unwrap();
        assert_eq!(xml.matches("<w:tblPrExChange ").count(), 1, "{mode}: {xml}");
        assert_eq!(
            exceptions(&accept_revisions(&redline).unwrap()),
            ["", MARGINS],
            "{mode}"
        );
        assert_eq!(
            exceptions(&reject_revisions(&redline).unwrap()),
            ["", BORDERS],
            "{mode}"
        );
    }
}

/// An edit elsewhere in the document leaves the rows as the source has
/// them (#1868 found it on `60dfd517a9`).
#[test]
fn an_edit_elsewhere_keeps_row_exceptions() {
    let source = docx(&format!(
        "{}{}",
        para("Intro text."),
        table(&[row(BORDERS, "First row"), row(MARGINS, "Second row")])
    ));
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(exceptions(&edited.clean), [BORDERS, MARGINS]);
    assert_eq!(
        exceptions(&accept_revisions(&edited.redline).unwrap()),
        [BORDERS, MARGINS]
    );
}
