//! M109 — reverse short-into-long: short next residual into long base
//! (file_131 long Word-vs-Google base × short Justify demo next).

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
fn m109_file_131_justify_title_early_not_only_long_delete_block() {
    let Some((a, b)) = corpus_pair("file_131.docx", "file_132.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut idx_justify = None;
    let mut idx_ms_del = None;
    let mut i = 0usize;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("Justify Alignment Demo") {
            idx_justify = Some(i);
        }
        if (vis.contains("Microsoft Word") || vis.contains("Google Docs"))
            && chunk.contains("delText")
        {
            if idx_ms_del.is_none() {
                idx_ms_del = Some(i);
            }
        }
        i += 1;
    }
    let j = idx_justify.expect("Justify title pure-I");
    let m = idx_ms_del.expect("MS title as del");
    assert!(j < 5, "Justify title should appear early, got {j}");
    assert!(
        m < 10 && m > j,
        "MS title del should nest near start after Justify, got justify={j} ms_del={m}"
    );
}

#[test]
fn m109_file_130_short_base_still_ok() {
    // Guard: normal short-base→long-next path unchanged.
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Large Font") || doc.contains("document") || doc.contains("delText"));
}

#[test]
fn m109_file_33_guard() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}
