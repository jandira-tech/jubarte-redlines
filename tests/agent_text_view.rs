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
        del(1, "Ann Counsel", "b"),
        ins(2, "Ann Counsel", "c"),
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
        "view: tracked                      # revisions as CriticMarkup, comments inline"
    );
    assert_eq!(
        lines[2],
        "track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it"
    );
    assert_eq!(
        lines[3],
        "revisions: 2                       # 1 insertion, 1 substitution (3 Word marks)"
    );
    assert_eq!(lines[4], "comments: 0");
    assert_eq!(lines[5], "authors:");
    assert_eq!(lines[6], "  document_owner: Jane Owner       # dc:creator");
    assert_eq!(
        lines[7],
        "  AC: Ann Counsel                  # 1 revision, 2026-10-01T09:00:00Z"
    );
    assert_eq!(
        lines[8],
        "  JD: John Doe                     # 1 revision, 2026-10-01T09:00:00Z"
    );
    assert_eq!(
        lines[9],
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
        lines[2],
        "track_changes: on                  # w:trackRevisions set; edits are tracked"
    );
    assert_eq!(lines[3], "revisions: 0");
    assert_eq!(lines[5], "authors:");
    assert_eq!(
        lines[6],
        "  document_owner: none             # no docProps/core.xml"
    );
}

#[test]
fn header_view_lines_for_hidden_comments_accept_and_reject() {
    let bytes = commented_docx(THREADED);
    assert_eq!(
        header_lines(&agent_with(&bytes, TrackChanges::All, false))[1],
        "view: tracked, comments hidden     # 2 threads open (3 comments); carrying paragraphs are marked"
    );
    let p = format!("<w:p>{}{}</w:p>", run("a "), ins(0, "Ann Counsel", "b"));
    assert_eq!(
        header_lines(&agent_with(&docx(&p), TrackChanges::Accept, true))[1],
        "view: accept-all                   # 1 revision by AC shown as accepted; file unchanged"
    );
    assert_eq!(
        header_lines(&agent_with(&docx(&p), TrackChanges::Reject, true))[1],
        "view: reject-all                   # 1 revision by AC shown as rejected; file unchanged"
    );
    assert_eq!(
        header_lines(&agent(&bytes))[4],
        "comments: 2 threads open           # 3 comments: c5 (+ reply c6), c11"
    );
}

#[test]
fn header_prints_a_day_range_for_an_author_with_several_timestamps() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        run("a "),
        ins_at(1, "Ann Counsel", "2026-10-01T09:00:00Z", "b"),
        run(" c "),
        ins_at(2, "Ann Counsel", "2026-10-03T14:05:00Z", "d")
    );
    let out = agent(&docx(&p));
    let lines = header_lines(&out);
    // lines[6] is the owner line, which every header prints.
    assert_eq!(
        lines[7],
        "  AC: Ann Counsel                  # 2 revisions, 2026-10-01..2026-10-03"
    );
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
    let header = out.splitn(3, "---\n").nth(1).unwrap();
    let expected = "\
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  \"#\": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
  table: TableGrid, all borders 0.5pt
headers:
  first: {id: header2, text: DRAFT}  # page 1 only (different first page)
  default: {id: header1, text: SIGNATURE PAGE, right}
footers:
  first: none                      # page 1 shows no page number
  default: {id: footer2, text: \"{PAGE}\"}
  even: {id: footer1, text: \"{PAGE}\", inactive}  # defined, but even/odd headers are off
";
    assert!(header.ends_with(expected), "header:\n{header}");
}

#[test]
fn header_page_line_for_a4_landscape_with_uneven_margins_and_a_gutter() {
    let sect = r#"<w:sectPr><w:pgSz w:w="16838" w:h="11906" w:orient="landscape"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="709" w:footer="709" w:gutter="720"/></w:sectPr>"#;
    let out = agent(&common::docx::docx_with_sect_pr(&para("x"), &[], sect));
    assert!(out.contains("\npage: A4 landscape, margins top 1in, right 1.25in, bottom 1in, left 1.25in, header/footer 0.49in, gutter 0.5in\n"), "{out}");
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
    let header = out.splitn(3, "---\n").nth(1).unwrap();
    assert!(header.ends_with("headers:\n  default: {id: header1, text: MAIN}\nsections:\n  2: {p2-p3, headers: {default: {id: header2, text: SCHEDULE A}}, columns: 2}\n"), "header:\n{header}");
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
        MarkdownOptions {
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
        "<!-- page 1 of 1 -->\n\n<!-- p1 -->\nOne\n\n<!-- t0 center 3x2, cells p2-p7 by row, header row repeats -->\n|Item|Due|\n|-|-|\n|Report|Day 10|\n|Call|Monthly|\n\n<!-- p9 -->\nNine\n"
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
        MarkdownOptions {
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
        ok(&["text", "received.docx"], dir.path()),
        tracked,
        "text is an alias of read"
    );
    assert!(
        tracked.starts_with("---\nsource: received.docx\nview: tracked "),
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
    assert!(dated.contains("{>>#0 @AC<<}"), "{dated}");
    let bad = jubarte(&["read", "received.docx", "-p", "p99"], dir.path());
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("p99 is past the last paragraph p20"));
    let both = jubarte(
        &["read", "received.docx", "--head", "2", "--tail", "2"],
        dir.path(),
    );
    assert!(!both.status.success());
}
