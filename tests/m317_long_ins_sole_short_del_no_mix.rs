// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M317 — long pure-I next + sole short pure-D base token.
//! basic_comment×sample: Word pure-I stream; engine was MIX last line+"test".

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn basic_comment_x_sample_no_test_token_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__basic_comment_d3ba5f1e.docx");
    let b = src.join("cli_legacy__sample_3a8f1f93.docx");
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
            && p.contains("empty pages")
            && (p.contains(">test<") || p.contains("delText>test"))
    });
    assert!(!mixed, "must not MIX sample line with sole base token test");
}
