// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M446 — last MIX free-mesh keeps live spacing (subtitle / short demos).
//!
//! subtitle_style × subtitle_default_missing (~82.8): Word free-meshes long
//! ins body × short del residual and keeps live `spacing line=240`. M94
//! parked spacing into pPrChange (M444 only covered short-ins free-mesh).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn subtitle_style_last_mix_keeps_live_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("subtitle_style_demo_id_paraid_overflow.docx");
    let b = src.join("subtitle_style_demo_style_default_missing.docx");
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

    // Last body MIX (Subtitle style provides… × Document Description).
    let body = xml
        .find("<w:body")
        .and_then(|i| xml[i..].find('>').map(|j| i + j + 1))
        .unwrap_or(0);
    let body_end = xml.find("</w:body>").unwrap_or(xml.len());
    let body_xml = &xml[body..body_end];
    let mut last_mix: Option<&str> = None;
    let mut rest = body_xml;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ") || after.starts_with("<w:p/"))
        {
            rest = &after[4..];
            continue;
        }
        if after.starts_with("<w:p/>") || after.starts_with("<w:p />") {
            rest = &after[after.find('>').unwrap_or(5) + 1..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if p.contains("<w:sectPr") {
            continue;
        }
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        if has_ins && has_del {
            last_mix = Some(p);
        }
    }
    let p = last_mix.expect("expected last MIX paragraph");
    let live_spacing = match (p.find("<w:spacing"), p.find("pPrChange")) {
        (Some(s), Some(c)) => s < c,
        (Some(_), None) => true,
        _ => false,
    };
    assert!(
        live_spacing && (p.contains("w:line=\"240\"") || p.contains("w:line='240'")),
        "Word keeps live spacing line=240 on subtitle free-mesh MIX; p={p}"
    );
}
