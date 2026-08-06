// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M260 — two_col_index×tab: Word pure-I "Page4" carries live spacing
//! line=276 (from B). Tip had tabs only (no spacing) → default line height.

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
fn two_col_page4_pure_ins_has_live_spacing() {
    let Some(a) = load("super_editor__sd_1480_two_col_index_0138dccc.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1480_two_col_tab_positions_00953280.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    // Word pure-I Page4: body is equal runs "Page"+"4" (not ins-wrapped) with
    // pPr rPr/w:ins pilcrow mark; no delText. Must carry live spacing 276.
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("<w:p") {
            continue;
        }
        let body = if let Some(idx) = chunk.rfind("</w:pPr>") {
            &chunk[idx + "</w:pPr>".len()..]
        } else {
            chunk
        };
        if body.contains("<w:del") || body.contains("delText") {
            continue;
        }
        let mut text = String::new();
        for part in body.split("<w:t").skip(1) {
            if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:t>")) {
                text.push_str(&part[a + 1..b]);
            }
        }
        if text.replace('\t', "").trim() != "Page4" {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("w:spacing") && chunk.contains("w:line=\"276\""),
            "pure-I Page4 must have live spacing 276; chunk={chunk}"
        );
        break;
    }
    assert!(found, "expected pure-I Page4 para (equal body, no del)");
}
