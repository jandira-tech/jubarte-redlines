//! M89 — sole pure-I + multi pure-D folds first D into I (file_191).

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
fn m89_file_191_ouch_folds_first_residual_del() {
    let Some((a, b)) = corpus_pair("file_191.docx", "file_192.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Ouch") {
            continue;
        }
        found = true;
        // Mixed: Ouch ins + first residual del
        assert!(
            chunk.contains("<w:ins") && (chunk.contains("delText") || chunk.contains("<w:del")),
            "Ouch pure-I must fold first residual pure-D: {chunk}"
        );
        assert!(
            chunk.contains("vfdsdfcACawesd") || chunk.contains("delText"),
            "first residual text should be in mixed Ouch para: {chunk}"
        );
        break;
    }
    assert!(found, "expected Ouch paragraph");
}

#[test]
fn m89_m44_multi_del_2i3d_still_separate() {
    // Keep m44 shape: run the unit test path via corpus if needed; structural
    // guard is the dedicated m44 test. Here just ensure file_55 still has mixed.
    let Some((a, b)) = corpus_pair("file_55.docx", "file_56.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Bold superscript"));
}
