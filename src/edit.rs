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

use crate::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use crate::inspect::{Opened, Piece, Projection, SCHEMA_VERSION, project_paragraph, source_sha256};
use crate::namespaces::{R, W};
use crate::xmllinq::{Dom, NodeId, XNamespace};

const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";
const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";

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
    /// What to do when the source already holds tracked changes.
    #[serde(default)]
    /// Policy that was applied.
    pub existing_revisions: ExistingRevisions,
    /// Operations in report order.
    pub operations: Vec<Operation>,
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
        /// Comment text anchored to the changed text.
        comment: Option<String>,
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
    },
    /// Insert a new paragraph next to the anchor paragraph, copying its
    /// paragraph properties (never its section break or revision marks).
    InsertParagraph {
        /// Paragraph to edit; must match exactly one.
        paragraph: Selector,
        #[serde(default)]
        /// Which side of the anchor paragraph.
        position: Side,
        /// Runs of the new paragraph, in order.
        runs: Vec<RunSpec>,
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
    },
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

/// Paragraph selector; every form must match exactly one body paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Selector {
    /// `{"id": "body:p:12"}`.
    Id {
        /// `body:p:N`.
        id: String,
    },
    /// `{"index": 12}`.
    Index {
        /// Zero-based body index.
        index: usize,
    },
    /// `{"starts_with": "..."}`; unique prefix match.
    StartsWith {
        /// Unique paragraph text prefix.
        starts_with: String,
    },
    /// `{"contains": "..."}`; unique substring match.
    Contains {
        /// Unique paragraph text substring.
        contains: String,
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
    /// Error message when failed.
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
    /// Formatting changes.
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
        push(serde_json::json!({
            "ev": "load",
            "sha256": self.source_sha256,
            "base_sha256": self.base_sha256,
            "guarded": self.guarded,
            "paras": self.paragraphs.from,
            "author": self.author,
            "date": self.date,
            "existing_revisions": self.existing_revisions,
        }));
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
            "replace" => &["find", "replacement", "comment"],
            "insert" => &["after", "before", "position", "text", "comment"],
            "delete" => &["find"],
            "comment" => &["find", "text"],
            "insert_paragraph" => &["position", "runs", "style", "comment"],
            "delete_paragraph" => &[],
            other => {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!("operations[{i}]: unknown kind {other:?}"),
                ));
            }
        };
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
    tx.apply()?;
    let clean = tx.finish_clean()?;
    let settings = WmlComparerSettings {
        author_for_revisions: plan.author.clone(),
        date_time_for_revisions: tx.date.clone(),
        ..WmlComparerSettings::default()
    };
    let redline =
        crate::document_comparer::compare_documents_with_settings(&tx.base, &clean, &settings)
            .map_err(|e| err("COMPARE_FAILED", None, e.to_string()))?;
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
    counts
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
    InsertParagraph {
        anchor: usize,
        side: Side,
        /// Runs of the new paragraph, in order.
        runs: Vec<RunSpec>,
        /// Paragraph style id to set instead of the anchor's.
        style: Option<String>,
        /// Comment text anchored to the changed text.
        comment: Option<String>,
    },
}

