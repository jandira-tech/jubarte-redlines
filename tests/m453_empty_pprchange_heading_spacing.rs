// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M453 — live heading residual spacing keeps empty pPrChange shell.
//!
//! calibri_font × calibri_heading_2_right: Word mid MIX has live
//! spacing(before=360,line=240) + empty pPrChange. Engine mid had live only.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn calibri_mid_mix_empty_pprchange_with_heading_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("calibri_font_demo_id_paraid_overflow.docx");
    let b = src.join("calibri_heading_2_right_id_paraid_overflow.docx");
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

    // Mid MIX: "Calibri font" free-mesh with live heading spacing + empty pPrChange.
    let mut rest = xml.as_str();
    let mut found = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if !p.contains("Calibri font") {
            continue;
        }
        if !(p.contains("<w:ins") && (p.contains("<w:del") || p.contains("delText"))) {
            continue;
        }
        assert!(
            p.contains("w:spacing") && p.contains("w:before"),
            "mid MIX needs live heading spacing; p={p}"
        );
        assert!(
            p.contains("pPrChange"),
            "Word mid MIX has empty pPrChange shell; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected calibri mid MIX free-mesh");
}
