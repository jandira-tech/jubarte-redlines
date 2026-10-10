// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Guarded document editing for agents and tools.
//!
//! An [`EditPlan`] names exact, uniquely anchored operations against one
//! snapshot of a DOCX (optionally bound to its SHA-256). Every operation is
//! resolved against the untouched source first; any failure fails the whole
//! plan and nothing is written. On success the operations are applied to a
//! copy, comments are authored in that copy, and the existing comparer turns
//! source plus copy into a Word tracked-changes document. The caller receives
//! the clean copy, the redline and a per-operation [`EditReport`].
//!
//! Text coordinates are the visible projection of [`crate::inspect`]: what
//! `jubarte inspect` prints is what a plan anchors to. Replacement text is
//! inserted into the run that held the anchor, so it inherits that run's
//! formatting; the plan never carries raw OOXML.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::changes::{Change, ChangeError, ChangeFilter};

use crate::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use crate::inspect::{
    BODY_STORY, Opened, Piece, Projection, SCHEMA_VERSION, project_paragraph, source_sha256,
};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XNamespace};

mod controls;
pub mod flags;
mod images;
mod notes;
mod rewrite;
mod runs;
mod sections;
mod structural;
mod tracked;
mod watermark;
mod whole;

pub use controls::ControlSelector;
pub use sections::{CustomPage, Margins, Orientation, PageSize, Paper, SectionScope};

/// A versioned, portable set of operations against one document snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditPlan {
    /// Wire schema version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// SHA-256 of the source bytes; when present the plan is refused unless it
    /// matches. Omit only when the caller owns the bytes end to end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    /// Revision and comment author.
    pub author: String,
    /// Revision and comment timestamp (`YYYY-MM-DDTHH:MM:SSZ`). When
    /// omitted, everything the plan writes takes the time it is applied, in
    /// UTC, as Word dates its changes; set it for reproducible output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Comment initials; derived from `author` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initials: Option<String>,
    /// Tracked changes to accept and reject before anything else; the
    /// changes it leaves follow `existing_revisions`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve_revisions: Option<ResolveRevisions>,
    /// What to do when the source already holds tracked changes.
    #[serde(default)]
    pub existing_revisions: ExistingRevisions,
    /// Operations in report order.
    pub operations: Vec<Operation>,
    /// Refresh `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and `TOC` results in the
    /// edited copy from jubarte's layout ([`crate::fields::update_fields`])
    /// before the redline is compared.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub update_fields: bool,
}

/// The tracked changes a plan accepts and rejects before it edits, as
/// Word's Accept / Reject This Change (see [`crate::changes`]). Each side
/// is a [`ChangeFilter`]; left out, it resolves nothing. No change may be
/// selected by both (`REVISION_CONFLICT`); both sides of a move count as
/// one change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolveRevisions {
    /// Changes to accept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accept: Option<ChangeFilter>,
    /// Changes to reject (after the accepted ones).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject: Option<ChangeFilter>,
}

/// The changes `resolve_revisions` resolved, by id.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRevisions {
    /// Accepted changes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted: Vec<String>,
    /// Rejected changes (a change an accepted one took along is not listed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected: Vec<String>,
}

impl ResolvedRevisions {
    /// Nothing resolved.
    pub fn is_empty(&self) -> bool {
        self.accepted.is_empty() && self.rejected.is_empty()
    }
}

/// Policy for a source that already contains tracked changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistingRevisions {
    /// Fail with `EXISTING_REVISIONS` (default).
    #[default]
    Refuse,
    /// Accept them first; the plan then edits that accepted base.
    Accept,
    /// Reject them first; the plan then edits that rejected base.
    Reject,
    /// Leave them tracked; the plan's edits become new revisions beside them
    /// (direct emission; no compare).
    Keep,
}

/// One operation. `id` defaults to `op-N` (1-based) in the report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Caller-chosen operation id echoed in the report.
    pub id: Option<String>,
    #[serde(flatten)]
    /// The operation.
    pub kind: OperationKind,
}

/// The supported operation kinds (wire tag `kind`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationKind {
    /// Replace exactly one occurrence of `find` in the paragraph.
    Replace {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// Exact text to locate; must occur exactly once.
        find: String,
        /// Plain replacement text (may be empty).
        replacement: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Formatting of the replacement on top of the replaced run's.
        format: Option<RunFormat>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Comment text anchored to the changed text.
        comment: Option<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        /// Show the change as all of `find` deleted, then all of
        /// `replacement` inserted, instead of Word Compare's word-level diff.
        whole: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert `text` after/before exactly one occurrence of an anchor, or at
    /// the paragraph's start/end. Exactly one of `after`, `before`,
    /// `position` must be given.
    Insert {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Insert right after this unique anchor text.
        after: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Insert right before this unique anchor text.
        before: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Insert at the paragraph start or end.
        position: Option<Edge>,
        /// Plain text to insert.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Formatting of the new text on top of the neighbouring run's.
        format: Option<RunFormat>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Comment text anchored to the changed text.
        comment: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
    },
    /// Delete exactly one occurrence of `find`.
    Delete {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// Exact text to delete; must occur exactly once.
        find: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
    },
    /// Comment on exactly one occurrence of `find`, or on the whole paragraph.
    Comment {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Text to anchor the comment to; whole paragraph when omitted.
        find: Option<String>,
        /// Plain text to insert.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Last paragraph of a range from the start of `paragraph` to the
        /// end of this one (same story, not before `paragraph`); `find`
        /// must be left out.
        through: Option<Selector>,
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert a new paragraph next to the anchor paragraph, copying its
    /// paragraph properties (never its section break or revision marks), or
    /// those of `like`.
    InsertParagraph {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        /// Runs of the new paragraph, in order.
        runs: Vec<RunSpec>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Paragraph whose properties and run formatting the new paragraph
        /// copies instead of the anchor's: a plain paragraph inserted after a
        /// list item, for one.
        like: Option<Selector>,
        /// Paragraph style id to set instead of the anchor's.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Paragraph style id to set instead of the anchor's.
        style: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Comment text anchored to the changed text.
        comment: Option<String>,
    },
    /// Delete a whole paragraph, mark included.
    DeleteParagraph {
        /// Paragraph to delete; must match exactly one.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Comment on the deleted text. It is in the redline only: the clean
        /// copy has no paragraph to hold it.
        comment: Option<String>,
    },
    /// Change a paragraph's style, alignment or spacing; the redline records
    /// the old properties (`w:pPrChange`). At least one field besides
    /// `paragraph`.
    FormatParagraph {
        /// Paragraph to format; must match exactly one.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Paragraph style, by id (`Heading1`) or name (`heading 1`).
        style: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Horizontal alignment.
        alignment: Option<Alignment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Line spacing as a multiple of single spacing (`1.5`, `2`).
        line_spacing: Option<LineSpacing>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Space before the paragraph, in points.
        space_before: Option<Points>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Space after the paragraph, in points.
        space_after: Option<Points>,
    },
    /// Make the paragraph read as `text`: the engine applies the smallest
    /// word-level edits, so unchanged words keep their runs and formatting,
    /// and new words take the formatting of the run before them. Tabs, line
    /// breaks and symbols stay where they are; `text` may write a tab or a
    /// break as a space and leave a symbol out.
    Rewrite {
        /// Paragraph to rewrite; must match exactly one.
        paragraph: Selector,
        /// The paragraph's new text, plain.
        text: String,
    },
    /// Join the paragraph with the one right after it. The first's text moves
    /// to the start of the second, whose paragraph properties (and section
    /// break) survive, as when Word accepts a deleted paragraph mark. The
    /// redline deletes the first paragraph's mark.
    MergeParagraphs {
        /// First paragraph; the next paragraph must follow it directly.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Plain text placed between the two (for example `" "`).
        separator: Option<String>,
    },
    /// Reply to comment `comment_id`, as the plan's author; the reply is
    /// anchored on the same range. Word threads are one level deep, so a
    /// reply to a reply joins the thread's first comment.
    ReplyComment {
        /// `w:id` of the comment replied to (`jubarte comments` lists them).
        comment_id: u32,
        /// Reply text; `\n` starts a new line.
        text: String,
    },
    /// Resolve comment `comment_id` and its replies (`done: false` opens
    /// them again).
    ResolveComment {
        /// `w:id` of the comment.
        comment_id: u32,
        #[serde(default = "resolve_done")]
        /// Resolved (default) or open.
        done: bool,
    },
    /// Replace the text of comment `comment_id`; its id, author, date and
    /// thread stay.
    EditComment {
        /// `w:id` of the comment.
        comment_id: u32,
        /// New text; `\n` starts a new line.
        text: String,
    },
    /// Remove comment `comment_id`, its replies, their range markers and
    /// references.
    DeleteComment {
        /// `w:id` of the comment.
        comment_id: u32,
    },
    /// Insert a table next to the anchor paragraph. Each cell holds one
    /// paragraph in the anchor's paragraph style; the redline shows the
    /// rows inserted. Body paragraphs outside tables only.
    InsertTable {
        /// Anchor paragraph; must match exactly one.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        /// Cell text, row by row; every row has the same number of cells.
        rows: Vec<Vec<String>>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        /// Mark the first row as a header row that repeats on each page.
        header_row: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Column widths in twentieths of a point; the text width split
        /// evenly when omitted.
        widths_dxa: Option<Vec<u32>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Table style, by id or name; `TableGrid` (added when the document
        /// lacks it) when omitted.
        style: Option<String>,
    },
    /// Make the paragraphs a list: each gets direct numbering at `level`,
    /// and `ListParagraph` when it has no style. The redline records each
    /// paragraph's old properties (`w:pPrChange`).
    List {
        /// Paragraphs to number, each selector matching exactly one; body
        /// only.
        paragraphs: Vec<Selector>,
        #[serde(default)]
        /// Bullets, decimal numbers or lowercase letters.
        kind_of_list: ListKind,
        #[serde(default)]
        /// List level, 0 (outermost) to 8.
        level: u32,
        #[serde(default = "default_true")]
        /// Start a new list (true), or continue the list of the nearest
        /// numbered paragraph before the first one, in that list's format.
        restart: bool,
    },
    /// Write Word's own text watermark (Insert > Watermark) into every
    /// default header, creating a header where the first section has none.
    /// One per document. It is header content, not a tracked change: the
    /// clean copy and the redline both carry it.
    Watermark {
        /// The watermark text, plain, 1 to 64 characters.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Fill colour, six hex digits (default `C0C0C0`).
        color: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Diagonal at 315 degrees (default); `false` lays it horizontal.
        diagonal: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Font family (default `Calibri`).
        font: Option<String>,
    },
    /// Fill one content control (`w:sdt`) in the body with exactly one of
    /// `text`, `choice`, `checked` or `date`. The control keeps its
    /// properties; its content becomes the value.
    FillControl {
        /// The control: `"body:sdt:N"`, `{"id"}`, `{"tag"}` or `{"alias"}`;
        /// must match exactly one.
        control: ControlSelector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Plain text, for text, rich-text and combo-box controls.
        text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// A list item's value or display text, for drop-down and combo-box
        /// controls; the display text is written.
        choice: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Checkbox state.
        checked: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// `YYYY-MM-DD`, for date controls; written in the control's format.
        date: Option<String>,
    },
    /// Change the run formatting of one occurrence of existing text; the
    /// redline records the old formatting (`w:rPrChange`).
    FormatRun {
        /// Paragraph to format; must match exactly one.
        paragraph: Selector,
        /// Exact text to format.
        find: String,
        /// Formatting to set; fields not given stay as they are.
        format: RunFormat,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which occurrence of `find` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert a footnote whose reference mark follows one occurrence of
    /// `after`; the note goes in the footnotes part, created when absent.
    InsertFootnote {
        /// Body paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// The reference mark goes right after this anchor text.
        after: String,
        /// Plain text of the note.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which occurrence of `after` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert a paragraph holding one inline picture next to the anchor
    /// paragraph. PNG, JPEG, GIF, BMP or TIFF.
    InsertImage {
        /// Body paragraph the picture goes next to; must match exactly one.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        /// The picture file, base64-encoded.
        image_base64: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// The picture's media type; checked against its bytes when given.
        content_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Width in EMU (914400 per inch); the height keeps the aspect
        /// ratio. Default: the pixel size at 96 dpi, at most 6.5 inches.
        width_emu: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Alternative text (`wp:docPr descr`).
        alt: Option<String>,
    },
    /// Set the page size, orientation and margins of the last section or of
    /// every section; the redline records the old ones (`w:sectPrChange`).
    /// At least one of `page`, `orientation`, `margins_dxa`.
    PageSetup {
        #[serde(default)]
        /// `last` (default) or `all`.
        section: SectionScope,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// `letter`, `a4`, or `{"width_dxa", "height_dxa"}`.
        page: Option<PageSize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// `portrait` or `landscape`.
        orientation: Option<Orientation>,
        #[serde(default, skip_serializing_if = "is_default_margins")]
        /// Margins to change, in twentieths of a point.
        margins_dxa: Margins,
    },
    /// Insert a table of contents next to the anchor paragraph: a `TOC \o
    /// "1-{levels}" \h \z \u` field, after an optional title paragraph
    /// styled `TOCHeading`. Its entries are written when the plan sets
    /// `update_fields`; otherwise the field is empty until Word updates it.
    InsertToc {
        /// Paragraph to insert next to; must match exactly one, in the body.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        #[serde(default = "default_toc_levels")]
        /// Heading levels listed, 1 to 9 (default 3).
        levels: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Title paragraph above the TOC (`"Contents"`).
        title: Option<String>,
    },
    /// Replace one occurrence of `find` with one full block
    /// (U+2588) per character, untracked: the clean copy and the redline
    /// both show the blocks and neither keeps the text. The plan is refused
    /// with `REDACTION_LEAK` when the text still occurs anywhere in either
    /// output, and the report never repeats it.
    Redact {
        /// Paragraph to redact in; must match exactly one.
        paragraph: Selector,
        /// Exact text to remove.
        find: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Which occurrence of `find` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Write document settings into `word/settings.xml` in schema order:
    /// Track Changes, update fields on open, editing restrictions. Settings
    /// are not revisions: the clean copy and the redline both carry them.
    /// One per plan; a setting left out stays as it is.
    Settings {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Turn Track Changes on (`w:trackRevisions`) or off.
        track_revisions: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Ask Word to update fields on open (`w:updateFields`), or not. The
        /// plan's own `update_fields` writes jubarte's results instead.
        update_fields: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Restrict editing (`w:documentProtection`); `edit: none` lifts it.
        protection: Option<crate::settings::Protection>,
    },
}

fn default_toc_levels() -> u8 {
    3
}

fn is_default_margins(margins: &Margins) -> bool {
    *margins == Margins::default()
}

fn resolve_done() -> bool {
    true
}

fn default_true() -> bool {
    true
}

/// The numbering of a [`OperationKind::List`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListKind {
    #[default]
    /// `•`, `◦`, `▪` by level.
    Bullet,
    /// `1.`, `2.`, ...
    Decimal,
    /// `a.`, `b.`, ...
    LowerLetter,
}

impl ListKind {
    fn format(self) -> crate::markdown::xml::ListFormat {
        use crate::markdown::xml::ListFormat;
        match self {
            Self::Bullet => ListFormat::Bullet,
            Self::Decimal => ListFormat::Decimal,
            Self::LowerLetter => ListFormat::LowerLetter,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Bullet => "bullet",
            Self::Decimal => "decimal",
            Self::LowerLetter => "lower_letter",
        }
    }
}

/// Line spacing, stored as `w:line` (240 = single); JSON is the multiple.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineSpacing(pub u32);

impl Serialize for LineSpacing {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(f64::from(self.0) / 240.0)
    }
}

impl<'de> Deserialize<'de> for LineSpacing {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let multiple = f64::deserialize(d)?;
        if !(0.25..=10.0).contains(&multiple) {
            return Err(serde::de::Error::custom(format!(
                "line_spacing {multiple} is outside 0.25..=10"
            )));
        }
        Ok(Self((multiple * 240.0).round() as u32))
    }
}

/// A length stored in twips (1/20 pt); JSON is points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Points(pub u32);

impl Serialize for Points {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(f64::from(self.0) / 20.0)
    }
}

impl<'de> Deserialize<'de> for Points {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let points = f64::deserialize(d)?;
        if !(0.0..=1584.0).contains(&points) {
            return Err(serde::de::Error::custom(format!(
                "spacing {points}pt is outside 0..=1584"
            )));
        }
        Ok(Self((points * 20.0).round() as u32))
    }
}

/// Paragraph spacing changes for [`OperationKind::FormatParagraph`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Spacing {
    line: Option<LineSpacing>,
    before: Option<Points>,
    after: Option<Points>,
}

/// Paragraph alignment for [`OperationKind::FormatParagraph`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    /// Left (start) aligned.
    Left,
    /// Centered.
    Center,
    /// Right (end) aligned.
    Right,
    /// Justified.
    Justify,
}

impl Alignment {
    /// The `w:jc` value Word writes.
    fn jc(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Justify => "both",
        }
    }
}

/// Run formatting for inserted or replacement text. Fields not given keep
/// the neighbouring run's formatting.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunFormat {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear bold.
    pub bold: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear italic.
    pub italic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear single underline.
    pub underline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Highlight color name (`yellow`, ...) or `none`.
    pub highlight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Font name, set for every script (`w:rFonts`).
    pub font: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Font size in points, rounded to half points.
    pub size_pt: Option<HalfPoints>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Text colour as six hex digits (`FF0000`) or `auto`.
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear single strikethrough.
    pub strike: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear all caps.
    pub caps: Option<bool>,
}

/// A font size stored in half points (`w:sz`); JSON is points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HalfPoints(pub u32);

impl Serialize for HalfPoints {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(f64::from(self.0) / 2.0)
    }
}

impl<'de> Deserialize<'de> for HalfPoints {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let points = f64::deserialize(d)?;
        if !points.is_finite() || !(0.0..=1638.0).contains(&points) {
            return Err(serde::de::Error::custom(format!(
                "size_pt {points} is outside 0..=1638"
            )));
        }
        Ok(Self((points * 2.0).round() as u32))
    }
}

/// Paragraph edge for an insertion without a text anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    /// Paragraph start.
    Start,
    /// Paragraph end.
    End,
}

/// Side of the anchor paragraph for [`OperationKind::InsertParagraph`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Before the anchor paragraph.
    Before,
    #[default]
    /// After the anchor paragraph.
    After,
}

/// The number after `prefix` in a part of a short id (`p12`, `r1`, `c2`),
/// digits only.
fn short_number(text: &str, prefix: char) -> Option<usize> {
    let digits = text.strip_prefix(prefix)?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Paragraph selector; every form must match exactly one paragraph. Ids
/// name their story (`body:p:3`, `header1:p:0`) or use the agent view's
/// short forms (`p3`, `header1`, `header1.p1`, `footer2`, `t0.r1.c2`,
/// `t0.r1.c2.p1`); the report prints the long form. The other forms search
/// the body unless they carry a `story` (`header1`, `footnotes`, ...).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Selector {
    /// `"body:p:12"`, the same as `{"id": "body:p:12"}`.
    Name(String),
    /// `{"id": "body:p:12"}`.
    Id {
        /// `{story}:p:N`.
        id: String,
    },
    /// `{"index": 12}`.
    Index {
        /// Zero-based index in the story.
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Story to search; the body when omitted.
        story: Option<String>,
    },
    /// `{"starts_with": "..."}`; unique prefix match.
    StartsWith {
        /// Unique paragraph text prefix.
        starts_with: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Story to search; the body when omitted.
        story: Option<String>,
    },
    /// `{"contains": "..."}`; unique substring match.
    Contains {
        /// Unique paragraph text substring.
        contains: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Story to search; the body when omitted.
        story: Option<String>,
    },
}

/// A run of an inserted paragraph. Formatting not given is inherited from the
/// anchor paragraph's first run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSpec {
    /// Run text.
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear bold.
    pub bold: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear italic.
    pub italic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Set or clear single underline.
    pub underline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Highlight color name (`yellow`, ...) or `none`.
    pub highlight: Option<String>,
}

impl RunSpec {
    /// The run's formatting toggles.
    pub fn format(&self) -> RunFormat {
        RunFormat {
            bold: self.bold,
            italic: self.italic,
            underline: self.underline,
            highlight: self.highlight.clone(),
            ..RunFormat::default()
        }
    }
}

/// Outcome of one operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditOutcome {
    /// Operation id (`op-N` when the plan gave none).
    pub id: String,
    /// Operation kind.
    pub kind: String,
    /// `ok`, `failed`, or `skipped` (not reached because an earlier
    /// operation failed the plan).
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Resolved paragraph id.
    pub paragraph: Option<String>,
    /// Anchor occurrences found (1 on success).
    pub matches: usize,
    /// `before {old→new} after`, `{+inserted}`, `{-deleted}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// `before {old→new} after`, `{+inserted}`, `{-deleted}`.
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Comment id written for this operation.
    pub comment_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The anchor as the plan wrote it, when it matched only without its
    /// Markdown marks.
    pub anchor_given: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The text actually matched in that case.
    pub anchor_read_as: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Error code when failed.
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Error message when failed; on an `ok` operation, a note that it
    /// could not be shown as asked (a `whole` replacement kept word-level).
    pub message: Option<String>,
}

/// Comparer revision counts in the redline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionCounts {
    /// Insertions.
    pub inserted: usize,
    /// Deletions.
    pub deleted: usize,
    /// Moves.
    pub moved: usize,
    /// Formatting changes: run (`w:rPrChange`) and paragraph
    /// (`w:pPrChange`) properties.
    pub format_changed: usize,
    /// All revision records.
    pub total: usize,
}

/// Paragraph count before and after.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParagraphDelta {
    /// Before the plan.
    pub from: usize,
    /// After the plan.
    pub to: usize,
}

/// What happened, per operation and overall.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditReport {
    /// Wire schema version; must equal [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Every operation succeeded.
    pub ok: bool,
    /// SHA-256 of the bytes handed in.
    pub source_sha256: String,
    /// SHA-256 of the base the plan edited (differs from `source_sha256`
    /// only when existing revisions were flattened first).
    pub base_sha256: String,
    /// The plan carried a `source_sha256` that was verified.
    pub guarded: bool,
    /// Revision and comment author.
    pub author: String,
    /// Revision and comment timestamp used.
    pub date: String,
    /// Policy that was applied.
    pub existing_revisions: ExistingRevisions,
    /// Changes `resolve_revisions` accepted and rejected.
    #[serde(default, skip_serializing_if = "ResolvedRevisions::is_empty")]
    pub resolved_revisions: ResolvedRevisions,
    /// Body paragraph count before and after.
    pub paragraphs: ParagraphDelta,
    /// One outcome per operation, in plan order.
    pub operations: Vec<EditOutcome>,
    /// Comments written by this plan.
    pub comments_added: usize,
    /// Zero until the redline exists (a preview never compares).
    pub revisions: RevisionCounts,
    /// Fields whose results `update_fields` wrote into the clean copy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<crate::fields::FieldUpdate>,
}

impl EditReport {
    /// JSON lines an agent can append to its own log: `load`, one `op` per
    /// operation, `summary`.
    pub fn to_jsonl(&self) -> String {
        let mut out = String::new();
        let mut push = |v: serde_json::Value| {
            out.push_str(&v.to_string());
            out.push('\n');
        };
        let mut load = serde_json::json!({
            "ev": "load",
            "sha256": self.source_sha256,
            "base_sha256": self.base_sha256,
            "guarded": self.guarded,
            "paras": self.paragraphs.from,
            "author": self.author,
            "date": self.date,
            "existing_revisions": self.existing_revisions,
        });
        if !self.resolved_revisions.is_empty() {
            load.as_object_mut().expect("object").insert(
                "resolved_revisions".into(),
                serde_json::to_value(&self.resolved_revisions).expect("serializes"),
            );
        }
        push(load);
        for (i, op) in self.operations.iter().enumerate() {
            let mut v = serde_json::json!({
                "ev": "op",
                "i": i + 1,
                "id": op.id,
                "op": op.kind,
                "status": op.status,
                "matches": op.matches,
            });
            let map = v.as_object_mut().expect("object");
            if let Some(p) = &op.paragraph {
                map.insert("at".into(), p.clone().into());
            }
            if let Some(c) = &op.context {
                map.insert("ctx".into(), c.clone().into());
            }
            if let Some(c) = op.comment_id {
                map.insert("comment_id".into(), c.into());
            }
            if let Some(given) = &op.anchor_given {
                map.insert("anchor_given".into(), given.clone().into());
            }
            if let Some(read_as) = &op.anchor_read_as {
                map.insert("anchor_read_as".into(), read_as.clone().into());
            }
            if let Some(c) = &op.code {
                map.insert("code".into(), c.clone().into());
            }
            if let Some(m) = &op.message {
                map.insert("message".into(), m.clone().into());
            }
            push(v);
        }
        let failed = self
            .operations
            .iter()
            .filter(|o| o.status == "failed")
            .count();
        let ok = self.operations.iter().filter(|o| o.status == "ok").count();
        push(serde_json::json!({
            "ev": "summary",
            "status": if self.ok { "ok" } else { "failed" },
            "ops": {"ok": ok, "failed": failed},
            "paras": {"from": self.paragraphs.from, "to": self.paragraphs.to,
                      "d": self.paragraphs.to as i64 - self.paragraphs.from as i64},
            "revs": self.revisions,
            "comments": {"added": self.comments_added},
        }));
        out
    }
}

/// Clean copy, redline and report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditResult {
    /// The edited document without tracked changes.
    pub clean: Vec<u8>,
    /// Source compared against the clean copy: Word tracked changes.
    pub redline: Vec<u8>,
    /// Per-operation outcomes and totals.
    pub report: EditReport,
}

/// Why a plan was refused. `outcomes` lists every operation's status so the
/// caller can see which anchors resolved before the failure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditError {
    /// Stable error code (`STALE_SOURCE`, `AMBIGUOUS_ANCHOR`, ...).
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Operation id that failed, when one did.
    pub operation: Option<String>,
    /// Human-readable detail.
    pub message: String,
    #[serde(default)]
    /// Status of every operation at the time of failure.
    pub outcomes: Vec<EditOutcome>,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.operation {
            Some(op) => write!(f, "{} ({op}): {}", self.code, self.message),
            None => write!(f, "{}: {}", self.code, self.message),
        }
    }
}

impl std::error::Error for EditError {}

