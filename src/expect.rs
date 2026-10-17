// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! V2-13 (issue #845): the checker core for `--expect` evaluations.
//!
//! See jubarte-edit-implementation-plan.md P5 and jubarte-cli-proposal-b-revised.md
//! §10 for the design. The wiring into the apply path is staged in V2-08;
//! this module owns the types, the parser, the evaluator and the receipt.
//! The standalone `jubarte check OUTPUT --expect FILE` CLI is a separate
//! scope and is not implemented here.
//!
//! The model:
//! - `ExpectationsV1` is the closed `schema_version: 1` document.
//! - `SourceToOutputCorrespondence` is an input type: V2-08 will construct
//!   it from the prepared transaction; the standalone `check` case uses
//!   `identity_correspondence` (the inspected file IS the reopened output).
//! - `evaluate` reads the source and the reopened output bytes, projects
//!   the body paragraphs in the final projection, and judges each typed
//!   check on its target. A missing, unmapped, merged, split, ambiguous or
//!   unsupported target refuses (and a vanished paragraph never passes
//!   `text_absent`).
//! - Natural-language requirements whose `checks` are empty are reported
//!   `unassessed`; a passing typed check is one check, not the whole
//!   requirement.
//! - `ExpectationReport` carries the per-check outcomes and a receipt
//!   with the path, digest, schema version, run counts, targets, and
//!   any caller-declared provenance, never an independence or
//!   completeness claim.

use crate::ids::Step;

/// The expected schema version, per `jubarte-expectations-v1.schema.json`.
pub const EXPECTATIONS_SCHEMA_VERSION: u32 = 1;

/// Text view required by the initial profile: the FINAL logical projection
/// (insertions included, deletions excluded, no review markup).
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextView {
    /// Insertions included, deletions excluded, with no review markup.
    Final,
}

/// The wire shape of a `Check`'s `target` field. Both ids must name the
/// same entity in the hash-bound original source; the parser checks shape
/// and non-empty strings, the evaluator checks identity against the
/// correspondence and refuses on a mismatch.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Snapshot-relative short id (`p12`, `header1.p0`).
    pub short_id: String,
    /// Long id (the contract's `j:short` form or `document.w:p.j:short.pN`).
    pub long_id: String,
}

/// The three typed checks of the initial profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    /// Exact final text of the target paragraph; empty is allowed.
    TextEquals,
    /// The target paragraph's final text contains the given text.
    TextContains,
    /// The target paragraph's final text does NOT contain the given text.
    TextAbsent,
}

/// One typed check, with kind, target and text payload.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    /// Which typed check to run.
    pub kind: CheckKind,
    /// The paragraph this check judges.
    pub target: Target,
    /// The text payload of the check.
    pub text: String,
}

/// One requirement: a key, a natural-language description, and zero or
/// more typed checks. A requirement with zero checks stays `unassessed`.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    /// Settled id: `[A-Za-z][A-Za-z0-9_.-]*`, unique per document.
    pub key: String,
    /// Natural-language description. The initial profile does not assess
    /// it; callers read it as reference text.
    pub description: String,
    /// Typed checks; empty means the requirement is unassessed.
    #[serde(default)]
    pub checks: Vec<Check>,
}

/// The source-digest binding.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Exactly 64 lowercase hexadecimal characters.
    pub sha256: String,
}

/// The closed `schema_version: 1` expectations document.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectationsV1 {
    /// Always 1 in this profile.
    pub schema_version: u32,
    /// The source revision the expectations name.
    pub source: Source,
    /// The projection the checks evaluate. `final` only in this profile.
    pub text_view: TextView,
    /// One or more requirements.
    pub requirements: Vec<Requirement>,
}

/// The outcome of a single typed check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The predicate held against the reopened output.
    Passed,
    /// The predicate did not hold; the change has not been made.
    Failed,
    /// The target was missing, unmapped, merged, split, ambiguous or
    /// unsupported; the predicate could not be evaluated. A refusal
    /// never counts as a passing `text_absent`.
    Refused,
    /// The check had no typed predicate; the natural-language
    /// description is the requirement's only content.
    Unassessed,
}

