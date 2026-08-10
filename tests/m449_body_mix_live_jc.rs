// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M449 — body MIX free-mesh keeps live jc (not park-only).
//!
//! right_aligned_italic × right_alignment (~75.9): Word mid MIX has live
//! `jc=right`. Engine parked jc into pPrChange only.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn right_align_body_mix_has_live_jc() {
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

    // Find MIX containing "aligned to the" (body free-mesh).
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
        if !(p.contains("aligned")
            && p.contains("<w:ins")
            && (p.contains("<w:del") || p.contains("delText")))
        {
            continue;
        }
        // Live jc before any pPrChange.
        let live = match p.find("pPrChange") {
            Some(c) => &p[..c],
            None => p,
        };
        assert!(
            live.contains("w:jc") && (live.contains("right") || live.contains("w:val=\"right\"")),
            "Word body MIX keeps live jc=right; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected body MIX with aligned free-mesh");
}
