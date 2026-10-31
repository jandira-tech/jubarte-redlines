// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Machine result envelope for the v2 routes (`edit apply`, `edit template`,
//! `review accept`, and friends).
//!
//! The envelope described in `jubarte-cli-proposal-b-revised.md` §11 is one
//! structure for every outcome: success and failure, with effects, with
//! artifacts, with terminal records. It is the typed contract that machine
//! callers (`--json`, `--jsonl`) consume and that the implementations of the
//! new v2 commands will eventually produce.
//!
//! Success requires a phase. Failures carry the last completed phase or
//! JSON null, with `not_applied` before publication. Post-commit reporting
//! failures retain the committed phase, actual effect and counts through
//! [`Envelope::with_failure_data`]. [`ResultData::validate`] checks effects
//! and predictions; [`Envelope::ok`] also rejects a missing phase. Error
//! builders never authorize retry when [`CommitState`] is unknown.
//!
//! JSONL starts with a typed [`StreamHeader`] (`"record": "header"`), then
//! zero or more typed progress records and exactly one terminal [`Envelope`].
//! The header carries schema version, command, engine version and source
//! SHA-256. Readers reject a missing/malformed header and multiple terminals;
//! a header-bearing stream without its terminal is incomplete. Progress uses
//! its own `ev` discriminator, so an operation's `status` is not terminal.
//! Exit status mapping follows proposal §12.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::inspect::source_sha256;

/// The wire schema version this module emits. Bumped together with any
/// backwards-incompatible change to the envelope.
pub(crate) const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// The terminal status of a v2 result.
///
/// The on-the-wire shape is a plain string (`"ok"`, `"refused"`, `"error"`)
/// so callers can pattern-match on it without descending into a nested
/// object. The typed error itself lives in the envelope's [`Envelope::error`]
/// field. All outcomes carry `data`; failures additionally carry `error`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The command completed; effects and artifacts describe what happened.
    Ok,
    /// The command refused the request (`NO_CHANGES_REQUESTED`,
    /// `AMBIGUOUS_ANCHOR`, `WORKING_BASE_CHANGED`, ...); no primary
    /// publication.
    Refused,
    /// The command could not complete because of an input, operational or
    /// I/O failure.
    Error,
}

/// The stage a result reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// Read-only inspection (no edit, no plan, no commit).
    Inspected,
    /// A plan artifact was generated and (optionally) saved; the document
    /// effect is always `not_applied`.
    Generated,
    /// A dry-run was validated; nothing was written.
    Validated,
    /// A primary publication occurred.
    Committed,
}

/// The primary document effect a result reports.
///
/// A `generated` or `validated` phase always has [`Effect::NotApplied`],
/// and a `committed` phase never does — see [`ResultData::validate`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// The primary bytes differ from the source.
    Changed,
    /// The primary bytes are an exact byte-identical copy of the source.
    Unchanged,
    /// The primary document was not published (generation or dry run).
    NotApplied,
}

/// The predicted primary effect of a dry run. Only valid with
/// [`Phase::Validated`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictedEffect {
    /// The dry run predicts a primary effect.
    Changed,
    /// The dry run predicts an unchanged primary.
    Unchanged,
}

/// Stable error codes the proposal names explicitly. Implementations are
/// free to use any string; these are the well-known ones.
pub mod codes {
    /// `--require-change` requested but the primary effect was zero.
    pub const NO_CHANGES_REQUESTED: &str = "NO_CHANGES_REQUESTED";
    /// A required postcondition did not hold after preparation.
    pub const POSTCONDITION_FAILED: &str = "POSTCONDITION_FAILED";
    /// The supplied `before_sha256` no longer matches the working baseline.
    pub const WORKING_BASE_CHANGED: &str = "WORKING_BASE_CHANGED";
    /// A literal anchor matched more than once without an `--occurrence`.
    pub const AMBIGUOUS_ANCHOR: &str = "AMBIGUOUS_ANCHOR";
    /// The source bound to the plan no longer matches the bytes on disk.
    pub const STALE_SOURCE: &str = "STALE_SOURCE";
    /// The plan named an operation kind the runtime does not support.
    pub const UNSUPPORTED_OPERATION: &str = "UNSUPPORTED_OPERATION";
    /// The JSON plan could not be parsed or validated.
    pub const INVALID_PLAN: &str = "INVALID_PLAN";
    /// A requested rendering/fidelity policy failed before publication.
    pub const RENDER_FIDELITY_FAILED: &str = "RENDER_FIDELITY_FAILED";
    /// A general rendering failure that is not fidelity-related.
    pub const RENDERING_FAILED: &str = "RENDERING_FAILED";
    /// A configured deadline elapsed before commit.
    pub const DEADLINE_EXCEEDED: &str = "DEADLINE_EXCEEDED";
    /// Invalid CLI usage or malformed request/schema.
    pub const USAGE: &str = "USAGE";
    /// Standalone check/validation findings meet the requested threshold.
    pub const CHECK_THRESHOLD_EXCEEDED: &str = "CHECK_THRESHOLD_EXCEEDED";
    /// Comparison differences when `--exit-code` was requested.
    pub const COMPARISON_DIFFERENCE: &str = "COMPARISON_DIFFERENCE";
}

