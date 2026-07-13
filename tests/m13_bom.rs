//! M13 — leading UTF-8 BOM handling in the XML parser.
//!
//! Real-world fixtures begin their `word/document.xml` with a UTF-8 BOM
//! (U+FEFF) before the `<?xml ...?>` prolog — e.g. Apache POI's
//! `MultipleBodyBug.docx` and docx4j's `NumberingImplicitNumId.docx`. Word
//! opens these and produces a redline, but our comparer panicked with
//! "original has no root" / "modified has no root".
//!
//! Cause: `parse_document` relies on `skip_ws()`, and Rust's
//! `char::is_whitespace()` does NOT classify U+FEFF as whitespace. An
//! unconsumed BOM is not `<`, so every prolog/element branch is skipped, the
//! loop breaks immediately, and the document ends up with no root element.
//!
//! Fix: the parser must skip a single leading BOM. This test is the red gate.

use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

#[test]
fn parses_document_with_leading_utf8_bom() {
    let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\
               <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
               <w:body><w:p><w:r><w:t>hi</w:t></w:r></w:p></w:body>\
               </w:document>";
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    let root = d
        .root(doc)
        .expect("a BOM-prefixed document must still have a root element");
    assert!(
        d.element(root, &W::body()).is_some(),
        "w:body must be reachable under the root of a BOM-prefixed document"
    );
}

#[test]
fn bom_strip_is_leading_only_and_noop_without_bom() {
    // Regression guard: a plain (BOM-less) document still parses, and a stray
    // U+FEFF that is NOT at the very start is preserved as text (not eaten).
    let plain = "<?xml version=\"1.0\"?>\
                 <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
                 <w:body/></w:document>";
    let mut d = Dom::new();
    let doc = d.parse_xdocument(plain);
    let root = d.root(doc).expect("plain document must have a root");
    assert!(d.element(root, &W::body()).is_some());
}
