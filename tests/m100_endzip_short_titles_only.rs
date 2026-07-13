//! M100 — stamp residual end-zip only for short title leftovers (≤4 tokens).
//! file_32: after Demo-title pair, do not end-zip "Main Title Section" with
//! "This text is both bold and underlined." — Word folds Main Title with
//! "Demonstrating bold and underline combined." instead.

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

fn para_visible(chunk: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < chunk.len() {
        let rest = &chunk[i..];
        if rest.starts_with("<w:t") || rest.starts_with("<w:delText") {
            if let Some(gt) = rest.find('>') {
                let end_tag = if rest.starts_with("<w:t") {
                    "</w:t>"
                } else {
                    "</w:delText>"
                };
                if let Some(end) = rest[gt + 1..].find(end_tag) {
                    out.push_str(&rest[gt + 1..gt + 1 + end]);
                    i += gt + 1 + end + end_tag.len();
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

#[test]
fn m100_file_32_main_title_folds_with_demonstrating() {
    let Some((a, b)) = corpus_pair("file_32.docx", "file_33.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word: MIX "Main Title Section" + del "Demonstrating bold and underline combined."
    // Not MIX Main Title + del "This text is both bold and underlined."
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if !vis.contains("Main Title Section") {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("delText") && vis.contains("Demonstrating"),
            "Main Title must fold with Demonstrating… not the last A sentence: {vis:?}"
        );
        assert!(
            !vis.contains("This text is both bold"),
            "Main Title must not nest with last A residual: {vis:?}"
        );
        break;
    }
    assert!(found, "expected Main Title Section residual");
    // Last A residual pure-del.
    let mut found_last = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("This text is both bold") {
            found_last = true;
            assert!(
                chunk.contains("delText") && !vis.contains("Main Title"),
                "last A residual should be pure-del: {vis:?}"
            );
        }
    }
    assert!(found_last, "expected pure-del of last A residual");
}

#[test]
fn m100_file_32_title_still_nests_demo() {
    let Some((a, b)) = corpus_pair("file_32.docx", "file_33.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // M96: Equal Demo on title residual.
    assert!(
        doc.contains("> Demo<") || doc.contains("> Demo <") || doc.contains(">Demo<"),
        "title must still nest Equal Demo"
    );
}

#[test]
fn m100_file_85_first_bullet_still_pure_ins() {
    let Some((a, b)) = corpus_pair("file_85.docx", "file_86.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Guard M82: first bullet not mixed with "This text is bold."
    for chunk in doc.split("</w:p>") {
        if chunk.contains("First bold bullet") {
            assert!(
                !chunk.contains("This text is"),
                "First bullet must not carry short bold del"
            );
            break;
        }
    }
}

#[test]
fn m100_file_33_main_title_survives() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Main Title") || doc.contains("Main Title Section"));
}
