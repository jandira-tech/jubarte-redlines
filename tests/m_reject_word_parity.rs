// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Reject All as Microsoft Word does it. Each rule was read off Word's own
//! Reject All of Word's redlines (the bench's `rejected_tracking` corpus,
//! 2026-09-29) and is pinned here with a synthetic package.

mod common;

use common::docx::{Part, docx, docx_with, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const REV: &str = r#"w:author="a" w:date="2026-01-01T00:00:00Z""#;

/// Body paragraphs of `document.xml` (tables included, in order), each as
/// its visible text.
fn paragraphs(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>()
        })
        .collect()
}

/// A move range is a pair of markers, not a container: Word ends a moveTo
/// range inside the first cell of the table after the moved heading (its
/// redline of 30f20e787b). Rejecting the move removes the moved heading and
/// its mark, and the table keeps its tblPr, tblGrid and surviving rows.
#[test]
fn reject_keeps_a_table_a_move_range_ends_inside() {
    let body = format!(
        r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:rPr><w:moveTo w:id="1" {REV}/></w:rPr></w:pPr><w:moveToRangeStart w:id="2" {REV} w:name="move1"/><w:moveTo w:id="3" {REV}><w:r><w:t>Heading</w:t></w:r></w:moveTo></w:p>
<w:tbl><w:tblPr><w:tblW w:type="auto" w:w="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>
<w:tr><w:trPr><w:ins w:id="4" {REV}/></w:trPr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="2000"/></w:tcPr><w:moveToRangeEnd w:id="2"/><w:p><w:pPr><w:rPr><w:ins w:id="5" {REV}/></w:rPr></w:pPr><w:ins w:id="6" {REV}><w:r><w:t>New row</w:t></w:r></w:ins></w:p></w:tc></w:tr>
<w:tr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="2000"/></w:tcPr><w:p><w:r><w:t>Kept row</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>
<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:rPr><w:moveFrom w:id="7" {REV}/></w:rPr></w:pPr><w:moveFromRangeStart w:id="8" {REV} w:name="move1"/><w:moveFrom w:id="9" {REV}><w:r><w:t>Heading</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="8"/></w:p>
<w:p/>"#
    );
    let rejected = reject_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    for kept in ["<w:tblPr>", "<w:tblGrid>", "<w:tcW"] {
        assert!(xml.contains(kept), "{kept} lost:\n{xml}");
    }
    for mark in ["w:moveFrom", "w:moveTo", "w:ins ", "w:del "] {
        assert!(!xml.contains(mark), "{mark} left:\n{xml}");
    }
    assert_eq!(paragraphs(&rejected), ["Before", "Kept row", "Heading", ""]);
}

/// A footnotes part holding Word's two separators and footnote 1.
fn footnotes(xml: &str) -> Part<'_> {
    Part {
        name: "word/footnotes.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        xml,
    }
}

fn footnotes_xml(note: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:r><w:footnoteRef/></w:r>{note}</w:p></w:footnote></w:footnotes>"#
    )
}

