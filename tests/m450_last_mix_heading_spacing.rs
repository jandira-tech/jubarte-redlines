// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M450 — last MIX keeps live Heading residual spacing (not park-only).
//!
//! calibri_font × calibri_heading_2_right (~82.5): Word last MIX has live
//! `spacing before=360 after=80 line=240` + empty `pPrChange`. Engine parked
//! spacing into pPrChange only.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn calibri_heading_last_mix_has_live_heading_spacing() {
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

    // Last MIX containing "distinctive" (body free-mesh).
    let mut rest = xml.as_str();
    let mut last_mix: Option<&str> = None;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if p.contains("<w:ins") && (p.contains("<w:del") || p.contains("delText")) {
            last_mix = Some(p);
        }
    }
    let p = last_mix.expect("expected last MIX");
    let live = match p.find("pPrChange") {
        Some(c) => &p[..c],
        None => p,
    };
    assert!(
        live.contains("w:spacing")
            && (live.contains("w:before=\"360\"") || live.contains("w:before='360'"))
            && (live.contains("w:line=\"240\"") || live.contains("w:line='240'")),
        "Word last MIX keeps live Heading spacing before=360 line=240; p={p}"
    );
}
