// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The changes between two documents as a git-style patch.
//!
//! ```text
//! --- a/agreement.md
//! +++ b/agreement.md<TAB>Arthur Rodrigues<TAB>2026-09-30T14:05:00Z
//! @@ [line:3] @@
//! Closing on [-04/20/26-]{+10/30/26+}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Financing.<<}.
//! ```
//!
//! Only the paragraphs that changed are shown, each whole, under the place it
//! has in the new version (`@@ [line:3] @@`), or in the old one when it was
//! removed (`@@ -[line:7] @@`). Deleted text is `[-...-]` and inserted text
//! `{+...+}`, as `git diff --word-diff` writes them; highlights and comments
//! stay CriticMarkup (`{==...==}`, `{>>...<<}`). The document owner and the
//! date of the patch are on the `+++` line, after tabs; a change by anyone else, or at
//! another time, is followed by its author and date (`{>>Name (date)<<}`),
//! and every comment names its author and date.
//!
//! The text is wrapped at [`DEFAULT_COLUMNS`], only at spaces, and never
//! where the break would change the Markdown: inside a code span or a link
//! target, after a backslash, before a list, heading, quote or table marker,
//! or in a heading, table row or code block.

use std::collections::HashSet;
use std::fmt;

use super::critic::{self, Piece, Token};
use super::diff::{continues, diff_markdown, marker_len, open_paragraph, table_row};
use super::redline::{RedlineOptions, Source, redline};
use super::write::named;
use super::{MarkdownError, MarkdownOptions, docx_to_markdown};
use crate::document_comparer::{accept_revisions, reject_revisions};
use crate::inspect;

/// The width [`Patch`] wraps to when displayed.
pub const DEFAULT_COLUMNS: usize = 72;

/// Who made a change or comment, and when (`YYYY-MM-DDTHH:MM:SSZ`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribution {
    /// The name Word shows.
    pub author: String,
    /// `YYYY-MM-DDTHH:MM:SSZ`.
    pub date: String,
}

/// The names on the `---` and `+++` lines and the document owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatchOptions {
    /// The old document's name, after `--- a/`.
    pub old_name: String,
    /// The new document's name, after `+++ b/`.
    pub new_name: String,
    /// Author and date of every change and comment that names none.
    pub owner: Attribution,
}

/// Where a hunk's paragraph is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Locator {
    /// The 1-based line of a Markdown document.
    Line(usize),
    /// A paragraph of a Word document by the id an edit plan names it by:
    /// `body:p:3`, `footnotes:p:0`.
    Paragraph {
        /// `body`, or the part a header, footer or note is in
        /// (`header1`, `footnotes`).
        story: String,
        /// Its 0-based order in the story.
        index: usize,
    },
}

impl fmt::Display for Locator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Line(line) => write!(f, "line:{line}"),
            Self::Paragraph { story, index } => write!(f, "{story}:p:{index}"),
        }
    }
}

/// A comment, on a change, on highlighted text, or at a point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    /// Who wrote it.
    pub author: String,
    /// When.
    pub date: String,
    /// What it says.
    pub text: String,
    /// The highlighted text the comment is on, if any.
    pub on: Option<String>,
}

/// Text deleted, inserted, or both (a replacement).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The text deleted (empty for an insertion).
    pub old: String,
    /// The text inserted (empty for a deletion).
    pub new: String,
    /// Who made the change.
    pub author: String,
    /// When.
    pub date: String,
    /// The comment written right after the change.
    pub comment: Option<Comment>,
}

/// One changed paragraph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    /// The paragraph in the new version, or in the old one when `removed`.
    pub at: Locator,
    /// The paragraph is not in the new version.
    pub removed: bool,
    /// The paragraph with its markup, unwrapped.
    pub text: String,
    /// The changes, in order.
    pub changes: Vec<Change>,
    /// Comments not written right after a change.
    pub comments: Vec<Comment>,
}

/// The changed paragraphs of a document. Displayed, it is
/// [`render`](Self::render)ed at [`DEFAULT_COLUMNS`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    /// The old document's name.
    pub old_name: String,
    /// The new document's name.
    pub new_name: String,
    /// Author and date of the changes and comments that name none.
    pub owner: Attribution,
    /// The changed paragraphs, in document order.
    pub hunks: Vec<Hunk>,
}

