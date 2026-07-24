// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.I — golden CONTENT-parity regression guard. Asserts the redlined *text*
//! (deleted `w:delText` + inserted `w:t` under `w:ins`) matches the TS golden
//! exactly — the real correctness metric. (`w:ins`/`w:del` *element* counts differ
//! only by run-coalescing granularity and are not asserted.) The actual decoded
//! text strings are compared, so this is true content equality, not just a length
//! check. Also asserts ooxmlsdk-loadability.

use std::io::Cursor;

mod common;
use common::validity::assert_word_valid_package;

/// The redlined text of a document part: (inserted, deleted).
fn redline_texts(bytes: &[u8]) -> (String, String) {
    let z = jubarte::opc::PartFs::open(bytes).unwrap();
    let x = z.part_string("word/document.xml").unwrap();
    (ins_text(&x), del_text(&x))
}

/// Concatenated, decoded `w:delText` content (deletions are never nested).
fn del_text(x: &str) -> String {
    let mut rest = x;
    let mut out = String::new();
    while let Some(i) = rest.find("<w:delText") {
        let after = &rest[i + "<w:delText".len()..];
        let Some(gt) = after.find('>') else { break };
        let body = &after[gt + 1..];
        let Some(j) = body.find("</w:delText>") else {
            break;
        };
        out.push_str(&decode(&body[..j]));
        rest = &body[j + "</w:delText>".len()..];
    }
    out
}

/// Depth-aware concatenation of inserted text: every `w:t` text node under at
/// least one `w:ins`, in document order. (Paragraph-mark insertions nest `w:ins`
/// inside `w:ins`; a naive block scan double-counts the inner text.)
fn ins_text(x: &str) -> String {
    let mut ins_depth: i32 = 0;
    let mut in_t = false;
    let mut out = String::new();
    let mut rest = x;
    while let Some(lt) = rest.find('<') {
        if in_t && ins_depth > 0 {
            out.push_str(&decode(&rest[..lt]));
        }
        rest = &rest[lt..];
        let Some(gt) = rest.find('>') else { break };
        let tag = &rest[..=gt];
        let closing = tag.starts_with("</");
        let self_close = tag.ends_with("/>");
        let name_start = if closing { 2 } else { 1 };
        let name: String = tag[name_start..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != '>' && *c != '/')
            .collect();
        if name == "w:ins" {
            if closing {
                ins_depth -= 1;
            } else if !self_close {
                ins_depth += 1;
            }
        } else if name == "w:t" {
            if closing {
                in_t = false;
            } else if !self_close {
                in_t = true;
            }
        }
        rest = &rest[gt + 1..];
    }
    out
}

/// Decode the XML predefined entities. `&amp;` MUST be decoded LAST, otherwise a
/// literal `&lt;` (encoded `&amp;lt;`) would be double-decoded to `<`.
fn decode(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn run(orig: &str, modi: &str) -> Vec<u8> {
    let o = std::fs::read(orig).unwrap();
    let m = std::fs::read(modi).unwrap();
    // This gate guards the POWERTOOLS-FAITHFUL configuration (the TS goldens
    // were generated with it); the library default is Word-visual mode.
    let settings = jubarte::comparer::WmlComparerSettings {
        author_for_revisions: "Test Author".to_string(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".to_string(),
        ..jubarte::comparer::WmlComparerSettings::powertools_faithful()
    };
    let out =
        jubarte::document_comparer::compare_documents_with_settings(&o, &m, &settings).unwrap();
    assert_word_valid_package(&out);
    out
}

fn golden_texts(name: &str) -> (String, String) {
    redline_texts(&std::fs::read(format!("tests/goldens/{name}.redline.docx")).unwrap())
}

fn ensure_loadable(out: Vec<u8>) {
    let doc =
        ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
            .unwrap();
    assert!(doc.main_document_part().is_ok());
}

/// Assert the redline of (orig, modified) reproduces the golden's inserted AND
/// deleted text exactly, and that the output is Word-valid.
fn assert_content_parity(name: &str, orig: &str, modi: &str) {
    let out = run(orig, modi);
    let (ins, del) = redline_texts(&out);
    let (gi, gd) = golden_texts(name);
    assert_eq!(del, gd, "{name}: deleted text must match golden exactly");
    assert_eq!(ins, gi, "{name}: inserted text must match golden exactly");
    ensure_loadable(out);
}

#[test]
fn parity_redline() {
    assert_content_parity(
        "redline",
        "tests/fixtures/redline/original.docx",
        "tests/fixtures/redline/modified.docx",
    );
}

/// inpi is pure-additive — its deleted text is empty.
#[test]
fn parity_inpi() {
    assert_content_parity(
        "inpi",
        "tests/fixtures/redline-inpi/original-new.docx",
        "tests/fixtures/redline-inpi/modified-new.docx",
    );
    let (_, gd) = golden_texts("inpi");
    assert!(gd.is_empty(), "inpi golden is pure-additive (no deletions)");
}

/// inpi2 is a heavy revision (orig carried 195 existing insertions; 90→11 paras).
#[test]
fn parity_inpi2() {
    assert_content_parity(
        "inpi2",
        "tests/fixtures/redline-inpi/original-new-2.docx",
        "tests/fixtures/redline-inpi/modified-new-2.docx",
    );
}

/// Unit-guard for the entity decoder: `&amp;` is decoded last (no double-decode).
#[test]
fn decode_order_is_correct() {
    assert_eq!(decode("a &amp; b"), "a & b");
    assert_eq!(decode("&lt;tag&gt;"), "<tag>");
    // encoded literal "&lt;" must round-trip to "&lt;", not "<"
    assert_eq!(decode("&amp;lt;"), "&lt;");
}
