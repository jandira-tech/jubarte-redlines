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
/// The properties of one style's `block` (`pPr`/`rPr`) in `styles_xml`, each
/// as `name(attr=value,…)` with the attributes sorted, in document order.
fn style_props(styles_xml: &str, id: &str, block: &str) -> Vec<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(styles_xml);
    let root = dom.root(doc).unwrap();
    let style = dom
        .elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some(id))
        .unwrap_or_else(|| panic!("no style {id}:\n{styles_xml}"));
    let Some(block) = dom.element(style, &W::name(block)) else {
        return Vec::new();
    };
    dom.elements(block, None)
        .into_iter()
        .map(|p| {
            let mut attrs: Vec<String> = dom
                .attributes(p)
                .into_iter()
                .map(|(a, v)| format!("{}={v}", a.local_name()))
                .collect();
            attrs.sort();
            format!("{}({})", dom.name(p).unwrap().local_name(), attrs.join(","))
        })
        .collect()
}

fn styles_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml,
    }
}

/// R27: a style change record holds the style's old properties ABSOLUTELY,
/// against Word's built-in defaults (Times New Roman, 10pt, single spacing,
/// widow control on): what the old style left unsaid was the built-in value,
/// not whatever the style chain says now. Rejecting the record writes each
/// property whose old value differs from what the style would inherit, and
/// only those, attribute by attribute for rFonts, lang and spacing. Read off
/// Word's Reject All of its own redlines: b42b3ae070 (Normal back to sz=20,
/// Times New Roman and single spacing exactly as the original has them, a
/// based List Paragraph gains jc=left over Normal's both), c719b900f0 (dropped
/// docDefaults restatements, header gains widowControl), 1b4dd65cb9,
/// 2288f27be1, 6fb9bbdb49 (lineRule stays with a different line).
#[test]
fn reject_of_a_style_record_restores_the_old_style_against_word_built_ins() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:widowControl w:val="0"/><w:spacing w:after="200" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/><w:pPrChange w:id="1" {REV}><w:pPr><w:widowControl w:val="0"/><w:spacing w:after="200" w:line="276" w:lineRule="auto"/><w:jc w:val="both"/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/><w:sz w:val="24"/><w:rPrChange w:id="2" {REV}><w:rPr><w:rFonts w:ascii="Aptos" w:eastAsiaTheme="minorEastAsia" w:hAnsi="Aptos" w:cstheme="minorBidi"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="es-ES" w:eastAsia="en-US" w:bidi="ar-SA"/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:pPrChange w:id="3" {REV}><w:pPr><w:keepNext/><w:spacing w:before="240"/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:sz w:val="32"/><w:rPrChange w:id="4" {REV}><w:rPr><w:b/><w:sz w:val="32"/><w:lang w:val="es-ES"/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Tight"><w:name w:val="Tight"/><w:pPr><w:spacing w:after="200" w:line="276" w:lineRule="auto"/><w:pPrChange w:id="5" {REV}><w:pPr><w:widowControl w:val="0"/><w:spacing w:after="200" w:line="280" w:lineRule="auto"/></w:pPr></w:pPrChange></w:pPr></w:style><w:style w:type="paragraph" w:styleId="Plain"><w:name w:val="Plain"/><w:rPr><w:sz w:val="22"/></w:rPr></w:style></w:styles>"#
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    assert!(!xml.contains("Change"), "record left:\n{xml}");
    // A root style inherits the docDefaults: their restatements go.
    assert_eq!(style_props(&xml, "Normal", "pPr"), ["jc(val=both)"]);
    assert_eq!(
        style_props(&xml, "Normal", "rPr"),
        ["rFonts(ascii=Aptos,hAnsi=Aptos)", "lang(val=es-ES)"]
    );
    // A based style: what the old record left unsaid was the built-in value.
    assert_eq!(
        style_props(&xml, "Heading1", "pPr"),
        [
            "keepNext()",
            "widowControl()",
            "spacing(after=0,before=240,line=240,lineRule=auto)",
            "jc(val=left)",
        ]
    );
    assert_eq!(
        style_props(&xml, "Heading1", "rPr"),
        [
            "rFonts(ascii=Times New Roman,cs=Times New Roman,eastAsia=Times New Roman,hAnsi=Times New Roman)",
            "b()",
            "sz(val=32)",
            "szCs(val=20)",
        ]
    );
    // lineRule stays with a line that differs; the docDefaults' after goes.
    assert_eq!(
        style_props(&xml, "Tight", "pPr"),
        ["spacing(line=280,lineRule=auto)"]
    );
    // A style without a record is left alone.
    assert_eq!(style_props(&xml, "Plain", "rPr"), ["sz(val=22)"]);
}

