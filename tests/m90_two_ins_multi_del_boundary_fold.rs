//! M90 — 2 pure-I + multi pure-D folds last I with first D (file_38/62/11).

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

#[test]
fn m90_file_38_table_folds_first_residual_del() {
    let Some((a, b)) = corpus_pair("file_38.docx", "file_39.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Mixed: table/ticket content ins + Center Alignment Demo del
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Center Alignment Demo") {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("<w:ins") && chunk.contains("delText"),
            "first residual del must fold into last pure-I: {chunk}"
        );
        break;
    }
    assert!(found, "expected Center Alignment Demo residual");
}

#[test]
fn m90_file_11_love_ms_folds_superscript_title() {
    let Some((a, b)) = corpus_pair("file_11.docx", "file_12.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Superscript Demo") {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("<w:ins") && (chunk.contains("delText") || chunk.contains("<w:del")),
            "Superscript Demo del must fold into last pure-I: {chunk}"
        );
        break;
    }
    assert!(found, "expected Superscript Demo residual");
}

#[test]
fn m90_file_191_still_folds() {
    let Some((a, b)) = corpus_pair("file_191.docx", "file_192.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Ouch"));
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if chunk.contains("Ouch") {
            found = true;
            assert!(chunk.contains("delText") || chunk.contains("<w:del"));
            break;
        }
    }
    assert!(found);
}
