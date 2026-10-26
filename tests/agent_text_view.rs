// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view of `jubarte text`: id lines, attribution notes, comment
//! ids, the YAML header and block selection. Goldens in
//! `tests/fixtures/agent-view/`.
mod common;
use common::docx::{Part, W_NS, docx, para};
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, Select, TrackChanges, docx_to_markdown, markdown_to_docx,
};
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

fn assert_golden(name: &str, actual: &str) {
    if std::env::var_os("JUBARTE_BLESS").is_some() {
        std::fs::write(fixture(name), actual).unwrap();
    }
    assert_eq!(
        actual,
        golden(name),
        "{name}: regenerate with JUBARTE_BLESS=1 and received_docx_*_view*_match*"
    );
}

fn agent(docx: &[u8]) -> String {
    agent_with(docx, TrackChanges::All, true)
}

fn agent_with(docx: &[u8], track_changes: TrackChanges, comments: bool) -> String {
    agent_options(
        docx,
        &MarkdownOptions {
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

fn agent_options(docx: &[u8], options: &MarkdownOptions) -> String {
    docx_to_markdown(docx, options).unwrap().markdown
}

/// The body of an agent view: what follows the YAML header and the blank
/// line after it.
fn body(markdown: &str) -> &str {
    markdown
        .splitn(3, "---\n")
        .nth(2)
        .map_or(markdown, |b| b.strip_prefix('\n').unwrap_or(b))
}

/// The header's lines, without the `---` fences.
fn header_lines(markdown: &str) -> Vec<&str> {
    markdown.split("---\n").nth(1).unwrap().lines().collect()
}

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

#[track_caller]
/// stdout and stderr of a run that must succeed.
fn ok_both(args: &[&str], dir: &Path) -> (String, String) {
    let out = jubarte(args, dir);
    assert!(
        out.status.success(),
        "jubarte {args:?} exited {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (
        String::from_utf8(out.stdout).expect("utf-8 stdout"),
        String::from_utf8(out.stderr).expect("utf-8 stderr"),
    )
}

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
        Select::parse("p2, p5-p7,p12,p17-,-p1,t0").unwrap(),
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
    // A bare number is expired (Arthur, 2026-10-13).
    assert_eq!(
        Select::parse("12").unwrap_err(),
        "12: expected pN, pN-pM or tN; the settled spelling is p12"
    );
    assert_eq!(
        Select::parse("-").unwrap_err(),
        "-: expected pN, pN-pM or tN"
    );
    // A range's hint is the whole range, settled; none when it would run
    // backwards.
    assert_eq!(
        Select::parse("p2-3").unwrap_err(),
        "p2-3: expected pN, pN-pM or tN; the settled spelling is p2-p3"
    );
    assert_eq!(
        Select::parse("2-").unwrap_err(),
        "2-: expected pN, pN-pM or tN; the settled spelling is p2-"
    );
    assert_eq!(
        Select::parse("2-1").unwrap_err(),
        "2-1: expected pN, pN-pM or tN"
    );
}

#[test]
fn id_lines_precede_every_paragraph_and_the_header_opens_the_output() {
    let bytes = docx(&format!(
        "{}{}",
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#,
        para("Body text.")
    ));
    let out = agent(&bytes);
    assert!(out.starts_with("---\nsource: sample.docx\n"), "{out}");
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
fn leading_empty_paragraphs_follow_the_first_page_marker() {
    let bytes = docx(&format!("<w:p/><w:p/>{}", para("Text")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0-p1 empty -->\n\n<!-- p2 -->\nText\n"
    );
}

#[test]
fn a_document_of_empty_paragraphs_still_opens_page_one() {
    let bytes = docx("<w:p/><w:p/>");
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0-p1 empty -->\n"
    );
}

const LETTER: &str = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#;

#[test]
fn without_cached_breaks_hard_page_breaks_number_the_pages() {
    let bytes = docx(&format!(
        r#"{}<w:p><w:r><w:br w:type="page"/></w:r></w:p>{}"#,
        para("One"),
        para("Two")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nOne\n\n<!-- p1 page-break -->\n\n<!-- page 2 of 2 -->\n\n<!-- p2 -->\nTwo\n"
    );
}

#[test]
fn a_section_break_opens_a_page_unless_the_next_section_is_continuous() {
    let body_xml = format!(
        r#"<w:p><w:pPr><w:sectPr>{LETTER}</w:sectPr></w:pPr><w:r><w:t>One</w:t></w:r></w:p>{}"#,
        para("Two")
    );
    let continuous = common::docx::docx_with_sect_pr(
        &body_xml,
        &[],
        &format!(r#"<w:sectPr><w:type w:val="continuous"/>{LETTER}</w:sectPr>"#),
    );
    assert_eq!(
        body(&agent(&continuous)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 section-break -->\nOne\n\n<!-- p1 -->\nTwo\n"
    );
    assert_eq!(
        body(&agent(&docx(&body_xml))),
        "<!-- page 1 of 2 -->\n\n<!-- p0 section-break -->\nOne\n\n<!-- page 2 of 2 -->\n\n<!-- p1 -->\nTwo\n"
    );
}

#[test]
fn a_page_break_inside_a_text_box_turns_no_page() {
    let boxed = r#"<w:p><w:r><w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shape><v:textbox><w:txbxContent><w:p><w:r><w:br w:type="page"/><w:t>Boxed</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r><w:r><w:t>Host</w:t></w:r></w:p>"#;
    let out = agent(&docx(&format!("{boxed}{}", para("After"))));
    assert!(body(&out).starts_with("<!-- page 1 of 1 -->\n"), "{out}");
    assert!(!out.contains("of 2"), "{out}");
}

#[test]
fn cached_breaks_inside_a_table_are_named_at_the_next_block() {
    let cell = |inner: &str| format!("<w:tr><w:tc><w:p>{inner}</w:p></w:tc></w:tr>");
    let table = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>{}{}{}</w:tbl>"#,
        cell("<w:r><w:t>A</w:t></w:r>"),
        cell("<w:r><w:lastRenderedPageBreak/><w:t>B</w:t></w:r>"),
        cell("<w:r><w:lastRenderedPageBreak/><w:t>C</w:t></w:r>")
    );
    let out = agent(&docx(&format!(
        "{}{table}{}",
        para("Before"),
        para("After")
    )));
    assert!(
        body(&out).starts_with(
            "<!-- page 1 of 3 -->\n\n<!-- p0 -->\nBefore\n\n<!-- t0 3x1, cells p1-p3 by row, widths 2.778in, width auto -->\n"
        ),
        "{out}"
    );
    assert!(
        body(&out).ends_with("\n\n<!-- page 3 of 3 -->\n\n<!-- p4 -->\nAfter\n"),
        "{out}"
    );
}

#[test]
fn page_markers_off_leaves_no_page_lines_and_keeps_the_rest() {
    let bytes = docx(&format!(
        r#"{}<w:p><w:r><w:br w:type="page"/></w:r></w:p><w:p/>{}"#,
        para("One"),
        para("Two")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            page_markers: false,
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&out),
        "<!-- p0 -->\nOne\n\n<!-- p1 page-break -->\n\n<!-- p2 empty -->\n\n<!-- p3 -->\nTwo\n"
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
        &MarkdownOptions {
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
    assert!(
        out.contains("\nbody: p0-p2, 0 tables, 2 pages     # pages from layout\n"),
        "{out}"
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
fn paginate_reads_plain_comment_text_as_the_block_it_opens() {
    // Document text that reads as an HTML comment opens its block in the
    // plain view: the text under it is mid-block, so its page is named at
    // the next block, exactly as before the agent view existed.
    let md = "Intro\n\n<!-- drafted by AC -->\nBeta text here\n\nGamma\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["intro", "beta text here", "gamma"]),
        "<!-- page 1 of 3 -->\n\nIntro\n\n<!-- drafted by AC -->\nBeta text here\n\n<!-- page 3 of 3 -->\n\nGamma\n"
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

/// A paragraph states its own outline level in its `w:pPr`, whatever
/// style it carries; a style that gives no level (`Body`) must not swallow
/// it, or the level is lost on write-back (#1524 review). One the `#` count
/// already gives needs no clause.
#[test]
fn a_styled_paragraph_keeps_the_outline_level_it_states() {
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:pStyle w:val="Body"/><w:outlineLvl w:val="8"/></w:pPr><w:r><w:t>Deep body</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Body"/><w:outlineLvl w:val="1"/></w:pPr><w:r><w:t>Shallow body</w:t></w:r></w:p>"#,
    );
    let styles = Part {
        name: "word/styles.xml",
        content_type: STYLES_CT,
        rel_type: STYLES_REL,
        xml: &format!(
            r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Body"><w:name w:val="Body Text"/></w:style></w:styles>"#
        ),
    };
    let full = agent(&common::docx::docx_with(body_xml, &[styles]));
    let out = body(&full).to_string();
    assert!(
        out.contains("<!-- p0 Body outline 9 -->\nDeep body"),
        "{out}"
    );
    assert!(out.contains("<!-- p1 Body -->\n## Shallow body"), "{out}");
    // The level comes back beside the style the line names.
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    let document = common::docx::part_string(&back, "word/document.xml").unwrap();
    assert!(
        document.contains(r#"<w:pStyle w:val="Body"/><w:outlineLvl w:val="8"/>"#),
        "{document}"
    );
    assert!(
        document.contains(r#"<w:pStyle w:val="Body"/><w:outlineLvl w:val="1"/>"#),
        "{document}"
    );
}

#[test]
fn paragraph_marks_print_inline_and_formatting_changes_on_the_id_line() {
    let body_xml = concat!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="4" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>Split here</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>Now bold</w:t></w:r></w:p>"#,
    );
    let out = body(&agent(&docx(body_xml))).to_string();
    assert!(out.contains("<!-- p0 -->\nSplit here¶{r4 @AC}"), "{out}");
    assert!(
        out.contains("<!-- p1 fmt r5 @AC -->\n**Now bold**"),
        "{out}"
    );
}

const TABLE_3X2: &str = r#"<w:tbl><w:tblPr><w:jc w:val="center"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:p><w:r><w:t>Item</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Due</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Report</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Day 10</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Call</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Monthly</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

#[test]
fn table_line_gives_grid_cells_and_header_row() {
    let bytes = docx(&format!("{}{TABLE_3X2}{}", para("Before"), para("After")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBefore\n\n<!-- t0 center 3x2, cells p1-p6 by row, widths 2.778in 2.778in, width auto, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p7 -->\nAfter\n"
    );
}

#[test]
fn table_line_lists_rows_when_a_cell_holds_two_paragraphs_and_notes_merges_and_marks() {
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>Head</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:pPr><w:rPr><w:ins w:id="12" w:author="Ann Counsel"/></w:rPr></w:pPr><w:r><w:t>a</w:t></w:r></w:p><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = body(&agent(&docx(tbl))).to_string();
    assert!(out.starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 2x2, cells t0.row0 p0 t0.row1 p1-p3, widths 2.778in 2.778in, width auto, merged t0.row0.cell0 across 2, break-ins r12 @AC in p1 -->\n"), "{out}");
}

#[test]
fn a_cells_empty_paragraphs_keep_their_breaks() {
    // Each paragraph of a cell after its first starts with a `<br>`, an
    // empty one too: the table line counts them all.
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let tbl = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{}<w:p/>{}</w:tc><w:tc>{}<w:p/></w:tc></w:tr></w:tbl>"#,
        p("a"),
        p("b"),
        p("c")
    );
    let out = body(&agent(&docx(&tbl))).to_string();
    assert!(
        out.starts_with(
            "<!-- page 1 of 1 -->\n\n<!-- t0 1x2, cells t0.row0 p0-p4, widths 2.778in 2.778in, width auto -->\n|a<br><br>b|c<br>|\n"
        ),
        "{out}"
    );
}

#[test]
fn table_line_names_each_merged_cell_and_how_far_it_reaches() {
    // A vertical merge names its first cell and the rows it covers; a cell
    // spanning columns names how many.
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="3000"/><w:gridCol w:w="3000"/><w:gridCol w:w="3000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc><w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = body(&agent(&docx(tbl))).to_string();
    assert!(out.starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 2x3, cells p0-p4 by row, widths 2.083in 2.083in 2.083in, width auto, merged t0.row0.cell0 down 2; t0.row0.cell1 across 2 -->\n|A|B||\n|-|-|-|\n||C|D|\n"), "{out}");
}

/// The ids round trip (Task 5b): a table's own property changes and a
/// cell's insertion show nowhere in the cells' text, so the tracked view's
/// table line names each where it sits; the resolved views have applied
/// them.
#[test]
fn table_line_names_the_tables_own_changes() {
    // A cell's paragraph has no id line: the table line names its
    // formatting changes too (`r6 @AC in p2`).
    let a = r#"w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z""#;
    let tbl = format!(
        r#"<w:tbl><w:tblPr><w:jc w:val="center"/><w:tblPrChange w:id="1" {a}><w:tblPr/></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/><w:tblGridChange w:id="2"><w:tblGrid><w:gridCol w:w="8000"/></w:tblGrid></w:tblGridChange></w:tblGrid><w:tr><w:trPr><w:trPrChange w:id="3" {a}><w:trPr/></w:trPrChange></w:trPr><w:tc><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc><w:tc><w:tcPr><w:tcPrChange w:id="4" {a}><w:tcPr/></w:tcPrChange></w:tcPr><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:pPr><w:jc w:val="right"/><w:pPrChange w:id="6" {a}><w:pPr/></w:pPrChange></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id="7" {a}><w:rPr/></w:rPrChange></w:rPr><w:t>c</w:t></w:r></w:p></w:tc><w:tc><w:tcPr><w:cellIns w:id="5" {a}/></w:tcPr><w:p><w:r><w:t>d</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
    );
    let bytes = docx(&tbl);
    let out = body(&agent(&bytes)).to_string();
    assert!(
        out.starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 center 2x2, cells p0-p3 by row, p2 right, widths 2.778in 2.778in, width auto, rev r5 @AC in t0.row1.cell1, fmt r1 @AC; r2 @??; r3 @AC in t0.row0; r4 @AC in t0.row0.cell1; r6 @AC in p2; r7 @AC in p2 -->\n"),
        "{out}"
    );
    let listed: Vec<String> = jubarte::changes::list_changes(&bytes)
        .unwrap()
        .into_iter()
        .map(|c| c.r#ref)
        .collect();
    assert_eq!(listed, ["r1", "r2", "r3", "r4", "r6", "r7", "r5"]);
    for track_changes in [TrackChanges::Accept, TrackChanges::Reject] {
        let resolved = body(&agent_with(&bytes, track_changes, true)).to_string();
        assert!(!resolved.contains(" fmt "), "{resolved}");
    }
}

fn ins(id: u32, author: &str, text: &str) -> String {
    ins_at(id, author, "2026-10-01T09:00:00Z", text)
}

fn ins_at(id: u32, author: &str, date: &str, text: &str) -> String {
    format!(
        r#"<w:ins w:id="{id}" w:author="{author}" w:date="{date}"><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:ins>"#
    )
}

fn del(id: u32, author: &str, text: &str) -> String {
    format!(
        r#"<w:del w:id="{id}" w:author="{author}" w:date="2026-10-01T09:00:00Z"><w:r><w:delText xml:space="preserve">{text}</w:delText></w:r></w:del>"#
    )
}

fn run(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

#[test]
fn attribution_notes_follow_each_change_with_id_and_handle() {
    let p = format!(
        "<w:p>{}{}{}{}{}{}{}</w:p>",
        run("Pay within "),
        ins(4, "Ann Counsel", "forty-five"),
        del(3, "Ann Counsel", "thirty"),
        run(" days, "),
        ins(5, "Ann Counsel", "quarterly"),
        run(" reports"),
        del(6, "Ann Counsel", ", nothing else")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nPay within {++forty-five++}{r4+3 @AC}{--thirty--}{r4+3 @AC} days, {++quarterly++}{r5 @AC} reports{--, nothing else--}{r6 @AC}\n"
    );
}

#[test]
fn neighbouring_marks_by_one_author_are_one_change_named_on_each_piece() {
    let p = format!(
        "<w:p>{}{}{}</w:p>",
        run("a "),
        ins(7, "Ann Counsel", "bold"),
        ins(8, "Ann Counsel", " words")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\na {++bold++}{r7+8 @AC}{++ words++}{r7+8 @AC}\n"
    );
}

#[test]
fn a_replacement_pairs_only_the_deletion_and_following_insertion_by_one_author() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("x "),
        ins(2, "John Doe", "new"),
        del(1, "Ann Counsel", "old"),
        ins(3, "Ann Counsel", "!")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nx {++new++}{r2 @JD}{~~old~>!~~}{r1+3 @AC}\n"
    );
}

#[test]
fn dates_go_inline_only_when_asked_and_only_for_an_author_with_several() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("a "),
        ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "b"),
        run(" c "),
        ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "d")
    );
    let bytes = docx(&p);
    assert!(
        body(&agent(&bytes)).contains("{++b++}{r1 @AC} c {++d++}{r2 @AC}"),
        "{}",
        agent(&bytes)
    );
    let dated = agent_options(
        &bytes,
        &MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert!(
        body(&dated).contains(
            "{++b++}{r1 @AC 2026-10-01T09:00:00Z} c {++d++}{r2 @AC 2026-10-03T14:05:00Z}"
        ),
        "{dated}"
    );
    let single = docx(&format!(
        "<w:p>{}{}</w:p>",
        run("a "),
        ins(1, "Ann Counsel", "b")
    ));
    let dated = agent_options(
        &single,
        &MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert!(body(&dated).contains("{++b++}{r1 @AC}"), "{dated}");
}

#[test]
fn legacy_output_keeps_author_notes() {
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(7, "Ann Counsel", "b"));
    let legacy = docx_to_markdown(&docx(&p), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert_eq!(legacy, "a {++b++}{r7 Ann Counsel (2026-10-01T09:00:00Z)}\n");
}

#[test]
fn an_author_cannot_forge_an_agent_tag_in_the_plain_view() {
    let p = format!("<w:p>{}</w:p>", ins(7, "\u{E000}0@JD", "b"));
    let legacy = docx_to_markdown(&docx(&p), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    // The id comes first, so the name never reads as a tag.
    assert_eq!(legacy, "{++b++}{r7 0@JD (2026-10-01T09:00:00Z)}\n");
}

#[test]
fn a_tracked_mark_inside_a_footnote_carries_its_agent_tag() {
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p><w:pPr><w:rPr><w:ins w:id="4" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>First</w:t></w:r></w:p><w:p><w:r><w:t>second</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let bytes = common::docx::docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &footnotes,
        }],
    );
    let out = agent(&bytes);
    assert!(
        out.contains("[^1]: First{++<br>++}{footnotes.r4 @AC}second"),
        "{out}"
    );
    assert!(!out.contains("Ann Counsel ("), "{out}");
}

/// A note's change names its story: `footnotes.r3` reads back as the
/// footnote part's `w:id` 3, never the body's.
#[test]
fn a_footnote_change_names_its_story() {
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p>{}{}</w:p></w:footnote></w:footnotes>"#,
        run("Source"),
        ins(3, "Ann Counsel", " x")
    );
    let bytes = common::docx::docx_with(
        r#"<w:p><w:r><w:t>Text</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &footnotes,
        }],
    );
    let out = agent(&bytes);
    assert!(
        out.contains("[^1]: Source{++ x++}{footnotes.r3 @AC}"),
        "{out}"
    );
    assert!(
        jubarte::changes::list_changes(&bytes)
            .unwrap()
            .iter()
            .any(|c| c.id == "footnotes.r3"),
        "the printed id names a listed change"
    );
}

/// A header's change shows on its `headers:` line with a story tag, and
/// the header counts it.
#[test]
fn a_header_change_is_visible_on_the_headers_line() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}</w:p></w:hdr>"#,
        ins(1, "Ann Counsel", "DRAFT")
    );
    let bytes = common::docx::docx_with_sect(
        &para("Body"),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert!(
        lines.contains(
            &"  default: {id: header1, text: \"{++DRAFT++}{header1.r1 @AC}\", style: Normal}"
        ),
        "{out}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("revisions: 1 ")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("  AC: Ann Counsel") && l.contains("# 1 revision,")),
        "{out}"
    );
    // --changed --by AC finds no body block but still shows the change.
    let changed = changed_by(&bytes, Some("AC"), true);
    assert!(changed.contains("{++DRAFT++}{header1.r1 @AC}"), "{changed}");
    // The resolved views print the plain text, without tags.
    let accepted = agent_with(&bytes, TrackChanges::Accept, true);
    assert!(
        accepted.contains("  default: {id: header1, text: DRAFT, style: Normal}"),
        "{accepted}"
    );
}

/// Task 5b: a change whose text the view cannot show still prints its id
/// once: a section's `w:sectPrChange` on `page:`, a style's change on
/// `style_changes:`, a header's formatting on its `headers:` entry, and a
/// footnote's formatting on its definition line. The resolved views print
/// none of them.
#[test]
fn changes_without_text_name_their_ids() {
    let rpr_change = |id: u32| {
        format!(
            r#"<w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Ann Counsel" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr>"#
        )
    };
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r>{}<w:t>Title</w:t></w:r></w:p></w:hdr>"#,
        rpr_change(4)
    );
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p><w:r>{}<w:t>Source</w:t></w:r></w:p></w:footnote></w:footnotes>"#,
        rpr_change(7)
    );
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/>{}</w:style></w:styles>"#,
        rpr_change(9)
    );
    let bytes = common::docx::docx_with_sect_pr(
        r#"<w:p><w:r><w:t>Text</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
            Part {
                name: "word/footnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
                xml: &footnotes,
            },
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: &styles,
            },
        ],
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/><w:sectPrChange w:id="2" w:author="Ann Counsel"><w:sectPr/></w:sectPrChange></w:sectPr>"#,
    );
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("page: ") && l.ends_with("; fmt r2 @AC")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l
                .starts_with(r#"style_changes: {Normal: "styles.r9 @AC 2026-01-01T00:00:00Z"}"#)),
        "{out}"
    );
    assert!(
        lines.contains(
            &r#"  default: {id: header1, text: Title, style: Normal, fmt: "header1.r4 @AC 2026-01-01T00:00:00Z in header1"}"#
        ),
        "{out}"
    );
    assert!(
        out.contains("[^1]: **Source** <!-- fmt footnotes.r7 @AC 2026-01-01T00:00:00Z -->"),
        "{out}"
    );
    let listed: Vec<String> = jubarte::changes::list_changes(&bytes)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    for id in ["r2", "styles.r9", "header1.r4", "footnotes.r7"] {
        assert!(listed.iter().any(|l| l == id), "{id} not in {listed:?}");
    }
    for view in [TrackChanges::Accept, TrackChanges::Reject] {
        let resolved = agent_with(&bytes, view, true);
        for tag in ["r2 @AC", "styles.r9", "header1.r4", "footnotes.r7"] {
            assert!(!resolved.contains(tag), "{tag} in {resolved}");
        }
    }
}

