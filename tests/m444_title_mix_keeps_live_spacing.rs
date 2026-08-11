// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M444 — last short-title MIX keeps live spacing (not pPrChange).
//!
//! title_style_centered × title_style_demo (~86.4): Word free-meshes
//! "Document Title" × cover prose and keeps live `spacing line=240`. M94
//! parked spacing into pPrChange.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn title_style_last_mix_keeps_live_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("title_style_centered_demo_id_paraid_overflow.docx");
    let b = src.join("title_style_demo_id_paraid_overflow.docx");
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

    // Last MIX with Document Title ins + long del cover prose.
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
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        if has_ins && has_del && p.contains("Document Title") {
            last_mix = Some(p);
        }
    }
    let p = last_mix.expect("expected MIX with Document Title");
    // Live spacing before any pPrChange.
    let live_spacing = match (p.find("<w:spacing"), p.find("pPrChange")) {
        (Some(s), Some(c)) => s < c,
        (Some(_), None) => true,
        _ => false,
    };
    assert!(
        live_spacing && p.contains("w:line=\"240\""),
        "Word keeps live spacing line=240 on short-title MIX; p={p}"
    );
}
