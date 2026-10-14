// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markdown events to Word items, CriticMarkup to tracked changes.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use unicode_properties::{GeneralCategory, GeneralCategoryGroup, UnicodeGeneralCategory};

use super::critic::{self, Piece, Pieces, Token};
use super::xml::{
    Comment, Content, Document, FormatMark, Indent, Item, Kind, Label, LabelUse, Link, List, Note,
    Paragraph, Picture, Run, RunFormat, TableFormats,
};
use super::{DocxOptions, MarkdownError, PageSize, TrackChanges, WrittenDocx, package};
use super::{math, view_notes, view_stories, view_styles};

/// Writes Markdown as a `.docx`: CommonMark with GitHub's tables,
/// strikethrough, task lists and footnotes, and CriticMarkup as tracked
/// changes and comments (see the [module docs](super)).
///
/// ```
/// use jubarte::markdown::{DocxOptions, markdown_to_docx};
///
/// let written = markdown_to_docx(
///     "# Terms\n\nPayment is due in {~~30~>45~~} days.",
///     &DocxOptions::default(),
/// )
/// .unwrap();
/// assert!(written.docx.starts_with(b"PK"));
/// ```
pub fn markdown_to_docx(
    markdown: &str,
    options: &DocxOptions<'_>,
) -> Result<WrittenDocx, MarkdownError> {
    let document = read(markdown, options);
    let mut warnings = document.warnings.clone();
    if options.reference.is_some() && options.page != PageSize::default() {
        warnings.push(format!(
            "page size {} ignored: the reference document's page setup is used",
            options.page.as_str()
        ));
    }
    let docx = package::assemble(&document, options)?;
    let docx = match options.track_changes {
        _ if !options.critic => docx,
        TrackChanges::All => docx,
        TrackChanges::Accept => crate::document_comparer::accept_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
        TrackChanges::Reject => crate::document_comparer::reject_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
    };
    let docx = match options.tracking.or(document.tracking) {
        Some(on) => crate::settings::apply_settings_to_docx(
            &docx,
            &crate::settings::SettingsRequest {
                track_revisions: Some(on),
                ..Default::default()
            },
        )
        .map_err(MarkdownError::Package)?,
        None => docx,
    };
    Ok(WrittenDocx { docx, warnings })
}

/// The Markdown read into Word items.
pub(super) fn read(markdown: &str, options: &DocxOptions<'_>) -> Document {
    let source = if options.critic {
        critic::encode(markdown)
    } else {
        markdown.to_string()
    };
    let mut writer = Writer::new(options);
    writer.tables = table_plans(&source);
    writer.note_breaks = note_breaks(&source);
    for event in Parser::new_ext(&source, parser_options()) {
        writer.event(event);
    }
    writer.finish()
}

/// What the view's table line says about one table, read before the table:
/// whether its first row repeats, its grid's columns, and how each row's
/// Markdown cells and `<br>`s map onto its cells and paragraphs.
#[derive(Clone, Debug, Default)]
struct TablePlan {
    header_repeats: bool,
    columns: Option<usize>,
    align: Option<&'static str>,
    rows: Vec<RowPlan>,
    /// The cell paragraphs' mark changes, by paragraph number.
    marks: HashMap<usize, (bool, view_notes::Tag)>,
    /// Cell paragraphs aligned against their column, by paragraph number.
    aligns: HashMap<usize, &'static str>,
    /// The style the table's line names (`style Ledger`).
    style: Option<String>,
    /// A comment's range runs through the table (`in c5`).
    in_comment: bool,
    /// The formatting changes the line names.
    fmt: Option<view_notes::Tag>,
    grid_fmt: Option<u32>,
    row_fmt: HashMap<usize, view_notes::Tag>,
    cell_fmt: HashMap<(usize, usize), view_notes::Tag>,
    /// The cell paragraphs' formatting changes, by paragraph number.
    para_fmt: HashMap<usize, Vec<view_notes::Tag>>,
    /// Rows' and cells' own changes the text does not show.
    row_marks: HashMap<usize, (bool, view_notes::Tag)>,
    cell_marks: HashMap<(usize, usize), (view_notes::CellMark, view_notes::Tag)>,
}

/// One row of a [`TablePlan`].
#[derive(Clone, Debug, Default)]
struct RowPlan {
    /// Whether the row's `<br>`s end paragraphs.
    breaks: bool,
    /// The row's first paragraph's number, when its paragraphs are known
    /// one by one.
    first: Option<usize>,
    /// Each Markdown cell's shape, in order.
    cells: Vec<CellShape>,
    /// The empty cells the view left out at the row's end.
    pad: Vec<Pad>,
    /// The grid columns the row leaves out before and after its cells.
    gaps: (usize, usize),
}

/// What one Markdown cell of a planned row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellShape {
    /// A cell spanning `across` grid columns, in a vertical merge or not
    /// (as [`Item::Cell`]).
    Cell { across: usize, merge: Option<bool> },
    /// A grid column the cell before spans: the view prints it as an empty
    /// cell, which is no cell of its own.
    Spanned,
}

/// Whether each footnote definition of `source`, in order, ends paragraphs
/// at its `<br>`s: when they and its id line (`<!-- footnotes.p2-p3 -->`)
/// count the same paragraphs, as a table row's do.
fn note_breaks(source: &str) -> Vec<bool> {
    let mut breaks = Vec::new();
    let mut html = String::new();
    // The open definition's paragraphs, as its id line counts them, and
    // its `<br>`s.
    let mut open: Option<(usize, usize)> = None;
    for event in Parser::new_ext(source, parser_options()) {
        match event {
            Event::Start(Tag::HtmlBlock) => html.clear(),
            Event::Html(text) => html.push_str(&text),
            Event::Start(Tag::FootnoteDefinition(_)) => {
                let counted = html
                    .lines()
                    .rev()
                    .find_map(view_notes::note_paragraphs)
                    .unwrap_or(0);
                open = Some((counted, 0));
                html.clear();
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                if let Some((counted, lines)) = open.take() {
                    breaks.push(counted > 1 && lines + 1 == counted);
                }
            }
            Event::InlineHtml(text) if is_line_break(&text) => {
                if let Some((_, lines)) = &mut open {
                    *lines += 1;
                }
            }
            Event::Start(_) if open.is_none() => html.clear(),
            _ => {}
        }
    }
    breaks
}

/// A plan for each table of `source`, in order; `None` for a table no table
/// line precedes. A row's `<br>`s end paragraphs when its cells and `<br>`s
/// add up to the paragraphs its table line counts: each cell holds at least
/// one paragraph and each line break or comment `<br>` adds none, so only
/// then is every `<br>` a paragraph's end.
fn table_plans(source: &str) -> Vec<Option<TablePlan>> {
    let mut plans = Vec::new();
    let mut html = String::new();
    let mut line = None;
    // Each row of the open table: whether each cell is empty, and its
    // `<br>`s.
    let mut rows: Vec<(Vec<bool>, usize)> = Vec::new();
    let mut in_cell = false;
    for event in Parser::new_ext(source, parser_options()) {
        match event {
            Event::Start(Tag::HtmlBlock) => html.clear(),
            Event::Html(text) => html.push_str(&text),
            Event::Start(Tag::Table(_)) => {
                line = html.lines().rev().find_map(view_notes::table_line);
                html.clear();
                rows.clear();
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => rows.push((Vec::new(), 0)),
            Event::Start(Tag::TableCell) => {
                in_cell = true;
                if let Some(row) = rows.last_mut() {
                    row.0.push(true);
                }
            }
            Event::End(TagEnd::TableCell) => in_cell = false,
            Event::End(TagEnd::Table) => {
                plans.push(line.take().map(|line| plan_table(line, &rows)));
            }
            Event::Start(_) if !in_cell => html.clear(),
            event if in_cell => {
                if let Some(row) = rows.last_mut() {
                    if let Event::InlineHtml(text) = &event
                        && is_line_break(text)
                    {
                        row.1 += 1;
                    }
                    if let Some(empty) = row.0.last_mut() {
                        *empty = false;
                    }
                }
            }
            _ => {}
        }
    }
    plans
}

/// One empty cell the view left out at a row's end: the grid columns it
/// spans, and its vertical merge (as [`Item::Cell`]).
type Pad = (usize, Option<bool>);

/// The plan a table line makes for a table whose rows hold `rows` (each
/// Markdown cell's emptiness, and the row's `<br>`s).
fn plan_table(line: view_notes::TableLine, rows: &[(Vec<bool>, usize)]) -> TablePlan {
    // The last row each grid column's vertical merge reaches.
    let mut merged_until: HashMap<usize, usize> = HashMap::new();
    let mut shaped: Vec<(Vec<CellShape>, usize, Vec<Pad>)> = Vec::new();
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    for (r, (cells, _)) in rows.iter().enumerate() {
        let mut shapes = Vec::with_capacity(cells.len());
        // The empty cells that stand for grid columns the row leaves out.
        let (before, after) = line.gaps.get(&r).copied().unwrap_or((0, 0));
        let lead = cells
            .iter()
            .take(before)
            .take_while(|&&empty| empty)
            .count();
        // Empty cells past the grid's columns pad the row to the view's
        // widest: they are no cells.
        let mut len = cells.len();
        if let Some(columns) = line.columns {
            while len > lead && before + (len - lead) > columns && cells[len - 1] {
                len -= 1;
            }
        }
        let tail = cells[..len]
            .iter()
            .skip(lead)
            .rev()
            .take(after)
            .take_while(|&&empty| empty)
            .count();
        // The gap before is the line's, also where the view shows fewer
        // empty cells for it (a row of empty cells shows one).
        let (mut cell, mut column, mut spanned) = (0, before, 0);
        for (at, &empty) in cells.iter().enumerate() {
            if at < lead || at >= len - tail {
                shapes.push(CellShape::Spanned);
                continue;
            }
            if spanned > 0 && empty {
                spanned -= 1;
                shapes.push(CellShape::Spanned);
                continue;
            }
            let (across, down) = line.merges.get(&(r, cell)).copied().unwrap_or((1, 1));
            let merge = if down > 1 {
                merged_until.insert(column, r + down - 1);
                Some(true)
            } else {
                merged_until
                    .get(&column)
                    .is_some_and(|&until| until >= r)
                    .then_some(false)
            };
            shapes.push(CellShape::Cell { across, merge });
            spanned = across - 1;
            column += across;
            cell += 1;
        }
        // `column` already counts a span the view cut at the right edge.
        // The view leaves out empty cells at a row's end: they come back,
        // but for the grid columns its `after` gap names past them.
        // The gap after is the line's too: the cells it leaves out of the
        // grid come back, each with its span and vertical merge as the
        // table line names them.
        let end = line
            .columns
            .map_or(0, |columns| columns.saturating_sub(after.max(tail)));
        let mut pad = Vec::new();
        let mut at = cell;
        while column < end {
            let (across, down) = line.merges.get(&(r, at)).copied().unwrap_or((1, 1));
            let across = across.clamp(1, end - column);
            let merge = if down > 1 {
                merged_until.insert(column, r + down - 1);
                Some(true)
            } else {
                merged_until
                    .get(&column)
                    .is_some_and(|&until| until >= r)
                    .then_some(false)
            };
            pad.push((across, merge));
            column += across;
            at += 1;
        }
        shaped.push((shapes, cell, pad));
        gaps.push((before, after.max(tail)));
    }
    let rows: Vec<RowPlan> = match (&line.row_paragraphs, line.one_per_cell) {
        // One paragraph a cell: a `<br>` is a line break.
        (_, Some(first)) => {
            let mut at = first;
            shaped
                .into_iter()
                .map(|(cells, count, pad)| {
                    at = at.saturating_add(count + pad.len());
                    RowPlan {
                        breaks: false,
                        first: Some(at - count - pad.len()),
                        cells,
                        pad,
                        gaps: (0, 0),
                    }
                })
                .collect()
        }
        (Some(counts), None) if counts.len() == rows.len() => counts
            .iter()
            .zip(rows)
            .zip(shaped)
            .map(|((&(first, n), &(_, breaks)), (cells, count, pad))| {
                // The view leaves out empty cells only at a row's end.
                let pad = if count + pad.len() + breaks == n {
                    pad
                } else {
                    Vec::new()
                };
                let exact = count + pad.len() + breaks == n;
                RowPlan {
                    breaks: exact && breaks > 0,
                    first: exact.then_some(first),
                    cells,
                    pad,
                    gaps: (0, 0),
                }
            })
            .collect(),
        _ => shaped
            .into_iter()
            .map(|(cells, _, _)| RowPlan {
                cells,
                ..RowPlan::default()
            })
            .collect(),
    };
    let rows: Vec<RowPlan> = rows
        .into_iter()
        .zip(gaps.into_iter().chain(std::iter::repeat((0, 0))))
        .map(|(row, gaps)| RowPlan { gaps, ..row })
        .collect();
    TablePlan {
        header_repeats: line.header_repeats,
        columns: line.columns,
        align: line.align,
        rows,
        marks: line.marks,
        aligns: line.aligns,
        style: line.style,
        in_comment: line.in_comment,
        fmt: line.fmt,
        grid_fmt: line.grid_fmt,
        row_fmt: line.row_fmt,
        cell_fmt: line.cell_fmt,
        para_fmt: line.para_fmt,
        row_marks: line.row_marks,
        cell_marks: line.cell_marks,
    }
}

/// A `{>>…<<}` in a header's or footer's text that is no change's tag.
fn has_comment(text: &str) -> bool {
    let mut rest = text;
    while let Some(at) = rest.find("{>>") {
        let after = &rest[at + 3..];
        let Some(end) = after.find("<<}") else {
            return false;
        };
        if view_notes::revision_tags(&after[..end]).is_none() {
            return true;
        }
        rest = &after[end + 3..];
    }
    false
}

/// `<br>`, `<br/>` or `<br class="x">`, as the view and authors write a
/// line break.
fn is_line_break(html: &str) -> bool {
    html_tag(html).is_some_and(|tag| !tag.closing && tag.name == "br")
}

/// The value of attribute `name` in the tag `html` (`<a href="u">`),
/// quoted either way or bare, with its entities decoded.
fn html_attribute(html: &str, name: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        let before = lower[..at].chars().next_back();
        if !before.is_some_and(char::is_whitespace) {
            continue;
        }
        let Some(value) = html[from..].trim_start().strip_prefix('=') else {
            continue;
        };
        let value = value.trim_start();
        let raw = match value.chars().next() {
            Some(quote @ ('"' | '\'')) => value[1..].split(quote).next()?,
            _ => value
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()?,
        };
        return Some(crate::convert::altchunk::decode_entities(raw));
    }
    None
}

