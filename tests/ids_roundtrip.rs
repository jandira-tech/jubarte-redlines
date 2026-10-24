// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The neurotic id round trip (plan Task 5b), both directions, nothing
//! sampled away:
//!
//! - Markdown → docx: every id the agent view prints parses back to exactly
//!   the element it labels (changes, comments, paragraphs, cells, stories).
//! - docx → Markdown: every change, comment and paragraph the document
//!   holds is printed with an id that parses back to it.
//!
//! The documents: a 51,200-paragraph body (every id width up to
//! `p51199`), tables (a 120×12 grid, merged and nested cells, twelve
//! tables), every story (three headers, three footers, footnotes,
//! endnotes, a style pair, a text box and its twin), ids that collide on
//! purpose, and 213 of Word's own redlines from the bench corpus. The
//! extractor below is one regex per printed form, written here and not
//! shared with the engine.

mod common;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::OnceLock;

use common::docx::{MC_NS, Part, docx_with_sect, esc, para};
use jubarte::ids::{ChangeId, parse_change_ids, parse_comment_id, short_paragraph};
use jubarte::markdown::{MarkdownOptions, docx_to_markdown, plain_anchor};
use regex::Regex;

const A: &str = r#"w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z""#;
const J: &str = r#"w:author="John Doe" w:date="2026-10-02T09:00:00Z""#;
const W14: &str = r#"xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
const W15: &str = r#"xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml""#;
const W_NS: &str = common::docx::W_NS;

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const FOOTNOTES_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml";
const FOOTNOTES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes";
const ENDNOTES_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml";
const ENDNOTES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
const EXTENDED_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml";
const EXTENDED_REL: &str =
    "http://schemas.microsoft.com/office/2011/relationships/commentsExtended";

/// The paragraph count of the long body: every width from `p0` to `p51199`.
const LONG: usize = 51_200;

// ---------------------------------------------------------------- builders

fn ins(id: &str, author: &str, text: &str) -> String {
    format!(
        r#"<w:ins w:id="{id}" {author}><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:ins>"#,
        esc(text)
    )
}

fn del(id: &str, author: &str, text: &str) -> String {
    format!(
        r#"<w:del w:id="{id}" {author}><w:r><w:delText xml:space="preserve">{}</w:delText></w:r></w:del>"#,
        esc(text)
    )
}

fn run(text: &str) -> String {
    format!(
        r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
        esc(text)
    )
}

fn p(inner: &str) -> String {
    format!("<w:p>{inner}</w:p>")
}

/// A comment's part entry: `w:id`, author, the `w14:paraId` its thread
/// links by, and its text.
fn comment(id: u32, author: &str, initials: &str, para_id: &str, text: &str) -> String {
    format!(
        r#"<w:comment w:id="{id}" w:author="{author}" w:initials="{initials}" w:date="2026-10-01T09:00:00Z"><w:p w14:paraId="{para_id}"><w:r><w:annotationRef/></w:r><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
    )
}

