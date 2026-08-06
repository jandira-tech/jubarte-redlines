// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! superdoc_table_tester × nda: Word pure-I all NDA signatures then pure-D
//! table residual (with at most a boundary MIX Date×title). Free LCS meshes
//! "Receiving Party" into A residual "LTR text…" (MIX).

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
fn superdoc_table_x_nda_receiving_party_is_pure_ins() {
    let Some(a) = load("super_editor__superdoc_table_tester_3b2de2e1.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("evals__nda_7f304918.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    let recv = doc
        .split("</w:p>")
        .find(|p| p.contains("Receiving Party") && p.contains("Signature"))
        .expect("Receiving Party para");
    // Word pure-I: no del mesh of LTR residual into the signature line.
    assert!(
        !recv.contains("LTR text")
            && !recv.contains("delText")
            && !recv.contains("<w:del ")
            && !recv.contains("<w:del>"),
        "Receiving Party must be pure-ins, not meshed with A LTR residual; para={recv}"
    );
    assert!(
        recv.contains("<w:ins") || recv.contains("w:ins "),
        "Receiving Party must appear as insert"
    );
    // A residual LTR still deleted somewhere.
    assert!(
        doc.contains("LTR text") && doc.contains("<w:del"),
        "A LTR residual must remain as pure-del"
    );
}
