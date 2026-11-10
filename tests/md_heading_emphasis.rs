// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A heading keeps bold and italic that belong to its runs. Bold or italic
//! that only the heading style supplies stays the style, so a plain heading
//! does not grow stars.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

fn written(markdown: &str) -> Vec<u8> {
    markdown_to_docx(markdown, &DocxOptions::default())
        .expect("the Markdown reads")
        .docx
}

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
        .filter(|line| !line.starts_with("source: ") && !line.starts_with("sha256: "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn paragraph<'a>(xml: &'a str, text: &str) -> &'a str {
    xml.split("</w:p>")
        .find(|paragraph| paragraph.contains(&format!(">{text}<")))
        .unwrap_or("")
}

#[test]
fn a_heading_keeps_a_runs_own_bold_and_italic() {
    let source = "\
# **Architectural Analysis**

## **Executive Summary**

### *Lean title*

# Plain heading

**body bold**
";
    let first = written(source);
    let text = view(&first);
    assert!(
        text.lines()
            .any(|line| line == "# **Architectural Analysis**"),
        "{text}"
    );
    assert!(
        text.lines().any(|line| line == "## **Executive Summary**"),
        "{text}"
    );
    assert!(
        text.lines().any(|line| line == "### _Lean title_"),
        "{text}"
    );
    assert!(text.lines().any(|line| line == "# Plain heading"), "{text}");
    assert!(text.lines().any(|line| line == "**body bold**"), "{text}");

    let second = written(&text);
    let back = view(&second);
    assert!(
        back.lines()
            .any(|line| line == "# **Architectural Analysis**"),
        "{back}"
    );
    assert!(
        back.lines().any(|line| line == "## **Executive Summary**"),
        "{back}"
    );
    assert!(
        back.lines().any(|line| line == "### _Lean title_"),
        "{back}"
    );
    let xml = common::docx::part_string(&second, "word/document.xml").expect("document.xml");
    for heading in ["Architectural Analysis", "Executive Summary"] {
        let para = paragraph(&xml, heading);
        assert!(
            para.contains("<w:b/>") || para.contains("<w:b "),
            "{heading}: {para}"
        );
    }
    let lean = paragraph(&xml, "Lean title");
    assert!(lean.contains("<w:i/>") || lean.contains("<w:i "), "{lean}");
    let plain = paragraph(&xml, "Plain heading");
    assert!(!plain.contains("<w:b"), "{plain}");
}

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

/// A styles part: a bold `Heading1`, a bold body paragraph style `Loud`,
/// and `HeadChar`, a character style whose `basedOn` names the heading's
/// paragraph style.
fn with_styles(body: &str) -> Vec<u8> {
    let styles = format!(
        r#"<w:styles xmlns:w="{}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Loud"><w:name w:val="Loud"/><w:basedOn w:val="Normal"/><w:rPr><w:b/></w:rPr></w:style><w:style w:type="character" w:styleId="HeadChar"><w:name w:val="Head Char"/><w:basedOn w:val="Heading1"/></w:style></w:styles>"#,
        common::docx::W_NS
    );
    common::docx::docx_with(
        body,
        &[common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// Runs bold by their paragraph style and runs bold by a toggle print the
/// same in a body paragraph, so they join under one pair of stars.
#[test]
fn body_runs_bold_by_style_and_by_toggle_print_as_one_span() {
    let source = with_styles(
        r#"<w:p><w:pPr><w:pStyle w:val="Loud"/></w:pPr><w:r><w:t>foo</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>bar</w:t></w:r></w:p>"#,
    );
    let text = view(&source);
    assert!(text.contains("**foobar**"), "{text}");
    assert!(!text.contains("<b>"), "{text}");
}

/// Word ignores a `basedOn` naming a style of another type (probed
/// 2026-11-10: a body run in such a character style, based on a bold,
/// all-caps paragraph style, prints plain). The run states no bold of its
/// own, so a heading does not grow stars and a body run stays plain.
#[test]
fn a_character_style_based_on_a_paragraph_style_states_no_emphasis() {
    let source = with_styles(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:rPr><w:rStyle w:val="HeadChar"/></w:rPr><w:t>Title</w:t></w:r></w:p><w:p><w:r><w:t xml:space="preserve">plain </w:t></w:r><w:r><w:rPr><w:rStyle w:val="HeadChar"/></w:rPr><w:t>cross</w:t></w:r></w:p>"#,
    );
    let text = view(&source);
    assert!(text.lines().any(|line| line == "# Title"), "{text}");
    assert!(!text.contains("**cross**"), "{text}");
    assert!(text.contains("plain cross"), "{text}");
}