/// The legend's own order governs `style_changes:`: `docDefaults` leads,
/// then the default style. `usize::MIN` is zero, so a part that lists the
/// default style before its defaults used to win the tie (#1521 review).
#[test]
fn the_style_changes_legend_leads_with_the_document_defaults() {
    let rpr_change = |id: u32| {
        format!(
            r#"<w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Ann Counsel" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr>"#
        )
    };
    // Word writes `w:docDefaults` first; a part that lists the styles
    // before it is read the same way, and the legend order still holds.
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/>{}</w:style><w:docDefaults><w:rPrDefault>{}</w:rPrDefault></w:docDefaults></w:styles>"#,
        rpr_change(9),
        rpr_change(11)
    );
    let bytes = common::docx::docx_with_sect_pr(
        r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    let out = agent(&bytes);
    assert!(
        header_lines(&out).iter().any(|l| {
            l.starts_with(r#"style_changes: {docDefaults: styles.r11 @AC, Normal: styles.r9 @AC}"#)
        }),
        "{out}"
    );
}

/// Task 5b, from Word redlines: an empty row's deletion has no text to tag,
/// a first-page header still exists when different-first-page is off, and
/// Word's footnote separators carry formatting changes no note shows.
#[test]
fn rows_inactive_headers_and_separators_name_their_changes() {
    let cell = |text: &str| {
        format!(
            r#"<w:tc><w:p>{}</w:p></w:tc>"#,
            if text.is_empty() {
                String::new()
            } else {
                format!("<w:r><w:t>{text}</w:t></w:r>")
            }
        )
    };
    let body = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol/></w:tblGrid><w:tr>{}</w:tr><w:tr><w:trPr><w:del w:id="6" w:author="Ann Counsel"/></w:trPr>{}</w:tr></w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
        cell("Kept"),
        cell("")
    );
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="4" w:author="Ann Counsel"><w:rPr/></w:rPrChange></w:rPr><w:t>Cover</w:t></w:r></w:p></w:hdr>"#
    );
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:pPr><w:spacing w:after="0"/><w:pPrChange w:id="9" w:author="Ann Counsel"><w:pPr/></w:pPrChange></w:pPr><w:r><w:separator/></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let bytes = common::docx::docx_with_sect(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
            Part {
                name: "word/footnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
                xml: &footnotes,
            },
        ],
        r#"<w:headerReference w:type="first" r:id="rIdX0"/>"#,
    );
    let out = agent(&bytes);
    assert!(
        out.contains("<!-- t0 2x1, cells p0-p1 by row, width auto, rev-del r6 @AC in t0.row1 -->"),
        "{out}"
    );
    let lines = header_lines(&out);
    assert!(
        lines.iter().any(|l| l.starts_with(
            "  first: {id: header1, text: Cover, style: Normal, inactive, fmt: header1.r4 @AC in header1}"
        ) && l.ends_with("# defined, but different first page is off")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("notes: {fmt: footnotes.r9 @AC, empty: [footnotes]}")),
        "{out}"
    );
    let accepted = agent_with(&bytes, TrackChanges::Accept, true);
    for tag in ["r6 @AC", "header1.r4", "footnotes.r9"] {
        assert!(!accepted.contains(tag), "{tag} in {accepted}");
    }
}

/// A run's `w:pict` text box, Word's VML copy of a DrawingML one.
fn pict_box(inner: &str) -> String {
    format!(
        r#"<w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p>{inner}</w:p></w:txbxContent></v:textbox></v:shape></w:pict>"#
    )
}

/// A run's DrawingML text box.
fn drawing_box(inner: &str) -> String {
    format!(
        r#"<w:drawing><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:txbx><w:txbxContent><w:p>{inner}</w:p></w:txbxContent></wps:txbx></wps:wsp></w:drawing>"#
    )
}

/// pi review of Task 5b F3–F5: a change kept in a formatting change's
/// history is no change the view names (`jubarte changes` lists none); a
/// list number's change is formatting; a row whose only text sits in Word's
/// Fallback copy shows no text, so its table line names its deletion.
#[test]
fn history_numbering_and_fallback_rows_follow_the_listing() {
    let body = format!(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/><w:numberingChange w:id="5" w:author="Ann Counsel" w:original="%1."/></w:numPr></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id="10" w:author="Ann Counsel"><w:rPr><w:del w:id="11" w:author="Ann Counsel"/></w:rPr></w:rPrChange></w:rPr><w:t>Item</w:t></w:r></w:p><w:tbl><w:tblGrid><w:gridCol/></w:tblGrid><w:tr><w:trPr><w:del w:id="6" w:author="Ann Counsel"/></w:trPr><w:tc><w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing/></mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
        pict_box("<w:r><w:t>Old</w:t></w:r>")
    );
    let bytes = docx(&body);
    let out = agent(&bytes);
    let listed: Vec<String> = jubarte::changes::list_changes(&bytes)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(!listed.iter().any(|l| l == "r11"), "{listed:?}");
    assert!(listed.iter().any(|l| l == "r5"), "{listed:?}");
    let r11 = regex::Regex::new(r"\b(r|\+)11\b").unwrap();
    assert!(!r11.is_match(&out), "{out}");
    let p0 = out
        .lines()
        .find(|l| l.starts_with("<!-- p0"))
        .unwrap_or_default();
    assert!(
        regex::Regex::new(r"\b(r|\+)5\b").unwrap().is_match(p0),
        "{out}"
    );
    assert!(out.contains("rev-del r6 @AC in t0.row0"), "{out}");
}

/// pi review of Task 5b F6: an author whose only changes sit in the styles
/// part gets a handle of their own.
#[test]
fn a_styles_only_author_gets_a_handle() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:b/><w:rPrChange w:id="9" w:author="Zed Styles"><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let bytes = common::docx::docx_with(
        &para("Text"),
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    );
    let out = agent(&bytes);
    assert!(out.contains("styles.r9 @ZS"), "{out}");
    assert!(!out.contains("@??"), "{out}");
}

/// pi review of Task 5b F1/F2: a change the view prints nowhere else (in
/// Word's Fallback copy with no Choice twin, or inside a drawing the view
/// carries whole as a `drawing:` payload, whose atomic token no mark can
/// name) is named on the header's `unplaced:` line, by paragraph.
#[test]
fn changes_printed_nowhere_else_are_unplaced() {
    let body = format!(
        r#"<w:p><w:r><w:t>Lead</w:t></w:r><w:r><mc:AlternateContent><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p><w:p><w:r><w:t>ping r3 @ops; 21 @AC</w:t></w:r></w:p>"#,
        drawing_box(&format!(
            r#"<w:r><w:t>Box</w:t></w:r><w:del w:id="3" w:author="Ann Counsel"><w:r><w:drawing/></w:r></w:del><w:del w:id="22"><w:r><w:drawing/></w:r></w:del>{}"#,
            ins(20, "Ann Counsel", "Choice")
        )),
        pict_box(&format!(
            r#"<w:r><w:t>Box</w:t></w:r>{}"#,
            ins(21, "Ann Counsel", "Other")
        ))
    );
    // Document text that reads like a tag (the body's, a header's) prints
    // no change.
    let header =
        format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>see r21 @AC</w:t></w:r></w:p></w:hdr>"#);
    let bytes = common::docx::docx_with_sect(
        &body,
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = agent(&bytes);
    let listed: Vec<String> = jubarte::changes::list_changes(&bytes)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    for id in ["r3", "r21", "r22"] {
        assert!(listed.iter().any(|l| l == id), "{id} not in {listed:?}");
    }
    let lines = header_lines(&out);
    let unplaced = lines
        .iter()
        .find(|l| l.starts_with("unplaced: "))
        .unwrap_or_else(|| panic!("no unplaced line in {out}"));
    assert!(unplaced.contains("r3 @AC in p0"), "{out}");
    assert!(
        unplaced.contains("r21 @AC 2026-10-01T09:00:00Z in p0"),
        "{out}"
    );
    // The shown branch's text box prints its content as the paragraph
    // after the drawing, so the Choice branch's own insertion is placed
    // there (`Box{++Choice++}{r20 @AC 2026-10-01T09:00:00Z}`), not unplaced.
    assert!(
        out.contains("\nBox{++Choice++}{r20 @AC 2026-10-01T09:00:00Z}\n"),
        "{out}"
    );
    assert!(!unplaced.contains("r20"), "{out}");
    // A handle no author gave (`@??`) is quoted: plain, YAML misreads it.
    assert!(unplaced.contains(r#""del r22 @?? in p0""#), "{out}");
    let yaml = header_yaml(&out);
    assert_eq!(yaml["unplaced"].as_vec().map(Vec::len), Some(3), "{out}");
    for view in [TrackChanges::Accept, TrackChanges::Reject] {
        assert!(
            !agent_with(&bytes, view, true).contains("unplaced:"),
            "{view:?}"
        );
    }
    // A document whose every change prints has no such line.
    assert!(
        !agent(&docx(&format!("<w:p>{}</w:p>", ins(1, "Ann Counsel", "x")))).contains("unplaced:")
    );
}

/// A textless change inside a printed container is not shown by the
/// container's tag: accepting the inserted row keeps the row and leaves
/// the deleted drawing tracked, so the drawing's own id must be printed.
/// Since 54621454 carries the deleted `w:drawing` whole, that id prints
/// inside the cell, in the drawing's own `{--![][d1]--}` mark where
/// Word's markup places the deletion — not on the `unplaced:` line the
/// view used when it could not carry a drawing at all.
#[test]
fn a_textless_change_in_a_printed_row_prints_its_own_tag() {
    let body_xml = format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:ins w:id="1" w:author="Ann Counsel"/></w:trPr><w:tc><w:p><w:del w:id="2" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:r><w:drawing/></w:r></w:del>{}</w:p></w:tc></w:tr></w:tbl><w:p/>"#,
        ins(3, "Ann Counsel", "Cell")
    );
    let out = agent(&docx(&body_xml));
    // The deleted drawing and the following insertion by the same author/date
    // form a substitution whose tag preserves both ids.
    assert!(
        body(&out).contains("{~~![][d1]~>Cell~~}{r2+3 @AC 2026-10-01T09:00:00Z}"),
        "{out}"
    );
    assert!(out.contains("rev r1 @AC in t0.row0"), "{out}");
    assert!(!body(&out).contains("{r1 @AC}"), "{out}");
    // Every change prints, so none lands on the `unplaced:` line.
    assert!(!out.contains("unplaced:"), "{out}");
}

/// pi review of the 5b fixes F2/F5/F6: an author whose only change is a
/// list number's gets a handle; a header's list number change is
/// formatting; an author only the styles name is numbered after the
/// comment authors, so theirs keep their handles.
#[test]
fn numbering_and_styles_authors_get_handles_after_comment_authors() {
    let numbered = |id: u32, author: &str| {
        format!(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/><w:numberingChange w:id="{id}" w:author="{author}" w:original="%1."/></w:numPr></w:pPr><w:r><w:t>Item</w:t></w:r></w:p>"#
        )
    };
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}">{}</w:hdr>"#,
        numbered(5, "Ann Counsel")
    );
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:b/><w:rPrChange w:id="9" w:author="Zed Styles"><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            1,
            "Zoe Stone",
            "ZS",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Note."
        )
    );
    let body = format!(
        r#"{}<w:p><w:commentRangeStart w:id="1"/>{}<w:commentRangeEnd w:id="1"/>{}</w:p>"#,
        numbered(4, "Nora Num"),
        run("x"),
        reference(1)
    );
    let bytes = common::docx::docx_with_sect(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: &styles,
            },
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = agent(&bytes);
    assert!(out.contains("r4 @NN"), "{out}");
    assert!(!out.contains("@??"), "{out}");
    let lines = header_lines(&out);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("id: header1") && l.contains("fmt: header1.r5 @AC")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.trim_start().starts_with("ZS: Zoe Stone")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.trim_start().starts_with("ZS2: Zed Styles")),
        "{out}"
    );
    assert!(out.contains("styles.r9 @ZS2"), "{out}");
    assert!(!out.contains("unplaced:"), "{out}");
}

/// pi review of Task 5 F1/F2: each view reads a header's changes its own
/// way, moves included, and the tracked view escapes document text that
/// reads like a mark. An insertion followed by the same author's deletion
/// is one replacement, its id on both pieces, as in the body (#1359).
#[test]
fn header_text_follows_each_view() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}{}{}{}<w:moveTo w:id="9" w:author="Ann Counsel"><w:r><w:t>2</w:t></w:r></w:moveTo><w:moveFrom w:id="8" w:author="Ann Counsel"><w:r><w:delText>7</w:delText></w:r></w:moveFrom>{}</w:p></w:hdr>"#,
        run("Status: "),
        ins(1, "Ann Counsel", "DRAFT"),
        del(2, "Ann Counsel", "FINAL"),
        run(" {>>x&lt;&lt;} Page "),
        run(" of 3")
    );
    let bytes = common::docx::docx_with_sect(
        &para("Body"),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let text = |view: &str| {
        header_lines(view)
            .into_iter()
            .find(|l| l.starts_with("  default: {id: header1"))
            .map_or_else(|| view.to_string(), str::to_string)
    };
    assert_eq!(
        text(&agent(&bytes)),
        r#"  default: {id: header1, text: "Status: {++DRAFT++}{header1.r1+2 @AC 2026-10-01T09:00:00Z}{--FINAL--}{header1.r1+2 @AC 2026-10-01T09:00:00Z} {\\>>x<<\\} Page {++2++}{header1.r9 @AC}{--7--}{header1.r8 @AC} of 3", style: Normal}"#
    );
    assert_eq!(
        text(&agent_with(&bytes, TrackChanges::Accept, true)),
        r#"  default: {id: header1, text: "Status: DRAFT {>>x<<} Page 2 of 3", style: Normal}"#
    );
    assert_eq!(
        text(&agent_with(&bytes, TrackChanges::Reject, true)),
        r#"  default: {id: header1, text: "Status: FINAL {>>x<<} Page 7 of 3", style: Normal}"#
    );
}

/// `--by AS` keeps the block whose only mark of AS is a hidden reply:
/// the id line names it `c6`.
#[test]
fn changed_by_finds_a_hidden_reply_by_its_comment_id() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let by = ok(
        &[
            "read",
            "received.docx",
            "--changed",
            "--by",
            "AS",
            "--comments",
            "none",
            "--no-page-markers",
        ],
        dir.path(),
    );
    assert!(
        by.contains("\nrange: changed by @AS (p5) of p0-p20\n"),
        "{by}"
    );
    assert!(by.contains("<!-- p5 comments c5 c6 -->"), "{by}");
}

#[test]
fn handles_keep_only_letters_and_digits() {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}</w:comments>"#,
        comment(
            9,
            "Ann Counsel",
            "A <<}C",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "One."
        ),
        comment(
            10,
            "Jo Doe",
            "+|@",
            "2026-10-01T09:00:00Z",
            "0B0B0B0B",
            "Two."
        )
    );
    let body_xml = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}<w:commentRangeEnd w:id="9"/>{}<w:commentRangeStart w:id="10"/>{}<w:commentRangeEnd w:id="10"/>{}</w:p>"#,
        run("a"),
        reference(9),
        run("b"),
        reference(10)
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let out = agent(&bytes);
    assert!(out.contains("{>>c9 @AC: One.<<}"), "{out}");
    // Initials with no letter or digit fall back to the name's initials.
    assert!(out.contains("{>>c10 @JD: Two.<<}"), "{out}");
}

const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
const EXTENDED_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml";
const EXTENDED_REL: &str =
    "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";
const W14: &str = r#"xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
const W15: &str = r#"xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml""#;

fn comment(id: u32, author: &str, initials: &str, date: &str, para_id: &str, text: &str) -> String {
    format!(
        r#"<w:comment w:id="{id}" w:author="{author}" w:initials="{initials}" w:date="{date}"><w:p w14:paraId="{para_id}"><w:r><w:annotationRef/></w:r><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
    )
}

fn reference(id: u32) -> String {
    format!(r#"<w:r><w:commentReference w:id="{id}"/></w:r>"#)
}

fn commented_docx(extended: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}{}</w:comments>"#,
        comment(
            5,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "11A5D0F2",
            "Cap in Delaware?"
        ),
        comment(
            6,
            "Arthur Souza Rodrigues",
            "AS",
            "2026-10-09T16:13:00Z",
            "33767091",
            "Disagree."
        ),
        comment(
            11,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "214DA01E",
            "Add survival?"
        )
    );
    let extended_xml =
        format!(r#"<w15:commentsEx xmlns:w="{W_NS}" {W15}>{extended}</w15:commentsEx>"#);
    let body_xml = format!(
        r#"<w:p>{}<w:commentRangeStart w:id="5"/><w:commentRangeStart w:id="6"/>{}<w:commentRangeEnd w:id="5"/>{}<w:commentRangeEnd w:id="6"/>{}</w:p><w:p>{}<w:commentRangeStart w:id="11"/><w:commentRangeEnd w:id="11"/>{}{}</w:p>"#,
        run("Fee. "),
        run("Late amounts accrue interest."),
        reference(5),
        reference(6),
        run("Keep it secret"),
        reference(11),
        run(".")
    );
    common::docx::docx_with(
        &body_xml,
        &[
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
            Part {
                name: "word/commentsExtended.xml",
                content_type: EXTENDED_CT,
                rel_type: EXTENDED_REL,
                xml: &extended_xml,
            },
        ],
    )
}

const THREADED: &str = r#"<w15:commentEx w15:paraId="11A5D0F2" w15:done="0"/><w15:commentEx w15:paraId="33767091" w15:paraIdParent="11A5D0F2" w15:done="0"/><w15:commentEx w15:paraId="214DA01E" w15:done="0"/>"#;

#[test]
fn comments_carry_ids_handles_and_thread_parents() {
    assert_eq!(
        body(&agent(&commented_docx(THREADED))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nFee. {==Late amounts accrue interest.==}{>>c5 @AC: Cap in Delaware?<<}{>>c6 @AS re c5: Disagree.<<}\n\n<!-- p1 -->\nKeep it secret{>>c11 @AC: Add survival?<<}.\n"
    );
}

#[test]
fn two_independent_comments_on_one_anchor_are_not_a_thread() {
    let independent = THREADED.replace(r#" w15:paraIdParent="11A5D0F2""#, "");
    let out = body(&agent(&commented_docx(&independent))).to_string();
    assert!(
        out.contains("{>>c5 @AC: Cap in Delaware?<<}{>>c6 @AS: Disagree.<<}"),
        "{out}"
    );
}

#[test]
fn a_resolved_thread_says_so_on_its_root() {
    let resolved = THREADED.replacen(
        r#"w15:paraId="11A5D0F2" w15:done="0""#,
        r#"w15:paraId="11A5D0F2" w15:done="1""#,
        1,
    );
    let out = body(&agent(&commented_docx(&resolved))).to_string();
    assert!(
        out.contains("{>>c5 @AC resolved: Cap in Delaware?<<}{>>c6 @AS re c5: Disagree.<<}"),
        "{out}"
    );
}

#[test]
fn hidden_comments_are_listed_on_the_id_line() {
    assert_eq!(
        body(&agent_with(
            &commented_docx(THREADED),
            TrackChanges::All,
            false
        )),
        "<!-- page 1 of 1 -->\n\n<!-- p0 comments c5 c6 -->\nFee. Late amounts accrue interest.\n\n<!-- p1 comments c11 -->\nKeep it secret.\n"
    );
}

/// One comment, #9 by Ann Counsel, around `body_xml`.
fn one_comment_docx(body_xml: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            9,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Check."
        )
    );
    common::docx::docx_with(
        body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

#[test]
fn hidden_comments_in_a_cell_go_on_the_table_line_not_the_next_paragraph() {
    let body_xml = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p>{}</w:p></w:tc><w:tc><w:p><w:commentRangeStart w:id="9"/>{}<w:commentRangeEnd w:id="9"/>{}</w:p></w:tc></w:tr></w:tbl>{}"#,
        run("a"),
        run("b"),
        reference(9),
        para("After")
    );
    let out = agent_with(&one_comment_docx(&body_xml), TrackChanges::All, false);
    assert!(
        body(&out).starts_with(
            "<!-- page 1 of 1 -->\n\n<!-- t0 1x2, cells p0-p1 by row, widths 2.778in 2.778in, width auto, comments c9 in p1 -->\n"
        ),
        "{out}"
    );
    assert!(body(&out).ends_with("<!-- p2 -->\nAfter\n"), "{out}");
}

#[test]
fn an_empty_paragraph_keeps_its_line_for_a_hidden_comment() {
    let body_xml = format!(
        r#"{}<w:p><w:commentRangeStart w:id="9"/><w:commentRangeEnd w:id="9"/>{}</w:p>{}"#,
        para("Before"),
        reference(9),
        para("After")
    );
    assert_eq!(
        body(&agent_with(
            &one_comment_docx(&body_xml),
            TrackChanges::All,
            false
        )),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBefore\n\n<!-- p1 empty, comments c9 -->\n\n<!-- p2 -->\nAfter\n"
    );
}

#[test]
fn an_inserted_empty_paragraph_keeps_its_line_for_the_mark() {
    let body_xml = format!(
        r#"{}<w:p><w:pPr><w:rPr><w:ins w:id="3" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr></w:p><w:p/>{}"#,
        para("Before"),
        para("After")
    );
    assert_eq!(
        body(&agent(&docx(&body_xml))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBefore\n\n<!-- p1 -->\n¶{r3 @AC}\n\n<!-- p2 empty -->\n\n<!-- p3 -->\nAfter\n"
    );
}

#[test]
fn accept_all_keeps_indices_comments_and_lists_the_revisions_applied() {
    let p0 = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("Deliver "),
        ins(0, "Ann Counsel", "quarterly"),
        run(" reports "),
        del(1, "Ann Counsel", "weekly")
    );
    let p1 = format!(
        "<w:p>{}{}{}</w:p>",
        run("Within "),
        ins(4, "Ann Counsel", "forty-five"),
        del(3, "Ann Counsel", "thirty")
    );
    let bytes = docx(&format!("{p0}{p1}"));
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Accept, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev r0 @AC; r1 @AC -->\nDeliver quarterly reports\n\n<!-- p1 rev r4+3 @AC -->\nWithin forty-five\n"
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Reject, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev r0 @AC; r1 @AC -->\nDeliver  reports weekly\n\n<!-- p1 rev r4+3 @AC -->\nWithin thirty\n"
    );
}

#[test]
fn resolved_views_list_formatting_and_paragraph_mark_revisions() {
    let body_xml = concat!(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>Now bold</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:rPr><w:del w:id="6" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr><w:r><w:t>Kept</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>Last</w:t></w:r></w:p>"#,
    );
    let out = agent_with(&docx(body_xml), TrackChanges::Accept, true);
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev r5 @AC -->\n**Now bold**\n\n<!-- p1 rev r6 @AC -->\nKeptLast\n"
    );
}

#[test]
fn a_paragraph_joined_on_resolution_keeps_both_paragraphs_revisions() {
    let body_xml = format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="10" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr>{}</w:p><w:p>{}{}</w:p>"#,
        run("Split "),
        run("kept "),
        ins(11, "Ann Counsel", "added")
    );
    let out = agent_with(&docx(&body_xml), TrackChanges::Reject, true);
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev r10 @AC; r11 @AC -->\nSplit kept\n"
    );
}

#[test]
fn rev_counts_revisions_inside_links_content_controls_and_fields() {
    let body_xml = format!(
        r#"<w:p>{}<w:hyperlink w:anchor="x">{}</w:hyperlink><w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt><w:fldSimple w:instr=" REF x ">{}</w:fldSimple></w:p>"#,
        ins(7, "Ann Counsel", "bold "),
        ins(8, "Ann Counsel", "linked "),
        ins(9, "Ann Counsel", "boxed "),
        ins(10, "Ann Counsel", "field")
    );
    let out = agent_with(&docx(&body_xml), TrackChanges::Accept, true);
    // The link's mark joins the one before it, and the change is named
    // once; the content control parts it from the rest.
    assert!(
        body(&out)
            .starts_with("<!-- page 1 of 1 -->\n\n<!-- p0 rev r7+8 @AC; r9 @AC; r10 @AC -->\n"),
        "{out}"
    );
}

#[test]
fn accept_all_keeps_comments_in_the_agent_view() {
    let out = body(&agent_with(
        &commented_docx(THREADED),
        TrackChanges::Accept,
        true,
    ))
    .to_string();
    assert!(
        out.contains("{>>c5 @AC: Cap in Delaware?<<}{>>c6 @AS re c5: Disagree.<<}"),
        "{out}"
    );
}