/// Whether an HTML block is only comments (the view's id lines).
fn comments_only(html: &str) -> bool {
    html.lines()
        .map(str::trim)
        .all(|line| line.is_empty() || (line.starts_with("<!--") && line.ends_with("-->")))
}

/// The elements that start a paragraph of their own in HTML.
const HTML_BLOCKS: [&str; 39] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "br",
    "caption",
    "dd",
    "details",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "ul",
];

/// The first tag's name and the text of a raw HTML block: comments, tags
/// and `<script>`/`<style>` bodies dropped, entities decoded, white space
/// collapsed, a paragraph per block element.
fn html_block_text(html: &str) -> (Option<String>, Vec<String>) {
    let mut first = None;
    let mut paragraphs = Vec::new();
    let mut text = String::new();
    let flush = |text: &mut String, paragraphs: &mut Vec<String>| {
        let decoded = crate::convert::altchunk::decode_entities(text);
        let words: Vec<&str> = decoded.split_whitespace().collect();
        if !words.is_empty() {
            paragraphs.push(words.join(" "));
        }
        text.clear();
    };
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        text.push_str(&rest[..lt]);
        rest = &rest[lt..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(gt) = rest.find('>') else {
            break;
        };
        let inner = rest[1..gt].trim();
        rest = &rest[gt + 1..];
        let name: String = inner
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if first.is_none() {
            first = Some(name.clone());
        }
        if (name == "script" || name == "style") && !inner.starts_with('/') && !inner.ends_with('/')
        {
            let close = format!("</{name}");
            rest = rest
                .to_ascii_lowercase()
                .find(&close)
                .and_then(|at| rest[at..].find('>').map(|gt| &rest[at + gt + 1..]))
                .unwrap_or("");
            continue;
        }
        if HTML_BLOCKS.contains(&name.as_str()) {
            flush(&mut text, &mut paragraphs);
        }
    }
    text.push_str(rest);
    flush(&mut text, &mut paragraphs);
    (first, paragraphs)
}

/// One inline HTML tag, as CommonMark names it.
struct HtmlTag {
    /// Lowercase, hyphens and all (`code-of-conduct`).
    name: String,
    /// `</b>`.
    closing: bool,
    /// `<b/>`: opens and closes at once.
    empty: bool,
}

/// The tag `html` is, if it is one: a comment, a declaration or a
/// processing instruction is none.
fn html_tag(html: &str) -> Option<HtmlTag> {
    let inner = html.trim().strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.starts_with(['!', '?']) {
        return None;
    }
    let (closing, inner) = match inner.strip_prefix('/') {
        Some(rest) => (true, rest.trim_start()),
        None => (false, inner),
    };
    let name: String = inner
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(HtmlTag {
        name,
        closing,
        empty: inner.ends_with('/'),
    })
}

/// The Markdown extensions jubarte reads, for the writer and for
/// [`super::pages`].
pub(super) fn parser_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_SUPERSCRIPT
        | Options::ENABLE_SUBSCRIPT
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_MATH
}

const HEADINGS: [&str; 6] = [
    "Heading1", "Heading2", "Heading3", "Heading4", "Heading5", "Heading6",
];

/// Picture width cap: 6.5 inches in EMU, the text width of a Letter page
/// with one-inch margins.
const MAX_PICTURE_WIDTH: u64 = 5_943_600;

/// A list being written: its numbering instance, its level, and how many
/// paragraphs its current item has.
struct OpenList {
    instance: usize,
    level: u32,
    paragraphs: u32,
}

/// Where a change span opened: in which paragraph and whether before any of
/// its text, so a paragraph that is one change whole can take its mark in.
#[derive(Clone, Copy)]
struct SpanStart {
    serial: u32,
    kind: Kind,
    paragraph: u64,
    at_start: bool,
    /// Runs of the start paragraph before the span opened.
    runs_before: usize,
    /// The story index of the paragraph the span opened at the end of, when
    /// its first content is that paragraph's break (`A{++\n\nB++}`).
    at_break: Option<usize>,
}

/// The cells of a table row so far: all empty, all one change whole of one
/// kind, or anything else.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RowCells {
    Empty,
    /// The kind, and the serial of the first cell's change.
    Whole(Kind, u32),
    Mixed,
}

/// A span that just closed; a comment right after it anchors to it.
#[derive(Clone, Copy)]
struct Anchor {
    slot: usize,
    group: Option<usize>,
    /// For a change: the serials of its deleted side (a substitution's) and
    /// of the rest, which attributions after it name the author of.
    serials: [Option<u32>; 2],
    /// How many attributions followed the change so far.
    attributed: u8,
}

impl Anchor {
    /// Whether a comment here may be an attribution: the first after a
    /// change, or the second after a substitution, for its inserted side.
    fn takes_attribution(&self) -> bool {
        self.group.is_none()
            && self.serials[1].is_some()
            && (self.attributed == 0 || (self.attributed == 1 && self.serials[0].is_some()))
    }
}

struct Writer<'o, 'a> {
    options: &'o DocxOptions<'a>,
    body: Vec<Item>,
    /// Footnote definitions by label.
    definitions: HashMap<String, Vec<Item>>,
    /// Substitutions one tag names (`r5+6+7`): the deleted side's serial and
    /// the inserted side's, whose ids the runs each has split.
    shared_tags: Vec<(u32, u32)>,
    /// The labels whose definition the view names an endnote
    /// (`<!-- endnotes.p0 -->` before `[^2]:`).
    endnotes: HashSet<String>,
    /// The id line just read names an endnote: the next definition is one.
    endnote_next: bool,
    /// The label of each footnote reference, in order, with its change.
    references: Vec<(String, Option<(Kind, u32)>)>,
    /// The footnote definition being read.
    target: Option<String>,
    /// What the comment closing a note's definition says, by label: the
    /// marks its text does not show (`<!-- rev footnotes.r11 @AL -->`) and
    /// its formatting changes.
    note_clauses: HashMap<String, view_notes::IdLine>,

    // Block context.
    paragraph: Option<usize>,
    paragraph_serial: u64,
    paragraph_runs: usize,
    heading: Option<usize>,
    /// What is left of a numbered heading's label at the start of its text.
    label_prefix: Option<String>,
    /// The YAML header says the text is an agent view (`view: tracked`):
    /// its id lines describe paragraphs. In other Markdown an HTML comment
    /// is the author's own and writes nothing.
    view: bool,
    /// The view shows a document with its changes accepted or rejected:
    /// the marks its lines name are history, not changes to write.
    resolved: bool,
    /// The lines of the HTML block open here, read whole when it ends.
    html_block: Option<String>,
    /// What the view's table lines say, one per table in order
    /// ([`table_plans`]); the open table's, and how many tables are read.
    tables: Vec<Option<TablePlan>>,
    table_plan: Option<TablePlan>,
    table_count: usize,
    /// The open table's rows so far, and whether the open row's `<br>`s
    /// end paragraphs.
    row_number: usize,
    row_breaks: bool,
    /// Whether each footnote definition's `<br>`s end paragraphs
    /// ([`note_breaks`]), how many definitions are read, and the open one's.
    note_breaks: Vec<bool>,
    note_count: usize,
    note_paragraph_breaks: bool,
    /// The number of the next cell paragraph, when the table line counts
    /// the open row's paragraphs one by one.
    cell_paragraph: Option<usize>,
    /// The open row's planned cells, and its left-out empty cells.
    row_shapes: Vec<CellShape>,
    row_pad: Vec<Pad>,
    /// Whether the open Markdown cell is a grid column the cell before
    /// spans ([`CellShape::Spanned`]).
    spanned: bool,
    quote: u32,
    lists: Vec<OpenList>,
    bullets: Option<usize>,
    code_block: bool,
    table: Option<Vec<Option<&'static str>>>,
    column: usize,
    cell: Option<Option<&'static str>>,
    row: Option<(usize, Option<(Kind, u32)>)>,
    /// Whether every cell of the row so far is one change whole.
    row_cells: RowCells,
    /// Runs a task list marker wrote at the start of the paragraph: a
    /// change that starts right after them takes them in.
    prefix_runs: Vec<usize>,
    image: Option<(String, String)>,
    metadata: Option<String>,

    // Inline formatting depth.
    bold: u32,
    italic: u32,
    strike: u32,
    underline: u32,
    superscript: u32,
    subscript: u32,
    /// Open `<mark>` tags: a yellow highlight.
    mark: u32,
    /// The inline HTML tags dropped so far, each warned of once.
    dropped_html: Vec<String>,
    code: u32,
    link: u32,
    /// The `<a href>` tags open in the block, which close with it.
    html_links: u32,

    // CriticMarkup.
    pieces: Pieces,
    change: Option<(Kind, u32)>,
    serial: u32,
    slot: usize,
    span_start: Option<SpanStart>,
    closed: Option<u32>,
    /// The deleted side's serial while a substitution's inserted side is open.
    substituted: Option<u32>,
    /// Where the open substitution's deleted and inserted sides start in
    /// the story: Word writes the inserted side first.
    substitution: Option<(usize, Option<usize>)>,
    /// The change the open comment follows, while the comment may still
    /// turn out to be the change's attribution.
    comment_anchor: Option<Anchor>,
    /// Highlights with no comment yet and no text since them outside a
    /// highlight: pieces of one range whose comment follows the last piece.
    pending_highlights: Vec<(usize, usize)>,
    /// The slot of the range a comment just took, while another comment
    /// may follow on the same text.
    shared_range: Option<usize>,
    /// The header's `unplaced:` marks, by the paragraph they are in: a cell
    /// paragraph has no id line to name them.
    unplaced: HashMap<usize, Vec<view_notes::Tag>>,
    run_after_close: bool,
    highlight: Option<(usize, usize)>,
    groups: usize,
    anchor: Option<Anchor>,
    comment: Option<Comment>,
    /// A body field's code being read (`{REF _Ref1 \h}`), up to its `}`.
    field_code: Option<String>,
    /// What the last id line says, for the paragraph after it.
    id_line: Option<view_notes::IdLine>,
    /// The open paragraph ends in a page break (`page-break`).
    page_break_due: bool,
    /// The view's `authors:` table, by handle (`AC` → Ann Counsel).
    authors: HashMap<String, view_notes::Author>,
    /// The changes of each author the view dates with a range of days, in
    /// the order their tags come.
    ranged: BTreeMap<String, Vec<u32>>,

    document: Document,
}

impl<'o, 'a> Writer<'o, 'a> {
    fn new(options: &'o DocxOptions<'a>) -> Self {
        Self {
            options,
            body: Vec::new(),
            definitions: HashMap::new(),
            endnotes: HashSet::new(),
            endnote_next: false,
            shared_tags: Vec::new(),
            references: Vec::new(),
            target: None,
            note_clauses: HashMap::new(),
            paragraph: None,
            paragraph_serial: 0,
            paragraph_runs: 0,
            heading: None,
            label_prefix: None,
            view: false,
            resolved: false,
            html_block: None,
            tables: Vec::new(),
            table_plan: None,
            table_count: 0,
            row_number: 0,
            row_breaks: false,
            note_breaks: Vec::new(),
            note_count: 0,
            note_paragraph_breaks: false,
            cell_paragraph: None,
            row_shapes: Vec::new(),
            row_pad: Vec::new(),
            spanned: false,
            quote: 0,
            lists: Vec::new(),
            bullets: None,
            code_block: false,
            table: None,
            column: 0,
            cell: None,
            row: None,
            row_cells: RowCells::Empty,
            prefix_runs: Vec::new(),
            image: None,
            metadata: None,
            bold: 0,
            italic: 0,
            strike: 0,
            underline: 0,
            superscript: 0,
            subscript: 0,
            mark: 0,
            dropped_html: Vec::new(),
            code: 0,
            link: 0,
            html_links: 0,
            pieces: Pieces::default(),
            change: None,
            serial: 0,
            slot: 0,
            span_start: None,
            closed: None,
            substituted: None,
            substitution: None,
            comment_anchor: None,
            pending_highlights: Vec::new(),
            shared_range: None,
            unplaced: HashMap::new(),
            id_line: None,
            page_break_due: false,
            run_after_close: false,
            highlight: None,
            groups: 0,
            anchor: None,
            comment: None,
            field_code: None,
            authors: HashMap::new(),
            ranged: BTreeMap::new(),
            document: Document::default(),
        }
    }

