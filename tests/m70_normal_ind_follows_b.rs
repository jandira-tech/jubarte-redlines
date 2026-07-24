// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M70 — when rewriting Normal spacing, drop A's `w:ind` if B has none
//! (file_197); keep copying B's ind when present (file_196 firstLine=432).

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

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_chunk(styles: &str) -> String {
    let start = styles.find("styleId=\"Normal\"").expect("Normal");
    let chunk = &styles[start..];
    let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(600));
    chunk[..end].to_string()
}

#[test]
fn m70_file_197_drops_a_firstline_when_b_bare() {
    let Some((a, b)) = corpus_pair("file_197.docx", "file_198.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_chunk(&styles_xml(&out));
    // Outer live pPr (before pPrChange) must not keep A firstLine; old value may.
    let live = normal.split("pPrChange").next().unwrap_or(&normal);
    assert!(
        !live.contains("firstLine"),
        "file_197 live Normal must not keep A firstLine: {live}"
    );
}

#[test]
fn m70_file_196_still_keeps_b_firstline() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_chunk(&styles_xml(&out));
    assert!(
        normal.contains("firstLine=\"432\"") || normal.contains("w:firstLine=\"432\""),
        "file_196 must still copy B firstLine=432: {normal}"
    );
}
