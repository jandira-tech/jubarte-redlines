// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M111 — cascade Normal pPrChange/rPrChange onto basedOn=Normal styles
//! (ListParagraph, BodyText, …) like Word file_130.

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

fn style_blob(styles: &str, sid: &str) -> Option<String> {
    let needle = format!("w:styleId=\"{sid}\"");
    let start = styles.find(&needle)?;
    let from = styles[..start].rfind("<w:style")?;
    let end = styles[start..].find("</w:style>")? + start + "</w:style>".len();
    Some(styles[from..end].to_string())
}

#[test]
fn m111_file_130_listparagraph_has_pprchange() {
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);
    let lp = style_blob(&styles, "ListParagraph").expect("ListParagraph");
    assert!(
        lp.contains("pPrChange"),
        "ListParagraph must cascade Normal pPrChange: {lp}"
    );
    assert!(
        lp.contains("rPrChange"),
        "ListParagraph must cascade Normal rPrChange: {lp}"
    );
    // Word keeps live ind/contextualSpacing
    assert!(
        lp.contains("ind") || lp.contains("contextualSpacing"),
        "live ListParagraph props preserved: {lp}"
    );
}

#[test]
fn m111_file_130_bodytext_has_change_if_present() {
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);
    if let Some(bt) = style_blob(&styles, "BodyText") {
        assert!(
            bt.contains("pPrChange") || bt.contains("rPrChange"),
            "BodyText should cascade Normal change: {bt}"
        );
    }
}

#[test]
fn m111_file_8_guard_no_flood() {
    // file_8 same-structured Normals should not flood styles with pPrChange.
    let Some((a, b)) = corpus_pair("file_8.docx", "file_9.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);
    let n = style_blob(&styles, "Normal").expect("Normal");
    // file_8 typically has no Normal pPrChange — cascade should no-op.
    if !n.contains("pPrChange") {
        let lp = style_blob(&styles, "ListParagraph").unwrap_or_default();
        assert!(
            !lp.contains("pPrChange"),
            "file_8 ListParagraph must not get cascade without Normal change: {lp}"
        );
    }
}

#[test]
fn m111_file_33_still_ok() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut doc = String::new();
    f.read_to_string(&mut doc).unwrap();
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}
