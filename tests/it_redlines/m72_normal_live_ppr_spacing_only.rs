// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M72 — after Normal spacing merge, live pPr is spacing (+ B ind only);
//! A's widowControl/tabs/suppress go in pPrChange old (file_77 Word parity).

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

fn normal_live_ppr(styles: &str) -> String {
    let start = styles.find("styleId=\"Normal\"").expect("Normal");
    let chunk = &styles[start..];
    let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(900));
    let normal = &chunk[..end];
    // Live pPr is before pPrChange.
    if let Some(i) = normal.find("<w:pPr") {
        let rest = &normal[i..];
        if let Some(chg) = rest.find("pPrChange") {
            return rest[..chg].to_string();
        }
        if let Some(e) = rest.find("</w:pPr>") {
            return rest[..=e + 7].to_string();
        }
    }
    String::new()
}

#[test]
fn m72_file_77_live_normal_without_widow_tabs() {
    let Some((a, b)) = corpus_pair("file_77.docx", "file_78.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let styles = styles_xml(&out);
    let live = normal_live_ppr(&styles);
    assert!(
        !live.contains("widowControl")
            && !live.contains("tabs")
            && !live.contains("suppressAutoHyphens"),
        "file_77 live Normal must not keep A widow/tabs: {live}"
    );
    assert!(
        live.contains("spacing") || live.contains("w:spacing"),
        "file_77 live Normal should still have spacing: {live}"
    );
    // Old value still records A's non-spacing pPr.
    assert!(
        styles.contains("widowControl") || styles.contains("w:widowControl"),
        "file_77 pPrChange old should keep widowControl"
    );
}

#[test]
fn m72_file_196_still_keeps_b_firstline_on_live() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let live = normal_live_ppr(&styles_xml(&out));
    assert!(
        live.contains("firstLine"),
        "file_196 live Normal must keep B firstLine: {live}"
    );
}
