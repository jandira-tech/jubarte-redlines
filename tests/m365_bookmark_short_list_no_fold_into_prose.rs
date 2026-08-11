// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M365 — bookmark × broken_complex_list: short pure-I "a" stays separate.
//!
//! Word keeps multi pure-I list residual "a" then pure-D bookmark prose.
//! Multi-del boundary fold invented MIX "aThis is a paragraph with a simple
//! bookmark…" (−5 pagefair). Skip when last pure-I is very short and first
//! pure-D is long unrelated prose.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn bookmark_x_broken_list_short_a_stays_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__bookmark_use_cases_d20f31f6.docx");
    let b = src.join("super_editor__broken_complex_list_293fda86.docx");
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
    let mut found_short_a = false;
    let mut found_bookmark_pure_d = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        // Pure-I ListParagraph residual "a" only (not MIX with bookmark text).
        if p.contains("ListParagraph")
            && p.contains("<w:ins")
            && !p.contains("<w:del")
            && (p.contains(">a</w:t>") || p.contains(">a</"))
            && !p.contains("bookmark")
            && !p.contains("This is a paragraph")
        {
            found_short_a = true;
        }
        if p.contains("This is a paragraph with a simple bookmark") {
            found_bookmark_pure_d = true;
            assert!(
                p.contains("<w:del") && !p.contains("<w:ins"),
                "bookmark prose must stay pure-D, not MIX with list residual: {p}"
            );
        }
    }
    assert!(found_short_a, "expected pure-I ListParagraph residual 'a'");
    assert!(
        found_bookmark_pure_d,
        "expected pure-D bookmark prose paragraph"
    );
}
