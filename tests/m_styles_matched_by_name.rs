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
    docx_body(
        styles_body,
        &format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"),
    )
}

fn docx_body(styles_body: &str, body: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
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

/// A style id names nothing across documents: a Russian original's `a3` is
/// Normal (Web) while its revision's `a3` is "Прижатый влево" and its
/// Normal (Web) is `ab` (1b4d). The redefined Normal (Web) takes the
/// revision's Normal (Web) metrics; reading B's `a3` gave it 12pt complex
/// script Times and dropped the 100/100 spacing.
#[test]
fn redefined_styles_read_the_revision_style_of_the_same_name() {
    let normal = r#"<w:style w:type="paragraph" w:default="1" w:styleId="a"><w:name w:val="Normal"/><w:qFormat/></w:style>"#;
    let a = format!(
        r#"{normal}<w:style w:type="paragraph" w:styleId="a3"><w:name w:val="Normal (Web)"/><w:basedOn w:val="a"/><w:rPr><w:sz w:val="22"/></w:rPr></w:style>"#
    );
    let b = format!(
        r#"{normal}<w:style w:type="paragraph" w:styleId="a3"><w:name w:val="Прижатый влево"/><w:basedOn w:val="a"/><w:rPr><w:rFonts w:cs="Times New Roman"/><w:sz w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="ab"><w:name w:val="Normal (Web)"/><w:basedOn w:val="a"/><w:pPr><w:spacing w:before="100" w:beforeAutospacing="1" w:after="100" w:afterAutospacing="1"/></w:pPr><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/><w:sz w:val="26"/></w:rPr></w:style>"#
    );
    let para = |id: &str, text: &str| {
        format!(r#"<w:p><w:pPr><w:pStyle w:val="{id}"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
    };
    let out = compare_documents(
        &docx_body(&a, &para("a3", "Hello world.")),
        &docx_body(
            &b,
            &format!(
                "{}{}",
                para("ab", "Hello brave world."),
                para("a3", "Left.")
            ),
        ),
        "Redline",
    )
    .expect("compare ok");
    let all = styles(&styles_xml(&out));
    let web: Vec<_> = all
        .iter()
        .filter(|(t, _, n, _)| t == "paragraph" && n == "Normal (Web)")
        .collect();
    assert_eq!(web.len(), 1, "one Normal (Web): {all:?}");
    let live = web[0].3.split("<w:rPrChange").next().unwrap();
    let live_ppr = web[0].3.split("<w:pPrChange").next().unwrap();
    for want in [r#"w:after="100""#, r#"w:before="100""#] {
        assert!(
            live_ppr.contains(want),
            "Normal (Web) lacks {want}: {}",
            web[0].3
        );
    }
    for want in [r#"w:ascii="Times New Roman""#, r#"<w:sz w:val="26""#] {
        assert!(
            live.contains(want),
            "Normal (Web) lacks {want}: {}",
            web[0].3
        );
    }
    assert!(!live.contains(r#"<w:sz w:val="24""#), "{}", web[0].3);
}

/// A stylesheet whose docDefaults has no pPrDefault gets Word's built-in
/// paragraph defaults, 160 after and 278 auto lines, while a present empty
/// pPrDefault means single spacing. Word's redline writes the original's
/// built-in defaults out and gives Normal the revision's single spacing
/// with a tracked change (221577c35b: the accepted page ran onto a second
/// one at 160/278).
#[test]
fn a_missing_paragraph_default_is_words_built_in_spacing() {
    let a = r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/><w:sz w:val="20"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#;
    let b = r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/></w:rPr></w:rPrDefault><w:pPrDefault/></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="a"><w:name w:val="Normal"/><w:qFormat/><w:rPr><w:sz w:val="18"/></w:rPr></w:style>"#;
    let out = compare_documents(
        &docx(a, "Hello world."),
        &docx(b, "Hello brave world."),
        "Redline",
    )
    .expect("compare ok");
    let xml = styles_xml(&out);
    let dd = &xml[xml.find("<w:docDefaults").unwrap()..xml.find("</w:docDefaults>").unwrap()];
    let ppr_default = &dd[dd.find("<w:pPrDefault").expect("a paragraph default")..];
    for want in [r#"w:after="160""#, r#"w:line="278""#] {
        assert!(ppr_default.contains(want), "pPrDefault lacks {want}: {dd}");
    }
    let all = styles(&xml);
    let (_, _, _, normal) = all
        .iter()
        .find(|(t, _, n, _)| t == "paragraph" && n == "Normal")
        .expect("a Normal style");
    let live = normal.split("<w:pPrChange").next().unwrap();
    for want in [r#"w:after="0""#, r#"w:line="240""#] {
        assert!(live.contains(want), "Normal lacks {want}: {normal}");
    }
    assert!(normal.contains("<w:pPrChange"), "{normal}");
}
