// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M377 — tiff×h_f short-title MIX free-meshes shared "document" as EQ.
//!
//! Word: ins "This is a" + del "TIFF test" + EQ " document" + ins " with:".
//! Wholesale ins+del keeps "document" on both (−3.4 vs 27c).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn tiff_x_h_f_title_shares_document_as_eq() {
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
        "first title is MIX"
    );
    // Full del "TIFF test document" is the miss.
    assert!(
        !first_p.contains("TIFF test document"),
        "del must not keep full base title; free-mesh EQ document"
    );
    // EQ document outside del/ins.
    if let Some(di) = first_p.find("</w:del>") {
        let after_del = &first_p[di..];
        assert!(
            after_del.contains("document"),
            "EQ 'document' after del (Word free-mesh)"
        );
    } else {
        panic!("expected del in first MIX");
    }
}