/// An unopenable source: admission refusals keep their own code
/// (`INPUT_LIMIT`, …), anything else is `INVALID_DOCUMENT`.
fn open_error(error: crate::inspect::InspectError) -> EditError {
    match error {
        crate::inspect::InspectError::Admission(refused) => {
            err(refused.code(), None, refused.message)
        }
        other => err("INVALID_DOCUMENT", None, other.to_string()),
    }
}

/// Accept, then reject, the changes `resolve_revisions` selects: the new
/// source (`None` when it selects nothing) and what was resolved.
fn resolve_selected(
    source: &[u8],
    selection: Option<&ResolveRevisions>,
) -> Result<(Option<Vec<u8>>, ResolvedRevisions), EditError> {
    let mut resolved = ResolvedRevisions::default();
    let Some(selection) = selection else {
        return Ok((None, resolved));
    };
    let change_err = |e: ChangeError| match e {
        ChangeError::UnknownChange(id) => err(
            "UNKNOWN_CHANGE",
            None,
            format!("resolve_revisions: no tracked change {id}"),
        ),
        ChangeError::Package(m) => err("INVALID_DOCUMENT", None, m),
    };
    let listed = crate::changes::list_changes(source).map_err(change_err)?;
    // The ids a side selects, a move's other side included.
    let select = |filter: Option<&ChangeFilter>| -> Result<Vec<String>, EditError> {
        let Some(filter) = filter else {
            return Ok(Vec::new());
        };
        if let Some(unknown) = filter
            .ids
            .iter()
            .flatten()
            .find(|id| !listed.iter().any(|c| &c.id == *id))
        {
            return Err(change_err(ChangeError::UnknownChange(unknown.clone())));
        }
        let hit: Vec<&Change> = listed.iter().filter(|c| filter.matches(c)).collect();
        Ok(listed
            .iter()
            .filter(|c| {
                hit.iter()
                    .any(|h| h.id == c.id || h.move_name.is_some() && h.move_name == c.move_name)
            })
            .map(|c| c.id.clone())
            .collect())
    };
    let accept = select(selection.accept.as_ref())?;
    let reject = select(selection.reject.as_ref())?;
    if let Some(both) = accept.iter().find(|id| reject.contains(id)) {
        return Err(err(
            "REVISION_CONFLICT",
            None,
            format!("resolve_revisions selects {both} to accept and to reject"),
        ));
    }
    if accept.is_empty() && reject.is_empty() {
        return Ok((None, resolved));
    }
    let mut bytes = source.to_vec();
    if !accept.is_empty() {
        bytes = crate::changes::accept_changes(&bytes, &ChangeFilter::ids(accept.clone()))
            .map_err(change_err)?;
        resolved.accepted = accept;
    }
    // A change the accepted ones took along is gone, not rejected.
    let left = crate::changes::list_changes(&bytes).map_err(change_err)?;
    let reject: Vec<String> = reject
        .into_iter()
        .filter(|id| left.iter().any(|c| &c.id == id))
        .collect();
    if !reject.is_empty() {
        bytes = crate::changes::reject_changes(&bytes, &ChangeFilter::ids(reject.clone()))
            .map_err(change_err)?;
        resolved.rejected = reject;
    }
    Ok((Some(bytes), resolved))
}

fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError {
    EditError {
        code: code.to_string(),
        operation: operation.map(str::to_string),
        message: message.into(),
        outcomes: Vec::new(),
    }
}

impl EditPlan {
    /// Parse and validate a plan's JSON (`INVALID_PLAN`, `UNSUPPORTED_SCHEMA`).
    pub fn from_json(json: &str) -> Result<Self, EditError> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| err("INVALID_PLAN", None, e.to_string()))?;
        check_operation_keys(&value)?;
        let plan: Self =
            serde_json::from_value(value).map_err(|e| err("INVALID_PLAN", None, e.to_string()))?;
        if plan.schema_version != SCHEMA_VERSION {
            return Err(err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!(
                    "schema_version {} is not supported (expected {SCHEMA_VERSION})",
                    plan.schema_version
                ),
            ));
        }
        Ok(plan)
    }

    /// Serialize the plan.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("plan serializes")
    }
}

/// Allowed keys per operation kind; serde's `flatten` cannot deny unknown
/// fields itself, and a misspelled key must not silently change meaning.
fn check_operation_keys(plan: &serde_json::Value) -> Result<(), EditError> {
    const COMMON: &[&str] = &["id", "kind", "paragraph"];
    let Some(ops) = plan.get("operations").and_then(|o| o.as_array()) else {
        return Ok(());
    };
    for (i, op) in ops.iter().enumerate() {
        let Some(map) = op.as_object() else {
            return Err(err(
                "INVALID_PLAN",
                None,
                format!("operations[{i}] is not an object"),
            ));
        };
        let kind = map.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let allowed: &[&str] = match kind {
            "replace" => &[
                "find",
                "replacement",
                "format",
                "comment",
                "whole",
                "occurrence",
            ],
            "insert" => &[
                "after",
                "before",
                "position",
                "text",
                "format",
                "comment",
                "occurrence",
            ],
            "delete" => &["find", "occurrence"],
            "comment" => &["find", "text", "through", "occurrence"],
            "insert_paragraph" => &["position", "runs", "like", "style", "comment"],
            "delete_paragraph" => &["comment"],
            "format_paragraph" => &[
                "style",
                "alignment",
                "line_spacing",
                "space_before",
                "space_after",
            ],
            "merge_paragraphs" => &["separator"],
            "rewrite" => &["text"],
            "reply_comment" => &["comment_id", "text"],
            "resolve_comment" => &["comment_id", "done"],
            "edit_comment" => &["comment_id", "text"],
            "delete_comment" => &["comment_id"],
            "insert_table" => &["position", "rows", "header_row", "widths_dxa", "style"],
            "list" if map.contains_key("paragraph") => {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!("operations[{i}] (list): takes paragraphs, not paragraph"),
                ));
            }
            "list" => &["paragraphs", "kind_of_list", "level", "restart"],
            "watermark" => {
                if map.contains_key("paragraph") {
                    return Err(err(
                        "INVALID_PLAN",
                        None,
                        format!("operations[{i}] (watermark): a watermark takes no paragraph"),
                    ));
                }
                &["text", "color", "diagonal", "font"]
            }
            "fill_control" => &["control", "text", "choice", "checked", "date"],
            "format_run" => &["find", "format", "occurrence"],
            "insert_footnote" => &["after", "text", "occurrence"],
            "insert_image" => &[
                "position",
                "image_base64",
                "content_type",
                "width_emu",
                "alt",
            ],
            "page_setup" => &["section", "page", "orientation", "margins_dxa"],
            "insert_toc" => &["position", "levels", "title"],
            "redact" => &["find", "occurrence"],
            "settings" => {
                if map.contains_key("paragraph") {
                    return Err(err(
                        "INVALID_PLAN",
                        None,
                        format!("operations[{i}] (settings): settings take no paragraph"),
                    ));
                }
                &["track_revisions", "update_fields", "protection"]
            }
            other => {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!("operations[{i}]: unknown kind {other:?}"),
                ));
            }
        };
        if kind == "fill_control" && map.contains_key("paragraph") {
            return Err(err(
                "INVALID_PLAN",
                None,
                format!("operations[{i}] (fill_control): selects a control, not a paragraph"),
            ));
        }
        if kind == "page_setup" && map.contains_key("paragraph") {
            return Err(err(
                "INVALID_PLAN",
                None,
                format!("operations[{i}] (page_setup): sections take no paragraph"),
            ));
        }
        for key in map.keys() {
            if !COMMON.contains(&key.as_str()) && !allowed.contains(&key.as_str()) {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!("operations[{i}] ({kind}): unknown field {key:?}"),
                ));
            }
        }
    }
    Ok(())
}

/// Apply a plan given as JSON.
pub fn apply_plan_json(source: &[u8], plan_json: &str) -> Result<EditResult, EditError> {
    apply_plan(source, &EditPlan::from_json(plan_json)?)
}

/// Resolve and apply the plan; compare source and copy into a redline.
pub fn apply_plan(source: &[u8], plan: &EditPlan) -> Result<EditResult, EditError> {
    check_update_fields(plan)?;
    let mut tx = Transaction::start(source, plan)?;
    tx.resolve()?;
    tracked::check(&tx)?;
    tx.apply()?;
    let (mut clean, mut marked) = tx.finish()?;
    if plan.existing_revisions == ExistingRevisions::Keep {
        let result = tracked::result(&tx, clean)?;
        tx.check_redactions(&[&result.clean, &result.redline])?;
        return Ok(result);
    }
    let mut fields = Vec::new();
    if plan.update_fields {
        let refresh = |bytes: &[u8]| {
            crate::fields::update_fields(bytes)
                .map_err(|e| err("FIELDS_FAILED", None, e.to_string()))
        };
        let updated = refresh(&clean)?;
        clean = updated.docx;
        fields = updated.fields;
        if let Some(bytes) = &marked {
            marked = Some(refresh(bytes)?.docx);
        }
    }
    let settings = WmlComparerSettings {
        author_for_revisions: plan.author.clone(),
        date_time_for_revisions: tx.date.clone(),
        ..WmlComparerSettings::default()
    };
    let revised = marked.as_deref().unwrap_or(&clean);
    let base = tx.commented_base()?;
    let mut redline =
        crate::document_comparer::compare_documents_with_settings(&base, revised, &settings)
            .map_err(|e| err("COMPARE_FAILED", None, e.to_string()))?;
    if tx
        .resolved
        .iter()
        .any(|(_, r)| matches!(r, Resolved::PageSetup { .. }))
    {
        redline = sections::record_mid_changes(&redline, &base, &plan.author, &tx.date)?;
    }
    if marked.is_some() {
        let (rewritten, fallbacks) = whole::rewrite(
            &redline,
            (&base, revised),
            &tx.whole_marks,
            &plan.author,
            &tx.date,
        )?;
        redline = rewritten;
        for (op, reason) in fallbacks {
            tx.outcomes[op].message = Some(format!("shown as a word-level diff: {reason}"));
        }
    }
    if let Some(&(_, op)) = tx
        .deletion_comments
        .iter()
        .find(|&&(id, _)| comment_holds_kept_text(&redline, id))
    {
        let mut error = tx.conflict(
            op,
            "the comparer deleted an identical paragraph instead, so the comment would sit on text that stays; delete without a comment, or comment on a neighbouring paragraph",
        );
        error.code = "UNSUPPORTED_STRUCTURE".into();
        error.outcomes[op].code = Some(error.code.clone());
        return Err(error);
    }
    // Settings are not revisions: the redline takes them as they are.
    if let Some(request) = &tx.settings {
        redline = crate::settings::apply_settings_to_docx(&redline, request)
            .map_err(|m| err("PACKAGE_WRITE", None, m))?;
    }
    tx.check_redactions(&[&clean, &redline])?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline, &settings);
    report.fields = fields;
    Ok(EditResult {
        clean,
        redline,
        report,
    })
}

/// `update_fields` refreshes the clean copy the comparer reads. Under
/// `existing_revisions: "keep"` the redline replays the edits instead, so a
/// refreshed clean copy would no longer be the accepted redline.
fn check_update_fields(plan: &EditPlan) -> Result<(), EditError> {
    if plan.update_fields && plan.existing_revisions == ExistingRevisions::Keep {
        return Err(err(
            "INVALID_PLAN",
            None,
            "update_fields cannot be combined with existing_revisions \"keep\"; refresh the fields with `jubarte fields update` after accepting or rejecting the kept changes",
        ));
    }
    Ok(())
}

/// Resolve every operation without producing documents.
pub fn preview_plan(source: &[u8], plan: &EditPlan) -> Result<EditReport, EditError> {
    check_update_fields(plan)?;
    let mut tx = Transaction::start(source, plan)?;
    tx.resolve()?;
    tracked::check(&tx)?;
    let mut report = tx.report(true);
    report.paragraphs.to = report.paragraphs.from;
    Ok(report)
}

fn revision_counts(redline: &[u8], settings: &WmlComparerSettings) -> RevisionCounts {
    let mut counts = RevisionCounts::default();
    if let Ok(revs) = crate::document_comparer::get_revisions(redline, settings) {
        for r in revs {
            match r.revision_type {
                WmlComparerRevisionType::Inserted => counts.inserted += 1,
                WmlComparerRevisionType::Deleted => counts.deleted += 1,
                WmlComparerRevisionType::Moved => counts.moved += 1,
                WmlComparerRevisionType::FormatChanged => counts.format_changed += 1,
            }
            counts.total += 1;
        }
    }
    // get_revisions follows Open-Xml-PowerTools and lists run formatting
    // only; a format_paragraph plan must not report zero changes.
    if let Ok(opened) = Opened::open(redline) {
        let paragraph_changes = opened
            .dom
            .descendants(opened.body, Some(&W::p_pr_change()))
            .len();
        counts.format_changed += paragraph_changes;
        counts.total += paragraph_changes;
        // get_revisions reads the body and notes only; count headers and
        // footers by their revision elements.
        for (_, kind, part) in opened.story_parts() {
            if !matches!(kind, "header" | "footer") {
                continue;
            }
            let Ok((dom, _, root)) = crate::inspect::parse_part(&opened.pkg, &part) else {
                continue;
            };
            for node in dom.descendants(root, None) {
                let Some(name) = dom.name(node) else { continue };
                let slot = match name.local_name() {
                    "ins" => &mut counts.inserted,
                    "del" => &mut counts.deleted,
                    "moveFrom" | "moveTo" => &mut counts.moved,
                    "rPrChange" | "pPrChange" => &mut counts.format_changed,
                    _ => continue,
                };
                if name.namespace_name() == W::URI {
                    *slot += 1;
                    counts.total += 1;
                }
            }
        }
    }
    counts
}

/// A text edit scheduled on one paragraph, in source projection bytes.
#[derive(Clone, Debug)]
struct ScheduledEdit {
    start: usize,
    end: usize,
    /// Index of the operation in the plan.
    op: usize,
    replacement: String,
    attach_before: bool,
    comment: Option<String>,
    format: Option<RunFormat>,
}

/// A resolved operation, in source coordinates (bytes of the projection).
#[derive(Clone, Debug)]
enum Resolved {
    Text {
        para: usize,
        start: usize,
        end: usize,
        /// Plain replacement text (may be empty).
        replacement: String,
        /// Comment text anchored to the changed text.
        comment: Option<String>,
        /// `Insert` attaches to the preceding run when true.
        attach_before: bool,
        /// Formatting of the new text.
        format: Option<RunFormat>,
    },
    CommentRange {
        para: usize,
        start: usize,
        end: usize,
        /// Plain text to insert.
        text: String,
    },
    DeleteParagraph {
        para: usize,
    },
    FormatParagraph {
        para: usize,
        /// Resolved style id.
        style: Option<String>,
        alignment: Option<Alignment>,
        spacing: Spacing,
    },
    MergeParagraphs {
        para: usize,
        next: usize,
        separator: String,
    },
    InsertParagraph {
        anchor: usize,
        side: Side,
        /// Runs of the new paragraph, in order.
        runs: Vec<RunSpec>,
        /// The paragraph whose properties it copies.
        like: usize,
        /// Paragraph style id to set instead of the anchor's.
        style: Option<String>,
        /// Comment text anchored to the changed text.
        comment: Option<String>,
        /// `insert_toc`: the paragraphs are the TOC field (and its title),
        /// not `runs`.
        toc: Option<TocSpec>,
    },
    InsertTable {
        anchor: usize,
        side: Side,
        rows: Vec<Vec<String>>,
        header_row: bool,
        /// One width per column, in twentieths of a point.
        widths: Vec<u32>,
        /// Resolved table style id.
        style: String,
        /// The style's definition must be added to the styles part.
        add_style: bool,
    },
    List {
        /// Paragraphs to number, in plan order.
        paras: Vec<usize>,
        kind: ListKind,
        level: u32,
        /// The `w:numId` to continue, or a new list when `None`.
        join: Option<String>,
        /// The document's "List Paragraph" style id, for unstyled items.
        style: String,
        /// That style's definition must be added to the styles part.
        add_style: bool,
    },
    /// A comment from the start of `para` to the end of `last`.
    CommentSpan {
        para: usize,
        last: usize,
        /// Comment text.
        text: String,
    },
    /// A reply to, resolution, edit or deletion of an existing comment.
    Thread {
        op: ThreadOp,
        /// Stories whose markers the operation adds to or removes.
        stories: Vec<usize>,
        /// Paragraph holding the target comment's reference, when found.
        anchor: Option<usize>,
    },
    FillControl {
        /// Index into `Transaction::controls`.
        control: usize,
        /// The paragraphs it spans (or the one holding it).
        paragraphs: Vec<usize>,
        /// The control holds paragraphs rather than runs.
        block: bool,
        value: controls::FillValue,
    },
    FormatRun {
        para: usize,
        start: usize,
        end: usize,
        format: RunFormat,
    },
    InsertFootnote {
        para: usize,
        /// Projection offset the reference mark follows.
        at: usize,
        text: String,
    },
    InsertImage {
        anchor: usize,
        side: Side,
        picture: crate::markdown::Picture,
    },
    PageSetup {
        targets: sections::Targets,
    },
}

/// What a thread operation does to an existing comment.
#[derive(Clone, Debug)]
enum ThreadOp {
    Reply { parent: u32, text: String },
    Resolve { id: u32, done: bool },
    Edit { id: u32, text: String },
    Delete { id: u32 },
}

impl ThreadOp {
    /// The existing comment the operation acts on.
    fn target(&self) -> u32 {
        match self {
            ThreadOp::Reply { parent: id, .. }
            | ThreadOp::Resolve { id, .. }
            | ThreadOp::Edit { id, .. }
            | ThreadOp::Delete { id } => *id,
        }
    }
}

/// A resolved `insert_toc`.
#[derive(Clone, Debug)]
struct TocSpec {
    levels: u8,
    title: Option<String>,
}

/// What resolving an operation gives: its resolved form and outcome, or the
/// error with the outcome so far.
type Resolution<T> = Result<(T, EditOutcome), Box<(EditError, EditOutcome)>>;

struct Transaction<'p> {
    plan: &'p EditPlan,
    source_sha256: String,
    base: Vec<u8>,
    base_sha256: String,
    resolved_revisions: ResolvedRevisions,
    /// The plan's date, else the time it is applied.
    date: String,
    initials: String,
    opened: Opened,
    /// The body first, then every header, footer and notes part.
    stories: Vec<StoryPart>,
    /// Every addressable paragraph: the body's, then each story's.
    paragraph_nodes: Vec<NodeId>,
    /// `(story, index in that story)` per entry of `paragraph_nodes`.
    paragraph_story: Vec<(usize, usize)>,
    projections: Vec<Projection>,
    outcomes: Vec<EditOutcome>,
    resolved: Vec<(usize, Resolved)>,
    comments: Vec<(u32, String)>,
    /// `(comment id, operation)` of each commented paragraph deletion.
    deletion_comments: Vec<(u32, usize)>,
    /// Ids new comments take, last first, before `next_comment_id`.
    preset_comment_ids: Vec<u32>,
    /// One past the highest source comment id; u64 so it cannot overflow.
    next_comment_id: u64,
    comments_added: usize,
    /// Helper bookmarks around `whole` replacements, one per operation.
    whole_marks: Vec<whole::Mark>,
    /// `w:abstractNum` and `w:num` elements new lists add.
    new_numbering: (Vec<String>, Vec<String>),
    /// Style definitions the edits need (`ListParagraph`).
    needed_styles: std::collections::BTreeSet<String>,
    /// The source's comment part family; taken when the parts are written.
    family: Option<crate::comments::CommentFamily>,
    /// Comment replied to, by the reply's new id.
    reply_parents: BTreeMap<u32, u32>,
    /// The plan's watermark, written into the default headers at finish.
    watermark: Option<watermark::Spec>,
    /// The body's content controls, `body:sdt:N` order.
    controls: Vec<NodeId>,
    /// What `inspect` reports for each of `controls`.
    control_records: Vec<crate::inspect::ContentControl>,
    /// The footnotes story when `insert_footnote` added notes to it.
    notes_story: Option<usize>,
    /// The plan's settings, written into the settings part at finish.
    settings: Option<crate::settings::SettingsRequest>,
}

/// The body, or a header, footer or notes part, parsed into the plan's DOM.
struct StoryPart {
    /// `body`, or the part's file stem (`header1`, `footnotes`).
    id: String,
    /// Package part name.
    part: String,
    document: NodeId,
    root: NodeId,
}

