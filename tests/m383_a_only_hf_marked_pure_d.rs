// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M383 — A has HF, B has none: original HF content is pure-D.
//!
//! h_f_normal_odd_even_firstpg × basic_footnotes: Word delText wraps even/odd
//! header labels; eng left them live (−45). Inverse of M378 B-only pure-I.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn h_f_x_basic_footnotes_a_headers_are_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__h_f_normal_odd_even_firstpg_9b210d9a.docx");
    let b = src.join("super_editor__basic_footnotes_5be96945.docx");
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
    let mut xml = String::new();
    zip.by_name("word/header1.xml")
        .expect("header1")
        .read_to_string(&mut xml)
        .unwrap();
    assert!(
        xml.contains("delText") || xml.contains("<w:del"),
        "A-only header content must be pure-D; got {}",
        &xml[..xml.len().min(300)]
    );
    assert!(
        xml.contains("Even page header") || xml.contains("page numbers"),
        "header label present under del"
    );
}
