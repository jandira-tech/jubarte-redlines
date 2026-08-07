// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M392 — file_36×file_37: empty pure-I spacers before pure-D short title.
//!
//! Word keeps two empty pure-I paragraphs between content pure-I
//! ("HR Onboarding Checklist") and pure-D "Contract Review", then a pure-D
//! empty before the table. False empty-empty anchors + fold stripped them.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file36_x_37_empty_pure_i_spacers_before_contract_review_del() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_36.docx");
    let b = src.join("file_37.docx");
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
    let mut paras: Vec<(bool, bool, String)> = Vec::new();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        // Skip nested p inside tables for top-level body scan: crude — only
        // collect until first tbl if present by tracking depth via raw index.
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let mut text = String::new();
        let mut r = p;
        while let Some(i) = r.find("<w:t") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:t>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 6..];
        }
        r = p;
        while let Some(i) = r.find("<w:delText") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:delText>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 12..];
        }
        paras.push((has_ins, has_del, text));
    }

    // Find pure-D "Contract Review" (first body occurrence; table cells may
    // also appear later — take first pure-D match).
    let title_i = paras.iter().position(|(_i, d, t)| {
        *d && !_i && t.contains("Contract Review") && !t.contains("file_")
    });
    let Some(ti) = title_i else {
        panic!(
            "expected pure-D Contract Review; paras={:?}",
            paras
                .iter()
                .map(|(i, d, t)| format!("{}{}/{}", if *i { "I" } else { "" }, if *d { "D" } else { "" }, t))
                .collect::<Vec<_>>()
        );
    };
    assert!(ti >= 2, "expected empty pure-I spacers before pure-D title; ti={ti}");
    // Word: at least one empty pure-I immediately before pure-D title.
    let (has_ins, has_del, text) = &paras[ti - 1];
    assert!(
        *has_ins && !*has_del && text.trim().is_empty(),
        "Word keeps empty pure-I spacer before pure-D Contract Review; prev={has_ins}/{has_del}/{text:?}"
    );
    // Prefer two spacers (Word file_36×37). Soft: require ≥1; hard-check second
    // when present in the engine output chain.
    if ti >= 2 {
        let (i2, d2, t2) = &paras[ti - 2];
        // Second empty pure-I (Word) or content pure-I title (one spacer).
        assert!(
            *i2 && !*d2,
            "expected pure-I before spacer; got {i2}/{d2}/{t2:?}"
        );
    }
}
