//! M135 — off-diagonal body residual pairs (shared sig, short demos) beat
//! diagonal ordered-prefix "This document demonstrates…" thrash (file_180).
//!
//! Word: pure-I B body0 (blue font color); MIX A body0 × B body1 on "text";
//! pure-D A body1. Pre-M135: single MIX thrashing size/color on body0.

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

fn top_para_kinds(doc: &str) -> Vec<(bool, bool, String)> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut out = Vec::new();
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
        while i < body.len() && !body[i..].starts_with('<') {
            i += 1;
        }
    }
    out
}

#[test]
fn m135_file_180_pure_i_blue_body_not_size_thrash() {
    let Some((a, b)) = corpus_pair("file_180.docx", "file_181.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_kinds(&doc);
    // Word: pure-I body with "blue" / "color" without "18 point" thrash.
    let pure_i_blue = paras.iter().any(|(i, d, t)| {
        *i && !*d
            && (t.contains("blue") || t.contains("color"))
            && !t.contains("18 point")
            && !t.contains("size 18")
    });
    // Must NOT thrash body0 as MIX with both blue color and size 18 point.
    let thrash = paras.iter().any(|(i, d, t)| {
        *i && *d
            && (t.contains("blue") || t.contains("color"))
            && (t.contains("18 point") || t.contains("size 18") || t.contains("18"))
            && t.contains("This document demonstrates")
    });
    assert!(
        pure_i_blue && !thrash,
        "Word: pure-I blue body0; off-diag text MIX — got: {:?}",
        paras
            .iter()
            .map(|(i, d, t)| format!(
                "{}{} {}",
                if *i { "I" } else { "" },
                if *d { "D" } else { "" },
                t.chars().take(70).collect::<String>()
            ))
            .collect::<Vec<_>>()
    );
}

#[test]
fn m135_file_120_still_pure_i_combines() {
    // M133 guard still holds under M135 tier sort.
    let Some((a, b)) = corpus_pair("file_120.docx", "file_121.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_kinds(&doc);
    let pure_i = paras
        .iter()
        .any(|(i, d, t)| *i && !*d && t.contains("combines blue"));
    let thrash = paras.iter().any(|(i, d, t)| {
        *i && *d && t.contains("combines blue") && t.contains("demonstrates Heading")
    });
    assert!(
        pure_i && !thrash,
        "file_120 M133 shape must hold: {:?}",
        paras
    );
}

#[test]
fn m135_file_140_keeps_diagonal_not_mid_font_off_diag() {
    // Shared mid-body "font" without trailing-token hit must not off-diag (was 95→65).
    let Some((a, b)) = corpus_pair("file_140.docx", "file_141.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_kinds(&doc);
    let pure_i_verdana_body = paras.iter().any(|(i, d, t)| {
        *i && !*d && t.contains("Verdana font family") && !t.contains("demonstrates")
    });
    assert!(
        !pure_i_verdana_body,
        "file_140 must keep body0 diagonal nest (Word), not pure-I Verdana body: {:?}",
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
}

#[test]
fn m135_file_93_keeps_diagonal_not_off_diag_this() {
    // Sole shared boiler "this" must not off-diag skip M123 (was 91→64).
    let Some((a, b)) = corpus_pair("file_93.docx", "file_94.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = top_para_kinds(&doc);
    // Word nests body0×body0 with Heading/style peels — not pure-I whole next body0.
    let pure_i_heading3_body = paras.iter().any(|(i, d, t)| {
        *i && !*d && t.contains("Heading 3 with center") && !t.contains("Suggesting")
    });
    assert!(
        !pure_i_heading3_body,
        "file_93 must not pure-I next body0 (M123 diagonal thrash): {:?}",
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
}

#[test]
fn m135_file_154_ordered_prefix_still_pairs() {
    let Some((a, b)) = corpus_pair("file_154.docx", "file_155.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("This document") || doc.contains("this document"),
        "file_154 must keep This document pairing"
    );
}
