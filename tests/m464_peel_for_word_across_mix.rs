// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M464 — peel trailing ` for <word>` from MIX ins onto next MIX.
//!
//! center_alignment × center_bold: Word has p2 `INS[perfect]` and p3
//! `EQ[ for ]|INS[document ]` instead of p2 `INS[perfect for document]` and
//! p3 `DEL[… great for]`.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn center_bold_peels_for_document_onto_next_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("center_alignment_demo_id_paraid_overflow.docx");
    let b = src.join("center_bold_demo_id_paraid_overflow.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    // Should not keep wholesale "perfect for document" in one ins.
    assert!(
        !xml.contains("perfect for document"),
        "still has unsplit 'perfect for document' ins"
    );
    // Should have bare "perfect" as ins and "document" as ins.
    assert!(
        xml.contains(">perfect<") || xml.contains(">perfect</"),
        "expected peeled ins 'perfect'; xml missing"
    );
    assert!(
        xml.contains("document"),
        "expected document token present after peel"
    );
    // Del residual should not end with "great for" wholesale on last line.
    // Allow "great" without trailing for on the del of titles para.
    assert!(
        !xml.contains("great for</w:delText>") && !xml.contains("great for"),
        "del still ends with 'great for' — peel did not trim next MIX del"
    );
}