    fn story(&mut self) -> &mut Vec<Item> {
        match &self.target {
            Some(label) => self.definitions.entry(label.clone()).or_default(),
            None => &mut self.body,
        }
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(text) => {
                self.code += 1;
                self.text(&text);
                self.code -= 1;
            }
            Event::InlineMath(text) => self.math(&text, false),
            Event::DisplayMath(text) => self.math(&text, true),
            Event::InlineHtml(html) => self.inline_html(&html),
            Event::Html(html) => match &mut self.html_block {
                Some(block) => block.push_str(&html),
                None => self.id_lines(&html),
            },
            Event::FootnoteReference(label) => self.note_reference(&label),
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => {
                if let Some(comment) = &mut self.comment {
                    comment.paragraphs.push(String::new());
                } else {
                    self.push(Content::Break);
                }
            }
            Event::Rule => {
                self.end_paragraph();
                self.begin_paragraph();
                if let Some(index) = self.paragraph
                    && let Some(Item::Paragraph(paragraph)) = self.story().get_mut(index)
                {
                    paragraph.rule = true;
                }
                self.end_paragraph();
            }
            Event::TaskListMarker(checked) => {
                self.push(Content::Text(
                    if checked { "\u{2612} " } else { "\u{2610} " }.to_string(),
                ));
                self.paragraph_runs -= 1;
                let index = self.story().len() - 1;
                self.prefix_runs.push(index);
            }
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                self.end_paragraph();
                self.begin_paragraph();
            }
            Tag::Heading { level, .. } => {
                self.end_paragraph();
                self.heading = Some(heading_level(level));
                self.begin_paragraph();
            }
            Tag::BlockQuote(_) => {
                self.end_paragraph();
                self.quote += 1;
            }
            Tag::CodeBlock(_) => {
                self.end_paragraph();
                self.code_block = true;
                self.begin_paragraph();
            }
            Tag::List(first) => {
                self.end_paragraph();
                let level = u32::try_from(self.lists.len()).unwrap_or(8).min(8);
                let instance = match first {
                    Some(start) => {
                        self.document.lists.push(List {
                            ordered: true,
                            level,
                            start: u32::try_from(start).unwrap_or(u32::MAX),
                        });
                        self.document.lists.len() - 1
                    }
                    None => *self.bullets.get_or_insert_with(|| {
                        self.document.lists.push(List {
                            ordered: false,
                            level: 0,
                            start: 1,
                        });
                        self.document.lists.len() - 1
                    }),
                };
                self.lists.push(OpenList {
                    instance,
                    level,
                    paragraphs: 0,
                });
            }
            Tag::Item => {
                self.end_paragraph();
                if let Some(list) = self.lists.last_mut() {
                    list.paragraphs = 0;
                }
            }
            Tag::FootnoteDefinition(label) => {
                // An id line with no text before it is a body paragraph.
                self.textless_paragraph();
                self.end_paragraph();
                self.note_paragraph_breaks = self
                    .note_breaks
                    .get(self.note_count)
                    .copied()
                    .unwrap_or(false);
                self.note_count += 1;
                let label = critic::decode(&label);
                // A second definition of a label is not the note's text.
                let label = if self.definitions.contains_key(&label) {
                    self.document.warnings.push(format!(
                        "a second definition of [^{label}] was dropped: the first is the note's text"
                    ));
                    format!("\u{0}{label}")
                } else {
                    label
                };
                if std::mem::take(&mut self.endnote_next) {
                    self.endnotes.insert(label.clone());
                }
                self.target = Some(label);
            }
            Tag::Table(alignments) => {
                self.end_paragraph();
                let mut columns: Vec<_> = alignments.iter().map(|a| alignment(*a)).collect();
                self.table_plan = self
                    .tables
                    .get(self.table_count)
                    .cloned()
                    .flatten()
                    .filter(|_| self.view);
                // The grid is the line's: it keeps the columns the view left
                // out, and has none for the view's padding past it.
                let mut grid = columns.clone();
                if let Some(count) = self.table_plan.as_ref().and_then(|plan| plan.columns) {
                    grid.resize(count, None);
                    columns.resize(count.max(columns.len()), None);
                }
                let align = self.table_plan.as_ref().and_then(|plan| plan.align);
                // The style the table's line names, else the legend's for
                // every table, else none (Word's Normal Table).
                let style = self
                    .table_plan
                    .as_ref()
                    .and_then(|plan| plan.style.clone())
                    .or_else(|| self.document.table_style.as_ref().map(|(id, _)| id.clone()));
                if let Some(id) = &style {
                    if id == "TableGrid" {
                        self.document.styles.insert("TableGrid");
                    } else {
                        let border = self
                            .document
                            .table_style
                            .as_ref()
                            .filter(|(legend, _)| legend == id)
                            .and_then(|(_, border)| *border);
                        self.document.table_styles.insert(id.clone(), border);
                    }
                }
                let formats = self.table_formats();
                self.story().push(Item::Table {
                    columns: grid,
                    align,
                    style,
                    formats,
                });
                self.table = Some(columns);
                self.table_count += 1;
                self.row_number = 0;
            }
            Tag::TableHead => self.begin_row(true),
            Tag::TableRow => self.begin_row(false),
            Tag::TableCell => {
                let align = self
                    .table
                    .as_ref()
                    .and_then(|columns| columns.get(self.column).copied())
                    .flatten();
                let shape = self.row_shapes.get(self.column).copied();
                self.column += 1;
                let (across, merge) = match shape {
                    Some(CellShape::Spanned) => {
                        self.spanned = true;
                        return;
                    }
                    Some(CellShape::Cell { across, merge }) => (across, merge),
                    None => (1, None),
                };
                self.cell = Some(align);
                self.story().push(Item::Cell {
                    align,
                    across,
                    merge,
                });
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Superscript => self.superscript += 1,
            Tag::Subscript => self.subscript += 1,
            Tag::Link {
                dest_url, title, ..
            } => {
                let title = (!title.is_empty()).then(|| critic::decode(&title));
                self.link_start(&critic::decode(&dest_url), title);
            }
            Tag::Image { dest_url, .. } => {
                if self.image.is_none() {
                    self.image = Some((critic::decode(&dest_url), String::new()));
                }
            }
            Tag::MetadataBlock(_) => self.metadata = Some(String::new()),
            Tag::HtmlBlock => self.html_block = Some(String::new()),
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        // Markdown's own marks close inside their block; an inline HTML tag
        // left open (`<b>` without `</b>`) closes with it.
        if matches!(
            tag,
            TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item | TagEnd::TableCell
        ) {
            self.close_inline_tags();
        }
        match tag {
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Heading(_) => {
                self.end_paragraph();
                self.heading = None;
            }
            TagEnd::BlockQuote(_) => {
                self.end_paragraph();
                self.quote = self.quote.saturating_sub(1);
            }
            TagEnd::CodeBlock => {
                // The block's last line ends in a newline: no break after it.
                let story = self.story();
                if matches!(
                    story.last(),
                    Some(Item::Run(Run {
                        content: Content::Break,
                        ..
                    }))
                ) {
                    story.pop();
                }
                self.end_paragraph();
                self.code_block = false;
            }
            TagEnd::List(_) => {
                self.end_paragraph();
                self.lists.pop();
            }
            TagEnd::Item => self.end_paragraph(),
            TagEnd::FootnoteDefinition => {
                self.end_paragraph();
                self.target = None;
                self.note_paragraph_breaks = false;
            }
            TagEnd::Table => {
                self.story().push(Item::TableEnd);
                if !self.table_in_range() {
                    self.pending_highlights.clear();
                    self.shared_range = None;
                }
                self.table = None;
                self.table_plan = None;
                self.row_breaks = false;
                self.cell_paragraph = None;
                self.row_shapes.clear();
                self.row_pad.clear();
            }
            TagEnd::TableHead | TagEnd::TableRow => {
                // The empty cells the view left out at the row's end.
                for (across, merge) in std::mem::take(&mut self.row_pad) {
                    self.cell = Some(None);
                    self.story().push(Item::Cell {
                        align: None,
                        across,
                        merge,
                    });
                    self.begin_paragraph();
                    self.end_paragraph();
                    self.cell = None;
                    self.story().push(Item::CellEnd);
                }
                self.end_row();
            }
            TagEnd::TableCell if std::mem::take(&mut self.spanned) => {}
            TagEnd::TableCell => {
                if self.paragraph.is_none() {
                    self.begin_paragraph();
                }
                self.end_paragraph();
                self.cell = None;
                self.story().push(Item::CellEnd);
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Superscript => self.superscript = self.superscript.saturating_sub(1),
            TagEnd::Subscript => self.subscript = self.subscript.saturating_sub(1),
            TagEnd::Link => self.link_end(),
            TagEnd::Image => self.picture(),
            TagEnd::MetadataBlock(_) => {
                if let Some(text) = self.metadata.take() {
                    self.metadata(&text);
                }
            }
            TagEnd::HtmlBlock => {
                if let Some(block) = self.html_block.take() {
                    if comments_only(&block) {
                        self.id_lines(&block);
                    } else {
                        self.html_text(&block);
                    }
                }
            }
            TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition => {}
        }
    }

    /// Whether the paragraph before the one at `index` ends in a break a
    /// change of `kind` crossed (`A{++\n\n++}{++B++}`). That break is the
    /// mark the change adds, so a block after it that is one change whole
    /// keeps its own mark, which was the earlier paragraph's.
    fn after_changed_break(&mut self, index: usize, kind: Kind) -> bool {
        self.story()[..index]
            .iter()
            .rev()
            .find_map(|item| match item {
                Item::Paragraph(p) => Some(p.mark == Some(kind) && !p.whole),
                Item::Table { .. } | Item::TableEnd | Item::Cell { .. } | Item::CellEnd => {
                    Some(false)
                }
                _ => None,
            })
            .unwrap_or(false)
    }

    fn begin_row(&mut self, header: bool) {
        self.column = 0;
        // In a view the table line says whether the first row repeats.
        let header = header
            && self
                .table_plan
                .as_ref()
                .is_none_or(|plan| plan.header_repeats);
        let row = self
            .table_plan
            .as_ref()
            .and_then(|plan| plan.rows.get(self.row_number))
            .cloned()
            .unwrap_or_default();
        self.row_breaks = row.breaks;
        self.cell_paragraph = row.first;
        self.row_shapes = row.cells;
        self.row_pad = row.pad;
        self.row_number += 1;
        let index = self.story().len();
        self.story().push(Item::Row {
            header,
            change: None,
            gaps: row.gaps,
        });
        self.row = Some((index, self.change));
        self.row_cells = RowCells::Empty;
    }

    /// A row that one change covers from its first cell to its last, or
    /// whose every cell is one change whole, is inserted or deleted as a row.
    fn end_row(&mut self) {
        self.story().push(Item::RowEnd);
        let Some((index, open)) = self.row.take() else {
            return;
        };
        let row_change = match (open, self.row_cells) {
            (Some((kind, serial)), _) if self.change == Some((kind, serial)) => {
                Some((kind, serial))
            }
            (_, RowCells::Whole(kind, serial)) => Some((kind, serial)),
            _ => None,
        };
        // A row the table line names a mark of its own for has that mark,
        // and its text keeps its own (`rev r80 @AL in t1.row1`).
        let own = !self.resolved
            && self.table_plan.as_ref().is_some_and(|plan| {
                plan.row_marks
                    .contains_key(&self.row_number.saturating_sub(1))
            });
        if let Some(row_change) = row_change.filter(|_| !own)
            && let Some(Item::Row { change, .. }) = self.story().get_mut(index)
        {
            *change = Some(row_change);
        }
    }

    /// The agent view's id lines: an `empty` one (`<!-- p2-p4 empty -->`)
    /// stands for the empty paragraphs it numbers, which have no text of
    /// their own to carry them; any other names the text after it, or,
    /// with no text before the next line (`<!-- p7 page-break -->`), a
    /// paragraph of its own.
    fn id_lines(&mut self, html: &str) {
        if !self.view || self.cell.is_some() || self.comment.is_some() || self.target.is_some() {
            return;
        }
        // The view writes id lines as blocks of their own: a comment inside
        // other HTML (`<div>`) is the author's.
        if !comments_only(html) {
            return;
        }
        for line in html.lines() {
            // A page marker sits between blocks: it ends no paragraph.
            if line.trim_start().starts_with("<!-- page ") {
                continue;
            }
            // A note's id line says which part the definition after it is
            // in; it describes no body paragraph.
            self.endnote_next = view_notes::endnote_line(line);
            if self.endnote_next || view_notes::footnote_line(line) {
                continue;
            }
            self.textless_paragraph();
            if let Some(count) = view_notes::empty_paragraphs(line) {
                self.end_paragraph();
                // One empty paragraph whose id line says more than `empty`.
                if let Some(said) =
                    view_notes::id_line(line).filter(|said| *said != view_notes::IdLine::default())
                {
                    self.id_line = Some(said);
                    self.textless_paragraph();
                    continue;
                }
                for _ in 0..count {
                    let story = self.story();
                    story.push(Item::Paragraph(Paragraph::default()));
                    story.push(Item::ParagraphEnd);
                }
            } else {
                self.id_line = view_notes::id_line(line);
            }
        }
    }

    /// A raw HTML block's text, a paragraph per block element, with a
    /// warning that its markup is dropped.
    fn html_text(&mut self, html: &str) {
        if self.comment.is_some() {
            return;
        }
        let (name, paragraphs) = html_block_text(html);
        if let Some(name) = name
            && !self.dropped_html.contains(&name)
        {
            self.document.warnings.push(format!(
                "HTML block <{name}> dropped: its text is kept, its markup is not"
            ));
            self.dropped_html.push(name);
        }
        for text in paragraphs {
            self.end_paragraph();
            self.begin_paragraph();
            self.text(&text);
            self.end_paragraph();
        }
    }

    /// The paragraph of an id line no text followed.
    fn textless_paragraph(&mut self) {
        if self.id_line.is_some() {
            self.end_paragraph();
            self.begin_paragraph();
            self.end_paragraph();
        }
    }

    fn begin_paragraph(&mut self) {
        let mut paragraph = Paragraph::default();
        let list = self.lists.last_mut().map(|list| {
            let first = list.paragraphs == 0;
            list.paragraphs += 1;
            (list.instance, list.level, first)
        });
        let list_indent = list.map(|(_, level, _)| 720 * (level + 1));
        if let Some(level) = self.heading {
            // The legend's style for the level (`"#": Title1, …`).
            let legend = self.document.style_specs.iter().find(|spec| {
                spec.heading == u32::try_from(level).ok() && spec.id != HEADINGS[level - 1]
            });
            paragraph.style = Some(match legend {
                Some(spec) => Cow::Owned(spec.id.clone()),
                None => HEADINGS[level - 1].into(),
            });
            paragraph.heading = true;
        } else if self.code_block {
            paragraph.style = Some("SourceCode".into());
            paragraph.indent = list_indent;
        } else if let Some((instance, level, first)) = list {
            paragraph.style = Some("ListParagraph".into());
            if first {
                paragraph.numbering = Some((instance, level));
            } else {
                paragraph.indent = list_indent;
            }
        } else if self.quote > 0 {
            paragraph.style = Some("Quote".into());
            if self.quote > 1 {
                paragraph.indent = Some(720 * self.quote);
            }
        } else if let Some(target) = &self.target {
            paragraph.style = Some(if self.endnotes.contains(target) {
                "EndnoteText".into()
            } else {
                "FootnoteText".into()
            });
        }
        if let Some(align) = self.cell {
            paragraph.align = align;
        }
        // What the id line before it says.
        if self.cell.is_none()
            && self.target.is_none()
            && let Some(line) = self.id_line.take()
        {
            let styled = line.style.is_some();
            if let Some(style) = line.style {
                paragraph.style = Some(style.into());
                // `# Text` under its own style: the level goes with it.
                paragraph.outline = self.heading.map(|level| level - 1);
            }
            paragraph.align = line.align.or(paragraph.align);
            self.page_break_due = line.page_break;
            paragraph.section_break = line.section_break;
            paragraph.ind = Indent {
                first_line: line.first_line,
                hanging: line.hanging,
                left: line.left,
                right: line.right,
            };
            if let Some(label) = line.label {
                // The label stands for the list's own number, and a heading
                // prints it before its text.
                if let (Some(_), Label::Text(text)) = (self.heading, &label) {
                    self.label_prefix = Some(format!("{text} "));
                }
                let level = line
                    .level
                    .unwrap_or_else(|| list.map_or(0, |(_, level, _)| level));
                let start = list
                    .filter(|&(_, _, first)| first)
                    .and_then(|(instance, _, _)| self.document.lists.get(instance))
                    .filter(|list| list.ordered)
                    .map(|list| list.start);
                paragraph.label = self.label_use(label, level, start);
                if paragraph.label.is_some() {
                    paragraph.numbering = None;
                    paragraph.indent = None;
                    // The view names a list paragraph's style when it has one.
                    if !styled && list.is_some() {
                        paragraph.style = None;
                    }
                }
            }
            paragraph.formats = line.fmt.iter().map(|tag| self.format_mark(tag)).collect();
            if !self.resolved {
                paragraph.unshown = line.rev.iter().map(|tag| self.format_mark(tag)).collect();
            }
            let mark = line
                .break_ins
                .map(|tag| (Kind::Insert, tag))
                .or_else(|| line.break_del.map(|tag| (Kind::Delete, tag)));
            self.fix_mark(&mut paragraph, mark);
        }
        // A cell paragraph the table line numbers: its mark is the line's.
        if self.cell.is_some()
            && let Some(at) = self.cell_paragraph
        {
            self.cell_paragraph = Some(at + 1);
            if let Some(align) = self
                .table_plan
                .as_ref()
                .and_then(|plan| plan.aligns.get(&at).copied())
            {
                paragraph.align = Some(align);
            }
            let formats = self
                .table_plan
                .as_ref()
                .and_then(|plan| plan.para_fmt.get(&at).cloned())
                .unwrap_or_default();
            paragraph.formats = formats.iter().map(|tag| self.format_mark(tag)).collect();
            if !self.resolved
                && let Some(tags) = self.unplaced.remove(&at)
            {
                paragraph.unshown = tags.iter().map(|tag| self.format_mark(tag)).collect();
            }
            let mark = self.table_plan.as_ref().and_then(|plan| {
                let (inserted, tag) = plan.marks.get(&at)?;
                let kind = if *inserted {
                    Kind::Insert
                } else {
                    Kind::Delete
                };
                Some((kind, tag.clone()))
            });
            self.fix_mark(&mut paragraph, mark);
        }
        match &paragraph.style {
            Some(Cow::Borrowed(style)) => {
                self.document.styles.insert(style);
            }
            Some(Cow::Owned(style)) => {
                self.document.named_styles.insert(style.clone());
            }
            None => {}
        }
        let index = self.story().len();
        self.story().push(Item::Paragraph(paragraph));
        self.paragraph = Some(index);
        self.paragraph_serial += 1;
        self.paragraph_runs = 0;
        self.prefix_runs.clear();
    }

    /// The formatting changes the open table's line names.
    fn table_formats(&mut self) -> TableFormats {
        let Some(plan) = self.table_plan.as_ref() else {
            return TableFormats::default();
        };
        let (table, rows, cells) = (
            plan.fmt.clone(),
            plan.row_fmt.clone(),
            plan.cell_fmt.clone(),
        );
        let grid = plan.grid_fmt;
        let (row_marks, cell_marks) = if self.resolved {
            (HashMap::new(), HashMap::new())
        } else {
            (plan.row_marks.clone(), plan.cell_marks.clone())
        };
        if let Some(id) = grid {
            self.document.reserved_ids.insert(id);
        }
        TableFormats {
            table: table.map(|tag| self.format_mark(&tag)),
            grid,
            row_marks: row_marks
                .into_iter()
                .map(|(row, (inserted, tag))| {
                    let kind = if inserted { Kind::Insert } else { Kind::Delete };
                    (row, (kind, self.format_mark(&tag)))
                })
                .collect(),
            cell_marks: cell_marks
                .into_iter()
                .map(|(cell, (kind, tag))| (cell, (kind, self.format_mark(&tag))))
                .collect(),
            rows: rows
                .into_iter()
                .map(|(row, tag)| (row, self.format_mark(&tag)))
                .collect(),
            cells: cells
                .into_iter()
                .map(|(cell, tag)| (cell, self.format_mark(&tag)))
                .collect(),
        }
    }

    /// A formatting change or unshown mark as its tag names it.
    fn format_mark(&mut self, tag: &view_notes::Tag) -> FormatMark {
        let (author, date) = self.view_author(&tag.handle, tag.date.as_deref());
        let id = tag.ids.first().copied();
        if let Some(id) = id {
            self.document.reserved_ids.insert(id);
        }
        FormatMark { id, author, date }
    }

    /// The open table lies in a comment's range (`in c5`): its cells'
    /// highlights are pieces of that range, which goes on past them.
    fn table_in_range(&self) -> bool {
        self.table_plan.as_ref().is_some_and(|plan| plan.in_comment)
    }

    fn ensure_paragraph(&mut self) {
        if self.paragraph.is_none() {
            self.begin_paragraph();
        }
    }

    /// Ends the open paragraph. Its mark is in the change still open, or in
    /// the change that is the paragraph's whole text.
    fn end_paragraph(&mut self) {
        let Some(index) = self.paragraph.take() else {
            return;
        };
        self.label_prefix = None;
        if std::mem::take(&mut self.page_break_due) {
            self.story().push(Item::Run(Run {
                content: Content::PageBreak,
                format: RunFormat::default(),
                change: None,
                highlight: None,
            }));
        }
        if let Some(comment) = &mut self.comment {
            comment.paragraphs.push(String::new());
        }
        let after_break = self
            .span_start
            .map(|span| span.kind)
            .map(|kind| self.after_changed_break(index, kind));
        // A span opened at this paragraph's end: its first content is the break.
        if let (Some((_, serial)), Some(span)) = (self.change, self.span_start.as_mut())
            && span.serial == serial
            && span.paragraph == self.paragraph_serial
            && span.runs_before == self.paragraph_runs
            && self.cell.is_none()
        {
            span.at_break = Some(index);
        }
        let (mark, whole) = match self.change {
            Some((kind, serial)) => (Some((kind, serial)), false),
            None => match self.span_start {
                Some(span)
                    if span.at_start
                        && span.paragraph == self.paragraph_serial
                        && self.closed == Some(span.serial)
                        && !self.run_after_close
                        && self.paragraph_runs > 0
                        && !after_break.is_some_and(|after| after) =>
                {
                    (Some((span.kind, span.serial)), true)
                }
                _ => (None, false),
            },
        };
        // In a cell, a change opened at or before the paragraph's start and
        // open or just closed at its end covers it whole, across the `<br>`
        // that ends a cell paragraph (`{++A<br>B++}`).
        let covered = self.span_start.and_then(|span| {
            let starts = span.paragraph != self.paragraph_serial || span.at_start;
            let ends = self.change == Some((span.kind, span.serial))
                || (self.closed == Some(span.serial) && !self.run_after_close);
            (starts && ends).then_some((span.kind, span.serial))
        });
        let whole_in_cell = if whole { mark } else { covered };
        if self.cell.is_some() && self.paragraph_runs > 0 {
            self.row_cells = match (self.row_cells, whole_in_cell) {
                (RowCells::Empty, Some((kind, serial))) => RowCells::Whole(kind, serial),
                (RowCells::Whole(before, serial), Some((kind, _))) if before == kind => {
                    RowCells::Whole(kind, serial)
                }
                _ => RowCells::Mixed,
            };
        }
        // Blocks added (or removed) after a paragraph, from its break to the
        // end of this one: Word records them as their own marks changed, so
        // the paragraph before keeps its mark and properties.
        let shift = match (mark, self.span_start) {
            (None, Some(span))
                if span.paragraph != self.paragraph_serial
                    && self.closed == Some(span.serial)
                    && !self.run_after_close
                    && self.paragraph_runs > 0
                    && self.cell.is_none() =>
            {
                span.at_break
                    .filter(|&first| no_table_between(self.story(), first, index))
                    .map(|first| (first, span.kind, span.serial))
            }
            _ => None,
        };
        let story = self.story();
        if let Some((first, kind, serial)) = shift
            && let Some(Item::Paragraph(paragraph)) = story.get_mut(first)
        {
            if !paragraph.mark_fixed {
                paragraph.mark = None;
                paragraph.mark_by = None;
            }
            if let Some(Item::Paragraph(paragraph)) = story.get_mut(index)
                && !paragraph.mark_fixed
            {
                paragraph.mark = Some(kind);
                paragraph.mark_by = Some(serial);
                paragraph.whole = false;
            }
        } else if let Some(Item::Paragraph(paragraph)) = story.get_mut(index)
            && !paragraph.mark_fixed
        {
            paragraph.mark = mark.map(|(kind, _)| kind);
            paragraph.mark_by = mark.map(|(_, serial)| serial);
            paragraph.whole = whole;
        }
        story.push(Item::ParagraphEnd);
    }

    fn format(&self) -> RunFormat {
        RunFormat {
            bold: self.bold > 0,
            italic: self.italic > 0,
            strike: self.strike > 0,
            underline: self.underline > 0,
            superscript: self.superscript > 0,
            subscript: self.subscript > 0,
            mark: self.mark > 0,
            code: self.code > 0,
            link: self.link > 0,
        }
    }

    fn push(&mut self, content: Content) {
        self.ensure_paragraph();
        let blank = matches!(&content, Content::Text(text) if text.trim().is_empty())
            || matches!(content, Content::Tab | Content::Break);
        if self.highlight.is_none() && !blank {
            self.pending_highlights.clear();
            self.shared_range = None;
        }
        if self.code > 0 {
            self.document.styles.insert("VerbatimChar");
        }
        let run = Run {
            content,
            format: self.format(),
            change: self.change,
            highlight: self.highlight.map(|(_, group)| group),
        };
        self.story().push(Item::Run(run));
        self.paragraph_runs += 1;
        self.run_after_close = true;
        self.anchor = None;
    }

    fn text(&mut self, text: &str) {
        if let Some(metadata) = &mut self.metadata {
            metadata.push_str(text);
            return;
        }
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(&critic::decode(text));
            return;
        }
        if !self.options.critic {
            self.plain(text);
            return;
        }
        let mut pieces = Vec::new();
        self.pieces.feed(text, |piece| {
            pieces.push(match piece {
                Piece::Text(text) => Ok(text.to_string()),
                Piece::Token(token) => Err(token),
            });
        });
        for piece in pieces {
            match piece {
                Ok(text) => self.plain(&text),
                Err(token) => self.token(token),
            }
        }
    }

