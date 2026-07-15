//! IDENTICAL-INPUT-01 — comparing a document to itself short-circuits after
//! package prep (no dual-body LCS/produce). Output is a valid empty redline
//! (no new tracked changes).

use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;

#[test]
fn self_compare_clean_fixture_has_no_tracked_changes() {
    let bytes = std::fs::read("tests/fixtures/f4/original.docx").expect("fixture");
    let out = compare_documents(&bytes, &bytes, "Bench").expect("self compare");
    let pkg = PartFs::open(&out).expect("open out");
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".into());
    let xml = pkg.part_string(&main).expect("document.xml");
    // Match revision elements only (not w:insideH / w:insideV table borders).
    assert!(
        !xml.contains("<w:ins ")
            && !xml.contains("<w:ins>")
            && !xml.contains("<w:del ")
            && !xml.contains("<w:del>"),
        "self-compare must not invent tracked changes"
    );
}

#[test]
fn self_compare_is_fast_enough_not_to_timeout() {
    let bytes = std::fs::read("tests/fixtures/f4/original.docx").expect("fixture");
    let t0 = std::time::Instant::now();
    let _ = compare_documents(&bytes, &bytes, "Bench").expect("self compare");
    assert!(
        t0.elapsed().as_secs() < 5,
        "self-compare should short-circuit the dual-body pipeline"
    );
}
