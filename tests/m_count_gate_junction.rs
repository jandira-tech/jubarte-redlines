// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Unrelated documents too short for the wholesale shortcut (a title and a
//! table against three headings) still take Word's junction: the revised
//! document's last paragraph joins the original's first paragraph, whose mark
//! is deleted. Full LCS had nothing to pair but a stray "1" and kept the two
//! apart.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[test]
fn revised_last_paragraph_joins_original_title() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_199.docx")).unwrap(),
        &std::fs::read(src.join("file_200.docx")).unwrap(),
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
    let joined = xml
        .split("</w:p>")
        .find(|p| p.contains("Quarterly Performance Report"))
        .expect("original title");
    assert!(
        joined.contains("Red bold headings are used for urgent document warnings."),
        "revised last paragraph not joined to the original title: {joined}"
    );
}