/// Publication state known when an error was reported.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitState {
    /// Primary publication completed.
    Committed,
    /// Primary publication did not occur.
    NotCommitted,
    /// Primary publication cannot be determined.
    Unknown,
}

/// The role of an artifact: which contract slot it fills.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaRole {
    /// The primary `.docx` publication.
    Document,
    /// A generated `PlanV2` JSON file.
    Plan,
    /// A unified text/diff patch.
    Patch,
    /// A structured report (e.g. `report.json`).
    Report,
    /// A rendered PDF or PNG page.
    Render,
    /// Any other artifact (test scaffolding, asset, ...).
    Other,
}

/// A committed or uncommitted artifact recorded in the result.
///
/// `committed` is independent of the document effect: a `Plan` artifact can
/// be committed with a `generated` phase and a `not_applied` effect, while
/// a `Document` artifact may remain uncommitted when publication is
/// refused.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    /// Absolute path of the artifact on the destination filesystem.
    pub path: String,
    /// Role of the artifact (`document`, `plan`, ...).
    pub media_role: MediaRole,
    /// IANA media type (e.g. `application/pdf`).
    pub media_type: String,
    /// SHA-256 of the artifact's bytes, lowercase hex.
    pub sha256: String,
    /// Byte length of the artifact.
    pub byte_size: u64,
    /// Whether this artifact was actually written to its path.
    pub committed: bool,
}

impl Artifact {
    /// Build an artifact from `payload`; the digest is computed and
    /// `committed` defaults to `false`.
    pub fn new(
        path: impl Into<String>,
        role: MediaRole,
        media_type: impl Into<String>,
        payload: &[u8],
    ) -> Self {
        Self {
            path: path.into(),
            media_role: role,
            media_type: media_type.into(),
            sha256: source_sha256(payload),
            byte_size: payload.len() as u64,
            committed: false,
        }
    }

    /// Build an artifact and mark it committed.
    pub fn committed(
        path: impl Into<String>,
        role: MediaRole,
        media_type: impl Into<String>,
        payload: &[u8],
    ) -> Self {
        let mut artifact = Self::new(path, role, media_type, payload);
        artifact.committed = true;
        artifact
    }

    /// Override `committed`.
    #[must_use]
    pub fn with_committed(mut self, committed: bool) -> Self {
        self.committed = committed;
        self
    }
}

/// Bounded context around a refused match. `truncated` distinguishes
/// "complete context shown" from "context bounded by length or column".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// Text before the matched span (bounded).
    pub before: String,
    /// The matched span, verbatim.
    pub matched: String,
    /// Text after the matched span (bounded).
    pub after: String,
    /// `true` when `before`, `matched` or `after` was bounded; the actual
    /// context continues past the shown window.
    pub truncated: bool,
}

impl Context {
    /// Build a context with `truncated: false`; the caller sets the flag if
    /// the window was bounded.
    pub fn new(
        before: impl Into<String>,
        matched: impl Into<String>,
        after: impl Into<String>,
    ) -> Self {
        Self {
            before: before.into(),
            matched: matched.into(),
            after: after.into(),
            truncated: false,
        }
    }

    /// Override the truncation flag.
    #[must_use]
    pub fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }
}

/// A structured next action: an `argv` array (never a shell string) with
/// prerequisites and purpose.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NextAction {
    /// Argument array as the caller would pass it to `jubarte`. The first
    /// entry is the program name.
    pub argv: Vec<String>,
    /// Optional human-readable prerequisites (e.g. `source file readable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prerequisites: Option<Vec<String>>,
    /// Human-readable purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// The action erases another author's markup. Omitted when it does not.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub destructive: bool,
}

impl NextAction {
    /// Build a next action from a structured argv array.
    pub fn new(argv: Vec<String>) -> Self {
        Self {
            argv,
            prerequisites: None,
            purpose: None,
            destructive: false,
        }
    }

    /// Mark the action as erasing another author's markup.
    #[must_use]
    pub fn with_destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Set the purpose string.
    #[must_use]
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self {
        self.purpose = Some(purpose.into());
        self
    }

    /// Set the prerequisites list.
    #[must_use]
    pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self {
        self.prerequisites = Some(prerequisites);
        self
    }
}

/// An error block for a refused or failed result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Error {
    /// Stable error code (`NO_CHANGES_REQUESTED`, `AMBIGUOUS_ANCHOR`, ...).
    pub code: String,
    /// Operation pointer/id (e.g. `op-2`, `/operations/1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// SHA-256 of the source the request bound to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_digest: Option<String>,
    /// Resolved target pointer (paragraph id or similar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Number of matches found (1 on success, >1 on ambiguity, 0 on miss).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matches: Option<usize>,
    /// Bounded actual context around the failure point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    /// Whitespace-only diagnostic occurrences. They never authorize selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub near: Option<crate::edit::NearMatches>,
    /// Human-readable detail.
    pub message: String,
    /// Structured argv next actions; never shell strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_actions: Vec<NextAction>,
    /// `retryable` hint: `Some(true)` when a retry with the same inputs is
    /// safe, `Some(false)` when it is not, `None` when the classification
    /// is unknown. Implementations are free to omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// Known primary publication state; unknown never authorizes retry.
    pub commit_state: CommitState,
    /// The bounded expectation report behind this failure, when the error
    /// is an expectation verdict (`EXPECTATION_FAILED`, ...). Kept beside
    /// the error so a refused envelope still discloses which checks ran,
    /// refused or stayed unassessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expectations: Option<crate::expect::ExpectationReport>,
}

