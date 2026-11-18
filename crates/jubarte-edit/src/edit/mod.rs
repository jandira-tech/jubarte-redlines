// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Guarded document editing for agents and tools.
//!
//! An [`EditPlan`](crate::edit::EditPlan) names exact, uniquely anchored operations against one
//! snapshot of a DOCX (optionally bound to its SHA-256). Every operation is
//! resolved against the untouched source first; any failure fails the whole
//! plan and nothing is written. On success the operations are applied to a
//! copy, comments are authored in that copy, and the existing comparer turns
//! source plus copy into a Word tracked-changes document. The caller receives
//! the clean copy, the redline and a per-operation [`EditReport`](crate::edit::EditReport).
//!
//! Text coordinates are the visible projection of [`crate::inspect`]: what
//! `jubarte read --json` prints is what a plan anchors to. Replacement text is
//! inserted into the run that held the anchor, so it inherits that run's
//! formatting; the plan never carries raw OOXML.
//!
//! The CLI subcommand is `jubarte edit FILE [--plan PLAN.json]`; the bindings
//! drive [`apply_plan`](crate::edit::apply_plan) /
//! [`apply_plan_json`](crate::edit::apply_plan_json) directly.
//! [`preview_plan`](crate::edit::preview_plan)
//! resolves every operation and returns a report without producing the
//! documents, for dry-run checks. Recording an edit from operation flags is
//! in [`flags`](crate::edit::flags) / [`record`](crate::edit::record).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use crate::changes::{Change, ChangeError, ChangeFilter, ChangeKind, MoveSide};

#[doc(hidden)]
pub use crate::inspect::SCHEMA_VERSION;
use crate::inspect::{BODY_STORY, Opened, Piece, Projection, project_paragraph, source_sha256};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XNamespace};

#[doc(hidden)]
pub mod anchor;
#[doc(hidden)]
pub mod anchor_error;
mod apply;
mod comments;
mod controls;
pub mod flags;
mod formatting;
mod hint;
mod images;
mod locating;
mod notes;
mod origin;
mod plan;
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
pub(crate) mod sections;
mod settlement;
mod skip;
mod structural;
mod tracked;
mod transaction;
mod visible;
mod watermark;
mod whole;

pub mod v2;
use crate::comment_settlement::AnchorChanges;
pub use crate::comment_settlement::{KeptComment, RemovedComment};
pub mod v2_apply;

/// `read --for-edit`: the working document, hashes and readiness (V2-10).
pub mod for_edit;
/// The one native-key policy (`w14:paraId`, revision `w:id`) shared by
/// generation, application and the expectations checker for paired
/// long IDs.
pub(crate) mod native_keys;
#[doc(hidden)]
pub mod source_refs;
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

use apply::*;
use comments::*;
use formatting::*;
use locating::*;
use plan::*;
use settlement::*;

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
    /// edited copy from jubarte's layout (`crate::fields::update_fields`)
    /// before the redline is compared.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub update_fields: bool,
    /// Accepted and ignored. Deleting, resolving or rewording another
    /// author's comment is allowed unless [`Others::Keep`] is set. The
    /// field stays so a plan that still names it parses.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rewrite_others_comments: bool,
    /// When `keep`, refuse to erase another author's change
    /// (`CHANGE_NOT_YOURS`) or comment (`COMMENT_NOT_YOURS`). The default
    /// `allow` permits both. Restore, strike, a comment and a reply stay
    /// allowed either way.
    #[serde(default, skip_serializing_if = "Others::is_allow")]
    pub others: Others,
}

/// Whether a plan may erase markup another author wrote.
///
/// Another author is anyone other than the plan's `author` (the CLI's
/// `--author`, `Jubarte` when that is left out). A change or comment that
/// names no author is not the plan's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Others {
    /// Accepting, rejecting, deleting and resolving another author's
    /// markup is allowed.
    #[default]
    Allow,
    /// Refuse those. Restore, strike, a comment and a reply stay allowed.
    Keep,
}

impl Others {
    /// The guard is on: do not erase another author's markup.
    fn keeps(self) -> bool {
        self == Self::Keep
    }

