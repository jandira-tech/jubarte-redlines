// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M83 — trailing empty pure-ins dropped; last pure-del spacing → pPrChange (file_23).

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
fn m83_file_23_no_trailing_empty_ins_after_table() {
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Body structure: after </w:tbl> should not have empty pure-ins before pure-dels.
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let after_tbl = body.split("</w:tbl>").nth(1).unwrap_or("");
    // First content after table should be a pure-del of Title Style Demo, not empty ins.
    assert!(
        after_tbl.contains("Title Style Demo") || after_tbl.contains("delText"),
        "expected deleted title after table"
    );
    // No pure-ins empty para immediately after table: look for ins-only empty p
    // between </w:tbl> and first delText of Title Style Demo.
    let before_del = after_tbl
        .split("Title Style Demo")
        .next()
        .unwrap_or(after_tbl);
    // Empty pure-ins would be pPr/rPr/ins with no t/delText between tbl and first del.
    let has_empty_ins_shell = before_del.contains("<w:ins")
        && !before_del.contains("<w:t")
        && !before_del.contains("delText");
    assert!(
        !has_empty_ins_shell,
        "trailing empty pure-ins after table must be dropped: {before_del}"
    );
}

#[test]
fn m83_file_23_last_del_spacing_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Find paragraph containing Document Title delText.
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Document Title") {
            continue;
        }
        found = true;
        // Live spacing (outside pPrChange) should not have line=240 for last pure-del.
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            !live.contains("w:line=\"240\""),
            "last pure-del must not keep live line=240: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("w:line=\"240\""),
            "spacing must sit under pPrChange: {chunk}"
        );
    }
    assert!(found, "expected Document Title pure-del");
}
