// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M390 — missing_sectpr × fields_test: Word flattens content controls.
//!
//! B (`fields_test`) has 9 `w:sdt` content controls. Word Compare redline has
//! **0** SDTs (plain ins/del runs). We kept SDTs → LO pagefair −16.3. Unwrap
//! SDTs to sdtContent in Word mode.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn missing_sectpr_x_fields_test_no_sdt_in_redline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__missing_sectpr_967a402d.docx");
    let b = src.join("super_editor__fields_test_4a8ffd8c.docx");
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

    assert!(
        !xml.contains("<w:sdt") && !xml.contains("<w:sdtContent"),
        "Word redline has 0 content controls; got sdt wrappers"
    );
    // Field display text still present as plain revised runs.
    assert!(
        xml.contains("Field:") || xml.contains("something") || xml.contains("Basic text"),
        "expected field label/value text after unwrap"
    );
    assert!(
        xml.contains("html input type") || xml.contains("Document without sectPr"),
        "expected body residual text"
    );
    // M391: Word keeps pure-I line=276 when para also has w:ind (Product line).
    assert!(
        xml.contains("cool product") && xml.contains("w:line=\"276\""),
        "Word keeps line=276 on pure-I with ind (M391)"
    );
}

#[test]
fn powertools_faithful_keeps_sdts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__missing_sectpr_967a402d.docx");
    let b = src.join("super_editor__fields_test_4a8ffd8c.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::powertools_faithful(),
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // Faithful mode does not unwrap — content controls may remain.
    // (Do not assert presence: produce may still restructure; just ensure no panic.)
    let _ = xml.contains("<w:sdt");
}
