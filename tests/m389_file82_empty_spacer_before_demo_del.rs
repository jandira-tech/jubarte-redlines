// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M389 — file_82×file_83: empty pure-I spacer before pure-D demo title.
//!
//! Word keeps an empty pure-I between last content pure-I and pure-D
//! "Title Style Centered Demo". M85a stripped the spacer (−18.9 pagefair).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file82_x_83_empty_pure_i_before_title_demo_del() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_82.docx");
    let b = src.join("file_83.docx");
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
    // Collect body para kinds + short text
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
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del");
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
        // delText
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

    // Find pure-D "Title Style Centered Demo"
    let title_i = paras
        .iter()
        .position(|(_i, d, t)| *d && t.contains("Title Style Centered Demo"));
    let Some(ti) = title_i else {
        panic!("expected pure-D Title Style Centered Demo");
    };
    assert!(ti > 0, "expected a para before pure-D title residual");
    let (has_ins, has_del, text) = &paras[ti - 1];
    assert!(
        *has_ins && !*has_del && text.trim().is_empty(),
        "Word keeps empty pure-I spacer before pure-D demo title; prev={has_ins}/{has_del}/{text:?}"
    );
}
