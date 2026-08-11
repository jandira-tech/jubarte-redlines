// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M311 — textless multi-para next layout vs content base.
//! Word (image_inline×rtl_page_numpages): pure-I all empty next (~31) then
//! pure-D base content. merge_replaced sole_del fold was eating empties.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn word_settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

#[test]
fn image_x_rtl_keeps_wholesale_empty_pure_ins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__image_inline_and_block_ad6109b3.docx");
    let b = src.join("behavior__rtl_page_numpages_54739e26.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus not available at {}", src.display());
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &word_settings(),
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    let ins = xml.matches("<w:ins").count();
    let del = xml.matches("<w:del").count();
    assert!(
        ins >= 20 && del >= 1,
        "Word keeps ~31 empty pure-I next + pure-D residual; got ins={ins} del={del}"
    );
}