#[test]
fn underline_renders_inside_bold_in_the_agent_view_only() {
    let p = r#"<w:p><w:r><w:t xml:space="preserve">keep it </w:t></w:r><w:r><w:rPr><w:b/><w:u w:val="single"/></w:rPr><w:t>secret</w:t></w:r><w:r><w:rPr><w:u w:val="none"/></w:rPr><w:t>.</w:t></w:r></w:p>"#;
    assert_eq!(
        body(&agent(&docx(p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nkeep it **<u>secret</u>**.\n"
    );
    let legacy = docx_to_markdown(&docx(p), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert_eq!(legacy, "keep it **secret**.\n");
}

#[test]
fn highlight_and_strike_render_in_the_agent_view() {
    // Spec §3: `<mark>x</mark>` is a highlight of any colour and `~~x~~` a
    // strike.
    // Text that holds `==` or `~~` itself is escaped, so it stays text.
    let p = r#"<w:p><w:r><w:t xml:space="preserve">pay </w:t></w:r><w:r><w:rPr><w:b/><w:highlight w:val="cyan"/></w:rPr><w:t>now</w:t></w:r><w:r><w:t xml:space="preserve">, </w:t></w:r><w:r><w:rPr><w:strike/></w:rPr><w:t>later</w:t></w:r><w:r><w:t xml:space="preserve">, </w:t></w:r><w:r><w:rPr><w:highlight w:val="none"/><w:strike w:val="0"/></w:rPr><w:t>a==b ~~c</w:t></w:r></w:p>"#;
    let full = agent(&docx(p));
    let view = body(&full);
    assert_eq!(
        view,
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\npay **<mark class=\"hl-cyan\">now</mark>**, ~~later~~, a\\=\\=b \\~\\~c\n"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

#[test]
fn link_entity_and_heading_syntax_in_text_reads_back_as_text() {
    // Text that looks like Markdown stays text through a round trip: an
    // entity reference, a link, an image, a reference definition, and a
    // heading whose text ends in `#` (a closing sequence otherwise).
    let p = [
        r#"<w:p><w:r><w:t>&amp;copy; and &amp;#35; at AT&amp;T</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>see [a](http://b) and ![i](p.png)</w:t></w:r></w:p>"#,
        r#"<w:p><w:r><w:t>[r]: /u</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t xml:space="preserve">C #</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>#</w:t></w:r></w:p>"#,
    ]
    .concat();
    let full = agent(&docx(&p));
    let view = body(&full);
    assert_eq!(
        view,
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n\\&copy; and \\&#35; at AT&T\n\n<!-- p1 -->\nsee [a\\](http://b) and ![i\\](p.png)\n\n<!-- p2 -->\n[r\\]: /u\n\n<!-- p3 -->\n# C \\#\n\n<!-- p4 -->\n# \\#\n"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

#[test]
fn code_reads_as_code_spans_and_fences() {
    // `VerbatimChar` runs print as code spans and a `SourceCode` paragraph
    // as a fence: their text stays as written, unescaped and with its
    // spaces, so it reads back the same. A fence's content is literal, so
    // a marked run's marks print nowhere inside it.
    let code = |t: &str| {
        format!(
            r#"<w:r><w:rPr><w:rStyle w:val="VerbatimChar"/></w:rPr><w:t xml:space="preserve">{t}</w:t></w:r>"#
        )
    };
    let p = [
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">Run </w:t></w:r>{}<w:r><w:t xml:space="preserve"> or </w:t></w:r>{}<w:r><w:t>.</w:t></w:r></w:p>"#,
            code("a*b  c"),
            code("x`y")
        ),
        r#"<w:p><w:pPr><w:pStyle w:val="SourceCode"/></w:pPr><w:r><w:t>fn main() {</w:t></w:r><w:r><w:br/></w:r><w:r><w:t xml:space="preserve">    println!("*hi*");</w:t></w:r><w:r><w:br/></w:r><w:r><w:t>}</w:t></w:r><w:r><w:br/></w:r><w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>underlined</w:t></w:r></w:p>"#.to_string(),
    ]
    .concat();
    let full = agent(&docx(&p));
    let view = body(&full);
    assert_eq!(
        view,
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nRun `a*b  c` or ``x`y``.\n\n<!-- p1 SourceCode -->\n```\nfn main() {\n    println!(\"*hi*\");\n}\nunderlined\n```\n"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

/// Literal CriticMarkup delimiters in a code block are text, not changes
/// (#518): the reader breaks them inside the fence, where Markdown reads
/// no backslash escape, and the writer unbreaks them there. Code that
/// already holds a backslash beside a delimiter comes back as written.
#[test]
fn literal_critic_delimiters_in_a_code_block_stay_text() {
    let code = |t: &str| {
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="SourceCode"/></w:pPr><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#,
            common::docx::esc(t)
        )
    };
    let texts = [
        "fenced {++x++} code",
        r"brace {\++x++} tail",
        r"double {\\++x++} tail",
        r"sub a~>b and a\~>b end",
        r"path C:\path\x regex \d+ latex a\\b end",
    ];
    let bytes = docx(&texts.iter().map(|t| code(t)).collect::<String>());
    let full = agent(&bytes);
    let view = body(&full).to_string();
    assert!(view.contains("fenced {\\++x++\\} code\n"), "{view}");
    assert!(view.contains("brace {\\\\\\++x++\\} tail\n"), "{view}");
    assert!(view.contains("double {\\\\\\\\\\++x++\\} tail\n"), "{view}");
    assert!(view.contains("sub a~\\>b and a\\~\\>b end\n"), "{view}");
    assert!(
        view.contains("path C:\\path\\x regex \\d+ latex a\\\\b end\n"),
        "{view}"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert!(
        jubarte::changes::list_changes(&back).unwrap().is_empty(),
        "literal delimiters invented no change"
    );
    assert_eq!(
        body(&agent(&back)),
        body(&full),
        "the view is a fixed point"
    );
    // The plain conversion prints the same fences, without the id lines.
    let plain = docx_to_markdown(&bytes, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(plain.contains("fenced {\\++x++\\} code\n"), "{plain}");
    let plain_back = markdown_to_docx(&plain, &DocxOptions::default())
        .unwrap()
        .docx;
    assert!(
        jubarte::changes::list_changes(&plain_back)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        docx_to_markdown(&plain_back, &MarkdownOptions::default())
            .unwrap()
            .markdown,
        plain,
        "the plain view is a fixed point"
    );
}

/// Hand-written Markdown's backslashes inside a code fence are not the
/// reader's escape (#518, second round): `C:\\path`, a regex `\\d+`, LaTeX
/// `\\` and `a\\b` a human wrote reach Word byte-identical, and no even
/// backslash run before a delimiter opens a change.
#[test]
fn hand_written_backslashes_in_a_fence_reach_word_as_written() {
    for hand in [
        "C:\\\\path\\\\x",
        "\\\\d+",
        "a\\\\b",
        "\\\\\\\\",
        "even {\\\\++ and ++\\\\} and ~\\\\>",
    ] {
        let markdown = format!("```\n{hand}\n```\n");
        let docx = markdown_to_docx(&markdown, &DocxOptions::default())
            .unwrap()
            .docx;
        assert!(
            jubarte::changes::list_changes(&docx).unwrap().is_empty(),
            "{hand}"
        );
        let paragraphs = jubarte::inspect::paragraphs(&docx).unwrap();
        assert_eq!(paragraphs.len(), 1, "{hand}");
        assert_eq!(paragraphs[0].text, hand, "{hand}");
        assert_eq!(paragraphs[0].style.as_deref(), Some("SourceCode"));
        // And that document reads back as a fence that names it from then
        // on: its own view is a fixed point. (The hand-written fence is
        // not: an even run before a delimiter becomes the reader's odd
        // one, which names the same text.)
        let view = docx_to_markdown(&docx, &MarkdownOptions::default())
            .unwrap()
            .markdown;
        let docx = markdown_to_docx(&view, &DocxOptions::default())
            .unwrap()
            .docx;
        assert_eq!(jubarte::inspect::paragraphs(&docx).unwrap()[0].text, hand);
        assert_eq!(
            docx_to_markdown(&docx, &MarkdownOptions::default())
                .unwrap()
                .markdown,
            view,
            "{hand}"
        );
    }
}

/// The fence rule stays: a tracked change inside a code block is written
/// as CriticMarkup there and read back as the same change (#518).
#[test]
fn tracked_changes_inside_a_code_block_still_round_trip() {
    let ins = r#"<w:ins w:id="7" w:author="Ana" w:date="2026-03-02T10:00:00Z"><w:r><w:t>new {++x++}</w:t></w:r></w:ins>"#;
    let bytes = docx(&format!(
        r#"<w:p><w:pPr><w:pStyle w:val="SourceCode"/></w:pPr><w:r><w:t xml:space="preserve">keep </w:t></w:r>{ins}<w:r><w:t xml:space="preserve"> old</w:t></w:r></w:p>"#
    ));
    let full = agent(&bytes);
    let view = body(&full);
    assert!(
        view.contains("keep {++new {\\++x++\\}++}{r7 @A} old"),
        "{view}"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    let listed = jubarte::changes::list_changes(&back).unwrap();
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert_eq!(listed[0].text, "new {++x++}");
    assert_eq!(body(&agent(&back)), view);
}

/// CommonMark reads a line indented four or more spaces as indented code,
/// not a fence and not prose (#518): its text is verbatim, so the fence
/// unwind must not eat a backslash there and CriticMarkup opens nothing.
#[test]
fn indented_code_keeps_its_text_verbatim() {
    for (hand, text) in [
        ("    ```\n    a{\\++b\n    ```\n", "```\na{\\++b\n```"),
        ("     ```\n     x++\\}y\n     ```\n", " ```\n x++\\}y\n ```"),
        ("    a{++b++}c\n", "a{++b++}c"),
    ] {
        let docx = markdown_to_docx(hand, &DocxOptions::default())
            .unwrap()
            .docx;
        assert!(
            jubarte::changes::list_changes(&docx).unwrap().is_empty(),
            "{hand}"
        );
        let paragraphs = jubarte::inspect::paragraphs(&docx).unwrap();
        assert_eq!(paragraphs.len(), 1, "{hand}");
        assert_eq!(paragraphs[0].text, text, "{hand}");
        assert_eq!(paragraphs[0].style.as_deref(), Some("SourceCode"), "{hand}");
        // The document reads back as the fence that names it, a fixed point.
        let view = docx_to_markdown(&docx, &MarkdownOptions::default())
            .unwrap()
            .markdown;
        let back = markdown_to_docx(&view, &DocxOptions::default())
            .unwrap()
            .docx;
        assert_eq!(jubarte::inspect::paragraphs(&back).unwrap()[0].text, text);
        assert_eq!(
            docx_to_markdown(&back, &MarkdownOptions::default())
                .unwrap()
                .markdown,
            view,
            "{hand}"
        );
    }
}

/// A comment anchored in a code block prints its note inside the fence,
/// where only the fence's escape is unwound (#518): the note keeps its
/// backslashes and markup characters across round trips instead of
/// doubling them, so the view and the comment are fixed points.
#[test]
fn a_note_anchored_in_a_code_block_round_trips_its_text() {
    let body_xml = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="SourceCode"/></w:pPr><w:r><w:t xml:space="preserve">code </w:t></w:r><w:commentRangeStart w:id="1"/><w:r><w:t xml:space="preserve">line one</w:t></w:r><w:commentRangeEnd w:id="1"/>{}<w:r><w:t xml:space="preserve"> more</w:t></w:r></w:p>"#,
        reference(1)
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            1,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            r"note with {++x++} and \ and ++}"
        )
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let note_texts = |docx: &[u8]| {
        common::docx::part_string(docx, "word/comments.xml")
            .unwrap()
            .split("<w:t")
            .skip(1)
            .filter_map(|t| {
                t.split_once("</")
                    .and_then(|(head, _)| head.split_once('>'))
            })
            .map(|(_, text)| text.to_string())
            .collect::<Vec<_>>()
    };
    let mut markdown = docx_to_markdown(&bytes, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(markdown.contains("code {==line one==}{>>"), "{markdown}");
    for generation in 0..3 {
        let docx = markdown_to_docx(&markdown, &DocxOptions::default())
            .unwrap()
            .docx;
        assert_eq!(
            note_texts(&docx),
            [r"note with {++x++} and \ and ++}"],
            "generation {generation}"
        );
        let next = docx_to_markdown(&docx, &MarkdownOptions::default())
            .unwrap()
            .markdown;
        assert_eq!(next, markdown, "generation {generation}");
        markdown = next;
    }
}

/// `--commonmark` escapes CriticMarkup and Markdown syntax as the agent
/// view does (#519): Word text that reads like markup round-trips through
/// both views as itself, with no change invented.
#[test]
fn commonmark_escapes_critic_and_markdown_syntax() {
    let texts = [
        "{++a++}",
        r"{\++a++}",
        "- not a list",
        "1. not a list",
        "# not a heading",
        "> not a quote",
        "*not em*",
        "`not code`",
        "[not](link)",
        "<b>not html</b>",
    ];
    let bytes = docx(&texts.iter().map(|t| para(t)).collect::<String>());
    let escaped = [
        "{\\++a++\\}",
        "{\\\\++a++\\}",
        "\\- not a list",
        "1\\. not a list",
        "\\# not a heading",
        "\\> not a quote",
        "\\*not em\\*",
        "\\`not code\\`",
        "[not\\](link)",
        "\\<b>not html\\</b>",
    ];
    for options in [&MarkdownOptions::default(), &agent_defaults()] {
        let markdown = docx_to_markdown(&bytes, options).unwrap().markdown;
        for want in escaped {
            assert!(markdown.contains(&format!("{want}\n")), "{{markdown}}");
        }
        let back = markdown_to_docx(&markdown, &DocxOptions::default())
            .unwrap()
            .docx;
        assert!(
            jubarte::changes::list_changes(&back).unwrap().is_empty(),
            "no change from literal markup"
        );
        // The header names the file's bytes, so the bodies are compared.
        assert_eq!(
            body(&docx_to_markdown(&back, options).unwrap().markdown),
            body(&markdown),
            "the view is a fixed point"
        );
    }
}

#[test]
fn body_fields_read_as_code_then_result() {
    // Spec §3: a body field prints `{CODE}result{/}`, a hyperlink field
    // stays a link, literal braces are escaped, and a field that runs past
    // its paragraph (a table of contents) prints its code at the start of
    // its first paragraph and `{/}` in its last.
    let begin = r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#;
    let separate = r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#;
    let end = r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#;
    let instr = |code: &str| {
        format!(r#"<w:r><w:instrText xml:space="preserve"> {code} </w:instrText></w:r>"#)
    };
    let p = [
        format!(
            r#"<w:p>{}{begin}{}{separate}{}{end}{}<w:fldSimple w:instr=" REF _Ref1 \h ">{}</w:fldSimple>{}</w:p>"#,
            run("Begins "),
            instr(r#"DATE \@ "MMMM d, yyyy""#),
            run("March 1, 2026"),
            run(" as "),
            run("Section 2"),
            run(" says, {Client Name}, {page} and {/}.")
        ),
        format!(
            r#"<w:p>{begin}{}{separate}{}{end}</w:p>"#,
            instr(r#"HYPERLINK "https://a.example/""#),
            run("site")
        ),
        format!(
            r#"<w:p>{begin}{}{separate}{}</w:p><w:p>{}{end}</w:p>"#,
            instr(r#"TOC \o "1-3""#),
            run("Intro 1"),
            run("Terms 2")
        ),
    ]
    .concat();
    let full = agent(&docx(&p));
    let view = body(&full);
    assert_eq!(
        view,
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBegins {DATE \\@ \"MMMM d, yyyy\"}March 1, 2026{/} as {REF _Ref1 \\h}Section 2{/} says, {Client Name}, \\{page} and \\{/}.\n\n<!-- p1 -->\n[site](https://a.example/)\n\n<!-- p2 -->\n{TOC \\o \"1-3\"}Intro 1\n\n<!-- p3 -->\nTerms 2{/}\n"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

#[test]
fn a_monospace_run_is_code_unless_its_paragraph_is_monospace_too() {
    // Spec §3: `` `x` `` is a run in a monospace font. A paragraph whose
    // own font is monospace (a contract set in Courier) is not code, and
    // neither are its runs.
    let mono = |font: &str, t: &str| {
        format!(
            r#"<w:r><w:rPr><w:rFonts w:ascii="{font}" w:hAnsi="{font}"/></w:rPr><w:t xml:space="preserve">{t}</w:t></w:r>"#
        )
    };
    let body_xml = [
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">Call </w:t></w:r>{}<w:r><w:t xml:space="preserve"> with </w:t></w:r>{}<w:r><w:t>.</w:t></w:r></w:p>"#,
            mono("Courier New", "open()"),
            mono("Consolas", "a*b")
        ),
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Typed"/></w:pPr><w:r><w:t xml:space="preserve">Typed </w:t></w:r>{}</w:p>"#,
            mono("Courier New", "clause")
        ),
    ]
    .concat();
    let styles = Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &format!(
            r#"<w:styles xmlns:w="{W_NS}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Typed"><w:name w:val="Typed"/><w:basedOn w:val="Normal"/><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/></w:rPr></w:style></w:styles>"#
        ),
    };
    let full = agent(&common::docx::docx_with(&body_xml, &[styles]));
    let view = body(&full);
    assert_eq!(
        view,
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nCall `open()` with `a*b`.\n\n<!-- p1 Typed -->\nTyped clause\n"
    );
}

const CORE_CT: &str = "application/vnd.openxmlformats-package.core-properties+xml";

fn core(xml: &str) -> Part<'_> {
    Part {
        name: "docProps/core.xml",
        content_type: CORE_CT,
        rel_type: "",
        xml,
    }
}

#[test]
fn header_counts_revisions_comments_and_authors() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("a "),
        ins(2, "Ann Counsel", "c"),
        del(1, "Ann Counsel", "b"),
        ins(3, "John Doe", "d")
    );
    let bytes = common::docx::docx_with(
        &p,
        &[core(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:creator>Jane Owner</dc:creator><cp:lastModifiedBy>Someone Else</cp:lastModifiedBy></cp:coreProperties>"#,
        )],
    );
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert_eq!(lines[0], "source: sample.docx");
    assert_eq!(
        lines[1],
        format!("sha256: {}", jubarte::inspect::source_sha256(&bytes))
    );
    assert_eq!(
        lines[2],
        "view: tracked                      # revisions as CriticMarkup, comments inline"
    );
    assert_eq!(
        lines[3],
        "track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it"
    );
    assert_eq!(
        lines[4],
        "revisions: 3                       # 3 marks shown as 2 changes: 1 insertion, 1 substitution"
    );
    assert_eq!(lines[5], "comments: 0");
    assert_eq!(lines[6], "authors:");
    assert_eq!(lines[7], "  document_owner: Jane Owner       # dc:creator");
    assert_eq!(
        lines[8],
        "  AC: Ann Counsel                  # 2 revisions, 2026-10-01T09:00:00Z"
    );
    assert_eq!(
        lines[9],
        "  JD: John Doe                     # 1 revision, 2026-10-01T09:00:00Z"
    );
    assert_eq!(
        lines[10],
        "body: p0-p0, 0 tables, 1 page      # page count estimated from breaks"
    );
}

#[test]
fn header_without_core_properties_names_no_owner_and_reads_tracking_from_settings() {
    let settings = Part {
        name: "word/settings.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
        xml: &format!(r#"<w:settings xmlns:w="{W_NS}"><w:trackRevisions/></w:settings>"#),
    };
    let out = agent(&common::docx::docx_with(&para("x"), &[settings]));
    let lines = header_lines(&out);
    assert_eq!(
        lines[3],
        "track_changes: on                  # w:trackRevisions set; edits are tracked"
    );
    assert_eq!(lines[4], "revisions: 0");
    assert_eq!(lines[6], "authors:");
    assert_eq!(
        lines[7],
        "  document_owner: none             # no docProps/core.xml"
    );
}

#[test]
fn header_view_lines_for_hidden_comments_accept_and_reject() {
    let bytes = commented_docx(THREADED);
    assert_eq!(
        header_lines(&agent_with(&bytes, TrackChanges::All, false))[2],
        "view: tracked, comments hidden     # 2 threads open (3 comments); carrying paragraphs are marked"
    );
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(0, "Ann Counsel", "b"));
    assert_eq!(
        header_lines(&agent_with(&docx(&p), TrackChanges::Accept, true))[2],
        "view: accept-all                   # 1 revision by AC shown as accepted; file unchanged"
    );
    assert_eq!(
        header_lines(&agent_with(&docx(&p), TrackChanges::Reject, true))[2],
        "view: reject-all                   # 1 revision by AC shown as rejected; file unchanged"
    );
    assert_eq!(
        header_lines(&agent(&bytes))[5],
        "comments: 2 threads open, style Normal  # 3 comments: c5 (+ reply c6), c11"
    );
}

#[test]
fn header_prints_latest_for_several_timestamps_and_bare_for_one() {
    for (comment_date, latest) in [
        ("2026-10-17T16:59:00Z", "2026-10-17T16:59:00Z"),
        ("2026-10-02T11:00:00Z", "2026-10-03T14:05:00Z"),
    ] {
        let p = format!(
            "<w:p>{} {} {}{}</w:p>",
            ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "one"),
            ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "two"),
            ins_at(3, "Bo Chen", "2026-10-17T04:56:00Z", "once"),
            reference(5),
        );
        let comments = format!(
            r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
            comment(5, "Ann Counsel", "AC", comment_date, "11A5D0F2", "Check.")
        );
        let bytes = common::docx::docx_with(
            &p,
            &[Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            }],
        );
        let out = agent(&bytes);
        let lines = header_lines(&out);
        assert_eq!(
            lines[8],
            format!("  AC: Ann Counsel                  # 2 revisions, 1 comment, latest {latest}")
        );
        assert_eq!(
            lines[9],
            "  BC: Bo Chen                      # 1 revision, 2026-10-17T04:56:00Z"
        );
        assert!(!lines.iter().any(|line| line.contains("..")), "{out}");
    }
}

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

fn hdr(text: &str, jc: Option<&str>) -> String {
    let ppr = jc.map_or(String::new(), |j| {
        format!(r#"<w:pPr><w:jc w:val="{j}"/></w:pPr>"#)
    });
    format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p>{ppr}<w:r><w:t>{text}</w:t></w:r></w:p></w:hdr>"#)
}

#[test]
fn header_describes_page_setup_styles_headers_and_footers() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style><w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/></w:tblBorders></w:tblPr></w:style></w:styles>"#
    );
    let header1 = hdr("SIGNATURE PAGE", Some("right"));
    let header2 =
        format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>DRAFT</w:t></w:r></w:p><w:p/></w:hdr>"#);
    let footer1 = format!(
        r#"<w:ftr xmlns:w="{W_NS}"><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:ftr>"#
    );
    let footer2 = format!(
        r#"<w:ftr xmlns:w="{W_NS}"><w:p><w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p></w:ftr>"#
    );
    let body_xml = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>{}<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/></w:tblPr><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        para("x")
    );
    // Part relationship ids are rIdX0.. in order (tests/common/docx.rs:101).
    let sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/><w:footerReference w:type="even" r:id="rIdX3"/><w:footerReference w:type="default" r:id="rIdX4"/><w:headerReference w:type="first" r:id="rIdX2"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/><w:titlePg/></w:sectPr>"#;
    let bytes = common::docx::docx_with_sect_pr(
        &body_xml,
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: &styles,
            },
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header1,
            },
            Part {
                name: "word/header2.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header2,
            },
            Part {
                name: "word/footer1.xml",
                content_type: FOOTER_CT,
                rel_type: FOOTER_REL,
                xml: &footer1,
            },
            Part {
                name: "word/footer2.xml",
                content_type: FOOTER_CT,
                rel_type: FOOTER_REL,
                xml: &footer2,
            },
        ],
        sect,
    );
    let out = agent(&bytes);
    let header = out.split("---\n").nth(1).unwrap();
    let expected = "\
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  \"#\": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
headers:
  first: {id: header2, text: [DRAFT, \"\"], style: Normal}  # page 1 only (different first page)
  default: {id: header1, text: SIGNATURE PAGE, right, style: Normal}
footers:
  first: none                      # page 1 shows no page number
  default: {id: footer2, text: \"{PAGE}1{/}\", style: Normal}
  even: {id: footer1, text: \"{PAGE}1{/}\", style: Normal, inactive}  # defined, but even/odd headers are off
";
    assert!(header.ends_with(expected), "header:\n{header}");
}

#[test]
fn header_page_line_for_a4_landscape_with_uneven_margins_and_a_gutter() {
    let sect = r#"<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="709" w:footer="709" w:gutter="720"/></w:sectPr>"#;
    let out = agent(&common::docx::docx_with_sect_pr(&para("x"), &[], sect));
    assert!(out.contains("\npage: A4 landscape, margins top 1in, right 1.25in, bottom 1in, left 1.25in, header/footer 0.4924in, gutter 0.5in\n"), "{out}");
}

#[test]
fn later_sections_list_their_range_and_what_differs_from_the_first() {
    let main = hdr("MAIN", None);
    let schedule = hdr("SCHEDULE A", None);
    let margins = r#"<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#;
    let body_xml = format!(
        r#"{}<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/>{margins}</w:sectPr></w:pPr><w:r><w:t>End of part one</w:t></w:r></w:p>{}{}"#,
        para("Intro"),
        para("Schedule"),
        para("Rows")
    );
    let sect = format!(
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/><w:cols w:num="2"/>{margins}</w:sectPr>"#
    );
    let bytes = common::docx::docx_with_sect_pr(
        &body_xml,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &main,
            },
            Part {
                name: "word/header2.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &schedule,
            },
        ],
        &sect,
    );
    let out = agent(&bytes);
    let header = out.split("---\n").nth(1).unwrap();
    assert!(header.ends_with("headers:\n  default: {id: header1, text: MAIN, style: Normal}\nsections:\n  2: {p2-p3, page: Letter portrait, margins 1in, header/footer 0.5in, columns 2, headers: {default: {id: header2, text: SCHEDULE A, style: Normal}}}\n"), "header:\n{header}");
    assert!(
        body(&out).contains("<!-- p1 section-break -->\nEnd of part one\n"),
        "{out}"
    );
}

fn four_paragraphs_and_a_table() -> Vec<u8> {
    docx(&format!(
        "{}{}{TABLE_3X2}{}{}",
        para("Zero"),
        para("One"),
        para("Eight"),
        para("Nine")
    ))
}

fn selected(select: Select) -> String {
    agent_options(
        &four_paragraphs_and_a_table(),
        &MarkdownOptions {
            select: Some(select),
            ..agent_defaults()
        },
    )
}

#[test]
fn head_prints_the_first_blocks_and_a_range_line() {
    let out = selected(Select::Head(2));
    assert!(out.contains("\nbody: p0-p9, 1 table, 1 page       # page count estimated from breaks\nrange: head 2 (p0-p1) of p0-p9\n"), "{out}");
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nZero\n\n<!-- p1 -->\nOne\n"
    );
}

#[test]
fn tail_prints_the_last_blocks_with_the_page_they_are_on() {
    let out = selected(Select::Tail(2));
    assert!(out.contains("\nrange: tail 2 (p8-p9) of p0-p9\n"), "{out}");
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p8 -->\nEight\n\n<!-- p9 -->\nNine\n"
    );
}

#[test]
fn picks_print_paragraphs_ranges_and_whole_tables_in_document_order() {
    let out = selected(Select::parse("p9, p1, p4").unwrap());
    assert!(out.contains("\nrange: p1, p4, p9 of p0-p9\n"), "{out}");
    assert_eq!(
        body(&out),
        "<!-- page 1 of 1 -->\n\n<!-- p1 -->\nOne\n\n<!-- t0 center 3x2, cells p2-p7 by row, widths 2.778in 2.778in, width auto, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p9 -->\nNine\n"
    );
    let out = selected(Select::parse("t0,p8-").unwrap());
    assert!(out.contains("\nrange: t0, p8-p9 of p0-p9\n"), "{out}");
    assert!(
        body(&out).starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 center 3x2"),
        "{out}"
    );
}

#[test]
fn a_pick_past_the_end_is_an_error() {
    let err = docx_to_markdown(
        &four_paragraphs_and_a_table(),
        &MarkdownOptions {
            select: Some(Select::parse("p12").unwrap()),
            ..agent_defaults()
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("p12 is past the last paragraph p9"), "{err}");
    let err = docx_to_markdown(
        &four_paragraphs_and_a_table(),
        &MarkdownOptions {
            select: Some(Select::Head(0)),
            ..agent_defaults()
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("head needs a count above 0"), "{err}");
}

#[test]
fn a_selection_keeps_the_page_marker_of_a_later_page() {
    let bytes = docx(&format!(
        "{}{}<w:p><w:r><w:lastRenderedPageBreak/><w:t>Two</w:t></w:r></w:p>{}",
        para("Zero"),
        para("One"),
        para("Three")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            select: Some(Select::parse("p3").unwrap()),
            ..agent_defaults()
        },
    );
    assert_eq!(body(&out), "<!-- page 2 of 2 -->\n\n<!-- p3 -->\nThree\n");
}

#[test]
fn cli_read_prints_the_agent_view_with_flags() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let tracked = ok(&["read", "received.docx"], dir.path());
    assert_eq!(
        ok(&["read", "received.docx"], dir.path()),
        tracked,
        "text is an alias of read"
    );
    assert!(
        tracked.starts_with(&format!(
            "---\nsource: received.docx\nsha256: {}\nview: tracked ",
            jubarte::inspect::source_sha256(&std::fs::read(fixture("received.docx")).unwrap())
        )),
        "{tracked}"
    );
    assert!(tracked.contains("# pages from layout\n"), "{tracked}");
    let hidden = ok(&["read", "received.docx", "--comments", "none"], dir.path());
    assert!(
        hidden.contains("view: tracked, comments hidden"),
        "{hidden}"
    );
    let accepted = ok(
        &["read", "received.docx", "--track-changes", "accept"],
        dir.path(),
    );
    assert!(accepted.contains("view: accept-all"), "{accepted}");
    let rejected = ok(
        &["read", "received.docx", "--track-changes", "reject"],
        dir.path(),
    );
    assert!(rejected.contains("view: reject-all"), "{rejected}");
    let fast = ok(&["read", "received.docx", "--no-page-markers"], dir.path());
    assert!(
        fast.contains("# pages from Word's cached layout\n"),
        "{fast}"
    );
    assert!(!fast.contains("<!-- page "), "{fast}");
    let head = ok(&["read", "received.docx", "--head", "3"], dir.path());
    assert!(
        head.contains("\nrange: head 3 (p0-p2) of p0-p20\n"),
        "{head}"
    );
    let picked = ok(&["read", "received.docx", "-p", "p5,t0"], dir.path());
    assert!(picked.contains("\nrange: p5, t0 of p0-p20\n"), "{picked}");
    let dated = ok(&["read", "received.docx", "--dates"], dir.path());
    assert!(dated.contains("{r0 @AC}"), "{dated}");
    let bad = jubarte(&["read", "received.docx", "-p", "p99"], dir.path());
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("p99 is past the last paragraph p20"));
    let both = jubarte(
        &["read", "received.docx", "--head", "2", "--tail", "2"],
        dir.path(),
    );
    assert!(!both.status.success());
}

#[test]
fn received_docx_tracked_view_matches_the_golden() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_golden(
        "received.tracked.md",
        &ok(&["read", "received.docx"], dir.path()),
    );
}

#[test]
fn received_docx_other_views_match_their_goldens() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_golden(
        "received.no-comments.md",
        &ok(&["read", "received.docx", "--comments", "none"], dir.path()),
    );
    assert_golden(
        "received.accept.md",
        &ok(
            &["read", "received.docx", "--track-changes", "accept"],
            dir.path(),
        ),
    );
    assert_golden(
        "received.reject.md",
        &ok(
            &["read", "received.docx", "--track-changes", "reject"],
            dir.path(),
        ),
    );
}

const BASE: &str = r#"<w:p><w:r><w:t>Fees</w:t></w:r></w:p><w:p><w:r><w:t>Client shall pay each invoice within thirty days of receipt.</w:t></w:r></w:p><w:p><w:r><w:t>Late amounts accrue interest at one percent per month.</w:t></w:r></w:p>"#;

fn edited(dir: &Path, plan: &str, out: &str) -> String {
    std::fs::write(dir.join("plan.json"), plan).unwrap();
    ok(
        &["edit", "base.docx", "--plan", "plan.json", "--out-dir", out],
        dir,
    );
    ok(
        &["read", &format!("{out}/redline.docx"), "--no-page-markers"],
        dir,
    )
}

fn captures<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    let re = regex::Regex::new(pattern).unwrap();
    let caps = re
        .captures(text)
        .unwrap_or_else(|| panic!("no match for {pattern} in:\n{text}"));
    (1..caps.len())
        .map(|i| caps.get(i).unwrap().as_str())
        .collect()
}

#[test]
fn an_edit_shows_up_with_the_ids_that_changes_and_reject_take() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let plan = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"thirty","replacement":"forty-five"},
        {"kind":"comment","paragraph":{"index":2},"find":"one percent","text":"Is one percent the statutory cap?"}]}"#;
    let text = edited(dir.path(), plan, "review");
    let ids = captures(
        &text,
        r"<!-- p1 -->\nClient shall pay each invoice within \{~~thirty~>forty-five~~\}\{r(\d+)\+(\d+) @AC\} days of receipt\.\n",
    );
    let (del_id, ins_id) = (ids[0], ids[1]);
    assert!(text.contains("<!-- p2 -->\nLate amounts accrue interest at {==one percent==}{>>c0 @AC: Is one percent the statutory cap?<<} per month.\n"), "{text}");
    assert!(
        text.contains(
            "revisions: 2                       # 2 marks shown as 1 change: 1 substitution"
        ),
        "{text}"
    );
    assert!(
        text.contains("comments: 1 thread open, style Normal  # 1 comment: c0"),
        "{text}"
    );
    assert!(
        text.contains(
            "  AC: Ann Counsel                  # 2 revisions, 1 comment, 2026-10-01T09:00:00Z"
        ),
        "{text}"
    );

    let changes = ok(&["changes", "review/redline.docx", "--json"], dir.path());
    let rows: Vec<serde_json::Value> = changes
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    // The view's substitution is the one replacement `changes` lists, by
    // the same id, with its deletion and insertion as members (plan 11).
    let composite = format!("r{del_id}+{ins_id}");
    let row = rows
        .iter()
        .find(|r| r["id"] == composite)
        .unwrap_or_else(|| panic!("no change {composite} in {changes}"));
    assert_eq!(row["ref"], composite);
    assert_eq!(row["kind"], "replacement");
    assert_eq!(row["text"], "thirty → forty-five");
    let members: Vec<(&str, &str, &str)> = row["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["id"].as_str().unwrap(),
                m["kind"].as_str().unwrap(),
                m["text"].as_str().unwrap(),
            )
        })
        .collect();
    let (del_ref, ins_ref) = (format!("r{del_id}"), format!("r{ins_id}"));
    assert_eq!(
        members,
        [
            (del_ref.as_str(), "deletion", "thirty"),
            (ins_ref.as_str(), "insertion", "forty-five")
        ]
    );

    // The JSON prints the id the view prints, and that id selects it.
    ok(
        &[
            "reject",
            "review/redline.docx",
            "-o",
            "rejected.docx",
            "--force",
            "--id",
            &format!("r{del_id}"),
            "--id",
            &format!("r{ins_id}"),
        ],
        dir.path(),
    );
    // The expired long id is refused, naming the id to use.
    let expired = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "reject",
            "review/redline.docx",
            "-o",
            "expired.docx",
            "--id",
        ])
        .arg(format!("body:rev:{del_id}"))
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(expired.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&expired.stderr);
    assert!(
        stderr.contains(&format!("the settled spelling is r{del_id}")),
        "{stderr}"
    );
    let bad = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "reject",
            "review/redline.docx",
            "-o",
            "bad.docx",
            "--id",
            "c5",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&bad.stderr),
        "error: c5: expected a change id such as r12, r1+2 or header1.r1\n"
    );
    assert!(!dir.path().join("bad.docx").exists());
    // A bare number and 0.12's `#N` are expired (Arthur, 2026-10-13):
    // refused with exit 2 and the settled spelling, nothing written.
    for expired in [del_id.to_string(), format!("#{del_id}")] {
        let refused = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args([
                "reject",
                "review/redline.docx",
                "-o",
                "bad.docx",
                "--id",
                &expired,
            ])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert_eq!(refused.status.code(), Some(2), "{expired}");
        assert_eq!(
            String::from_utf8_lossy(&refused.stderr),
            format!(
                "error: {expired}: expected a change id such as r12, r1+2 or header1.r1; \
                 the settled spelling is r{del_id}\n"
            )
        );
        assert!(!dir.path().join("bad.docx").exists());
    }
    let rejected = ok(&["read", "rejected.docx", "--no-page-markers"], dir.path());
    assert!(
        rejected.contains(
            "<!-- p1 -->\nClient shall pay each invoice within thirty days of receipt.\n"
        ),
        "{rejected}"
    );
    assert!(rejected.contains("\nrevisions: 0\n"), "{rejected}");
    assert!(
        rejected.contains("{>>c0 @AC: Is one percent the statutory cap?<<}"),
        "{rejected}"
    );
}

