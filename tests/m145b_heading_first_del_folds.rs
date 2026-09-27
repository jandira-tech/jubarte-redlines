// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M145b skips the fold when the first deleted paragraph looks like a
//! checklist cell (one or two short tokens). A styled Title/Heading is not a
//! cell: in file_134 × file_135 Word mixes the last inserted paragraph
//! ("…below the main title.") with the deleted Title "Table Widths". Kept
//! apart, every line below it moves down one line. With more deletions
//! after it, the mixed paragraph takes the deleted Title's properties.

use std::io::{Cursor, Read};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn deleted_title_mixes_with_last_inserted_paragraph() {
    let dir = std::path::Path::new("tests/corpus/broken_ones_two/sources");
    let a = std::fs::read(dir.join("file_134.docx")).unwrap();
    let b = std::fs::read(dir.join("file_135.docx")).unwrap();
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(&a, &b, &settings).unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let mixed = xml
        .split("</w:p>")
        .find(|p| {
            p.contains("below the main title.") && p.contains("<w:delText>Table Widths</w:delText>")
        })
        .expect("Word mixes the inserted paragraph with the deleted title");
    // More deletions follow, so the mixed paragraph keeps the deleted
    // Title's properties and its deleted paragraph mark, as Word does.
    let mark = mixed[mixed.rfind("<w:p ").or(mixed.rfind("<w:p>")).unwrap()..]
        .split("</w:pPr>")
        .next()
        .unwrap();
    assert!(
        mark.contains(r#"<w:pStyle w:val="Title""#) && mark.contains("<w:del "),
        "mixed paragraph should carry the deleted Title's pPr and mark: {mark}"
    );
}