impl Patch {
    /// The patch as text, wrapped at `columns` (`0`: not wrapped). Empty
    /// when nothing changed.
    pub fn render(&self, columns: usize) -> String {
        if self.hunks.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "--- a/{}\n+++ b/{}\t{}\t{}\n",
            path_name(&self.old_name),
            path_name(&self.new_name),
            self.owner.author,
            self.owner.date
        );
        for (index, hunk) in self.hunks.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            let sign = if hunk.removed { "-" } else { "" };
            out.push_str(&format!("@@ {sign}[{}] @@\n", hunk.at));
            out.push_str(&wrap(&hunk.text, columns));
            out.push('\n');
        }
        out
    }
}

impl fmt::Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render(DEFAULT_COLUMNS))
    }
}

/// A file name as `git diff` shows it after `a/`: without `./` or `/`.
fn path_name(name: &str) -> &str {
    let name = name.strip_prefix("./").unwrap_or(name);
    name.trim_start_matches('/')
}

/// The changes from `old` to `new`, two Markdown documents, as a patch whose
/// changes are the owner's.
///
/// ```
/// use jubarte::markdown::{Attribution, PatchOptions, patch_markdown};
///
/// let options = PatchOptions {
///     old_name: "a.md".into(),
///     new_name: "b.md".into(),
///     owner: Attribution { author: "Ana".into(), date: "2026-09-30T14:05:00Z".into() },
/// };
/// let patch = patch_markdown("Due in 30 days.\n", "Due in 45 days.\n", &options);
/// assert_eq!(
///     patch.to_string(),
///     "--- a/a.md\n+++ b/b.md\tAna\t2026-09-30T14:05:00Z\n@@ [line:1] @@\nDue in [-30-]{+45+} days.\n"
/// );
/// ```
pub fn patch_markdown(old: &str, new: &str, options: &PatchOptions) -> Patch {
    let (mut patch, texts) = build(&diff_markdown(old, new), options);
    // Each hunk at the line its paragraph starts on in the document itself,
    // found in order; where it is not found (text the diff escaped), the
    // line counted from the markup stays.
    let lines = |text: &str| -> Vec<String> {
        text.replace("\r\n", "\n")
            .lines()
            .map(str::to_string)
            .collect()
    };
    let (old_lines, new_lines) = (lines(old), lines(new));
    let (mut old_from, mut new_from) = (0, 0);
    let texts = texts.into_iter().filter(|t| t.hunk.is_some());
    for (
        hunk,
        Paragraphs {
            old: old_text,
            new: new_text,
            ..
        },
    ) in patch.hunks.iter_mut().zip(texts)
    {
        let (text, lines, from) = if hunk.removed {
            (old_text, &old_lines, &mut old_from)
        } else {
            (new_text, &new_lines, &mut new_from)
        };
        let first = text.trim_start_matches('\n').lines().next().unwrap_or("");
        if first.is_empty() {
            continue;
        }
        if let Some(found) = lines[(*from).min(lines.len())..]
            .iter()
            .position(|l| l == first)
        {
            let line = *from + found;
            hunk.at = Locator::Line(line + 1);
            *from = line + 1;
        }
    }
    patch
}

/// A Markdown document whose CriticMarkup holds the changes (as
/// [`diff_markdown`] or [`docx_to_markdown`](super::docx_to_markdown) write
/// it) as a patch. `{>>Name (date)<<}` right after a change is that change's
/// author and date; `{>>Name (date): text<<}` is a comment by `Name`; any
/// other comment, and every change without an author, is the owner's.
pub fn patch_critic(critic: &str, options: &PatchOptions) -> Patch {
    build(critic, options).0
}

/// A Word redline's tracked changes and comments as a patch, each hunk at
/// the id ([`body:p:N`](Locator::Paragraph)) [`jubarte::inspect`](crate::inspect) gives its
/// paragraph once every change is accepted (or rejected, for a removed
/// paragraph): the id an edit plan names it by. A change in a text box is
/// at the paragraph that holds the box. Every comment is shown.
pub fn patch_redline(docx: &[u8], options: &PatchOptions) -> Result<Patch, MarkdownError> {
    let package = |error: crate::opc::OpcError| MarkdownError::Package(error.to_string());
    let old = reject_revisions(docx).map_err(package)?;
    let new = accept_revisions(docx).map_err(package)?;
    patch_word(docx, &old, &new, None, options)
}

