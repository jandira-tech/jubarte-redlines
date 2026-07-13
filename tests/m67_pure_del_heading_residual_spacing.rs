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
    // Scope to LIVE formatting only: before=400 recorded inside
    // `w:pPrChange` is deliberate M78/M81 history (Word keeps the Inserted
    // pPr live and records A's original spacing in pPrChange; re-promoting
    // it re-bloats file_33). Only live pPr spacing must not survive.
    let live = strip_ppr_change(&xml);
    assert!(
        !live.contains("w:before=\"400\""),
        "file_33 pure-del must not keep heading residual before=400 as LIVE spacing"
    );
}

/// Remove every `w:pPrChange` span (self-closing or paired) so assertions
/// see only live formatting, not recorded history.
fn strip_ppr_change(xml: &str) -> String {
    let mut s = xml.to_string();
    while let Some(i) = s.find("<w:pPrChange") {
        let gt = match s[i..].find('>') {
            Some(j) => i + j,
            None => break,
        };
        let end = if s[..gt].ends_with('/') {
            gt + 1
        } else {
            match s[gt..].find("</w:pPrChange>") {
                Some(j) => gt + j + "</w:pPrChange>".len(),
                None => break,
            }
        };
        s.replace_range(i..end, "");
    }
    s
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
