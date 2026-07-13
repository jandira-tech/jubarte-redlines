//! M67 — pure-del Heading residual spacing (before≥360 + after + line, no
//! pStyle): strip whole `w:spacing`. Keep bare before=800 (file_196) and
//! before≤300 (file_14).

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
fn m67_file_33_strips_pure_del_heading_residual_spacing() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = document_xml(&out);
    // Heading residual before=400 after=120 line=240 must not survive on pure-del.
    assert!(
        !xml.contains("w:before=\"400\""),
        "file_33 pure-del must not keep heading residual before=400"
    );
}

#[test]
fn m67_file_196_keeps_pure_del_before_800() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = document_xml(&out);
    assert!(
        xml.contains("w:before=\"800\""),
        "file_196 pure-del before=800 must survive (Word keeps it)"
    );
}

#[test]
fn m67_file_14_keeps_pure_del_before_300() {
    let Some((a, b)) = corpus_pair("file_14.docx", "file_15.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = document_xml(&out);
    assert!(
        xml.contains("w:before=\"300\""),
        "file_14 pure-del before=300 must not be stripped"
    );
}