impl<'p> Transaction<'p> {
    fn start(source: &[u8], plan: &'p EditPlan) -> Result<Self, EditError> {
        if plan.schema_version != SCHEMA_VERSION {
            return Err(err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!("schema_version {} is not supported", plan.schema_version),
            ));
        }
        let source_hash = source_sha256(source);
        if let Some(expected) = &plan.source_sha256
            && expected != &source_hash
        {
            return Err(err(
                "STALE_SOURCE",
                None,
                "source bytes do not match the plan's source_sha256",
            ));
        }
        if plan.author.trim().is_empty() {
            return Err(err("INVALID_PLAN", None, "author must be nonempty"));
        }
        let probe = Opened::open(source).map_err(open_error)?;
        if probe
            .pkg
            .parts()
            .iter()
            .any(|name| name.starts_with("_xmlsignatures/") || name.ends_with("vbaProject.bin"))
        {
            return Err(err(
                "UNSUPPORTED_PACKAGE",
                None,
                "signed or macro-bearing package",
            ));
        }
        let (resolved_source, resolved_revisions) =
            resolve_selected(source, plan.resolve_revisions.as_ref())?;
        let (source, probe) = match &resolved_source {
            Some(bytes) => (bytes.as_slice(), Opened::open(bytes).map_err(open_error)?),
            None => (source, probe),
        };
        let story_revisions: usize = probe
            .story_parts()
            .iter()
            .filter_map(|(_, _, part)| crate::inspect::parse_part(&probe.pkg, part).ok())
            .map(|(dom, _, root)| crate::inspect::revision_count(&dom, root))
            .sum();
        let has_revisions =
            crate::inspect::revision_count(&probe.dom, probe.body) + story_revisions > 0;
        let (base, mut opened) = match (has_revisions, plan.existing_revisions) {
            (false, _) | (true, ExistingRevisions::Keep) => (source.to_vec(), probe),
            (true, ExistingRevisions::Refuse) => {
                return Err(err(
                    "EXISTING_REVISIONS",
                    None,
                    "the document already holds tracked changes; set existing_revisions to keep, accept or reject",
                ));
            }
            (true, policy) => {
                let flattened = match policy {
                    ExistingRevisions::Accept => crate::document_comparer::accept_revisions(source),
                    _ => crate::document_comparer::reject_revisions(source),
                }
                .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
                let reopened = Opened::open(&flattened).map_err(open_error)?;
                (flattened, reopened)
            }
        };
        let base_sha256 = source_sha256(&base);
        let mut stories = vec![StoryPart {
            id: BODY_STORY.to_string(),
            part: opened.main.clone(),
            document: opened.document,
            root: opened.body,
        }];
        for (id, _, part) in opened.story_parts() {
            let invalid = |m: String| err("INVALID_DOCUMENT", None, format!("{part}: {m}"));
            let xml = opened
                .pkg
                .part_string(&part)
                .ok_or_else(|| invalid("missing part".into()))?;
            crate::xmllinq::parse::validate_xml(&xml).map_err(|e| invalid(e.to_string()))?;
            let document = opened.dom.parse_xdocument(&xml);
            let root = opened
                .dom
                .root(document)
                .ok_or_else(|| invalid("missing XML root".into()))?;
            stories.push(StoryPart {
                id,
                part,
                document,
                root,
            });
        }
        let mut paragraph_nodes = Vec::new();
        let mut paragraph_story = Vec::new();
        for (index, story) in stories.iter().enumerate() {
            let nodes = if index == 0 {
                crate::inspect::body_paragraph_nodes(&opened.dom, story.root)
            } else {
                crate::inspect::story_paragraph_nodes(&opened.dom, story.root)
            };
            for (local, node) in nodes.into_iter().enumerate() {
                paragraph_nodes.push(node);
                paragraph_story.push((index, local));
            }
        }
        let projections = paragraph_nodes
            .iter()
            .map(|&p| project_paragraph(&opened.dom, p))
            .collect();
        let date = plan
            .date
            .clone()
            .unwrap_or_else(crate::convert::utc_now_iso8601);
        let initials = plan.initials.clone().unwrap_or_else(|| {
            plan.author
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect::<String>()
                .to_uppercase()
        });
        let next_comment_id = existing_comment_ids(&opened).map_or(0, |max| u64::from(max) + 1);
        let controls = crate::inspect::body_control_nodes(&opened.dom, opened.body);
        let control_records = crate::inspect::collect_controls(&opened.dom, opened.body);
        let family = crate::comments::CommentFamily::load(&opened.pkg, &opened.main)
            .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
        Ok(Self {
            plan,
            source_sha256: source_hash,
            base,
            base_sha256,
            resolved_revisions,
            date,
            initials,
            opened,
            stories,
            paragraph_nodes,
            paragraph_story,
            projections,
            outcomes: Vec::new(),
            resolved: Vec::new(),
            comments: Vec::new(),
            deletion_comments: Vec::new(),
            preset_comment_ids: Vec::new(),
            next_comment_id,
            comments_added: 0,
            whole_marks: Vec::new(),
            new_numbering: (Vec::new(), Vec::new()),
            needed_styles: std::collections::BTreeSet::new(),
            family: Some(family),
            reply_parents: BTreeMap::new(),
            watermark: None,
            controls,
            control_records,
            notes_story: None,
            settings: None,
        })
    }

    fn report(&self, ok: bool) -> EditReport {
        EditReport {
            schema_version: SCHEMA_VERSION,
            ok,
            source_sha256: self.source_sha256.clone(),
            base_sha256: self.base_sha256.clone(),
            guarded: self.plan.source_sha256.is_some(),
            author: self.plan.author.clone(),
            date: self.date.clone(),
            existing_revisions: self.plan.existing_revisions,
            resolved_revisions: self.resolved_revisions.clone(),
            paragraphs: ParagraphDelta {
                from: self.body_paragraph_count(),
                to: self.body_paragraph_count(),
            },
            operations: self.outcomes.clone(),
            comments_added: self.comments_added,
            revisions: RevisionCounts::default(),
            fields: Vec::new(),
        }
    }

    /// Resolve every operation against the untouched source; the first
    /// failure is returned after every operation has been tried.
    fn resolve(&mut self) -> Result<(), EditError> {
        let mut first_failure: Option<EditError> = None;
        for (i, op) in self.plan.operations.iter().enumerate() {
            let id = op.id.clone().unwrap_or_else(|| format!("op-{}", i + 1));
            let kind = kind_name(&op.kind).to_string();
            let result = match &op.kind {
                OperationKind::Rewrite { paragraph, text } => {
                    self.resolve_rewrite(&id, paragraph, text)
                }
                OperationKind::List {
                    paragraphs,
                    kind_of_list,
                    level,
                    restart,
                } => self
                    .resolve_list(&id, paragraphs, *kind_of_list, *level, *restart)
                    .map(|(resolved, outcome)| (vec![resolved], outcome)),
                OperationKind::PageSetup {
                    section,
                    page,
                    orientation,
                    margins_dxa,
                } => {
                    let mut outcome = EditOutcome {
                        id: id.clone(),
                        kind: String::new(),
                        status: String::new(),
                        paragraph: None,
                        matches: 0,
                        context: None,
                        comment_id: None,
                        code: None,
                        message: None,
                        anchor_given: None,
                        anchor_read_as: None,
                    };
                    match self.resolve_page_setup(
                        *section,
                        *page,
                        *orientation,
                        *margins_dxa,
                        &mut outcome,
                    ) {
                        Ok(targets) => {
                            outcome.context = Some(format!("{{§ {} section(s)}}", targets.len()));
                            Ok((vec![Resolved::PageSetup { targets }], outcome))
                        }
                        Err((code, message)) => {
                            Err(Box::new((err(&code, Some(&id), message), outcome)))
                        }
                    }
                }
                OperationKind::ReplyComment { .. }
                | OperationKind::ResolveComment { .. }
                | OperationKind::EditComment { .. }
                | OperationKind::DeleteComment { .. } => self
                    .resolve_thread(&id, &op.kind)
                    .map(|(resolved, outcome)| (vec![resolved], outcome)),
                OperationKind::Watermark {
                    text,
                    color,
                    diagonal,
                    font,
                } => self
                    .resolve_watermark(&id, (text, color.as_deref(), *diagonal, font.as_deref()))
                    .map(|(spec, outcome)| {
                        self.watermark = Some(spec);
                        (Vec::new(), outcome)
                    }),
                OperationKind::FillControl {
                    control,
                    text,
                    choice,
                    checked,
                    date,
                } => self.resolve_fill_control(
                    &id,
                    &controls::FillRequest {
                        control,
                        text: text.as_deref(),
                        choice: choice.as_deref(),
                        checked: *checked,
                        date: date.as_deref(),
                    },
                ),
                OperationKind::Settings {
                    track_revisions,
                    update_fields,
                    protection,
                } => {
                    let request = crate::settings::SettingsRequest {
                        track_revisions: *track_revisions,
                        update_fields: *update_fields,
                        protection: protection.clone(),
                    };
                    self.resolve_settings(&id, &request).map(|outcome| {
                        self.settings = Some(request);
                        (Vec::new(), outcome)
                    })
                }
                other => self
                    .resolve_one(&id, other)
                    .map(|(resolved, outcome)| (vec![resolved], outcome)),
            };
            match result {
                Ok((resolved, mut outcome)) => {
                    outcome.id = id;
                    outcome.kind = kind;
                    outcome.status = "ok".into();
                    self.outcomes.push(outcome);
                    self.resolved
                        .extend(resolved.into_iter().map(|resolved| (i, resolved)));
                }
                Err(boxed) => {
                    let (mut e, outcome) = *boxed;
                    e.operation = Some(id.clone());
                    self.outcomes.push(EditOutcome {
                        id,
                        kind,
                        status: "failed".into(),
                        code: Some(e.code.clone()),
                        message: Some(e.message.clone()),
                        ..outcome
                    });
                    first_failure.get_or_insert(e);
                }
            }
        }
        if let Some(mut e) = first_failure {
            e.outcomes = self.outcomes.clone();
            return Err(e);
        }
        self.check_conflicts()?;
        Ok(())
    }

    fn resolve_one(
        &self,
        id: &str,
        kind: &OperationKind,
    ) -> Result<(Resolved, EditOutcome), Box<(EditError, EditOutcome)>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let selector = match kind {
            OperationKind::Replace { paragraph, .. }
            | OperationKind::Insert { paragraph, .. }
            | OperationKind::Delete { paragraph, .. }
            | OperationKind::Comment { paragraph, .. }
            | OperationKind::InsertParagraph { paragraph, .. }
            | OperationKind::DeleteParagraph { paragraph, .. }
            | OperationKind::FormatParagraph { paragraph, .. }
            | OperationKind::MergeParagraphs { paragraph, .. }
            | OperationKind::Rewrite { paragraph, .. }
            | OperationKind::InsertTable { paragraph, .. }
            | OperationKind::InsertToc { paragraph, .. }
            | OperationKind::FormatRun { paragraph, .. }
            | OperationKind::InsertFootnote { paragraph, .. }
            | OperationKind::InsertImage { paragraph, .. }
            | OperationKind::Redact { paragraph, .. } => paragraph,
            OperationKind::List { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "list resolves through resolve_list".into(),
                    outcome,
                ));
            }
            OperationKind::PageSetup { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "page_setup resolves through resolve_page_setup".into(),
                    outcome,
                ));
            }
            OperationKind::ReplyComment { .. }
            | OperationKind::ResolveComment { .. }
            | OperationKind::EditComment { .. }
            | OperationKind::DeleteComment { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "thread operations resolve through resolve_thread".into(),
                    outcome,
                ));
            }
            OperationKind::Watermark { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "watermark resolves through resolve_watermark".into(),
                    outcome,
                ));
            }
            OperationKind::FillControl { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "fill_control resolves through resolve_fill_control".into(),
                    outcome,
                ));
            }
            OperationKind::Settings { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "settings resolve through resolve_settings".into(),
                    outcome,
                ));
            }
        };
        let para = match self.select(selector) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        // `occurrence` picks a hit of an anchor; with no anchor it would be
        // silently ignored.
        let anchorless = matches!(
            kind,
            OperationKind::Insert {
                after: None,
                before: None,
                occurrence: Some(_),
                ..
            } | OperationKind::Comment {
                find: None,
                occurrence: Some(_),
                ..
            }
        );
        if anchorless {
            return Err(fail(
                "INVALID_EDIT",
                "occurrence needs an anchor: give find, after or before".into(),
                outcome,
            ));
        }
        let comments = match kind {
            OperationKind::Replace { comment, .. }
            | OperationKind::Insert { comment, .. }
            | OperationKind::InsertParagraph { comment, .. } => comment.is_some(),
            OperationKind::Comment { .. } => true,
            _ => false,
        };
        if comments && self.paragraph_story[para].0 != 0 {
            return Err(fail(
                "COMMENT_NOT_IN_BODY",
                "comments are supported in the body only: Word cannot anchor one in a header, footer or note".into(),
                outcome,
            ));
        }
        let projection = &self.projections[para];
        let text = &projection.text;
        match kind {
            OperationKind::Replace {
                find,
                replacement,
                format,
                comment,
                occurrence,
                ..
            } => {
                check_text(replacement).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if let Some(format) = format {
                    check_format(format, replacement)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{{}→{}}}", &text[start..end], replacement),
                ));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: replacement.clone(),
                        comment: comment.clone(),
                        attach_before: true,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Redact {
                find, occurrence, ..
            } => {
                // Refusals name the text by its place, never by itself.
                let hide = |m: String| m.replace(&format!("{find:?}"), "the text to redact");
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, hide(m), outcome.clone()))?;
                let blocks = redaction(find);
                outcome.context = Some(context(text, start, end, &format!("{{{blocks}}}")));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: blocks,
                        comment: None,
                        attach_before: true,
                        format: None,
                    },
                    outcome,
                ))
            }
            OperationKind::Delete {
                find, occurrence, ..
            } => {
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{-{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement: String::new(),
                        comment: None,
                        attach_before: true,
                        format: None,
                    },
                    outcome,
                ))
            }
            OperationKind::Insert {
                after,
                before,
                position,
                text: new,
                format,
                comment,
                occurrence,
                ..
            } => {
                check_text(new).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if let Some(format) = format {
                    check_format(format, new)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if new.is_empty() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "text must be nonempty".into(),
                        outcome,
                    ));
                }
                let (pos, attach_before) = match (after, before, position) {
                    (Some(after), None, None) => {
                        let (_, end) = self
                            .find_range(projection, after, *occurrence, &mut outcome)
                            .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                        (end, true)
                    }
                    (None, Some(before), None) => {
                        let (start, _) = self
                            .find_range(projection, before, *occurrence, &mut outcome)
                            .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                        (start, false)
                    }
                    (None, None, Some(Edge::Start)) => {
                        outcome.matches = 1;
                        (0, false)
                    }
                    (None, None, Some(Edge::End)) => {
                        outcome.matches = 1;
                        (text.len(), true)
                    }
                    _ => {
                        return Err(fail(
                            "INVALID_EDIT",
                            "insert needs exactly one of after, before, position".into(),
                            outcome,
                        ));
                    }
                };
                self.check_insert_position(projection, pos, attach_before)
                    .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                outcome.context = Some(context(text, pos, pos, &format!("{{+{new}}}")));
                Ok((
                    Resolved::Text {
                        para,
                        start: pos,
                        end: pos,
                        replacement: new.clone(),
                        comment: comment.clone(),
                        attach_before,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Comment {
                find,
                text: note,
                through: Some(through),
                ..
            } => {
                check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                if find.is_some() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "find and through cannot be combined: through comments on whole paragraphs"
                            .into(),
                        outcome,
                    ));
                }
                let last = match self.select(through) {
                    Ok(p) => p,
                    Err((code, msg, matches)) => {
                        outcome.matches = matches;
                        return Err(fail(&code, format!("through: {msg}"), outcome));
                    }
                };
                if self.paragraph_story[last].0 != self.paragraph_story[para].0 || last < para {
                    return Err(fail(
                        "INVALID_EDIT",
                        format!(
                            "through ({}) must be in the same story as paragraph and not before it",
                            self.paragraph_id(last)
                        ),
                        outcome,
                    ));
                }
                outcome.matches = 1;
                outcome.context = Some(format!(
                    "{{#{} ... {}}}",
                    self.paragraph_id(para),
                    self.paragraph_id(last)
                ));
                Ok((
                    Resolved::CommentSpan {
                        para,
                        last,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::Comment {
                find,
                text: note,
                occurrence,
                ..
            } => {
                check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let (start, end) = match find {
                    Some(find) => self
                        .find_range(projection, find, *occurrence, &mut outcome)
                        .map_err(|(c, m)| fail(&c, m, outcome.clone()))?,
                    None => {
                        outcome.matches = 1;
                        if !text.is_empty() {
                            self.check_range(projection, 0, text.len())
                                .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                        }
                        (0, text.len())
                    }
                };
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{#{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::CommentRange {
                        para,
                        start,
                        end,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::DeleteParagraph { comment, .. } => {
                outcome.matches = 1;
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                    if !text.is_empty() {
                        self.check_range(projection, 0, text.len())
                            .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                    }
                }
                let node = self.paragraph_nodes[para];
                let dom = &self.opened.dom;
                if !dom.descendants(node, Some(&W::sect_pr())).is_empty() {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "paragraph carries section properties".into(),
                        outcome,
                    ));
                }
                if let Some(cell) = dom.ancestors(node, Some(&W::tc())).first()
                    && dom.elements(*cell, Some(&W::p())).len() == 1
                {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "a table cell must keep one paragraph".into(),
                        outcome,
                    ));
                }
                let (container, what) = self.story_container(node);
                let siblings = dom.elements(container, Some(&W::p()));
                if siblings.len() == 1 && siblings[0] == node {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        format!("{what} must keep one paragraph"),
                        outcome,
                    ));
                }
                outcome.context = Some(format!("{{-¶ {}}}", excerpt(text, 60)));
                Ok((Resolved::DeleteParagraph { para }, outcome))
            }
            OperationKind::FormatRun {
                find,
                format,
                occurrence,
                ..
            } => {
                let (start, end) = self
                    .resolve_format_run(projection, find, *occurrence, format, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    start,
                    end,
                    &format!("{{~{}}}", &text[start..end]),
                ));
                Ok((
                    Resolved::FormatRun {
                        para,
                        start,
                        end,
                        format: format.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::InsertFootnote {
                after,
                text: note,
                occurrence,
                ..
            } => {
                let at = self
                    .resolve_footnote(para, projection, after, *occurrence, note, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(context(
                    text,
                    at,
                    at,
                    &format!("{{^{}}}", excerpt(note, 40)),
                ));
                Ok((
                    Resolved::InsertFootnote {
                        para,
                        at,
                        text: note.clone(),
                    },
                    outcome,
                ))
            }
            OperationKind::InsertImage {
                position,
                image_base64,
                content_type,
                width_emu,
                alt,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "pictures can be inserted in the body only".into(),
                        outcome,
                    ));
                }
                let picture = images::picture(
                    image_base64,
                    content_type.as_deref(),
                    *width_emu,
                    alt.as_deref().unwrap_or_default(),
                )
                .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                outcome.context = Some(format!(
                    "{{+¶ picture {} {}x{} EMU}} {}",
                    picture.content_type,
                    picture.width,
                    picture.height,
                    excerpt(text, 40)
                ));
                Ok((
                    Resolved::InsertImage {
                        anchor: para,
                        side: *position,
                        picture,
                    },
                    outcome,
                ))
            }
            OperationKind::Rewrite { .. }
            | OperationKind::List { .. }
            | OperationKind::ReplyComment { .. }
            | OperationKind::ResolveComment { .. }
            | OperationKind::EditComment { .. }
            | OperationKind::DeleteComment { .. }
            | OperationKind::PageSetup { .. } => Err(fail(
                "INVALID_PLAN",
                "rewrite, list, page_setup and thread operations resolve on their own paths".into(),
                outcome,
            )),
            OperationKind::Watermark { .. } => Err(fail(
                "INVALID_PLAN",
                "watermark resolves through resolve_watermark".into(),
                outcome,
            )),
            OperationKind::FillControl { .. } => Err(fail(
                "INVALID_PLAN",
                "fill_control resolves through resolve_fill_control".into(),
                outcome,
            )),
            OperationKind::Settings { .. } => Err(fail(
                "INVALID_PLAN",
                "settings resolve through resolve_settings".into(),
                outcome,
            )),
            OperationKind::InsertParagraph {
                position,
                runs,
                like,
                style,
                comment,
                ..
            } => {
                outcome.matches = 1;
                let like = match like {
                    Some(selector) => self.select(selector).map_err(|(code, msg, _)| {
                        fail(&code, format!("like: {msg}"), outcome.clone())
                    })?,
                    None => para,
                };
                if runs.is_empty() || runs.iter().all(|r| r.text.is_empty()) {
                    return Err(fail("INVALID_EDIT", "runs must carry text".into(), outcome));
                }
                for r in runs {
                    check_text(&r.text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                    check_format(&r.format(), &r.text)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                if let Some(note) = comment {
                    check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let style = match style {
                    Some(style) => Some(
                        self.resolve_style(style)
                            .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?,
                    ),
                    None => None,
                };
                let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
                outcome.context = Some(format!("{{+¶ {}}}", excerpt(&joined, 60)));
                Ok((
                    Resolved::InsertParagraph {
                        anchor: para,
                        side: *position,
                        runs: runs.clone(),
                        like,
                        style,
                        comment: comment.clone(),
                        toc: None,
                    },
                    outcome,
                ))
            }
            OperationKind::FormatParagraph {
                style,
                alignment,
                line_spacing,
                space_before,
                space_after,
                ..
            } => {
                outcome.matches = 1;
                let spacing = Spacing {
                    line: *line_spacing,
                    before: *space_before,
                    after: *space_after,
                };
                if style.is_none() && alignment.is_none() && spacing == Spacing::default() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "format_paragraph needs style, alignment, line_spacing, space_before or space_after".into(),
                        outcome,
                    ));
                }
                let style = match style {
                    Some(style) => Some(
                        self.resolve_style(style)
                            .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?,
                    ),
                    None => None,
                };
                let mut changes = Vec::new();
                if let Some(style) = &style {
                    changes.push(format!("style={style}"));
                }
                if let Some(alignment) = alignment {
                    changes.push(format!("alignment={}", alignment.jc()));
                }
                if let Some(LineSpacing(line)) = spacing.line {
                    changes.push(format!("line_spacing={}", f64::from(line) / 240.0));
                }
                if let Some(Points(before)) = spacing.before {
                    changes.push(format!("space_before={}pt", f64::from(before) / 20.0));
                }
                if let Some(Points(after)) = spacing.after {
                    changes.push(format!("space_after={}pt", f64::from(after) / 20.0));
                }
                outcome.context =
                    Some(format!("{{¶ {}}} {}", changes.join(" "), excerpt(text, 40)));
                Ok((
                    Resolved::FormatParagraph {
                        para,
                        style,
                        alignment: *alignment,
                        spacing,
                    },
                    outcome,
                ))
            }
            OperationKind::MergeParagraphs { separator, .. } => {
                outcome.matches = 1;
                let separator = separator.clone().unwrap_or_default();
                check_text(&separator).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let next = self
                    .merge_partner(para)
                    .map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
                let tail: String = text
                    .chars()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                outcome.context = Some(format!(
                    "{tail}{{¶→{separator}}}{}",
                    excerpt(&self.projections[next].text, 20)
                ));
                Ok((
                    Resolved::MergeParagraphs {
                        para,
                        next,
                        separator,
                    },
                    outcome,
                ))
            }
            OperationKind::InsertTable {
                position,
                rows,
                header_row,
                widths_dxa,
                style,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "insert_table is supported in the body only".into(),
                        outcome,
                    ));
                }
                let dom = &self.opened.dom;
                if !dom
                    .ancestors(self.paragraph_nodes[para], Some(&W::tc()))
                    .is_empty()
                {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "the anchor paragraph is in a table cell; nested tables are not supported"
                            .into(),
                        outcome,
                    ));
                }
                let columns = structural::check_rows(rows, widths_dxa.as_deref())
                    .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let widths = match widths_dxa {
                    Some(widths) => widths.clone(),
                    None => {
                        let each = structural::text_width(dom, self.opened.body)
                            / u32::try_from(columns).unwrap_or(1);
                        vec![each; columns]
                    }
                };
                let (style, add_style) = structural::resolve_table_style(
                    &structural::table_styles(&self.opened),
                    style.as_deref().unwrap_or("TableGrid"),
                )
                .map_err(|m| fail("UNKNOWN_STYLE", m, outcome.clone()))?;
                outcome.context = Some(format!(
                    "{{+table {}x{columns}}} {}",
                    rows.len(),
                    excerpt(&rows[0].join(" | "), 60)
                ));
                Ok((
                    Resolved::InsertTable {
                        anchor: para,
                        side: *position,
                        rows: rows.clone(),
                        header_row: *header_row,
                        widths,
                        style,
                        add_style,
                    },
                    outcome,
                ))
            }
            OperationKind::InsertToc {
                position,
                levels,
                title,
                ..
            } => {
                outcome.matches = 1;
                if self.paragraph_story[para].0 != 0 {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "a table of contents goes in the body".into(),
                        outcome,
                    ));
                }
                if !(1..=9).contains(levels) {
                    return Err(fail(
                        "INVALID_EDIT",
                        format!("levels {levels} is outside 1..=9"),
                        outcome,
                    ));
                }
                if let Some(title) = title {
                    if title.is_empty() {
                        return Err(fail(
                            "INVALID_EDIT",
                            "title must carry text".into(),
                            outcome,
                        ));
                    }
                    check_text(title).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                outcome.context = Some(format!("{{+TOC 1-{levels}}}"));
                if !self.plan.update_fields {
                    outcome.message = Some(
                        "the TOC has no entries until its fields are updated: set \"update_fields\": true, or update fields in Word".into(),
                    );
                }
                Ok((
                    Resolved::InsertParagraph {
                        anchor: para,
                        side: *position,
                        runs: Vec::new(),
                        like: para,
                        style: None,
                        comment: None,
                        toc: Some(TocSpec {
                            levels: *levels,
                            title: title.clone(),
                        }),
                    },
                    outcome,
                ))
            }
        }
    }

    /// The paragraph `para` merges with: the next sibling paragraph, with only
    /// range markup (bookmarks, comment ranges, ...) between them. `para`'s
    /// properties are discarded, so it may not carry a section break.
    /// `reply_comment`, `resolve_comment`, `edit_comment`, `delete_comment`:
    /// the comment must exist (`UNKNOWN_COMMENT`); a reply needs the
    /// comment's reference to anchor on.
    fn resolve_thread(&self, id: &str, kind: &OperationKind) -> Resolution<Resolved> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let op = match kind {
            OperationKind::ReplyComment { comment_id, text } => ThreadOp::Reply {
                parent: *comment_id,
                text: text.clone(),
            },
            OperationKind::ResolveComment { comment_id, done } => ThreadOp::Resolve {
                id: *comment_id,
                done: *done,
            },
            OperationKind::EditComment { comment_id, text } => ThreadOp::Edit {
                id: *comment_id,
                text: text.clone(),
            },
            OperationKind::DeleteComment { comment_id } => ThreadOp::Delete { id: *comment_id },
            _ => {
                return Err(fail(
                    "INVALID_PLAN",
                    "not a thread operation".into(),
                    outcome,
                ));
            }
        };
        let target = op.target();
        let family = self.family.as_ref().expect("loaded in start");
        if !family.contains(target) {
            return Err(fail(
                "UNKNOWN_COMMENT",
                format!("no comment {target} in the document (jubarte comments lists them)"),
                outcome,
            ));
        }
        if let ThreadOp::Reply { text, .. } | ThreadOp::Edit { text, .. } = &op {
            check_comment(text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        }
        let ids = match &op {
            ThreadOp::Delete { id } => family.with_replies(*id),
            _ => vec![target],
        };
        let (stories, reference) = self.comment_markers(&ids, target);
        let anchor = reference.and_then(|r| {
            self.opened
                .dom
                .ancestors(r, Some(&W::p()))
                .into_iter()
                .find_map(|p| self.paragraph_nodes.iter().position(|&n| n == p))
        });
        if matches!(op, ThreadOp::Reply { .. }) && reference.is_none() {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("comment {target} has no reference in the document to anchor a reply on"),
                outcome,
            ));
        }
        outcome.matches = 1;
        outcome.paragraph = anchor.map(|p| self.paragraph_id(p));
        if !matches!(op, ThreadOp::Reply { .. }) {
            outcome.comment_id = Some(target);
        }
        outcome.context = Some(format!("comment {target}"));
        let stories = match op {
            ThreadOp::Resolve { .. } | ThreadOp::Edit { .. } => Vec::new(),
            _ => stories,
        };
        Ok((
            Resolved::Thread {
                op,
                stories,
                anchor,
            },
            outcome,
        ))
    }

    /// Stories holding markers of the comments `ids`, and the reference
    /// element of comment `target`.
    fn comment_markers(&self, ids: &[u32], target: u32) -> (Vec<usize>, Option<NodeId>) {
        let dom = &self.opened.dom;
        let wanted: Vec<String> = ids.iter().map(u32::to_string).collect();
        let target = target.to_string();
        let mut stories = Vec::new();
        let mut reference = None;
        for (index, story) in self.stories.iter().enumerate() {
            for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
                for marker in dom.descendants(story.root, Some(&W::name(local))) {
                    let Some(value) = dom.attribute(marker, &W::id()) else {
                        continue;
                    };
                    if !wanted.iter().any(|w| w == value) {
                        continue;
                    }
                    if !stories.contains(&index) {
                        stories.push(index);
                    }
                    if local == "commentReference" && value == target && reference.is_none() {
                        reference = Some(marker);
                    }
                }
            }
        }
        (stories, reference)
    }

    /// `rewrite`: the smallest word-level edits that make the paragraph read
    /// as `new_text`, each checked as a `replace` or an `insert` is.
    fn resolve_rewrite(
        &self,
        id: &str,
        paragraph: &Selector,
        new_text: &str,
    ) -> Resolution<Vec<Resolved>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        let para = match self.select(paragraph) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        let new_text: String = new_text
            .chars()
            .map(|c| {
                if matches!(c, '\t' | '\n' | '\r') {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        check_text(&new_text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        let projection = &self.projections[para];
        let text = &projection.text;
        let edits = rewrite::rewrite_ranges(text, &new_text);
        let mut resolved = Vec::with_capacity(edits.len());
        for (start, end, replacement) in edits {
            // New text joins the run before it, unless a tab, break or
            // symbol is there.
            let attach_before = text[..start]
                .chars()
                .next_back()
                .is_some_and(|c| !matches!(c, '\t' | '\n' | '\r' | '\u{FFFC}'));
            let checked = if start == end {
                self.check_insert_position(projection, start, attach_before)
            } else {
                self.check_range(projection, start, end)
            };
            checked.map_err(|m| fail("UNSUPPORTED_STRUCTURE", m, outcome.clone()))?;
            resolved.push(Resolved::Text {
                para,
                start,
                end,
                replacement,
                comment: None,
                attach_before,
                format: None,
            });
        }
        outcome.matches = 1;
        outcome.context = Some(format!("{{≡ {}}}", excerpt(&new_text, 60)));
        Ok((resolved, outcome))
    }

    /// `list`: every selector matches one body paragraph, once; `restart:
    /// false` finds the list to continue.
    fn resolve_list(
        &self,
        id: &str,
        selectors: &[Selector],
        kind: ListKind,
        level: u32,
        restart: bool,
    ) -> Resolution<Resolved> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if selectors.is_empty() {
            return Err(fail(
                "INVALID_EDIT",
                "paragraphs must select at least one paragraph".into(),
                outcome,
            ));
        }
        if level > 8 {
            return Err(fail(
                "INVALID_EDIT",
                format!("level {level} is outside 0..=8"),
                outcome,
            ));
        }
        let mut paras: Vec<usize> = Vec::with_capacity(selectors.len());
        for (i, selector) in selectors.iter().enumerate() {
            let para = match self.select(selector) {
                Ok(p) => p,
                Err((code, msg, matches)) => {
                    outcome.matches = matches;
                    return Err(fail(&code, format!("paragraphs[{i}]: {msg}"), outcome));
                }
            };
            if paras.contains(&para) {
                return Err(fail(
                    "INVALID_EDIT",
                    format!("paragraphs[{i}] selects {} again", self.paragraph_id(para)),
                    outcome,
                ));
            }
            if self.paragraph_story[para].0 != 0 {
                return Err(fail(
                    "UNSUPPORTED_STRUCTURE",
                    format!(
                        "paragraphs[{i}]: list is supported in the body only, not {}",
                        self.paragraph_id(para)
                    ),
                    outcome,
                ));
            }
            paras.push(para);
        }
        outcome.matches = paras.len();
        outcome.paragraph = Some(
            paras
                .iter()
                .map(|&p| self.paragraph_id(p))
                .collect::<Vec<_>>()
                .join(", "),
        );
        let join = if restart {
            None
        } else {
            let first = paras.iter().copied().min().unwrap_or(0);
            let found = (0..first)
                .rev()
                .find_map(|p| structural::direct_num_id(&self.opened.dom, self.paragraph_nodes[p]));
            match found {
                Some(num_id) => Some(num_id),
                None => {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "restart: false continues the list of a numbered paragraph before the first one, and there is none".into(),
                        outcome,
                    ));
                }
            }
        };
        // The document's own "List Paragraph" (localized documents give it
        // another id), else the engine's, added when used.
        let styles = self.paragraph_styles();
        let (style, add_style) = styles
            .iter()
            .find(|(id, _)| id == "ListParagraph")
            .or_else(|| {
                styles
                    .iter()
                    .find(|(_, name)| name.eq_ignore_ascii_case("List Paragraph"))
            })
            .map_or(("ListParagraph".to_string(), true), |(id, _)| {
                (id.clone(), false)
            });
        let how = match &join {
            Some(num_id) => format!("continues list {num_id}"),
            None => format!("list {}", kind.name()),
        };
        outcome.context = Some(format!(
            "{{¶ {how} level {level}}} {} paragraph(s): {}",
            paras.len(),
            excerpt(&self.projections[paras[0]].text, 40)
        ));
        Ok((
            Resolved::List {
                paras,
                kind,
                level,
                join,
                style,
                add_style,
            },
            outcome,
        ))
    }

    fn merge_partner(&self, para: usize) -> Result<usize, String> {
        let dom = &self.opened.dom;
        let node = self.paragraph_nodes[para];
        let mut next_node = dom.next_element(node);
        while let Some(n) = next_node.filter(|&n| is_range_markup(dom, n)) {
            next_node = dom.next_element(n);
        }
        let next_node = next_node
            .filter(|&n| dom.name_is(n, &W::p()))
            .ok_or_else(|| {
                "the next element is not a paragraph in the same container".to_string()
            })?;
        let next = self
            .paragraph_nodes
            .iter()
            .position(|&n| n == next_node)
            .ok_or_else(|| "the next paragraph is not addressable".to_string())?;
        let ppr = dom.element(node, &W::p_pr());
        if ppr.is_some_and(|ppr| dom.element(ppr, &W::sect_pr()).is_some()) {
            return Err("the paragraph carries section properties a merge would drop".into());
        }
        Ok(next)
    }

    /// A paragraph style id from its id or its name (case-insensitive).
    fn resolve_style(&self, requested: &str) -> Result<String, String> {
        let styles = self.paragraph_styles();
        if styles.iter().any(|(id, _)| id == requested) {
            return Ok(requested.to_string());
        }
        if let Some((id, _)) = styles
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(requested))
        {
            return Ok(id.clone());
        }
        let known: Vec<&str> = styles.iter().map(|(id, _)| id.as_str()).take(12).collect();
        Err(format!(
            "no paragraph style has id or name {requested:?}; defined: {}",
            known.join(", ")
        ))
    }

    /// `(styleId, name)` of every paragraph style in the styles part.
    fn paragraph_styles(&self) -> Vec<(String, String)> {
        let Some(name) = self.opened.related("styles").into_iter().next() else {
            return Vec::new();
        };
        let Some(xml) = self.opened.pkg.part_string(&name) else {
            return Vec::new();
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            return Vec::new();
        };
        dom.elements(root, Some(&W::name("style")))
            .into_iter()
            .filter(|&s| dom.attribute(s, &W::name("type")).unwrap_or("paragraph") == "paragraph")
            .filter_map(|s| {
                let id = dom.attribute(s, &W::name("styleId"))?.to_string();
                let name = dom
                    .element(s, &W::name("name"))
                    .and_then(|n| dom.attribute(n, &W::val()))
                    .unwrap_or("")
                    .to_string();
                Some((id, name))
            })
            .collect()
    }

    /// The long id of a short one, as the agent view prints them: `p3` →
    /// `body:p:3`; `header1` and `header1.p1` → `header1:p:0` and
    /// `header1:p:1` (the part stem, Word's own numbering); `footer2`
    /// likewise; `t0.r1.c2` and `t0.r1.c2.p1` → that cell's first (or K-th)
    /// paragraph. `None` when `id` is not a short id.
    fn long_id(&self, id: &str) -> Option<Result<String, String>> {
        let (head, rest): (&str, Vec<&str>) = match id.split_once('.') {
            Some((head, rest)) => (head, rest.split('.').collect()),
            None => (id, Vec::new()),
        };
        if let Some(n) = short_number(head, 'p') {
            return rest.is_empty().then(|| Ok(format!("body:p:{n}")));
        }
        if let Some(n) = short_number(head, 't') {
            return Some(self.table_paragraph(head, n, &rest));
        }
        let is_part = ["header", "footer"].iter().any(|kind| {
            head.strip_prefix(kind)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        if !is_part {
            return None;
        }
        let index = match rest.as_slice() {
            [] => 0,
            [p] => short_number(p, 'p')?,
            _ => return None,
        };
        Some(if self.stories.iter().any(|s| s.id == head) {
            Ok(format!("{head}:p:{index}"))
        } else {
            let known: Vec<&str> = self
                .stories
                .iter()
                .map(|s| s.id.as_str())
                .filter(|s| s.starts_with("header") || s.starts_with("footer"))
                .collect();
            Err(format!(
                "{head} is not a part of this document (headers and footers: {})",
                if known.is_empty() {
                    "none".to_string()
                } else {
                    known.join(", ")
                }
            ))
        })
    }

    /// `t{n}.r{R}.c{C}[.p{K}]`: the long id of that cell's K-th own
    /// paragraph. Tables are the body's top-level `w:tbl` elements outside
    /// text boxes, in document order (nested tables are not numbered, as in
    /// the agent view); rows and cells count as they appear in the XML, a
    /// merged cell once.
    fn table_paragraph(&self, head: &str, n: usize, rest: &[&str]) -> Result<String, String> {
        let dom = &self.opened.dom;
        let (tc, tr, tbl, p, txbx) = (W::tc(), W::name("tr"), W::tbl(), W::p(), W::txbx_content());
        let nearest = |node: NodeId, name: &crate::xmllinq::XName| {
            dom.ancestors(node, Some(name)).first().copied()
        };
        let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let tables: Vec<NodeId> = dom
            .descendants(self.opened.body, Some(&tbl))
            .into_iter()
            .filter(|&t| nearest(t, &tc).is_none() && nearest(t, &txbx).is_none())
            .collect();
        let Some(&table) = tables.get(n) else {
            return Err(format!(
                "{head} is not a table of this document ({})",
                plural(tables.len(), "table")
            ));
        };
        let (row, cell, index) = match rest {
            [r, c] => (short_number(r, 'r'), short_number(c, 'c'), Some(0)),
            [r, c, k] => (
                short_number(r, 'r'),
                short_number(c, 'c'),
                short_number(k, 'p'),
            ),
            _ => (None, None, None),
        };
        let (Some(row), Some(cell), Some(index)) = (row, cell, index) else {
            return Err(format!(
                "{head}: a table id needs a row and a cell, as t0.r1.c2"
            ));
        };
        let rows: Vec<NodeId> = dom
            .descendants(table, Some(&tr))
            .into_iter()
            .filter(|&r| nearest(r, &tbl) == Some(table))
            .collect();
        let Some(&row_node) = rows.get(row) else {
            return Err(format!(
                "{head} has {}, no row {row}",
                plural(rows.len(), "row")
            ));
        };
        let cells: Vec<NodeId> = dom
            .descendants(row_node, Some(&tc))
            .into_iter()
            .filter(|&c| nearest(c, &tr) == Some(row_node))
            .collect();
        let Some(&cell_node) = cells.get(cell) else {
            return Err(format!(
                "{head}.r{row} has {}, no cell {cell}",
                plural(cells.len(), "cell")
            ));
        };
        let own: Vec<NodeId> = dom
            .descendants(cell_node, Some(&p))
            .into_iter()
            .filter(|&q| nearest(q, &tc) == Some(cell_node))
            .collect();
        let Some(&node) = own.get(index) else {
            return Err(format!(
                "{head}.r{row}.c{cell} has {}, no p{index}",
                plural(own.len(), "paragraph")
            ));
        };
        let global = self
            .paragraph_nodes
            .iter()
            .position(|&q| q == node)
            .ok_or_else(|| {
                format!("{head}: that paragraph is inside a text box and cannot be edited")
            })?;
        let (story, in_story) = self.paragraph_story[global];
        Ok(format!("{}:p:{in_story}", self.stories[story].id))
    }

    fn select(&self, selector: &Selector) -> Result<usize, (String, String, usize)> {
        let not_found = |message: String| Err(("ANCHOR_NOT_FOUND".to_string(), message, 0));
        let (story, index) = match selector {
            Selector::Name(id) | Selector::Id { id } => {
                let long = match self.long_id(id) {
                    Some(Ok(long)) => long,
                    Some(Err(message)) => return not_found(message),
                    None => id.clone(),
                };
                match long
                    .rsplit_once(":p:")
                    .and_then(|(story, n)| Some((story.to_string(), n.parse::<usize>().ok()?)))
                {
                    Some((story, n)) => (story, Some(n)),
                    None => return not_found(format!("unknown paragraph id {id}")),
                }
            }
            Selector::Index { index, story } => (
                story.as_deref().unwrap_or(BODY_STORY).to_string(),
                Some(*index),
            ),
            Selector::StartsWith { story, .. } | Selector::Contains { story, .. } => {
                (story.as_deref().unwrap_or(BODY_STORY).to_string(), None)
            }
        };
        let Some(story_index) = self.stories.iter().position(|s| s.id == story) else {
            let known: Vec<&str> = self.stories.iter().map(|s| s.id.as_str()).collect();
            return not_found(format!(
                "unknown story {story:?}; this document has {}",
                known.join(", ")
            ));
        };
        let members: Vec<usize> = (0..self.paragraph_nodes.len())
            .filter(|&g| self.paragraph_story[g].0 == story_index)
            .collect();
        if let Some(index) = index {
            return match members.get(index) {
                Some(&g) => Ok(g),
                None => not_found(format!(
                    "paragraph index {index} does not exist in {story} ({} paragraphs)",
                    members.len()
                )),
            };
        }
        let (wanted, prefix) = match selector {
            Selector::StartsWith { starts_with, .. } => (starts_with, true),
            Selector::Contains { contains, .. } => (contains, false),
            _ => unreachable!("ids and indexes returned above"),
        };
        if wanted.is_empty() {
            return Err((
                "INVALID_EDIT".into(),
                "paragraph selector text must be nonempty".into(),
                0,
            ));
        }
        let hits: Vec<usize> = members
            .into_iter()
            .filter(|&g| {
                let text = &self.projections[g].text;
                if prefix {
                    text.starts_with(wanted.as_str())
                } else {
                    text.contains(wanted.as_str())
                }
            })
            .collect();
        match hits.as_slice() {
            [one] => Ok(*one),
            [] => not_found(format!("no paragraph matches {wanted:?}")),
            many => Err((
                "AMBIGUOUS_ANCHOR".into(),
                format!(
                    "{} paragraphs match {wanted:?}: {}",
                    many.len(),
                    many.iter()
                        .map(|&g| self.paragraph_id(g))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                many.len(),
            )),
        }
    }

    fn body_paragraph_count(&self) -> usize {
        self.paragraph_story.iter().filter(|(s, _)| *s == 0).count()
    }

    /// Stories an operation of the plan edits.
    fn touched_stories(&self) -> std::collections::BTreeSet<usize> {
        self.resolved
            .iter()
            .flat_map(|(_, r)| match r {
                Resolved::PageSetup { .. } => Vec::new(),
                Resolved::Text { para, .. }
                | Resolved::CommentRange { para, .. }
                | Resolved::CommentSpan { para, .. }
                | Resolved::DeleteParagraph { para }
                | Resolved::FormatParagraph { para, .. }
                | Resolved::MergeParagraphs { para, .. }
                | Resolved::FormatRun { para, .. }
                | Resolved::InsertFootnote { para, .. } => vec![self.paragraph_story[*para].0],
                Resolved::InsertParagraph { anchor, .. }
                | Resolved::InsertTable { anchor, .. }
                | Resolved::InsertImage { anchor, .. } => {
                    vec![self.paragraph_story[*anchor].0]
                }
                Resolved::List { paras, .. } => vec![self.paragraph_story[paras[0]].0],
                Resolved::Thread { stories, .. } => stories.clone(),
                Resolved::FillControl { .. } => vec![0],
            })
            .collect()
    }

    /// `{story}:p:{index}` of a paragraph.
    fn paragraph_id(&self, para: usize) -> String {
        let (story, local) = self.paragraph_story[para];
        format!("{}:p:{local}", self.stories[story].id)
    }

    /// The body, header, footer or note a paragraph belongs to, for the
    /// "keeps one paragraph" checks.
    fn story_container(&self, node: NodeId) -> (NodeId, &'static str) {
        let dom = &self.opened.dom;
        for (name, what) in [
            ("footnote", "a footnote"),
            ("endnote", "an endnote"),
            ("hdr", "a header"),
            ("ftr", "a footer"),
        ] {
            if let Some(&found) = dom.ancestors(node, Some(&W::name(name))).first() {
                return (found, what);
            }
        }
        (self.opened.body, "the body")
    }

    /// The unique occurrence of `find`, or its `occurrence`-th hit (1-based)
    /// when given (overlapping occurrences count), checked to lie within
    /// editable direct text.
    fn find_range(
        &self,
        projection: &Projection,
        find: &str,
        occurrence: Option<usize>,
        outcome: &mut EditOutcome,
    ) -> Result<(usize, usize), (String, String)> {
        if find.is_empty() {
            return Err(("INVALID_EDIT".into(), "find must be nonempty".into()));
        }
        let text = &projection.text;
        let hits_of = |needle: &str| -> Vec<usize> {
            text.char_indices()
                .map(|(i, _)| i)
                .filter(|&i| text[i..].starts_with(needle))
                .collect()
        };
        let mut needle = find.to_string();
        let mut hits = hits_of(find);
        // An anchor copied out of the agent view can carry Markdown marks
        // (`# `, `**`, CriticMarkup notes) that are not document text: the
        // literal is tried first, then the text without them.
        if hits.is_empty() {
            let plain = crate::markdown::plain_anchor(find);
            if !plain.is_empty() && plain != find {
                let plain_hits = hits_of(&plain);
                if !plain_hits.is_empty() {
                    outcome.anchor_given = Some(find.to_string());
                    outcome.anchor_read_as = Some(plain.clone());
                    needle = plain;
                    hits = plain_hits;
                }
            }
        }
        outcome.matches = hits.len();
        let start = match (hits.as_slice(), occurrence) {
            ([], _) => {
                return Err((
                    "ANCHOR_NOT_FOUND".into(),
                    format!("{find:?} does not occur in the paragraph"),
                ));
            }
            (_, Some(0)) => {
                return Err(("INVALID_EDIT".into(), "occurrence is 1-based".into()));
            }
            ([one], None) => *one,
            (many, None) => {
                let n = many.len();
                return Err((
                    "AMBIGUOUS_ANCHOR".into(),
                    format!(
                        "{find:?} occurs {n} times in the paragraph; set \"occurrence\" to 1..={n}"
                    ),
                ));
            }
            (many, Some(k)) if k <= many.len() => many[k - 1],
            (many, Some(k)) => {
                let n = many.len();
                return Err((
                    "AMBIGUOUS_ANCHOR".into(),
                    format!(
                        "{find:?} occurs {n} times in the paragraph; occurrence {k} is outside occurrence 1..={n}"
                    ),
                ));
            }
        };
        let end = start + needle.len();
        self.check_range(projection, start, end)
            .map_err(|m| ("UNSUPPORTED_STRUCTURE".to_string(), m))?;
        Ok((start, end))
    }

    /// Every byte of `[start, end)` must come from a `w:t` of a direct run.
    fn check_range(&self, projection: &Projection, start: usize, end: usize) -> Result<(), String> {
        let mut covered = start;
        for seg in &projection.segments {
            if seg.end <= covered || seg.start >= end {
                continue;
            }
            if !seg.direct {
                return Err(
                    "the text sits inside a hyperlink, field, content control or revision".into(),
                );
            }
            if !matches!(seg.piece, Piece::Text { .. }) {
                return Err("the text crosses a tab, break or symbol".into());
            }
            covered = covered.max(seg.end);
        }
        if covered < end {
            return Err("the text is not addressable".into());
        }
        if projection
            .field_marks
            .iter()
            .any(|&mark| start < mark && mark < end)
        {
            return Err(
                "the text sits inside a hyperlink, field, content control or revision".into(),
            );
        }
        Ok(())
    }

    fn check_insert_position(
        &self,
        projection: &Projection,
        pos: usize,
        attach_before: bool,
    ) -> Result<(), String> {
        if projection.text.is_empty() {
            return Err("cannot insert text into an empty paragraph; use insert_paragraph".into());
        }
        let seg = attach_segment(projection, pos, attach_before)
            .ok_or_else(|| "no run holds the insertion point".to_string())?;
        if !seg.direct {
            return Err(
                "the insertion point sits inside a hyperlink, field, content control or revision"
                    .into(),
            );
        }
        if !matches!(seg.piece, Piece::Text { .. }) {
            return Err("the insertion point touches a tab, break or symbol".into());
        }
        Ok(())
    }

    fn check_conflicts(&self) -> Result<(), EditError> {
        let mut deleted: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                if deleted.contains(para) {
                    return Err(self.conflict(*i, "deletes a paragraph another operation deletes"));
                }
                deleted.push(*para);
            }
        }
        self.check_deletions_leave_valid_containers(&deleted)?;
        self.check_paragraph_ops(&deleted)?;
        self.check_control_conflicts(&deleted)?;
        self.check_comment_ids_fit()?;
        self.check_thread_ops()?;
        let mut ranges: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
        for (i, r) in &self.resolved {
            let (para, start, end) = match r {
                Resolved::Text {
                    para, start, end, ..
                } => (*para, *start, *end),
                Resolved::CommentRange { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "comments on a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::InsertParagraph { anchor, .. } | Resolved::InsertImage { anchor, .. } => {
                    if deleted.contains(anchor) {
                        return Err(
                            self.conflict(*i, "anchors a new paragraph on a deleted paragraph")
                        );
                    }
                    continue;
                }
                Resolved::InsertTable { anchor, .. } => {
                    if deleted.contains(anchor) {
                        return Err(self.conflict(*i, "anchors a new table on a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::FormatRun { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "formats text of a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::InsertFootnote { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "adds a footnote to a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::CommentSpan { para, last, .. } => {
                    if (*para..=*last).any(|p| deleted.contains(&p)) {
                        return Err(self.conflict(*i, "comments on a deleted paragraph"));
                    }
                    continue;
                }
                Resolved::Thread {
                    op: ThreadOp::Reply { .. },
                    anchor: Some(anchor),
                    ..
                } if deleted.contains(anchor) => {
                    return Err(self.conflict(
                        *i,
                        "replies to a comment whose reference is in a deleted paragraph",
                    ));
                }
                Resolved::DeleteParagraph { .. }
                | Resolved::FormatParagraph { .. }
                | Resolved::MergeParagraphs { .. }
                | Resolved::List { .. }
                | Resolved::PageSetup { .. }
                | Resolved::Thread { .. }
                | Resolved::FillControl { .. } => continue,
            };
            if deleted.contains(&para) {
                return Err(self.conflict(*i, "edits text of a deleted paragraph"));
            }
            ranges.entry(para).or_default().push((start, end, *i));
        }
        for list in ranges.values_mut() {
            list.sort_by_key(|&(s, e, i)| (s, e, i));
            for &[(s0, e0, _), (s1, e1, i1)] in list.array_windows() {
                let strictly_inside = s1 > s0 && s1 < e0;
                let same_start_nonempty = s1 == s0 && e0 > s0 && e1 > s1;
                if strictly_inside || same_start_nonempty {
                    return Err(self.conflict(i1, "overlaps an earlier edit's text range"));
                }
            }
        }
        // A comment boundary strictly inside a changed range has no position
        // in the edited text. Containing or touching an edit is fine.
        for (i, r) in &self.resolved {
            let Resolved::CommentRange {
                para, start, end, ..
            } = r
            else {
                continue;
            };
            let cuts =
                ranges.get(para).into_iter().flatten().any(|&(s, e, _)| {
                    e > s && ((*start > s && *start < e) || (*end > s && *end < e))
                });
            if cuts {
                return Err(self.conflict(*i, "comment range cuts through an edited range"));
            }
        }
        // Formatting applies to source text: no text edit may change it.
        for (i, r) in &self.resolved {
            let Resolved::FormatRun {
                para, start, end, ..
            } = r
            else {
                continue;
            };
            let overlaps = ranges
                .get(para)
                .into_iter()
                .flatten()
                .any(|&(s, e, _)| runs::format_overlaps_edit((*start, *end), (s, e)));
            if overlaps {
                return Err(self.conflict(*i, "formats text another operation changes"));
            }
        }
        // One page setup per section.
        let setups: Vec<(usize, &sections::Targets)> = self
            .resolved
            .iter()
            .filter_map(|(i, r)| match r {
                Resolved::PageSetup { targets } => Some((*i, targets)),
                _ => None,
            })
            .collect();
        for (n, (i, targets)) in setups.iter().enumerate() {
            if setups[..n]
                .iter()
                .any(|(_, earlier)| sections::targets_overlap(earlier, targets))
            {
                return Err(self.conflict(*i, "sets up a section another operation sets up"));
            }
        }
        // A reference mark needs its anchor's end to survive the text edits.
        for (i, r) in &self.resolved {
            let Resolved::InsertFootnote { para, at, .. } = r else {
                continue;
            };
            let inside = ranges
                .get(para)
                .into_iter()
                .flatten()
                .any(|&(s, e, _)| e > s && runs::format_overlaps_edit((*at, *at), (s, e)));
            if inside {
                return Err(
                    self.conflict(*i, "puts a footnote inside text another operation changes")
                );
            }
        }
        Ok(())
    }

    /// Formatting and merging, checked against each other, against deletions
    /// and against paragraph insertions.
    fn check_paragraph_ops(&self, deleted: &[usize]) -> Result<(), EditError> {
        let mut formatted: Vec<usize> = Vec::new();
        let mut merge_heads: Vec<usize> = Vec::new();
        let mut merge_tails: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            match r {
                Resolved::FormatParagraph { para, .. } => {
                    if deleted.contains(para) {
                        return Err(self.conflict(*i, "formats a deleted paragraph"));
                    }
                    if formatted.contains(para) {
                        return Err(
                            self.conflict(*i, "formats a paragraph another operation formats")
                        );
                    }
                    formatted.push(*para);
                }
                Resolved::MergeParagraphs { para, next, .. } => {
                    if deleted.contains(para) || deleted.contains(next) {
                        return Err(self.conflict(*i, "merges a deleted paragraph"));
                    }
                    if merge_heads.contains(para) {
                        return Err(
                            self.conflict(*i, "merges a paragraph another operation merges")
                        );
                    }
                    merge_heads.push(*para);
                    merge_tails.push(*next);
                }
                _ => {}
            }
        }
        for (i, r) in &self.resolved {
            match r {
                Resolved::FormatParagraph { para, .. } if merge_heads.contains(para) => {
                    return Err(
                        self.conflict(*i, "formats a paragraph whose properties a merge discards")
                    );
                }
                Resolved::InsertParagraph { anchor, side, .. }
                | Resolved::InsertImage { anchor, side, .. }
                    if (*side == Side::After && merge_heads.contains(anchor))
                        || (*side == Side::Before && merge_tails.contains(anchor)) =>
                {
                    return Err(self.conflict(*i, "inserts a paragraph between two a merge joins"));
                }
                Resolved::InsertTable { anchor, side, .. }
                    if (*side == Side::After && merge_heads.contains(anchor))
                        || (*side == Side::Before && merge_tails.contains(anchor)) =>
                {
                    return Err(self.conflict(*i, "inserts a table between two a merge joins"));
                }
                _ => {}
            }
        }
        let mut listed: Vec<usize> = Vec::new();
        for (i, r) in &self.resolved {
            let Resolved::List { paras, .. } = r else {
                continue;
            };
            for para in paras {
                let message = if deleted.contains(para) {
                    "numbers a deleted paragraph"
                } else if listed.contains(para) {
                    "numbers a paragraph another list operation numbers"
                } else if formatted.contains(para) {
                    "numbers a paragraph another operation formats"
                } else if merge_heads.contains(para) {
                    "numbers a paragraph whose properties a merge discards"
                } else {
                    listed.push(*para);
                    continue;
                };
                return Err(self.conflict(*i, message));
            }
        }
        Ok(())
    }

    /// Deletions, taken together, must leave every table cell they touch
    /// ending in a paragraph, and the body with a paragraph.
    fn check_deletions_leave_valid_containers(&self, deleted: &[usize]) -> Result<(), EditError> {
        let dom = &self.opened.dom;
        let gone: Vec<NodeId> = deleted.iter().map(|&p| self.paragraph_nodes[p]).collect();
        for (i, r) in &self.resolved {
            let Resolved::DeleteParagraph { para } = r else {
                continue;
            };
            let node = self.paragraph_nodes[*para];
            if let Some(&cell) = dom.ancestors(node, Some(&W::tc())).first() {
                let children = dom.elements(cell, None);
                let before = children.last().copied();
                let after = children.into_iter().rev().find(|c| !gone.contains(c));
                if after != before && !after.is_some_and(|c| dom.name_is(c, &W::p())) {
                    return Err(self.conflict(
                        *i,
                        "the plan's deletions leave a table cell without a closing paragraph",
                    ));
                }
            } else {
                let (container, what) = self.story_container(node);
                if dom
                    .elements(container, Some(&W::p()))
                    .iter()
                    .all(|p| gone.contains(p))
                {
                    return Err(self.conflict(
                        *i,
                        &format!("the plan's deletions leave {what} without a paragraph"),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Thread operations against each other: nothing may act on a comment
    /// another operation deletes, and a comment is edited once.
    fn check_thread_ops(&self) -> Result<(), EditError> {
        let family = self.family.as_ref().expect("loaded in start");
        let mut gone: Vec<(usize, Vec<u32>)> = Vec::new();
        let mut edited: Vec<u32> = Vec::new();
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            match op {
                ThreadOp::Delete { id } => gone.push((*i, family.with_replies(*id))),
                ThreadOp::Edit { id, .. } => {
                    if edited.contains(id) {
                        return Err(self.conflict(*i, "edits a comment another operation edits"));
                    }
                    edited.push(*id);
                }
                _ => {}
            }
        }
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            let target = op.target();
            if gone.iter().any(|(j, ids)| j != i && ids.contains(&target)) {
                return Err(self.conflict(*i, "acts on a comment another operation deletes"));
            }
        }
        Ok(())
    }

    /// New comments take ids after the source's highest; refuse a plan whose
    /// comments would not fit in `w:id`'s 32 bits.
    fn check_comment_ids_fit(&self) -> Result<(), EditError> {
        let needed = self
            .resolved
            .iter()
            .filter(|(i, r)| match r {
                Resolved::Text { comment, .. } | Resolved::InsertParagraph { comment, .. } => {
                    comment.is_some()
                }
                Resolved::CommentRange { .. } | Resolved::CommentSpan { .. } => true,
                Resolved::Thread { op, .. } => matches!(op, ThreadOp::Reply { .. }),
                Resolved::DeleteParagraph { .. } => self.deletion_comment(*i).is_some(),
                Resolved::FormatParagraph { .. }
                | Resolved::MergeParagraphs { .. }
                | Resolved::InsertTable { .. }
                | Resolved::List { .. }
                | Resolved::FillControl { .. }
                | Resolved::FormatRun { .. }
                | Resolved::InsertFootnote { .. }
                | Resolved::InsertImage { .. }
                | Resolved::PageSetup { .. } => false,
            })
            .count() as u64;
        if needed > 0 && self.next_comment_id + needed - 1 > u64::from(u32::MAX) {
            return Err(err(
                "INVALID_DOCUMENT",
                None,
                "the source's comment ids leave no room for new comments",
            ));
        }
        Ok(())
    }

    /// Check a `settings` operation: something to write, no password, and
    /// one per plan.
    fn resolve_settings(
        &self,
        id: &str,
        request: &crate::settings::SettingsRequest,
    ) -> Result<EditOutcome, Box<(EditError, EditOutcome)>> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        };
        let fail = |code: &str, msg: &str, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if *request == crate::settings::SettingsRequest::default() {
            return Err(fail(
                "INVALID_EDIT",
                "settings needs track_revisions, update_fields or protection",
                outcome,
            ));
        }
        if request
            .protection
            .as_ref()
            .is_some_and(|p| p.password.is_some())
        {
            return Err(fail(
                "UNSUPPORTED",
                "a protection password is not written: Word's legacy hash needs w:cryptProviderType, w:cryptAlgorithmSid, a spin count and a salt; leave the password out to enforce the restriction without one",
                outcome,
            ));
        }
        if self.settings.is_some() {
            return Err(fail(
                "OVERLAPPING_EDITS",
                "one settings operation per plan; put every setting in the first",
                outcome,
            ));
        }
        outcome.matches = 1;
        outcome.context = Some(request.describe());
        Ok(outcome)
    }

    /// Refuse the plan when the text of a redaction still occurs in one of
    /// `outputs` (a comment on it, another paragraph, a header, the
    /// document properties). The message names the parts, never the text.
    fn check_redactions(&self, outputs: &[&[u8]]) -> Result<(), EditError> {
        for (op, operation) in self.plan.operations.iter().enumerate() {
            let OperationKind::Redact { find, .. } = &operation.kind else {
                continue;
            };
            let parts: std::collections::BTreeSet<String> = outputs
                .iter()
                .flat_map(|doc| crate::scrub::leaks(doc, find))
                .collect();
            if parts.is_empty() {
                continue;
            }
            let message = format!(
                "the redacted text still occurs in {}; redact every copy in the same plan, or remove the comment or part that holds it",
                parts.into_iter().collect::<Vec<_>>().join(", ")
            );
            let mut error = self.conflict(op, &message);
            error.code = "REDACTION_LEAK".into();
            error.outcomes[op].code = Some(error.code.clone());
            return Err(error);
        }
        Ok(())
    }

    fn conflict(&self, op_index: usize, message: &str) -> EditError {
        let id = self.outcomes[op_index].id.clone();
        let mut e = err("OVERLAPPING_EDITS", Some(&id), message);
        let mut outcomes = self.outcomes.clone();
        outcomes[op_index].status = "failed".into();
        outcomes[op_index].code = Some(e.code.clone());
        outcomes[op_index].message = Some(message.to_string());
        e.outcomes = outcomes;
        e
    }

    /// Mutate the copy. Resolution succeeded, so every step here is total.
    fn apply(&mut self) -> Result<(), EditError> {
        // 0. Comment ids follow plan order, whatever paragraph they land in.
        let mut ids: BTreeMap<usize, u32> = BTreeMap::new();
        let comments: Vec<(usize, String)> =
            self.resolved
                .iter()
                .filter_map(|(i, r)| {
                    let text = match r {
                        Resolved::Text { comment, .. }
                        | Resolved::InsertParagraph { comment, .. } => comment.clone(),
                        Resolved::CommentRange { text, .. }
                        | Resolved::CommentSpan { text, .. } => Some(text.clone()),
                        Resolved::Thread {
                            op: ThreadOp::Reply { text, .. },
                            ..
                        } => Some(text.clone()),
                        Resolved::DeleteParagraph { .. } => {
                            self.deletion_comment(*i).map(str::to_string)
                        }
                        Resolved::FormatParagraph { .. }
                        | Resolved::MergeParagraphs { .. }
                        | Resolved::InsertTable { .. }
                        | Resolved::List { .. }
                        | Resolved::FormatRun { .. }
                        | Resolved::InsertFootnote { .. }
                        | Resolved::InsertImage { .. }
                        | Resolved::PageSetup { .. }
                        | Resolved::Thread { .. }
                        | Resolved::FillControl { .. } => None,
                    };
                    text.map(|t| (*i, t))
                })
                .collect();
        for (i, text) in comments {
            let id = if self.deletion_comment(i).is_some() {
                // Written into the source copy the comparer reads, not here.
                let id = self.reserve_comment_id();
                self.deletion_comments.push((id, i));
                id
            } else {
                self.new_comment(text)
            };
            ids.insert(i, id);
            self.outcomes[i].comment_id = Some(id);
        }
        for (i, r) in &self.resolved {
            if let Resolved::Thread {
                op: ThreadOp::Reply { parent, .. },
                ..
            } = r
            {
                self.reply_parents.insert(ids[i], *parent);
            }
        }
        // 1. Text edits and their comments, paragraph by paragraph.
        let mut by_para: BTreeMap<usize, Vec<ScheduledEdit>> = BTreeMap::new();
        let mut comment_ranges: BTreeMap<usize, Vec<(usize, usize, usize, String)>> =
            BTreeMap::new();
        let mut format_runs: BTreeMap<usize, Vec<(usize, usize, RunFormat)>> = BTreeMap::new();
        let mut references: BTreeMap<usize, Vec<(usize, u32)>> = BTreeMap::new();
        let reference_style = self
            .style_defined("FootnoteReference")
            .then_some("FootnoteReference");
        let notes: Vec<(usize, usize, String)> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::InsertFootnote { para, at, text } => Some((*para, *at, text.clone())),
                _ => None,
            })
            .collect();
        if !notes.is_empty() {
            let story = self.footnotes_story()?;
            self.notes_story = Some(story);
            let root = self.stories[story].root;
            let text_style = self.style_defined("FootnoteText").then_some("FootnoteText");
            let mut id = notes::next_footnote_id(&self.opened.dom, root);
            for (para, at, text) in notes {
                notes::append_footnote(
                    &mut self.opened.dom,
                    root,
                    id,
                    &text,
                    (text_style, reference_style),
                );
                references.entry(para).or_default().push((at, id));
                id = id.saturating_add(1);
            }
        }
        for (i, r) in &self.resolved {
            match r {
                Resolved::Text {
                    para,
                    start,
                    end,
                    replacement,
                    attach_before,
                    comment,
                    format,
                } => by_para.entry(*para).or_default().push(ScheduledEdit {
                    start: *start,
                    end: *end,
                    op: *i,
                    replacement: replacement.clone(),
                    attach_before: *attach_before,
                    comment: comment.clone(),
                    format: format.clone(),
                }),
                Resolved::CommentRange {
                    para,
                    start,
                    end,
                    text,
                } => {
                    comment_ranges
                        .entry(*para)
                        .or_default()
                        .push((*start, *end, *i, text.clone()));
                }
                Resolved::FormatRun {
                    para,
                    start,
                    end,
                    format,
                } => {
                    format_runs
                        .entry(*para)
                        .or_default()
                        .push((*start, *end, format.clone()));
                }
                _ => {}
            }
        }
        let touched: Vec<usize> = by_para
            .keys()
            .chain(comment_ranges.keys())
            .chain(format_runs.keys())
            .chain(references.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        for para in touched {
            let node = self.paragraph_nodes[para];
            let mut edits = by_para.remove(&para).unwrap_or_default();
            edits.sort_by_key(|e| (e.start, e.end, e.op));
            // Apply in reverse so earlier source offsets stay valid.
            for edit in edits.iter().rev() {
                let projection = project_paragraph(&self.opened.dom, node);
                apply_text_edit(
                    &mut self.opened.dom,
                    &projection,
                    edit.start,
                    edit.end,
                    &edit.replacement,
                    edit.attach_before,
                );
            }
            // Formatting of new text, in new coordinates.
            for edit in &edits {
                if let Some(format) = &edit.format {
                    let s = new_position(&edits, edit.start, true, Some(edit.op));
                    format_range(
                        &mut self.opened.dom,
                        node,
                        s,
                        s + edit.replacement.len(),
                        format,
                    );
                }
            }
            // `format_run` ranges, in new coordinates and plan order.
            for (start, end, format) in format_runs.remove(&para).unwrap_or_default() {
                let s = new_position(&edits, start, true, None);
                let e = new_position(&edits, end, false, None);
                format_range(&mut self.opened.dom, node, s, e, &format);
            }
            // Footnote reference marks; reversed so marks sharing a point
            // end up in plan order.
            for (at, id) in references
                .remove(&para)
                .unwrap_or_default()
                .into_iter()
                .rev()
            {
                let at = new_position(&edits, at, true, None);
                notes::insert_reference(&mut self.opened.dom, node, at, id, reference_style);
            }
            // Helper bookmarks around `whole` replacements. Comment ranges
            // placed below land inside them, on the inserted text.
            let plan = self.plan;
            for edit in &edits {
                let OperationKind::Replace {
                    find, whole: true, ..
                } = &plan.operations[edit.op].kind
                else {
                    continue;
                };
                if edit.replacement.is_empty() {
                    continue;
                }
                let s = new_position(&edits, edit.start, true, Some(edit.op));
                let end = s + edit.replacement.len();
                let story = self.paragraph_story[para].0;
                let part = (story != 0).then(|| self.stories[story].part.clone());
                if let Some(mark) = whole::mark(
                    &mut self.opened.dom,
                    node,
                    (s, end),
                    (edit.op, part),
                    (find, &edit.replacement),
                ) {
                    self.whole_marks.push(mark);
                }
            }
            // Comment ranges, in new coordinates.
            let mut pending: Vec<(usize, usize, usize, String)> = Vec::new();
            for edit in &edits {
                if let Some(text) = &edit.comment {
                    let s = new_position(&edits, edit.start, true, Some(edit.op));
                    pending.push((s, s + edit.replacement.len(), edit.op, text.clone()));
                }
            }
            for (start, end, i, text) in comment_ranges.remove(&para).unwrap_or_default() {
                let s = new_position(&edits, start, true, None);
                let e = new_position(&edits, end, false, None);
                pending.push((s, e, i, text));
            }
            pending.sort_by_key(|&(s, _, i, _)| (s, i));
            for (start, end, i, _) in pending {
                anchor_comment(&mut self.opened.dom, node, start, end, ids[&i]);
            }
        }
        // 1b. Comments over several paragraphs, in their edited text.
        for (i, r) in &self.resolved {
            if let Resolved::CommentSpan { para, last, .. } = r {
                anchor_span(
                    &mut self.opened.dom,
                    self.paragraph_nodes[*para],
                    self.paragraph_nodes[*last],
                    ids[i],
                );
            }
        }
        // 1c. Content control fills (text edits never reach control content).
        self.apply_fills();
        // 2. Paragraph and table insertions (anchors are source paragraphs,
        // untouched by 1).
        let inserts: Vec<(usize, Resolved)> = self
            .resolved
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r,
                    Resolved::InsertParagraph { .. }
                        | Resolved::InsertTable { .. }
                        | Resolved::InsertImage { .. }
                )
            })
            .cloned()
            .collect();
        let mut tables: Vec<NodeId> = Vec::new();
        let mut media_used = std::collections::HashSet::new();
        let mut drawing_id = crate::markdown::max_drawing_id(&self.opened.pkg);
        // Several paragraphs after one anchor follow it in plan order: each
        // goes after the one inserted there before it.
        let mut last_after: BTreeMap<usize, NodeId> = BTreeMap::new();
        for (i, r) in inserts {
            let (anchor, side, news, commented) = match r {
                Resolved::InsertParagraph {
                    anchor,
                    side,
                    runs,
                    like,
                    style,
                    comment,
                    toc,
                } => {
                    let like_node = self.paragraph_nodes[like];
                    let news = match &toc {
                        Some(toc) => toc_paragraphs(&mut self.opened.dom, toc),
                        None => vec![build_paragraph(
                            &mut self.opened.dom,
                            like_node,
                            &runs,
                            style.as_deref(),
                        )],
                    };
                    (anchor, side, news, comment.is_some())
                }
                Resolved::InsertTable {
                    anchor,
                    side,
                    rows,
                    header_row,
                    widths,
                    style,
                    ..
                } => {
                    let new = structural::build_table(
                        &mut self.opened.dom,
                        self.paragraph_nodes[anchor],
                        &rows,
                        header_row,
                        &widths,
                        &style,
                    );
                    tables.push(new);
                    (anchor, side, vec![new], false)
                }
                Resolved::InsertImage {
                    anchor,
                    side,
                    picture,
                } => {
                    drawing_id = drawing_id.saturating_add(1);
                    let new = self.image_paragraph(&picture, &mut media_used, drawing_id);
                    (anchor, side, vec![new], false)
                }
                _ => continue,
            };
            let anchor_node = self.paragraph_nodes[anchor];
            for &new in &news {
                match side {
                    Side::After => {
                        let prev = last_after.get(&anchor).copied().unwrap_or(anchor_node);
                        self.opened.dom.add_after_self(prev, new);
                        last_after.insert(anchor, new);
                    }
                    Side::Before => self.opened.dom.add_before_self(anchor_node, new),
                }
            }
            let new = news[news.len() - 1];
            if commented {
                let projection = project_paragraph(&self.opened.dom, new);
                anchor_comment(&mut self.opened.dom, new, 0, projection.text.len(), ids[&i]);
            }
        }
        // 3. Paragraph formatting.
        for (_, r) in &self.resolved {
            if let Resolved::FormatParagraph {
                para,
                style,
                alignment,
                spacing,
            } = r
            {
                format_paragraph(
                    &mut self.opened.dom,
                    self.paragraph_nodes[*para],
                    style.as_deref(),
                    *alignment,
                    *spacing,
                );
            }
        }
        // 3b. Lists: new numbering instances take ids after the source's.
        let (mut next_abstract, mut next_num) = structural::next_numbering_ids(&self.opened);
        for (_, r) in &self.resolved {
            let Resolved::List {
                paras,
                kind,
                level,
                join,
                style,
                add_style,
            } = r
            else {
                continue;
            };
            let num_id = match join {
                Some(num_id) => num_id.clone(),
                None => {
                    let (abstracts, nums) = &mut self.new_numbering;
                    abstracts.push(crate::markdown::xml::abstract_num(
                        next_abstract,
                        kind.format(),
                    ));
                    nums.push(crate::markdown::xml::num(next_num, next_abstract, None));
                    next_abstract += 1;
                    next_num += 1;
                    (next_num - 1).to_string()
                }
            };
            for &para in paras {
                let styled = structural::number_paragraph(
                    &mut self.opened.dom,
                    self.paragraph_nodes[para],
                    *level,
                    &num_id,
                    style,
                );
                if styled && *add_style {
                    self.needed_styles.insert(style.clone());
                }
            }
        }
        // 4. Merges, first first, so a chain folds into its last paragraph.
        let mut merges: Vec<(usize, usize, String)> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::MergeParagraphs {
                    para,
                    next,
                    separator,
                } => Some((*para, *next, separator.clone())),
                _ => None,
            })
            .collect();
        merges.sort_by_key(|&(para, ..)| para);
        for (para, next, separator) in merges {
            merge_into(
                &mut self.opened.dom,
                self.paragraph_nodes[para],
                self.paragraph_nodes[next],
                &separator,
            );
        }
        // 5. Paragraph deletions.
        for (_, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                self.opened.dom.remove(self.paragraph_nodes[*para]);
            }
        }
        // 6. Reply markers beside their comment's; deleted comments' markers.
        let roots: Vec<NodeId> = self.stories.iter().map(|s| s.root).collect();
        for (i, r) in &self.resolved {
            let Resolved::Thread { op, .. } = r else {
                continue;
            };
            match op {
                ThreadOp::Reply { parent, .. } => {
                    place_reply_markers(&mut self.opened.dom, &roots, *parent, ids[i]);
                }
                ThreadOp::Delete { id } => {
                    let family = self.family.as_ref().expect("loaded in start");
                    let gone: Vec<String> = family
                        .with_replies(*id)
                        .iter()
                        .map(u32::to_string)
                        .collect();
                    remove_comment_markers(&mut self.opened.dom, &roots, &gone);
                }
                ThreadOp::Resolve { .. } | ThreadOp::Edit { .. } => {}
            }
        }
        // 7. A paragraph after each new table, and between it and a table
        // before it, once the deletions have settled its neighbours.
        for table in tables {
            structural::separate(&mut self.opened.dom, table);
        }
        // 8. Page setup.
        let setups: Vec<sections::Targets> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::PageSetup { targets } => Some(targets.clone()),
                _ => None,
            })
            .collect();
        for targets in &setups {
            self.apply_page_setup(targets);
        }
        Ok(())
    }

    fn new_comment(&mut self, text: String) -> u32 {
        let id = self.reserve_comment_id();
        self.comments.push((id, text));
        id
    }

    fn reserve_comment_id(&mut self) -> u32 {
        self.comments_added += 1;
        if let Some(id) = self.preset_comment_ids.pop() {
            return id;
        }
        // check_conflicts proved every new id fits in u32.
        let id = u32::try_from(self.next_comment_id).unwrap_or(u32::MAX);
        self.next_comment_id += 1;
        id
    }

    /// The comment of operation `op` when it deletes a paragraph.
    fn deletion_comment(&self, op: usize) -> Option<&str> {
        match &self.plan.operations[op].kind {
            OperationKind::DeleteParagraph { comment, .. } => comment.as_deref(),
            _ => None,
        }
    }

    /// The copy the comparer reads as the original: the base, with each
    /// deleted paragraph's comment anchored on its whole text. The comparer
    /// carries the comment onto the deleted text of the redline. A plan's
    /// watermark is written into this copy as well, so it stays untracked.
    fn commented_base(&self) -> Result<std::borrow::Cow<'_, [u8]>, EditError> {
        // Edited and deleted comments are edited and deleted in the original
        // too: the comparer carries the copy's comment parts only when they
        // define every comment of the original unchanged.
        let thread_ops: Vec<Operation> = self
            .plan
            .operations
            .iter()
            .filter(|op| {
                matches!(
                    op.kind,
                    OperationKind::EditComment { .. } | OperationKind::DeleteComment { .. }
                )
            })
            .cloned()
            .collect();
        // A redaction is no change either: the base loses the text too.
        let redactions: Vec<Operation> = self
            .plan
            .operations
            .iter()
            .filter(|op| matches!(op.kind, OperationKind::Redact { .. }))
            .cloned()
            .collect();
        if self.deletion_comments.is_empty()
            && thread_ops.is_empty()
            && self.watermark.is_none()
            && redactions.is_empty()
        {
            return Ok(std::borrow::Cow::Borrowed(&self.base));
        }
        let mut operations: Vec<Operation> = self
            .deletion_comments
            .iter()
            .map(|&(_, op)| {
                let operation = &self.plan.operations[op];
                let OperationKind::DeleteParagraph {
                    paragraph,
                    comment: Some(text),
                } = &operation.kind
                else {
                    unreachable!("deletion_comments holds commented deletions");
                };
                Operation {
                    id: operation.id.clone(),
                    kind: OperationKind::Comment {
                        paragraph: paragraph.clone(),
                        find: None,
                        text: text.clone(),
                        through: None,
                        occurrence: None,
                    },
                }
            })
            .collect();
        operations.extend(thread_ops);
        // A watermark is header content, not a change: the base carries it
        // too, so the comparer leaves it untracked.
        operations.extend(
            self.plan
                .operations
                .iter()
                .filter(|op| matches!(op.kind, OperationKind::Watermark { .. }))
                .cloned(),
        );
        operations.extend(redactions);
        let plan = EditPlan {
            schema_version: SCHEMA_VERSION,
            source_sha256: None,
            author: self.plan.author.clone(),
            date: Some(self.date.clone()),
            initials: Some(self.initials.clone()),
            resolve_revisions: None,
            existing_revisions: ExistingRevisions::default(),
            operations,
            update_fields: false,
        };
        let mut tx = Transaction::start(&self.base, &plan)?;
        tx.preset_comment_ids = self
            .deletion_comments
            .iter()
            .rev()
            .map(|&(id, _)| id)
            .collect();
        tx.resolve()?;
        tx.apply()?;
        let (commented, _) = tx.finish()?;
        Ok(std::borrow::Cow::Owned(commented))
    }

    /// The clean copy, and the copy the comparer reads when `whole`
    /// replacements carry helper bookmarks (the clean copy never does).
    fn finish(&mut self) -> Result<(Vec<u8>, Option<Vec<u8>>), EditError> {
        let threads = self
            .resolved
            .iter()
            .any(|(_, r)| matches!(r, Resolved::Thread { .. }));
        if !self.comments.is_empty() || threads {
            self.write_comments_part()?;
        }
        let mut styles: std::collections::BTreeSet<String> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::InsertTable {
                    style,
                    add_style: true,
                    ..
                } => Some(style.clone()),
                _ => None,
            })
            .collect();
        styles.extend(self.needed_styles.iter().cloned());
        let main = self.opened.main.clone();
        if !styles.is_empty() {
            structural::add_styles(&mut self.opened.pkg, &main, &styles);
        }
        let (abstracts, nums) = std::mem::take(&mut self.new_numbering);
        if !nums.is_empty() {
            structural::splice_numbering(&mut self.opened.pkg, &main, &abstracts, &nums);
        }
        // The body is always written; a story part only when an operation
        // edits it, so untouched parts keep their exact bytes.
        let mut written = self.touched_stories();
        written.insert(0);
        written.extend(self.apply_watermark()?);
        written.extend(self.notes_story);
        if let Some(request) = &self.settings {
            crate::settings::apply_settings(&mut self.opened.pkg, &main, request)
                .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
        }
        let marked = if self.whole_marks.is_empty() {
            None
        } else {
            self.write_stories(&written);
            let bytes = self
                .opened
                .pkg
                .to_zip()
                .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
            for &story in &written {
                whole::strip(&mut self.opened.dom, self.stories[story].root);
            }
            Some(bytes)
        };
        self.write_stories(&written);
        let clean = self
            .opened
            .pkg
            .to_zip()
            .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
        Ok((clean, marked))
    }

    fn write_stories(&mut self, stories: &std::collections::BTreeSet<usize>) {
        for &story in stories {
            let StoryPart { part, document, .. } = &self.stories[story];
            let xml = self.opened.dom.serialize_document(*document);
            self.opened.pkg.set_part(part, xml.into_bytes());
        }
    }

    /// Write the comment part family: the plan's new comments and replies,
    /// then its edits, resolutions and deletions; every part consistent
    /// (see [`crate::comments`]).
    fn write_comments_part(&mut self) -> Result<(), EditError> {
        let main = self.opened.main.clone();
        let mut family = match self.family.take() {
            Some(family) => family,
            None => crate::comments::CommentFamily::load(&self.opened.pkg, &main)
                .map_err(|m| err("INVALID_DOCUMENT", None, m))?,
        };
        for (id, text) in &self.comments {
            family.add(&crate::comments::NewComment {
                id: *id,
                author: &self.plan.author,
                date: &self.date,
                initials: &self.initials,
                text,
                parent: self.reply_parents.get(id).copied(),
            });
        }
        let ops: Vec<&ThreadOp> = self
            .resolved
            .iter()
            .filter_map(|(_, r)| match r {
                Resolved::Thread { op, .. } => Some(op),
                _ => None,
            })
            .collect();
        for op in &ops {
            if let ThreadOp::Edit { id, text } = op {
                family.set_text(*id, text);
            }
        }
        for op in &ops {
            if let ThreadOp::Resolve { id, done } = op {
                family.set_done(*id, *done);
            }
        }
        for op in &ops {
            if let ThreadOp::Delete { id } = op {
                family.remove(*id);
            }
        }
        family.store(&mut self.opened.pkg, &main);
        Ok(())
    }
}

