//! Identical-input / drawing-id fixup must not accept a package whose main
//! document part is missing. Returning Ok(bytes) masks corrupt input that the
//! normal compare path would reject with PartNotFound.

use jubarte::comparer::fixups::fix_up_drawing_ids_in_package;
use jubarte::opc::OpcError;
use std::io::{Cursor, Write};

/// Minimal OPC zip: content types + root rel pointing at main, but **no**
/// `word/document.xml` part bytes.
fn package_missing_main_document() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
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
        // intentionally no word/document.xml
        z.finish().unwrap();
    }
    buf
}

#[test]
fn fix_up_drawing_ids_errors_when_main_document_missing() {
    let docx = package_missing_main_document();
    let err = fix_up_drawing_ids_in_package(&docx).expect_err("must not succeed on missing main");
    match err {
        OpcError::PartNotFound(msg) => {
            assert!(
                msg.contains("document") || msg.contains("main") || msg.contains("word/"),
                "error should name the missing main part: {msg}"
            );
        }
        other => panic!("expected PartNotFound, got {other:?}"),
    }
}
