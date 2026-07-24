// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M21 — header/footer CONTENT must be redlined (Word redlines header/footer
//! changes; we only copied the original's). Word redlines header parts in 30 of
//! the 100 benchmark pairs and footers in 83. We diff footnotes/endnotes but not
//! headers/footers. Match A's↔B's parts by sectPr header/footerReference
//! (kind+type) and diff their content. (v1: text-only parts — no relationship
//! refs — to avoid dangling refs in the redlined part.)

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn build_docx(doc_xml: &str, rels: &[(&str, &str, &str)], extra: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc_xml.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        let mut r = String::from(
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for (id, ty, tg) in rels {
            r.push_str(&format!(
                r#"<Relationship Id="{id}" Type="{ty}" Target="{tg}"/>"#
            ));
        }
        r.push_str("</Relationships>");
        z.write_all(r.as_bytes()).unwrap();
        for (name, content) in extra {
            z.start_file(*name, opt).unwrap();
            z.write_all(content.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf
}

/// Document whose body is identical, but whose default header carries `header_text`.
fn doc_with_header(header_text: &str) -> Vec<u8> {
    let body = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:r=\"{REL_NS}\"><w:body>\
         <w:p><w:r><w:t>shared body</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rId1\"/>\
         <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>"
    );
    let header =
        format!("<w:hdr xmlns:w=\"{W_NS}\"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>");
    build_docx(
        &body,
        &[("rId1", &format!("{REL_NS}/header"), "header1.xml")],
        &[("word/header1.xml", &header)],
    )
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn header_content_change_is_redlined() {
    // Non-sharing words so the diff is a clean delete+insert (no common run).
    let a = doc_with_header("Alphaword");
    let b = doc_with_header("Bravoword");
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let hdr = read_part(&out, "word/header1.xml");
    assert!(
        hdr.contains("<w:ins") && hdr.contains("<w:del"),
        "header1.xml content change must be redlined (ins+del), got: {hdr}"
    );
    assert!(
        hdr.contains("Bravoword"),
        "inserted (new) header text must be present: {hdr}"
    );
    assert!(
        hdr.contains("Alphaword"),
        "deleted (old) header text must be present: {hdr}"
    );
}
