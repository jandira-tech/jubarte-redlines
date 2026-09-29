// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What did not change stays unmarked in a Word-mode redline. Word's own
//! redline of fixtures_500 00b81efae883 ("Overall purpose of the post" →
//! "Overall aim of the post" in a text box) holds no property change and no
//! footer revision; ours struck and reinserted an unchanged footer and wrote
//! three `w:pPrChange`s. Each probe here was redlined by Word too, and Word
//! left the properties live with no change record.

mod common;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

use common::docx::{Part, docx_with_sect, para, part_string};
use common::validity::assert_word_valid_package;

const FOOTER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const HEADER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

fn redline(a: &[u8], b: &[u8]) -> Vec<u8> {
    let settings = WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(a, b, &settings).expect("compare");
    assert_word_valid_package(&out);
    out
}

fn story(root: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{root} xmlns:w="{}">{body}</w:{root}>"#,
        common::docx::W_NS
    )
}

/// A paragraph with space before and an indent, as Word writes a text-box
/// heading.
fn spaced(word: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:spacing w:before="120"/><w:ind w:left="108"/></w:pPr><w:r><w:t>Overall {word} of the post</w:t></w:r></w:p>"#
    )
}

fn assert_spacing_stays_live(xml: &str) {
    assert!(
        !xml.contains("pPrChange"),
        "no property changed, so none is recorded: {xml}"
    );
    assert!(
        xml.contains(r#"<w:spacing w:before="120" />"#)
            || xml.contains(r#"<w:spacing w:before="120"/>"#),
        "the paragraph keeps its space before: {xml}"
    );
}

/// Two sections, each with its own default footer: section 1's reads
/// "Page 1 of 4", section 2's "Page 4 of 4". Neither changed, so neither may
/// carry a revision; pairing footers by kind and type alone matched section
/// 1's footer against section 2's.
#[test]
fn each_section_keeps_its_own_unchanged_footer() {
    let footer = |n: &str| story("ftr", &para(&format!("Page {n} of 4")));
    let (f1, f2) = (footer("1"), footer("4"));
    let parts = [
        Part {
            name: "word/footer1.xml",
            content_type: FOOTER,
            rel_type: FOOTER_REL,
            xml: &f1,
        },
        Part {
            name: "word/footer2.xml",
            content_type: FOOTER,
            rel_type: FOOTER_REL,
            xml: &f2,
        },
    ];
    let body = |word: &str| {
        format!(
            r#"{}<w:p><w:pPr><w:sectPr><w:footerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:pPr></w:p>{}"#,
            para(&format!("Overall {word} of the post")),
            para("Second section.")
        )
    };
    let refs = r#"<w:footerReference w:type="default" r:id="rIdX1"/>"#;
    let a = docx_with_sect(&body("purpose"), &parts, refs);
    let b = docx_with_sect(&body("aim"), &parts, refs);
    let out = redline(&a, &b);
    for name in ["word/footer1.xml", "word/footer2.xml"] {
        let xml = part_string(&out, name).expect("footer");
        assert!(
            !xml.contains("<w:ins") && !xml.contains("<w:del"),
            "{name} did not change: {xml}"
        );
    }
    let doc = part_string(&out, "word/document.xml").unwrap();
    assert!(
        doc.contains("<w:ins") && doc.contains("<w:del"),
        "the body change is still marked: {doc}"
    );
}

/// The last paragraph of the body with one word revised keeps its own
/// properties live; only a paragraph replaced whole parks them (file_139).
#[test]
fn a_revised_word_in_the_last_paragraph_keeps_its_spacing() {
    let body = |word: &str| format!("{}{}", para("Intro paragraph here."), spaced(word));
    let out = redline(
        &docx_with_sect(&body("purpose"), &[], ""),
        &docx_with_sect(&body("aim"), &[], ""),
    );
    assert_spacing_stays_live(&part_string(&out, "word/document.xml").unwrap());
}

/// The same in a header's last paragraph.
#[test]
fn a_revised_word_in_a_header_keeps_its_spacing() {
    let header = |word: &str| story("hdr", &spaced(word));
    let doc = |word: &str| {
        let h = header(word);
        docx_with_sect(
            &para("Intro paragraph here."),
            &[Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &h,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        )
    };
    let out = redline(&doc("purpose"), &doc("aim"));
    let xml = part_string(&out, "word/header1.xml").unwrap();
    assert!(
        xml.contains("<w:ins") && xml.contains("<w:del"),
        "the header change is marked: {xml}"
    );
    assert_spacing_stays_live(&xml);
}

/// An unchanged justified paragraph beside a revised one records no property
/// change; M454's empty `w:pPrChange` belongs to a paragraph that gained its
/// alignment (center_alignment_2's title).
#[test]
fn an_unchanged_justified_paragraph_records_no_property_change() {
    let justified = r#"<w:p><w:pPr><w:jc w:val="both"/></w:pPr></w:p><w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:t>Justified text stays.</w:t></w:r></w:p>"#;
    let body = |word: &str| {
        format!(
            "{}{justified}{}",
            para(&format!("Overall {word} of the post")),
            para("After paragraph here.")
        )
    };
    let out = redline(
        &docx_with_sect(&body("purpose"), &[], ""),
        &docx_with_sect(&body("aim"), &[], ""),
    );
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(xml.contains("<w:ins"), "the body change is marked: {xml}");
    assert!(
        !xml.contains("pPrChange"),
        "no paragraph's properties changed: {xml}"
    );
}
