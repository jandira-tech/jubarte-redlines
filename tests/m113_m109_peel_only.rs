//! M113 — M109 reverse short-into-long only when peel token relatedness fires.
//! file_59: long greek alphabet base × short Font Size 24 next — zero shared
//! vocab. Ungated M109 nested "Αα Alpha" into the short body; Word pure-I's
//! the whole short next, pure-D's the greek list, then boundary-folds Alpha
//! into the last insert.

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
fn m113_file_59_pure_i_short_body_not_mixed_with_alpha() {
    let Some((a, b)) = corpus_pair("file_59.docx", "file_60.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word p2: pure-I "This document demonstrates font size 24."
    let p2 = para_segments(&doc, 2);
    let p2_text: String = p2.iter().map(|(_, t)| t.as_str()).collect();
    assert!(
        p2_text.contains("font size 24") || p2_text.contains("demonstrates"),
        "p2 should be short body, got {p2:?}"
    );
    let p2_has_alpha = p2
        .iter()
        .any(|(_, t)| t.contains("Alpha") || t.contains('Α'));
    assert!(
        !p2_has_alpha,
        "Word keeps short body pure-I; Alpha must not nest into p2: {p2:?}"
    );
    // Word p3: MIX last short body + del Alpha
    let p3 = para_segments(&doc, 3);
    let p3_has_larger = p3
        .iter()
        .any(|(_, t)| t.contains("Larger") || t.contains("presentations"));
    let p3_has_alpha = p3
        .iter()
        .any(|(k, t)| *k == 'D' && (t.contains("Alpha") || t.contains('Α')));
    assert!(
        p3_has_larger && p3_has_alpha,
        "Word folds Alpha into last short insert: {p3:?}"
    );
}

#[test]
fn m113_file_131_peel_still_fires() {
    let Some((a, b)) = corpus_pair("file_131.docx", "file_132.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Justify Alignment Demo"),
        "short title must remain"
    );
    // Justify title early (M109 peel path)
    let idx = doc.find("Justify Alignment Demo").expect("title");
    let before = &doc[..idx];
    let para_opens = before.matches("<w:p").count();
    assert!(
        para_opens < 8,
        "Justify title should appear early under peel gate, para_opens={para_opens}"
    );
}