impl Error {
    /// Set the known publication state.
    #[must_use]
    pub fn with_commit_state(mut self, state: CommitState) -> Self {
        self.commit_state = state;
        self.normalize_retryable();
        self
    }

    fn normalize_retryable(&mut self) {
        if self.commit_state == CommitState::Unknown && self.retryable == Some(true) {
            self.retryable = Some(false);
        }
    }

    /// Build an error with unknown publication state and no retry hint.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            operation: None,
            source_digest: None,
            target: None,
            matches: None,
            context: None,
            near: None,
            message: String::new(),
            next_actions: Vec::new(),
            retryable: None,
            commit_state: CommitState::Unknown,
            expectations: None,
        }
    }

    /// Set the message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the operation pointer/id.
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Bind the source digest.
    #[must_use]
    pub fn with_source(mut self, source: Source) -> Self {
        self.source_digest = Some(source.sha256);
        self
    }

    /// Override the source digest directly.
    #[must_use]
    pub fn with_source_digest(mut self, digest: impl Into<String>) -> Self {
        self.source_digest = Some(digest.into());
        self
    }

    /// Set the resolved target pointer.
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set the matches count.
    #[must_use]
    pub fn with_matches(mut self, matches: usize) -> Self {
        self.matches = Some(matches);
        self
    }

    /// Set the bounded context.
    #[must_use]
    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    /// Set the bounded context (in-place variant for chaining after
    /// mutating other fields).
    pub fn set_context(&mut self, context: Context) {
        self.context = Some(context);
    }

    /// Push a structured argv next action.
    pub fn push_next_action(&mut self, action: NextAction) {
        self.next_actions.push(action);
    }

    /// Set the retryable hint. Unknown publication coerces `true` to `false`.
    #[must_use]
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = Some(retryable);
        self.normalize_retryable();
        self
    }
}

/// A reference to the bound source. Only `sha256` is mandatory in the
/// envelope; `path` and `byte_size` may appear when they help the caller.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// SHA-256 of the source bytes (lowercase hex).
    pub sha256: String,
    /// Absolute path of the source, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Source byte size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
}

impl Source {
    /// Build a `Source` from raw bytes: the digest is computed and the
    /// byte size recorded; no path is set.
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            sha256: source_sha256(bytes),
            path: None,
            byte_size: Some(bytes.len() as u64),
        }
    }

    /// Build a `Source` from a precomputed digest.
    pub fn from_digest(digest: impl Into<String>) -> Self {
        Self {
            sha256: digest.into(),
            path: None,
            byte_size: None,
        }
    }

    /// Set the path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// The runtime that produced the result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Runtime {
    /// `native`, `python`, `wasm`, ...
    pub name: String,
    /// Optional build identity (e.g. a git sha or release tag).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

impl Runtime {
    /// The native CLI runtime.
    pub fn native() -> Self {
        Self {
            name: "native".to_string(),
            build: None,
        }
    }
}

/// Verification block for an `ok` envelope. Mirrors the proposal's
/// `data.verification`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// `passed`, `failed`, or `skipped` (operation postconditions).
    pub operation_postconditions: String,
    /// `passed`, `failed`, or `skipped` (package/revision/comment integrity).
    pub integrity: String,
    /// `passed`, `failed`, `not_requested`, or `unresolved` (typed expectations).
    pub expectations: String,
    /// `assessed`, `not_assessed`, `unresolved` (natural-language task semantics).
    pub task_semantics: String,
    /// Input findings, separately from candidate findings; absent when unassessed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_findings: Option<Vec<crate::validate::Finding>>,
    /// Findings newly introduced by the edit, including nonblocking leads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced_findings: Option<Vec<crate::validate::Finding>>,
    /// Findings still present in the checked candidates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unresolved_findings: Option<Vec<crate::validate::Finding>>,
}

impl Default for Verification {
    fn default() -> Self {
        Self {
            operation_postconditions: "skipped".to_string(),
            integrity: "skipped".to_string(),
            expectations: "not_requested".to_string(),
            task_semantics: "not_assessed".to_string(),
            baseline_findings: None,
            introduced_findings: None,
            unresolved_findings: None,
        }
    }
}

