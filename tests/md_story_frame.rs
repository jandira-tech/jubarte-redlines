// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A header's or footer's paragraph keeps its frame (`w:framePr`): the
//! classic right-aligned PAGE frame (`hAnchor=margin, vAnchor=text,
//! wrap=none, xAlign=right, y=1`) and the footer mirror
//! (`hAnchor=margin, vAnchor=text, wrap=none, y=1`). A frame prints as a
//! `frame:` key in the paragraph-by-paragraph grammar:
//!
//! ```yaml
//! footers:
//!   default:
//!     id: footer2
//!     p0: {text: "{PAGE}1{/}", frame: "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)"}
//!     p1: {text: ""}
//! ```
//!
//! A `p0:` whose `frame:` is `(attr=val, attr=val, …)` produces a paragraph
//! with a `<w:framePr>` element carrying the same attributes in schema
//! order (`w`, `h`, `hRule`, `hAnchor`, `vAnchor`, `x`, `y`, `xAlign`,
//! `yAlign`, `wrap`).

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with_sect_pr, part_string};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

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

fn footer(paragraphs: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="{W_NS}" xmlns:r="{R_NS}">{paragraphs}</w:ftr>"#
    )
}

fn with_footer(paragraphs: &str) -> Vec<u8> {
    let xml = footer(paragraphs);
    docx_with_sect_pr(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/footer1.xml",
            content_type: FOOTER,
            rel_type: FOOTER_REL,
            xml: &xml,
        }],
        r#"<w:sectPr><w:footerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#,
    )
}

fn footers_block(view: &str) -> String {
    view.lines()
        .skip_while(|l| *l != "footers:")
        .take_while(|l| *l == "footers:" || l.starts_with(' '))
        .collect::<Vec<_>>()
        .join("\n")
}

const PAGE_FRAME: &str = r#"<w:p><w:pPr><w:pStyle w:val="Footer"/><w:framePr w:wrap="none" w:vAnchor="text" w:hAnchor="margin" w:xAlign="right" w:y="1"/></w:pPr><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Footer"/></w:pPr></w:p>"#;

#[test]
fn a_right_aligned_page_frame_prints_its_frame_and_comes_back() {
    let first = view(&with_footer(PAGE_FRAME));
    let block = footers_block(&first);
    assert!(
        block.contains(
            r#"p0: {text: "{PAGE}1{/}", frame: "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)"}"#
        ),
        "{block}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/footer1.xml").expect("a footer part");
    assert!(
        xml.contains(
            r#"<w:framePr w:hAnchor="margin" w:vAnchor="text" w:y="1" w:xAlign="right" w:wrap="none"/>"#
        ),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

#[test]
fn a_left_anchored_page_frame_omits_xalign() {
    let first = view(&with_footer(
        r#"<w:p><w:pPr><w:pStyle w:val="Footer"/><w:framePr w:wrap="none" w:vAnchor="text" w:hAnchor="margin" w:y="1"/></w:pPr><w:r><w:t>2</w:t></w:r></w:p>"#,
    ));
    let block = footers_block(&first);
    assert!(
        block.contains(r#"p0: {text: "2", frame: "(hAnchor=margin,vAnchor=text,wrap=none,y=1)"}"#),
        "{block}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/footer1.xml").expect("a footer part");
    assert!(
        xml.contains(r#"<w:framePr w:hAnchor="margin" w:vAnchor="text" w:y="1" w:wrap="none"/>"#),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";

/// The paragraph's own frame over its style's: the Footer style's `w:h`
/// joins the paragraph's own attributes, as `w:framePr` merges attribute
/// by attribute.
#[test]
fn a_style_frame_merges_with_the_paragraphs_own() {
    let styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/><w:pPr><w:framePr w:h="334"/></w:pPr></w:style></w:styles>"#
    );
    let footer_xml = footer(
        r#"<w:p><w:pPr><w:pStyle w:val="Footer"/><w:framePr w:wrap="none" w:vAnchor="text" w:hAnchor="margin" w:y="1"/></w:pPr><w:r><w:t>2</w:t></w:r></w:p>"#,
    );
    let docx = docx_with_sect_pr(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: &styles,
            },
            Part {
                name: "word/footer1.xml",
                content_type: FOOTER,
                rel_type: FOOTER_REL,
                xml: &footer_xml,
            },
        ],
        r#"<w:sectPr><w:footerReference w:type="default" r:id="rIdX1"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#,
    );
    let first = view(&docx);
    let block = footers_block(&first);
    assert!(
        block.contains(
            r#"p0: {text: "2", frame: "(h=334,hAnchor=margin,vAnchor=text,wrap=none,y=1)"}"#
        ),
        "{block}"
    );
    let again = write(&first);
    let xml = part_string(&again, "word/footer1.xml").expect("a footer part");
    assert!(
        xml.contains(
            r#"<w:framePr w:h="334" w:hAnchor="margin" w:vAnchor="text" w:y="1" w:wrap="none"/>"#
        ),
        "{xml}"
    );
    assert_eq!(view(&again), first);
}

/// A `frame:` that does not read drops the whole story entry, like an
/// unreadable `border:`, and the warning says so.
#[test]
fn an_unreadable_frame_drops_the_story_entry() {
    let markdown = "---\nview: tracked\nfooters:\n  default:\n    id: footer2\n    p0: {text: \"1\", frame: \"(hAnchor)\"}\n---\n\nBody\n";
    let written = markdown_to_docx(markdown, &DocxOptions::default()).expect("the Markdown reads");
    assert!(
        written
            .warnings
            .iter()
            .any(|w| w.contains("footers: default does not read")),
        "{:?}",
        written.warnings
    );
    assert!(
        part_string(&written.docx, "word/footer2.xml").is_none(),
        "no footer is written for a dropped story"
    );
}
