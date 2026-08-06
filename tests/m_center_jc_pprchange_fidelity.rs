// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Format/visual LO fidelity: Word keeps `w:jc` history via `w:pPrChange` on
//! center-alignment demo redlines. Band pair
//! `center_alignment_demo × center_bold_demo` (shape-matched residual, LO ~83.6
//! vs Word 100) was missing pPrChange on the title — empty `<w:pPr/>` — so LO
//! lost center layout history that Word records.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let roots = [
        Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source"),
        Path::new("tests/corpus/broken_ones_two/sources"),
    ];
    for root in roots {
        let p = root.join(name);
        if p.is_file() {
            return std::fs::read(p).ok();
        }
    }
    None
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn body_paras(doc: &str) -> Vec<String> {
    doc.split("</w:p>")
        .filter(|c| c.contains("<w:p") || c.contains("<w:p>"))
        .map(|s| format!("{s}</w:p>"))
        .collect()
}

fn ppr_of(para: &str) -> String {
    if let Some(i) = para.find("<w:pPr") {
        if let Some(end) = para[i..].find("</w:pPr>") {
            return para[i..i + end + "</w:pPr>".len()].to_string();
        }
        if let Some(end) = para[i..].find("/>") {
            return para[i..i + end + 2].to_string();
        }
    }
    String::new()
}

/// Word: title carries pPrChange with old jc=center (A had center, B dropped it).
/// Ours previously emitted empty `<w:pPr/>` on that title.
#[test]
fn center_align_x_center_bold_title_has_pprchange_jc() {
    let Some(a) = load("center_alignment_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let Some(b) = load("center_bold_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    assert!(!paras.is_empty(), "expected body paragraphs");
    let p0 = &paras[0];
    let ppr = ppr_of(p0);
    assert!(
        ppr.contains("pPrChange"),
        "title must carry pPrChange for jc history (Word parity); pPr={ppr}"
    );
    assert!(
        ppr.contains("jc") && ppr.contains("center"),
        "pPrChange/old must record jc=center; pPr={ppr}"
    );
    // Empty self-closing pPr is the measured defect.
    assert!(
        !ppr.trim_end_matches('>').ends_with("pPr /") && ppr != "<w:pPr />" && ppr != "<w:pPr/>",
        "title pPr must not be empty; pPr={ppr}"
    );
}

/// When next adds center jc (bare A → centered B), live jc + empty pPrChange.
#[test]
fn center2_x_center1_title_live_jc_with_pprchange() {
    let Some(a) = load("center_alignment_demo_id_paraid_overflow_2.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let Some(b) = load("center_alignment_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let p0 = &paras[0];
    let ppr = ppr_of(p0);
    // Live jc before any pPrChange.
    let live = ppr.split("pPrChange").next().unwrap_or(&ppr);
    assert!(
        live.contains("w:val=\"center\"") || live.contains("w:val='center'"),
        "title must keep live center jc; pPr={ppr}"
    );
}
