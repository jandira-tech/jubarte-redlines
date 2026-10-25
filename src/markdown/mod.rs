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
//!
//! Backs `jubarte convert FILE.md`, `jubarte convert FILE.docx -t md`,
//! `jubarte read`, the redline task's Markdown side
//! ([`crate::markdown::RedlineOptions`], [`crate::markdown::redline`]) and
//! the patch task ([`crate::markdown::Patch`]), plus the Python and WASM
//! bindings over them.

mod anchor;
pub(crate) mod borders;
mod critic;
mod diff;
mod from_docx;
mod math;
pub(crate) mod package;
mod pages;
mod patch;
mod redline;
mod unified;
mod view_notes;
mod view_stories;
mod view_styles;
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

/// What happens to the tracked changes a Markdown / `.docx` document
/// describes. Backs the `convert`, `read`, and `diff` CLI flags of the
/// same names and pandoc's `--track-changes`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TrackChanges {
    /// Keep tracked changes as tracked changes (the default).
    #[default]
    All,
    /// Accept every change, as Word's Accept All does.
    Accept,
    /// Reject every change, as Word's Reject All does.
    Reject,
}

impl TrackChanges {
    /// Parses `"all"`, `"accept"` or `"reject"`, the values of pandoc's
    /// `--track-changes`. Returns `None` for any other input.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "accept" => Some(Self::Accept),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }
}

/// The page a Markdown document is written on when no reference
/// document gives one. Backs the `convert --page` CLI flag and the
/// Markdown path of the redline. Both sizes use one-inch margins;
/// Word's own A4 template uses 2 cm margins, but jubarte changes only
/// the page size the user asks for.
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
    /// Parses `"letter"` or `"a4"`, the values of the CLI's `--page`.
    /// Returns `None` for any other input.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "letter" => Some(Self::Letter),
            "a4" => Some(Self::A4),
            _ => None,
        }
    }

    /// Returns the name [`Self::parse`] reads (`"letter"` or `"a4"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Letter => "letter",
            Self::A4 => "a4",
        }
    }
}

/// Resolves the CriticMarkup in `markdown` as `track_changes` says:
/// `Accept` keeps insertions and drops deletions, `Reject` the other way
/// round; both keep highlighted text (`{==…==}`) and drop comments
/// (`{>>…<<}`). `All` returns the Markdown as it is.
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

/// Callback that, given the path an image in a Markdown document
/// names, returns the image bytes. Backs `convert --resource-path`
/// and the Python / WASM image-loader binding; `None` falls back to the
/// image's alt text.
pub type ImageLoader<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

/// How [`markdown_to_docx`] writes a Markdown document to a `.docx`.
/// Defaults to Word-faithful settings: no reference document,
/// `critic = true` (CriticMarkup is read as tracked changes),
/// `track_changes = All`, author `"Redline"`, date pinned to
/// `1970-01-01T00:00:00Z` (so the same Markdown writes the same bytes),
/// no image loader, US Letter page size.
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
    /// `author` names the changes and comments that name no one, even when
    /// the front matter has an `author:` (an `--author` the user typed).
    /// `false`: the front matter's wins over `author`.
    pub author_fixed: bool,
    /// `date` dates the undated changes and comments, even when the front
    /// matter has a date (`last_modified:`, `modified:`, `date:`,
    /// `created:`). `false`: the front matter's wins over `date`.
    pub date_fixed: bool,
    /// The bytes of each image the Markdown names. Without it, or when it
    /// returns `None`, an image is written as its alt text.
    pub images: Option<ImageLoader<'a>>,
    /// The page size when there is no reference document. With a reference,
    /// its page setup wins and a page other than the default is reported in
    /// [`WrittenDocx::warnings`].
    pub page: PageSize,
    /// Word's Track Changes switch (`w:trackRevisions`) in the output:
    /// `Some` sets it whatever the Markdown's `track_changes:` key says;
    /// `None` follows the key, and without one keeps the reference's
    /// setting (off without a reference).
    pub tracking: Option<bool>,
    /// Set the document as LaTeX's article class sets a manuscript: Latin
    /// Modern Roman at 10pt on 12pt lines, justified, bold headings, and
    /// the page number centered in the footer from page 2. Only what the
    /// front matter leaves out: its own `styles:` entries, headers and
    /// footers win.
    pub scholar: bool,
}