/// Counts of operations, revisions and comments. The four categories the
/// proposal names (operations, logical revisions, revision XML entries,
/// comments) stay distinct in the schema; aggregate counts are deliberately
/// absent so a reader cannot accidentally conflate them.
///
/// Every field is optional in the JSON; unset zero fields are skipped.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    // Operations
    /// Total operations the request named.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub requested_operations: usize,
    /// Operations whose effect on the body was a no-op (e.g. a rewrite
    /// row whose `text` matched the verified baseline exactly).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub unchanged_operations: usize,
    /// Body operations that applied a real edit.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub body_operations_applied: usize,
    /// Review operations that applied (accept, reject, comment, ...).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub review_operations_applied: usize,

    // Revisions (logical vs. XML entry)
    /// Existing revisions accepted by the request.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub revisions_accepted: usize,
    /// Existing revisions rejected by the request.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub revisions_rejected: usize,
    /// Logical revision decisions: a move is one logical decision even
    /// though it spans two XML entries.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub logical_revisions: usize,
    /// Underlying XML `<w:ins>` / `<w:del>` carrier elements.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub revision_xml_entries: usize,

    // Comments
    /// Root comment threads added or modified by the request.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub comment_threads: usize,
    /// Replies added by the request.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub comment_replies: usize,
    /// Sum of root comments added plus replies added; convenient for a
    /// reader that does not care about the breakdown.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub comments_added: usize,
}

impl Counts {
    /// Sum of `body_operations_applied` + `review_operations_applied`.
    /// Distinct from `requested_operations` (the latter includes
    /// unchanged and skipped operations).
    #[must_use]
    pub fn total_operations(&self) -> usize {
        self.body_operations_applied + self.review_operations_applied
    }
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

/// Result data for success or failure: phase (null before any completed
/// stage), effect, optional predicted effect, counts and verification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResultData {
    /// Last completed stage; null when no stage completed. Success requires it.
    pub phase: Option<Phase>,
    /// The primary document effect.
    pub effect: Effect,
    /// Predicted effect for dry runs; only valid with [`Phase::Validated`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicted_effect: Option<PredictedEffect>,
    /// Counts of operations, revisions and comments.
    pub counts: Counts,
    /// Threads removed by revision settlement, in the source's ids.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_comments: Vec<crate::edit::RemovedComment>,
    /// Threads rescued on empty anchors by an edit plan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kept_comments: Vec<crate::edit::KeptComment>,
    /// Verification block.
    pub verification: Verification,
    /// Generator-only coverage and bounded working readiness; never plan authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<serde_json::Value>,
    /// The bounded expectation report when `--expect` was evaluated: the
    /// receipt (path, digest, schema version, checks run/refused/
    /// unsupported, targets, text view, declared provenance) plus the
    /// per-check outcomes. Absent when no expectations were supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expectations: Option<crate::expect::ExpectationReport>,
}

/// Invalid combinations of (phase, effect, predicted_effect).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultDataError {
    /// Success or an actual/predicted effect requires a completed phase.
    MissingPhase,
    /// Failure data can only be attached to a failure envelope.
    FailureRequired,
    /// A pre-commit phase was paired with a `changed` or `unchanged` effect.
    NotAppliedRequired {
        /// The phase that requires `not_applied`.
        phase: Phase,
        /// The effect that was supplied.
        effect: Effect,
    },
    /// A committed phase was paired with a `not_applied` effect: only
    /// actual primary publication reports phase `committed`, and it
    /// reports an actual effect.
    EffectRequired {
        /// The phase that requires an actual effect.
        phase: Phase,
        /// The effect that was supplied.
        effect: Effect,
    },
    /// `predicted_effect` was set without a `validated` phase.
    PredictedWithoutValidation {
        /// The phase that was supplied.
        phase: Phase,
    },
}

impl std::fmt::Display for ResultDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPhase => write!(f, "a completed phase is required"),
            Self::FailureRequired => write!(f, "failure data requires a refused or error status"),
            Self::NotAppliedRequired { phase, effect } => write!(
                f,
                "phase {phase:?} requires effect NotApplied, got {effect:?}"
            ),
            Self::EffectRequired { phase, effect } => write!(
                f,
                "phase {phase:?} requires effect Changed or Unchanged, got {effect:?}"
            ),
            Self::PredictedWithoutValidation { phase } => write!(
                f,
                "predicted_effect is only valid with phase Validated, got {phase:?}"
            ),
        }
    }
}

impl std::error::Error for ResultDataError {}

impl ResultData {
    /// Build pre-commit failure data with the last completed phase.
    pub fn failure(phase: Option<Phase>) -> Self {
        Self {
            phase,
            effect: Effect::NotApplied,
            predicted_effect: None,
            counts: Counts::default(),
            removed_comments: Vec::new(),
            kept_comments: Vec::new(),
            verification: Verification::default(),
            generation: None,
            expectations: None,
        }
    }

    /// Build a result data block. Pre-commit phases require `not_applied`;
    /// the `predicted_effect` is set with
    /// [`ResultData::set_predicted`] and only allowed for `validated`.
    /// [`Envelope::ok`] enforces both rules through
    /// [`ResultData::validate`].
    pub fn new(phase: Phase, effect: Effect) -> Self {
        Self {
            phase: Some(phase),
            effect,
            predicted_effect: None,
            counts: Counts::default(),
            removed_comments: Vec::new(),
            kept_comments: Vec::new(),
            verification: Verification::default(),
            generation: None,
            expectations: None,
        }
    }

