// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M123 zips stamped demo bodies positionally when a body pair is related.
//! "This" and "." are not a relation: Calibri heading × underline bodies
//! share nothing else, and Word inserts the revised first body whole, then
//! joins the revised last body to the original's first. The positional zip
//! meshed "This" and "." across unrelated sentences.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[test]
fn unrelated_bodies_do_not_zip_on_this_and_period() {
    let src =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(src.join("file_209.docx")).unwrap(),
        &std::fs::read(src.join("file_210.docx")).unwrap(),
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
    let first_body = xml
        .split("</w:p>")
        .find(|p| p.contains("demonstrates underline"))
        .expect("revised first body");
    assert!(
        !first_body.contains("Calibri font"),
        "revised first body meshed with the original's: {first_body}"
    );
}
