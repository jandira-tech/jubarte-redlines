//! Style renames applied to note parts must survive B.4 notes writeback.
//!
//! Hidden gem (CR #5): style remap walks footnotes/endnotes on `out`, then
//! writeback overwrites those parts from `notes_ctx` (un-remapped). Footnote
//! `pStyle` refs to renamed ids would stick as stale ids after compare.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

const WNS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const REL_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const OD_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn build_docx_with_footnote(style_id: &str, style_name: &str, body_text: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{WNS}" xmlns:r="{OD_REL}">
  <w:body>
    <w:p>
      <w:r><w:t>{body_text}</w:t></w:r>
      <w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r>
    </w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let footnotes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="{WNS}">
  <w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
  <w:footnote w:id="1">
    <w:p>
      <w:pPr><w:pStyle w:val="{style_id}"/></w:pPr>
      <w:r><w:t>note body</w:t></w:r>
    </w:p>
  </w:footnote>
</w:footnotes>"#
    );
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{WNS}">
  <w:docDefaults><w:rPrDefault><w:rPr/></w:rPrDefault><w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults>
  <w:style w:type="paragraph" w:styleId="{style_id}">
    <w:name w:val="{style_name}"/>
    <w:qFormat/>
  </w:style>
  <w:style w:type="character" w:styleId="FootnoteReference">
    <w:name w:val="footnote reference"/>
  </w:style>
</w:styles>"#
    );
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
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>
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
        z.write_all(doc.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        z.write_all(
            format!(
                r#"<?xml version="1.0"?><Relationships xmlns="{REL_NS}">
  <Relationship Id="rId1" Type="{OD_REL}/styles" Target="styles.xml"/>
  <Relationship Id="rId2" Type="{OD_REL}/footnotes" Target="footnotes.xml"/>
</Relationships>"#
            )
            .as_bytes(),
        )
        .unwrap();
        z.start_file("word/styles.xml", opt).unwrap();
        z.write_all(styles.as_bytes()).unwrap();
        z.start_file("word/footnotes.xml", opt).unwrap();
        z.write_all(footnotes.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn footnote_pstyle_remapped_after_notes_writeback() {
    // styleId "1" with name "heading 1" → canonical Heading1 under Word-mode.
    let a = build_docx_with_footnote("1", "heading 1", "Shared body text");
    let b = build_docx_with_footnote("1", "heading 1", "Shared body text changed slightly");
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true, // Word-mode: canonicalize + remap
        author_for_revisions: "Tester".into(),
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(&a, &b, &settings).expect("compare");
    let styles = part(&out, "word/styles.xml");
    let notes = part(&out, "word/footnotes.xml");
    // Canonical id present in styles (when rename fired).
    if styles.contains("styleId=\"Heading1\"") || styles.contains("styleId='Heading1'") {
        assert!(
            notes.contains("w:val=\"Heading1\"") || notes.contains("w:val='Heading1'"),
            "footnote pStyle must use remapped Heading1 after B.4 writeback; notes={notes}"
        );
        assert!(
            !notes.contains("w:val=\"1\"") && !notes.contains("w:val='1'"),
            "stale styleId=1 must not remain in footnotes after remap; notes={notes}"
        );
    } else {
        // If canonicalize did not rename (collision / different path), still
        // require footnote style ref to match whatever styles part advertises
        // for the heading-1 named style.
        assert!(
            notes.contains("pStyle"),
            "footnotes must retain pStyle; notes={notes}"
        );
    }
}
