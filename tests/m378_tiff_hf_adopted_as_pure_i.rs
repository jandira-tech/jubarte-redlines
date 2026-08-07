// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M378 — tiff×h_f: B-only headers/footers are pure-I (not live EQ).
//!
//! Base has no HF; Word wraps PAGE fields + labels in w:ins. Live copy of B
//! thrash LO pagefair (−3.3 vs 27c).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn tiff_x_h_f_header2_content_is_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__tiff_image_2d531f83.docx");
    let b = src.join("super_editor__h_f_normal_5d2a8d96.docx");
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
    let mut f = zip.by_name("word/header2.xml").expect("header2 adopted");
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    assert!(
        xml.contains("<w:ins"),
        "B-only header content must be pure-I; got {}",
        &xml[..xml.len().min(300)]
    );
    // Label text must live under ins, not bare w:t outside ins.
    assert!(xml.contains("Normal header"), "header label present");
    // Crude: after stripping ins blocks, no "Normal header" remains as live.
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("<w:ins") {
        let after = &rest[i..];
        let end = after
            .find("</w:ins>")
            .map(|j| j + "</w:ins>".len())
            .unwrap_or(after.len());
        rest = &after[end..];
    }
    assert!(
        !rest.contains("Normal header"),
        "header label must not remain as Equal outside ins"
    );
}
