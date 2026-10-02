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
use crate::inspect::{Opened, Piece, Projection, SCHEMA_VERSION, project_paragraph, source_sha256};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XNamespace};

mod controls;
mod rewrite;
mod structural;
mod tracked;
mod watermark;
mod whole;

pub use controls::ControlSelector;

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
    /// Revision and comment timestamp (`YYYY-MM-DDTHH:MM:SSZ`); fixed default
    /// when omitted so output is reproducible.
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
    },
    /// Delete exactly one occurrence of `find`.
    Delete {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// Exact text to delete; must occur exactly once.
        find: String,
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

/// Paragraph selector; every form must match exactly one paragraph. Ids
/// name their story (`body:p:3`, `header1:p:0`); the other forms search the
/// body unless they carry a `story` (`header1`, `footnotes`, ...).
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
            "replace" => &["find", "replacement", "format", "comment", "whole"],
            "insert" => &["after", "before", "position", "text", "format", "comment"],
            "delete" => &["find"],
            "comment" => &["find", "text", "through"],
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
    let mut tx = Transaction::start(source, plan)?;
    tx.resolve()?;
    tracked::check(&tx)?;
    tx.apply()?;
    let (clean, marked) = tx.finish()?;
    if plan.existing_revisions == ExistingRevisions::Keep {
        return tracked::result(&tx, clean);
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
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline, &settings);
    Ok(EditResult {
        clean,
        redline,
        report,
    })
}

/// Resolve every operation without producing documents.
pub fn preview_plan(source: &[u8], plan: &EditPlan) -> Result<EditReport, EditError> {
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

/// What resolving an operation gives: its resolved form and outcome, or the
/// error with the outcome so far.
type Resolution<T> = Result<(T, EditOutcome), Box<(EditError, EditOutcome)>>;

struct Transaction<'p> {
    plan: &'p EditPlan,
    source_sha256: String,
    base: Vec<u8>,
    base_sha256: String,
    resolved_revisions: ResolvedRevisions,
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
            id: "body".to_string(),
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
            .unwrap_or_else(|| WmlComparerSettings::default().date_time_for_revisions);
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
            | OperationKind::InsertTable { paragraph, .. } => paragraph,
            OperationKind::List { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "list resolves through resolve_list".into(),
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
        };
        let para = match self.select(selector) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
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
                    .find_range(projection, find, &mut outcome)
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
            OperationKind::Delete { find, .. } => {
                let (start, end) = self
                    .find_range(projection, find, &mut outcome)
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
                            .find_range(projection, after, &mut outcome)
                            .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                        (end, true)
                    }
                    (None, Some(before), None) => {
                        let (start, _) = self
                            .find_range(projection, before, &mut outcome)
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
                find, text: note, ..
            } => {
                check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                let (start, end) = match find {
                    Some(find) => self
                        .find_range(projection, find, &mut outcome)
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
            OperationKind::Rewrite { .. }
            | OperationKind::List { .. }
            | OperationKind::ReplyComment { .. }
            | OperationKind::ResolveComment { .. }
            | OperationKind::EditComment { .. }
            | OperationKind::DeleteComment { .. } => Err(fail(
                "INVALID_PLAN",
                "rewrite, list and thread operations resolve on their own paths".into(),
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

    fn select(&self, selector: &Selector) -> Result<usize, (String, String, usize)> {
        let not_found = |message: String| Err(("ANCHOR_NOT_FOUND".to_string(), message, 0));
        let (story, index) = match selector {
            Selector::Name(id) | Selector::Id { id } => match id
                .rsplit_once(":p:")
                .and_then(|(story, n)| Some((story, n.parse::<usize>().ok()?)))
            {
                Some((story, n)) => (story, Some(n)),
                None => return not_found(format!("unknown paragraph id {id}")),
            },
            Selector::Index { index, story } => (story.as_deref().unwrap_or("body"), Some(*index)),
            Selector::StartsWith { story, .. } | Selector::Contains { story, .. } => {
                (story.as_deref().unwrap_or("body"), None)
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
                Resolved::Text { para, .. }
                | Resolved::CommentRange { para, .. }
                | Resolved::CommentSpan { para, .. }
                | Resolved::DeleteParagraph { para }
                | Resolved::FormatParagraph { para, .. }
                | Resolved::MergeParagraphs { para, .. } => vec![self.paragraph_story[*para].0],
                Resolved::InsertParagraph { anchor, .. } | Resolved::InsertTable { anchor, .. } => {
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

    /// The unique occurrence of `find` (overlapping occurrences count), checked
    /// to lie within editable direct text.
    fn find_range(
        &self,
        projection: &Projection,
        find: &str,
        outcome: &mut EditOutcome,
    ) -> Result<(usize, usize), (String, String)> {
        if find.is_empty() {
            return Err(("INVALID_EDIT".into(), "find must be nonempty".into()));
        }
        let text = &projection.text;
        let hits: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .filter(|&i| text[i..].starts_with(find))
            .collect();
        outcome.matches = hits.len();
        let start = match hits.as_slice() {
            [one] => *one,
            [] => {
                return Err((
                    "ANCHOR_NOT_FOUND".into(),
                    format!("{find:?} does not occur in the paragraph"),
                ));
            }
            many => {
                return Err((
                    "AMBIGUOUS_ANCHOR".into(),
                    format!("{find:?} occurs {} times in the paragraph", many.len()),
                ));
            }
        };
        let end = start + find.len();
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
                Resolved::InsertParagraph { anchor, .. } => {
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
            for pair in list.windows(2) {
                let (s0, e0, _) = pair[0];
                let (s1, _, i1) = pair[1];
                let strictly_inside = s1 > s0 && s1 < e0;
                let same_start_nonempty = s1 == s0 && e0 > s0 && pair[1].1 > s1;
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
                | Resolved::FillControl { .. } => false,
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
                _ => {}
            }
        }
        let touched: Vec<usize> = by_para
            .keys()
            .chain(comment_ranges.keys())
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
                    Resolved::InsertParagraph { .. } | Resolved::InsertTable { .. }
                )
            })
            .cloned()
            .collect();
        let mut tables: Vec<NodeId> = Vec::new();
        // Several paragraphs after one anchor follow it in plan order: each
        // goes after the one inserted there before it.
        let mut last_after: BTreeMap<usize, NodeId> = BTreeMap::new();
        for (i, r) in inserts {
            let (anchor, side, new, commented) = match r {
                Resolved::InsertParagraph {
                    anchor,
                    side,
                    runs,
                    like,
                    style,
                    comment,
                } => {
                    let like_node = self.paragraph_nodes[like];
                    let new =
                        build_paragraph(&mut self.opened.dom, like_node, &runs, style.as_deref());
                    (anchor, side, new, comment.is_some())
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
                    (anchor, side, new, false)
                }
                _ => continue,
            };
            let anchor_node = self.paragraph_nodes[anchor];
            match side {
                Side::After => {
                    let prev = last_after.get(&anchor).copied().unwrap_or(anchor_node);
                    self.opened.dom.add_after_self(prev, new);
                    last_after.insert(anchor, new);
                }
                Side::Before => self.opened.dom.add_before_self(anchor_node, new),
            }
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
        if self.deletion_comments.is_empty() && thread_ops.is_empty() && self.watermark.is_none() {
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
        let plan = EditPlan {
            schema_version: SCHEMA_VERSION,
            source_sha256: None,
            author: self.plan.author.clone(),
            date: Some(self.date.clone()),
            initials: Some(self.initials.clone()),
            resolve_revisions: None,
            existing_revisions: ExistingRevisions::default(),
            operations,
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
