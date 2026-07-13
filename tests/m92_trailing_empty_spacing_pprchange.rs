//! M92 — trailing empty body para: live spacing → pPrChange (file_30).

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

/// Last body paragraph fragment (before sectPr), roughly.
fn last_body_p(doc: &str) -> &str {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    // Take last </w:p> chunk that is not inside sectPr-only noise.
    body.rsplit("</w:p>").nth(1).unwrap_or("")
}

#[test]
fn m92_file_30_trailing_empty_spacing_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_30.docx", "file_31.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let last = last_body_p(&doc);
    // Trailing empty: no t/delText in last para; check last p chunk.
    let chunks: Vec<&str> = doc.split("</w:p>").collect();
    // Walk from end for a chunk with no delText and no w:t content text.
    let mut found_empty = false;
    for chunk in chunks.iter().rev().take(4) {
        let has_text = chunk.contains("<w:t") || chunk.contains("delText");
        if has_text {
            continue;
        }
        if !chunk.contains("<w:p") && !chunk.contains("pPr") {
            continue;
        }
        found_empty = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            *chunk
        };
        assert!(
            !live.contains("<w:spacing") && !live.contains("w:line="),
            "trailing empty must not keep live spacing: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("spacing"),
            "trailing empty spacing must sit under pPrChange: {chunk}"
        );
        break;
    }
    assert!(found_empty, "expected trailing empty paragraph: {last}");
}

#[test]
fn m92_file_23_last_del_still_spacing_pprchange() {
    // Guard M83b under M92.
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