    /// Set the predicted effect (dry runs only). Mutates in place so the
    /// builder can be called several times before validation.
    pub fn set_predicted(&mut self, predicted: PredictedEffect) {
        self.predicted_effect = Some(predicted);
    }

    /// Validate the (phase, effect, predicted_effect) combination. Returns
    /// `Ok(())` for a legal combination, `Err(ResultDataError)` otherwise.
    pub fn validate(&self) -> Result<(), ResultDataError> {
        match self.phase {
            Some(phase) => {
                if phase == Phase::Committed {
                    // Only actual primary publication reports phase
                    // `committed`, and it reports an actual effect.
                    if self.effect == Effect::NotApplied {
                        return Err(ResultDataError::EffectRequired {
                            phase,
                            effect: self.effect,
                        });
                    }
                } else if self.effect != Effect::NotApplied {
                    return Err(ResultDataError::NotAppliedRequired {
                        phase,
                        effect: self.effect,
                    });
                }
                if self.predicted_effect.is_some() && phase != Phase::Validated {
                    return Err(ResultDataError::PredictedWithoutValidation { phase });
                }
            }
            None if self.effect != Effect::NotApplied || self.predicted_effect.is_some() => {
                return Err(ResultDataError::MissingPhase);
            }
            None => {}
        }
        Ok(())
    }
}

/// The terminal record of a v2 command. The envelope is one of three
/// shapes, distinguished by `status`:
///
/// - `ok`: `data` carries the phase/effect/counts/verification; `error` is
///   absent; `artifacts` lists committed (and any uncommitted) outputs.
/// - `refused`: `error` carries the refusal; `data` records pre-commit state.
/// - `error`: `error` carries the failure; `data` retains the last completed
///   phase, including committed publication state after a reporting failure.
///
/// The Rust identifier is `Envelope` to avoid shadowing
/// `std::result::Result` in function signatures.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Wire schema version; equals `ENVELOPE_SCHEMA_VERSION`.
    pub schema_version: u32,
    /// Command identifier (`edit apply`, `edit template`, ...).
    pub command: String,
    /// Engine crate version.
    pub engine_version: String,
    /// Runtime metadata.
    pub runtime: Runtime,
    /// Terminal status (`ok` | `refused` | `error`).
    pub status: Status,
    /// Source the request bound to.
    pub source: Source,
    /// Result data, including the last completed phase on failures.
    /// Only the scaffold constructor leaves it absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<ResultData>,
    /// Committed and uncommitted artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<Artifact>,
    /// Non-fatal warnings collected during preparation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Error block; present on `refused` and `error`, absent on `ok`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
    /// Effective option values (resolved flags, defaults, configuration).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub effective_options: BTreeMap<String, serde_json::Value>,
    /// Continuation cursor for paginated reads; `None` here means
    /// exhausted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<serde_json::Value>,
}

impl Envelope {
    /// Attach validated phase/effect/counts to a failure envelope. A committed
    /// phase records known publication; earlier phases preserve the error's
    /// commit state, including unknown. Refusals cannot report publication.
    pub fn with_failure_data(mut self, data: ResultData) -> Result<Self, ResultDataError> {
        if self.status == Status::Ok {
            return Err(ResultDataError::FailureRequired);
        }
        data.validate()?;
        // Refusals never publish a primary document.
        if self.status == Status::Refused && data.phase == Some(Phase::Committed) {
            return Err(ResultDataError::FailureRequired);
        }
        if let Some(error) = &mut self.error {
            if data.phase == Some(Phase::Committed) {
                error.commit_state = CommitState::Committed;
            }
            error.normalize_retryable();
        }
        self.data = Some(data);
        Ok(self)
    }

