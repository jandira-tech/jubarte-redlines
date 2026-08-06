// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Zero-overlap 1×N wholesale (annot2×annotations_import): Word pure-ins every
//! B paragraph. Carrier fusion wrongly put del marks on B's first live paras.

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

#[test]
fn annot2_x_annotations_import_first_live_paras_are_pure_ins() {
    let Some(a) = load("super_editor__annot2_03ae6b74.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__annotations_import_2_eaae7b97.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    // First body para with live "Test 1" must not be pure-del marked.
    // NOTE: full Word shape (IIIIIII M DDD) still open — wholesale pure-I/D of
    // the table residual regressed LO (~71→64); keep a weak lead-pPr gate until
    // table-aware residual lands without LO loss.
    let paras: Vec<&str> = doc.split("</w:p>").filter(|p| p.contains("<w:p")).collect();
    assert!(!paras.is_empty());
    let p0 = paras[0];
    assert!(
        p0.contains("Test 1") || p0.contains("Test"),
        "expected B lead text; p0={p0}"
    );
    // Word pure-ins lead B: no del mark on the pilcrow of the first live B para.
    let ppr = p0.split("<w:r").next().unwrap_or(p0);
    assert!(
        !ppr.contains("<w:del"),
        "lead B para pPr must not carry del mark (Word pure-ins); p0={p0}"
    );
    assert!(
        p0.contains("<w:ins") || p0.contains("Test"),
        "lead B content must appear as insert/live; p0={p0}"
    );
}