/// The changes from `old` to `new` as a patch. Two Markdown documents give
/// [`patch_markdown`]'s patch, at lines; otherwise the documents are
/// compared as [`redline`] compares them and each hunk is at a `body:p:N`
/// of the Word document on its side (of the redline, accepted or rejected,
/// where that side is Markdown). The comparer's changes are the owner's;
/// a comment the old document has too is not a change.
pub fn patch_documents(
    old: Source<'_>,
    new: Source<'_>,
    redline_options: &RedlineOptions<'_>,
    options: &PatchOptions,
) -> Result<Patch, MarkdownError> {
    if let (Source::Markdown(old), Source::Markdown(new)) = (old, new) {
        return Ok(patch_markdown(old, new, options));
    }
    let mut compared = redline_options.clone();
    compared.settings.author_for_revisions = options.owner.author.clone();
    compared.settings.date_time_for_revisions = options.owner.date.clone();
    let docx = redline(old, new, &compared)?;
    let package = |error: crate::opc::OpcError| MarkdownError::Package(error.to_string());
    let old = match old {
        Source::Docx(docx) => docx.to_vec(),
        Source::Markdown(_) => reject_revisions(&docx).map_err(package)?,
    };
    let new = match new {
        Source::Docx(docx) => docx.to_vec(),
        Source::Markdown(_) => accept_revisions(&docx).map_err(package)?,
    };
    let known = segments(&docx_to_markdown(&old, &MarkdownOptions::default())?.markdown)
        .into_iter()
        .filter_map(|segment| match segment {
            Segment::Note(note) => Some(note),
            _ => None,
        })
        .collect();
    patch_word(&docx, &old, &new, Some(&known), options)
}

/// The redline's patch with each hunk at the paragraph of `old` or `new`
/// its text is, found in order.
fn patch_word(
    docx: &[u8],
    old: &[u8],
    new: &[u8],
    known: Option<&HashSet<String>>,
    options: &PatchOptions,
) -> Result<Patch, MarkdownError> {
    let critic = docx_to_markdown(docx, &MarkdownOptions::default())?.markdown;
    let (mut patch, texts) = build_with(&critic, options, known);
    let (old, new) = (word_paragraphs(old)?, word_paragraphs(new)?);
    // Every paragraph is looked for, changed or not, so the search follows
    // the document; text found in no paragraph is in a text box, and goes
    // to the paragraph found last, the one that holds the box.
    let mut sides = [Search::new(&old), Search::new(&new)];
    for block in texts {
        let old_at = sides[0].next(&block.old);
        let new_at = sides[1].next(&block.new);
        if let Some(hunk) = block.hunk.and_then(|i| patch.hunks.get_mut(i)) {
            let at = if hunk.removed { old_at } else { new_at };
            if let Some(at) = at {
                hunk.at = at;
            }
        }
    }
    Ok(patch)
}

/// A Word paragraph as [`patch_word`] looks for it.
struct WordParagraph {
    at: Locator,
    letters: String,
}

/// The paragraphs of one version, looked for in order.
struct Search<'a> {
    letters: Vec<&'a str>,
    paragraphs: &'a [WordParagraph],
    /// Where the search goes on from.
    next: usize,
    /// The paragraph found last.
    last: usize,
}

impl<'a> Search<'a> {
    fn new(paragraphs: &'a [WordParagraph]) -> Self {
        Self {
            letters: paragraphs.iter().map(|p| p.letters.as_str()).collect(),
            paragraphs,
            next: 0,
            last: 0,
        }
    }

    /// Where a block with this Markdown is: the next paragraph with its
    /// text, else the one found last. `None` when the block has no text in
    /// this version, or the document no paragraphs.
    fn next(&mut self, markdown: &str) -> Option<Locator> {
        if !visible(markdown) {
            return None;
        }
        if let Some(found) = find(&letters(&plain(markdown)), &self.letters, self.next) {
            self.next = found + 1;
            self.last = found;
        }
        self.paragraphs.get(self.last).map(|p| p.at.clone())
    }
}

/// The body's paragraphs, then the headers', footers' and notes', as
/// `jubarte text` lists them.
fn word_paragraphs(docx: &[u8]) -> Result<Vec<WordParagraph>, MarkdownError> {
    let error = |error: inspect::InspectError| MarkdownError::Docx(error.to_string());
    let body = inspect::paragraphs(docx).map_err(error)?;
    let stories = inspect::stories(docx).map_err(error)?;
    let all = body.into_iter().map(|p| ("body".to_string(), p)).chain(
        stories
            .into_iter()
            .flat_map(|s| s.paragraphs.into_iter().map(move |p| (s.id.clone(), p))),
    );
    Ok(all
        .map(|(story, p)| WordParagraph {
            at: Locator::Paragraph {
                story,
                index: p.index,
            },
            letters: letters(&p.text),
        })
        .collect())
}

