// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M308 — unrelated list wholesale (broken_list × multiple_nodes_in_list):
//! Word pure-I all B list items then pure-D all A list items. M-CARRIER
//! must NOT fuse B's last item into a MIX with A's first list item
//! (that drop scores ~52 vs Word's pure-I/D shape).
//!
//! Corpus exhibit: super_editor__broken_list_missing_items ×
//! super_editor__multiple_nodes_in_list (restored pin ~51.67).

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn word_settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn docx_list(items: &[(&str, u32)]) -> Vec<u8> {
    // items: (text, ilvl). All share numId=1.
    let mut body = String::new();
    for (text, ilvl) in items {
        body.push_str(&format!(
            r#"<w:p>
              <w:pPr>
                <w:pStyle w:val="ListParagraph"/>
                <w:numPr>
                  <w:ilvl w:val="{ilvl}"/>
                  <w:numId w:val="1"/>
                </w:numPr>
              </w:pPr>
              <w:r><w:t xml:space="preserve">{text}</w:t></w:r>
            </w:p>"#
        ));
    }
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body>
</w:document>"#
    );
    let numbering = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:abstractNum w:abstractNumId="0">
    <w:multiLevelType w:val="hybridMultilevel"/>
    <w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/>
      <w:lvlText w:val="·"/><w:lvlJc w:val="left"/></w:lvl>
  </w:abstractNum>
  <w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
</w:numbering>"#;
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="ListParagraph">
    <w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Normal" w:default="1">
    <w:name w:val="Normal"/>
  </w:style>
</w:styles>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        let parts: [(&str, &[u8]); 5] = [
            (
                "[Content_Types].xml",
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/></Types>"#,
            ),
            (
                "_rels/.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            ),
            (
                "word/_rels/document.xml.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/></Relationships>"#,
            ),
            ("word/styles.xml", styles.as_bytes()),
            ("word/numbering.xml", numbering.as_bytes()),
        ];
        for (name, data) in parts {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
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

fn body_paragraphs(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end = after
            .find("</w:p>")
            .map(|i| i + "</w:p>".len())
            .unwrap_or(after.len());
        out.push(after[..end].to_string());
        rest = &after[end..];
    }
    out
}

fn has_ins(p: &str) -> bool {
    p.contains("<w:ins")
}
fn has_del(p: &str) -> bool {
    p.contains("<w:del")
}
fn text_of(p: &str) -> String {
    let mut t = String::new();
    for tag in ["w:t", "w:delText"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut rest = p;
        while let Some(i) = rest.find(&open) {
            let after = &rest[i..];
            let Some(gt) = after.find('>') else { break };
            let content = &after[gt + 1..];
            let Some(end) = content.find(&close) else {
                break;
            };
            t.push_str(&content[..end]);
            rest = &content[end..];
        }
    }
    t
}

#[test]
fn unrelated_list_wholesale_is_pure_id_not_carrier_mix() {
    // A: longer broken list (5 contentful items). B: short unrelated list (2).
    let a = docx_list(&[
        ("Item 1. Text 1.", 0),
        ("Item 2. Text 2", 0),
        ("Item 3.", 0),
        ("Sub a. text", 1),
        ("Sub b.", 1),
        ("First shown item. Text", 0),
        ("Shown 2. Text", 0),
    ]);
    let b = docx_list(&[("Onetestafter space", 0), ("TWO", 0)]);
    let out = compare_documents_with_settings(&a, &b, &word_settings()).expect("compare");
    let xml = document_xml(&out);
    let paras = body_paragraphs(&xml);

    // B's "TWO" must appear as pure-ins (Word), not MIX with a deleted A item.
    let two = paras
        .iter()
        .find(|p| text_of(p).contains("TWO"))
        .expect("TWO must appear in output");
    assert!(
        has_ins(two) && !has_del(two),
        "TWO must be pure-ins (Word pure-I/D wholesale), not MIX carrier; got {two}"
    );

    // First B item also pure-ins.
    let one = paras
        .iter()
        .find(|p| text_of(p).contains("Onetestafter"))
        .expect("first B item");
    assert!(
        has_ins(one) && !has_del(one),
        "first B list item pure-ins: {one}"
    );

    // At least one pure-del residual carrying A's list text.
    let pure_del_a = paras.iter().any(|p| {
        has_del(p) && !has_ins(p) && (text_of(p).contains("Item 1") || text_of(p).contains("Shown"))
    });
    assert!(
        pure_del_a,
        "expected pure-del residual of A's list content; paras={}",
        paras
            .iter()
            .enumerate()
            .map(|(i, p)| format!(
                "p{i}:ins={} del={} {:?}",
                has_ins(p),
                has_del(p),
                text_of(p).chars().take(40).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join(" | ")
    );
}

#[test]
fn plain_demo_x_catalog_still_mixes_body() {
    // Guard: M307 shape must not break when we gate list wholesale pure-I/D.
    // Plain (non-list) 3×2 demo×catalog still wants MIX on the body.
    fn plain(paras: &[&str]) -> Vec<u8> {
        let mut body = String::new();
        for t in paras {
            body.push_str(&format!(
                r#"<w:p><w:r><w:t xml:space="preserve">{t}</w:t></w:r></w:p>"#
            ));
        }
        let doc = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr/></w:body></w:document>"#
        );
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = ZipWriter::new(&mut buf);
            let opt = SimpleFileOptions::default();
            z.start_file("[Content_Types].xml", opt).unwrap();
            z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
            z.start_file("_rels/.rels", opt).unwrap();
            z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
            z.start_file("word/document.xml", opt).unwrap();
            z.write_all(doc.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    let a = plain(&[
        "Bold Underline Highlight Demo",
        "This document shows bold underline highlight.",
        "Critical review annotations.",
    ]);
    let b = plain(&["Book Catalog", "Title Author Genre The Great Gatsby"]);
    let out = compare_documents_with_settings(&a, &b, &word_settings()).expect("compare");
    let xml = document_xml(&out);
    let paras = body_paragraphs(&xml);
    let mix = paras.iter().any(|p| has_ins(p) && has_del(p));
    assert!(
        mix,
        "plain demo×catalog must keep MIX body (M307); got only pure-I/D"
    );
}
