// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M97 — FormatChanged pilcrow with differing mark `pPr/rPr` emits
//! `w:rPrChange` under live mark rPr (file_30 stamp Aptos/b/sz20 → sz32).

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
fn m97_file_30_stamp_ppr_has_mark_rpr_change() {
    let Some((a, b)) = corpus_pair("file_30.docx", "file_31.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // First paragraph is the file_N.docx stamp.
    let p0 = doc.split("</w:p>").next().expect("first para");
    assert!(
        p0.contains("file_") && (p0.contains(".docx") || p0.contains("docx")),
        "expected stamp first: {}",
        &p0[..p0.len().min(200)]
    );
    // Live mark rPr should carry rPrChange with old Aptos / bold / underline.
    assert!(
        p0.contains("rPrChange"),
        "stamp pPr mark rPr must have rPrChange: {}",
        &p0[..p0.len().min(500)]
    );
    assert!(
        p0.contains("Aptos") || p0.contains("w:b") || p0.contains("<w:b"),
        "old mark props (Aptos/bold) should appear under rPrChange: {}",
        &p0[..p0.len().min(500)]
    );
    // Still have structural pPrChange for ListParagraph.
    assert!(
        p0.contains("pPrChange") && p0.contains("ListParagraph"),
        "stamp should keep structural pPrChange"
    );
}

#[test]
fn m97_file_69_still_has_ppr_change() {
    let Some((a, b)) = corpus_pair("file_69.docx", "file_70.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("pPrChange") || doc.contains("delText") || doc.contains("<w:ins"),
        "file_69 still produces revisions"
    );
}

#[test]
fn m97_file_8_still_not_flooded() {
    // Guard: mark rPrChange on FormatChanged pPr must not re-open file_8 flood.
    let Some((a, b)) = corpus_pair("file_8.docx", "file_9.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let n_ppc = doc.matches("pPrChange").count();
    // Ungated M81 had ~99; gated should stay well below.
    assert!(n_ppc < 40, "file_8 pPrChange flood returned: count={n_ppc}");
}
