// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M65 — both Normals bare (no stored rPr): do not promote B docDefaults
//! fonts onto Normal (file_170 Word leaves bare Normal).

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
fn m65_file_170_leaves_bare_normal_no_rpr_promote() {
    let Some((a, b)) = corpus_pair("file_170.docx", "file_171.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_chunk(&styles_xml(&out));
    assert!(
        !normal.contains("<w:rPr"),
        "file_170 both-bare Normal must not get B dd rPr: {normal}"
    );
    assert!(
        !normal.contains("rPrChange"),
        "file_170 bare Normal must not synthesize rPrChange: {normal}"
    );
}

#[test]
fn m65_file_196_still_merges_structured_normal_rpr() {
    // B Normal has stored rPr (Ubuntu); merge path must still run for structured.
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_chunk(&styles_xml(&out));
    assert!(
        normal.contains("Ubuntu") || normal.contains("w:rPr"),
        "file_196 structured B Normal rPr must still land: {normal}"
    );
}
