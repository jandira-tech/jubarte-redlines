// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M18 — `w:pPr` must be the FIRST child of `w:p` (OOXML schema).
//!
//! Our reassembly emitted the paragraph mark (`w:pPr`) LAST in the paragraph, so
//! Word/LibreOffice ignored ALL paragraph formatting — centering (`w:jc`),
//! spacing, indentation, numbering — silently (87% of our paragraphs were
//! affected; Word: ~0). This is the "lost centralization" defect, and far broader
//! than centering. The text-reconstruction oracle was blind to it (pPr carries no
//! visible text). Fix: move each paragraph's `w:pPr` to the front.

use std::io::{Cursor, Read};

use jubarte::comparer::finalize::move_paragraph_properties_first;
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const WNS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const IMAGE_DOC: &[u8] = include_bytes!("fixtures/relids/image_doc.docx");

#[test]
fn moves_ppr_to_front_of_paragraph() {
    // pPr emitted AFTER the run -> must be moved to the front.
    let xml = format!(
        "<w:document xmlns:w=\"{WNS}\"><w:body><w:p>\
         <w:r><w:t>hi</w:t></w:r>\
         <w:pPr><w:jc w:val=\"center\"/></w:pPr>\
         </w:p></w:body></w:document>"
    );
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    move_paragraph_properties_first(&mut d, root);

    let p = d.descendants(root, Some(&W::p()))[0];
    let first = d.nodes(p)[0];
    assert_eq!(
        d.name(first).as_ref(),
        Some(&W::p_pr()),
        "w:pPr must be the first child of w:p"
    );
}

#[test]
fn redline_output_keeps_ppr_first_in_every_paragraph() {
    let out = compare_documents(IMAGE_DOC, IMAGE_DOC, "Test").expect("compare ok");
    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
    let xml = {
        let mut f = zip.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    // crude but effective: no paragraph may have its <w:pPr> appear after a <w:r>.
    let mut misplaced = 0;
    let mut search = xml.as_str();
    while let Some(p0) = search.find("<w:p>").or_else(|| search.find("<w:p ")) {
        let rest = &search[p0..];
        let end = rest.find("</w:p>").map(|e| e + 6).unwrap_or(rest.len());
        let para = &rest[..end];
        if let Some(ppr) = para.find("<w:pPr") {
            // first run/content before pPr ⇒ misplaced
            let before = &para[..ppr];
            if before.contains("<w:r>")
                || before.contains("<w:r ")
                || before.contains("<w:hyperlink")
            {
                misplaced += 1;
            }
        }
        search = &rest[end..];
    }
    assert_eq!(
        misplaced, 0,
        "every w:pPr must precede the paragraph's content"
    );
}