/// A `Refused` reason, kept for the receipt (proposal §10).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefusalReason {
    /// The target's `short_id` is not present in the correspondence.
    TargetMissing,
    /// The target was mapped to multiple output paragraphs.
    Merged,
    /// The target was mapped to no output paragraph.
    Split,
    /// The two halves of the target name different entities.
    Ambiguous,
    /// The target is not a paragraph the initial profile supports.
    Unsupported,
    /// The expectations' `source.sha256` does not name the bytes the
    /// correspondence is built from.
    StaleSource,
    /// The output bytes cannot be opened for projection.
    InvalidOutput,
}

/// Per-check report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckOutcome {
    /// Requirement key this check belongs to.
    pub key: String,
    /// Target's short id, when the target resolved.
    pub target: String,
    /// Kind of the check.
    pub kind: CheckKind,
    /// What happened.
    pub outcome: Outcome,
    /// Evidence text: the projected paragraph text when resolved,
    /// or a refusal reason.
    pub evidence: Option<String>,
    /// Refusal reason when `outcome == Refused`.
    pub reason: Option<RefusalReason>,
}

/// The compact, structured receipt (proposal §10 / #1017).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectationReceipt {
    /// Schema version of the parsed expectations.
    pub schema_version: u32,
    /// The source digest the expectations name.
    pub source_sha256: String,
    /// The digest of the expectations bytes themselves.
    pub expectations_sha256: String,
    /// Path of the expectations file, when supplied.
    pub expectations_path: Option<String>,
    /// Number of typed checks that ran.
    pub checks_run: usize,
    /// Targets evaluated, in encounter order.
    pub targets: Vec<String>,
    /// Caller-declared provenance string, if any; the receipt carries it
    /// verbatim but does not verify it.
    pub declared_provenance: Option<String>,
    /// Always `false`. A separately supplied file is not a claim of
    /// independence.
    pub independent: bool,
    /// Always `false`. The CLI cannot claim the user's task is complete.
    pub complete_task: bool,
}

/// One source paragraph and the set of output paragraphs it mapped to
/// during preparation. The initial profile requires exact one-to-one
/// correspondence: the `output` slice must have exactly one element for
/// a passing typed check; anything else refuses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorrespondenceEntry {
    /// The source paragraph's short id (`p0`, `header1.p0`).
    pub short_id: String,
    /// The output paragraph indices this source paragraph maps to.
    pub output: Vec<usize>,
    /// Whether the target's two ids name the same source entity. The
    /// initial profile refuses when they disagree.
    pub identity_ok: bool,
}

impl CorrespondenceEntry {
    /// A 1:1 mapping of one source paragraph to one output paragraph.
    pub fn one_to_one(short_id: impl Into<String>, output: usize) -> Self {
        Self {
            short_id: short_id.into(),
            output: vec![output],
            identity_ok: true,
        }
    }

    /// A merged mapping (one source paragraph → many output paragraphs).
    pub fn merged(short_id: impl Into<String>, outputs: &[usize]) -> Self {
        Self {
            short_id: short_id.into(),
            output: outputs.to_vec(),
            identity_ok: true,
        }
    }

    /// A split/vanished mapping (one source paragraph → no output
    /// paragraph).
    pub fn split(short_id: impl Into<String>) -> Self {
        Self {
            short_id: short_id.into(),
            output: Vec::new(),
            identity_ok: true,
        }
    }
}

/// The S→output correspondence V2-08 will supply, and the standalone
/// `check` case builds with [`identity_correspondence`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SourceToOutputCorrespondence {
    entries: Vec<CorrespondenceEntry>,
}

impl SourceToOutputCorrespondence {
    /// Build a correspondence from explicit entries. Order is preserved.
    pub fn from_entries(entries: Vec<CorrespondenceEntry>) -> Self {
        Self { entries }
    }

    /// The correspondence entries.
    pub fn entries(&self) -> &[CorrespondenceEntry] {
        &self.entries
    }

    /// Look up an entry by source short id.
    pub fn lookup(&self, short_id: &str) -> Option<&CorrespondenceEntry> {
        self.entries.iter().find(|e| e.short_id == short_id)
    }
}

/// The full per-check report and the receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectationReport {
    /// Per-check outcomes in encounter order.
    pub checks: Vec<CheckOutcome>,
    /// The compact, structured receipt.
    pub receipt: ExpectationReceipt,
}

