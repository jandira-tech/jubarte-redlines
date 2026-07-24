// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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

/// Reconcile must classify source hyperlinks without TargetMode as external
/// (same rule as `parse_relationship_rows`). Otherwise it tries an internal
/// part copy for `http://…` targets and orphans the rId.
#[test]
fn m4_h3_reconcile_hyperlink_without_target_mode_is_external() {
    use std::io::{Cursor, Write};

    // Dest: empty-ish package with a main part (from fixture).
    let orig = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    let mut dest = PartFs::open(&orig).unwrap();

    // Source package whose document rels have a hyperlink WITHOUT TargetMode.
    let mut src_buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut src_buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )
        .unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body><w:p/></w:body></w:document>"#,
        )
        .unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        // NO TargetMode — must still be treated as external via hyperlink type.
        z.write_all(
            br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdHL" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/page"/>
</Relationships>"#,
        )
        .unwrap();
        z.finish().unwrap();
    }
    let src = PartFs::open(&src_buf).unwrap();

    let mut d = Dom::new();
    let root = d.new_element(W::document());
    let body = d.new_element(W::body());
    let p = d.new_element(W::p());
    let hl = d.new_element(W::name("hyperlink"));
    d.set_attribute_value(hl, &R::name("id"), Some("rIdHL"));
    d.add(p, hl);
    d.add(body, p);
    d.add(root, body);

    reconcile_dangling_relationships(&mut d, root, &mut dest, &[&src]);
    let new_id = d
        .attribute(hl, &R::name("id"))
        .expect("hyperlink rId must be preserved/remapped, not dropped");
    let dest_doc = dest
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".into());
    let rels = dest.read_rels_for(&dest_doc).expect("dest rels");
    let row = rels
        .items
        .iter()
        .find(|r| r.id == new_id)
        .expect("dest must carry the hyperlink relationship");
    assert_eq!(
        row.target_mode.as_deref(),
        Some("External"),
        "hyperlink without TargetMode in source must land as External: {row:?}"
    );
    assert!(
        row.target.contains("example.com"),
        "target preserved: {row:?}"
    );
}

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