/// Whether comment `id`'s range in `redline` holds text that is not deleted.
fn comment_holds_kept_text(redline: &[u8], id: u32) -> bool {
    let Ok(opened) = Opened::open(redline) else {
        return false;
    };
    let id = id.to_string();
    let parts = std::iter::once(opened.main.clone())
        .chain(opened.story_parts().into_iter().map(|(_, _, part)| part));
    for part in parts {
        let Ok((dom, _, root)) = crate::inspect::parse_part(&opened.pkg, &part) else {
            continue;
        };
        let mut inside = false;
        for node in dom.descendants(root, None) {
            let Some(name) = dom.name(node).filter(|n| n.namespace_name() == W::URI) else {
                continue;
            };
            let is_id = || dom.attribute(node, &W::id()) == Some(id.as_str());
            match name.local_name() {
                "commentRangeStart" if is_id() => inside = true,
                "commentRangeEnd" if is_id() => inside = false,
                "t" if inside => return true,
                _ => {}
            }
        }
    }
    false
}

fn existing_comment_ids(opened: &Opened) -> Option<u32> {
    let name = opened.related("comments").into_iter().next()?;
    let xml = opened.pkg.part_string(&name)?;
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document)?;
    dom.descendants(root, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| {
            dom.attribute(c, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
        })
        .max()
}

