// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A revised document without any `w:sectPr` opens in Word with Word's
//! default section: Letter, one-inch margins, one column. Word's redline
//! makes that section live and records the original's two-column section in
//! a `w:sectPrChange`. The body section used to keep the original's two
//! columns with no change recorded.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx(body: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, body) in [
            ("[Content_Types].xml", &ct[..]),
            ("_rels/.rels", &root_rels[..]),
            ("word/document.xml", doc.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(body).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

#[test]
fn revised_without_sectpr_takes_word_default_section() {
    let original = docx(
        r#"<w:p><w:r><w:t>Two columns of text.</w:t></w:r></w:p><w:sectPr><w:type w:val="continuous"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/><w:cols w:num="2" w:space="720"/></w:sectPr>"#,
    );
    let revised = docx(r#"<w:p><w:r><w:t>Document without sectPr</w:t></w:r></w:p>"#);
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(&original, &revised, &settings).unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let body_sect = &xml[xml.rfind("<w:sectPr").expect("body sectPr")..];
    // rfind lands on the nested old sectPr when a change exists; take the
    // outer one instead.
    let outer = xml[..xml.rfind("<w:sectPrChange").unwrap_or(xml.len())]
        .rfind("<w:sectPr")
        .map(|i| &xml[i..])
        .unwrap_or(body_sect);
    let live = outer.split("<w:sectPrChange").next().unwrap();
    assert!(
        !live.contains("w:num=\"2\""),
        "live body section kept the original's two columns: {outer}"
    );
    let old = outer
        .split("<w:sectPrChange")
        .nth(1)
        .unwrap_or_else(|| panic!("no sectPrChange: {outer}"));
    assert!(
        old.contains("w:num=\"2\""),
        "change should record the two columns: {outer}"
    );
}