/// R7: a note lives by its reference. Rejecting an inserted footnote
/// reference removes the footnote, leaving Word's separators (546a6e0c15,
/// 5620c6bac3).
#[test]
fn reject_drops_the_footnote_of_an_inserted_reference() {
    let body = format!(
        r#"<w:p><w:r><w:t>Claim</w:t></w:r><w:ins w:id="1" {REV}><w:r><w:footnoteReference w:id="1"/></w:r></w:ins></w:p>"#
    );
    let note = format!(r#"<w:ins w:id="2" {REV}><w:r><w:t>Source</w:t></w:r></w:ins>"#);
    let xml = footnotes_xml(&note);
    let rejected = reject_revisions(&docx_with(&body, &[footnotes(&xml)])).unwrap();
    assert_word_valid_package(&rejected);
    let notes = part_string(&rejected, "word/footnotes.xml").unwrap();
    assert!(
        notes.contains(r#"w:id="-1""#) && notes.contains(r#"w:id="0""#),
        "{notes}"
    );
    assert!(!notes.contains(r#"w:id="1""#), "footnote 1 kept:\n{notes}");
}

/// A one-column table: `tbl_pr`, then one row per text carrying `row_pr`;
/// `deleted` makes each cell's text and paragraph mark deleted.
fn table(tbl_pr: &str, row_pr: &str, texts: &[&str], deleted: bool) -> String {
    let rows: String = texts
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (mark, run) = if deleted {
                (
                    format!(r#"<w:pPr><w:rPr><w:del w:id="{}" {REV}/></w:rPr></w:pPr>"#, 50 + i),
                    format!(r#"<w:del w:id="{}" {REV}><w:r><w:delText>{t}</w:delText></w:r></w:del>"#, 70 + i),
                )
            } else {
                (String::new(), format!("<w:r><w:t>{t}</w:t></w:r>"))
            };
            format!(r#"<w:tr>{row_pr}<w:tc><w:tcPr><w:tcW w:type="dxa" w:w="4000"/></w:tcPr><w:p>{mark}{run}</w:p></w:tc></w:tr>"#)
        })
        .collect();
    format!(
        r#"<w:tbl><w:tblPr>{tbl_pr}</w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>{rows}</w:tbl>"#
    )
}

const BORDERS: &str =
    r#"<w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/></w:tblBorders>"#;

/// R8: in Word's model adjacent tables are one table unless a property only
/// a whole table carries (its style, float, direction) tells them apart.
/// Word's compare split one table in two (its redline of 72cc9f4ac6); its
/// Reject All puts them back together, and the second table's own borders
/// ride on its rows as tblPrEx.
#[test]
fn reject_rejoins_adjacent_tables_word_split() {
    let first = table(
        &format!(
            r#"<w:tblStyle w:val="TableGrid"/><w:tblW w:type="auto" w:w="0"/><w:tblPrChange w:id="1" {REV}><w:tblPr><w:tblW w:type="pct" w:w="5000"/></w:tblPr></w:tblPrChange>"#
        ),
        "",
        &["One"],
        false,
    );
    let second = table(
        &format!(r#"<w:tblW w:type="pct" w:w="5000"/>{BORDERS}"#),
        &format!(r#"<w:trPr><w:del w:id="2" {REV}/></w:trPr>"#),
        &["Two", "Three"],
        true,
    );
    let rejected = reject_revisions(&docx(&format!("{first}{second}<w:p/>"))).unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    assert_eq!(xml.matches("<w:tbl>").count(), 1, "{xml}");
    assert_eq!(paragraphs(&rejected), ["One", "Two", "Three", ""]);
    assert!(!xml.contains("tblStyle"), "{xml}");
    assert_eq!(xml.matches("<w:tblPrEx><w:tblBorders>").count(), 2, "{xml}");
}

/// R8's boundary: tables of different styles stay two tables (Word's own
/// Accept All and Reject All outputs keep 11 such pairs).
#[test]
fn accept_keeps_adjacent_tables_of_different_styles_apart() {
    let first = table(r#"<w:tblW w:type="auto" w:w="0"/>"#, "", &["One"], false);
    let second = table(
        r#"<w:tblStyle w:val="LightShading"/><w:tblW w:type="auto" w:w="0"/>"#,
        "",
        &["Two"],
        false,
    );
    let third = table(
        r#"<w:tblW w:type="auto" w:w="0"/>"#,
        &format!(r#"<w:trPr><w:del w:id="3" {REV}/></w:trPr>"#),
        &["Gone"],
        true,
    );
    let body = format!("{first}{second}<w:p/>{third}<w:p/>");
    let accepted = accept_revisions(&docx(&body)).unwrap();
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    assert_eq!(xml.matches("<w:tbl>").count(), 2, "{xml}");
    assert_eq!(paragraphs(&accepted), ["One", "Two", "", ""]);
}

fn header(name: &'static str, xml: &'static str) -> Part<'static> {
    Part {
        name,
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        xml,
    }
}

const HEADER_1: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>First header</w:t></w:r></w:p></w:hdr>"#;
const HEADER_2: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Last header</w:t></w:r></w:p></w:hdr>"#;

/// R9: Word's sectPrChange records a section's properties, never its header
/// and footer references (CT_SectPrBase has none). Rejecting the change keeps
/// the section's headers; Word's Reject All of 205503ead9 kept the last
/// section's header, while jubarte lost every reference.
#[test]
fn reject_of_a_section_change_keeps_its_header() {
    let refs = format!(
        r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:titlePg w:val="0"/><w:sectPrChange w:id="1" {REV}><w:sectPr><w:titlePg/></w:sectPr></w:sectPrChange>"#
    );
    let body = r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#;
    let rejected = reject_revisions(&docx_with_sect(
        body,
        &[header("word/header1.xml", HEADER_1)],
        &refs,
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    assert!(
        xml.contains(r#"<w:headerReference w:type="default" r:id="rIdX0""#),
        "{xml}"
    );
    assert!(
        xml.contains("<w:titlePg />") && !xml.contains("sectPrChange"),
        "{xml}"
    );
}

/// R9: Word's save keeps no header or footer part that no section
/// references. Rejecting an inserted section break takes its sectPr and so
/// its header's last reference (205503ead9: Word's output holds one header).
#[test]
fn reject_of_an_inserted_section_break_drops_its_header_part() {
    let body = format!(
        r#"<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/></w:sectPr><w:rPr><w:ins w:id="1" {REV}/></w:rPr></w:pPr><w:ins w:id="2" {REV}><w:r><w:t>New</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>Old</w:t></w:r></w:p>"#
    );
    let refs = r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#;
    let pkg = docx_with_sect(
        &body,
        &[
            header("word/header1.xml", HEADER_1),
            header("word/header2.xml", HEADER_2),
        ],
        refs,
    );
    let rejected = reject_revisions(&pkg).unwrap();
    assert_word_valid_package(&rejected);
    assert_eq!(paragraphs(&rejected), ["Old"]);
    assert!(
        part_string(&rejected, "word/header1.xml").is_none(),
        "header1 kept"
    );
    assert!(
        part_string(&rejected, "word/header2.xml").is_some(),
        "header2 lost"
    );
    let rels = part_string(&rejected, "word/_rels/document.xml.rels").unwrap();
    assert!(!rels.contains("header1.xml"), "{rels}");
    let types = part_string(&rejected, "[Content_Types].xml").unwrap();
    assert!(!types.contains("header1.xml"), "{types}");
}

/// A header part two relationships target stays while a section still
/// references one of them: dropping the unreferenced relationship must not
/// take the part the other one shows.
#[test]
fn reject_keeps_a_header_part_a_referenced_relationship_still_targets() {
    let body = r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#;
    let refs = r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#;
    let mut pkg = jubarte::opc::PartFs::open(&docx_with_sect(
        body,
        &[header("word/header1.xml", HEADER_1)],
        refs,
    ))
    .unwrap();
    let spare = pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        "header1.xml",
    );
    let rejected = reject_revisions(&pkg.to_zip().unwrap()).unwrap();
    assert_word_valid_package(&rejected);
    assert!(
        part_string(&rejected, "word/header1.xml").is_some(),
        "header1 lost"
    );
    let rels = part_string(&rejected, "word/_rels/document.xml.rels").unwrap();
    assert!(rels.contains(r#"Id="rIdX0""#), "{rels}");
    assert!(!rels.contains(&format!(r#"Id="{spare}""#)), "{rels}");
}
