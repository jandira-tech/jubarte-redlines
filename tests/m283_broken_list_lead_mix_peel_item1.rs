// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M283 — broken_list_missing × multiple_nodes: Word is pure-I lead
//! `Onetestafter space`, pure-I `TWO`, then pure-D stream starting with
//! `Item 1`. Tip meshes Item1 del onto lead as MIX. Peel del off lead MIX
//! onto pure-D stream head.

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
fn broken_list_missing_x_multi_lead_pure_i_item1_pure_d() {
    let Some(a) = load("super_editor__broken_list_missing_items_36b4199e.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__multiple_nodes_in_list_79d915a2.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    assert!(!paras.is_empty());

    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    assert_eq!(
        shape(lead),
        'I',
        "lead pure-I (not MIX with Item1); shapes={shapes} del={}",
        collect_del(lead)
    );
    let t = collect_t(lead);
    assert!(
        t.to_ascii_lowercase().contains("one") || t.to_ascii_lowercase().contains("test"),
        "lead ins list text; t={t:?}"
    );
    assert!(
        !collect_del(lead).to_ascii_lowercase().contains("item"),
        "Item residual must leave lead; del={}",
        collect_del(lead)
    );

    // First pure-D residual should carry Item 1.
    let first_d = paras
        .iter()
        .find(|p| shape(p) == 'D' && !collect_del(p).trim().is_empty())
        .expect("pure-D residual");
    assert!(
        collect_del(first_d).to_ascii_lowercase().contains("item"),
        "first pure-D Item residual; del={}",
        collect_del(first_d)
    );
}

/// Style demos must keep lead MIX (M268 lesson — do not over-peel).
#[test]
fn style_demo_calibri_lead_stays_mix() {
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
    assert_eq!(shape(lead), 'M', "style-demo lead MIX");
}
