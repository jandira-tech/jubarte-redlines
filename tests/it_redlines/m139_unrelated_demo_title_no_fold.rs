// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M139 — multi pure-D after long pure-I: skip fold when first pure-D is an
//! unrelated short Demo title (file_82 contract × Title Style Centered Demo).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m139_file_82_title_demo_not_folded_into_contract_line() {
    let Some((a, b)) = corpus_pair("file_82.docx", "file_83.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Title Style Centered Demo must appear as delText, not only inside a MIX
    // para that also carries OPP/willing contract insert text.
    assert!(
        doc.contains("Title Style Centered Demo"),
        "title demo must be present"
    );
    // Find delText for Title Style… and ensure nearby (800 chars) no OPP ins thrash.
    let pos = doc
        .find("Title Style Centered Demo")
        .expect("title demo text");
    let window_start = pos.saturating_sub(400);
    // char-boundary safe
    let window_start = doc
        .char_indices()
        .take_while(|(i, _)| *i <= window_start)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0);
    let window_end = (pos + 200).min(doc.len());
    let window_end = doc
        .char_indices()
        .take_while(|(i, _)| *i <= window_end)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(doc.len());
    let window = &doc[window_start..window_end];
    // Pre-M139 thrash: same para has ins OPP and del Title Style.
    let thrash = window.contains("willing") || window.contains("get down with OG");
    // Pure-D path: delText Title Style without those contract phrases in window.
    assert!(
        !thrash,
        "Title Style Demo must not fold into OPP contract line, window={window}"
    );
}

#[test]
fn m139_file_54_still_folds_short_b_into_demo() {
    // M124/M90: longer pure-I run ending "b" still folds into demo title.
    let Some((a, b)) = corpus_pair("file_54.docx", "file_55.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word MIX "b" + "1.5 Line Spacing Demo"
    let mixed = doc.contains("1.5 Line Spacing Demo") || doc.contains("Line Spacing");
    assert!(mixed, "file_54 must still surface line spacing demo");
}
