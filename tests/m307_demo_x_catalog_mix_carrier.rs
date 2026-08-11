// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M307 — demo-style short doc × catalog-style short doc must keep a MIX
//! carrier on the body paragraph (Word oracle shape).
//!
//! Regression class: pure-I/D wholesale peels that split
//! `bold_underline_highlight_demo × book_catalog` from score 100 → ~48 by
//! emitting pure-ins body + pure-del empties with live `line=276` instead of
//! a single MIX paragraph (ins catalog text + del demo residual on one `w:p`).
//!
//! Shape under Word / ebf1a79 baseline:
//! - p0 pure-ins title (B)
//! - p1 MIX ins+del body
//! - trailing pure-del residual empties / demo body

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx_from_paragraphs(paras: &[&str]) -> Vec<u8> {
    let mut body = String::new();
    for t in paras {
        // Demo docs carry line=276 on the title; body bare.
        let ppr = if t.contains("Demo") || t.contains("Catalog") {
            r#"<w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr>"#
        } else {
            ""
        };
        body.push_str(&format!(
            "<w:p>{ppr}<w:r><w:t xml:space=\"preserve\">{t}</w:t></w:r></w:p>"
        ));
    }
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body>
</w:document>"#
    );
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in [
            (
                "[Content_Types].xml",
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_slice(),
            ),
            (
                "_rels/.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_slice(),
            ),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
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

/// Split body paragraphs (shallow).
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

fn has_ins(p: &str) -> bool {
    p.contains("<w:ins") || p.contains("w:ins ")
}
fn has_del(p: &str) -> bool {
    p.contains("<w:del") || p.contains("w:del ")
}
fn text_of(p: &str) -> String {
    let mut t = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:t") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:t>") else {
            break;
        };
        t.push_str(&content[..end]);
        rest = &content[end..];
    }
    t
}

#[test]
fn demo_x_catalog_body_is_mix_not_pure_id() {
    // Mirrors bold_underline_highlight_demo (3 paras) × book_catalog (2 paras).
    let a = docx_from_paragraphs(&[
        "Bold Underline Highlight Demo",
        "This document shows bold, underline, and yellow highlight combined.",
        "This triple combination is used for critical review annotations.",
    ]);
    let b = docx_from_paragraphs(&[
        "Book Catalog",
        "Title Author Genre The Great Gatsby F. Scott Fitzgerald Fiction",
    ]);
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let xml = document_xml(&out);
    let paras = body_paragraphs(&xml);
    assert!(
        paras.len() >= 3,
        "expected at least title + body + residual, got {} paras\n{xml}",
        paras.len()
    );

    // Title is pure-ins of B's catalog title.
    let p0 = &paras[0];
    assert!(has_ins(p0), "p0 must be inserted title: {p0}");
    assert!(
        text_of(p0).contains("Book Catalog"),
        "p0 text should be Book Catalog, got {:?}",
        text_of(p0)
    );

    // At least one body paragraph must be MIX (ins+del) carrying catalog body
    // text — not pure-ins body after a pure-del empty with line=276.
    let mix = paras.iter().find(|p| has_ins(p) && has_del(p));
    assert!(
        mix.is_some(),
        "expected a MIX carrier paragraph (ins+del on one w:p); pure-I/D wholesale \
         regressed bold_underline×book_catalog 100→48. paras={}",
        paras
            .iter()
            .enumerate()
            .map(|(i, p)| format!(
                "p{i}:ins={} del={} text={:?}",
                has_ins(p),
                has_del(p),
                text_of(p).chars().take(40).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let mix = mix.unwrap();
    assert!(
        text_of(mix).contains("Title") || text_of(mix).contains("Gatsby"),
        "MIX body should carry catalog content, got {:?}",
        text_of(mix)
    );

    // Forbidden regression shape: pure-del empty with live line=276 sitting
    // between pure-ins title and pure-ins body (audit pure-I/D split).
    for (i, p) in paras.iter().enumerate() {
        let empty_text = text_of(p).trim().is_empty();
        let pure_del = has_del(p) && !has_ins(p);
        let live_276 = p.contains(r#"w:line="276""#) || p.contains("w:line='276'");
        assert!(
            !(empty_text && pure_del && live_276 && i > 0 && i + 1 < paras.len()),
            "p{i}: pure-del empty with live line=276 mid-body is the 100→48 regression shape"
        );
    }
}
