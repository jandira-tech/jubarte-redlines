// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M88 — mixed I+D fold keeps Deleted structural numPr (file_55 "notation"+"a").

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m88_file_55_mixed_notation_keeps_numpr() {
    let Some((a, b)) = corpus_pair("file_55.docx", "file_56.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Bold superscript is used in mathematical") {
            continue;
        }
        if !chunk.contains("delText") || !chunk.contains(">a<") && !chunk.contains(">a</") {
            // delText a
            if !chunk.contains("<w:delText>a</w:delText>") {
                continue;
            }
        }
        found = true;
        assert!(
            chunk.contains("<w:numPr") || chunk.contains("numId"),
            "mixed notation+a must keep Deleted numPr: {chunk}"
        );
        break;
    }
    assert!(found, "expected mixed para with notation + del a");
}

#[test]
fn m88_file_23_guard() {
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Title Style Demo") || doc.contains("delText"));
}

#[test]
fn m88_file_33_unrelated_not_forced_numpr() {
    // Guard: file_33 must still open and not invent mixed numPr on Summary.
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    for chunk in doc.split("</w:p>") {
        if (chunk.contains(">Summary<") || chunk.contains(">Summary</"))
            && chunk.contains("Heading")
            && chunk.contains("delText")
            && chunk.contains("numPr")
        {
            // mixed Summary+Heading with numPr would be wrong
            panic!("Summary must not fold Heading with numPr: {chunk}");
        }
    }
}
