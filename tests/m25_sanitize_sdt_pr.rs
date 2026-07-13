//! M25 — non-standard `w:`-namespace children inside `<w:sdtPr>` must be stripped
//! (Word recovers corrupt input by dropping them; we match to produce valid output).
//!
//! Agreement/form tools emit `<w:fieldType>`, `<w:fieldColor>`, etc. inside
//! `<w:sdtPr>` in the w: namespace — NOT valid CT_SdtPr content. Real Word reports
//! "unreadable content" on these inputs and recovers by stripping them; ooxmlsdk
//! rejects with `UnexpectedTag { ty: SdtProperties, found: fieldType }`. We must
//! sanitize so the redline output is Word-valid (5 corpus pairs:
//! annotations_import_2_*, falsy-block_*, fields_attrs1_*, fields_attrs2_*,
//! list-numbering-reimport_*). Extension namespaces (w14:/w15:) are left untouched.

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

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
        "<w:document xmlns:w=\"{W_NS}\"><w:body>\
           <w:p><w:r><w:t>DIFF_{marker}</w:t></w:r></w:p>\
           <w:sdt><w:sdtPr>\
             <w:alias w:val=\"Enter name\"/><w:tag w:val=\"t1\"/><w:id w:val=\"1\"/>\
             <w:fieldType w:val=\"NAMETEXTINPUT\"/><w:fieldColor w:val=\"#6943d0\"/>\
             <w:fieldMultipleImage w:val=\"false\"/>\
           </w:sdtPr><w:sdtContent><w:p><w:r><w:t>content</w:t></w:r></w:p></w:sdtContent></w:sdt>\
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
fn nonstandard_sdtpr_children_are_stripped() {
    let out = compare_documents(&doc("A"), &doc("B"), "Test").expect("compare ok");
    let x = read_part(&out, "word/document.xml");

    assert!(
        !x.contains("<w:fieldType"),
        "non-standard <w:fieldType> must be stripped: {x}"
    );
    assert!(
        !x.contains("<w:fieldColor"),
        "non-standard <w:fieldColor> must be stripped"
    );
    assert!(
        !x.contains("<w:fieldMultipleImage"),
        "non-standard <w:fieldMultipleImage> must be stripped"
    );
    // Valid CT_SdtPr children survive.
    assert!(x.contains("<w:alias"), "valid <w:alias> must survive: {x}");
    assert!(x.contains("<w:tag"), "valid <w:tag> must survive");
    // And the output is ooxmlsdk-loadable (was UnexpectedTag SdtProperties/fieldType).
    ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
        .expect("output must be ooxmlsdk-loadable after sdtPr sanitization");
}
