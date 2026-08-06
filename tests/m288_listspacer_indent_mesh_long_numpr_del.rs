// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M288 — listwithspacernodes × list_with_indents: Word meshes last pure-I
//! list item with following long pure-D body residual even when pure-D has
//! numPr. M269 previously required pure-D without numPr.

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
fn spacer_x_indents_last_list_meshes_long_body_del() {
    let Some(a) = load("super_editor__listwithspacernodes_cf53e890.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__list_with_indents_efc7d4f5.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    // Find MIX that carries long body residual (denounce…).
    let mesh = paras.iter().find(|p| {
        shape(p) == 'M'
            && collect_del(p).to_ascii_lowercase().contains("denounce")
            && collect_t(p).to_ascii_lowercase().contains("unless")
    });
    assert!(
        mesh.is_some(),
        "Unless…|long body residual MIX; shapes={shapes}"
    );
}

/// M269b still: Export Restrictions must not mesh short pure-D ONE.
#[test]
fn list_table_export_still_not_mesh_one() {
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
    let export = paras
        .iter()
        .find(|p| collect_t(p).to_ascii_lowercase().contains("export"))
        .expect("Export");
    assert_eq!(shape(export), 'I', "Export stays pure-I");
    assert!(
        !collect_del(export).to_ascii_uppercase().contains("ONE"),
        "ONE not meshed onto Export"
    );
}
