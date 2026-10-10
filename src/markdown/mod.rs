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

mod anchor;
mod critic;
mod diff;
mod from_docx;
pub(crate) mod package;
mod pages;
mod patch;
mod redline;
mod unified;
mod write;
pub(crate) mod xml;

pub use anchor::{plain_anchor, unescape_markdown};
pub use diff::diff_markdown;
pub use pages::paginate;
pub use patch::{Patched, apply_markdown};
pub use redline::{RedlineOptions, Source, redline};
pub use unified::{
    Attribution, Change, Comment, DEFAULT_COLUMNS, Hunk, Locator, Patch, PatchOptions,
    patch_critic, patch_documents, patch_markdown, patch_own_changes, patch_redline,
};
pub use write::markdown_to_docx;

pub(crate) use package::{ensure_footnotes_part, max_drawing_id};
pub(crate) use write::read_picture;
pub(crate) use xml::{Picture, drawing_xml};

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

/// The page a Markdown document is written on when no reference document
/// gives one. Both sizes keep one-inch margins: Word's own A4 template uses
/// 2 cm margins, but jubarte changes only the page size the user asks for.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum PageSize {
    /// US Letter, 8.5 by 11 inches (the default, as Word's US template).
    #[default]
    Letter,
    /// ISO A4, 210 by 297 mm.
    A4,
}

impl PageSize {
    /// `letter` or `a4`, the values of the CLI's `--page`.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "letter" => Some(Self::Letter),
            "a4" => Some(Self::A4),
            _ => None,
        }
    }

    /// The name [`PageSize::parse`] reads.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Letter => "letter",
            Self::A4 => "a4",
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

/// Accepted text snapshots remove wholly deleted clause lines.
pub(crate) fn accepted_clauses(markdown: &str) -> String {
    critic::accept_clauses(markdown)
}

/// Reads an image a Markdown document names, by the path it names.
pub type ImageLoader<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

/// How [`markdown_to_docx`] writes a document.
#[derive(Clone)]
pub struct DocxOptions<'a> {
    /// A `.docx` whose styles, numbering, page setup, headers and footers
    /// the output takes, as pandoc's `--reference-doc`; its text is not
    /// used. `None` uses built-in styles on the page [`DocxOptions::page`]
    /// names.
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
    /// The page size when there is no reference document. With a reference,
    /// its page setup wins and a page other than the default is reported in
    /// [`WrittenDocx::warnings`].
    pub page: PageSize,
}

impl Default for DocxOptions<'_> {
    fn default() -> Self {
        Self {
            reference: None,
            critic: true,
            track_changes: TrackChanges::All,
            author: "Redline".to_string(),
            date: crate::document_comparer::DEFAULT_DATE.to_string(),
            images: None,
            page: PageSize::Letter,
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
            .field("page", &self.page)
            .finish()
    }
}

/// How [`docx_to_markdown`] reads a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownOptions {
    /// `All` writes tracked changes and comments as CriticMarkup; `Accept`
    /// and `Reject` write the text after Word's Accept All or Reject All,
    /// without comments.
    pub track_changes: TrackChanges,
    /// Collect the document's raster pictures and name them in the Markdown
    /// under this directory, as pandoc's `--extract-media`. `None` writes a
    /// picture as its alt text.
    pub extract_media: Option<String>,
    /// Agent view: a YAML header, an id line before every block, Word's
    /// ids on revisions and comments. Off, the plain conversion, unchanged.
    pub ids: bool,
    /// With `ids`: comments inline (`true`), or hidden with their ids on the
    /// id line of the paragraph that holds them.
    pub comments: bool,
    /// With `ids`: the name printed as `source:` in the header; `None`
    /// prints `(bytes)`.
    pub source: Option<String>,
    /// With `ids`: the text painted on each page by the layout pass
    /// (`convert::RenderReport::pages`), for `<!-- page N of M -->` lines.
    /// `None` falls back to Word's cached page breaks.
    pub pages: Option<Vec<String>>,
    /// With `ids` and no `pages`: write `<!-- page N of M -->` lines from
    /// Word's cached breaks (`true`, the default), or no page lines at all;
    /// the header's page count is unaffected.
    pub page_markers: bool,
    /// With `ids`: timestamps inline on the notes of an author whose marks
    /// and comments do not all share one timestamp.
    pub dates: bool,
    /// With `ids`: which blocks of the body to print.
    pub select: Option<Select>,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            track_changes: TrackChanges::All,
            extract_media: None,
            ids: false,
            comments: true,
            source: None,
            pages: None,
            page_markers: true,
            dates: false,
            select: None,
        }
    }
}

