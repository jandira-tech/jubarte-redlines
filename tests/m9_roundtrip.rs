// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M9 — round-trip reconstruction invariant (fixture f-4).
//!
//! The defining property of a *correct* redline: with every revision applied one
//! way it must reproduce the ORIGINAL, and the other way the MODIFIED.
//! Concretely, walking the output in document order:
//!   - deleted text (`w:delText`, i.e. under `w:del`) is ORIGINAL-only,
//!   - inserted text (`w:t` under `w:ins`) is MODIFIED-only,
//!   - everything else (`w:t` not under `w:ins`) is EQUAL — in BOTH.
//!
//! So `delText + equal == original` and `ins + equal == modified`.
//!
//! This reconstruction is invariant to *coalescing granularity* (how runs are
//! split) and to the *equal-vs-(del+ins)* choice — those move the same text into
//! the same bucket(s). It fires only on true **source misattribution**: content
//! assigned to the wrong side. Microsoft Word's own Compare output satisfies it
//! exactly; we must too.
//!
//! Regression: f-4 (`eigenpal` vs `page-numbering`). The original-only npm/github
//! install-table hyperlink content (e.g. `@eigenpal/docx-js-editor`) was leaking
//! into our MODIFIED reconstruction — i.e. original-only content was NOT marked
//! deleted — so accepting our redline kept content the modified document removed.
//! The oracle is Word's own redline of the same pair.

use std::io::{Cursor, Read};

mod common;
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::compare_documents;
use quick_xml::Reader;
use quick_xml::events::Event;

const ORIGINAL: &[u8] = include_bytes!("fixtures/f4/original.docx");
const MODIFIED: &[u8] = include_bytes!("fixtures/f4/modified.docx");
/// Microsoft Word's own Compare output for the same pair — the parity oracle.
const WORD_REDLINE: &[u8] = include_bytes!("fixtures/f4/word-redline.docx");

/// `(original_text, modified_text)` reconstructed from a redline's
/// `word/document.xml`, per the bucket rules above.
fn reconstruct(docx: &[u8]) -> (String, String) {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).expect("valid docx");
    let xml = {
        let mut f = zip.by_name("word/document.xml").expect("has document.xml");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("utf8 document.xml");
        s
    };

    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);

    let (mut ins, mut del, mut in_del_text, mut in_t) = (0i32, 0i32, 0i32, 0i32);
    let (mut orig, mut modi) = (String::new(), String::new());

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"w:ins" => ins += 1,
                b"w:del" => del += 1,
                b"w:delText" => in_del_text += 1,
                b"w:t" => in_t += 1,
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"w:ins" => ins -= 1,
                b"w:del" => del -= 1,
                b"w:delText" => in_del_text -= 1,
                b"w:t" => in_t -= 1,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                let raw = t.into_inner();
                let s = String::from_utf8_lossy(&raw).into_owned();
                if in_del_text > 0 {
                    orig.push_str(&s); // deleted == original-only
                } else if in_t > 0 && ins > 0 {
                    modi.push_str(&s); // inserted == modified-only
                } else if in_t > 0 {
                    orig.push_str(&s); // equal == both
                    modi.push_str(&s);
                }
                let _ = del; // del-depth is implied by delText nesting; kept for clarity
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {}
        }
    }
    (orig, modi)
}

#[test]
fn f4_original_only_content_is_not_attributed_to_modified() {
    let out = compare_documents(ORIGINAL, MODIFIED, "Redline").expect("compare ok");
    assert_word_valid_package(&out);
    let (orig, modi) = reconstruct(&out);

    // `github` is the npm/github install-table label, present ONLY in the original
    // (verified: 1 in eigenpal, 0 in page-numbering — and, unlike the package name
    // `@eigenpal/docx-js-editor`, it does NOT also appear in the equal code sample).
    // It MUST reconstruct into the original side and MUST NOT appear on the modified.
    assert!(
        orig.contains("github"),
        "BUG: original-only content (the install table) is missing from the original \
         reconstruction — it was misattributed to the modified side"
    );
    assert!(
        !modi.contains("github"),
        "BUG: original-only content `github` leaked into the modified reconstruction — \
         accepting this redline would keep content the modified document removed"
    );
}

#[test]
fn f4_reconstruction_matches_word() {
    // Coalescing-invariant parity: every character must land in the same source
    // bucket as Microsoft Word's own Compare output.
    //
    // Pinned to the PowerTools-faithful preset: THIS WORD_REDLINE artifact was
    // produced by a Word run with accept-first semantics for the original's
    // pre-existing tracked changes, matching the faithful contract. The
    // 166-pair benchmark ground truth (a different Word batch) PRESERVES
    // pre-existing deletions as struck-through history, which word-mode now
    // matches instead (m32 w14 — forensics: cicerodo, page-numbering pairs).
    use jubarte::comparer::WmlComparerSettings;
    use jubarte::document_comparer::compare_documents_with_settings;
    let settings = WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        ..WmlComparerSettings::powertools_faithful()
    };
    let out = compare_documents_with_settings(ORIGINAL, MODIFIED, &settings).expect("compare ok");
    assert_word_valid_package(&out);
    let (our_orig, our_mod) = reconstruct(&out);
    let (word_orig, word_mod) = reconstruct(WORD_REDLINE);

    assert_eq!(
        our_orig,
        word_orig,
        "original reconstruction diverges from Word (Δ {} chars)",
        our_orig.len() as i64 - word_orig.len() as i64
    );
    assert_eq!(
        our_mod,
        word_mod,
        "modified reconstruction diverges from Word (Δ {} chars)",
        our_mod.len() as i64 - word_mod.len() as i64
    );
}
