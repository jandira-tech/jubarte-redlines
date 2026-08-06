// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M304 — exported_list_font×exporttest: Word pure-I list items with numPr carry
//! live `w:spacing line=240 lineRule=auto` (default single). Tip omits spacing.
//! Format emit only (structure already MIX b|APPOINTMENT). Fair tip-to-tip.

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

fn has_numpr(p: &str) -> bool {
    p.contains("<w:numPr") || p.contains("<w:numPr>")
}

fn has_live_spacing_line_240(p: &str) -> bool {
    // Live spacing before pPrChange only.
    let head = p.split("<w:pPrChange").next().unwrap_or(p);
    head.contains("<w:spacing")
        && (head.contains(r#"w:line="240""#) || head.contains("line=\"240\""))
}

#[test]
fn exporttest_pure_i_list_items_have_default_single_spacing() {
    let Some(a) = load("super_editor__exported_list_font_8e6db734.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__exporttest_68b3b898.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    // Pure-I+numPr non-empty list labels (a, x, …) need line=240.
    let mut checked = 0;
    for p in &paras {
        if shape(p) != 'I' || !has_numpr(p) {
            continue;
        }
        if collect_t(p).trim().is_empty() {
            continue;
        }
        assert!(
            has_live_spacing_line_240(p),
            "pure-I list item needs spacing line=240 (Word default); t={} p={}",
            collect_t(p),
            &p[..p.len().min(350)]
        );
        checked += 1;
    }
    assert!(checked >= 2, "expected ≥2 pure-I list labels; n={checked}");
}
