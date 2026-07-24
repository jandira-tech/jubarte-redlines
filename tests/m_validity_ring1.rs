//! Ring 1 probes: each check must fire on intentionally broken packages,
//! and a healthy compare output must pass.

mod common;

use std::io::{Cursor, Write};

use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const ORIG: &[u8] = include_bytes!("fixtures/redline/original.docx");
const MOD: &[u8] = include_bytes!("fixtures/redline/modified.docx");

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

const MINIMAL_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:r><w:t>hello</w:t></w:r></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;

#[test]
fn healthy_compare_output_passes_ring1() {
    let out = compare_documents(ORIG, MOD, "Ring1").expect("compare");
    assert_word_valid_package(&out);
}

#[test]
fn probe_dangling_rid_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:hyperlink r:id="rId999"><w:r><w:t>x</w:t></w:r></w:hyperlink></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[
        ("word/document.xml", doc),
        (
            "word/_rels/document.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
        ),
    ]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("dangling") || e.contains("rId999")),
        "expected dangling rId error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_duplicate_revision_id_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:ins w:id="1" w:author="A"><w:r><w:t>a</w:t></w:r></w:ins>
      <w:del w:id="1" w:author="A"><w:r><w:delText>b</w:delText></w:r></w:del>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|e| e.contains("duplicate w:id")),
        "expected duplicate w:id error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_wt_under_del_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:del w:id="1" w:author="A"><w:r><w:t>should be delText</w:t></w:r></w:del>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report.errors.iter().any(|e| e.contains("w:t under w:del")),
        "expected w:t under w:del error, got: {:?}",
        report.errors
    );
}

/// KNOWN ISSUE 1 settled: Word requires `w:t` under `w:moveFrom` — `delText` fails open.
#[test]
fn probe_deltext_under_movefrom_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:r><w:delText>moved text</w:delText></w:r></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("w:delText under w:moveFrom")),
        "expected delText-under-moveFrom error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_wt_under_movefrom_passes_ring1() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:moveFrom w:id="1" w:author="A"><w:r><w:t>moved text</w:t></w:r></w:moveFrom>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        !report.errors.iter().any(|e| e.contains("moveFrom")),
        "w:t under moveFrom must be accepted: {:?}",
        report.errors
    );
}

#[test]
fn probe_orphan_comment_ref_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:commentReference w:id="42"/></w:r>
    </w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("commentReference") && e.contains("42")),
        "expected orphan commentReference error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_malformed_xml_part_fails() {
    // Clearly illegal: bare ampersand + mismatched end tag.
    let bytes = zip_with_parts(&[(
        "word/document.xml",
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>& not entity</w:wrong></w:document>"#,
    )]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("well-formed") || e.contains("not a readable")),
        "expected XML parse error, got: {:?}",
        report.errors
    );
}

#[test]
fn probe_paraid_overflow_fails() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
  <w:body>
    <w:p w14:paraId="80000000"><w:r><w:t>x</w:t></w:r></w:p>
    <w:sectPr/>
  </w:body>
</w:document>"#;
    let bytes = zip_with_parts(&[("word/document.xml", doc)]);
    let report = check_word_valid_package(&bytes);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("0x80000000") || e.contains("paraId")),
        "expected paraId overflow error, got: {:?}",
        report.errors
    );
}

#[test]
fn minimal_valid_package_passes() {
    let bytes = zip_with_parts(&[("word/document.xml", MINIMAL_DOC)]);
    // May warn about missing content type defaults for nothing else — must pass core checks.
    let report = check_word_valid_package(&bytes);
    assert!(
        report.ok(),
        "minimal package should pass: {:?}",
        report.errors
    );
    // Ensure PartFs can open it too
    PartFs::open(&bytes).expect("open");
}
