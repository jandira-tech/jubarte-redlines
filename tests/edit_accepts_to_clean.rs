// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1868: on a clean file, Accept All of an edit's redline is the clean copy
//! the plan asked for. The redline tracks the edit, as Word's Track Changes
//! would, and changes nothing else: no style renamed or merged, no spacing
//! stripped or written out, no hyperlink unwrapped.

mod common;

use std::io::Read as _;

use common::docx::{docx, docx_with_sect_pr, para, part_string, replace_entry};
use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::accept_revisions;
use jubarte::edit::{EditPlan, apply_plan};

/// A bench fixture under `tests/corpus`, by file name.
fn bench_clean(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "tests/corpus/neurotic_docx_bench/corpus/word/clean/docx/{name}"
    ))
    .unwrap()
}

/// `jubarte debug A B --check runs --check styledefs`: one `… identical`
/// line per check when `a` and `b` agree.
fn differences(a: &[u8], b: &[u8]) -> String {
    report(
        a,
        Some(b),
        &Options {
            checks: vec![Check::Runs, Check::StyleDefs],
            ..Options::default()
        },
    )
    .unwrap()
}

/// How many `w:hyperlink`s the body holds.
fn hyperlinks(docx: &[u8]) -> usize {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml.matches("<w:hyperlink ").count()
}

/// Apply the plan made of `ops` to `source`, then check that the redline
/// accepts to the clean copy and keeps every hyperlink.
fn assert_plan_accepts_to_clean(source: &[u8], ops: &str, label: &str) {
    let edited = apply_plan(
        source,
        &EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Probe","operations":[{ops}]}}"#
        ))
        .unwrap(),
    )
    .unwrap();
    let accepted = accept_revisions(&edited.redline).unwrap();
    assert_eq!(
        differences(&accepted, &edited.clean),
        "runs identical\nstyledefs identical\n",
        "{label}: {ops}"
    );
    assert_eq!(
        hyperlinks(&edited.redline),
        hyperlinks(source),
        "{label}: {ops}"
    );
}

/// [`assert_plan_accepts_to_clean`] on the bench fixture `name`.
fn assert_accepts_to_clean(name: &str, op: &str) {
    assert_plan_accepts_to_clean(&bench_clean(name), op, name);
}

/// Through the comparer, the redline of a new paragraph renamed
/// `0b2a46481b`'s `style20` to `TableContents`, stripped `0018843794`'s
/// `spacing line=276` from every body paragraph, and renamed `8441c4ae21`'s
/// localized `Sumrio1` to `TOC1`, unwrapping its 133 TOC hyperlinks. It
/// wrote `lineRule=auto` on `00da16e738`'s closing heading, and dropped
/// the new paragraph's `rtl=0` on `2f23773d63`.
#[test]
fn a_new_paragraph_accepts_to_the_clean_copy() {
    for name in [
        "0b2a46481b_nested_table_rowspan.docx",
        "0018843794_889f4c3958e28978a0d96a95a8938d9eb0dc100e66f7c483.docx",
        "8441c4ae21_sd_2517_localized_heading_styles.docx",
        "00da16e738_heading_4_style_demo_id_paraid_overflow_2.docx",
        "2f23773d63_super_editor__pagination_blank_2a98ed7a.docx",
    ] {
        assert_accepts_to_clean(
            name,
            r#"{"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}"#,
        );
    }
}

/// A one-word replacement goes through the comparer too, and must leave the
/// rest of the document as the source has it.
#[test]
fn a_replaced_word_accepts_to_the_clean_copy() {
    for (name, op) in [
        (
            "0b2a46481b_nested_table_rowspan.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"AAA"},"find":"AAA","replacement":"AAX"}"#,
        ),
        (
            "0018843794_889f4c3958e28978a0d96a95a8938d9eb0dc100e66f7c483.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"Proposal Number"},"find":"Number","replacement":"Code"}"#,
        ),
        (
            "8441c4ae21_sd_2517_localized_heading_styles.docx",
            r#"{"kind":"replace","paragraph":{"starts_with":"dolor 10"},"find":"10","replacement":"11"}"#,
        ),
    ] {
        assert_accepts_to_clean(name, op);
    }
}

