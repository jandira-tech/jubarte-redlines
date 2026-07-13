//! M14 — documents with multiple `w:body` elements (Apache POI `MultipleBodyBug`).
//!
//! Some producers emit a `w:document` with more than one `w:body` (invalid per
//! the schema, but real). Word merges them: its Compare output contains the text
//! of ALL bodies. We previously took only the FIRST body
//! (`dom.element(root, &W::body())`), silently dropping bodies 2..n — a content
//! loss. The comparer must consider every body.
//!
//! Red gate: an identity compare of a 3-body document must retain all three
//! bodies' text in the output.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const DOC: &[u8] = include_bytes!("fixtures/multibody/multibody.docx");

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn multibody_identity_retains_all_three_bodies() {
    let out = compare_documents(DOC, DOC, "Test").expect("compare ok");
    let xml = document_xml(&out);
    for marker in ["START BODY 1", "START BODY 2", "START BODY 3"] {
        assert!(
            xml.contains(marker),
            "redline output must retain {marker:?} (all w:body elements), got len {}",
            xml.len()
        );
    }
}