/// The first paragraph from `from` on whose text is the block's, else one
/// the block holds (a table row's cell, a paragraph and the one joined to
/// it, a list item's text after its number).
fn find(block: &str, paragraphs: &[&str], from: usize) -> Option<usize> {
    let rest = paragraphs.get(from..)?;
    let at = |test: &dyn Fn(&str) -> bool| {
        rest.iter()
            .position(|p| !p.is_empty() && test(p))
            .map(|i| from + i)
    };
    at(&|p| p == block).or_else(|| at(&|p| block.contains(p)))
}

/// Letters and digits only.
fn letters(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Markdown without what a paragraph's text does not have: the list or
/// heading marker, link and image targets, footnote references and HTML
/// tags.
fn plain(markdown: &str) -> String {
    let markdown = markdown.trim_start_matches('\n');
    let mut out = String::new();
    let mut rest = &markdown[marker_len(markdown)..];
    while let Some(c) = rest.chars().next() {
        let skip = if rest.starts_with("](") {
            out.push(']');
            rest.find(')').map(|end| end + 1)
        } else if rest.starts_with("[^") {
            rest.find(']').map(|end| end + 1)
        } else if c == '<' && rest[1..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '/') {
            rest.find('>').map(|end| end + 1)
        } else {
            None
        };
        match skip {
            Some(length) => rest = &rest[length..],
            None => {
                out.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    out
}

/// A block's text in the old and new versions, and its hunk if it has one.
struct Paragraphs {
    hunk: Option<usize>,
    old: String,
    new: String,
}

/// The patch, with every block's text in the old and new versions.
fn build(critic: &str, options: &PatchOptions) -> (Patch, Vec<Paragraphs>) {
    build_with(critic, options, None)
}

/// [`build`], where a paragraph whose only marks are highlights and
/// comments in `known` (the old document's) is not a hunk.
fn build_with(
    critic: &str,
    options: &PatchOptions,
    known: Option<&HashSet<String>>,
) -> (Patch, Vec<Paragraphs>) {
    let critic = critic.replace("\r\n", "\n");
    let blocks = blocks(detach_breaks(attribute(segments(&critic))));
    let owner = &options.owner;
    let mut hunks = Vec::new();
    let mut texts = Vec::new();
    let (mut old_line, mut new_line) = (1, 1);
    for block in &blocks {
        old_line += newlines(&block.separator, block.old_before_visible);
        new_line += newlines(&block.separator, block.new_before_visible);
        let (old_text, new_text) = (block.text(Side::Old), block.text(Side::New));
        if block.marked(known) {
            let (at, removed) = if visible(&new_text) || !visible(&old_text) {
                (Locator::Line(new_line), false)
            } else {
                (Locator::Line(old_line), true)
            };
            texts.push(Paragraphs {
                hunk: Some(hunks.len()),
                old: old_text.clone(),
                new: new_text.clone(),
            });
            hunks.push(hunk(block, at, removed, owner));
        } else {
            texts.push(Paragraphs {
                hunk: None,
                old: old_text.clone(),
                new: new_text.clone(),
            });
        }
        old_line += old_text.matches('\n').count();
        new_line += new_text.matches('\n').count();
    }
    let patch = Patch {
        old_name: options.old_name.clone(),
        new_name: options.new_name.clone(),
        owner: owner.clone(),
        hunks,
    };
    (patch, texts)
}

fn newlines(separator: &str, counted: bool) -> usize {
    if counted {
        separator.matches('\n').count()
    } else {
        0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Old,
    New,
}

/// Deleted and inserted text: `plain` without the markup nested in it,
/// `marked` with it.
#[derive(Clone, Debug, Default)]
struct Edit {
    old: String,
    new: String,
    old_marked: String,
    new_marked: String,
    by: Option<(String, String)>,
}

impl Edit {
    fn push(&mut self, side: Side, text: &str, plain: bool) {
        let (plain_text, marked) = match side {
            Side::Old => (&mut self.old, &mut self.old_marked),
            Side::New => (&mut self.new, &mut self.new_marked),
        };
        if plain {
            plain_text.push_str(text);
        }
        marked.push_str(text);
    }
}

#[derive(Clone, Debug)]
enum Segment {
    Same(String),
    Edit(Edit),
    HighlightStart,
    HighlightEnd,
    /// A comment's text, as written.
    Note(String),
}

/// The document as text, changes, highlight delimiters and comments.
fn segments(critic: &str) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut edit: Option<Edit> = None;
    let mut side = Side::New;
    let mut note: Option<String> = None;
    // Comments that end inside a change, written after it.
    let mut held: Vec<String> = Vec::new();
    critic::Pieces::default().feed(&critic::encode_spans_only(critic), |piece| match piece {
        Piece::Text(text) => {
            if let Some(note) = &mut note {
                note.push_str(text);
            } else if let Some(edit) = &mut edit {
                edit.push(side, text, true);
            } else if let Some(Segment::Same(same)) = out.last_mut() {
                same.push_str(text);
            } else {
                out.push(Segment::Same(text.to_string()));
            }
        }
        Piece::Token(token) => match token {
            Token::InsertStart | Token::DeleteStart | Token::SubstituteStart => {
                edit = Some(Edit::default());
                side = if token == Token::InsertStart {
                    Side::New
                } else {
                    Side::Old
                };
            }
            Token::SubstituteSeparator => side = Side::New,
            Token::InsertEnd | Token::DeleteEnd | Token::SubstituteEnd => {
                if let Some(edit) = edit.take() {
                    out.push(Segment::Edit(edit));
                }
                out.extend(held.drain(..).map(Segment::Note));
            }
            Token::HighlightStart | Token::HighlightEnd => {
                let start = token == Token::HighlightStart;
                match &mut edit {
                    Some(edit) => edit.push(side, if start { "{==" } else { "==}" }, false),
                    None if start => out.push(Segment::HighlightStart),
                    None => out.push(Segment::HighlightEnd),
                }
            }
            Token::CommentStart => note = Some(String::new()),
            Token::CommentEnd => {
                let text = note.take().unwrap_or_default();
                if edit.is_some() {
                    held.push(text);
                } else {
                    out.push(Segment::Note(text));
                }
            }
        },
    });
    out
}

/// `Name (date)` in the comment right after a change, or after the
/// comments that follow it, becomes the change's author and date.
fn attribute(segments: Vec<Segment>) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::with_capacity(segments.len());
    for segment in segments {
        if let Segment::Note(text) = &segment
            && let Some(Segment::Edit(edit)) = out
                .iter_mut()
                .rev()
                .find(|s| !matches!(s, Segment::Note(_)))
            && edit.by.is_none()
            && let Some((Some(author), date, "")) = named(text.trim())
        {
            edit.by = Some((author, date.to_string()));
            continue;
        }
        out.push(segment);
    }
    out
}

/// Word deletes or inserts a paragraph with its mark, so the change ends
/// with the paragraph break (or, for the last paragraph, starts with the
/// one before it). That break moves out of the change, after its comments,
/// so the paragraph is a block of its own. A change that is only a break
/// joins or splits paragraphs and stays as it is.
fn detach_breaks(segments: Vec<Segment>) -> Vec<Segment> {
    fn same(out: &mut Vec<Segment>, text: &str) {
        match out.last_mut() {
            Some(Segment::Same(same)) => same.push_str(text),
            _ => out.push(Segment::Same(text.to_string())),
        }
    }
    let mut out: Vec<Segment> = Vec::with_capacity(segments.len());
    let mut after: Option<String> = None;
    for segment in segments {
        if !matches!(segment, Segment::Note(_))
            && let Some(text) = after.take()
        {
            same(&mut out, &text);
        }
        let Segment::Edit(mut edit) = segment else {
            match segment {
                Segment::Same(text) => same(&mut out, &text),
                other => out.push(other),
            }
            continue;
        };
        let side = match (edit.old.is_empty(), edit.new.is_empty()) {
            (false, true) => Side::Old,
            (true, false) => Side::New,
            _ => {
                out.push(Segment::Edit(edit));
                continue;
            }
        };
        let (plain, marked) = match side {
            Side::Old => (&mut edit.old, &mut edit.old_marked),
            Side::New => (&mut edit.new, &mut edit.new_marked),
        };
        let trailing = plain.len() - plain.trim_end_matches('\n').len();
        let leading = plain.len() - plain.trim_start_matches('\n').len();
        if plain.trim().is_empty() {
            out.push(Segment::Edit(edit));
        } else if trailing >= 2 && marked.ends_with(&plain[plain.len() - trailing..]) {
            let text = plain.split_off(plain.len() - trailing);
            marked.truncate(marked.len() - trailing);
            out.push(Segment::Edit(edit));
            after = Some(text);
        } else if leading >= 2 && marked.starts_with(&plain[..leading]) {
            let text: String = plain.drain(..leading).collect();
            marked.drain(..leading);
            same(&mut out, &text);
            out.push(Segment::Edit(edit));
        } else {
            out.push(Segment::Edit(edit));
        }
    }
    if let Some(text) = after {
        same(&mut out, &text);
    }
    out
}

/// A paragraph (or table row) of segments, with the line breaks before it.
struct Block {
    separator: String,
    /// The block before this one is in that version, so the separator is
    /// too.
    old_before_visible: bool,
    new_before_visible: bool,
    segments: Vec<Segment>,
}

impl Block {
    fn text(&self, side: Side) -> String {
        let mut out = String::new();
        for segment in &self.segments {
            match segment {
                Segment::Same(text) => out.push_str(text),
                Segment::Edit(edit) => out.push_str(match side {
                    Side::Old => &edit.old,
                    Side::New => &edit.new,
                }),
                _ => {}
            }
        }
        out
    }

    fn marked(&self, known: Option<&HashSet<String>>) -> bool {
        self.segments.iter().any(|s| match (s, known) {
            (Segment::Same(_), _) => false,
            (Segment::Edit(_), _) | (_, None) => true,
            (Segment::Note(note), Some(known)) => !known.contains(note),
            (_, Some(_)) => false,
        })
    }
}

/// The segments split into blocks at the line breaks between paragraphs,
/// outside changes: a change across a paragraph break keeps both
/// paragraphs in one block.
fn blocks(segments: Vec<Segment>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut current: Vec<Segment> = Vec::new();
    let mut separator = String::new();
    // The new version's text of the line being read.
    let mut line = String::new();
    let mut visible = (true, true);
    let mut close = |current: &mut Vec<Segment>, separator: &mut String, out: &mut Vec<Block>| {
        let block = Block {
            separator: std::mem::take(separator),
            old_before_visible: visible.0,
            new_before_visible: visible.1,
            segments: std::mem::take(current),
        };
        visible = (
            self::visible(&block.text(Side::Old)),
            self::visible(&block.text(Side::New)),
        );
        out.push(block);
    };
    for segment in segments {
        let Segment::Same(text) = segment else {
            if let Segment::Edit(edit) = &segment {
                match edit.new.rfind('\n') {
                    Some(at) => line = edit.new[at + 1..].to_string(),
                    None => line.push_str(&edit.new),
                }
            }
            current.push(segment);
            continue;
        };
        let mut rest = text.as_str();
        while let Some(at) = rest.find('\n') {
            let before = &rest[..at];
            if !before.is_empty() {
                push_same(&mut current, before);
            }
            line.push_str(before);
            let after = &rest[at + 1..];
            let next = after.split('\n').next().unwrap_or("");
            let boundary = next.trim().is_empty() || !open_paragraph(&line) || !continues(next);
            if boundary {
                if current.is_empty() {
                    separator.push('\n');
                } else {
                    close(&mut current, &mut separator, &mut out);
                    separator.push('\n');
                }
            } else if current.is_empty() {
                separator.push('\n');
            } else {
                push_same(&mut current, "\n");
            }
            line.clear();
            rest = after;
        }
        if !rest.is_empty() {
            push_same(&mut current, rest);
            line.push_str(rest);
        }
    }
    if !current.is_empty() {
        close(&mut current, &mut separator, &mut out);
    }
    out
}

/// Whether a block has text in a version: more than its list, quote or
/// heading marker.
fn visible(text: &str) -> bool {
    let text = text.trim_start();
    if table_row(text) {
        return text
            .chars()
            .any(|c| !matches!(c, '|' | '-' | ':') && !c.is_whitespace());
    }
    !text[marker_len(text)..].trim().is_empty()
}

fn push_same(segments: &mut Vec<Segment>, text: &str) {
    if let Some(Segment::Same(same)) = segments.last_mut() {
        same.push_str(text);
    } else {
        segments.push(Segment::Same(text.to_string()));
    }
}

/// A comment's author, date and text: `Name (date): text`, or the owner's.
fn comment(text: &str, owner: &Attribution, on: Option<String>) -> Comment {
    let trimmed = text.trim();
    if let Some((Some(author), date, rest)) = named(trimmed)
        && let Some(body) = rest.strip_prefix(':').or(rest.is_empty().then_some(""))
    {
        return Comment {
            author,
            date: date.to_string(),
            text: body.trim().to_string(),
            on,
        };
    }
    Comment {
        author: owner.author.clone(),
        date: owner.date.clone(),
        text: trimmed.to_string(),
        on,
    }
}

fn hunk(block: &Block, at: Locator, removed: bool, owner: &Attribution) -> Hunk {
    let mut text = String::new();
    let mut changes: Vec<Change> = Vec::new();
    let mut comments = Vec::new();
    // The highlighted text being read, and the last highlight's.
    let mut highlight: Option<String> = None;
    let mut highlighted: Option<String> = None;
    let mut previous: Option<&Segment> = None;
    for segment in &block.segments {
        match segment {
            Segment::Same(same) => {
                text.push_str(&escape(same));
                if let Some(h) = &mut highlight {
                    h.push_str(same);
                }
            }
            Segment::Edit(edit) => {
                if !edit.old_marked.is_empty() {
                    text.push_str(&format!("[-{}-]", escape(&edit.old_marked)));
                }
                if !edit.new_marked.is_empty() {
                    text.push_str(&format!("{{+{}+}}", escape(&edit.new_marked)));
                }
                let (author, date) = match &edit.by {
                    Some((author, date)) => {
                        if author != &owner.author || date != &owner.date {
                            text.push_str(&format!("{{>>{author} ({date})<<}}"));
                        }
                        (author.clone(), date.clone())
                    }
                    None => (owner.author.clone(), owner.date.clone()),
                };
                if let Some(h) = &mut highlight {
                    h.push_str(&edit.new);
                }
                changes.push(Change {
                    old: edit.old.clone(),
                    new: edit.new.clone(),
                    author,
                    date,
                    comment: None,
                });
            }
            Segment::HighlightStart => {
                text.push_str("{==");
                highlight = Some(String::new());
            }
            Segment::HighlightEnd => {
                text.push_str("==}");
                highlighted = highlight.take();
            }
            Segment::Note(note) => {
                let on = match previous {
                    Some(Segment::HighlightEnd) => highlighted.take(),
                    _ => None,
                };
                let comment = comment(note, owner, on);
                text.push_str(&format!(
                    "{{>>{} ({}): {}<<}}",
                    comment.author, comment.date, comment.text
                ));
                match (previous, changes.last_mut()) {
                    (Some(Segment::Edit(edit)), Some(change))
                        if change.comment.is_none()
                            && (edit.by.is_none()
                                || edit.by.as_ref().is_some_and(|(a, d)| {
                                    a == &owner.author && d == &owner.date
                                })) =>
                    {
                        change.comment = Some(comment);
                    }
                    _ => comments.push(comment),
                }
            }
        }
        previous = Some(segment);
    }
    Hunk {
        at,
        removed,
        text,
        changes,
        comments,
    }
}

/// The patch's own delimiters in text, escaped so they stay text.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut backslashes = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        match (c, next) {
            ('[', Some('-')) | ('{', Some('+')) if backslashes % 2 == 0 => out.push('\\'),
            _ => {}
        }
        out.push(c);
        match (c, next) {
            ('-', Some(']')) | ('+', Some('}')) => out.push('\\'),
            _ => {}
        }
        backslashes = if c == '\\' { backslashes + 1 } else { 0 };
    }
    out
}