impl Default for DocxOptions<'_> {
    fn default() -> Self {
        Self {
            reference: None,
            critic: true,
            track_changes: TrackChanges::All,
            author: "Redline".to_string(),
            date: crate::document_comparer::DEFAULT_DATE.to_string(),
            author_fixed: false,
            date_fixed: false,
            images: None,
            page: PageSize::Letter,
            tracking: None,
            scholar: false,
        }
    }
}

/// [`DocxOptions`] as the Python and WebAssembly bindings pass them (JSON,
/// every field optional).
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DocxOptionsJson {
    #[serde(default)]
    page: PageSize,
    author: Option<String>,
    date: Option<String>,
    critic: Option<bool>,
    #[serde(alias = "trackChanges")]
    track_changes: Option<String>,
    tracking: Option<bool>,
    #[serde(default)]
    scholar: bool,
}

/// [`DocxOptions`] from the JSON the Python and WebAssembly bindings pass:
/// `page`, `author`, `date`, `critic`, `track_changes` (`trackChanges`),
/// `tracking` and `scholar`, each optional. An `author` or `date` given
/// wins over the front matter's, as the CLI's flags do. Images are written
/// as their alt text.
///
/// # Errors
///
/// A field the options do not have, or a value they do not take.
pub fn docx_options_from_json<'a>(
    json: &str,
    reference: Option<&'a [u8]>,
) -> Result<DocxOptions<'a>, String> {
    let parsed: DocxOptionsJson = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let track_changes = match parsed.track_changes.as_deref() {
        None => TrackChanges::All,
        Some(value) => TrackChanges::parse(value)
            .ok_or_else(|| format!("track_changes must be all, accept or reject, not {value:?}"))?,
    };
    let defaults = DocxOptions::default();
    Ok(DocxOptions {
        reference,
        critic: parsed.critic.unwrap_or(defaults.critic),
        track_changes,
        author_fixed: parsed.author.is_some(),
        author: parsed.author.unwrap_or(defaults.author),
        date_fixed: parsed.date.is_some(),
        date: parsed.date.unwrap_or(defaults.date),
        page: parsed.page,
        tracking: parsed.tracking,
        scholar: parsed.scholar,
        ..defaults
    })
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
            .field("tracking", &self.tracking)
            .field("scholar", &self.scholar)
            .finish()
    }
}

/// How [`docx_to_markdown`] reads a `.docx`. Defaults to the plain
/// Markdown conversion with all revisions kept; `<!-- page N of M -->`
/// lines from Word's cached breaks come only with `ids` (the agent view).
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

/// Which blocks of the agent view to print. Backs the `read` and
/// `edit` CLI flags `-p`, `--head`, `--tail`, and `--changed` (with
/// `--by`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Select {
    /// The first `n` blocks (`--head N`); a table is one block.
    Head(usize),
    /// The last `n` blocks (`--tail N`).
    Tail(usize),
    /// Paragraphs and tables by id, in document order (`-p`).
    Picks(Vec<Pick>),
    /// The blocks that carry a tracked change or a comment (`--changed`);
    /// with `by`, only those with that author's marks (`by` is a handle
    /// such as `AC`, with or without the `@`, or the author's full name).
    Changed {
        /// The author whose marks a block must hold.
        by: Option<String>,
    },
}

/// One item of a `-p` selection: a contiguous paragraph range (`pN`,
/// `pN-pM`, `pN-`, `-pM`) or a whole table (`tN`).
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

/// Validates the author `--changed --by` names: a handle such as `AC` or
/// `@AC` (one `@`, the handle right after it), or the full name. The CLI
/// and the Python / WASM bindings route their check through here.
///
/// Returns the original (untrimmed outer whitespace removed) handle on
/// success; an error string otherwise.
pub fn author(by: &str) -> Result<&str, String> {
    let by = by.trim();
    let name = by.strip_prefix('@').unwrap_or(by);
    if name.is_empty() || name.starts_with('@') || name.starts_with(char::is_whitespace) {
        Err("needs an author: a handle such as AC, or the full name".into())
    } else {
        Ok(by)
    }
}

