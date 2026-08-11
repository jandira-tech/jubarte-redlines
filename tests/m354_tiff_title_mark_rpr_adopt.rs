// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M354/M361 — tiff×h_f first title MIX.
//!
//! Word keeps next Arial/sz=32 on the paragraph mark with `rPrChange(empty
//! old)`. Live pPr fonts without empty-old thrash LO pagefair 41→37 vs 27c
//! bare MIX (fonts still on ins runs). M361 restores bare MIX after fold
//! when pPr/rPr only carried ins/del marks — run-level rPr retains fonts.

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
    // M361: bare MIX after fold (no mark-only pPr). Fonts stay on ins runs.
    // Live pPr fonts alone thrash LO pagefair 41→37 vs 27c.
    assert!(
        first_p.contains("Arial") || first_p.contains("w:val=\"32\""),
        "ins run must keep next fonts/sz even if pPr mark stripped; got {}",
        &first_p[..first_p.len().min(200)]
    );
    // pPr/rPr must not retain live fonts under mark-only path (27c bare).
    assert!(
        !first_p.contains("<w:pPr") || !first_p.contains("<w:pPr><w:rPr><w:rFonts"),
        "pPr must not keep live fonts after mark strip (LO thrash); use run rPr"
    );
}
