// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M263 — annot2×annotations_import: Word after pure-I "Test 1" has two pure-I
//! empties then "Test 2". Tip had pure-D empty + pure-I empty. Convert the
//! pure-D empty mark to pure-I (not drop) so both B empties survive as pure-I.

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

fn para_body_text(chunk: &str) -> String {
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
    for part in body.split("<w:delText").skip(1) {
        if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:delText>")) {
            text.push_str(&part[a + 1..b]);
        }
    }
    text
}

#[test]
fn annot2_after_test1_two_pure_i_empties_no_pure_d() {
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
    let paras: Vec<&str> = doc.split("</w:p>").filter(|p| p.contains("<w:p")).collect();

    let mut test1_i = None;
    for (i, p) in paras.iter().enumerate() {
        let body = if let Some(idx) = p.rfind("</w:pPr>") {
            &p[idx + "</w:pPr>".len()..]
        } else {
            p
        };
        if para_body_text(p).trim() != "Test 1" {
            continue;
        }
        if body.contains("<w:del") || body.contains("delText") {
            continue;
        }
        test1_i = Some(i);
        break;
    }
    let ti = test1_i.expect("Test 1 pure-I");
    let mut pure_i_empties = 0usize;
    for p in &paras[ti + 1..] {
        let text = para_body_text(p);
        if text.trim() == "Test 2" {
            break;
        }
        if !text.trim().is_empty() {
            continue;
        }
        let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:del/");
        let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>") || p.contains("<w:ins/");
        assert!(
            !has_del || has_ins,
            "empty between Test 1 and Test 2 must not be pure-D; para={p}"
        );
        if has_ins && !has_del {
            pure_i_empties += 1;
        }
    }
    assert!(
        pure_i_empties >= 2,
        "Word has two pure-I empties after Test 1; got {pure_i_empties}"
    );
}
