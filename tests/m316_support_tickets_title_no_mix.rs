// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M316 — support_tickets × table_bookmark_end: Word pure-I "Test 1…" then
//! pure-D "Support Tickets" (IIIDDD…). Engine was MIX-ing them.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn support_tickets_x_table_bookmark_no_test1_title_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("support_tickets_table.docx");
    let b = src.join("table_bookmark_end.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus missing");
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
    let mixed = xml.split("<w:p").any(|p| {
        p.contains("w:ins")
            && p.contains("w:del")
            && p.contains("Test 1")
            && p.contains("Support Tickets")
    });
    assert!(
        !mixed,
        "Word keeps pure-I Test 1 header and pure-D Support Tickets title"
    );
}