#[test]
fn a_reply_and_a_resolution_by_a_second_author_thread_under_the_root() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let first = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"comment","paragraph":{"index":2},"find":"one percent","text":"Cap?"}]}"#;
    edited(dir.path(), first, "one");
    std::fs::copy(
        dir.path().join("one/redline.docx"),
        dir.path().join("base.docx"),
    )
    .unwrap();
    // Bob resolves Ann's thread on purpose (#1846).
    let second = r#"{"schema_version":1,"author":"Bob Lee","date":"2026-10-02T10:00:00Z","rewrite_others_comments":true,"operations":[
        {"kind":"reply_comment","comment_id":0,"text":"Yes, in Delaware."},
        {"kind":"resolve_comment","comment_id":0,"done":true}]}"#;
    let text = edited(dir.path(), second, "two");
    assert!(
        text.contains(
            "{==one percent==}{>>c0 @AC resolved: Cap?<<}{>>c1 @BL re c0: Yes, in Delaware.<<}"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "comments: 0 threads open, 1 resolved, style Normal  # 2 comments: c0 (+ reply c1)"
        ),
        "{text}"
    );
    assert!(
        text.contains("  BL: Bob Lee                      # 1 comment, 2026-10-02T10:00:00Z"),
        "{text}"
    );
}

#[test]
fn a_second_author_editing_a_redline_gets_their_own_notes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let first = r#"{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"thirty","replacement":"forty-five"}]}"#;
    edited(dir.path(), first, "one");
    std::fs::copy(
        dir.path().join("one/redline.docx"),
        dir.path().join("base.docx"),
    )
    .unwrap();
    let second = r#"{"schema_version":1,"author":"John Doe","date":"2026-10-02T10:00:00Z","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":{"index":1},"find":"receipt","replacement":"the invoice"}]}"#;
    let text = edited(dir.path(), second, "two");
    captures(
        &text,
        r"\{~~thirty~>forty-five~~\}\{r\d+\+\d+ @AC\} days of \{~~receipt~>the invoice~~\}\{r\d+\+\d+ @JD\}\.",
    );
    assert!(
        text.contains(
            "revisions: 4                       # 4 marks shown as 2 changes: 2 substitutions"
        ),
        "{text}"
    );
}

fn marked_docx() -> Vec<u8> {
    docx(&format!(
        r#"{}<w:p><w:r><w:t xml:space="preserve">Pay in </w:t></w:r>{}<w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>{}<w:p><w:r><w:t xml:space="preserve">Fee: </w:t></w:r>{}</w:p>"#,
        para("Quiet"),
        ins(0, "Ann Counsel", "ten"),
        para("Also quiet"),
        del(1, "Bob Lee", "waived"),
    ))
}

fn changed(by: Option<&str>) -> MarkdownOptions {
    MarkdownOptions {
        select: Some(Select::Changed {
            by: by.map(str::to_string),
        }),
        ..agent_defaults()
    }
}

#[test]
fn changed_keeps_the_blocks_with_marks_by_anyone_or_by_one_author() {
    let bytes = marked_docx();
    let all = agent_options(&bytes, &changed(None));
    assert!(
        all.contains("\nrange: changed (p1, p3) of p0-p3\n"),
        "{all}"
    );
    assert_eq!(
        body(&all),
        "<!-- page 1 of 1 -->\n\n<!-- p1 -->\nPay in {++ten++}{r0 @AC} days.\n\n<!-- p3 -->\nFee: {--waived--}{r1 @BL}\n"
    );
    let bob = agent_options(&bytes, &changed(Some("Bob Lee")));
    assert!(
        bob.contains("\nrange: changed by @BL (p3) of p0-p3\n"),
        "{bob}"
    );
    assert_eq!(
        body(&bob),
        "<!-- page 1 of 1 -->\n\n<!-- p3 -->\nFee: {--waived--}{r1 @BL}\n"
    );
    for handle in ["BL", "@BL"] {
        let by_handle = agent_options(&bytes, &changed(Some(handle)));
        assert_eq!(body(&by_handle), body(&bob), "{handle}");
    }
    let nobody = agent_options(&bytes, &changed(Some("Cy Young")));
    assert!(
        nobody.contains("\nrange: changed by Cy Young (none) of p0-p3\n"),
        "{nobody}"
    );
    assert!(body(&nobody).trim().is_empty(), "{nobody}");
    // An unknown short name is not taken for a handle.
    let bo = agent_options(&bytes, &changed(Some("Bo")));
    assert!(
        bo.contains("\nrange: changed by Bo (none) of p0-p3\n"),
        "{bo}"
    );
}

#[test]
fn changed_finds_marks_the_resolved_views_carry_on_the_id_line() {
    let bytes = marked_docx();
    let accepted = agent_options(
        &bytes,
        &MarkdownOptions {
            track_changes: TrackChanges::Accept,
            ..changed(Some("AC"))
        },
    );
    assert!(
        accepted.contains("\nrange: changed by @AC (p1) of p0-p3\n"),
        "{accepted}"
    );
    assert!(
        accepted.contains("<!-- p1 rev r0 @AC -->\nPay in ten days.\n"),
        "{accepted}"
    );
}

#[test]
fn select_from_flags_refuses_by_without_changed_and_changed_with_picks() {
    assert_eq!(
        Select::from_flags(None, None, None, true, Some("AC")),
        Ok(Some(Select::Changed {
            by: Some("AC".into())
        }))
    );
    assert!(Select::from_flags(None, None, None, false, Some("AC")).is_err());
    assert!(Select::from_flags(Some("p1"), None, None, true, None).is_err());
    assert!(Select::from_flags(None, Some(2), None, true, None).is_err());
    // pi review r392 F3: the Python and WASM APIs call from_flags directly, so
    // a blank author is refused here as well as by clap, and padding is cut.
    // pi review r392b F6: one `@` marks a handle; `@@AC` and `@ AC` name
    // nobody.
    for blank in ["", "  ", "@", " @ ", "@@AC", "@ AC"] {
        assert!(
            Select::from_flags(None, None, None, true, Some(blank)).is_err(),
            "{blank:?}"
        );
    }
    assert_eq!(
        Select::from_flags(None, None, None, true, Some(" AC ")),
        Ok(Some(Select::Changed {
            by: Some("AC".into())
        }))
    );
}

#[test]
fn cli_read_changed_prints_only_the_changed_blocks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    let changed = ok(
        &["read", "received.docx", "--changed", "--no-page-markers"],
        dir.path(),
    );
    assert!(
        changed.contains("\nrange: changed (p3, p5, p7, t0, p18) of p0-p20\n"),
        "{changed}"
    );
    assert!(
        !changed.contains("<!-- p0 ") && !changed.contains("<!-- p1 "),
        "{changed}"
    );
    let by = ok(
        &[
            "read",
            "received.docx",
            "--changed",
            "--by",
            "AS",
            "--no-page-markers",
        ],
        dir.path(),
    );
    assert!(
        by.contains("\nrange: changed by @AS (p5) of p0-p20\n"),
        "{by}"
    );
    assert!(
        by.contains("{>>c6 @AS re c5: Disagree.<<}") && !by.contains("<!-- p3 "),
        "{by}"
    );
    let bad = jubarte(
        &["read", "received.docx", "--changed", "-p", "p1"],
        dir.path(),
    );
    assert!(!bad.status.success(), "--changed conflicts with -p");
    let bad = jubarte(&["read", "received.docx", "--by", "AS"], dir.path());
    assert!(!bad.status.success(), "--by needs --changed");
}

/// PR #385 review: every cached or hard break counts, not every paragraph
/// holding one.
#[test]
fn a_paragraph_spanning_three_pages_counts_every_break() {
    let cached = docx(&format!(
        r#"{}<w:p><w:r><w:t>Long</w:t></w:r><w:r><w:lastRenderedPageBreak/><w:t>more</w:t></w:r><w:r><w:lastRenderedPageBreak/><w:t>most</w:t></w:r></w:p>{}"#,
        para("One"),
        para("Tail")
    ));
    assert_eq!(
        body(&agent(&cached)),
        "<!-- page 1 of 3 -->\n\n<!-- p0 -->\nOne\n\n<!-- page 2 of 3 -->\n\n<!-- p1 -->\nLongmoremost\n\n<!-- page 3 of 3 -->\n\n<!-- p2 -->\nTail\n"
    );
    let hard = docx(&format!(
        r#"<w:p><w:r><w:t>A</w:t><w:br w:type="page"/><w:t>B</w:t><w:br w:type="page"/><w:t>C</w:t></w:r></w:p>{}"#,
        para("D")
    ));
    let out = body(&agent(&hard)).to_string();
    assert!(
        out.starts_with("<!-- page 1 of 3 -->\n\n<!-- p0 page-break -->\n"),
        "{out}"
    );
    assert!(
        out.ends_with("\n<!-- page 3 of 3 -->\n\n<!-- p1 -->\nD\n"),
        "{out}"
    );
}

/// PR #385 review: Word shows an empty numbered paragraph's label and spends
/// its number; the agent view keeps that paragraph's line with its label.
/// The plain conversion still counts written paragraphs only (see
/// `Writer::paragraph`).
#[test]
fn an_empty_numbered_paragraph_spends_its_number() {
    let numbering = Part {
        name: "word/numbering.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        xml: &format!(
            r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
        ),
    };
    let item = |text: &str| {
        format!(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr>{}</w:p>"#,
            if text.is_empty() {
                String::new()
            } else {
                format!("<w:r><w:t>{text}</w:t></w:r>")
            }
        )
    };
    let bytes = common::docx::docx_with(
        &format!("{}{}{}", item("First"), item(""), item("Third")),
        &[numbering],
    );
    let out = body(&agent(&bytes)).to_string();
    assert!(out.contains("<!-- p1 empty, num \"2.\" -->\n"), "{out}");
    assert!(out.contains("<!-- p2 num \"3.\" -->\n3. Third"), "{out}");
    let plain = docx_to_markdown(&bytes, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(plain.contains("2. Third"), "{plain}");
}

/// PR #385 review: rows and cells inside content controls count on the
/// table line, as the pipe table below it prints them.
#[test]
fn table_line_counts_rows_inside_content_controls() {
    let row = |a: &str, b: &str| {
        format!(
            "<w:tr><w:tc><w:p><w:r><w:t>{a}</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>{b}</w:t></w:r></w:p></w:tc></w:tr>"
        )
    };
    let tbl = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid>{}<w:sdt><w:sdtContent>{}</w:sdtContent></w:sdt></w:tbl>"#,
        row("a", "b"),
        row("c", "d")
    );
    let out = body(&agent(&docx(&tbl))).to_string();
    assert!(
        out.contains("<!-- t0 2x2, cells p0-p3 by row, widths 2.778in 2.778in, width auto -->\n"),
        "{out}"
    );
}

/// PR #385 review: an inserted row's revision survives into the resolved
/// views' table line; rejecting it removes the row.
#[test]
fn a_row_revision_shows_on_the_table_line_after_resolution() {
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:ins w:id="7" w:author="Ann Counsel"/></w:trPr><w:tc><w:p><w:ins w:id="8" w:author="Ann Counsel"><w:r><w:t>c</w:t></w:r></w:ins></w:p></w:tc><w:tc><w:p><w:ins w:id="9" w:author="Ann Counsel"><w:r><w:t>d</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl>"#;
    let bytes = docx(tbl);
    let accepted = body(&agent_with(&bytes, TrackChanges::Accept, true)).to_string();
    assert!(accepted.contains(" rev r7 @AC in t0.row1"), "{accepted}");
    let rejected = body(&agent_with(&bytes, TrackChanges::Reject, true)).to_string();
    assert!(
        rejected
            .contains("<!-- t0 1x2, cells p0-p1 by row, widths 2.778in 2.778in, width auto -->"),
        "{rejected}"
    );
}

