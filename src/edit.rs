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
//! `jubarte read --json` prints is what a plan anchors to. Replacement text is
//! inserted into the run that held the anchor, so it inherits that run's
//! formatting; the plan never carries raw OOXML.
//!
//! The CLI subcommand is `jubarte edit FILE [--plan PLAN.json]`; the bindings
//! drive [`apply_plan`] / [`apply_plan_json`] directly. [`preview_plan`]
//! resolves every operation and returns a report without producing the
//! documents, for dry-run checks. Recording an edit from operation flags is
//! in [`flags`] / [`record`].

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::changes::{Change, ChangeError, ChangeFilter, ChangeKind};

use crate::inspect::{
    BODY_STORY, Opened, Piece, Projection, SCHEMA_VERSION, project_paragraph, source_sha256,
};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XNamespace};

mod anchor;
mod anchor_error;
mod controls;
pub mod flags;
mod hint;
mod images;
mod notes;
mod origin;
/// One preparation path for dry run and apply: the chosen primary bytes
/// and the checks the serialized candidate passes (V2-08).
pub mod prepared;
mod ranges;
pub mod record;
/// Caller-specific, neutral review choices for revision-blocked edits.
pub mod refusal;
mod replacement_order;
pub mod resolution;
mod rewrite;
mod runs;
mod sections;
mod skip;
mod structural;
mod tracked;
mod visible;
mod watermark;
mod whole;

pub(crate) mod comment_settlement;
#[cfg(feature = "cli")]
/// Compile typed inline CLI actions into the same source-bound PlanV2 contract.
pub mod inline;
pub mod v2;
use comment_settlement::AnchorChanges;
pub use comment_settlement::{KeptComment, RemovedComment};
pub mod v2_apply;

/// `read --for-edit`: the working document, hashes and readiness (V2-10).
pub mod for_edit;
/// The one native-key policy (`w14:paraId`, revision `w:id`) shared by
/// generation, application and the expectations checker for paired
/// long IDs.
pub(crate) mod native_keys;
mod source_refs;
/// Source-bound desired-text template generation over the for-edit view.
pub mod template;

/// Exact v2 literal-anchor candidates and selection; v1 retains its fallbacks.
pub use anchor::{NearMatches, NearOccurrence, v2_candidates, v2_select, v2_select_all};
/// v2 anchor selection with structured, bounded, actionable refusals.
pub use anchor_error::{AnchorRefusal, Working, v2_resolve, v2_resolve_all};
/// Which content control a `fill_control` operation fills.
pub use controls::ControlSelector;
/// The request syntax a refusal hint quotes (#1108).
pub use hint::PlanSyntax;
/// The shared preparation path [`apply_plan`] and [`preview_plan`] run.
pub use prepared::{
    CandidateChecks, PrepareOptions, PreparedEdit, PrimaryBytes, PrimaryRole, introduced_findings,
    prepare, prepare_v2,
};
/// The `page_setup` operation's `page`, `margins_dxa`, `orientation` and
/// `section` values.
pub use sections::{CustomPage, Margins, Orientation, PageSize, Paper, SectionScope};
/// `--skip-failed`: what was left out, and the plan to fix and run.
pub use skip::{SkippedOperation, failed_plan_json, keep_notes};

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
    /// Remove threads whose anchors disappear (default), or keep them on
    /// empty, untracked anchors at the removed text's position.
    #[serde(default, skip_serializing_if = "RemovedAnchorComments::is_remove")]
    pub removed_anchor_comments: RemovedAnchorComments,
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
    /// Let `edit_comment` change a comment another author wrote; the
    /// comment keeps its author, and the report notes each one. Without
    /// it such an edit is refused (`COMMENT_NOT_YOURS`): a reply is how
    /// one author answers another.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rewrite_others_comments: bool,
}

/// Treatment of comment anchors removed by a plan's revision settlement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemovedAnchorComments {
    /// Follow Word: remove the thread and report it.
    #[default]
    Remove,
    /// Keep the thread on an empty, untracked anchor.
    Keep,
}
impl RemovedAnchorComments {
    pub(crate) fn is_remove(&self) -> bool {
        *self == Self::Remove
    }
}

/// The tracked changes a plan accepts and rejects before it edits, as
/// Word's Accept / Reject This Change (see [`crate::changes`]). Each side
/// is a [`ChangeFilter`]; left out, it resolves nothing. No change may be
/// selected by both (`REVISION_CONFLICT`); both sides of a move count as
/// one change.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "ResolveRevisionsWire", into = "ResolveRevisionsWire")]
pub struct ResolveRevisions {
    /// Changes to accept.
    pub accept: Option<ChangeFilter>,
    /// Changes to reject (after the accepted ones).
    pub reject: Option<ChangeFilter>,
    /// `reject.visible`: reject them visibly instead, as our tracked
    /// changes (a `strike` of each insertion, a `restore` of each
    /// deletion) that Reject takes back. Needs `existing_revisions` `keep`.
    pub reject_visible: bool,
}

/// [`ResolveRevisions`] as plans write it: `visible` inside `reject`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolveRevisionsWire {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accept: Option<ChangeFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reject: Option<RejectWire>,
}

/// A [`ChangeFilter`] with `visible`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RejectWire {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kinds: Option<Vec<ChangeKind>>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    visible: bool,
}

impl From<ResolveRevisionsWire> for ResolveRevisions {
    fn from(wire: ResolveRevisionsWire) -> Self {
        let reject_visible = wire.reject.as_ref().is_some_and(|r| r.visible);
        Self {
            accept: wire.accept,
            reject: wire.reject.map(|r| ChangeFilter {
                ids: r.ids,
                authors: r.authors,
                kinds: r.kinds,
            }),
            reject_visible,
        }
    }
}

impl From<ResolveRevisions> for ResolveRevisionsWire {
    fn from(selection: ResolveRevisions) -> Self {
        Self {
            accept: selection.accept,
            reject: selection.reject.map(|r| RejectWire {
                ids: r.ids,
                authors: r.authors,
                kinds: r.kinds,
                visible: selection.reject_visible,
            }),
        }
    }
}

/// Where a source paragraph id went when the plan's revisions were settled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParagraphMove {
    /// The id in the source (`p12`).
    pub from: String,
    /// The id in the output, or `null` when the paragraph is gone with its
    /// text. A paragraph merged with others maps to the merged one.
    pub to: Option<String>,
}

/// Another author's insertion the plan split to edit inside it (#1136):
/// its source id and the ids its pieces carry in the redline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionSplit {
    /// The insertion's id in the source (`r3`).
    pub from: String,
    /// Its pieces' ids in the redline, in id order (`r3`, `r9`).
    pub to: Vec<String>,
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
    /// Whether no change was accepted or rejected.
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Operation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Caller-chosen operation id echoed in the report.
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Operations sharing a group stand or fall together under
    /// `--skip-failed` ([`PrepareOptions::skip_failed`]); without it a
    /// group changes nothing.
    pub group: Option<String>,
    #[serde(flatten)]
    /// The operation.
    pub kind: OperationKind,
}

