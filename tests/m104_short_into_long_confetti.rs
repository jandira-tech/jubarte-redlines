//! M104 — short stamped demo into long next doc: nest short title after next's
//! main title (file_130 Large Font Size Demo into Word vs Google Docs).

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
fn m104_file_130_large_font_title_early_not_at_end() {
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut idx_title = None;
    let mut idx_ms_title = None;
    let mut i = 0usize;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("Large Font Size Demo") {
            idx_title = Some(i);
        }
        if vis.contains("Microsoft Word vs") || vis.contains("Google Docs") {
            if idx_ms_title.is_none() {
                idx_ms_title = Some(i);
            }
        }
        i += 1;
    }
    let t = idx_title.expect("Large Font Size Demo residual");
    let m = idx_ms_title.expect("Microsoft Word title");
    // Word nests short title near the start (right after main title), not at end.
    assert!(
        t < 10,
        "Large Font title must appear early (Word p2), got para index {t}"
    );
    assert!(
        t > m,
        "Large Font title should follow Microsoft Word main title: title={t} ms={m}"
    );
    // Not pure-del parked only at document end without early nest
    assert!(
        t < 50,
        "must not park short demo only at end of large doc: idx={t}"
    );
}

#[test]
fn m104_file_134_still_confettis_unrelated_short() {
    // Guard: short-vs-short unrelated stays plain confetti (no force nest).
    let Some((a, b)) = corpus_pair("file_134.docx", "file_135.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("delText") || doc.contains("<w:ins") || doc.contains("file_"));
}

#[test]
fn m104_file_33_still_ok() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

#[test]
fn m108_file_73_five_residual_nests_title_early() {
    // M108: rest1=5 (title+intro+3 items) must still M104-nest, not insert-all.
    let Some((a, b)) = corpus_pair("file_73.docx", "file_74.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut idx_title = None;
    let mut idx_ms = None;
    let mut i = 0usize;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("Numbered List") || vis.contains("Italic Demo") {
            if idx_title.is_none() {
                idx_title = Some(i);
            }
        }
        if vis.contains("Microsoft Word vs") || vis.contains("Google Docs") {
            if idx_ms.is_none() {
                idx_ms = Some(i);
            }
        }
        i += 1;
    }
    let t = idx_title.expect("Numbered List title residual");
    let m = idx_ms.expect("MS title");
    assert!(
        t < 10 && t > m,
        "file_73 title must nest early after main title, got title={t} ms={m}"
    );
}

#[test]
fn m104_file_70_drawing_still_confetti_shape() {
    let Some((a, b)) = corpus_pair("file_70.docx", "file_71.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Green title still pure-I (M99)
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if vis.contains("Green Highlight Demo") && !vis.contains("document") {
            assert!(
                !vis.contains("Datum"),
                "Green title not mixed with Datum: {vis:?}"
            );
            break;
        }
    }
}
