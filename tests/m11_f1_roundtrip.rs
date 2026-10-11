// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M11 — f-1 acceptance round-trip (parity with Word).
//!
//! f-1 is the *acceptance* round-trip: its input documents are obtained by
//! reject-all / accept-all of an existing redline. Word's redline (`word-redline`,
//! = `third_step_eigen` vs `eigen_via_word`) carries the revisions, so
//! reject-all(word) reproduces the ORIGINAL and accept-all(word) the MODIFIED.
//!
//! We re-derive those inputs with our own `reject_revisions_document` /
//! `accept_revisions_document`, re-run OUR `compare_documents`, and assert the
//! result buckets every character into the same source side as Word's redline
//! (coalescing-invariant reconstruction parity — see `m9_roundtrip`). This proves
//! the accept/reject round trip and the compare agree with Word end-to-end.

use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::Dom;

/// Word's f-1 redline (the source of both derived inputs).
const WORD_REDLINE: &[u8] = include_bytes!("fixtures/f1/word-redline.docx");

/// Reject-all (`accept=false`) or accept-all (`accept=true`) every revision in a
/// docx, returning the resulting docx bytes.
fn derive(docx: &[u8], accept: bool) -> Vec<u8> {
    // Open the package once: read the main part, transform it, write it back.
    let mut pkg = PartFs::open(docx).unwrap();
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main).unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let new_root = if accept {
        accept_revisions_document(&mut dom, root)
    } else {
        reject_revisions_document(&mut dom, root)
    };
    let out_xml = dom.serialize_element(new_root);
    pkg.set_part(&main, out_xml.into_bytes());
    pkg.to_zip().unwrap()
}

/// Visible text of a resolved package. A resolved story has ordinary `w:t`
/// leaves regardless of whether its source ownership came from run revisions,
/// native moves, or whole-row lifetime markers.
fn resolved_text(docx: &[u8]) -> String {
    let pkg = PartFs::open(docx).unwrap();
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::t()))
        .into_iter()
        .map(|node| dom.value(node))
        .collect()
}

/// Resolve both projections using the same public revision semantics as Word
/// round trips. Counting only `w:ins` ancestors misclassified `moveTo` text in
/// inserted rows as shared content, duplicating it in the original projection.
fn reconstruct(docx: &[u8]) -> (String, String) {
    (
        resolved_text(&derive(docx, false)),
        resolved_text(&derive(docx, true)),
    )
}

#[test]
fn f1_acceptance_roundtrip_matches_word() {
    let orig = derive(WORD_REDLINE, false); // reject-all == original (third_step_eigen)
    let modi = derive(WORD_REDLINE, true); // accept-all == modified (eigen_via_word)
    let ours = compare_documents(&orig, &modi, "Author").expect("compare ok");

    let (oo, om) = reconstruct(&ours);
    let (wo, wm) = reconstruct(WORD_REDLINE);
    assert_eq!(
        oo,
        wo,
        "f-1 original reconstruction diverges from Word (Δ {})",
        oo.len() as i64 - wo.len() as i64
    );
    assert_eq!(
        om,
        wm,
        "f-1 modified reconstruction diverges from Word (Δ {})",
        om.len() as i64 - wm.len() as i64
    );
}

/// `derive` runs accept/reject and repackages via `PartFs` (set_part + to_zip).
/// Cover that repackaging path: both derived inputs must be valid, loadable docx
/// packages on their own (a broken set_part/to_zip would surface here).
#[test]
fn derived_inputs_are_loadable() {
    use std::io::Cursor;
    for accept in [false, true] {
        let docx = derive(WORD_REDLINE, accept);
        let doc = ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(
            Cursor::new(docx),
        )
        .unwrap_or_else(|e| panic!("derived (accept={accept}) docx must load: {e:?}"));
        assert!(doc.main_document_part().is_ok());
    }
}

#[test]
fn reconstruction_respects_native_moves_and_whole_row_lifetimes() {
    let mut pkg = PartFs::open(WORD_REDLINE).unwrap();
    let main = pkg.main_document_part().unwrap();
    pkg.set_part(&main, format!(
        "<w:document xmlns:w='{}'><w:body><w:p><w:r><w:t>Base</w:t></w:r></w:p><w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w='1000'/></w:tblGrid><w:tr><w:trPr><w:del w:id='1' w:author='Author' w:date='2001-02-03T04:05:06Z'/></w:trPr><w:tc><w:tcPr><w:tcW w:w='1000' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>Old row</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:ins w:id='2' w:author='Author' w:date='2001-02-03T04:05:06Z'/></w:trPr><w:tc><w:tcPr><w:tcW w:w='1000' w:type='dxa'/></w:tcPr><w:p><w:r><w:t>New row</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:moveFromRangeStart w:id='3'/><w:moveFrom w:id='4' w:author='Author' w:date='2001-02-03T04:05:06Z'><w:r><w:delText>Old move</w:delText></w:r></w:moveFrom><w:moveFromRangeEnd w:id='3'/><w:moveToRangeStart w:id='5'/><w:moveTo w:id='6' w:author='Author' w:date='2001-02-03T04:05:06Z'><w:r><w:t>New move</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id='5'/><w:r><w:t>Tail</w:t></w:r></w:p><w:sectPr/></w:body></w:document>", W::URI
    ).into_bytes());
    let input = pkg.to_zip().unwrap();
    assert_eq!(
        reconstruct(&input),
        (
            "BaseOld rowOld moveTail".to_string(),
            "BaseNew rowNew moveTail".to_string()
        )
    );
}
