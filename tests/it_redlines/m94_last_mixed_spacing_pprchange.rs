// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M94 — last mixed I+D moves live spacing into pPrChange (file_139).

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
fn m94_file_139_last_mixed_spacing_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_139.docx", "file_140.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Last residual mixed: Size 12… + Heading 3 hierarchy
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("standard readable font size")
            && !chunk.contains("third level of document hierarchy")
        {
            continue;
        }
        // Prefer the chunk that has both (mixed)
        if !(chunk.contains("standard readable") || chunk.contains("third level")) {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            !live.contains("<w:spacing") && !live.contains("w:line="),
            "last mixed must not keep live spacing: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("spacing"),
            "last mixed spacing must sit under pPrChange: {chunk}"
        );
        break;
    }
    assert!(
        found,
        "expected last mixed residual about font size / heading"
    );
}

#[test]
fn m94_file_23_pure_del_still_spacing_pprchange() {
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Document Title") {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(!live.contains("w:line=\"240\""));
        assert!(chunk.contains("pPrChange") && chunk.contains("w:line=\"240\""));
    }
    assert!(found);
}

#[test]
fn m94_file_59_pstyle_still_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_59.docx", "file_60.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Omega") && !chunk.contains("Ω") {
            continue;
        }
        found = true;
        assert!(chunk.contains("pPrChange"));
        break;
    }
    assert!(found);
}
