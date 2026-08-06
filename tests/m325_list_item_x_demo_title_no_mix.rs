// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M325 — italic_rstyle × base_ordered: Word pure-I list items then pure-D
//! demo title (no MIX "Three"+italic).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn italic_rstyle_x_base_ordered_no_list_item_title_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_italic_rstyle_combos_demo_90894ac1.docx");
    let b = src.join("super_editor__base_ordered_fdff1fb2.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus missing");
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
        p.contains("w:ins") && p.contains("w:del") && p.contains("Three") && p.contains("italic")
    });
    assert!(
        !mixed,
        "Word keeps pure-I list item 'Three' separate from pure-D italic title"
    );
}
