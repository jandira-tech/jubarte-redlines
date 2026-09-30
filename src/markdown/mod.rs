// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markdown to Word.
//!
//! [`markdown_to_docx`] writes CommonMark, with GitHub's tables,
//! strikethrough, task lists and footnotes, as a `.docx`. CriticMarkup in the
//! Markdown becomes Word tracked changes and comments:
//!
//! | CriticMarkup | Word |
//! | --- | --- |
//! | `{++new++}` | inserted text (`w:ins`) |
//! | `{--old--}` | deleted text (`w:del`) |
//! | `{~~old~>new~~}` | deleted, then inserted text |
//! | `{==text==}{>>note<<}` | a comment on `text` |
//! | `{>>note<<}` after a change | a comment on the change |
//! | `{==text==}` alone | highlighted text |
//!
//! A change can span paragraphs: the paragraph marks inside it are inserted
//! or deleted too, so `A{++\n\nB++}` adds paragraph `B` after `A`. A block
//! whose whole text is one change (`{++New paragraph.++}`) is inserted or
//! deleted with its paragraph mark, as Word records a paragraph added or
//! removed whole.
//!
//! Styles use Word's built-in ids (`Normal`, `Heading1`..`Heading6`,
//! `Quote`, `ListParagraph`, `FootnoteText`, `Hyperlink`, `TableGrid`...)
//! and pandoc's for code (`SourceCode`, `VerbatimChar`), so a reference
//! document made by Word or by pandoc styles the output
//! ([`DocxOptions::reference`]).

mod critic;
mod diff;
mod package;
mod patch;
mod redline;
mod write;
mod xml;

pub use diff::diff_markdown;
pub use patch::{Patched, apply_markdown};
pub use redline::{RedlineOptions, Source, redline};
pub use write::markdown_to_docx;

/// What happens to the tracked changes a document describes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TrackChanges {
    /// Keep them as tracked changes (the default).
    #[default]
    All,
    /// Accept every change, as Word's Accept All does.
    Accept,
    /// Reject every change, as Word's Reject All does.
    Reject,
}

impl TrackChanges {
    /// `all`, `accept` or `reject`, the values of pandoc's `--track-changes`.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "accept" => Some(Self::Accept),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }
}

/// The Markdown with its CriticMarkup resolved: `Accept` keeps insertions
/// and drops deletions, `Reject` the other way round, and both keep
/// highlighted text and drop comments. `All` returns the Markdown as it is.
///
/// ```
/// use jubarte::markdown::{TrackChanges, resolve_critic};
///
/// let text = "Due in {~~30~>45~~} days.{>>Agreed on the call.<<}";
/// assert_eq!(resolve_critic(text, TrackChanges::Accept), "Due in 45 days.");
/// assert_eq!(resolve_critic(text, TrackChanges::Reject), "Due in 30 days.");
/// ```
pub fn resolve_critic(markdown: &str, track_changes: TrackChanges) -> String {
    match track_changes {
        TrackChanges::All => markdown.to_string(),
        TrackChanges::Accept => critic::resolve(markdown, true),
        TrackChanges::Reject => critic::resolve(markdown, false),
    }
}

/// Reads an image a Markdown document names, by the path it names.
pub type ImageLoader<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

/// How [`markdown_to_docx`] writes a document.
#[derive(Clone)]
pub struct DocxOptions<'a> {
    /// A `.docx` whose styles, numbering, page setup, headers and footers
    /// the output takes, as pandoc's `--reference-doc`; its text is not
    /// used. `None` uses built-in styles on a US Letter page.
    pub reference: Option<&'a [u8]>,
    /// Read CriticMarkup as tracked changes and comments. Off, its
    /// delimiters are text.
    pub critic: bool,
    /// Keep, accept or reject the tracked changes CriticMarkup describes.
    pub track_changes: TrackChanges,
    /// Author of the tracked changes and comments.
    pub author: String,
    /// Their date, `YYYY-MM-DDTHH:MM:SSZ`; fixed by default so the same
    /// Markdown writes the same bytes.
    pub date: String,
    /// The bytes of each image the Markdown names. Without it, or when it
    /// returns `None`, an image is written as its alt text.
    pub images: Option<ImageLoader<'a>>,
}

impl Default for DocxOptions<'_> {
    fn default() -> Self {
        Self {
            reference: None,
            critic: true,
            track_changes: TrackChanges::All,
            author: "Redline".to_string(),
            date: "1970-01-01T00:00:00Z".to_string(),
            images: None,
        }
    }
}

impl std::fmt::Debug for DocxOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocxOptions")
            .field("reference", &self.reference.map(<[u8]>::len))
            .field("critic", &self.critic)
            .field("track_changes", &self.track_changes)
            .field("author", &self.author)
            .field("date", &self.date)
            .field("images", &self.images.is_some())
            .finish()
    }
}

/// A written document and what could not be written as asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenDocx {
    /// The `.docx` bytes.
    pub docx: Vec<u8>,
    /// Images written as alt text, and the like.
    pub warnings: Vec<String>,
}

/// Why a document could not be written.
#[derive(Debug)]
pub enum MarkdownError {
    /// The reference document is not a readable `.docx`.
    Reference(String),
    /// The package could not be assembled or accepted/rejected.
    Package(String),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(message) => write!(f, "reference document: {message}"),
            Self::Package(message) => write!(f, "cannot write the document: {message}"),
        }
    }
}

impl std::error::Error for MarkdownError {}
