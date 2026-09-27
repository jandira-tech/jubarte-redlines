// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Both documents write the same list under `numId` 1, but the original's
//! definition draws a circle bullet and the revised one a disc. The text is
//! unchanged. Word moves the paragraphs to a fresh `w:num` that carries the
//! revised definition and records the original `numId` in a `w:pPrChange`, so
//! the redline shows the circle struck and the disc inserted. Before this fix,
//! unchanged paragraphs kept `numId` 1, which resolves to the original's
//! circle, and no change was recorded.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx_bullets(bullet: &str) -> Vec<u8> {
    let items: String = ["First", "Second"]
        .iter()
        .map(|t| {
            format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{t}</w:t></w:r></w:p>"#
            )
        })
        .collect();
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{items}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let numbering = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="{bullet}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, body) in [
            ("[Content_Types].xml", &ct[..]),
            ("_rels/.rels", &root_rels[..]),
            ("word/_rels/document.xml.rels", &doc_rels[..]),
            ("word/document.xml", doc.as_bytes()),
            ("word/numbering.xml", numbering.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(body).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut s = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}

#[test]
fn unchanged_items_take_the_revised_list_with_a_ppr_change() {
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out =
        compare_documents_with_settings(&docx_bullets("o"), &docx_bullets("\u{2022}"), &settings)
            .unwrap();
    let doc = part(&out, "word/document.xml");
    let first = doc.split("</w:p>").next().unwrap();
    let live = first
        .split("<w:pPrChange")
        .next()
        .unwrap()
        .split("<w:numId w:val=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("live numId");
    assert_ne!(
        live, "1",
        "unchanged item still points at the original list: {first}"
    );
    let old = first
        .split("<w:pPrChange")
        .nth(1)
        .unwrap_or_else(|| panic!("no pPrChange: {first}"));
    assert!(
        old.contains("<w:numId w:val=\"1\""),
        "pPrChange keeps the original list: {first}"
    );
    let numbering = part(&out, "word/numbering.xml");
    let num = numbering
        .split(&format!("<w:num w:numId=\"{live}\""))
        .nth(1)
        .expect("live num defined");
    let abs = num
        .split("<w:abstractNumId w:val=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let def = numbering
        .split(&format!("w:abstractNumId=\"{abs}\""))
        .nth(1)
        .unwrap()
        .split("</w:abstractNum>")
        .next()
        .unwrap();
    assert!(
        def.contains("\u{2022}"),
        "live list should draw the revised disc: {def}"
    );
}
