// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M309 — basic_list × sd_1707-style short next (title + list heading)
//! vs long short-item list base. Unpacked Word oracle is pure-I all next
//! then pure-D all base (seq IIDDDDDDDDD), not M-CARRIER MIX of heading
//! with first base list item.

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

fn docx_from_paras(paras: &[(&str, Option<u32>)]) -> Vec<u8> {
    // Option ilvl: None = no numPr
    let mut body = String::new();
    for (text, ilvl) in paras {
        let ppr = if let Some(ilvl) = ilvl {
            format!(
                r#"<w:pPr>
                <w:pStyle w:val="ListParagraph"/>
                <w:numPr>
                  <w:ilvl w:val="{ilvl}"/>
                  <w:numId w:val="1"/>
                </w:numPr>
              </w:pPr>"#
            )
        } else {
            String::new()
        };
        body.push_str(&format!(
            r#"<w:p>{ppr}<w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#
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
        for (name, data) in [
            (
                "[Content_Types].xml",
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/></Types>"#.as_slice(),
            ),
            (
                "_rels/.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_slice(),
            ),
            (
                "word/_rels/document.xml.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/></Relationships>"#.as_slice(),
            ),
            ("word/styles.xml", styles.as_bytes()),
            ("word/numbering.xml", numbering.as_bytes()),
        ] {
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
fn basic_list_x_short_next_is_pure_id_not_carrier_mix() {
    // A = long short-item list base (basic_list shape).
    // B = title (no numPr) + list heading (numPr) — sd_1707 shape.
    let a = docx_from_paras(&[
        ("List item 1", Some(0)),
        ("List item 2", Some(0)),
        ("Indentation 1", Some(0)),
        ("Back", Some(0)),
        ("Numbered 1", Some(0)),
        ("Num 2", Some(0)),
        ("With indend 1", Some(0)),
        ("last", Some(0)),
    ]);
    let b = docx_from_paras(&[
        ("Minimal tracked changes fixture (candidate)", None),
        ("Heading. Body copy for repro", Some(0)),
    ]);
    let out = compare_documents_with_settings(&a, &b, &word_settings()).expect("compare");
    let xml = document_xml(&out);
    let paras = body_paragraphs(&xml);

    let heading = paras
        .iter()
        .find(|p| text_of(p).contains("Heading. Body copy for repro"))
        .expect("heading must appear");
    assert!(
        has_ins(heading) && !has_del(heading),
        "Word pure-I heading (not MIX with List item 1); got {heading}"
    );

    let pure_del = paras
        .iter()
        .any(|p| has_del(p) && !has_ins(p) && text_of(p).contains("List item 1"));
    assert!(
        pure_del,
        "expected pure-del of List item 1; paras={}",
        paras
            .iter()
            .enumerate()
            .map(|(i, p)| format!(
                "p{i}:ins={} del={} {:?}",
                has_ins(p),
                has_del(p),
                text_of(p).chars().take(50).collect::<String>()
            ))
            .collect::<Vec<_>>()
            .join(" | ")
    );
}