/// `text` with each line broken at spaces to fit `columns`, where Markdown
/// reads the break as a space.
fn wrap(text: &str, columns: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in text.split('\n') {
        let trimmed = line.trim_start();
        let fence = trimmed.starts_with("```") || trimmed.starts_with("~~~");
        let keep = columns == 0
            || fenced
            || fence
            || trimmed.starts_with('#')
            || table_row(line)
            || line.chars().count() <= columns;
        if fence {
            fenced = !fenced;
        }
        if keep {
            out.push(line.to_string());
        } else {
            out.extend(break_line(line, columns).into_iter().map(str::to_string));
        }
    }
    out.join("\n")
}

fn break_line(line: &str, columns: usize) -> Vec<&str> {
    let protected = protected(line);
    let breaks: Vec<usize> = line
        .char_indices()
        .filter(|&(at, c)| c == ' ' && can_break(line, at, &protected))
        .map(|(at, _)| at)
        .collect();
    let mut out = Vec::new();
    let mut start = 0;
    loop {
        let width = |end: usize| line[start..end].chars().count();
        if width(line.len()) <= columns {
            break;
        }
        let candidates = breaks.iter().copied().filter(|&b| b > start);
        let fitting = candidates
            .clone()
            .take_while(|&b| width(b) <= columns)
            .last();
        let Some(at) = fitting.or_else(|| candidates.clone().next()) else {
            break;
        };
        out.push(&line[start..at]);
        start = at + 1;
    }
    out.push(&line[start..]);
    out
}

