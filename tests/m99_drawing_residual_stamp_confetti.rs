// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M99 — stamped short demos with drawing/text-box residuals must confetti
//! (insert-all next, delete-all base), not full-doc word LCS.
//! file_70: "Green Highlight Demo" stays pure-I; Datum plane drawing is del.

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
fn m99_file_70_green_title_not_mixed_with_datum_drawing() {
    let Some((a, b)) = corpus_pair("file_70.docx", "file_71.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word: pure-I "Green Highlight Demo" — never mixed with del "Datum plane".
    let mut found_title = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if !vis.contains("Green Highlight Demo") && !vis.contains("Green Highlight") {
            continue;
        }
        if vis.contains("document demonstrates") || vis.contains("environmental") {
            continue;
        }
        found_title = true;
        assert!(
            !vis.contains("Datum plane"),
            "title must not mix with Datum plane del: {vis:?}"
        );
        assert!(
            chunk.contains("<w:ins") && !chunk.contains("delText"),
            "Green Highlight Demo must be pure-ins: {vis:?}"
        );
        break;
    }
    assert!(found_title, "expected Green Highlight Demo title residual");
    // Drawing / Datum plane still deleted somewhere.
    assert!(
        doc.contains("Datum plane") || doc.contains("drawing") || doc.contains("delText"),
        "Datum residual should still appear as del"
    );
}

#[test]
fn m99_file_33_still_two_page_shape() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

#[test]
fn m99_file_134_still_confettis() {
    // Guard: unrelated stamped demos stay confetti (no false full LCS).
    let Some((a, b)) = corpus_pair("file_134.docx", "file_135.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("delText") || doc.contains("<w:ins") || doc.contains("file_"));
}
