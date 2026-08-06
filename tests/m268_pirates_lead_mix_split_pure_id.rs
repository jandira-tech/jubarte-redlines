// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M268 un-gated REVERTED after full 04-22 (style demos 100→~50). M284 re-gates
//! with pure-D stream requirement: pirates lead pure-I|D; style demos stay MIX.
//! See also tests/m284_pirates_lead_mix_gated_pure_d_stream.rs.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn load_wb(name: &str) -> Option<Vec<u8>> {
    let p = Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source")
        .join(name);
    std::fs::read(p).ok()
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

fn para_shape(p: &str) -> &'static str {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    match (has_ins, has_del) {
        (true, true) => "M",
        (true, false) => "I",
        (false, true) => "D",
        _ => "E",
    }
}

fn collect_t(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:t") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:t>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 6..];
    }
    out
}

/// Style-demo lead must remain MIX (Word shape; M268 un-gated split LO-neg).
#[test]
fn style_demo_lead_stays_mix_after_m268_revert() {
    let Some(a) = load_wb("calibri_heading_2_right_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_wb("center_aligned_bold_text_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    assert!(!paras.is_empty());
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty())
        .expect("content");
    assert_eq!(
        para_shape(lead),
        "M",
        "style-demo lead MIX (not pure-I|D split); t={}",
        collect_t(lead)
    );
    assert!(
        collect_t(lead).to_ascii_lowercase().contains("center"),
        "lead ins title; t={}",
        collect_t(lead)
    );
}

/// M284 REVERTED after full 06-22 — free-LCS lead MIX allowed (was pure-I under gate).
#[test]
fn pirates_lead_residual_after_m284_revert() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: pirates source missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip: table indent source missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    assert!(!paras.is_empty());
    let s0 = para_shape(&paras[0]);
    assert!(
        s0 == "I" || s0 == "M",
        "pirates lead free-LCS residual after M284 revert; s0={s0}"
    );
}
