// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M419 — math delimiter next (OMML shells) × math eqarr base: MIX last shell
//! into first pure-D title.
//!
//! Word IIIIMDD (last math pure-I folded into "m:d Delimiter…"). Engine kept
//! pure-I/D (IIIIIDDD) because empty_shell × long pure-D was blocked when
//! document-scale multi-del skip fired.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn math_delimiter_x_eqarr_mixes_title() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__math_delimiter_tests_c9d034dc.docx");
    let b = src.join("behavior__math_eqarr_tests_40a1adb0.docx");
    // Note: pair order in corpus is delimiter base × eqarr next in some maps;
    // bottom score uses delimiter × eqarr. Engine pure-I eqarr math shells then
    // pure-D delimiter titles when next is textless OMML.
    // Use delimiter as base, eqarr as next (matches score key order).
    let base = a;
    let next = b;
    if !base.exists() || !next.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&base).unwrap(),
        &std::fs::read(&next).unwrap(),
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

    // Find MIX paragraph containing "Delimiter" or "m:d".
    let mut rest = xml.as_str();
    let mut mix_title = false;
    let mut pure_i_then_pure_d_title = false;
    let mut saw_pure_i = false;
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
        let t = text.to_ascii_lowercase();
        if has_ins && has_del && (t.contains("delimiter") || t.contains("m:d")) {
            mix_title = true;
        }
        if has_ins && !has_del {
            saw_pure_i = true;
        }
        if saw_pure_i && has_del && !has_ins && t.contains("delimiter") {
            pure_i_then_pure_d_title = true;
        }
    }
    assert!(
        mix_title,
        "expected MIX of math pure-I into Delimiter title; pure_i_then_pure_d={pure_i_then_pure_d_title}"
    );
}
