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
//! Phase, effect and predicted effect are coupled by [`ResultData::validate`]:
//! a `generated` or `validated` phase is only representable with a
//! `not_applied` effect, and `predicted_effect` only appears with a
//! `validated` phase. [`Envelope::ok`] returns a typed [`ResultDataError`]
//! for invalid combinations so the build can fail loudly during preparation,
//! instead of producing a JSON shape that contradicts its own rules.
//!
//! Exit status mapping follows proposal §12 verbatim. JSONL streams carry
//! zero or more progress lines followed by exactly one terminal
//! [`Result`]; a stream without its terminal line is incomplete, and a
//! stream with two is malformed.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::inspect::source_sha256;

/// The wire schema version this module emits. Bumped together with any
/// backwards-incompatible change to the envelope.
pub const ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// The terminal status of a v2 result.
///
/// The on-the-wire shape is a plain string (`"ok"`, `"refused"`, `"error"`)
/// so callers can pattern-match on it without descending into a nested
/// object. The typed error itself lives in the envelope's [`Envelope::error`]
/// field; the two are mutually exclusive with `data` (an `Ok` envelope
/// carries `data`, a `Refused` or `Error` envelope carries `error`).
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
/// A `generated` or `validated` phase always has [`Effect::NotApplied`] —
/// see [`ResultData::validate`].
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
    /// `true` when `before` or `after` was bounded; the actual context
    /// continues past the shown window.
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
}

impl NextAction {
    /// Build a next action from a structured argv array.
    pub fn new(argv: Vec<String>) -> Self {
        Self {
            argv,
            prerequisites: None,
            purpose: None,
        }
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
}

impl Error {
    /// Build a new error with `code` and an empty message.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            operation: None,
            source_digest: None,
            target: None,
            matches: None,
            context: None,
            message: String::new(),
            next_actions: Vec::new(),
            retryable: None,
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

