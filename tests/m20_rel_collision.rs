//! M20 — relationship-id COLLISION across the two compared documents.
//!
//! The output package is based on the ORIGINAL (A). Inserted content from the
//! MODIFIED (B) carries B's rIds, which live in a different namespace than A's.
//! `reconcile_dangling_relationships` matched referenced rIds by STRING only, so
//! a B `headerReference r:id="rId1"` (→ B's header) matched A's unrelated
//! `rId1` (→ endnotes) and was kept — producing a `headerReference` that points
//! to `endnotes.xml` (Word-invalid). Reconcile must be TYPE-AWARE: a reference's
//! rId must resolve to a relationship of the REQUIRED type (header/footer/image/
//! hyperlink/…); on mismatch, carry the correctly-typed relationship from source.

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Build a minimal valid .docx. `rels` = (id, type, target); `extra` = (part, xml).
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

fn doc(body: &str) -> String {
    format!(
        "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
         xmlns:r=\"{REL_NS}\"><w:body>{body}</w:body></w:document>"
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
fn header_reference_does_not_bind_to_a_colliding_non_header_relationship() {
    // A: rId1 -> endnotes (a NON-header part).
    let a = build_docx(
        &doc(
            "<w:p><w:r><w:t>shared</w:t></w:r></w:p><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
        ),
        &[("rId1", &format!("{REL_NS}/endnotes"), "endnotes.xml")],
        &[(
            "word/endnotes.xml",
            "<w:endnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>",
        )],
    );
    // B: a mid-document section break whose headerReference is also rId1 -> a header.
    let b = build_docx(
        &doc(
            "<w:p><w:pPr><w:sectPr><w:headerReference w:type=\"default\" r:id=\"rId1\"/><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr></w:p>\
              <w:p><w:r><w:t>shared</w:t></w:r></w:p><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
        ),
        &[("rId1", &format!("{REL_NS}/header"), "header1.xml")],
        &[(
            "word/header1.xml",
            "<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:r><w:t>HDR</w:t></w:r></w:p></w:hdr>",
        )],
    );

    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let docx = read_part(&out, "word/document.xml");
    let rels = read_part(&out, "word/_rels/document.xml.rels");

    // Map rId -> Type from the output rels.
    let mut type_of = std::collections::HashMap::new();
    for cap in rels.split("<Relationship").skip(1) {
        let id = cap.split("Id=\"").nth(1).and_then(|s| s.split('"').next());
        let ty = cap
            .split("Type=\"")
            .nth(1)
            .and_then(|s| s.split('"').next());
        if let (Some(id), Some(ty)) = (id, ty) {
            type_of.insert(id.to_string(), ty.to_string());
        }
    }
    // Every headerReference in the output must resolve to a /header relationship.
    let mut checked = 0;
    for seg in docx.split("<w:headerReference").skip(1) {
        let rid = seg
            .split("r:id=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .unwrap_or("");
        checked += 1;
        let ty = type_of.get(rid).cloned().unwrap_or_default();
        assert!(
            ty.ends_with("/header"),
            "headerReference {rid} must point to a /header relationship, got {ty:?}"
        );
    }
    assert!(
        checked >= 1,
        "expected the inserted headerReference to be present in the output"
    );
}
