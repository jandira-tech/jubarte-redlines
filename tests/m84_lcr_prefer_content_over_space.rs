//! M84 — LCR prefers content words over pure-space when lengths tie (file_81).
//! Word: …Title [paragraph ]style with center alignment.
//! Was:  …Title style with center alignment[paragraph style].

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

/// Ordered (kind, text) for body paragraph `idx` (I/D/E).
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
        // strip pPr
        let mut c = chunk.to_string();
        if let Some(i) = c.find("<w:pPr")
            && let Some(j) = c[i..].find("</w:pPr>")
        {
            c = format!("{}{}", &c[..i], &c[i + j + 8..]);
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
fn m84_file_81_keeps_equal_style_before_period() {
    let Some((a, b)) = corpus_pair("file_81.docx", "file_82.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let segs = para_segments(&doc, 2);
    // Word shape: … Title | D "paragraph " | E "style" | I " with center alignment" | E "."
    let has_equal_style = segs.iter().any(|(k, t)| *k == 'E' && t.trim() == "style");
    let bad_replace = segs
        .iter()
        .any(|(k, t)| *k == 'I' && t.contains("style with center"))
        && segs
            .iter()
            .any(|(k, t)| *k == 'D' && t.contains("paragraph style"));
    assert!(
        has_equal_style && !bad_replace,
        "expected Equal 'style' (Word), not replace of whole 'paragraph style': {segs:?}"
    );
}