/// Which blocks of the agent view to print (`-p`, `--head`, `--tail`,
/// `--changed`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Select {
    /// The first `n` blocks (a table is one block).
    Head(usize),
    /// The last `n` blocks.
    Tail(usize),
    /// Paragraphs and tables by id, in document order.
    Picks(Vec<Pick>),
    /// The blocks that carry a tracked change or a comment; with `by`, only
    /// those with that author's marks (`by` is a handle such as `AC`, with
    /// or without the `@`, or the author's full name).
    Changed {
        /// The author whose marks a block must hold.
        by: Option<String>,
    },
}

/// One item of a `-p` selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// `pN`, `pN-pM`, `pN-` (to the end, `to: None`), `-pM` (`from: 0`).
    Paragraphs {
        /// The first paragraph of the pick.
        from: usize,
        /// The last paragraph of the pick; `None` runs to the end.
        to: Option<usize>,
    },
    /// `tN`: the whole table.
    Table(usize),
}

impl Select {
    /// The selection of `read`'s `-p`, `--head` and `--tail` (in that order
    /// of precedence), or of `--changed` with its `--by`; `None` when none is
    /// given. `--changed` excludes the other three, and `--by` needs it.
    pub fn from_flags(
        paragraphs: Option<&str>,
        head: Option<usize>,
        tail: Option<usize>,
        changed: bool,
        by: Option<&str>,
    ) -> Result<Option<Self>, String> {
        if by.is_some() && !changed {
            return Err("by needs changed".to_string());
        }
        if changed && (paragraphs.is_some() || head.is_some() || tail.is_some()) {
            return Err("changed excludes paragraphs, head and tail".to_string());
        }
        Ok(match (paragraphs, head, tail) {
            _ if changed => Some(Self::Changed {
                by: by.map(str::to_string),
            }),
            (Some(spec), _, _) => Some(Self::parse(spec)?),
            (None, Some(n), _) => Some(Self::Head(n)),
            (None, None, Some(n)) => Some(Self::Tail(n)),
            (None, None, None) => None,
        })
    }

    /// `p2, p5-p7, 12, p17-, -p1, t0`: comma-separated picks; a bare number
    /// is a paragraph.
    pub fn parse(spec: &str) -> Result<Self, String> {
        fn number(item: &str, text: &str) -> Result<usize, String> {
            let text = text.trim();
            text.strip_prefix('p')
                .unwrap_or(text)
                .parse()
                .map_err(|_| format!("{item}: expected pN, pN-pM or tN"))
        }
        let mut picks = Vec::new();
        for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(table) = item.strip_prefix('t') {
                let n = table
                    .parse()
                    .map_err(|_| format!("{item}: expected pN, pN-pM or tN"))?;
                picks.push(Pick::Table(n));
                continue;
            }
            if !item.contains(|c: char| c.is_ascii_digit()) {
                return Err(format!("{item}: expected pN, pN-pM or tN"));
            }
            let (from, to) = match item.split_once('-') {
                None => {
                    let n = number(item, item)?;
                    (n, Some(n))
                }
                Some((a, b)) => (
                    if a.trim().is_empty() {
                        0
                    } else {
                        number(item, a)?
                    },
                    if b.trim().is_empty() {
                        None
                    } else {
                        Some(number(item, b)?)
                    },
                ),
            };
            if to.is_some_and(|to| to < from) {
                return Err(format!("{item}: the range runs backwards"));
            }
            picks.push(Pick::Paragraphs { from, to });
        }
        if picks.is_empty() {
            return Err("no paragraphs selected".to_string());
        }
        Ok(Self::Picks(picks))
    }
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
    // A `.doc`, an encrypted document or RTF is named for what it is
    // (`LEGACY_DOC`, `UNSUPPORTED_PACKAGE`), not reported as a bad ZIP.
    crate::admission::sniff(docx).map_err(|refused| MarkdownError::Docx(refused.to_string()))?;
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
            ids: options.ids,
            comments: options.comments,
            source: options.source.clone(),
            pages: options.pages.clone(),
            page_markers: options.page_markers,
            dates: options.dates,
            select: options.select.clone(),
        },
    )
    .map_err(|error| MarkdownError::Docx(error.to_string()))?;
    Ok(ReadDocx {
        markdown: converted.markdown,
        media: converted.media.into_iter().collect(),
    })
}

