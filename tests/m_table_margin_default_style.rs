// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's Compare stamps `w:tblInd w=10` and `w:tblCellMar` left/right 10 on a
//! bordered table only when the document the table comes from has no default
//! table style. Across the pool corpus's Word redlines, 94 bordered tables from
//! documents with a `TableNormal` default carry no synthesized indent. The 19
//! tables whose source lacks one all carry it (file_46_file_47: the stamped
//! indent shifted every row below the table).

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const TABLE_NORMAL: &str = r#"<w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>"#;

fn docx(body: &str, table_normal: bool) -> Vec<u8> {
    let w = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{w}"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>"#
    );
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>{}</w:styles>"#,
        if table_normal { TABLE_NORMAL } else { "" }
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, part) in [
            ("[Content_Types].xml", &ct[..]),
            ("_rels/.rels", &root_rels[..]),
            ("word/_rels/document.xml.rels", &doc_rels[..]),
            ("word/document.xml", doc.as_bytes()),
            ("word/styles.xml", styles.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(part).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

const TABLE: &str = r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="2500"/><w:gridCol w:w="2500"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>alpha cell</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>beta cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

/// The redline's first `w:tblPr`, serialized.
fn first_tblpr(original: &[u8], revised: &[u8]) -> String {
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(original, revised, &settings).unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let at = xml.find("<w:tblPr>").expect("table in redline");
    let end = xml[at..].find("</w:tblPr>").expect("tblPr end") + at;
    xml[at..end].to_string()
}

#[test]
fn table_from_document_with_default_table_style_keeps_bare_indent() {
    let a = docx(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>"#, true);
    let b = docx(
        &format!(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>{TABLE}"#),
        true,
    );
    let tblpr = first_tblpr(&a, &b);
    assert!(!tblpr.contains("tblInd"), "no synthesized indent: {tblpr}");
    assert!(
        !tblpr.contains("tblCellMar"),
        "no synthesized margins: {tblpr}"
    );
}

#[test]
fn table_from_document_without_default_table_style_gains_word_margins() {
    let a = docx(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>"#, true);
    let b = docx(
        &format!(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>{TABLE}"#),
        false,
    );
    let tblpr = first_tblpr(&a, &b);
    assert!(
        tblpr.contains(r#"<w:tblInd w:w="10" w:type="dxa" />"#),
        "inserted table from the style-less revised document: {tblpr}"
    );
}

#[test]
fn deleted_table_follows_the_original_document_styles() {
    let a = docx(
        &format!(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>{TABLE}"#),
        false,
    );
    let b = docx(r#"<w:p><w:r><w:t>intro text</w:t></w:r></w:p>"#, true);
    let tblpr = first_tblpr(&a, &b);
    assert!(
        tblpr.contains(r#"<w:tblInd w:w="10" w:type="dxa" />"#),
        "deleted table from the style-less original document: {tblpr}"
    );
}
