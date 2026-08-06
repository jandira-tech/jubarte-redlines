// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M270c — full 06-22 regression: diff_after9×diff_before free-LCS is long pure-I
//! stream then single short pure-D ("This is the end."). M270 relocated that pure-D
//! early after the lead pure-I (LO ~97→46). Require ≥2 non-empty pure-D and
//! remaining pure-I after lead ≤3 so single trailing pure-D stays late.

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

#[test]
fn diff_after9_single_trailing_pure_d_stays_late() {
    let Some(a) = load("super_editor__diff_after9_a9d4b4b0.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__diff_before_019353a2.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    // Find pure-D carrying "This is the end"
    let d_idx = paras
        .iter()
        .position(|p| shape(p) == 'D' && collect_del(p).to_ascii_lowercase().contains("end"));
    assert!(
        d_idx.is_some(),
        "expected pure-D 'This is the end'; shapes={shapes}"
    );
    let d_idx = d_idx.unwrap();
    // Must not sit early after first pure-I: most pure-I content precedes it.
    let ne_i_before = paras[..d_idx]
        .iter()
        .filter(|p| shape(p) == 'I' && !collect_t(p).trim().is_empty())
        .count();
    let ne_i_after = paras[d_idx + 1..]
        .iter()
        .filter(|p| shape(p) == 'I' && !collect_t(p).trim().is_empty())
        .count();
    assert!(
        ne_i_before >= 5 && ne_i_after == 0,
        "single trailing pure-D must stay at end of pure-I stream (M270c); \
         shapes={shapes} d_idx={d_idx} ne_i_before={ne_i_before} ne_i_after={ne_i_after}"
    );
}