/// The supported operation kinds (wire tag `kind`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationKind {
    /// Replace exactly one occurrence of `find` in the paragraph.
    Replace {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// Exact text to locate; must occur exactly once.
        find: String,
        /// Replacement text (may be empty); `\t` writes a tab, `\n` a line break.
        replacement: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Formatting of the replacement on top of the replaced run's.
        format: Option<RunFormat>,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment text anchored to the changed text; blank is none.
        comment: Option<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        /// Show the change as all of `find` deleted, then all of
        /// `replacement` inserted, instead of Word Compare's word-level diff.
        whole: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert `text` after/before exactly one occurrence of an anchor, or at
    /// the paragraph's start/end. Exactly one of `after`, `before`,
    /// `position` must be given.
    #[schemars(extend("oneOf" = exactly_one(&["after", "before", "position"])))]
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
        /// Text to insert; `\t` writes a tab, `\n` a line break.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Formatting of the new text on top of the neighbouring run's.
        format: Option<RunFormat>,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment text anchored to the changed text; blank is none.
        comment: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
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
        #[schemars(range(min = 1))]
        /// Which hit of a repeated anchor to use (1-based); required when
        /// the anchor occurs more than once.
        occurrence: Option<usize>,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment on the deleted text, blank is none. It is in the redline
        /// only: the clean copy no longer holds the text.
        comment: Option<String>,
    },
    /// Comment on exactly one occurrence of `find`, or on the whole paragraph.
    /// The range may cross tabs, symbols, hyperlinks, fields, content
    /// controls and revisions: a comment edits nothing.
    #[schemars(extend("x-jubarte-targets" = ["body", "table_cell"]))]
    Comment {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Text to anchor the comment to; whole paragraph when omitted.
        find: Option<String>,
        /// The comment, plain text; each `\n` starts a new comment paragraph.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Last paragraph of a range from the start of `paragraph` to the
        /// end of this one (same story, not before `paragraph`); `find`
        /// must be left out.
        through: Option<Selector>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// Paragraph style id to set instead of the anchor's.
        style: Option<String>,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment text anchored to the changed text; blank is none.
        comment: Option<String>,
    },
    /// Delete a whole paragraph, mark included.
    DeleteParagraph {
        /// Paragraph to delete; must match exactly one.
        paragraph: Selector,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment on the deleted text, blank is none. It is in the redline
        /// only: the clean copy has no paragraph to hold it.
        comment: Option<String>,
    },
    /// Change a paragraph's style, alignment or spacing; the redline records
    /// the old properties (`w:pPrChange`). At least one field besides
    /// `paragraph`.
    #[schemars(extend("anyOf" = at_least_one(&["style", "alignment", "line_spacing", "space_before", "space_after"])))]
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
    /// Make the paragraph read as `text`: the engine applies the smallest
    /// word-level edits, so unchanged words keep their runs and formatting,
    /// and new words take the formatting of the run before them. A tab or a
    /// line break the paragraph has may be written as a space and stays; a
    /// `\t` or `\n` in `text` writes one. Symbols stay where they are and
    /// may be left out of `text`.
    Rewrite {
        /// Paragraph to rewrite; must match exactly one.
        paragraph: Selector,
        /// The paragraph's new text, plain.
        text: String,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment on the rewritten paragraph, blank is none.
        comment: Option<String>,
    },
    /// Reply to comment `comment_id`, as the plan's author; the reply is
    /// anchored on the same range. Word threads are one level deep, so a
    /// reply to a reply joins the thread's first comment.
    ReplyComment {
        /// `w:id` of the comment replied to (`jubarte comments` lists them):
        /// a number, or `"c27"`.
        #[serde(deserialize_with = "crate::ids::comment_id_de")]
        #[schemars(with = "crate::ids::RawCommentId")]
        comment_id: u32,
        /// Reply text; `\n` starts a new line.
        text: String,
    },
    /// Resolve comment `comment_id` and its replies (`done: false` opens
    /// them again).
    ResolveComment {
        /// `w:id` of the comment: a number, or `"c27"`.
        #[serde(deserialize_with = "crate::ids::comment_id_de")]
        #[schemars(with = "crate::ids::RawCommentId")]
        comment_id: u32,
        #[serde(default = "resolve_done")]
        /// Resolved (default) or open.
        done: bool,
    },
    /// Replace the text of comment `comment_id`; its id, author, date and
    /// thread stay.
    EditComment {
        /// `w:id` of the comment: a number, or `"c27"`.
        #[serde(deserialize_with = "crate::ids::comment_id_de")]
        #[schemars(with = "crate::ids::RawCommentId")]
        comment_id: u32,
        /// New text; `\n` starts a new line.
        text: String,
    },
    /// Remove comment `comment_id`, its replies, their range markers and
    /// references.
    DeleteComment {
        /// `w:id` of the comment: a number, or `"c27"`.
        #[serde(deserialize_with = "crate::ids::comment_id_de")]
        #[schemars(with = "crate::ids::RawCommentId")]
        comment_id: u32,
    },
    /// Reject another author's insertion visibly: our tracked deletion of
    /// its words, nested inside it, so Word shows it struck under our name
    /// and Reject of our change gives it back. Needs `existing_revisions`
    /// `keep`.
    #[schemars(extend("x-jubarte-keep-only" = true))]
    Strike {
        /// The insertion, as `changes` lists it (`r366`, `header1.r3`).
        revision: String,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment on the struck words; blank is none.
        comment: Option<String>,
    },
    /// Reject another author's deletion visibly: their deletion stays, and
    /// our tracked insertion right after it holds a copy of its words,
    /// formatting included. Needs `existing_revisions` `keep`.
    #[schemars(extend("x-jubarte-keep-only" = true))]
    Restore {
        /// The deletion, as `changes` lists it (`r12`, `header1.r3`).
        revision: String,
        #[serde(
            default,
            deserialize_with = "blank_is_none",
            skip_serializing_if = "Option::is_none"
        )]
        /// Comment on the restored words; blank is none.
        comment: Option<String>,
    },
    /// Insert a table next to the anchor paragraph. Each cell holds one
    /// paragraph in the anchor's paragraph style; the redline shows the
    /// rows inserted. Body paragraphs outside tables only.
    #[schemars(extend("x-jubarte-targets" = ["body"]))]
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
        #[schemars(inner(range(min = 1, max = structural::MAX_WIDTH_DXA)))]
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
    #[schemars(extend("x-jubarte-targets" = ["body", "table_cell"]))]
    List {
        /// Paragraphs to number, each selector matching exactly one; body
        /// only.
        paragraphs: Vec<Selector>,
        #[serde(default)]
        /// Bullets, decimal numbers or lowercase letters.
        kind_of_list: ListKind,
        #[serde(default)]
        #[schemars(range(min = 0, max = 8))]
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
        #[schemars(length(min = 1, max = watermark::MAX_TEXT))]
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
    #[schemars(extend("x-jubarte-keep" = false), extend("oneOf" = exactly_one(&["text", "choice", "checked", "date"])))]
    FillControl {
        /// The control: `"p3.cc0"` / `"cc1"`, `{"id"}`, `{"tag"}` or `{"alias"}`;
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
        #[schemars(range(min = 1))]
        /// Which occurrence of `find` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert a footnote whose reference mark follows one occurrence of
    /// `after`; the note goes in the footnotes part, created when absent.
    #[schemars(extend("x-jubarte-targets" = ["body", "table_cell"]), extend("x-jubarte-keep" = false))]
    InsertFootnote {
        /// Body paragraph to edit; must match exactly one.
        paragraph: Selector,
        /// The reference mark goes right after this anchor text.
        after: String,
        /// Plain text of the note.
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
        /// Which occurrence of `after` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Insert a paragraph holding one inline picture next to the anchor
    /// paragraph. PNG, JPEG, GIF, BMP or TIFF.
    #[schemars(extend("x-jubarte-targets" = ["body", "table_cell"]), extend("x-jubarte-keep" = false))]
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
        #[schemars(range(min = 1, max = images::MAX_WIDTH_EMU))]
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
    #[schemars(extend("x-jubarte-keep" = false), extend("anyOf" = at_least_one(&["page", "orientation", "margins_dxa"])))]
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
    #[schemars(extend("x-jubarte-targets" = ["body", "table_cell"]))]
    InsertToc {
        /// Paragraph to insert next to; must match exactly one, in the body.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        #[serde(default = "default_toc_levels")]
        #[schemars(range(min = 1, max = 9))]
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
        #[schemars(range(min = 1))]
        /// Which occurrence of `find` (1-based) when it occurs more than once.
        occurrence: Option<usize>,
    },
    /// Write document settings into `word/settings.xml` in schema order:
    /// Track Changes, update fields on open, editing restrictions. Settings
    /// are not revisions: the clean copy and the redline both carry them.
    /// One per plan; a setting left out stays as it is.
    #[schemars(extend("anyOf" = at_least_one(&["track_revisions", "update_fields", "protection"])))]
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
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct LineSpacing(#[schemars(with = "f64", range(min = 0.25, max = 10.0))] pub u32);

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct Points(#[schemars(with = "f64", range(min = 0.0, max = 1584.0))] pub u32);

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct HalfPoints(#[schemars(with = "f64", range(min = 0.0, max = 1638.0))] pub u32);

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    /// Paragraph start.
    Start,
    /// Paragraph end.
    End,
}

/// Side of the anchor paragraph for [`OperationKind::InsertParagraph`].
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// Before the anchor paragraph.
    Before,
    #[default]
    /// After the anchor paragraph.
    After,
}

/// Paragraph selector; every form must match exactly one paragraph. Ids
/// are the agent view's (`p3`, `header1`, `header1.p1`, `footnotes.p2`,
/// `t0.row1.cell2`, `t0.row1.cell2.p1`); the old `body:p:3` is refused,
/// naming its settled spelling, and the report prints the same ids
/// ([`crate::ids::paragraph_id`]). The other forms search
/// the body unless they carry a `story` (`header1`, `footnotes`, ...).
///
/// Ids, short ids and indexes name the source's paragraphs, the ones the
/// plan's author read, even when `resolve_revisions` or `existing_revisions`
/// settles changes first; the text forms search the settled text. An edit
/// anchored on text follows a paragraph into the one it merged with, and so
/// does a whole-paragraph edit when that paragraph has no other source; a
/// whole-paragraph edit of a paragraph merged with others, or of a vanished
/// one, is refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Selector {
    /// `"p12"`, the same as `{"id": "p12"}`.
    Name(String),
    /// `{"id": "p12"}`.
    Id {
        /// `p12`, `header1.p1`, `t0.row1.cell2`.
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
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
    pub context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Comment id written for this operation.
    pub comment_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The anchor as the plan wrote it, when it matched only without its
    /// Markdown marks, or only as it reads: a space for a non-breaking
    /// one, straight quotes for curly ones, `-` for a non-breaking hyphen,
    /// zero-width characters left out.
    pub anchor_given: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The paragraph's text actually matched in that case.
    pub anchor_read_as: Option<String>,
    /// Report-only whitespace occurrences when the literal anchor was refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near: Option<NearMatches>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Error code when failed.
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Error message when failed; on an `ok` operation, a note that it
    /// could not be shown as asked (a `whole` replacement kept word-level).
    pub message: Option<String>,
}

/// The redline's changes by kind, as `jubarte changes` lists them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionCounts {
    /// Insertions.
    pub inserted: usize,
    /// Deletions.
    pub deleted: usize,
    /// Moves.
    pub moved: usize,
    /// Formatting changes: run, paragraph, section, table and style
    /// properties.
    pub format_changed: usize,
    /// Replacements: a deletion and the insertion that replaces it, listed
    /// as one change ([`ChangeKind::Replacement`]).
    #[serde(default)]
    pub replaced: usize,
    /// All changes, each logical change once (#1261: a change of several
    /// marks, or a replacement, counts one): the sum of the counts above.
    pub total: usize,
}

impl RevisionCounts {
    /// The counts of `changes`, one per listed change.
    pub(crate) fn from_changes<'a>(changes: impl IntoIterator<Item = &'a Change>) -> Self {
        let mut counts = Self::default();
        for change in changes {
            let slot = match change.kind {
                ChangeKind::Insertion => &mut counts.inserted,
                ChangeKind::Deletion => &mut counts.deleted,
                ChangeKind::Move => &mut counts.moved,
                ChangeKind::Formatting => &mut counts.format_changed,
                ChangeKind::Replacement => &mut counts.replaced,
            };
            *slot += 1;
            counts.total += 1;
        }
        counts
    }
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
    /// Threads removed with the settled text, in source comment ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_comments: Vec<RemovedComment>,
    /// Threads moved onto empty anchors by opt-in settlement, excluding
    /// comments and replies subsequently deleted by the plan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kept_comments: Vec<KeptComment>,
    /// Source paragraph ids that name another paragraph once the plan's
    /// revisions are settled (`resolve_revisions`, `existing_revisions`
    /// accept or reject), in source order. Operations still address the
    /// source's ids; the output documents carry these.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paragraph_ids: Vec<ParagraphMove>,
    /// Other authors' insertions the redline splits, by source id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub split_revisions: Vec<RevisionSplit>,
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
    /// Operations `--skip-failed` left out (status `skipped`); `ok` is
    /// false whenever one was.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub skipped: usize,
    /// What the plan said that jubarte read another way (`anchor` read as
    /// `find`, …): the CLI prints each as a `note:` line.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// SHA-256 of the document written, set by whoever writes it (the
    /// CLI): the bytes a follow-up plan binds to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_sha256: Option<String>,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
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
        if !self.paragraph_ids.is_empty() {
            load.as_object_mut().expect("object").insert(
                "paragraph_ids".into(),
                serde_json::to_value(&self.paragraph_ids).expect("serializes"),
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
            if let Some(near) = &op.near {
                map.insert(
                    "near".into(),
                    serde_json::to_value(near).expect("near serializes"),
                );
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
        // Under --skip-failed the operations that passed landed: partial.
        let status = match (self.ok, self.skipped) {
            (true, _) => "ok",
            (false, 0) => "failed",
            (false, _) if ok > 0 => "partial",
            (false, _) => "failed",
        };
        let mut ops = serde_json::json!({"ok": ok, "failed": failed});
        if self.skipped > 0 {
            ops["skipped"] = self.skipped.into();
        }
        let mut summary = serde_json::json!({
            "ev": "summary",
            "status": status,
            "ops": ops,
            "paras": {"from": self.paragraphs.from, "to": self.paragraphs.to,
                      "d": self.paragraphs.to as i64 - self.paragraphs.from as i64},
            "revs": self.revisions,
            "comments": {"added": self.comments_added},
            "removed_comments": self.removed_comments,
            "kept_comments": self.kept_comments,
        });
        if !self.split_revisions.is_empty() {
            summary.as_object_mut().expect("object").insert(
                "split_revisions".into(),
                serde_json::to_value(&self.split_revisions).expect("serializes"),
            );
        }
        if let Some(digest) = &self.output_sha256 {
            summary["output_sha256"] = digest.clone().into();
        }
        if !self.notes.is_empty() {
            summary["notes"] = self.notes.clone().into();
        }
        push(summary);
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

#[cfg(test)]
thread_local! {
    static COMMENT_MARK_SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Stable engine code for comment mapping conflicts, independent of prose.
const COMMENT_ANCHOR_CONFLICT: &str = "COMMENT_ANCHOR_CONFLICT";

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
    /// Report-only whitespace occurrences; never used to resolve an operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near: Option<Box<NearMatches>>,
    /// Blocking revision identities and both review choices, when applicable.
    #[serde(
        default,
        skip_serializing_if = "<[refusal::RevisionRefusal]>::is_empty"
    )]
    pub revision_refusals: Box<[refusal::RevisionRefusal]>,
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

/// A [`ChangeError`] of a `resolve_revisions` selection as the plan's error.
fn selection_error(e: ChangeError) -> EditError {
    match e {
        ChangeError::UnknownChange(id) => err(
            "UNKNOWN_CHANGE",
            None,
            format!("resolve_revisions: no tracked change {id}"),
        ),
        ChangeError::InvalidId(m) => err("INVALID_PLAN", None, format!("resolve_revisions: {m}")),
        ambiguous @ ChangeError::Ambiguous { .. } => err(
            "AMBIGUOUS_CHANGE",
            None,
            format!("resolve_revisions: {ambiguous}"),
        ),
        part @ ChangeError::PartOfChange { .. } => err(
            "INCOMPLETE_CHANGE",
            None,
            format!("resolve_revisions: {part}"),
        ),
        ChangeError::Package(m) => err("INVALID_DOCUMENT", None, m),
    }
}

/// The ids of `listed` a `resolve_revisions` side selects, in document
/// order, a move's other side included.
fn selected_ids(
    listed: &[Change],
    filter: Option<&ChangeFilter>,
) -> Result<Vec<String>, EditError> {
    let Some(filter) = filter else {
        return Ok(Vec::new());
    };
    let filter = &filter.normalized().map_err(selection_error)?;
    filter.check(listed).map_err(selection_error)?;
    let hit: Vec<&Change> = listed.iter().filter(|c| filter.matches(c)).collect();
    Ok(listed
        .iter()
        .filter(|c| {
            hit.iter()
                .any(|h| h.id == c.id || h.move_name.is_some() && h.move_name == c.move_name)
        })
        .map(|c| c.id.clone())
        .collect())
}

/// The ids `filter` itself selects in `listed`, before [`selected_ids`]
/// adds each move's other side: the ones a receipt names as resolved.
fn named_ids(listed: &[Change], filter: Option<&ChangeFilter>) -> Vec<String> {
    let Some(filter) = filter.and_then(|f| f.normalized().ok()) else {
        return Vec::new();
    };
    listed
        .iter()
        .filter(|c| filter.matches(c))
        .map(|c| c.id.clone())
        .collect()
}

/// Accept, then reject, the changes `resolve_revisions` selects: the new
/// source (`None` when it selects nothing) and what was resolved.
fn resolve_selected(
    source: &[u8],
    selection: Option<&ResolveRevisions>,
    policy: RemovedAnchorComments,
) -> Result<(Option<Vec<u8>>, ResolvedRevisions, AnchorChanges), EditError> {
    let mut comments = AnchorChanges::default();
    let mut resolved = ResolvedRevisions::default();
    let Some(selection) = selection else {
        return Ok((None, resolved, comments));
    };
    let change_err = selection_error;
    // Each side names whole changes, checked on the source: an accept can
    // join what is left (#1261).
    for side in [selection.accept.as_ref(), selection.reject.as_ref()]
        .into_iter()
        .flatten()
    {
        crate::changes::check_whole_changes(source, side).map_err(change_err)?;
    }
    let listed = crate::changes::list_changes(source).map_err(change_err)?;
    let accept = selected_ids(&listed, selection.accept.as_ref())?;
    // A visible reject settles nothing: it is lowered to operations.
    let reject = if selection.reject_visible {
        Vec::new()
    } else {
        selected_ids(&listed, selection.reject.as_ref())?
    };
    if let Some(both) = accept.iter().find(|id| reject.contains(id)) {
        return Err(err(
            "REVISION_CONFLICT",
            None,
            format!("resolve_revisions selects {both} to accept and to reject"),
        ));
    }
    if accept.is_empty() && reject.is_empty() {
        return Ok((None, resolved, comments));
    }
    let mut bytes = source.to_vec();
    if !accept.is_empty() {
        let (document, removed, kept) = crate::changes::resolve_for_edit(
            &bytes,
            &ChangeFilter::ids(accept.clone()),
            crate::revision_processor::Resolution::Accept,
            policy == RemovedAnchorComments::Keep,
        )
        .map_err(change_err)?;
        comments.extend(
            removed,
            kept,
            crate::revision_processor::Resolution::Accept,
            Some(&named_ids(&listed, selection.accept.as_ref())),
        );
        bytes = document;
        resolved.accepted = accept;
    }
    // A change the accepted ones took along is gone, not rejected.
    let left = crate::changes::list_changes(&bytes).map_err(change_err)?;
    let reject: Vec<String> = reject
        .into_iter()
        .filter(|id| left.iter().any(|c| &c.id == id))
        .collect();
    if !reject.is_empty() {
        let (document, removed, kept) = crate::changes::resolve_for_edit(
            &bytes,
            &ChangeFilter::ids(reject.clone()),
            crate::revision_processor::Resolution::Reject,
            policy == RemovedAnchorComments::Keep,
        )
        .map_err(change_err)?;
        comments.extend(
            removed,
            kept,
            crate::revision_processor::Resolution::Reject,
            Some(&named_ids(&listed, selection.reject.as_ref())),
        );
        bytes = document;
        resolved.rejected = reject;
    }
    Ok((Some(bytes), resolved, comments))
}

/// Whether `source` still carries a tracked change of its text (body,
/// headers, footers, notes; a style's recorded formatting is not one) once
/// `plan`'s `resolve_revisions` has accepted and rejected what it selects.
/// A selection that does not resolve is the plan's error, reported when it
/// runs; here it leaves every change in place.
pub(crate) fn revisions_left(source: &[u8], plan: &EditPlan) -> bool {
    let resolved = resolve_selected(
        source,
        plan.resolve_revisions.as_ref(),
        plan.removed_anchor_comments,
    )
    .ok()
    .and_then(|(bytes, _, _)| bytes);
    crate::changes::list_changes(resolved.as_deref().unwrap_or(source))
        .is_ok_and(|changes| changes.iter().any(|c| !c.id.starts_with("styles:")))
}

/// The replies `plan` makes to a comment on a paragraph it deletes, each
/// as the parent's id and the deleted paragraph's id. Tracked, the reply
/// sits on the struck text; untracked, as `--editing-mode` publishes the
/// edit, the paragraph and the thread are gone and the reply lands nowhere.
/// A plan that does not resolve names none here: running it reports why.
pub fn replies_on_deleted_paragraphs(source: &[u8], plan: &EditPlan) -> Vec<(u32, Option<String>)> {
    let Ok(lowered) = visible::lower(source, plan) else {
        return Vec::new();
    };
    let plan = lowered.as_ref().unwrap_or(plan);
    let Ok(tx) = resolved(source, plan, PlanSyntax::V1) else {
        return Vec::new();
    };
    (0..plan.operations.len())
        .filter_map(|op| tx.reply_on_deleted(op))
        .map(|(parent, by)| (parent, tx.outcomes[by].paragraph.clone()))
        .collect()
}

/// The document a plan edits once `resolve_revisions` and
/// `existing_revisions` have settled the tracked changes they select.
struct Settled {
    /// The bytes the plan edits and the redline compares against.
    base: Vec<u8>,
    opened: Opened,
    resolved_revisions: ResolvedRevisions,
    comment_changes: AnchorChanges,
    /// Settling rewrote the document: paragraphs may have merged or gone.
    changed: bool,
}

/// Settle `source` (opened as `probe`) as `plan` asks: accept and reject
/// what `resolve_revisions` selects, then apply `existing_revisions` to the
/// changes left.
fn settle(source: &[u8], plan: &EditPlan, probe: Opened) -> Result<Settled, EditError> {
    let (resolved_source, resolved_revisions, mut comment_changes) = resolve_selected(
        source,
        plan.resolve_revisions.as_ref(),
        plan.removed_anchor_comments,
    )?;
    let (source, probe) = match &resolved_source {
        Some(bytes) => (bytes.as_slice(), Opened::open(bytes).map_err(open_error)?),
        None => (source, probe),
    };
    // The listed changes of the body, headers, footers and notes; a style's
    // recorded formatting does not make the text tracked.
    let has_revisions = crate::changes::changes_in(&probe.pkg)
        .iter()
        .any(|c| !c.id.starts_with("styles:"));
    let (base, opened, flattened) = match (has_revisions, plan.existing_revisions) {
        (false, _) | (true, ExistingRevisions::Keep) => (source.to_vec(), probe, false),
        (true, ExistingRevisions::Refuse) => {
            return Err(err(
                "EXISTING_REVISIONS",
                None,
                "the document already holds tracked changes; set existing_revisions to keep, accept or reject",
            ));
        }
        (true, policy) => {
            let resolution = if policy == ExistingRevisions::Accept {
                crate::revision_processor::Resolution::Accept
            } else {
                crate::revision_processor::Resolution::Reject
            };
            let (flattened, removed, kept) = crate::changes::resolve_for_edit(
                source,
                &ChangeFilter::default(),
                resolution,
                plan.removed_anchor_comments == RemovedAnchorComments::Keep,
            )
            .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
            comment_changes.extend(removed, kept, resolution, None);
            let reopened = Opened::open(&flattened).map_err(open_error)?;
            (flattened, reopened, true)
        }
    };
    Ok(Settled {
        base,
        opened,
        resolved_revisions,
        comment_changes,
        changed: resolved_source.is_some() || flattened,
    })
}

/// A document's stories and its addressable paragraphs.
struct Loaded {
    /// The body first, then every header, footer and notes part.
    stories: Vec<StoryPart>,
    /// The body's paragraphs, then each story's.
    paragraph_nodes: Vec<NodeId>,
    /// `(story, index in that story)` per entry of `paragraph_nodes`.
    paragraph_story: Vec<(usize, usize)>,
}

/// The body and every header, footer and notes part of `opened`, parsed
/// into its DOM, with every addressable paragraph.
fn load_stories(opened: &mut Opened) -> Result<Loaded, EditError> {
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
    Ok(Loaded {
        stories,
        paragraph_nodes,
        paragraph_story,
    })
}

/// The paragraph id ([`crate::ids::paragraph_id`]) each of `ids` names in
/// `docx`, as `edit -p` reads it on a document without tracked changes to
/// settle: `p3` and `header1.p1` as given, `header1.p0` → `header1`,
/// `t0.row1.cell2[.p1]` → that cell's paragraph (`p40`). An id that names
/// no table cell or story of `docx`, or is no paragraph id, gives its `Err`
/// message, as `edit -p` does; the old `body:p:3` gives its settled
/// spelling. A body or story index is not checked against the paragraph
/// count.
pub fn paragraph_ids(docx: &[u8], ids: &[&str]) -> Result<Vec<Result<String, String>>, EditError> {
    let mut opened = Opened::open(docx).map_err(open_error)?;
    let Loaded {
        stories,
        paragraph_nodes,
        paragraph_story,
    } = load_stories(&mut opened)?;
    let resolver = origin::Ids {
        dom: &opened.dom,
        body: opened.body,
        story_ids: stories.iter().map(|s| s.id.as_str()).collect(),
        paragraph_nodes: &paragraph_nodes,
        paragraph_story: &paragraph_story,
    };
    Ok(ids
        .iter()
        .map(|id| match resolver.paragraph_id(id) {
            Some(found) => found,
            None => match id.rsplit_once(":p:") {
                Some((story, n)) if n.parse::<usize>().is_ok() => {
                    if resolver.story_ids.contains(&story) {
                        Ok(id.to_string())
                    } else {
                        Err(format!(
                            "unknown story {story:?}; this document has {}",
                            resolver.story_ids.join(", ")
                        ))
                    }
                }
                Some(_) => Err(format!("unknown paragraph id {id}")),
                None => Err(format!("{id} is not a paragraph id")),
            },
        })
        .collect())
}

fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError {
    EditError {
        code: code.to_string(),
        operation: operation.map(str::to_string),
        message: message.into(),
        outcomes: Vec::new(),
        near: None,
        revision_refusals: Box::default(),
    }
}

impl EditPlan {
    /// Parse and validate a plan's JSON. A field read as another (`anchor`
    /// as `find`, …) is read without its note: [`Self::from_json_with_notes`]
    /// returns the notes, and [`apply_plan_json`] puts them in
    /// `EditReport.notes`.
    ///
    /// # Errors
    ///
    /// As [`Self::from_json_with_notes`].
    pub fn from_json(json: &str) -> Result<Self, EditError> {
        Self::from_json_with_notes(json).map(|(plan, _)| plan)
    }

    /// Parse and validate a plan's JSON, with a note for each field read
    /// as another: `anchor` as `find` on `replace`, `delete` and `comment`,
    /// and `comment` as `text` on `comment` and `reply_comment`.
    ///
    /// # Errors
    ///
    /// `INVALID_PLAN` when the JSON is malformed or more than one value,
    /// lacks `schema_version`, or names an unknown field or operation kind:
    /// the message names the operation, its kind, the fields the kind takes
    /// and the nearest one. `UNSUPPORTED_SCHEMA` when `schema_version`
    /// differs from [`SCHEMA_VERSION`].
    pub fn from_json_with_notes(json: &str) -> Result<(Self, Vec<String>), EditError> {
        let json = without_byte_order_mark(json);
        let mut value = plan_value(json)?;
        let Some(version) = value.get("schema_version") else {
            return Err(err(
                "INVALID_PLAN",
                None,
                format!("schema_version is required ({SCHEMA_VERSION} for this build)"),
            ));
        };
        if let Some(version) = version.as_u64().filter(|v| *v != u64::from(SCHEMA_VERSION)) {
            return Err(err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!("schema_version {version} is not supported (expected {SCHEMA_VERSION})"),
            ));
        }
        check_plan_keys(&value)?;
        check_change_filters(&value)?;
        let notes = check_operations(&mut value)?;
        // Each operation is read on its own, so its error names it.
        let operations = match value.get_mut("operations") {
            Some(serde_json::Value::Array(operations)) => std::mem::take(operations),
            _ => Vec::new(),
        };
        let mut plan: Self =
            serde_json::from_value(value).map_err(|e| err("INVALID_PLAN", None, e.to_string()))?;
        plan.operations = operations
            .into_iter()
            .enumerate()
            .map(|(i, op)| {
                let label = operation_label(i, &op);
                let kind = op["kind"].as_str().unwrap_or_default().to_string();
                serde_json::from_value(op)
                    .map_err(|e| err("INVALID_PLAN", Some(label.as_str()), format!("{kind}: {e}")))
            })
            .collect::<Result<_, _>>()?;
        Ok((plan, notes))
    }

    /// Serialize the plan.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("plan serializes")
    }
}

/// The legacy dispatcher contract, derived from its typed Serde operations.
/// Native, Python and WASM all dispatch through `apply_plan_json` / `apply_plan`.
/// The parser's key gate and capability discovery share these exact field facts.
pub(crate) fn operation_schemas() -> &'static [serde_json::Value] {
    static SCHEMAS: std::sync::LazyLock<Vec<serde_json::Value>> = std::sync::LazyLock::new(|| {
        let settings = schemars::generate::SchemaSettings::draft2020_12()
            .with(|s| s.inline_subschemas = true)
            .for_deserialize();
        let common = settings
            .clone()
            .into_generator()
            .into_root_schema_for::<Operation>();
        let common = serde_json::to_value(common).expect("common schema serializes");
        let schema = settings
            .into_generator()
            .into_root_schema_for::<OperationKind>();
        let value = serde_json::to_value(schema).expect("operation schema serializes");
        let mut variants = value["oneOf"]
            .as_array()
            .expect("tagged operation variants")
            .clone();
        for variant in &mut variants {
            variant["properties"]["id"] = common["properties"]["id"].clone();
            variant["properties"]["group"] = common["properties"]["group"].clone();
            // A failed plan's note; read and ignored.
            variant["properties"]["why"] = serde_json::json!({
                "type": "string",
                "description": "Why `--skip-failed` left this operation out; ignored."
            });
            // Closed as `check_operation_keys` is: a key outside these
            // properties (besides the flattened `id`) is an error, not a
            // silently ignored extra.
            variant["additionalProperties"] = serde_json::json!(false);
        }
        variants
    });
    &SCHEMAS
}

/// Names accepted by the common v1 dispatcher, in the legacy manifest order.
pub fn dispatched_operation_kinds() -> Vec<String> {
    operation_schemas()
        .iter()
        .map(|schema| {
            schema["properties"]["kind"]["const"]
                .as_str()
                .expect("operation tag")
                .to_string()
        })
        .collect()
}

/// The operation answers another author's revision beside it, so it runs
/// only when the plan keeps revisions (`strike`, `restore`).
pub(crate) fn operation_needs_keep(kind: &str) -> bool {
    operation_schemas()
        .iter()
        .find(|s| s["properties"]["kind"]["const"] == kind)
        .is_some_and(|s| s["x-jubarte-keep-only"] == true)
}

pub(crate) fn operation_supports_keep(kind: &str) -> bool {
    operation_schemas()
        .iter()
        .find(|s| s["properties"]["kind"]["const"] == kind)
        .is_some_and(|s| s["x-jubarte-keep"] != false)
}

fn at_least_one(fields: &[&str]) -> serde_json::Value {
    serde_json::json!(
        fields
            .iter()
            .map(|field| serde_json::json!({"required": [field]}))
            .collect::<Vec<_>>()
    )
}

fn exactly_one(fields: &[&str]) -> serde_json::Value {
    // Each alternative requires one field and excludes every other field.
    serde_json::json!(
        fields
            .iter()
            .map(|field| serde_json::json!({
                "required": [field],
                "not": {"anyOf": fields.iter().filter(|other| *other != field)
                    .map(|other| serde_json::json!({"required": [other]})).collect::<Vec<_>>()}
            }))
            .collect::<Vec<_>>()
    )
}

/// The top-level fields of a plan, as [`EditPlan`] reads them.
const PLAN_FIELDS: &[&str] = &[
    "author",
    "date",
    "existing_revisions",
    "initials",
    "operations",
    "removed_anchor_comments",
    "resolve_revisions",
    "rewrite_others_comments",
    "schema_version",
    "source_sha256",
    "update_fields",
];

/// A field one kind's operations take under another name, read as meant
/// with a note: (written, read as, kinds).
const ALIASES: &[(&str, &str, &[&str])] = &[
    ("comment", "text", &["comment", "reply_comment"]),
    ("anchor", "find", &["replace", "delete", "comment"]),
];

/// `json` without a leading byte order mark, which RFC 8259 (§8.1) lets a
/// parser ignore: editors that save UTF-8 with one write it first.
pub(crate) fn without_byte_order_mark(json: &str) -> &str {
    json.strip_prefix('\u{feff}').unwrap_or(json)
}

/// The plan's JSON: one value, an object, with nothing after it.
fn plan_value(json: &str) -> Result<serde_json::Value, EditError> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        // One whole value parses: what the parser refused follows it.
        let whole = serde_json::Deserializer::from_str(json)
            .into_iter::<serde::de::IgnoredAny>()
            .next()
            .is_some_and(|first| first.is_ok());
        let message = if whole {
            format!(
                "the plan must be exactly one JSON object; characters follow it at line {} column {}",
                e.line(),
                e.column()
            )
        } else {
            e.to_string()
        };
        err("INVALID_PLAN", None, message)
    })?;
    if !value.is_object() {
        return Err(err(
            "INVALID_PLAN",
            None,
            format!(
                "the plan must be exactly one JSON object, not {}",
                json_kind(&value)
            ),
        ));
    }
    Ok(value)
}

