//! M16 — relationship-id attributes (images, hyperlinks) must survive the redline.
//!
//! `reconcile_dangling_relationships` ensures every rId referenced by the result
//! resolves in the output package; rIds found nowhere have their attribute dropped
//! (Word "unreadable content" preventer). But it read a part's relationships via
//! `part_bytes("word/_rels/document.xml.rels")` — and the OPC layer (rdocx-opc)
//! does NOT expose `.rels` as parts (it parses them into `Relationships`, read via
//! `read_rels_for`). So `dest_existing`/source rels came back EMPTY, EVERY rId was
//! treated as an orphan, and `r:embed` (images) / `r:id` (hyperlinks) were stripped
//! from ALL redlines — images and hyperlinks silently broken.
//!
//! Red gate: an identity compare of a document with an image (and one with a
//! hyperlink) must retain the relationship reference and its relationship entry.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const IMAGE_DOC: &[u8] = include_bytes!("fixtures/relids/image_doc.docx");
const HYPERLINK_DOC: &[u8] = include_bytes!("fixtures/relids/hyperlink_doc.docx");

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn image_embed_reference_survives_redline() {
    let out = compare_documents(IMAGE_DOC, IMAGE_DOC, "Test").expect("compare ok");
    let doc = part(&out, "word/document.xml");
    assert!(
        doc.contains("r:embed="),
        "the image blip must keep its r:embed reference (else the image is lost)"
    );
    // and the referenced relationship + media must still be present/resolvable.
    let rels = part(&out, "word/_rels/document.xml.rels");
    assert!(
        rels.contains("/image"),
        "the image relationship must remain in document.xml.rels"
    );
}

#[test]
fn hyperlink_reference_survives_redline() {
    let out = compare_documents(HYPERLINK_DOC, HYPERLINK_DOC, "Test").expect("compare ok");
    let doc = part(&out, "word/document.xml");
    assert!(
        doc.contains("r:id="),
        "the hyperlink must keep its r:id reference (else the link target is lost)"
    );
}
