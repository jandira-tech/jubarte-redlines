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
}

/// The cells of a table row so far: all empty, all one change whole of one
/// kind, or anything else.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RowCells {
    Empty,
    Whole(Kind),
    Mixed,
}

/// A span that just closed; a comment right after it anchors to it.
#[derive(Clone, Copy)]
struct Anchor {
    slot: usize,
    group: Option<usize>,
}

struct Writer<'o, 'a> {
    options: &'o DocxOptions<'a>,
    body: Vec<Item>,
    /// Footnote definitions by label.
    definitions: HashMap<String, Vec<Item>>,
    /// The label of each footnote reference, in order, with its change.
    references: Vec<(String, Option<Kind>)>,
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
        let kind = match (open, self.row_cells) {
            (Some((kind, serial)), _) if self.change == Some((kind, serial)) => Some(kind),
            (_, RowCells::Whole(kind)) => Some(kind),
            _ => None,
        };
        if let Some(kind) = kind
            && let Some(Item::Row { change, .. }) = self.story().get_mut(index)
        {
            *change = Some(kind);
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
        let (mark, whole) = match self.change {
            Some((kind, _)) => (Some(kind), false),
            None => match self.span_start {
                Some(span)
                    if span.at_start
                        && span.paragraph == self.paragraph_serial
                        && self.closed == Some(span.serial)
                        && !self.run_after_close
                        && self.paragraph_runs > 0 =>
                {
                    (Some(span.kind), true)
                }
                _ => (None, false),
            },
        };
        if self.cell.is_some() && self.paragraph_runs > 0 {
            self.row_cells = match (self.row_cells, whole.then_some(mark).flatten()) {
                (RowCells::Empty, Some(kind)) => RowCells::Whole(kind),
                (RowCells::Whole(before), Some(kind)) if before == kind => RowCells::Whole(kind),
                _ => RowCells::Mixed,
            };
        }
        let story = self.story();
        if let Some(Item::Paragraph(paragraph)) = story.get_mut(index) {
            paragraph.mark = mark;
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
        });
        self.anchor = None;
    }

    fn close(&mut self) {
        self.change = None;
        self.anchor = Some(Anchor {
            slot: self.slot,
            group: None,
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
                    self.anchor = Some(Anchor {
                        slot,
                        group: Some(group),
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
            Some(anchor) => {
                self.document.slots[anchor.slot] = Some(id);
                if let Some(group) = anchor.group {
                    self.document.commented.insert(group);
                }
            }
            None => {
                self.ensure_paragraph();
                let slot = self.new_slot();
                self.document.slots[slot] = Some(id);
                self.story().push(Item::RangeStart(slot));
            }
        }
        self.comment = Some(Comment {
            id,
            paragraphs: vec![String::new()],
        });
    }

    fn end_comment(&mut self) {
        let Some(mut comment) = self.comment.take() else {
            return;
        };
        while comment.paragraphs.len() > 1
            && comment
                .paragraphs
                .last()
                .is_some_and(|p| p.trim().is_empty())
        {
            comment.paragraphs.pop();
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
        self.references
            .push((label, self.change.map(|(kind, _)| kind)));
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
            if let Some(kind) = change
                && !has_changes(&items)
            {
                self.serial += 1;
                mark_whole(&mut items, kind, self.serial);
            }
            // The note's reference mark goes with its text when all of it
            // is the reference's change.
            let change = change.filter(|kind| only_changed_by(&items, *kind));
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
        let kind = first.filter(|kind| rows.all(|change| change == Some(*kind)));
        items.push(Item::Paragraph(Paragraph {
            mark: kind,
            whole: kind.is_some(),
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
                    let kind = match &mut items[at] {
                        Item::Paragraph(ending) => {
                            ending.whole = false;
                            ending.mark.take()
                        }
                        _ => None,
                    };
                    let Item::Paragraph(before) = &mut items[index] else {
                        return;
                    };
                    if before.mark.is_none() {
                        before.mark = kind;
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
        only.whole = false;
    }
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
            Item::Paragraph(paragraph) if Some(index) != last => paragraph.mark = Some(kind),
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
fn read_picture(bytes: Vec<u8>, alt: &str) -> Option<Picture> {
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