/// What a JSON value is, for a message.
fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Array(_) => "an array",
        serde_json::Value::Object(_) => "an object",
    }
}

/// The name an error gives operation `i`: its `id`, else `op-N` as the
/// report names it.
fn operation_label(i: usize, op: &serde_json::Value) -> String {
    op.get("id")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| format!("op-{}", i + 1), str::to_string)
}

/// The plan's own fields: an unknown one names the fields and the nearest.
fn check_plan_keys(plan: &serde_json::Value) -> Result<(), EditError> {
    let map = plan.as_object().expect("plan_value checked an object");
    match map.keys().find(|key| !PLAN_FIELDS.contains(&key.as_str())) {
        Some(key) => Err(err(
            "INVALID_PLAN",
            None,
            unknown_field(key, PLAN_FIELDS, None),
        )),
        None => Ok(()),
    }
}

/// `unknown field "key"; did you mean "nearest"? fields: …`.
fn unknown_field(key: &str, fields: &[&str], meant: Option<&str>) -> String {
    let hint = nearest(key, fields, meant)
        .map(|name| format!("did you mean {name:?}? "))
        .unwrap_or_default();
    format!("unknown field {key:?}; {hint}fields: {}", fields.join(", "))
}

/// `resolve_revisions.accept.ids` (and `authors`, `kinds`) are arrays; a
/// single string is the slip worth naming.
fn check_change_filters(plan: &serde_json::Value) -> Result<(), EditError> {
    for side in ["accept", "reject"] {
        for key in ["ids", "authors", "kinds"] {
            if let Some(one) = plan
                .pointer(&format!("/resolve_revisions/{side}/{key}"))
                .and_then(serde_json::Value::as_str)
            {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!(
                        "resolve_revisions.{side}.{key} is an array: write {}",
                        serde_json::json!([one])
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Each operation's fields against its kind's schema (the table
/// `capabilities` lists): an error names the operation, its kind, the
/// fields the kind takes and the nearest one. Serde's `flatten` cannot
/// deny unknown fields itself, and a misspelled key must not silently
/// change meaning. The [`ALIASES`] are read as meant, each with a note,
/// and a failed plan's `why` is dropped: a note for whoever fixes it.
fn check_operations(plan: &mut serde_json::Value) -> Result<Vec<String>, EditError> {
    const COMMON: &[&str] = &["id", "kind", "group", "why"];
    let mut notes = Vec::new();
    let Some(ops) = plan
        .get_mut("operations")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(notes);
    };
    let kinds = dispatched_operation_kinds();
    let kind_names: Vec<&str> = kinds.iter().map(String::as_str).collect();
    for (i, op) in ops.iter_mut().enumerate() {
        let label = operation_label(i, op);
        let invalid = |message: String| err("INVALID_PLAN", Some(label.as_str()), message);
        let what = json_kind(op);
        let Some(map) = op.as_object_mut() else {
            return Err(invalid(format!(
                "an operation is a JSON object, not {what}"
            )));
        };
        match map.get("id") {
            Some(serde_json::Value::Number(n)) => {
                return Err(invalid(format!(
                    "id is a string, not a number: write \"{n}\""
                )));
            }
            Some(id) if !id.is_string() => {
                return Err(invalid(format!("id is a string, not {}", json_kind(id))));
            }
            _ => {}
        }
        let kind = match map.get("kind") {
            Some(serde_json::Value::String(kind)) => kind.clone(),
            None => {
                return Err(invalid(format!(
                    "kind is required; kinds: {}",
                    kinds.join(", ")
                )));
            }
            Some(other) => {
                return Err(invalid(format!(
                    "kind is a string, not {}; kinds: {}",
                    json_kind(other),
                    kinds.join(", ")
                )));
            }
        };
        let Some(schema) = operation_schemas()
            .iter()
            .find(|schema| schema["properties"]["kind"]["const"] == kind.as_str())
        else {
            let hint = nearest(&kind, &kind_names, None)
                .map(|name| format!("did you mean {name:?}? "))
                .unwrap_or_default();
            return Err(invalid(format!(
                "unknown kind {kind:?}; {hint}kinds: {}",
                kinds.join(", ")
            )));
        };
        for (written, read, takes) in ALIASES {
            if takes.contains(&kind.as_str())
                && !map.contains_key(*read)
                && let Some(value) = map.remove(*written)
            {
                map.insert((*read).to_string(), value);
                notes.push(format!(
                    "{label} ({kind}): field {written:?} is read as {read:?}"
                ));
            }
        }
        let refusal = match kind.as_str() {
            "list" => Some("takes paragraphs, not paragraph"),
            "watermark" => Some("a watermark takes no paragraph"),
            "settings" => Some("settings take no paragraph"),
            "fill_control" => Some("selects a control, not a paragraph"),
            "page_setup" => Some("sections take no paragraph"),
            _ => None,
        };
        if let Some(message) = refusal.filter(|_| map.contains_key("paragraph")) {
            return Err(invalid(format!("{kind}: {message}")));
        }
        let allowed = schema["properties"]
            .as_object()
            .expect("operation properties");
        let fields: Vec<&str> = allowed
            .keys()
            .map(String::as_str)
            .filter(|field| !COMMON.contains(field))
            .collect();
        if let Some(key) = map
            .keys()
            .find(|key| !COMMON.contains(&key.as_str()) && !allowed.contains_key(*key))
        {
            if let Some((_, read, _)) = ALIASES.iter().find(|(written, read, takes)| {
                written == key && takes.contains(&kind.as_str()) && map.contains_key(*read)
            }) {
                return Err(invalid(format!(
                    "{kind}: {key:?} is read as {read:?}, which the operation gives too: drop one"
                )));
            }
            // A name the operation already gives is no suggestion.
            let absent: Vec<&str> = fields
                .iter()
                .copied()
                .filter(|field| !map.contains_key(*field))
                .collect();
            let hint = nearest(key, &absent, meant_field(&kind, key))
                .map(|name| format!("did you mean {name:?}? "))
                .unwrap_or_default();
            return Err(invalid(format!(
                "{kind}: unknown field {key:?}; {hint}fields: {}",
                fields.join(", ")
            )));
        }
        for key in ["paragraph", "like", "through"] {
            if let Some(n) = map.get(key).filter(|v| v.is_number()) {
                return Err(invalid(format!(
                    "{kind}: {key} is a selector, not a number: write {}",
                    index_selector(n)
                )));
            }
        }
        if let Some(n) = map
            .get("paragraphs")
            .and_then(serde_json::Value::as_array)
            .and_then(|list| list.iter().find(|v| v.is_number()))
        {
            return Err(invalid(format!(
                "{kind}: paragraphs holds selectors, not numbers: write {}",
                index_selector(n)
            )));
        }
        map.remove("why");
    }
    Ok(notes)
}

/// The selector a bare paragraph number meant: `{"index": n}` for a whole
/// number from 0, else what an index is.
fn index_selector(n: &serde_json::Value) -> String {
    n.as_u64().map_or_else(
        || r#"{"index": N}, N a whole number from 0"#.to_string(),
        |n| format!(r#"{{"index": {n}}}"#),
    )
}

/// The field a plan gives another one's name: `text` for a replacement,
/// `comment` for a comment's text, `anchor` for `find`.
/// `text` on `replace` is no alias, only a hint: `text` is what `insert`
/// and `rewrite` take, so the slip is named rather than read.
fn meant_field(kind: &str, key: &str) -> Option<&'static str> {
    match (kind, key) {
        ("replace", "text") => Some("replacement"),
        (_, "comment") => Some("text"),
        (_, "anchor") => Some("find"),
        _ => None,
    }
}

/// The name a slip most likely meant: `meant` when it is one of `names`,
/// else the closest by edit distance within a third of the slip's length.
fn nearest<'a>(key: &str, names: &[&'a str], meant: Option<&str>) -> Option<&'a str> {
    if let Some(name) = meant.and_then(|m| names.iter().copied().find(|name| *name == m)) {
        return Some(name);
    }
    let limit = (key.chars().count() / 3).max(1);
    names
        .iter()
        .map(|name| (edit_distance(key, name), *name))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, name)| name)
}

/// Levenshtein distance between two names, by characters.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != *cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

/// Apply a plan given as JSON: [`EditPlan::from_json_with_notes`], then
/// [`apply_plan`]; the report's `notes` name each field read as another.
///
/// # Errors
///
/// `INVALID_PLAN` and `UNSUPPORTED_SCHEMA` from parsing the plan, and every
/// error [`apply_plan`] reports.
pub fn apply_plan_json(source: &[u8], plan_json: &str) -> Result<EditResult, EditError> {
    apply_plan_json_with(source, plan_json, &PrepareOptions::default())
}

/// [`apply_plan_json`] with the shared preparation's options:
/// `require_change` and `exact_text` reach the JSON callers the CLI
/// already exposes.
///
/// # Errors
///
/// As [`apply_plan_json`], plus `NO_CHANGES_REQUESTED` under
/// [`PrepareOptions::require_change`] and the [`PrepareOptions::exact_text`]
/// form of `POSTCONDITION_FAILED`.
pub fn apply_plan_json_with(
    source: &[u8],
    plan_json: &str,
    options: &PrepareOptions,
) -> Result<EditResult, EditError> {
    let (plan, mut notes) = EditPlan::from_json_with_notes(plan_json)?;
    let mut result = prepared::prepare(source, &plan, options)?.result;
    notes.append(&mut result.report.notes);
    result.report.notes = notes;
    Ok(result)
}

/// Resolve and apply the plan; compare source and copy into a redline.
/// Text replacements use Word's typing order: deletion before insertion.
/// The public document comparer retains Word Compare's insertion-first order.
///
/// Nothing is written: the clean copy, the redline and the report are
/// returned in memory for the caller (the CLI) to write.
///
/// # Errors
///
/// `STALE_SOURCE` when `source` mismatches the plan's `source_sha256`, and
/// `UNSUPPORTED_SCHEMA` when its `schema_version` differs; `INVALID_PLAN`
/// when the plan itself is invalid (an empty author, `update_fields` with
/// kept revisions); `UNSUPPORTED_PACKAGE` for a signed or macro-bearing
/// source. `INVALID_DOCUMENT` when the source cannot be read; the admission
/// code (`INPUT_LIMIT`, …) when it is refused before parsing.
/// `EXISTING_REVISIONS`, `REVISION_CONFLICT`, `UNKNOWN_CHANGE` and
/// `AMBIGUOUS_CHANGE` while settling revisions; `ANCHOR_NOT_FOUND`,
/// `AMBIGUOUS_ANCHOR`, `INVALID_EDIT`, `UNKNOWN_STYLE`, `UNKNOWN_COMMENT`,
/// `COMMENT_NOT_YOURS`, `LOCKED_CONTROL`, `COMMENT_NOT_IN_BODY`,
/// `PARAGRAPH_RESOLVED`, `OVERLAPPING_EDITS`, `UNSUPPORTED_STRUCTURE` and
/// `UNSUPPORTED_IMAGE`
/// while resolving and applying operations; `UNSUPPORTED` for a `settings`
/// operation with a protection password. `FIELDS_FAILED` when field
/// results cannot be refreshed, `COMPARE_FAILED` when the redline cannot be
/// compared, `PACKAGE_WRITE` when it cannot be packaged, and
/// `REDACTION_LEAK` when redacted text still occurs in an output.
pub fn apply_plan(source: &[u8], plan: &EditPlan) -> Result<EditResult, EditError> {
    prepared::prepare(source, plan, &PrepareOptions::default()).map(|p| p.result)
}

/// `update_fields` refreshes the clean copy the comparer reads. Under
/// `existing_revisions: "keep"` the redline replays the edits instead, so a
/// refreshed clean copy would no longer be the accepted redline.
fn check_update_fields(plan: &EditPlan) -> Result<(), EditError> {
    if plan.update_fields && plan.existing_revisions == ExistingRevisions::Keep {
        return Err(err(
            "INVALID_PLAN",
            None,
            "update_fields cannot be combined with existing_revisions \"keep\"; refresh the fields with `jubarte convert FILE --update-fields -o OUT.docx` after accepting or rejecting the kept changes",
        ));
    }
    Ok(())
}

/// Resolve every operation and run the plan's checks, without applying
/// it: what `add --plan` runs on each recorded operation, so recording a
/// long plan never re-applies it.
pub fn check_plan(source: &[u8], plan: &EditPlan) -> Result<(), EditError> {
    check_plan_quoting(source, plan, PlanSyntax::V1)
}

/// [`check_plan`], its refusals quoting their routes in `syntax`: the
/// flags `add --plan` records come back as flags.
pub fn check_plan_quoting(
    source: &[u8],
    plan: &EditPlan,
    syntax: PlanSyntax,
) -> Result<(), EditError> {
    let lowered = visible::lower(source, plan)?;
    resolved(source, lowered.as_ref().unwrap_or(plan), syntax).map(|_| ())
}

fn resolved<'a>(
    source: &'a [u8],
    plan: &'a EditPlan,
    syntax: PlanSyntax,
) -> Result<Transaction<'a>, EditError> {
    check_update_fields(plan)?;
    let mut tx = Transaction::start(source, plan)?;
    tx.syntax = syntax;
    tx.resolve()?;
    tracked::check(&tx)?;
    tx.emits_directly()?;
    Ok(tx)
}

/// Resolve every operation without producing documents.
pub fn preview_plan(source: &[u8], plan: &EditPlan) -> Result<EditReport, EditError> {
    preview_plan_with(source, plan, &PrepareOptions::default())
}

/// [`preview_plan`] with the shared preparation's options: the dry run
/// prepares and checks the same serialized candidate as an apply and stops
/// before publication.
pub fn preview_plan_with(
    source: &[u8],
    plan: &EditPlan,
    options: &PrepareOptions,
) -> Result<EditReport, EditError> {
    // One preparation, resolved once: the dry run's report is the apply's
    // own report — outcomes (comment ids among them), paragraphs, revisions
    // and comments_added all come from the produced candidate. `fields`
    // keeps its v1 dry-run meaning: a preview never publishes the clean
    // copy the refreshes land in, so it reports none.
    let mut report = prepared::prepare(source, plan, options)?.result.report;
    report.fields = Vec::new();
    Ok(report)
}

