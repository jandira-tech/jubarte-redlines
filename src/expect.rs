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
//!   `identity_correspondence` (the inspected file IS the reopened
//!   output). The `output` indices address the projection below;
//!   duplicate short ids refuse at construction.
//! - `evaluate` reads the source and the reopened output bytes, projects
//!   the output's paragraphs in the final projection — the body's in
//!   body order, then the story paragraphs (`header1`, `header1.p1`,
//!   ...) story by story, the order `inspect::markdown` prints — and
//!   judges each typed check on its target. Both halves of a target
//!   must name the same entity through the one ids codec, or the check
//!   refuses `Ambiguous`. A missing, unmapped, merged, split or
//!   unsupported target refuses, a correspondence index outside the
//!   projection refuses `StaleCorrespondence` (never a panic), and a
//!   vanished paragraph never passes `text_absent`.
//! - Natural-language requirements whose `checks` are empty are listed
//!   in `ExpectationReport::unassessed_requirements`, apart from the
//!   typed checks (no pseudo-check with a fabricated kind); a passing
//!   typed check is one check, not the whole requirement.
//! - `ExpectationReport` carries the per-check outcomes and a receipt
//!   with the path, digest (`None` when absent, never a fake empty
//!   string), schema version, the fixed final projection, separate
//!   counts of checks that ran, refused, were unsupported and
//!   requirements left unassessed, the evaluated targets, and any
//!   caller-declared provenance, never an independence or
//!   completeness claim.

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
    /// Typed checks; an empty array means the requirement is unassessed.
    /// The schema requires this field, so missing keys refuse.
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
    /// The correspondence maps the target to an output index outside
    /// the reopened output's projection: a stale or foreign
    /// correspondence, not a judgement about the document.
    StaleCorrespondence,
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

/// The compact, structured receipt (proposal §10 / #1017, edit plan P5:
/// "state which checks ran, were unsupported or remained unassessed").
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectationReceipt {
    /// Schema version of the parsed expectations.
    pub schema_version: u32,
    /// The source digest the expectations name.
    pub source_sha256: String,
    /// The digest of the expectations bytes themselves, when the caller
    /// supplied one. A missing digest is `None`, never an empty string
    /// that reads like one (review row 11).
    pub expectations_sha256: Option<String>,
    /// Path of the expectations file, when supplied.
    pub expectations_path: Option<String>,
    /// The fixed final projection the checks evaluated (edit plan P5:
    /// the receipt records the projection; `final` only in this
    /// profile).
    pub text_view: TextView,
    /// Typed checks that ran (passed or failed). A refused check never
    /// ran, in any branch (review row 3).
    pub checks_run: usize,
    /// Typed checks that refused, for any reason except an unsupported
    /// target.
    pub checks_refused: usize,
    /// Typed checks that refused because their target is not a
    /// paragraph the initial profile supports.
    pub checks_unsupported: usize,
    /// Natural-language requirements that remained unassessed (their
    /// `checks` list was empty).
    pub requirements_unassessed: usize,
    /// Targets evaluated, in encounter order, deduplicated.
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
/// `check` case builds with [`identity_correspondence`]. The `output`
/// indices of its entries address the projected output paragraphs in
/// the checker's projection order: the body's paragraphs in body order,
/// then the story paragraphs (`header1`, `header1.p1`, ...) story by
/// story.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SourceToOutputCorrespondence {
    entries: Vec<CorrespondenceEntry>,
    /// short id → position in `entries`, so [`lookup`] is one hash
    /// lookup instead of a linear scan per check.
    index: std::collections::HashMap<String, usize>,
}

impl SourceToOutputCorrespondence {
    /// Build a correspondence from explicit entries. Order is preserved.
    ///
    /// # Errors
    ///
    /// A plain string, when two entries share a `short_id`: a
    /// correspondence names each source paragraph once, and a silent
    /// first-wins lookup would hide the construction bug until it
    /// surfaced as the wrong paragraph.
    pub fn from_entries(entries: Vec<CorrespondenceEntry>) -> Result<Self, String> {
        let mut index = std::collections::HashMap::with_capacity(entries.len());
        for (position, entry) in entries.iter().enumerate() {
            if index.insert(entry.short_id.clone(), position).is_some() {
                return Err(format!(
                    "duplicate correspondence short id: {}",
                    entry.short_id
                ));
            }
        }
        Ok(Self { entries, index })
    }

    /// The correspondence entries.
    pub fn entries(&self) -> &[CorrespondenceEntry] {
        &self.entries
    }

    /// Look up an entry by source short id.
    pub fn lookup(&self, short_id: &str) -> Option<&CorrespondenceEntry> {
        self.index
            .get(short_id)
            .map(|&position| &self.entries[position])
    }
}

