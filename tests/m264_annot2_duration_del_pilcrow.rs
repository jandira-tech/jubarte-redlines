// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M264 — annot2×annotations_import: Word pure-I "Enter duration" carries a
//! deleted pilcrow (pPr/rPr/w:del) before the following pure-D empty + table.
//! Tip has pure-I pilcrow (w:ins). Gate: pure-I body immediately before tbl.

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
fn annot2_enter_duration_has_del_pilcrow_before_table() {
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

    // Find first "Enter duration" para; assert pPr has del (not only ins).
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
        let mut text = String::new();
        for part in body.split("<w:t").skip(1) {
            if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:t>")) {
                text.push_str(&part[a + 1..b]);
            }
        }
        if text.trim() != "Enter duration" {
            continue;
        }
        found = true;
        let ppr = if let Some(s) = chunk.find("<w:pPr") {
            let rest = &chunk[s..];
            if let Some(e) = rest.find("</w:pPr>") {
                &rest[..e + "</w:pPr>".len()]
            } else {
                ""
            }
        } else {
            ""
        };
        assert!(
            ppr.contains("<w:del ") || ppr.contains("<w:del>") || ppr.contains("<w:del/"),
            "duration pure-I must carry del pilcrow in pPr; pPr={ppr} chunk={chunk}"
        );
        break;
    }
    assert!(found, "expected Enter duration para");
}