/// Body text wrapped in comment `ids`' ranges, followed by their references.
fn commented(ids: &[u32], text: &str) -> String {
    let starts: String = ids
        .iter()
        .map(|i| format!(r#"<w:commentRangeStart w:id="{i}"/>"#))
        .collect();
    let ends: String = ids
        .iter()
        .map(|i| {
            format!(r#"<w:commentRangeEnd w:id="{i}"/><w:r><w:commentReference w:id="{i}"/></w:r>"#)
        })
        .collect();
    format!("{starts}{}{ends}", run(text))
}

fn part<'a>(name: &'a str, content_type: &'a str, rel_type: &'a str, xml: &'a str) -> Part<'a> {
    Part {
        name,
        content_type,
        rel_type,
        xml,
    }
}

/// Comments `entries` and the `commentsExtended` threads `extended`.
fn comment_parts(entries: &str, extended: &str) -> (String, String) {
    (
        format!(r#"<w:comments xmlns:w="{W_NS}" {W14}>{entries}</w:comments>"#),
        format!(r#"<w15:commentsEx xmlns:w="{W_NS}" {W15}>{extended}</w15:commentsEx>"#),
    )
}

// --------------------------------------------------------------- documents

/// 51,200 paragraphs; revisions on every width's first and last number,
/// with `w:id`s out of order, a joined pair, and comments 0, 9, 10, 99 and
/// 51000 with a reply and a resolved thread.
fn long_body() -> &'static [u8] {
    static DOC: OnceLock<Vec<u8>> = OnceLock::new();
    DOC.get_or_init(|| {
        let revised: HashMap<usize, &str> = [
            (0, "51000"),
            (9, "9"),
            (10, "10000"),
            (99, "0"),
            (100, "99"),
            (999, "1000"),
            (1000, "999"),
            (9999, "100"),
            (10000, "10"),
            (51000, "51199"),
            (LONG - 1, "7"),
        ]
        .into_iter()
        .collect();
        let commented_at: HashMap<usize, Vec<u32>> = [
            (5, vec![0]),
            (9, vec![9, 10]),
            (99, vec![99]),
            (51000, vec![51000]),
        ]
        .into_iter()
        .collect();
        let mut body = String::with_capacity(LONG * 60);
        for i in 0..LONG {
            let mut inner = run(&format!("P{i} "));
            if let Some(w) = revised.get(&i) {
                inner.push_str(&ins(w, A, &format!("added{i}")));
            }
            if i == 7 {
                // Word Compare's order, the insertion then the deletion:
                // one substitution, `r3+4`.
                inner.push_str(&ins("4", A, "new"));
                inner.push_str(&del("3", A, "old"));
            }
            if let Some(ids) = commented_at.get(&i) {
                inner.push_str(&commented(ids, &format!("noted{i}")));
            }
            body.push_str(&p(&inner));
        }
        let (comments, extended) = comment_parts(
            &[
                comment(0, "Ann Counsel", "AC", "00000001", "Zero?"),
                comment(9, "Ann Counsel", "AC", "00000009", "Nine?"),
                comment(10, "John Doe", "JD", "0000000A", "Reply to nine."),
                comment(99, "John Doe", "JD", "00000063", "Settled."),
                comment(51000, "Ann Counsel", "AC", "0000C738", "Far away."),
            ]
            .concat(),
            r#"<w15:commentEx w15:paraId="00000001" w15:done="0"/><w15:commentEx w15:paraId="00000009" w15:done="0"/><w15:commentEx w15:paraId="0000000A" w15:paraIdParent="00000009" w15:done="0"/><w15:commentEx w15:paraId="00000063" w15:done="1"/><w15:commentEx w15:paraId="0000C738" w15:done="0"/>"#,
        );
        docx_with_sect(
            &body,
            &[
                part("word/comments.xml", COMMENTS_CT, COMMENTS_REL, &comments),
                part("word/commentsExtended.xml", EXTENDED_CT, EXTENDED_REL, &extended),
            ],
            "",
        )
    })
}

fn cell(inner: &str, props: &str) -> String {
    format!("<w:tc><w:tcPr>{props}</w:tcPr>{inner}</w:tc>")
}

fn grid(cols: usize) -> String {
    format!(
        "<w:tblGrid>{}</w:tblGrid>",
        r#"<w:gridCol w:w="700"/>"#.repeat(cols)
    )
}

/// A 120×12 grid (`t0.row119.cell11`), a cell with three paragraphs,
/// merged cells, a nested table, a table whose row, cell and text are
/// revised and commented, and enough tables to print `t11`.
fn tables() -> &'static [u8] {
    static DOC: OnceLock<Vec<u8>> = OnceLock::new();
    DOC.get_or_init(|| {
        let mut body = para("Before");
        // t0: 120 × 12, one paragraph per cell.
        body.push_str(&format!("<w:tbl><w:tblPr/>{}", grid(12)));
        for r in 0..120 {
            body.push_str("<w:tr>");
            for c in 0..12 {
                body.push_str(&cell(&para(&format!("g{r}x{c}")), ""));
            }
            body.push_str("</w:tr>");
        }
        body.push_str("</w:tbl>");
        // t1: a three-paragraph cell, a horizontal and a vertical merge and
        // a nested table.
        let nested = format!(
            "<w:tbl><w:tblPr/>{}<w:tr>{}</w:tr></w:tbl>",
            grid(1),
            cell(&para("inner"), "")
        );
        body.push_str(&format!(
            "<w:tbl><w:tblPr/>{}<w:tr>{}{}</w:tr><w:tr>{}{}{}</w:tr><w:tr>{}{}{}</w:tr></w:tbl>",
            grid(3),
            cell(
                &format!("{}{}{}", para("three a"), para("three b"), para("three c")),
                r#"<w:gridSpan w:val="2"/>"#
            ),
            cell(&para("tall"), r#"<w:vMerge w:val="restart"/>"#),
            cell(&para("left"), ""),
            cell(&format!("{}{nested}{}", para("outer"), para("after")), ""),
            cell(&p(""), "<w:vMerge/>"),
            cell(&para("x"), ""),
            cell(&para("y"), ""),
            cell(&para("z"), ""),
        ));
        // t2: a revised row and cell, revised and commented text.
        body.push_str(&format!(
            r#"<w:tbl><w:tblPr/>{}<w:tr>{}{}</w:tr><w:tr><w:trPr><w:ins w:id="5" {A}/></w:trPr>{}{}</w:tr></w:tbl>"#,
            grid(2),
            cell(&para("plain"), ""),
            cell(&p(&commented(&[3], "noted cell")), ""),
            cell(&p(&ins("6", A, "new row")), ""),
            cell(&p(&ins("7", J, "by JD")), r#"<w:cellIns w:id="8" w:author="John Doe" w:date="2026-10-02T09:00:00Z"/>"#),
        ));
        // t3 … t11, each one cell, between paragraphs.
        for t in 3..12 {
            body.push_str(&para(&format!("between {t}")));
            body.push_str(&format!(
                "<w:tbl><w:tblPr/>{}<w:tr>{}</w:tr></w:tbl>",
                grid(1),
                cell(&para(&format!("only {t}")), "")
            ));
        }
        body.push_str(&para("After"));
        let (comments, extended) = comment_parts(
            &comment(3, "Ann Counsel", "AC", "00000003", "In a cell."),
            r#"<w15:commentEx w15:paraId="00000003" w15:done="0"/>"#,
        );
        docx_with_sect(
            &body,
            &[
                part("word/comments.xml", COMMENTS_CT, COMMENTS_REL, &comments),
                part("word/commentsExtended.xml", EXTENDED_CT, EXTENDED_REL, &extended),
            ],
            "",
        )
    })
}

fn story_xml(root: &str, inner: &str) -> String {
    format!(r#"<w:{root} xmlns:w="{W_NS}">{inner}</w:{root}>"#)
}

/// Three headers and footers (default, first, even), footnotes and
/// endnotes with ids up to 51000, a style's `pPrChange`+`rPrChange` pair
/// on one `w:id`, a text box and its Fallback twin, and the collisions:
/// body `r5` beside `footnotes.r5`, comment `c5` beside `r5` and `p5`, a
/// header part named `hdr.xml` and one outside the stem rule
/// (`Header_1.xml`).
fn stories() -> &'static [u8] {
    static DOC: OnceLock<Vec<u8>> = OnceLock::new();
    DOC.get_or_init(|| {
        let hdr = |id: &str, text: &str| story_xml("hdr", &p(&ins(id, A, text)));
        let ftr = |id: &str, text: &str| story_xml("ftr", &p(&ins(id, J, text)));
        let header1 = hdr("1", "Default head");
        let header2 = hdr("2", "First head");
        let header3 = hdr("1", "Even head");
        let footer1 = ftr("1", "Default foot");
        let footer2 = ftr("4", "First foot");
        let footer3 = ftr("9", "Even foot");
        let short_stem = hdr("1", "Short stem");
        let outside = hdr("1", "Outside the rule");
        let footnotes = story_xml(
            "footnotes",
            &format!(
                r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p>{}{}</w:p></w:footnote><w:footnote w:id="51000"><w:p>{}{}</w:p></w:footnote>"#,
                run("Note one"),
                ins("5", A, " five"),
                run("Far note"),
                del("51000", J, " gone")
            ),
        );
        let endnotes = story_xml(
            "endnotes",
            &format!(
                r#"<w:endnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:endnote><w:endnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:endnote><w:endnote w:id="2"><w:p>{}{}</w:p></w:endnote>"#,
                run("End note"),
                ins("12", A, " twelve")
            ),
        );
        let styles = story_xml(
            "styles",
            &format!(
                r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="30" {A}><w:pPr/></w:pPrChange></w:pPr><w:rPr><w:i/><w:rPrChange w:id="30" {A}><w:rPr/></w:rPrChange></w:rPr></w:style>"#
            ),
        );
        let (comments, extended) = comment_parts(
            &comment(5, "Ann Counsel", "AC", "00000005", "Five."),
            r#"<w15:commentEx w15:paraId="00000005" w15:done="0"/>"#,
        );
        // Word's own shape: a DrawingML text box, its VML twin in Fallback.
        let boxed = |id: &str| {
            format!(
                r#"<w:txbxContent><w:p>{}</w:p></w:txbxContent>"#,
                ins(id, A, "Boxed")
            )
        };
        let drawing = |content: String| {
            format!(
                r#"<w:drawing><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:txbx>{content}</wps:txbx></wps:wsp></w:drawing>"#
            )
        };
        let pict = |content: String| {
            format!(
                r#"<w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox>{content}</v:textbox></v:shape></w:pict>"#
            )
        };
        let mut body = String::new();
        for i in 0..8 {
            let inner = match i {
                1 => format!(
                    r#"{}<w:r><w:footnoteReference w:id="1"/></w:r>{}<w:r><w:footnoteReference w:id="51000"/></w:r><w:r><w:endnoteReference w:id="2"/></w:r>"#,
                    run("Notes"),
                    run(" and ")
                ),
                3 => format!(
                    r#"{}<w:r><mc:AlternateContent xmlns:mc="{MC_NS}"><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r>"#,
                    run("Lead"),
                    drawing(boxed("40")),
                    pict(boxed("41"))
                ),
                5 => format!(
                    "{}{}{}",
                    run("Five "),
                    ins("5", A, "body five"),
                    commented(&[5], " commented")
                ),
                6 => format!(r#"<w:pPr><w:pStyle w:val="Quote"/></w:pPr>{}"#, run("Quoted")),
                _ => run(&format!("Para {i}")),
            };
            body.push_str(&p(&inner));
        }
        // Part i gets relationship id rIdX{i}: headers 0-2, footers 3-5,
        // the extra headers 6-7.
        let sect = r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:headerReference w:type="first" r:id="rIdX1"/><w:headerReference w:type="even" r:id="rIdX2"/><w:footerReference w:type="default" r:id="rIdX3"/><w:footerReference w:type="first" r:id="rIdX4"/><w:footerReference w:type="even" r:id="rIdX5"/><w:titlePg/>"#;
        docx_with_sect(
            &format!(
                r#"{body}<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX6"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:pPr></w:p>{}<w:p><w:pPr><w:sectPr><w:headerReference w:type="default" r:id="rIdX7"/><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:pPr></w:p>"#,
                para("Second section")
            ),
            &[
                part("word/header1.xml", HEADER_CT, HEADER_REL, &header1),
                part("word/header2.xml", HEADER_CT, HEADER_REL, &header2),
                part("word/header3.xml", HEADER_CT, HEADER_REL, &header3),
                part("word/footer1.xml", FOOTER_CT, FOOTER_REL, &footer1),
                part("word/footer2.xml", FOOTER_CT, FOOTER_REL, &footer2),
                part("word/footer3.xml", FOOTER_CT, FOOTER_REL, &footer3),
                part("word/hdr.xml", HEADER_CT, HEADER_REL, &short_stem),
                part("word/Header_1.xml", HEADER_CT, HEADER_REL, &outside),
                part("word/footnotes.xml", FOOTNOTES_CT, FOOTNOTES_REL, &footnotes),
                part("word/endnotes.xml", ENDNOTES_CT, ENDNOTES_REL, &endnotes),
                part("word/styles.xml", STYLES_CT, STYLES_REL, &styles),
                part("word/comments.xml", COMMENTS_CT, COMMENTS_REL, &comments),
                part("word/commentsExtended.xml", EXTENDED_CT, EXTENDED_REL, &extended),
            ],
            sect,
        )
    })
}

// --------------------------------------------------------------- the view

fn view(docx: &[u8]) -> String {
    docx_to_markdown(
        docx,
        &MarkdownOptions {
            ids: true,
            comments: true,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .expect("the view reads the document")
    .markdown
}

/// What the view printed, one regex per form.
#[derive(Default, Debug)]
struct Printed {
    /// Change tokens: tags in notes and on id and table lines.
    changes: BTreeSet<String>,
    /// Comment ids in note heads (`c5`), with how often each opens a note.
    comment_notes: BTreeMap<u32, usize>,
    /// Comment ids named anywhere else: `re c5`, id lines, the legend.
    comment_mentions: BTreeSet<u32>,
    /// Body paragraphs by id line: `(first, last, text after the line)`;
    /// text is `None` for a run of empties.
    paragraphs: Vec<(usize, usize, Option<String>)>,
    /// Table lines: `(t, rows, grid columns, first paragraph, last
    /// paragraph)`.
    tables: Vec<(usize, usize, usize, usize, usize)>,
    /// Story ids on the `headers:`/`footers:` lines.
    stories: BTreeSet<String>,
}

struct Patterns {
    note: Regex,
    tag: Regex,
    comment_head: Regex,
    reply: Regex,
    id_line: Regex,
    line_tag: Regex,
    comment_word: Regex,
    table_line: Regex,
    cells_by_row: Regex,
    cells_rows: Regex,
    story: Regex,
    definition: Regex,
    drawing_definition: Regex,
    backing_comment: Regex,
}

fn patterns() -> &'static Patterns {
    static P: OnceLock<Patterns> = OnceLock::new();
    P.get_or_init(|| Patterns {
        note: Regex::new(r"\{>>(.*?)<<\}").unwrap(),
        // `r12 @AC`, `header1.r1+2 @AC`, `Header_1:rev:4 @AC`, with a date.
        tag: Regex::new(r"^(\S+) @[A-Za-z0-9?]+(?: \S+)?$").unwrap(),
        comment_head: Regex::new(
            r"^c(\d+)(?: @[A-Za-z0-9?]+)?(?: \S+)?(?: re c(\d+)| resolved)?: ",
        )
        .unwrap(),
        reply: Regex::new(r" re c(\d+)").unwrap(),
        id_line: Regex::new(r"^ *<!-- p(\d+)(?:-p(\d+))?(.*) -->$").unwrap(),
        line_tag: Regex::new(r"(?:^|[ ,;\[])([^\s,;\[]+) @[A-Za-z0-9?]+").unwrap(),
        comment_word: Regex::new(r"(?:^|[\s,;(])c(\d+)\b").unwrap(),
        table_line: Regex::new(r"^<!-- t(\d+)(?: center| right)? (\d+)x(\d+), (.*) -->$").unwrap(),
        cells_by_row: Regex::new(r"^cells p(\d+)-p(\d+) by row").unwrap(),
        cells_rows: Regex::new(r"t\d+\.row\d+ p(\d+)(?:-p(\d+))?").unwrap(),
        // A long story id holds colons, so the header quotes it.
        story: Regex::new(r#"\{id: "?([^,}\s"]+)"?"#).unwrap(),
        definition: Regex::new(r"^(?:\{(?:--|\+\+))?\[\^\d+\]: ").unwrap(),
        // Drawing payloads: `[dN]: drawing:…` definitions and the
        // Google Docs form's `[imageN]: <data:…>` ones.
        drawing_definition: Regex::new(r"^\[(?:d|image)\d+\]: ").unwrap(),
        // A payload that backs a readable form: `<!-- dN drawing:… -->`,
        // `<!-- imageN drawing:… -->`.
        backing_comment: Regex::new(r"^ *<!-- \S+ drawing:").unwrap(),
    })
}

/// Splits the view into its YAML header and its body.
/// A note definition's id line: `<!-- footnotes.p0 -->`, `<!-- endnotes.p2-p3 -->`.
fn is_note_id_line(line: &str) -> bool {
    line.strip_prefix("<!-- ")
        .and_then(|rest| {
            rest.strip_prefix("footnotes.p")
                .or_else(|| rest.strip_prefix("endnotes.p"))
        })
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

fn split_view(view: &str) -> (&str, &str) {
    let mut parts = view.splitn(3, "---\n");
    let _ = parts.next();
    let header = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or(view);
    (header, body)
}

fn printed(view: &str) -> Printed {
    let pat = patterns();
    let (header, body) = split_view(view);
    let mut out = Printed::default();
    let notes = pat
        .note
        .captures_iter(body)
        .map(|n| (true, n.get(1).unwrap().as_str()))
        .chain(
            pat.note
                .captures_iter(header)
                .map(|n| (false, n.get(1).unwrap().as_str())),
        );
    for (in_body, note) in notes {
        if let Some(head) = pat.comment_head.captures(note) {
            let id: u32 = head[1].parse().unwrap();
            if in_body {
                *out.comment_notes.entry(id).or_default() += 1;
            } else {
                out.comment_mentions.insert(id);
            }
            if let Some(root) = head.get(2) {
                out.comment_mentions.insert(root.as_str().parse().unwrap());
            }
            continue;
        }
        for one in note.split("; ") {
            match pat.tag.captures(one) {
                Some(tag) => {
                    out.changes.insert(tag[1].to_string());
                }
                None => {
                    out.changes.insert(format!("UNREADABLE NOTE {note}"));
                }
            }
        }
    }
    // Header tags outside notes: `page: …; fmt r2 @AC`, `changes:
    // {Normal: styles.r9 @AC}`, a header entry's `rev:`/`fmt:`, the
    // `unplaced: [r3 @AC in p0]` safety net.
    for line in header.lines() {
        let bare = pat.note.replace_all(line, "");
        for tag in pat.line_tag.captures_iter(&bare) {
            out.changes.insert(tag[1].trim_matches('"').to_string());
        }
    }
    // The paragraph whose text the next lines are; `None` inside a table.
    let mut current: Option<usize> = None;
    for line in body.lines() {
        // A note definition's own changes: `[^1]: text <!-- fmt footnotes.r7 @AC -->`,
        // also when the definition is wrapped: `{--[^1]: text--}{>>r18 @C<<} <!-- … -->`.
        if line.contains("[^")
            && !line.starts_with("<!-- ")
            && line.ends_with(" -->")
            && let Some(at) = line.rfind("<!-- ")
        {
            for tag in pat.line_tag.captures_iter(&line[at..]) {
                out.changes.insert(tag[1].to_string());
            }
        }
        if let Some(caps) = pat.id_line.captures(line) {
            let first: usize = caps[1].parse().unwrap();
            let last: usize = caps.get(2).map_or(first, |m| m.as_str().parse().unwrap());
            let rest = &caps[3];
            for tag in pat.line_tag.captures_iter(rest) {
                out.changes.insert(tag[1].to_string());
            }
            for c in pat.comment_word.captures_iter(rest) {
                out.comment_mentions.insert(c[1].parse().unwrap());
            }
            let text = (!rest.contains(" empty")).then(String::new);
            current = text.is_some().then_some(out.paragraphs.len());
            out.paragraphs.push((first, last, text));
            continue;
        }
        if let Some(caps) = pat.table_line.captures(line) {
            let t: usize = caps[1].parse().unwrap();
            let rows: usize = caps[2].parse().unwrap();
            let cols: usize = caps[3].parse().unwrap();
            let clauses = &caps[4];
            let (a, b) = if let Some(c) = pat.cells_by_row.captures(clauses) {
                (c[1].parse().unwrap(), c[2].parse().unwrap())
            } else {
                let numbers: Vec<usize> = pat
                    .cells_rows
                    .captures_iter(clauses)
                    .flat_map(|c| {
                        let a: usize = c[1].parse().unwrap();
                        let b = c.get(2).map_or(a, |m| m.as_str().parse().unwrap());
                        [a, b]
                    })
                    .collect();
                (
                    numbers.iter().copied().min().unwrap_or(usize::MAX),
                    numbers.iter().copied().max().unwrap_or(0),
                )
            };
            for tag in pat.line_tag.captures_iter(clauses) {
                out.changes.insert(tag[1].to_string());
            }
            for c in pat.comment_word.captures_iter(clauses) {
                out.comment_mentions.insert(c[1].parse().unwrap());
            }
            out.tables.push((t, rows, cols, a, b));
            current = None;
            continue;
        }
        // A note's definition (`[^1]: …`, or wrapped in the change that
        // deleted its reference: `{--[^1]: …--}`) closes the paragraph's text,
        // and so does the definition's own id line (`<!-- footnotes.p0 -->`).
        // So does the drawings appendix: its `[dN]:`/`[imageN]:` lines and
        // `<!-- … drawing:… -->` backings are payload, not text.
        if pat.definition.is_match(line)
            || is_note_id_line(line)
            || pat.drawing_definition.is_match(line)
            || pat.backing_comment.is_match(line)
        {
            current = None;
            continue;
        }
        if let Some((_, _, Some(text))) = current.and_then(|i| out.paragraphs.get_mut(i)) {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(line);
        }
    }
    for line in header.lines() {
        if let Some(caps) = pat.story.captures(line) {
            out.stories.insert(caps[1].to_string());
        }
        if line.starts_with("comments:") {
            for c in pat.comment_word.captures_iter(line) {
                out.comment_mentions.insert(c[1].parse().unwrap());
            }
        }
    }
    for c in pat.reply.captures_iter(body) {
        out.comment_mentions.insert(c[1].parse().unwrap());
    }
    out
}

/// Letters and digits only: what both the view and `inspect` agree on once
/// the view's Markdown (links, note references, table pipes) is gone.
fn letters(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// The paragraph text a view block stands for, without the Markdown
/// `inspect` does not print: note references `[^1]` and link targets.
fn view_text(block: &str) -> String {
    let refs = Regex::new(r"\[\^\d+\]").unwrap();
    // `![alt](…)` and the reference form `![alt][dN]` the drawings use.
    let images = Regex::new(r"!\[[^\]]*\](?:\([^)]*\)|\[[^\]]*\])").unwrap();
    let links = Regex::new(r"\]\([^)]*\)").unwrap();
    // `<mailto:a@b>` shows the address the document holds.
    let text = plain_anchor(block).replace("<mailto:", "<");
    let text = refs.replace_all(&text, "");
    let text = images.replace_all(&text, "");
    let tags = Regex::new(r#"</?[a-z]+(?:\s[^>]*)?>"#).unwrap();
    let text = tags.replace_all(&text, "");
    // A field prints `{CODE}result{/}`: only its result is text.
    let fields = Regex::new(r#"\{[A-Z]+(?:\s(?:[^{}"]|"[^"]*")*)?\}|\{/\}"#).unwrap();
    let text = fields.replace_all(&text, "");
    letters(&links.replace_all(&text, "]"))
}

// ---------------------------------------------------------------- checks

/// Every check, both directions, for one document. Failures are collected,
/// so one run names every defect.
fn round_trip(name: &str, docx: &[u8], fails: &mut Vec<String>) {
    let mut fail = |what: String| fails.push(format!("{name}: {what}"));
    let view = view(docx);
    let printed = printed(&view);
    let listed = jubarte::changes::list_changes(docx).expect("changes lists the document");
    let by_id: HashMap<&str, &jubarte::changes::Change> =
        listed.iter().map(|c| (c.id.as_str(), c)).collect();

    // Markdown → docx: changes.
    let mut printed_longs: BTreeSet<String> = BTreeSet::new();
    for token in &printed.changes {
        match parse_change_ids(token) {
            Err(e) => fail(format!("printed change {token:?} does not parse: {e}")),
            Ok(ids) => {
                for id in ids {
                    let long = id.short();
                    match by_id.get(long.as_str()) {
                        None => fail(format!("printed {token} names {long}, not a listed change")),
                        Some(change) if change.r#ref != id.short() => fail(format!(
                            "printed {token}: {long} lists as {} but prints as {}",
                            change.r#ref,
                            id.short()
                        )),
                        Some(_) => {}
                    }
                    printed_longs.insert(long);
                }
            }
        }
    }
    // docx → Markdown: changes. A change inside another (a deleted cell's text in a deleted row) is
    // shown by its container's tag: resolving the container takes it along.
    let shown = |change: &jubarte::changes::Change| {
        let mut at = Some(change);
        let mut hops = 0;
        while let Some(c) = at {
            if printed_longs.contains(&c.id) {
                return true;
            }
            at = c.inside.as_deref().and_then(|i| by_id.get(i).copied());
            hops += 1;
            if hops > listed.len() {
                return false;
            }
        }
        false
    };
    for change in &listed {
        if shown(change) {
            continue;
        }
        fail(format!(
            "listed {} ({:?} {:?}) is printed nowhere",
            change.id, change.kind, change.text
        ));
    }
    // Every listed id is a short id that reads back as that change.
    for change in &listed {
        let Some(id) = jubarte::ids::change_id(&change.id) else {
            fail(format!("listed {} does not read back", change.id));
            continue;
        };
        if id.short() != change.id {
            fail(format!("listed {} reads back as {}", change.id, id.short()));
        }
    }

    // Comments, both directions.
    let comments = jubarte::comments::list_comments(docx).unwrap_or_default();
    let known: BTreeSet<u32> = comments.iter().map(|c| c.id).collect();
    for c in &comments {
        if c.r#ref != format!("c{}", c.id) {
            fail(format!("comment {} lists as {}", c.id, c.r#ref));
        }
        if parse_comment_id(&c.r#ref) != Ok(c.id) {
            fail(format!("comment ref {} does not read back", c.r#ref));
        }
        let notes = printed.comment_notes.get(&c.id).copied().unwrap_or(0);
        if notes > 1 {
            fail(format!("c{} opens {notes} notes", c.id));
        }
        if notes == 0 && !printed.comment_mentions.contains(&c.id) {
            fail(format!("c{} is printed nowhere", c.id));
        }
    }
    for id in printed
        .comment_notes
        .keys()
        .chain(&printed.comment_mentions)
    {
        if !known.contains(id) {
            fail(format!("printed c{id} names no listed comment"));
        }
    }

    // Paragraphs, both directions.
    let paragraphs = jubarte::inspect::paragraphs(docx).expect("inspect reads the body");
    let mut covered = vec![false; paragraphs.len()];
    let mut tokens: Vec<String> = Vec::new();
    for &(a, b, _) in &printed.paragraphs {
        tokens.push(format!("p{a}"));
        tokens.push(format!("p{b}"));
    }
    let token_refs: Vec<&str> = tokens.iter().map(String::as_str).collect();
    let longs = jubarte::edit::paragraph_ids(docx, &token_refs).expect("edit opens it");
    for (i, &(a, b, ref text)) in printed.paragraphs.iter().enumerate() {
        for (token, n, long) in [
            (&tokens[2 * i], a, &longs[2 * i]),
            (&tokens[2 * i + 1], b, &longs[2 * i + 1]),
        ] {
            if long.as_deref() != Ok(format!("p{n}").as_str()) {
                fail(format!("{token} resolves to {long:?}"));
            }
        }
        if b >= paragraphs.len() || a > b {
            fail(format!(
                "id line p{a}-p{b} is past the {} paragraphs",
                paragraphs.len()
            ));
            continue;
        }
        covered[a..=b].iter_mut().for_each(|c| *c = true);
        if let Some(text) = text
            && a == b
        {
            let want = letters(&paragraphs[a].text);
            let got = view_text(text);
            // A text box prints as blocks of its own after its host
            // paragraph, under the host's id line: only the text before
            // them is the host's, and none of it when the host is empty.
            let boxed = paragraphs[a].limitations.iter().any(|l| {
                matches!(
                    l.as_str(),
                    "text_box_omitted" | "alternate_content" | "drawing"
                )
            });
            let host = text.split("\n\n").next().unwrap_or(text);
            let boxed_ok =
                boxed && (want == view_text(host) || want.is_empty() || got.starts_with(&want));
            if want != got && !boxed_ok {
                fail(format!(
                    "p{a} prints {got:?} but inspect has {want:?} (view {text:?})"
                ));
            }
        }
    }
    for &(t, rows, cols, a, b) in &printed.tables {
        let cells = table_cells(docx, t, rows, cols);
        // Every paragraph of every cell, in the settled spelling: the cell
        // is its first paragraph (`t0.row1.cell2`), the others `.pK`.
        let mut want = Vec::new();
        let mut settled = Vec::new();
        for (r, c, longs) in &cells {
            for (k, long) in longs.iter().enumerate() {
                want.push(long.clone());
                settled.push(match k {
                    0 => format!("t{t}.row{r}.cell{c}"),
                    k => format!("t{t}.row{r}.cell{c}.p{k}"),
                });
            }
        }
        let refs: Vec<&str> = settled.iter().map(String::as_str).collect();
        let back = jubarte::edit::paragraph_ids(docx, &refs).unwrap();
        for ((token, long), back) in settled.iter().zip(&want).zip(&back) {
            if back.as_ref() != Ok(long) {
                fail(format!("{token} resolves to {back:?}, not {long}"));
            }
        }
        let range: Vec<String> = (a..=b).map(|n| format!("p{n}")).collect();
        if want != range {
            fail(format!(
                "t{t}'s cells hold {} paragraphs ({:?} … {:?}), the table line says p{a}-p{b}",
                want.len(),
                want.first(),
                want.last()
            ));
        }
        if a <= b && b < covered.len() {
            covered[a..=b].iter_mut().for_each(|c| *c = true);
        }
        let row_count = cells.iter().map(|(r, _, _)| *r).max().map_or(0, |r| r + 1);
        if row_count != rows {
            fail(format!(
                "t{t} prints {rows} rows, its cells sit in {row_count}"
            ));
        }
    }
    if let Some(n) = covered.iter().position(|c| !c) {
        let missing = covered.iter().filter(|c| !**c).count();
        fail(format!(
            "{missing} paragraphs reachable by no id line, first p{n}"
        ));
    }
    // Every paragraph's short id (body and stories) reads back as its long
    // id.
    let stories = jubarte::inspect::stories(docx).expect("inspect lists the stories");
    let all: Vec<&String> = paragraphs
        .iter()
        .chain(stories.iter().flat_map(|s| &s.paragraphs))
        .map(|p| &p.id)
        .collect();
    let shorts: Vec<String> = all.iter().map(|id| short_paragraph(id)).collect();
    let refs: Vec<&str> = shorts.iter().map(String::as_str).collect();
    let back = jubarte::edit::paragraph_ids(docx, &refs).unwrap();
    for ((long, short), back) in all.iter().zip(&shorts).zip(&back) {
        if back.as_ref() != Ok(*long) {
            fail(format!("{long} → {short} reads back as {back:?}"));
        }
    }

    // Stories: every id the header prints is a story, and reads back.
    // A header's paragraph prints as `header2` (its first) or `header2.p1`.
    for id in &printed.stories {
        let back = jubarte::edit::paragraph_ids(docx, &[id.as_str()]).unwrap();
        let found = back[0].as_ref().ok().and_then(|printed| {
            let (stem, _) = jubarte::ids::paragraph_place(printed)?;
            stories
                .iter()
                .find(|s| s.id == stem)
                .and_then(|s| s.paragraphs.iter().find(|p| &p.id == printed))
        });
        if found.is_none() || back[0].as_ref() != Ok(id) {
            fail(format!(
                "headers line names {id}, which resolves to {:?}",
                back[0]
            ));
        }
    }
}

/// Every cell of table `t` with its paragraphs' long ids, as `edit -p`
/// resolves `tT.rowR.cellC.pK`, row by row. Probes one row and one cell
/// past what the table line prints, so a row or cell it hides shows up.
fn table_cells(
    docx: &[u8],
    t: usize,
    rows: usize,
    cols: usize,
) -> Vec<(usize, usize, Vec<String>)> {
    let resolve = |tokens: &[String]| {
        let refs: Vec<&str> = tokens.iter().map(String::as_str).collect();
        jubarte::edit::paragraph_ids(docx, &refs).unwrap()
    };
    let probe: Vec<(usize, usize)> = (0..=rows)
        .flat_map(|r| (0..=cols).map(move |c| (r, c)))
        .collect();
    let firsts = resolve(
        &probe
            .iter()
            .map(|(r, c)| format!("t{t}.row{r}.cell{c}"))
            .collect::<Vec<_>>(),
    );
    let mut found: Vec<(usize, usize, Vec<String>)> = probe
        .into_iter()
        .zip(firsts)
        .filter_map(|((r, c), first)| Some((r, c, vec![first.ok()?])))
        .collect();
    // Each cell's further paragraphs, one index at a time for the cells
    // that still have one.
    let mut alive: Vec<usize> = (0..found.len()).collect();
    let mut k = 1;
    while !alive.is_empty() {
        let tokens: Vec<String> = alive
            .iter()
            .map(|&i| format!("t{t}.row{}.cell{}.p{k}", found[i].0, found[i].1))
            .collect();
        let next = resolve(&tokens);
        let mut still = Vec::new();
        for (&i, long) in alive.iter().zip(next) {
            if let Ok(long) = long {
                found[i].2.push(long);
                still.push(i);
            }
        }
        alive = still;
        k += 1;
    }
    found
}

/// The change ids before and after `jubarte accept --id TOKEN`: exactly the
/// token's changes go.
fn accept_by_cli(name: &str, docx: &[u8], token: &str, fails: &mut Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("in.docx");
    let out = dir.path().join("out.docx");
    std::fs::write(&source, docx).unwrap();
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["accept"])
        .arg(&source)
        .arg("-o")
        .arg(&out)
        .args(["--id", token])
        .output()
        .unwrap();
    if !run.status.success() {
        fails.push(format!(
            "{name}: accept --id {token} failed: {}",
            String::from_utf8_lossy(&run.stderr)
        ));
        return;
    }
    let ids = |bytes: &[u8]| -> BTreeSet<String> {
        jubarte::changes::list_changes(bytes)
            .unwrap()
            .into_iter()
            .map(|c| c.id)
            .collect()
    };
    let before = ids(docx);
    let after = ids(&std::fs::read(&out).unwrap());
    let gone: BTreeSet<String> = before.difference(&after).cloned().collect();
    let named: BTreeSet<String> = parse_change_ids(token)
        .unwrap()
        .iter()
        .map(ChangeId::short)
        .collect();
    if gone != named || !after.is_subset(&before) {
        fails.push(format!(
            "{name}: accept --id {token} removed {gone:?}, named {named:?}"
        ));
    }
}

fn report(fails: &[String]) {
    assert!(
        fails.is_empty(),
        "{} round-trip failures:\n{}",
        fails.len(),
        fails.join("\n")
    );
}

// ------------------------------------------------------------------ tests

#[test]
fn the_long_body_round_trips_every_id_width() {
    let docx = long_body();
    let view = view(docx);
    for id in [
        "<!-- p0 -->",
        "<!-- p9 -->",
        "<!-- p10 -->",
        "<!-- p99 -->",
        "<!-- p100 -->",
        "<!-- p999 -->",
        "<!-- p1000 -->",
        "<!-- p9999 -->",
        "<!-- p10000 -->",
        "<!-- p51000 -->",
        "<!-- p51199 -->",
    ] {
        assert!(view.contains(id), "{id}");
    }
    for tag in [
        "{>>r51000 @AC<<}",
        "{>>r9 @AC<<}",
        "{>>r10000 @AC<<}",
        "{>>r0 @AC<<}",
        "{>>r51199 @AC<<}",
        "{~~old~>new~~}{>>r3+4 @AC<<}",
        "{>>c51000 @AC: Far away.<<}",
        "{>>c10 @JD re c9: Reply to nine.<<}",
        "{>>c99 @JD resolved: Settled.<<}",
    ] {
        assert!(view.contains(tag), "{tag}");
    }
    let mut fails = Vec::new();
    round_trip("long", docx, &mut fails);
    for token in ["r51000", "r0", "r3+4", "r7"] {
        accept_by_cli("long", docx, token, &mut fails);
    }
    report(&fails);
}

#[test]
fn tables_round_trip_every_row_cell_and_paragraph() {
    let docx = tables();
    let view = view(docx);
    assert!(
        view.contains("<!-- t0 120x12, cells p1-p1440 by row"),
        "{view}"
    );
    assert!(view.contains("<!-- t11 "), "{view}");
    let cells = table_cells(docx, 0, 120, 12);
    assert_eq!(cells.len(), 1440);
    assert_eq!(cells[1439].0, 119);
    assert_eq!(cells[1439].1, 11);
    let mut fails = Vec::new();
    round_trip("tables", docx, &mut fails);
    for token in ["r5", "r6+8", "r7"] {
        accept_by_cli("tables", docx, token, &mut fails);
    }
    report(&fails);
}

#[test]
fn every_story_and_every_collision_round_trips() {
    let docx = stories();
    let view = view(docx);
    for printed in [
        "header1.r1 @AC",
        "header2.r2 @AC",
        "header3.r1 @AC",
        "footer1.r1 @JD",
        "footer2.r4 @JD",
        "footer3.r9 @JD",
        // Stems outside `header N` / `footer N` keep the long id (spec §2.2).
        "hdr:rev:1 @AC",
        "Header_1:rev:1 @AC",
        "footnotes.r5 @AC",
        "footnotes.r51000 @JD",
        "endnotes.r12 @AC",
        "{++body five++}{>>r5 @AC<<}",
        "{>>c5 @AC: Five.<<}",
        "<!-- p5 -->",
        "{++Boxed++}{>>r40 @AC<<}",
    ] {
        assert!(view.contains(printed), "{printed}:\n{view}");
    }
    // The view never renders a Fallback, so the fold shows in the listing:
    // r41 joins r40 unlisted, and nothing is left unplaced.
    let listed: Vec<String> = jubarte::changes::list_changes(docx)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert!(listed.iter().any(|id| id == "r40"), "{listed:?}");
    assert!(
        !listed.iter().any(|id| id == "r41"),
        "the Fallback twin folds into r40: {listed:?}"
    );
    assert!(!view.contains("unplaced:"), "{view}");
    let mut fails = Vec::new();
    round_trip("stories", docx, &mut fails);
    for token in [
        "r5",
        "footnotes.r5",
        "header1.r1",
        "header3.r1",
        "hdr:rev:1",
        "Header_1:rev:1",
        "footnotes.r51000",
        "endnotes.r12",
        "r40",
        "styles.r30",
    ] {
        accept_by_cli("stories", docx, token, &mut fails);
    }
    report(&fails);
}

/// Word's own redlines: the bench's 100 rejected-tracking pairs, its 25
/// commented, tracked documents and its 88 tracked ones without comments.
#[test]
fn word_redlines_round_trip() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/neurotic_docx_bench");
    let mut files = Vec::new();
    for dir in [
        "grok_run/wr0928/rejected_tracking/docx",
        "corpus/word/with_comments_tracking/docx",
        "corpus/word/tracking_without_comments/docx",
    ] {
        let mut here: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "docx"))
            .collect();
        here.sort();
        files.extend(here);
    }
    assert_eq!(files.len(), 213, "the corpus is all here");
    let mut fails = Vec::new();
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let docx = std::fs::read(file).unwrap();
        round_trip(&name, &docx, &mut fails);
        let listed = jubarte::changes::list_changes(&docx).unwrap();
        if let Some(change) = listed.first() {
            accept_by_cli(&name, &docx, &change.r#ref, &mut fails);
        }
    }
    report(&fails);
}

/// codex 4244120691: a long id reads back only when its story is the
/// document's and its index a number, as `edit -p` reads it.
#[test]
fn a_long_id_reads_back_only_when_its_story_and_index_do() {
    let back = jubarte::edit::paragraph_ids(
        stories(),
        &["missing:p:0", "body:p:x", "Header_1:p:0", "p0"],
    )
    .unwrap();
    assert!(
        back[0].as_ref().is_err_and(|e| e.contains("unknown story")),
        "{back:?}"
    );
    assert!(
        back[1].as_ref().is_err_and(|e| e.contains("-p takes")),
        "{back:?}"
    );
    assert_eq!(back[2], Ok("Header_1:p:0".to_string()));
    assert_eq!(back[3], Ok("p0".to_string()));
}

/// codex 4242942149: a header part whose stem a shell would expand
/// (`hdr!old.xml`) gets a story id that is safe unquoted, and that id
/// names its change and paragraph back.
#[test]
fn a_shell_unsafe_part_stem_prints_a_safe_story_id() {
    let header = story_xml("hdr", &p(&ins("1", A, "Head")));
    let docx = docx_with_sect(
        &para("Body"),
        &[part("word/hdr!old.xml", HEADER_CT, HEADER_REL, &header)],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let changes = jubarte::changes::list_changes(&docx).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].id, "hdr%21old:rev:1");
    assert_eq!(changes[0].r#ref, "hdr%21old:rev:1");
    let stories = jubarte::inspect::stories(&docx).unwrap();
    assert_eq!(stories[0].id, "hdr%21old");
    let back = jubarte::edit::paragraph_ids(&docx, &["hdr%21old:p:0"]).unwrap();
    assert_eq!(back[0], Ok("hdr%21old:p:0".to_string()));
    let accepted = jubarte::changes::accept_changes(
        &docx,
        &jubarte::changes::ChangeFilter {
            ids: Some(vec![changes[0].r#ref.clone()]),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        jubarte::changes::list_changes(&accepted)
            .unwrap()
            .is_empty()
    );
    assert!(view(&docx).contains("hdr%21old"), "{}", view(&docx));
}
