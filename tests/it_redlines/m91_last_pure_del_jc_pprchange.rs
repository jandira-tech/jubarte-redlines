// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M91 — last pure-del live `jc` → pPrChange (file_105).

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
fn m91_file_105_last_del_jc_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_105.docx", "file_106.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Last residual pure-del is the signature line.
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("elegant signature") {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            !live.contains("w:val=\"right\"") && !live.contains("<w:jc"),
            "last pure-del must not keep live jc: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("right"),
            "jc must sit under pPrChange: {chunk}"
        );
        // No mark-only del when pPrChange present.
        if let Some(i) = chunk.find("<w:pPr") {
            let rest = &chunk[i..];
            let end = rest.find("</w:pPr>").unwrap_or(rest.len().min(500));
            let ppr = &rest[..end];
            if ppr.contains("pPrChange") {
                let before_chg = ppr.split("pPrChange").next().unwrap_or(ppr);
                assert!(
                    !before_chg.contains("<w:del"),
                    "last pure-del with pPrChange must drop mark del: {chunk}"
                );
            }
        }
        break;
    }
    assert!(found, "expected last pure-del about elegant signature");
}

#[test]
fn m91_file_105_mid_dels_keep_live_jc() {
    let Some((a, b)) = corpus_pair("file_105.docx", "file_106.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Mid pure-del "Right Aligned Italic Demo" keeps live jc.
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Right Aligned Italic Demo") {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(
            live.contains("<w:jc") || live.contains("w:val=\"right\""),
            "mid pure-del must keep live jc: {chunk}"
        );
        break;
    }
    assert!(found, "expected mid pure-del title");
}

#[test]
fn m91_file_55_numpr_still_in_pprchange() {
    // Guard M87 under M91 movable expansion.
    let Some((a, b)) = corpus_pair("file_55.docx", "file_56.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !(chunk.contains("<w:delText>b</w:delText>") || chunk.contains(">b</w:delText>")) {
            continue;
        }
        if chunk.contains("Bold") || chunk.contains("notation") {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("pPrChange") && chunk.contains("numId"),
            "file_55 last pure-del numPr still under pPrChange: {chunk}"
        );
        break;
    }
    assert!(found, "expected last pure-del b");
}
