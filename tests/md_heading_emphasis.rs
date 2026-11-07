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