impl Select {
    /// Selects the right [`Select`] variant from `read`'s CLI flags:
    /// `-p` (`paragraphs`), `--head`, `--tail`, `--changed` with
    /// `--by`, in that order of precedence. Returns `Ok(None)` when none
    /// is given.
    ///
    /// # Errors
    ///
    /// An error string when:
    /// - `--by` is set without `--changed`;
    /// - `--changed` is set with any of `-p` / `--head` / `--tail`;
    /// - the `--by` value does not parse as an author
    ///   ([`author`]);
    /// - the `-p` spec does not parse as a paragraph / table selection
    ///   ([`Self::parse`]).
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
        let by = by.map(author).transpose()?;
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

    /// `p2, p5-p7, p17-, -p1, t0`: comma-separated picks. A pick is a body
    /// paragraph or a table, as the settled short ids spell them
    /// ([`crate::ids::ShortId`]), or a range of paragraphs, which may be
    /// open at one end (`p17-` to the end, `-p1` from the start). An expired
    /// spelling (`12`) is refused with its settled one. The open ends are
    /// transitional: the 17 October 2026 contract retires them (closed
    /// `pN-pM` ranges only); commas stay option-level separators.
    pub fn parse(spec: &str) -> Result<Self, String> {
        use crate::ids::{ShortId, Step, Story};
        // The refusal of `item`, naming the settled spelling when known.
        let refused = |item: &str, hint: Option<String>| {
            let hint = hint.map_or_else(String::new, |h| format!("; the settled spelling is {h}"));
            format!("{item}: {SELECT_EXPECTED}{hint}")
        };
        // The body path `text` names.
        let body = |item: &str, text: &str| match text.trim().parse::<ShortId>() {
            Ok(ShortId::Block {
                story: Story::Body,
                path,
            }) => Ok(path),
            Ok(_) => Err(refused(item, None)),
            Err(e) => Err(refused(
                item,
                crate::ids::hint_where(e.hint, |id| {
                    matches!(
                        id,
                        ShortId::Block {
                            story: Story::Body,
                            ..
                        }
                    )
                }),
            )),
        };
        // The body paragraph an end of a range names; `None` when open.
        let end = |item: &str, text: &str| {
            if text.trim().is_empty() {
                return Ok(None);
            }
            match body(item, text)?[..] {
                [Step::Paragraph(n)] => Ok(Some(n)),
                _ => Err(refused(item, None)),
            }
        };
        // The settled spelling of a whole range whose ends are expired
        // (`2-3` is `p2-p3`): `None` when an end has none, or the range
        // would run backwards.
        let settled_range = |from: &str, to: &str| {
            let settle = |text: &str| -> Option<Option<usize>> {
                let text = text.trim();
                if text.is_empty() {
                    return Some(None);
                }
                let spelled = match text.parse::<ShortId>() {
                    Ok(_) => text.to_string(),
                    Err(e) => crate::ids::hint_where(e.hint, |id| {
                        matches!(
                            id,
                            ShortId::Block {
                                story: Story::Body,
                                ..
                            }
                        )
                    })?,
                };
                match spelled.parse::<ShortId>() {
                    Ok(ShortId::Block {
                        story: Story::Body,
                        path,
                    }) => match path[..] {
                        [Step::Paragraph(n)] => Some(Some(n)),
                        _ => None,
                    },
                    _ => None,
                }
            };
            let (from, to) = (settle(from)?, settle(to)?);
            if from.is_none() && to.is_none() || matches!((from, to), (Some(a), Some(b)) if b < a) {
                return None;
            }
            let spell = |n: Option<usize>| n.map_or_else(String::new, |n| format!("p{n}"));
            Some(format!("{}-{}", spell(from), spell(to)))
        };
        let mut picks = Vec::new();
        for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some((from, to)) = item.split_once('-') {
                let (Ok(from_end), Ok(to_end)) = (end(item, from), end(item, to)) else {
                    return Err(refused(item, settled_range(from, to)));
                };
                let (from, to) = (from_end, to_end);
                if from.is_none() && to.is_none() {
                    return Err(refused(item, None));
                }
                let from = from.unwrap_or(0);
                if to.is_some_and(|to| to < from) {
                    return Err(format!("{item}: the range runs backwards"));
                }
                picks.push(Pick::Paragraphs { from, to });
                continue;
            }
            picks.push(match body(item, item)?[..] {
                [Step::Paragraph(n)] => Pick::Paragraphs {
                    from: n,
                    to: Some(n),
                },
                [Step::Table(n)] => Pick::Table(n),
                _ => return Err(refused(item, None)),
            });
        }
        if picks.is_empty() {
            return Err("no paragraphs selected".to_string());
        }
        Ok(Self::Picks(picks))
    }
}

