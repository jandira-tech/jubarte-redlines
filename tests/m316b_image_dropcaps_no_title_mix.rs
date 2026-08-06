// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M316b — image_p_spacing × dropcaps: Word pure-I dropcap body then pure-D
//! "SOME TITLE". Engine was MIX-ing them (empty pure-D first).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn image_spacing_x_dropcaps_no_title_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__image_p_spacing_9b211fd7.docx");
    let b = src.join("super_editor__dropcaps_520cd049.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip");
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
    let mixed = xml.split("<w:p").any(|p| {
        p.contains("w:ins")
            && p.contains("w:del")
            && p.contains("rop caps")
            && p.contains("SOME TITLE")
    });
    assert!(!mixed, "must not MIX dropcap body with SOME TITLE");
}
