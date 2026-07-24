// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M10 — deleted opaque drawings/text-boxes must serialize text as `w:delText`
//! (fixture f-3).
//!
//! A deletion's text leaf must be `<w:delText>`, never `<w:t>` — `w:t` inside
//! `w:del` is non-conformant OOXML (Word writes `w:delText`, and a `w:t` under
//! `w:del` is invisible to revision-accept tooling, so accepting/rejecting leaves
//! the text behind).
//!
//! Bug: when an original-only `w:drawing` / text box / `w:sdt` is deleted, the
//! comparer cloned the opaque subtree verbatim in `produce.rs`, leaving its inner
//! `w:t` unconverted; `finalize` then wrapped the run in `<w:del>` around `<w:t>`.
//! Fixture f-3 (`Strict01` cover page: an author field, an abstract `w:sdt`, and a
//! text box) is original-only and entirely deleted, so it exercised this path.
//!
//! Requires the ISO Strict input support (`Strict01` is a Strict document); this
//! change is stacked on that PR.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;
use quick_xml::NsReader;
use quick_xml::events::Event;
use quick_xml::name::{Namespace, ResolveResult};

/// WordprocessingML main namespace — OOXML validity is keyed on this URI, not on
/// the `w:` prefix, so the scan resolves namespaces instead of matching prefixes.
const W_NS: &[u8] = b"http://schemas.openxmlformats.org/wordprocessingml/2006/main";

const STRICT_ORIGINAL: &[u8] = include_bytes!("fixtures/strict/Strict01.docx");
const MODIFIED: &[u8] = include_bytes!("fixtures/strict/sd-2517-localized-heading-styles.docx");

/// Count `<w:t>` elements that appear as descendants of a `<w:del>`, returning a
/// sample of the offending text for diagnostics.
fn w_t_under_w_del(docx: &[u8]) -> (usize, Vec<String>) {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).expect("valid docx");
    let xml = {
        let mut f = zip.by_name("word/document.xml").expect("document.xml");
        let mut s = String::new();
        f.read_to_string(&mut s).expect("utf8");
        s
    };

    let mut reader = NsReader::from_str(&xml);
    reader.config_mut().trim_text(false);

    let mut del_depth = 0i32;
    let mut in_offending_t = false;
    let mut count = 0usize;
    let mut samples: Vec<String> = Vec::new();

    let is_w = |ns: &ResolveResult| matches!(ns, ResolveResult::Bound(Namespace(n)) if *n == W_NS);

    loop {
        match reader.read_resolved_event() {
            Ok((ns, Event::Start(e))) if is_w(&ns) => match e.local_name().as_ref() {
                b"del" => del_depth += 1,
                b"t" if del_depth > 0 => {
                    count += 1;
                    in_offending_t = true;
                }
                _ => {}
            },
            Ok((ns, Event::End(e))) if is_w(&ns) => match e.local_name().as_ref() {
                b"del" => del_depth -= 1,
                b"t" => in_offending_t = false,
                _ => {}
            },
            Ok((_, Event::Text(t))) if in_offending_t => {
                if samples.len() < 5 {
                    let raw = t.into_inner();
                    let s = String::from_utf8_lossy(&raw).trim().to_string();
                    if !s.is_empty() {
                        samples.push(s);
                    }
                }
            }
            Ok((_, Event::Eof)) => break,
            Err(e) => panic!("XML parse error: {e}"),
            _ => {}
        }
    }
    (count, samples)
}

#[test]
fn f3_deleted_drawing_text_uses_deltext_not_t() {
    let out = compare_documents(STRICT_ORIGINAL, MODIFIED, "Redline").expect("compare ok");
    let (count, samples) = w_t_under_w_del(&out);
    assert_eq!(
        count, 0,
        "found {count} <w:t> element(s) inside <w:del> (must be <w:delText>); \
         non-conformant deleted content — sample text: {samples:?}"
    );
}