    /// `$latex$` (or `$$latex$$` when `display`): an equation, or its
    /// LaTeX as text where an equation cannot go (a comment, alt text).
    fn math(&mut self, latex: &str, display: bool) {
        if self.metadata.is_some() || self.image.is_some() || self.comment.is_some() {
            let fence = if display { "$$" } else { "$" };
            let text = format!("{fence}{}{fence}", critic::restore(latex));
            if let Some(metadata) = &mut self.metadata {
                metadata.push_str(&text);
            } else if let Some((_, alt)) = &mut self.image {
                alt.push_str(&text);
            } else {
                self.plain(&text);
            }
            return;
        }
        // An insertion or deletion inside the math (`$A={--π--}r^2$`) marks
        // its runs; any other CriticMarkup there is text.
        let mut source = String::with_capacity(latex.len());
        Pieces::default().feed(latex, |piece| match piece {
            Piece::Text(text) => source.push_str(text),
            Piece::Token(Token::InsertStart) => source.push(math::INSERT_START),
            Piece::Token(Token::InsertEnd) => source.push(math::INSERT_END),
            Piece::Token(Token::DeleteStart) => source.push(math::DELETE_START),
            Piece::Token(Token::DeleteEnd) => source.push(math::DELETE_END),
            Piece::Token(token) => source.push_str(critic::delimiter(token)),
        });
        self.push(Content::Math {
            latex: source,
            display,
        });
    }

