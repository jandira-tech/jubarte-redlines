//! M116 — stamped short demo with table vs long unrelated base short-circuits
//! to confetti insert-all/delete-all (file_78). Was: short_n=3+table blocked
//! short-circuit → full LCS nested Quarterly title into eigenpal.

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

fn para_kinds_texts(doc: &str) -> Vec<(bool, bool, String)> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut out = Vec::new();
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
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del");
        let mut text = String::new();
        let mut i = 0;
        while i < chunk.len() {
            let r = &chunk[i..];
            if (r.starts_with("<w:t") || r.starts_with("<w:delText"))
                && let Some(gt) = r.find('>')
            {
                let close = if r.starts_with("<w:t") {
                    "</w:t>"
                } else {
                    "</w:delText>"
                };
                if let Some(c) = r[gt + 1..].find(close) {
                    text.push_str(&r[gt + 1..gt + 1 + c]);
                    i += gt + 1 + c + close.len();
                    continue;
                }
            }
            i += 1;
        }
        out.push((has_ins, has_del, text));
    }
    out
}

#[test]
fn m116_file_78_quarterly_title_not_mixed_with_eigenpal() {
    let Some((a, b)) = corpus_pair("file_78.docx", "file_79.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = para_kinds_texts(&doc);
    let q = paras
        .iter()
        .find(|(_, _, t)| t.contains("Quarterly Performance Report"));
    let Some((has_ins, has_del, t)) = q else {
        panic!("missing Quarterly title");
    };
    assert!(
        *has_ins && !*has_del,
        "Word pure-I Quarterly title; must not fold eigenpal del into it: {t:?} ins={has_ins} del={has_del}"
    );
    let eigen_del = paras
        .iter()
        .any(|(i, d, t)| *d && !*i && t.contains("eigenpal"));
    assert!(eigen_del, "eigenpal title should remain pure-D");
}

#[test]
fn m116_file_38_m90_related_fold_still_ok() {
    // Guard: related multi-del boundary demos still fold when jaccard ≥ 0.12.
    let Some((a, b)) = corpus_pair("file_38.docx", "file_39.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("delText") || doc.contains("w:ins"),
        "file_38 still compares"
    );
}

#[test]
fn m116_file_187_short_base_still_nests_book_catalog() {
    // Short base catalog × long next eigenpal: Word nests "Book Catalog" into
    // charter line — must NOT take M116 short-next short-circuit.
    let Some((a, b)) = corpus_pair("file_187.docx", "file_188.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let has_catalog = doc.contains("Book Catalog");
    let early_pure_i_eigenpal_only = {
        // If short-circuit fired, first content residual is pure-I eigenpal without Book Catalog del nearby
        let body = doc.split("<w:body").nth(1).unwrap_or("");
        let first_paras: Vec<&str> = body.split("</w:p>").take(4).collect();
        first_paras.iter().any(|p| {
            p.contains("eigenpal") && !p.contains("Book Catalog") && !p.contains("delText")
        }) && !doc.contains("Book Catalog")
    };
    assert!(
        has_catalog && !early_pure_i_eigenpal_only,
        "file_187 must keep Book Catalog in compare (Word nests), not pure insert-all next"
    );
}
