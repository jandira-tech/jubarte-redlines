// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A wholesale replacement between unrelated documents keeps the story-final
//! paragraph mark: when both documents end in an empty paragraph, Word pairs
//! those two final marks and the redline ends on that live empty paragraph
//! (line_break × line_space_table). Unpaired, B's trailing empty insert was
//! welded onto A's first deleted paragraph, which then kept a live mark.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn docx(body: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        )
        .unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        write!(
            z,
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
        )
        .unwrap();
        z.finish().unwrap();
    }
    buf
}

fn p(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

#[test]
fn unrelated_replacement_pairs_the_final_empty_paragraphs() {
    let a: String = [
        "Annual business management report",
        "Comprehensive results analysis for the fiscal year",
        "Confidential document prepared by strategic analysis",
        "Executive summary of operations and revenue",
        "This document presents a comprehensive analysis of results",
        "Quarterly figures reflect performance across every region",
    ]
    .iter()
    .map(|t| p(t))
    .collect::<String>()
        + "<w:p/>";
    let b = format!(
        "<w:p/><w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"9000\" w:type=\"dxa\"/></w:tcPr>{}</w:tc></w:tr></w:tbl><w:p/>",
        p("PARTIES")
    );
    let out = compare_documents_with_settings(
        &docx(&a),
        &docx(&b),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let body = &xml[xml.find("<w:body").unwrap()..xml.find("<w:sectPr").unwrap()];
    let paras: Vec<&str> = body
        .split("</w:p>")
        .filter(|s| s.contains("<w:p"))
        .collect();

    let first_del = paras
        .iter()
        .find(|s| s.contains("Annual business"))
        .expect("A's first paragraph is present");
    assert!(
        !first_del.contains("<w:ins") && first_del.contains("<w:rPr><w:del"),
        "A's first paragraph is purely deleted, mark included: {first_del}"
    );
    let last = paras.last().unwrap();
    assert!(
        !last.contains("<w:ins") && !last.contains("<w:del") && !last.contains("<w:t"),
        "the redline ends on the paired, live, empty final paragraph: {last}"
    );
}
