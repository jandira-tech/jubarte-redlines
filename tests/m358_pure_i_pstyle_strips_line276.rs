// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M358 — pure-I + pStyle strips demo-default line=276 (LO pagefair).
//!
//! fields×localized: wholesale pure-I of DocumentTitle/Heading paras. Word
//! keeps line=276, but LO pagefair thrash (−20 vs 27c). Strip pure-I+pStyle
//! demo-default spacing; bare pure-I without pStyle still keeps (M352
//! line_break×line_space).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn fields_x_localized_strips_pure_i_pstyle_line276() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__fields_test_4a8ffd8c.docx");
    let b = src.join("behavior__sd_2517_localized_heading_styles_39c2e4a1.docx");
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
    let n_line276 = xml.matches("line=\"276\"").count();
    // Pre-M352/M353 (27c) had 1; M352+M353 thrash had 66. M358 should strip
    // pure-I+pStyle demo defaults back near 27c (≤5 residual non-demo).
    assert!(
        n_line276 <= 12,
        "pure-I+pStyle demo line=276 must strip (LO pagefair); got {n_line276}"
    );
}

#[test]
fn line_break_still_keeps_bare_pure_i_line276() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__line_break_627a7159.docx");
    let b = src.join("super_editor__line_space_table_9b1ee54b.docx");
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
        xml.contains("line=\"276\""),
        "M352 bare pure-I line=276 must still keep for line_break×line_space"
    );
}
