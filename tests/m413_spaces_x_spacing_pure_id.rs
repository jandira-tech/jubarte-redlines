// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M413 — short section base × short title-page next: pure-I/D (not free-mesh).
//!
//! Word pure-I's all title-page next then pure-D section base. Engine free-
//! meshed last next date into base engagement header (DI).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_paras(xml: &str) -> Vec<(bool, bool, String)> {
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut paras = Vec::new();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let mut text = String::new();
        for (tag, end_tag) in [("<w:t", "</w:t>"), ("<w:delText", "</w:delText>")] {
            let mut r = p;
            while let Some(i) = r.find(tag) {
                let r2 = &r[i..];
                let Some(gt) = r2.find('>') else { break };
                let after_t = &r2[gt + 1..];
                let Some(end) = after_t.find(end_tag) else {
                    break;
                };
                text.push_str(&after_t[..end]);
                r = &after_t[end + end_tag.len()..];
            }
        }
        paras.push((has_ins, has_del, text));
    }
    paras
}

#[test]
fn spaces_x_spacing_pure_i_date() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__doc_with_spaces_from_styles_734ca26f.docx");
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
    let paras = body_paras(&xml);

    let pure_i_date = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.contains("March") && t.contains("2040"));
    assert!(
        pure_i_date,
        "expected pure-I March 10, 2040; got {:?}",
        paras
            .iter()
            .map(|(i, d, t)| format!(
                "{}{} {:?}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(50).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
    // No DI free-mesh of date into engagement header.
    let di_date = paras
        .iter()
        .any(|(i, d, t)| *i && *d && (t.contains("March") || t.contains("ENGAGEMENT")));
    assert!(!di_date, "unexpected DI free-mesh of date/engagement");
}
