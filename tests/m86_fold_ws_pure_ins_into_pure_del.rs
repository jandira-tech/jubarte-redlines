//! M86 — whitespace-only pure-ins folds into following pure-del (file_88).

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
fn m86_file_88_space_ins_folds_with_first_pure_del() {
    let Some((a, b)) = corpus_pair("file_88.docx", "file_89.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Title residual should share a paragraph with whitespace ins, not sit alone
    // after a pure space-only pure-ins.
    assert!(
        doc.contains("Track Changes Suggesting Bold Demo"),
        "expected deleted title residual"
    );
    // Find the para chunk with the title delText.
    let mut found_mixed = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Track Changes Suggesting Bold Demo") {
            continue;
        }
        // Mixed: has both ins (space or comment-bearing) and del of title.
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del") || chunk.contains("delText");
        found_mixed = has_ins && has_del;
        break;
    }
    assert!(
        found_mixed,
        "whitespace pure-ins must fold into first pure-del (mixed para)"
    );
}

#[test]
fn m86_file_49_still_no_empty_after_table() {
    // Guard: M85 catalog demos stay clean under M86.
    let Some((a, b)) = corpus_pair("file_49.docx", "file_50.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let after = doc.rsplit("</w:tbl>").next().unwrap_or("");
    let before_del = after.split("Small Font Size Demo").next().unwrap_or(after);
    let has_empty_ins = before_del.contains("<w:ins")
        && !before_del.contains("<w:t")
        && !before_del.contains("delText");
    assert!(
        !has_empty_ins,
        "file_49 must still drop empty pure-ins after table: {before_del}"
    );
}

#[test]
fn m86_file_33_content_pure_ins_not_folded_into_unrelated_del() {
    // Guard: content pure-I must not fold into unrelated pure-D (file_33).
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // If Summary is present as pure-ins, it should not also contain a Heading del
    // in the same paragraph (unrelated fold).
    for chunk in doc.split("</w:p>") {
        if (chunk.contains(">Summary<") || chunk.contains(">Summary</"))
            && chunk.contains("Heading")
            && chunk.contains("delText")
        {
            panic!("content pure-ins Summary must not fold unrelated Heading del: {chunk}");
        }
    }
}
