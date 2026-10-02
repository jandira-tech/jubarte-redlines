// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markdown events to Word items, CriticMarkup to tracked changes.

use std::collections::HashMap;

use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::critic::{self, Piece, Pieces, Token};
use super::xml::{
    Comment, Content, Document, Item, Kind, Link, List, Note, Paragraph, Picture, Run, RunFormat,
};
use super::{DocxOptions, MarkdownError, TrackChanges, WrittenDocx, package};

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
    let warnings = document.warnings.clone();
    let docx = package::assemble(&document, options)?;
    let docx = match options.track_changes {
        _ if !options.critic => docx,
        TrackChanges::All => docx,
        TrackChanges::Accept => crate::document_comparer::accept_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
        TrackChanges::Reject => crate::document_comparer::reject_revisions(&docx)
            .map_err(|e| MarkdownError::Package(e.to_string()))?,
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
    for event in Parser::new_ext(&source, parser_options()) {
        writer.event(event);
    }
    writer.finish()
}

fn parser_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
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
    /// The label of each footnote reference, in order, with its change.
    references: Vec<(String, Option<(Kind, u32)>)>,
    /// The footnote definition being read.
    target: Option<String>,

    // Block context.
    paragraph: Option<usize>,
    paragraph_serial: u64,
    paragraph_runs: usize,
    heading: Option<usize>,
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
    code: u32,
    link: u32,

    // CriticMarkup.
    pieces: Pieces,
    change: Option<(Kind, u32)>,
    serial: u32,
    slot: usize,
    span_start: Option<SpanStart>,
    closed: Option<u32>,
    /// The deleted side's serial while a substitution's inserted side is open.
    substituted: Option<u32>,
    /// The change the open comment follows, while the comment may still
    /// turn out to be the change's attribution.
    comment_anchor: Option<Anchor>,
    /// Highlights with no comment yet and no text since them outside a
    /// highlight: pieces of one range whose comment follows the last piece.
    pending_highlights: Vec<(usize, usize)>,
    run_after_close: bool,
    highlight: Option<(usize, usize)>,
    groups: usize,
    anchor: Option<Anchor>,
    comment: Option<Comment>,

    document: Document,
}

