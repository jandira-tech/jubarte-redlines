// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::document_comparer::compare_documents_with_options;

const ORIGINAL: &[u8] = include_bytes!("fixtures/redline/original.docx");
const MODIFIED: &[u8] = include_bytes!("fixtures/redline/modified.docx");

/// M5.2: the façade runs end-to-end on real fixtures and produces a valid zip
/// whose document.xml carries tracked-revision markup.
#[test]
fn compare_documents_produces_redline_zip() {
    let out =
        compare_documents_with_options(ORIGINAL, MODIFIED, "Test Author", "2020-01-01T00:00:00Z")
            .expect("compare ok");
    assert!(!out.is_empty());

    // reopen as a package and inspect document.xml
    let pkg = jubarte::opc::PartFs::open(&out).expect("reopen");
    let doc = pkg.part_string("word/document.xml").expect("document.xml");
    assert!(doc.contains("<w:document"));
    // tracked-revision markup is present (these fixtures differ in body text)
    assert!(
        doc.contains("<w:ins") || doc.contains("<w:del"),
        "expected tracked changes in output"
    );
    assert!(doc.contains("w:author=\"Test Author\""));
}

/// M5.3 (Word-valid gate, proxy): the produced redline re-opens cleanly through
/// ooxmlsdk's typed loader — a strong proxy for "Word valid".
#[test]
fn redline_is_ooxmlsdk_loadable() {
    use std::io::Cursor;

    let out =
        compare_documents_with_options(ORIGINAL, MODIFIED, "Test Author", "2020-01-01T00:00:00Z")
            .expect("compare ok");

    let doc =
        ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
            .expect("ooxmlsdk must load the produced redline");
    assert!(doc.main_document_part().is_ok());
}