/// The author and timestamp a `.docx` gives the changes and comments that
/// name none, when no parameter does: its first `dc:creator` and its
/// `dcterms:created`. `None` for each it does not tell (or bytes that are
/// no package); the caller falls back to `Redline` and
/// [`crate::document_comparer::DEFAULT_DATE`].
///
/// ```
/// let (author, date) = jubarte::markdown::docx_stamp(b"not a docx");
/// assert_eq!((author, date), (None, None));
/// ```
pub fn docx_stamp(docx: &[u8]) -> (Option<String>, Option<String>) {
    from_docx::creator_created(docx)
}

/// The author and timestamp a Markdown document gives the changes and
/// comments that name none, when no parameter does: its front matter
/// (`author:`; `last_modified:`, else `modified:`, `date:`, `created:`),
/// else the reference `.docx`'s creator and creation time ([`docx_stamp`]),
/// each on its own. `None` for each neither tells: the caller falls back
/// to `Redline` and now, as `jubarte convert` does.
///
/// ```
/// let md = "---\nauthor: Ann\ncreated: 2026-03-02\n---\n\nText.\n";
/// let (author, date) = jubarte::markdown::markdown_stamp(md, None);
/// assert_eq!(author.as_deref(), Some("Ann"));
/// assert_eq!(date.as_deref(), Some("2026-03-02T00:00:00Z"));
/// assert_eq!(jubarte::markdown::markdown_stamp("Text.\n", None), (None, None));
/// ```
pub fn markdown_stamp(
    markdown: &str,
    reference: Option<&[u8]>,
) -> (Option<String>, Option<String>) {
    let front = view_notes::file_defaults(markdown);
    let (creator, created) = reference.map(docx_stamp).unwrap_or_default();
    (front.author.or(creator), front.date.or(created))
}

/// Why a `read -p` pick was refused.
const SELECT_EXPECTED: &str = "expected pN, pN-pM or tN";

/// A document read as Markdown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadDocx {
    /// The Markdown, with CriticMarkup when changes are kept.
    pub markdown: String,
    /// Pictures to write under [`MarkdownOptions::extract_media`]: file
    /// name and bytes, sorted by name.
    pub media: Vec<(String, Vec<u8>)>,
    /// The body paragraphs [`MarkdownOptions::select`] kept (`N` of `pN`,
    /// ascending); `None` without a selection.
    pub selected: Option<Vec<usize>>,
    /// The header, footer and note stories (`header1`, `footnotes`) a
    /// `--changed` selection keeps for their changes; empty otherwise.
    pub selected_stories: Vec<String>,
}

/// Reads a `.docx` as Markdown: headings, emphasis, links, lists,
/// tables, footnotes and equations (as LaTeX), with tracked changes and
/// comments as CriticMarkup (`{++new++}`, `{--old--}`, `{~~old~>new~~}`,
/// `{==text==}{>>note<<}`), each change followed by its author and
/// date. Backs the Markdown output of `jubarte convert FILE.docx -t md`
/// and the Python / WASM `docx_to_markdown` binding.
///
/// # Errors
///
/// [`MarkdownError::Docx`] when the bytes are not a `.docx` (a `.doc`,
/// an encrypted document, or RTF is named for what it is rather than
/// read as a bad ZIP), or when the document cannot be read by
/// `from_docx` (an OPC read failure, a malformed part, or a missing
/// main document part).
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
        selected: converted.selected,
        selected_stories: converted.selected_stories,
    })
}

/// How [`read`] (and `jubarte read`) prints the agent view. Defaults
/// to all tracked changes as CriticMarkup, comments inline, no inline
/// timestamps on notes, `<!-- page N of M -->` lines from the layout
/// pass, no selection, and no source name.
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

/// A document's agent view and what [`read`] could not show (always
/// populated, even empty).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadView {
    /// The YAML header and the Markdown with id lines.
    pub markdown: String,
    /// Why the page markers are missing (the layout pass failed) and the
    /// like. The view is complete otherwise.
    pub warnings: Vec<String>,
    /// The body paragraphs [`ReadOptions::select`] kept (`N` of `pN`,
    /// ascending); `None` without a selection.
    pub selected: Option<Vec<usize>>,
    /// The header, footer and note stories (`header1`, `footnotes`) a
    /// `--changed` selection keeps for their changes; empty otherwise.
    pub selected_stories: Vec<String>,
}