/// Markup that may sit between two block paragraphs and also inside one.
fn is_range_markup(dom: &Dom, node: NodeId) -> bool {
    const RANGES: &[&str] = &[
        "bookmarkStart",
        "bookmarkEnd",
        "commentRangeStart",
        "commentRangeEnd",
        "permStart",
        "permEnd",
        "proofErr",
    ];
    dom.name(node)
        .is_some_and(|n| n.namespace_name() == W::URI && RANGES.contains(&n.local_name()))
}

fn kind_name(kind: &OperationKind) -> &'static str {
    match kind {
        OperationKind::Replace { .. } => "replace",
        OperationKind::Insert { .. } => "insert",
        OperationKind::Delete { .. } => "delete",
        OperationKind::Comment { .. } => "comment",
        OperationKind::InsertParagraph { .. } => "insert_paragraph",
        OperationKind::DeleteParagraph { .. } => "delete_paragraph",
        OperationKind::FormatParagraph { .. } => "format_paragraph",
        OperationKind::MergeParagraphs { .. } => "merge_paragraphs",
        OperationKind::Rewrite { .. } => "rewrite",
        OperationKind::ReplyComment { .. } => "reply_comment",
        OperationKind::ResolveComment { .. } => "resolve_comment",
        OperationKind::EditComment { .. } => "edit_comment",
        OperationKind::DeleteComment { .. } => "delete_comment",
        OperationKind::InsertTable { .. } => "insert_table",
        OperationKind::List { .. } => "list",
        OperationKind::Watermark { .. } => "watermark",
        OperationKind::FillControl { .. } => "fill_control",
        OperationKind::FormatRun { .. } => "format_run",
        OperationKind::InsertFootnote { .. } => "insert_footnote",
        OperationKind::InsertImage { .. } => "insert_image",
        OperationKind::PageSetup { .. } => "page_setup",
        OperationKind::InsertToc { .. } => "insert_toc",
        OperationKind::Redact { .. } => "redact",
        OperationKind::Settings { .. } => "settings",
    }
}

