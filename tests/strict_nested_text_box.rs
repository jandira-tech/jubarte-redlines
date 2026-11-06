// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A nested text box in an ISO Strict document is the same Ring-1 finding
//! as the Transitional shape.

use std::io::{Cursor, Write};

use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const STRICT_W: &str = "http://purl.oclc.org/ooxml/wordprocessingml/main";
const TRANSITIONAL_W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn story(inner: &str) -> String {
    format!("<w:txbxContent><w:p><w:r><w:t>Alice</w:t>{inner}</w:r></w:p></w:txbxContent>")
}

fn drawing(inner: &str) -> String {
    format!(
        "<w:drawing><wp:anchor><wps:wsp><wps:txbx>{}</wps:txbx></wps:wsp></wp:anchor></w:drawing>",
        story(inner)
    )
}

fn vml(inner: &str) -> String {
    format!(
        "<w:pict><v:shape><v:textbox>{}</v:textbox></v:shape></w:pict>",
        story(inner)
    )
}

/// Probe shape with one box inside another's readable content.
fn nested_document(w_ns: &str) -> String {
    let nested = format!(
        "<mc:AlternateContent><mc:Choice Requires=\"wps\">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>",
        drawing(""),
        vml("")
    );
    let outer = format!(
        "<mc:AlternateContent><mc:Choice Requires=\"wps\">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>",
        drawing(&nested),
        vml(&vml(""))
    );
    document(w_ns, &outer)
}

/// One text box and no second box inside it.
fn lone_document(w_ns: &str) -> String {
    document(w_ns, &drawing(""))
}

fn document(w_ns: &str, body: &str) -> String {
    format!(
        r#"<w:document xmlns:w="{w_ns}" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r>{body}</w:r></w:p></w:body></w:document>"#
    )
}

fn nested_findings(xml: &str) -> Vec<jubarte::validate::Finding> {
    let bytes = zip_with_document(xml);
    jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "NESTED_TEXT_BOX")
        .collect()
}

fn zip_with_document(document: &str) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opts).unwrap();
        z.write_all(
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
        )
        .unwrap();
        z.start_file("word/document.xml", opts).unwrap();
        z.write_all(document.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

#[test]
fn a_strict_nested_text_box_is_reported() {
    let findings = nested_findings(&nested_document(STRICT_W));
    assert!(
        !findings.is_empty(),
        "strict nested text box produced no finding: {findings:?}"
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = &findings[0];
    assert_eq!(finding.code, "NESTED_TEXT_BOX");
    assert_eq!(finding.part, "word/document.xml");
    assert!(finding.word_fatal, "{finding:?}");
    assert!(finding.location.contains("w:txbxContent[0]"), "{finding:?}");

    let transitional = nested_findings(&nested_document(TRANSITIONAL_W));
    assert_eq!(transitional.len(), 1, "{transitional:?}");
    assert_eq!(transitional[0].code, "NESTED_TEXT_BOX");

    let lone = nested_findings(&lone_document(STRICT_W));
    assert!(
        lone.is_empty(),
        "a single strict text box is not a nest: {lone:?}"
    );
}
