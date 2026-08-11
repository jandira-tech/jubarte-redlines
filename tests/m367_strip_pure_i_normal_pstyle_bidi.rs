// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M367 — sdts × shape_group: pure-I has no pStyle=Normal / bidi=0.
//!
//! B (shape_group) writes Normal+bidi=0 on every paragraph. Word redlines omit
//! both on pure-I (bare mark rPr only). Keeping them thrash LO pagefair (−4.3).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn sdts_x_shape_pure_i_no_normal_pstyle_or_bidi() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sdts_basic_45263ca5.docx");
    let b = src.join("super_editor__shape_group_ce60e1e6.docx");
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

    // Pure-I shape body: "My test with some shapes."
    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut found = false;
    let mut pure_i_with_normal = 0usize;
    let mut pure_i_with_bidi = 0usize;
    let mut pure_i_count = 0usize;
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
        if has_ins && !has_del {
            pure_i_count += 1;
            if p.contains("pStyle") && p.contains("Normal") {
                pure_i_with_normal += 1;
            }
            if p.contains("<w:bidi") {
                pure_i_with_bidi += 1;
            }
            if p.contains("My test with some shapes") {
                found = true;
                assert!(
                    !p.contains("pStyle") || !p.contains("Normal"),
                    "Word pure-I bare — no pStyle=Normal: {p}"
                );
                assert!(!p.contains("<w:bidi"), "Word pure-I bare — no bidi=0: {p}");
            }
        }
    }
    assert!(found, "expected pure-I My test with some shapes");
    assert!(
        pure_i_count >= 5,
        "expected multi pure-I shape body, got {pure_i_count}"
    );
    assert_eq!(
        pure_i_with_normal, 0,
        "no pure-I should keep pStyle=Normal (got {pure_i_with_normal}/{pure_i_count})"
    );
    assert_eq!(
        pure_i_with_bidi, 0,
        "no pure-I should keep bidi=0 (got {pure_i_with_bidi}/{pure_i_count})"
    );
}