/// Parse the closed `schema_version: 1` expectations document. The
/// `ExpectationsV1` Rust type and the bundled JSON Schema enforce the
/// same closed shape; parsing fails with a plain string for the
/// caller to surface.
pub fn parse_expectations(json: &str) -> Result<ExpectationsV1, String> {
    // Closed shape first (Rust type, deny_unknown_fields). Then run the
    // bundled JSON Schema for the additional constraints the Rust type
    // cannot express (the key regex, text_view == "final", the check
    // `oneOf`, the empty-text rules on `text_contains`/`text_absent`).
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("malformed JSON: {e}"))?;
    let schema =
        serde_json::from_str(SCHEMA_JSON).expect("bundled expectations JSON Schema parses");
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| format!("could not build expectations JSON Schema validator: {e}"))?;
    let errors: Vec<_> = validator.iter_errors(&value).collect();
    if !errors.is_empty() {
        let detail = errors
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("expectations JSON Schema rejected: {detail}"));
    }
    let parsed: ExpectationsV1 = serde_json::from_value(value)
        .map_err(|e| format!("expectations Rust type rejected: {e}"))?;
    // The schema's `$comment` calls for a "compiler check" on unique
    // keys; `uniqueItems` does not express it because the keys are
    // object properties, not array elements.
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for requirement in &parsed.requirements {
        if !seen.insert(requirement.key.as_str()) {
            return Err(format!("duplicate requirement key: {}", requirement.key));
        }
    }
    Ok(parsed)
}

/// SHA-256 of arbitrary content, lowercase hex.
pub fn content_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Build the identity correspondence: the inspected file IS the reopened
/// output. Used for the standalone `check` case (#943) and for tests
/// that bind the expectations to the file the inspector reads.
pub fn identity_correspondence(
    source_bytes: &[u8],
) -> Result<SourceToOutputCorrespondence, String> {
    let paragraphs =
        crate::inspect::paragraphs(source_bytes).map_err(|e| format!("cannot open source: {e}"))?;
    let entries = paragraphs
        .into_iter()
        .enumerate()
        .map(|(index, p)| CorrespondenceEntry::one_to_one(p.id, index))
        .collect();
    Ok(SourceToOutputCorrespondence::from_entries(entries))
}

/// Evaluate typed checks on the reopened output, given the bytes the
/// correspondence was built from, the output bytes, and the
/// correspondence.
pub fn evaluate(
    spec: &ExpectationsV1,
    source: &[u8],
    output: &[u8],
    correspondence: &SourceToOutputCorrespondence,
) -> ExpectationReport {
    evaluate_with_receipt(spec, source, output, correspondence, ReceiptMeta::default())
}

/// Metadata for the receipt: the file path the caller read, the digest
/// of the file bytes, and any declared provenance.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReceiptMeta {
    /// Path of the expectations file (verbatim from the caller).
    pub path: Option<String>,
    /// SHA-256 of the expectations file bytes.
    pub digest: Option<String>,
    /// Caller-declared provenance string.
    pub declared_provenance: Option<String>,
}

