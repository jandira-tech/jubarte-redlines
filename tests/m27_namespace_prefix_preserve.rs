//! M27 — `mc:Choice Requires="wps"` must resolve: the wordprocessingShape (and
//! other Microsoft drawing-extension) namespaces must serialize with their
//! CONVENTIONAL prefixes (wps/wp14/wpg/wpc/wpi), not generic `nsN`.
//!
//! `Requires` is a prefix STRING evaluated against in-scope xmlns declarations.
//! Our serializer assigned the wordprocessingShape URI a generated prefix (`ns1`)
//! because it was missing from the well-known table, so `Requires="wps"` pointed
//! at an UNDECLARED prefix → real Word rejected the whole AlternateContent as
//! "unreadable content" (root cause of contract-acc_replaceTwoImages, etc.).

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
const WP_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
const WPS_NS: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";

fn build_docx(doc_xml: &str) -> Vec<u8> {
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
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#).unwrap();
        z.finish().unwrap();
    }
    buf
}

fn doc(marker: &str) -> Vec<u8> {
    let body = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:mc=\"{MC_NS}\" xmlns:wp=\"{WP_NS}\" xmlns:wps=\"{WPS_NS}\"><w:body>\
           <w:p><w:r><w:t>DIFF_{marker}</w:t></w:r></w:p>\
           <w:p><w:r>\
             <mc:AlternateContent>\
               <mc:Choice Requires=\"wps\"><w:drawing><wp:inline><wps:wsp><wps:bodyPr/></wps:wsp></wp:inline></w:drawing></mc:Choice>\
               <mc:Fallback><w:pict/></mc:Fallback>\
             </mc:AlternateContent></w:r></w:p>\
           <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
         </w:body></w:document>"
    );
    build_docx(&body)
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn mc_requires_prefix_is_declared() {
    let out = compare_documents(&doc("A"), &doc("B"), "Test").expect("compare ok");
    let x = read_part(&out, "word/document.xml");

    // The Requires value is a prefix string; the matching xmlns must be declared.
    assert!(
        x.contains("Requires=\"wps\""),
        "AltContent Choice must be preserved: {x}"
    );
    assert!(
        x.contains(&format!("xmlns:wps=\"{WPS_NS}\"")),
        "the wps prefix referenced by Requires must be declared (not renamed to nsN): {x}"
    );
    assert!(
        x.contains("<wps:wsp"),
        "wps elements must use the wps prefix, not nsN: {x}"
    );
}
