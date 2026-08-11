// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M448 — pure-I-dominant body drops trailing bare empty after pure-D residual.
//!
//! diff_after8 × doc_with_spacing (~75.6): Word ends `…IDD`. Engine left
//! trailing bare empty EQ (`…IDDE`).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn diff_after8_no_trailing_bare_empty_after_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__diff_after8_58e5c288.docx");
    let b = src.join("super_editor__doc_with_spacing_e3d47bd7.docx");
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

    let body = xml
        .find("<w:body")
        .and_then(|i| xml[i..].find('>').map(|j| i + j + 1))
        .unwrap_or(0);
    let body_end = xml.find("</w:body>").unwrap_or(xml.len());
    let body_xml = &xml[body..body_end];
    let mut last_p: Option<&str> = None;
    let mut rest = body_xml;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ") || after.starts_with("<w:p/"))
        {
            rest = &after[4..];
            continue;
        }
        if after.starts_with("<w:p/>") || after.starts_with("<w:p />") {
            last_p = Some(&after[..after.find('>').unwrap_or(5) + 1]);
            rest = &after[after.find('>').unwrap_or(5) + 1..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        if p.contains("<w:sectPr") && !p.contains("<w:t") && !p.contains("<w:delText") {
            continue;
        }
        last_p = Some(p);
    }
    let p = last_p.expect("expected last body paragraph");
    let has_del = p.contains("<w:del") || p.contains("<w:delText");
    let has_ins = p.contains("<w:ins");
    let has_t = p.contains("<w:t") || p.contains("<w:delText");
    assert!(
        has_del && !has_ins,
        "Word ends on pure-D residual, not bare empty; last_p={p}"
    );
    assert!(has_t, "last pure-D has delText content; last_p={p}");
}
