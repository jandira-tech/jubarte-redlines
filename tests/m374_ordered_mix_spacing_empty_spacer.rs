// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M374 — ordered×sublist residual: MIX carries pure-I spacing; empty after MIX.
//!
//! Word MIX for "Lvl 1 – a" has line=276 + ind from pure-I "a", and a pure-I
//! empty spacer follows the MIX. Body zip alone left consecutive MIX without
//! spacing (−2.2 thrash residual).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn ordered_x_sublist_mix_has_line276_and_empty_after() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__simple_ordered_list_8288421a.docx");
    let b = src.join("super_editor__sublist_issue_66a1800a.docx");
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

    // MIX with Lvl 1 – a (del) + short-label ins must carry pure-I line=276.
    let idx = xml
        .find("Lvl 1")
        .expect("expected Lvl 1 delText in redline");
    // Search backward for containing pPr spacing
    let window_start = idx.saturating_sub(800);
    let window = &xml[window_start..idx];
    assert!(
        window.contains("line=\"276\""),
        "Word MIX carries pure-I line=276 near Lvl 1 residual"
    );

    // Pattern: MIX (ins+del Lvl) then pure-I empty (mark ins only) then another MIX.
    // Coarse: two pure-I empties with line=276 appear after first Lvl MIX region.
    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut saw_mix_lvl = false;
    let mut empty_after = false;
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
        let has_lvl = p.contains("Lvl");
        if has_ins && has_del && has_lvl {
            saw_mix_lvl = true;
            continue;
        }
        if saw_mix_lvl && !empty_after {
            // pure-I empty: mark ins, no del body, no delText
            if has_ins && !p.contains("delText") && !p.contains("Lvl") {
                let has_body_t = p.contains("<w:t ") || p.contains("<w:t>");
                if !has_body_t {
                    empty_after = true;
                }
            }
            // only check the first para after first MIX
            if !empty_after {
                // allow one more if this wasn't empty
                saw_mix_lvl = false; // require immediately after
            }
        }
    }
    // Simpler structural check: after first Lvl MIX, next p is mark-only pure-I.
    // Re-scan with index.
    let mut paras = Vec::new();
    let mut rest2 = xml.as_str();
    if let Some(i) = rest2.find("<w:body") {
        rest2 = &rest2[i..];
    }
    if let Some(i) = rest2.find("</w:body>") {
        rest2 = &rest2[..i];
    }
    while let Some(start) = rest2.find("<w:p") {
        let after = &rest2[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        paras.push(&after[..end_rel]);
        rest2 = &after[end_rel..];
    }
    let mut ok = false;
    for i in 0..paras.len().saturating_sub(1) {
        let p = paras[i];
        let n = paras[i + 1];
        if p.contains("<w:ins")
            && p.contains("<w:del")
            && p.contains("Lvl 1")
            && p.contains("line=\"276\"")
            && n.contains("<w:ins")
            && !n.contains("delText")
            && !n.contains("<w:t ")
            && !n.contains("<w:t>")
        {
            ok = true;
            break;
        }
    }
    assert!(
        ok,
        "expected MIX Lvl 1 with line=276 followed by pure-I empty spacer"
    );
}
