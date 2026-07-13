//! M19 — mid-document section breaks (`w:sectPr` inside a paragraph's `w:pPr`)
//! must survive the redline.
//!
//! `compare_bodies_faithful` reinstated the body section properties by removing
//! EVERY `w:sectPr` the diff produced and appending one clean body-level sectPr —
//! which also deleted mid-document section breaks. So multi-section documents
//! collapsed: `multi_section_doc` (3 mid-doc section breaks) rendered as 1 page vs
//! Word's 4. The text oracle was blind (sectPr carries no visible text); found
//! via headless visual comparison (page-count mismatch). Only the body-level
//! sectPr should be reinstated; mid-document section breaks must be preserved.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const MULTI: &[u8] = include_bytes!("fixtures/sections/multi_section.docx");

/// Total number of `w:sectPr` (= number of sections). Each section break is a
/// `w:sectPr`; collapsing them loses page/section structure.
fn section_count(docx: &[u8]) -> usize {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s.matches("<w:sectPr").count()
}

#[test]
fn identity_redline_preserves_all_sections() {
    let input_sections = section_count(MULTI);
    assert!(
        input_sections >= 4,
        "fixture sanity: expected >=4 sections, got {input_sections}"
    );

    let out = compare_documents(MULTI, MULTI, "Test").expect("compare ok");
    let out_sections = section_count(&out);
    assert_eq!(
        out_sections, input_sections,
        "identity redline must preserve all {input_sections} sections (got {out_sections}) — \
         mid-document section breaks must not be dropped"
    );
}
