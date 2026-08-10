// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M442 — last pure-D list residual gets live numPr (list_spacer).
//!
//! list_spacer1 × list_with_break_exported_broken residual: Word last pure-D
//! "14.11…" has live `numPr` (numId matching pure-I list) + pPrChange parking
//! base StandardL1. Engine had pPrChange only (no live numPr).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_spacer_last_pure_d_has_live_numpr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_spacer1_06383c66.docx");
    let b = src.join("super_editor__list_with_break_exported_broken_45f7bd19.docx");
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

    // Find paragraph containing 14.11 Survival.
    let needle = "14.11";
    let pos = xml.find(needle).expect("expected 14.11 delText");
    let head = &xml[..pos];
    let p_start = head
        .rmatch_indices("<w:p")
        .find(|(i, _)| {
            let after = &head[*i..];
            after.starts_with("<w:p>") || after.starts_with("<w:p ")
        })
        .map(|(i, _)| i)
        .expect("p before 14.11");
    let p_end = xml[pos..]
        .find("</w:p>")
        .map(|j| pos + j + 6)
        .expect("end p");
    let p = &xml[p_start..p_end];
    // Live numPr before pPrChange (not only inside pPrChange).
    let numpr_live = p.find("<w:numPr").filter(|&i| {
        // Before any pPrChange in this para.
        match p.find("pPrChange") {
            Some(c) => i < c,
            None => true,
        }
    });
    assert!(
        numpr_live.is_some(),
        "Word stamps live numPr on last pure-D 14.11; p={p}"
    );
    assert!(
        p.contains("pPrChange"),
        "base StandardL1/numId stays under pPrChange; p={p}"
    );
}
