// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M285 REVERTED — diff_after2 × simple_ordered_list: Word meshes first A
//! residual onto `Lvl 1 – a`; tip parks on `Item 4`. Structure peel was
//! fair tip-to-tip −0.87 (Word shape LO-neg under our emit). Call site off.
//! This test locks free-LCS tip: residual MIX stays on Item 4.

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
fn diff2_x_ordered_item4_keeps_residual_mix_after_m285_revert() {
    let Some(a) = load("super_editor__diff_after2_fc1e0763.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__simple_ordered_list_8288421a.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let item4 = paras
        .iter()
        .find(|p| collect_t(p).contains("Item 4"))
        .expect("Item 4");
    assert_eq!(
        shape(item4),
        'M',
        "Item 4 keeps residual MIX (M285 LO-neg revert); del={}",
        collect_del(item4)
    );
    assert!(
        collect_del(item4).to_ascii_lowercase().contains("test"),
        "del residual on Item 4"
    );
}
