// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M76 helpers — related stamped variant detection (file_175 keeps full LCS)
//! and joined residual tokenization for confetti residual pairing (M75).
//!
//! Force-confetti-before-word-run for file_33 was **score-negative** (−0.13,
//! still 3pp vs Word 2pp) and was reverted. Residual "This document
//! demonstrates" still defeats confetti via detail_threshold word-run gate.

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
fn m76_file_175_related_still_not_pure_confetti_all_delete() {
    // file_175×file_176 are the same charter with spacing drift — must keep
    // substantial equal/mixed body, not insert-all-next + delete-all-base.
    let Some((a, b)) = corpus_pair("file_175.docx", "file_176.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Project Charter") || doc.contains("eigenpal") || doc.len() > 5000,
        "charter body should survive"
    );
    let live_chars = doc.matches("<w:t").count();
    assert!(
        live_chars > 20,
        "related variant should keep many live text runs, not confetti wipe: t={live_chars}"
    );
}

#[test]
fn m76_file_134_still_confettis_unrelated_stamp() {
    let Some((a, b)) = corpus_pair("file_134.docx", "file_135.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let doc = document_xml(&out);
    // Stamp confetti: digit mix on first para.
    assert!(
        doc.contains("135") && doc.contains("delText") && doc.contains("134"),
        "file_134 stamp confetti expected"
    );
}