impl<'o, 'a> Writer<'o, 'a> {
    fn new(options: &'o DocxOptions<'a>) -> Self {
        Self {
            options,
            body: Vec::new(),
            definitions: HashMap::new(),
            references: Vec::new(),
            target: None,
            paragraph: None,
            paragraph_serial: 0,
            paragraph_runs: 0,
            heading: None,
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
            code: 0,
            link: 0,
            pieces: Pieces::default(),
            change: None,
            serial: 0,
            slot: 0,
            span_start: None,
            closed: None,
            substituted: None,
            comment_anchor: None,
            pending_highlights: Vec::new(),
            run_after_close: false,
            highlight: None,
            groups: 0,
            anchor: None,
            comment: None,
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
            Event::InlineMath(text) | Event::DisplayMath(text) => self.text(&text),
            Event::InlineHtml(html) => self.inline_html(&html),
            Event::Html(_) => {}
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
                self.end_paragraph();
                let label = critic::decode(&label);
                // A second definition of a label is not the note's text.
                let label = if self.definitions.contains_key(&label) {
                    format!("\u{0}{label}")
                } else {
                    label
                };
                self.target = Some(label);
            }
            Tag::Table(alignments) => {
                self.end_paragraph();
                let columns: Vec<_> = alignments.iter().map(|a| alignment(*a)).collect();
                self.story().push(Item::Table {
                    columns: columns.clone(),
                });
                self.table = Some(columns);
                self.document.styles.insert("TableGrid");
            }
            Tag::TableHead => self.begin_row(true),
            Tag::TableRow => self.begin_row(false),
            Tag::TableCell => {
                let align = self
                    .table
                    .as_ref()
                    .and_then(|columns| columns.get(self.column).copied())
                    .flatten();
                self.column += 1;
                self.cell = Some(align);
                self.story().push(Item::Cell(align));
            }
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Superscript => self.superscript += 1,
            Tag::Subscript => self.subscript += 1,
            Tag::Link { dest_url, .. } => {
                if self.comment.is_none() && self.image.is_none() {
                    self.ensure_paragraph();
                    let url = critic::decode(&dest_url);
                    let link = match url.strip_prefix('#') {
                        Some(anchor) => Link::Anchor(anchor.to_string()),
                        None => Link::External(url),
                    };
                    self.story().push(Item::LinkStart(link));
                    self.document.styles.insert("Hyperlink");
                    self.link += 1;
                }
            }
            Tag::Image { dest_url, .. } => {
                if self.image.is_none() {
                    self.image = Some((critic::decode(&dest_url), String::new()));
                }
            }
            Tag::MetadataBlock(_) => self.metadata = Some(String::new()),
            Tag::HtmlBlock
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
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
            }
            TagEnd::Table => {
                self.story().push(Item::TableEnd);
                self.pending_highlights.clear();
                self.table = None;
            }
            TagEnd::TableHead | TagEnd::TableRow => self.end_row(),
            TagEnd::TableCell => {
                if self.paragraph.is_none() {
                    self.begin_paragraph();
                }
                self.end_paragraph();
                self.cell = None;
                self.story().push(Item::CellEnd);
                self.pending_highlights.clear();
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Superscript => self.superscript = self.superscript.saturating_sub(1),
            TagEnd::Subscript => self.subscript = self.subscript.saturating_sub(1),
            TagEnd::Link => {
                if self.link > 0 && self.comment.is_none() && self.image.is_none() {
                    self.link -= 1;
                    self.story().push(Item::LinkEnd);
                }
            }
            TagEnd::Image => self.picture(),
            TagEnd::MetadataBlock(_) => {
                if let Some(text) = self.metadata.take() {
                    self.metadata(&text);
                }
            }
            TagEnd::HtmlBlock
            | TagEnd::DefinitionList
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
                Item::Table { .. } | Item::TableEnd | Item::Cell(_) | Item::CellEnd => Some(false),
                _ => None,
            })
            .unwrap_or(false)
    }

    fn begin_row(&mut self, header: bool) {
        self.column = 0;
        let index = self.story().len();
        self.story().push(Item::Row {
            header,
            change: None,
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
        if let Some(row_change) = row_change
            && let Some(Item::Row { change, .. }) = self.story().get_mut(index)
        {
            *change = Some(row_change);
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
            paragraph.style = Some(HEADINGS[level - 1]);
        } else if self.code_block {
            paragraph.style = Some("SourceCode");
            paragraph.indent = list_indent;
        } else if let Some((instance, level, first)) = list {
            paragraph.style = Some("ListParagraph");
            if first {
                paragraph.numbering = Some((instance, level));
            } else {
                paragraph.indent = list_indent;
            }
        } else if self.quote > 0 {
            paragraph.style = Some("Quote");
            if self.quote > 1 {
                paragraph.indent = Some(720 * self.quote);
            }
        } else if self.target.is_some() {
            paragraph.style = Some("FootnoteText");
        }
        if let Some(align) = self.cell {
            paragraph.align = align;
        }
        if let Some(style) = paragraph.style {
            self.document.styles.insert(style);
        }
        let index = self.story().len();
        self.story().push(Item::Paragraph(paragraph));
        self.paragraph = Some(index);
        self.paragraph_serial += 1;
        self.paragraph_runs = 0;
        self.prefix_runs.clear();
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
        if self.cell.is_some() && self.paragraph_runs > 0 {
            self.row_cells = match (self.row_cells, whole.then_some(mark).flatten()) {
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
            paragraph.mark = None;
            paragraph.mark_by = None;
            if let Some(Item::Paragraph(paragraph)) = story.get_mut(index) {
                paragraph.mark = Some(kind);
                paragraph.mark_by = Some(serial);
                paragraph.whole = false;
            }
        } else if let Some(Item::Paragraph(paragraph)) = story.get_mut(index) {
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

    /// Text without CriticMarkup: into the open comment, or as runs, with
    /// tabs and (in code blocks) line breaks as their own runs.
    fn plain(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some(comment) = &mut self.comment {
            if let Some(last) = comment.paragraphs.last_mut() {
                last.push_str(text);
            }
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

    fn token(&mut self, token: Token) {
        if self.comment.is_some() {
            if token == Token::CommentEnd {
                self.end_comment();
            }
            return;
        }
        match token {
            Token::InsertStart => self.open(Kind::Insert),
            Token::DeleteStart | Token::SubstituteStart => self.open(Kind::Delete),
            Token::SubstituteSeparator => {
                self.substituted = self.change.map(|(_, serial)| serial);
                self.serial += 1;
                self.change = Some((Kind::Insert, self.serial));
                self.span_start = None;
            }
            Token::InsertEnd | Token::DeleteEnd | Token::SubstituteEnd => self.close(),
            Token::HighlightStart => {
                self.ensure_paragraph();
                let slot = self.new_slot();
                self.story().push(Item::RangeStart(slot));
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
            by: None,
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
        // `Name (date): text`, or `Name (date)` for a comment with no text.
        if let Some(first) = comment.paragraphs.first_mut()
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
        self.references.push((label, self.change));
        let position = u32::try_from(self.references.len()).unwrap_or(u32::MAX);
        self.document.styles.insert("FootnoteReference");
        self.document.styles.insert("FootnoteText");
        self.push(Content::NoteReference(position));
    }

    fn inline_html(&mut self, html: &str) {
        let tag: String = html
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();
        match tag.as_str() {
            "<br>" | "<br/>" => {
                if let Some(comment) = &mut self.comment {
                    comment.paragraphs.push(String::new());
                } else {
                    self.push(Content::Break);
                }
            }
            "<sup>" => self.superscript += 1,
            "</sup>" => self.superscript = self.superscript.saturating_sub(1),
            "<sub>" => self.subscript += 1,
            "</sub>" => self.subscript = self.subscript.saturating_sub(1),
            "<u>" | "<ins>" => self.underline += 1,
            "</u>" | "</ins>" => self.underline = self.underline.saturating_sub(1),
            _ => {}
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

    /// `title:` and `author:` from a YAML front matter block.
    fn metadata(&mut self, text: &str) {
        for line in text.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim().trim_matches(|c| c == '"' || c == '\'').trim();
            if value.is_empty() {
                continue;
            }
            match key.trim() {
                "title" => self.document.title = Some(critic::decode(value)),
                "author" => self.document.author = Some(critic::decode(value)),
                _ => {}
            }
        }
    }

    fn finish(mut self) -> Document {
        self.end_paragraph();
        if let Some(comment) = self.comment.take() {
            // An unclosed comment cannot happen with complete spans; keep
            // its text anyway.
            self.document.comments.push(comment);
        }
        let mut body = std::mem::take(&mut self.body);
        finish_story(&mut body);
        self.document.body = body;
        for (label, change) in std::mem::take(&mut self.references) {
            let mut items = self.definitions.get(&label).cloned().unwrap_or_default();
            if items.is_empty() {
                items = vec![Item::Paragraph(Paragraph {
                    style: Some("FootnoteText"),
                    ..Paragraph::default()
                })];
                items.push(Item::ParagraphEnd);
            }
            // A note without changes of its own is in its reference's
            // change, attribution and all.
            if let Some((kind, serial)) = change
                && !has_changes(&items)
            {
                mark_whole(&mut items, kind, serial);
            }
            // The note's reference mark goes with its text when all of it
            // is the reference's change.
            let change = change.filter(|(kind, _)| only_changed_by(&items, *kind));
            finish_story(&mut items);
            self.document.notes.push(Note { items, change });
        }
        self.document
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
                    if before.mark.is_none() {
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
fn is_date(text: &str) -> bool {
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
