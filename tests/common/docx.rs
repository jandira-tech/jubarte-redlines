// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! In-memory DOCX builder for inspection/edit/render tests. Produces the
//! smallest Word-openable package: `[Content_Types].xml`, package rels,
//! `word/document.xml` and the optional related parts a test asks for.

use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
pub const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

/// Extra part: `(part name, content type, relationship type or "", xml)`.
pub struct Part<'a> {
    pub name: &'a str,
    pub content_type: &'a str,
    pub rel_type: &'a str,
    pub xml: &'a str,
}

/// A body paragraph: `<w:p><w:r><w:t>text</w:t></w:r></w:p>`.
pub fn para(text: &str) -> String {
    format!(
        r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
        esc(text)
    )
}

/// A run with optional bold/italic/highlight run properties.
pub fn run(text: &str, bold: bool, italic: bool, highlight: Option<&str>) -> String {
    let mut rpr = String::new();
    if bold {
        rpr.push_str("<w:b/>");
    }
    if italic {
        rpr.push_str("<w:i/>");
    }
    if let Some(h) = highlight {
        rpr.push_str(&format!(r#"<w:highlight w:val="{h}"/>"#));
    }
    let rpr = if rpr.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{rpr}</w:rPr>")
    };
    format!(
        r#"<w:r>{rpr}<w:t xml:space="preserve">{}</w:t></w:r>"#,
        esc(text)
    )
}

pub fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Build a DOCX whose body is `body_xml` (paragraphs/tables, no `w:body`
/// wrapper) followed by a default `w:sectPr`.
pub fn docx(body_xml: &str) -> Vec<u8> {
    docx_with(body_xml, &[])
}

/// Build a DOCX with extra related parts (styles, comments, headers, ...).
pub fn docx_with(body_xml: &str, extras: &[Part<'_>]) -> Vec<u8> {
    docx_with_sect(body_xml, extras, "")
}

/// [`docx_with`] whose final `w:sectPr` starts with `sect_refs`, e.g.
/// `<w:headerReference w:type="default" r:id="rIdX0"/>`: extra part `i`
/// gets relationship id `rIdX{i}`.
pub fn docx_with_sect(body_xml: &str, extras: &[Part<'_>], sect_refs: &str) -> Vec<u8> {
    docx_with_sect_pr(
        body_xml,
        extras,
        &format!(
            r#"<w:sectPr>{sect_refs}<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#
        ),
    )
}

/// [`docx_with`] whose body ends on `sect_pr`, the whole final `w:sectPr`.
pub fn docx_with_sect_pr(body_xml: &str, extras: &[Part<'_>], sect_pr: &str) -> Vec<u8> {
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="{W_NS}" xmlns:r="{R_NS}" xmlns:mc="{MC_NS}"><w:body>{body_xml}{sect_pr}</w:body></w:document>"#
    );
    let mut overrides = String::from(
        r#"<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
    );
    let mut doc_rels = String::new();
    for (i, part) in extras.iter().enumerate() {
        overrides.push_str(&format!(
            r#"<Override PartName="/{}" ContentType="{}"/>"#,
            part.name, part.content_type
        ));
        if !part.rel_type.is_empty() {
            let target = part.name.trim_start_matches("word/");
            doc_rels.push_str(&format!(
                r#"<Relationship Id="rIdX{i}" Type="{}" Target="{target}"/>"#,
                part.rel_type
            ));
        }
    }
    let content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>{overrides}</Types>"#
    );
    let package_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let document_rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{doc_rels}</Relationships>"#
    );
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    let mut put = |name: &str, data: &[u8]| {
        zip.start_file(name, opts).unwrap();
        zip.write_all(data).unwrap();
    };
    put("[Content_Types].xml", content_types.as_bytes());
    put("_rels/.rels", package_rels.as_bytes());
    put("word/document.xml", document.as_bytes());
    put("word/_rels/document.xml.rels", document_rels.as_bytes());
    for part in extras {
        put(part.name, part.xml.as_bytes());
    }
    zip.finish().unwrap().into_inner()
}

/// Read one part of a DOCX as a string.
pub fn part_string(docx: &[u8], name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(docx)).ok()?;
    let mut file = archive.by_name(name).ok()?;
    let mut out = String::new();
    std::io::Read::read_to_string(&mut file, &mut out).ok()?;
    Some(out)
}
