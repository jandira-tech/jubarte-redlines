//! M8 — ISO/IEC 29500 **Strict** input support (regression: fixture f-3).
//!
//! Microsoft Word reads a Strict `.docx` natively and, when you Compare it,
//! emits a redline whose entire package is **Transitional** — verified against
//! Word's own f-3 output (`Strict01` vs `sd-2517`), which carries ZERO
//! `purl.oclc.org/ooxml` URIs in any part. Our comparer must match that.
//!
//! Before the fix, `compare_documents` PANICKED at `document_comparer.rs:55`
//! ("original has no body"): a Strict `w:body` lives in
//! `http://purl.oclc.org/ooxml/wordprocessingml/main`, but `W::body()` only
//! matches the Transitional URI, so the body lookup returned `None`.
//!
//! The fix (`strict_translation`) normalizes Strict → Transitional at load — the
//! same step the OpenXML SDK performs before PowerTools' WmlComparer runs — so
//! the body is found, the diff runs, and the output is uniformly Transitional
//! and Word-valid.
//!
//! Fixtures are vendored under `tests/fixtures/strict/` (the f-3 input pair).

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;

/// The Strict ISO original of the f-3 pair.
const STRICT_ORIGINAL: &[u8] = include_bytes!("fixtures/strict/Strict01.docx");
/// A Transitional modified document (the f-3 modified).
const MODIFIED: &[u8] = include_bytes!("fixtures/strict/sd-2517-localized-heading-styles.docx");

/// Every part (parts + `.rels` + `[Content_Types].xml`) of a docx, by raw zip read.
fn all_zip_entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("valid zip/docx");
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).expect("zip entry");
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).expect("read entry");
        out.push((name, buf));
    }
    out
}

#[test]
fn f3_strict_original_compares_without_panic_and_has_a_body() {
    // Was: panic "original has no body". Now: a real redline.
    let out = compare_documents(STRICT_ORIGINAL, MODIFIED, "Redline")
        .expect("compare must succeed when the ORIGINAL is an ISO Strict document");

    let pkg = PartFs::open(&out).expect("output is a valid OPC package");
    let doc = pkg
        .part_string("word/document.xml")
        .expect("output has a main document part");
    assert!(
        doc.contains("<w:body"),
        "redline output must contain a <w:body>"
    );
}

#[test]
fn f3_strict_output_is_fully_transitional() {
    let out = compare_documents(STRICT_ORIGINAL, MODIFIED, "Redline")
        .expect("compare must succeed when the ORIGINAL is an ISO Strict document");

    // Word's f-3 output has no strict URIs anywhere. Match it across every XML /
    // .rels / [Content_Types].xml entry — including the relationship parts that
    // PartFs hides behind its parsed model.
    for (name, bytes) in all_zip_entries(&out) {
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let s = String::from_utf8_lossy(&bytes);
            assert!(
                !s.contains("purl.oclc.org/ooxml"),
                "ISO Strict URI leaked into output part `{name}` — output is not Transitional"
            );
        }
    }
}
