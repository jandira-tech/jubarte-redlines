// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M468 — when trailing EMPTY pure-I paragraphs separate a LONG content
//! pure-I from a short unrelated pure-D title, Word folds nothing: the
//! content paragraph stays pure-INS and the title keeps its own MARK-DEL
//! paragraph. The multi-del boundary fold previously reached backward across
//! the empties and merged the deleted title into the content paragraph.
//!
//! Oracle: super_editor__h_f_normal_odd_even_unchecked_first_p ×
//! super_editor__sd_1495_auto_page_break — [ins "This icon appears…"]
//! [ins empty ×2][del "This is a document with:"] … The merge lost the
//! oracle's ~56pt gap and shifted all 15 pages (79.6 → 46.1 once the
//! heading stamps stopped masking it).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn deleted_title_stays_separate_across_empty_ins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__h_f_normal_odd_even_unchecked_first_p_817ad6e9.docx");
    let b = src.join("super_editor__sd_1495_auto_page_break_854a2dd9.docx");
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

    // The icon paragraph must not swallow the deleted title.
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("<w:p ") {
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        let p = &after[..j];
        rest = &after[j + 6..];
        if p.contains("This icon appears") {
            assert!(
                !p.contains("This is a document with"),
                "deleted title must not fold into the icon paragraph"
            );
            assert!(
                !p.contains("<w:delText"),
                "icon paragraph must stay pure-INS, got a delText"
            );
        }
    }
    assert!(
        xml.contains("This is a document with"),
        "deleted title text must survive as its own content"
    );
}