/// Review t09-15: a pick that accepting or rejecting merged into its
/// neighbour is named, not silently dropped.
#[test]
fn a_pick_merged_away_by_resolution_names_the_paragraph_that_holds_it() {
    let body_xml = format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="10" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:rPr></w:pPr>{}</w:p><w:p>{}{}</w:p>"#,
        run("Split "),
        run("kept "),
        ins(11, "Ann Counsel", "added")
    );
    let bytes = docx(&body_xml);
    let pick = || Some(Select::parse("p1").unwrap());
    let err = docx_to_markdown(
        &bytes,
        &MarkdownOptions {
            track_changes: TrackChanges::Reject,
            select: pick(),
            ..agent_defaults()
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("p1 is part of p0 in this view"), "{err}");
    let tracked = agent_options(
        &bytes,
        &MarkdownOptions {
            select: pick(),
            ..agent_defaults()
        },
    );
    assert!(tracked.contains("\n<!-- p1"), "{tracked}");
}

/// Review t09-15: a selection that starts mid-page still names its page.
#[test]
fn a_selection_starting_mid_page_names_its_page() {
    let bytes = docx(&format!(
        r#"{}<w:p><w:r><w:lastRenderedPageBreak/><w:t>Two</w:t></w:r></w:p>{}{}"#,
        para("One"),
        para("Three"),
        para("Four")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            select: Some(Select::parse("p3").unwrap()),
            ..agent_defaults()
        },
    );
    assert_eq!(body(&out), "<!-- page 2 of 2 -->\n\n<!-- p3 -->\nFour\n");
}

/// Review t09-15: formatting-only changes are counted in the header, by
/// author too.
#[test]
fn header_counts_formatting_only_changes() {
    let bytes = docx(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Ann Counsel"><w:rPr/></w:rPrChange></w:rPr><w:t>x</w:t></w:r></w:p>"#,
    );
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("revisions: 1 ") && l.ends_with("# 1 formatting")),
        "{out}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("  AC: Ann Counsel ") && l.ends_with("# 1 revision")),
        "{out}"
    );
}

/// Review t09-15: a comment reference with no comment behind it is listed
/// nowhere, in a cell as in a paragraph.
#[test]
fn an_orphan_comment_reference_in_a_cell_is_not_listed() {
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>a</w:t></w:r><w:r><w:commentReference w:id="9"/></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = agent_with(&docx(tbl), TrackChanges::All, false);
    assert!(
        !regex::Regex::new(r"\bc9\b").unwrap().is_match(&out),
        "{out}"
    );
}

/// Review t09-15: the header's style facts skip text boxes, as every other
/// count does.
#[test]
fn header_table_styles_skip_tables_in_text_boxes() {
    let boxed = r#"<w:p><w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:tbl><w:tblPr><w:tblStyle w:val="PlainTable"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>in box</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>"#;
    // Table Grid is the view's default and goes unnamed; the top table
    // takes a style the legend must name.
    let top = r#"<w:tbl><w:tblPr><w:tblStyle w:val="Ledger"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>top</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = agent(&docx(&format!("{top}{boxed}")));
    assert!(!out.contains("PlainTable"), "{out}");
    assert!(out.contains("\n  table: Ledger\n"), "{out}");
}

/// Review t09-15: underline from a character style shows as `<u>`, as bold
/// and italic from a style do.
#[test]
fn underline_from_a_character_style_shows() {
    let styles = Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &format!(
            r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="character" w:styleId="Under"><w:name w:val="Under"/><w:rPr><w:u w:val="single"/></w:rPr></w:style><w:style w:type="character" w:styleId="Plain"><w:name w:val="Plain"/><w:basedOn w:val="Under"/><w:rPr><w:u w:val="none"/></w:rPr></w:style></w:styles>"#
        ),
    };
    let bytes = common::docx::docx_with(
        r#"<w:p><w:r><w:rPr><w:rStyle w:val="Under"/></w:rPr><w:t>styled</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:rStyle w:val="Plain"/></w:rPr><w:t>plain</w:t></w:r></w:p>"#,
        &[styles],
    );
    let out = body(&agent(&bytes)).to_string();
    assert!(out.contains("<u>styled</u> plain"), "{out}");
}

/// Task 18: literal Markdown marks in document text read back as text,
/// in the agent view and in the plain conversion alike (#519).
#[test]
fn literal_markdown_marks_are_escaped_in_both_views() {
    let bytes = docx(&format!(
        "{}{}{}{}{}{}{}",
        para("# Not a heading"),
        para("**stars** and a_b and snake_case"),
        para("1. one and 2) two"),
        para("x < y, <u>tag</u>, a*b"),
        para("- dash and `tick`"),
        para("~~gone~~ ==hi== [^1] ok \\ done"),
        para("1.5 litres and _x_"),
    ));
    let out = agent(&bytes);
    let out = body(&out);
    assert!(out.contains("<!-- p0 -->\n\\# Not a heading\n"), "{out}");
    assert!(
        out.contains("<!-- p1 -->\n\\*\\*stars\\*\\* and a_b and snake_case\n"),
        "{out}"
    );
    assert!(out.contains("<!-- p2 -->\n1\\. one and 2) two\n"), "{out}");
    assert!(
        out.contains("<!-- p3 -->\nx < y, \\<u>tag\\</u>, a\\*b\n"),
        "{out}"
    );
    assert!(
        out.contains("<!-- p4 -->\n\\- dash and \\`tick\\`\n"),
        "{out}"
    );
    assert!(
        out.contains("<!-- p5 -->\n\\~\\~gone\\~\\~ \\=\\=hi\\=\\= \\[^1] ok \\\\ done\n"),
        "{out}"
    );
    assert!(
        out.contains("<!-- p6 -->\n1.5 litres and \\_x\\_\n"),
        "{out}"
    );
    let legacy = docx_to_markdown(&bytes, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(
        legacy.contains("\\# Not a heading\n") && legacy.contains("\\*\\*stars\\*\\* and a_b"),
        "escaped as the agent view: {legacy}"
    );
}

/// Task 18: a line after a hard break that starts like a block is escaped
/// too (an ATX heading or a list can interrupt a paragraph).
#[test]
fn a_block_mark_after_a_hard_break_is_escaped() {
    let bytes = docx(
        r#"<w:p><w:r><w:t>Intro</w:t><w:br/><w:t># Not a heading</w:t><w:br/><w:t>- nor a list</w:t></w:r></w:p>"#,
    );
    let out = body(&agent(&bytes)).to_string();
    assert!(out.contains("\n\\# Not a heading"), "{out}");
    assert!(out.contains("\n\\- nor a list"), "{out}");
}

/// Task 18: the escaped view converts back to the same document text.
#[test]
fn escaped_text_reads_back_as_the_characters() {
    let bytes = docx(&format!(
        "{}{}",
        para("# Not a heading"),
        para("**stars** and `tick` ~~gone~~")
    ));
    let view = agent(&bytes);
    let written = jubarte::markdown::markdown_to_docx(
        body(&view),
        &jubarte::markdown::DocxOptions::default(),
    )
    .unwrap()
    .docx;
    // The plain conversion escapes too (#519), so its escapes name the
    // literal characters the document holds.
    let text = docx_to_markdown(&written, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(text.contains("\\# Not a heading"), "{text}");
    assert!(
        text.contains("\\*\\*stars\\*\\* and \\`tick\\` \\~\\~gone\\~\\~"),
        "{text}"
    );
    let xml = common::docx::part_string(&written, "word/document.xml").unwrap();
    assert!(!xml.contains("Heading"), "no heading style: {xml}");
    assert!(
        xml.contains("# Not a heading"),
        "the text as written: {xml}"
    );
}

/// Assert the publication protocol, then inspect only the after-side lines
/// (added or retained context). Removed text must not satisfy content checks.
fn confirmation_after(stdout: &str) -> String {
    assert!(stdout.contains("SUCCESS:\n~~~~diff\n"), "{stdout}");
    assert!(
        stdout.contains("\n--- before ") && stdout.contains("\n+++ after "),
        "{stdout}"
    );
    assert!(
        stdout.contains("\n@@ ") && stdout.ends_with("~~~~\n"),
        "{stdout}"
    );
    let mut after = String::new();
    for line in stdout.lines() {
        if line.starts_with("+++") {
            continue;
        }
        if let Some(line) = line.strip_prefix('+').or_else(|| line.strip_prefix(' ')) {
            after.push_str(line);
            after.push('\n');
        }
    }
    after
}

fn confirmed(args: &[&str], dir: &Path) -> String {
    confirmation_after(&ok(args, dir))
}

#[test]
fn cli_edit_says_when_an_anchor_was_read_without_its_marks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let plan = r##"{"schema_version":1,"author":"Ann Counsel","operations":[
        {"kind":"replace","paragraph":"p0","find":"# Fees","replacement":"Fees and Expenses"}]}"##;
    std::fs::write(dir.path().join("plan.json"), plan).unwrap();
    let (out, err) = ok_both(
        &["edit", "base.docx", "--plan", "plan.json", "--out-dir", "r"],
        dir.path(),
    );
    // The notes are status lines: stderr, beside the wrote line (D6).
    assert!(
        err.contains(
            "\nnote: op-1: anchor \"# Fees\" read as \"Fees\" (Markdown marks are not document text)\n"
        ) && err.starts_with("wrote r (4 files: "),
        "{err}"
    );
    assert!(!out.contains("note:") && !out.contains("\nwrote "), "{out}");
    let report = std::fs::read_to_string(dir.path().join("r/report.jsonl")).unwrap();
    assert!(
        report.contains(r##""anchor_given":"# Fees","anchor_read_as":"Fees""##),
        "{report}"
    );
}

