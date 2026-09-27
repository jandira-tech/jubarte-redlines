// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A single shared word that opens a paragraph on both sides stays an anchor
//! in a long unrelated window, as in Word's redline: "Second" opens both the
//! green list's "Second green underlined item" and the revised "Second page",
//! so that paragraph keeps "Second" live, deletes the rest of the item and
//! inserts "page". The detail threshold (1 word of 54) used to void it.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[test]
fn paragraph_opening_word_anchors_across_unrelated_documents() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_201.docx")).unwrap(),
        &std::fs::read(src.join("file_202.docx")).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let para = xml
        .split("</w:p>")
        .find(|p| p.contains("green underlined item") && p.contains(">page<"))
        .expect("the anchored paragraph mixes the deleted item and the inserted page");
    let live_second = para.split("<w:t").any(|t| {
        t.split_once('>')
            .is_some_and(|(_, rest)| rest.starts_with("Second"))
    });
    assert!(live_second, "Second is not kept live: {para}");
}

/// A word that ends a paragraph on both sides anchors a wholesale
/// replacement, with Word's seam on each side of it: "2026" closes both
/// "Product Roadmap 2026" and "Date: February 1, 2026", so that paragraph
/// keeps "2026" and its mark live and replaces the words before it.
#[test]
fn paragraph_closing_word_anchors_wholesale_replacement() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_203.docx")).unwrap(),
        &std::fs::read(src.join("file_204.docx")).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let para = xml
        .split("</w:p>")
        .find(|p| p.contains("Product Roadmap"))
        .expect("original title");
    assert!(
        para.contains("Date: February 1,"),
        "title does not carry the revised date: {para}"
    );
    let mark = para.split("</w:pPr>").next().unwrap_or_default();
    assert!(
        !mark.contains("<w:del ") && !mark.contains("<w:ins "),
        "anchored paragraph mark is not live: {para}"
    );
}