    /// Build a placeholder envelope with `status: ok` and no data; used by
    /// tests and scaffold code. Production callers should use
    /// [`Envelope::ok`] (with data) or [`Envelope::refused`] /
    /// [`Envelope::new_error`].
    pub fn new(command: impl Into<String>, source: Source, runtime: Runtime) -> Self {
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime,
            status: Status::Ok,
            source,
            data: None,
            artifacts: Vec::new(),
            warnings: Vec::new(),
            error: None,
            effective_options: BTreeMap::new(),
            continuation: None,
        }
    }

    /// Build a terminal envelope with `status: ok` and the supplied `data`.
    ///
    /// Returns [`ResultDataError`] when `data` pairs its phase with an
    /// effect or predicted effect the envelope rules forbid (see
    /// [`ResultData::validate`]), so no contradictory `ok` envelope can be
    /// built.
    pub fn ok(
        command: impl Into<String>,
        source: Source,
        data: ResultData,
        runtime: Runtime,
    ) -> Result<Self, ResultDataError> {
        data.validate()?;
        if data.phase.is_none() {
            return Err(ResultDataError::MissingPhase);
        }
        Ok(Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime,
            status: Status::Ok,
            source,
            data: Some(data),
            artifacts: Vec::new(),
            warnings: Vec::new(),
            error: None,
            effective_options: BTreeMap::new(),
            continuation: None,
        })
    }

    /// Build a terminal envelope with `status: refused` and a typed
    /// refusal. No stage is assumed complete; use [`Self::with_failure_data`]
    /// to report a known last completed phase. Refusals are not committed.
    pub fn refused(command: impl Into<String>, source: Source, error: Error) -> Self {
        let error = error.with_commit_state(CommitState::NotCommitted);
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime: Runtime::native(),
            status: Status::Refused,
            source,
            data: Some(ResultData::failure(None)),
            artifacts: Vec::new(),
            warnings: Vec::new(),
            error: Some(error),
            effective_options: BTreeMap::new(),
            continuation: None,
        }
    }

    /// Build a terminal envelope with `status: error` and a typed
    /// failure. No stage is assumed complete; use [`Self::with_failure_data`]
    /// to retain completed phases, including publication after reporting errors.
    pub fn new_error(command: impl Into<String>, source: Source, mut error: Error) -> Self {
        error.normalize_retryable();
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime: Runtime::native(),
            status: Status::Error,
            source,
            data: Some(ResultData::failure(None)),
            artifacts: Vec::new(),
            warnings: Vec::new(),
            error: Some(error),
            effective_options: BTreeMap::new(),
            continuation: None,
        }
    }

    /// Append an artifact to the envelope.
    pub fn push_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Append a warning to the envelope.
    pub fn push_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Set an effective option.
    pub fn set_effective_option(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.effective_options.insert(key.into(), value);
    }
}

/// First JSONL record. The stable discriminator is `"record": "header"`;
/// the source block contains the bound source's SHA-256.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "record", rename = "header")]
pub struct StreamHeader {
    /// Envelope schema version.
    pub schema_version: u32,
    /// Command identifier.
    pub command: String,
    /// Engine crate version.
    pub engine_version: String,
    /// Bound source identity.
    pub source: Source,
}

impl From<&Envelope> for StreamHeader {
    fn from(envelope: &Envelope) -> Self {
        Self {
            schema_version: envelope.schema_version,
            command: envelope.command.clone(),
            engine_version: envelope.engine_version.clone(),
            source: envelope.source.clone(),
        }
    }
}

/// Progress event for JSONL streams: a free-form JSON value so callers can
/// attach any `ev`/`load`/`op`/`save` line they want. [`read_jsonl_stream`]
/// requires every progress record to carry a string `ev` discriminator.
/// The terminal record is a typed [`Envelope`]; see
/// [`jsonl_terminal_line`] and [`write_jsonl_stream`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProgressEvent {
    /// Free-form JSON.
    value: serde_json::Value,
}

impl ProgressEvent {
    /// Build a progress event from a JSON value.
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }

    /// Borrow the underlying JSON value.
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }
}

impl From<serde_json::Value> for ProgressEvent {
    fn from(value: serde_json::Value) -> Self {
        Self::new(value)
    }
}

impl From<ProgressEvent> for serde_json::Value {
    fn from(event: ProgressEvent) -> Self {
        event.value
    }
}

/// Serialize `terminal` as a single JSONL line (no trailing newline).
pub fn jsonl_terminal_line(terminal: &Envelope) -> String {
    serde_json::to_string(terminal).expect("Envelope serializes")
}

/// Serialize a typed header, `events`, then exactly one terminal result.
/// Each record occupies its own line; the terminal is always last.
pub fn write_jsonl_stream(events: &[ProgressEvent], terminal: &Envelope) -> String {
    let mut out =
        serde_json::to_string(&StreamHeader::from(terminal)).expect("StreamHeader serializes");
    out.push('\n');
    for event in events {
        out.push_str(&serde_json::to_string(event.value()).expect("progress serializes"));
        out.push('\n');
    }
    out.push_str(&jsonl_terminal_line(terminal));
    out.push('\n');
    out
}

/// A parsed JSONL stream: the progress events and the terminal record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedStream {
    /// Required first record with schema, engine and source identity.
    pub header: StreamHeader,
    /// Progress events in order.
    pub events: Vec<ProgressEvent>,
    /// The terminal record.
    pub terminal: Envelope,
}

/// Why a JSONL stream is malformed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    /// The stream did not start with a typed header.
    MissingHeader,
    /// The header discriminator was present but its fields were malformed.
    HeaderParse {
        /// 1-based line number.
        line: usize,
        /// Parse error.
        message: String,
    },
    /// Invalid JSON, a duplicate header, or a record after termination.
    InvalidLine {
        /// 1-based line number.
        line: usize,
        /// The line content.
        content: String,
    },
    /// The stream ended without a terminal record.
    MissingTerminal,
    /// The stream carries more than one terminal record.
    MultipleTerminals,
    /// A terminal record could not be parsed.
    TerminalParse {
        /// 1-based line number.
        line: usize,
        /// Parse error.
        message: String,
    },
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "stream has no initial header record"),
            Self::HeaderParse { line, message } => {
                write!(f, "line {line}: header record parse error: {message}")
            }
            Self::InvalidLine { line, content } => {
                write!(f, "line {line}: invalid JSONL line: {content}")
            }
            Self::MissingTerminal => write!(f, "stream has no terminal result line"),
            Self::MultipleTerminals => write!(f, "stream carries more than one terminal result"),
            Self::TerminalParse { line, message } => {
                write!(f, "line {line}: terminal record parse error: {message}")
            }
        }
    }
}

