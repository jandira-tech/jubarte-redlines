// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view of `jubarte text`: id lines, attribution notes, comment
//! ids, the YAML header and block selection. Goldens in
//! `tests/fixtures/agent-view/`.
mod common;
use common::docx::{Part, W_NS, docx, para};
use jubarte::markdown::{MarkdownOptions, Select, TrackChanges, docx_to_markdown};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/agent-view")
        .join(name)
}

fn golden(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).unwrap()
}

fn agent(docx: &[u8]) -> String {
    agent_with(docx, TrackChanges::All, true)
}

fn agent_with(docx: &[u8], track_changes: TrackChanges, comments: bool) -> String {
    agent_options(
        docx,
        MarkdownOptions {
            track_changes,
            comments,
            ..agent_defaults()
        },
    )
}

fn agent_defaults() -> MarkdownOptions {
    MarkdownOptions {
        ids: true,
        comments: true,
        source: Some("sample.docx".into()),
        ..MarkdownOptions::default()
    }
}

fn agent_options(docx: &[u8], options: MarkdownOptions) -> String {
    docx_to_markdown(docx, &options).unwrap().markdown
}

/// The body of an agent view: what follows the YAML header.
fn body(markdown: &str) -> &str {
    markdown.splitn(3, "---\n").nth(2).unwrap_or(markdown)
}

/// The header's lines, without the `---` fences.
fn header_lines(markdown: &str) -> Vec<&str> {
    markdown
        .splitn(3, "---\n")
        .nth(1)
        .unwrap()
        .lines()
        .collect()
}

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

#[track_caller]
fn ok(args: &[&str], dir: &Path) -> String {
    let out = jubarte(args, dir);
    assert!(
        out.status.success(),
        "jubarte {args:?} exited {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 stdout")
}

#[test]
fn options_default_to_the_plain_conversion() {
    let options = MarkdownOptions::default();
    assert!(!options.ids);
    assert!(options.comments);
    assert_eq!(options.source, None);
    assert_eq!(options.pages, None);
    assert!(options.page_markers);
    assert!(!options.dates);
    assert!(options.select.is_none());
    let bytes = docx(&para("Plain text."));
    let plain = docx_to_markdown(&bytes, &options).unwrap().markdown;
    assert_eq!(plain, "Plain text.\n");
}

#[test]
fn select_parses_single_paragraphs_ranges_lists_and_tables() {
    use jubarte::markdown::Pick;
    assert_eq!(
        Select::parse("p2, p5-p7,12,p17-,-p1,t0").unwrap(),
        Select::Picks(vec![
            Pick::Paragraphs {
                from: 2,
                to: Some(2)
            },
            Pick::Paragraphs {
                from: 5,
                to: Some(7)
            },
            Pick::Paragraphs {
                from: 12,
                to: Some(12)
            },
            Pick::Paragraphs { from: 17, to: None },
            Pick::Paragraphs {
                from: 0,
                to: Some(1)
            },
            Pick::Table(0),
        ])
    );
    assert_eq!(
        Select::parse("p7-p5").unwrap_err(),
        "p7-p5: the range runs backwards"
    );
    assert_eq!(
        Select::parse("x3").unwrap_err(),
        "x3: expected pN, pN-pM or tN"
    );
    assert_eq!(Select::parse(" , ").unwrap_err(), "no paragraphs selected");
}

#[test]
fn id_lines_precede_every_paragraph_and_the_header_opens_the_output() {
    let bytes = docx(&format!(
        "{}{}",
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#,
        para("Body text.")
    ));
    let out = agent(&bytes);
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p0 center -->\n# Title\n\n<!-- p1 -->\nBody text.\n"
    );
}

#[test]
fn empty_paragraphs_collapse_and_cached_breaks_number_the_pages() {
    let bytes = docx(&format!(
        "{}<w:p/><w:p/><w:p/>{}<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p><w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:lastRenderedPageBreak/><w:t>Second page</w:t></w:r></w:p>",
        para("One"),
        para("Five")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nOne\n\n<!-- p1-p3 empty -->\n\n<!-- p4 -->\nFive\n\n<!-- p5 page-break -->\n\n<!-- page 2 of 2 -->\n\n<!-- p6 center -->\nSecond page\n"
    );
}

#[test]
fn trailing_empty_paragraphs_are_still_listed() {
    let bytes = docx(&format!("{}<w:p/>", para("Only")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nOnly\n\n<!-- p1 empty -->\n"
    );
}

#[test]
fn layout_page_texts_place_the_markers_above_the_id_lines() {
    let bytes = docx(&format!(
        "{}{}{}",
        para("Alpha text here"),
        para("Beta text here"),
        para("Gamma text here")
    ));
    let out = agent_options(
        &bytes,
        MarkdownOptions {
            pages: Some(vec![
                "Alpha text here".into(),
                "Beta text here Gamma text here".into(),
            ]),
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&out),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- page 2 of 2 -->\n\n<!-- p1 -->\nBeta text here\n\n<!-- p2 -->\nGamma text here\n"
    );
}

#[test]
fn paginate_holds_comment_lines_at_a_block_start() {
    let md = "<!-- p0 -->\nAlpha text here\n\n<!-- p1 page-break -->\n\n<!-- t0 1x1, cells p2-p2 by row -->\n|Beta text here|\n|-|\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["alpha text here", "beta text here"]),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- p1 page-break -->\n\n<!-- page 2 of 2 -->\n\n<!-- t0 1x1, cells p2-p2 by row -->\n|Beta text here|\n|-|\n"
    );
}

#[test]
fn id_line_shows_direct_alignment_indent_style_and_number() {
    let numbering = Part {
        name: "word/numbering.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        xml: &format!(
            r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
        ),
    };
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:pStyle w:val="Quote"/><w:jc w:val="both"/><w:ind w:left="720" w:hanging="360"/></w:pPr><w:r><w:t>Quoted.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>First item</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>Numbered heading</w:t></w:r></w:p>"#,
    );
    let styles = Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &format!(
            r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:style></w:styles>"#
        ),
    };
    let bytes = common::docx::docx_with(body_xml, &[numbering, styles]);
    let out = body(&agent(&bytes)).to_string();
    assert!(
        out.contains("<!-- p0 Quote justify, hanging 0.25in, left 0.5in -->\nQuoted."),
        "{out}"
    );
    assert!(
        out.contains("<!-- p1 num \"1.\" -->\n1. First item"),
        "{out}"
    );
    assert!(
        out.contains("<!-- p2 num \"2.\" -->\n## 2. Numbered heading"),
        "{out}"
    );
}

#[test]
fn id_line_names_tracked_paragraph_marks_and_formatting_changes() {
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="4" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>Split here</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>Now bold</w:t></w:r></w:p>"#,
    );
    let out = body(&agent(&docx(body_xml))).to_string();
    assert!(
        out.contains("<!-- p0 break-ins #4 @AC -->\nSplit here"),
        "{out}"
    );
    assert!(
        out.contains("<!-- p1 fmt #5 @AC -->\n**Now bold**"),
        "{out}"
    );
}
