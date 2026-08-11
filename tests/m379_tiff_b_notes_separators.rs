// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M379 — tiff×h_f: when A has no footnotes, carry B separator notes.
//!
//! Word ships footnote/endnote separator + continuationSeparator (with drawings)
//! even with zero body footnote refs. Empty shells thrash LO page geometry.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn tiff_x_h_f_adopts_b_footnote_separators() {
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
    let mut fn_xml = String::new();
    zip.by_name("word/footnotes.xml")
        .expect("footnotes part")
        .read_to_string(&mut fn_xml)
        .unwrap();
    assert!(
        fn_xml.contains("w:type=\"separator\"") || fn_xml.contains("w:type='separator'"),
        "must carry B separator footnote; got {}",
        &fn_xml[..fn_xml.len().min(200)]
    );
    assert!(
        fn_xml.contains("continuationSeparator") || fn_xml.contains("continuation"),
        "must carry continuationSeparator"
    );
    // Separators in h_f_normal carry drawings.
    assert!(
        fn_xml.contains("drawing") || fn_xml.contains("pict"),
        "separator footnotes carry drawings in Word oracle"
    );
}
