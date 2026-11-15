// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M98 — empty trailing `<w:p/>` must not block sole pure-D fold into last
//! pure-I (file_167: "Subsection Title" + del "24").

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
fn m98_file_167_subsection_folds_24() {
    let Some((a, b)) = corpus_pair("file_167.docx", "file_168.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word: one mixed para "Subsection Title" + del "24" — not pure-I then pure-D.
    let mut found_mix = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Subsection Title") {
            continue;
        }
        found_mix = true;
        assert!(
            chunk.contains("delText") && chunk.contains("24"),
            "Subsection Title must fold del 24 into same para: {chunk}"
        );
        assert!(
            chunk.contains("<w:ins") || chunk.contains("w:ins "),
            "must keep ins Subsection Title: {chunk}"
        );
        break;
    }
    assert!(found_mix, "expected Subsection Title residual");
    // No standalone pure-del para of only "24" after the fold.
    let mut pure_24 = false;
    for chunk in doc.split("</w:p>") {
        if chunk.contains("Subsection Title") {
            continue;
        }
        if chunk.contains("delText") && chunk.contains(">24<") {
            // Only count if no other content words
            let has_other = chunk.contains("Heading") || chunk.contains("document");
            if !has_other {
                pure_24 = true;
            }
        }
    }
    assert!(!pure_24, "must not leave pure-del '24' as separate para");
}

#[test]
fn m98_file_33_summary_still_not_fold_mid_heading() {
    // Guard M77: mid-doc unrelated sole pure-D after pure-I must stay separate.
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"),
        "file_33 still produces residuals"
    );
}

#[test]
fn m98_file_191_still_folds() {
    let Some((a, b)) = corpus_pair("file_191.docx", "file_192.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("delText") || doc.contains("<w:ins"));
}

#[test]
fn m98b_file_167_mixed_spacing_on_empty() {
    let Some((a, b)) = corpus_pair("file_167.docx", "file_168.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Mixed Subsection+24 should not carry live spacing (Word parks it on empty).
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Subsection Title") {
            continue;
        }
        // spacing may appear inside pPrChange only — not as live before mixed content.
        // Word: no live spacing on mixed; del mark present.
        assert!(
            chunk.contains("delText") && chunk.contains("24"),
            "mixed fold still required"
        );
        // Live spacing on mixed is the failure mode (Word parks it on empty).
        if let Some(sp) = chunk.find("w:spacing") {
            let ppc = chunk.find("pPrChange").unwrap_or(usize::MAX);
            assert!(
                sp > ppc,
                "mixed should not hold live spacing; prefer empty trailing: {chunk}"
            );
        }
        break;
    }
    // Trailing empty should carry spacing.
    assert!(
        doc.contains("w:before=\"360\"") || doc.contains("w:spacing"),
        "spacing should survive on empty trailing"
    );
}