/// As [`evaluate`], but also records receipt metadata: the file path,
/// the file digest, and any caller-declared provenance.
pub fn evaluate_with_receipt(
    spec: &ExpectationsV1,
    source: &[u8],
    output: &[u8],
    correspondence: &SourceToOutputCorrespondence,
    meta: ReceiptMeta,
) -> ExpectationReport {
    let mut checks: Vec<CheckOutcome> = Vec::new();
    let mut targets: Vec<String> = Vec::new();

    // Project the body paragraphs of the output in the final projection.
    let output_paragraphs = match crate::inspect::paragraphs(output) {
        Ok(paragraphs) => paragraphs,
        Err(error) => {
            // Every typed check refuses with the same InvalidOutput
            // reason. Natural-language requirements stay Unassessed.
            for requirement in &spec.requirements {
                if requirement.checks.is_empty() {
                    checks.push(CheckOutcome {
                        key: requirement.key.clone(),
                        target: String::new(),
                        kind: CheckKind::TextEquals,
                        outcome: Outcome::Unassessed,
                        evidence: None,
                        reason: None,
                    });
                } else {
                    for _ in &requirement.checks {
                        checks.push(CheckOutcome {
                            key: requirement.key.clone(),
                            target: String::new(),
                            kind: CheckKind::TextEquals,
                            outcome: Outcome::Refused,
                            evidence: Some(error.to_string()),
                            reason: Some(RefusalReason::InvalidOutput),
                        });
                    }
                }
            }
            return ExpectationReport {
                checks,
                receipt: ExpectationReceipt {
                    schema_version: spec.schema_version,
                    source_sha256: spec.source.sha256.clone(),
                    expectations_sha256: meta.digest.clone().unwrap_or_default(),
                    expectations_path: meta.path.clone(),
                    checks_run: 0,
                    targets,
                    declared_provenance: meta.declared_provenance,
                    independent: false,
                    complete_task: false,
                },
            };
        }
    };
    let source_sha = content_sha256(source);
    let stale_source = source_sha != spec.source.sha256;

    for requirement in &spec.requirements {
        if requirement.checks.is_empty() {
            checks.push(CheckOutcome {
                key: requirement.key.clone(),
                target: String::new(),
                kind: CheckKind::TextEquals,
                outcome: Outcome::Unassessed,
                evidence: None,
                reason: None,
            });
            continue;
        }
        for check in &requirement.checks {
            let target_label = check.target.short_id.clone();
            if !targets.contains(&target_label) {
                targets.push(target_label.clone());
            }

            if stale_source {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some(
                        "source.sha256 does not name the bytes the correspondence was built from"
                            .to_string(),
                    ),
                    reason: Some(RefusalReason::StaleSource),
                });
                continue;
            }

            let Some(entry) = correspondence.lookup(&check.target.short_id) else {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some("target is not in the correspondence".to_string()),
                    reason: Some(RefusalReason::TargetMissing),
                });
                continue;
            };
            if !entry.identity_ok {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some("target ids disagree".to_string()),
                    reason: Some(RefusalReason::Ambiguous),
                });
                continue;
            }
            // The initial profile supports paragraphs only. Targets
            // naming anything else (tables, cells, ranges, comments)
            // refuse as unsupported.
            if !is_paragraph_target(&check.target.short_id) {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some(
                        "target is not a paragraph the initial profile supports".to_string(),
                    ),
                    reason: Some(RefusalReason::Unsupported),
                });
                continue;
            }
            if entry.output.is_empty() {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some("target mapped to no output paragraph".to_string()),
                    reason: Some(RefusalReason::Split),
                });
                continue;
            }
            if entry.output.len() != 1 {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some(format!(
                        "target mapped to {} output paragraphs (one-to-one required)",
                        entry.output.len()
                    )),
                    reason: Some(RefusalReason::Merged),
                });
                continue;
            }
            let paragraph = &output_paragraphs[entry.output[0]];
            let evidence = paragraph.text.clone();
            let outcome = match check.kind {
                CheckKind::TextEquals => {
                    if paragraph.text == check.text {
                        Outcome::Passed
                    } else {
                        Outcome::Failed
                    }
                }
                CheckKind::TextContains => {
                    if paragraph.text.contains(&check.text) {
                        Outcome::Passed
                    } else {
                        Outcome::Failed
                    }
                }
                CheckKind::TextAbsent => {
                    if paragraph.text.contains(&check.text) {
                        Outcome::Failed
                    } else {
                        Outcome::Passed
                    }
                }
            };
            checks.push(CheckOutcome {
                key: requirement.key.clone(),
                target: target_label,
                kind: check.kind,
                outcome,
                evidence: Some(evidence),
                reason: None,
            });
        }
    }

    let checks_run = checks
        .iter()
        .filter(|c| c.outcome != Outcome::Unassessed)
        .count();
    let receipt = ExpectationReceipt {
        schema_version: spec.schema_version,
        source_sha256: spec.source.sha256.clone(),
        expectations_sha256: meta.digest.clone().unwrap_or_default(),
        expectations_path: meta.path.clone(),
        checks_run,
        targets,
        declared_provenance: meta.declared_provenance,
        independent: false,
        complete_task: false,
    };
    ExpectationReport { checks, receipt }
}

fn is_paragraph_target(short_id: &str) -> bool {
    // The initial profile supports paragraph short ids (`pN`,
    // `header1.pN`, `footer1.pN`, ...). Tables, cells, ranges and
    // comments are unsupported until an explicit alternative
    // predicate/mapping lands.
    use crate::ids::ShortId;
    match short_id.parse::<ShortId>() {
        Ok(ShortId::Block { path, .. }) => matches!(path.as_slice(), [Step::Paragraph(_)]),
        Ok(ShortId::Range { .. }) => false,
        Ok(_) => false,
        Err(_) => false,
    }
}

/// The bundled JSON Schema, mirrored from `docs/design/cli-v2/`'s
/// `jubarte-expectations-v1.schema.json` (the design, unchanged).
const SCHEMA_JSON: &str = include_str!("../schemas/expectations-v1.schema.json");
