// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `w:tblGridChange` carries `w:id` only — never `w:author` / `w:date`.
//!
//! `CT_TblGridChange` is the one revision-history element that does **not**
//! extend `CT_TrackChange`. The repo's own schema oracle
//! (`tests/data/wml_main_schema.json`) states it exactly:
//!
//! ```text
//! w:CT_TblGridChange/w:tblGridChange  ['w:id']
//! w:CT_TblPrChange/w:tblPrChange      ['w:author', 'w:date', 'w16du:dateUtc', 'w:id']
//! ```
//!
//! `produce.rs` builds `tblGridChange` by copying the `tblPrChange` block a few
//! lines above it, author and date included. `tools/validate-docx` rejects the
//! result with `Sch_UndeclaredAttribute` on both attributes — the single
//! largest defect our output introduces that no source document carries
//! (104 occurrences of each across the 504-document probe set).
//!
//! The sibling `tblPrChange` assertion is deliberate: the fix must not
//! over-reach and strip author/date from the change elements that do declare
//! them.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        merge_replaced_paragraphs: true,
        detect_format_changes: true,
        ..WmlComparerSettings::default()
    }
}

/// A one-row two-column table whose grid column widths differ between the two
/// sides — the input that makes `produce.rs` emit a `w:tblGridChange`.
fn docx_table(widths: (u32, u32), text: &str) -> Vec<u8> {
    let (w1, w2) = widths;
    let total = w1 + w2;
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:tbl>
      <w:tblPr><w:tblW w:w="{total}" w:type="dxa"/></w:tblPr>
      <w:tblGrid><w:gridCol w:w="{w1}"/><w:gridCol w:w="{w2}"/></w:tblGrid>
      <w:tr>
        <w:tc><w:tcPr><w:tcW w:w="{w1}" w:type="dxa"/></w:tcPr>
          <w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p></w:tc>
        <w:tc><w:tcPr><w:tcW w:w="{w2}" w:type="dxa"/></w:tcPr>
          <w:p><w:r><w:t xml:space="preserve">cell two</w:t></w:r></w:p></w:tc>
      </w:tr>
    </w:tbl>
    <w:p><w:r><w:t xml:space="preserve">after the table</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(ct).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(root_rels).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// The open tags of every `<w:name ...>` in document order.
fn open_tags<'a>(xml: &'a str, name: &str) -> Vec<&'a str> {
    let needle = format!("<w:{name} ");
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(i) = xml[from..].find(&needle) {
        let start = from + i;
        let Some(end) = xml[start..].find('>') else {
            break;
        };
        out.push(&xml[start..start + end]);
        from = start + end;
    }
    out
}

#[test]
fn tblgridchange_carries_id_only() {
    let a = docx_table((4675, 4675), "cell one");
    let b = docx_table((3000, 6350), "cell one changed");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let xml = document_xml(&out);

    let grid_changes = open_tags(&xml, "tblGridChange");
    assert!(
        !grid_changes.is_empty(),
        "expected a w:tblGridChange for a table whose grid widths changed: {}",
        &xml[..xml.len().min(1200)]
    );
    for tag in &grid_changes {
        assert!(
            tag.contains("w:id="),
            "w:tblGridChange must carry w:id: {tag}"
        );
        assert!(
            !tag.contains("w:author="),
            "CT_TblGridChange does not declare w:author: {tag}"
        );
        assert!(
            !tag.contains("w:date="),
            "CT_TblGridChange does not declare w:date: {tag}"
        );
    }
}

#[test]
fn sibling_tblprchange_keeps_author_and_date() {
    let a = docx_table((4675, 4675), "cell one");
    let b = docx_table((3000, 6350), "cell one changed");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let xml = document_xml(&out);

    // CT_TblPrChange DOES extend CT_TrackChange — the fix must not strip it.
    for tag in open_tags(&xml, "tblPrChange") {
        assert!(
            tag.contains("w:author=") && tag.contains("w:date="),
            "CT_TblPrChange declares w:author and w:date; they must survive: {tag}"
        );
    }
}
