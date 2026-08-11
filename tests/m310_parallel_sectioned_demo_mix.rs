// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M310 — parallel lettered-section demos (OOXML property testers).
//! Word meshes MIX line-by-line (A) with A), bullets with bullets).
//! Unrelated pure-I/D wholesale must not fire when both sides share ≥3
//! lettered section headers (unpacked oracle highlight×bold ~21 MIX).

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn word_settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn docx_paras(lines: &[&str]) -> Vec<u8> {
    let mut body = String::new();
    for t in lines {
        body.push_str(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">{t}</w:t></w:r></w:p>"#
        ));
    }
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
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

fn body_paragraphs(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end = after
            .find("</w:p>")
            .map(|i| i + "</w:p>".len())
            .unwrap_or(after.len());
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

#[test]
fn parallel_lettered_sections_mesh_not_pure_id_wholesale() {
    // Distinct property text but shared A) B) C) D) skeleton — Word MIX.
    let a = docx_paras(&[
        "OOXML w:highlight tester",
        "A) highlight values:",
        "  - w:highlight yellow: sample",
        "  - w:highlight green: sample",
        "B) rStyle only:",
        "  - style HighlightYellow: sample",
        "C) overrides:",
        "  - combo one",
        "D) linked styles:",
        "  - linked pair",
        "Table examples below",
        "row one highlight",
        "row two highlight",
    ]);
    let b = docx_paras(&[
        "OOXML w:b bold tester",
        "A) ST_OnOff values:",
        "  - w:b true: sample",
        "  - w:b false: sample",
        "B) rStyle only:",
        "  - style BoldOnly: sample",
        "C) overrides:",
        "  - combo bold",
        "D) linked styles:",
        "  - linked bold",
        "Table examples below",
        "row one bold",
        "row two bold",
    ]);
    let out = compare_documents_with_settings(&a, &b, &word_settings()).expect("compare");
    let xml = document_xml(&out);
    let paras = body_paragraphs(&xml);
    let mix = paras
        .iter()
        .filter(|p| p.contains("<w:ins") && p.contains("<w:del"))
        .count();
    let pure_i = paras
        .iter()
        .filter(|p| p.contains("<w:ins") && !p.contains("<w:del"))
        .count();
    let pure_d = paras
        .iter()
        .filter(|p| p.contains("<w:del") && !p.contains("<w:ins"))
        .count();
    assert!(
        mix >= 3,
        "expected Word-like MIX mesh on parallel sections; mix={mix} I={pure_i} D={pure_d} n={}",
        paras.len()
    );
    // Must not be pure wholesale I-block then D-block only.
    assert!(
        mix > 0 && !(pure_i >= 10 && pure_d >= 10 && mix == 0),
        "must not pure-I/D wholesale; mix={mix} I={pure_i} D={pure_d}"
    );
}
