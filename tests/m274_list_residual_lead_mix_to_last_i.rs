// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M271b / M274 — free-LCS list residual I* M D* is already Word-shaped when
//! the junction MIX carries a short del (`ONE`, `Num 1`). M271 must NOT peel
//! those onto the first pure-I (that over-fire produced M I* D* and LO-neg).
//! custom_list SECTION (long alpha) still relocates via M271 (see m271 test).

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

#[test]
fn complex_list_x_basic_keeps_junction_mix_one() {
    let Some(a) = load("super_editor__complex_list_def_short_fde20a67.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__basic_list_0fcfe705.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    assert_eq!(
        shape(&paras[0]),
        'I',
        "lead pure-I (Word); M271 must not peel ONE onto first; shapes={shapes}"
    );
    let mix = paras
        .iter()
        .position(|p| shape(p) == 'M' && collect_del(p).to_ascii_uppercase().contains("ONE"));
    assert!(mix.is_some(), "junction MIX keeps del ONE; shapes={shapes}");
    assert!(mix.unwrap() > 0, "ONE stays off lead; shapes={shapes}");
}

#[test]
fn list_def_mix_x_reimport_keeps_junction_mix() {
    let Some(a) = load("super_editor__list_def_mix_d7cec092.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__list_numbering_reimport_d788d573.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    assert_eq!(
        shape(&paras[0]),
        'I',
        "lead pure-I Word shape; shapes={shapes}"
    );
    assert!(
        shapes.contains('M'),
        "junction MIX retained; shapes={shapes}"
    );
}
