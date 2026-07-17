// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C5 — formatting-only residue: rPrChange when run properties differ.
//!
//! Synthetic: same text, A plain vs B bold. Word records the formatting
//! change as `w:rPrChange` (or at least bold on the surviving run with
//! tracked history). We pin that a formatting-only pair emits revision
//! markup and does not invent extra body paragraphs.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        detect_format_changes: true,
        ..WmlComparerSettings::default()
    }
}

fn docx_run(text: &str, bold: bool) -> Vec<u8> {
    let rpr = if bold { "<w:rPr><w:b/></w:rPr>" } else { "" };
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r>{rpr}<w:t xml:space="preserve">{text}</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(ct).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(root_rels).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn bold_only_change_emits_format_revision_or_bold_markup() {
    let a = docx_run("Same title text", false);
    let b = docx_run("Same title text", true);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let xml = document_xml(&out);
    // Formatting-only: body should not explode into many paragraphs.
    let p_count = xml.matches("<w:p").count();
    assert!(
        p_count <= 3,
        "formatting-only pair must not invent many paragraphs; p_count={p_count}"
    );
    // Word records rPrChange; some paths emit ins/del of runs with rPr.
    let has_rpr_change = xml.contains("rPrChange");
    let has_bold = xml.contains("<w:b") || xml.contains("w:b ");
    let has_rev = xml.contains("<w:ins") || xml.contains("<w:del") || has_rpr_change;
    assert!(
        has_rev && has_bold,
        "expected format revision + bold on output (rPrChange={has_rpr_change} bold={has_bold}): {}",
        &xml[..xml.len().min(800)]
    );
}