/// R28: Word keeps a linked pair in step. Rejecting a paragraph style's rPr
/// record gives its linked character style the same old rPr, replacing the
/// character style's own, each resolved against its own chain: the
/// paragraph style drops the color its parent holds, the character style
/// keeps it (Word's Reject All of its redlines of d8b0c2ae01, Heading 1 Char;
/// bf3d5eb650, Header Char; 3866f441cc, Comment Text Char). A character
/// style whose paragraph style kept its rPr is left alone.
#[test]
fn reject_of_a_paragraph_style_record_resyncs_its_linked_character_style() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:color w:val="000000"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:link w:val="Heading1Char"/><w:rPr><w:sz w:val="24"/><w:rPrChange w:id="1" {REV}><w:rPr><w:b/><w:color w:val="000000"/><w:kern w:val="32"/><w:sz w:val="32"/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="character" w:customStyle="1" w:styleId="Heading1Char"><w:name w:val="Heading 1 Char"/><w:link w:val="Heading1"/><w:rPr><w:color w:val="000000"/><w:sz w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:link w:val="Heading2Char"/><w:pPr><w:keepNext/><w:pPrChange w:id="2" {REV}><w:pPr/></w:pPrChange></w:pPr><w:rPr><w:sz w:val="28"/></w:rPr></w:style><w:style w:type="character" w:customStyle="1" w:styleId="Heading2Char"><w:name w:val="Heading 2 Char"/><w:link w:val="Heading2"/><w:rPr><w:color w:val="000000"/></w:rPr></w:style></w:styles>"#
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    assert_eq!(
        style_props(&xml, "Heading1", "rPr"),
        ["b()", "kern(val=32)", "sz(val=32)"]
    );
    assert_eq!(
        style_props(&xml, "Heading1Char", "rPr"),
        ["b()", "color(val=000000)", "kern(val=32)", "sz(val=32)"]
    );
    assert_eq!(
        style_props(&xml, "Heading2Char", "rPr"),
        ["color(val=000000)"]
    );
}

fn numbering_part(xml: &str) -> Part<'_> {
    Part {
        name: "word/numbering.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        xml,
    }
}

/// R29: a numbered style's restored indent and tabs go where they equal its
/// numbering level's, since the level supplies them anyway. The level is the
/// style's own numPr or the one it inherits, with a `lvlOverride` level
/// taking over the abstract one; tabs compare by value, not attribute order.
/// Only equal values go: a style's own tabs other than the level's stay, and
/// the level never adds a value the old record lacks. Read off Word's Reject
/// All of its own redlines: 2288f27be1 and 2e3f1e261d (List Bullet / List
/// Number 1-3 lose ind and the num tab), 512b24be1e (Bullets loses its
/// ind=170, keeps its own 252/284 tabs), f8c1ce3e92 (LAP Table Bullet).
#[test]
fn reject_of_a_numbered_style_record_drops_what_its_numbering_level_says() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let old = |num: &str, body: &str| {
        format!(
            r#"<w:pPr><w:numPr>{num}</w:numPr><w:contextualSpacing/><w:pPrChange w:id="1" {REV}><w:pPr><w:numPr>{num}</w:numPr>{body}<w:contextualSpacing/></w:pPr></w:pPrChange></w:pPr>"#
        )
    };
    let bullet = old(
        r#"<w:numId w:val="1"/>"#,
        r#"<w:tabs><w:tab w:pos="360" w:val="num"/></w:tabs><w:ind w:hanging="360" w:left="360"/>"#,
    );
    let own_tabs = old(
        r#"<w:numId w:val="1"/>"#,
        r#"<w:tabs><w:tab w:val="left" w:pos="252"/></w:tabs><w:ind w:left="360" w:hanging="360"/>"#,
    );
    let second = old(
        r#"<w:ilvl w:val="1"/><w:numId w:val="1"/>"#,
        r#"<w:ind w:left="720" w:hanging="360"/>"#,
    );
    let overridden = old(
        r#"<w:numId w:val="2"/>"#,
        r#"<w:spacing w:before="120"/><w:ind w:left="360" w:hanging="360"/>"#,
    );
    let derived = format!(
        r#"<w:pPr><w:pPrChange w:id="2" {REV}><w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr></w:pPrChange></w:pPr>"#
    );
    let style = |id: &str, based: &str, ppr: &str| {
        format!(
            r#"<w:style w:type="paragraph" w:styleId="{id}"><w:name w:val="{id}"/><w:basedOn w:val="{based}"/>{ppr}</w:style>"#
        )
    };
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>{}{}{}{}{}</w:styles>"#,
        style("ListBullet", "Normal", &bullet),
        style("OwnTabs", "Normal", &own_tabs),
        style("Second", "Normal", &second),
        style("Overridden", "Normal", &overridden),
        style("Derived", "ListBullet", &derived),
    );
    let level = |ilvl: u8, left: u16| {
        format!(
            r#"<w:lvl w:ilvl="{ilvl}"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="-"/><w:lvlJc w:val="left"/><w:pPr><w:tabs><w:tab w:val="num" w:pos="{left}"/></w:tabs><w:ind w:left="{left}" w:hanging="360"/></w:pPr></w:lvl>"#
        )
    };
    let numbering = format!(
        r#"<w:numbering xmlns:w="{w}"><w:abstractNum w:abstractNumId="0">{}{}</w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num><w:num w:numId="2"><w:abstractNumId w:val="0"/><w:lvlOverride w:ilvl="0">{}</w:lvlOverride></w:num></w:numbering>"#,
        level(0, 360),
        level(1, 720),
        level(0, 1080),
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles), numbering_part(&numbering)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    let pairs = [
        ("ListBullet", vec!["numPr()", "contextualSpacing()"]),
        ("OwnTabs", vec!["numPr()", "tabs()", "contextualSpacing()"]),
        ("Second", vec!["numPr()", "contextualSpacing()"]),
        (
            "Overridden",
            vec![
                "numPr()",
                "spacing(before=120)",
                "ind(hanging=360,left=360)",
                "contextualSpacing()",
            ],
        ),
        // Its old record leaves contextualSpacing unsaid, which is the
        // built-in off against ListBullet's on (R27).
        ("Derived", vec!["contextualSpacing(val=0)"]),
    ];
    for (id, want) in pairs {
        assert_eq!(style_props(&xml, id, "pPr"), want, "{id}");
    }
}