impl std::error::Error for StreamError {}

/// Require a typed header first and exactly one terminal result last.
/// Progress uses `ev`; its per-operation `status` is not a terminal
/// marker. A progress record must carry a string `ev` discriminator:
/// JSONL emits typed records, so an untyped line between the header and
/// the terminal is malformed. Empty lines are ignored. Missing
/// termination is incomplete; missing or malformed headers, duplicate
/// terminals and records after termination are malformed.
pub fn read_jsonl_stream(input: &str) -> std::result::Result<ParsedStream, StreamError> {
    let mut header: Option<StreamHeader> = None;
    let mut events: Vec<ProgressEvent> = Vec::new();
    let mut terminal: Option<Envelope> = None;
    for (index, raw) in input.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|_| StreamError::InvalidLine {
                line,
                content: trimmed.to_string(),
            })?;
        if header.is_none() {
            if value.get("record").and_then(serde_json::Value::as_str) != Some("header") {
                return Err(StreamError::MissingHeader);
            }
            header = Some(
                serde_json::from_value(value).map_err(|e| StreamError::HeaderParse {
                    line,
                    message: e.to_string(),
                })?,
            );
            continue;
        }
        if value.get("record").and_then(serde_json::Value::as_str) == Some("header") {
            return Err(StreamError::InvalidLine {
                line,
                content: trimmed.to_string(),
            });
        }
        if value.get("status").is_some() && value.get("ev").is_none() {
            let record: Envelope =
                serde_json::from_value(value).map_err(|e| StreamError::TerminalParse {
                    line,
                    message: e.to_string(),
                })?;
            if terminal.is_some() {
                return Err(StreamError::MultipleTerminals);
            }
            terminal = Some(record);
        } else {
            if terminal.is_some() {
                return Err(StreamError::InvalidLine {
                    line,
                    content: trimmed.to_string(),
                });
            }
            if value
                .get("ev")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                return Err(StreamError::InvalidLine {
                    line,
                    content: trimmed.to_string(),
                });
            }
            events.push(ProgressEvent::new(value));
        }
    }
    let header = header.ok_or(StreamError::MissingHeader)?;
    terminal
        .map(|terminal| ParsedStream {
            header,
            events,
            terminal,
        })
        .ok_or(StreamError::MissingTerminal)
}

/// Exit status values per the proposal §12 table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExitStatus(pub u8);

impl ExitStatus {
    /// Command completed.
    pub const SUCCESS: Self = Self(0);
    /// Input, operational or I/O failure.
    pub const INPUT_FAILURE: Self = Self(1);
    /// Standalone check/validation findings meet the requested threshold.
    pub const CHECK_FINDINGS: Self = Self(2);
    /// Refused stale, ambiguous, unsafe, unsupported request, failed
    /// operation/expectation invariant, or unmet require-change condition.
    pub const REFUSED: Self = Self(3);
    /// Requested rendering/fidelity policy failed before publication.
    pub const RENDER_FAILURE: Self = Self(4);
    /// Comparison differences when `--exit-code` was requested.
    pub const COMPARISON_DIFFERENCE: Self = Self(5);
    /// Invalid CLI usage or malformed request/schema.
    pub const USAGE: Self = Self(64);
    /// Deadline exceeded before commit.
    pub const DEADLINE_EXCEEDED: Self = Self(124);
}

impl From<ExitStatus> for u8 {
    fn from(value: ExitStatus) -> Self {
        value.0
    }
}

/// Map a [`Status`] and the optional [`Error`] code to the proposal §12
/// exit code.
///
/// The §12 table is keyed by the code's meaning, not by the envelope
/// status: a dedicated code decides whatever status the envelope carries
/// (`USAGE`→64, `RENDER_FIDELITY_FAILED`→4, `DEADLINE_EXCEEDED`→124,
/// `CHECK_THRESHOLD_EXCEEDED`→2, `COMPARISON_DIFFERENCE`→5). Only
/// unclassified codes fall back to the status: `Ok`→0, `Refused`→3,
/// `Error`→1.
pub fn exit_status_for(status: Status, error: Option<&Error>) -> ExitStatus {
    if let Some(code) = error.map(|e| e.code.as_str()) {
        match code {
            codes::CHECK_THRESHOLD_EXCEEDED => return ExitStatus::CHECK_FINDINGS,
            codes::COMPARISON_DIFFERENCE => return ExitStatus::COMPARISON_DIFFERENCE,
            codes::RENDER_FIDELITY_FAILED => return ExitStatus::RENDER_FAILURE,
            codes::USAGE => return ExitStatus::USAGE,
            codes::DEADLINE_EXCEEDED => return ExitStatus::DEADLINE_EXCEEDED,
            _ => {}
        }
    }
    match status {
        Status::Ok => ExitStatus::SUCCESS,
        Status::Refused => ExitStatus::REFUSED,
        Status::Error => ExitStatus::INPUT_FAILURE,
    }
}