/// Reads a `.docx` as the agent view every surface prints for `read`
/// (alias `text`): a YAML header, then Markdown with an `<!-- pN -->`
/// id line before every paragraph, tracked changes and comments with
/// their ids, and `<!-- page N of M -->` markers from the layout pass.
/// Backs `jubarte read`, `jubarte text`, and the Python / WASM `read`
/// binding. See `docs/MARKDOWN.md`, "Agent view".
///
/// The view is built with [`docx_to_markdown`] and, when `page_markers`
/// is on, [`page_texts`] feeds [`paginate`] the per-page painted text;
/// the converter failure (if any) is recorded as a warning and the
/// markers come from the document's own breaks instead.
///
/// # Errors
///
/// [`MarkdownError::Docx`] when [`docx_to_markdown`] cannot read the
/// document.
///
/// ```
/// use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
///
/// let docx = markdown_to_docx("Due in {~~30~>45~~} days.", &DocxOptions::default())?.docx;
/// let view = read(&docx, &ReadOptions { page_markers: false, ..ReadOptions::default() })?;
/// assert!(view.markdown.contains("<!-- p0 -->\nDue in {~~30~>45~~}{>>r"), "{}", view.markdown);
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
        // The converter still marks pages, from the document's own breaks.
        .map_err(|e| {
            let e = e.to_string();
            warnings.push(format!(
                "layout failed, page markers come from the document's breaks: {}",
                e.strip_prefix("layout failed: ").unwrap_or(&e)
            ));
        })
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
        selected: read.selected,
        selected_stories: read.selected_stories,
    })
}

/// The snapshot `read --json` prints: [`crate::inspect::snapshot`], kept to
/// the body paragraphs `options.select` picks (and the changes and comments
/// in them, and under `--changed` those of the stories it changed) when it
/// picks some. The selection reads the view `options`
/// describes, without page markers. The view prints a table whole, but a
/// paragraph pick keeps only the paragraphs it names: `-p p5` of a cell
/// is p5, not its table; `-p t0` keeps the table's every paragraph.
pub fn read_snapshot(
    docx: &[u8],
    options: &ReadOptions,
) -> Result<crate::inspect::Snapshot, MarkdownError> {
    let docx_error = |e: crate::inspect::InspectError| MarkdownError::Docx(e.to_string());
    let mut snapshot = crate::inspect::snapshot(docx).map_err(docx_error)?;
    if options.select.is_some() {
        let view = read(
            docx,
            &ReadOptions {
                page_markers: false,
                ..options.clone()
            },
        )?;
        let mut kept = view.selected.unwrap_or_default();
        if let Some(Select::Picks(picks)) = &options.select {
            let tables: Vec<Pick> = picks
                .iter()
                .filter(|p| matches!(p, Pick::Table(_)))
                .cloned()
                .collect();
            let in_tables = if tables.is_empty() {
                Vec::new()
            } else {
                read(
                    docx,
                    &ReadOptions {
                        page_markers: false,
                        select: Some(Select::Picks(tables)),
                        ..options.clone()
                    },
                )?
                .selected
                .unwrap_or_default()
            };
            let named = |n: usize| {
                picks.iter().any(|p| match p {
                    Pick::Paragraphs { from, to } => n >= *from && to.is_none_or(|to| n <= to),
                    Pick::Table(_) => false,
                })
            };
            kept.retain(|&n| named(n) || in_tables.contains(&n));
        }
        snapshot.retain_with(&kept, &view.selected_stories);
    }
    Ok(snapshot)
}

/// The view `jubarte edit` and `jubarte add` print after applying:
/// the blocks of `docx` (the redline, or with `accepted` its accepted
/// text with `rev` tags on the id lines) that carry `author`'s marks,
/// as the agent view without page markers. When none does, as after
/// resolving another author's comment, the blocks changed by anyone.
///
/// # Inputs
///
/// - `docx` — the document to read.
/// - `author` — the author whose marks select the blocks.
/// - `accepted` — `false` keeps the tracked changes as CriticMarkup;
///   `true` reads the accepted text with `rev` clauses on the id
///   lines.
/// - `source` — the `source:` line written into the header; `None`
///   leaves it as `(bytes)`.
///
/// # Errors
///
/// [`MarkdownError::Docx`] when [`docx_to_markdown`] cannot read the
/// document.
pub fn changed_view(
    docx: &[u8],
    author: &str,
    accepted: bool,
    source: Option<&str>,
) -> Result<String, MarkdownError> {
    let options = |by: Option<String>| MarkdownOptions {
        track_changes: if accepted {
            TrackChanges::Accept
        } else {
            TrackChanges::All
        },
        ids: true,
        source: source.map(str::to_string),
        page_markers: false,
        select: Some(Select::Changed { by }),
        ..MarkdownOptions::default()
    };
    let view = docx_to_markdown(docx, &options(Some(author.to_string())))?.markdown;
    let empty = view
        .splitn(3, "---\n")
        .nth(2)
        .is_none_or(|body| body.trim().is_empty());
    if empty {
        return Ok(docx_to_markdown(docx, &options(None))?.markdown);
    }
    Ok(view)
}

