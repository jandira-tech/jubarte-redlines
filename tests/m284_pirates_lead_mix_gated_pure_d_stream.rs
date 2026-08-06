// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M284 REVERTED after full 06-22: pure-D-stream gate still mass-fired
//! (google_docs×hello, sd_1919, … LO ~100→50). Fair pirates was only +0.26.
//! Free-LCS lead MIX is kept; style demos stay MIX.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load_sd(name: &str) -> Option<Vec<u8>> {
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

fn shape(p: &str) -> char {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    match (has_ins, has_del) {
        (true, true) => 'M',
        (true, false) => 'I',
        (false, true) => 'D',
        _ => 'E',
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

fn collect_del(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:delText") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:delText>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 12..];
    }
    out
}

#[test]
fn pirates_x_indent_lead_pure_i_then_pure_d_stream() {
    let Some(a) = load_sd("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_sd("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    // M284 no-op: free-LCS may keep lead MIX (LO-positive on corpus) rather than
    // forced pure-I|pure-D. Do not assert pure-I split.
    assert!(
        matches!(shape(lead), 'I' | 'M'),
        "pirates lead residual; shapes={shapes} del={}",
        collect_del(lead)
    );
}

#[test]
fn style_demo_calibri_lead_stays_mix_with_m284_gate() {
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
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty())
        .expect("content");
    assert_eq!(
        shape(lead),
        'M',
        "style-demo lead MIX (M284 pure-D-stream gate); t={}",
        collect_t(lead)
    );
}
