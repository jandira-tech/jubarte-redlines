//! Word-parity: strip `w:pStyle`/`w:rStyle` that styles.xml does not define.
//!
//! Source demos often set `pStyle=Heading1` without defining Heading1 in
//! styles.xml. Word's redline omits the attribute; LibreOffice still paints
//! built-in Heading look when it remains. Package path must match Word.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::footnotes::{defined_style_ids, strip_unresolved_style_refs};
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// Minimal zip package: document with pStyle=Heading1, styles with only Normal.
fn minimal_heading_pkg(body_inner: &str) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("[Content_Types].xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#,
        )
        .unwrap();
        zip.start_file("_rels/.rels", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )
        .unwrap();
        zip.start_file("word/_rels/document.xml.rels", opts)
            .unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#,
        )
        .unwrap();
        zip.start_file("word/document.xml", opts).unwrap();
        let doc = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body_inner}<w:sectPr/></w:body>
</w:document>"#
        );
        zip.write_all(doc.as_bytes()).unwrap();
        zip.start_file("word/styles.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
  </w:style>
</w:styles>"#,
        )
        .unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}

#[test]
fn strip_unresolved_removes_heading1_keeps_normal() {
    let mut dom = Dom::new();
    let xml = format!(
        r#"<w:document xmlns:w="{w}">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:rPr><w:rStyle w:val="Strong"/></w:rPr><w:t>x</w:t></w:r></w:p>
    <w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr>
      <w:r><w:rPr><w:rStyle w:val="Normal"/></w:rPr><w:t>y</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let mut defined = std::collections::HashSet::new();
    defined.insert("Normal".to_string());
    let n = strip_unresolved_style_refs(&mut dom, root, &defined);
    assert_eq!(n, 2, "removed Heading1 and Strong only, got {n}");
    let out = dom.serialize_element(root);
    assert!(
        !out.contains("Heading1"),
        "Heading1 pStyle must be gone: {out}"
    );
    assert!(!out.contains("Strong"), "Strong rStyle must be gone: {out}");
    assert!(
        out.matches("Normal").count() >= 2,
        "defined Normal refs must remain: {out}"
    );
}

#[test]
fn defined_style_ids_reads_style_elements() {
    let mut dom = Dom::new();
    let xml = format!(
        r#"<w:styles xmlns:w="{w}">
  <w:style w:type="paragraph" w:styleId="Normal"/>
  <w:style w:type="paragraph" w:styleId="Heading1"/>
</w:styles>"#,
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let ids = defined_style_ids(&dom, root);
    assert!(ids.contains("Normal"));
    assert!(ids.contains("Heading1"));
    assert_eq!(ids.len(), 2);
}

/// Full package: two nearly-identical Heading1 demos with no Heading1 in styles.
/// Output must not carry pStyle=Heading1 (Word omits it).
#[test]
fn package_compare_strips_dangling_heading1_pstyle() {
    let para = |text: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:spacing w:line="276"/></w:pPr>
            <w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:b/><w:sz w:val="40"/></w:rPr>
            <w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let a = minimal_heading_pkg(&format!(
        "{}{}{}",
        para("Heading 1 Bold Demo"),
        para("This document shows Heading 1 style with extra bold emphasis."),
        para("Heading 1 with bold creates the strongest document headers.")
    ));
    let b = minimal_heading_pkg(&format!(
        "{}{}{}",
        para("Heading 1 Style Demo"),
        para("This document demonstrates Heading 1 paragraph style."),
        para("Main Title Section")
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = read_part(&out, "word/document.xml");
    assert!(
        !doc.contains(r#"pStyle w:val="Heading1""#) && !doc.contains("w:val=\"Heading1\""),
        "dangling Heading1 pStyle must be stripped from package output: {}",
        &doc[..doc.len().min(1200)]
    );
    // text-level mix still present
    assert!(
        doc.contains("Style") || doc.contains("Bold"),
        "expected text revisions in body"
    );
}

/// Real batch fixtures when present (CI may skip if batch_to_fix absent).
#[test]
fn batch_heading1_bold_vs_style_no_dangling_pstyle() {
    let base = std::path::Path::new("tests/corpus/batch_to_fix/pairs");
    // Not in bottom-50 folder layout by that name — use corpus via path relative
    // if batch extract has a related pair; otherwise synthetic package test covers it.
    let a_path = std::path::Path::new(
        "tests/corpus/batch_to_fix/pairs/41_heading_1_bold_demo_id_paraid_overflow_heading_1_style_demo_id_paraid_overflow/base.docx",
    );
    let b_path = std::path::Path::new(
        "tests/corpus/batch_to_fix/pairs/41_heading_1_bold_demo_id_paraid_overflow_heading_1_style_demo_id_paraid_overflow/next.docx",
    );
    let (a, b) = if a_path.exists() && b_path.exists() {
        (
            std::fs::read(a_path).unwrap(),
            std::fs::read(b_path).unwrap(),
        )
    } else {
        // no fixtures — synthetic test above is the gate
        let _ = base;
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = read_part(&out, "word/document.xml");
    assert!(
        !doc.contains("Heading1"),
        "batch/corpus heading_1 pair must not emit dangling Heading1: {}",
        &doc[..doc.len().min(800)]
    );
}
