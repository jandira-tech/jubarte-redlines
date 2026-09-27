// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Both documents write the same list under `numId` 1, but the original's
//! definition draws a circle bullet and the revised one a disc. The text is
//! unchanged. Word moves the paragraphs to a fresh `w:num` that carries the
//! revised definition and records the original `numId` in a `w:pPrChange`, so
//! the redline shows the circle struck and the disc inserted. Before this fix,
//! unchanged paragraphs kept `numId` 1, which resolves to the original's
//! circle, and no change was recorded.

use std::io::{Cursor, Read, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx_bullets(bullet: &str) -> Vec<u8> {
    docx_bullets_with(bullet, "", &[])
}

/// `tail` is appended to the last item; `extra` = (part, content type, rel type, xml).
fn docx_bullets_with(bullet: &str, tail: &str, extra: &[(&str, &str, &str, &str)]) -> Vec<u8> {
    let items: String = ["First", "Second"]
        .iter()
        .map(|t| {
            let tail = if *t == "Second" { tail } else { "" };
            format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{t}</w:t></w:r>{tail}</w:p>"#
            )
        })
        .collect();
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{items}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let numbering = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="{bullet}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
    );
    let overrides: String = extra
        .iter()
        .map(|(name, ct, _, _)| format!(r#"<Override PartName="/{name}" ContentType="{ct}"/>"#))
        .collect();
    let ct = format!(
        r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>{overrides}</Types>"#
    );
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let extra_rels: String = extra
        .iter()
        .enumerate()
        .map(|(i, (name, _, rel, _))| {
            let target = name.trim_start_matches("word/");
            format!(r#"<Relationship Id="rIdX{i}" Type="{rel}" Target="{target}"/>"#)
        })
        .collect();
    let doc_rels = format!(
        r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>{extra_rels}</Relationships>"#
    );
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        let parts = [
            ("[Content_Types].xml", ct.as_bytes()),
            ("_rels/.rels", &root_rels[..]),
            ("word/_rels/document.xml.rels", doc_rels.as_bytes()),
            ("word/document.xml", doc.as_bytes()),
            ("word/numbering.xml", numbering.as_bytes()),
        ];
        let extra_parts = extra
            .iter()
            .map(|(name, _, _, xml)| (*name, xml.as_bytes()));
        for (name, body) in parts.into_iter().chain(extra_parts) {
            z.start_file(name, opt).unwrap();
            z.write_all(body).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut s = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}

#[test]
fn unchanged_items_take_the_revised_list_with_a_ppr_change() {
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out =
        compare_documents_with_settings(&docx_bullets("o"), &docx_bullets("\u{2022}"), &settings)
            .unwrap();
    let doc = part(&out, "word/document.xml");
    let first = doc.split("</w:p>").next().unwrap();
    let live = first
        .split("<w:pPrChange")
        .next()
        .unwrap()
        .split("<w:numId w:val=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("live numId");
    assert_ne!(
        live, "1",
        "unchanged item still points at the original list: {first}"
    );
    let old = first
        .split("<w:pPrChange")
        .nth(1)
        .unwrap_or_else(|| panic!("no pPrChange: {first}"));
    assert!(
        old.contains("<w:numId w:val=\"1\""),
        "pPrChange keeps the original list: {first}"
    );
    let numbering = part(&out, "word/numbering.xml");
    let num = numbering
        .split(&format!("<w:num w:numId=\"{live}\""))
        .nth(1)
        .expect("live num defined");
    let abs = num
        .split("<w:abstractNumId w:val=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let def = numbering
        .split(&format!("w:abstractNumId=\"{abs}\""))
        .nth(1)
        .unwrap()
        .split("</w:abstractNum>")
        .next()
        .unwrap();
    assert!(
        def.contains("\u{2022}"),
        "live list should draw the revised disc: {def}"
    );
}

/// The recorded list change takes a revision id no comment uses, including
/// a point comment (a `w:commentReference` with no range). The point comment
/// used to be dropped, and the first change then took its id 1.
#[test]
fn list_change_ids_skip_point_comment_ids() {
    let settings = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let comments = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="1" w:author="R" w:initials="R"><w:p><w:r><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#;
    let extra = [(
        "word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        comments,
    )];
    let tail = r#"<w:r><w:commentReference w:id="1"/></w:r>"#;
    let out = compare_documents_with_settings(
        &docx_bullets_with("o", tail, &extra),
        &docx_bullets_with("\u{2022}", tail, &extra),
        &settings,
    )
    .unwrap();
    let doc = part(&out, "word/document.xml");
    assert!(doc.contains("<w:commentReference w:id=\"1\""), "{doc}");
    let change_ids: Vec<&str> = doc
        .split("<w:pPrChange ")
        .skip(1)
        .filter_map(|c| c.split("w:id=\"").nth(1)?.split('"').next())
        .collect();
    assert!(!change_ids.is_empty(), "no pPrChange: {doc}");
    assert!(
        !change_ids.contains(&"1"),
        "a pPrChange reuses comment 1's id: {change_ids:?}"
    );
}
