// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M384 — file_45×file_46 residual: short list pure-D MIX adopts numPr.
//!
//! M382 free-meshes long pure-I Times body with 2-word list pure-D "First item".
//! Word MIX keeps Deleted live numPr + del mark (list label folded into body).
//! adopt_del_ppr used del_list_multi = word_atoms > 1, blocking numPr adopt for
//! the 2-word label (−13.6 residual). Align with M382 (≥3).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file45_x_46_mix_times_first_item_adopts_numpr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_45.docx");
    let b = src.join("file_46.docx");
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
        if p.contains("Times New Roman is a classic")
            && p.contains("First item")
            && p.contains("<w:ins")
            && p.contains("<w:del")
        {
            found = true;
            assert!(
                p.contains("<w:numPr") || p.contains("ListParagraph"),
                "Word MIX adopts short list pure-D numPr/ListParagraph; pPr missing"
            );
            break;
        }
    }
    assert!(
        found,
        "expected MIX Times body + First item with list pPr (M384)"
    );
}
