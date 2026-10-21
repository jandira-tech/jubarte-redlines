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

/// Word's page-number frame: a header paragraph holding `PAGE` in a
/// `w:framePr` on the margin's right edge (#526).
const PAGE_FRAME: &str = r#"<w:p><w:pPr><w:framePr w:wrap="none" w:vAnchor="text" w:hAnchor="margin" w:xAlign="right" w:y="1"/></w:pPr><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p><w:p><w:r><w:t>Odd page header</w:t></w:r></w:p>"#;

/// A framed paragraph prints the frame in its `pN:` entry, in the schema's
/// attribute order, and the writer puts it back on the paragraph (#526).
#[test]
fn a_framed_page_number_keeps_its_frame() {
    let first = view(&with_header(PAGE_FRAME));
    let block = headers_block(&first);
    assert!(
        block.contains("frame: {wrap: none, hAnchor: margin, vAnchor: text, xAlign: right, y: 1}}"),
        "{first}"
    );
    assert!(block.contains("p1: {text: Odd page header}"), "{first}");
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(
        xml.contains(
            r#"<w:pPr><w:framePr w:wrap="none" w:hAnchor="margin" w:vAnchor="text" w:xAlign="right" w:y="1"/>"#
        ),
        "{xml}"
    );
    assert_eq!(xml.matches("<w:framePr").count(), 1, "{xml}");
    assert_eq!(view(&again), first);
}

/// The frame a paragraph lays out in is its style's under its own: the
/// view prints that one, and the writer writes it on the paragraph, which
/// keeps no style's frame (#526).
#[test]
fn a_style_s_frame_under_the_paragraph_s_own_is_the_one_printed() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="PageBox"><w:name w:val="Page Box"/><w:basedOn w:val="Normal"/><w:pPr><w:framePr w:h="363" w:wrap="around" w:hAnchor="page"/></w:pPr></w:style></w:styles>"#
    );
    let paragraphs = r#"<w:p><w:pPr><w:pStyle w:val="PageBox"/><w:framePr w:wrap="none" w:x="-20"/></w:pPr><w:r><w:t>7</w:t></w:r></w:p>"#;
    let xml = header(paragraphs);
    let docx = docx_with_sect(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &xml,
            },
            Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &styles,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let first = view(&docx);
    assert!(
        headers_block(&first).contains("frame: {h: 363, wrap: none, hAnchor: page, x: -20}"),
        "{first}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(
        xml.contains(r#"<w:framePr w:h="363" w:wrap="none" w:hAnchor="page" w:x="-20"/>"#),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

/// A frame the schema does not allow is no frame the writer writes: the
/// story entry does not read, as a border that does not read (#526).
#[test]
fn a_frame_value_the_schema_forbids_is_not_written() {
    let first = view(&with_header(PAGE_FRAME));
    for bad in ["wrap: sideways", "colour: red", "y: high"] {
        let edited = first.replace("wrap: none", bad);
        assert_ne!(edited, first);
        let docx = write(&edited);
        let xml = part_string(&docx, "word/header1.xml").unwrap_or_default();
        assert!(
            !xml.contains("sideways") && !xml.contains("colour") && !xml.contains("high"),
            "{bad}: {xml}"
        );
    }
}

/// Word's page-number footer: the number in a frame on the margin, the
/// text in the next paragraph, indented clear of it (#1422).
const FOOTER_FRAME: &str = r#"<w:p><w:pPr><w:framePr w:wrap="none" w:vAnchor="text" w:hAnchor="margin" w:y="1"/></w:pPr><w:r><w:t>1</w:t></w:r></w:p><w:p><w:pPr><w:ind w:firstLine="360"/></w:pPr><w:r><w:t>Footer text</w:t></w:r></w:p>"#;

/// A story paragraph's own indent prints as the id lines say it, in an
/// `indent:` map of its `pN:` entry, and comes back; an indent alone prints
/// the story paragraph by paragraph (#1422).
#[test]
fn a_story_paragraph_keeps_its_own_indent() {
    let first = view(&with_header(FOOTER_FRAME));
    assert!(
        headers_block(&first).contains("p1: {text: Footer text, indent: {first-line: 0.25in}}"),
        "{first}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(xml.contains(r#"<w:ind w:firstLine="360"/>"#), "{xml}");
    assert_eq!(view(&again), first);

    let sides = r#"<w:p><w:pPr><w:ind w:left="720" w:right="1440" w:hanging="360"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Indented</w:t></w:r></w:p>"#;
    let first = view(&with_header(sides));
    assert!(
        headers_block(&first).contains(
            "p0: {text: Indented, center, indent: {hanging: 0.25in, left: 0.5in, right: 1in}}"
        ),
        "{first}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/header1.xml").expect("a header part");
    assert!(
        xml.contains(
            r#"<w:ind w:left="720" w:right="1440" w:hanging="360"/><w:jc w:val="center"/>"#
        ),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

/// An indent that is no length, names a side the id lines do not, or is no
/// map does not read, and the story entry with it, as a border that does
/// not read (#1422).
#[test]
fn a_story_indent_that_is_no_length_does_not_read() {
    let first = view(&with_header(FOOTER_FRAME));
    for bad in [
        "indent: {first-line: wide}",
        "indent: {first-line: 25in}",
        "indent: {first-line: 0.25}",
        "indent: {start: 0.25in}",
        "indent: 0.25in",
    ] {
        let edited = first.replace("indent: {first-line: 0.25in}", bad);
        assert_ne!(edited, first, "{first}");
        let xml = part_string(&write(&edited), "word/header1.xml").unwrap_or_default();
        assert!(!xml.contains("<w:ind"), "{bad}: {xml}");
    }
}

/// `right` is the alignment and `indent: {right: 1in}` the indent: one
/// paragraph says both, and no key of its entry twice (#1422).
#[test]
fn a_story_paragraph_s_alignment_and_indent_keep_apart() {
    let both = r#"<w:p><w:pPr><w:ind w:right="1440"/><w:jc w:val="right"/></w:pPr><w:r><w:t>Both</w:t></w:r></w:p>"#;
    let first = view(&with_header(both));
    assert!(
        headers_block(&first).contains("p0: {text: Both, right, indent: {right: 1in}}"),
        "{first}"
    );
    let xml = part_string(&write(&first), "word/header1.xml").expect("a header part");
    assert!(
        xml.contains(r#"<w:ind w:right="1440"/><w:jc w:val="right"/>"#),
        "{xml}"
    );
}
