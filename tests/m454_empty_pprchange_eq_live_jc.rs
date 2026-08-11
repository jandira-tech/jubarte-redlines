// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M454 — EQ with live jc keeps empty pPrChange shell.
//!
//! center_alignment_2 × center_alignment: Word title EQ has live
//! `jc=center` + empty pPrChange. Engine had live jc only (~87 → 100 LO).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn center2_title_eq_has_empty_pprchange() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("center_alignment_demo_id_paraid_overflow_2.docx");
    let b = src.join("center_alignment_demo_id_paraid_overflow.docx");
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
        if !p.contains("Center Alignment Demo") {
            continue;
        }
        if p.contains("<w:ins") || p.contains("<w:del") || p.contains("delText") {
            continue;
        }
        assert!(
            p.contains("w:jc") && p.contains("center"),
            "title needs live jc=center; p={p}"
        );
        assert!(
            p.contains("pPrChange"),
            "Word title EQ has empty pPrChange shell; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected center2 title EQ para");
}