    /// Text without CriticMarkup: into the open comment, or as runs, with
    /// tabs and (in code blocks) line breaks as their own runs.
    fn plain(&mut self, text: &str) {
        // A numbered heading's label, which its id line holds: not text.
        let mut text = text;
        if self.comment.is_none()
            && let Some(prefix) = self.label_prefix.take()
        {
            if let Some(rest) = text.strip_prefix(prefix.as_str()) {
                if rest.is_empty() {
                    return;
                }
                text = rest;
            } else if let Some(left) = prefix.strip_prefix(text) {
                self.label_prefix = Some(left.to_string());
                return;
            }
        }
        if text.is_empty() {
            return;
        }
        if let Some(comment) = &mut self.comment {
            if let Some(last) = comment.paragraphs.last_mut() {
                last.push_str(text);
            }
            return;
        }
        if let Some(code) = &mut self.field_code {
            code.push_str(text);
            return;
        }
        let lines: Vec<&str> = if self.code_block {
            text.split('\n').collect()
        } else {
            vec![text]
        };
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                self.push(Content::Break);
            }
            for (at, part) in line.split('\t').enumerate() {
                if at > 0 {
                    self.push(Content::Tab);
                }
                if !part.is_empty() {
                    self.push(Content::Text(part.to_string()));
                }
            }
        }
    }

    fn new_slot(&mut self) -> usize {
        self.document.slots.push(None);
        self.document.slots.len() - 1
    }

    fn open(&mut self, kind: Kind) {
        self.ensure_paragraph();
        self.serial += 1;
        self.change = Some((kind, self.serial));
        self.slot = self.new_slot();
        let slot = self.slot;
        self.story().push(Item::RangeStart(slot));
        if self.paragraph_runs == 0 {
            let change = self.change;
            for index in std::mem::take(&mut self.prefix_runs) {
                if let Some(Item::Run(run)) = self.story().get_mut(index) {
                    run.change = change;
                }
            }
        }
        self.span_start = Some(SpanStart {
            serial: self.serial,
            kind,
            paragraph: self.paragraph_serial,
            at_start: self.paragraph_runs == 0,
            runs_before: self.paragraph_runs,
            at_break: None,
        });
        self.anchor = None;
    }

    fn close(&mut self) {
        let serial = self.change.take().map(|(_, serial)| serial);
        self.anchor = Some(Anchor {
            slot: self.slot,
            group: None,
            serials: [self.substituted.take(), serial],
            attributed: 0,
        });
        self.closed = self.span_start.map(|span| span.serial);
        self.run_after_close = false;
    }

    /// `{~~old~>new~~}` is Word's substitution: the inserted text, then
    /// the deleted text, as Word's Compare writes them. The view reads a
    /// deletion before an insertion as two changes, so they keep that
    /// order. Each side moves whole with its links, bookmarks and comments;
    /// a side holding more (a paragraph's end, half a link or comment range)
    /// stays where it is.
    fn inserted_side_first(&mut self) {
        /// Runs, bookmarks, and links and comment ranges that open and
        /// close within.
        fn whole(items: &[Item]) -> bool {
            let (mut links, mut ranges) = (0usize, 0usize);
            for item in items {
                match item {
                    Item::Run(_) | Item::Bookmark(_) | Item::CommentReference(_) => {}
                    Item::LinkStart(..) => links += 1,
                    Item::LinkEnd if links > 0 => links -= 1,
                    Item::RangeStart(_) => ranges += 1,
                    Item::RangeEnd(_) if ranges > 0 => ranges -= 1,
                    _ => return false,
                }
            }
            links == 0 && ranges == 0
        }
        let Some((start, Some(split))) = self.substitution.take() else {
            return;
        };
        let story = self.story();
        let end = story.len();
        if start <= split
            && split <= end
            && whole(&story[start..split])
            && whole(&story[split..end])
        {
            story[start..end].rotate_left(split - start);
        }
    }

    fn token(&mut self, token: Token) {
        if self.comment.is_some() {
            if token == Token::CommentEnd {
                self.end_comment();
            }
            return;
        }
        match token {
            Token::InsertStart => self.open(Kind::Insert),
            Token::DeleteStart => self.open(Kind::Delete),
            Token::SubstituteStart => {
                self.open(Kind::Delete);
                self.substitution = Some((self.story().len(), None));
            }
            Token::SubstituteSeparator => {
                self.substituted = self.change.map(|(_, serial)| serial);
                self.serial += 1;
                self.change = Some((Kind::Insert, self.serial));
                self.span_start = None;
                let at = self.story().len();
                if let Some((_, split)) = &mut self.substitution {
                    *split = Some(at);
                }
            }
            Token::InsertEnd | Token::DeleteEnd => self.close(),
            Token::SubstituteEnd => {
                self.close();
                self.inserted_side_first();
            }
            Token::HighlightStart => {
                self.ensure_paragraph();
                let slot = self.new_slot();
                self.story().push(Item::RangeStart(slot));
                self.shared_range = None;
                self.highlight = Some((slot, self.groups));
                self.groups += 1;
                self.anchor = None;
            }
            Token::HighlightEnd => {
                if let Some((slot, group)) = self.highlight.take() {
                    self.pending_highlights.push((slot, group));
                    self.anchor = Some(Anchor {
                        slot,
                        group: Some(group),
                        serials: [None, None],
                        attributed: 0,
                    });
                }
            }
            Token::CommentStart => self.begin_comment(),
            Token::CommentEnd => {}
            Token::MarkStart => self.mark += 1,
            Token::MarkEnd => self.mark = self.mark.saturating_sub(1),
            Token::FieldStart => self.field_code = Some(String::new()),
            Token::FieldSeparator => {
                let code = self.field_code.take().unwrap_or_default();
                self.push(Content::FieldBegin(code));
            }
            Token::FieldEnd => self.push(Content::FieldEnd),
        }
    }

    /// A comment on the span that just closed, or on this point.
    fn begin_comment(&mut self) {
        let id = u32::try_from(self.document.comments.len()).unwrap_or(u32::MAX);
        match self.anchor.take() {
            // Anchored when it ends, unless it names the change's author.
            Some(anchor) if anchor.takes_attribution() => self.comment_anchor = Some(anchor),
            Some(anchor) => self.anchor_comment(anchor, id),
            None => self.anchor_here(id),
        }
        self.comment = Some(Comment {
            id,
            paragraphs: vec![String::new()],
            ..Comment::default()
        });
    }

    /// Anchors comment `id` on the span that just closed. A highlight's
    /// range starts at the first piece of the range it ends.
    fn anchor_comment(&mut self, anchor: Anchor, id: u32) {
        let pieces = std::mem::take(&mut self.pending_highlights);
        match anchor.group {
            Some(group) => {
                let slot = pieces.first().map_or(anchor.slot, |(slot, _)| *slot);
                self.document.slots[slot] = Some(id);
                self.shared_range = Some(slot);
                self.document.commented.insert(group);
                for (_, group) in pieces {
                    self.document.commented.insert(group);
                }
            }
            None => self.document.slots[anchor.slot] = Some(id),
        }
    }

    /// Anchors comment `id` on the highlights just before it, when there
    /// are some (`{++{==text==}++}{>>who<<}{>>note<<}`), else on this point.
    fn anchor_here(&mut self, id: u32) {
        // The next comment on the text the last one took shares its range.
        if self.pending_highlights.is_empty()
            && let Some(slot) = self.shared_range
        {
            self.document.shared.entry(slot).or_default().push(id);
            return;
        }
        if let Some(&(slot, group)) = self.pending_highlights.last() {
            let anchor = Anchor {
                slot,
                group: Some(group),
                serials: [None, None],
                attributed: 0,
            };
            self.anchor_comment(anchor, id);
            return;
        }
        self.ensure_paragraph();
        let slot = self.new_slot();
        self.document.slots[slot] = Some(id);
        self.story().push(Item::RangeStart(slot));
    }

    fn end_comment(&mut self) {
        let Some(mut comment) = self.comment.take() else {
            return;
        };
        if let Some(mut anchor) = self.comment_anchor.take() {
            let whole = comment.paragraphs.join("\n");
            // The view's tag note: `r1+2 @AC` names both sides, `r1 @AC;
            // r2 @JD` each side in turn.
            if let Some(tags) = view_notes::revision_tags(&whole) {
                let serials: Vec<u32> = if anchor.attributed == 0 {
                    anchor.serials.iter().flatten().copied().collect()
                } else {
                    anchor.serials[1..].iter().flatten().copied().collect()
                };
                let ids = mark_ids(&tags, serials.len());
                if let ([_], [deleted, inserted]) = (tags.as_slice(), serials.as_slice()) {
                    self.shared_tags.push((*deleted, *inserted));
                }
                // A substitution is written insertion first: a dated range's
                // last day goes on the mark that comes last, its deletion.
                let mut ranged = Vec::new();
                for (i, serial) in serials.iter().enumerate() {
                    let tag = if tags.len() == 1 {
                        &tags[0]
                    } else {
                        &tags[i.min(tags.len() - 1)]
                    };
                    let (author, date) = self.view_author(&tag.handle, tag.date.as_deref());
                    self.document.attributions.insert(*serial, (author, date));
                    if tag.date.is_none()
                        && self
                            .authors
                            .get(&tag.handle)
                            .is_some_and(|known| known.date.is_none() && known.range.is_some())
                    {
                        ranged.push((tag.handle.clone(), *serial));
                    }
                    if let Some(ids) = ids.get(i).filter(|ids| !ids.is_empty()) {
                        self.document.revision_ids.insert(*serial, ids.clone());
                    }
                }
                if serials.len() == 2 {
                    ranged.reverse();
                }
                for (handle, serial) in ranged {
                    self.ranged.entry(handle).or_default().push(serial);
                }
                // One number after a substitution names its deleted side:
                // the inserted side's tag may follow (`{>>r0 @AC<<}{>>r1
                // @AS<<}` when two authors made them).
                let numbers: usize = tags.iter().map(|tag| tag.ids.len()).sum();
                anchor.attributed += 1;
                if serials.len() == 2 && numbers == 1 {
                    self.anchor = anchor.takes_attribution().then_some(anchor);
                }
                return;
            }
            if let Some((author, date)) = attribution(whole.trim()) {
                let author = author.unwrap_or_else(|| self.options.author.clone());
                let serials = if anchor.attributed == 0 {
                    &anchor.serials[..]
                } else {
                    &anchor.serials[1..]
                };
                for serial in serials.iter().flatten() {
                    self.document
                        .attributions
                        .insert(*serial, (author.clone(), date.to_string()));
                }
                anchor.attributed += 1;
                // A substitution's inserted side may name its own author next;
                // any other comment after an attribution is on its own point.
                self.anchor = anchor.takes_attribution().then_some(anchor);
                return;
            }
            if anchor.attributed == 0 {
                self.anchor_comment(anchor, comment.id);
            } else {
                // After an attribution, a comment is on its own point: a
                // comment on the change would have been `{==...==}{>>...<<}`.
                self.anchor_here(comment.id);
            }
        }
        while comment.paragraphs.len() > 1
            && comment
                .paragraphs
                .last()
                .is_some_and(|p| p.trim().is_empty())
        {
            comment.paragraphs.pop();
        }
        // The view's head (`c5 @AC: text`, `c6 @AS re c5: text`), else
        // `Name (date): text`, or `Name (date)` for a comment with no text.
        let head = comment
            .paragraphs
            .first()
            .and_then(|first| view_notes::comment_head(first.trim()))
            .map(|(head, text)| (head, text.to_string()));
        if let Some((head, text)) = head {
            comment.number = Some(head.number).filter(|&n| n <= view_notes::MOST_ID);
            comment.parent = head.reply_to.filter(|&n| n <= view_notes::MOST_ID);
            comment.done = head.resolved;
            if let Some(handle) = &head.handle {
                let (author, date) = self.view_author(handle, head.date.as_deref());
                comment.by = Some((author, date));
                // A handle stands for initials unless it carries the
                // digit that tells two authors' initials apart (`U2`):
                // then the writer takes them from the name.
                if !handle.chars().any(|c| c.is_ascii_digit()) {
                    comment.initials = Some(handle.clone());
                }
            }
            comment.paragraphs[0] = text;
        } else if let Some(first) = comment.paragraphs.first_mut()
            && let Some((Some(author), date, rest)) = named(first.trim())
            && let Some(text) = rest.strip_prefix(':').or((rest.is_empty()).then_some(""))
        {
            comment.by = Some((author, date.to_string()));
            *first = text.trim_start().to_string();
        }
        self.ensure_paragraph();
        let id = comment.id;
        let story = self.story();
        story.push(Item::RangeEnd(id));
        story.push(Item::CommentReference(id));
        self.document.styles.insert("CommentText");
        self.document.styles.insert("CommentReference");
        self.document.comments.push(comment);
    }

    fn note_reference(&mut self, label: &str) {
        let label = critic::decode(label);
        if let Some(comment) = &mut self.comment {
            if let Some(last) = comment.paragraphs.last_mut() {
                last.push_str(&format!("[^{label}]"));
            }
            return;
        }
        // A label referenced again names the same note.
        let at = match self
            .references
            .iter()
            .position(|(known, _)| *known == label)
        {
            Some(at) => at,
            None => {
                self.references.push((label, self.change));
                self.references.len() - 1
            }
        };
        let position = u32::try_from(at + 1).unwrap_or(u32::MAX);
        self.push(Content::NoteReference(position));
    }

    fn inline_html(&mut self, html: &str) {
        if self.note_clause(html) {
            return;
        }
        let Some(HtmlTag {
            name,
            closing,
            empty,
        }) = html_tag(html)
        else {
            return;
        };
        if name == "br" {
            if !closing {
                self.line_break();
            }
            return;
        }
        // `<b/>` formats nothing; a comment's text is plain, so a tag in it
        // must not format the body after it.
        if empty || self.comment.is_some() {
            return;
        }
        // `<a href>` is a link; `<a>` without one (`<a name>`) is text.
        if name == "a" {
            if closing {
                if self.html_links > 0 {
                    self.html_links -= 1;
                    self.link_end();
                }
            } else if let Some(href) = html_attribute(html, "href") {
                let before = self.link;
                let title = html_attribute(html, "title");
                self.link_start(&href, title);
                self.html_links += u32::from(self.link > before);
            } else if let Some(name) =
                html_attribute(html, "id").or_else(|| html_attribute(html, "name"))
                && !name.is_empty()
                && self.comment.is_none()
                && self.image.is_none()
            {
                self.ensure_paragraph();
                self.story().push(Item::Bookmark(name));
            }
            return;
        }
        let counter = match name.as_str() {
            "b" | "strong" => &mut self.bold,
            "i" | "em" => &mut self.italic,
            "s" | "del" | "strike" => &mut self.strike,
            "u" | "ins" => &mut self.underline,
            "sup" => &mut self.superscript,
            "sub" => &mut self.subscript,
            "code" => &mut self.code,
            "mark" => &mut self.mark,
            _ => {
                if !self.dropped_html.contains(&name) {
                    self.document.warnings.push(format!(
                        "inline HTML <{name}> dropped: its text is kept, its markup is not"
                    ));
                    self.dropped_html.push(name);
                }
                return;
            }
        };
        if closing {
            *counter = counter.saturating_sub(1);
        } else {
            *counter += 1;
        }
    }

    /// Reads the comment that closes a note's definition in the view
    /// (`[^1]: Text. <!-- rev footnotes.r11 @AL -->`), and drops the space
    /// before it, which the view writes only to set the comment apart.
    fn note_clause(&mut self, html: &str) -> bool {
        let Some(label) = self.target.clone().filter(|_| self.view) else {
            return false;
        };
        let Some(line) = html
            .trim()
            .strip_prefix("<!--")
            .and_then(|rest| rest.strip_suffix("-->"))
            .and_then(|inner| view_notes::id_line(&format!("<!-- p0 {} -->", inner.trim())))
            .filter(|line| line.style.is_none() && !(line.rev.is_empty() && line.fmt.is_empty()))
        else {
            return false;
        };
        let story = self.story();
        if let Some(Item::Run(Run {
            content: Content::Text(text),
            ..
        })) = story.last_mut()
        {
            text.truncate(text.trim_end().len());
            if text.is_empty() {
                story.pop();
            }
        }
        self.note_clauses.insert(label, line);
        true
    }

    /// Closes every inline formatting mark still open.
    /// A link to `url` (`#anchor` for a heading) starts.
    fn link_start(&mut self, url: &str, title: Option<String>) {
        if self.comment.is_some() || self.image.is_some() {
            return;
        }
        self.ensure_paragraph();
        let link = match url.strip_prefix('#') {
            Some(anchor) => Link::Anchor(anchor.to_string()),
            None => Link::External(url.to_string()),
        };
        self.story().push(Item::LinkStart(link, title));
        self.document.styles.insert("Hyperlink");
        self.link += 1;
    }

    fn link_end(&mut self) {
        if self.link > 0 && self.comment.is_none() && self.image.is_none() {
            self.link -= 1;
            self.story().push(Item::LinkEnd);
        }
    }

    fn close_inline_tags(&mut self) {
        while self.html_links > 0 {
            self.html_links -= 1;
            self.link_end();
        }
        for counter in [
            &mut self.bold,
            &mut self.italic,
            &mut self.strike,
            &mut self.underline,
            &mut self.superscript,
            &mut self.subscript,
            &mut self.code,
            &mut self.mark,
        ] {
            *counter = 0;
        }
    }

    /// `<br>`: a new paragraph of a comment, of a cell whose `<br>`s end
    /// paragraphs, or else a line break.
    fn line_break(&mut self) {
        if let Some(comment) = &mut self.comment {
            comment.paragraphs.push(String::new());
        } else if (self.cell.is_some() && self.row_breaks)
            || (self.target.is_some() && self.note_paragraph_breaks)
        {
            // A cell's or a note's paragraphs, as the view joins them.
            self.ensure_paragraph();
            self.end_paragraph();
            self.begin_paragraph();
        } else {
            self.push(Content::Break);
        }
    }

    fn picture(&mut self) {
        let Some((url, alt)) = self.image.take() else {
            return;
        };
        let loaded = self.options.images.map(|load| load(&url));
        let picture = loaded
            .flatten()
            .and_then(|bytes| read_picture(bytes, alt.trim()));
        match picture {
            Some(picture) => {
                self.document.pictures.push(picture);
                let index = self.document.pictures.len() - 1;
                self.push(Content::Picture(index));
            }
            None => {
                if self.options.images.is_some() {
                    self.document
                        .warnings
                        .push(format!("image '{url}' was written as its alt text"));
                }
                self.plain(alt.trim());
            }
        }
    }

    /// Word lets no one comment in a footnote and shows no comment from
    /// one: a note's comment anchors on the note's first reference mark in
    /// the body instead, and the warnings say so.
    fn comments_out_of_notes(&mut self) {
        let mut moved: Vec<(u32, u32)> = Vec::new();
        for (index, note) in self.document.notes.iter_mut().enumerate() {
            let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
            for item in &note.items {
                if let Item::RangeStart(slot) = item
                    && let Some(id) = self.document.slots.get_mut(*slot).and_then(Option::take)
                    && !moved.iter().any(|(_, seen)| *seen == id)
                {
                    moved.push((position, id));
                }
            }
            note.items
                .retain(|item| !matches!(item, Item::RangeEnd(_) | Item::CommentReference(_)));
        }
        for (position, id) in moved {
            let Some(at) = self.document.body.iter().position(|item| {
                matches!(item, Item::Run(Run { content: Content::NoteReference(p), .. }) if *p == position)
            }) else {
                continue;
            };
            let slot = self.new_slot();
            self.document.slots[slot] = Some(id);
            self.document.body.splice(
                at..=at,
                [
                    Item::RangeStart(slot),
                    self.document.body[at].clone(),
                    Item::RangeEnd(id),
                    Item::CommentReference(id),
                ],
            );
            self.document.warnings.push(format!(
                "comment in footnote {position} moved to its reference mark: Word shows comments only in the body"
            ));
        }
    }

    /// A section's headers and footers from the front matter, with a
    /// warning for each entry that did not read.
    fn stories(&mut self, number: usize, stories: view_stories::Stories) {
        for name in &stories.dropped {
            self.document.warnings.push(format!(
                "section {number}: front matter {name} does not read and is dropped; quote its text ('…' keeps backslashes)"
            ));
        }
        self.document.stories.insert(number, stories);
    }

    /// `title:`, `author:` and `track_changes:` from a YAML front matter
    /// block.
    fn metadata(&mut self, text: &str) {
        self.authors = view_notes::authors(text);
        self.document.owner = view_notes::document_owner(text);
        self.document.table_style = view_styles::table_style(text);
        self.stories(1, view_stories::first_section(text));
        self.document.style_specs = view_styles::legend(text);
        let mut key_above = "";
        for line in text.lines() {
            // A YAML comment (`  # …`) is no part of the value.
            let line = view_styles::without_comment(line);
            if !line.starts_with(' ') {
                key_above = line.split(':').next().unwrap_or("").trim();
            } else if key_above == "sections"
                && let Some((number, page)) = view_notes::section_page(line)
            {
                if let Some(page) = page {
                    self.document.pages.insert(number, page);
                }
                if let Some((_, entry)) = line.trim().split_once(": ") {
                    self.stories(number, view_stories::section(entry));
                    if let Some(tag) = view_notes::section_format(entry) {
                        let mark = self.format_mark(&tag);
                        self.document.section_formats.insert(number, mark);
                    }
                }
            }
            // A key under another (`options:` then `  track_changes: on`)
            // is that key's, not the document's.
            if line.starts_with([' ', '\t']) {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let text = value.trim();
            let value = text.trim_matches(|c| c == '"' || c == '\'').trim();
            if value.is_empty() {
                if key.trim() == "track_changes" {
                    self.document
                        .warnings
                        .push("track_changes: with no value ignored: it is on or off".into());
                }
                continue;
            }
            match key.trim() {
                // A quoted scalar keeps the quotes inside it.
                "title" | "author" => {
                    let value = Some(critic::decode(&view_notes::scalar_and_comment(text).0));
                    if key.trim() == "title" {
                        self.document.title = value;
                    } else {
                        self.document.author = value;
                    }
                }
                "track_changes" => match value {
                    "on" => self.document.tracking = Some(true),
                    "off" => self.document.tracking = Some(false),
                    _ => self
                        .document
                        .warnings
                        .push(format!("track_changes: {value} ignored: it is on or off")),
                },
                "unplaced" => {
                    for (at, tag) in view_notes::unplaced(text) {
                        self.unplaced.entry(at).or_default().push(tag);
                    }
                }
                "view" => {
                    self.view = true;
                    self.resolved =
                        value.starts_with("accept-all") || value.starts_with("reject-all");
                }
                "page" => {
                    if let Some(page) = view_notes::page_setup(value) {
                        self.document.pages.insert(1, page);
                    }
                    if let Some(tag) = view_notes::section_format(value) {
                        let mark = self.format_mark(&tag);
                        self.document.section_formats.insert(1, mark);
                    }
                }
                _ => {}
            }
        }
        let handles: Vec<String> = self.authors.keys().cloned().collect();
        for handle in handles {
            let person = self.view_author(&handle, None);
            self.document.people.insert(handle, person);
        }
        // Word lets no one comment in a header or footer, and shows no
        // comment from one: the text stays, the comment goes. A change's
        // tag (`{>>header1.r3 @AC<<}`) is no comment.
        let mut dropped: Vec<String> = self
            .document
            .stories
            .values()
            .flat_map(|stories| &stories.stories)
            .filter(|story| {
                std::iter::once(&story.text)
                    .chain(&story.more)
                    .any(|text| has_comment(text))
            })
            .map(|story| story.part.clone())
            .collect();
        dropped.sort_unstable();
        dropped.dedup();
        for part in dropped {
            self.document.warnings.push(format!(
                "comment in {part} dropped: Word shows comments only in the body, never in a header or footer"
            ));
        }
    }

    /// A change of its own for `tag`: attributed and numbered as it says.
    fn tag_serial(&mut self, tag: &view_notes::Tag) -> u32 {
        self.serial += 1;
        let serial = self.serial;
        // An undated tag of an author dated `first..last` may be the end
        // of the range ([`Self::range_ends`]).
        if tag.date.is_none()
            && self
                .authors
                .get(&tag.handle)
                .is_some_and(|known| known.range.is_some())
        {
            self.ranged
                .entry(tag.handle.clone())
                .or_default()
                .push(serial);
        }
        let attribution = self.view_author(&tag.handle, tag.date.as_deref());
        self.document.attributions.insert(serial, attribution);
        if !tag.ids.is_empty() {
            self.document.revision_ids.insert(serial, tag.ids.clone());
        }
        serial
    }

    /// Fixes `paragraph`'s mark as a view line says it: `mark`, numbered and
    /// attributed as its tag says, or none.
    fn fix_mark(&mut self, paragraph: &mut Paragraph, mark: Option<(Kind, view_notes::Tag)>) {
        paragraph.mark_fixed = true;
        let Some((kind, tag)) = mark else {
            return;
        };
        let serial = self.tag_serial(&tag);
        paragraph.mark = Some(kind);
        paragraph.mark_by = Some(serial);
    }

    /// A paragraph's use of `label` (added to the document's labels once):
    /// its index in `Document::label_uses`; `None` for a label Word would
    /// read a number into (`%1`).
    fn label_use(&mut self, label: Label, level: u32, start: Option<u32>) -> Option<usize> {
        if let Label::Text(text) = &label
            && text
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'%' && (b'1'..=b'9').contains(&pair[1]))
        {
            return None;
        }
        let labels = &mut self.document.labels;
        let label = labels
            .iter()
            .position(|known| *known == label)
            .unwrap_or_else(|| {
                labels.push(label);
                labels.len() - 1
            });
        let uses = &mut self.document.label_uses;
        uses.push(LabelUse {
            label,
            level: level.min(8),
            start,
        });
        Some(uses.len() - 1)
    }

    /// The author a view handle names and the date for one of their notes:
    /// the note's own, else the one the `authors:` table gives them all. An
    /// author the table knows without one date (several, printed as a day
    /// range) gets none: the note is written undated, never with the
    /// conversion's date. An unknown handle stands for itself, dated as the
    /// conversion is.
    fn view_author(&self, handle: &str, date: Option<&str>) -> (String, String) {
        let known = self.authors.get(handle);
        let author = known.map_or_else(|| handle.to_string(), |a| a.name.clone());
        let date = match (date, known) {
            (Some(date), _) => date.to_string(),
            (None, Some(known)) => known.date.clone().unwrap_or_else(|| {
                // A range's first day; its last mark takes the last day.
                known
                    .range
                    .as_ref()
                    .map(|(first, _)| format!("{first}T00:00:00Z"))
                    .unwrap_or_default()
            }),
            (None, None) => self.options.date.clone(),
        };
        (author, date)
    }

    fn finish(mut self) -> Document {
        self.end_paragraph();
        self.textless_paragraph();
        if let Some(comment) = self.comment.take() {
            // An unclosed comment cannot happen with complete spans; keep
            // its text anyway.
            self.document.comments.push(comment);
        }
        let mut body = std::mem::take(&mut self.body);
        finish_story(&mut body);
        let targets: HashSet<&str> = body
            .iter()
            .chain(self.definitions.values().flatten())
            .filter_map(|item| match item {
                Item::LinkStart(Link::Anchor(name), _) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        // `<a id>` bookmarks are where their links land; no heading takes
        // their names.
        let anchors: HashSet<&str> = body
            .iter()
            .chain(self.definitions.values().flatten())
            .filter_map(|item| match item {
                Item::Bookmark(name) => Some(name.as_str()),
                _ => None,
            })
            .collect();
        let to_headings: HashSet<&str> = targets.difference(&anchors).copied().collect();
        let headings = heading_bookmarks(&body, &to_headings);
        // A link whose heading is missing still goes in; Word's jump goes
        // nowhere, so say so, once per name.
        let landed: HashSet<&str> = headings
            .iter()
            .map(|(_, slug)| slug.as_str())
            .chain(anchors.iter().copied())
            .collect();
        let mut missing: Vec<&str> = targets.difference(&landed).copied().collect();
        missing.sort_unstable();
        for name in missing {
            self.document.warnings.push(format!(
                "link to #{name} names no heading: Word writes it, but it leads nowhere"
            ));
        }
        // Slugs Word cannot take as names, and the names they get instead.
        let mut renamed: HashMap<String, String> = HashMap::new();
        let mut used: HashSet<String> = headings
            .iter()
            .filter(|(_, slug)| word_takes(slug))
            .map(|(_, slug)| slug.clone())
            .chain(anchors.iter().map(|name| (*name).to_string()))
            .collect();
        for (at, slug) in headings {
            let name = bookmark_name(&slug, &used);
            if name != slug {
                used.insert(name.clone());
                renamed.insert(slug, name.clone());
            }
            if let Item::Paragraph(paragraph) = &mut body[at] {
                paragraph.bookmark = Some(name);
            }
        }
        if !renamed.is_empty() {
            for item in body
                .iter_mut()
                .chain(self.definitions.values_mut().flatten())
            {
                if let Item::LinkStart(Link::Anchor(anchor), _) = item
                    && let Some(name) = renamed.get(anchor.as_str())
                {
                    anchor.clone_from(name);
                }
            }
        }
        self.document.body = body;
        for (label, change) in std::mem::take(&mut self.references) {
            let endnote = self.endnotes.contains(&label);
            let (text, reference) = if endnote {
                ("EndnoteText", "EndnoteReference")
            } else {
                ("FootnoteText", "FootnoteReference")
            };
            self.document.styles.insert(text);
            self.document.styles.insert(reference);
            let mut items = self.definitions.get(&label).cloned().unwrap_or_default();
            if items.is_empty() {
                items = vec![Item::Paragraph(Paragraph {
                    style: Some(text.into()),
                    ..Paragraph::default()
                })];
                items.push(Item::ParagraphEnd);
            }
            let clauses = self.note_clauses.remove(&label).unwrap_or_default();
            // A note without changes of its own is in its reference's
            // change, attribution and all.
            let wrapped = has_changes(&items);
            if let Some((kind, serial)) = change
                && !wrapped
            {
                mark_whole(&mut items, kind, serial);
            }
            // The note's reference mark goes with its text when all of it
            // is the reference's change.
            let mut change = change.filter(|(kind, _)| only_changed_by(&items, *kind));
            // Word's shape for a note inserted or deleted with its
            // reference: each paragraph's mark a change of its own, and one
            // more holding its text (the first's with the reference mark),
            // numbered in that order as the view names them.
            if let Some((kind, _)) = change
                && !self.resolved
                && in_one_change(&items, kind)
                && clauses.rev.len() == 2 * paragraphs(&items)
            {
                let serials: Vec<u32> =
                    clauses.rev.iter().map(|tag| self.tag_serial(tag)).collect();
                own_note_marks(&mut items, kind, &serials);
                change = serials.get(1).map(|&first| (kind, first));
            } else if let Some((kind, _)) = change
                && wrapped
                && clauses.rev.is_empty()
                && in_one_change(&items, kind)
            {
                // The view prints a note in its reference's change and names
                // no mark of its own: its text has none.
                for item in &mut items {
                    match item {
                        Item::Run(run) => run.change = None,
                        Item::Paragraph(paragraph) => {
                            paragraph.mark = None;
                            paragraph.mark_by = None;
                        }
                        _ => {}
                    }
                }
                change = None;
            } else if !self.resolved
                && let Some(Item::Paragraph(last)) = items
                    .iter_mut()
                    .rev()
                    .find(|item| matches!(item, Item::Paragraph(_)))
            {
                last.unshown = clauses
                    .rev
                    .iter()
                    .map(|tag| self.format_mark(tag))
                    .collect();
            }
            if !clauses.fmt.is_empty()
                && let Some(Item::Paragraph(first)) = items.first_mut()
            {
                first.formats = clauses
                    .fmt
                    .iter()
                    .map(|tag| self.format_mark(tag))
                    .collect();
            }
            finish_story(&mut items);
            self.document.notes.push(Note {
                items,
                change,
                endnote,
            });
        }
        self.share_substitution_ids();
        self.range_ends();
        self.comments_out_of_notes();
        renumber_comments(&mut self.document);
        self.document
    }

    /// An author dated `first..last` gets two timestamps back, when they
    /// have two marks or comments or more to tell apart: everything is
    /// stamped on the first day, and their last comment, else the last mark
    /// of their last change, on the last day.
    fn range_ends(&mut self) {
        let mut ranged = std::mem::take(&mut self.ranged);
        let mut handles: BTreeSet<String> = ranged.keys().cloned().collect();
        handles.extend(
            self.authors
                .iter()
                .filter(|(_, known)| known.range.is_some())
                .map(|(handle, _)| handle.clone()),
        );
        for handle in handles {
            let serials = ranged.remove(&handle).unwrap_or_default();
            let Some((name, first, last)) = self.authors.get(&handle).and_then(|known| {
                let (first, last) = known.range.as_ref()?;
                Some((
                    known.name.clone(),
                    format!("{first}T00:00:00Z"),
                    format!("{last}T23:59:59Z"),
                ))
            }) else {
                continue;
            };
            // The comments the range's first day stamped.
            let comments: Vec<usize> = self
                .document
                .comments
                .iter()
                .enumerate()
                .filter(|(_, comment)| {
                    comment
                        .by
                        .as_ref()
                        .is_some_and(|(author, date)| *author == name && *date == first)
                })
                .map(|(at, _)| at)
                .collect();
            let marks: usize = serials
                .iter()
                .map(|s| {
                    self.document
                        .revision_ids
                        .get(s)
                        .map_or(1, |ids| ids.len().max(1))
                })
                .sum();
            if let Some(&at) = comments.last()
                && marks + comments.len() > 1
            {
                if let Some((_, date)) = &mut self.document.comments[at].by {
                    date.clone_from(&last);
                }
            } else if let Some(&serial) = serials.last()
                && marks > 1
            {
                self.document.range_ends.insert(serial, last);
            }
        }
    }
}

/// The longest bookmark name Word's schema takes.
const MOST_BOOKMARK: usize = 40;

/// Word keeps a bookmark named `slug`: one that starts with a letter, of
/// at most 40 characters.
fn word_takes(slug: &str) -> bool {
    slug.starts_with(char::is_alphabetic) && slug.chars().count() <= MOST_BOOKMARK
}

/// The bookmark name for `slug`: the slug itself when Word takes it; else
/// `_` before a slug that starts with no letter (hidden, like Word's own
/// `_Toc` bookmarks), cut to its first 40 characters, or fewer and `-1`,
/// `-2`, … when that cut is `used`.
fn bookmark_name(slug: &str, used: &HashSet<String>) -> String {
    if word_takes(slug) {
        return slug.to_string();
    }
    let slug = if slug.starts_with(char::is_alphabetic) {
        Cow::Borrowed(slug)
    } else {
        Cow::Owned(format!("_{slug}"))
    };
    if slug.chars().count() <= MOST_BOOKMARK && !used.contains(slug.as_ref()) {
        return slug.into_owned();
    }
    (0..)
        .map(|n| {
            let suffix = if n == 0 {
                String::new()
            } else {
                format!("-{n}")
            };
            let head: String = slug.chars().take(MOST_BOOKMARK - suffix.len()).collect();
            head + &suffix
        })
        .find(|name| !used.contains(name))
        .unwrap_or_default()
}

/// The slug github-slugger gives a heading's text: lowercase, then every
/// character dropped that is no letter, mark, decimal or letter number,
/// connector punctuation (`_`) or `-`, and each plain space made `-`.
pub(super) fn github_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' => Some(c),
            c if c.is_alphabetic() => Some(c),
            c => match c.general_category_group() {
                GeneralCategoryGroup::Letter | GeneralCategoryGroup::Mark => Some(c),
                GeneralCategoryGroup::Number
                    if c.general_category() != GeneralCategory::OtherNumber =>
                {
                    Some(c)
                }
                GeneralCategoryGroup::Punctuation
                    if c.general_category() == GeneralCategory::ConnectorPunctuation =>
                {
                    Some(c)
                }
                _ => None,
            },
        })
        .collect()
}

