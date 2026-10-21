// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A `.rels` or `[Content_Types].xml` written with explicit closes
//! (`<Relationship …></Relationship>`) must read like one written with
//! empty elements. rdocx-opc's parsers read `Event::Empty` only, so the
//! bench corpus document 37c6c62345 (its footer relationship written that
//! way) converted without its footer and scored 49 against Word's PDF.

mod common;

use std::io::{Cursor, Read, Write};

use common::docx::{Part, docx_with_sect};
use jubarte::convert::{PdfOptions, docx_render_report};
use jubarte::opc::PartFs;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

/// The package with every empty `Relationship`, `Default` and `Override`
/// element rewritten with an explicit close.
fn with_explicit_closes(docx: &[u8]) -> Vec<u8> {
    let mut zip = ZipArchive::new(Cursor::new(docx)).expect("zip");
    let mut out = ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).expect("entry");
        let name = file.name().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).expect("read");
        if name.ends_with(".rels") || name == "[Content_Types].xml" {
            let mut text = String::from_utf8(data).expect("utf8");
            for tag in ["Relationship", "Default", "Override"] {
                let open = format!("<{tag} ");
                let mut folded = String::new();
                let mut rest = text.as_str();
                while let Some(at) = rest.find(&open) {
                    let end = at + rest[at..].find("/>").expect("empty element");
                    folded.push_str(&rest[..end]);
                    folded.push_str(&format!("></{tag}>"));
                    rest = &rest[end + 2..];
                }
                folded.push_str(rest);
                text = folded;
            }
            data = text.into_bytes();
        }
        out.start_file(name, SimpleFileOptions::default())
            .expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

fn footer_docx() -> Vec<u8> {
    let footer = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="{W_NS}"><w:p><w:r><w:t>FooterInk</w:t></w:r></w:p></w:ftr>"#
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body text.</w:t></w:r></w:p>",
        &[Part {
            name: "word/footer1.xml",
            content_type: FOOTER_CT,
            rel_type: FOOTER_REL,
            xml: &footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn explicitly_closed_relationships_are_read() {
    let plain = footer_docx();
    let closed = with_explicit_closes(&plain);
    assert!(
        String::from_utf8_lossy(&closed).contains("</Relationship>") || closed != plain,
        "the fixture rewrote its rels"
    );
    let pkg = PartFs::open(&closed).expect("opens");
    let rels = pkg
        .read_rels_for("word/document.xml")
        .expect("document relationships");
    assert!(
        rels.items.iter().any(|r| r.rel_type == FOOTER_REL),
        "the footer relationship is read: {:?}",
        rels.items.iter().map(|r| &r.target).collect::<Vec<_>>()
    );
    assert!(
        pkg.content_type_for("word/footer1.xml")
            .is_some_and(|ct| ct == FOOTER_CT),
        "the footer's content-type override is read"
    );
}

#[test]
fn a_footer_named_by_an_explicitly_closed_relationship_is_painted() {
    let closed = with_explicit_closes(&footer_docx());
    let report = docx_render_report(&closed, PdfOptions::default()).expect("report");
    assert!(
        report.pages[0].text.contains("FooterInk"),
        "the footer is painted: {:?}",
        report.pages[0].text
    );
}
