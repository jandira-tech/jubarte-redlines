// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M466 — the trailing bare-period attach (old-M463 boiler-EQ fold) must skip
//! a period run that carries a tracked format change. file_168 × file_169:
//! the mixed heading residual ends `[del "demonstrates Heading 2 paragraph
//! style"][live "." with strike + rPrChange]` — Word keeps the period live
//! with its rPrChange; merging it into the delText dropped the format-change
//! record and the doc fell 100 → 89.7.

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn formatted_period_stays_live_with_rprchange() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_168.docx");
    let b = src.join("file_169.docx");
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

    // The deleted heading text must NOT have swallowed the trailing period.
    assert!(
        !xml.contains("demonstrates Heading 2 paragraph style.</w:delText>"),
        "formatted trailing period must not merge into delText"
    );
    assert!(
        xml.contains("demonstrates Heading 2 paragraph style</w:delText>"),
        "expected the heading residual delText without the period"
    );
}
