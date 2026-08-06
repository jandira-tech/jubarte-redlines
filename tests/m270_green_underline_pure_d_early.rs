// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M270 — green_underline_bullet × header_no_rels: tip emits pure-I* then
//! pure-D* (B header pages, then A list). Word starts pure-D green residual
//! right after the lead header pure-I. Reorder pure-D (no drawings) after
//! lead pure-I+empties. Python probe fair tip-to-tip +17.6 (54.9→72.5).

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
fn green_underline_x_header_pure_d_starts_after_lead() {
    let Some(a) = load("green_underline_bullet_list_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("header_no_rels.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    // Pure-D green residual must begin before mid pure-I page content.
    let first_d = shapes.find('D');
    let second_page = paras
        .iter()
        .position(|p| collect_t(p).contains("Second page"));
    assert!(
        first_d.is_some(),
        "expected pure-D residual; shapes={shapes}"
    );
    if let Some(sp) = second_page {
        assert!(
            first_d.unwrap() < sp,
            "pure-D green must precede mid pure-I 'Second page' (Word early del); shapes={shapes} sp={sp} first_d={first_d:?}"
        );
    }
    // Green title del still present early.
    let early: String = paras.iter().take(6).map(|p| collect_del(p)).collect();
    assert!(
        early.contains("Green") || early.contains("green"),
        "early pure-D must carry green list; early={early:?} shapes={shapes}"
    );
}
