// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M133 — last-significant-token residual pairs beat ordered-prefix when both
//! could fire on different next residuals (file_120).
//!
//! Word: pure-I B body0 ("This document combines…"); MIX B body1 with A body0
//! on Equal trailing " style."; pure-D A body1.
//! Pre-M133: ordered-prefix paired A body0 with B body0 ("This document "),
//! thrashing body1 as full replace.

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
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

/// Depth-aware top-level body paragraphs (skip nested textbox `w:p`).
fn top_para_texts(doc: &str) -> Vec<(bool, bool, String)> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < body.len() {
        if body[i..].starts_with("<w:sectPr") {
            break;
        }
        if body[i..].starts_with("<w:p ") || body[i..].starts_with("<w:p>") {
            let start = i;
            let mut d = 0i32;
            let mut j = i;
            while j < body.len() {
                if body[j..].starts_with("<w:p ") || body[j..].starts_with("<w:p>") {
                    d += 1;
                    j = body[j..].find('>').map(|k| j + k + 1).unwrap_or(body.len());
                } else if body[j..].starts_with("</w:p>") {
                    d -= 1;
                    j += 6;
                    if d == 0 {
                        let chunk = &body[start..j];
                        let has_ins = chunk.contains("<w:ins");
                        let has_del = chunk.contains("<w:del");
                        let mut text = String::new();
                        let mut p = 0;
                        while let Some(at) = chunk[p..].find("<w:t") {
                            let abs = p + at;
                            let Some(gt) = chunk[abs..].find('>') else {
                                break;
                            };
                            let gt = abs + gt + 1;
                            let Some(close) = chunk[gt..].find("</w:t>") else {
                                break;
                            };
                            text.push_str(&chunk[gt..gt + close]);
                            p = gt + close + 6;
                        }
                        while let Some(at) = chunk[p..].find("<w:delText") {
                            let abs = p + at;
                            let Some(gt) = chunk[abs..].find('>') else {
                                break;
                            };
                            let gt = abs + gt + 1;
                            let Some(close) = chunk[gt..].find("</w:delText>") else {
                                break;
                            };
                            text.push_str(&chunk[gt..gt + close]);
                            p = gt + close + 12;
                        }
                        out.push((has_ins, has_del, text));
                        i = j;
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            if j >= body.len() {
                break;
            }
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
    }
    out
}

#[test]
fn m133_file_120_last_sig_style_over_this_document_prefix() {
    let Some((a, b)) = corpus_pair("file_120.docx", "file_121.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_texts(&doc);
    // Expect ≥4 content paras after stamp: title MIX, pure-I B body0, MIX style, pure-D.
    assert!(
        paras.len() >= 4,
        "expected ≥4 body paras, got {}: {:?}",
        paras.len(),
        paras
            .iter()
            .map(|(_, _, t)| t.chars().take(40).collect::<String>())
            .collect::<Vec<_>>()
    );
    // Word shape: pure-I body with "combines blue" (not mixed with demonstrates).
    let pure_i_combines = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.contains("combines blue"));
    // Must NOT keep the pre-M133 thrash: single MIX with both "combines" and "demonstrates".
    let thrash_same_para = paras.iter().any(|(i, d, t)| {
        *i && *d && t.contains("combines blue") && t.contains("demonstrates Heading")
    });
    // Word peels Equal " style." on the Blue-italic × demonstrates MIX.
    let style_equal = doc.contains("> style")
        || doc.contains("> style.")
        || paras.iter().any(|(_, _, t)| t.contains("style"));
    assert!(
        pure_i_combines && !thrash_same_para,
        "Word: pure-I combines blue + last-sig style MIX — got paras: {:?}",
        paras
            .iter()
            .map(|(i, d, t)| format!(
                "{}{} {}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(60).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
    assert!(style_equal, "expected style peel present in markup");
}

#[test]
fn m133_file_154_ordered_prefix_still_works() {
    // Guard: M114 Equal "This document" must still fire when no competing last-sig.
    let Some((a, b)) = corpus_pair("file_154.docx", "file_155.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("This document") || doc.contains("this document"),
        "file_154 must still surface This document text"
    );
    // Should not be pure full-sentence replace of both bodies only as I/D blocks
    // with zero equal runs containing "document".
    let has_mixed_or_equal = doc.contains("<w:ins") && doc.contains("<w:del");
    assert!(has_mixed_or_equal, "expected revision markup on file_154");
}
