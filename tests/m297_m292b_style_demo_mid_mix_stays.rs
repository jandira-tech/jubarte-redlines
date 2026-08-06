// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M292b — full 06-22 regression: calibri×center mid MIX (short body del ~9 toks)
//! was peeled to pure-I|pure-D (LO ~100→57). auto_listenter keeps peel (del ≥20
//! toks, often hundreds). Style demos keep mid MIX.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
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

#[test]
fn calibri_x_center_mid_mix_stays() {
    let Some(a) = load("calibri_heading_2_right_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("center_aligned_bold_text_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let shapes: String = body_paras(&document_xml(&out))
        .iter()
        .map(|p| shape(p))
        .collect();
    // Word/02-22 shape MIMD — at least two MIX, not MIIDD (single mid peel).
    let m_count = shapes.chars().filter(|&c| c == 'M').count();
    assert!(
        m_count >= 2,
        "style-demo mid MIX must stay (M292b n_del≥20); shapes={shapes}"
    );
    assert!(
        !shapes.contains("IIDD") && !shapes.starts_with("MIIDD"),
        "must not peel mid MIX to pure-I|D stream; shapes={shapes}"
    );
}
