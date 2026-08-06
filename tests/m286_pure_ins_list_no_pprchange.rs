// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M286 — list_with_table_break × listwithspacernodes: Word pure-I list items
//! have no `w:pPrChange`. Tip stamped A list numPr/ind under pPrChange.

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

fn is_pure_ins(p: &str) -> bool {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    has_ins && !has_del
}

fn has_numpr(p: &str) -> bool {
    p.contains("numPr")
}

fn has_pprchange(p: &str) -> bool {
    p.contains("pPrChange")
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
fn list_table_x_spacer_pure_ins_list_no_pprchange() {
    let Some(a) = load("super_editor__list_with_table_break_ff0c4c1f.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__listwithspacernodes_cf53e890.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let pure_i_lists: Vec<_> = paras
        .iter()
        .filter(|p| is_pure_ins(p) && has_numpr(p) && !collect_t(p).trim().is_empty())
        .collect();
    assert!(!pure_i_lists.is_empty(), "expected pure-I list items");
    for p in &pure_i_lists {
        assert!(
            !has_pprchange(p),
            "pure-I list must not carry pPrChange; t={}",
            collect_t(p)
        );
    }
    // Export Restrictions is a known pure-I list item.
    let export = pure_i_lists
        .iter()
        .find(|p| collect_t(p).to_ascii_lowercase().contains("export"));
    assert!(export.is_some(), "Export Restrictions pure-I list item");
}