/// Byte ranges no break may fall in: code spans, link targets, autolinks.
fn protected(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b'`' => {
                let run = bytes[at..].iter().take_while(|b| **b == b'`').count();
                let fence = &line[at..at + run];
                let mut end = at + run;
                let mut closed = None;
                while let Some(found) = line[end..].find(fence) {
                    let start = end + found;
                    let length = bytes[start..].iter().take_while(|b| **b == b'`').count();
                    if length == run {
                        closed = Some(start + run);
                        break;
                    }
                    end = start + length;
                }
                match closed {
                    Some(close) => {
                        out.push((at, close));
                        at = close;
                    }
                    None => at += run,
                }
            }
            b']' if bytes.get(at + 1) == Some(&b'(') => {
                let mut depth = 0;
                let mut end = at + 1;
                while end < bytes.len() {
                    match bytes[end] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        b'\\' => end += 1,
                        _ => {}
                    }
                    end += 1;
                }
                out.push((at, end.min(bytes.len())));
                at = end + 1;
            }
            b'<' if line[at + 1..].starts_with("http") || line[at + 1..].starts_with("mailto:") => {
                let end = line[at..].find('>').map_or(bytes.len(), |e| at + e + 1);
                out.push((at, end));
                at = end;
            }
            _ => at += 1,
        }
    }
    out
}