/// The comparer rebuilds the document from its body and the section from
/// its geometry: a page background, a vertically centred page, page borders,
/// line numbers and the section's spelled-out defaults were lost, and a
/// numbering part was made up for a list the source left undefined.
#[test]
fn a_new_paragraph_keeps_the_page_and_section_as_the_source_has_them() {
    let sect_pr = concat!(
        r#"<w:sectPr><w:type w:val="nextPage"/><w:pgSz w:w="12240" w:h="15840"/>"#,
        r#"<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#,
        r#"<w:pgBorders><w:top w:val="single" w:sz="4" w:space="24" w:color="auto"/></w:pgBorders>"#,
        r#"<w:lnNumType w:countBy="1"/><w:cols w:space="720" w:equalWidth="1"/>"#,
        r#"<w:vAlign w:val="center"/></w:sectPr>"#
    );
    let body = format!(
        r#"{}<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="5"/></w:numPr></w:pPr><w:r><w:t>Listed.</w:t></w:r></w:p>{}"#,
        para("Alpha one."),
        para("Omega.")
    );
    let built = docx_with_sect_pr(&body, &[], sect_pr);
    let document = part_string(&built, "word/document.xml")
        .unwrap()
        .replace("<w:body>", r#"<w:background w:color="FFFF00"/><w:body>"#);
    let source = replace_entry(&built, "word/document.xml", document.as_bytes());
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":{"starts_with":"Alpha"},"runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let accepted = accept_revisions(&edited.redline).unwrap();
    let xml = part_string(&accepted, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    for kept in [
        r#"<w:background w:color="FFFF00"/>"#,
        r#"<w:type w:val="nextPage"/>"#,
        "<w:pgBorders>",
        "<w:lnNumType ",
        r#"w:equalWidth="1""#,
        r#"<w:vAlign w:val="center"/>"#,
    ] {
        assert!(xml.contains(kept), "{kept} lost: {xml}");
    }
    assert!(
        part_string(&edited.redline, "word/numbering.xml").is_none(),
        "the redline made up a numbering part"
    );
}

/// Word Compare redraws a grid that declares fewer columns than its rows
/// hold: equal columns over the page, the table width `0 auto` (M-TBL
/// rule 4). The edit's clean copy keeps the table as the source has it, so
/// the redline must too: `0b2a46481b`'s and `45c1c5abec`'s tables were
/// drawn anew, a table the plan never touched.
#[test]
fn a_new_paragraph_keeps_a_table_grid_as_the_source_has_it() {
    let table = concat!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="9000" w:type="dxa"/></w:tblPr>"#,
        r#"<w:tblGrid><w:gridCol w:w="9000"/></w:tblGrid><w:tr>"#,
        r#"<w:tc><w:tcPr><w:tcW w:w="3000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Left</w:t></w:r></w:p></w:tc>"#,
        r#"<w:tc><w:tcPr><w:tcW w:w="6000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Right</w:t></w:r></w:p></w:tc>"#,
        "</w:tr></w:tbl>"
    );
    let source = docx(&format!("{}{table}{}", para("Alpha one."), para("Omega.")));
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":{"starts_with":"Alpha"},"runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let accepted = accept_revisions(&edited.redline).unwrap();
    let xml = part_string(&accepted, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    for kept in [
        r#"<w:tblW w:w="9000" w:type="dxa"/>"#,
        r#"<w:tblGrid><w:gridCol w:w="9000"/></w:tblGrid>"#,
    ] {
        assert!(xml.contains(kept), "{kept} lost: {xml}");
    }
}
