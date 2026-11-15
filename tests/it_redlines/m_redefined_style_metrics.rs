// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A redefined paragraph style carries the revised document's effective
//! metrics as a delta against the output context. The output keeps the
//! original's docDefaults (theme fonts, line 278), so the revised Heading1,
//! which inherits Arial and line 276 from its own docDefaults, must state them.
//! Word writes exactly the values that differ from what the output chain
//! resolves (mined over the 747 pool redlines). Heading1 used to take only the
//! revised style's own spacing and size and rendered in the original's fonts
//! and line pitch.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx(doc_defaults: &str, heading_ppr: &str) -> Vec<u8> {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p><w:p><w:r><w:t>Body text.</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#;
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{doc_defaults}<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/>{heading_ppr}<w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:sz w:val="40"/></w:rPr></w:style></w:styles>"#
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

#[test]
fn redefined_heading_states_the_revised_defaults_it_inherits() {
    let original = docx(
        r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="278" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>"#,
        r#"<w:spacing w:before="360" w:after="80"/>"#,
    );
    let revised = docx(
        r#"<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Arial" w:eastAsia="Arial" w:hAnsi="Arial" w:cs="Arial"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>"#,
        r#"<w:spacing w:before="400" w:after="120"/>"#,
    );
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(&original, &revised, &settings).unwrap();
    let mut styles = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut styles)
        .unwrap();
    let heading = styles
        .split("w:styleId=\"Heading1\"")
        .nth(1)
        .and_then(|s| s.split("</w:style>").next())
        .expect("Heading1");
    let live_ppr = heading.split("<w:pPrChange").next().unwrap();
    assert!(
        live_ppr.contains("w:line=\"276\""),
        "Heading1 should state the revised line 276: {heading}"
    );
    let live_rpr = heading
        .split("<w:rPr>")
        .nth(1)
        .and_then(|s| s.split("<w:rPrChange").next())
        .unwrap_or("");
    assert!(
        live_rpr.contains("w:ascii=\"Arial\""),
        "Heading1 should state the revised Arial: {heading}"
    );
}
