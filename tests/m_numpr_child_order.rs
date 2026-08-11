// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `w:numPr` children must be emitted in `CT_NumPr` sequence order.
//!
//! `wml.xsd` sequences `CT_NumPr` as `ilvl`, `numId`, `numberingChange`, `ins`.
//! `wml_order_elements_per_standard` sorts the children of `pPr`, `rPr`,
//! `tblPr`, `tcPr`, `tcBorders`, `tblBorders` and `pBdr` — but **not** `numPr`,
//! so a source that writes `numId` before `ilvl` is copied through in that
//! order and the output is schema-invalid.
//!
//! This is a passthrough of malformed input rather than an order the comparer
//! invents, and it is still ours to repair: the same Word-parity argument the
//! dangling-`numId` repair already makes (`document_comparer.rs`, "Word
//! synthesizes a default numbering definition … Word repairs it on open, so
//! match it at compare time") applies unchanged here.
//!
//! Oracle evidence: across the 504-document benchmark probe set, **10**
//! jubarte-rust outputs carry `numId` before `ilvl` (43 elements) and the Word
//! oracle for the same pairs carries **zero** — Word normalises the order when
//! it writes a comparison. `tools/validate-docx` rejects the affected outputs
//! with `Sch_UnexpectedElementContentExpectingComplex` on `w:ilvl`.

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
        ..WmlComparerSettings::default()
    }
}

/// A list paragraph whose `w:numPr` writes `numId` **before** `ilvl` — the
/// malformed shape the corpus sources ship and the comparer copies through.
fn docx_list(text: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:numPr><w:numId w:val="1"/><w:ilvl w:val="0"/></w:numPr></w:pPr>
      <w:r><w:t xml:space="preserve">{text}</w:t></w:r>
    </w:p>
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

/// Offsets of every `w:ilvl` / `w:numId` inside each `w:numPr`, in document
/// order. Reading offsets rather than counting occurrences keeps the assertion
/// about *order* and not about how many list paragraphs survived the compare.
fn numpr_child_orders(xml: &str) -> Vec<Vec<&'static str>> {
    let mut out = Vec::new();
    for seg in xml.split("<w:numPr>").skip(1) {
        let Some(end) = seg.find("</w:numPr>") else {
            continue;
        };
        let body = &seg[..end];
        let ilvl = body.find("<w:ilvl");
        let numid = body.find("<w:numId");
        let mut order: Vec<(usize, &'static str)> = Vec::new();
        if let Some(i) = ilvl {
            order.push((i, "ilvl"));
        }
        if let Some(i) = numid {
            order.push((i, "numId"));
        }
        order.sort_by_key(|(i, _)| *i);
        out.push(order.into_iter().map(|(_, n)| n).collect());
    }
    out
}

#[test]
fn numpr_emits_ilvl_before_numid() {
    let a = docx_list("First list item");
    let b = docx_list("Second list item");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let xml = document_xml(&out);

    let orders = numpr_child_orders(&xml);
    assert!(
        !orders.is_empty(),
        "expected the output to carry at least one w:numPr: {}",
        &xml[..xml.len().min(900)]
    );
    for order in &orders {
        if order.len() < 2 {
            continue;
        }
        assert_eq!(
            order,
            &vec!["ilvl", "numId"],
            "CT_NumPr sequences ilvl before numId; got {order:?} in: {}",
            &xml[..xml.len().min(900)]
        );
    }
}