/// Map an [`Envelope`] to the proposal §12 exit code.
pub fn exit_status_for_envelope(envelope: &Envelope) -> ExitStatus {
    exit_status_for(envelope.status, envelope.error.as_ref())
}
#[cfg(test)]
mod tests {
    //! Inline tests covering the type-level invariants the rest of the
    //! crate relies on. The full envelope/JSONL contract is covered by the
    //! integration suite in `tests/result_envelope.rs`.

    use super::*;

    fn fixture() -> Vec<u8> {
        b"fixture-bytes".to_vec()
    }

    #[test]
    fn not_applied_required_for_generated_phase() {
        let data = ResultData::new(Phase::Generated, Effect::NotApplied);
        assert!(data.validate().is_ok());
        let data = ResultData::new(Phase::Generated, Effect::Changed);
        assert!(matches!(
            data.validate(),
            Err(ResultDataError::NotAppliedRequired { .. })
        ));
    }

    #[test]
    fn not_applied_required_for_validated_phase() {
        let data = ResultData::new(Phase::Validated, Effect::NotApplied);
        assert!(data.validate().is_ok());
        let data = ResultData::new(Phase::Validated, Effect::Changed);
        assert!(data.validate().is_err());
    }

    #[test]
    fn committed_phase_may_carry_changed_or_unchanged() {
        for effect in [Effect::Changed, Effect::Unchanged] {
            let data = ResultData::new(Phase::Committed, effect);
            assert!(data.validate().is_ok());
        }
    }

    #[test]
    fn committed_phase_with_not_applied_is_rejected() {
        // Only actual primary publication reports phase committed, and it
        // reports an actual effect; not_applied contradicts the phase.
        let data = ResultData::new(Phase::Committed, Effect::NotApplied);
        assert_eq!(
            data.validate(),
            Err(ResultDataError::EffectRequired {
                phase: Phase::Committed,
                effect: Effect::NotApplied,
            })
        );
    }

    #[test]
    fn predicted_effect_requires_validated_phase() {
        let mut data = ResultData::new(Phase::Committed, Effect::Changed);
        data.set_predicted(PredictedEffect::Unchanged);
        assert!(matches!(
            data.validate(),
            Err(ResultDataError::PredictedWithoutValidation { .. })
        ));
        let mut data = ResultData::new(Phase::Validated, Effect::NotApplied);
        data.set_predicted(PredictedEffect::Changed);
        assert!(data.validate().is_ok());
    }

    #[test]
    fn exit_status_for_ok_is_success() {
        assert_eq!(exit_status_for(Status::Ok, None), ExitStatus::SUCCESS);
    }

    #[test]
    fn exit_status_for_refused_with_no_code_is_refused() {
        // Empty code falls into the default branch.
        let err = Error::new("UNKNOWN");
        assert_eq!(
            exit_status_for(Status::Refused, Some(&err)),
            ExitStatus::REFUSED
        );
    }

    #[test]
    fn error_status_for_unclassified_code_is_input_failure() {
        let err = Error::new("INVALID_PLAN");
        assert_eq!(
            exit_status_for(Status::Error, Some(&err)),
            ExitStatus::INPUT_FAILURE
        );
    }

    #[test]
    fn dedicated_exit_codes_win_over_the_status_fallback() {
        // The exit table is keyed by meaning: the dedicated code decides
        // whatever status the envelope carries, and only unclassified
        // codes fall back to the status.
        assert_eq!(
            exit_status_for(Status::Refused, Some(&Error::new("USAGE"))),
            ExitStatus::USAGE
        );
        assert_eq!(
            exit_status_for(Status::Error, Some(&Error::new("CHECK_THRESHOLD_EXCEEDED"))),
            ExitStatus::CHECK_FINDINGS
        );
    }

    #[test]
    fn status_serializes_as_lowercase_string() {
        let envelope = Envelope::refused(
            "edit apply",
            Source::new(&fixture()),
            Error::new("STALE_SOURCE"),
        );
        let value: serde_json::Value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(value["status"], "refused");
    }

    #[test]
    fn artifact_sha256_matches_source_sha256_helper() {
        let payload = b"a payload";
        let artifact = Artifact::new("/p", MediaRole::Document, "x", payload);
        assert_eq!(artifact.sha256, source_sha256(payload));
    }

    #[test]
    fn jsonl_round_trip_preserves_terminal_record() {
        let mut data = ResultData::new(Phase::Validated, Effect::NotApplied);
        data.counts.requested_operations = 1;
        data.set_predicted(PredictedEffect::Unchanged);
        let terminal = Envelope::ok(
            "edit apply",
            Source::new(&fixture()),
            data,
            Runtime::native(),
        )
        .expect("valid result data");
        let stream = write_jsonl_stream(&[], &terminal);
        let parsed = read_jsonl_stream(&stream).expect("round-trip parses");
        assert!(matches!(
            parsed.terminal.data.unwrap().phase,
            Some(Phase::Validated)
        ));
        assert_eq!(parsed.events.len(), 0);
    }
}
