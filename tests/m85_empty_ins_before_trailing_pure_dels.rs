//! M85 — empty pure-ins before trailing pure-dels dropped; last pure-del mark-only pPr stripped.
//! file_49: B ends empty after table; Word is `tbl DDD`, we had `tbl Ei DDD`.

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

/// Body fragment after last `</w:tbl>` (file_49 has one table).
fn after_last_tbl(doc: &str) -> &str {
    doc.rsplit("</w:tbl>").next().unwrap_or("")
}

#[test]
fn m85_file_49_no_empty_ins_between_table_and_pure_dels() {
    let Some((a, b)) = corpus_pair("file_49.docx", "file_50.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let after = after_last_tbl(&doc);
    // First residual content should be pure-del of Small Font Size Demo.
    assert!(
        after.contains("Small Font Size Demo") || after.contains("delText"),
        "expected pure-del residual after table: {after}"
    );
    // Between </w:tbl> and first delText: no empty pure-ins shell.
    let before_del = after.split("Small Font Size Demo").next().unwrap_or(after);
    let has_empty_ins = before_del.contains("<w:ins")
        && !before_del.contains("<w:t")
        && !before_del.contains("delText");
    assert!(
        !has_empty_ins,
        "empty pure-ins between table and pure-dels must be dropped: {before_del}"
    );
}

#[test]
fn m85_file_49_last_pure_del_no_mark_only_ppr() {
    let Some((a, b)) = corpus_pair("file_49.docx", "file_50.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Last residual: "Small fonts are used in footnotes and disclaimers."
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Small fonts are used in footnotes") {
            continue;
        }
        found = true;
        // Mark-only pPr is pPr > rPr > del with no other props — strip it.
        let has_mark_shell = if let Some(i) = chunk.find("<w:pPr") {
            let rest = &chunk[i..];
            let end = rest.find("</w:pPr>").unwrap_or(rest.len().min(400));
            let ppr = &rest[..end];
            ppr.contains("<w:del")
                && !ppr.contains("spacing")
                && !ppr.contains("pStyle")
                && !ppr.contains("pPrChange")
        } else {
            false
        };
        assert!(
            !has_mark_shell,
            "last pure-del must not keep mark-only pPr: {chunk}"
        );
        break;
    }
    assert!(found, "expected last pure-del about footnotes");
}

#[test]
fn m85_file_23_still_no_trailing_empty_after_table() {
    // Guard: M83 file_23 must remain clean under M85.
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let after = after_last_tbl(&doc);
    let before_del = after.split("Title Style Demo").next().unwrap_or(after);
    let has_empty_ins = before_del.contains("<w:ins")
        && !before_del.contains("<w:t")
        && !before_del.contains("delText");
    assert!(
        !has_empty_ins,
        "file_23 must still drop empty pure-ins after table: {before_del}"
    );
}
