// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M265 — annot2: Word has pure-D empty between "Enter duration" and the table.
//! Tip goes duration → table. Insert pure-D empty when pure-I content (with
//! del pilcrow) is immediately followed by a table.

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
fn annot2_pure_d_empty_between_duration_and_table() {
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
    // Body sequence: find "Enter duration" then next non-empty or table.
    // Expect pure-D empty (has del, no ins, empty text) before first table
    // after duration.
    let body = doc
        .split("<w:body>")
        .nth(1)
        .and_then(|b| b.split("</w:body>").next())
        .unwrap_or("");
    // crude: after first "Enter duration" chunk, before first tbl
    let Some(di) = body.find("Enter duration") else {
        panic!("no duration");
    };
    let after = &body[di..];
    let Some(ti) = after.find("<w:tbl") else {
        panic!("no table after duration");
    };
    let between = &after[..ti];
    // Must contain a pure-D empty: del in pPr without body text between
    assert!(
        between.contains("<w:del ") || between.contains("<w:del>") || between.contains("<w:del/"),
        "expected pure-D empty (del mark) between duration and table; between={between}"
    );
}