/// The markers that open and close a deletion, insertion, highlight and
/// comment.
const OPENERS: [&str; 4] = ["[-", "{+", "{==", "{>>"];
const CLOSERS: [&str; 4] = ["-]", "+}", "==}", "<<}"];

/// Whether the space at byte `at` may become a line break.
fn can_break(line: &str, at: usize, protected: &[(usize, usize)]) -> bool {
    if protected.iter().any(|&(start, end)| start < at && at < end) {
        return false;
    }
    let before = &line[..at];
    let after = &line[at + 1..];
    if before.is_empty()
        || before.ends_with([' ', '\\'])
        || after.is_empty()
        || after.starts_with(' ')
    {
        return false;
    }
    // A change marker stays on the line of the text it marks: no line ends
    // with an opening marker or starts with a closing one (`{+paragraph` /
    // `+}style` read as two changes).
    if OPENERS.iter().any(|m| before.ends_with(m)) || CLOSERS.iter().any(|m| after.starts_with(m)) {
        return false;
    }
    !starts_block(after)
}

/// Whether a line starting with `text` would start a block of its own.
fn starts_block(text: &str) -> bool {
    let word = text.split(' ').next().unwrap_or("");
    marker_len(text) > 0
        || text.starts_with(['|', '>', '#', '<'])
        || text.starts_with("```")
        || text.starts_with("~~~")
        || (!word.is_empty() && word.chars().all(|c| c == '=' || c == '-'))
        || (word.len() > 1 && word.chars().all(|c| c == '*' || c == '_'))
        || (word.ends_with(['.', ')'])
            && word[..word.len() - 1].bytes().all(|b| b.is_ascii_digit())
            && word.len() > 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wrap_never_strands_a_change_marker() {
        // The heading pair wrapped as `{+paragraph` / `+}style`: the break
        // was the inserted space itself.
        let line = "**This document [-shows-]{+demonstrates+} Heading 1 {+paragraph +}style[- with extra bold emphasis-].** {==see==}{>>note <<} end";
        for columns in [20, 40, 56, 60, 72] {
            let wrapped = wrap(line, columns);
            assert_eq!(wrapped.replace('\n', " "), line, "{columns}");
            for row in wrapped.lines() {
                for close in ["+}", "-]", "==}", "<<}"] {
                    assert!(
                        !row.starts_with(close),
                        "{columns}: {row:?} starts with {close}"
                    );
                }
                for open in ["{+", "[-", "{==", "{>>"] {
                    assert!(!row.ends_with(open), "{columns}: {row:?} ends with {open}");
                }
            }
        }
    }

    #[test]
    fn escapes_only_unescaped_delimiters() {
        assert_eq!(escape("a [-b-] {+c+}"), "a \\[-b-\\] \\{+c+\\}");
        assert_eq!(escape("\\[-b"), "\\[-b");
        assert_eq!(escape("x-y [z]"), "x-y [z]");
    }

    #[test]
    fn protects_code_links_and_autolinks() {
        let line = "a `b c` [d](e f) <http://g h>";
        let ranges = protected(line);
        assert_eq!(ranges.len(), 3, "{ranges:?}");
        assert!(!can_break(line, line.find(" c`").unwrap(), &ranges));
        assert!(!can_break(line, line.find(" f)").unwrap(), &ranges));
        assert!(can_break(line, 1, &ranges));
    }

    #[test]
    fn plain_text_drops_markers_targets_note_references_and_tags() {
        assert_eq!(plain("- a<u>b</u>c [d](e)[^1] x < y"), "abc [d] x < y");
    }

    #[test]
    fn no_line_starts_a_block() {
        for text in [
            "- x", "* x", "+ x", "1. x", "12) x", "# x", "> x", "| x", "=== x", "--- x", "```",
            "<div>",
        ] {
            assert!(starts_block(text), "{text}");
        }
        for text in ["x", "1.5 x", "-5 x", "word"] {
            assert!(!starts_block(text), "{text}");
        }
    }
}
