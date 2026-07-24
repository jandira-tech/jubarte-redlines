// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M24 — Strict/ISO OOXML documents (namespace URIs under
//! `http://purl.oclc.org/ooxml/…`) must be handled, not crash with "no body".
//!
//! `ole-object.docx` is Strict OOXML: its root is
//! `<w:document xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main">`.
//! Our XName tables only model the Transitional namespace
//! (`http://schemas.openxmlformats.org/wordprocessingml/2006/main`), so
//! `merged_body` found no `w:body` and `compare_documents` panicked at
//! document_comparer.rs:133/134 ("original/modified has no body") —
//! image-inline-and-block_ole-object and ole-object_imageExistingMultiple.
//!
//! Fix: normalize Strict namespace URIs to Transitional before parsing.

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const STRICT_W: &str = "http://purl.oclc.org/ooxml/wordprocessingml/main";

fn build_strict_docx(text: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
        // Strict OOXML: w: namespace is the purl.oclc.org variant.
        let doc = format!(
            "<w:document xmlns:w=\"{STRICT_W}\"><w:body>\
               <w:p><w:r><w:t>{text}</w:t></w:r></w:p>\
               <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>"
        );
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#).unwrap();
        z.finish().unwrap();
    }
    buf
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn strict_ooxml_document_is_compared_not_crashed() {
    // Was: panic "original/modified has no body" (document_comparer.rs:133/134).
    let out = compare_documents(
        &build_strict_docx("Alphaword"),
        &build_strict_docx("Bravoword"),
        "Test",
    )
    .expect("Strict OOXML must compare, not panic with 'no body'");
    let x = read_part(&out, "word/document.xml");
    assert!(
        x.contains("<w:ins") && x.contains("<w:del"),
        "Strict doc text change must be redlined: {x}"
    );
    assert!(
        x.contains("Bravoword") && x.contains("Alphaword"),
        "both texts present: {x}"
    );
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable");
}