/// The comparer's redline holds this edit's changes only: all of them.
fn revision_counts(redline: &[u8]) -> RevisionCounts {
    RevisionCounts::from_changes(&crate::changes::list_logical_changes(redline).unwrap_or_default())
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
        /// Replacement text (may be empty); `\t` writes a tab, `\n` a line break.
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
        /// The comment, plain text; each `\n` starts a new comment paragraph.
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
    /// The comment of a `strike` or a `delete`, on the words `[start, end)`
    /// of `para` (source offsets) it deletes: anchored before they are
    /// deleted, in the redline only, as a commented paragraph deletion's.
    StrikeComment {
        para: usize,
        start: usize,
        end: usize,
    },
    /// A `restore`: plain copies of deletion `wid`'s runs right after it
    /// (projection offset `at`), inserted as ours in the redline.
    Restore {
        para: usize,
        /// The deletion's `w:id`.
        wid: String,
        at: usize,
        /// The copies' text, as the projection reads it.
        text: String,
        /// Comment text anchored on the copies.
        comment: Option<String>,
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

/// The text a comment covers: from `start` in paragraph `first` to `end` in
/// paragraph `last` (projection offsets; global paragraph indexes).
#[derive(Clone, Copy, Debug)]
struct Covered {
    first: usize,
    start: usize,
    last: usize,
    end: usize,
}

struct Transaction<'p> {
    plan: &'p EditPlan,
    revision_refusal: std::cell::RefCell<Option<refusal::RevisionRefusal>>,
    source_sha256: String,
    base: Vec<u8>,
    base_sha256: String,
    resolved_revisions: ResolvedRevisions,
    removed_comments: Vec<RemovedComment>,
    kept_comments: Vec<KeptComment>,
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
    /// The source's paragraphs and where they went, when settling
    /// revisions rewrote the document.
    origin: Option<origin::Origin>,
    outcomes: Vec<EditOutcome>,
    resolved: Vec<(usize, Resolved)>,
    /// `(paragraph, start, end)` a `replace` names, by operation, where
    /// its diff wrote only some of it: other operations conflict with the
    /// whole range, its kept words too.
    claims: BTreeMap<usize, (usize, usize, usize)>,
    /// While an operation naming `cN.text` resolves: `N` and the span of
    /// its paragraph's projection the comment covers. Anchors are looked
    /// up there only.
    anchor_window: std::cell::Cell<Option<(u32, usize, usize)>>,
    /// The request syntax refusals quote (#1108).
    syntax: PlanSyntax,
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
    /// Another author's comments `edit_comment` rewrites under
    /// `rewrite_others_comments`: (outcome index, comment id, author).
    rewritten: Vec<(usize, u32, String)>,
    /// The plan's watermark, written into the default headers at finish.
    watermark: Option<watermark::Spec>,
    /// The body's content controls, in document order.
    controls: Vec<NodeId>,
    /// What `inspect` reports for each of `controls`.
    control_records: Vec<crate::inspect::ContentControl>,
    /// The footnotes story when `insert_footnote` added notes to it.
    notes_story: Option<usize>,
    /// The plan's settings, written into the settings part at finish.
    settings: Option<crate::settings::SettingsRequest>,
    /// The story indices `finish` last wrote, body included: the
    /// serialized-stories check compares each one's DOM against the bytes.
    written_stories: std::collections::BTreeSet<usize>,
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
        visible::check_keep(plan)?;
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
        let source_bytes = source;
        let source_comment_max = existing_comment_ids(&probe);
        let Settled {
            mut base,
            mut opened,
            resolved_revisions,
            comment_changes,
            changed,
        } = settle(source, plan, probe)?;
        if let Some(normalized) = ranges::normalize_permission_ids(&base)? {
            base = normalized;
            opened = Opened::open(&base).map_err(open_error)?;
        }
        let (removed_comments, kept_comments) = comment_changes.records(source)?;
        let base_sha256 = source_sha256(&base);
        let Loaded {
            stories,
            paragraph_nodes,
            paragraph_story,
        } = load_stories(&mut opened)?;
        // Settling merged or removed paragraphs: positional ids keep
        // naming the source's paragraphs through a map.
        let origin = if changed {
            let ids: Vec<String> = stories.iter().map(|s| s.id.clone()).collect();
            Some(origin::Origin::build(
                source_bytes,
                plan,
                &ids,
                &paragraph_story,
            )?)
        } else {
            None
        };
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
        // Above the source's ids too: an id the settling removed never
        // names a new comment in the same report (#1264).
        let next_comment_id = existing_comment_ids(&opened)
            .max(source_comment_max)
            .map_or(0, |max| u64::from(max) + 1);
        let controls = crate::inspect::body_control_nodes(&opened.dom, opened.body);
        let control_records = crate::inspect::collect_controls(&opened.dom, opened.body);
        let family = crate::comments::CommentFamily::load(&opened.pkg, &opened.main)
            .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
        Ok(Self {
            plan,
            revision_refusal: std::cell::RefCell::new(None),
            source_sha256: source_hash,
            base,
            base_sha256,
            resolved_revisions,
            removed_comments,
            kept_comments,
            date,
            initials,
            opened,
            stories,
            paragraph_nodes,
            paragraph_story,
            projections,
            origin,
            outcomes: Vec::new(),
            resolved: Vec::new(),
            claims: BTreeMap::new(),
            anchor_window: std::cell::Cell::new(None),
            syntax: PlanSyntax::default(),
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
            rewritten: Vec::new(),
            watermark: None,
            controls,
            control_records,
            notes_story: None,
            settings: None,
            written_stories: std::collections::BTreeSet::new(),
        })
    }

    /// Whether the redline is emitted directly, as under keep: the comparer
    /// moves a comment's range off text an edit replaces whole, so a plan
    /// that does is emitted, unless it holds an operation the emitter
    /// cannot track (a footnote, an image). `update_fields` refreshes only
    /// the clean copy, which the emitted redline would then not accept to:
    /// refused beside such an edit, as under keep.
    ///
    /// The comparer also places a bookmark, comment or permission marker by
    /// the words around it, so an edit right beside one (or over one) can
    /// carry it into the new insertion, or drop it from the rejected side:
    /// such a plan is emitted too, and its new words join no range (#1156).
    /// Under `update_fields` it is compared, as before: a heading the TOC
    /// lists holds its `_Toc` bookmark around its text, and renaming it is
    /// what a plan refreshing the TOC does.
    fn emits_directly(&self) -> Result<bool, EditError> {
        if !tracked::emits_all(self) {
            return Ok(false);
        }
        // The comparer resolves compatibility branches before comparing.
        // Emit edits beside raw AC tables directly so an untouched Fallback
        // survives, as it does under keep.
        let has_alternate_table = self
            .opened
            .dom
            .descendants(self.opened.body, Some(&W::tbl()))
            .into_iter()
            .any(|table| {
                !crate::inspect::in_text_box(&self.opened.dom, table)
                    && !self
                        .opened
                        .dom
                        .ancestors(
                            table,
                            Some(&crate::namespaces::MC::name("AlternateContent")),
                        )
                        .is_empty()
            });
        if self.edits_a_comment_range() || has_alternate_table {
            if self.plan.update_fields {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    "update_fields cannot be combined with replacing the text a comment covers; refresh the fields with `jubarte convert FILE --update-fields -o OUT.docx` afterwards",
                ));
            }
            return Ok(true);
        }
        Ok(!self.plan.update_fields && self.edits_beside_a_range_marker())
    }

    /// Whether a text edit writes new words beside a range marker: one in
    /// a word the edit touches (the comparer rewrites whole words), or one
    /// at body level next to the paragraph's first or last character. A
    /// comment's marker inside the text a replacement removes does not
    /// count: a replacement of the text a comment covers is
    /// [`Self::edits_a_comment_range`], and the comparer takes a comment
    /// whose words an edit removes into the deletion, as Word does. One at
    /// the replaced text's edge, or further into its word, counts.
    fn edits_beside_a_range_marker(&self) -> bool {
        let separators = crate::comparer::WmlComparerSettings::default().word_separators;
        self.resolved.iter().any(|(_, r)| match r {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                ..
            } if !replacement.is_empty() => {
                let text = &self.projections[*para].text;
                let (from, to) = word_window(text, *start, *end, &separators);
                let marks = self.range_marks(*para);
                marks.iter().any(|(name, _, at)| {
                    (from..=to).contains(at)
                        && !(name.starts_with("comment") && (*start + 1..*end).contains(at))
                }) || (from == 0 && self.body_level_marker_beside(*para, true))
                    || (to == text.len() && self.body_level_marker_beside(*para, false))
            }
            _ => false,
        })
    }

    /// Whether a range marker sits at body level right before (or after)
    /// paragraph `para`.
    fn body_level_marker_beside(&self, para: usize, before: bool) -> bool {
        let dom = &self.opened.dom;
        let node = self.paragraph_nodes[para];
        let beside = if before {
            dom.nodes_before_self(node)
                .into_iter()
                .rev()
                .find(|&n| dom.is_element(n))
        } else {
            dom.next_element(node)
        };
        beside.is_some_and(|n| range_marker(dom, n).is_some())
    }

    /// Whether the plan names a comment's text (`cN.text`), or one of its
    /// text edits replaces exactly the text a comment covers with new text.
    fn edits_a_comment_range(&self) -> bool {
        let names_comment_text = self
            .plan
            .operations
            .iter()
            .any(|op| selector_of(&op.kind).and_then(comment_text_of).is_some());
        if names_comment_text {
            return true;
        }
        self.resolved.iter().any(|(_, r)| match r {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                ..
            } if !replacement.is_empty() => self.comment_ranges(*para).contains(&(*start, *end)),
            _ => false,
        })
    }

    /// The `(start, end)` projection offsets of each comment whose range
    /// opens and closes in paragraph `para`.
    fn comment_ranges(&self, para: usize) -> Vec<(usize, usize)> {
        let mut open: Vec<(String, usize)> = Vec::new();
        let mut ranges = Vec::new();
        for (id, starts, offset) in self.comment_marks(para) {
            if starts {
                open.push((id, offset));
            } else if let Some(at) = open.iter().position(|(o, _)| *o == id) {
                ranges.push((open.remove(at).1, offset));
            }
        }
        ranges
    }

    /// Each comment range marker of paragraph `para`, in document order:
    /// its `w:id`, whether it opens the range, and its projection offset.
    fn comment_marks(&self, para: usize) -> Vec<(String, bool, usize)> {
        #[cfg(test)]
        COMMENT_MARK_SCANS.with(|scans| scans.set(scans.get() + 1));
        self.range_marks(para)
            .into_iter()
            .filter_map(|(name, id, offset)| {
                let starts = match name {
                    "commentRangeStart" => true,
                    "commentRangeEnd" => false,
                    _ => return None,
                };
                Some((id?, starts, offset))
            })
            .collect()
    }

    /// Each range marker of paragraph `para` (see [`range_marker`]), in
    /// document order: its local name, its `w:id`, and its projection
    /// offset.
    fn range_marks(&self, para: usize) -> Vec<(&'static str, Option<String>, usize)> {
        let dom = &self.opened.dom;
        let mut ends = BTreeMap::new();
        for segment in &self.projections[para].segments {
            let node = match segment.piece {
                Piece::Text { t, .. } => t,
                Piece::Glyph { element, .. } => element,
            };
            ends.entry(node.0).or_insert(segment.end);
        }
        let mut offset = 0;
        let mut marks = Vec::new();
        for node in dom.descendants(self.paragraph_nodes[para], None) {
            if let Some(&end) = ends.get(&node.0) {
                offset = end;
                continue;
            }
            if let Some(name) = range_marker(dom, node) {
                let id = dom.attribute(node, &W::id()).map(str::to_string);
                marks.push((name, id, offset));
            }
        }
        marks
    }

    /// Every complete range in one document pass. Like `comment_text`,
    /// retain the first start and first following end for each literal id.
    fn comment_anchors(&self) -> BTreeMap<String, Covered> {
        let mut open = BTreeMap::new();
        let mut covered = BTreeMap::new();
        for para in 0..self.paragraph_nodes.len() {
            for (id, starts, offset) in self.comment_marks(para) {
                if covered.contains_key(&id) {
                    continue;
                }
                if starts {
                    open.entry(id).or_insert((para, offset));
                } else if let Some((first, start)) = open.remove(&id) {
                    covered.insert(
                        id,
                        Covered {
                            first,
                            start,
                            last: para,
                            end: offset,
                        },
                    );
                }
            }
        }
        covered
    }

    /// The text comment `n` covers (`cN.text`), from its range start to
    /// its range end.
    fn comment_text(&self, n: u32) -> Result<Covered, origin::Miss> {
        let id = n.to_string();
        let mut opened: Option<(usize, usize)> = None;
        for para in 0..self.paragraph_nodes.len() {
            for (mark, starts, offset) in self.comment_marks(para) {
                if mark != id {
                    continue;
                }
                match opened {
                    None if starts => opened = Some((para, offset)),
                    Some((first, start)) if !starts => {
                        return Ok(Covered {
                            first,
                            start,
                            last: para,
                            end: offset,
                        });
                    }
                    _ => {}
                }
            }
        }
        Err((
            "ANCHOR_NOT_FOUND".into(),
            format!("no comment c{n} covers text in this document"),
            0,
        ))
    }

    /// `cN.text` rewritten as `new_text` (empty: deleted): one tracked
    /// substitution inside the comment. A range over two paragraphs that
    /// follow each other also deletes the mark between them, as Word does.
    fn resolve_comment_text(
        &self,
        n: u32,
        new_text: &str,
        outcome: &mut EditOutcome,
    ) -> Result<Vec<Resolved>, (String, String)> {
        let covered = self.comment_text(n).map_err(|(code, message, matches)| {
            outcome.matches = matches;
            (code, message)
        })?;
        let Covered {
            first,
            start,
            last,
            end,
        } = covered;
        outcome.paragraph = Some(self.paragraph_id(first));
        if first == last && start == end {
            return Err((
                "INVALID_EDIT".into(),
                format!(
                    "c{n} covers no text; edit its paragraph ({}) instead",
                    crate::ids::short_paragraph(&self.paragraph_id(first))
                ),
            ));
        }
        check_run_text(new_text).map_err(|m| ("INVALID_EDIT".to_string(), m))?;
        let structure = |m| ("UNSUPPORTED_STRUCTURE".to_string(), m);
        outcome.matches = 1;
        let text = &self.projections[first].text;
        if first == last {
            self.check_range(&self.projections[first], start, end)
                .map_err(structure)?;
            outcome.context = Some(context(
                text,
                start,
                end,
                &format!("{{{}→{}}}", &text[start..end], new_text),
            ));
            return Ok(vec![Resolved::Text {
                para: first,
                start,
                end,
                replacement: new_text.to_string(),
                comment: None,
                attach_before: false,
                format: None,
            }]);
        }
        let dom = &self.opened.dom;
        let adjacent = last == first + 1
            && self.paragraph_story[first].0 == self.paragraph_story[last].0
            && dom.next_element(self.paragraph_nodes[first]) == Some(self.paragraph_nodes[last]);
        if !adjacent {
            return Err(structure(format!(
                "c{n} covers text from {} to {}; cN.text edits one paragraph, or two that follow each other",
                crate::ids::short_paragraph(&self.paragraph_id(first)),
                crate::ids::short_paragraph(&self.paragraph_id(last))
            )));
        }
        let first_end = text.len();
        self.check_range(&self.projections[first], start, first_end)
            .map_err(structure)?;
        self.check_range(&self.projections[last], 0, end)
            .map_err(structure)?;
        outcome.context = Some(context(
            text,
            start,
            first_end,
            &format!(
                "{{{}¶{}→{}}}",
                &text[start..],
                &self.projections[last].text[..end],
                new_text
            ),
        ));
        let mut resolved = vec![Resolved::Text {
            para: first,
            start,
            end: first_end,
            replacement: new_text.to_string(),
            comment: None,
            attach_before: start == first_end,
            format: None,
        }];
        if end > 0 {
            resolved.push(Resolved::Text {
                para: last,
                start: 0,
                end,
                replacement: String::new(),
                comment: None,
                attach_before: false,
                format: None,
            });
        }
        resolved.push(Resolved::MergeParagraphs {
            para: first,
            next: last,
            separator: String::new(),
        });
        Ok(resolved)
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
            removed_comments: self.removed_comments.clone(),
            kept_comments: self
                .kept_comments
                .iter()
                .cloned()
                .map(|mut c| {
                    let mut family: Vec<u32> = std::iter::once(c.id.as_str())
                        .chain(c.replies.iter().map(String::as_str))
                        .filter_map(|s| s.strip_prefix('c')?.parse().ok())
                        .collect();
                    for (&id, &parent) in &self.reply_parents {
                        if family.contains(&parent) && !family.contains(&id) {
                            family.push(id);
                            c.replies.push(format!("c{id}"));
                        }
                    }
                    c
                })
                .collect(),
            paragraph_ids: self
                .origin
                .as_ref()
                .map(|o| o.moves(|p| self.paragraph_id(p)))
                .unwrap_or_default(),
            split_revisions: Vec::new(),
            paragraphs: ParagraphDelta {
                from: self.body_paragraph_count(),
                to: self.body_paragraph_count(),
            },
            operations: self.outcomes.clone(),
            comments_added: self.comments_added,
            revisions: RevisionCounts::default(),
            fields: Vec::new(),
            skipped: 0,
            notes: self.rewrite_notes(),
            output_sha256: None,
        }
    }

    /// New comments take ids above every comment `source` defines, as
    /// [`Self::start`] does when it settles the source itself.
    fn skip_comment_ids_of(&mut self, source: &[u8]) -> Result<(), EditError> {
        let opened = Opened::open(source).map_err(open_error)?;
        if let Some(max) = existing_comment_ids(&opened) {
            self.next_comment_id = self.next_comment_id.max(u64::from(max) + 1);
        }
        Ok(())
    }

    /// One note per comment of another author an `edit_comment` rewrote
    /// under `rewrite_others_comments`: their name now carries our words.
    fn rewrite_notes(&self) -> Vec<String> {
        self.rewritten
            .iter()
            .filter_map(|(i, comment, author)| {
                let outcome = self.outcomes.get(*i).filter(|o| o.status == "ok")?;
                let what = if author.is_empty() {
                    format!("c{comment} names no author; its words were changed")
                } else {
                    format!(
                        "c{comment} is by {author:?}; their words were changed under their name"
                    )
                };
                Some(format!("{} (edit_comment): {what}", outcome.id))
            })
            .collect()
    }

    /// Record each `edit_comment` among `resolved` (the operation whose
    /// outcome will be `outcomes[outcome]`) that changes a comment not the
    /// plan author's, for [`Self::rewrite_notes`]: v1 and v2 alike.
    pub(crate) fn note_rewrites(&mut self, outcome: usize, resolved: &[Resolved]) {
        for r in resolved {
            if let Resolved::Thread {
                op: ThreadOp::Edit { id: comment, .. },
                ..
            } = r
                && let Some(family) = &self.family
                && let Some(author) = self.not_own_comment(family, *comment)
            {
                self.rewritten.push((outcome, *comment, author));
            }
        }
    }

    /// The author of comment `id` when it is not the plan author's, `""`
    /// for one that names no author: only a comment that names the plan's
    /// author is its own (#1264).
    fn not_own_comment(&self, family: &crate::comments::CommentFamily, id: u32) -> Option<String> {
        let author = family.author(id).unwrap_or_default();
        (author != self.plan.author).then(|| author.to_string())
    }

    /// Resolve every operation against the untouched source; the first
    /// failure is returned after every operation has been tried.
    fn resolve(&mut self) -> Result<(), EditError> {
        let mut first_failure: Option<EditError> = None;
        for (i, op) in self.plan.operations.iter().enumerate() {
            self.revision_refusal.take();
            let id = op.id.clone().unwrap_or_else(|| format!("op-{}", i + 1));
            let kind = kind_name(&op.kind).to_string();
            let window = selector_of(&op.kind)
                .and_then(comment_text_of)
                .and_then(|n| {
                    self.comment_text(n)
                        .ok()
                        .filter(|c| c.first == c.last)
                        .map(|c| (n, c.start, c.end))
                });
            self.anchor_window.set(window);
            let mut claim = None;
            let result = match &op.kind {
                OperationKind::Rewrite {
                    paragraph,
                    text,
                    comment,
                } => self.resolve_rewrite(&id, paragraph, text, comment.as_deref()),
                // `-p cN.text --delete`: the covered text goes.
                OperationKind::DeleteParagraph {
                    paragraph,
                    comment: None,
                } if comment_text_of(paragraph).is_some() => {
                    self.resolve_rewrite(&id, paragraph, "", None)
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
                        near: None,
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
                OperationKind::Strike { revision, comment }
                | OperationKind::Restore { revision, comment } => {
                    let visible = match &op.kind {
                        OperationKind::Strike { .. } => visible::Visible::Strike,
                        _ => visible::Visible::Restore,
                    };
                    self.resolve_visible(&id, visible, revision, comment.as_deref())
                }
                other => self.resolve_one(&id, other).map(|(resolved, mut outcome)| {
                    if let OperationKind::Replace {
                        whole: false,
                        format: None,
                        ..
                    } = other
                    {
                        if let Resolved::Text {
                            para, start, end, ..
                        } = resolved
                        {
                            claim = Some((para, start, end));
                        }
                        let pieces = self.replace_pieces(resolved, &mut outcome);
                        return (pieces, outcome);
                    }
                    // A deletion's comment sits on the words it strikes, as a
                    // strike's does: in the redline only.
                    let comment = match (other, &resolved) {
                        (
                            OperationKind::Delete {
                                comment: Some(_), ..
                            },
                            Resolved::Text {
                                para, start, end, ..
                            },
                        ) => Some(Resolved::StrikeComment {
                            para: *para,
                            start: *start,
                            end: *end,
                        }),
                        _ => None,
                    };
                    (comment.into_iter().chain([resolved]).collect(), outcome)
                }),
            };
            self.anchor_window.set(None);
            match result {
                Ok((resolved, mut outcome)) => {
                    outcome.id = id;
                    outcome.kind = kind;
                    outcome.status = "ok".into();
                    if let Some(claim) = claim {
                        self.claims.insert(i, claim);
                    }
                    self.note_rewrites(self.outcomes.len(), &resolved);
                    self.outcomes.push(outcome);
                    self.resolved
                        .extend(resolved.into_iter().map(|resolved| (i, resolved)));
                }
                Err(boxed) => {
                    let (e, outcome) = *boxed;
                    let mut e = self.attach_revision_refusal(e);
                    e.operation = Some(id.clone());
                    e.near = outcome.near.clone().map(Box::new);
                    self.outcomes.push(EditOutcome {
                        id,
                        kind,
                        status: "failed".into(),
                        code: Some(e.code.clone()),
                        message: Some(e.message.clone()),
                        ..outcome
                    });
                    if let Some(first) = &mut first_failure {
                        first.revision_refusals = std::mem::take(&mut first.revision_refusals)
                            .into_vec()
                            .into_iter()
                            .chain(e.revision_refusals.into_vec())
                            .collect();
                    } else {
                        first_failure = Some(e);
                    }
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
        match kind {
            OperationKind::Redact { find, .. } => {
                let result = self.resolve_one_unreported(id, kind);
                // A refusal can quote a selector, field, control or revision
                // author as well as the anchor. None is safe locating help for
                // a redaction, including the deferred revision diagnostics.
                self.revision_refusal.take();
                let scrub = |outcome: EditOutcome| EditOutcome {
                    id: outcome.id,
                    kind: outcome.kind,
                    status: outcome.status,
                    paragraph: outcome.paragraph,
                    matches: outcome.matches,
                    context: outcome.context.map(|_| format!("{{{}}}", redaction(find))),
                    comment_id: None,
                    anchor_given: None,
                    anchor_read_as: None,
                    near: None,
                    code: None,
                    message: None,
                };
                // Rebuild from safe fields on both paths. Adding an outcome
                // field requires an explicit privacy decision here, rather
                // than silently forwarding another copy of document text.
                match result {
                    Ok((resolved, outcome)) => Ok((resolved, scrub(outcome))),
                    Err(boxed) => {
                        let (error, outcome) = *boxed;
                        let at = outcome.paragraph.as_ref().map_or_else(
                            || "in the selected paragraphs".to_string(),
                            |p| format!("in {}", crate::ids::short_paragraph(p)),
                        );
                        let message = match error.code.as_str() {
                            "ANCHOR_NOT_FOUND" => format!(
                                "the text to redact does not occur {at}{}",
                                if outcome.near.is_some() {
                                    "; a passage differs from it only in whitespace"
                                } else {
                                    ""
                                }
                            ),
                            "AMBIGUOUS_ANCHOR" => format!(
                                "the text to redact has {} matches {at}; select one paragraph and set a 1-based occurrence",
                                outcome.matches
                            ),
                            "INVALID_EDIT" => format!(
                                "the text to redact needs a nonempty find; occurrence is 1-based {at}"
                            ),
                            _ => format!("the text to redact cannot be edited {at}"),
                        };
                        Err(Box::new((
                            err(&error.code, Some(id), message),
                            scrub(outcome),
                        )))
                    }
                }
            }
            _ => self.resolve_one_unreported(id, kind),
        }
    }

    fn resolve_one_unreported(
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
            near: None,
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
            OperationKind::Strike { .. } | OperationKind::Restore { .. } => {
                return Err(fail(
                    "INVALID_PLAN",
                    "strike and restore resolve through resolve_visible".into(),
                    outcome,
                ));
            }
        };
        let para = match self.select(selector, finds_text(kind)) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        if let Some(n) = comment_text_of(selector) {
            let paragraph_level = matches!(
                kind,
                OperationKind::InsertParagraph { .. }
                    | OperationKind::FormatParagraph { .. }
                    | OperationKind::MergeParagraphs { .. }
                    | OperationKind::InsertTable { .. }
                    | OperationKind::InsertToc { .. }
                    | OperationKind::InsertImage { .. }
            );
            let message = if matches!(kind, OperationKind::DeleteParagraph { .. }) {
                Some(format!("deleting c{n}.text takes no comment"))
            } else if paragraph_level {
                Some(format!(
                    "c{n}.text names text, not a paragraph; use the paragraph's id ({})",
                    crate::ids::short_paragraph(&self.paragraph_id(para))
                ))
            } else if self.anchor_window.get().is_none() {
                Some(format!(
                    "c{n} covers text in more than one paragraph; give the anchor with a paragraph id, or rewrite c{n}.text whole"
                ))
            } else {
                None
            };
            if let Some(message) = message {
                return Err(fail("INVALID_EDIT", message, outcome));
            }
        }
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
            | OperationKind::Delete { comment, .. }
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
                check_run_text(replacement)
                    .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
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
                let (start, end) = self
                    .find_range(projection, find, *occurrence, &mut outcome)
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?;
                let blocks = redaction(find);
                outcome.context = Some(format!("{{{blocks}}}"));
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
                find,
                occurrence,
                comment,
                ..
            } => {
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
                check_run_text(new).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
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
                        (
                            self.anchor_window.get().map_or(0, |(_, from, _)| from),
                            false,
                        )
                    }
                    (None, None, Some(Edge::End)) => {
                        outcome.matches = 1;
                        (
                            self.anchor_window.get().map_or(text.len(), |(_, _, to)| to),
                            true,
                        )
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
                let last = match self.select(through, false) {
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
                // A comment edits nothing: its range may cross a tab, a
                // symbol, a hyperlink, a field or a revision.
                let (start, end) = match find {
                    Some(find) => locate(
                        projection,
                        find,
                        *occurrence,
                        &mut outcome,
                        self.anchor_window.get(),
                    )
                    .map_err(|(c, m)| fail(&c, m, outcome.clone()))?,
                    None => {
                        outcome.matches = 1;
                        self.anchor_window
                            .get()
                            .map_or((0, text.len()), |(_, from, to)| (from, to))
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
            OperationKind::Strike { .. } | OperationKind::Restore { .. } => Err(fail(
                "INVALID_PLAN",
                "strike and restore resolve through resolve_visible".into(),
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
                    Some(selector) => self.select(selector, true).map_err(|(code, msg, _)| {
                        fail(&code, format!("like: {msg}"), outcome.clone())
                    })?,
                    None => para,
                };
                if runs.is_empty() || runs.iter().all(|r| r.text.is_empty()) {
                    return Err(fail("INVALID_EDIT", "runs must carry text".into(), outcome));
                }
                for r in runs {
                    check_run_text(&r.text)
                        .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
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
    /// comment's reference to anchor on; `edit_comment` on another
    /// author's comment needs `rewrite_others_comments`
    /// (`COMMENT_NOT_YOURS`).
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
            near: None,
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
            let message = if self.origin.as_ref().is_some_and(|o| o.had_comment(target)) {
                let revisions = self
                    .resolved_revisions
                    .accepted
                    .iter()
                    .chain(&self.resolved_revisions.rejected)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                // A blanket settlement names no revision: no parenthesis.
                let revisions = if revisions.is_empty() {
                    String::new()
                } else {
                    format!(" ({revisions})")
                };
                format!(
                    "c{target} is gone once the plan's revisions are settled{revisions}: its anchor and replies are removed, as Word does; a reply, edit or resolve on that thread would be lost; settle fewer revisions, drop the thread operation, or set \"removed_anchor_comments\": \"keep\"",
                )
            } else {
                format!("no comment c{target} in the document (jubarte comments lists them)")
            };
            return Err(fail("UNKNOWN_COMMENT", message, outcome));
        }
        if let ThreadOp::Reply { text, .. } | ThreadOp::Edit { text, .. } = &op {
            check_comment(text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        }
        // A comment speaks for its author: another author answers it.
        if let ThreadOp::Edit { id, .. } = op
            && !self.plan.rewrite_others_comments
            && let Some(author) = self.not_own_comment(family, id)
        {
            let (reply, rewrite) = match self.syntax {
                PlanSyntax::Flags | PlanSyntax::Typed => (
                    format!("-p c{id} --reply TEXT"),
                    "--rewrite-others-comments",
                ),
                PlanSyntax::V1 | PlanSyntax::V2 => (
                    "reply_comment".to_string(),
                    r#""rewrite_others_comments": true"#,
                ),
            };
            let message = if author.is_empty() {
                format!(
                    "c{id} names no author; reply instead ({reply}), or pass {rewrite} to change its words"
                )
            } else {
                format!(
                    "c{id} is by {author:?}; reply instead ({reply}), or pass {rewrite} to change their words under their name"
                )
            };
            return Err(fail("COMMENT_NOT_YOURS", message, outcome));
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

    /// A `replace` as the word-level edits that make its matched text read
    /// as the replacement (plan 12): kept words stay in their runs, with
    /// their formatting and the document's characters (a curly quote the
    /// anchor typed straight), and only the words that change are written,
    /// a new one formatted like the text before it. A comment covers the
    /// whole matched text. When a piece has no place to go, the replacement
    /// stays one edit that re-types the range, as `whole` does.
    fn replace_pieces(&self, resolved: Resolved, outcome: &mut EditOutcome) -> Vec<Resolved> {
        let Resolved::Text {
            para,
            start,
            end,
            replacement,
            comment,
            ..
        } = &resolved
        else {
            return vec![resolved];
        };
        let (para, start, end) = (*para, *start, *end);
        let projection = &self.projections[para];
        let text = &projection.text;
        let mut pieces = Vec::new();
        for (s, e, piece) in rewrite::replace_ranges(&text[start..end], replacement) {
            let (s, e) = (start + s, start + e);
            let mut attach_before = text[..s]
                .chars()
                .next_back()
                .is_some_and(|c| c != '\u{FFFC}');
            let checked = if s == e {
                self.check_insert_either(projection, s, &mut attach_before)
            } else {
                self.check_range(projection, s, e)
            };
            if checked.is_err() {
                return vec![resolved];
            }
            pieces.push(Resolved::Text {
                para,
                start: s,
                end: e,
                replacement: piece,
                comment: None,
                attach_before,
                format: None,
            });
        }
        if pieces.is_empty() {
            outcome.message = Some(
                "nothing changed: the text already reads as the replacement (a look-alike such as a curly quote stays; whole: true writes the replacement as typed)"
                    .into(),
            );
        }
        if let Some(text) = comment.clone() {
            pieces.push(Resolved::CommentRange {
                para,
                start,
                end,
                text,
            });
        }
        pieces
    }

    /// `rewrite`: the smallest word-level edits that make the paragraph read
    /// as `new_text`, each checked as a `replace` or an `insert` is.
    fn resolve_rewrite(
        &self,
        id: &str,
        paragraph: &Selector,
        new_text: &str,
        comment: Option<&str>,
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
            near: None,
        };
        let fail = |code: &str, msg: String, outcome: EditOutcome| {
            Box::new((err(code, Some(id), msg), outcome))
        };
        if let Some(note) = comment {
            check_comment(note).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        }
        if let Some(n) = comment_text_of(paragraph) {
            if comment.is_some() {
                return Err(fail(
                    "INVALID_EDIT",
                    format!(
                        "a rewrite of the text comment c{n} covers takes no comment; add one with a comment operation"
                    ),
                    outcome,
                ));
            }
            return match self.resolve_comment_text(n, new_text, &mut outcome) {
                Ok(resolved) => Ok((resolved, outcome)),
                Err((code, message)) => Err(fail(&code, message, outcome)),
            };
        }
        let para = match self.select(paragraph, false) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        if comment.is_some() && self.paragraph_story[para].0 != 0 {
            return Err(fail(
                "COMMENT_NOT_IN_BODY",
                "comments are supported in the body only: Word cannot anchor one in a header, footer or note".into(),
                outcome,
            ));
        }
        let projection = &self.projections[para];
        // The paragraph's symbols may be copied into the text; they stay.
        check_run_text(&new_text.replace('\u{FFFC}', ""))
            .map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
        let text = &projection.text;
        let symbols = |s: &str| s.matches('\u{FFFC}').count();
        if symbols(new_text) > symbols(text) {
            return Err(fail(
                "INVALID_EDIT",
                format!(
                    "text holds {} U+FFFC symbol placeholders but the paragraph has {} symbols; a symbol cannot be written as text",
                    symbols(new_text),
                    symbols(text)
                ),
                outcome,
            ));
        }
        let edits = rewrite::rewrite_ranges(text, new_text);
        if edits.is_empty() {
            outcome.message = Some(
                "nothing changed: the paragraph already reads as the text (a tab or a break written as a space stays)"
                    .into(),
            );
        }
        let mut resolved = Vec::with_capacity(edits.len());
        for (start, end, replacement) in edits {
            // New text joins the run before it (a tab or a break takes it
            // after itself), else the run after it when a symbol, a link or
            // a field is before it.
            let mut attach_before = text[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c != '\u{FFFC}');
            let checked = if start == end {
                self.check_insert_either(projection, start, &mut attach_before)
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
        // The comment covers the rewritten paragraph, words kept and words
        // inserted at either edge included, as PlanV2's rewrite anchors it.
        if let Some(note) = comment {
            resolved.push(Resolved::CommentSpan {
                para,
                last: para,
                text: note.to_string(),
            });
        }
        outcome.matches = 1;
        outcome.context = Some(format!("{{≡ {}}}", excerpt(new_text, 60)));
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
            near: None,
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
            let para = match self.select(selector, false) {
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

    /// What this plan's own ids resolve against.
    fn ids(&self) -> origin::Ids<'_> {
        origin::Ids {
            dom: &self.opened.dom,
            body: self.opened.body,
            story_ids: self.stories.iter().map(|s| s.id.as_str()).collect(),
            paragraph_nodes: &self.paragraph_nodes,
            paragraph_story: &self.paragraph_story,
        }
    }

    /// A plan's selector that reaches paragraph `global` by its text: its
    /// first words, or all of it, as `{"contains": …}` (with its story
    /// outside the body), when no other paragraph of its story holds them.
    /// Only a JSON plan of the first version has one.
    fn text_selector(&self, global: usize) -> Option<String> {
        if self.syntax != PlanSyntax::V1 {
            return None;
        }
        let text = &self.projections[global].text;
        let story = self.paragraph_story[global].0;
        let unique = |wanted: &str| {
            !wanted.trim().is_empty()
                && (0..self.paragraph_nodes.len())
                    .filter(|&g| self.paragraph_story[g].0 == story)
                    .filter(|&g| self.projections[g].text.contains(wanted))
                    .count()
                    == 1
        };
        let first: String = text.chars().take(SELECTOR_CHARS).collect();
        let first = match first.rfind(char::is_whitespace) {
            Some(at) if first.len() < text.len() => first[..at].trim_end(),
            _ => first.as_str(),
        };
        let contains = [first, text.as_str()]
            .into_iter()
            .find(|wanted| unique(wanted))?
            .to_string();
        let selector = Selector::Contains {
            contains,
            story: (story != 0).then(|| self.stories[story].id.clone()),
        };
        serde_json::to_string(&selector).ok()
    }

    /// The paragraph `selector` names. Positional ids name the source's
    /// paragraphs, as its author read them, even when settling revisions
    /// moved them; a paragraph merged with others is only followed when
    /// `merged_ok` (an edit that finds its text there). Text selectors read
    /// the settled text.
    fn select(&self, selector: &Selector, merged_ok: bool) -> Result<usize, origin::Miss> {
        if let Some(n) = comment_text_of(selector) {
            return self.comment_text(n).map(|covered| covered.first);
        }
        let story_index = match &self.origin {
            Some(origin) => match origin.ids().position(selector)? {
                Ok(source) => {
                    return origin.follow(
                        source,
                        merged_ok,
                        |p| self.paragraph_id(p),
                        |p| self.text_selector(p),
                    );
                }
                // The text selector's story, by name in the settled document.
                Err(_) => self.ids().position(selector)?,
            },
            None => self.ids().position(selector)?,
        };
        let story_index = match story_index {
            Ok(global) => return Ok(global),
            Err(story_index) => story_index,
        };
        let story = &self.stories[story_index].id;
        let members: Vec<usize> = (0..self.paragraph_nodes.len())
            .filter(|&g| self.paragraph_story[g].0 == story_index)
            .collect();
        let (wanted, prefix) = match selector {
            Selector::StartsWith { starts_with, .. } => (starts_with, true),
            Selector::Contains { contains, .. } => (contains, false),
            _ => unreachable!("ids and indexes are positional"),
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
            [] => Err((
                "ANCHOR_NOT_FOUND".into(),
                format!("no paragraph of {story} matches {wanted:?}"),
                0,
            )),
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
                | Resolved::InsertFootnote { para, .. }
                | Resolved::StrikeComment { para, .. }
                | Resolved::Restore { para, .. } => vec![self.paragraph_story[*para].0],
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

    /// The id of a paragraph ([`crate::ids::paragraph_id`]).
    fn paragraph_id(&self, para: usize) -> String {
        let (story, local) = self.paragraph_story[para];
        crate::ids::paragraph_id(&self.stories[story].id, local)
    }

    /// Every paragraph of the transaction's current DOM, in the fixed
    /// projection order the expectations checker reopens bytes in: the
    /// body's paragraphs in body order, then each story's own, story by
    /// story (the same collectors `load_stories` numbers paragraphs
    /// with). Structural edits shift what this walk returns; the
    /// expectations correspondence reads it off the DOM that produced the
    /// chosen primary bytes, so it never re-matches text.
    pub(super) fn paragraph_walk(&self) -> Vec<NodeId> {
        let dom = &self.opened.dom;
        let mut nodes = Vec::new();
        for (index, story) in self.stories.iter().enumerate() {
            if index == 0 {
                nodes.extend(crate::inspect::body_paragraph_nodes(dom, story.root));
            } else {
                nodes.extend(crate::inspect::story_paragraph_nodes(dom, story.root));
            }
        }
        nodes
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
        let (start, end) = match locate(
            projection,
            find,
            occurrence,
            outcome,
            self.anchor_window.get(),
        ) {
            Err((code, message)) if code == "ANCHOR_NOT_FOUND" => {
                return Err(match self.removed_anchor(projection, find) {
                    Some(hint) => ("UNSUPPORTED_STRUCTURE".into(), hint),
                    None => (code, message),
                });
            }
            found => found?,
        };
        self.check_range(projection, start, end)
            .map_err(|m| ("UNSUPPORTED_STRUCTURE".to_string(), m))?;
        Ok((start, end))
    }

    /// Every byte of `[start, end)` must come from a direct run or an
    /// editable insertion: text, a tab, break or non-breaking hyphen,
    /// which an edit over it removes. A symbol is never edited: its run's font would draw the text.
    fn check_range(&self, projection: &Projection, start: usize, end: usize) -> Result<(), String> {
        let mut covered = start;
        for seg in &projection.segments {
            if seg.end <= covered || seg.start >= end {
                continue;
            }
            if !seg.direct && editable_insertion(&self.opened.dom, &seg.piece).is_none() {
                return Err(self.refuse_opaque(run_of(&seg.piece), "the text"));
            }
            if is_symbol(projection, seg) {
                return Err("the text crosses a symbol".into());
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
                "the text crosses a field; edit the text on either side of it, or comment on it"
                    .into(),
            );
        }
        Ok(())
    }

    /// Why `subject` (the text, the insertion point) in `run` cannot be
    /// edited, and the way around it: another author's move or deletion by
    /// id, author and text, with each route around it (#1108); else the
    /// innermost hyperlink, field or content control. An insertion is
    /// editable (#1136), so what refuses inside one is the structure
    /// around or inside it.
    fn refuse_opaque(&self, run: NodeId, subject: &str) -> String {
        let dom = &self.opened.dom;
        let revision = dom
            .ancestors(run, None)
            .into_iter()
            .take_while(|&n| !dom.name_is(n, &W::p()))
            .find(|&n| hint::Held::of(dom, n).is_some_and(|held| held != hint::Held::Inserted));
        match revision {
            Some(revision) => self.revision_hint(revision, &format!("{subject} is inside")),
            None => format!("{subject} sits inside {}", self.opaque_structure(run)),
        }
    }

    /// The innermost hyperlink, field or content control that holds `run`,
    /// and the way around it.
    fn opaque_structure(&self, run: NodeId) -> String {
        let dom = &self.opened.dom;
        let ancestors = dom.ancestors(run, None);
        let mut inner: Option<String> = None;
        for &node in &ancestors {
            if dom.name_is(node, &W::p()) {
                break;
            }
            if inner.is_some() {
                continue;
            }
            let attr = |local: &str| dom.attribute(node, &W::name(local)).unwrap_or_default();
            if dom.name_is(node, &W::name("hyperlink")) {
                inner = Some(
                    "a hyperlink; edit the text on either side of it, or comment on it".into(),
                );
            } else if dom.name_is(node, &W::name("fldSimple")) {
                inner = Some(format!(
                    "the result of the field{}; edit the text on either side of it, or comment on it",
                    match attr("instr").trim() {
                        "" => String::new(),
                        instr => format!(" {instr:?}"),
                    }
                ));
            } else if dom.name_is(node, &W::name("sdt")) {
                let property = |local: &str| {
                    dom.element(node, &W::name("sdtPr"))
                        .and_then(|pr| dom.element(pr, &W::name(local)))
                        .and_then(|e| dom.attribute(e, &W::val()))
                        .map(|v| format!(" ({local} {v:?})"))
                };
                inner = Some(format!(
                    "a content control{}; change its text with fill_control, or comment on it",
                    property("alias")
                        .or_else(|| property("tag"))
                        .unwrap_or_default()
                ));
            }
        }
        inner.unwrap_or_else(|| {
            "the result of a field; edit the text on either side of it, or comment on it".into()
        })
    }

    /// The refusal for an edit `revision` holds (a `w:ins`, `w:del`,
    /// `w:moveTo`, `w:moveFrom`, `w:pPrChange` or `w:rPrChange`): `lead`
    /// says what was refused up to the revision's id, then the revision and
    /// its routes in the plan's syntax.
    fn revision_hint(&self, revision: NodeId, lead: &str) -> String {
        let dom = &self.opened.dom;
        let ancestors = dom.ancestors(revision, None);
        let story = self.stories.iter().find(|s| ancestors.contains(&s.root));
        // The same revision as data (`revision_refusals`, #1147), beside
        // the routes the message spells out.
        if story.is_some() {
            self.revision_refusal
                .replace(Some(refusal::RevisionRefusal::new(self, lead, &[revision])));
        }
        let wid = dom.attribute(revision, &W::id()).unwrap_or_default();
        let story_id = story.map_or("body", |s| s.id.as_str());
        let id = crate::ids::ChangeId::new(story_id, wid).short();
        let local = dom.name(revision).map(|n| n.local_name().to_string());
        let long_id = v2_apply::revision_long_id(
            story.map_or("word/document.xml", |s| s.part.as_str()),
            local.as_deref().unwrap_or("ins"),
            story_id,
            wid,
        );
        let held = hint::Held::of(dom, revision).unwrap_or(hint::Held::Inserted);
        // A formatting change holds the text of the run or paragraph it
        // formats; a mark revision holds none.
        let owner = if held == hint::Held::Formatted {
            ancestors
                .iter()
                .copied()
                .find(|&n| dom.name_is(n, &W::r()) || dom.name_is(n, &W::p()))
                .unwrap_or(revision)
        } else {
            revision
        };
        let text: String = dom
            .descendants(owner, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            .map(|n| dom.value(n))
            .collect();
        let paragraph = ancestors
            .iter()
            .find(|&&n| dom.name_is(n, &W::p()))
            .and_then(|p| self.paragraph_nodes.iter().position(|n| n == p))
            .map_or_else(String::new, |para| self.paragraph_id(para));
        let author = dom.attribute(revision, &W::author()).unwrap_or_default();
        // Restore takes another author's deletion in one stretch.
        let restorable = held == hint::Held::Deleted
            && author != self.plan.author
            && story.is_some_and(|s| {
                let stretches = dom
                    .descendants(s.root, Some(&W::name("del")))
                    .into_iter()
                    .filter(|&n| {
                        dom.attribute(n, &W::id()) == Some(wid)
                            && dom.parent(n).is_some_and(|p| !dom.name_is(p, &W::r_pr()))
                    })
                    .count();
                visible::restorable(dom, revision, stretches)
            });
        let holder = hint::Holder {
            held,
            id: &id,
            long_id: &long_id,
            author,
            text: &text,
            paragraph: &paragraph,
            lead,
            restorable,
        };
        // A restore of the same deletion in this command lands first; the
        // edit of its words waits for the next command.
        let restored_here = held == hint::Held::Deleted
            && self.plan.operations.iter().any(|op| {
                matches!(&op.kind, OperationKind::Restore { revision, .. }
                    if crate::ids::parse_change_ids(revision.trim())
                        .is_ok_and(|ids| ids.iter().any(|c| c.short() == id)))
            });
        if restored_here {
            return hint::restored_here_hint(&holder);
        }
        hint::settle_hint(&holder, self.syntax)
    }

    /// When `find` misses `projection`'s visible text but reaches into text
    /// another author deleted or moved away, the refusal that names that
    /// revision; the anchor is otherwise just not there.
    fn removed_anchor(&self, projection: &Projection, find: &str) -> Option<String> {
        // The projections are the transaction's own, one per paragraph.
        let para = self
            .projections
            .iter()
            .position(|p| std::ptr::eq(p, projection))?;
        self.removed_anchor_in(para, find)
    }

    /// [`Self::removed_anchor`] for paragraph `para`.
    fn removed_anchor_in(&self, para: usize, find: &str) -> Option<String> {
        let dom = &self.opened.dom;
        hint::removed_holder(dom, self.paragraph_nodes[para], find)
            .map(|revision| self.revision_hint(revision, "the text is inside"))
    }

    /// [`Self::check_insert_position`] on the side `attach_before` names,
    /// else on the other, which `attach_before` then names. A first side
    /// that failed leaves no refusal behind when the other holds: its
    /// revision did not block the edit (#1308 review).
    fn check_insert_either(
        &self,
        projection: &Projection,
        pos: usize,
        attach_before: &mut bool,
    ) -> Result<(), String> {
        self.check_insert_position(projection, pos, *attach_before)
            .or_else(|first| {
                self.check_insert_position(projection, pos, !*attach_before)
                    .map(|()| {
                        *attach_before = !*attach_before;
                        self.revision_refusal.take();
                    })
                    .map_err(|_| first)
            })
    }

    fn check_insert_position(
        &self,
        projection: &Projection,
        pos: usize,
        attach_before: bool,
    ) -> Result<(), String> {
        if projection.text.is_empty() {
            // Typed as Word types it: a run formatted as the mark.
            return Ok(());
        }
        let seg = attach_segment(projection, pos, attach_before)
            .ok_or_else(|| "no run holds the insertion point".to_string())?;
        if !seg.direct && editable_insertion(&self.opened.dom, &seg.piece).is_none() {
            return Err(self.refuse_opaque(run_of(&seg.piece), "the insertion point"));
        }
        if is_symbol(projection, seg) {
            return Err("the insertion point touches a symbol".into());
        }
        Ok(())
    }

    fn check_conflicts(&self) -> Result<(), EditError> {
        self.check_conflicts_with_comments(false)
    }

    fn check_conflicts_v2(&self) -> Result<(), EditError> {
        self.check_conflicts_with_comments(true)
    }

    fn check_conflicts_with_comments(&self, v2: bool) -> Result<(), EditError> {
        let comment_conflict = |i, message| {
            let mut error = self.conflict(i, message);
            if v2 {
                error.code = COMMENT_ANCHOR_CONFLICT.into();
                error.outcomes[i].code = Some(error.code.clone());
            }
            error
        };
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
        for (&i, &(para, start, end)) in &self.claims {
            if deleted.contains(&para) {
                return Err(self.conflict(i, "edits text of a deleted paragraph"));
            }
            ranges.entry(para).or_default().push((start, end, i));
        }
        for (i, r) in &self.resolved {
            let (para, start, end) = match r {
                // A replace's pieces lie inside the range it claims.
                Resolved::Text { .. } if self.claims.contains_key(i) => continue,
                Resolved::Text {
                    para, start, end, ..
                } => (*para, *start, *end),
                // A restore inserts its copy where the deletion is.
                Resolved::Restore { para, at, .. } => (*para, *at, *at),
                Resolved::CommentRange { para, .. } => {
                    if deleted.contains(para) {
                        return Err(comment_conflict(*i, "comments on a deleted paragraph"));
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
                    // V2 has tested preservation across interior deletions;
                    // the adapter separately checks content and endpoints.
                    let removes_anchor = if v2 {
                        deleted.contains(para) || deleted.contains(last)
                    } else {
                        (*para..=*last).any(|p| deleted.contains(&p))
                    };
                    if removes_anchor {
                        return Err(comment_conflict(*i, "comments on a deleted paragraph"));
                    }
                    continue;
                }
                // A reply to a comment whose paragraph the plan deletes:
                // the tracked deletion keeps the reference in struck text,
                // where Word shows the thread (#1159). Editing mode removes
                // the anchor, which v2's anchor check refuses.
                Resolved::DeleteParagraph { .. }
                | Resolved::FormatParagraph { .. }
                | Resolved::MergeParagraphs { .. }
                | Resolved::List { .. }
                | Resolved::PageSetup { .. }
                | Resolved::Thread { .. }
                | Resolved::FillControl { .. }
                | Resolved::StrikeComment { .. } => continue,
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
                return Err(comment_conflict(
                    *i,
                    "comment range cuts through an edited range",
                ));
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
        let needed = self.comments_planned() as u64;
        if needed > 0 && self.next_comment_id + needed - 1 > u64::from(u32::MAX) {
            return Err(err(
                "INVALID_DOCUMENT",
                None,
                "the source's comment ids leave no room for new comments",
            ));
        }
        Ok(())
    }

    /// How many comments the resolved operations write: a comment on a
    /// text edit, an inserted or deleted paragraph, a comment operation and
    /// a reply each write one.
    fn comments_planned(&self) -> usize {
        self.resolved
            .iter()
            .filter(|(i, r)| match r {
                Resolved::Text { comment, .. } | Resolved::InsertParagraph { comment, .. } => {
                    comment.is_some()
                }
                Resolved::CommentRange { .. } | Resolved::CommentSpan { .. } => true,
                Resolved::Thread { op, .. } => matches!(op, ThreadOp::Reply { .. }),
                Resolved::DeleteParagraph { .. } | Resolved::StrikeComment { .. } => {
                    self.deletion_comment(*i).is_some()
                }
                Resolved::Restore { comment, .. } => comment.is_some(),
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
            .count()
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
            near: None,
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
    fn apply_comments(&mut self) -> BTreeMap<usize, u32> {
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
                        Resolved::DeleteParagraph { .. } | Resolved::StrikeComment { .. } => {
                            self.deletion_comment(*i).map(str::to_string)
                        }
                        Resolved::Restore { comment, .. } => comment.clone(),
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
            let id = if self.deletion_comment(i).is_some() || self.reply_in_deleted(i) {
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
        ids
    }

    fn apply_text_edits(&mut self, ids: &BTreeMap<usize, u32>) -> Result<(), EditError> {
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
            let mut empty = project_paragraph(&self.opened.dom, node)
                .text
                .is_empty()
                .then(EmptyPlacement::default);
            sort_edits(&mut edits, empty.is_some());
            // Apply in reverse so earlier source offsets stay valid.
            for edit in edits.iter().rev() {
                let projection = project_paragraph(&self.opened.dom, node);
                // The insertions the edit reaches into: one it empties goes,
                // as in the redline.
                let mut carriers: Vec<NodeId> = projection
                    .segments
                    .iter()
                    .filter(|s| s.start < edit.end && edit.start < s.end)
                    .filter_map(|s| {
                        self.opened
                            .dom
                            .parent(run_of(&s.piece))
                            .filter(|&n| self.opened.dom.name_is(n, &W::ins()))
                    })
                    .collect();
                carriers.dedup();
                if let Some(placement) = empty.as_mut() {
                    if !edit.replacement.is_empty() {
                        let run = mark_run(&mut self.opened.dom, node);
                        placement.place(&mut self.opened.dom, node, run, edit.attach_before);
                        write_mark_run_text(&mut self.opened.dom, run, &edit.replacement);
                    }
                } else {
                    apply_text_edit(
                        &mut self.opened.dom,
                        &projection,
                        edit.start,
                        edit.end,
                        &edit.replacement,
                        edit.attach_before,
                    );
                }
                for ins in carriers {
                    tracked::remove_empty_insertion(&mut self.opened.dom, ins);
                }
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
                let OperationKind::Replace { whole: true, .. } = &plan.operations[edit.op].kind
                else {
                    continue;
                };
                if edit.replacement.is_empty() {
                    continue;
                }
                // The text the anchor matched, as the document has it: an
                // anchor typed with look-alikes or Markdown marks reads as it.
                let find = self.projections[para].text[edit.start..edit.end].to_string();
                let s = new_position(&edits, edit.start, true, Some(edit.op));
                let end = s + edit.replacement.len();
                let story = self.paragraph_story[para].0;
                let part = (story != 0).then(|| self.stories[story].part.clone());
                if let Some(mark) = whole::mark(
                    &mut self.opened.dom,
                    node,
                    (s, end),
                    (edit.op, part),
                    (&find, &edit.replacement),
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
                // The operation's own insertions at either end of its
                // range are its new text: its comment holds them.
                let own_at_end: usize = edits
                    .iter()
                    .filter(|x| x.op == i && x.start == end && x.end == end)
                    .map(|x| x.replacement.len())
                    .sum();
                let s = new_position(&edits, start, true, Some(i));
                let e = new_position(&edits, end, false, None) + own_at_end;
                pending.push((s, e, i, text));
            }
            pending.sort_by_key(|&(s, _, i, _)| (s, i));
            for (start, end, i, _) in pending {
                anchor_comment(&mut self.opened.dom, node, start, end, ids[&i]);
            }
        }
        // 1a. Restores: plain copies of the deletion's runs after it, which
        // stays; found by `w:id`, so the text edits before do not move them.
        for (i, r) in &self.resolved {
            let Resolved::Restore { para, wid, .. } = r else {
                continue;
            };
            let node = self.paragraph_nodes[*para];
            let placed = visible::restore_plain(&mut self.opened.dom, node, wid);
            if let (Some((first, last)), Some(&id)) = (placed, ids.get(i)) {
                visible::comment_around(&mut self.opened.dom, first, last, id);
            }
        }
        Ok(())
    }

    fn apply_comment_spans(&mut self, ids: &BTreeMap<usize, u32>) {
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
    }

    fn apply_insertions(&mut self, ids: &BTreeMap<usize, u32>) -> Vec<NodeId> {
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
        tables
    }

    fn apply_paragraph_formatting(&mut self) {
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
    }

    fn apply_lists(&mut self) {
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
    }

    fn apply_merges(&mut self) {
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
    }

    fn apply_paragraph_deletions(&mut self) {
        // 5. Paragraph deletions.
        for (_, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                self.opened.dom.remove(self.paragraph_nodes[*para]);
            }
        }
    }

    fn apply_reply_markers(&mut self, ids: &BTreeMap<usize, u32>) {
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
    }

    fn apply_table_separations(&mut self, tables: Vec<NodeId>) {
        // 7. A paragraph after each new table, and between it and a table
        // before it, once the deletions have settled its neighbours.
        for table in tables {
            structural::separate(&mut self.opened.dom, table);
        }
    }

    fn apply_page_setups(&mut self) {
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
    }

    /// Mutate the copy. Resolution succeeded, so every step here is total.
    fn apply(&mut self) -> Result<(), EditError> {
        let ids = self.apply_comments();
        self.apply_text_edits(&ids)?;
        self.apply_comment_spans(&ids);
        // 1c. Content control fills (text edits never reach control content).
        self.apply_fills();
        let tables = self.apply_insertions(&ids);
        self.apply_paragraph_formatting();
        self.apply_lists();
        self.apply_merges();
        self.apply_paragraph_deletions();
        self.apply_reply_markers(&ids);
        self.apply_table_separations(tables);
        self.apply_page_setups();
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

    /// The comment of operation `op` when it deletes text or a paragraph,
    /// or strikes an insertion: on deleted text, so in the redline only.
    fn deletion_comment(&self, op: usize) -> Option<&str> {
        match &self.plan.operations[op].kind {
            OperationKind::Delete { comment, .. }
            | OperationKind::DeleteParagraph { comment, .. }
            | OperationKind::Strike { comment, .. } => comment.as_deref(),
            _ => None,
        }
    }

    /// Whether operation `op` replies to a comment whose reference sits in
    /// a paragraph this plan deletes: the copy loses the reference, so the
    /// reply goes where the parent's stays, on the deleted text (#1159).
    fn reply_in_deleted(&self, op: usize) -> bool {
        self.reply_on_deleted(op).is_some()
    }

    /// The parent of reply `op` and the operation that deletes the
    /// paragraph the parent is on, when the plan deletes it.
    fn reply_on_deleted(&self, op: usize) -> Option<(u32, usize)> {
        self.resolved.iter().find_map(|(i, r)| match r {
            Resolved::Thread {
                op: ThreadOp::Reply { parent, .. },
                anchor: Some(anchor),
                ..
            } if *i == op => self.deletes_paragraph(*anchor).map(|by| (*parent, by)),
            _ => None,
        })
    }

    /// The operation that deletes paragraph `para`, if one does.
    fn deletes_paragraph(&self, para: usize) -> Option<usize> {
        self.resolved.iter().find_map(|(i, r)| {
            matches!(r, Resolved::DeleteParagraph { para: p } if *p == para).then_some(*i)
        })
    }

    /// The text of [`Self::deletion_comment`], or of a reply
    /// [`Self::reply_in_deleted`] names: what goes on the deleted text.
    fn deleted_text_comment(&self, op: usize) -> Option<&str> {
        self.deletion_comment(op)
            .or_else(|| match &self.plan.operations[op].kind {
                OperationKind::ReplyComment { text, .. } if self.reply_in_deleted(op) => {
                    Some(text.as_str())
                }
                _ => None,
            })
    }

    /// The copy the comparer reads as the original: the base, with each
    /// deleted paragraph's comment anchored on its whole text, and each
    /// reply to a comment in a deleted paragraph beside its parent. The comparer
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
        // A strike's comment is emitted with it, under keep: never compared.
        let mut operations: Vec<Operation> = self
            .deletion_comments
            .iter()
            .filter_map(|&(_, op)| {
                let operation = &self.plan.operations[op];
                // A reply sits beside its parent in the base, as in Word.
                if matches!(operation.kind, OperationKind::ReplyComment { .. }) {
                    return Some(operation.clone());
                }
                // A deleted text's comment takes the deletion's own anchor,
                // which finds the same words in the base.
                let (paragraph, find, occurrence, text) = match &operation.kind {
                    OperationKind::DeleteParagraph {
                        paragraph,
                        comment: Some(text),
                    } => (paragraph, None, None, text),
                    OperationKind::Delete {
                        paragraph,
                        find,
                        occurrence,
                        comment: Some(text),
                    } => (paragraph, Some(find.clone()), *occurrence, text),
                    _ => return None,
                };
                // A positional id names the paragraph as the plan's author
                // read it, and the base is settled: the copy names the
                // deletion's own paragraph by its id there. A comment's
                // text (`c5.text`) reads the same in both.
                let paragraph = match comment_text_of(paragraph) {
                    Some(_) => paragraph.clone(),
                    None => self
                        .resolved
                        .iter()
                        .find_map(|(i, resolved)| match resolved {
                            Resolved::Text { para, .. } | Resolved::DeleteParagraph { para }
                                if *i == op =>
                            {
                                Some(Selector::Name(self.paragraph_id(*para)))
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| paragraph.clone()),
                };
                Some(Operation {
                    id: operation.id.clone(),
                    group: operation.group.clone(),
                    kind: OperationKind::Comment {
                        paragraph,
                        find,
                        text: text.clone(),
                        through: None,
                        occurrence,
                    },
                })
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
            removed_anchor_comments: RemovedAnchorComments::default(),
            existing_revisions: ExistingRevisions::default(),
            operations,
            update_fields: false,
            rewrite_others_comments: self.plan.rewrite_others_comments,
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
        let lost_reference = self.removes_comment_reference(&written);
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
        self.written_stories = written;
        if lost_reference {
            // A comment whose reference went with deleted text goes too, as
            // Accept All takes it (Word rule R1): no definition is left
            // without an anchor.
            let stories: Vec<String> =
                crate::revision_processor::revision_bearing_parts(&self.opened.pkg)
                    .into_iter()
                    .filter(|(_, is_styles)| !is_styles)
                    .map(|(part, _)| part)
                    .collect();
            crate::revision_processor::prune_orphan_comments(&mut self.opened.pkg, &stories);
        }
        let clean = self
            .opened
            .pkg
            .to_zip()
            .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
        #[cfg(test)]
        let clean = prepared::tests::serialized_candidate(clean);
        Ok((clean, marked))
    }

    /// Whether the plan took a comment reference out of a story it writes
    /// (a deleted paragraph's, or deleted text's), read against the story as
    /// the package still holds it.
    fn removes_comment_reference(&self, stories: &std::collections::BTreeSet<usize>) -> bool {
        if self.opened.pkg.part_bytes("word/comments.xml").is_none() {
            return false;
        }
        let name = W::name("commentReference");
        let ids = |dom: &Dom, root: NodeId| -> std::collections::HashSet<String> {
            dom.descendants(root, Some(&name))
                .into_iter()
                .filter_map(|r| dom.attribute(r, &W::id()).map(str::to_string))
                .collect()
        };
        stories.iter().any(|&story| {
            let StoryPart { part, root, .. } = &self.stories[story];
            let Some(xml) = self.opened.pkg.part_string(part) else {
                return false;
            };
            let mut source = Dom::new();
            let document = source.parse_xdocument(&xml);
            source.root(document).is_some_and(|source_root| {
                !ids(&source, source_root).is_subset(&ids(&self.opened.dom, *root))
            })
        })
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
                // Rescue at settlement is provisional: both plan schemas'
                // reports must describe the family we actually publish.
                let removed: Vec<_> = family
                    .remove(*id)
                    .into_iter()
                    .map(|id| format!("c{id}"))
                    .collect();
                self.kept_comments.retain(|c| !removed.contains(&c.id));
                for kept in &mut self.kept_comments {
                    kept.replies.retain(|id| !removed.contains(id));
                }
            }
        }
        family.store(&mut self.opened.pkg, &main);
        Ok(())
    }
}

/// Comment `id`'s markers under `scope`, anchored on words a deletion then
/// took: the range start leaves the deletion it opens in, and the range
/// end, with the reference run after it, the one it closes in. A marker in
/// the middle of a deletion (the comparer joins deletions that touch into
/// one) splits it in two, the second part under the fresh revision id
/// `next_id`. So the comment wraps the deleted words, as a strike's does,
/// and accepting the deletion keeps the comment's reference.
fn lift_out_of_deletion(dom: &mut Dom, scope: NodeId, id: u32, next_id: &mut u64) {
    let id = id.to_string();
    let marker = |dom: &Dom, local: &str| {
        dom.descendants(scope, Some(&W::name(local)))
            .into_iter()
            .find(|&m| dom.attribute(m, &W::id()) == Some(id.as_str()))
    };
    if let Some(start) = marker(dom, "commentRangeStart") {
        while let Some(del) = dom.parent(start).filter(|&p| dom.name_is(p, &W::del())) {
            if dom.elements(del, None).first() != Some(&start) {
                split_deletion_before(dom, del, start, next_id);
                continue;
            }
            dom.remove(start);
            dom.add_before_self(del, start);
            drop_if_empty(dom, del);
        }
    }
    let Some(end) = marker(dom, "commentRangeEnd") else {
        return;
    };
    let reference = dom.next_element(end).filter(|&run| {
        dom.element(run, &W::name("commentReference"))
            .is_some_and(|r| dom.attribute(r, &W::id()) == Some(id.as_str()))
    });
    let last = reference.unwrap_or(end);
    while let Some(del) = dom.parent(end).filter(|&p| dom.name_is(p, &W::del())) {
        if dom.elements(del, None).last() != Some(&last) {
            if let Some(&after) = dom.nodes_after_self(last).first() {
                split_deletion_before(dom, del, after, next_id);
            }
            continue;
        }
        dom.remove(end);
        dom.add_after_self(del, end);
        if let Some(run) = reference {
            dom.remove(run);
            dom.add_after_self(end, run);
        }
        drop_if_empty(dom, del);
    }
}

/// Split deletion `del` before its child `at`: `at` and what follows it
/// move into a copy of `del` right after it, under revision id `next_id`.
fn split_deletion_before(dom: &mut Dom, del: NodeId, at: NodeId, next_id: &mut u64) {
    let tail = dom.new_element(W::del());
    for (name, value) in dom.attributes(del) {
        dom.set_attribute_value(tail, &name, Some(&value));
    }
    dom.set_attribute_value(tail, &W::id(), Some(&next_id.to_string()));
    *next_id += 1;
    let mut moved = vec![at];
    moved.extend(dom.nodes_after_self(at));
    for node in moved {
        dom.remove(node);
        dom.add(tail, node);
    }
    dom.add_after_self(del, tail);
}

/// Remove deletion `del` once a lifted marker leaves it no element.
fn drop_if_empty(dom: &mut Dom, del: NodeId) {
    if dom.elements(del, None).is_empty() {
        dom.remove(del);
    }
}

/// One more than the largest `w:id` in the stories of `opened`: a revision
/// id no annotation holds.
fn next_annotation_id(opened: &Opened) -> u64 {
    let largest = |dom: &Dom, root: NodeId| {
        dom.descendants(root, None)
            .into_iter()
            .filter_map(|n| dom.attribute(n, &W::id())?.parse::<u64>().ok())
            .max()
            .unwrap_or(0)
    };
    let mut max = largest(&opened.dom, opened.document);
    for (_, _, part) in opened.story_parts() {
        if let Ok((dom, document, _)) = crate::inspect::parse_part(&opened.pkg, &part) {
            max = max.max(largest(&dom, document));
        }
    }
    max + 1
}

/// `redline` with the markers of each comment in `ids`, on words the
/// comparer deleted, lifted out of those deletions
/// ([`lift_out_of_deletion`]): the comment wraps the deletion,
/// so accepting it keeps the comment's reference.
fn lift_deletion_comments(redline: &[u8], ids: &[u32]) -> Result<Vec<u8>, EditError> {
    if ids.is_empty() {
        return Ok(redline.to_vec());
    }
    let mut red = Opened::open(redline).map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
    let mut next_id = next_annotation_id(&red);
    for &id in ids {
        lift_out_of_deletion(&mut red.dom, red.body, id, &mut next_id);
    }
    let xml = red.dom.serialize_document(red.document);
    let main = red.main.clone();
    red.pkg.set_part(&main, xml.into_bytes());
    red.pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
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

pub(crate) fn kind_name(kind: &OperationKind) -> &'static str {
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
        OperationKind::Strike { .. } => "strike",
        OperationKind::Restore { .. } => "restore",
    }
}

/// The operation edits or comments text it finds in its paragraph, so it
/// may follow a paragraph into the one it merged with.
/// The comment `N` a selector `cN.text` names.
fn comment_text_of(selector: &Selector) -> Option<u32> {
    match selector {
        Selector::Name(name) | Selector::Id { id: name } => crate::ids::comment_text_id(name),
        _ => None,
    }
}

/// The paragraph an operation names, when it names one.
fn selector_of(kind: &OperationKind) -> Option<&Selector> {
    match kind {
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
        | OperationKind::Redact { paragraph, .. } => Some(paragraph),
        _ => None,
    }
}

/// Every paragraph an operation names: its paragraph, a comment's
/// `through`, an inserted paragraph's `like`, a list's `paragraphs`.
fn selectors_mut(kind: &mut OperationKind) -> Vec<&mut Selector> {
    match kind {
        OperationKind::Comment {
            paragraph, through, ..
        } => std::iter::once(paragraph).chain(through.as_mut()).collect(),
        OperationKind::InsertParagraph {
            paragraph, like, ..
        } => std::iter::once(paragraph).chain(like.as_mut()).collect(),
        OperationKind::List { paragraphs, .. } => paragraphs.iter_mut().collect(),
        OperationKind::Replace { paragraph, .. }
        | OperationKind::Insert { paragraph, .. }
        | OperationKind::Delete { paragraph, .. }
        | OperationKind::DeleteParagraph { paragraph, .. }
        | OperationKind::FormatParagraph { paragraph, .. }
        | OperationKind::MergeParagraphs { paragraph, .. }
        | OperationKind::Rewrite { paragraph, .. }
        | OperationKind::InsertTable { paragraph, .. }
        | OperationKind::InsertToc { paragraph, .. }
        | OperationKind::FormatRun { paragraph, .. }
        | OperationKind::InsertFootnote { paragraph, .. }
        | OperationKind::InsertImage { paragraph, .. }
        | OperationKind::Redact { paragraph, .. } => vec![paragraph],
        _ => Vec::new(),
    }
}

fn finds_text(kind: &OperationKind) -> bool {
    match kind {
        OperationKind::Replace { .. }
        | OperationKind::Delete { .. }
        | OperationKind::FormatRun { .. }
        | OperationKind::Redact { .. } => true,
        OperationKind::Insert { after, before, .. } => after.is_some() || before.is_some(),
        OperationKind::Comment { find, through, .. } => find.is_some() && through.is_none(),
        _ => false,
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

/// Run text: plain text in which `\t` writes a tab and `\n` a line break,
/// as the agent view prints them; any other control character, and the
/// view's symbol placeholder U+FFFC, is refused by name.
fn check_run_text(text: &str) -> Result<(), String> {
    let bad = text.chars().find(|&c| {
        (c.is_control() && !matches!(c, '\t' | '\n'))
            || matches!(c, '\u{fffe}' | '\u{ffff}' | '\u{FFFC}')
    });
    match bad {
        None => Ok(()),
        Some('\r') => Err("text holds U+000D, a carriage return; write a line break as \\n".into()),
        Some('\u{FFFC}') => Err(
            "text holds U+FFFC, which stands for a symbol in the view; a symbol cannot be written as text".into(),
        ),
        Some(c) => Err(format!(
            "text holds U+{:04X}, which a run cannot carry; write a tab as \\t and a line break as \\n",
            u32::from(c)
        )),
    }
}

/// An optional comment as a plan gives it: blank text (`""`, spaces, line
/// breaks) is no comment, as an agent with nothing to say writes it.
fn blank_is_none<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error> {
    let text: Option<String> = serde::Deserialize::deserialize(de)?;
    Ok(text.filter(|t| !t.trim().is_empty()))
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
    // Insertions at one point stand in `edits`' order (see `sort_edits`).
    let own_at = own.and_then(|o| edits.iter().position(|e| e.op == o));
    let mut delta: i64 = 0;
    for (k, edit) in edits.iter().enumerate() {
        let (start, end, i) = (edit.start, edit.end, edit.op);
        if Some(i) == own {
            continue;
        }
        let shifts = if start == end {
            // insertion
            if inclusive {
                start < pos || (start == pos && own_at.is_none_or(|o| k < o))
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

/// The unique occurrence of `find` in the paragraph, or its
/// `occurrence`-th hit (1-based) when given (overlapping occurrences
/// count), whatever structure it crosses.
fn locate(
    projection: &Projection,
    find: &str,
    occurrence: Option<usize>,
    outcome: &mut EditOutcome,
    window: Option<(u32, usize, usize)>,
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
    // Under `cN.text`, only hits inside the text the comment covers count.
    let outside = std::cell::Cell::new(false);
    let inside = |hits: Vec<(usize, usize)>| -> Vec<(usize, usize)> {
        let Some((_, from, to)) = window else {
            return hits;
        };
        let kept: Vec<_> = hits
            .iter()
            .copied()
            .filter(|&(s, e)| from <= s && e <= to)
            .collect();
        if kept.len() < hits.len() {
            outside.set(true);
        }
        kept
    };
    let exact = |needle: &str| -> Vec<(usize, usize)> {
        inside(
            hits_of(needle)
                .into_iter()
                .map(|i| (i, i + needle.len()))
                .collect(),
        )
    };
    let mut hits = exact(find);
    // An anchor copied out of the agent view can carry Markdown marks
    // (`# `, `**`, CriticMarkup notes) that are not document text, and
    // one typed by hand a space where the paragraph has a non-breaking
    // one, or a straight quote for a curly one: the literal is tried
    // first, then the text without marks, then each as it reads.
    if hits.is_empty() {
        let plain = crate::markdown::plain_anchor(find);
        let plain = (!plain.is_empty() && plain != find).then_some(plain);
        let mut tries = Vec::new();
        if let Some(plain) = &plain {
            tries.push(exact(plain));
        }
        tries.push(inside(anchor::reading_hits(text, find)));
        if let Some(plain) = &plain {
            tries.push(inside(anchor::reading_hits(text, plain)));
        }
        if let Some(found) = tries.into_iter().find(|t| !t.is_empty()) {
            let read_as: String = text[found[0].0..found[0].1].to_string();
            if read_as != find {
                outcome.anchor_given = Some(find.to_string());
                outcome.anchor_read_as = Some(read_as);
            }
            hits = found;
        }
    }
    outcome.matches = hits.len();
    let shown = anchor::shown(find);
    let (start, end) = match (hits.as_slice(), occurrence) {
        ([], _) => {
            if window.is_none() {
                outcome.near = anchor::near_matches(text, find);
            }
            return Err((
                "ANCHOR_NOT_FOUND".into(),
                match window {
                    Some((n, ..)) if outside.get() => {
                        format!("{shown} is not in the text c{n} covers")
                    }
                    Some((n, ..)) => format!("{shown} does not occur in the text c{n} covers"),
                    None => format!("{shown} does not occur in the paragraph"),
                },
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
                format!("{shown} occurs {n} times in the paragraph; set \"occurrence\" to 1..={n}"),
            ));
        }
        (many, Some(k)) if k <= many.len() => many[k - 1],
        (many, Some(k)) => {
            let n = many.len();
            return Err((
                "AMBIGUOUS_ANCHOR".into(),
                format!(
                    "{shown} occurs {n} times in the paragraph; occurrence {k} is outside occurrence 1..={n}"
                ),
            ));
        }
    };
    Ok((start, end))
}

/// A run directly inside a paragraph's pending insertion is editable.
/// Other wrappers and complex field results retain their refusal: an
/// insertion around a field does not make that field ordinary text.
fn editable_insertion(dom: &Dom, piece: &Piece) -> Option<NodeId> {
    let run = run_of(piece);
    let ins = dom.parent(run).filter(|&n| dom.name_is(n, &W::ins()))?;
    let paragraph = dom.parent(ins).filter(|&n| dom.name_is(n, &W::p()))?;
    let target = match *piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut fields = 0usize;
    for node in dom.descendants(paragraph, None) {
        if node == target {
            return (fields == 0).then_some(ins);
        }
        // Markers in removed text count too: Word's Accept All takes the
        // words between a deleted `begin` and `end` with the field.
        if dom.name_is(node, &W::name("fldChar")) {
            match dom.attribute(node, &W::name("fldCharType")) {
                Some("begin") => fields += 1,
                Some("end") => fields = fields.saturating_sub(1),
                _ => {}
            }
        }
    }
    None
}

/// `seg` is a `w:sym`, projected as U+FFFC.
fn is_symbol(projection: &Projection, seg: &crate::inspect::Segment) -> bool {
    matches!(seg.piece, Piece::Glyph { .. }) && projection.text[seg.start..seg.end] == *"\u{FFFC}"
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

/// How many characters of a paragraph's text a suggested `contains`
/// selector quotes at most.
const SELECTOR_CHARS: usize = 60;

/// The byte span of the words of `text` that `[start, end)` touches: out
/// to the nearest whitespace or comparer word separator on each side, so
/// at least the words the comparer would rewrite. `start` and `end` are
/// byte offsets on char boundaries, as every projection offset is.
fn word_window(text: &str, start: usize, end: usize, separators: &[char]) -> (usize, usize) {
    let wordy = |c: char| !c.is_whitespace() && !separators.contains(&c);
    let from = text[..start]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| wordy(c))
        .last()
        .map_or(start, |(i, _)| i);
    let to = text[end..]
        .char_indices()
        .find(|&(_, c)| !wordy(c))
        .map_or(text.len(), |(i, _)| end + i);
    (from, to)
}

/// `node`'s local name when it is a range marker: an opener of
/// [`tracked::RANGE_STARTS`], a closer of [`tracked::RANGE_ENDS`], or a
/// comment range's end.
fn range_marker(dom: &Dom, node: NodeId) -> Option<&'static str> {
    let name = dom.name(node).filter(|n| n.namespace_name() == W::URI)?;
    tracked::RANGE_STARTS
        .into_iter()
        .chain(tracked::RANGE_ENDS)
        .chain([tracked::COMMENT_RANGE_END])
        .find(|&marker| marker == name.local_name())
}

/// Insert `replacement` at `at` of `t` (of `len` bytes: a `w:t`, or a tab
/// or break glyph) in its own run, formatted as `run`, when `at` is an edge
/// of `run` with a range marker beside it: the new words then join no range
/// they meet, as the redline writes them (#1156). Whether it did.
fn insert_outside_ranges(
    dom: &mut Dom,
    run: NodeId,
    t: NodeId,
    at: usize,
    len: usize,
    replacement: &str,
) -> bool {
    let content: Vec<NodeId> = dom
        .elements(run, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::r_pr()))
        .collect();
    let before = if at == 0 && content.first() == Some(&t) {
        true
    } else if at == len && content.last() == Some(&t) {
        false
    } else {
        return false;
    };
    let beside = tracked::outside_ranges(dom, run, before);
    if beside == run {
        return false;
    }
    let new = dom.new_element(W::r());
    if let Some(rpr) = dom.element(run, &W::r_pr()) {
        let rpr = tracked::clean_rpr(dom, rpr);
        dom.add(new, rpr);
    }
    let text = dom.new_element(W::t());
    dom.add(new, text);
    write_text(dom, text, replacement);
    if before {
        dom.add_before_self(beside, new);
    } else {
        dom.add_after_self(beside, new);
    }
    true
}

/// Replace `[start, end)` of the paragraph's projection with `replacement`.
/// The replacement goes to the first piece of the range; the other pieces
/// lose their text; a tab, a line break or a non-breaking hyphen in the range
/// is removed. A page or column break is no text and stays.
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
        // At a run edge the source range endpoints keep their side of the
        // words. A separate run lets both the clean copy and the comparer
        // retain that boundary instead of extending the source range.
        let (leaf, run) = match seg.piece {
            Piece::Text { t, run } => (t, run),
            Piece::Glyph { element, run } => (element, run),
        };
        let before = start == seg.start;
        let content: Vec<_> = dom
            .elements(run, None)
            .into_iter()
            .filter(|&n| !dom.name_is(n, &W::r_pr()))
            .collect();
        let at_run_edge = if before {
            content.first() == Some(&leaf)
        } else {
            start == seg.end && content.last() == Some(&leaf)
        };
        if at_run_edge {
            let anchor = tracked::insertion_boundary_anchor(dom, run, before);
            if anchor != run {
                let inserted = dom.new_element(W::r());
                if let Some(rpr) = dom.element(run, &W::r_pr()) {
                    let rpr = tracked::clean_rpr(dom, rpr);
                    dom.add(inserted, rpr);
                }
                let t = dom.new_element(W::t());
                dom.add(inserted, t);
                write_text(dom, t, replacement);
                if before {
                    dom.add_before_self(anchor, inserted);
                } else {
                    dom.add_after_self(anchor, inserted);
                }
                return;
            }
        }
        match seg.piece {
            Piece::Text { t, run } => {
                let value = dom.value(t);
                let at = start - seg.start;
                if !insert_outside_ranges(dom, run, t, at, value.len(), replacement) {
                    let updated = format!("{}{}{}", &value[..at], replacement, &value[at..]);
                    write_text_new_in(dom, t, &updated, at..at + replacement.len());
                }
            }
            // Beside a tab or a break: new text of the glyph's run, after it
            // when it holds the char before the point.
            Piece::Glyph { element, run } => {
                let (at, len) = (start - seg.start, seg.end - seg.start);
                if !insert_outside_ranges(dom, run, element, at, len, replacement) {
                    let t = dom.new_element(W::t());
                    if seg.end == start {
                        dom.add_after_self(element, t);
                    } else {
                        dom.add_before_self(element, t);
                    }
                    write_text(dom, t, replacement);
                }
            }
        }
        return;
    }
    let mut inside = inner_comment_references(dom, projection, start, end);
    inside.extend(closing_comment_references(dom, projection, start, end));
    let mut pending = Some(replacement);
    for seg in &projection.segments {
        if seg.end <= start || seg.start >= end {
            continue;
        }
        match seg.piece {
            Piece::Text { t, .. } => {
                let value = dom.value(t);
                let from = start.saturating_sub(seg.start).min(value.len());
                let to = end.saturating_sub(seg.start).min(value.len());
                let middle = pending.take().unwrap_or_default();
                let updated = format!("{}{}{}", &value[..from], middle, &value[to..]);
                write_text_new_in(dom, t, &updated, from..from + middle.len());
            }
            // One char, so inside the range it is covered whole.
            Piece::Glyph { element, .. } => {
                if let Some(text) = pending.take().filter(|text| !text.is_empty()) {
                    let t = dom.new_element(W::t());
                    dom.add_before_self(element, t);
                    write_text(dom, t, text);
                }
                dom.remove(element);
            }
        }
    }
    remove_comment_references(dom, &inside);
}

/// The comment references between the first and the last character of
/// `[start, end)`: a selection over them takes their mark along, and Word
/// then drops the comment (rule R1).
fn inner_comment_references(
    dom: &Dom,
    projection: &Projection,
    start: usize,
    end: usize,
) -> Vec<NodeId> {
    let node = |seg: &crate::inspect::Segment| match seg.piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut covered = projection
        .segments
        .iter()
        .filter(|seg| seg.end > start && seg.start < end);
    // One covered piece has nothing between its ends.
    let (Some(first), Some(last)) = (covered.next().map(node), covered.next_back().map(node))
    else {
        return Vec::new();
    };
    let Some(&paragraph) = dom.ancestors(first, Some(&W::p())).first() else {
        return Vec::new();
    };
    let order = dom.descendants(paragraph, None);
    let at = |n: NodeId| order.iter().position(|&o| o == n);
    let (Some(from), Some(to)) = (at(first), at(last)) else {
        return Vec::new();
    };
    dom.descendants(paragraph, Some(&W::name("commentReference")))
        .into_iter()
        .filter(|&r| at(r).is_some_and(|i| from < i && i < to))
        .collect()
}

/// The references of the comments wholly inside `[start, end)` that close
/// where it ends: the edit deletes all their text, as it does a comment
/// whose reference stands between its first and last character, but the
/// reference follows the last one, where [`inner_comment_references`]
/// does not look. A comment that opens where the range opens is not one:
/// replaced exactly, it keeps the new text.
fn closing_comment_references(
    dom: &Dom,
    projection: &Projection,
    start: usize,
    end: usize,
) -> Vec<NodeId> {
    let node = |seg: &crate::inspect::Segment| match seg.piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut covered = projection
        .segments
        .iter()
        .filter(|seg| seg.end > start && seg.start < end);
    let (Some(first), Some(last)) = (covered.next(), covered.next_back()) else {
        return Vec::new();
    };
    if last.end != end {
        return Vec::new();
    }
    let Some(&paragraph) = dom.ancestors(node(first), Some(&W::p())).first() else {
        return Vec::new();
    };
    let order = dom.descendants(paragraph, None);
    let at = |n: NodeId| order.iter().position(|&o| o == n);
    let (Some(from), Some(to)) = (at(node(first)), at(node(last))) else {
        return Vec::new();
    };
    if from >= to {
        return Vec::new();
    }
    let next = projection
        .segments
        .iter()
        .find(|seg| seg.start >= end)
        .and_then(|seg| at(node(seg)))
        .unwrap_or(order.len())
        .max(to + 1);
    let marker = |local: &str, id: &str, range: std::ops::Range<usize>| {
        order[range]
            .iter()
            .any(|&n| dom.name_is(n, &W::name(local)) && dom.attribute(n, &W::id()) == Some(id))
    };
    let opened: Vec<&str> = order[from + 1..to]
        .iter()
        .filter(|&&n| dom.name_is(n, &W::name("commentRangeStart")))
        .filter_map(|&n| dom.attribute(n, &W::id()))
        .filter(|id| marker("commentRangeEnd", id, to + 1..next))
        .collect();
    order[to + 1..next]
        .iter()
        .copied()
        .filter(|&n| {
            dom.name_is(n, &W::name("commentReference"))
                && dom
                    .attribute(n, &W::id())
                    .is_some_and(|id| opened.contains(&id))
        })
        .collect()
}

/// Remove comment references, with the run when it holds nothing else.
fn remove_comment_references(dom: &mut Dom, references: &[NodeId]) {
    for &reference in references {
        let run = dom.parent(reference).filter(|&r| {
            dom.name_is(r, &W::r())
                && dom
                    .elements(r, None)
                    .into_iter()
                    .all(|c| c == reference || dom.name_is(c, &W::r_pr()))
        });
        dom.remove(run.unwrap_or(reference));
    }
}

fn set_text(dom: &mut Dom, t: NodeId, value: &str) {
    dom.set_value(t, value);
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
}

/// A new run for text typed into the empty `paragraph`, as Word types it:
/// formatted as the paragraph mark (its `w:pPr/w:rPr`, without the mark's
/// own revisions or `w:oMath`, which only a mark takes). Not yet in the
/// tree.
pub(super) fn mark_run(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    let run = dom.new_element(W::r());
    if let Some(rpr) = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    {
        let rpr = tracked::clean_rpr(dom, rpr);
        if let Some(math) = dom.element(rpr, &W::name("oMath")) {
            dom.remove(math);
        }
        if dom.has_elements(rpr) {
            dom.add(run, rpr);
        }
    }
    run
}

/// `text` written into `run`, a [`mark_run`] already in the tree.
pub(super) fn write_mark_run_text(dom: &mut Dom, run: NodeId, text: &str) {
    let t = dom.new_element(W::t());
    dom.add(run, t);
    write_text(dom, t, text);
}

/// Text typed into an empty paragraph, placed edit by edit in reverse
/// [`sort_edits`] order, so that each edge keeps plan order and what the
/// paragraph holds (a field, a bookmark's markers) stays between them: the
/// text joins no range it was not typed in (#1303).
#[derive(Default)]
pub(super) struct EmptyPlacement {
    /// The end text placed last, which the next end text goes before.
    next_end: Option<NodeId>,
}

impl EmptyPlacement {
    /// Put `node` where its text was typed in `paragraph`: last for an
    /// insert at the end (before the end text already placed), else first,
    /// after the paragraph's properties (an insert at the start, a rewrite).
    pub(super) fn place(
        &mut self,
        dom: &mut Dom,
        paragraph: NodeId,
        node: NodeId,
        attach_before: bool,
    ) {
        if attach_before {
            match self.next_end {
                Some(next) => dom.add_before_self(next, node),
                None => dom.add(paragraph, node),
            }
            self.next_end = Some(node);
        } else if let Some(ppr) = dom.element(paragraph, &W::p_pr()) {
            dom.add_after_self(ppr, node);
        } else {
            dom.add_first(paragraph, node);
        }
    }
}

/// Sort a paragraph's edits for applying: by source range, then plan
/// order. In an `empty` paragraph every edit sits at its one point, and the
/// start edge's text comes before the end edge's (#1303).
fn sort_edits(edits: &mut [ScheduledEdit], empty: bool) {
    edits.sort_by_key(|e| (e.start, e.end, empty && e.attach_before, e.op));
}

/// Set the text of `t`, a `w:t` in a run, writing each `\t` as a `w:tab`
/// and each `\n` as a `w:br` of that run, with the text between them in
/// `w:t`s of its own; `t` itself goes when no text is left for it.
fn write_text(dom: &mut Dom, t: NodeId, value: &str) {
    write_text_new_in(dom, t, value, 0..value.len());
}

/// [`write_text`] for `value` whose new text is `new` alone: the text kept
/// around it stays as the source wrote it, a tab typed in `w:t` included,
/// so an edit never restates what it did not touch.
fn write_text_new_in(dom: &mut Dom, t: NodeId, value: &str, new: std::ops::Range<usize>) {
    let cuts: Vec<usize> = value[new.clone()]
        .match_indices(['\t', '\n'])
        .map(|(at, _)| new.start + at)
        .collect();
    let Some(&first) = cuts.first() else {
        set_text(dom, t, value);
        return;
    };
    set_text(dom, t, &value[..first]);
    let mut last = t;
    for (i, &cut) in cuts.iter().enumerate() {
        let glyph = if value.as_bytes()[cut] == b'\t' {
            "tab"
        } else {
            "br"
        };
        let element = dom.new_element(W::name(glyph));
        dom.add_after_self(last, element);
        last = element;
        let piece = &value[cut + 1..cuts.get(i + 1).copied().unwrap_or(value.len())];
        if !piece.is_empty() {
            let next = dom.new_element(W::t());
            dom.add_after_self(last, next);
            set_text(dom, next, piece);
            last = next;
        }
    }
    if first == 0 {
        dom.remove(t);
    }
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
        place_reference(dom, range_start, range_end, reference_run);
    } else {
        dom.add(paragraph, range_start);
        dom.add(paragraph, range_end);
        dom.add(paragraph, reference_run);
    }
}

/// Put a comment's reference run right after its end marker, or, when the
/// range starts outside the tracked insertion holding that marker, after
/// the insertion: rejecting it would take a reference inside it, and the
/// comment with it, though only part of the commented text was inserted.
/// A comment wholly on inserted text keeps its reference inside and goes
/// with the text, as in Word.
fn place_reference(dom: &mut Dom, range_start: NodeId, range_end: NodeId, reference_run: NodeId) {
    let mut after = range_end;
    for ancestor in dom.ancestors(range_end, None) {
        if dom.name_is(ancestor, &W::p()) {
            break;
        }
        if (dom.name_is(ancestor, &W::ins()) || dom.name_is(ancestor, &W::name("moveTo")))
            && !dom.ancestors(range_start, None).contains(&ancestor)
        {
            after = ancestor;
        }
    }
    dom.add_after_self(after, reference_run);
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
        place_reference(dom, range_start, range_end, reference_run);
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
    for run in runs_in_range(dom, paragraph, start, end) {
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

/// The runs holding projection range `[start, end)` of `paragraph`, in
/// order, split at the boundaries first; none for an empty range.
fn runs_in_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId> {
    if start >= end {
        return Vec::new();
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
    runs
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
        dom.add(r, t);
        write_text(dom, t, &spec.text);
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
pub(crate) const HIGHLIGHTS: &[&str] = &[
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn plan_json_roundtrips() {
        let json = r#"{"schema_version":1,"author":"A","operations":[
            {"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b"},
            {"kind":"insert","paragraph":{"id":"p1"},"after":"x","text":"y","comment":"c"},
            {"kind":"insert","paragraph":{"index":0},"position":"end","text":"z"},
            {"kind":"comment","paragraph":{"contains":"q"},"text":"note"},
            {"kind":"insert_paragraph","paragraph":{"starts_with":"s"},"runs":[{"text":"t","bold":true}]},
            {"kind":"delete_paragraph","paragraph":{"index":2}}]}"#;
        let plan = EditPlan::from_json(json).unwrap();
        assert_eq!(plan.operations.len(), 6);
        assert_eq!(plan.existing_revisions, ExistingRevisions::Refuse);
        let again = EditPlan::from_json(&plan.to_json()).unwrap();
        assert_eq!(again, plan);
    }

    #[test]
    fn plan_json_rejects_unknown_fields() {
        let bad = r#"{"schema_version":1,"author":"A","extra":1,"operations":[]}"#;
        assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
        let bad = r#"{"schema_version":1,"author":"A","operations":[{"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b","typo":1}]}"#;
        assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
    }

    #[test]
    fn plan_json_rejects_malformed_json() {
        let e = EditPlan::from_json("{").unwrap_err();
        assert!(e.to_string().starts_with("INVALID_PLAN: "));
    }

    #[test]
    fn plan_json_refuses_another_schema_version() {
        let other = SCHEMA_VERSION + 1;
        let json = format!(r#"{{"schema_version":{other},"author":"A","operations":[]}}"#);
        let e = EditPlan::from_json(&json).unwrap_err();
        assert_eq!(e.code, "UNSUPPORTED_SCHEMA");
        assert!(e.to_string().starts_with(&format!(
            "UNSUPPORTED_SCHEMA: schema_version {other} is not supported (expected {SCHEMA_VERSION})"
        )));
    }

    #[test]
    fn plan_to_json_is_pretty_and_a_fixed_point() {
        let pretty = r#"{
  "schema_version": 1,
  "source_sha256": "sha",
  "author": "Author",
  "date": "2024-01-01T00:00:00Z",
  "initials": "A",
  "existing_revisions": "refuse",
  "operations": []
}"#;
        assert_eq!(EditPlan::from_json(pretty).unwrap().to_json(), pretty);
    }

    #[test]
    fn plan_fields_name_every_field_a_plan_serializes() {
        let plan = EditPlan {
            schema_version: SCHEMA_VERSION,
            source_sha256: Some("0".repeat(64)),
            author: "A".into(),
            date: Some("2026-10-01T09:00:00Z".into()),
            initials: Some("A".into()),
            resolve_revisions: Some(ResolveRevisions::default()),
            removed_anchor_comments: RemovedAnchorComments::Keep,
            existing_revisions: ExistingRevisions::Keep,
            operations: Vec::new(),
            update_fields: true,
            rewrite_others_comments: true,
        };
        let value = serde_json::to_value(&plan).unwrap();
        let written: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(written, PLAN_FIELDS);
        let bad = r#"{"schema_version":1,"author":"A","operatons":[]}"#;
        assert_eq!(
            EditPlan::from_json(bad).unwrap_err().to_string(),
            format!(
                r#"INVALID_PLAN: unknown field "operatons"; did you mean "operations"? fields: {}"#,
                PLAN_FIELDS.join(", ")
            )
        );
    }

    #[test]
    fn nearest_names_the_closest_within_a_third() {
        assert_eq!(edit_distance("ocurrence", "occurrence"), 1);
        assert_eq!(edit_distance("", "find"), 4);
        assert_eq!(edit_distance("é", "e"), 1);
        let fields = ["comment_id", "text"];
        assert_eq!(nearest("txt", &fields, None), Some("text"));
        assert_eq!(nearest("comment", &fields, Some("text")), Some("text"));
        assert_eq!(nearest("urgency", &fields, None), None);
        // A meant name the operation cannot take is no suggestion.
        assert_eq!(nearest("anchor", &fields, Some("find")), None);
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "../tests/common/docx.rs"]
mod deeper_boundary_fixture;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
            near: None,
        }
    }

    // Baseline gaps: 113, 916, 933, 936, 942, 945, 958.
    /// A retry that holds clears the refusal its first side stored, so a
    /// later failure of the same operation names no revision that did not
    /// block it (#1308 review).
    #[test]
    fn a_retried_insertion_point_leaves_no_refusal() {
        let source = docx(concat!(
            r#"<w:p><w:r><w:t xml:space="preserve">Alpha </w:t></w:r>"#,
            r#"<w:moveTo w:id="9" w:author="Bob" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Beta</w:t></w:r></w:moveTo>"#,
            r#"<w:r><w:t>Gamma</w:t></w:r></w:p>"#,
        ));
        let plan = EditPlan::from_json(
            r#"{"schema_version":1,"author":"Carol","date":"2026-02-02T00:00:00Z","existing_revisions":"keep","operations":[]}"#,
        )
        .unwrap();
        let tx = Transaction::start(&source, &plan).unwrap();
        let projection = &tx.projections[0];
        assert_eq!(projection.text, "Alpha BetaGamma");
        let mut attach_before = true;
        assert_eq!(
            tx.check_insert_either(projection, 10, &mut attach_before),
            Ok(())
        );
        assert!(!attach_before);
        assert!(tx.revision_refusal.borrow().is_none());
    }

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
                reject_visible: false,
            },
        ] {
            assert_eq!(
                resolve_selected(&source, Some(&selection), RemovedAnchorComments::Remove).unwrap(),
                (None, ResolvedRevisions::default(), AnchorChanges::default())
            );
        }
        let selection = ResolveRevisions {
            accept: None,
            reject: Some(ChangeFilter::ids(["r1"])),
            reject_visible: false,
        };
        let (bytes, resolved, _) =
            resolve_selected(&source, Some(&selection), RemovedAnchorComments::Remove).unwrap();
        assert_eq!(
            resolved,
            ResolvedRevisions {
                accepted: vec![],
                rejected: vec!["r1".into()]
            }
        );
        let bytes = bytes.unwrap();
        let changes = crate::changes::list_changes(&bytes).unwrap();
        assert_eq!(
            changes.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["r2"]
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
        assert_eq!(revision_counts(b"invalid zip"), RevisionCounts::default());
        assert!(!comment_holds_kept_text(b"invalid zip", 0));
    }

    // 2259, 2312, 2339, 2503, 2703, 3173.
    #[test]
    fn resolution_empty_text_and_combined_comment_anchors() {
        let source = docx("<w:p/><w:p><w:r><w:t>tail</w:t></w:r></w:p>");
        for (ops, message) in [
            (
                r#"[{"kind":"comment","paragraph":"p0","find":"x","through":"p1","text":"n"}]"#,
                "find and through cannot be combined: through comments on whole paragraphs",
            ),
            (
                r#"[{"kind":"insert_paragraph","paragraph":"p0","runs":[]}]"#,
                "runs must carry text",
            ),
            (
                r#"[{"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":""},{"text":""}]}]"#,
                "runs must carry text",
            ),
            (
                r#"[{"kind":"insert_toc","paragraph":"p0","title":""}]"#,
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
            r#"[{"kind":"comment","paragraph":"p0","text":"empty anchor"}]"#,
            r#"[{"kind":"delete_paragraph","paragraph":"p0","comment":"empty deletion"}]"#,
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
                r#"[{"kind":"insert_image","paragraph":"header1","image_base64":""}]"#,
                "pictures can be inserted in the body only",
            ),
            (
                r#"[{"kind":"insert_toc","paragraph":"header1"}]"#,
                "a table of contents goes in the body",
            ),
            (
                r#"[{"kind":"list","paragraphs":["header1"],"kind_of_list":"bullet"}]"#,
                "paragraphs[0]: list is supported in the body only, not header1",
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
            Err(
                "the text crosses a field; edit the text on either side of it, or comment on it"
                    .into()
            )
        );
        // Text may go beside a tab, never beside a symbol.
        let (d, root) = dom(
            r#"<w:p><w:r><w:tab/></w:r><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r></w:p>"#,
        );
        let paragraph = d.element(root, &W::p()).unwrap();
        let glyphs = project_paragraph(&d, paragraph);
        for before in [false, true] {
            assert_eq!(tx.check_insert_position(&glyphs, 0, before), Ok(()));
            assert_eq!(
                tx.check_insert_position(&glyphs, 4, before),
                Err("the insertion point touches a symbol".into())
            );
        }
        assert_eq!(tx.check_range(&glyphs, 0, 1), Ok(()));
        assert_eq!(
            tx.check_range(&glyphs, 0, 4),
            Err("the text crosses a symbol".into())
        );
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
                        group: None,
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
        // A reply to a comment in the deleted paragraph is not one: the
        // tracked deletion keeps the reference in struck text (#1159).
        let reply = Resolved::Thread {
            op: ThreadOp::Reply {
                parent: 7,
                text: "n".into(),
            },
            stories: vec![0],
            anchor: Some(0),
        };
        conflicts(&source, vec![Resolved::DeleteParagraph { para: 0 }, reply]).unwrap();
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
        // Beside a glyph, new text joins its run: before it, or after it
        // when the glyph holds the char before the point.
        apply_text_edit(&mut d, &projection, 2, 2, "+", false);
        assert_eq!(project_paragraph(&d, p).text, "ab+\tcd");
        let projection = project_paragraph(&d, p);
        apply_text_edit(&mut d, &projection, 4, 4, "-", true);
        assert_eq!(project_paragraph(&d, p).text, "ab+\t-cd");
        // A range over the glyph removes it.
        let projection = project_paragraph(&d, p);
        apply_text_edit(&mut d, &projection, 1, 5, "X", false);
        assert_eq!(project_paragraph(&d, p).text, "aXcd");
        assert!(d.descendants(p, Some(&W::name("tab"))).is_empty());
        let texts: Vec<String> = d
            .descendants(p, Some(&W::t()))
            .iter()
            .map(|&t| d.value(t))
            .filter(|text| !text.is_empty())
            .collect();
        assert_eq!(texts, ["aX", "cd"]);
        // A tab or a break in new text is one of the run.
        let projection = project_paragraph(&d, p);
        apply_text_edit(&mut d, &projection, 2, 2, "\tY\n", true);
        assert_eq!(project_paragraph(&d, p).text, "aX\tY\ncd");
        assert_eq!(d.descendants(p, Some(&W::name("tab"))).len(), 1);
        assert_eq!(d.descendants(p, Some(&W::name("br"))).len(), 1);
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
            r#"[{"kind":"replace","paragraph":"p0","find":"middle","replacement":"","whole":true}]"#,
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
        assert_eq!(revision_counts(&source), RevisionCounts::default());
        let broken =
            fixture::replace_entry(&source, "word/header1.xml", b"<?xml version=\"1.0\"?>");
        assert_eq!(revision_counts(&broken), RevisionCounts::default());
        assert!(!comment_holds_kept_text(&broken, 7));
        assert_eq!(
            fixture::part_string(&broken, "word/header1.xml").as_deref(),
            Some("<?xml version=\"1.0\"?>")
        );
    }
}
