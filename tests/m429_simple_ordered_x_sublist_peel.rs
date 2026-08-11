// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::Read;
use std::path::PathBuf;

#[test]
fn simple_ordered_x_sublist_pure_i_one_two_before_base_del() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__simple_ordered_list_8288421a.docx");
    let b = src.join("super_editor__sublist_issue_66a1800a.docx");
    if !a.exists() || !b.exists() {
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
    let one = xml.find(">One<").expect("One");
    let two = xml.find(">Two<").expect("Two");
    let base_del = xml.find("Simple ordered list").expect("base title");
    assert!(
        one < base_del && two < base_del,
        "one={one} two={two} base={base_del}"
    );
}
