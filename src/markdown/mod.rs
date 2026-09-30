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
mod from_docx;
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

/// How [`docx_to_markdown`] reads a document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarkdownOptions {
    /// `All` writes tracked changes and comments as CriticMarkup; `Accept`
    /// and `Reject` write the text after Word's Accept All or Reject All,
    /// without comments.
    pub track_changes: TrackChanges,
    /// Collect the document's raster pictures and name them in the Markdown
    /// under this directory, as pandoc's `--extract-media`. `None` writes a
    /// picture as its alt text.
    pub extract_media: Option<String>,
}

/// A document read as Markdown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadDocx {
    /// The Markdown, with CriticMarkup when changes are kept.
    pub markdown: String,
    /// Pictures to write under [`MarkdownOptions::extract_media`]: file
    /// name and bytes, sorted by name.
    pub media: Vec<(String, Vec<u8>)>,
}

/// A `.docx` as Markdown: headings, emphasis, links, lists, tables,
/// footnotes and equations (as LaTeX), with tracked changes and comments as
/// CriticMarkup (`{++new++}`, `{--old--}`, `{==text==}{>>note<<}`), each
/// change followed by its author and date.
///
/// ```
/// use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};
///
/// let docx = markdown_to_docx("Due in {~~30~>45~~} days.", &DocxOptions::default())?.docx;
/// let read = docx_to_markdown(&docx, &MarkdownOptions::default())?;
/// assert!(read.markdown.starts_with("Due in {~~30~>45~~}"), "{}", read.markdown);
/// # Ok::<(), jubarte::markdown::MarkdownError>(())
/// ```
pub fn docx_to_markdown(docx: &[u8], options: &MarkdownOptions) -> Result<ReadDocx, MarkdownError> {
    let revisions = match options.track_changes {
        TrackChanges::All => from_docx::Revisions::Markup,
        TrackChanges::Accept => from_docx::Revisions::Accept,
        TrackChanges::Reject => from_docx::Revisions::Reject,
    };
    let converted = from_docx::convert(
        docx,
        &from_docx::Options {
            revisions,
            media_dir: options.extract_media.clone(),
        },
    )
    .map_err(|error| MarkdownError::Docx(error.to_string()))?;
    Ok(ReadDocx {
        markdown: converted.markdown,
        media: converted.media.into_iter().collect(),
    })
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
    /// The document to read is not a readable `.docx`.
    Docx(String),
}

impl std::fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(message) => write!(f, "reference document: {message}"),
            Self::Package(message) => write!(f, "cannot write the document: {message}"),
            Self::Docx(message) => write!(f, "cannot read the document: {message}"),
        }
    }
}

impl std::error::Error for MarkdownError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_changes_takes_pandocs_values() {
        assert_eq!(TrackChanges::parse("all"), Some(TrackChanges::All));
        assert_eq!(TrackChanges::parse("accept"), Some(TrackChanges::Accept));
        assert_eq!(TrackChanges::parse("reject"), Some(TrackChanges::Reject));
        assert_eq!(TrackChanges::parse("markup"), None);
        assert_eq!(TrackChanges::default(), TrackChanges::All);
    }

    #[test]
    fn resolve_critic_all_is_the_markdown_as_it_is() {
        let text = "a {++b++} c";
        assert_eq!(resolve_critic(text, TrackChanges::All), text);
    }

    #[test]
    fn errors_and_options_describe_themselves() {
        assert_eq!(
            MarkdownError::Reference("not a zip".into()).to_string(),
            "reference document: not a zip"
        );
        assert_eq!(
            MarkdownError::Package("full".into()).to_string(),
            "cannot write the document: full"
        );
        let loader = |_: &str| None;
        let reference = [0u8; 3];
        let options = DocxOptions {
            reference: Some(&reference),
            images: Some(&loader),
            ..DocxOptions::default()
        };
        let debug = format!("{options:?}");
        assert!(
            debug.contains("reference: Some(3)") && debug.contains("images: true"),
            "{debug}"
        );
        let debug = format!("{:?}", RedlineOptions::default());
        assert!(
            debug.contains("critic: false") && debug.contains("images: false"),
            "{debug}"
        );
    }
}