#[test]
fn edit_flags_group_operations_by_location_and_print_the_changed_blocks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let out = confirmed(
        &[
            "edit",
            "base.docx",
            "--out-dir",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty-five",
            "-p",
            "p2",
            "--anchor",
            " per month",
            "--delete",
            "-p",
            "p0",
            "--content",
            "Fees and Expenses",
            "--datetime",
            "2026-10-01T09:00:00Z",
        ],
        dir.path(),
    );
    for name in ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"] {
        assert!(
            dir.path().join("base.edit").join(name).is_file(),
            "--out-dir's default holds {name}"
        );
    }
    assert!(out.contains("\nsource: redline.docx\n"), "{out}");
    assert!(
        out.contains("<!-- p0 -->") && out.contains("<!-- p1 -->") && out.contains("<!-- p2 -->"),
        "{out}"
    );
    assert!(
        out.contains("J: Jubarte") && out.contains("# 4 revisions, 2026-10-01T09:00:00Z"),
        "{out}"
    );
    captures(
        &out,
        r"<!-- p0 -->\nFees\{\+\+ and Expenses\+\+\}\{r\d+ @J\}\n",
    );
    captures(
        &out,
        r"<!-- p1 -->\nClient shall pay each invoice within \{~~thirty~>forty-five~~\}\{r\d+\+\d+ @J\} days of receipt\.\n",
    );
    captures(
        &out,
        r"<!-- p2 -->\nLate amounts accrue interest at one percent ?\{-- ?per month--\}\{r\d+ @J\} ?\.\n",
    );
    // confirmation_after already asserted unified hunks on the raw stdout.
    let report = std::fs::read_to_string(dir.path().join("base.edit/report.jsonl")).unwrap();
    assert!(
        report.contains(r#""op":"replace""#)
            && report.contains(r#""op":"delete""#)
            && report.contains(r#""op":"rewrite""#),
        "{report}"
    );

    let again = jubarte(
        &[
            "edit",
            "base.docx",
            "--out-dir",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "sixty",
        ],
        dir.path(),
    );
    assert!(
        !again.status.success(),
        "base.edit exists: --force or --out-dir"
    );
    // --plan with operation flags records them into the plan, editing
    // nothing (tests/cli_plan_record.rs covers recording).
    let recorded = jubarte(
        &[
            "edit",
            "base.docx",
            "--plan",
            "x.json",
            "-p",
            "p1",
            "--delete",
        ],
        dir.path(),
    );
    assert!(recorded.status.success(), "{recorded:?}");
    assert!(dir.path().join("x.json").exists());
    for bad in [
        &["edit", "base.docx"][..],
        &[
            "edit",
            "base.docx",
            "-p",
            "p1",
            "--delete",
            "--content",
            "x",
        ][..],
        &[
            "edit",
            "base.docx",
            "--anchor",
            "a",
            "-p",
            "p1",
            "--content",
            "b",
        ][..],
        &["edit", "base.docx", "-p", "p1"][..],
        &[
            "edit",
            "base.docx",
            "-p",
            "c0",
            "--anchor",
            "a",
            "--content",
            "b",
        ][..],
        &["edit", "base.docx", "-p", "p1", "--resolve"][..],
    ] {
        assert!(!jubarte(bad, dir.path()).status.success(), "{bad:?}");
    }
}

#[test]
fn edit_flags_turn_a_kept_anchor_into_an_insertion_and_keep_the_other_party_s_marks() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let keep = [
        "--existing-revisions",
        "keep",
        "--datetime",
        "2026-10-01T09:00:00Z",
    ];
    let out = confirmed(
        &[
            &[
                "edit",
                "base.docx",
                "-p",
                "p1",
                "--anchor",
                "thirty",
                "--content",
                "thirty (30)",
                "-p",
                "p2",
                "--anchor",
                "Late",
                "--content",
                "Note: Late",
                "--out-dir",
                "one",
            ][..],
            &keep[..],
        ]
        .concat(),
        dir.path(),
    );
    captures(&out, r"within thirty\{\+\+ \(30\)\+\+\}\{r\d+ @J\} days");
    captures(
        &out,
        r"<!-- p2 -->\n\{\+\+Note: \+\+\}\{r\d+ @J\}Late amounts",
    );
    let report = std::fs::read_to_string(dir.path().join("one/report.jsonl")).unwrap();
    assert_eq!(report.matches(r#""op":"insert""#).count(), 2, "{report}");

    let out = confirmed(
        &[
            "edit",
            "one/redline.docx",
            "-p",
            "p1",
            "--anchor",
            "receipt",
            "--content",
            "the invoice",
            "--author",
            "John Doe",
            "--datetime",
            "2026-10-02T10:00:00Z",
            "--out-dir",
            "two",
        ],
        dir.path(),
    );
    assert!(
        out.contains("source: redline.docx") && out.contains("<!-- p1 -->"),
        "{out}"
    );
    captures(
        &out,
        r"\{\+\+ \(30\)\+\+\}\{r\d+ @J\} days of \{~~receipt~>the invoice~~\}\{r\d+\+\d+ @JD\}\.",
    );
    let report = std::fs::read_to_string(dir.path().join("two/report.jsonl")).unwrap();
    assert!(
        report.contains(r#""existing_revisions":"keep""#),
        "auto picked keep: {report}"
    );
    let refused = jubarte(
        &[
            "edit",
            "one/redline.docx",
            "-p",
            "p1",
            "--anchor",
            "receipt",
            "--content",
            "x",
            "--existing-revisions",
            "refuse",
            "--out-dir",
            "three",
        ],
        dir.path(),
    );
    assert_eq!(refused.status.code(), Some(3), "refuse is still available");
}

#[test]
fn add_and_edit_cover_a_comment_thread_from_the_command_line() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    let out = confirmed(
        &[
            "add",
            "base.docx",
            "-p",
            "p2",
            "--anchor",
            "one percent",
            "--content",
            "Cap?",
            "--author",
            "Ann Counsel",
            "--datetime",
            "2026-10-01T09:00:00Z",
            "--out-dir",
            "one",
        ],
        dir.path(),
    );
    assert!(out.contains("{==one percent==}{>>c0 @AC: Cap?<<}"), "{out}");
    let out = confirmed(
        &[
            "add",
            "one/redline.docx",
            "-p",
            "c0",
            "--content",
            "Yes, in Delaware.",
            "--author",
            "Bob Lee",
            "--datetime",
            "2026-10-02T10:00:00Z",
            "--out-dir",
            "two",
        ],
        dir.path(),
    );
    assert!(
        out.contains("{>>c0 @AC: Cap?<<}{>>c1 @BL re c0: Yes, in Delaware.<<}"),
        "{out}"
    );
    let out = confirmed(
        &[
            "edit",
            "two/redline.docx",
            "-p",
            "c1",
            "--content",
            "Yes, in Delaware (6 Del. C. 2301).",
            "-p",
            "c0",
            "--resolve",
            "--author",
            "Cy Young",
            // c1 is Bob's and c0 Ann's: Cy rewrites one under Bob's name
            // and resolves the other only on request (#1264, #1846).
            "--rewrite-others-comments",
            "--out-dir",
            "three",
        ],
        dir.path(),
    );
    assert!(
        out.contains("source: redline.docx") && out.contains("<!-- p2 -->"),
        "the diff confirms the changed thread: {out}"
    );
    assert!(
        out.contains(
            "{>>c0 @AC resolved: Cap?<<}{>>c1 @BL re c0: Yes, in Delaware (6 Del. C. 2301).<<}"
        ),
        "{out}"
    );
    let out = confirmed(
        &[
            "add",
            "three/redline.docx",
            "-p",
            "p0",
            "--comment",
            "--content",
            "Add a fee schedule.",
            "--out-dir",
            "four",
        ],
        dir.path(),
    );
    assert!(
        out.contains("<!-- p0 -->\n{==Fees==}{>>c2 @J: Add a fee schedule.<<}\n"),
        "{out}"
    );
    let out = confirmed(
        &[
            "edit",
            "four/redline.docx",
            "-p",
            "c0",
            "--delete",
            // Ann's thread, Bob's reply with it: deleted only on request (#1846).
            "--rewrite-others-comments",
            "--out-dir",
            "five",
        ],
        dir.path(),
    );
    // As words: the `sha256:` line's hex may hold `c0` by chance.
    let gone = regex::Regex::new(r"\bc[01]\b").unwrap();
    assert!(!gone.is_match(&out), "the thread is gone: {out}");
    assert!(
        out.contains("source: redline.docx")
            && out.contains("<!-- p2 -->")
            && out.contains("1 comment: c2"),
        "{out}"
    );
    // The surviving p0 note is unchanged and outside the three context
    // lines of the p2 deletion hunk; retain its content check via read.
    let remaining = ok(&["read", "five/redline.docx"], dir.path());
    assert!(
        remaining.contains("<!-- p0 -->\n{==Fees==}{>>c2 @J: Add a fee schedule.<<}\n"),
        "{remaining}"
    );
    assert!(
        !gone.is_match(&remaining),
        "the thread is gone: {remaining}"
    );
    for bad in [
        &["add", "base.docx", "-p", "p1"][..],
        &[
            "add",
            "base.docx",
            "-p",
            "p1",
            "--anchor",
            "x",
            "--content",
            "y",
            "--before",
        ][..],
        &["add", "base.docx", "-p", "c0", "--content", "y", "--before"][..],
        &["add", "base.docx", "-p", "p1", "--content", "y", "--delete"][..],
    ] {
        assert!(!jubarte(bad, dir.path()).status.success(), "{bad:?}");
    }
}

#[test]
fn add_places_paragraphs_styles_them_and_editing_mode_writes_no_redline() {
    let dir = tempfile::tempdir().unwrap();
    // `--style Heading2` names a style the document defines; the engine
    // refuses an undefined one (UNKNOWN_STYLE).
    let styles = Part {
        name: "word/styles.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
        xml: &format!(
            r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr></w:style></w:styles>"#
        ),
    };
    std::fs::write(
        dir.path().join("base.docx"),
        common::docx::docx_with(BASE, &[styles]),
    )
    .unwrap();
    let after = confirmed(
        &[
            "add",
            "base.docx",
            "-p",
            "p1",
            "--content",
            "Invoices are due in full.",
            "--out-dir",
            "after",
        ],
        dir.path(),
    );
    assert!(
        after.contains("source: redline.docx") && after.contains("<!-- p2"),
        "{after}"
    );
    captures(
        &after,
        r"<!-- p2[^>]*-->\n\{\+\+Invoices are due in full\.\+\+\}\{r\d+\+\d+ @J\}¶\{r\d+\+\d+ @J\}\n",
    );
    let before = confirmed(
        &[
            "add",
            "base.docx",
            "-p",
            "p1",
            "--content",
            "Payment",
            "--before",
            "--style",
            "Heading2",
            "--style",
            "bold",
            "--out-dir",
            "before",
        ],
        dir.path(),
    );
    captures(
        &before,
        r"<!-- p1[^>]*-->\n## \{\+\+Payment\+\+\}\{r\d+\+\d+ @J\}¶\{r\d+\+\d+ @J\}\n",
    );
    let xml = common::docx::part_string(
        &std::fs::read(dir.path().join("before/clean.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert!(
        xml.contains(r#"<w:pStyle w:val="Heading2" />"#)
            && xml.contains("<w:rPr><w:b />")
            && xml.contains(">Payment<"),
        "{xml}"
    );

    let out = confirmed(
        &[
            "edit",
            "base.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty-five",
            "--editing-mode",
            "--out-dir",
            "e",
        ],
        dir.path(),
    );
    assert!(
        dir.path().join("e/clean.docx").is_file() && dir.path().join("e/report.jsonl").is_file()
    );
    assert!(
        !dir.path().join("e/redline.docx").exists() && !dir.path().join("e/patch.diff").exists(),
        "editing mode writes no redline"
    );
    assert!(
        out.contains("source: clean.docx") && out.contains("<!-- p1 -->"),
        "{out}"
    );
    captures(
        &out,
        r"<!-- p1 -->\nClient shall pay each invoice within forty-five days of receipt\.\n",
    );

    let (noted, noted_err) = ok_both(
        &[
            "edit",
            "base.docx",
            "-p",
            "p0",
            "--anchor",
            "# Fees",
            "--content",
            "Fees \\* Costs",
            "--out-dir",
            "n",
        ],
        dir.path(),
    );
    assert!(noted_err.contains("\nnote: op-1: anchor \"# Fees\" read as \"Fees\" (Markdown marks are not document text)\n"), "{noted_err}");
    captures(
        &confirmation_after(&noted),
        r"<!-- p0 -->\nFees\{\+\+ \\\* Costs\+\+\}\{r\d+ @J\}\n",
    );
    let (_, marks) = ok_both(
        &[
            "edit",
            "base.docx",
            "-p",
            "p0",
            "--content",
            "**Fees**",
            "--out-dir",
            "m",
        ],
        dir.path(),
    );
    assert!(
        marks.contains(
            "\nnote: op-1: content keeps its Markdown marks as text; use --style for formatting\n"
        ),
        "{marks}"
    );
}

#[test]
fn one_file_prints_the_agent_view_and_two_files_print_their_redline() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    assert_eq!(
        ok(&["received.docx"], dir.path()),
        golden("received.tracked.md")
    );
    assert_eq!(
        ok(
            &["received.docx", "--head", "2", "--no-page-markers"],
            dir.path()
        ),
        ok(
            &["read", "received.docx", "--head", "2", "--no-page-markers"],
            dir.path()
        )
    );

    std::fs::write(dir.path().join("old.docx"), docx(BASE)).unwrap();
    std::fs::write(
        dir.path().join("new.docx"),
        docx(&BASE.replace("thirty", "forty-five")),
    )
    .unwrap();
    let out = ok(
        &["old.docx", "new.docx", "-d", "1970-01-01T00:00:00Z"],
        dir.path(),
    );
    assert!(
        out.starts_with("---\nsource: old_v_new.docx (not written; -o keeps it)\n"),
        "{out}"
    );
    // The comparer writes the insertion first: one replacement, its id
    // on both pieces (#1359).
    let ids = captures(
        &out,
        r"<!-- p1 -->\nClient shall pay each invoice within \{\+\+forty-five\+\+\}\{(r\d+\+\d+) @J\}\{--thirty--\}\{(r\d+\+\d+) @J\} days of receipt\.\n",
    );
    assert_eq!(ids[0], ids[1], "{out}");
    assert!(out.contains("J: Jubarte"), "{out}");
    // The digest is the in-memory redline's, neither input's.
    let digest = captures(&out, r"\nsha256: ([0-9a-f]{64})\n")[0].to_string();
    for input in ["old.docx", "new.docx"] {
        let bytes = std::fs::read(dir.path().join(input)).unwrap();
        assert_ne!(digest, jubarte::inspect::source_sha256(&bytes), "{input}");
    }
    assert!(
        !dir.path().join("old_v_new.docx").exists(),
        "nothing written without -o"
    );
    let written = jubarte(
        &[
            "old.docx",
            "new.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "r.docx",
        ],
        dir.path(),
    );
    assert!(written.status.success());
    let confirmation = String::from_utf8_lossy(&written.stdout);
    assert!(
        confirmation.starts_with("SUCCESS:\n~~~~diff\n"),
        "{confirmation}"
    );
    assert!(
        confirmation.contains("+++ after r.docx\n"),
        "{confirmation}"
    );
    assert!(
        confirmation
            .lines()
            .any(|line| line.starts_with('+') && line.contains("forty-five")),
        "{confirmation}"
    );
    let stderr = String::from_utf8_lossy(&written.stderr);
    assert!(stderr.starts_with("wrote r.docx (docx, "), "{stderr}");
    assert!(dir.path().join("r.docx").is_file());
    // -o FILE.md writes the view, read options included.
    let view = jubarte(
        &[
            "old.docx",
            "new.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "-o",
            "v.md",
            "--head",
            "1",
        ],
        dir.path(),
    );
    assert!(
        view.status.success(),
        "{}",
        String::from_utf8_lossy(&view.stderr)
    );
    let view = std::fs::read_to_string(dir.path().join("v.md")).unwrap();
    assert!(view.contains("\nrange: "), "{view}");
    let changed = ok(
        &[
            "old.docx",
            "new.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "--changed",
        ],
        dir.path(),
    );
    assert!(
        changed.contains("\nrange: changed (p1) of p0-p2\n"),
        "{changed}"
    );
    let bad = jubarte(
        &["old.docx", "new.docx", "-o", "x.docx", "--head", "1"],
        dir.path(),
    );
    assert_eq!(
        bad.status.code(),
        Some(2),
        "read options go with the view, not with a .docx"
    );
    let bad = jubarte(&["compare", "old.docx"], dir.path());
    assert!(!bad.status.success(), "compare still needs two documents");
}

#[test]
fn editing_mode_refuses_to_keep_other_tracked_changes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("base.docx"), docx(BASE)).unwrap();
    ok(
        &[
            "edit",
            "base.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty-five",
            "--out-dir",
            "one",
        ],
        dir.path(),
    );
    // The redline holds tracked changes: `auto` would keep them, and a
    // "clean" copy that still carries revisions is not what editing mode
    // promises.
    let kept = jubarte(
        &[
            "edit",
            "one/redline.docx",
            "-p",
            "p2",
            "--anchor",
            "Late",
            "--content",
            "Overdue",
            "--editing-mode",
            "--out-dir",
            "two",
        ],
        dir.path(),
    );
    assert_eq!(
        kept.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&kept.stderr)
    );
    assert!(
        String::from_utf8_lossy(&kept.stderr).contains("--existing-revisions accept or reject")
    );
    assert!(!dir.path().join("two").exists());
    // Accepting them first gives a clean copy with no revision left.
    ok(
        &[
            "edit",
            "one/redline.docx",
            "-p",
            "p2",
            "--anchor",
            "Late",
            "--content",
            "Overdue",
            "--editing-mode",
            "--existing-revisions",
            "accept",
            "--out-dir",
            "three",
        ],
        dir.path(),
    );
    let xml = common::docx::part_string(
        &std::fs::read(dir.path().join("three/clean.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert!(
        !xml.contains("<w:ins ") && !xml.contains("<w:del "),
        "{xml}"
    );
    assert!(
        xml.contains("forty-five") && xml.contains("Overdue"),
        "{xml}"
    );
    // pi review av4 F14: rejecting them restores the base text, and a second
    // run into the same folder needs --force with flag operations too.
    let reject = |force: bool| {
        let mut args = vec![
            "edit",
            "one/redline.docx",
            "-p",
            "p2",
            "--anchor",
            "Late",
            "--content",
            "Overdue",
            "--editing-mode",
            "--existing-revisions",
            "reject",
            "--out-dir",
            "four",
        ];
        if force {
            args.push("--force");
        }
        jubarte(&args, dir.path())
    };
    let first = reject(false);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let xml = common::docx::part_string(
        &std::fs::read(dir.path().join("four/clean.docx")).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert!(
        xml.contains("thirty") && !xml.contains("forty-five") && xml.contains("Overdue"),
        "{xml}"
    );
    assert!(
        !reject(false).status.success(),
        "an existing folder needs --force"
    );
    let forced = reject(true);
    assert!(
        forced.status.success(),
        "{}",
        String::from_utf8_lossy(&forced.stderr)
    );
}

/// A backslash in a table cell is escaped once, and a pipe stays a pipe.
#[test]
fn a_table_cell_escapes_a_backslash_once() {
    let bytes = docx(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>a\b</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>x|y</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#,
    );
    let out = body(&agent(&bytes)).to_string();
    assert!(
        out.contains(r"| a\\b |") || out.contains(r"|a\\b|"),
        "{out}"
    );
    assert!(!out.contains(r"a\\\\b"), "{out}");
    assert!(out.contains(r"x\|y"), "{out}");
}

/// A line of `=` after a line break would underline the line before it.
#[test]
fn a_setext_underline_after_a_hard_break_is_escaped() {
    // `==` pairs are escaped as marks already; a lone `=` needs the block rule.
    let bytes = docx(r#"<w:p><w:r><w:t>Title</w:t><w:br/><w:t>=</w:t></w:r></w:p>"#);
    let out = body(&agent(&bytes)).to_string();
    assert!(out.contains("Title\\\n\\=\n"), "{out}");
}

/// An anchor copied across the view's hard break (`\` then a newline)
/// matches the paragraph's line break, and deleting the range removes the
/// break with the text around it.
#[test]
fn cli_edit_matches_an_anchor_copied_across_a_hard_break() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.docx"),
        docx(r#"<w:p><w:r><w:t>Intro</w:t><w:br/><w:t># Not a heading</w:t></w:r></w:p>"#),
    )
    .unwrap();
    let edited = jubarte(
        &[
            "edit",
            "a.docx",
            "-p",
            "p0",
            "--anchor",
            "Intro\\\n\\# Not",
            "--delete",
            "--out-dir",
            "e",
        ],
        dir.path(),
    );
    let stdout = String::from_utf8_lossy(&edited.stdout);
    assert_eq!(edited.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains(r#""ok":1"#), "{stdout}");
    let clean =
        jubarte::inspect::paragraphs(&std::fs::read(dir.path().join("e/clean.docx")).unwrap())
            .unwrap();
    assert_eq!(clean[0].text, " a heading");
}

fn changed_by(docx: &[u8], by: Option<&str>, comments: bool) -> String {
    agent_options(
        docx,
        &MarkdownOptions {
            comments,
            page_markers: false,
            select: Some(Select::Changed {
                by: by.map(str::to_string),
            }),
            ..agent_defaults()
        },
    )
}

/// pi review av3 F3: document text that reads like CriticMarkup is escaped in
/// the view, so neither a reader nor `--changed` takes it for a mark.
#[test]
fn literal_critic_markup_in_the_text_is_escaped_and_not_a_change() {
    let doc = docx(&format!(
        "{}{}",
        para("Use {++bold++} and {r0 @AC} marks"),
        para("quiet")
    ));
    let view = agent(&doc);
    assert!(
        view.contains("Use {\\++bold++\\} and \\{r0 @AC} marks"),
        "{view}"
    );
    let changed = changed_by(&doc, None, true);
    assert!(
        changed.contains("\nrange: changed (none) of p0-p1\n"),
        "{changed}"
    );
}

/// pi review av3 F2: with comments hidden, `--by` still finds the paragraph
/// that holds only that author's comment.
#[test]
fn changed_by_keeps_a_hidden_comment_of_that_author() {
    let doc = one_comment_docx(&format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}<w:commentRangeEnd w:id="9"/>{}</w:p>{}"#,
        run("Fee."),
        reference(9),
        para("quiet")
    ));
    let hidden = changed_by(&doc, Some("AC"), false);
    assert!(
        hidden.contains("\nrange: changed by @AC (p0) of p0-p1\n"),
        "{hidden}"
    );
    let inline = changed_by(&doc, Some("AC"), true);
    assert!(
        inline.contains("\nrange: changed by @AC (p0) of p0-p1\n"),
        "{inline}"
    );
}

/// A handle typed in the document's own text is not that author's mark.
#[test]
fn changed_by_ignores_a_handle_written_in_the_text() {
    let doc = docx(&format!(
        "<w:p>{}{}</w:p><w:p>{}</w:p>",
        run("Ask @AC first."),
        ins(1, "Bob Day", " Done."),
        ins(2, "Ann Counsel", "Agreed.")
    ));
    let view = changed_by(&doc, Some("AC"), true);
    assert!(
        view.contains("\nrange: changed by @AC (p1) of p0-p1\n"),
        "{view}"
    );
}

/// pi review av3 F4/F7: `--by` is read as a handle first (the handle the
/// view prints), then as a full name; an empty author is refused.
#[test]
fn changed_by_reads_a_handle_before_a_name_and_refuses_an_empty_author() {
    // "AC" is Ann Counsel's handle; a second author literally named "AC"
    // gets another handle and must not take `--by AC` from her.
    let doc = docx(&format!(
        "<w:p>{}</w:p><w:p>{}</w:p>",
        ins(1, "Ann Counsel", "Hers."),
        ins(2, "AC", "Theirs.")
    ));
    let by_handle = changed_by(&doc, Some("AC"), true);
    assert!(
        by_handle.contains("\nrange: changed by @AC (p0) of p0-p1\n"),
        "{by_handle}"
    );
    let by_name = changed_by(&doc, Some("Ann Counsel"), true);
    assert!(
        by_name.contains("\nrange: changed by @AC (p0) of p0-p1\n"),
        "{by_name}"
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture("received.docx"), dir.path().join("received.docx")).unwrap();
    for empty in ["", "@", " "] {
        let out = jubarte(
            &["read", "received.docx", "--changed", "--by", empty],
            dir.path(),
        );
        assert_eq!(out.status.code(), Some(2), "{empty:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("needs an author"), "{empty:?}: {stderr}");
    }
}

/// The header between the `---` fences, parsed by a YAML parser.
fn header_yaml(markdown: &str) -> saphyr::Yaml<'_> {
    use saphyr::LoadableYamlNode;
    let text = markdown.split("---\n").nth(1).unwrap();
    saphyr::Yaml::load_from_str(text)
        .unwrap_or_else(|e| panic!("header is not YAML: {e}\n{text}"))
        .remove(0)
}

/// pi review av2 F1: document values in the header (authors, owner,
/// header/footer text) are quoted whenever plain YAML would misread them.
#[test]
fn the_header_stays_yaml_for_values_with_colons_commas_and_hashes() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>SIGNATURE PAGE, v2 {{draft}}</w:t></w:r></w:p></w:hdr>"#
    );
    let sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX1"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#;
    let bytes = common::docx::docx_with_sect_pr(
        &format!(
            "<w:p>{}</w:p><w:p>{}</w:p>",
            ins(1, "Counsel: Ann", "a"),
            ins(2, "yes", "b")
        ),
        &[
            core(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:creator>Jo #1, Esq.</dc:creator></cp:coreProperties>"#,
            ),
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
        ],
        sect,
    );
    let view = agent(&bytes);
    let yaml = header_yaml(&view);
    let authors = yaml["authors"].as_mapping().expect("authors map");
    let names: Vec<&str> = authors.values().filter_map(saphyr::Yaml::as_str).collect();
    assert!(names.contains(&"Counsel: Ann"), "{names:?}");
    assert!(names.contains(&"yes"), "{names:?}");
    assert_eq!(
        yaml["authors"]["document_owner"].as_str(),
        Some("Jo #1, Esq.")
    );
    assert_eq!(
        yaml["headers"]["default"]["text"].as_str(),
        Some("SIGNATURE PAGE, v2 {draft}")
    );
}

/// Every golden's header parses as YAML.
#[test]
fn the_golden_headers_parse_as_yaml() {
    for name in [
        "received.tracked.md",
        "received.no-comments.md",
        "received.accept.md",
        "received.reject.md",
    ] {
        let text = golden(name);
        let yaml = header_yaml(&text);
        assert!(yaml["source"].as_str().is_some(), "{name}");
    }
}

/// pi review av2 F2: a revision in a text box counts once in the header, as
/// Word shows it once: the same `w:id` in the `mc:Choice` shape and its VML
/// fallback copy is one revision. The body carries the whole
/// `mc:AlternateContent` as one payload (54621454), so the revision prints
/// in no mark and is named once on `unplaced:`.
#[test]
fn header_counts_a_text_box_revision_once() {
    let story = ins(7, "Ann Counsel", "boxed");
    let boxed = format!(
        r#"<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:txbx><w:txbxContent><w:p>{story}</w:p></w:txbxContent></wps:txbx></wps:wsp></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p>{story}</w:p></w:txbxContent></v:textbox></v:shape></w:pict></mc:Fallback></mc:AlternateContent></w:r><w:r><w:t>Host</w:t></w:r></w:p>"#
    );
    let view = agent(&docx(&boxed));
    // One shape, not one per branch: the whole AlternateContent is a
    // single payload token, and the revision it holds is named once.
    assert_eq!(view.matches("![][d1]").count(), 1, "{view}");
    // The shown branch's box prints once, as comment lines beside the
    // drawing, never the Fallback copy: its revision is placed there.
    assert_eq!(view.matches("{++boxed++}").count(), 1, "{view}");
    let lines = header_lines(&view);
    assert!(
        !lines
            .iter()
            .any(|l| l.starts_with("unplaced: ") && l.contains("r7")),
        "{view}"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("revisions: 1 ")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("  AC: Ann Counsel") && l.contains("# 1 revision")),
        "{lines:?}"
    );
}

/// pi review av1 F16 / av2 F14: `w:left` and its strict twin `w:start` (and
/// `w:right`/`w:end`) give one clause per side, read as the layout reads
/// them: `left` first, `start` when `left` is absent.
#[test]
fn id_line_gives_one_indent_per_side_when_both_spellings_are_present() {
    let out = body(&agent(&docx(concat!(
        r#"<w:p><w:pPr><w:ind w:left="720" w:start="1440" w:right="360" w:end="720"/></w:pPr><w:r><w:t>Both.</w:t></w:r></w:p>"#,
        r#"<w:p><w:pPr><w:ind w:start="1440" w:end="720"/></w:pPr><w:r><w:t>Strict.</w:t></w:r></w:p>"#,
    ))))
    .to_string();
    assert!(
        out.contains("<!-- p0 left 0.5in, right 0.25in -->\nBoth."),
        "{out}"
    );
    assert!(
        out.contains("<!-- p1 left 1in, right 0.5in -->\nStrict."),
        "{out}"
    );
}

/// pi review av2 F13 / av3 F6: a body with no paragraph names none: no
/// `p0-p0`, and a paragraph pick is refused instead of printing nothing.
#[test]
fn an_empty_body_names_no_paragraph_and_refuses_a_pick() {
    let empty = docx("");
    let view = agent(&empty);
    let lines = header_lines(&view);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("body: no paragraphs, 0 tables, 1 page")),
        "{lines:?}"
    );
    let changed = changed_by(&empty, None, true);
    assert!(changed.contains("\nrange: changed (none)\n"), "{changed}");
    let picked = docx_to_markdown(
        &empty,
        &MarkdownOptions {
            select: Some(Select::parse("p0").unwrap()),
            ..agent_defaults()
        },
    );
    let error = picked.expect_err("p0 does not exist").to_string();
    assert!(error.contains("the body has no paragraphs"), "{error}");
}

/// pi review av2 F8: a core.xml that names nobody is not a missing one.
#[test]
fn an_owner_less_core_part_is_not_called_missing() {
    let bytes = common::docx::docx_with(
        &para("x"),
        &[core(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:creator> </dc:creator></cp:coreProperties>"#,
        )],
    );
    let view = agent(&bytes);
    assert!(
        header_lines(&view).contains(
            &"  document_owner: none             # docProps/core.xml names no dc:creator or cp:lastModifiedBy"
        ),
        "{view}"
    );
}

/// pi review av2 F5: when layout fails the view still marks pages, from the
/// document's own breaks; the warning says so instead of "no page markers".
#[test]
fn a_failed_layout_warns_that_markers_come_from_the_breaks() {
    let bytes = common::docx::docx_with(
        &para("Hello"),
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: &format!(r#"<w:styles xmlns:w="{W_NS}"><w:style"#),
        }],
    );
    let view = jubarte::markdown::read(
        &bytes,
        &jubarte::markdown::ReadOptions {
            track_changes: TrackChanges::All,
            comments: true,
            dates: false,
            page_markers: true,
            select: None,
            source: None,
        },
    )
    .unwrap();
    assert!(
        view.markdown.contains("<!-- page 1 of 1 -->"),
        "{}",
        view.markdown
    );
    assert_eq!(view.warnings.len(), 1, "{:?}", view.warnings);
    let warning = &view.warnings[0];
    assert!(
        warning.starts_with("layout failed, page markers come from the document's breaks: "),
        "{warning}"
    );
}

/// pi review av2 F10: `w:bdo`/`w:dir` are transparent to revisions as they
/// are to the text: an insertion then the same author's deletion is one
/// replacement printed in document order (#1359), and the resolved `rev`
/// agrees.
#[test]
fn a_bidi_wrapper_keeps_insertion_then_deletion_in_document_order() {
    let doc = docx(&format!(
        r#"<w:p>{}<w:bdo w:val="rtl">{}</w:bdo></w:p>"#,
        ins(2, "Ann Counsel", "b"),
        del(1, "Ann Counsel", "a")
    ));
    let tracked = agent(&doc);
    assert!(
        tracked.contains("{++b++}{r2+1 @AC}{--a--}{r2+1 @AC}"),
        "{tracked}"
    );
    let accepted = agent_with(&doc, TrackChanges::Accept, true);
    assert!(accepted.contains("<!-- p0 rev r2+1 @AC -->"), "{accepted}");
}

/// pi review av2 F19: `read --dates` reaches the view through the CLI; an
/// author with two timestamps gets them inline only with the flag.
#[test]
fn the_dates_flag_reaches_the_cli_view() {
    let dir = tempfile::tempdir().unwrap();
    let p = format!(
        "<w:p>{}{}{}</w:p>",
        ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "b"),
        run(" c "),
        ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "d")
    );
    std::fs::write(dir.path().join("dated.docx"), docx(&p)).unwrap();
    let plain = ok(&["read", "dated.docx", "--no-page-markers"], dir.path());
    assert!(plain.contains("{++b++}{r1 @AC}"), "{plain}");
    let dated = ok(
        &["read", "dated.docx", "--no-page-markers", "--dates"],
        dir.path(),
    );
    assert!(
        dated.contains("{++b++}{r1 @AC 2026-10-01T09:00:00Z}"),
        "{dated}"
    );
}

/// pi review r392 F2: a formatting change in a text box Word stores twice
/// (Choice and Fallback) counts once in the total, once for its author and
/// once on the id line.
#[test]
fn header_counts_a_text_box_formatting_change_once() {
    let story = |id: u32| {
        format!(
            r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>boxed</w:t></w:r></w:p>"#
        )
    };
    let boxed = format!(
        r#"<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:txbx><w:txbxContent>{}</w:txbxContent></wps:txbx></wps:wsp></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent>{}</w:txbxContent></v:textbox></v:shape></w:pict></mc:Fallback></mc:AlternateContent></w:r><w:r><w:t>Host</w:t></w:r></w:p>"#,
        story(31),
        story(32)
    );
    let view = agent(&docx(&boxed));
    let lines = header_lines(&view);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("revisions: 1 ") && l.ends_with("# 1 formatting")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("  AC: Ann Counsel") && l.contains("1 revision")),
        "{lines:?}"
    );
    assert!(view.contains("fmt r31 @AC"), "{view}");
    assert!(!view.contains("#32"), "{view}");
}

/// pi review av4 F10: a change only in a header leaves no body block to
/// show; the range line says where the change is to be read instead of
/// printing a bare empty view.
#[test]
fn a_header_only_change_points_at_the_headers_line() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}</w:p></w:hdr>"#,
        ins(5, "Ann Counsel", "NEW HEADER")
    );
    let sect = r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#;
    let bytes = common::docx::docx_with_sect_pr(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        sect,
    );
    let view = jubarte::markdown::changed_view(&bytes, "Ann Counsel", false, None).unwrap();
    let range = header_lines(&view)
        .into_iter()
        .find(|l| l.starts_with("range: "))
        .unwrap_or_default();
    assert!(range.contains("(none)"), "{view}");
    assert!(
        range.contains("# no body block; header and footer text is on the headers:/footers: lines"),
        "{view}"
    );
    assert!(
        view.contains("text: \"{++NEW HEADER++}{header1.r5 @AC}\""),
        "{view}"
    );
}