/// The text the layout pass paints on each page, with the document's
/// changes kept (`TrackChanges::All`), accepted (`TrackChanges::Accept`)
/// or rejected (`TrackChanges::Reject`). What [`paginate`] matches body
/// blocks against to place `<!-- page N of M -->` lines, and what [`read`]
/// feeds to its markdown converter.
///
/// # Inputs
///
/// - `docx` — the document to lay out.
/// - `track_changes` — accept or reject its revisions before laying out,
///   or keep them.
/// - `revisions` — how the layout pass paints revision marks (the
///   conventional red/blue/green redline, the Word-style marks, or a
///   [`crate::convert::RevisionPalette`]). The bench suite measures
///   against `RevisionStyle::Word`; call sites that compare a `jubarte`
///   PDF against a Word one must pass `RevisionStyle::Word`.
///
/// # Errors
///
/// An error string when accepting or rejecting every revision fails, or
/// the layout pass fails to admit the package, find its main document
/// part, or emit its PDF object stream.
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

/// `markdown` (read from `docx`) with `<!-- page N of M -->` lines, as
/// `convert --to md` writes it: `docx` is laid out as its PDF would be,
/// with its changes kept, accepted or rejected as the Markdown has them,
/// and each block found on its page. When layout fails, the Markdown alone
/// and why.
#[must_use]
pub fn with_page_markers(
    docx: &[u8],
    markdown: &str,
    track_changes: TrackChanges,
    revisions: crate::convert::RevisionStyle,
) -> (String, Option<String>) {
    match page_texts(docx, track_changes, revisions) {
        Ok(pages) => {
            let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
            (paginate(markdown, &pages), None)
        }
        Err(e) => (markdown.to_string(), Some(format!("no page markers: {e}"))),
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

/// Why a Markdown / `.docx` document could not be read or written. The
/// variants name which side failed: the reference document the caller
/// gave, the assembled package, or the document the caller asked to
/// read.
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
    fn the_bindings_options_read_from_json() {
        let reference = [1u8, 2];
        let options = docx_options_from_json(
            r#"{"page": "a4", "author": "Legal", "date": "2026-10-02T00:00:00Z",
               "critic": false, "track_changes": "accept", "tracking": true,
               "scholar": true}"#,
            Some(&reference),
        )
        .unwrap();
        assert_eq!(options.page, PageSize::A4);
        assert_eq!(
            (options.author.as_str(), options.author_fixed),
            ("Legal", true)
        );
        assert_eq!(
            (options.date.as_str(), options.date_fixed),
            ("2026-10-02T00:00:00Z", true)
        );
        assert!(!options.critic);
        assert_eq!(options.track_changes, TrackChanges::Accept);
        assert_eq!(options.tracking, Some(true));
        assert!(options.scholar);
        assert_eq!(options.reference, Some(&reference[..]));
        // `trackChanges` is the JavaScript spelling.
        let options = docx_options_from_json(r#"{"trackChanges": "reject"}"#, None).unwrap();
        assert_eq!(options.track_changes, TrackChanges::Reject);
        // Nothing given: the defaults.
        let options = docx_options_from_json("{}", None).unwrap();
        assert!(options.critic && !options.scholar && !options.author_fixed);
        assert_eq!(options.tracking, None);
        for (json, word) in [
            (r#"{"track_changes": "keep"}"#, "keep"),
            (r#"{"page": "legal"}"#, "legal"),
            (r#"{"scholarly": true}"#, "scholarly"),
        ] {
            let error = docx_options_from_json(json, None).unwrap_err();
            assert!(error.contains(word), "{error}");
        }
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
