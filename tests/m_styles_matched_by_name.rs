// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word pairs the two stylesheets by style NAME, not by id. A Dutch original
//! stores Normal as `Standaard` and Default Paragraph Font as
//! `Standaardalinea-lettertype`; a Brazilian revision stores them as `Normal`
//! and `Fontepargpadro`. Word's redline keeps one style per name under the
//! canonical id (`Normal`, `DefaultParagraphFont`) and folds the revision's
//! effective Normal into it with a tracked change. Copying B's styles by id
//! left two styles named "Normal": the merge then rewrote B's bare copy and
//! the real default kept A's Verdana and 280 atLeast pitch, so every accepted
//! page rendered in the original's metrics (6fb9bbdb49: 6 pages, Word 7).

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx(styles_body: &str, text: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{styles_body}</w:styles>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, body) in [
            ("[Content_Types].xml", &ct[..]),
            ("_rels/.rels", &root_rels[..]),
            ("word/_rels/document.xml.rels", &doc_rels[..]),
            ("word/document.xml", doc.as_bytes()),
            ("word/styles.xml", styles.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(body).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

const DUTCH: &str = r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Arial" w:eastAsiaTheme="minorHAnsi" w:hAnsi="Arial" w:cstheme="minorBidi"/><w:lang w:val="nl-NL" w:eastAsia="en-US" w:bidi="ar-SA"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:line="300" w:lineRule="atLeast"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Standaard"><w:name w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:line="280" w:lineRule="atLeast"/></w:pPr><w:rPr><w:rFonts w:ascii="Verdana" w:hAnsi="Verdana"/></w:rPr></w:style><w:style w:type="character" w:default="1" w:styleId="Standaardalinea-lettertype"><w:name w:val="Default Paragraph Font"/><w:uiPriority w:val="1"/><w:semiHidden/><w:unhideWhenUsed/></w:style>"#;

const BRAZILIAN: &str = r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="pt-BR" w:eastAsia="en-US" w:bidi="ar-SA"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="character" w:default="1" w:styleId="Fontepargpadro"><w:name w:val="Default Paragraph Font"/><w:uiPriority w:val="1"/><w:semiHidden/><w:unhideWhenUsed/></w:style>"#;

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut s = String::new();
    zip.by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    s
}

/// Each `<w:style …>…</w:style>` block with its (type, styleId, name).
fn styles(xml: &str) -> Vec<(String, String, String, String)> {
    let attr = |tag: &str, name: &str| {
        let key = format!("{name}=\"");
        tag.find(&key).map(|i| {
            let rest = &tag[i + key.len()..];
            rest[..rest.find('"').unwrap()].to_string()
        })
    };
    xml.match_indices("<w:style ")
        .map(|(i, _)| {
            let end = xml[i..].find("</w:style>").unwrap() + i + "</w:style>".len();
            let block = &xml[i..end];
            let open = &block[..block.find('>').unwrap()];
            let name = block
                .find("<w:name ")
                .and_then(|n| attr(&block[n..], "w:val"))
                .unwrap_or_default();
            (
                attr(open, "w:type").unwrap_or_default(),
                attr(open, "w:styleId").unwrap_or_default(),
                name,
                block.to_string(),
            )
        })
        .collect()
}

#[test]
fn styles_with_the_same_name_merge_under_the_canonical_id() {
    let out = compare_documents(
        &docx(DUTCH, "Hello world."),
        &docx(BRAZILIAN, "Hello brave world."),
        "Redline",
    )
    .expect("compare ok");
    let all = styles(&styles_xml(&out));
    let named = |ty: &str, name: &str| {
        all.iter()
            .filter(|(t, _, n, _)| t == ty && n == name)
            .map(|(_, id, _, _)| id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(named("paragraph", "Normal"), ["Normal"]);
    assert_eq!(
        named("character", "Default Paragraph Font"),
        ["DefaultParagraphFont"]
    );
}

#[test]
fn the_default_paragraph_style_takes_the_revised_metrics() {
    let out = compare_documents(
        &docx(DUTCH, "Hello world."),
        &docx(BRAZILIAN, "Hello brave world."),
        "Redline",
    )
    .expect("compare ok");
    let all = styles(&styles_xml(&out));
    let (_, _, _, normal) = all
        .iter()
        .find(|(t, _, n, _)| t == "paragraph" && n == "Normal")
        .expect("a Normal style");
    // Word: <w:spacing w:after="160" w:line="259" w:lineRule="auto"/> with
    // the original's line=280 atLeast in the pPrChange; B's theme fonts,
    // 11pt and pt-BR over the original's Verdana in the rPrChange.
    for want in [
        r#"w:after="160""#,
        r#"w:line="259""#,
        r#"w:lineRule="auto""#,
        "<w:pPrChange",
        r#"w:asciiTheme="minorHAnsi""#,
        r#"<w:sz w:val="22""#,
        "<w:rPrChange",
    ] {
        assert!(normal.contains(want), "Normal lacks {want}: {normal}");
    }
}

/// The revision's own styles still arrive, and their `basedOn` follows the
/// pairing: a Korean revision's `Quote` based on `a` (its Normal) is based
/// on the output's `Normal` (8836f9bbdb, 73105518ef). Left on `a`, the
/// copied chain dangled and every quote lost the revision's fonts and
/// spacing: one accepted page grew to two.
#[test]
fn copied_styles_are_based_on_the_paired_style() {
    let korean = r#"<w:style w:type="paragraph" w:default="1" w:styleId="a"><w:name w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="0"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="QuoteKo"><w:name w:val="Quote"/><w:basedOn w:val="a"/><w:next w:val="a"/><w:rPr><w:i/></w:rPr></w:style>"#;
    let english = r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>"#;
    let out = compare_documents(
        &docx(english, "Hello world."),
        &docx(korean, "Hello brave world."),
        "Redline",
    )
    .expect("compare ok");
    let all = styles(&styles_xml(&out));
    let (_, _, _, quote) = all
        .iter()
        .find(|(t, _, n, _)| t == "paragraph" && n == "Quote")
        .expect("the revision's Quote is copied");
    assert!(quote.contains(r#"<w:basedOn w:val="Normal""#), "{quote}");
    assert!(quote.contains(r#"<w:next w:val="Normal""#), "{quote}");
    assert!(
        !all.iter().any(|(_, id, _, _)| id == "a"),
        "no second Normal: {all:?}"
    );
}