/// The id line names a list paragraph by the label Word prints: its level's
/// `lvlText` with each `%k` in that level's number format (`1)`, `a)`,
/// `i)`, `1.a`), while the body keeps a Markdown list marker.
#[test]
fn id_line_number_is_the_label_word_prints() {
    let lvl = |i: u8, fmt: &str, text: &str| {
        format!(
            r#"<w:lvl w:ilvl="{i}"><w:start w:val="1"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{text}"/></w:lvl>"#
        )
    };
    let numbering = Part {
        name: "word/numbering.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
        xml: &format!(
            r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0">{}{}{}{}{}{}</w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#,
            lvl(0, "decimal", "%1)"),
            lvl(1, "lowerLetter", "%2)"),
            lvl(2, "lowerRoman", "%3)"),
            lvl(3, "upperLetter", "%1.%4"),
            lvl(4, "decimalZero", r"&quot;Ch\&quot; %5"),
            lvl(5, "decimal", "X%6.%7Y")
        ),
    };
    let item = |ilvl: u8, text: &str| {
        format!(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="{ilvl}"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
        )
    };
    let body_xml = [
        item(0, "one"),
        item(1, "one a"),
        item(1, "one b"),
        item(0, "two"),
        item(1, "two a"),
        item(2, "two a i"),
        item(3, "deep"),
        item(4, "quoted"),
        item(5, "deeper"),
    ]
    .concat();
    let out = body(&agent(&common::docx::docx_with(&body_xml, &[numbering]))).to_string();
    // A level past the first is indented under the item it nests in (the
    // Markdown marker `1.` takes three columns), its id line with it.
    for (p, indent, clauses) in [
        (0, 0, r#""1)""#),
        (1, 3, r#""a)""#),
        (2, 3, r#""b)""#),
        (3, 0, r#""2)""#),
        (4, 3, r#""a)""#),
        (5, 6, r#""i)""#),
        (6, 9, r#""2.A""#),
        // pi review r392b tests F5: a quote or backslash in the label is
        // escaped, so the clause still parses.
        (7, 12, r#""\"Ch\\\" 01""#),
    ] {
        let indent = " ".repeat(indent);
        assert!(
            out.contains(&format!("\n{indent}<!-- p{p} num {clauses} -->\n")),
            "p{p} {clauses}:\n{out}"
        );
    }
    assert!(!out.contains("level"), "{out}");
    assert!(out.contains("1. one\n"), "{out}");
    // pi review r392b F7: a `%k` deeper than the paragraph's level (here
    // `%7` at level 6, undefined) makes Word print no label and no list
    // indent (Word probe lvl_undef_1010): a plain paragraph.
    assert!(out.contains("<!-- p8 -->\ndeeper"), "{out}");
}

/// A comment no story anchors (no range, no reference) is not an open
/// thread: Word draws no balloon for it. The header counts it apart.
#[test]
fn an_unanchored_comment_is_not_an_open_thread() {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}</w:comments>"#,
        comment(
            54,
            "Eric White",
            "EW",
            "2014-10-28T20:22:00Z",
            "0B0B0B0B",
            "Old."
        ),
        comment(
            9,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Check."
        )
    );
    let body_xml = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}<w:commentRangeEnd w:id="9"/>{}</w:p>"#,
        run("Fees"),
        reference(9)
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let view = agent(&bytes);
    let line = header_lines(&view)
        .into_iter()
        .find(|l| l.starts_with("comments: "))
        .unwrap_or_default();
    assert!(
        line.starts_with("comments: 1 thread open, 1 unanchored, style Normal ")
            && line
                .contains("# 2 comments: c9, c54 (unanchored: not in the body or a note, Word shows no balloon)"),
        "{line}"
    );
    assert!(!body(&view).contains("c54"), "{view}");
}

/// A comment range over several paragraphs repeats `{==…==}` in each, with
/// its note only at the end; every id line it crosses names the thread so
/// the highlights read as one comment. A one-paragraph range adds nothing.
#[test]
fn id_lines_inside_a_multi_paragraph_comment_name_it() {
    let body_xml = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}</w:p>{}<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{}</w:tc></w:tr></w:tbl><w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p><w:p><w:commentRangeStart w:id="7"/>{}<w:commentRangeEnd w:id="7"/>{}</w:p>"#,
        run("Start"),
        para("Middle"),
        para("Cell"),
        run("End"),
        reference(9),
        run("Alone"),
        reference(7)
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}</w:comments>"#,
        comment(
            9,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Span."
        ),
        comment(
            7,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0B0B0B0B",
            "One."
        )
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let out = body(&agent(&bytes)).to_string();
    for p in [0, 1, 3] {
        assert!(
            out.contains(&format!("<!-- p{p} in c9 -->")),
            "p{p}:\n{out}"
        );
    }
    assert!(
        out.contains("<!-- t0 1x1, cells p2-p2 by row, widths 2.778in, width auto, in c9 -->"),
        "{out}"
    );
    assert!(out.contains("<!-- p4 -->"), "{out}");
    assert!(
        out.contains("{==Start==}") && out.contains("{==End==}{>>c9 @AC: Span.<<}"),
        "{out}"
    );
}

/// pi review r392b F3: `read --changed` keeps every paragraph a comment
/// range crosses, not only the one that carries the note, as the hidden
/// mode already does through its `comments cN` clause.
#[test]
fn changed_keeps_every_paragraph_inside_a_comment_range() {
    let body_xml = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}</w:p>{}<w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p>{}"#,
        run("Start"),
        para("Middle"),
        run("End"),
        reference(9),
        para("Outside")
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            9,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Span."
        )
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    for by in [None, Some("@AC")] {
        let out = changed_by(&bytes, by, true);
        for p in 0..=2 {
            assert!(
                out.contains(&format!("<!-- p{p} in c9 -->")),
                "{by:?} p{p}:\n{out}"
            );
        }
        assert!(!out.contains("Outside"), "{by:?}:\n{out}");
        assert!(out.contains(" (p0, p1, p2) of p0-p3"), "{by:?}:\n{out}");
    }
}

/// A comment anchored in a header is unanchored: Word opens such a file and
/// shows no balloon for it (Word probe d27_comment_in_header, 2026-10-10),
/// and the view cannot carry it back into the header.
#[test]
fn a_comment_anchored_in_a_header_is_unanchored() {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}{}</w:comments>"#,
        comment(
            5,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Header note."
        ),
        comment(
            7,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0B0B0B0B",
            "Lost."
        )
    );
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:commentRangeStart w:id="5"/>{}<w:commentRangeEnd w:id="5"/>{}</w:p></w:hdr>"#,
        run("HdrText"),
        reference(5)
    );
    let bytes = common::docx::docx_with_sect(
        &para("Body"),
        &[
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
    );
    let view = agent(&bytes);
    let line = header_lines(&view)
        .into_iter()
        .find(|l| l.starts_with("comments: "))
        .unwrap_or_default();
    assert!(
        line.starts_with("comments: 0 threads open, 2 unanchored, style Normal ")
            && line.contains("c5 (unanchored: not in the body or a note")
            && line.contains("c7 (unanchored"),
        "{line}"
    );
}

/// pi review r392b F4: a comment range that starts inside a table and runs
/// past it is named on the table line as well.
#[test]
fn a_table_line_names_a_range_that_starts_in_a_cell() {
    let body_xml = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:commentRangeStart w:id="9"/>{}</w:p></w:tc></w:tr></w:tbl><w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p>{}"#,
        run("Cell"),
        run("After"),
        reference(9),
        para("Quiet")
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            9,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Span."
        )
    );
    let bytes = common::docx::docx_with(
        &body_xml,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let out = body(&agent(&bytes)).to_string();
    assert!(
        out.contains("<!-- t0 1x1, cells p0-p0 by row, widths 2.778in, width auto, in c9 -->"),
        "{out}"
    );
    assert!(out.contains("<!-- p1 in c9 -->"), "{out}");
    assert!(out.contains("<!-- p2 -->"), "{out}");
}

/// pi review r392b F2: the range line points at the headers:/footers: lines
/// only when a header or footer holds a change of the selection, and it
/// names the footnote blocks it keeps.
#[test]
fn the_range_line_names_kept_notes_and_points_only_at_changed_headers() {
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p>{}</w:p></w:footnote></w:footnotes>"#,
        ins(7, "Ann Counsel", "footnote change")
    );
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}</w:p></w:hdr>"#,
        run("Plain header")
    );
    let bytes = common::docx::docx_with_sect(
        r#"<w:p><w:r><w:t>Text</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[
            Part {
                name: "word/footnotes.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
                xml: &footnotes,
            },
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
    );
    let range = |by: Option<&str>| {
        header_lines(&changed_by(&bytes, by, true))
            .into_iter()
            .find(|l| l.starts_with("range: "))
            .map(str::to_string)
            .unwrap_or_default()
    };
    let kept = range(None);
    assert!(
        kept.starts_with("range: changed ([^1]) of p0-p0") && !kept.contains("headers:"),
        "{kept}"
    );
    let nobody = range(Some("Zed Zimmer"));
    assert!(
        nobody.starts_with("range: changed by Zed Zimmer (none)") && !nobody.contains('#'),
        "{nobody}"
    );
    assert!(changed_by(&bytes, None, true).contains("[^1]: {++footnote change++}"));
    assert!(!changed_by(&bytes, Some("Zed Zimmer"), true).contains("[^1]:"));
}

/// CodeRabbit #392: a comment hidden with `--comments none` still names
/// every paragraph its range runs through (`in c9`), so `--changed` keeps
/// the middle paragraph the way the inline view does.
#[test]
fn a_hidden_comment_names_every_paragraph_its_range_covers() {
    let body_xml = format!(
        r#"{}<w:p><w:commentRangeStart w:id="9"/>{}</w:p>{}<w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p>{}"#,
        para("Zero"),
        run("one"),
        para("two"),
        run("three"),
        reference(9),
        para("Quiet")
    );
    let bytes = one_comment_docx(&body_xml);
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::All, false)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nZero\n\n<!-- p1 comments c9 -->\none\n\n<!-- p2 in c9 -->\ntwo\n\n<!-- p3 comments c9 -->\nthree\n\n<!-- p4 -->\nQuiet\n"
    );
    assert_eq!(
        body(&changed_by(&bytes, None, false)),
        "<!-- p1 comments c9 -->\none\n\n<!-- p2 in c9 -->\ntwo\n\n<!-- p3 comments c9 -->\nthree\n"
    );
}

/// CodeRabbit #392: a hidden comment whose range runs through a table names
/// it on the table line, as the inline view does.
#[test]
fn a_hidden_comment_running_through_a_table_names_it_on_the_table_line() {
    let body_xml = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}</w:p><w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{}</w:tc></w:tr></w:tbl><w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p>"#,
        run("before"),
        para("cell"),
        run("after"),
        reference(9)
    );
    let bytes = one_comment_docx(&body_xml);
    for comments in [true, false] {
        let out = agent_with(&bytes, TrackChanges::All, comments);
        let table = body(&out)
            .lines()
            .find(|l| l.starts_with("<!-- t0 "))
            .unwrap_or_default()
            .to_string();
        assert!(
            table.ends_with(", in c9 -->"),
            "comments inline {comments}: {out}"
        );
    }
}

/// CodeRabbit #392: a note inserted with its reference carries its label
/// inside the mark (`{++[^1]: …++}`); a selection that keeps the citing
/// paragraph keeps the definition, and `--changed` names it.
#[test]
fn a_selection_keeps_the_definition_of_an_inserted_note() {
    let footnotes = format!(
        r#"<w:footnotes xmlns:w="{W_NS}"><w:footnote w:id="1"><w:p>{}</w:p></w:footnote></w:footnotes>"#,
        run("Added note")
    );
    let bytes = common::docx::docx_with(
        &format!(
            r#"<w:p>{}<w:ins w:id="5" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:r><w:footnoteReference w:id="1"/></w:r></w:ins></w:p>{}"#,
            run("Text"),
            para("Quiet")
        ),
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &footnotes,
        }],
    );
    let definition = "{++[^1]: Added note++}{r5 @AC}";
    let full = agent(&bytes);
    assert!(full.contains(definition), "{full}");
    let picked = agent_options(
        &bytes,
        &MarkdownOptions {
            select: Some(Select::parse("p0").unwrap()),
            ..agent_defaults()
        },
    );
    assert!(picked.contains(definition), "{picked}");
    let changed = changed_by(&bytes, None, true);
    assert!(changed.contains(definition), "{changed}");
    assert!(
        changed.contains("\nrange: changed (p0, [^1]) of p0-p1"),
        "{changed}"
    );
}

/// CodeRabbit #392: an author whose only changes sit in a header has a
/// handle like any other, so `--by` takes the handle as well as the name.
#[test]
fn an_author_who_changed_only_a_header_has_a_handle() {
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}</w:p></w:hdr>"#,
        ins(7, "Hedda Gabler", "Draft")
    );
    let bytes = common::docx::docx_with_sect(
        &para("Body"),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let full = agent(&bytes);
    assert!(full.contains("\n  HG: Hedda Gabler"), "{full}");
    for by in ["HG", "@HG", "Hedda Gabler"] {
        let out = changed_by(&bytes, Some(by), true);
        let range = header_lines(&out)
            .into_iter()
            .find(|l| l.starts_with("range: "))
            .unwrap_or_default()
            .to_string();
        assert!(
            range.contains("header and footer text is on the headers:/footers: lines"),
            "--by {by}: {range}"
        );
    }
}

#[test]
fn a_header_reviser_takes_a_handle_before_a_commenter_with_the_same_initials() {
    // pi review of #394: revision authors, a header's too, take handles
    // before commenters; the later of two equal handles gets a 2.
    let header = format!(
        r#"<w:hdr xmlns:w="{W_NS}"><w:p>{}</w:p></w:hdr>"#,
        ins(7, "Hedda Gabler", "Draft")
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            9,
            "Henry Gale",
            "HG",
            "2026-10-01T09:00:00Z",
            "0A0A0A0A",
            "Why?"
        )
    );
    let body = format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}<w:commentRangeEnd w:id="9"/>{}</w:p>"#,
        run("Body"),
        reference(9)
    );
    let bytes = common::docx::docx_with_sect(
        &body,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER_CT,
                rel_type: HEADER_REL,
                xml: &header,
            },
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = agent(&bytes);
    let reviser = out.find("\n  HG: Hedda Gabler").expect(&out);
    let commenter = out.find("\n  HG2: Henry Gale").expect(&out);
    assert!(reviser < commenter, "{out}");
    assert!(out.contains("{>>c9 @HG2: Why?<<}"), "{out}");
}

#[test]
fn selection_parser_accepts_zero_equal_endpoints_and_whitespace() {
    use jubarte::markdown::Pick;
    assert_eq!(
        Select::parse(" , p0 , p2 - p2, p3 - , - p4, t0, ,").unwrap(),
        Select::Picks(vec![
            Pick::Paragraphs {
                from: 0,
                to: Some(0)
            },
            Pick::Paragraphs {
                from: 2,
                to: Some(2)
            },
            Pick::Paragraphs { from: 3, to: None },
            Pick::Paragraphs {
                from: 0,
                to: Some(4)
            },
            Pick::Table(0),
        ])
    );
}

#[test]
fn selection_parser_rejects_malformed_items_even_after_a_valid_pick() {
    for item in [
        "p", "t", "p-", "-p", "p1-p2-p3", "t1-t2", "p1.5", "p1x", "P1",
    ] {
        let expected = format!("{item}: expected pN, pN-pM or tN");
        assert_eq!(
            Select::parse(&format!("p0, {item}")),
            Err(expected),
            "{item}"
        );
    }
    for spec in ["", " ", ",,,", " , \t, \n"] {
        assert_eq!(Select::parse(spec), Err("no paragraphs selected".into()));
    }
}

#[test]
fn selection_parser_checks_platform_integer_boundaries() {
    use jubarte::markdown::Pick;
    let max = usize::MAX;
    assert_eq!(
        Select::parse(&format!("p{max},t{max}")),
        Ok(Select::Picks(vec![
            Pick::Paragraphs {
                from: max,
                to: Some(max)
            },
            Pick::Table(max),
        ]))
    );
    let overflow = (max as u128 + 1).to_string();
    for item in [
        format!("p{overflow}"),
        format!("t{overflow}"),
        format!("p0-p{overflow}"),
    ] {
        assert_eq!(
            Select::parse(&item),
            Err(format!("{item}: expected pN, pN-pM or tN"))
        );
    }
}

#[test]
fn selection_parser_rejects_backwards_ranges_in_all_number_spellings() {
    for item in ["p2-p1", "p2 - p1"] {
        assert_eq!(
            Select::parse(item),
            Err(format!("{item}: the range runs backwards"))
        );
    }
    // A bare number at either end is expired. The settled range would run
    // backwards too, so no spelling is suggested.
    for item in ["2-1", "p2-1", "2-p1"] {
        assert_eq!(
            Select::parse(item),
            Err(format!("{item}: expected pN, pN-pM or tN"))
        );
    }
}

#[test]
fn paginate_keeps_multiple_id_lines_with_their_block() {
    let md = "<!-- p0 -->\nAlpha text here\n\n<!-- p1 empty -->\n<!-- p2 -->\nBeta text here\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["Alpha text here", "Beta text here"]),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- page 2 of 2 -->\n\n<!-- p1 empty -->\n<!-- p2 -->\nBeta text here\n"
    );
}

#[test]
fn paginate_preserves_trailing_id_lines_without_a_final_newline() {
    for md in [
        "<!-- p0 empty -->",
        "Alpha text here\n\n<!-- p1 empty -->\n<!-- p2 empty -->",
    ] {
        assert_eq!(
            jubarte::markdown::paginate(md, &["Alpha text here"]),
            format!("<!-- page 1 of 1 -->\n\n{md}")
        );
        assert_eq!(jubarte::markdown::paginate(md, &[]), md);
    }
}

#[test]
fn paginate_does_not_move_id_like_comments_out_of_code_fences() {
    let md = "Alpha text here\n\n```html\n<!-- p99 -->\nBeta text here\n```\n\n<!-- p1 -->\nGamma text here\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["Alpha text here", "Beta text here Gamma text here"]),
        "<!-- page 1 of 2 -->\n\nAlpha text here\n\n```html\n<!-- p99 -->\nBeta text here\n```\n\n<!-- page 2 of 2 -->\n\n<!-- p1 -->\nGamma text here\n"
    );
}

#[test]
fn paginate_keeps_id_lines_inside_a_continuing_list() {
    let md = "<!-- p0 -->\n- Alpha text here\n\n<!-- p1 -->\n- Beta text here\n\n<!-- p2 -->\nGamma text here\n";
    assert_eq!(
        jubarte::markdown::paginate(md, &["Alpha text here", "Beta text here Gamma text here"]),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\n- Alpha text here\n\n<!-- p1 -->\n- Beta text here\n\n<!-- page 2 of 2 -->\n\n<!-- p2 -->\nGamma text here\n"
    );
}

#[test]
fn cached_page_markers_can_be_disabled_without_losing_ids_or_text() {
    let bytes = docx(&format!(
        "{}<w:p><w:r><w:lastRenderedPageBreak/><w:t>Beta text here</w:t></w:r></w:p>",
        para("Alpha text here")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            page_markers: false,
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&out),
        "<!-- p0 -->\nAlpha text here\n\n<!-- p1 -->\nBeta text here\n"
    );
}

#[test]
fn layout_pages_override_cached_breaks_even_when_cached_markers_are_disabled() {
    let bytes = docx(&format!(
        "{}{}",
        para("Alpha text here"),
        para("Beta text here")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            pages: Some(vec!["Alpha text here".into(), "Beta text here".into()]),
            page_markers: false,
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&out),
        "<!-- page 1 of 2 -->\n\n<!-- p0 -->\nAlpha text here\n\n<!-- page 2 of 2 -->\n\n<!-- p1 -->\nBeta text here\n"
    );
}

#[test]
fn empty_layout_pages_do_not_fall_back_to_cached_breaks() {
    let bytes = docx(r#"<w:p><w:r><w:lastRenderedPageBreak/><w:t>Text</w:t></w:r></w:p>"#);
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            pages: Some(vec![]),
            ..agent_defaults()
        },
    );
    assert_eq!(body(&out), "<!-- p0 -->\nText\n");
}

#[test]
fn agent_only_options_leave_plain_conversion_byte_identical() {
    let bytes = commented_docx(THREADED);
    for track_changes in [
        TrackChanges::All,
        TrackChanges::Accept,
        TrackChanges::Reject,
    ] {
        let plain = agent_options(
            &bytes,
            &MarkdownOptions {
                track_changes,
                ..MarkdownOptions::default()
            },
        );
        let configured = agent_options(
            &bytes,
            &MarkdownOptions {
                track_changes,
                ids: false,
                comments: false,
                source: Some("different.docx".into()),
                pages: Some(vec!["Fee.".into(), "Keep it secret".into()]),
                page_markers: false,
                dates: true,
                select: Some(Select::Head(0)),
                ..MarkdownOptions::default()
            },
        );
        assert_eq!(configured, plain, "{track_changes:?}");
    }
}

#[test]
fn agent_preserves_spaces_across_runs_while_plain_conversion_collapses_them() {
    let bytes = docx(&format!(
        "<w:p>{}{}</w:p>",
        run("  Alpha  "),
        run("  Beta  ")
    ));
    // The paragraph's own leading spaces are text Word draws; Markdown would
    // drop a literal one, so the agent view writes each as `&#32;`.
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n&#32;&#32;Alpha    Beta\n"
    );
    assert_eq!(
        agent_options(&bytes, &MarkdownOptions::default()),
        "Alpha Beta\n"
    );
}

#[test]
fn insertion_before_deletion_keeps_each_author_in_document_order() {
    let bytes = docx(&format!(
        "<w:p>{}{}</w:p>",
        ins(2, "John Doe", "new"),
        del(1, "Ann Counsel", "old")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++new++}{r2 @JD}{--old--}{r1 @AC}\n"
    );
}

#[test]
fn a_plain_run_prevents_adjacent_revision_tags_from_merging() {
    let bytes = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        ins(1, "Ann Counsel", "one"),
        run(" / "),
        ins(2, "Ann Counsel", "two")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++one++}{r1 @AC} / {++two++}{r2 @AC}\n"
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Accept, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev r1 @AC; r2 @AC -->\none / two\n"
    );
}

#[test]
fn three_colliding_author_initials_stay_distinct_in_revision_notes() {
    let bytes = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        ins(1, "Ann Counsel", "one"),
        ins(2, "Al Cooper", "two"),
        ins(3, "Amy Cole", "three")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++one++}{r1 @AC}{++two++}{r2 @AC2}{++three++}{r3 @AC3}\n"
    );
}

#[test]
fn missing_revision_id_and_author_have_explicit_unknown_tags() {
    let bytes = docx("<w:p><w:ins><w:r><w:t>new</w:t></w:r></w:ins></w:p>");
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++new++}{noid @??}\n"
    );
}

/// A cell's replacement is one change on the table line, named once
/// whichever of its marks comes first (#1359), as on a paragraph's line.
#[test]
fn resolved_views_keep_table_cell_revision_ids_and_following_paragraph_numbers() {
    for (marks, id) in [
        (
            [ins(2, "Ann Counsel", "new"), del(1, "Ann Counsel", "old")],
            "r2+1",
        ),
        (
            [del(1, "Ann Counsel", "old"), ins(2, "Ann Counsel", "new")],
            "r1+2",
        ),
    ] {
        let bytes = docx(&format!(
            "<w:tbl><w:tr><w:tc><w:p>{}</w:p></w:tc></w:tr></w:tbl>{}",
            marks.concat(),
            para("After")
        ));
        for (mode, text) in [(TrackChanges::Accept, "new"), (TrackChanges::Reject, "old")] {
            assert_eq!(
                body(&agent_with(&bytes, mode, true)),
                format!(
                    "<!-- page 1 of 1 -->\n\n<!-- t0 1x1, cells p0-p0 by row, width auto, rev {id} @AC in p0 -->\n|{text}|\n|-|\n\n<!-- p1 -->\nAfter\n"
                )
            );
        }
    }
}

#[test]
fn rejecting_changes_preserves_comment_threads_and_hidden_comment_ids() {
    let bytes = commented_docx(THREADED);
    for comments in [true, false] {
        assert_eq!(
            body(&agent_with(&bytes, TrackChanges::Reject, comments)),
            body(&agent_with(&bytes, TrackChanges::All, comments))
        );
    }
}

#[test]
fn unknown_thread_parents_do_not_create_dangling_reply_tags() {
    let extended = THREADED.replace(
        "w15:paraIdParent=\"11A5D0F2\"",
        "w15:paraIdParent=\"FFFFFFFF\"",
    );
    let out = agent(&commented_docx(&extended));
    assert!(body(&out).contains("{>>c6 @AS: Disagree.<<}"), "{out}");
    assert!(!body(&out).contains(" re #c"), "{out}");
}

#[test]
fn resolved_boolean_true_is_equivalent_to_one_on_comment_threads() {
    let numeric = THREADED.replacen("w15:done=\"0\"", "w15:done=\"1\"", 1);
    let boolean = THREADED.replacen("w15:done=\"0\"", "w15:done=\"true\"", 1);
    let out = agent(&commented_docx(&boolean));
    assert!(body(&out).contains("c5 @AC resolved:"), "{out}");
    // Two files, two digests; every other line is the same.
    let without_digest = |view: &str| -> String {
        view.lines()
            .filter(|l| !l.starts_with("sha256: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        without_digest(&out),
        without_digest(&agent(&commented_docx(&numeric)))
    );
}

#[test]
fn comment_dates_include_revision_dates_when_deciding_whether_to_print_inline() {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            5,
            "Ann Counsel",
            "AC",
            "2026-10-03T14:05:00Z",
            "11A5D0F2",
            "Check this."
        )
    );
    let bytes = common::docx::docx_with(
        &format!(
            "<w:p>{}{}</w:p>",
            ins(1, "Ann Counsel", "New text"),
            reference(5)
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    let dated = agent_options(
        &bytes,
        &MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&dated),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++New text++}{r1 @AC 2026-10-01T09:00:00Z}{>>c5 @AC 2026-10-03T14:05:00Z: Check this.<<}\n"
    );
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++New text++}{r1 @AC}{>>c5 @AC: Check this.<<}\n"
    );
}

#[test]
fn hidden_comments_deduplicate_references_and_ignore_missing_comments() {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" {W14}>{}</w:comments>"#,
        comment(
            5,
            "Ann Counsel",
            "AC",
            "2026-10-01T09:00:00Z",
            "11A5D0F2",
            "Check this."
        )
    );
    let bytes = common::docx::docx_with(
        &format!(
            "<w:p>{}{}{}{}</w:p>{}",
            run("Text"),
            reference(5),
            reference(5),
            reference(999),
            para("After")
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::All, false)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 comments c5 -->\nText\n\n<!-- p1 -->\nAfter\n"
    );
}

