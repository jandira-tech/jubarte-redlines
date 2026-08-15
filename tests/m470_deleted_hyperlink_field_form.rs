// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M470 — a fully deleted anchor `w:hyperlink` serializes as Word's FIELD
//! form: `[fldChar begin][delInstrText HYPERLINK \l "<anchor>"]
//! [fldChar separate] …content… [fldChar end]`, all inside the deletion.
//! Plain unwrapping (the live-hyperlink rule from file_21) drops the wrapper
//! and leaves the inner PAGEREF field's chars unbalanced — LibreOffice then
//! mis-renders TOC leader dots and page numbers, wraps lines, and drifts a
//! 115-page document one page short (file_22 × file_23: 46.1, docxodus 94.1).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn deleted_toc_hyperlink_keeps_field_form() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_22.docx");
    let b = src.join("file_23.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    // The first deleted TOC1 entry (paraId 1B62F586 in both source and
    // oracle) must carry the HYPERLINK field wrapper in delInstrText form.
    let i = xml
        .find("aliqua eiusmod elit elit consectetur")
        .expect("deleted TOC text present");
    let start = xml[..i]
        .rfind("<w:p ")
        .or_else(|| xml[..i].rfind("<w:p>"))
        .unwrap();
    let end = xml[i..].find("</w:p>").map(|j| i + j + 6).unwrap();
    let para = &xml[start..end];

    assert!(
        para.contains("HYPERLINK \\l"),
        "deleted hyperlink must serialize its HYPERLINK instr, got: {}",
        &para[..para.len().min(400)]
    );
    assert!(
        para.contains("delInstrText"),
        "instr text inside a deletion must be delInstrText"
    );
    let begins = para.matches("w:fldCharType=\"begin\"").count();
    let ends = para.matches("w:fldCharType=\"end\"").count();
    let seps = para.matches("w:fldCharType=\"separate\"").count();
    assert!(
        begins >= 2 && seps >= 2,
        "expected HYPERLINK + PAGEREF field chars, got begins={begins} seps={seps}"
    );
    assert_eq!(
        begins, ends,
        "field chars must balance: begins={begins} ends={ends}"
    );
}
