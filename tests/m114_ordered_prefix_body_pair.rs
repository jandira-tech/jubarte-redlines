//! M114 — residual body cousins that share ordered significant prefix
//! ("This document …") pair for nested word-LCS (file_154).
//! Word: Equal "This document " + ins/del tails.
//! Was: pure-I whole B body + pure-D whole A body (jaccard ~0.13 failed gate).

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

fn para_segments(doc: &str, idx: usize) -> Vec<(char, String)> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut paras = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let ok =
            after.starts_with("<w:p>") || after.starts_with("<w:p ") || after.starts_with("<w:p\n");
        if !ok {
            rest = &after[4..];
            continue;
        }
        let Some(end) = after.find("</w:p>") else {
            break;
        };
        let chunk = &after[..end + 6];
        rest = &after[end + 6..];
        let mut c = chunk.to_string();
        if let Some(i) = c.find("<w:pPr") {
            if let Some(j) = c[i..].find("</w:pPr>") {
                c = format!("{}{}", &c[..i], &c[i + j + 8..]);
            }
        }
        let mut segs = Vec::new();
        let mut i = 0usize;
        let bytes = c.as_bytes();
        while i < c.len() {
            if c[i..].starts_with("<w:ins") {
                let end = c[i..]
                    .find("</w:ins>")
                    .map(|e| i + e + 8)
                    .unwrap_or(c.len());
                let frag = &c[i..end];
                let mut t = String::new();
                let mut p = 0;
                while let Some(at) = frag[p..].find("<w:t") {
                    let abs = p + at;
                    let gt = frag[abs..].find('>').unwrap() + abs + 1;
                    let close = frag[gt..].find("</w:t>").unwrap() + gt;
                    t.push_str(&frag[gt..close]);
                    p = close + 6;
                }
                segs.push(('I', t));
                i = end;
            } else if c[i..].starts_with("<w:del") {
                let end = c[i..]
                    .find("</w:del>")
                    .map(|e| i + e + 8)
                    .unwrap_or(c.len());
                let frag = &c[i..end];
                let mut t = String::new();
                let mut p = 0;
                while let Some(at) = frag[p..].find("<w:delText") {
                    let abs = p + at;
                    let gt = frag[abs..].find('>').unwrap() + abs + 1;
                    let close = frag[gt..].find("</w:delText>").unwrap() + gt;
                    t.push_str(&frag[gt..close]);
                    p = close + 12;
                }
                segs.push(('D', t));
                i = end;
            } else if c[i..].starts_with("<w:r") {
                let end = c[i..].find("</w:r>").map(|e| i + e + 6).unwrap_or(c.len());
                let frag = &c[i..end];
                let mut t = String::new();
                let mut p = 0;
                while let Some(at) = frag[p..].find("<w:t") {
                    let abs = p + at;
                    let gt = frag[abs..].find('>').unwrap() + abs + 1;
                    let close = frag[gt..].find("</w:t>").unwrap() + gt;
                    t.push_str(&frag[gt..close]);
                    p = close + 6;
                }
                if !t.is_empty() {
                    segs.push(('E', t));
                }
                i = end;
            } else {
                i += 1;
                while i < bytes.len() && bytes[i] != b'<' {
                    i += 1;
                }
            }
        }
        paras.push(segs);
    }
    paras.into_iter().nth(idx).unwrap_or_default()
}

#[test]
fn m114_file_154_equal_this_document_prefix() {
    let Some((a, b)) = corpus_pair("file_154.docx", "file_155.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let segs = para_segments(&doc, 2);
    let has_equal_prefix = segs
        .iter()
        .any(|(k, t)| *k == 'E' && t.to_ascii_lowercase().contains("document"));
    let pure_replace = segs
        .iter()
        .any(|(k, t)| *k == 'I' && t.contains("combines justify"))
        && segs.iter().any(|(k, t)| {
            *k == 'D' && t.contains("demonstrates italic") && t.contains("This document")
        });
    assert!(
        has_equal_prefix && !pure_replace,
        "expected Equal 'This document' prefix (Word), not full-sentence replace: {segs:?}"
    );
}

#[test]
fn m114_file_160_guard_no_overpair() {
    // Connector title pairing must still work; body modules must not explode.
    let Some((a, b)) = corpus_pair("file_160.docx", "file_161.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Italic") || doc.contains("Module") || doc.contains("delText"),
        "file_160 still compares"
    );
}
