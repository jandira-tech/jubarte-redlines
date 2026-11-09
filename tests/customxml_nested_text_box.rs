// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A nested text box in custom XML is not a story, so Ring-1 does not
//! report it. The same markup in the main document still is.

use std::io::{Cursor, Write};

use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const NEST: &str = r#"<w:txbxContent xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:txbx/></w:txbxContent>"#;

fn findings(parts: &[(&str, &str)]) -> Vec<jubarte::validate::Finding> {
    let bytes = zip_with_parts(parts);
    jubarte::validate::ring1(&PartFs::open(&bytes).unwrap())
        .into_iter()
        .filter(|f| f.code == "NESTED_TEXT_BOX")
        .collect()
}

fn zip_with_parts(parts: &[(&str, &str)]) -> Vec<u8> {
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
        for (name, body) in parts {
            z.start_file(*name, opts).unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

#[test]
fn a_nested_text_box_in_custom_xml_is_not_reported() {
    let document = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" Target="../customXml/item1.xml"/>
</Relationships>"#;
    let custom = findings(&[
        ("word/document.xml", document),
        ("word/_rels/document.xml.rels", rels),
        ("customXml/item1.xml", NEST),
    ]);
    assert!(
        custom.is_empty(),
        "custom xml nested text box was reported: {custom:?}"
    );

    let story_document = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><w:body><w:p><w:r><w:txbxContent><wps:txbx/></w:txbxContent></w:r></w:p></w:body></w:document>"#;
    let story = findings(&[("word/document.xml", story_document)]);
    assert_eq!(story.len(), 1, "a nest in the main document: {story:?}");
    assert_eq!(story[0].part, "word/document.xml");
}
