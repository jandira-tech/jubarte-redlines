// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M352 — pure-inserted paras keep demo-default line=276 spacing.
//!
//! line_break×line_space_table: Word pure-I next carries
//! `w:spacing w:line="276" w:lineRule="auto"`. Strip of demo-default
//! spacing dropped it (pagefair ~49); keep pure-I line=276 → 100.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn line_break_x_line_space_keeps_pure_i_line276() {
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
        "Word pure-I next keeps line=276 spacing; strip must not remove pure-I demo default"
    );
}
