// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M451 — mid MIX with live jc drops empty pPrChange.
//!
//! center_alignment_2 × center_alignment (~81.5): Word mid MIX has live
//! `jc=center` + rPr/ins only. Engine left empty `pPrChange` shell.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn center2_mid_mix_no_empty_pprchange() {
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

    // Mid MIX: "centered on the page"
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
        if !p.contains("centered on the page") {
            continue;
        }
        if !(p.contains("<w:ins") && (p.contains("<w:del") || p.contains("delText"))) {
            continue;
        }
        // Empty pPrChange: pPrChange with empty inner pPr only.
        let has_empty_chg = p.contains("pPrChange")
            && (p.contains("<w:pPr/>")
                || p.contains("<w:pPr />")
                || p.contains("<w:pPr></w:pPr>")
                || p.contains("<w:pPr xmlns") && p.contains("pPrChange") && {
                    // crude: pPrChange block has no spacing/jc/ind inside
                    if let Some(i) = p.find("pPrChange") {
                        let slice = &p[i..];
                        !slice.contains("w:spacing")
                            && !slice.contains("w:jc")
                            && !slice.contains("w:ind")
                            && !slice.contains("w:numPr")
                    } else {
                        false
                    }
                });
        assert!(
            !has_empty_chg || {
                // allow non-empty; fail only if empty shell remains
                if let Some(i) = p.find("<w:pPrChange") {
                    let s = &p[i..];
                    let end = s.find("</w:pPrChange>").map(|j| j + 14).unwrap_or(s.len());
                    let block = &s[..end];
                    block.contains("w:spacing")
                        || block.contains("w:jc")
                        || block.contains("w:ind")
                        || block.contains("w:numPr")
                        || block.contains("w:pStyle")
                } else {
                    true
                }
            },
            "Word mid MIX has no empty pPrChange; p={p}"
        );
        // Stronger: no pPrChange at all on this mid MIX.
        assert!(
            !p.contains("pPrChange"),
            "Word mid MIX drops empty pPrChange; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected mid MIX centered on the page");
}