/// Plain text only: no control characters (tabs and breaks are not run text).
fn check_text(text: &str) -> Result<(), String> {
    if text
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{fffe}' | '\u{ffff}'))
    {
        return Err("text must be plain, without control characters (tabs and line breaks are not supported inside run text)".into());
    }
    Ok(())
}

/// Comment text: nonempty, and each line (`\n` starts a new comment
/// paragraph) plain text that `comments.xml` can carry.
fn check_comment(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("comment text must be nonempty".into());
    }
    text.split('\n').try_for_each(check_text)
}

/// `before {mark} after` with up to 20 chars of context on either side.
/// What a redaction leaves of `find`: one full block per character.
fn redaction(find: &str) -> String {
    "\u{2588}".repeat(find.chars().count())
}

fn context(text: &str, start: usize, end: usize, mark: &str) -> String {
    const WINDOW: usize = 20;
    let before: String = text[..start]
        .chars()
        .rev()
        .take(WINDOW)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let after: String = text[end..].chars().take(WINDOW).collect();
    format!("{before}{mark}{after}")
}

fn excerpt(text: &str, max: usize) -> String {
    let mut s: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        s.push('…');
    }
    s
}

/// Position `pos` of the source projection after `edits` were applied.
/// `inclusive` shifts past insertions sitting exactly at `pos`; `own` is the
/// operation whose own insertion must not shift its comment start. Insertions
/// sharing a point land in plan order, so `own` shifts only past the ones
/// planned before it.
fn new_position(edits: &[ScheduledEdit], pos: usize, inclusive: bool, own: Option<usize>) -> usize {
    let mut delta: i64 = 0;
    for edit in edits {
        let (start, end, i) = (edit.start, edit.end, edit.op);
        if Some(i) == own {
            continue;
        }
        let shifts = if start == end {
            // insertion
            if inclusive {
                start < pos || (start == pos && own.is_none_or(|o| i < o))
            } else {
                start < pos
            }
        } else {
            end <= pos
        };
        if shifts {
            delta += edit.replacement.len() as i64 - (end - start) as i64;
        }
    }
    (pos as i64 + delta).max(0) as usize
}

/// The segment an insertion at `pos` attaches to.
fn attach_segment(
    projection: &Projection,
    pos: usize,
    attach_before: bool,
) -> Option<&crate::inspect::Segment> {
    let segs = &projection.segments;
    if attach_before {
        // The segment holding the char before `pos`, else the one starting at it.
        segs.iter()
            .rev()
            .find(|s| s.start < pos && pos <= s.end)
            .or_else(|| segs.iter().find(|s| s.start == pos))
    } else {
        segs.iter()
            .find(|s| s.start <= pos && pos < s.end)
            .or_else(|| segs.iter().rev().find(|s| s.end == pos))
    }
}

/// Replace `[start, end)` of the paragraph's projection with `replacement`.
fn apply_text_edit(
    dom: &mut Dom,
    projection: &Projection,
    start: usize,
    end: usize,
    replacement: &str,
    attach_before: bool,
) {
    if start == end {
        let seg = attach_segment(projection, start, attach_before).expect("checked at resolution");
        if let Piece::Text { t, .. } = seg.piece {
            let value = dom.value(t);
            let at = start - seg.start;
            let updated = format!("{}{}{}", &value[..at], replacement, &value[at..]);
            set_text(dom, t, &updated);
        }
        return;
    }
    let mut inserted = false;
    for seg in &projection.segments {
        if seg.end <= start || seg.start >= end {
            continue;
        }
        let Piece::Text { t, .. } = seg.piece else {
            continue;
        };
        let value = dom.value(t);
        let from = start.saturating_sub(seg.start).min(value.len());
        let to = end.saturating_sub(seg.start).min(value.len());
        let middle = if inserted {
            ""
        } else {
            inserted = true;
            replacement
        };
        let updated = format!("{}{}{}", &value[..from], middle, &value[to..]);
        set_text(dom, t, &updated);
    }
}

fn set_text(dom: &mut Dom, t: NodeId, value: &str) {
    dom.set_value(t, value);
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
}

/// Split the run owning `seg` so that a run boundary falls at projection byte
/// `at` (inside `seg`). Returns nothing when `at` is already a boundary.
fn split_run_at(dom: &mut Dom, seg: &crate::inspect::Segment, at: usize) {
    if at <= seg.start || at >= seg.end {
        return;
    }
    let Piece::Text { t, run } = seg.piece else {
        return;
    };
    let value = dom.value(t);
    let cut = at - seg.start;
    let (left, right) = (value[..cut].to_string(), value[cut..].to_string());
    // Right half: a clone of the run keeping rPr plus this t (with the right
    // text) and every child after it; the original keeps children up to t.
    let clone = dom.clone_subtree(run);
    let clone_children = dom.elements(clone, None);
    let run_children = dom.elements(run, None);
    let index = run_children
        .iter()
        .position(|&c| c == t)
        .expect("t is a child of its run");
    for (i, &child) in clone_children.iter().enumerate() {
        let is_rpr = dom.name_is(child, &W::r_pr());
        if !is_rpr && i < index {
            dom.remove(child);
        }
    }
    for (i, &child) in run_children.iter().enumerate() {
        if i > index {
            dom.remove(child);
        }
    }
    set_text(dom, t, &left);
    let clone_t = clone_children[index];
    set_text(dom, clone_t, &right);
    dom.add_after_self(run, clone);
}

/// Wrap the projection range `[start, end)` of `paragraph` in comment
/// markers for comment `id`, splitting runs at the boundaries as needed.
fn anchor_comment(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, id: u32) {
    let id_str = id.to_string();
    let range_start = dom.new_element(W::name("commentRangeStart"));
    dom.set_attribute_value(range_start, &W::id(), Some(&id_str));
    let range_end = dom.new_element(W::name("commentRangeEnd"));
    dom.set_attribute_value(range_end, &W::id(), Some(&id_str));
    let reference_run = dom.new_element(W::r());
    let reference = dom.new_element(W::name("commentReference"));
    dom.set_attribute_value(reference, &W::id(), Some(&id_str));
    dom.add(reference_run, reference);
    if wrap_range(dom, paragraph, start, end, range_start, range_end) {
        dom.add_after_self(range_end, reference_run);
    } else {
        dom.add(paragraph, range_start);
        dom.add(paragraph, range_end);
        dom.add(paragraph, reference_run);
    }
}

fn comment_marker(dom: &mut Dom, local: &str, id: &str) -> NodeId {
    let marker = dom.new_element(W::name(local));
    dom.set_attribute_value(marker, &W::id(), Some(id));
    marker
}

fn comment_reference_run(dom: &mut Dom, id: &str) -> NodeId {
    let run = dom.new_element(W::r());
    let reference = comment_marker(dom, "commentReference", id);
    dom.add(run, reference);
    run
}

/// Comment `id` from the start of paragraph `first` to the end of `last`:
/// the start marker before `first`'s first run, the end marker and the
/// reference run after `last`'s last run (appended to an empty paragraph).
fn anchor_span(dom: &mut Dom, first: NodeId, last: NodeId, id: u32) {
    let id = id.to_string();
    let range_start = comment_marker(dom, "commentRangeStart", &id);
    let range_end = comment_marker(dom, "commentRangeEnd", &id);
    let reference_run = comment_reference_run(dom, &id);
    let len = project_paragraph(dom, first).text.len();
    let scratch = dom.new_element(W::name("commentRangeEnd"));
    if len > 0 && wrap_range(dom, first, 0, len, range_start, scratch) {
        dom.remove(scratch);
    } else {
        match dom.element(first, &W::p_pr()) {
            Some(ppr) => dom.add_after_self(ppr, range_start),
            None => dom.add_first(first, range_start),
        }
    }
    let len = project_paragraph(dom, last).text.len();
    let scratch = dom.new_element(W::name("commentRangeStart"));
    if len > 0 && wrap_range(dom, last, 0, len, scratch, range_end) {
        dom.remove(scratch);
        dom.add_after_self(range_end, reference_run);
    } else {
        dom.add(last, range_end);
        dom.add(last, reference_run);
    }
}

/// Markers of reply `id` beside those of comment `parent`, as Word writes
/// them: the start after the parent's start, the end after the parent's
/// end, the reference run after the parent's reference run.
fn place_reply_markers(dom: &mut Dom, roots: &[NodeId], parent: u32, id: u32) {
    let parent = parent.to_string();
    let id = id.to_string();
    let find = |dom: &Dom, local: &str| {
        roots.iter().find_map(|&root| {
            dom.descendants(root, Some(&W::name(local)))
                .into_iter()
                .find(|&m| dom.attribute(m, &W::id()) == Some(parent.as_str()))
        })
    };
    let (start, end, reference) = (
        find(dom, "commentRangeStart"),
        find(dom, "commentRangeEnd"),
        find(dom, "commentReference"),
    );
    let Some(reference) = reference else {
        return;
    };
    let parent_run = dom
        .parent(reference)
        .filter(|&r| dom.name_is(r, &W::r()))
        .unwrap_or(reference);
    let new_run = comment_reference_run(dom, &id);
    dom.add_after_self(parent_run, new_run);
    let new_end = comment_marker(dom, "commentRangeEnd", &id);
    match end {
        Some(end) => dom.add_after_self(end, new_end),
        None => dom.add_before_self(new_run, new_end),
    }
    let new_start = comment_marker(dom, "commentRangeStart", &id);
    match start {
        Some(start) => dom.add_after_self(start, new_start),
        None => dom.add_before_self(new_end, new_start),
    }
}

/// Remove the range markers and references of the comments `ids`; a run
/// left holding nothing but its properties goes too.
fn remove_comment_markers(dom: &mut Dom, roots: &[NodeId], ids: &[String]) {
    for &root in roots {
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for marker in dom.descendants(root, Some(&W::name(local))) {
                if !dom
                    .attribute(marker, &W::id())
                    .is_some_and(|v| ids.iter().any(|id| id == v))
                {
                    continue;
                }
                let run = dom.parent(marker).filter(|&r| dom.name_is(r, &W::r()));
                dom.remove(marker);
                if let Some(run) = run
                    && dom
                        .elements(run, None)
                        .iter()
                        .all(|&c| dom.name_is(c, &W::r_pr()))
                {
                    dom.remove(run);
                }
            }
        }
    }
}

/// Put `open` right before the run holding projection offset `start` and
/// `close` right after the run ending at `end`, splitting runs at both
/// offsets first. False, with nothing placed, when the range holds no run.
fn wrap_range(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    open: NodeId,
    close: NodeId,
) -> bool {
    let projection = project_paragraph(dom, paragraph);
    if let Some(seg) = projection
        .segments
        .iter()
        .find(|s| s.start < start && start < s.end)
        .cloned()
    {
        split_run_at(dom, &seg, start);
    }
    let projection = project_paragraph(dom, paragraph);
    if let Some(seg) = projection
        .segments
        .iter()
        .find(|s| s.start < end && end < s.end)
        .cloned()
    {
        split_run_at(dom, &seg, end);
    }
    let projection = project_paragraph(dom, paragraph);
    let first = projection
        .segments
        .iter()
        .find(|s| s.start >= start && s.start < end.max(start + 1))
        .map(|s| run_of(&s.piece));
    let last = projection
        .segments
        .iter()
        .rev()
        .find(|s| s.end <= end && s.end > start)
        .map(|s| run_of(&s.piece));
    match (first, last) {
        (Some(first), Some(last)) if start < end => {
            dom.add_before_self(first, open);
            dom.add_after_self(last, close);
            true
        }
        _ => false,
    }
}

fn run_of(piece: &Piece) -> NodeId {
    match piece {
        Piece::Text { run, .. } | Piece::Glyph { run, .. } => *run,
    }
}

/// Apply `format` to the runs holding projection range `[start, end)` of
/// `paragraph`, splitting runs at the boundaries first.
fn format_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize, format: &RunFormat) {
    if start >= end {
        return;
    }
    for at in [start, end] {
        let projection = project_paragraph(dom, paragraph);
        if let Some(seg) = projection
            .segments
            .iter()
            .find(|s| s.start < at && at < s.end)
            .cloned()
        {
            split_run_at(dom, &seg, at);
        }
    }
    let projection = project_paragraph(dom, paragraph);
    let mut runs: Vec<NodeId> = Vec::new();
    for seg in &projection.segments {
        if seg.start >= start && seg.end <= end && seg.end > seg.start {
            let run = run_of(&seg.piece);
            if !runs.contains(&run) {
                runs.push(run);
            }
        }
    }
    for run in runs {
        let rpr = match dom.element(run, &W::r_pr()) {
            Some(rpr) => rpr,
            None => {
                let rpr = dom.new_element(W::r_pr());
                dom.add_first(run, rpr);
                rpr
            }
        };
        apply_run_format(dom, rpr, format);
        if dom.elements(rpr, None).is_empty() {
            dom.remove(rpr);
        }
    }
}

/// Set the paragraph style, alignment and spacing in `paragraph`'s `w:pPr`.
fn format_paragraph(
    dom: &mut Dom,
    paragraph: NodeId,
    style: Option<&str>,
    alignment: Option<Alignment>,
    spacing: Spacing,
) {
    let ppr = match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    };
    if let Some(style) = style {
        if let Some(existing) = dom.element(ppr, &W::p_style()) {
            dom.set_attribute_value(existing, &W::val(), Some(style));
        } else {
            let el = dom.new_element(W::p_style());
            dom.set_attribute_value(el, &W::val(), Some(style));
            dom.add_first(ppr, el);
        }
    }
    if let Some(alignment) = alignment {
        if let Some(existing) = dom.element(ppr, &W::name("jc")) {
            dom.set_attribute_value(existing, &W::val(), Some(alignment.jc()));
        } else {
            let el = dom.new_element(W::name("jc"));
            dom.set_attribute_value(el, &W::val(), Some(alignment.jc()));
            insert_ppr_child(dom, ppr, el);
        }
    }
    if spacing != Spacing::default() {
        let el = match dom.element(ppr, &W::name("spacing")) {
            Some(el) => el,
            None => {
                let el = dom.new_element(W::name("spacing"));
                insert_ppr_child(dom, ppr, el);
                el
            }
        };
        if let Some(LineSpacing(line)) = spacing.line {
            dom.set_attribute_value(el, &W::name("line"), Some(&line.to_string()));
            dom.set_attribute_value(el, &W::name("lineRule"), Some("auto"));
        }
        // An autospacing flag overrides the explicit value, so it goes.
        if let Some(Points(before)) = spacing.before {
            dom.set_attribute_value(el, &W::name("before"), Some(&before.to_string()));
            dom.set_attribute_value(el, &W::name("beforeAutospacing"), None);
        }
        if let Some(Points(after)) = spacing.after {
            dom.set_attribute_value(el, &W::name("after"), Some(&after.to_string()));
            dom.set_attribute_value(el, &W::name("afterAutospacing"), None);
        }
    }
}

/// Schema order of `w:pPr` children (CT_PPrBase, then rPr/sectPr/pPrChange).
const PPR_ORDER: &[&str] = &[
    "pStyle",
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "framePr",
    "widowControl",
    "numPr",
    "suppressLineNumbers",
    "pBdr",
    "shd",
    "tabs",
    "suppressAutoHyphens",
    "kinsoku",
    "wordWrap",
    "overflowPunct",
    "topLinePunct",
    "autoSpaceDE",
    "autoSpaceDN",
    "bidi",
    "adjustRightInd",
    "snapToGrid",
    "spacing",
    "ind",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
    "jc",
    "textDirection",
    "textAlignment",
    "textboxTightWrap",
    "outlineLvl",
    "divId",
    "cnfStyle",
    "rPr",
    "sectPr",
    "pPrChange",
];

/// Insert `child` into `ppr` at its schema position.
fn insert_ppr_child(dom: &mut Dom, ppr: NodeId, child: NodeId) {
    let rank = |local: &str| {
        PPR_ORDER
            .iter()
            .position(|&n| n == local)
            .unwrap_or(PPR_ORDER.len())
    };
    let my_rank = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let after = dom.elements(ppr, None).into_iter().rev().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && rank(n.local_name()) <= my_rank)
    });
    match after {
        Some(after) => dom.add_after_self(after, child),
        None => dom.add_first(ppr, child),
    }
}

/// Move `head`'s content, then `separator` as a run, then any range markup
/// between the two, to the start of `next`, and drop `head`. `next` keeps
/// its own paragraph properties.
fn merge_into(dom: &mut Dom, head: NodeId, next: NodeId, separator: &str) {
    let mut moved: Vec<NodeId> = dom
        .nodes(head)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    if !separator.is_empty() {
        let r = dom.new_element(W::r());
        let last_rpr = dom
            .elements(head, Some(&W::r()))
            .last()
            .and_then(|&r| dom.element(r, &W::r_pr()));
        if let Some(rpr) = last_rpr {
            let rpr = dom.clone_subtree(rpr);
            for child in dom.elements(rpr, None) {
                if dom.name_is(child, &W::r_pr_change())
                    || dom.name_is(child, &W::ins())
                    || dom.name_is(child, &W::del())
                {
                    dom.remove(child);
                }
            }
            dom.add(r, rpr);
        }
        let t = dom.new_element(W::t());
        set_text(dom, t, separator);
        dom.add(r, t);
        moved.push(r);
    }
    let mut cursor = dom.next_element(head);
    while let Some(n) = cursor.filter(|&n| n != next) {
        moved.push(n);
        cursor = dom.next_element(n);
    }
    let first_content = dom
        .nodes(next)
        .into_iter()
        .find(|&c| !dom.name_is(c, &W::p_pr()));
    for n in moved {
        if dom.parent(n).is_some() {
            dom.remove(n);
        }
        match first_content {
            Some(anchor) => dom.add_before_self(anchor, n),
            None => dom.add(next, n),
        }
    }
    dom.remove(head);
}

/// `insert_toc`'s paragraphs: the optional `TOCHeading` title, then a
/// paragraph holding an empty `TOC` field.
fn toc_paragraphs(dom: &mut Dom, toc: &TocSpec) -> Vec<NodeId> {
    let mut out = Vec::new();
    if let Some(title) = &toc.title {
        let p = dom.new_element(W::p());
        let ppr = dom.new_element(W::p_pr());
        let style = dom.new_element(W::p_style());
        dom.set_attribute_value(style, &W::val(), Some("TOCHeading"));
        dom.add(ppr, style);
        dom.add(p, ppr);
        let run = dom.new_element(W::r());
        let t = dom.new_element(W::t());
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t, title);
        dom.add(run, t);
        dom.add(p, run);
        out.push(p);
    }
    let p = dom.new_element(W::p());
    let mark = |dom: &mut Dom, p: NodeId, kind: &str| {
        let run = dom.new_element(W::r());
        let fld = dom.new_element(W::name("fldChar"));
        dom.set_attribute_value(fld, &W::name("fldCharType"), Some(kind));
        dom.add(run, fld);
        dom.add(p, run);
    };
    mark(dom, p, "begin");
    let run = dom.new_element(W::r());
    let instr = dom.new_element(W::name("instrText"));
    dom.set_attribute_value(instr, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(
        instr,
        &format!(" TOC \\o \"1-{}\" \\h \\z \\u ", toc.levels),
    );
    dom.add(run, instr);
    dom.add(p, run);
    mark(dom, p, "separate");
    mark(dom, p, "end");
    out.push(p);
    out
}

/// A new paragraph modeled on `anchor`: its `pPr` minus section break and
/// revision marks, runs formatted like the anchor's first run plus the
/// requested toggles.
fn build_paragraph(dom: &mut Dom, anchor: NodeId, runs: &[RunSpec], style: Option<&str>) -> NodeId {
    let p = dom.new_element(W::p());
    if let Some(ppr) = dom.element(anchor, &W::p_pr()) {
        let ppr_clone = dom.clone_subtree(ppr);
        for child in dom.elements(ppr_clone, None) {
            let drop = dom.name_is(child, &W::sect_pr())
                || dom.name_is(child, &W::p_pr_change())
                || dom.name_is(child, &W::r_pr());
            if drop {
                dom.remove(child);
            }
        }
        if let Some(style) = style {
            if let Some(existing) = dom.element(ppr_clone, &W::p_style()) {
                dom.set_attribute_value(existing, &W::val(), Some(style));
            } else {
                let el = dom.new_element(W::p_style());
                dom.set_attribute_value(el, &W::val(), Some(style));
                dom.add_first(ppr_clone, el);
            }
        }
        dom.add(p, ppr_clone);
    } else if let Some(style) = style {
        let ppr = dom.new_element(W::p_pr());
        let el = dom.new_element(W::p_style());
        dom.set_attribute_value(el, &W::val(), Some(style));
        dom.add(ppr, el);
        dom.add(p, ppr);
    }
    // New runs start from the anchor's dominant run, the one holding the most
    // text: a bold lead-in ("(f) Notice of Inability to Comply.") must not make
    // every inserted run bold.
    let base_rpr = dom
        .elements(anchor, Some(&W::r()))
        .into_iter()
        .rev()
        .max_by_key(|&r| {
            dom.elements(r, Some(&W::t()))
                .iter()
                .map(|&t| dom.value(t).chars().count())
                .sum::<usize>()
        })
        .and_then(|r| dom.element(r, &W::r_pr()));
    for spec in runs {
        if spec.text.is_empty() {
            continue;
        }
        let r = dom.new_element(W::r());
        let rpr = match base_rpr {
            Some(rpr) => dom.clone_subtree(rpr),
            None => dom.new_element(W::r_pr()),
        };
        for child in dom.elements(rpr, None) {
            if dom.name_is(child, &W::r_pr_change())
                || dom.name_is(child, &W::ins())
                || dom.name_is(child, &W::del())
            {
                dom.remove(child);
            }
        }
        apply_run_format(dom, rpr, &spec.format());
        if dom.elements(rpr, None).is_empty() {
            dom.remove(rpr);
        } else {
            dom.add(r, rpr);
        }
        let t = dom.new_element(W::t());
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t, &spec.text);
        dom.add(r, t);
        dom.add(p, r);
    }
    p
}

/// Set or clear the requested properties in `rpr`.
fn apply_run_format(dom: &mut Dom, rpr: NodeId, format: &RunFormat) {
    toggle(dom, rpr, "b", format.bold);
    toggle(dom, rpr, "bCs", format.bold);
    toggle(dom, rpr, "i", format.italic);
    toggle(dom, rpr, "iCs", format.italic);
    if let Some(underline) = format.underline {
        if let Some(u) = dom.element(rpr, &W::name("u")) {
            dom.remove(u);
        }
        if underline {
            let u = dom.new_element(W::name("u"));
            dom.set_attribute_value(u, &W::val(), Some("single"));
            insert_rpr_child(dom, rpr, u);
        }
    }
    if let Some(highlight) = &format.highlight {
        if let Some(h) = dom.element(rpr, &W::name("highlight")) {
            dom.remove(h);
        }
        if highlight != "none" {
            let h = dom.new_element(W::name("highlight"));
            dom.set_attribute_value(h, &W::val(), Some(highlight));
            insert_rpr_child(dom, rpr, h);
        }
    }
    toggle(dom, rpr, "strike", format.strike);
    toggle(dom, rpr, "caps", format.caps);
    if let Some(font) = &format.font {
        if let Some(old) = dom.element(rpr, &W::name("rFonts")) {
            dom.remove(old);
        }
        let fonts = dom.new_element(W::name("rFonts"));
        for script in ["ascii", "hAnsi", "eastAsia", "cs"] {
            dom.set_attribute_value(fonts, &W::name(script), Some(font));
        }
        insert_rpr_child(dom, rpr, fonts);
    }
    if let Some(color) = &format.color {
        if let Some(old) = dom.element(rpr, &W::name("color")) {
            dom.remove(old);
        }
        let el = dom.new_element(W::name("color"));
        dom.set_attribute_value(el, &W::val(), Some(color));
        insert_rpr_child(dom, rpr, el);
    }
    if let Some(HalfPoints(size)) = format.size_pt {
        for local in ["sz", "szCs"] {
            if let Some(old) = dom.element(rpr, &W::name(local)) {
                dom.remove(old);
            }
            let el = dom.new_element(W::name(local));
            dom.set_attribute_value(el, &W::val(), Some(&size.to_string()));
            insert_rpr_child(dom, rpr, el);
        }
    }
}

