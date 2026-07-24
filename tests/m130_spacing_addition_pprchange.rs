// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M130 — bare A body + B spacing → live spacing + pPrChange(empty old)
//! (file_165 Verdana × Ultimate Demo).

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m130_file_165_body_spacing_pprchange_empty_old() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_165.docx");
    let b = root.join("file_166.docx");
    if !a.is_file() {
        eprintln!("skip");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let xml = doc_xml(&out);
    let n = xml.matches("<w:pPrChange").count();
    // Word has pPrChange on both body paras (p2 and p3). Pre-M130 often 1.
    assert!(
        n >= 2,
        "spacing addition should surface ≥2 pPrChange, got {n}"
    );
}

#[test]
fn m130_file_8_guard_no_pprchange_flood() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_8.docx");
    let b = root.join("file_9.docx");
    if !a.is_file() {
        eprintln!("skip");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let xml = doc_xml(&out);
    let n = xml.matches("<w:pPrChange").count();
    // Word file_8 has 0 body pPrChange flood; keep well below thrash.
    assert!(
        n < 30,
        "spacing-addition must not flood file_8 pPrChange, got {n}"
    );
}
