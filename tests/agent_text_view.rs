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
    assert_eq!(
        Select::parse("-").unwrap_err(),
        "-: expected pN, pN-pM or tN"
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

const TABLE_3X2: &str = r#"<w:tbl><w:tblPr><w:jc w:val="center"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:trPr><w:tblHeader/></w:trPr><w:tc><w:p><w:r><w:t>Item</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Due</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Report</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Day 10</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Call</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Monthly</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;

#[test]
fn table_line_gives_grid_cells_and_header_row() {
    let bytes = docx(&format!("{}{TABLE_3X2}{}", para("Before"), para("After")));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nBefore\n\n<!-- t0 center 3x2, cells p1-p6 by row, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p7 -->\nAfter\n"
    );
}

#[test]
fn table_line_lists_rows_when_a_cell_holds_two_paragraphs_and_notes_merges_and_marks() {
    let tbl = r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/></w:tcPr><w:p><w:r><w:t>Head</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:pPr><w:rPr><w:ins w:id="12" w:author="Ann Counsel"/></w:rPr></w:pPr><w:r><w:t>a</w:t></w:r></w:p><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let out = body(&agent(&docx(tbl))).to_string();
    assert!(out.starts_with("<!-- page 1 of 1 -->\n\n<!-- t0 2x2, cells r0 p0 r1 p1-p3, merged cells, break-ins #12 @AC in p1 -->\n"), "{out}");
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
        del(3, "Ann Counsel", "thirty"),
        ins(4, "Ann Counsel", "forty-five"),
        run(" days, "),
        ins(5, "Ann Counsel", "quarterly"),
        run(" reports"),
        del(6, "Ann Counsel", ", nothing else")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nPay within {~~thirty~>forty-five~~}{>>#3+4 @AC<<} days, {++quarterly++}{>>#5 @AC<<} reports{--, nothing else--}{>>#6 @AC<<}\n"
    );
}

