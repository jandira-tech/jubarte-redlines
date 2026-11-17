// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Direct spacing of `line=276` restates a default only when the paragraph's
//! own source document resolves line 276 for an unstyled paragraph (its
//! Normal chain, then docDefaults). Word drops such restatements and keeps a
//! 276 that differs from its source's default: sd_2517_localized_heading_styles
//! writes `after=200 line=276` over a Normal of line 240, and Word keeps it in
//! all four pool pairs (30 inserted paragraphs each), while the demo titles'
//! bare `line=276` over a 276 docDefault go (orphan comment × yellow
//! highlight, over an original that defaults to single spacing). The strip
//! used to fire whatever the source defaults were.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx(body: &str, default_spacing: &str, normal_ppr: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:pPrDefault><w:pPr>{default_spacing}</w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/>{normal_ppr}</w:style></w:styles>"#
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

const SHARED: &str = r#"<w:p><w:r><w:t>The parties agree to the terms below.</w:t></w:r></w:p>"#;
const INSERTED: &str = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>An entirely new closing paragraph appears here.</w:t></w:r></w:p>"#;

fn inserted_paragraph(original_default: &str, revised_normal_ppr: &str) -> String {
    let original = docx(SHARED, original_default, "");
    let revised = docx(
        &format!("{SHARED}{INSERTED}"),
        r#"<w:spacing w:after="200" w:line="276" w:lineRule="auto"/>"#,
        revised_normal_ppr,
    );
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
    let at = xml.find("An entirely new").expect("inserted paragraph");
    let start = xml[..at]
        .rfind("<w:p>")
        .or_else(|| xml[..at].rfind("<w:p "))
        .unwrap();
    xml[start..at].to_string()
}

#[test]
fn line_276_over_a_single_spaced_normal_is_kept() {
    let p = inserted_paragraph(
        r#"<w:spacing w:after="160" w:line="278" w:lineRule="auto"/>"#,
        r#"<w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr>"#,
    );
    assert!(
        p.contains("w:line=\"276\""),
        "the revised Normal is single spaced, so 276 is a real value: {p}"
    );
}

#[test]
fn line_276_restating_the_revised_default_is_dropped() {
    let p = inserted_paragraph("", "");
    assert!(
        !p.contains("w:line=\"276\""),
        "the 276 only restates the revised docDefaults: {p}"
    );
}