/// The full per-check report and the receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectationReport {
    /// Per-check outcomes in encounter order. Only typed checks appear
    /// here; an unassessed requirement is listed below, never as a
    /// pseudo-check with a fabricated kind (review row 12).
    pub checks: Vec<CheckOutcome>,
    /// Requirement keys whose `checks` list was empty: reported
    /// `unassessed`, apart from the checks, in encounter order.
    pub unassessed_requirements: Vec<String>,
    /// The compact, structured receipt.
    pub receipt: ExpectationReceipt,
}

/// Parse the closed `schema_version: 1` expectations document. The
/// Rust type's `deny_unknown_fields` and the explicit checks below
/// enforce every constraint the bundled JSON Schema states, so the
/// parser does not need a runtime JSON Schema validator (the WASM and
/// Python lockfiles therefore stay free of `jsonschema` and ~40
/// transitive crates for a closed object with four fields). The tests
/// run the accepted/rejected corpus through both this function and
/// the bundled schema to keep them in lockstep.
///
/// The parser is additionally stricter on two document classes the
/// bundled schema accepts (the safe direction; pinned by
/// `parser_is_stricter_than_the_bundled_schema_on_two_pinned_divergences`):
/// - `"schema_version": 1.0` (or `1e0`): JSON Schema `const` compares
///   numbers by mathematical equality, so the schema's `const: 1`
///   accepts it, while serde cannot deserialize a float into the
///   `u32` field.
/// - a JSON object with a duplicated field: the schema side reads a
///   last-value-wins `serde_json::Value`, while the derived
///   `Deserialize` errors on the duplicate field.
///
/// Parsing fails with a plain string for the caller to surface.
pub fn parse_expectations(json: &str) -> Result<ExpectationsV1, String> {
    // Closed shape first (Rust type, deny_unknown_fields).
    let parsed: ExpectationsV1 =
        serde_json::from_str(json).map_err(|e| format!("expectations Rust type rejected: {e}"))?;

    // schema_version == 1 (the schema's `const`).
    if parsed.schema_version != EXPECTATIONS_SCHEMA_VERSION {
        return Err(format!(
            "schema_version {} is not supported (expected {})",
            parsed.schema_version, EXPECTATIONS_SCHEMA_VERSION
        ));
    }

    // source.sha256: exactly 64 lowercase hexadecimal characters, no
    // trailing newline (the schema's anchored `^[0-9a-f]{64}$`).
    if !is_lowercase_hex_64(&parsed.source.sha256) {
        return Err(
            "source.sha256 must be exactly 64 lowercase hexadecimal characters".to_string(),
        );
    }

    // text_view: `final` only at this stage (the schema's `const`).
    if !matches!(parsed.text_view, TextView::Final) {
        return Err("text_view must be \"final\"".to_string());
    }

    // requirements: non-empty (the schema's `minItems: 1`).
    if parsed.requirements.is_empty() {
        return Err("requirements must contain at least one item".to_string());
    }

    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for requirement in &parsed.requirements {
        // key: `^[A-Za-z][A-Za-z0-9_.-]*$` (the schema's `pattern`
        // plus its `not` clause, which forbids any character outside
        // the allowed set including whitespace).
        if !is_valid_key(&requirement.key) {
            return Err(format!(
                "requirement key {:?} must match ^[A-Za-z][A-Za-z0-9_.-]*$",
                requirement.key
            ));
        }

        // description: non-empty (the schema's `minLength: 1`).
        if requirement.description.is_empty() {
            return Err(format!(
                "requirement {:?} description must be non-empty",
                requirement.key
            ));
        }

        // The schema's `$comment` calls for a "compiler check" on
        // unique keys; `uniqueItems` does not express it because the
        // keys are object properties, not array elements.
        if !seen.insert(requirement.key.as_str()) {
            return Err(format!("duplicate requirement key: {}", requirement.key));
        }

        for check in &requirement.checks {
            // target: short_id and long_id must be non-empty
            // (the schema's `minLength: 1`).
            if check.target.short_id.is_empty() || check.target.long_id.is_empty() {
                return Err(format!(
                    "requirement {:?} check target requires non-empty short_id and long_id",
                    requirement.key
                ));
            }

            // text_contains / text_absent: non-empty text. text_equals
            // allows empty text (the schema's per-kind `minLength`).
            if matches!(check.kind, CheckKind::TextContains | CheckKind::TextAbsent)
                && check.text.is_empty()
            {
                return Err(format!(
                    "requirement {:?} text_contains/text_absent requires non-empty text",
                    requirement.key
                ));
            }
        }
    }

    Ok(parsed)
}