#[test]
fn neighbouring_marks_by_one_author_share_one_note_and_keep_their_text() {
    let p = format!(
        "<w:p>{}{}{}</w:p>",
        run("a "),
        ins(7, "Ann Counsel", "bold"),
        ins(8, "Ann Counsel", " words")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\na {++bold words++}{>>#7+8 @AC<<}\n"
    );
}

#[test]
fn a_substitution_by_two_authors_gets_two_notes_deleted_side_first() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("x "),
        del(1, "Ann Counsel", "old"),
        ins(2, "John Doe", "new"),
        ins(3, "Ann Counsel", "!")
    );
    assert_eq!(
        body(&agent(&docx(&p))),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nx {~~old~>new~~}{>>#1 @AC<<}{>>#2 @JD<<}{++!++}{>>#3 @AC<<}\n"
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
        body(&agent(&bytes)).contains("{++b++}{>>#1 @AC<<} c {++d++}{>>#2 @AC<<}"),
        "{}",
        agent(&bytes)
    );
    let dated = agent_options(
        &bytes,
        MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert!(
        body(&dated).contains(
            "{++b++}{>>#1 @AC 2026-10-01T09:00:00Z<<} c {++d++}{>>#2 @AC 2026-10-03T14:05:00Z<<}"
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
        MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert!(body(&dated).contains("{++b++}{>>#1 @AC<<}"), "{dated}");
}

#[test]
fn legacy_output_keeps_author_notes() {
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(7, "Ann Counsel", "b"));
    let legacy = docx_to_markdown(&docx(&p), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert_eq!(
        legacy,
        "a {++b++}{>>Ann Counsel (2026-10-01T09:00:00Z)<<}\n"
    );
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
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nFee. {==Late amounts accrue interest.==}{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}\n\n<!-- p1 -->\nKeep it secret{>>#c11 @AC: Add survival?<<}.\n"
    );
}

#[test]
fn two_independent_comments_on_one_anchor_are_not_a_thread() {
    let independent = THREADED.replace(r#" w15:paraIdParent="11A5D0F2""#, "");
    let out = body(&agent(&commented_docx(&independent))).to_string();
    assert!(
        out.contains("{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS: Disagree.<<}"),
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
        out.contains("{>>#c5 @AC resolved: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}"),
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
        "<!-- page 1 of 1 -->\n\n<!-- p0 comments #c5 #c6 -->\nFee. Late amounts accrue interest.\n\n<!-- p1 comments #c11 -->\nKeep it secret.\n"
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
        del(3, "Ann Counsel", "thirty"),
        ins(4, "Ann Counsel", "forty-five")
    );
    let bytes = docx(&format!("{p0}{p1}"));
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Accept, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev #0 @AC; #1 @AC -->\nDeliver quarterly reports\n\n<!-- p1 rev #3+4 @AC -->\nWithin forty-five\n"
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Reject, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev #0 @AC; #1 @AC -->\nDeliver  reports weekly\n\n<!-- p1 rev #3+4 @AC -->\nWithin thirty\n"
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
        out.contains("{>>#c5 @AC: Cap in Delaware?<<}{>>#c6 @AS re #c5: Disagree.<<}"),
        "{out}"
    );
}

#[test]
fn selection_parser_accepts_zero_equal_endpoints_and_whitespace() {
    use jubarte::markdown::Pick;
    assert_eq!(
        Select::parse(" , p0 , 2 - p2, p3 - , - p4, t0, ,").unwrap(),
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
    for item in ["p2-p1", "2-1", "p2-1", "2-p1", "p2 - p1"] {
        assert_eq!(
            Select::parse(item),
            Err(format!("{item}: the range runs backwards"))
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
        MarkdownOptions {
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
        MarkdownOptions {
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
        MarkdownOptions {
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
            MarkdownOptions {
                track_changes,
                ..MarkdownOptions::default()
            },
        );
        let configured = agent_options(
            &bytes,
            MarkdownOptions {
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
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\nAlpha    Beta\n"
    );
    assert_eq!(
        agent_options(&bytes, MarkdownOptions::default()),
        "Alpha Beta\n"
    );
}

#[test]
fn insertion_before_deletion_still_attributes_the_deleted_side_first() {
    let bytes = docx(&format!(
        "<w:p>{}{}</w:p>",
        ins(2, "John Doe", "new"),
        del(1, "Ann Counsel", "old")
    ));
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{~~old~>new~~}{>>#1 @AC<<}{>>#2 @JD<<}\n"
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
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++one++}{>>#1 @AC<<} / {++two++}{>>#2 @AC<<}\n"
    );
    assert_eq!(
        body(&agent_with(&bytes, TrackChanges::Accept, true)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 rev #1 @AC; #2 @AC -->\none / two\n"
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
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++one++}{>>#1 @AC<<}{++two++}{>>#2 @AC2<<}{++three++}{>>#3 @AC3<<}\n"
    );
}

#[test]
fn missing_revision_id_and_author_have_explicit_unknown_tags() {
    let bytes = docx("<w:p><w:ins><w:r><w:t>new</w:t></w:r></w:ins></w:p>");
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++new++}{>>#? @??<<}\n"
    );
}

#[test]
fn resolved_views_keep_table_cell_revision_ids_and_following_paragraph_numbers() {
    let bytes = docx(&format!(
        "<w:tbl><w:tr><w:tc><w:p>{}{}</w:p></w:tc></w:tr></w:tbl>{}",
        del(1, "Ann Counsel", "old"),
        ins(2, "Ann Counsel", "new"),
        para("After")
    ));
    for (mode, text) in [(TrackChanges::Accept, "new"), (TrackChanges::Reject, "old")] {
        assert_eq!(
            body(&agent_with(&bytes, mode, true)),
            format!(
                "<!-- page 1 of 1 -->\n\n<!-- t0 1x1, cells p0-p0 by row, rev #1+2 @AC in p0 -->\n|{text}|\n|-|\n\n<!-- p1 -->\nAfter\n"
            )
        );
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
    assert!(body(&out).contains("{>>#c6 @AS: Disagree.<<}"), "{out}");
    assert!(!body(&out).contains(" re #c"), "{out}");
}

#[test]
fn resolved_boolean_true_is_equivalent_to_one_on_comment_threads() {
    let numeric = THREADED.replacen("w15:done=\"0\"", "w15:done=\"1\"", 1);
    let boolean = THREADED.replacen("w15:done=\"0\"", "w15:done=\"true\"", 1);
    let out = agent(&commented_docx(&boolean));
    assert!(body(&out).contains("#c5 @AC resolved:"), "{out}");
    assert_eq!(out, agent(&commented_docx(&numeric)));
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
        MarkdownOptions {
            dates: true,
            ..agent_defaults()
        },
    );
    assert_eq!(
        body(&dated),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++New text++}{>>#1 @AC 2026-10-01T09:00:00Z<<}{>>#c5 @AC 2026-10-03T14:05:00Z: Check this.<<}\n"
    );
    assert_eq!(
        body(&agent(&bytes)),
        "<!-- page 1 of 1 -->\n\n<!-- p0 -->\n{++New text++}{>>#1 @AC<<}{>>#c5 @AC: Check this.<<}\n"
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
        "<!-- page 1 of 1 -->\n\n<!-- p0 comments #c5 -->\nText\n\n<!-- p1 -->\nAfter\n"
    );
}
