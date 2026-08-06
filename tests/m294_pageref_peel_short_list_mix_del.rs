// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M294 — pageref_standalone × restart_numbering_sub_list:
//! Tip free-LCS meshes list residual `C` with `Introduction1` as MIX before
//! pure-D stream. Word peels pure-I `C` + pure-D `Introduction1`. M290 required
//! n_del ≥ 4 tokens; `Introduction1` is one alnum token — allow long del by
//! character count when short ins (≤2 chars).

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
fn pageref_x_restart_c_not_meshed_with_introduction1() {
    let Some(a) = load("behavior__pageref_standalone_uppercase_h_7701e07f.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__restart_numbering_sub_list_85ddcb79.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let shapes: String = paras.iter().map(|p| shape(p)).collect();

    // Structure: pure-I list prefix then pure-D Introduction residual (no MIX).
    assert!(!shapes.contains('M'), "no MIX after peel; shapes={shapes}");
    assert!(
        shapes.starts_with("IIIIIII") || shapes.starts_with("IIIIII"),
        "lead pure-I list residual including C; shapes={shapes}"
    );
    let intro_d = paras
        .iter()
        .any(|p| shape(p) == 'D' && collect_del(p).contains("Introduction"));
    assert!(intro_d, "Introduction as pure-D; shapes={shapes}");
    // Any pure-I that still holds Introduction del would be a fail — none should.
    let meshed = paras
        .iter()
        .any(|p| shape(p) == 'M' && collect_del(p).contains("Introduction"));
    assert!(!meshed, "Introduction not meshed; shapes={shapes}");
}

#[test]
fn list_spacer1_and_bookmark_still_peeled() {
    let Some(a) = load("super_editor__list_spacer1_06383c66.docx") else {
        return;
    };
    let Some(b) = load("super_editor__list_with_break_exported_broken_45f7bd19.docx") else {
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    let b_para = paras.iter().find(|p| {
        let t = collect_t(p).trim().to_string();
        t == "b" || t.starts_with('b')
    });
    if let Some(bp) = b_para {
        assert!(!collect_del(bp).contains("14.9"), "M290 still peels 14.9");
    }
}
