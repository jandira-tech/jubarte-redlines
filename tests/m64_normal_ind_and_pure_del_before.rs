// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M64 — (1) keep Word-kept pure-del `before=800` (file_196; M61 reverted);
//! (2) copy B Normal `w:ind` when merging spacing (file_196 firstLine=432).

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

fn part_xml(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m64_file_196_keeps_pure_del_before_800() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = part_xml(&out, "word/document.xml");
    // Word keeps pure-del before=800 ("Themes and styles…"); M61 wrongly cleared it.
    assert!(
        xml.contains("w:before=\"800\""),
        "file_196 pure-del must keep before=800 (Word parity)"
    );
}

#[test]
fn m64_file_196_normal_copies_b_firstline_ind() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let styles = part_xml(&out, "word/styles.xml");
    // B Normal has firstLine=432; Word keeps it on merged Normal with after=0/line=276.
    assert!(
        styles.contains("w:firstLine=\"432\"") || styles.contains("w:firstLine=\"432\""),
        "file_196 Normal must copy B firstLine=432: {}",
        &styles[styles.find("styleId=\"Normal\"").unwrap_or(0)..]
            .chars()
            .take(400)
            .collect::<String>()
    );
    // M60b after=0 when line present still applies.
    let normal = {
        let start = styles.find("styleId=\"Normal\"").expect("Normal style");
        let chunk = &styles[start..];
        let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(800));
        chunk[..end].to_string()
    };
    assert!(
        normal.contains("w:after=\"0\"") || normal.contains("w:after=\"0\""),
        "file_196 Normal should materialize after=0 with line: {normal}"
    );
}

#[test]
fn m64_file_14_still_keeps_pure_del_before_300() {
    let Some((a, b)) = corpus_pair("file_14.docx", "file_15.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = part_xml(&out, "word/document.xml");
    assert!(
        xml.contains("w:before=\"300\""),
        "file_14 pure-del before=300 must not be stripped"
    );
}
