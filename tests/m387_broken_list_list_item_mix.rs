// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M387 — broken_list×list_spacer: list pure-I free-meshes list pure-D.
//!
//! Word MIX: "14.11Survival of Terms p3" + del "Item 1. Text 1." with numPr.
//! M363 long-prose×multi-word-list skip blocked list×list free-mesh (−15.9).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn broken_list_x_spacer_survival_mixes_item1() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__broken_list_missing_items_36b4199e.docx");
    let b = src.join("super_editor__list_spacer1_06383c66.docx");
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
    let mut found = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        // Text is split across runs ("14.11" / "Survival of Terms" / "p3").
        if p.contains("14.11")
            && p.contains("Survival of Terms")
            && p.contains("Item 1")
            && p.contains("<w:ins")
            && p.contains("<w:del")
        {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "Word free-meshes list pure-I Survival with list pure-D Item 1 (M387)"
    );
}
