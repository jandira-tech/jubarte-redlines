// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M434 — last MIX list residual keeps live numPr/pStyle (not pPrChange-only).
//!
//! image_doc (drawing-only base) × document (list next): last para is MIX
//! (ins list text + del drawing). Word keeps live `w:numPr` + `w:pStyle` on
//! that MIX; M94 parked them into pPrChange and cleared live list chrome.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn image_doc_x_document_last_mix_keeps_live_numpr() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__image_doc_c7d007ac.docx");
    let b = src.join("evals__document_9bff7e1b.docx");
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
    let xml = {
        let mut f = zip.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };

    // Prefer the last MIX para (ins+del); else last body p before sectPr.
    let body = xml
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or(&xml);
    let mut last_p = None;
    let mut last_mix = None;
    let mut rest = body;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        if p.contains("<w:sectPr") {
            break;
        }
        last_p = Some(p);
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText") || p.contains("<w:drawing");
        if has_ins && has_del {
            last_mix = Some(p);
        }
        rest = &after[end_rel..];
    }
    let last = last_mix.or(last_p).expect("last paragraph");
    assert!(
        last.contains("<w:ins")
            && (last.contains("<w:del")
                || last.contains("<w:delText")
                || last.contains("<w:drawing")),
        "last list residual must be MIX with drawing del"
    );
    // Live pPr (before any pPrChange) must still carry numPr.
    let ppr = last
        .find("<w:pPr")
        .map(|i| &last[i..])
        .and_then(|s| {
            // first pPr only
            let end = s.find("</w:pPr>").map(|j| j + "</w:pPr>".len())?;
            Some(&s[..end])
        })
        .expect("pPr");
    // Split off pPrChange content if present — live props are outside it.
    let live = if let Some(i) = ppr.find("<w:pPrChange") {
        &ppr[..i]
    } else {
        ppr
    };
    assert!(
        live.contains("numPr") || live.contains("w:numPr"),
        "Word keeps live numPr on last MIX list residual; live pPr was: {live}"
    );
    assert!(
        live.contains("pStyle") || live.contains("w:pStyle"),
        "Word keeps live pStyle on last MIX list residual; live pPr was: {live}"
    );
}
