// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Same-slot residual pairing (file_111 × file_112). Every changed
//! paragraph pairs with its counterpart in the same position, as in Word's
//! own redline of the pair: four mixed paragraphs, "This document" kept.
//! The tuned candidates paired the base body with the NEXT body on a shared
//! trailing "bold", inserting the first body whole and deleting the last.

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn source(name: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    std::fs::read(root.join(name)).unwrap()
}

fn body_paragraphs(docx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let body = xml.split("<w:body>").nth(1).unwrap();
    body.split("</w:p>")
        .filter(|p| p.contains("<w:p ") || p.contains("<w:p>"))
        .map(str::to_string)
        .collect()
}

#[test]
fn every_changed_paragraph_pairs_with_its_same_slot_counterpart() {
    let out = compare_documents(
        &source("file_111.docx"),
        &source("file_112.docx"),
        "Jubarte",
    )
    .unwrap();
    let paragraphs = body_paragraphs(&out);
    assert_eq!(paragraphs.len(), 4, "Word keeps four paragraphs");
    let body0 = &paragraphs[2];
    assert!(body0.contains("<w:ins ") && body0.contains("<w:del "));
    let kept = body0.split("<w:ins ").next().unwrap();
    assert!(
        kept.contains(">This document </w:t>"),
        "\"This document \" stays unchanged text: {body0}"
    );
}
