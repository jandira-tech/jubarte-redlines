// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1873: a table row keeps Word's save marks (`w:rsidR`, `w:rsidTr`)
//! through a redline, Accept All and Reject All, as Word keeps them. The
//! comparer accepts both inputs first, and that accept stripped every rsid,
//! so a redline's rows carried none. The PDF renderer keeps a table's saved
//! grid only when its rows carry Word's marks (#1716, #1732), so it re-fitted
//! the auto-width tables of every Word-saved document's redline and of its
//! Accept All: `b6dd0b6ae8`'s statistics tables wrapped differently.

mod common;

use common::docx::{docx, para, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// A row of two auto-width cells holding `text`, saved by Word in session
/// `rsid` (its paragraphs carry the session too).
fn row(rsid: &str, text: &str) -> String {
    let cell = |t: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p w:rsidR="{rsid}" w:rsidRDefault="{rsid}"><w:r><w:t>{t}</w:t></w:r></w:p></w:tc>"#
        )
    };
    format!(
        r#"<w:tr w:rsidR="{rsid}" w:rsidTr="{rsid}">{}{}</w:tr>"#,
        cell(text),
        cell(&format!("{text}, second"))
    )
}

/// "Intro" (or `intro`), a two-column auto table of `rows`, "End".
fn document(intro: &str, rows: &[String]) -> Vec<u8> {
    docx(&format!(
        r#"{}<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid>{}</w:tbl>{}"#,
        para(intro),
        rows.concat(),
        para("End")
    ))
}

/// Each row's (`w:rsidR`, `w:rsidTr`) in `docx`'s body, in order.
fn marks(docx: &[u8]) -> Vec<(Option<String>, Option<String>)> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let attr = |tag: &str, name: &str| {
        let key = format!(r#" w:{name}=""#);
        tag.find(&key).map(|at| {
            let rest = &tag[at + key.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
    };
    xml.match_indices("<w:tr")
        .filter(|&(at, _)| matches!(xml.as_bytes().get(at + 5), Some(b'>' | b' ')))
        .map(|(at, _)| {
            let tag = &xml[at..at + xml[at..].find('>').unwrap()];
            (attr(tag, "rsidR"), attr(tag, "rsidTr"))
        })
        .collect()
}

fn saved(rsid: &str) -> (Option<String>, Option<String>) {
    (Some(rsid.to_string()), Some(rsid.to_string()))
}

/// An edit outside the table leaves each row's marks in the redline, in
/// Accept All and in Reject All.
#[test]
fn a_redline_keeps_its_rows_marks() {
    let rows = [row("00A1B2C3", "one"), row("00D4E5F6", "two")];
    let original = document("Intro", &rows);
    let revised = document("Intro changed", &rows);
    let want = vec![saved("00A1B2C3"), saved("00D4E5F6")];
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert!(
            part_string(&redline, "word/document.xml")
                .unwrap()
                .contains("<w:ins "),
            "{mode}: the edit is marked"
        );
        assert_eq!(marks(&redline), want, "{mode}, redline");
        assert_eq!(
            marks(&accept_revisions(&redline).unwrap()),
            want,
            "{mode}, accept"
        );
        assert_eq!(
            marks(&reject_revisions(&redline).unwrap()),
            want,
            "{mode}, reject"
        );
    }
}

/// An inserted row carries the revision's marks and a deleted row the
/// original's.
#[test]
fn an_inserted_or_deleted_row_carries_its_own_side_marks() {
    let (one, two) = (row("00A1B2C3", "one"), row("00D4E5F6", "two"));
    let added = row("00112233", "a row the revision adds");
    let gone = row("00445566", "a row the revision removes");
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(
            &document("Intro", &[one.clone(), two.clone()]),
            &document("Intro", &[one.clone(), two.clone(), added.clone()]),
            &settings,
        )
        .unwrap();
        assert_eq!(
            marks(&redline),
            vec![saved("00A1B2C3"), saved("00D4E5F6"), saved("00112233")],
            "{mode}, inserted"
        );
        let redline = compare_documents_with_settings(
            &document("Intro", &[one.clone(), two.clone(), gone.clone()]),
            &document("Intro", &[one.clone(), two.clone()]),
            &settings,
        )
        .unwrap();
        assert_eq!(
            marks(&redline),
            vec![saved("00A1B2C3"), saved("00D4E5F6"), saved("00445566")],
            "{mode}, deleted"
        );
    }
}

/// An `add` leaves the rows it did not touch with their marks, in the
/// redline and in its Accept All, as in the clean copy.
#[test]
fn an_add_keeps_the_rows_marks() {
    let source = document("Intro", &[row("00A1B2C3", "one"), row("00D4E5F6", "two")]);
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let want = vec![saved("00A1B2C3"), saved("00D4E5F6")];
    assert_eq!(marks(&edited.clean), want, "clean");
    assert_eq!(marks(&edited.redline), want, "redline");
    assert_eq!(
        marks(&accept_revisions(&edited.redline).unwrap()),
        want,
        "accept"
    );
}