/// The headings a link names (`[x](#the-intro)`), by position in `body`,
/// each with the slug GitHub gives its text ([`github_slug`], a repeat
/// `-1`, `-2`, …). Deleted text is no part of it; an empty slug names
/// nothing.
fn heading_bookmarks(body: &[Item], targets: &HashSet<&str>) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    if targets.is_empty() {
        return found;
    }
    let mut seen: HashMap<String, usize> = HashMap::new();
    for (at, item) in body.iter().enumerate() {
        let Item::Paragraph(paragraph) = item else {
            continue;
        };
        if !paragraph.heading {
            continue;
        }
        let text: String = body[at + 1..]
            .iter()
            .take_while(|item| !matches!(item, Item::ParagraphEnd))
            .filter_map(|item| match item {
                Item::Run(Run {
                    content: Content::Text(text),
                    change,
                    ..
                }) if !matches!(change, Some((Kind::Delete, _))) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        let slug = github_slug(&text);
        // github-slugger: a repeat retries its suffixed name against every
        // name given out (`Intro`, `Intro`, `Intro 1`: `intro`, `intro-1`,
        // `intro-1-1`).
        let mut name = slug.clone();
        while seen.contains_key(&name) {
            let repeats = seen.entry(slug.clone()).or_insert(0);
            *repeats += 1;
            name = format!("{slug}-{repeats}");
        }
        seen.insert(name.clone(), 0);
        if slug.is_empty() {
            continue;
        }
        if targets.contains(name.as_str()) {
            found.push((at, name));
        }
    }
    found
}

/// The `w:id`s of each of `serials` changes a revision note names: one
/// tag names them all (`r1+2`: the deletion 1, the insertion 2; a change of
/// several runs `r5+6`), each in turn and the last the rest; several tags
/// (`r1 @AC; r2 @JD`) one change each.
fn mark_ids(tags: &[view_notes::Tag], serials: usize) -> Vec<Vec<u32>> {
    match tags {
        [] => Vec::new(),
        [tag] => (0..serials)
            .map(|i| match i.cmp(&(serials - 1)) {
                std::cmp::Ordering::Less => tag.ids.get(i).copied().into_iter().collect(),
                _ => tag.ids.get(i..).map(<[u32]>::to_vec).unwrap_or_default(),
            })
            .collect(),
        tags => (0..serials)
            .map(|i| tags[i.min(tags.len() - 1)].ids.clone())
            .collect(),
    }
}

impl Writer<'_, '_> {
    /// A substitution's one tag (`r5+6+7`) lists the deleted side's ids,
    /// then the inserted side's. Word numbers marks in document order and
    /// writes the insertion first, so where the numbers drop (`r3+4+1+2`)
    /// the inserted side's begin; with no drop the deleted side takes one
    /// id per run it has (`plain _italic_` is two). Either side keeps one
    /// at least.
    fn share_substitution_ids(&mut self) {
        let mut runs: HashMap<u32, usize> = HashMap::new();
        let notes = self.document.notes.iter().flat_map(|note| &note.items);
        for item in self.document.body.iter().chain(notes) {
            if let Item::Run(Run {
                change: Some((_, by)),
                ..
            }) = item
            {
                *runs.entry(*by).or_default() += 1;
            }
        }
        for (deleted, inserted) in std::mem::take(&mut self.shared_tags) {
            let ids = &mut self.document.revision_ids;
            let mut all = ids.get(&deleted).cloned().unwrap_or_default();
            all.extend(ids.get(&inserted).cloned().unwrap_or_default());
            if all.len() < 3 {
                continue;
            }
            let drop = all.windows(2).position(|pair| pair[1] < pair[0]);
            let pieces = drop.map_or_else(|| runs.get(&deleted).copied().unwrap_or(1), |at| at + 1);
            let cut = pieces.clamp(1, all.len() - 1);
            let rest = all.split_off(cut);
            ids.insert(deleted, all);
            ids.insert(inserted, rest);
        }
    }
}

/// Comments keep the number the view gives them (`c5` is `w:id="5"`), so
/// a reply's `re c5` and every later `c5` still name the same comment; one
/// with no number, or with a number an earlier comment took, gets the
/// lowest free one. Replies then name their parent by its `w:id`.
fn renumber_comments(document: &mut Document) {
    let mut taken = HashSet::new();
    let claimed: Vec<Option<u32>> = document
        .comments
        .iter()
        .map(|c| c.number.filter(|&n| taken.insert(n)))
        .collect();
    let mut next = 0u32;
    let mut ids = HashMap::new();
    for (comment, claimed) in document.comments.iter().zip(&claimed) {
        let id = claimed.unwrap_or_else(|| {
            while taken.contains(&next) {
                next += 1;
            }
            taken.insert(next);
            next
        });
        ids.insert(comment.id, id);
    }
    let numbered: HashSet<u32> = claimed.iter().flatten().copied().collect();
    for comment in &mut document.comments {
        comment.id = ids[&comment.id];
        comment.parent = comment
            .parent
            .filter(|p| numbered.contains(p) && *p != comment.id);
    }
    for id in document
        .slots
        .iter_mut()
        .flatten()
        .chain(document.shared.values_mut().flatten())
    {
        *id = ids[id];
    }
    let notes = document
        .notes
        .iter_mut()
        .flat_map(|note| note.items.iter_mut());
    for item in document.body.iter_mut().chain(notes) {
        if let Item::RangeEnd(id) | Item::CommentReference(id) = item {
            *id = ids[id];
        }
    }
}

/// A story's last paragraph mark cannot be inserted or deleted in Word; a
/// last paragraph that was added or removed whole puts its change on the
/// mark before it instead. A story that ends in a table gets the empty
/// paragraph Word ends one with.
fn finish_story(items: &mut Vec<Item>) {
    if matches!(items.last(), Some(Item::TableEnd)) {
        // The table's own change, when every row of it has one: the closing
        // paragraph came with the table, or goes with it.
        let start = items
            .iter()
            .rposition(|item| matches!(item, Item::Table { .. }))
            .unwrap_or(0);
        let mut rows = items[start..].iter().filter_map(|item| match item {
            Item::Row { change, .. } => Some(*change),
            _ => None,
        });
        let first = rows.next().flatten();
        let change = first
            .filter(|(kind, _)| rows.all(|change| change.is_some_and(|(other, _)| other == *kind)));
        items.push(Item::Paragraph(Paragraph {
            mark: change.map(|(kind, _)| kind),
            mark_by: change.map(|(_, serial)| serial),
            whole: change.is_some(),
            ..Paragraph::default()
        }));
        items.push(Item::ParagraphEnd);
    }
    let mut depth = 0usize;
    let mut last: Option<usize> = None;
    for index in (0..items.len()).rev() {
        match &items[index] {
            Item::TableEnd => depth += 1,
            Item::Table { .. } => depth = depth.saturating_sub(1),
            Item::Paragraph(paragraph) if depth == 0 => match last {
                None => {
                    if !(paragraph.whole && paragraph.mark.is_some()) {
                        return;
                    }
                    last = Some(index);
                }
                Some(at) => {
                    let (kind, by) = match &mut items[at] {
                        Item::Paragraph(ending) => {
                            ending.whole = false;
                            (ending.mark.take(), ending.mark_by.take())
                        }
                        _ => (None, None),
                    };
                    let Item::Paragraph(before) = &mut items[index] else {
                        return;
                    };
                    if before.mark_fixed {
                        // Its id line said what its mark is.
                    } else if before.mark.is_none() {
                        before.mark = kind;
                        before.mark_by = by;
                    } else if before.whole && before.mark == kind {
                        // Paragraphs added (or removed) whole up to the end:
                        // the mark before the first of them takes the change.
                        let earlier = items[..index].iter().rposition(|item| match item {
                            Item::Paragraph(p) => !(p.whole && p.mark == kind),
                            _ => false,
                        });
                        if let Some(earlier) = earlier
                            && let Item::Paragraph(p) = &mut items[earlier]
                            && p.mark.is_none()
                            && !p.mark_fixed
                        {
                            p.mark = kind;
                            p.mark_by = by;
                        }
                    } else if before.whole && before.mark != kind && at == end_of(items, index) + 1
                    {
                        // A paragraph removed whole, then the last one added
                        // whole (or the other way round): the last paragraph
                        // is replaced, so both texts go in it.
                        let content: Vec<Item> = items.drain(index + 1..at - 1).collect();
                        items.drain(index..index + 2);
                        let last = index;
                        items.splice(last + 1..last + 1, content);
                    }
                    return;
                }
            },
            _ => {}
        }
    }
    if let Some(at) = last
        && let Item::Paragraph(only) = &mut items[at]
    {
        only.mark = None;
        only.mark_by = None;
        only.whole = false;
    }
}

/// The author and date of an attribution, a comment that is only
/// `Name (date)` or `(date)` (the date ISO 8601, as Word writes it): how
/// CriticMarkup records who made the change before it, and how a document
/// read from Word writes it.
fn attribution(text: &str) -> Option<(Option<String>, &str)> {
    match named(text)? {
        (author, date, "") => Some((author, date)),
        _ => None,
    }
}

/// `Name (date)` or `(date)` at the start of `text`: the name, the date and
/// the text after them.
pub(super) fn named(text: &str) -> Option<(Option<String>, &str, &str)> {
    let open = text.find('(')?;
    let close = open + text[open..].find(')')?;
    let date = &text[open + 1..close];
    if !is_date(date) {
        return None;
    }
    let name = &text[..open];
    let author = match name.trim() {
        "" if name.is_empty() => None,
        "" => return None,
        author if name.ends_with(' ') && !author.contains(['(', ')']) => Some(author.to_string()),
        _ => return None,
    };
    Some((author, date, &text[close + 1..]))
}

/// `YYYY-MM-DDTHH:MM`, then optional seconds and fraction, then an
/// optional `Z` or `+HH:MM` offset.
pub(super) fn is_date(text: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let Some((day, time)) = text.split_once('T') else {
        return false;
    };
    let day: Vec<&str> = day.split('-').collect();
    if day.len() != 3
        || day[0].len() != 4
        || day[1..].iter().any(|p| p.len() != 2)
        || !day.iter().all(|p| digits(p))
    {
        return false;
    }
    let (clock, zone) = match time.find(['Z', '+', '-']) {
        Some(at) => time.split_at(at),
        None => (time, ""),
    };
    let zone_ok = match zone {
        "" | "Z" => true,
        zone => {
            let offset = &zone[1..];
            offset.len() == 5
                && offset.as_bytes()[2] == b':'
                && digits(&offset[..2])
                && digits(&offset[3..])
        }
    };
    let clock: Vec<&str> = clock.split(':').collect();
    let seconds_ok = clock.get(2).is_none_or(|seconds| {
        let (whole, fraction) = seconds.split_once('.').unwrap_or((seconds, "0"));
        whole.len() == 2 && digits(whole) && digits(fraction)
    });
    zone_ok
        && (2..=3).contains(&clock.len())
        && clock[..2].iter().all(|p| p.len() == 2 && digits(p))
        && seconds_ok
}

/// Whether no table starts or ends between two story indexes.
fn no_table_between(items: &[Item], from: usize, to: usize) -> bool {
    !items[from..to]
        .iter()
        .any(|item| matches!(item, Item::Table { .. } | Item::TableEnd))
}

/// The index of the `ParagraphEnd` of the paragraph that starts at `start`.
fn end_of(items: &[Item], start: usize) -> usize {
    items[start..]
        .iter()
        .position(|item| matches!(item, Item::ParagraphEnd))
        .map_or(items.len(), |offset| start + offset)
}

/// Whether every run of `items` is in a change of `kind`.
fn only_changed_by(items: &[Item], kind: Kind) -> bool {
    items.iter().all(|item| match item {
        Item::Run(run) => run.change.is_some_and(|(k, _)| k == kind),
        _ => true,
    })
}

/// Whether every run and every paragraph mark but the last of a note is in
/// one change of `kind`.
fn in_one_change(items: &[Item], kind: Kind) -> bool {
    let last = items
        .iter()
        .rposition(|item| matches!(item, Item::Paragraph(_)));
    let mut serial = None;
    items.iter().enumerate().all(|(index, item)| {
        let change = match item {
            Item::Run(run) => run.change,
            Item::Paragraph(paragraph) if Some(index) != last => {
                paragraph.mark.zip(paragraph.mark_by)
            }
            _ => return true,
        };
        change.is_some_and(|(own, by)| own == kind && *serial.get_or_insert(by) == by)
    })
}

fn paragraphs(items: &[Item]) -> usize {
    items
        .iter()
        .filter(|item| matches!(item, Item::Paragraph(_)))
        .count()
}

/// Gives a note in one change Word's own marks: paragraph by paragraph,
/// its mark the change `serials` names first and its runs the next.
fn own_note_marks(items: &mut [Item], kind: Kind, serials: &[u32]) {
    let mut next = serials.iter().copied();
    let mut runs = None;
    for item in items {
        match item {
            Item::Paragraph(paragraph) => {
                paragraph.mark = Some(kind);
                paragraph.mark_by = next.next();
                paragraph.mark_fixed = true;
                runs = next.next();
            }
            Item::Run(run) => run.change = runs.map(|serial| (kind, serial)),
            _ => {}
        }
    }
}

fn has_changes(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Run(run) => run.change.is_some(),
        Item::Paragraph(paragraph) => paragraph.mark.is_some(),
        _ => false,
    })
}

