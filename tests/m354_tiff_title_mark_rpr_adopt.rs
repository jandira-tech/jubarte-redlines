// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M354 (planned) — tiff×h_f first title MIX keeps next-side mark rPr.
//!
//! Word free-meshes "TIFF test document" × "This is a document with:" and
//! keeps next Arial/sz=32 on the paragraph mark with `rPrChange(empty old)`.
//! Wholesale pure-I/D drops the pPr (pagefair thrash 41→37).
//!
//! Safer than skipping M315 (that thrash-ed hummingbird): adopt next mark
//! rPr onto the MIX title paragraph when pPr is missing.
//!
//! Status: RED against HEAD ec66729 until implement. Do not flip to green
//! with a hard-coded skip — fix produce/finalize on the real path.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn tiff_x_h_f_first_mix_has_mark_rpr() {
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
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    let body = xml
        .split_once("<w:body")
        .map(|(_, r)| r)
        .unwrap_or(xml.as_str());
    let first_p = body.split("</w:p>").next().expect("first para");
    assert!(
        first_p.contains("<w:ins") && first_p.contains("<w:del"),
        "first title is MIX (I next + D base); got {}",
        &first_p[..first_p.len().min(180)]
    );
    // Word keeps next mark rPr (Arial/sz32) under pPr, not only on the ins run.
    assert!(
        first_p.contains("<w:pPr")
            && (first_p.contains("w:val=\"32\"") || first_p.contains("Arial")),
        "MIX title must have pPr with next mark rPr (Word rPrChange path); pure-I/D thrash drops pPr"
    );
}
