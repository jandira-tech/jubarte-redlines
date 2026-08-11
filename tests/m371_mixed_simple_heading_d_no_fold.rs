// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M371 — word_mixed × word_simple: pure-D Heading title stays separate.
//!
//! Word: pure-I "Chapter One" then pure-D "Mixed Formatting Test" (Heading1),
//! not MIX of last pure-I body with that heading. Multi-del fold adopted
//! Heading1 onto "A final paragraph…" (−2.2 thrash).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn mixed_x_simple_heading_title_stays_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sd_1919_word_mixed_33e049ca.docx");
    let b = src.join("super_editor__sd_1919_word_simple_e3a4a818.docx");
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
    let mut found_pure_d_title = false;
    let mut found_bad_mix = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        if p.contains("Mixed Formatting Test") {
            let has_ins = p.contains("<w:ins");
            let has_del = p.contains("<w:del");
            if has_del && !has_ins {
                found_pure_d_title = true;
                assert!(
                    p.contains("Heading1") || p.contains("Heading"),
                    "pure-D title should keep Heading style"
                );
            }
            if has_ins && has_del && p.contains("final paragraph") {
                found_bad_mix = true;
            }
        }
        // final pure-I body must not wear Heading1 from the pure-D title.
        if p.contains("final paragraph")
            && p.contains("<w:ins")
            && p.contains("Heading1")
            && !p.contains("Mixed Formatting")
        {
            found_bad_mix = true;
        }
    }
    assert!(
        found_pure_d_title,
        "Word keeps pure-D Mixed Formatting Test Heading separate"
    );
    assert!(
        !found_bad_mix,
        "must not MIX final pure-I body with Heading1 from pure-D title"
    );
}
