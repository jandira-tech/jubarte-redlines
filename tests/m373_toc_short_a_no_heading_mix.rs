// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M373 — toc×broken_list: pure-I "a" stays separate from pure-D Heading.
//!
//! Word keeps ListParagraph residual "a" then pure-D Heading1 "Generalities".
//! Multi-del fold MIX-ed them (Heading1 on "aGeneralities", −2.3 thrash).
//! M365 floor ≥20 missed "Generalities" (12 alnum); lower to ≥10.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn toc_x_broken_list_short_a_stays_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__sd_2447_toc_tab_alignment_8319c14c.docx");
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
    let mut found_pure_i_a = false;
    let mut found_pure_d_gen = false;
    let mut bad_mix = false;
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
        if p.contains("Generalities") {
            if has_del && !has_ins {
                found_pure_d_gen = true;
            }
            if has_ins && has_del {
                bad_mix = true;
            }
        }
        // pure-I ListParagraph residual "a" without Generalities
        if has_ins
            && !has_del
            && p.contains("ListParagraph")
            && (p.contains(">a</w:t>") || p.contains(">a </w:t>"))
            && !p.contains("Generalities")
        {
            found_pure_i_a = true;
        }
    }
    assert!(found_pure_i_a, "expected pure-I ListParagraph residual 'a'");
    assert!(found_pure_d_gen, "expected pure-D Generalities Heading");
    assert!(!bad_mix, "must not MIX short 'a' with Heading Generalities");
}
