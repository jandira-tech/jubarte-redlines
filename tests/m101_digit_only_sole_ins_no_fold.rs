// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M101 — sole pure-I that is digits-only ("24") + multi pure-D stays separate
//! (file_166). Content sole pure-I ("Ouch.") still folds (file_191).

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

fn para_visible(chunk: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < chunk.len() {
        let rest = &chunk[i..];
        if (rest.starts_with("<w:t") || rest.starts_with("<w:delText"))
            && let Some(gt) = rest.find('>')
        {
            let end_tag = if rest.starts_with("<w:t") {
                "</w:t>"
            } else {
                "</w:delText>"
            };
            if let Some(end) = rest[gt + 1..].find(end_tag) {
                out.push_str(&rest[gt + 1..gt + 1 + end]);
                i += gt + 1 + end + end_tag.len();
                continue;
            }
        }
        i += 1;
    }
    out
}

#[test]
fn m101_file_166_twenty_four_stays_pure_ins() {
    let Some((a, b)) = corpus_pair("file_166.docx", "file_167.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word: pure-I "24" then pure-D "Document 100 - Ultimate Demo" — not MIX.
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.trim() == "24"
            || (vis.contains("24") && !vis.contains("Document") && !vis.contains("100"))
        {
            found = true;
            assert!(
                chunk.contains("<w:ins") && !chunk.contains("delText"),
                "digits-only pure-I must not fold with catalog del: {vis:?}"
            );
            break;
        }
        if vis.contains("24") && vis.contains("Document") {
            panic!("24 must not mix with Document title: {vis:?}");
        }
    }
    assert!(found, "expected pure-I 24 para");
    assert!(
        doc.contains("Document 100") || doc.contains("Ultimate Demo") || doc.contains("delText"),
        "catalog residual still deleted"
    );
}

#[test]
fn m101_file_191_ouch_still_folds() {
    let Some((a, b)) = corpus_pair("file_191.docx", "file_192.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Ouch. should still fold with first residual del (not digit-only).
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("Ouch") {
            found = true;
            assert!(
                chunk.contains("delText"),
                "Ouch must still fold first residual del: {vis:?}"
            );
            break;
        }
    }
    assert!(found, "expected Ouch residual");
}

#[test]
fn m101_file_167_subsection_still_folds_24_del() {
    // Guard M98: multi-I + sole pure-D "24" still folds into Subsection Title.
    let Some((a, b)) = corpus_pair("file_167.docx", "file_168.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    for chunk in doc.split("</w:p>") {
        if chunk.contains("Subsection Title") {
            assert!(
                chunk.contains("delText") && para_visible(chunk).contains("24"),
                "Subsection still folds del 24"
            );
            return;
        }
    }
    panic!("expected Subsection Title residual");
}