/// R30: Word reads an old record's toggle (b, i, caps, …) against the nearest
/// ancestor whose record the same reject restores: one that is on turns it
/// off, so the restored style writes it `val=0` (512b24be1e's Heading 1 and
/// Heading 3 over their restored Leaders Heading 1 / Heading 2). An ancestor
/// without a record does not count, a toggle the old record lacks takes the
/// built-in off, and a character style's record reads the same way. Read off
/// Word's Reject All of synthetic records (2026-09-29 probes).
#[test]
fn reject_of_a_style_record_reads_its_toggles_against_restored_ancestors() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/></w:style><w:style w:type="paragraph" w:styleId="GP"><w:name w:val="GP"/><w:basedOn w:val="Normal"/><w:rPr><w:b/><w:i/><w:rPrChange w:id="1" {REV}><w:rPr><w:b/><w:i/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Mid"><w:name w:val="Mid"/><w:basedOn w:val="GP"/></w:style><w:style w:type="paragraph" w:styleId="Kid"><w:name w:val="Kid"/><w:basedOn w:val="Mid"/><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="2" {REV}><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="paragraph" w:styleId="KOff"><w:name w:val="KOff"/><w:basedOn w:val="GP"/><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="3" {REV}><w:rPr><w:b w:val="0"/><w:i/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Plain"><w:name w:val="Plain"/><w:basedOn w:val="Normal"/><w:rPr><w:b/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="POn"><w:name w:val="POn"/><w:basedOn w:val="Plain"/><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="4" {REV}><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="character" w:styleId="CRec"><w:name w:val="CRec"/><w:basedOn w:val="DefaultParagraphFont"/><w:rPr><w:b/><w:rPrChange w:id="5" {REV}><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="character" w:styleId="CKid"><w:name w:val="CKid"/><w:basedOn w:val="CRec"/><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="6" {REV}><w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="character" w:styleId="CPlain"><w:name w:val="CPlain"/><w:basedOn w:val="DefaultParagraphFont"/><w:rPr><w:b/></w:rPr></w:style><w:style w:type="character" w:styleId="COn"><w:name w:val="COn"/><w:basedOn w:val="CPlain"/><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="7" {REV}><w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    let pairs: [(&str, &[&str]); 6] = [
        // b on over GP's restored b is off; the lacking i takes the built-in off.
        ("Kid", &["b(val=0)", "i(val=0)"]),
        // b off over GP's b is on, as inherited; i on over GP's i is off.
        ("KOff", &["i(val=0)"]),
        // Plain has no record: b on is on, as inherited.
        ("POn", &[]),
        ("GP", &["b()", "i()"]),
        // A character style's record, restored as recorded, reads the same way.
        ("CKid", &["b(val=0)", "sz(val=28)"]),
        ("COn", &["b()", "sz(val=28)"]),
    ];
    for (id, want) in pairs {
        assert_eq!(style_props(&xml, id, "rPr"), want, "{id}");
    }
}

