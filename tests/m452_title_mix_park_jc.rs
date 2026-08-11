// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M452 — short title MIX parks jc into pPrChange (no live jc).
//!
//! right_aligned_italic × right_alignment_2 (~84.3): Word first MIX has
//! `pPrChange(jc=right)` only. Engine left title MIX without pPr.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn right_align_title_mix_parks_jc() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("right_aligned_italic_demo_id_paraid_overflow.docx");
    let b = src.join("right_alignment_demo_id_paraid_overflow_2.docx");
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

    // First MIX (title free-mesh) — park-only pPrChange(jc=right), no live jc.
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
        if !(p.contains("<w:ins") && (p.contains("<w:del") || p.contains("delText"))) {
            continue;
        }
        // Title shape: short free-mesh with "Alignment" / "Aligned" tokens.
        if !(p.contains("Alignment") || p.contains("Aligned") || p.contains("Right")) {
            continue;
        }
        assert!(
            p.contains("pPrChange"),
            "Word title MIX parks jc in pPrChange; p={p}"
        );
        // Live jc before pPrChange must be absent (park-only).
        let live = match p.find("pPrChange") {
            Some(c) => &p[..c],
            None => p,
        };
        assert!(
            !live.contains("w:jc"),
            "title MIX must not have live jc (park-only); live={live}"
        );
        assert!(
            p.contains("right") || p.contains("w:val=\"right\""),
            "parked jc should be right; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected first title MIX with parked jc");
}