/// `ST_HighlightColor`.
const HIGHLIGHTS: &[&str] = &[
    "black",
    "blue",
    "cyan",
    "green",
    "magenta",
    "red",
    "yellow",
    "white",
    "darkBlue",
    "darkCyan",
    "darkGreen",
    "darkMagenta",
    "darkRed",
    "darkYellow",
    "darkGray",
    "lightGray",
    "none",
];

/// A format must name a real highlight colour and apply to some text.
fn check_format(format: &RunFormat, text: &str) -> Result<(), String> {
    if let Some(highlight) = &format.highlight
        && !HIGHLIGHTS.contains(&highlight.as_str())
    {
        return Err(format!(
            "highlight {highlight:?} is not a Word highlight colour ({})",
            HIGHLIGHTS.join(", ")
        ));
    }
    if let Some(font) = &format.font
        && (font.trim().is_empty()
            || font.chars().count() > 31
            || font.chars().any(char::is_control))
    {
        return Err(format!(
            "font {font:?} must be a nonempty name of at most 31 characters"
        ));
    }
    if let Some(color) = &format.color
        && color != "auto"
        && !(color.len() == 6 && color.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(format!(
            "color {color:?} must be six hex digits (FF0000) or auto"
        ));
    }
    if let Some(HalfPoints(size)) = format.size_pt
        && !(2..=3276).contains(&size)
    {
        return Err(format!(
            "size_pt {} is outside 1..=1638",
            f64::from(size) / 2.0
        ));
    }
    if text.is_empty() && *format != RunFormat::default() {
        return Err("format needs nonempty text".into());
    }
    Ok(())
}

/// Schema order of the `w:rPr` children this module writes.
const RPR_ORDER: &[&str] = &[
    "rStyle",
    "rFonts",
    "b",
    "bCs",
    "i",
    "iCs",
    "caps",
    "smallCaps",
    "strike",
    "dstrike",
    "outline",
    "shadow",
    "emboss",
    "imprint",
    "noProof",
    "snapToGrid",
    "vanish",
    "webHidden",
    "color",
    "spacing",
    "w",
    "kern",
    "position",
    "sz",
    "szCs",
    "highlight",
    "u",
    "effect",
    "bdr",
    "shd",
    "fitText",
    "vertAlign",
    "rtl",
    "cs",
    "em",
    "lang",
    "eastAsianLayout",
    "specVanish",
    "oMath",
];

fn rank(local: &str) -> usize {
    RPR_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(RPR_ORDER.len())
}

/// Insert `child` into `rpr` at its schema position.
fn insert_rpr_child(dom: &mut Dom, rpr: NodeId, child: NodeId) {
    let my_rank = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let after = dom.elements(rpr, None).into_iter().rev().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && rank(n.local_name()) <= my_rank)
    });
    match after {
        Some(after) => dom.add_after_self(after, child),
        None => dom.add_first(rpr, child),
    }
}

/// Set or clear a boolean run property (`w:b`, `w:i`, ...).
fn toggle(dom: &mut Dom, rpr: NodeId, local: &str, value: Option<bool>) {
    let Some(value) = value else { return };
    if let Some(existing) = dom.element(rpr, &W::name(local)) {
        dom.remove(existing);
    }
    let el = dom.new_element(W::name(local));
    if !value {
        dom.set_attribute_value(el, &W::val(), Some("0"));
    }
    insert_rpr_child(dom, rpr, el);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn plan_json_roundtrips_and_rejects_unknown_fields() {
        let json = r#"{"schema_version":1,"author":"A","operations":[
            {"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b"},
            {"kind":"insert","paragraph":{"id":"body:p:1"},"after":"x","text":"y","comment":"c"},
            {"kind":"insert","paragraph":{"index":0},"position":"end","text":"z"},
            {"kind":"comment","paragraph":{"contains":"q"},"text":"note"},
            {"kind":"insert_paragraph","paragraph":{"starts_with":"s"},"runs":[{"text":"t","bold":true}]},
            {"kind":"delete_paragraph","paragraph":{"index":2}}]}"#;
        let plan = EditPlan::from_json(json).unwrap();
        assert_eq!(plan.operations.len(), 6);
        assert_eq!(plan.existing_revisions, ExistingRevisions::Refuse);
        let again = EditPlan::from_json(&plan.to_json()).unwrap();
        assert_eq!(again, plan);
        let bad = r#"{"schema_version":1,"author":"A","extra":1,"operations":[]}"#;
        assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
        let bad = r#"{"schema_version":1,"author":"A","operations":[{"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b","typo":1}]}"#;
        assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
        let e = EditPlan::from_json("{").unwrap_err();
        assert!(e.to_string().starts_with("INVALID_PLAN: "));
    }

    #[test]
    fn context_windows_and_excerpts() {
        let text = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJ";
        let c = context(text, 24, 26, "{op}");
        assert_eq!(c, "456789abcdefghijklmn{op}qrstuvwxyzABCDEFGHIJ");
        assert_eq!(context("ab", 0, 0, "{+x}"), "{+x}ab");
        assert_eq!(excerpt("short", 10), "short");
        assert_eq!(excerpt("a longer sentence", 8), "a longer…");
    }

    #[test]
    fn new_position_accounts_for_earlier_edits_and_insertions() {
        let edit = |start, end, op, replacement: &str| ScheduledEdit {
            start,
            end,
            op,
            replacement: replacement.to_string(),
            attach_before: true,
            comment: None,
            format: None,
        };
        let edits = vec![
            edit(2, 4, 0, "XYZ"), // +1
            edit(6, 6, 1, "++"),  // insertion at 6
            edit(8, 9, 2, ""),    // -1
        ];
        assert_eq!(new_position(&edits, 1, true, None), 1);
        assert_eq!(new_position(&edits, 5, true, None), 6);
        assert_eq!(
            new_position(&edits, 6, true, None),
            9,
            "inclusive shifts past the insertion at 6"
        );
        assert_eq!(
            new_position(&edits, 6, false, None),
            7,
            "exclusive does not"
        );
        assert_eq!(new_position(&edits, 10, true, None), 12);
        assert_eq!(
            new_position(&edits, 6, true, Some(1)),
            7,
            "an operation's own insertion is skipped"
        );
    }

    #[test]
    fn check_text_rejects_controls() {
        assert!(check_text("plain “quotes” é").is_ok());
        assert!(check_text("tab\there").is_err());
        assert!(check_text("line\nbreak").is_err());
        assert!(check_text("\u{ffff}").is_err());
    }

    #[test]
    fn rpr_children_are_inserted_in_schema_order() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:rPr xmlns:w="{}"><w:rFonts w:ascii="Arial"/><w:sz w:val="22"/></w:rPr>"#,
            W::URI
        ));
        let rpr = dom.root(doc).unwrap();
        toggle(&mut dom, rpr, "b", Some(true));
        toggle(&mut dom, rpr, "i", Some(false));
        let h = dom.new_element(W::name("highlight"));
        insert_rpr_child(&mut dom, rpr, h);
        let names: Vec<String> = dom
            .elements(rpr, None)
            .iter()
            .map(|&c| dom.name(c).unwrap().local_name().to_string())
            .collect();
        assert_eq!(names, ["rFonts", "b", "i", "sz", "highlight"]);
        let i = dom.element(rpr, &W::name("i")).unwrap();
        assert_eq!(dom.attribute(i, &W::val()), Some("0"));
        toggle(&mut dom, rpr, "b", None);
        assert_eq!(dom.elements(rpr, None).len(), 5);
    }

    #[test]
    fn error_display_names_the_operation() {
        let e = err("X", Some("op-2"), "why");
        assert_eq!(e.to_string(), "X (op-2): why");
        let e = err("X", None, "why");
        assert_eq!(e.to_string(), "X: why");
    }
}
// Reuse the repository's existing, entirely in-memory package fixture.
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[path = "../tests/common/docx.rs"]
mod deeper_boundary_fixture;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod deeper_boundary_tests {
    use super::deeper_boundary_fixture::{self as fixture, Part, docx, docx_with, para};
    use super::*;

