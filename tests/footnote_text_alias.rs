// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A localized style named Footnote Text stays itself when the document
//! already defines a different FootnoteText. The round trip keeps that
//! style justified and not bold.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";

/// `LocalNote` is Footnote Text and bold. `FootnoteText` is Notes body and
/// justified. The body uses the real style; the footnote uses the localized one.
fn package() -> Vec<u8> {
    let (w, r) = (common::docx::W_NS, common::docx::R_NS);
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="Notes body"/><w:basedOn w:val="Normal"/><w:pPr><w:jc w:val="both"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="LocalNote"><w:name w:val="Footnote Text"/><w:basedOn w:val="Normal"/><w:rPr><w:b/></w:rPr></w:style></w:styles>"#
    );
    let body = r#"<w:p><w:pPr><w:pStyle w:val="FootnoteText"/></w:pPr><w:r><w:t>Body notes.</w:t></w:r></w:p><w:p><w:r><w:t>Clause.</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#;
    let notes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="{w}" xmlns:r="{r}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:pPr><w:pStyle w:val="LocalNote"/></w:pPr><w:r><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> The note.</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let rels = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let styles_rel = format!("{rels}/styles");
    let notes_rel = format!("{rels}/footnotes");
    let parts = [
        common::docx::Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: &notes_rel,
            xml: &notes,
        },
        common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: &styles,
        },
    ];
    common::docx::docx_with_sect(body, &parts, "")
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

fn style_definition(docx: &[u8], id: &str) -> String {
    part(docx, "word/styles.xml")
        .split("<w:style ")
        .find(|style| style.contains(&format!(r#"w:styleId="{id}""#)))
        .unwrap_or("")
        .to_string()
}

fn footnote_style(docx: &[u8]) -> String {
    part(docx, "word/footnotes.xml")
        .split("<w:footnote ")
        .skip(1)
        .filter(|note| !note.contains("w:type="))
        .map(|note| {
            let paragraph = &note[..note.find("</w:p>").unwrap_or(note.len())];
            paragraph
                .split("<w:pStyle w:val=\"")
                .nth(1)
                .and_then(|value| value.split('"').next())
                .unwrap_or("-")
                .to_string()
        })
        .next()
        .unwrap_or_default()
}

#[test]
fn a_real_footnote_text_keeps_its_justification() {
    let source = package();
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let view = read(&source, &options).expect("the docx reads").markdown;
    let again = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    let definition = style_definition(&again, "FootnoteText");
    assert!(
        definition.contains(r#"<w:jc w:val="both"/>"#),
        "real FootnoteText was rewritten from the localized style: {definition}"
    );
    assert!(
        !definition.contains("<w:b/>") && !definition.contains("<w:b "),
        "real FootnoteText took the localized bold: {definition}"
    );
    assert_eq!(
        footnote_style(&again),
        "LocalNote",
        "localized footnote style was printed as the canonical id"
    );
    let localized = style_definition(&again, "LocalNote");
    assert!(
        localized.contains("<w:b/>") || localized.contains("<w:b "),
        "localized style lost its bold: {localized}"
    );
    common::validity::assert_word_valid_package(&again);
}
