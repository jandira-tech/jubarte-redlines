//! M58 — stamped filenames on otherwise-unrelated docs: confetti the stamp,
//! then pure insert-all-next / delete-all-base for the body (Word pattern on
//! file_134_file_135). Full-doc LCS was mixing next titles into base deletions.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn doc_body(dom: &mut Dom, inner: &str) -> (jubarte::xmllinq::NodeId, jubarte::xmllinq::NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

#[test]
fn m58_stamp_then_pure_ins_before_pure_del_body() {
    let mut dom = Dom::new();
    // Fully disjoint content words (only file_N stamp shared) so the
    // unrelated short-circuit fires. Any shared token keeps full LCS.
    let base = [
        para("file_134.docx"),
        para("TableWidths alphabase charter"),
        para("ExplicitlyDefinedWidths betabase zebra"),
        para("FixedWidthTable gammabase yak"),
        para("ExtraBaseParagraph deltacount"),
    ]
    .concat();
    let next = [
        para("file_135.docx"),
        para("SubtitleStyleDemo nextalpha heading"),
        para("DemonstratesSubtitle nextbeta quilt"),
        para("SecondaryHeading nextgamma xylophone"),
        para("ExtraNextParagraph nextdelta"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<_> = dom
        .elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| dom.serialize_element(p))
        .collect();
    assert!(
        paras.len() >= 5,
        "expected stamp + body paras, got {}",
        paras.len()
    );
    // p0 confetti
    assert!(
        paras[0].contains("135") && paras[0].contains("delText") && paras[0].contains("134"),
        "stamp confetti: {p0}",
        p0 = paras[0]
    );
    // No paragraph mixes next title with base title (Word keeps pure-I titles).
    let mixed = paras
        .iter()
        .any(|p| p.contains("SubtitleStyleDemo") && p.contains("TableWidths"));
    assert!(
        !mixed,
        "must not mix next title into base title del in one para: {paras:?}"
    );
    // Base body deleted somewhere as pure del (no next title text)
    let has_pure_del = paras
        .iter()
        .any(|p| p.contains("TableWidths") && p.contains("delText") && !p.contains("Subtitle"));
    assert!(
        has_pure_del,
        "base body should appear as pure del somewhere: {paras:?}"
    );
}

#[test]
fn m58_real_file_134_package_pure_ins_titles() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_134.docx");
    let b = root.join("file_135.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    use std::io::Read;
    f.read_to_string(&mut xml).unwrap();
    // No single paragraph should mix Subtitle insert with Table Widths del.
    let bad = xml
        .split("<w:p")
        .any(|chunk| chunk.contains("Subtitle Style Demo") && chunk.contains("Table Widths"));
    assert!(
        !bad,
        "Subtitle Style Demo must not share a paragraph with deleted Table Widths"
    );
}

/// file_175 related charters: confetti still used for LO score (pairing deferred).
/// Compare must succeed with shared body vocabulary.
#[test]
fn m58_related_stamped_variants_compare_ok() {
    use jubarte::document_comparer::compare_documents_with_settings;
    use std::path::PathBuf;
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_175.docx");
    let b = root.join("file_176.docx");
    if !a.is_file() || !b.is_file() {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    use std::io::Read;
    f.read_to_string(&mut xml).unwrap();
    assert!(
        xml.contains("eigenpal") || xml.contains("docx-editor") || xml.contains("Project Charter"),
        "body vocabulary from both sides should survive compare"
    );
    assert!(
        xml.contains("w:ins") || xml.contains("w:del"),
        "related variants still produce revisions"
    );
}
