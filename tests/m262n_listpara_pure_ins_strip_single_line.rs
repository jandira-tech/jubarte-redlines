// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M262n — comment×complex_list: Word pure-I ListParagraph+numPr items omit
//! B's body `after=0 line=240 lineRule=auto`. M262 broad pure-I strip killed
//! multipara (PARTIES uses bare pPr spacing, not ListParagraph) — gate to
//! ListParagraph + numPr only.

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
fn comment_x_complex_list_listpara_pure_ins_omits_line240() {
    let Some(a) = load("super_editor__comment_23ee5ec1.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__complex_list_def_issue_326369f9.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("ListParagraph") || !chunk.contains("numPr") {
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
        if text.trim() != "ONE" {
            continue;
        }
        found = true;
        assert!(
            !chunk.contains("w:line=\"240\""),
            "pure-I ListParagraph+numPr ONE must omit line=240; chunk={chunk}"
        );
        break;
    }
    assert!(found, "expected pure-I ListParagraph ONE");
}

#[test]
fn multipara_parties_keeps_line240_if_present() {
    // Guard: M262n must not strip multipara pure-I PARTIES (no ListParagraph).
    let Some(a) = load("behavior__sd_2672_multipara_cell_4d1f068e.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__missing_separator_41c823b9.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    // If PARTIES pure-I carries line=240, it must still be present after M262n.
    let parties = doc
        .split("</w:p>")
        .find(|c| c.contains(">PARTIES<") || c.contains(">PARTIES</"));
    let Some(chunk) = parties else {
        eprintln!("no PARTIES para");
        return;
    };
    // Pre-M262 multipara tip had line=240 on PARTIES — require it still if
    // the body is pure-I (no del).
    let body = if let Some(idx) = chunk.rfind("</w:pPr>") {
        &chunk[idx + "</w:pPr>".len()..]
    } else {
        chunk
    };
    if body.contains("<w:del") {
        return;
    }
    assert!(
        chunk.contains("w:line=\"240\"") || !chunk.contains("w:spacing"),
        "PARTIES pure-I must not lose single-line spacing to ListParagraph gate; chunk={chunk}"
    );
}