    fn is_allow(&self) -> bool {
        *self == Self::Allow
    }
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
    fn is_remove(&self) -> bool {
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
        /// The insertion, as `changes` lists it (`r366`, `header1.r3`): every
        /// insertion mark of a change of several (`r10+11`), a replacement's
        /// insertion only.
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
        /// The deletion, as `changes` lists it (`r12`, `header1.r3`): every
        /// deletion mark of a change of several (`r12+13`), a replacement's
        /// deletion only.
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
    /// output, and with `REDACTION_INCOMPLETE` when a part uses an encoding
    /// the scan does not certify. The report never repeats the text.
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
#[doc(hidden)]
pub struct Spacing {
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

/// A paragraph an anchor does occur in, named by the `ANCHOR_NOT_FOUND`
/// that refused it elsewhere (#1889): the id the views print and a
/// bounded excerpt of the paragraph's text. A suggestion to read and
/// re-address, never a retarget (#957).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorCandidate {
    /// Paragraph id as the views print it (`p53`, `header1.p1`).
    pub at: String,
    /// A grapheme-safe excerpt around the match, with ellipses at cuts.
    pub ctx: String,
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
    /// Paragraphs the anchor does occur in when the named paragraph does
    /// not hold it (#1889); empty on every other outcome.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<AnchorCandidate>,
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
    fn from_changes<'a>(changes: impl IntoIterator<Item = &'a Change>) -> Self {
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
    /// SHA-256 of the base the plan edited. It differs from `source_sha256`
    /// when existing revisions were flattened first, and when a repeated
    /// permission id was normalized on a revision-free source.
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
    /// Threads the clean copy drops that the redline still carries, in
    /// source comment ids. Editing mode publishes that copy, so its
    /// receipt names these. Suggesting mode keeps them on the redline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clean_removed_comments: Vec<RemovedComment>,
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
    #[cfg(feature = "render")]
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
            if !op.candidates.is_empty() {
                map.insert(
                    "candidates".into(),
                    serde_json::to_value(&op.candidates).expect("candidates serialize"),
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
        if !self.clean_removed_comments.is_empty() {
            summary["clean_removed_comments"] =
                serde_json::to_value(&self.clean_removed_comments).expect("serializes");
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

/// The prefix of a style's change id (`styles.r0`, as `changes` lists it):
/// a style's recorded formatting does not make the text tracked (#1885).
const STYLES_STORY: &str = "styles.";

/// What an `EXISTING_REVISIONS` refusal, and a guard that would erase
/// every tracked change, tells the caller to write instead.
const KEEP_REVISIONS: &str = concat!(
    r#"add "existing_revisions": "keep" to the plan "#,
    "(it keeps every tracked change, yours and theirs; accept and reject resolve them all first)"
);

/// Whether `source` still carries a tracked change of its text (body,
/// headers, footers, notes; a style's recorded formatting is not one) once
/// `plan`'s `resolve_revisions` has accepted and rejected what it selects.
/// A selection that does not resolve is the plan's error, reported when it
/// runs; here it leaves every change in place.
fn revisions_left(source: &[u8], plan: &EditPlan) -> bool {
    let resolved = resolve_selected(
        source,
        plan.resolve_revisions.as_ref(),
        plan.removed_anchor_comments,
        PlanSyntax::V1,
    )
    .ok()
    .and_then(|(bytes, _, _)| bytes);
    crate::changes::list_changes(resolved.as_deref().unwrap_or(source))
        .is_ok_and(|changes| changes.iter().any(|c| !c.id.starts_with(STYLES_STORY)))
}

/// The replies `plan` makes to a comment on a paragraph it deletes, each
/// as the parent's id and the deleted paragraph's id. Tracked, the reply
/// sits on the struck text; untracked, as `--editing-mode` publishes the
/// edit, the paragraph and the thread are gone and the reply lands nowhere.
/// A plan that does not resolve names none here: running it reports why.
pub fn replies_on_deleted_paragraphs(source: &[u8], plan: &EditPlan) -> Vec<(u32, Option<String>)> {
    let Ok(lowered) = visible::lower(source, plan, PlanSyntax::V1) else {
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

/// Refuse to erase another author's tracked change when the plan keeps
/// others' markup (`CHANGE_NOT_YOURS`). A style's recorded formatting is
/// not one (#1885). A visible reject settles nothing, so it is not one
/// either. `wholesale` is `"accept"` or `"reject"` when `existing_revisions`
/// would resolve every change the selection leaves.
///
/// A decision that removes content is also refused when a revision nested
/// inside it, or a comment anchor it holds, belongs to someone else
/// (`COMMENT_NOT_YOURS` for the comment). Comment-body revisions are listed
/// as `comments.rN` (#1706), so a comment-only file holds tracked changes.
/// `settle_comment_bodies` still loads that part when a settlement resolves
/// it and the listing has no other text change.
fn guard_keep_others(
    source: &[u8],
    author: &str,
    selection: Option<&ResolveRevisions>,
    wholesale: Option<&str>,
    settle_comment_bodies: bool,
    syntax: PlanSyntax,
) -> Result<(), EditError> {
    if selection.is_none() && wholesale.is_none() {
        return Ok(());
    }
    let listed =
        crate::changes::list_changes(source).map_err(|e| selection_error(e, syntax, ""))?;
    if let Some(selection) = selection {
        let accept = selected_ids(&listed, selection.accept.as_ref(), syntax)?;
        let reject = if selection.reject_visible {
            Vec::new()
        } else {
            selected_ids(&listed, selection.reject.as_ref(), syntax)?
        };
        let mut removals = Vec::new();
        for (id, accepting) in accept
            .iter()
            .map(|id| (id, true))
            .chain(reject.iter().map(|id| (id, false)))
        {
            let Some(change) = listed.iter().find(|c| &c.id == id) else {
                continue;
            };
            if change.id.starts_with(STYLES_STORY) {
                continue;
            }
            if !owns_change(change.author.as_deref(), author) {
                return Err(err("CHANGE_NOT_YOURS", None, change_not_yours(change)));
            }
            if !removes_content(change, accepting) {
                continue;
            }
            let nested = taken_along(&listed, change);
            if let Some(foreign) = nested
                .iter()
                .find(|inner| !owns_change(inner.author.as_deref(), author))
            {
                return Err(err("CHANGE_NOT_YOURS", None, change_not_yours(foreign)));
            }
            removals.push((change, accepting, nested));
        }
        guard_removed_comments(source, author, &removals)?;
    }
    let Some(policy) = wholesale else {
        return Ok(());
    };
    // Comment-body revisions are in `listed` as `comments.rN` (#1706).
    // Also count the comments part when this settlement resolves it and
    // the listing has not already got a text change (a style-only file).
    let comment_changes =
        if settle_comment_bodies || listed.iter().any(|c| !c.id.starts_with(STYLES_STORY)) {
            let opened = Opened::open(source).map_err(open_error)?;
            crate::changes::comment_changes_in(&opened.pkg)
        } else {
            Vec::new()
        };
    let mut authors = Vec::new();
    let mut nameless = false;
    for change in listed.iter().chain(&comment_changes) {
        note_foreign(change, author, &mut authors, &mut nameless);
    }
    if authors.is_empty() && !nameless {
        return Ok(());
    }
    Err(err(
        "CHANGE_NOT_YOURS",
        None,
        format!(
            r#""existing_revisions": "{policy}" would erase changes by {}; {KEEP_REVISIONS}"#,
            join_authors(&authors, nameless)
        ),
    ))
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

impl From<crate::comment_settlement::SettlementError> for EditError {
    fn from(error: crate::comment_settlement::SettlementError) -> Self {
        err(&error.code, None, error.message)
    }
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
    let lowered = visible::lower(source, plan, syntax)?;
    resolved(source, lowered.as_ref().unwrap_or(plan), syntax).map(|_| ())
}

fn resolved<'a>(
    source: &'a [u8],
    plan: &'a EditPlan,
    syntax: PlanSyntax,
) -> Result<Transaction<'a>, EditError> {
    check_update_fields(plan)?;
    let mut tx = Transaction::start_quoting(source, plan, syntax)?;
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
    #[cfg(feature = "render")]
    {
        report.fields = Vec::new();
    }
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
#[doc(hidden)]
pub enum Resolved {
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
        /// A `strike` of another author's inserted mark: under keep, our
        /// deletion joins their insertion in the mark.
        strike: bool,
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
        /// A `restore` of another author's deleted mark: the new paragraph
        /// keeps `like`'s mark formatting too, as the break it gives back.
        keep_mark: bool,
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
#[doc(hidden)]
pub enum ThreadOp {
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

/// A comment of another author a thread operation changes. With
/// [`Others::Keep`] it is refused (`COMMENT_NOT_YOURS`); otherwise the
/// report notes it (#1264, #1846, #1892).
#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct OthersComment {
    /// The comment changed.
    comment: u32,
    /// Its author; `""` when it names none.
    author: String,
    change: OthersChange,
}

#[derive(Clone, Copy, Debug)]
enum OthersChange {
    Reword,
    Delete,
    Resolve,
    Reopen,
    /// A reply deleted with the plan author's own comment, the root given.
    DeletedWith(u32),
}

impl OthersComment {
    /// The refusal: reply, or resolve your own thread when a delete would
    /// take their reply. `reply` and `resolve` are spelled for the plan's
    /// syntax. Accepting the rewrite flag does not change this.
    fn refusal(&self, reply: &str, resolve: &str) -> String {
        let (comment, author) = (self.comment, &self.author);
        if let OthersChange::DeletedWith(root) = self.change {
            let whose = if author.is_empty() {
                "a reply that names no author".to_string()
            } else {
                format!("a reply by {author:?}")
            };
            return format!(
                "deleting c{root} deletes c{comment}, {whose}; resolve c{root} instead ({resolve})"
            );
        }
        format!("{}; reply instead ({reply})", self.whose())
    }

    /// The report's note when the operation changed another author's comment.
    fn note(&self) -> String {
        let named = !self.author.is_empty();
        let what = match (self.change, named) {
            (OthersChange::Reword, true) => "their words were changed under their name".into(),
            (OthersChange::Reword, false) => "its words were changed".into(),
            (OthersChange::Delete, true) => "their comment was deleted".into(),
            (OthersChange::Resolve, true) => "their comment was resolved".into(),
            (OthersChange::Reopen, true) => "their comment was reopened".into(),
            (OthersChange::Delete, false) => "it was deleted".into(),
            (OthersChange::Resolve, false) => "it was resolved".into(),
            (OthersChange::Reopen, false) => "it was reopened".into(),
            (OthersChange::DeletedWith(root), true) => {
                format!("their reply was deleted with c{root}")
            }
            (OthersChange::DeletedWith(root), false) => format!("it was deleted with c{root}"),
        };
        format!("{}; {what}", self.whose())
    }

    /// `c5 is by "Bob"`, or `c5 names no author`.
    fn whose(&self) -> String {
        if self.author.is_empty() {
            format!("c{} names no author", self.comment)
        } else {
            format!("c{} is by {:?}", self.comment, self.author)
        }
    }
}

/// A resolved `insert_toc`.
#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct TocSpec {
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

#[doc(hidden)]
pub struct Transaction<'p> {
    pub plan: &'p EditPlan,
    pub revision_refusal: std::cell::RefCell<Option<refusal::RevisionRefusal>>,
    pub source_sha256: String,
    pub base: Vec<u8>,
    pub base_sha256: String,
    pub resolved_revisions: ResolvedRevisions,
    pub removed_comments: Vec<RemovedComment>,
    pub kept_comments: Vec<KeptComment>,
    /// The plan's date, else the time it is applied.
    pub date: String,
    pub initials: String,
    pub opened: Opened,
    /// The body first, then every header, footer and notes part.
    pub stories: Vec<StoryPart>,
    /// Every addressable paragraph: the body's, then each story's.
    pub paragraph_nodes: Vec<NodeId>,
    /// `(story, index in that story)` per entry of `paragraph_nodes`.
    pub paragraph_story: Vec<(usize, usize)>,
    pub projections: Vec<Projection>,
    /// Candidate lookup uses the fixed v1 normalization ladder against these
    /// immutable projections. Within a transaction the anchor therefore keys
    /// both the search text and its normalization, including empty results.
    pub anchor_candidate_cache: std::cell::RefCell<HashMap<String, (Vec<AnchorCandidate>, usize)>>,
    /// The source's paragraphs and where they went, when settling
    /// revisions rewrote the document.
    pub origin: Option<origin::Origin>,
    pub outcomes: Vec<EditOutcome>,
    pub resolved: Vec<(usize, Resolved)>,
    /// `(paragraph, start, end)` a `replace` names, by operation, where
    /// its diff wrote only some of it: other operations conflict with the
    /// whole range, its kept words too.
    pub claims: BTreeMap<usize, (usize, usize, usize)>,
    /// While an operation naming `cN.text` resolves: `N` and the span of
    /// its paragraph's projection the comment covers. Anchors are looked
    /// up there only.
    pub anchor_window: std::cell::Cell<Option<(u32, usize, usize)>>,
    /// The request syntax refusals quote (#1108).
    pub syntax: PlanSyntax,
    pub comments: Vec<(u32, String)>,
    /// `(comment id, operation)` of each commented paragraph deletion.
    pub deletion_comments: Vec<(u32, usize)>,
    /// Ids new comments take, last first, before `next_comment_id`.
    pub preset_comment_ids: Vec<u32>,
    /// One past the highest source comment id; u64 so it cannot overflow.
    pub next_comment_id: u64,
    pub comments_added: usize,
    /// Helper bookmarks around `whole` replacements, one per operation.
    pub whole_marks: Vec<whole::Mark>,
    /// `w:abstractNum` and `w:num` elements new lists add.
    pub new_numbering: (Vec<String>, Vec<String>),
    /// Style definitions the edits need (`ListParagraph`).
    pub needed_styles: std::collections::BTreeSet<String>,
    /// The source's comment part family; taken when the parts are written.
    pub family: Option<crate::comments::CommentFamily>,
    /// Comment replied to, by the reply's new id.
    pub reply_parents: BTreeMap<u32, u32>,
    /// Another author's comments a thread operation changes, by outcome
    /// index. The report notes each one. `rewrite_others_comments` does
    /// not decide that.
    pub rewritten: Vec<(usize, OthersComment)>,
    /// The plan's watermark, written into the default headers at finish.
    pub watermark: Option<watermark::Spec>,
    /// The body's content controls, in document order.
    pub controls: Vec<NodeId>,
    /// What `inspect` reports for each of `controls`.
    pub control_records: Vec<crate::inspect::ContentControl>,
    /// The footnotes story when `insert_footnote` added notes to it.
    pub notes_story: Option<usize>,
    /// The plan's settings, written into the settings part at finish.
    pub settings: Option<crate::settings::SettingsRequest>,
    /// The story indices `finish` last wrote, body included: the
    /// serialized-stories check compares each one's DOM against the bytes.
    pub written_stories: std::collections::BTreeSet<usize>,
    /// The ids `apply` gives the records it copies onto the retained
    /// fragments of a split run: none a source record uses, so the copy of
    /// another author's formatting history collides with nothing (#1656).
    pub revision_ids: RevisionCounter,
    /// Every revision id the source and its settled base use. An id the
    /// settling removed never names a new record in the same report.
    pub used_revision_ids: HashSet<u64>,
}

/// The body, or a header, footer or notes part, parsed into the plan's DOM.
#[doc(hidden)]
pub struct StoryPart {
    /// `body`, or the part's file stem (`header1`, `footnotes`).
    id: String,
    /// Package part name.
    part: String,
    document: NodeId,
    root: NodeId,
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

/// One output's revision ids (#1656). The edit's own records and the
/// copies a split makes of a source record come from one counter: above
/// every id the source uses, and inside Word's signed 32-bit range once it
/// runs out ([`crate::revision_processor::RevisionIds`]).
#[doc(hidden)]
pub struct RevisionCounter {
    ids: crate::revision_processor::RevisionIds,
    /// The id of the record each copy was made from, by the copy's id.
    copied_from: HashMap<u64, u64>,
    /// The ids of the records the edit writes: its own changes. A copy of
    /// a source record is not one.
    own: BTreeSet<u64>,
}

impl Default for RevisionCounter {
    fn default() -> Self {
        Self::new(HashSet::new())
    }
}

impl RevisionCounter {
    /// A counter that never hands out one of `used`.
    pub(super) fn new(used: HashSet<u64>) -> Self {
        Self {
            ids: crate::revision_processor::RevisionIds::from_used(used),
            copied_from: HashMap::new(),
            own: BTreeSet::new(),
        }
    }

    /// An id for a record the edit writes.
    pub(super) fn allocate(&mut self) -> u64 {
        let id = self.ids.allocate();
        self.own.insert(id);
        id
    }

    /// An id for a copy of the record that had id `of`: the edit's own
    /// change when it copies one of the edit's own records, a copy of a
    /// source record otherwise.
    fn copy(&mut self, of: Option<u64>) -> u64 {
        let id = if of.is_some_and(|id| self.own.contains(&id)) {
            self.allocate()
        } else {
            self.ids.allocate()
        };
        if let Some(of) = of {
            self.copied_from.insert(id, of);
        }
        id
    }

    /// The record `id` is a copy of, through copies of copies; `id` itself
    /// when it is no copy. A copy takes an id no record had, so the chain
    /// ends.
    fn original(&self, mut id: u64) -> u64 {
        while let Some(&of) = self.copied_from.get(&id) {
            id = of;
        }
        id
    }

    /// The ids of the records the edit wrote, copies of source records
    /// excluded.
    pub(super) fn own(&self) -> &BTreeSet<u64> {
        &self.own
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/tests.rs"]
mod tests;
#[cfg(test)]
thread_local! {
    static CANDIDATE_SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

// Reuse the repository's existing, entirely in-memory package fixture.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "../../../../tests/common/docx.rs"]
mod deeper_boundary_fixture;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/deeper_boundary_tests.rs"]
mod deeper_boundary_tests;
