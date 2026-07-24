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
use jubarte::opc::PartFs;
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::Dom;
use quick_xml::Reader;
use quick_xml::events::Event;

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

/// `(original_text, modified_text)` reconstructed from a redline (see m9):
/// `delText`→original, `w:t` under `w:ins`→modified, other `w:t`→both.
fn reconstruct(docx: &[u8]) -> (String, String) {
    // Resolve the main part via the package instead of hard-coding the path.
    let pkg = PartFs::open(docx).unwrap();
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main).unwrap();
    let mut r = Reader::from_str(&xml);
    r.config_mut().trim_text(false);
    let (mut ins, mut dt, mut t) = (0i32, 0i32, 0i32);
    let (mut o, mut m) = (String::new(), String::new());
    loop {
        match r.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"w:ins" => ins += 1,
                b"w:delText" => dt += 1,
                b"w:t" => t += 1,
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"w:ins" => ins -= 1,
                b"w:delText" => dt -= 1,
                b"w:t" => t -= 1,
                _ => {}
            },
            Ok(Event::Text(x)) => {
                let s = String::from_utf8_lossy(&x.into_inner()).into_owned();
                if dt > 0 {
                    o.push_str(&s);
                } else if t > 0 && ins > 0 {
                    m.push_str(&s);
                } else if t > 0 {
                    o.push_str(&s);
                    m.push_str(&s);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {}
        }
    }
    (o, m)
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
