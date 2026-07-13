//! M4.H.1 — relationship parsing helpers.

use jubarte::comparer::parts::{
    decode_xml_attribute, is_external_relationship, parse_relationship_rows,
    required_rel_type_suffix,
};

#[test]
fn m4_h1_parse_rels() {
    let xml = r#"<?xml version="1.0"?><Relationships xmlns="x">
      <Relationship Id="rId1" Type="http://x/image" Target="media/i1.png"/>
      <Relationship Id="rId2" Type="http://x/hyperlink" Target="http://a.com/?q=1&amp;r=2" TargetMode="External"/>
      <Relationship Id="rId3" Type="http://x/chart"/>
    </Relationships>"#;
    let rows = parse_relationship_rows(xml);
    assert_eq!(rows.len(), 2, "row missing Target is dropped");
    assert_eq!(rows[0].id, "rId1");
    assert_eq!(rows[0].target, "media/i1.png");
    assert!(!rows[0].external);
    assert_eq!(rows[1].target, "http://a.com/?q=1&r=2", "entities decoded");
    assert!(rows[1].external);
}

#[test]
fn m4_h1_decode() {
    assert_eq!(
        decode_xml_attribute("a&amp;b&lt;c&gt;d&quot;e"),
        "a&b<c>d\"e"
    );
}

#[test]
fn m4_h1_required_suffix() {
    assert_eq!(required_rel_type_suffix("hyperlink"), Some("/hyperlink"));
    assert_eq!(required_rel_type_suffix("hlinkClick"), Some("/hyperlink"));
    assert_eq!(required_rel_type_suffix("blip"), Some("/image"));
    assert_eq!(required_rel_type_suffix("imagedata"), Some("/image"));
    assert_eq!(required_rel_type_suffix("OLEObject"), Some("/oleObject"));
    assert_eq!(required_rel_type_suffix("chart"), Some("/chart"));
    assert_eq!(required_rel_type_suffix("p"), None);
}

#[test]
fn m4_h1_is_external() {
    assert!(is_external_relationship(
        "http://x/hyperlink",
        "media/x.png"
    ));
    assert!(is_external_relationship(
        "http://x/image",
        "https://a.com/i.png"
    ));
    assert!(is_external_relationship("http://x/image", "mailto:a@b.com"));
    assert!(!is_external_relationship("http://x/image", "media/x.png"));
    assert!(!is_external_relationship(
        "http://x/image",
        "/word/media/x.png"
    ));
}

/// `parse_relationship_rows` must mark a row external when `is_external_relationship`
/// says so, even if `TargetMode="External"` is absent — otherwise an external
/// hyperlink / absolute-URI row is treated as internal and its part-copy fails in
/// reconcile, orphaning the rId. Internal relative targets stay internal.
#[test]
fn m4_h1_parse_rels_external_classification() {
    let xml = r#"<?xml version="1.0"?><Relationships xmlns="x">
      <Relationship Id="rId1" Type="http://x/hyperlink" Target="http://a.com/p"/>
      <Relationship Id="rId2" Type="http://x/image" Target="https://cdn.example/i.png"/>
      <Relationship Id="rId3" Type="http://x/image" Target="media/i1.png"/>
      <Relationship Id="rId4" Type="http://x/image" Target="https://e.example/x.png" TargetMode="External"/>
    </Relationships>"#;
    let rows = parse_relationship_rows(xml);
    assert_eq!(rows.len(), 4);
    // hyperlink rel type, no TargetMode → external
    assert!(
        rows[0].external,
        "hyperlink type is external even without TargetMode: {rows:?}"
    );
    // absolute-URI target, no TargetMode → external
    assert!(
        rows[1].external,
        "absolute-URI target is external even without TargetMode: {rows:?}"
    );
    // internal relative target → internal
    assert!(
        !rows[2].external,
        "relative media target stays internal: {rows:?}"
    );
    // explicit TargetMode=External still honored
    assert!(
        rows[3].external,
        "explicit TargetMode=External honored: {rows:?}"
    );
}

use jubarte::comparer::parts::reconcile_dangling_relationships;
use jubarte::namespaces::{A, R, W};
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;

/// M4.H.3 — an orphan rId (in no package) has its attribute dropped (no dangling rel).
#[test]
fn m4_h3_reconcile_drops_orphan() {
    let orig = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut dest = PartFs::open(&orig).unwrap();
    let mut d = Dom::new();
    let root = d.new_element(W::document());
    let body = d.new_element(W::body());
    let p = d.new_element(W::p());
    let r = d.new_element(W::r());
    let drawing = d.new_element(W::name("drawing"));
    let blip = d.new_element(A::name("blip"));
    d.set_attribute_value(blip, &R::name("embed"), Some("rIdORPHAN999"));
    d.add(drawing, blip);
    d.add(r, drawing);
    d.add(p, r);
    d.add(body, p);
    d.add(root, body);

    reconcile_dangling_relationships(&mut d, root, &mut dest, &[]);
    // orphan attribute dropped → no dangling reference remains
    assert_eq!(
        d.attribute(blip, &R::name("embed")),
        None,
        "dangling rId dropped"
    );
}

/// M4.H.3 — text result (no rId references) is untouched and still produces a
/// Word-valid (ooxmlsdk-loadable) redline end-to-end.
#[test]
fn m4_h3_text_redline_still_valid() {
    use std::io::Cursor;
    let o = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let m = std::fs::read("tests/fixtures/redline/modified.docx").unwrap();
    let out = jubarte::document_comparer::compare_documents(&o, &m, "Test Author").unwrap();
    let doc =
        ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(out))
            .unwrap();
    assert!(doc.main_document_part().is_ok());
}