/// How [`read`] prints the agent view (`jubarte read`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadOptions {
    /// Keep the changes as CriticMarkup (`All`), or print the document
    /// after Word's Accept All or Reject All with `rev` clauses.
    pub track_changes: TrackChanges,
    /// Comments inline (`true`), or hidden with their ids on the id lines.
    pub comments: bool,
    /// Timestamps inline on the notes of an author with several.
    pub dates: bool,
    /// `<!-- page N of M -->` lines from the layout pass (`true`), or none.
    pub page_markers: bool,
    /// Which blocks of the body to print.
    pub select: Option<Select>,
    /// The name printed as `source:` in the header.
    pub source: Option<String>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            track_changes: TrackChanges::All,
            comments: true,
            dates: false,
            page_markers: true,
            select: None,
            source: None,
        }
    }
}

/// The agent view of a document and what it could not show.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadView {
    /// The YAML header and the Markdown with id lines.
    pub markdown: String,
    /// Why the page markers are missing (the layout pass failed), and the
    /// like. The view is complete otherwise.
    pub warnings: Vec<String>,
}

/// The agent view every surface prints for `read` (alias `text`): a YAML
/// header, then Markdown with an `<!-- pN -->` id line before every
/// paragraph, tracked changes and comments with their ids, and page markers
/// from the layout pass. See `docs/MARKDOWN.md`, "Agent view".
///
/// ```
/// use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
///
/// let docx = markdown_to_docx("Due in {~~30~>45~~} days.", &DocxOptions::default())?.docx;
/// let view = read(&docx, &ReadOptions { page_markers: false, ..ReadOptions::default() })?;
/// assert!(view.markdown.contains("<!-- p0 -->\nDue in {~~30~>45~~}{>>#"), "{}", view.markdown);
/// # Ok::<(), jubarte::markdown::MarkdownError>(())
/// ```
pub fn read(docx: &[u8], options: &ReadOptions) -> Result<ReadView, MarkdownError> {
    let mut warnings = Vec::new();
    let pages = if options.page_markers {
        page_texts(
            docx,
            options.track_changes,
            crate::convert::RevisionStyle::default(),
        )
        .map_err(|e| warnings.push(format!("no page markers: {e}")))
        .ok()
    } else {
        None
    };
    let read = docx_to_markdown(
        docx,
        &MarkdownOptions {
            track_changes: options.track_changes,
            extract_media: None,
            ids: true,
            comments: options.comments,
            source: options.source.clone(),
            pages,
            page_markers: options.page_markers,
            dates: options.dates,
            select: options.select.clone(),
        },
    )?;
    Ok(ReadView {
        markdown: read.markdown,
        warnings,
    })
}

/// The text the layout pass paints on each page, with the document's
/// changes kept, accepted or rejected: what [`paginate`] matches blocks
/// against for `<!-- page N of M -->` lines.
pub fn page_texts(
    docx: &[u8],
    track_changes: TrackChanges,
    revisions: crate::convert::RevisionStyle,
) -> Result<Vec<String>, String> {
    let resolved = match track_changes {
        TrackChanges::All => Ok(std::borrow::Cow::Borrowed(docx)),
        TrackChanges::Accept => crate::document_comparer::accept_revisions(docx)
            .map(std::borrow::Cow::Owned)
            .map_err(|e| format!("accepting the changes failed: {e:?}")),
        TrackChanges::Reject => crate::document_comparer::reject_revisions(docx)
            .map(std::borrow::Cow::Owned)
            .map_err(|e| format!("rejecting the changes failed: {e:?}")),
    };
    let rendered = resolved.and_then(|bytes| {
        crate::convert::render(
            &bytes,
            crate::convert::PdfOptions {
                revisions,
                ..crate::convert::PdfOptions::default()
            },
            crate::convert::RenderRequest::default(),
        )
        .map_err(|e| format!("layout failed: {e}"))
    });
    rendered.map(|rendered| rendered.report.pages.into_iter().map(|p| p.text).collect())
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