    /// Set the retryable hint.
    #[must_use]
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = Some(retryable);
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
}

impl Default for Verification {
    fn default() -> Self {
        Self {
            operation_postconditions: "skipped".to_string(),
            integrity: "skipped".to_string(),
            expectations: "not_requested".to_string(),
            task_semantics: "not_assessed".to_string(),
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

/// The body of a successful envelope: phase, effect, optional predicted
/// effect, counts, verification.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResultData {
    /// The stage this result reports.
    pub phase: Phase,
    /// The primary document effect.
    pub effect: Effect,
    /// Predicted effect for dry runs; only valid with [`Phase::Validated`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicted_effect: Option<PredictedEffect>,
    /// Counts of operations, revisions and comments.
    pub counts: Counts,
    /// Verification block.
    pub verification: Verification,
}

/// Invalid combinations of (phase, effect, predicted_effect).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultDataError {
    /// A `generated` or `validated` phase was paired with a `changed` or
    /// `unchanged` effect.
    NotAppliedRequired {
        /// The phase that requires `not_applied`.
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
            Self::NotAppliedRequired { phase, effect } => write!(
                f,
                "phase {phase:?} requires effect NotApplied, got {effect:?}"
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
    /// Build a result data block. `generated` and `validated` phases
    /// require `not_applied`; the `predicted_effect` is set with
    /// [`ResultData::set_predicted`] and only allowed for `validated`.
    /// [`Envelope::ok`] enforces both rules through
    /// [`ResultData::validate`].
    pub fn new(phase: Phase, effect: Effect) -> Self {
        Self {
            phase,
            effect,
            predicted_effect: None,
            counts: Counts::default(),
            verification: Verification::default(),
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
            Phase::Generated | Phase::Validated => {
                if self.effect != Effect::NotApplied {
                    return Err(ResultDataError::NotAppliedRequired {
                        phase: self.phase,
                        effect: self.effect,
                    });
                }
            }
            Phase::Inspected | Phase::Committed => {}
        }
        if self.predicted_effect.is_some() && self.phase != Phase::Validated {
            return Err(ResultDataError::PredictedWithoutValidation { phase: self.phase });
        }
        Ok(())
    }
}

/// The terminal record of a v2 command. The envelope is one of three
/// shapes, distinguished by `status`:
///
/// - `ok`: `data` carries the phase/effect/counts/verification; `error` is
///   absent; `artifacts` lists committed (and any uncommitted) outputs.
/// - `refused`: `error` carries the typed refusal; `data` is absent.
/// - `error`: `error` carries the typed failure; `data` is absent.
///
/// `status`, `error` and `data` are mutually exclusive in shape: an `ok`
/// envelope has a `data` block and no `error`; `refused` and `error`
/// envelopes carry the `error` block and no `data`.
///
/// The Rust identifier is `Envelope` to avoid shadowing
/// `std::result::Result` in function signatures.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Wire schema version; equals [`ENVELOPE_SCHEMA_VERSION`].
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
    /// Successful result data; absent on refusal or failure.
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
    /// refusal.
    pub fn refused(command: impl Into<String>, source: Source, error: Error) -> Self {
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime: Runtime::native(),
            status: Status::Refused,
            source,
            data: None,
            artifacts: Vec::new(),
            warnings: Vec::new(),
            error: Some(error),
            effective_options: BTreeMap::new(),
            continuation: None,
        }
    }

    /// Build a terminal envelope with `status: error` and a typed
    /// failure.
    pub fn new_error(command: impl Into<String>, source: Source, error: Error) -> Self {
        Self {
            schema_version: ENVELOPE_SCHEMA_VERSION,
            command: command.into(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime: Runtime::native(),
            status: Status::Error,
            source,
            data: None,
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

/// Progress event for JSONL streams: a free-form JSON value so callers can
/// attach any `ev`/`load`/`op`/`save` line they want. The terminal record
/// is a typed [`Envelope`]; see [`jsonl_terminal_line`] and [`write_jsonl_stream`].
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

/// Serialize `events` followed by `terminal` as a JSONL stream. Each
/// event and the terminal record are written on their own line; the stream
/// always ends with the terminal record.
pub fn write_jsonl_stream(events: &[ProgressEvent], terminal: &Envelope) -> String {
    let mut out = String::new();
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
    /// Progress events in order.
    pub events: Vec<ProgressEvent>,
    /// The terminal record.
    pub terminal: Envelope,
}

/// Why a JSONL stream is malformed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    /// A line that is neither a progress event nor a recognized terminal
    /// record (no `status` field).
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

/// Read and validate a JSONL stream. A line is recognized as the terminal
/// record when it parses as an [`Envelope`] (i.e. it has a `status`
/// field). Every other line is treated as a progress event.
pub fn read_jsonl_stream(input: &str) -> std::result::Result<ParsedStream, StreamError> {
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
        if value.get("status").is_some() {
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
            events.push(ProgressEvent::new(value));
        }
    }
    terminal
        .map(|terminal| ParsedStream { events, terminal })
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
/// Refusals whose code is `CHECK_THRESHOLD_EXCEEDED` or
/// `COMPARISON_DIFFERENCE` map to their dedicated exit codes (2 and 5).
/// Other refusals map to 3. Errors whose code is `RENDER_FIDELITY_FAILED`,
/// `DEADLINE_EXCEEDED` or `USAGE` map to their dedicated exit codes
/// (4, 124 and 64); other errors map to 1.
pub fn exit_status_for(status: Status, error: Option<&Error>) -> ExitStatus {
    match status {
        Status::Ok => ExitStatus::SUCCESS,
        Status::Refused => match error.map(|e| e.code.as_str()) {
            Some(codes::CHECK_THRESHOLD_EXCEEDED) => ExitStatus::CHECK_FINDINGS,
            Some(codes::COMPARISON_DIFFERENCE) => ExitStatus::COMPARISON_DIFFERENCE,
            _ => ExitStatus::REFUSED,
        },
        Status::Error => match error.map(|e| e.code.as_str()) {
            Some(codes::RENDER_FIDELITY_FAILED) => ExitStatus::RENDER_FAILURE,
            Some(codes::DEADLINE_EXCEEDED) => ExitStatus::DEADLINE_EXCEEDED,
            Some(codes::USAGE) => ExitStatus::USAGE,
            _ => ExitStatus::INPUT_FAILURE,
        },
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
            Phase::Validated
        ));
        assert_eq!(parsed.events.len(), 0);
    }
}
