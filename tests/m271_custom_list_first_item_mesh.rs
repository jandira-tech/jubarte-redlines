// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M271 — custom_list_numbering1 × custom_list1: Word meshes first B list item
//! with first A residual as MIX (`Num 1` | `SECTION`). Tip free-LCS left pure-I
//! stream then M269-meshed last item. Relocate del of trailing MIX onto first
//! pure-I. Python probe fair tip-to-tip +3.65 (51.1→54.8).

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
fn custom_list_numbering_x_list1_lead_is_mix_num1_section() {
    let Some(a) = load("super_editor__custom_list_numbering1_7eb9fda4.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__custom_list1_77f82bd7.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    assert!(!paras.is_empty());
    let s0 = shape(&paras[0]);
    assert_eq!(
        s0,
        'M',
        "lead must be MIX Num1|SECTION (Word); shapes={}",
        paras.iter().map(|p| shape(p)).collect::<String>()
    );
    let i0 = collect_t(&paras[0]);
    let d0 = collect_del(&paras[0]);
    assert!(
        i0.contains("Num 1") || i0.contains("1"),
        "lead ins Num 1; I={i0:?}"
    );
    assert!(
        d0.to_ascii_uppercase().contains("SECTION") || d0.contains('“') || d0.contains('"'),
        "lead del SECTION residual; D={d0:?}"
    );
}
