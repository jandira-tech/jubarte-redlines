//! M87 — last pure-del live numPr → pPrChange; drop mark del (file_55).

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
fn m87_file_55_last_del_numpr_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_55.docx", "file_56.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Last residual pure-del is "b" (list item from A).
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        // Match delText b carefully — avoid partials.
        if !chunk.contains("delText") || !chunk.contains(">b<") {
            // also allow <w:delText>b</w:delText>
            if !(chunk.contains("<w:delText>b</w:delText>") || chunk.contains(">b</w:delText>")) {
                continue;
            }
        } else if !(chunk.contains("<w:delText>b</w:delText>") || chunk.contains(">b</w:delText>"))
        {
            continue;
        }
        // Prefer the chunk that is the short "b" residual (not long body text).
        if chunk.contains("Bold") || chunk.contains("superscript") || chunk.contains("notation") {
            continue;
        }
        found = true;
        // Live numPr must not remain outside pPrChange.
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            !live.contains("<w:numPr") && !live.contains("numId"),
            "last pure-del must not keep live numPr: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("numId"),
            "numPr must sit under pPrChange: {chunk}"
        );
        // No mark-only del in pPr when pPrChange present.
        if let Some(i) = chunk.find("<w:pPr") {
            let rest = &chunk[i..];
            let end = rest.find("</w:pPr>").unwrap_or(rest.len().min(500));
            let ppr = &rest[..end];
            // pPr region before body del: should not have rPr>del when pPrChange exists.
            if ppr.contains("pPrChange") {
                // allow del inside pPrChange? no - del mark is rPr/del sibling of pPrChange
                let before_chg = ppr.split("pPrChange").next().unwrap_or(ppr);
                assert!(
                    !before_chg.contains("<w:del"),
                    "last pure-del with pPrChange must drop mark del: {chunk}"
                );
            }
        }
        break;
    }
    assert!(found, "expected last pure-del of list item b");
}

#[test]
fn m87_file_23_spacing_still_in_pprchange() {
    // Guard M83b still works under generalized M87 movable set.
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Document Title") {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            !live.contains("w:line=\"240\""),
            "last pure-del must not keep live line=240: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("w:line=\"240\""),
            "spacing must sit under pPrChange: {chunk}"
        );
    }
    assert!(found, "expected Document Title pure-del");
}
