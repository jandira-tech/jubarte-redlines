//! M81 — Equal pilcrow with differing pPr emits w:pPrChange (file_69 after=20).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m81b_conjoin_keeps_deleted_spacing_when_inserted_empty() {
    use jubarte::comparer::WmlComparerSettings;
    use jubarte::comparer::finalize::conjoin_paragraph_marks;
    use jubarte::namespaces::{PT, W};
    use jubarte::xmllinq::Dom;

    let mut d = Dom::new();
    let p = d.new_element(W::p());
    // Deleted: A's after=20 (file_69 drawing residual)
    let ppr_del = d.new_element(W::p_pr());
    d.set_attribute_value(ppr_del, &PT::status(), Some("Deleted"));
    let sp = d.new_element(W::name("spacing"));
    d.set_attribute_value(sp, &W::name("after"), Some("20"));
    d.add(ppr_del, sp);
    d.add(p, ppr_del);
    // Inserted: empty structural props (B bare pPr)
    let ppr_ins = d.new_element(W::p_pr());
    d.set_attribute_value(ppr_ins, &PT::status(), Some("Inserted"));
    d.add(p, ppr_ins);
    let r = d.new_element(W::r());
    d.add(p, r);

    let s = WmlComparerSettings::default();
    let out = conjoin_paragraph_marks(&mut d, p, &s);
    let pprs = d.elements(out, Some(&W::p_pr()));
    assert_eq!(pprs.len(), 1);
    let live = d.serialize_element(pprs[0]);
    assert!(
        live.contains("after=\"20\""),
        "live must keep Deleted after=20: {live}"
    );
    // after=20 must not be buried only in pPrChange when live is Deleted.
    if let Some(chg) = live.find("pPrChange") {
        let before = &live[..chg];
        assert!(
            before.contains("after=\"20\""),
            "after=20 live before pPrChange: {live}"
        );
    }
}

#[test]
fn m81_file_69_stamp_pprchange_after20() {
    let Some((a, b)) = corpus_pair("file_69.docx", "file_70.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("pPrChange"),
        "body must carry w:pPrChange for stamp spacing delta, got len={}",
        doc.len()
    );
    assert!(
        doc.contains("w:after=\"20\"") || doc.contains("w:after=\"20\""),
        "pPrChange / live should record A's after=20: {}",
        &doc[..doc.len().min(2000)]
    );
}

#[test]
fn m81_synthetic_equal_ppr_spacing_emits_pprchange() {
    // Minimal A/B packages: same paragraph text, different after spacing.
    use jubarte::document_comparer::compare_documents_with_options;

    fn mini(text: &str, after: Option<&str>) -> Vec<u8> {
        let spacing = match after {
            Some(a) => format!(r#"<w:spacing w:after="{a}"/>"#),
            None => String::new(),
        };
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr>{spacing}</w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
        );
        let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/><w:qFormat/>
  </w:style>
</w:styles>"#;
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
        let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("[Content_Types].xml", opts).unwrap();
            use std::io::Write;
            zip.write_all(content_types.as_bytes()).unwrap();
            zip.start_file("_rels/.rels", opts).unwrap();
            zip.write_all(rels.as_bytes()).unwrap();
            zip.start_file("word/document.xml", opts).unwrap();
            zip.write_all(document.as_bytes()).unwrap();
            zip.start_file("word/_rels/document.xml.rels", opts)
                .unwrap();
            zip.write_all(doc_rels.as_bytes()).unwrap();
            zip.start_file("word/styles.xml", opts).unwrap();
            zip.write_all(styles.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    let a = mini("Hello", Some("20"));
    let b = mini("Hello", None);
    let out =
        compare_documents_with_options(&a, &b, "Tester", "1970-01-01T00:00:00Z").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("pPrChange"),
        "Equal text + spacing delta must emit pPrChange: {doc}"
    );
    assert!(
        doc.contains("after=\"20\""),
        "old spacing after=20 in pPrChange: {doc}"
    );
}