/// True iff `s` is exactly 64 bytes of `[0-9a-f]` (no trailing
/// newline, no uppercase).
fn is_lowercase_hex_64(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 64
        && bytes
            .iter()
            .all(|&b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// True iff `s` matches the schema's key regex `^[A-Za-z][A-Za-z0-9_.-]*$`.
fn is_valid_key(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_alphabetic() {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-')
}

/// SHA-256 of arbitrary content, lowercase hex.
pub fn content_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Build the identity correspondence: the inspected file IS the reopened
/// output. Used for the standalone `check` case (#943) and for tests
/// that bind the expectations to the file the inspector reads. Covers
/// the story paragraphs too (`header1`, `header1.p1`, ...): the ids
/// `docx_text`/`jubarte read` print are checkable targets, in the same
/// projection order [`evaluate`] uses.
///
/// # Errors
///
/// A plain string when the file cannot be opened for projection.
pub fn identity_correspondence(
    source_bytes: &[u8],
) -> Result<SourceToOutputCorrespondence, String> {
    let paragraphs =
        projected_paragraphs(source_bytes).map_err(|e| format!("cannot open source: {e}"))?;
    let entries = paragraphs
        .into_iter()
        .enumerate()
        .map(|(index, p)| CorrespondenceEntry::one_to_one(p.id, index))
        .collect();
    Ok(SourceToOutputCorrespondence::from_entries(entries)
        .expect("the inspector reports each paragraph id once"))
}

/// The paragraphs a expectations run judges, in the fixed projection
/// order: the body's paragraphs in body order, then the story
/// paragraphs (`header1`, `header1.p1`, ...) story by story — the same
/// order `inspect::markdown` prints.
fn projected_paragraphs(
    bytes: &[u8],
) -> Result<Vec<crate::inspect::Paragraph>, crate::inspect::InspectError> {
    let mut all = crate::inspect::paragraphs(bytes)?;
    for story in crate::inspect::stories(bytes)? {
        all.extend(story.paragraphs);
    }
    Ok(all)
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
    let mut unassessed_requirements: Vec<String> = Vec::new();
    let mut targets: Vec<String> = Vec::new();
    let mut seen_targets: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    // Project the output's paragraphs in the final projection: the
    // body's, then the stories'. Unopenable output bytes refuse every
    // typed check with its real kind and target (review row 2);
    // natural-language requirements stay unassessed, listed apart
    // from the checks.
    let output_paragraphs = match projected_paragraphs(output) {
        Ok(paragraphs) => paragraphs,
        Err(error) => {
            for requirement in &spec.requirements {
                if requirement.checks.is_empty() {
                    unassessed_requirements.push(requirement.key.clone());
                } else {
                    for check in &requirement.checks {
                        checks.push(CheckOutcome {
                            key: requirement.key.clone(),
                            target: check.target.short_id.clone(),
                            kind: check.kind,
                            outcome: Outcome::Refused,
                            evidence: Some(error.to_string()),
                            reason: Some(RefusalReason::InvalidOutput),
                        });
                    }
                }
            }
            return assemble_report(spec, meta, checks, unassessed_requirements, targets);
        }
    };
    let source_sha = content_sha256(source);
    let stale_source = source_sha != spec.source.sha256;

    for requirement in &spec.requirements {
        if requirement.checks.is_empty() {
            // Unassessed, listed apart from the checks: no pseudo-check
            // with a fabricated kind (review row 12).
            unassessed_requirements.push(requirement.key.clone());
            continue;
        }
        for check in &requirement.checks {
            let target_label = check.target.short_id.clone();

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

            // The initial profile checks paragraphs only: the body's
            // `pN`, a story's own first paragraph (`header1`) or a
            // story's `pN`. Targets naming anything else (tables,
            // ranges, comments, revisions) refuse as unsupported --
            // before the correspondence lookup, so the reason does not
            // depend on what the correspondence happens to carry
            // (review row 5).
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

            // Both ids must name the SAME entity (schema `$defs.target`),
            // judged through the one ids codec: a disagreeing pair
            // refuses as ambiguous before any lookup (review row 6).
            if !ids_name_one_entity(&check.target.short_id, &check.target.long_id) {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some(
                        "the target's short id and long id do not name the same entity".to_string(),
                    ),
                    reason: Some(RefusalReason::Ambiguous),
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
            let Some(paragraph) = output_paragraphs.get(entry.output[0]) else {
                checks.push(CheckOutcome {
                    key: requirement.key.clone(),
                    target: target_label,
                    kind: check.kind,
                    outcome: Outcome::Refused,
                    evidence: Some(format!(
                        "correspondence maps the target to output paragraph {}, but the \
                         reopened output's projection holds {} paragraphs",
                        entry.output[0],
                        output_paragraphs.len()
                    )),
                    reason: Some(RefusalReason::StaleCorrespondence),
                });
                continue;
            };
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
            // Only an evaluated check ran, and only an evaluated check
            // lists its target (review row 3).
            if seen_targets.insert(target_label.clone()) {
                targets.push(target_label.clone());
            }
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

    assemble_report(spec, meta, checks, unassessed_requirements, targets)
}

/// Assemble the report and its receipt. Both the evaluated path and
/// the InvalidOutput path end here, so the receipt's counts mean one
/// thing in every branch (review row 3): `checks_run` counts the
/// checks that ran (passed or failed), never the refused ones.
fn assemble_report(
    spec: &ExpectationsV1,
    meta: ReceiptMeta,
    checks: Vec<CheckOutcome>,
    unassessed_requirements: Vec<String>,
    targets: Vec<String>,
) -> ExpectationReport {
    let checks_run = checks
        .iter()
        .filter(|c| matches!(c.outcome, Outcome::Passed | Outcome::Failed))
        .count();
    let checks_unsupported = checks
        .iter()
        .filter(|c| {
            matches!(
                (c.outcome, &c.reason),
                (Outcome::Refused, Some(RefusalReason::Unsupported))
            )
        })
        .count();
    let checks_refused = checks
        .iter()
        .filter(|c| {
            c.outcome == Outcome::Refused
                && !matches!(c.reason.as_ref(), Some(RefusalReason::Unsupported))
        })
        .count();
    let requirements_unassessed = unassessed_requirements.len();
    ExpectationReport {
        checks,
        unassessed_requirements,
        receipt: ExpectationReceipt {
            schema_version: spec.schema_version,
            source_sha256: spec.source.sha256.clone(),
            expectations_sha256: meta.digest,
            expectations_path: meta.path,
            text_view: spec.text_view,
            checks_run,
            checks_refused,
            checks_unsupported,
            requirements_unassessed,
            targets,
            declared_provenance: meta.declared_provenance,
            independent: false,
            complete_task: false,
        },
    }
}

/// Whether a target's two ids name the same entity, judged through the
/// one ids codec (`crate::ids`); there is no second parser. The short
/// id must parse as the paragraph place this profile checks, the long
/// id must be the four-field form `PART.ELEMENT.ATTRIBUTE.VALUE`
/// (short-and-long id contract §5: parse on the first three literal
/// dots, the remainder is the VALUE), its ELEMENT must be `w:p`, and
/// its PART must be the part the short id's story lives in
/// (`document` for the body). When the ATTRIBUTE is the contract's
/// `j:short`, its VALUE must parse to the very same
/// [`crate::ids::ShortId`]. A long id of any other shape, or one
/// naming another entity, makes the pair ambiguous. A native-key long
/// id (`w14:paraId`) is proven against part and element only; its key
/// identity is the apply path's resolver's to check (the v2
/// `TARGET_ID_MISMATCH` refusal).
fn ids_name_one_entity(short_id: &str, long_id: &str) -> bool {
    use crate::ids::{ShortId, Story};
    let Ok(short) = short_id.parse::<ShortId>() else {
        return false;
    };
    let ShortId::Block { story, .. } = &short else {
        return false;
    };
    let fields: Vec<&str> = long_id.splitn(4, '.').collect();
    let [part, element, attribute, value] = fields.as_slice() else {
        return false;
    };
    if *element != "w:p" {
        return false;
    }
    let part_matches = match story {
        Story::Body => *part == "document",
        Story::Part(stem) => *part == stem.as_str(),
    };
    if !part_matches {
        return false;
    }
    if *attribute == "j:short" {
        return value.parse::<ShortId>().is_ok_and(|named| named == short);
    }
    // A native key: part and element agree; the value names the key.
    !value.is_empty()
}

fn is_paragraph_target(short_id: &str) -> bool {
    // The initial profile supports paragraph short ids: the body's `pN`
    // (a cell's paragraphs carry their flat body `pN` too), a story's
    // own first paragraph (`header1`) and a story's `header1.pN`,
    // `footer1.pN`, `footnotes.pN`, ... — the same set
    // `ids::parse_location` resolves as a paragraph. Tables, ranges,
    // comments and revisions are unsupported until an explicit
    // alternative predicate/mapping lands.
    use crate::ids::{ShortId, Step, Story};
    match short_id.parse::<ShortId>() {
        Ok(ShortId::Block { story, path }) => matches!(
            (story, path.as_slice()),
            (Story::Body, [Step::Paragraph(_)]) | (Story::Part(_), [] | [Step::Paragraph(_)])
        ),
        Ok(ShortId::Range { .. }) => false,
        Ok(_) => false,
        Err(_) => false,
    }
}
