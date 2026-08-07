// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M385 — nda×report: ListNumber pure-I free-meshes short Heading1 pure-D.
//!
//! Word MIX: ListNumber "[3] Data Safety…" + pure-D Heading1 "MUTUAL
//! NON-DISCLOSURE AGREEMENT" → MIX Heading1 with ins citation + del title.
//! M371 multi-word Heading skip + document-scale Jaccard miss blocked the fold
//! (−26.6 residual).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn nda_x_report_listnumber_mixes_short_heading1_title() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__nda_7f304918.docx");
    let b = src.join("evals__report_with_formatting_03f385ed.docx");
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
        if p.contains("Data Safety Monitoring Board")
            && p.contains("TRIAL-2025-001 Interim Safety")
            && p.contains("MUTUAL NON-DISCLOSURE AGREEMENT")
            && p.contains("<w:ins")
            && p.contains("<w:del")
        {
            found = true;
            assert!(
                p.contains("Heading1") || p.contains("Heading"),
                "Word MIX adopts short pure-D Heading1 onto ListNumber carrier"
            );
            break;
        }
    }
    assert!(
        found,
        "expected MIX ListNumber citation + Heading1 NDA title (M385)"
    );
}