/// R27 built-ins beyond the first set: suppressAutoHyphens off,
/// textAlignment auto and color auto, written where the chain says otherwise
/// (440c36d875's heading 2 over a restored Normal with suppressAutoHyphens
/// and textAlignment baseline; the color probe's Kid over a red parent).
#[test]
fn reject_of_a_style_record_writes_the_alignment_hyphenation_and_color_built_ins() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:textAlignment w:val="auto"/><w:pPrChange w:id="1" {REV}><w:pPr><w:suppressAutoHyphens/><w:textAlignment w:val="baseline"/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:color w:val="FF0000"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:pPrChange w:id="2" {REV}><w:pPr><w:keepNext/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:sz w:val="30"/><w:rPrChange w:id="3" {REV}><w:rPr><w:sz w:val="28"/></w:rPr></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    assert_eq!(
        style_props(&xml, "Normal", "pPr"),
        ["suppressAutoHyphens()", "textAlignment(val=baseline)"]
    );
    assert_eq!(
        style_props(&xml, "Heading2", "pPr"),
        [
            "keepNext()",
            "suppressAutoHyphens(val=0)",
            "textAlignment(val=auto)"
        ]
    );
    assert_eq!(
        style_props(&xml, "Heading2", "rPr"),
        ["color(val=auto)", "sz(val=28)"]
    );
}

/// R28, as Word writes it: a linked character style takes its paragraph
/// style's effective rPr, the paragraph style's own properties over its
/// restored basedOn chain, less what the docDefaults already say. Its own
/// chain is not consulted (f8c1ce3e92's Comment Subject Char keeps Comment
/// Text Char's fonts). A linked character style based on a resynced one is
/// resynced the same way (2288f27be1 and 2e3f1e261d's Comment Subject Char
/// gains Comment Text's szCs). 108 of 108 records of Word's Reject All of its
/// own redlines; a linked pair outside both is left alone.
#[test]
fn reject_resyncs_a_linked_character_style_to_its_paragraph_styles_effective_rpr() {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val="22"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:sz w:val="22"/><w:lang w:val="en-AU"/></w:rPr></w:style><w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/></w:style><w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/><w:basedOn w:val="Normal"/><w:link w:val="CommentTextChar"/><w:rPr><w:sz w:val="20"/><w:rPrChange w:id="1" {REV}><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type="character" w:customStyle="1" w:styleId="CommentTextChar"><w:name w:val="Comment Text Char"/><w:basedOn w:val="DefaultParagraphFont"/><w:link w:val="CommentText"/><w:rPr><w:sz w:val="20"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="CommentSubject"><w:name w:val="annotation subject"/><w:basedOn w:val="CommentText"/><w:link w:val="CommentSubjectChar"/><w:rPr><w:b/><w:bCs/></w:rPr></w:style><w:style w:type="character" w:customStyle="1" w:styleId="CommentSubjectChar"><w:name w:val="Comment Subject Char"/><w:basedOn w:val="CommentTextChar"/><w:link w:val="CommentSubject"/><w:rPr><w:b/><w:bCs/><w:sz w:val="20"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:link w:val="TitleChar"/><w:rPr><w:sz w:val="40"/></w:rPr></w:style><w:style w:type="character" w:customStyle="1" w:styleId="TitleChar"><w:name w:val="Title Char"/><w:basedOn w:val="DefaultParagraphFont"/><w:link w:val="Title"/><w:rPr><w:color w:val="FF0000"/><w:sz w:val="40"/></w:rPr></w:style></w:styles>"#
    );
    let rejected = reject_revisions(&docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[styles_part(&styles)],
    ))
    .unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/styles.xml").unwrap();
    let pairs: [(&str, &[&str]); 4] = [
        ("CommentText", &["rFonts(ascii=Arial)", "sz(val=20)"]),
        (
            "CommentTextChar",
            &["rFonts(ascii=Arial)", "sz(val=20)", "lang(val=en-AU)"],
        ),
        (
            "CommentSubjectChar",
            &[
                "rFonts(ascii=Arial)",
                "b()",
                "bCs()",
                "sz(val=20)",
                "lang(val=en-AU)",
            ],
        ),
        ("TitleChar", &["color(val=FF0000)", "sz(val=40)"]),
    ];
    for (id, want) in pairs {
        assert_eq!(style_props(&xml, id, "rPr"), want, "{id}");
    }
}