struct Transaction<'p> {
    plan: &'p EditPlan,
    source_sha256: String,
    base: Vec<u8>,
    base_sha256: String,
    date: String,
    initials: String,
    opened: Opened,
    paragraph_nodes: Vec<NodeId>,
    projections: Vec<Projection>,
    outcomes: Vec<EditOutcome>,
    resolved: Vec<(usize, Resolved)>,
    comments: Vec<(u32, String)>,
    next_comment_id: u32,
    comments_added: usize,
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
        let probe =
            Opened::open(source).map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
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
        let has_revisions = crate::inspect::revision_count(&probe.dom, probe.body) > 0;
        let (base, opened) = match (has_revisions, plan.existing_revisions) {
            (false, _) => (source.to_vec(), probe),
            (true, ExistingRevisions::Refuse) => {
                return Err(err(
                    "EXISTING_REVISIONS",
                    None,
                    "the document already holds tracked changes; set existing_revisions to accept or reject",
                ));
            }
            (true, policy) => {
                let flattened = match policy {
                    ExistingRevisions::Accept => crate::document_comparer::accept_revisions(source),
                    _ => crate::document_comparer::reject_revisions(source),
                }
                .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
                let reopened = Opened::open(&flattened)
                    .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
                (flattened, reopened)
            }
        };
        let base_sha256 = source_sha256(&base);
        let paragraph_nodes = crate::inspect::body_paragraph_nodes(&opened.dom, opened.body);
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
        let next_comment_id = existing_comment_ids(&opened).map_or(0, |max| max + 1);
        Ok(Self {
            plan,
            source_sha256: source_hash,
            base,
            base_sha256,
            date,
            initials,
            opened,
            paragraph_nodes,
            projections,
            outcomes: Vec::new(),
            resolved: Vec::new(),
            comments: Vec::new(),
            next_comment_id,
            comments_added: 0,
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
            paragraphs: ParagraphDelta {
                from: self.paragraph_nodes.len(),
                to: self.paragraph_nodes.len(),
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
            match self.resolve_one(&id, &op.kind) {
                Ok((resolved, mut outcome)) => {
                    outcome.id = id;
                    outcome.kind = kind;
                    outcome.status = "ok".into();
                    self.outcomes.push(outcome);
                    self.resolved.push((i, resolved));
                }
                Err((mut e, outcome)) => {
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
    ) -> Result<(Resolved, EditOutcome), (EditError, EditOutcome)> {
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
        let fail =
            |code: &str, msg: String, outcome: EditOutcome| (err(code, Some(id), msg), outcome);
        let selector = match kind {
            OperationKind::Replace { paragraph, .. }
            | OperationKind::Insert { paragraph, .. }
            | OperationKind::Delete { paragraph, .. }
            | OperationKind::Comment { paragraph, .. }
            | OperationKind::InsertParagraph { paragraph, .. }
            | OperationKind::DeleteParagraph { paragraph } => paragraph,
        };
        let para = match self.select(selector) {
            Ok(p) => p,
            Err((code, msg, matches)) => {
                outcome.matches = matches;
                return Err(fail(&code, msg, outcome));
            }
        };
        outcome.paragraph = Some(format!("body:p:{para}"));
        let projection = &self.projections[para];
        let text = &projection.text;
        match kind {
            OperationKind::Replace {
                find,
                replacement,
                comment,
                ..
            } => {
                check_text(replacement).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
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
                    },
                    outcome,
                ))
            }
            OperationKind::Insert {
                after,
                before,
                position,
                text: new,
                comment,
                ..
            } => {
                check_text(new).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
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
                    },
                    outcome,
                ))
            }
            OperationKind::Comment {
                find, text: note, ..
            } => {
                if note.trim().is_empty() {
                    return Err(fail(
                        "INVALID_EDIT",
                        "comment text must be nonempty".into(),
                        outcome,
                    ));
                }
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
            OperationKind::DeleteParagraph { .. } => {
                outcome.matches = 1;
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
                let body_children = dom.elements(self.opened.body, Some(&W::p()));
                if body_children.len() == 1 && body_children[0] == node {
                    return Err(fail(
                        "UNSUPPORTED_STRUCTURE",
                        "the body must keep one paragraph".into(),
                        outcome,
                    ));
                }
                outcome.context = Some(format!("{{-¶ {}}}", excerpt(text, 60)));
                Ok((Resolved::DeleteParagraph { para }, outcome))
            }
            OperationKind::InsertParagraph {
                position,
                runs,
                style,
                comment,
                ..
            } => {
                outcome.matches = 1;
                if runs.is_empty() || runs.iter().all(|r| r.text.is_empty()) {
                    return Err(fail("INVALID_EDIT", "runs must carry text".into(), outcome));
                }
                for r in runs {
                    check_text(&r.text).map_err(|m| fail("INVALID_EDIT", m, outcome.clone()))?;
                }
                let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
                outcome.context = Some(format!("{{+¶ {}}}", excerpt(&joined, 60)));
                Ok((
                    Resolved::InsertParagraph {
                        anchor: para,
                        side: *position,
                        runs: runs.clone(),
                        style: style.clone(),
                        comment: comment.clone(),
                    },
                    outcome,
                ))
            }
        }
    }

    fn select(&self, selector: &Selector) -> Result<usize, (String, String, usize)> {
        let count = self.paragraph_nodes.len();
        let by_index = |index: usize| {
            if index < count {
                Ok(index)
            } else {
                Err((
                    "ANCHOR_NOT_FOUND".to_string(),
                    format!("paragraph index {index} does not exist ({count} paragraphs)"),
                    0,
                ))
            }
        };
        match selector {
            Selector::Index { index } => by_index(*index),
            Selector::Id { id } => match id
                .strip_prefix("body:p:")
                .and_then(|n| n.parse::<usize>().ok())
            {
                Some(index) => by_index(index),
                None => Err((
                    "ANCHOR_NOT_FOUND".into(),
                    format!("unknown paragraph id {id}"),
                    0,
                )),
            },
            Selector::StartsWith { starts_with }
            | Selector::Contains {
                contains: starts_with,
            } => {
                let prefix = matches!(selector, Selector::StartsWith { .. });
                if starts_with.is_empty() {
                    return Err((
                        "INVALID_EDIT".into(),
                        "paragraph selector text must be nonempty".into(),
                        0,
                    ));
                }
                let hits: Vec<usize> = self
                    .projections
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        if prefix {
                            p.text.starts_with(starts_with.as_str())
                        } else {
                            p.text.contains(starts_with.as_str())
                        }
                    })
                    .map(|(i, _)| i)
                    .collect();
                match hits.as_slice() {
                    [one] => Ok(*one),
                    [] => Err((
                        "ANCHOR_NOT_FOUND".into(),
                        format!("no paragraph matches {starts_with:?}"),
                        0,
                    )),
                    many => Err((
                        "AMBIGUOUS_ANCHOR".into(),
                        format!(
                            "{} paragraphs match {starts_with:?}: {}",
                            many.len(),
                            many.iter()
                                .map(|i| format!("body:p:{i}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        many.len(),
                    )),
                }
            }
        }
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
        for (_, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                deleted.push(*para);
            }
        }
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
                Resolved::DeleteParagraph { .. } => continue,
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
                        Resolved::CommentRange { text, .. } => Some(text.clone()),
                        Resolved::DeleteParagraph { .. } => None,
                    };
                    text.map(|t| (*i, t))
                })
                .collect();
        for (i, text) in comments {
            let id = self.new_comment(text);
            ids.insert(i, id);
            self.outcomes[i].comment_id = Some(id);
        }
        // 1. Text edits and their comments, paragraph by paragraph.
        let mut by_para: BTreeMap<usize, Vec<(usize, usize, usize, String, bool, Option<String>)>> =
            BTreeMap::new();
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
                } => by_para.entry(*para).or_default().push((
                    *start,
                    *end,
                    *i,
                    replacement.clone(),
                    *attach_before,
                    comment.clone(),
                )),
                Resolved::CommentRange {
                    para,
                    start,
                    end,
                    text,
                } => {
                    comment_ranges
                        .entry(*para)
                        .or_default()
                        .push((*start, *end, *i, text.clone()))
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
            edits.sort_by_key(|&(s, e, i, ..)| (s, e, i));
            // Apply in reverse so earlier source offsets stay valid.
            for (start, end, _, replacement, attach_before, _) in edits.iter().rev() {
                let projection = project_paragraph(&self.opened.dom, node);
                apply_text_edit(
                    &mut self.opened.dom,
                    &projection,
                    *start,
                    *end,
                    replacement,
                    *attach_before,
                );
            }
            // Comment ranges, in new coordinates.
            let mut pending: Vec<(usize, usize, usize, String)> = Vec::new();
            for (start, _, i, replacement, _, comment) in &edits {
                if let Some(text) = comment {
                    let s = new_position(&edits, *start, true, Some(*i));
                    pending.push((s, s + replacement.len(), *i, text.clone()));
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
        // 2. Paragraph insertions (anchors are source paragraphs, untouched by 1).
        let inserts: Vec<(usize, Resolved)> = self
            .resolved
            .iter()
            .filter(|(_, r)| matches!(r, Resolved::InsertParagraph { .. }))
            .cloned()
            .collect();
        for (i, r) in inserts {
            if let Resolved::InsertParagraph {
                anchor,
                side,
                runs,
                style,
                comment,
            } = r
            {
                let anchor_node = self.paragraph_nodes[anchor];
                let new =
                    build_paragraph(&mut self.opened.dom, anchor_node, &runs, style.as_deref());
                match side {
                    Side::After => self.opened.dom.add_after_self(anchor_node, new),
                    Side::Before => self.opened.dom.add_before_self(anchor_node, new),
                }
                if comment.is_some() {
                    let projection = project_paragraph(&self.opened.dom, new);
                    anchor_comment(&mut self.opened.dom, new, 0, projection.text.len(), ids[&i]);
                }
            }
        }
        // 3. Paragraph deletions.
        for (_, r) in &self.resolved {
            if let Resolved::DeleteParagraph { para } = r {
                self.opened.dom.remove(self.paragraph_nodes[*para]);
            }
        }
        Ok(())
    }

    fn new_comment(&mut self, text: String) -> u32 {
        let id = self.next_comment_id;
        self.next_comment_id += 1;
        self.comments_added += 1;
        self.comments.push((id, text));
        id
    }

    fn finish_clean(&mut self) -> Result<Vec<u8>, EditError> {
        let xml = self.opened.dom.serialize_document(self.opened.document);
        let main = self.opened.main.clone();
        self.opened.pkg.set_part(&main, xml.into_bytes());
        if !self.comments.is_empty() {
            self.write_comments_part()?;
        }
        self.opened
            .pkg
            .to_zip()
            .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
    }

    fn write_comments_part(&mut self) -> Result<(), EditError> {
        let existing = self.opened.related("comments").into_iter().next();
        let part_name = existing
            .clone()
            .unwrap_or_else(|| "word/comments.xml".to_string());
        let mut dom = Dom::new();
        let (document, root) = match &existing {
            Some(name) => {
                let xml = self.opened.pkg.part_string(name).ok_or_else(|| {
                    err(
                        "INVALID_DOCUMENT",
                        None,
                        format!("missing comments part {name}"),
                    )
                })?;
                let document = dom.parse_xdocument(&xml);
                let root = dom
                    .root(document)
                    .ok_or_else(|| err("INVALID_DOCUMENT", None, "empty comments part"))?;
                (document, root)
            }
            None => {
                let document = dom.parse_xdocument(&format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{}" xmlns:r="{}"/>"#,
                    W::URI,
                    R::URI
                ));
                let root = dom.root(document).expect("root");
                (document, root)
            }
        };
        for (id, text) in &self.comments {
            let comment = dom.new_element(W::name("comment"));
            dom.set_attribute_value(comment, &W::id(), Some(&id.to_string()));
            dom.set_attribute_value(comment, &W::author(), Some(&self.plan.author));
            dom.set_attribute_value(comment, &W::date(), Some(&self.date));
            dom.set_attribute_value(comment, &W::name("initials"), Some(&self.initials));
            let p = dom.new_element(W::p());
            let ref_run = dom.new_element(W::r());
            let annotation = dom.new_element(W::name("annotationRef"));
            dom.add(ref_run, annotation);
            dom.add(p, ref_run);
            for (i, line) in text.split('\n').enumerate() {
                let run = dom.new_element(W::r());
                if i > 0 {
                    let br = dom.new_element(W::name("br"));
                    dom.add(run, br);
                }
                let t = dom.new_element(W::t());
                dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
                dom.add_text(t, line);
                dom.add(run, t);
                dom.add(p, run);
            }
            dom.add(comment, p);
            dom.add(root, comment);
        }
        let xml = dom.serialize_document(document);
        self.opened.pkg.set_part(&part_name, xml.into_bytes());
        if existing.is_none() {
            let main = self.opened.main.clone();
            self.opened
                .pkg
                .add_document_relationship(&main, COMMENTS_REL, "comments.xml");
            self.opened
                .pkg
                .add_content_type_override("/word/comments.xml", COMMENTS_CT);
        }
        Ok(())
    }
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

fn kind_name(kind: &OperationKind) -> &'static str {
    match kind {
        OperationKind::Replace { .. } => "replace",
        OperationKind::Insert { .. } => "insert",
        OperationKind::Delete { .. } => "delete",
        OperationKind::Comment { .. } => "comment",
        OperationKind::InsertParagraph { .. } => "insert_paragraph",
        OperationKind::DeleteParagraph { .. } => "delete_paragraph",
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
/// operation whose own insertion must not shift its comment start.
fn new_position(
    edits: &[(usize, usize, usize, String, bool, Option<String>)],
    pos: usize,
    inclusive: bool,
    own: Option<usize>,
) -> usize {
    let mut delta: i64 = 0;
    for (start, end, i, replacement, ..) in edits {
        if Some(*i) == own {
            continue;
        }
        let shifts = if start == end {
            // insertion
            if inclusive {
                *start <= pos
            } else {
                *start < pos
            }
        } else {
            *end <= pos
        };
        if shifts {
            delta += replacement.len() as i64 - (*end - *start) as i64;
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
    let id_str = id.to_string();
    let range_start = dom.new_element(W::name("commentRangeStart"));
    dom.set_attribute_value(range_start, &W::id(), Some(&id_str));
    let range_end = dom.new_element(W::name("commentRangeEnd"));
    dom.set_attribute_value(range_end, &W::id(), Some(&id_str));
    let reference_run = dom.new_element(W::r());
    let reference = dom.new_element(W::name("commentReference"));
    dom.set_attribute_value(reference, &W::id(), Some(&id_str));
    dom.add(reference_run, reference);
    match (first, last) {
        (Some(first), Some(last)) if start < end => {
            dom.add_before_self(first, range_start);
            dom.add_after_self(last, range_end);
            dom.add_after_self(range_end, reference_run);
        }
        _ => {
            dom.add(paragraph, range_start);
            dom.add(paragraph, range_end);
            dom.add(paragraph, reference_run);
        }
    }
}

fn run_of(piece: &Piece) -> NodeId {
    match piece {
        Piece::Text { run, .. } | Piece::Glyph { run, .. } => *run,
    }
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
    let base_rpr = dom
        .elements(anchor, Some(&W::r()))
        .first()
        .and_then(|&r| dom.element(r, &W::r_pr()));
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
        toggle(dom, rpr, "b", spec.bold);
        toggle(dom, rpr, "bCs", spec.bold);
        toggle(dom, rpr, "i", spec.italic);
        toggle(dom, rpr, "iCs", spec.italic);
        if let Some(underline) = spec.underline {
            if let Some(u) = dom.element(rpr, &W::name("u")) {
                dom.remove(u);
            }
            if underline {
                let u = dom.new_element(W::name("u"));
                dom.set_attribute_value(u, &W::val(), Some("single"));
                insert_rpr_child(dom, rpr, u);
            }
        }
        if let Some(highlight) = &spec.highlight {
            if let Some(h) = dom.element(rpr, &W::name("highlight")) {
                dom.remove(h);
            }
            if highlight != "none" {
                let h = dom.new_element(W::name("highlight"));
                dom.set_attribute_value(h, &W::val(), Some(highlight));
                insert_rpr_child(dom, rpr, h);
            }
        }
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
    let after = dom
        .elements(rpr, None)
        .into_iter()
        .filter(|&c| {
            dom.name(c)
                .is_some_and(|n| n.namespace_name() == W::URI && rank(n.local_name()) <= my_rank)
        })
        .last();
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
        // (start, end, op, replacement, attach_before, comment)
        let edits = vec![
            (2, 4, 0, "XYZ".to_string(), true, None), // +1
            (6, 6, 1, "++".to_string(), true, None),  // insertion at 6
            (8, 9, 2, String::new(), true, None),     // -1
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