/// Puts a whole footnote in the change of its reference: its text and every
/// paragraph mark but the last, which the note keeps.
fn mark_whole(items: &mut [Item], kind: Kind, serial: u32) {
    let last = items
        .iter()
        .rposition(|item| matches!(item, Item::Paragraph(_)));
    for (index, item) in items.iter_mut().enumerate() {
        match item {
            Item::Run(run) => run.change = Some((kind, serial)),
            Item::Paragraph(paragraph) if Some(index) != last => {
                paragraph.mark = Some(kind);
                paragraph.mark_by = Some(serial);
            }
            _ => {}
        }
    }
}

fn heading_level(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn alignment(alignment: Alignment) -> Option<&'static str> {
    match alignment {
        Alignment::None => None,
        Alignment::Left => Some("left"),
        Alignment::Center => Some("center"),
        Alignment::Right => Some("right"),
    }
}

/// A picture Word can show, sized at 96 dots per inch and at most the text
/// width.
pub(crate) fn read_picture(bytes: Vec<u8>, alt: &str) -> Option<Picture> {
    use image::ImageFormat;
    let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .ok()?;
    let (extension, content_type) = match reader.format()? {
        ImageFormat::Png => ("png", "image/png"),
        ImageFormat::Jpeg => ("jpeg", "image/jpeg"),
        ImageFormat::Gif => ("gif", "image/gif"),
        ImageFormat::Bmp => ("bmp", "image/bmp"),
        ImageFormat::Tiff => ("tiff", "image/tiff"),
        _ => return None,
    };
    let (width, height) = reader.into_dimensions().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    let mut cx = u64::from(width) * 9525;
    let mut cy = u64::from(height) * 9525;
    if cx > MAX_PICTURE_WIDTH {
        cy = cy * MAX_PICTURE_WIDTH / cx;
        cx = MAX_PICTURE_WIDTH;
    }
    Some(Picture {
        bytes,
        extension,
        content_type,
        width: cx,
        height: cy.max(1),
        alt: alt.to_string(),
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn dates_are_iso_8601_to_the_minute_or_finer() {
        for date in [
            "2026-01-02T03:04",
            "2026-01-02T03:04:05",
            "2026-01-02T03:04:05Z",
            "2026-01-02T03:04:05.123Z",
            "2026-01-02T03:04:05+01:00",
            "2026-01-02T03:04-05:30",
        ] {
            assert!(is_date(date), "{date}");
        }
        for text in [
            "",
            "2026-01-02",
            "2026-1-02T03:04",
            "26-01-02T03:04",
            "2026-01-02T3:04",
            "2026-01-02T03:04:5",
            "2026-01-02T03:04:05.Z",
            "2026-01-02T03:04:05+0100",
            "2026-01-02T03:04:05+01:0x",
            "2026-01-02T03:04:05:06",
            "yesterday",
        ] {
            assert!(!is_date(text), "{text}");
        }
    }

    #[test]
    fn a_revision_note_numbers_the_marks_of_its_changes() {
        let tag = |ids: &[u32]| view_notes::Tag {
            handle: "AC".into(),
            date: None,
            ids: ids.to_vec(),
        };
        // A substitution: the deletion, then the insertion.
        assert_eq!(mark_ids(&[tag(&[1, 2])], 2), [vec![1], vec![2]]);
        // One change of several runs takes every id.
        assert_eq!(mark_ids(&[tag(&[5, 6])], 1), [vec![5, 6]]);
        // The insertion of a substitution in two runs takes the rest.
        assert_eq!(mark_ids(&[tag(&[1, 2, 3])], 2), [vec![1], vec![2, 3]]);
        // Too few ids: the changes left over take none.
        assert_eq!(mark_ids(&[tag(&[4])], 2), [vec![4], vec![]]);
        // One tag per change.
        assert_eq!(mark_ids(&[tag(&[1]), tag(&[2])], 2), [vec![1], vec![2]]);
        assert!(mark_ids(&[], 2).is_empty());
        assert!(mark_ids(&[tag(&[1])], 0).is_empty());
    }

    #[test]
    fn names_and_attributions() {
        assert_eq!(
            named("Ana Lima (2026-01-02T03:04Z): hi"),
            Some((Some("Ana Lima".to_string()), "2026-01-02T03:04Z", ": hi"))
        );
        assert_eq!(
            named("(2026-01-02T03:04Z)"),
            Some((None, "2026-01-02T03:04Z", ""))
        );
        for text in [
            "Ana(2026-01-02T03:04Z)",
            " (2026-01-02T03:04Z)",
            "A (b) (2026-01-02T03:04Z)",
            "Ana (soon)",
            "Ana 2026-01-02T03:04Z",
        ] {
            assert_eq!(named(text), None, "{text}");
        }
        assert_eq!(
            attribution("Bo (2026-01-02T03:04:05Z)"),
            Some((Some("Bo".to_string()), "2026-01-02T03:04:05Z"))
        );
        assert_eq!(attribution("Bo (2026-01-02T03:04:05Z): why"), None);
        assert_eq!(attribution("Bo"), None);
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod markdown_authored_boundary_tests {
    use super::*;

    #[test]
    fn malformed_iso_components_cannot_become_revision_attributions() {
        let invalid = [
            "2026-10T12:34",
            "2026-10-09-01T12:34",
            "xxxx-10-09T12:34",
            "2026-xx-09T12:34",
            "2026-10-xxT12:34",
            "2026-10-09T12:34:ab",
            "2026-10-09T12:34:01.ab",
            "2026-10-09T12:34+01x30",
            "2026-10-09T12:34+ab:30",
            "2026-10-09T12:34+01:ab",
            "2026-10-09T12:34+",
            "2026-10-09T12:34:01:02",
            "2026-10-09T12:34:01.",
            "2026-10-09Txx:34",
            "2026-10-09T12:xx",
        ];
        for date in invalid {
            assert!(!is_date(date), "{date}");
            assert_eq!(
                named(&format!("Ada ({date}): owned comment")),
                None,
                "{date}"
            );
            assert_eq!(attribution(&format!("Ada ({date})")), None, "{date}");
        }
        for date in [
            "2026-10-09T12:34",
            "2026-10-09T12:34:00.5-03:30",
            "2026-10-09T12:34+00:00",
        ] {
            assert!(is_date(date), "{date}");
            let input = format!("Ada ({date}): owned comment");
            assert_eq!(
                named(&input),
                Some((Some("Ada".to_string()), date, ": owned comment"))
            );
            assert_eq!(attribution(&input), None);
        }
        let date = "2026-10-09T12:34Z";
        for author in ["Ad)a ", "Ada", " ", "\t"] {
            assert_eq!(named(&format!("{author}({date})")), None, "{author:?}");
        }
    }

    #[test]
    fn front_matter_quote_variants_preserve_decoded_authored_metadata() {
        for (title, author) in [
            ("'Agreement'", "'Ada'"),
            ("\"Agreement\"", "\"Ada\""),
            ("Agreement", "Ada"),
        ] {
            let source = format!(
                "---\ntitle: {title}\nauthor: {author}\nignored: extra\nempty: ''\n---\n\nBody"
            );
            let document = read(&source, &DocxOptions::default());
            assert_eq!(document.title.as_deref(), Some("Agreement"));
            assert_eq!(document.author.as_deref(), Some("Ada"));
            assert_eq!(
                document.body,
                vec![
                    Item::Paragraph(Paragraph::default()),
                    Item::Run(Run {
                        content: Content::Text("Body".to_string()),
                        format: RunFormat::default(),
                        change: None,
                        highlight: None
                    }),
                    Item::ParagraphEnd,
                ]
            );
            assert!(document.comments.is_empty());
            assert!(document.warnings.is_empty());
        }
    }

    #[test]
    fn comment_links_and_non_attribution_suffixes_keep_the_complete_comment() {
        let date = "2026-10-09T12:34Z";
        for (source, expected, by) in [
            (
                "Body{>>[owned](https://example.invalid) tail<<}".to_string(),
                "owned tail".to_string(),
                None,
            ),
            (
                format!("Body{{>>Ada ({date}) suffix<<}}"),
                format!("Ada ({date}) suffix"),
                None,
            ),
            (
                format!("Body{{>>Ada ({date}): owned text<<}}"),
                "owned text".to_string(),
                Some(("Ada".to_string(), date.to_string())),
            ),
            (
                "Body{>>one<br>two<<}".to_string(),
                "one\ntwo".to_string(),
                None,
            ),
        ] {
            let document = read(&source, &DocxOptions::default());
            assert_eq!(document.comments.len(), 1, "{source}");
            let comment = &document.comments[0];
            assert_eq!(comment.id, 0);
            assert_eq!(comment.paragraphs.join("\n"), expected, "{source}");
            assert_eq!(comment.by, by, "{source}");
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter_map(|item| match item {
                        Item::Run(run) => Some(&run.content),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                vec![&Content::Text("Body".to_string())]
            );
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter(|item| matches!(item, Item::CommentReference(0)))
                    .count(),
                1
            );
            assert_eq!(
                document
                    .body
                    .iter()
                    .filter(|item| matches!(item, Item::LinkStart(..) | Item::LinkEnd))
                    .count(),
                0
            );
            assert!(document.attributions.is_empty());
            assert!(document.warnings.is_empty());
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod memory_picture_import_contract_tests {
    use super::*;

    fn encoded(format: image::ImageFormat, width: u32, height: u32) -> Vec<u8> {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            width,
            height,
            image::Rgb([31, 97, 163]),
        ));
        let mut out = std::io::Cursor::new(Vec::new());
        image.write_to(&mut out, format).unwrap();
        out.into_inner()
    }

    #[test]
    fn memory_picture_formats_keep_exact_payload_alt_and_capped_aspect_ratio() {
        for (format, extension, mime) in [
            (image::ImageFormat::Png, "png", "image/png"),
            (image::ImageFormat::Jpeg, "jpeg", "image/jpeg"),
            (image::ImageFormat::Gif, "gif", "image/gif"),
            (image::ImageFormat::Bmp, "bmp", "image/bmp"),
            (image::ImageFormat::Tiff, "tiff", "image/tiff"),
        ] {
            for (width, height, cx, cy) in [
                (1, 2, 9525, 19050),
                (624, 1, 5_943_600, 9525),
                (625, 2, 5_943_600, 19019),
            ] {
                let bytes = encoded(format, width, height);
                let got = read_picture(bytes.clone(), "owned Ω alt").unwrap();
                assert_eq!(got.bytes, bytes);
                assert_eq!(got.extension, extension);
                assert_eq!(got.content_type, mime);
                assert_eq!((got.width, got.height), (cx, cy));
                assert_eq!(got.alt, "owned Ω alt");
            }
        }
    }

    #[test]
    fn rejected_memory_images_preserve_alt_text_and_report_only_requested_load_failures() {
        for bytes in [
            Vec::new(),
            b"not an image".to_vec(),
            b"\x89PNG\r\n\x1a\n".to_vec(),
            b"RIFF\x04\0\0\0WEBP".to_vec(),
        ] {
            assert!(read_picture(bytes.clone(), "owned").is_none());
            let load = |url: &str| {
                assert_eq!(url, "owned.png");
                Some(bytes.clone())
            };
            let options = DocxOptions {
                images: Some(&load),
                ..DocxOptions::default()
            };
            let doc = read("before ![owned Ω](owned.png) after", &options);
            assert!(doc.pictures.is_empty());
            assert_eq!(
                doc.warnings,
                ["image 'owned.png' was written as its alt text"]
            );
            let text: String = doc
                .body
                .iter()
                .filter_map(|item| match item {
                    Item::Run(Run {
                        content: Content::Text(text),
                        ..
                    }) => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(text, "before owned Ω after");
        }
        let no_load = read("![owned Ω](owned.png)", &DocxOptions::default());
        assert!(no_load.warnings.is_empty());
        assert!(no_load.pictures.is_empty());
        assert!(no_load.body.iter().any(|item| matches!(item, Item::Run(Run { content: Content::Text(text), .. }) if text == "owned Ω")));
        let load = |_: &str| None;
        let missing = read(
            "![owned Ω](owned.png)",
            &DocxOptions {
                images: Some(&load),
                ..DocxOptions::default()
            },
        );
        assert_eq!(
            missing.warnings,
            ["image 'owned.png' was written as its alt text"]
        );
        assert!(missing.pictures.is_empty());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod html_tag_tests {
    use super::*;

    #[test]
    fn a_line_break_is_br_by_name_with_or_without_attributes() {
        for br in ["<br>", "<br/>", "<BR />", "<br class=\"x\">", "< br >"] {
            assert!(is_line_break(br), "{br}");
        }
        for not in ["</br>", "<bra>", "<br-x>", "<b>"] {
            assert!(!is_line_break(not), "{not}");
        }
    }
}
