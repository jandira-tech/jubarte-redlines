// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Unrelated short documents that both end on an empty paragraph (a titled
//! table against a short item list): Word inserts the revised document whole,
//! deletes the original, and pairs the two final empties. The revised last
//! text paragraph keeps its own inserted mark. Full LCS welded the table's
//! title onto the last item.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[test]
fn last_item_keeps_its_own_paragraph() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_207.docx")).unwrap(),
        &std::fs::read(src.join("file_208.docx")).unwrap(),
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
    let last_item = xml
        .split("</w:p>")
        .find(|p| p.contains("Shown 2."))
        .expect("last item");
    assert!(
        !last_item.contains("Plain grid"),
        "table title welded onto the last item: {last_item}"
    );
}
