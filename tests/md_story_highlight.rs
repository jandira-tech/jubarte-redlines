// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A header or footer run keeps its character marks. Cyan and yellow
//! highlights print as `<mark class="hl-cyan">` and `<mark>`, and bold,
//! italic, underline, strike, code and superscript print as the same tags
//! the body uses. Converting the view back writes those properties on the
//! story's runs.

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with_sect, part_string};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const HEADER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        .filter(|l| !l.starts_with("source: ") && !l.starts_with("sha256: "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn write(markdown: &str) -> Vec<u8> {
    let docx = markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx;
    common::validity::assert_word_valid_package(&docx);
    docx
}

fn with_part(
    name: &str,
    root: &str,
    reference: &str,
    content_type: &str,
    rel_type: &str,
    paragraphs: &str,
) -> Vec<u8> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{root} xmlns:w="{W_NS}" xmlns:r="{R_NS}">{paragraphs}</w:{root}>"#
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name,
            content_type,
            rel_type,
            xml: &xml,
        }],
        &format!(r#"<w:{reference}Reference w:type="default" r:id="rIdX0"/>"#),
    )
}

fn block(view: &str, key: &str) -> String {
    view.lines()
        .skip_while(|line| *line != key)
        .take_while(|line| *line == key || line.starts_with(' '))
        .collect::<Vec<_>>()
        .join("\n")
}

const HEADER_RUNS: &str = r#"<w:p><w:r><w:rPr><w:highlight w:val="cyan"/></w:rPr><w:t>HDR</w:t></w:r><w:r><w:rPr><w:highlight w:val="yellow"/></w:rPr><w:t xml:space="preserve"> LIT</w:t></w:r><w:r><w:t xml:space="preserve"> end</w:t></w:r></w:p>"#;

#[test]
fn a_header_keeps_cyan_and_yellow_highlights() {
    let source = with_part(
        "word/header1.xml",
        "hdr",
        "header",
        HEADER,
        HEADER_REL,
        HEADER_RUNS,
    );
    let first = view(&source);
    let headers = block(&first, "headers:");
    assert!(headers.contains("hl-cyan"), "{headers}");
    assert!(headers.contains(">HDR</mark>"), "{headers}");
    assert!(headers.contains("<mark> LIT</mark>"), "{headers}");
    assert!(headers.contains("</mark> end"), "{headers}");
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(xml.contains(r#"<w:highlight w:val="cyan"/>"#), "{xml}");
    assert!(xml.contains(r#"<w:highlight w:val="yellow"/>"#), "{xml}");
    assert_eq!(xml.matches("<w:highlight ").count(), 2, "{xml}");
    assert!(xml.contains(">HDR<"), "{xml}");
    assert!(xml.contains("> LIT<"), "{xml}");
    assert!(xml.contains("> end<"), "{xml}");
    assert_eq!(view(&again), first);
}

const FOOTER_RUNS: &str = r#"<w:p><w:r><w:rPr><w:b/><w:i/><w:strike/><w:u w:val="single"/></w:rPr><w:t>Note</w:t></w:r><w:r><w:rPr><w:rStyle w:val="VerbatimChar"/></w:rPr><w:t xml:space="preserve"> ID</w:t></w:r><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:t>2</w:t></w:r></w:p>"#;

#[test]
fn a_footer_keeps_the_other_character_marks() {
    let source = with_part(
        "word/footer1.xml",
        "ftr",
        "footer",
        FOOTER,
        FOOTER_REL,
        FOOTER_RUNS,
    );
    let first = view(&source);
    let footers = block(&first, "footers:");
    assert!(
        footers.contains("<b><i><u><s>Note</s></u></i></b>"),
        "{footers}"
    );
    assert!(footers.contains("<code> ID</code>"), "{footers}");
    assert!(footers.contains("<sup>2</sup>"), "{footers}");
    let again = write(&first);
    let xml = part_string(&again, "word/footer1.xml").expect("a footer part");
    assert!(xml.contains("<w:b/>"), "{xml}");
    assert!(xml.contains("<w:i/>"), "{xml}");
    assert!(xml.contains("<w:strike/>"), "{xml}");
    assert!(xml.contains(r#"<w:u w:val="single"/>"#), "{xml}");
    assert!(xml.contains(r#"w:val="VerbatimChar""#), "{xml}");
    assert!(xml.contains(r#"w:val="superscript""#), "{xml}");
    assert!(xml.contains(">Note<"), "{xml}");
    assert!(xml.contains("> ID<"), "{xml}");
    assert!(xml.contains(">2<"), "{xml}");
    assert_eq!(view(&again), first);
}