    fn plan(operations: &str) -> EditPlan {
        EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Boundary","date":"2026-01-01T00:00:00Z","operations":{operations}}}"#
        )).unwrap()
    }

    fn dom(xml: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            r#"<root xmlns:w="{}" xmlns:x="urn:foreign">{xml}</root>"#,
            W::URI
        ));
        let root = dom.root(document).unwrap();
        (dom, root)
    }

    fn names(dom: &Dom, parent: NodeId) -> Vec<String> {
        dom.elements(parent, None)
            .iter()
            .map(|&n| dom.name(n).unwrap().local_name().to_string())
            .collect()
    }

    fn outcome(id: &str) -> EditOutcome {
        EditOutcome {
            id: id.into(),
            kind: "boundary".into(),
            status: "ok".into(),
            paragraph: None,
            matches: 1,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
        }
    }

    // Baseline gaps: 113, 916, 933, 936, 942, 945, 958.
    #[test]
    fn report_optional_fields_and_resolution_truth_table() {
        let source = docx(&format!(
            "<w:p>{}</w:p>",
            fixture::run("source", false, false, None)
        ));
        let plan = plan("[]");
        let tx = Transaction::start(&source, &plan).unwrap();
        for (accepted, rejected, empty) in [
            (vec![], vec![], true),
            (vec!["a"], vec![], false),
            (vec![], vec!["r"], false),
            (vec!["a"], vec!["r"], false),
        ] {
            let resolved = ResolvedRevisions {
                accepted: accepted.into_iter().map(String::from).collect(),
                rejected: rejected.into_iter().map(String::from).collect(),
            };
            assert_eq!(resolved.is_empty(), empty);
            let mut report = tx.report(false);
            report.resolved_revisions = resolved.clone();
            let mut failed = outcome("bad");
            failed.status = "failed".into();
            failed.code = Some("INVALID_EDIT".into());
            failed.message = Some("boundary refusal".into());
            report.operations = vec![failed];
            let rows: Vec<serde_json::Value> = report
                .to_jsonl()
                .lines()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect();
            assert_eq!(rows.len(), 3);
            let expected_resolved = serde_json::to_value(&resolved).unwrap();
            assert_eq!(
                rows[0].get("resolved_revisions"),
                if empty {
                    None
                } else {
                    Some(&expected_resolved)
                }
            );
            assert_eq!(
                rows[1],
                serde_json::json!({
                    "ev":"op", "i":1, "id":"bad", "op":"boundary", "status":"failed",
                    "matches":1, "code":"INVALID_EDIT", "message":"boundary refusal"
                })
            );
            assert_eq!(rows[2]["status"], "failed");
            assert_eq!(rows[2]["ops"], serde_json::json!({"ok":0,"failed":1}));
        }
    }

    // 702's two disjuncts, including values JSON cannot represent.
    #[test]
    fn half_points_finite_range_and_rounding_boundaries() {
        use serde::de::value::{Error, F64Deserializer};
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1638.01] {
            let result = HalfPoints::deserialize(F64Deserializer::<Error>::new(value));
            assert_eq!(
                result.unwrap_err().to_string(),
                format!("size_pt {value} is outside 0..=1638")
            );
        }
        for (value, expected) in [(-0.0, 0), (0.24, 0), (0.25, 1), (0.75, 2), (1638.0, 3276)] {
            assert_eq!(
                HalfPoints::deserialize(F64Deserializer::<Error>::new(value)).unwrap(),
                HalfPoints(expected)
            );
        }
    }

    // 1055, 1069, 1073: no selection, empty filters, reject-only selection.
    #[test]
    fn revision_selection_empty_and_reject_only_preserve_unselected_change() {
        let source = docx(
            r#"<w:p><w:ins w:id="1" w:author="A"><w:r><w:t>one</w:t></w:r></w:ins><w:ins w:id="2" w:author="B"><w:r><w:t>two</w:t></w:r></w:ins></w:p>"#,
        );
        for selection in [
            ResolveRevisions::default(),
            ResolveRevisions {
                accept: Some(ChangeFilter::ids(Vec::<String>::new())),
                reject: Some(ChangeFilter::ids(Vec::<String>::new())),
            },
        ] {
            assert_eq!(
                resolve_selected(&source, Some(&selection)).unwrap(),
                (None, ResolvedRevisions::default())
            );
        }
        let selection = ResolveRevisions {
            accept: None,
            reject: Some(ChangeFilter::ids(["body:rev:1"])),
        };
        let (bytes, resolved) = resolve_selected(&source, Some(&selection)).unwrap();
        assert_eq!(
            resolved,
            ResolvedRevisions {
                accepted: vec![],
                rejected: vec!["body:rev:1".into()]
            }
        );
        let bytes = bytes.unwrap();
        let changes = crate::changes::list_changes(&bytes).unwrap();
        assert_eq!(
            changes.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["body:rev:2"]
        );
        assert_eq!(crate::inspect::paragraphs(&bytes).unwrap()[0].text, "two");
    }

    // 1652, 1669, 1677; validation precedes package parsing.
    #[test]
    fn admission_boundaries_and_signature_prefix_are_exact() {
        let mut bad = plan("[]");
        bad.schema_version = SCHEMA_VERSION + 1;
        let e = preview_plan(b"", &bad).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("UNSUPPORTED_SCHEMA", "schema_version 2 is not supported")
        );
        bad.schema_version = SCHEMA_VERSION;
        bad.author = " \t\n".into();
        let e = preview_plan(b"", &bad).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("INVALID_PLAN", "author must be nonempty")
        );
        let source = docx(&para("source"));
        for (name, refused) in [
            ("_xmlsignatures/sig.xml", true),
            ("word/vbaProject.bin", true),
            ("word/not-vbaProject.bin.xml", false),
        ] {
            let mut pkg = crate::opc::PartFs::open(&source).unwrap();
            pkg.set_part(name, b"<probe/>".to_vec());
            let bytes = pkg.to_zip().unwrap();
            let result = preview_plan(&bytes, &plan("[]"));
            if refused {
                let e = result.unwrap_err();
                assert_eq!(
                    (e.code.as_str(), e.message.as_str()),
                    ("UNSUPPORTED_PACKAGE", "signed or macro-bearing package")
                );
            } else {
                assert_eq!(
                    result.unwrap().paragraphs,
                    ParagraphDelta { from: 1, to: 1 }
                );
            }
        }
    }

    // 1376, 1389, 4519: invalid byte input is handled without panic.
    #[test]
    fn invalid_package_has_no_revisions_or_kept_comment_text() {
        assert_eq!(
            revision_counts(b"invalid zip", &WmlComparerSettings::default()),
            RevisionCounts::default()
        );
        assert!(!comment_holds_kept_text(b"invalid zip", 0));
    }

    // 2259, 2312, 2339, 2503, 2703, 3173.
    #[test]
    fn resolution_empty_text_and_combined_comment_anchors() {
        let source = docx("<w:p/><w:p><w:r><w:t>tail</w:t></w:r></w:p>");
        for (ops, message) in [
            (
                r#"[{"kind":"comment","paragraph":"body:p:0","find":"x","through":"body:p:1","text":"n"}]"#,
                "find and through cannot be combined: through comments on whole paragraphs",
            ),
            (
                r#"[{"kind":"insert_paragraph","paragraph":"body:p:0","runs":[]}]"#,
                "runs must carry text",
            ),
            (
                r#"[{"kind":"insert_paragraph","paragraph":"body:p:0","runs":[{"text":""},{"text":""}]}]"#,
                "runs must carry text",
            ),
            (
                r#"[{"kind":"insert_toc","paragraph":"body:p:0","title":""}]"#,
                "title must carry text",
            ),
            (
                r#"[{"kind":"comment","paragraph":{"contains":""},"text":"n"}]"#,
                "paragraph selector text must be nonempty",
            ),
        ] {
            let e = preview_plan(&source, &plan(ops)).unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.message.as_str()),
                ("INVALID_EDIT", message),
                "{ops}"
            );
            assert_eq!(e.outcomes[0].message.as_deref(), Some(message));
        }
        for ops in [
            r#"[{"kind":"comment","paragraph":"body:p:0","text":"empty anchor"}]"#,
            r#"[{"kind":"delete_paragraph","paragraph":"body:p:0","comment":"empty deletion"}]"#,
        ] {
            let p = plan(ops);
            let mut tx = Transaction::start(&source, &p).unwrap();
            tx.resolve().unwrap();
            assert_eq!(tx.outcomes[0].matches, 1);
            tx.apply().unwrap();
            if ops.contains("delete_paragraph") {
                assert_eq!(tx.comments, Vec::<(u32, String)>::new());
                assert_eq!(tx.deletion_comments, [(0, 0)]);
                assert_eq!(tx.outcomes[0].comment_id, Some(0));
                assert_eq!(
                    tx.opened.dom.elements(tx.opened.body, Some(&W::p())).len(),
                    1
                );
            } else {
                assert_eq!(tx.comments, [(0, "empty anchor".into())]);
                assert_eq!(
                    tx.opened.dom.elements(tx.opened.body, Some(&W::p())).len(),
                    2
                );
            }
        }
    }

    // Body-only operations with a real in-memory header story: 2432, 2688, 2987.
    #[test]
    fn header_refuses_body_only_picture_toc_and_list() {
        let header = format!(r#"<w:hdr xmlns:w="{}">{}</w:hdr>"#, W::URI, para("header"));
        let source = fixture::docx_with_sect(
            &para("body"),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        );
        for (ops, message) in [
            (
                r#"[{"kind":"insert_image","paragraph":"header1:p:0","image_base64":""}]"#,
                "pictures can be inserted in the body only",
            ),
            (
                r#"[{"kind":"insert_toc","paragraph":"header1:p:0"}]"#,
                "a table of contents goes in the body",
            ),
            (
                r#"[{"kind":"list","paragraphs":["header1:p:0"],"kind_of_list":"bullet"}]"#,
                "paragraphs[0]: list is supported in the body only, not header1:p:0",
            ),
        ] {
            let e = preview_plan(&source, &plan(ops)).unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.message.as_str()),
                ("UNSUPPORTED_STRUCTURE", message)
            );
        }
    }

    // 3109, 3114: a related style part can disappear/become rootless.
    #[test]
    fn paragraph_styles_missing_and_rootless_parts() {
        let styles = format!(
            r#"<w:styles xmlns:w="{}"><w:style w:styleId="S" w:type="paragraph"><w:name w:val="Style"/></w:style></w:styles>"#,
            W::URI
        );
        let source = docx_with(
            &para("body"),
            &[Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &styles,
            }],
        );
        let p = plan("[]");
        let mut tx = Transaction::start(&source, &p).unwrap();
        assert_eq!(tx.paragraph_styles(), [("S".into(), "Style".into())]);
        tx.opened
            .pkg
            .set_part("word/styles.xml", b"<?xml version=\"1.0\"?>".to_vec());
        assert_eq!(tx.paragraph_styles(), Vec::<(String, String)>::new());
        tx.opened.pkg.remove_part("word/styles.xml");
        assert_eq!(tx.paragraph_styles(), Vec::<(String, String)>::new());
    }

    // 3336, 3342, 3368: end coverage, field-boundary equality, and glyph attachment.
    #[test]
    fn projection_range_and_insert_boundary_truth_tables() {
        let source = docx(&para("abc"));
        let p = plan("[]");
        let tx = Transaction::start(&source, &p).unwrap();
        let mut projection = tx.projections[0].clone();
        assert_eq!(
            tx.check_range(&projection, 0, 4),
            Err("the text is not addressable".into())
        );
        projection.field_marks = vec![0, 3];
        assert_eq!(tx.check_range(&projection, 0, 3), Ok(()));
        projection.field_marks = vec![1];
        assert_eq!(
            tx.check_range(&projection, 0, 3),
            Err("the text sits inside a hyperlink, field, content control or revision".into())
        );
        let (d, root) = dom("<w:p><w:r><w:tab/></w:r></w:p>");
        let paragraph = d.element(root, &W::p()).unwrap();
        let glyph = project_paragraph(&d, paragraph);
        for before in [false, true] {
            for at in [0, 1] {
                assert_eq!(
                    tx.check_insert_position(&glyph, at, before),
                    Err("the insertion point touches a tab, break or symbol".into())
                );
            }
        }
        assert_eq!(
            tx.check_insert_position(&projection, 4, false),
            Err("no run holds the insertion point".into())
        );
    }

    fn text_range(start: usize, end: usize) -> Resolved {
        Resolved::Text {
            para: 0,
            start,
            end,
            replacement: "X".into(),
            attach_before: false,
            comment: None,
            format: None,
        }
    }

    fn conflicts(source: &[u8], resolved: Vec<Resolved>) -> Result<(), EditError> {
        let mut p = plan("[]");
        // Conflict checks still consult the original operation metadata (in
        // particular deleted-paragraph comments), so keep it alongside each
        // synthetic resolution rather than constructing an empty plan.
        let operations = {
            let seed = Transaction::start(source, &p).unwrap();
            let selector = |index| Selector::Index { index, story: None };
            let text = |para: usize, start: usize, end: usize| {
                seed.projections[para]
                    .text
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect::<String>()
            };
            resolved
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let kind = match r {
                        Resolved::Text {
                            para,
                            start,
                            end,
                            replacement,
                            comment,
                            format,
                            attach_before,
                        } => {
                            if start == end {
                                OperationKind::Insert {
                                    paragraph: selector(*para),
                                    after: (!*attach_before && *start > 0)
                                        .then(|| text(*para, 0, *start)),
                                    before: (*attach_before).then(|| {
                                        text(
                                            *para,
                                            *start,
                                            seed.projections[*para].text.chars().count(),
                                        )
                                    }),
                                    position: (!*attach_before && *start == 0)
                                        .then_some(Edge::Start),
                                    text: replacement.clone(),
                                    format: format.clone(),
                                    comment: comment.clone(),
                                    occurrence: None,
                                }
                            } else {
                                OperationKind::Replace {
                                    paragraph: selector(*para),
                                    find: text(*para, *start, *end),
                                    replacement: replacement.clone(),
                                    format: format.clone(),
                                    comment: comment.clone(),
                                    whole: false,
                                    occurrence: None,
                                }
                            }
                        }
                        Resolved::DeleteParagraph { para } => OperationKind::DeleteParagraph {
                            paragraph: selector(*para),
                            comment: None,
                        },
                        Resolved::CommentRange {
                            para,
                            start,
                            end,
                            text: note,
                        } => OperationKind::Comment {
                            paragraph: selector(*para),
                            find: Some(text(*para, *start, *end)),
                            text: note.clone(),
                            through: None,
                            occurrence: None,
                        },
                        Resolved::CommentSpan { para, last, text } => OperationKind::Comment {
                            paragraph: selector(*para),
                            find: None,
                            text: text.clone(),
                            through: Some(selector(*last)),
                            occurrence: None,
                        },
                        Resolved::InsertParagraph {
                            anchor,
                            side,
                            runs,
                            like,
                            style,
                            comment,
                            toc: None,
                        } => OperationKind::InsertParagraph {
                            paragraph: selector(*anchor),
                            position: *side,
                            runs: runs.clone(),
                            like: Some(selector(*like)),
                            style: style.clone(),
                            comment: comment.clone(),
                        },
                        Resolved::InsertTable {
                            anchor,
                            side,
                            rows,
                            header_row,
                            widths,
                            style,
                            ..
                        } => OperationKind::InsertTable {
                            paragraph: selector(*anchor),
                            position: *side,
                            rows: rows.clone(),
                            header_row: *header_row,
                            widths_dxa: Some(widths.clone()),
                            style: Some(style.clone()),
                        },
                        Resolved::FormatRun {
                            para,
                            start,
                            end,
                            format,
                        } => OperationKind::FormatRun {
                            paragraph: selector(*para),
                            find: text(*para, *start, *end),
                            format: format.clone(),
                            occurrence: None,
                        },
                        Resolved::InsertFootnote {
                            para,
                            at,
                            text: note,
                        } => OperationKind::InsertFootnote {
                            paragraph: selector(*para),
                            after: text(*para, 0, *at),
                            text: note.clone(),
                            occurrence: None,
                        },
                        Resolved::FormatParagraph {
                            para,
                            style,
                            alignment,
                            spacing,
                        } => OperationKind::FormatParagraph {
                            paragraph: selector(*para),
                            style: style.clone(),
                            alignment: *alignment,
                            line_spacing: spacing.line,
                            space_before: spacing.before,
                            space_after: spacing.after,
                        },
                        Resolved::MergeParagraphs {
                            para, separator, ..
                        } => OperationKind::MergeParagraphs {
                            paragraph: selector(*para),
                            separator: Some(separator.clone()),
                        },
                        Resolved::Thread { op, .. } => match op {
                            ThreadOp::Reply { parent, text } => OperationKind::ReplyComment {
                                comment_id: *parent,
                                text: text.clone(),
                            },
                            ThreadOp::Resolve { id, done } => OperationKind::ResolveComment {
                                comment_id: *id,
                                done: *done,
                            },
                            ThreadOp::Edit { id, text } => OperationKind::EditComment {
                                comment_id: *id,
                                text: text.clone(),
                            },
                            ThreadOp::Delete { id } => {
                                OperationKind::DeleteComment { comment_id: *id }
                            }
                        },
                        _ => panic!("conflict fixture needs matching operation metadata"),
                    };
                    Operation {
                        id: Some(format!("case-{i}")),
                        kind,
                    }
                })
                .collect()
        };
        p.operations = operations;
        let mut tx = Transaction::start(source, &p).unwrap();
        tx.outcomes = (0..resolved.len())
            .map(|i| outcome(&format!("case-{i}")))
            .collect();
        tx.resolved = resolved.into_iter().enumerate().collect();
        tx.check_conflicts()
    }

    // 3460/3461 and 3477: same-start replacements vs insertions, and either comment edge.
    #[test]
    fn overlapping_range_and_comment_edge_truth_tables() {
        let source = docx(&para("abcdefgh"));
        for (left, right, culprit) in [
            ((1, 4), (1, 2), Some("case-0")),
            ((1, 1), (1, 4), None),
            ((1, 4), (4, 4), None),
            ((1, 4), (2, 2), Some("case-1")),
            ((1, 4), (4, 7), None),
            ((1, 1), (1, 1), None),
        ] {
            let result = conflicts(
                &source,
                vec![text_range(left.0, left.1), text_range(right.0, right.1)],
            );
            if let Some(culprit) = culprit {
                let e = result.unwrap_err();
                assert_eq!(
                    (e.code.as_str(), e.message.as_str()),
                    ("OVERLAPPING_EDITS", "overlaps an earlier edit's text range")
                );
                assert_eq!(e.operation.as_deref(), Some(culprit));
            } else {
                assert_eq!(result, Ok(()));
            }
        }
        for (start, end, conflict) in [
            (2, 5, true),
            (0, 2, true),
            (0, 4, false),
            (1, 4, false),
            (4, 7, false),
            (1, 1, false),
        ] {
            let result = conflicts(
                &source,
                vec![
                    text_range(1, 4),
                    Resolved::CommentRange {
                        para: 0,
                        start,
                        end,
                        text: "note".into(),
                    },
                ],
            );
            if conflict {
                let e = result.unwrap_err();
                assert_eq!(e.message, "comment range cuts through an edited range");
                assert_eq!(e.operation.as_deref(), Some("case-1"));
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }

    // All deletion conflict arms use explicit functional error assertions, not a call sweep.
    #[test]
    fn deleted_paragraph_rejects_each_anchor_owner() {
        let source = docx(&(para("one") + &para("two") + &para("three")));
        let cases = vec![
            (
                Resolved::CommentRange {
                    para: 0,
                    start: 0,
                    end: 1,
                    text: "n".into(),
                },
                "comments on a deleted paragraph",
            ),
            (
                Resolved::InsertParagraph {
                    anchor: 0,
                    side: Side::Before,
                    runs: vec![],
                    like: 0,
                    style: None,
                    comment: None,
                    toc: None,
                },
                "anchors a new paragraph on a deleted paragraph",
            ),
            (
                Resolved::InsertTable {
                    anchor: 0,
                    side: Side::After,
                    rows: vec![vec!["cell".into()]],
                    header_row: false,
                    widths: vec![100],
                    style: "TableGrid".into(),
                    add_style: false,
                },
                "anchors a new table on a deleted paragraph",
            ),
            (
                Resolved::FormatRun {
                    para: 0,
                    start: 0,
                    end: 1,
                    format: RunFormat {
                        bold: Some(true),
                        ..RunFormat::default()
                    },
                },
                "formats text of a deleted paragraph",
            ),
            (
                Resolved::InsertFootnote {
                    para: 0,
                    at: 1,
                    text: "n".into(),
                },
                "adds a footnote to a deleted paragraph",
            ),
            (
                Resolved::CommentSpan {
                    para: 0,
                    last: 1,
                    text: "n".into(),
                },
                "comments on a deleted paragraph",
            ),
            (
                Resolved::Thread {
                    op: ThreadOp::Reply {
                        parent: 7,
                        text: "n".into(),
                    },
                    stories: vec![0],
                    anchor: Some(0),
                },
                "replies to a comment whose reference is in a deleted paragraph",
            ),
            (
                Resolved::FormatParagraph {
                    para: 0,
                    style: None,
                    alignment: Some(Alignment::Left),
                    spacing: Spacing::default(),
                },
                "formats a deleted paragraph",
            ),
            (
                Resolved::MergeParagraphs {
                    para: 0,
                    next: 1,
                    separator: "".into(),
                },
                "merges a deleted paragraph",
            ),
        ];
        for (r, message) in cases {
            let e = conflicts(&source, vec![Resolved::DeleteParagraph { para: 0 }, r]).unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.message.as_str()),
                ("OVERLAPPING_EDITS", message)
            );
            assert_eq!(e.operation.as_deref(), Some("case-1"));
            assert_eq!(e.outcomes[0].status, "ok");
            assert_eq!(e.outcomes[1].message.as_deref(), Some(message));
        }
        let e = conflicts(
            &source,
            vec![
                Resolved::DeleteParagraph { para: 0 },
                Resolved::DeleteParagraph { para: 1 },
                Resolved::DeleteParagraph { para: 2 },
            ],
        )
        .unwrap_err();
        assert_eq!(
            e.message,
            "the plan's deletions leave the body without a paragraph"
        );
    }

    // 3586's Before/tail arm, with a table rather than the existing paragraph integration case.
    #[test]
    fn table_insertion_sides_at_merge_boundaries() {
        let source = docx(&(para("one") + &para("two")));
        for (anchor, side, conflict) in [
            (0, Side::Before, false),
            (0, Side::After, true),
            (1, Side::Before, true),
            (1, Side::After, false),
        ] {
            let result = conflicts(
                &source,
                vec![
                    Resolved::MergeParagraphs {
                        para: 0,
                        next: 1,
                        separator: "".into(),
                    },
                    Resolved::InsertTable {
                        anchor,
                        side,
                        rows: vec![vec!["cell".into()]],
                        header_row: false,
                        widths: vec![100],
                        style: "TableGrid".into(),
                        add_style: false,
                    },
                ],
            );
            if conflict {
                assert_eq!(
                    result.unwrap_err().message,
                    "inserts a table between two a merge joins"
                );
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }

    // 2804, 2842, 2851, 3667, 3680.
    #[test]
    fn comment_reference_identity_and_thread_conflicts() {
        let comments = format!(
            r#"<w:comments xmlns:w="{}"><w:comment w:id="7" w:author="A"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
            W::URI
        );
        let source = docx_with(
            r#"<w:p><w:commentRangeStart/><w:commentRangeStart w:id="7"/><w:r><w:t>body</w:t><w:commentReference w:id="7"/><w:commentReference w:id="7"/></w:r><w:commentRangeEnd w:id="7"/></w:p>"#,
            &[Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            }],
        );
        let p = plan("[]");
        let mut tx = Transaction::start(&source, &p).unwrap();
        let (stories, reference) = tx.comment_markers(&[7], 7);
        assert_eq!(stories, [0]);
        let references = tx
            .opened
            .dom
            .descendants(tx.opened.body, Some(&W::name("commentReference")));
        assert_eq!(reference, Some(references[0]));
        for r in references {
            tx.opened.dom.remove(r);
        }
        let (e, _) = *tx
            .resolve_thread(
                "reply",
                &OperationKind::ReplyComment {
                    comment_id: 7,
                    text: "reply".into(),
                },
            )
            .unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            (
                "UNSUPPORTED_STRUCTURE",
                "comment 7 has no reference in the document to anchor a reply on"
            )
        );
        for ops in [
            vec![
                ThreadOp::Edit {
                    id: 7,
                    text: "a".into(),
                },
                ThreadOp::Edit {
                    id: 7,
                    text: "b".into(),
                },
            ],
            vec![
                ThreadOp::Delete { id: 7 },
                ThreadOp::Resolve { id: 7, done: true },
            ],
        ] {
            let edit = matches!(ops[0], ThreadOp::Edit { .. });
            let e = conflicts(
                &source,
                ops.into_iter()
                    .map(|op| Resolved::Thread {
                        op,
                        stories: vec![0],
                        anchor: Some(0),
                    })
                    .collect(),
            )
            .unwrap_err();
            assert_eq!(
                e.message,
                if edit {
                    "edits a comment another operation edits"
                } else {
                    "acts on a comment another operation deletes"
                }
            );
            assert_eq!(e.operation.as_deref(), Some("case-1"));
        }
    }

    // 4697, 4713, 4726: insertion affinity and glyphs never turn into text nodes.
    #[test]
    fn text_edit_glyphs_and_segment_attachment_are_exact() {
        let (mut d, root) = dom("<w:p><w:r><w:t>ab</w:t><w:tab/><w:t>cd</w:t></w:r></w:p>");
        let p = d.element(root, &W::p()).unwrap();
        let projection = project_paragraph(&d, p);
        for (pos, before, index) in [
            (0, false, 0),
            (2, true, 0),
            (2, false, 1),
            (3, true, 1),
            (3, false, 2),
            (5, false, 2),
        ] {
            assert_eq!(
                attach_segment(&projection, pos, before),
                Some(&projection.segments[index])
            );
        }
        assert_eq!(attach_segment(&projection, 6, false), None);
        apply_text_edit(&mut d, &projection, 2, 2, "ignored", false);
        assert_eq!(project_paragraph(&d, p).text, "ab\tcd");
        apply_text_edit(&mut d, &projection, 1, 4, "X", false);
        assert_eq!(project_paragraph(&d, p).text, "aX\td");
        assert_eq!(d.descendants(p, Some(&W::name("tab"))).len(), 1);
        let texts: Vec<String> = d
            .descendants(p, Some(&W::t()))
            .iter()
            .map(|&t| d.value(t))
            .collect();
        assert_eq!(texts, ["aX", "d"]);
    }

    // 4751, 4754, 4771, 4776: both edges, a glyph, and children on either side of t.
    #[test]
    fn splitting_multichild_run_preserves_order_and_properties() {
        let (mut d, root) =
            dom("<w:p><w:r><w:rPr><w:b/></w:rPr><w:tab/><w:t>abcd</w:t><w:br/></w:r></w:p>");
        let p = d.element(root, &W::p()).unwrap();
        let projection = project_paragraph(&d, p);
        let seg = &projection.segments[1];
        let before = d.serialize_element(p);
        for at in [seg.start, seg.end, seg.end + 1] {
            split_run_at(&mut d, seg, at);
            assert_eq!(d.serialize_element(p), before);
        }
        let (mut glyph_dom, glyph_root) = dom("<w:p><w:r><w:noBreakHyphen/></w:r></w:p>");
        let glyph_p = glyph_dom.element(glyph_root, &W::p()).unwrap();
        let glyph_projection = project_paragraph(&glyph_dom, glyph_p);
        split_run_at(&mut glyph_dom, &glyph_projection.segments[0], 1);
        assert_eq!(project_paragraph(&glyph_dom, glyph_p).text, "\u{2011}");
        assert_eq!(glyph_dom.elements(glyph_p, Some(&W::r())).len(), 1);
        assert_eq!(d.serialize_element(p), before);
        split_run_at(&mut d, seg, 3);
        let runs = d.elements(p, Some(&W::r()));
        assert_eq!(runs.len(), 2);
        assert_eq!(names(&d, runs[0]), ["rPr", "tab", "t"]);
        assert_eq!(names(&d, runs[1]), ["rPr", "t", "br"]);
        for r in &runs {
            let rpr = d.element(*r, &W::r_pr()).unwrap();
            assert_eq!(names(&d, rpr), ["b"]);
        }
        assert_eq!(d.value(d.element(runs[0], &W::t()).unwrap()), "ab");
        assert_eq!(d.value(d.element(runs[1], &W::t()).unwrap()), "cd");
        assert_eq!(project_paragraph(&d, p).text, "\tabcd\n");
    }

    // 4798, 4830, 4840, 4867, 4903, 4957.
    #[test]
    fn empty_comment_anchors_and_reference_cleanup() {
        let (mut d, root) = dom("<w:p><w:pPr><w:jc w:val=\"left\"/></w:pPr></w:p><w:p/>");
        let ps = d.elements(root, Some(&W::p()));
        anchor_span(&mut d, ps[0], ps[1], 7);
        assert_eq!(names(&d, ps[0]), ["pPr", "commentRangeStart"]);
        assert_eq!(names(&d, ps[1]), ["commentRangeEnd", "r"]);
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            let markers = d.descendants(root, Some(&W::name(local)));
            assert_eq!(markers.len(), 1);
            assert_eq!(d.attribute(markers[0], &W::id()), Some("7"));
        }
        anchor_comment(&mut d, ps[1], 0, 0, 8);
        assert_eq!(
            names(&d, ps[1]),
            [
                "commentRangeEnd",
                "r",
                "commentRangeStart",
                "commentRangeEnd",
                "r"
            ]
        );
        let before = d.serialize_element(root);
        place_reply_markers(&mut d, &[root], 999, 9);
        assert_eq!(d.serialize_element(root), before);
        let (mut d, root) = dom(
            r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:commentReference w:id="7"/><w:tab/></w:r><w:r><w:rPr/><w:commentReference w:id="8"/></w:r></w:p>"#,
        );
        let p = d.element(root, &W::p()).unwrap();
        remove_comment_markers(&mut d, &[root], &["7".into(), "8".into()]);
        assert_eq!(d.elements(p, Some(&W::r())).len(), 1);
        let r = d.element(p, &W::r()).unwrap();
        assert_eq!(names(&d, r), ["rPr", "tab"]);
        assert_eq!(project_paragraph(&d, p).text, "\t");
        let open = comment_marker(&mut d, "commentRangeStart", "9");
        let close = comment_marker(&mut d, "commentRangeEnd", "9");
        assert!(!wrap_range(&mut d, p, 0, 0, open, close));
        assert_eq!(d.parent(open), None);
        assert_eq!(d.parent(close), None);
        let (mut d, root) = dom("<w:p><w:r><w:t>a</w:t><w:t>b</w:t></w:r></w:p>");
        let p = d.element(root, &W::p()).unwrap();
        let open = comment_marker(&mut d, "commentRangeStart", "10");
        let close = comment_marker(&mut d, "commentRangeEnd", "10");
        assert!(!wrap_range(&mut d, p, 1, 1, open, close));
        assert_eq!(project_paragraph(&d, p).text, "ab");
        assert_eq!(d.parent(open), None);
        assert_eq!(d.parent(close), None);
    }

    // 4975, 4992, 4994, 5009: no-op ranges, empty t, and two segments in one run.
    #[test]
    fn format_range_deduplicates_runs_and_removes_empty_properties() {
        let (mut d, root) = dom(
            "<w:p><w:r><w:rPr><w:u/></w:rPr><w:t/><w:t>a</w:t><w:t>b</w:t></w:r><w:r><w:t>c</w:t></w:r></w:p>",
        );
        let p = d.element(root, &W::p()).unwrap();
        let format = RunFormat {
            underline: Some(false),
            ..RunFormat::default()
        };
        let original = d.serialize_element(p);
        for (start, end) in [(0, 0), (2, 1)] {
            format_range(&mut d, p, start, end, &format);
            assert_eq!(d.serialize_element(p), original);
        }
        format_range(&mut d, p, 0, 2, &format);
        assert_eq!(d.descendants(p, Some(&W::r_pr())).len(), 0);
        assert_eq!(d.elements(p, Some(&W::r())).len(), 2);
        assert_eq!(project_paragraph(&d, p).text, "abc");
    }

    // 5032, 5041, 5058, 5063, 5067; only the requested spacing axis changes.
    #[test]
    fn paragraph_format_partial_spacing_preserves_other_axes() {
        for (spacing, expected) in [
            (
                Spacing {
                    before: Some(Points(40)),
                    ..Spacing::default()
                },
                [Some("360"), Some("40"), Some("60"), None, Some("1")],
            ),
            (
                Spacing {
                    after: Some(Points(80)),
                    ..Spacing::default()
                },
                [Some("360"), Some("20"), Some("80"), Some("1"), None],
            ),
            (
                Spacing {
                    line: Some(LineSpacing(480)),
                    ..Spacing::default()
                },
                [Some("480"), Some("20"), Some("60"), Some("1"), Some("1")],
            ),
        ] {
            let (mut d, root) = dom(
                r#"<w:p><w:pPr><w:pStyle w:val="Old"/><w:spacing w:line="360" w:before="20" w:after="60" w:beforeAutospacing="1" w:afterAutospacing="1"/><w:jc w:val="left"/></w:pPr><w:r><w:t>x</w:t></w:r></w:p>"#,
            );
            let p = d.element(root, &W::p()).unwrap();
            format_paragraph(&mut d, p, Some("New"), Some(Alignment::Justify), spacing);
            let ppr = d.element(p, &W::p_pr()).unwrap();
            assert_eq!(names(&d, ppr), ["pStyle", "spacing", "jc"]);
            assert_eq!(
                d.attribute(d.element(ppr, &W::p_style()).unwrap(), &W::val()),
                Some("New")
            );
            assert_eq!(
                d.attribute(d.element(ppr, &W::name("jc")).unwrap(), &W::val()),
                Some("both")
            );
            let s = d.element(ppr, &W::name("spacing")).unwrap();
            for (local, value) in [
                "line",
                "before",
                "after",
                "beforeAutospacing",
                "afterAutospacing",
            ]
            .into_iter()
            .zip(expected)
            {
                assert_eq!(d.attribute(s, &W::name(local)), value, "{local}");
            }
            assert_eq!(project_paragraph(&d, p).text, "x");
        }
    }

    // 5125, 5482: higher-ranked and foreign children do not supply the insertion anchor.
    #[test]
    fn schema_insertion_ignores_foreign_and_later_properties() {
        for paragraph in [false, true] {
            let xml = if paragraph {
                "<w:pPr><x:pStyle/><w:sectPr/></w:pPr>"
            } else {
                "<w:rPr><x:rStyle/><w:sz/></w:rPr>"
            };
            let (mut d, root) = dom(xml);
            let parent = d.elements(root, None)[0];
            let child = d.new_element(if paragraph {
                W::p_style()
            } else {
                W::name("b")
            });
            if paragraph {
                insert_ppr_child(&mut d, parent, child);
            } else {
                insert_rpr_child(&mut d, parent, child);
            }
            assert_eq!(d.elements(parent, None)[0], child);
            assert_eq!(
                names(&d, parent),
                if paragraph {
                    vec!["pStyle", "pStyle", "sectPr"]
                } else {
                    vec!["b", "rStyle", "sz"]
                }
            );
            assert_eq!(
                d.name(d.elements(parent, None)[1])
                    .unwrap()
                    .namespace_name(),
                "urn:foreign"
            );
        }
        let (d, root) = dom("<w:bookmarkStart/><x:bookmarkStart/><w:r/>");
        let children = d.elements(root, None);
        assert!(is_range_markup(&d, children[0]));
        assert!(!is_range_markup(&d, children[1]));
        assert!(!is_range_markup(&d, children[2]));
    }

    // 5142, 5148, 5151-5153: separator inherits clean properties only.
    #[test]
    fn merge_separator_copies_properties_without_revision_children() {
        for separator in ["", " / "] {
            let (mut d, root) = dom(
                r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange/><w:ins/><w:del/></w:rPr><w:t>head</w:t></w:r></w:p><w:bookmarkStart w:id="1"/><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>tail</w:t></w:r></w:p>"#,
            );
            let ps = d.elements(root, Some(&W::p()));
            merge_into(&mut d, ps[0], ps[1], separator);
            assert_eq!(d.elements(root, Some(&W::p())), [ps[1]]);
            assert_eq!(
                project_paragraph(&d, ps[1]).text,
                format!("head{separator}tail")
            );
            assert_eq!(
                d.descendants(ps[1], Some(&W::name("bookmarkStart"))).len(),
                1
            );
            let runs = d.elements(ps[1], Some(&W::r()));
            assert_eq!(runs.len(), if separator.is_empty() { 2 } else { 3 });
            if !separator.is_empty() {
                let rpr = d.element(runs[1], &W::r_pr()).unwrap();
                assert_eq!(names(&d, rpr), ["b"]);
                assert_eq!(d.value(d.element(runs[1], &W::t()).unwrap()), separator);
            }
            let ppr = d.element(ps[1], &W::p_pr()).unwrap();
            assert_eq!(
                d.attribute(d.element(ppr, &W::name("jc")).unwrap(), &W::val()),
                Some("right")
            );
        }
    }

    // 5238, 5244, 5245, 5276, 5285-5287: property cloning and empty run specs.
    #[test]
    fn new_paragraph_style_and_revision_cleanup_truth_table() {
        for existing_style in [false, true] {
            let style = if existing_style {
                "<w:pStyle w:val=\"Old\"/>"
            } else {
                ""
            };
            let (mut d, root) = dom(&format!(
                "<w:p><w:pPr>{style}<w:keepNext/><w:sectPr/><w:pPrChange/><w:rPr/></w:pPr><w:r><w:rPr><w:i/><w:rPrChange/><w:ins/><w:del/></w:rPr><w:t>anchor</w:t></w:r></w:p>"
            ));
            let anchor = d.element(root, &W::p()).unwrap();
            let original = d.serialize_element(anchor);
            let p = build_paragraph(
                &mut d,
                anchor,
                &[
                    RunSpec::default(),
                    RunSpec {
                        text: "new".into(),
                        ..RunSpec::default()
                    },
                ],
                Some("Chosen"),
            );
            assert_eq!(d.serialize_element(anchor), original);
            assert_eq!(project_paragraph(&d, p).text, "new");
            assert_eq!(d.elements(p, Some(&W::r())).len(), 1);
            let ppr = d.element(p, &W::p_pr()).unwrap();
            assert_eq!(names(&d, ppr), ["pStyle", "keepNext"]);
            assert_eq!(
                d.attribute(d.element(ppr, &W::p_style()).unwrap(), &W::val()),
                Some("Chosen")
            );
            let r = d.element(p, &W::r()).unwrap();
            assert_eq!(names(&d, d.element(r, &W::r_pr()).unwrap()), ["i"]);
            assert_eq!(
                d.attribute(
                    d.element(r, &W::t()).unwrap(),
                    &XNamespace::xml().name("space")
                ),
                Some("preserve")
            );
        }
    }

    // 5314, 5317, 5324, 5327, 5336, 5346, 5355.
    #[test]
    fn replacing_and_clearing_existing_run_properties() {
        for clear in [false, true] {
            let (mut d, root) = dom(
                r#"<w:rPr><w:rFonts w:ascii="Old"/><w:color w:val="000000"/><w:sz w:val="20"/><w:szCs w:val="21"/><w:highlight w:val="yellow"/><w:u w:val="double"/></w:rPr>"#,
            );
            let rpr = d.element(root, &W::r_pr()).unwrap();
            apply_run_format(
                &mut d,
                rpr,
                &RunFormat {
                    underline: Some(!clear),
                    highlight: Some(if clear { "none" } else { "green" }.into()),
                    font: Some("Bundled Name".into()),
                    color: Some("aBcD09".into()),
                    size_pt: Some(HalfPoints(25)),
                    ..RunFormat::default()
                },
            );
            assert_eq!(
                names(&d, rpr),
                if clear {
                    vec!["rFonts", "color", "sz", "szCs"]
                } else {
                    vec!["rFonts", "color", "sz", "szCs", "highlight", "u"]
                }
            );
            let fonts = d.element(rpr, &W::name("rFonts")).unwrap();
            for script in ["ascii", "hAnsi", "eastAsia", "cs"] {
                assert_eq!(d.attribute(fonts, &W::name(script)), Some("Bundled Name"));
            }
            for size in ["sz", "szCs"] {
                assert_eq!(
                    d.attribute(d.element(rpr, &W::name(size)).unwrap(), &W::val()),
                    Some("25")
                );
            }
            assert_eq!(
                d.attribute(d.element(rpr, &W::name("color")).unwrap(), &W::val()),
                Some("aBcD09")
            );
            if !clear {
                assert_eq!(
                    d.attribute(d.element(rpr, &W::name("u")).unwrap(), &W::val()),
                    Some("single")
                );
                assert_eq!(
                    d.attribute(d.element(rpr, &W::name("highlight")).unwrap(), &W::val()),
                    Some("green")
                );
            }
        }
    }

    // 5398, 5399, 5406, 5407, 5421. Font names are validated as strings only.
    #[test]
    fn format_validation_unicode_length_and_color_truth_tables() {
        for (font, valid) in [
            ("é".repeat(31), true),
            ("é".repeat(32), false),
            ("ab\u{0001}cd".into(), false),
        ] {
            let f = RunFormat {
                font: Some(font.clone()),
                ..RunFormat::default()
            };
            assert_eq!(
                check_format(&f, "x"),
                if valid {
                    Ok(())
                } else {
                    Err(format!(
                        "font {font:?} must be a nonempty name of at most 31 characters"
                    ))
                }
            );
        }
        for (color, valid) in [
            ("auto", true),
            ("0aBf19", true),
            ("ABCDEF", true),
            ("12345", false),
            ("12345g", false),
        ] {
            let f = RunFormat {
                color: Some(color.into()),
                ..RunFormat::default()
            };
            assert_eq!(
                check_format(&f, "x"),
                if valid {
                    Ok(())
                } else {
                    Err(format!(
                        "color {color:?} must be six hex digits (FF0000) or auto"
                    ))
                }
            );
        }
        assert_eq!(check_format(&RunFormat::default(), ""), Ok(()));
        assert_eq!(
            check_format(
                &RunFormat {
                    bold: Some(false),
                    ..RunFormat::default()
                },
                ""
            ),
            Err("format needs nonempty text".into())
        );
    }

    // 4009: whole deletion skips replacement bookmarks.
    #[test]
    fn whole_deletion_builds_no_bookmark_and_keeps_surrounding_text() {
        let source = docx(&para("left middle right"));
        let p = plan(
            r#"[{"kind":"replace","paragraph":"body:p:0","find":"middle","replacement":"","whole":true}]"#,
        );
        let mut tx = Transaction::start(&source, &p).unwrap();
        tx.resolve().unwrap();
        tx.apply().unwrap();
        assert_eq!(tx.whole_marks.len(), 0);
        assert_eq!(
            project_paragraph(&tx.opened.dom, tx.paragraph_nodes[0]).text,
            "left  right"
        );
    }

    // 1402, 1414, 4526: tolerate an unreadable story and ignore foreign revisions.
    #[test]
    fn revision_and_comment_scans_ignore_rootless_and_foreign_story_parts() {
        let header = format!(
            r#"<w:hdr xmlns:w="{}" xmlns:x="urn:foreign"><w:p><x:ins/><w:r><w:t>header</w:t></w:r></w:p></w:hdr>"#,
            W::URI
        );
        let source = docx_with(
            &para("body"),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
        );
        assert_eq!(
            revision_counts(&source, &WmlComparerSettings::default()),
            RevisionCounts::default()
        );
        let broken =
            fixture::replace_entry(&source, "word/header1.xml", b"<?xml version=\"1.0\"?>");
        assert_eq!(
            revision_counts(&broken, &WmlComparerSettings::default()),
            RevisionCounts::default()
        );
        assert!(!comment_holds_kept_text(&broken, 7));
        assert_eq!(
            fixture::part_string(&broken, "word/header1.xml").as_deref(),
            Some("<?xml version=\"1.0\"?>")
        );
    }
}
