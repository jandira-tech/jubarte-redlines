//! M106 — same docDefaults both sides, A bare Normal, B structured rPr:
//! Word emits empty live Normal pPr + pPrChange(old = dd spacing) with rPrChange.

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

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_style(styles: &str) -> Option<&str> {
    let start = styles.find("w:styleId=\"Normal\"")?;
    let from = styles[..start].rfind("<w:style")?;
    let end = styles[start..].find("</w:style>")? + start + "</w:style>".len();
    Some(&styles[from..end])
}

#[test]
fn m106_file_7_normal_has_pprchange_dd_spacing() {
    let Some((a, b)) = corpus_pair("file_7.docx", "file_8.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);
    let n = normal_style(&styles).expect("Normal style");
    assert!(
        n.contains("pPrChange"),
        "Normal must have pPrChange (Word same-dd clear): {n}"
    );
    assert!(
        n.contains("after=\"200\"") || n.contains("w:after=\"200\""),
        "pPrChange old should carry dd after=200: {n}"
    );
    assert!(
        n.contains("line=\"276\"") || n.contains("w:line=\"276\""),
        "pPrChange old should carry dd line=276: {n}"
    );
    assert!(
        n.contains("rPrChange"),
        "Normal should still have rPrChange for Aptos: {n}"
    );
}

#[test]
fn m106_file_8_still_no_normal_pprchange() {
    // Both large-doc, same structured path must not flood Normal pPrChange.
    let Some((a, b)) = corpus_pair("file_8.docx", "file_9.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);
    let n = normal_style(&styles).expect("Normal style");
    assert!(
        !n.contains("pPrChange"),
        "file_8 must not get M106 Normal pPrChange: {n}"
    );
}

#[test]
fn m106_file_33_still_ok() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = {
        let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
        let mut f = zip.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}
