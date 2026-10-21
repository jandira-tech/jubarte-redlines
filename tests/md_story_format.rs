// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A header's or footer's paragraph keeps its own text colour and border.
//! Demo.docx's header is grey (`w:color 595959`) over a thin blue rule (its
//! paragraph's `w:pBdr` bottom border); the round trip printed it as body
//! text. A story whose paragraphs carry such formatting prints one `pN:`
//! entry per paragraph (Arthur, 2026-10-15); a colour that opens with a
//! digit is quoted, as YAML would read `000000` as the number 0:
//!
//! ```yaml
//! headers:
//!   default:
//!     id: header1
//!     p0: {text: "docen · canvas typesetting", center, color: "595959", border: bottom single 0.5pt 8496B0 space 4pt}
//! ```

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with_sect, docx_with_sect_pr, part_string};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const HEADER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

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

fn header(paragraphs: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}" xmlns:r="{R_NS}">{paragraphs}</w:hdr>"#
    )
}

/// A one-section docx whose default header is `paragraphs`.
fn with_header(paragraphs: &str) -> Vec<u8> {
    let xml = header(paragraphs);
    docx_with_sect(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER,
            rel_type: HEADER_REL,
            xml: &xml,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// The `headers:` block of a view, through the line before the next key.
fn headers_block(view: &str) -> String {
    view.lines()
        .skip_while(|l| *l != "headers:")
        .take_while(|l| *l == "headers:" || l.starts_with(' '))
        .collect::<Vec<_>>()
        .join("\n")
}

const DEMO: &str = r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:color="8496B0" w:sz="4" w:space="4"/></w:pBdr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:color w:val="595959"/></w:rPr><w:t xml:space="preserve">docen · canvas typesetting</w:t></w:r></w:p>"#;

#[test]
fn a_grey_ruled_header_prints_its_paragraph_and_comes_back() {
    let first = view(&with_header(DEMO));
    assert_eq!(
        headers_block(&first),
        "headers:\n  default:\n    id: header1\n    \
         p0: {text: docen · canvas typesetting, center, color: \"595959\", border: bottom single 0.5pt 8496B0 space 4pt}\n    \
         style: Normal",
        "{first}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(
        xml.contains(r#"<w:pBdr><w:bottom w:val="single" w:sz="4" w:space="4" w:color="8496B0"/></w:pBdr><w:jc w:val="center"/>"#),
        "{xml}"
    );
    assert!(
        xml.contains(r#"<w:r><w:rPr><w:color w:val="595959"/></w:rPr><w:t"#),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

#[test]
fn a_plain_header_keeps_its_one_line_entry() {
    let first = view(&with_header(
        r#"<w:p><w:pPr><w:pStyle w:val="Header"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Plain</w:t></w:r></w:p>"#,
    ));
    assert!(
        first.contains("  default: {id: header1, text: Plain, center}"),
        "{first}"
    );
}

#[test]
fn each_paragraph_keeps_its_own_alignment() {
    let first = view(&with_header(
        r#"<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:t>Left</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Header"/><w:jc w:val="right"/></w:pPr><w:r><w:t>Right</w:t></w:r></w:p>"#,
    ));
    assert_eq!(
        headers_block(&first),
        "headers:\n  default:\n    id: header1\n    p0: {text: Left}\n    p1: {text: Right, right}",
        "{first}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    let paragraphs: Vec<&str> = xml.split("<w:p>").skip(1).collect();
    assert!(!paragraphs[0].contains("<w:jc"), "{xml}");
    assert!(paragraphs[1].contains(r#"<w:jc w:val="right"/>"#), "{xml}");
    assert_eq!(view(&again), first);
}

#[test]
fn runs_of_different_colours_print_no_colour() {
    let first = view(&with_header(
        r#"<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:rPr><w:color w:val="FF0000"/></w:rPr><w:t>Red </w:t></w:r><w:r><w:rPr><w:color w:val="0000FF"/></w:rPr><w:t>blue</w:t></w:r></w:p>"#,
    ));
    assert!(
        first.contains("  default: {id: header1, text: Red blue}"),
        "{first}"
    );
}

#[test]
fn a_later_sections_formatted_header_comes_back() {
    let page = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#;
    let plain =
        header(r#"<w:p><w:pPr><w:pStyle w:val="Header"/></w:pPr><w:r><w:t>One</w:t></w:r></w:p>"#);
    let red = header(
        r#"<w:p><w:pPr><w:pStyle w:val="Header"/><w:pBdr><w:top w:val="double" w:sz="6" w:space="1"/></w:pBdr></w:pPr><w:r><w:rPr><w:color w:val="C00000"/></w:rPr><w:t>Two</w:t></w:r></w:p>"#,
    );
    let body = format!(
        r#"<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/>{page}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p><w:p><w:r><w:t>Two</w:t></w:r></w:p>"#
    );
    let source = docx_with_sect_pr(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &plain,
            },
            Part {
                name: "word/header2.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &red,
            },
        ],
        &format!(
            r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/>{page}</w:sectPr>"#
        ),
    );
    let first = view(&source);
    assert!(
        first.contains(
            "headers: {default: {id: header2, p0: {text: Two, color: C00000, border: top double 0.75pt space 1pt}}}"
        ),
        "{first}"
    );
    let again = write(&first);
    assert_eq!(view(&again), first);
}

#[test]
fn a_stated_left_or_justified_paragraph_keeps_its_alignment() {
    // The Header style may centre its paragraphs: a paragraph that states
    // `left` or `both` itself must say so, or it comes back centred.
    let source = with_header(
        r#"<w:p><w:pPr><w:jc w:val="left"/></w:pPr><w:r><w:t>Left</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:t>Justified</w:t></w:r></w:p><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Centred</w:t></w:r></w:p>"#,
    );
    let first = view(&source);
    let block = headers_block(&first);
    assert!(block.contains(r#"p0: {text: Left, left}"#), "{block}");
    assert!(
        block.contains(r#"p1: {text: Justified, justify}"#),
        "{block}"
    );
    assert!(block.contains(r#"p2: {text: Centred, center}"#), "{block}");
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").unwrap();
    let paragraphs: Vec<&str> = xml.split("<w:p>").skip(1).collect();
    assert!(paragraphs[0].contains(r#"<w:jc w:val="left"/>"#), "{xml}");
    assert!(paragraphs[1].contains(r#"<w:jc w:val="both"/>"#), "{xml}");
    assert!(paragraphs[2].contains(r#"<w:jc w:val="center"/>"#), "{xml}");
    assert_eq!(view(&again), first);
}
