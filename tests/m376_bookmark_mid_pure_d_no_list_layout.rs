// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M376 — bookmark×broken_complex_list: mid pure-D has no pure-I list layout.
//!
//! Word keeps first pure-D bookmark prose bare (mark-only rPr). Eng polluted it
//! with after=0 line=240 + jc=both + Aptos mark fonts from the pure-I list
//! residual (−0.6 pagefair).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn bookmark_mid_pure_d_no_line240_list_layout() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__bookmark_use_cases_d20f31f6.docx");
    let b = src.join("super_editor__broken_complex_list_293fda86.docx");
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

    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut paras = Vec::new();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        paras.push(&after[..end_rel]);
        rest = &after[end_rel..];
    }

    // First pure-D with bookmark prose.
    let mut found = false;
    for p in &paras {
        if !p.contains("simple bookmark") {
            continue;
        }
        found = true;
        assert!(p.contains("<w:del"), "bookmark prose must be pure-D");
        assert!(
            !p.contains("line=\"240\""),
            "mid pure-D must not carry pure-I list line=240: {p}"
        );
        assert!(
            !p.contains("w:val=\"both\""),
            "mid pure-D must not carry jc=both from list residual"
        );
        break;
    }
    assert!(found, "expected pure-D simple bookmark prose");
}
