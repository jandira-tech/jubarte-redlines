// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A glossary is not a story, so compare, accept and reject used to copy
//! `word/glossary/document.xml` unchanged. The three shapes the schema
//! refuses in a story are repaired there too. A building block's text and
//! its widths stay, and a glossary those repairs do not touch stays byte
//! for byte.

mod common;

use common::docx::{Part, W_NS, docx_with, para, part_string};
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};

const GLOSSARY: &str = "word/glossary/document.xml";
const GLOSSARY_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.glossary+xml";
const GLOSSARY_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/glossaryDocument";

const SHADE: &str = r#"<w:shd w:val="clear" w:color="auto" w:fill="" w:themeFill="accent1"/>"#;
const GRID: &str = r#"<w:gridCol w:w="2400" w:type="dxa"/>"#;
const TABLE_MARGINS: &str = r#"<w:right w:w="82" w:type="dxa"/><w:bottom w:w="83" w:type="dxa"/><w:left w:w="80" w:type="dxa"/><w:top w:w="81" w:type="dxa"/>"#;
const CELL_MARGINS: &str = r#"<w:end w:w="91" w:type="dxa"/><w:top w:w="92" w:type="dxa"/><w:start w:w="90" w:type="dxa"/>"#;

fn glossary_document() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:glossaryDocument xmlns:w="{W_NS}"><w:docParts><w:docPart><w:docPartPr><w:name w:val="Block"/></w:docPartPr><w:docPartBody><w:tbl><w:tblPr><w:tblW w:w="2400" w:type="dxa"/><w:tblCellMar>{TABLE_MARGINS}</w:tblCellMar></w:tblPr><w:tblGrid>{GRID}</w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/>{SHADE}<w:tcMar>{CELL_MARGINS}</w:tcMar></w:tcPr><w:p><w:r><w:t>Block.</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:docPartBody></w:docPart></w:docParts></w:glossaryDocument>"#
    )
}

fn package(body: &str, glossary: &str) -> Vec<u8> {
    docx_with(
        body,
        &[Part {
            name: GLOSSARY,
            content_type: GLOSSARY_CT,
            rel_type: GLOSSARY_REL,
            xml: glossary,
        }],
    )
}

fn tags<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(&open) {
        let tail = &rest[at..];
        let next = tail.as_bytes().get(open.len()).copied();
        let end = tail.find('>').expect("tag closes") + 1;
        if matches!(next, Some(b' ' | b'/' | b'>')) {
            out.push(&tail[..end]);
        }
        rest = &tail[end..];
    }
    out
}

fn margin_orders(xml: &str, container: &str) -> Vec<Vec<String>> {
    let open = format!("<w:{container}>");
    let close = format!("</w:{container}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(&open) {
        let inner = &rest[at + open.len()..];
        let end = inner.find(&close).expect("margins close");
        out.push(
            inner[..end]
                .split("<w:")
                .skip(1)
                .map(|child| {
                    child
                        .split([' ', '/', '>'])
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
                .collect(),
        );
        rest = &inner[end..];
    }
    out
}

#[test]
fn glossary_write_repairs_the_three_schema_invalid_shapes() {
    let glossary = glossary_document();
    let original = package(&para("Old words."), &glossary);
    let revised = package(&para("New words."), &glossary);
    let source = part_string(&original, GLOSSARY).expect("source glossary");
    assert!(
        tags(&source, "w:shd")
            .iter()
            .any(|tag| tag.contains(r#"w:fill="""#)),
        "{source}"
    );
    assert!(
        tags(&source, "w:gridCol")
            .iter()
            .any(|tag| tag.contains("w:type=")),
        "{source}"
    );
    assert_eq!(
        margin_orders(&source, "tblCellMar"),
        vec![vec!["right", "bottom", "left", "top"]]
    );

    let outputs = [
        (
            "compare",
            compare_documents(&original, &revised, "Redline").expect("compare"),
        ),
        ("accept", accept_revisions(&original).expect("accept")),
        ("reject", reject_revisions(&original).expect("reject")),
    ];
    for (name, out) in outputs {
        let xml = part_string(&out, GLOSSARY).unwrap_or_default();
        assert!(xml.contains("Block."), "{name}: {xml}");
        let shading = tags(&xml, "w:shd");
        assert_eq!(shading.len(), 1, "{name}: {xml}");
        assert!(
            !shading[0].contains("w:fill")
                && shading[0].contains(r#"w:val="clear""#)
                && shading[0].contains(r#"w:color="auto""#)
                && shading[0].contains(r#"w:themeFill="accent1""#),
            "{name}: {xml}"
        );
        let grid = tags(&xml, "w:gridCol");
        assert_eq!(grid.len(), 1, "{name}: {xml}");
        assert!(
            !grid[0].contains("w:type") && grid[0].contains(r#"w:w="2400""#),
            "{name}: {xml}"
        );
        assert!(
            xml.contains(r#"<w:tblW w:w="2400" w:type="dxa""#),
            "{name}: {xml}"
        );
        assert_eq!(
            margin_orders(&xml, "tblCellMar"),
            vec![vec!["top", "left", "bottom", "right"]],
            "{name}: {xml}"
        );
        assert_eq!(
            margin_orders(&xml, "tcMar"),
            vec![vec!["top", "start", "end"]],
            "{name}: {xml}"
        );
        for (side, width) in [("left", 80), ("top", 81), ("right", 82), ("bottom", 83)] {
            let mark = format!(r#"<w:{side} w:w="{width}" w:type="dxa""#);
            assert!(xml.contains(&mark), "{name} {side}: {xml}");
        }
        for (side, width) in [("start", 90), ("end", 91), ("top", 92)] {
            let mark = format!(r#"<w:{side} w:w="{width}" w:type="dxa""#);
            assert!(xml.contains(&mark), "{name} {side}: {xml}");
        }
    }
}

#[test]
fn a_valid_glossary_with_comments_in_its_properties_stays_byte_for_byte() {
    // Compare used the comparison canonicalizer, which drops comments and
    // whitespace inside property containers: a glossary with none of the
    // three shapes was still rewritten (#2117 review).
    let glossary = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:glossaryDocument xmlns:w="{W_NS}"><w:docParts><w:docPart><w:docPartPr><w:name w:val="Block"/></w:docPartPr><w:docPartBody><w:p><w:pPr> <!-- kept --> <w:jc w:val="left"/></w:pPr><w:r><w:rPr><!-- also kept --><w:b/></w:rPr><w:t>Block.</w:t></w:r></w:p></w:docPartBody></w:docPart></w:docParts></w:glossaryDocument>"#
    );
    let original = package(&para("Old words."), &glossary);
    let revised = package(&para("New words."), &glossary);
    let source = part_string(&original, GLOSSARY).expect("source glossary");
    let outputs = [
        (
            "compare",
            compare_documents(&original, &revised, "Redline").expect("compare"),
        ),
        ("accept", accept_revisions(&original).expect("accept")),
        ("reject", reject_revisions(&original).expect("reject")),
    ];
    for (name, out) in outputs {
        let xml = part_string(&out, GLOSSARY).unwrap_or_default();
        assert_eq!(xml, source, "{name}");
    }
}