#[test]
fn changed_author_handles_match_whole_handles_including_collision_suffixes() {
    let bytes = docx(&format!(
        "<w:p>{}</w:p><w:p>{}</w:p>{}",
        ins(0, "Ann Counsel", "First author"),
        ins(1, "Alice Cooper", "Second author"),
        para("Quiet")
    ));
    // Both authors have the same initials; matching @AC must not match @AC2.
    for (by, kept, excluded) in [
        ("Ann Counsel", "First author", "Second author"),
        ("Alice Cooper", "Second author", "First author"),
    ] {
        let out = agent_options(&bytes, &changed(Some(by)));
        assert!(body(&out).contains(kept), "{out}");
        assert!(!body(&out).contains(excluded), "{out}");
        assert!(!body(&out).contains("Quiet"), "{out}");
    }
    for unknown in ["@missing", "@", "", "ann counsel", "AC999"] {
        let out = agent_options(&bytes, &changed(Some(unknown)));
        assert!(body(&out).trim().is_empty(), "{unknown}: {out}");
    }
}

#[test]
fn changed_keeps_comment_only_blocks_and_ignores_orphan_references() {
    let bytes = one_comment_docx(&format!(
        "{}<w:p>{}{}</w:p><w:p>{}{}</w:p>",
        para("Quiet"),
        run("Reviewed"),
        reference(9),
        run("Orphan"),
        reference(99)
    ));
    for comments in [true, false] {
        let out = agent_options(
            &bytes,
            &MarkdownOptions {
                comments,
                ..changed(None)
            },
        );
        assert!(out.contains("range: changed (p1) of p0-p2"), "{out}");
        assert!(body(&out).contains("Reviewed"), "{out}");
        assert!(
            !body(&out).contains("Quiet") && !body(&out).contains("Orphan"),
            "{out}"
        );
    }
    let by_author = agent_options(&bytes, &changed(Some("Ann Counsel")));
    assert!(body(&by_author).contains("Reviewed"), "{by_author}");
    assert!(!body(&by_author).contains("Orphan"), "{by_author}");
}

#[test]
fn changed_selects_whole_tables_for_hidden_comments() {
    let bytes = one_comment_docx(&format!(
        "<w:tbl><w:tr><w:tc>{}</w:tc><w:tc><w:p>{}{}</w:p></w:tc></w:tr></w:tbl>{}",
        para("Context cell"),
        run("Reviewed cell"),
        reference(9),
        para("Quiet")
    ));
    let out = agent_options(
        &bytes,
        &MarkdownOptions {
            comments: false,
            ..changed(None)
        },
    );
    assert!(out.contains("range: changed (t0) of p0-p2"), "{out}");
    assert!(
        body(&out).contains("Context cell") && body(&out).contains("Reviewed cell"),
        "{out}"
    );
    assert!(!body(&out).contains("Quiet"), "{out}");
}

#[test]
fn changed_finds_formatting_and_break_marks_without_inline_revisions() {
    let bytes = docx(&format!(
        r#"{}<w:p><w:pPr><w:rPr><w:ins w:id="4" w:author="Ann Counsel"/></w:rPr></w:pPr>{}</w:p><w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Bob Lee"><w:rPr/></w:rPrChange></w:rPr><w:t>Formatted</w:t></w:r></w:p>"#,
        para("Quiet"),
        run("Split")
    ));
    let all = agent_options(&bytes, &changed(None));
    assert!(all.contains("range: changed (p1, p2) of p0-p2"), "{all}");
    for (by, kept, excluded) in [("AC", "Split", "Formatted"), ("BL", "Formatted", "Split")] {
        let out = agent_options(&bytes, &changed(Some(by)));
        assert!(body(&out).contains(kept), "{out}");
        assert!(
            !body(&out).contains(excluded) && !body(&out).contains("Quiet"),
            "{out}"
        );
    }
}

#[test]
fn changed_in_reject_view_keeps_the_deleted_authors_block() {
    let out = agent_options(
        &marked_docx(),
        &MarkdownOptions {
            track_changes: TrackChanges::Reject,
            ..changed(Some("Bob Lee"))
        },
    );
    assert!(out.contains("range: changed by @BL (p3) of p0-p3"), "{out}");
    assert!(body(&out).contains("Fee: waived"), "{out}");
    assert!(
        !body(&out).contains("Pay in") && !body(&out).contains("Quiet"),
        "{out}"
    );
    assert!(!body(&out).contains("{--"), "{out}");
}

#[test]
fn changed_flags_reject_even_zero_length_or_empty_conflicting_selections() {
    for (paragraphs, head, tail) in [
        (Some(""), None, None),
        (None, Some(0), None),
        (None, None, Some(0)),
    ] {
        assert_eq!(
            Select::from_flags(paragraphs, head, tail, true, None),
            Err("changed excludes paragraphs, head and tail".into())
        );
    }
    assert_eq!(
        Select::from_flags(None, None, None, false, Some("")),
        Err("by needs changed".into())
    );
    assert_eq!(Select::from_flags(None, None, None, false, None), Ok(None));
    assert_eq!(
        Select::from_flags(None, None, None, true, None),
        Ok(Some(Select::Changed { by: None }))
    );
    assert_eq!(
        Select::from_flags(None, None, Some(0), false, None),
        Ok(Some(Select::Tail(0)))
    );
}

#[test]
fn pr392_yaml_source_round_trips_reserved_scalars_and_control_characters() {
    let bytes = docx(&para("Body"));
    for source in [
        "",
        "null",
        "TRUE",
        "off",
        "~",
        "0123",
        "12:30",
        "2026-10-01",
        "-draft",
        "[draft]",
        "name: draft",
        "draft #1",
        " trailing ",
        "quoted \"draft\"\\copy",
        "first\nsecond\tcolumn",
        "café.docx",
    ] {
        let view = agent_options(
            &bytes,
            &MarkdownOptions {
                source: Some(source.into()),
                ..agent_defaults()
            },
        );
        assert_eq!(
            header_yaml(&view)["source"].as_str(),
            Some(source),
            "{source:?}: {view}"
        );
        assert!(body(&view).contains("<!-- p0 -->\nBody"), "{view}");
    }
}

/// PR #392, restated for the drawing payload (54621454): Word renders one
/// branch of an `mc:AlternateContent`; the view carries the element whole
/// as one atomic payload, so no branch's text prints, while every listed
/// change — the Choice's and the Fallback's alike — keeps its author a
/// handle and is named, by paragraph, on `unplaced:`.
#[test]
fn pr392_alternate_content_carries_one_payload_and_names_every_listed_author() {
    let choice = format!(
        r#"<mc:Choice Requires="wps"><w:drawing><w:txbxContent><w:p>{}</w:p></w:txbxContent></w:drawing></mc:Choice>"#,
        ins(1, "Ann Counsel", "Chosen")
    );
    let fallback = format!(
        r#"<mc:Fallback><w:pict><w:txbxContent><w:p>{}</w:p></w:txbxContent></w:pict></mc:Fallback>"#,
        ins(2, "Bob Day", "Fallback")
    );
    for (branches, kept, absent, handle, author, unplaced) in [
        (
            format!("{choice}{fallback}"),
            "Chosen",
            "Fallback",
            "AC",
            "Ann Counsel",
            vec!["r2 @BD in p0"],
        ),
        (fallback, "Fallback", "Chosen", "BD", "Bob Day", vec![]),
    ] {
        let bytes = docx(&format!(
            r#"<w:p>{}<w:r><mc:AlternateContent>{branches}</mc:AlternateContent></w:r></w:p>"#,
            run("Host")
        ));
        let view = agent(&bytes);
        // One payload for the whole element, not one token per branch; the
        // branch Word shows prints its box once, as comment lines beside
        // the drawing, and the other branch's text never prints.
        assert_eq!(body(&view).matches("![][d1]").count(), 1, "{view}");
        assert_eq!(
            body(&view).matches(&format!("{{++{kept}++}}")).count(),
            1,
            "{view}"
        );
        assert!(!body(&view).contains(absent), "{view}");
        // The hidden branch's change prints nowhere: it is named on
        // `unplaced:`; the shown branch's is placed in its box.
        let lines = header_lines(&view);
        let unplaced_line = lines
            .iter()
            .find(|l| l.starts_with("unplaced: "))
            .map_or("", |l| &**l);
        for entry in &unplaced {
            assert!(unplaced_line.contains(entry), "{entry}: {view}");
        }
        assert!(!unplaced_line.contains(&format!("@{handle}")), "{view}");
        let yaml = header_yaml(&view);
        assert_eq!(yaml["authors"][handle].as_str(), Some(author), "{view}");
        // A Fallback change that is no twin of the Choice's (another text)
        // is still a change of the file: listed and counted (D8), carried
        // in the payload and named on `unplaced:`.
        let changes = jubarte::changes::list_changes(&bytes).unwrap();
        let listed = changes.len();
        // Handles name the authors of the listed changes: a Fallback
        // change the Choice does not twin prints on `unplaced:` with its
        // author's handle (codex 4244120690). The only other entry is
        // document_owner.
        let authors: std::collections::BTreeSet<_> =
            changes.iter().filter_map(|c| c.author.as_deref()).collect();
        assert_eq!(
            yaml["authors"].as_mapping().unwrap().len(),
            1 + authors.len(),
            "{view}"
        );
        assert!(
            header_lines(&view)
                .iter()
                .any(|line| line.starts_with(&format!("revisions: {listed} "))),
            "{view}"
        );
    }
}

#[test]
fn pr392_changed_filter_does_not_treat_literal_critic_forms_as_revisions() {
    for literal in [
        "{++new++}",
        "{--old--}",
        "{~~old~>new~~}",
        "{==highlight==}",
        "{r7 @AC: note}",
    ] {
        let bytes = docx(&format!(
            "{}<w:p>{}</w:p>",
            para(literal),
            ins(7, "Ann Counsel", "Actual change")
        ));
        for by in [None, Some("AC"), Some("@AC"), Some("Ann Counsel")] {
            let view = changed_by(&bytes, by, true);
            assert!(
                view.contains(" (p1) of p0-p1\n"),
                "{literal}, {by:?}: {view}"
            );
            assert!(body(&view).contains("{++Actual change++}"), "{view}");
            assert!(!body(&view).contains("<!-- p0"), "{view}");
        }
    }
}

#[test]
fn pr392_footer_change_hint_is_scoped_to_the_selected_author() {
    let footer = format!(
        r#"<w:ftr xmlns:w="{W_NS}"><w:p>{}</w:p></w:ftr>"#,
        ins(1, "Ann Counsel", "New footer")
    );
    let bytes = common::docx::docx_with_sect(
        &format!("<w:p>{}</w:p>", ins(2, "Bob Day", "Body change")),
        &[Part {
            name: "word/footer1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
            xml: &footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    );
    for (by, expected_blocks, hint) in [
        // Header/footer-only authors are selected by full name; this view
        // assigns handles to body/note revisions and comments.
        ("Ann Counsel", "(none)", true),
        ("BD", "(p0)", false),
        ("Bob Day", "(p0)", false),
        ("Nobody", "(none)", false),
    ] {
        let view = changed_by(&bytes, Some(by), true);
        let range = header_lines(&view)
            .into_iter()
            .find(|line| line.starts_with("range:"))
            .unwrap();
        assert!(range.contains(expected_blocks), "{by}: {view}");
        assert_eq!(range.contains("headers:/footers:"), hint, "{by}: {view}");
    }
}

#[test]
fn pr392_comment_range_ending_in_a_table_does_not_leak_to_the_next_paragraph() {
    let bytes = one_comment_docx(&format!(
        r#"<w:p><w:commentRangeStart w:id="9"/>{}</w:p><w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p>{}<w:commentRangeEnd w:id="9"/>{}</w:p></w:tc></w:tr></w:tbl>{}"#,
        run("Start"),
        run("End"),
        reference(9),
        para("Quiet")
    ));
    let view = agent(&bytes);
    assert!(view.contains("<!-- p0 in c9 -->"), "{view}");
    assert!(
        view.contains("<!-- t0 1x1, cells p1-p1 by row, widths 2.778in, width auto, in c9 -->"),
        "{view}"
    );
    assert!(view.contains("<!-- p2 -->\nQuiet"), "{view}");
    let selected = changed_by(&bytes, Some("AC"), true);
    assert!(body(&selected).contains("<!-- t0"), "{selected}");
    assert!(!body(&selected).contains("Quiet"), "{selected}");
}

#[test]
fn pr392_word_list_labels_keep_start_values_across_number_format_boundaries() {
    for (format, start, labels) in [
        ("lowerLetter", 26, ["z)", "aa)"]),
        ("upperLetter", 26, ["Z)", "AA)"]),
        ("upperRoman", 49, ["XLIX)", "L)"]),
        ("decimalZero", 9, ["09)", "10)"]),
    ] {
        let numbering = format!(
            r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="{start}"/><w:numFmt w:val="{format}"/><w:lvlText w:val="%1)"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
        );
        let item = |text| {
            format!(
                r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#
            )
        };
        let bytes = common::docx::docx_with(
            &format!("{}{}", item("First"), item("Second")),
            &[Part {
                name: "word/numbering.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                xml: &numbering,
            }],
        );
        let view = agent(&bytes);
        for (index, label) in labels.iter().enumerate() {
            assert!(
                view.contains(&format!("<!-- p{index} num \"{label}\" -->")),
                "{format}: {view}"
            );
        }
        assert!(body(&view).contains(&format!("{start}. First")), "{view}");
        assert!(
            body(&view).contains(&format!("{}. Second", start + 1)),
            "{view}"
        );
    }
}

#[test]
fn table_cells_preserve_adjacent_backslashes_pipes_and_unicode_in_both_views() {
    for (text, escaped) in [
        (r"é\|尾", r"é\\\|尾"),
        (r"a\\|b", r"a\\\\\|b"),
        (r"ends\", r"ends\\"),
        ("a|b|c", r"a\|b\|c"),
    ] {
        let bytes = docx(&format!(
            "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl><w:p/>",
            para(text)
        ));
        for ids in [false, true] {
            let out = agent_options(
                &bytes,
                &MarkdownOptions {
                    ids,
                    ..MarkdownOptions::default()
                },
            );
            let row = format!("|{escaped}|");
            assert!(
                out.lines().any(|line| line == row),
                "ids={ids}, {text:?}: {out}"
            );
        }
    }
}

#[test]
fn an_internal_link_does_not_read_as_underlined() {
    // Word's Hyperlink style underlines a link to a bookmark (a table of
    // contents line) too: that is the link's look, not `<u>`. The link
    // reads back as the Markdown link it was, and the heading's bookmark,
    // which the writer makes from its slug, prints no `<a id>`.
    let md = "# Intro\n\nSee [the intro](#intro).\n";
    let docx = markdown_to_docx(md, &DocxOptions::default()).unwrap().docx;
    let full = agent(&docx);
    let view = body(&full);
    assert!(view.contains("\n# Intro\n"), "{view}");
    assert!(view.contains("\nSee [the intro](#intro).\n"), "{view}");
}

#[test]
fn a_field_across_a_change_boundary_shows_its_result_only() {
    // `{CODE}` and `{/}` print only where the field's begin, separate and
    // end share their tracked changes: the writer reads a field inside one
    // change, never one cut by `{++`, so a cut field shows its result.
    let ins = r#"<w:ins w:id="10" w:author="Ann Counsel" w:date="2026-01-01T00:00:00Z">"#;
    let del = r#"<w:del w:id="11" w:author="Ann Counsel" w:date="2026-01-01T00:00:00Z">"#;
    let begin = r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r>"#;
    let separate = r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#;
    let end = r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#;
    let p = format!(
        "<w:p>{}{ins}{begin}{separate}</w:ins>{}{end}{}</w:p>\
         <w:p>{}{begin}{separate}{del}<w:r><w:delText>1</w:delText></w:r>{end}</w:del>{}</w:p>\
         <w:p>{}{begin}{separate}{del}<w:r><w:delText>1</w:delText></w:r></w:del>{end}{}</w:p>",
        run("A "),
        run("1"),
        run(" B."),
        run("C "),
        run(" D."),
        run("E "),
        run(" F."),
    );
    let full = agent(&docx(&p));
    let view = body(&full);
    assert!(
        !view.contains("A {PAGE}") && !view.contains("{++{PAGE}"),
        "{view}"
    );
    assert!(!view.contains("C {PAGE}"), "{view}");
    assert!(view.contains("E {PAGE}{--1--}"), "{view}");
    // The insertion that holds only the field's start shows as `rev` on
    // p0's id line and has nothing to write back; the rest survives, and
    // from there on the view is a fixed point.
    let once = agent(
        &markdown_to_docx(&full, &DocxOptions::default())
            .unwrap()
            .docx,
    );
    assert!(body(&once).contains("C {--1--}"), "{once}");
    assert!(body(&once).contains("E {PAGE}{--1--}"), "{once}");
    let twice = agent(
        &markdown_to_docx(&once, &DocxOptions::default())
            .unwrap()
            .docx,
    );
    assert_eq!(body(&twice), body(&once));
}

#[test]
fn a_field_code_with_quoted_braces_reads_back() {
    let field = |code: &str, result: &str| {
        format!(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> {code} </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            run(result)
        )
    };
    let p = format!(
        "<w:p>{}{}{}</w:p>",
        field("QUOTE &quot;a}b&quot;", "X"),
        run(" and {QUOTE \"c}d\"}literal{/}."),
        ""
    );
    let full = agent(&docx(&p));
    let view = body(&full);
    assert!(
        view.contains("{QUOTE \"a}b\"}X{/} and \\{QUOTE \"c}d\"}literal\\{/}."),
        "{view}"
    );
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

#[test]
fn an_empty_table_row_keeps_its_place_in_the_agent_view() {
    // The table's id line counts its rows (row1, row2...): an empty row
    // dropped from the Markdown table would shift every row after it.
    let cell = |t: &str| {
        format!(
            "<w:tc><w:p>{}</w:p></w:tc>",
            if t.is_empty() { String::new() } else { run(t) }
        )
    };
    let row = |a: &str, b: &str| format!("<w:tr>{}{}</w:tr>", cell(a), cell(b));
    let tbl = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid>{}{}{}</w:tbl>"#,
        row("Item", "Due"),
        row("", ""),
        row("Call", "Monthly")
    );
    let full = agent(&docx(&format!("{tbl}<w:p>{}</w:p>", run("End."))));
    let view = body(&full);
    let lines: Vec<&str> = view.lines().filter(|l| l.starts_with('|')).collect();
    assert_eq!(lines.len(), 4, "{view}");
    let back = markdown_to_docx(&full, &DocxOptions::default())
        .unwrap()
        .docx;
    assert_eq!(body(&agent(&back)), view);
}

/// codex 4244120690: a change only a text box's `mc:Fallback` holds is
/// listed, so its author, with no change in the `mc:Choice`, gets a handle
/// and a legend line, and the `unplaced:` entry names it.
#[test]
fn a_fallback_only_author_gets_a_handle() {
    let body = format!(
        r#"<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#,
        drawing_box(r#"<w:r><w:t>Box</w:t></w:r>"#),
        pict_box(&format!(
            r#"<w:r><w:t>Box</w:t></w:r>{}"#,
            ins(21, "Zoe Fallback", "Other")
        ))
    );
    let out = agent(&docx(&body));
    let lines = header_lines(&out);
    assert!(
        lines.iter().any(|l| l.starts_with("  ZF: Zoe Fallback")),
        "{out}"
    );
    let unplaced = lines
        .iter()
        .find(|l| l.starts_with("unplaced: "))
        .unwrap_or_else(|| panic!("no unplaced line in {out}"));
    assert!(unplaced.contains("r21 @ZF"), "{out}");
}

/// codex 4244378287: free-text header fields (the title, the owner, an
/// author's name) that read like a tag do not count as printing it.
#[test]
fn a_title_or_name_that_reads_like_a_tag_prints_no_change() {
    let body = format!(
        r#"<w:p>{}</w:p><w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#,
        ins(1, "Ann Counsel", "x"),
        drawing_box(
            r#"<w:r><w:t>Box</w:t></w:r><w:del w:id="3" w:author="Ann Counsel"><w:r><w:drawing/></w:r></w:del>"#
        ),
        pict_box(r#"<w:r><w:t>Box</w:t></w:r>"#)
    );
    let sect = r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#;
    let bytes = common::docx::docx_with_sect_pr(
        &body,
        &[core(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>r3 @AC</dc:title><dc:creator>r3 @AC</dc:creator></cp:coreProperties>"#,
        )],
        sect,
    );
    let out = agent(&bytes);
    let lines = header_lines(&out);
    assert!(lines.iter().any(|l| l.starts_with("title: ")), "{out}");
    let unplaced = lines
        .iter()
        .find(|l| l.starts_with("unplaced: "))
        .unwrap_or_else(|| panic!("no unplaced line in {out}"));
    assert!(unplaced.contains("r3 @AC"), "{out}");
}

/// codex 4244378292: a story id outside the short grammar
/// (`Header_1:p:0`) holds colons, so its flow-map value is quoted.
#[test]
fn a_long_story_id_is_quoted_in_the_header() {
    let header =
        format!(r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t>Head</w:t></w:r></w:p></w:hdr>"#);
    let bytes = common::docx::docx_with_sect_pr(
        "<w:p><w:r><w:t>Body</w:t></w:r></w:p>",
        &[Part {
            name: "word/Header_1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:sectPr><w:headerReference w:type="default" r:id="rIdX0"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    let out = agent(&bytes);
    assert!(
        out.contains(r#"{id: "Header_1:p:0", text: Head, style: Normal}"#),
        "{out}"
    );
    let yaml = header_yaml(&out);
    assert_eq!(
        yaml["headers"]["default"]["id"].as_str(),
        Some("Header_1:p:0")
    );
}

/// codex 4244378295: a default style whose id is `changes` keeps its
/// `styles:` entry; the style changes print under `style_changes:`.
#[test]
fn a_style_named_changes_does_not_collide_with_the_style_changes() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="changes"><w:name w:val="changes"/><w:rPr><w:b/><w:rPrChange w:id="9" w:author="Ann Counsel" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#
    );
    let bytes = common::docx::docx_with_sect_pr(
        "<w:p><w:r><w:t>Text</w:t></w:r></w:p>",
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>"#,
    );
    let out = agent(&bytes);
    let yaml = header_yaml(&out);
    assert!(yaml["styles"]["changes"].as_str().is_some(), "{out}");
    assert_eq!(
        yaml["style_changes"]["changes"].as_str(),
        Some("styles.r9 @AC"),
        "{out}"
    );
    assert!(!out.contains("unplaced:"), "{out}");
}

/// coderabbit 4269828968 (#427): a `w:id` that holds a `+` (`1+2`, only a
/// malformed file has one) is one change. Its tag prints the long id
/// `jubarte changes` lists, never `r1+2`, which names r1 and r2.
#[test]
fn a_revision_id_holding_a_plus_prints_as_one_change() {
    let bytes = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Lead </w:t></w:r><w:ins w:id="1+2" w:author="Ann Counsel"><w:r><w:t>new</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> mid </w:t></w:r>{}</w:p>"#,
        ins(1, "Ann Counsel", "one")
    ));
    let listed: Vec<String> = jubarte::changes::list_changes(&bytes)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(listed.contains(&"body:rev:1+2".to_string()), "{listed:?}");
    let out = agent(&bytes);
    assert!(out.contains("{body:rev:1+2 @AC}"), "{out}");
    assert!(out.contains("{r1 @AC 2026-10-01T09:00:00Z}"), "{out}");
    assert!(!out.contains("r1+2 @AC"), "{out}");
    assert!(!out.contains("unplaced:"), "{out}");
}

#[test]
fn undated_peer_keeps_a_dated_unplaced_change_timestamp() {
    let alternate = format!(
        r#"<mc:AlternateContent><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent>"#,
        drawing_box(&run("Choice")),
        pict_box(&ins(3, "Alice", "Fallback")),
    );
    let source = docx(&format!(
        r#"<w:p><w:r>{alternate}</w:r><w:ins w:id="2" w:author="Alice"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#
    ));
    let view = agent(&source);
    assert!(
        view.contains("ins r3 @A 2026-10-01T09:00:00Z in p0"),
        "{view}"
    );
    let back = markdown_to_docx(&view, &DocxOptions::default())
        .unwrap()
        .docx;
    let changes = jubarte::changes::list_changes(&back).unwrap();
    assert!(
        changes.iter().any(|c| c.text == "new" && c.date.is_none()),
        "{changes:?}"
    );
}
