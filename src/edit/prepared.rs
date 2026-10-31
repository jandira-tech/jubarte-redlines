// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `PreparedEdit`: one preparation path for dry run and apply (V2-08).
//!
//! The plan resolves and applies in memory, the result serializes, and the
//! *actual primary bytes* are chosen before final checks: the serialized
//! candidate for a changed result, or the original source bytes verbatim
//! for a truly unchanged one. Those chosen bytes are what findings, output
//! references and digests describe — a reserialized package is never
//! validated and then swapped for the source at commit. Dry run and apply
//! share this whole path; they differ only at publication.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::{
    EditError, EditOutcome, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved,
    StoryPart, Transaction, check_update_fields, comment_holds_kept_text, comment_text_of,
    controls, lift_deletion_comments, revision_counts, sections, tracked, whole,
};
use crate::comparer::WmlComparerSettings;
use crate::inspect::{body_paragraph_nodes, project_paragraph, source_sha256};
use crate::namespaces::W;
use crate::result::{Effect, codes};
use crate::validate::{self, Finding};

/// A finding identity the publication policy compares on: rule set, code,
/// part and the entity/range the finding points at. Findings are compared
/// by identity, never by aggregate counts: a baseline warning healed beside
/// a new same-code warning elsewhere still reports the new one.
fn finding_key(f: &Finding) -> (String, String, String, String) {
    (
        f.set.clone(),
        f.code.clone(),
        f.part.clone(),
        f.location.clone(),
    )
}

/// The findings in `candidate` that `baseline` does not carry, first
/// occurrence each, in `candidate` order.
pub fn introduced_findings(baseline: &[Finding], candidate: &[Finding]) -> Vec<Finding> {
    let held: HashSet<_> = baseline.iter().map(finding_key).collect();
    let mut seen = HashSet::new();
    candidate
        .iter()
        .filter(|f| !held.contains(&finding_key(f)))
        .filter(|f| seen.insert(finding_key(f)))
        .cloned()
        .collect()
}

/// A non-`word_fatal` finding code that still blocks publication when the
/// edit introduces it. `BOOKMARK_CROSSES_REVISION` is a protected-range
/// regression the general validator reports as a warning; an introduced
/// crossing is a failed invariant, never a disclosed lead.
/// `CELL_FINAL_MARK_REVISION` opens in Word, but its Accept All or Reject
/// All never returns (#1669).
const BLOCKING_CODES: &[&str] = &["BOOKMARK_CROSSES_REVISION", "CELL_FINAL_MARK_REVISION"];

/// Whether `f` blocks publication when it is newly introduced.
fn blocking(f: &Finding) -> bool {
    f.word_fatal || BLOCKING_CODES.contains(&f.code.as_str())
}

/// Which serialized bytes the prepared edit chose as its primary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryBytes {
    /// The original source, byte for byte: a truly unchanged primary is
    /// never a reserialization.
    Source,
    /// The serialized candidate the engine produced.
    Candidate,
}

/// The candidate's findings split by provenance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidateChecks {
    /// Findings the input already carried; disclosed, never blocking on
    /// their own.
    pub baseline: Vec<Finding>,
    /// Findings the edit introduced. Only blocking ones fail preparation;
    /// the rest are disclosed here.
    pub introduced: Vec<Finding>,
    /// Findings persisting in either validated representation, deduplicated by identity.
    pub unresolved: Vec<Finding>,
    /// Both serialized representations passed the required integrity gate.
    pub integrity: bool,
    /// DOM-realized and serialized-story postconditions completed successfully.
    pub operation_postconditions: bool,
}

impl CandidateChecks {
    /// Report only completed checks and their computed findings.
    pub fn verification(&self) -> crate::result::Verification {
        let outcome = |ran| if ran { "passed" } else { "skipped" }.to_string();
        crate::result::Verification {
            integrity: outcome(self.integrity),
            operation_postconditions: outcome(self.operation_postconditions),
            baseline_findings: Some(self.baseline.clone()),
            introduced_findings: Some(self.introduced.clone()),
            unresolved_findings: Some(self.unresolved.clone()),
            ..Default::default()
        }
    }
}

/// Which serialized candidate is the prepared transaction's publication:
/// the clean copy (changes accepted) or the redline (changes tracked).
/// The role drives the bytes `prepare` chooses as `primary`, the digest
/// it derives, and the serialized-story postcondition it runs: validation
/// describes the chosen primary, not a reserialized package swapped in at
/// commit. An unchanged transaction always returns the original source
/// bytes, regardless of role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PrimaryRole {
    /// Publish the clean (untracked) candidate. The default — preserves
    /// the v1 helper callers, the Python and WASM bindings, and every
    /// caller that has not chosen a publication track yet.
    #[default]
    Clean,
    /// Publish the redline (tracked) candidate. The v1 typed CLI uses it;
    /// `prepare_v2` likewise publishes redline, by construction.
    Redline,
}

/// What to check beyond the shared candidate checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrepareOptions {
    /// Refuse `NO_CHANGES_REQUESTED` after the checks pass when the primary
    /// has no document effect. Renders, reports and other derivatives are
    /// not consulted: the condition judges the primary only.
    pub require_change: bool,
    /// Compare each `rewrite`'s requested text with the realized paragraph
    /// exactly: a request the word-level rewrite algorithm normalizes away
    /// is `POSTCONDITION_FAILED`, not an unchanged success. The v1 `rewrite`
    /// contract documents that normalization, so it stays off there; the v2
    /// exact-text codec sets it.
    pub exact_text: bool,
    /// Which serialized candidate `prepare` chooses as the publication.
    /// `Clean` (the default) keeps the helper-callers' contract, where the
    /// clean copy is the primary and the redline is a side artifact.
    /// `Redline` makes the redline the primary: the chosen bytes the
    /// serialized-story postcondition describes and the receipt digest
    /// names, so a redline-only defect cannot slip through.
    pub role: PrimaryRole,
    /// The request syntax a refusal hint quotes: a flags-built plan names
    /// flags, a JSON plan names JSON (#1108).
    pub syntax: super::PlanSyntax,
    /// `--skip-failed` (#1253): leave out every operation that fails, and
    /// every operation of its `group`, instead of refusing the plan; the
    /// rest is applied as a plan of its own. [`PreparedEdit::skipped`]
    /// lists what was left out. A failure no operation is blamed for is
    /// never skipped, and a plan whose every operation fails is refused.
    pub skip_failed: bool,
}

impl Default for PrepareOptions {
    fn default() -> Self {
        Self {
            require_change: false,
            exact_text: false,
            role: PrimaryRole::Clean,
            syntax: super::PlanSyntax::V1,
            skip_failed: false,
        }
    }
}

/// A plan applied and verified in memory, up to publication.
#[derive(Clone, Debug)]
pub struct PreparedEdit {
    /// The v1 artifacts: clean copy, redline and per-operation report.
    pub result: EditResult,
    /// SHA-256 of the original source.
    pub source_sha256: String,
    /// SHA-256 of the normalized plan JSON the engine applied.
    pub request_sha256: String,
    /// The chosen primary bytes, the same bytes `checks` describe.
    pub primary: Vec<u8>,
    /// Which bytes `primary` holds.
    pub primary_bytes: PrimaryBytes,
    /// SHA-256 of `primary`: output digests come from the checked bytes.
    pub primary_sha256: String,
    /// The primary transaction's document effect (`changed`/`unchanged`
    /// only; `not_applied` belongs to the dry-run envelope).
    pub primary_effect: Effect,
    /// Baseline and introduced findings on the primary's basis.
    pub checks: CandidateChecks,
    /// Source paragraph → chosen-primary correspondence for `--expect`
    /// (V2-13). `None` on the v1 path, whose published primary is a
    /// comparer redline with no tracked correspondence: a v1 plan plus
    /// `--expect` refuses instead of guessing.
    pub correspondence: Option<crate::expect::SourceToOutputCorrespondence>,
    /// The operations [`PrepareOptions::skip_failed`] left out, in plan
    /// order; empty otherwise.
    pub skipped: Vec<super::SkippedOperation>,
}

/// Run the shared preparation: resolve, apply, choose the primary bytes,
/// verify the serialized candidate, then apply `options` last. A dry run is
/// this same result without publication.
///
/// # Errors
///
/// Every error [`crate::edit::apply_plan`] reports, plus
/// `POSTCONDITION_FAILED` when a requested effect is not realized in the
/// serialized candidate (or, with [`PrepareOptions::exact_text`], when the
/// requested text differs from it), `CANDIDATE_INVALID` when the candidate
/// introduces a blocking finding, and `NO_CHANGES_REQUESTED` under
/// [`PrepareOptions::require_change`] when the primary is unchanged.
pub fn prepare(
    source: &[u8],
    plan: &EditPlan,
    options: &PrepareOptions,
) -> Result<PreparedEdit, EditError> {
    if options.skip_failed {
        // Each attempt comes back here with `skip_failed` off: the whole
        // plan, refused at its first failure.
        return super::skip::prepare_skipping(source, plan, options);
    }
    let known = SourceChecks::default();
    prepare_once(source, plan, options, &known).map_err(|failure| {
        // A v1 plan's operations are independent (`check_conflicts`):
        // any of them make a plan of their own.
        let mut reruns = Reruns::new(|ops: &[usize]| {
            let part = EditPlan {
                operations: ops.iter().map(|&i| plan.operations[i].clone()).collect(),
                ..plan.clone()
            };
            prepare_once(source, &part, options, &known)
                .err()
                .and_then(|f| f.refusal)
                .map(|(refusals, _)| refusals)
        })
        .knowing(plan.operations.len(), &failure);
        attributed(*failure, plan.operations.len(), true, &mut reruns)
    })
}

/// `plan` prepared once for `--skip-failed`, which may leave out only its
/// first `count` operations: the rest follow every rerun. A failure the
/// candidate checks did not raise comes back [`Blame::Plain`]; a candidate
/// refusal comes back with every defect attributed at once, the reruns
/// shared, so a retry never bisects again what this one did.
pub(super) fn prepare_blaming(
    source: &[u8],
    plan: &EditPlan,
    count: usize,
    options: &PrepareOptions,
) -> Result<PreparedEdit, Box<Blame>> {
    let known = SourceChecks::default();
    let failure = match prepare_once(source, plan, options, &known) {
        Ok(prepared) => return Ok(prepared),
        Err(failure) => failure,
    };
    let Some((refusals, outcomes)) = failure.refusal.clone() else {
        return Err(Box::new(Blame::Plain(failure.error)));
    };
    let mut reruns = Reruns::new(|ops: &[usize]| {
        let part = EditPlan {
            operations: ops
                .iter()
                .copied()
                .chain(count..plan.operations.len())
                .map(|i| plan.operations[i].clone())
                .collect(),
            ..plan.clone()
        };
        prepare_once(source, &part, options, &known)
            .err()
            .and_then(|f| f.refusal)
            .map(|(refusals, _)| refusals)
    })
    .knowing(count, &failure);
    let mut distinct: Vec<&Refusal> = Vec::new();
    for refusal in &refusals {
        if !distinct.iter().any(|d| d.same(refusal)) {
            distinct.push(refusal);
        }
    }
    // Each operation alone is one short preparation; a defect's bisection
    // about log2(count) long ones. With about as many defects as
    // operations over log2(count), the operations alone cost less: a
    // defect an operation meets alone is blamed on that one alone.
    let bits = (usize::BITS - count.leading_zeros()) as usize;
    let alone = distinct.len() * bits >= count;
    // Every defect, once: one the plan without operations meets too is
    // the plan's own, and no operation left out clears it.
    let mut blamed: Vec<(usize, EditError)> = Vec::new();
    for refusal in distinct {
        let single = alone
            .then(|| (0..count).find(|&i| reruns.reproduces(&[i], refusal)))
            .flatten()
            .map(|i| vec![i]);
        let Some(ops) =
            single.or_else(|| culprits(count, true, |ops| reruns.reproduces(ops, refusal)))
        else {
            return Err(Box::new(Blame::Candidate(
                attributed(*failure, count, true, &mut reruns),
                Vec::new(),
            )));
        };
        if !blamed.iter().any(|(i, _)| *i == ops[0]) {
            let error = advised(
                naming(refusal.error.clone(), &ops, &outcomes),
                failure.advice,
            );
            blamed.push((ops[0], error));
        }
    }
    // Each blamed operation carries its own error; the plan's is the
    // refusal unattributed.
    Err(Box::new(Blame::Candidate(failure.error, blamed)))
}

/// Why [`prepare_blaming`] refused a plan.
pub(super) enum Blame {
    /// A failure the candidate checks did not raise: its error, as
    /// [`prepare`] reports it.
    Plain(EditError),
    /// A candidate refusal: the error [`prepare`] reports, and each
    /// operation (plan index) a defect is blamed on, with the error that
    /// names it for that defect, in the order the candidate checks met
    /// them. Empty when a defect is the plan's own.
    Candidate(EditError, Vec<(usize, EditError)>),
}

/// What the candidate checks read in the source, read once for a plan and
/// every rerun of it.
#[derive(Default)]
struct SourceChecks {
    /// `validate::validate(source)`, `None` when unreadable.
    findings: std::cell::OnceCell<Option<Vec<Finding>>>,
    /// `validate::revision_id_occurrences(source)`, `None` when unreadable.
    revision_ids: std::cell::OnceCell<Option<HashMap<(String, String), usize>>>,
}

/// One defect a candidate gate refuses, as the attribution reruns compare
/// it: the artifact refused, the finding's code (or the gate's own), its
/// part, and its subject, which tells it from another defect of that code
/// in that part and stays put when a rerun leaves operations out (a
/// source's revision id, a bookmark's name; never an index or a minted
/// id). `error` refuses the candidate for this defect.
#[derive(Clone, Debug)]
struct Refusal {
    artifact: &'static str,
    code: String,
    part: String,
    subject: String,
    error: EditError,
}

impl Refusal {
    /// The same defect: `error` aside, as reruns find it again.
    fn same(&self, other: &Self) -> bool {
        self.artifact == other.artifact
            && self.code == other.code
            && self.part == other.part
            && self.subject == other.subject
    }
}

/// A candidate gate's refusal: every defect it met, the one its error
/// reports first. Never empty.
type Refused = Vec<Refusal>;

/// The refusal of one defect: `CANDIDATE_INVALID` with `message`.
fn refused(
    artifact: &'static str,
    code: &str,
    part: &str,
    subject: String,
    message: String,
) -> Refusal {
    Refusal {
        artifact,
        code: code.to_string(),
        part: part.to_string(),
        subject,
        error: err("CANDIDATE_INVALID", None, message),
    }
}

/// A failed preparation, and the candidate refusal behind it with every
/// operation's outcome, when the candidate checks refused it. `advice`
/// closes the message, after the operations [`attributed`] names.
struct Failure {
    error: EditError,
    refusal: Option<(Refused, Vec<EditOutcome>)>,
    advice: Option<&'static str>,
}

/// The candidate refusals reruns of some of a plan's operations meet, each
/// set of operations prepared once.
struct Reruns<F> {
    rerun: F,
    met: HashMap<Vec<usize>, Option<Refused>>,
}

impl<F: FnMut(&[usize]) -> Option<Refused>> Reruns<F> {
    fn new(rerun: F) -> Self {
        Self {
            rerun,
            met: HashMap::new(),
        }
    }

    /// `failure`'s refusal as what the whole plan of `count` operations
    /// meets: a rerun of them all is the preparation that failed.
    fn knowing(mut self, count: usize, failure: &Failure) -> Self {
        if let Some((refusals, _)) = &failure.refusal {
            self.met
                .insert((0..count).collect(), Some(refusals.clone()));
        }
        self
    }

    /// Whether the plan's operations `ops` alone meet `refusal`'s defect
    /// again, among every defect they meet.
    fn reproduces(&mut self, ops: &[usize], refusal: &Refusal) -> bool {
        let rerun = &mut self.rerun;
        self.met
            .entry(ops.to_vec())
            .or_insert_with(|| rerun(ops))
            .as_ref()
            .is_some_and(|met| met.iter().any(|r| r.same(refusal)))
    }
}

/// `error` with `advice` closing its message.
fn advised(mut error: EditError, advice: Option<&'static str>) -> EditError {
    if let Some(advice) = advice {
        error.message = format!("{}; {advice}", error.message);
    }
    error
}

impl From<EditError> for Box<Failure> {
    fn from(error: EditError) -> Self {
        Box::new(Failure {
            error,
            refusal: None,
            advice: None,
        })
    }
}

/// `failure`'s error, naming the operations that cause the defect its
/// candidate refusal reports first. `reruns` prepares only some of the
/// plan's operations and tells whether they meet that defect again;
/// [`culprits`] finds the operations from it, and [`naming`] names them. A
/// failure the candidate checks did not raise, or one the plan without
/// operations meets too, stays the plan's own (`operation` unset).
fn attributed<F: FnMut(&[usize]) -> Option<Refused>>(
    failure: Failure,
    count: usize,
    together: bool,
    reruns: &mut Reruns<F>,
) -> EditError {
    let Failure {
        error,
        refusal,
        advice,
    } = failure;
    let Some((refusals, outcomes)) = refusal else {
        return advised(error, advice);
    };
    let Some(first) = refusals.first() else {
        return advised(error, advice);
    };
    let Some(ops) = culprits(count, together, |ops| reruns.reproduces(ops, first)) else {
        return advised(error, advice);
    };
    advised(naming(error, &ops, &outcomes), advice)
}

/// `error` naming the operations `ops` (plan indices, the one that brings
/// the defect in first): the first becomes its `operation`, and the
/// message names each by its id and target, from `outcomes`, or by its
/// pointer (`/operations/N`) when `outcomes` has none.
fn naming(mut error: EditError, ops: &[usize], outcomes: &[EditOutcome]) -> EditError {
    let name = |i: usize| match outcomes.get(i) {
        Some(EditOutcome {
            id,
            kind,
            paragraph: Some(paragraph),
            ..
        }) => format!("{id} ({kind} on {paragraph})"),
        Some(EditOutcome { id, kind, .. }) => format!("{id} ({kind})"),
        None => format!("/operations/{i}"),
    };
    let Some(&cause) = ops.first() else {
        return error;
    };
    error.operation = Some(match outcomes.get(cause) {
        Some(outcome) => outcome.id.clone(),
        None => format!("/operations/{cause}"),
    });
    error.message = format!("{}; {} introduces it", error.message, name(cause));
    if let [_, others @ ..] = ops
        && let Some((&last, rest)) = others.split_last()
    {
        let mut message = format!("{}, together with ", error.message);
        for (n, &i) in rest.iter().enumerate() {
            if n > 0 {
                message.push_str(", ");
            }
            message.push_str(&name(i));
        }
        if !rest.is_empty() {
            message.push_str(" and ");
        }
        message.push_str(&name(last));
        error.message = message;
    }
    error
}

/// The operations (plan indices) that cause a refusal the whole plan of
/// `count` operations meets: `reproduces(ops)` tells whether the plan's
/// operations `ops` alone, in plan order, meet it again. The first is the
/// operation that brings it in: the first prefix of the plan that
/// reproduces it ends there, the prefix before it does not. Under
/// `together` (operations independent of each other), the others are the
/// earlier operations that one needs, when it does not reproduce the
/// refusal alone, in plan order, each found the same way. `None` when the
/// plan without operations reproduces it: no operation causes it.
///
/// Prefixes are bisected, so the reruns are logarithmic in `count`, once
/// for the first operation and once for each other one.
fn culprits(
    count: usize,
    together: bool,
    mut reproduces: impl FnMut(&[usize]) -> bool,
) -> Option<Vec<usize>> {
    if count == 0 || reproduces(&[]) {
        return None;
    }
    // Every prefix is a slice of this one list, built once.
    let all: Vec<usize> = (0..count).collect();
    // `lo` operations do not reproduce it; `hi` do: the whole plan does.
    let (mut lo, mut hi) = (0, count);
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if reproduces(&all[..mid]) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let first = hi - 1;
    let mut needed = vec![first];
    if !together {
        return Some(needed);
    }
    let with = |pool: &[usize], needed: &[usize]| {
        let mut ops: Vec<usize> = pool.iter().chain(needed).copied().collect();
        ops.sort_unstable();
        ops
    };
    // The candidates: the operations before the last one found. With all of
    // them `needed` reproduces it (the whole prefix through `first` at the
    // start, the prefix the last search ended on after it); with none,
    // `needed` alone does not.
    let mut pool = all[..first].to_vec();
    while !reproduces(&with(&[], &needed)) {
        let (mut lo, mut hi) = (0, pool.len());
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if reproduces(&with(&pool[..mid], &needed)) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let Some(&next) = pool.get(hi.wrapping_sub(1)) else {
            break;
        };
        needed.push(next);
        pool.truncate(hi - 1);
    }
    needed[1..].sort_unstable();
    Some(needed)
}

/// [`prepare`] without `skip_failed` and without attribution: the failure
/// keeps the candidate refusal, if one was met, for [`attributed`].
/// `known` holds what the candidate checks read in `source`.
fn prepare_once(
    source: &[u8],
    plan: &EditPlan,
    options: &PrepareOptions,
    known: &SourceChecks,
) -> Result<PreparedEdit, Box<Failure>> {
    #[cfg(test)]
    tests::PREPARATIONS.with(|count| count.set(count.get() + 1));
    check_update_fields(plan)?;
    let lowered = super::visible::lower(source, plan)?;
    let plan = lowered.as_ref().unwrap_or(plan);
    let mut tx = Transaction::start(source, plan)?;
    tx.syntax = options.syntax;
    tx.resolve()?;
    tracked::check(&tx)?;
    let emit = tx.emits_directly()?;
    // `update_fields` compares an edit beside a range marker (#1156): a
    // refused candidate then names the route that keeps the marker.
    let compared_beside = !emit && plan.update_fields && tx.edits_beside_a_range_marker();
    tx.apply()?;
    check_dom_realized(&tx, options.exact_text)?;
    let result = produce(&mut tx, emit, options.role)?;
    let effect = primary_effect(&tx, &result)?;
    let (primary, primary_bytes) = match effect {
        Effect::Unchanged => (source.to_vec(), PrimaryBytes::Source),
        _ => match options.role {
            PrimaryRole::Clean => (result.clean.clone(), PrimaryBytes::Candidate),
            PrimaryRole::Redline => (result.redline.clone(), PrimaryBytes::Candidate),
        },
    };
    let mut checks = screen_candidates(source, &result, known)
        .and_then(|checks| {
            if compared_beside {
                check_rejected_ranges(source, &result.redline)?;
            }
            Ok(checks)
        })
        .map_err(|refusals| {
            Box::new(Failure {
                error: refusals[0].error.clone(),
                refusal: Some((refusals, tx.outcomes.clone())),
                advice: compared_beside.then_some(
                    "update_fields compares an edit beside a bookmark, comment or permission range, so drop update_fields and refresh the fields with `jubarte convert FILE --update-fields -o OUT.docx` afterwards",
                ),
            })
        })?;
    checks.operation_postconditions = true;
    if options.require_change && effect == Effect::Unchanged {
        return Err(err(
            codes::NO_CHANGES_REQUESTED,
            None,
            "the primary transaction has no document effect; --require-change was given",
        )
        .into());
    }
    Ok(PreparedEdit {
        result,
        source_sha256: source_sha256(source),
        request_sha256: source_sha256(plan.to_json().as_bytes()),
        primary_sha256: source_sha256(&primary),
        primary,
        primary_bytes,
        primary_effect: effect,
        checks,
        correspondence: None,
        skipped: Vec::new(),
    })
}

/// The primary transaction's document effect. Aggregated text equality
/// alone never proves `unchanged`: revision resolution, comments, thread
/// state, formatting, media, fields and structure are effects too.
fn primary_effect(tx: &Transaction<'_>, result: &EditResult) -> Result<Effect, EditError> {
    let report = &result.report;
    if !report.resolved_revisions.is_empty()
        || report.revisions.total > 0
        || report.paragraphs.from != report.paragraphs.to
        || !report.fields.is_empty()
        || report.comments_added > 0
        || tx.watermark.is_some()
        || tx.settings.is_some()
        || tx.notes_story.is_some()
    {
        return Ok(Effect::Changed);
    }
    for (op, resolved) in &tx.resolved {
        // Only plain text operations can be truly no-effect; a comment,
        // thread, formatting, control, table or structural resolution is an
        // effect even when the aggregated text is equal. A `redact`
        // resolves as text but empties the base too, so the diff misses it.
        if !matches!(resolved, Resolved::Text { .. })
            || !matches!(
                tx.plan.operations[*op].kind,
                OperationKind::Replace { .. }
                    | OperationKind::Insert { .. }
                    | OperationKind::Delete { .. }
                    | OperationKind::Rewrite { .. }
            )
        {
            return Ok(Effect::Changed);
        }
    }
    // A text edit that meant to change the text is an effect even when the
    // engine wrote nothing for it.
    if expected_texts(tx, false)?.into_iter().any(|(p, e)| {
        matches!(e.scope, ExpectedScope::Paragraph) && e.text != tx.projections[p].text
    }) {
        return Ok(Effect::Changed);
    }
    Ok(Effect::Unchanged)
}

/// Reject All of `redline` gives the source back, so it keeps each of the
/// source's bookmarks (by name; Word's own `_GoBack` aside), comment ranges
/// and permission ranges. Compare places them by the words around them and
/// can carry one into an insertion, which Reject All then removes: the
/// TOC entry linking to a heading's `_Toc` bookmark reads "Error! Bookmark
/// not defined." (#1156).
fn check_rejected_ranges(source: &[u8], redline: &[u8]) -> Result<(), Refused> {
    let rejected = crate::document_comparer::reject_revisions(redline).map_err(|e| {
        vec![refused(
            "redline",
            "REJECT_FAILED",
            "",
            String::new(),
            format!("the redline cannot be rejected: {e}"),
        )]
    })?;
    let (before, after) = (range_markers(source), range_markers(&rejected));
    let lost = before
        .bookmarks
        .difference(&after.bookmarks)
        .map(|name| format!("bookmark {name}"))
        .chain((after.comments < before.comments).then(|| "a comment's range".to_string()))
        .chain((after.permissions < before.permissions).then(|| "a permission range".to_string()))
        .collect::<Vec<_>>();
    if lost.is_empty() {
        return Ok(());
    }
    // The ranges lost, together, are the defect: a rerun that loses only
    // some of them meets another.
    let lost = lost.join(", ");
    Err(vec![refused(
        "redline",
        "RANGE_LOST",
        "",
        lost.clone(),
        format!("Reject All of the redline drops {lost} the source holds"),
    )])
}

/// The range markers of a package's stories: bookmark names, and how many
/// comment and permission ranges open.
#[derive(Default)]
struct RangeMarkers {
    bookmarks: std::collections::BTreeSet<String>,
    comments: usize,
    permissions: usize,
}

fn range_markers(docx: &[u8]) -> RangeMarkers {
    let mut markers = RangeMarkers::default();
    let Ok(opened) = crate::inspect::Opened::open(docx) else {
        return markers;
    };
    let mut parts = vec![opened.main.clone()];
    parts.extend(opened.story_parts().into_iter().map(|(_, _, part)| part));
    for part in parts {
        let Ok((dom, _, root)) = crate::inspect::parse_part(&opened.pkg, &part) else {
            continue;
        };
        for start in dom.descendants(root, Some(&W::bookmark_start())) {
            if let Some(name) = dom.attribute(start, &W::name("name"))
                && name != "_GoBack"
            {
                markers.bookmarks.insert(name.to_string());
            }
        }
        markers.comments += dom.descendants(root, Some(&W::comment_range_start())).len();
        markers.permissions += dom.descendants(root, Some(&W::perm_start())).len();
    }
    markers
}

/// Findings on the input and what each serialized artifact adds. The
/// baseline discloses without blocking; an introduced blocking finding
/// fails preparation.
#[cfg(test)]
fn check_candidates(source: &[u8], result: &EditResult) -> Result<CandidateChecks, EditError> {
    screen_candidates(source, result, &SourceChecks::default())
        .map_err(|refusals| refusals[0].error.clone())
}

/// The candidate gates, with every defect they meet (the test-only
/// `check_candidates` keeps the first), in
/// the order they meet them: the clean copy's, then the redline's; in each,
/// the duplicate revision ids, then the introduced blocking findings. The
/// first is the one the refusal reports, as when the gates stopped there;
/// the others let a rerun tell whether it meets that defect again.
/// `known` holds what the gates read in `source`.
fn screen_candidates(
    source: &[u8],
    result: &EditResult,
    known: &SourceChecks,
) -> Result<CandidateChecks, Refused> {
    // A source the validator cannot read gives no trustworthy baseline:
    // an empty one would call every candidate finding introduced, so with
    // no baseline nothing is claimed introduced (the candidate's own
    // validation below still gates what the edit produced).
    let baseline = known
        .findings
        .get_or_init(|| validate::validate(source).ok())
        .as_ref();
    let mut refusals = Vec::new();
    let mut introduced = Vec::new();
    let mut unresolved = Vec::new();
    for (artifact, bytes) in [("clean copy", &result.clean), ("redline", &result.redline)] {
        // An unreadable artifact ends the checks; the defects met before
        // it still come first.
        let findings = match validate::validate(bytes) {
            Ok(findings) => findings,
            Err(e) => {
                refusals.push(refused(
                    artifact,
                    "UNREADABLE",
                    "",
                    String::new(),
                    format!("the serialized candidate cannot be validated: {e}"),
                ));
                return Err(refusals);
            }
        };
        // Counted only when a candidate repeats an id at all: its own
        // `DUPLICATE_REVISION_ID` reads the same parts, records and keys.
        if findings.iter().any(|f| f.code == "DUPLICATE_REVISION_ID")
            && let Some(source_ids) = known
                .revision_ids
                .get_or_init(|| validate::revision_id_occurrences(source).ok())
        {
            match check_revision_ids(source_ids, artifact, bytes) {
                Ok(duplicates) => refusals.extend(duplicates),
                Err(unreadable) => {
                    refusals.extend(unreadable);
                    return Err(refusals);
                }
            }
        }
        for finding in &findings {
            if !unresolved
                .iter()
                .any(|f| finding_key(f) == finding_key(finding))
            {
                unresolved.push(finding.clone());
            }
        }
        let Some(baseline) = baseline else {
            continue;
        };
        for finding in introduced_findings(baseline, &findings) {
            if blocking(&finding) {
                refusals.push(refused(
                    artifact,
                    &finding.code,
                    &finding.part,
                    finding.unnumbered(),
                    format!(
                        "the edit introduces {} in {}#{}: {}",
                        finding.code, finding.part, finding.location, finding.message
                    ),
                ));
                continue;
            }
            if !introduced
                .iter()
                .any(|f| finding_key(f) == finding_key(&finding))
            {
                introduced.push(finding);
            }
        }
    }
    if !refusals.is_empty() {
        return Err(refusals);
    }
    Ok(CandidateChecks {
        baseline: baseline.cloned().unwrap_or_default(),
        introduced,
        unresolved,
        integrity: true,
        operation_postconditions: false,
    })
}

/// A candidate may not give a revision id a same-kind duplicate the source
/// did not have: per part, record type and id, an id that occurs more than
/// once may occur no more often than in the source. Inherited duplicates
/// pass, healing one hides no collision under another, and kinds may share
/// an id. `jubarte validate` still calls the duplicate non-fatal: Word
/// opens it; publication refuses it. `artifact` names the candidate; every
/// duplicate it introduces is one defect, in key order. An id the source
/// holds is the defect's subject; one the edit minted is not, since a rerun
/// of fewer operations mints others.
fn check_revision_ids(
    source: &HashMap<(String, String), usize>,
    artifact: &'static str,
    candidate: &[u8],
) -> Result<Vec<Refusal>, Refused> {
    let occurrences = validate::revision_id_occurrences(candidate).map_err(|e| {
        vec![refused(
            artifact,
            "UNREADABLE",
            "",
            String::new(),
            format!("the serialized candidate cannot be read: {e}"),
        )]
    })?;
    let mut introduced: Vec<_> = occurrences
        .iter()
        .filter(|&(key, &count)| count > 1 && count > source.get(key).copied().unwrap_or(0))
        .collect();
    introduced.sort();
    Ok(introduced
        .into_iter()
        .map(|(entry, &count)| {
            let (part, key) = entry;
            let mut fields = key.splitn(3, ':');
            let (kind, mark, id) = (
                fields.next().unwrap_or_default(),
                fields.next() == Some("true"),
                fields.next().unwrap_or_default(),
            );
            let mark = if mark { " on a mark" } else { "" };
            let had = source.get(entry).copied().unwrap_or(0);
            let subject = if had > 0 {
                format!("w:{kind}{mark} w:id {id}")
            } else {
                format!("w:{kind}{mark}")
            };
            refused(
                artifact,
                "DUPLICATE_REVISION_ID",
                part,
                subject,
                format!(
                    "the edit introduces a duplicate revision id in the {artifact}: w:{kind}{mark} w:id {id} occurs {count} times in {part} (the source has {had})"
                ),
            )
        })
        .collect())
}

/// What one paragraph reads as once the plan applies, and whether symbol
/// placeholders are out of the comparison.
struct ExpectedText {
    /// The operation that defined it (`op-N`), for error reports.
    op: String,
    /// The expected exact logical text.
    text: String,
    /// Compare without U+FFFC placeholders: a `rewrite` keeps the
    /// paragraph's symbols wherever they sit.
    ignore_symbols: bool,
    /// What `text` covers.
    scope: ExpectedScope,
}

/// The part of a paragraph an [`ExpectedText`] covers.
#[derive(Clone, Copy)]
enum ExpectedScope {
    /// The paragraph's projected text.
    Paragraph,
    /// The content of a content control: the value indexes
    /// `Transaction::controls`. A `fill_control` replaces its control's
    /// projected text, which the span composition cannot predict, so the
    /// fill's own requested value is the postcondition instead.
    Control(usize),
}

/// The exact text each paragraph must read as in the candidate: resolution
/// composed over the base projection, or, under `exact`, the `rewrite`'s
/// requested text itself — so a requested change the algorithm normalizes
/// away is detected instead of passing as a no-op.
fn expected_texts(
    tx: &Transaction<'_>,
    exact: bool,
) -> Result<Vec<(usize, ExpectedText)>, EditError> {
    /// `para` → (first op id, `(start, end, replacement, op, restore,
    /// attach_before)` spans).
    type Spans = (String, Vec<(usize, usize, String, usize, bool, bool)>);
    let mut edits: BTreeMap<usize, Spans> = BTreeMap::new();
    let mut merges = Vec::new();
    let mut filled: HashSet<usize> = HashSet::new();
    // `(para, control, requested text, op id)` — a fill's own postcondition.
    let mut fills = Vec::new();
    for (op, resolved) in &tx.resolved {
        match resolved {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                attach_before,
                ..
            } => {
                let entry = edits
                    .entry(*para)
                    .or_insert_with(|| (tx.outcomes[*op].id.clone(), Vec::new()));
                entry.1.push((
                    *start,
                    *end,
                    replacement.clone(),
                    *op,
                    false,
                    *attach_before,
                ));
            }
            // A restore's copy is text inserted where the deletion is.
            Resolved::Restore { para, at, text, .. } => {
                let entry = edits
                    .entry(*para)
                    .or_insert_with(|| (tx.outcomes[*op].id.clone(), Vec::new()));
                entry.1.push((*at, *at, text.clone(), *op, true, false));
            }
            Resolved::MergeParagraphs {
                para,
                next,
                separator,
                ..
            } => {
                merges.push((*para, *next, separator.clone(), tx.outcomes[*op].id.clone()));
            }
            Resolved::FillControl {
                control,
                paragraphs,
                value,
                ..
            } => {
                let text = match value {
                    controls::FillValue::Text(text) | controls::FillValue::Date { text, .. } => {
                        text.clone()
                    }
                    controls::FillValue::Checked { glyph, .. } => glyph.to_string(),
                };
                for para in paragraphs {
                    fills.push((*para, *control, text.clone(), tx.outcomes[*op].id.clone()));
                }
                filled.extend(paragraphs.iter().copied());
            }
            _ => {}
        }
    }
    let mut expected = BTreeMap::new();
    for (para, (op, mut spans)) in edits {
        let rewrite = if exact { rewrite_text(tx, para) } else { None };
        let rewrite_op = rewrite.as_ref().map(|&(op, _)| op);
        let base = &tx.projections[para].text;
        // A restore's copy follows its deletion, ahead of the visible text
        // that starts where the deletion is; other spans keep their order,
        // but in an empty paragraph the start edge's text comes first, as
        // `sort_edits` applies it (#1303).
        let empty = base.is_empty();
        // Match the writers' source-range order: an insertion at a removal's
        // start precedes that removal, independent of plan order (#1922).
        spans.sort_by_key(|&(start, end, _, op, restore, after)| {
            (start, !restore, end, empty && after, op)
        });
        let mut text = String::with_capacity(base.len());
        // The rewrite's own spans alone, composed on the base: a rewrite
        // beside co-resident edits must still realize its request, or the
        // failure is the rewrite's, not the composed text's.
        let mut own = String::with_capacity(base.len());
        let mut at = 0;
        let mut own_at = 0;
        let mut previous_op = None;
        for (start, end, replacement, span_op, ..) in spans {
            let conflict = || {
                let previous = previous_op.unwrap_or(span_op);
                tx.conflict(
                    span_op,
                    &format!(
                        "{} and {} have text ranges that cannot be composed in p{para}",
                        tx.outcomes[previous].id, tx.outcomes[span_op].id
                    ),
                )
            };
            // Check every bound, including UTF-8 boundaries. Even an invalid
            // resolved span must refuse rather than reach slice_error_fail.
            let gap = base.get(at..start).ok_or_else(conflict)?;
            base.get(start..end).ok_or_else(conflict)?;
            text.push_str(gap);
            text.push_str(&replacement);
            at = end;
            if Some(span_op) == rewrite_op {
                own.push_str(base.get(own_at..start).ok_or_else(conflict)?);
                own.push_str(&replacement);
                own_at = end;
            }
            previous_op = Some(span_op);
        }
        // Both cursors came from a checked span (or remain at zero).
        text.push_str(base.get(at..).ok_or_else(|| {
            tx.conflict(
                previous_op.unwrap_or(0),
                "text range ends outside the paragraph",
            )
        })?);
        own.push_str(base.get(own_at..).ok_or_else(|| {
            tx.conflict(
                rewrite_op.unwrap_or(0),
                "rewrite range ends outside the paragraph",
            )
        })?);
        if let Some((rewrite_op, requested)) = rewrite {
            let entry = if own == requested {
                // The rewrite's own edits realize its request: the
                // postcondition is the full composition — the request with
                // every co-resident edit applied — never the bare request
                // discarding a neighbour's effect.
                ExpectedText {
                    op,
                    text,
                    ignore_symbols: true,
                    scope: ExpectedScope::Paragraph,
                }
            } else {
                ExpectedText {
                    op: tx.outcomes[rewrite_op].id.clone(),
                    text: requested,
                    ignore_symbols: true,
                    scope: ExpectedScope::Paragraph,
                }
            };
            expected.insert(para, entry);
            continue;
        }
        expected.insert(
            para,
            ExpectedText {
                op,
                text,
                ignore_symbols: false,
                scope: ExpectedScope::Paragraph,
            },
        );
    }
    // A rewrite that resolved to no edits still has a postcondition.
    if exact {
        for (i, op) in tx.plan.operations.iter().enumerate() {
            let OperationKind::Rewrite {
                paragraph, text, ..
            } = &op.kind
            else {
                continue;
            };
            if comment_text_of(paragraph).is_some() {
                continue;
            }
            let Some(para) = outcome_paragraph(tx, i) else {
                continue;
            };
            expected.entry(para).or_insert_with(|| ExpectedText {
                op: tx.outcomes[i].id.clone(),
                text: text.clone(),
                ignore_symbols: true,
                scope: ExpectedScope::Paragraph,
            });
        }
    }
    for (para, next, separator, op) in merges {
        let survivor = expected
            .get(&next)
            .map(|e| e.text.clone())
            .unwrap_or_else(|| tx.projections[next].text.clone());
        let first = expected
            .get(&para)
            .map(|e| e.text.clone())
            .unwrap_or_else(|| tx.projections[para].text.clone());
        expected.insert(
            next,
            ExpectedText {
                op,
                text: format!("{first}{separator}{survivor}"),
                ignore_symbols: false,
                scope: ExpectedScope::Paragraph,
            },
        );
        expected.remove(&para);
    }
    for para in filled {
        expected.remove(&para);
    }
    let mut out: Vec<(usize, ExpectedText)> = expected.into_iter().collect();
    for (para, control, text, op) in fills {
        out.push((
            para,
            ExpectedText {
                op,
                text,
                ignore_symbols: false,
                scope: ExpectedScope::Control(control),
            },
        ));
    }
    out.sort_by_key(|(para, _)| *para);
    Ok(out)
}

/// The first `rewrite` op on `para` and the text it asks for, or `None`
/// when no rewrite op resolved onto it (`cN.text` rewrites excluded: their
/// target is the covered span, not the paragraph).
fn rewrite_text(tx: &Transaction<'_>, para: usize) -> Option<(usize, String)> {
    tx.plan.operations.iter().enumerate().find_map(|(i, op)| {
        let OperationKind::Rewrite {
            paragraph, text, ..
        } = &op.kind
        else {
            return None;
        };
        if comment_text_of(paragraph).is_some() {
            return None;
        }
        (outcome_paragraph(tx, i) == Some(para)).then(|| (i, text.clone()))
    })
}

/// The paragraph index an outcome names (`p3`, `header1.p1`, …) inside the
/// transaction's story list.
fn outcome_paragraph(tx: &Transaction<'_>, op: usize) -> Option<usize> {
    let id = tx.outcomes[op].paragraph.as_deref()?;
    let (story, index) = crate::ids::paragraph_place(id)?;
    let story = tx.stories.iter().position(|s| s.id == story)?;
    tx.paragraph_story
        .iter()
        .position(|&(s, i)| s == story && i == index)
}

/// `para`'s node still belongs to its story in the mutated DOM.
fn attached(tx: &Transaction<'_>, para: usize) -> bool {
    let node = tx.paragraph_nodes[para];
    let (story, _) = tx.paragraph_story[para];
    let root = tx.stories[story].root;
    node == root || tx.opened.dom.ancestors(node, None).contains(&root)
}

/// Each expected paragraph reads exactly so after apply, in the DOM the
/// candidate is serialized from. A realized difference is
/// `POSTCONDITION_FAILED` with the plan's outcomes, never a silent miss.
pub(super) fn check_dom_realized(tx: &Transaction<'_>, exact: bool) -> Result<(), EditError> {
    for (para, expected) in expected_texts(tx, exact)? {
        if !attached(tx, para) {
            // Merged-away or deleted paragraphs carry no text postcondition;
            // merge survivors are checked on their own entry.
            continue;
        }
        let realized = match expected.scope {
            ExpectedScope::Paragraph => {
                project_paragraph(&tx.opened.dom, tx.paragraph_nodes[para]).text
            }
            ExpectedScope::Control(control) => control_text(&tx.opened.dom, tx.controls[control]),
        };
        let (want, got) = if expected.ignore_symbols {
            (
                expected.text.replace('\u{FFFC}', ""),
                realized.replace('\u{FFFC}', ""),
            )
        } else {
            (expected.text.clone(), realized)
        };
        if want != got {
            let mut e = err(
                codes::POSTCONDITION_FAILED,
                Some(&expected.op),
                format!(
                    "the requested text was not realized in paragraph {}",
                    crate::ids::paragraph_id(
                        &tx.stories[tx.paragraph_story[para].0].id,
                        tx.paragraph_story[para].1,
                    ),
                ),
            );
            e.outcomes = tx.outcomes.clone();
            return Err(e);
        }
    }
    Ok(())
}

/// The projected text of a content control (`w:sdtContent` content).
fn control_text(dom: &crate::xmllinq::Dom, sdt: crate::xmllinq::NodeId) -> String {
    dom.element(sdt, &crate::namespaces::W::sdt_content())
        .map(|content| project_paragraph(dom, content).text)
        .unwrap_or_default()
}

/// The serialized paragraphs of every story the plan wrote read exactly
/// as the DOM they were written from — the body always, plus each header,
/// footer and notes story an operation touched: a character the writer
/// dropped or restated is `POSTCONDITION_FAILED`.
pub(super) fn check_serialized_stories(
    tx: &Transaction<'_>,
    clean: &[u8],
) -> Result<(), EditError> {
    let post = body_paragraph_nodes(&tx.opened.dom, tx.opened.body);
    let reopened = crate::inspect::paragraphs(clean)
        .map_err(|e| err("POSTCONDITION_FAILED", None, e.to_string()))?;
    if reopened.len() != post.len() {
        return Err(err(
            codes::POSTCONDITION_FAILED,
            None,
            format!(
                "the serialized candidate holds {} body paragraphs, the prepared document {}",
                reopened.len(),
                post.len(),
            ),
        ));
    }
    for (node, paragraph) in post.iter().zip(&reopened) {
        let expected = project_paragraph(&tx.opened.dom, *node).text;
        if expected != paragraph.text {
            return Err(err(
                codes::POSTCONDITION_FAILED,
                None,
                format!(
                    "the serialized candidate's paragraph {} reads differently than prepared",
                    paragraph.id,
                ),
            ));
        }
    }
    let written: Vec<usize> = tx
        .written_stories
        .iter()
        .copied()
        .filter(|&story| story != 0)
        .collect();
    if written.is_empty() {
        return Ok(());
    }
    let reopened = crate::inspect::stories(clean)
        .map_err(|e| err("POSTCONDITION_FAILED", None, e.to_string()))?;
    for story in written {
        let StoryPart { part, root, .. } = &tx.stories[story];
        let post = crate::inspect::story_paragraph_nodes(&tx.opened.dom, *root);
        let Some(reopened) = reopened.iter().find(|s| &s.part == part) else {
            return Err(err(
                codes::POSTCONDITION_FAILED,
                None,
                format!("the serialized candidate lost the {part} story"),
            ));
        };
        if reopened.paragraphs.len() != post.len() {
            return Err(err(
                codes::POSTCONDITION_FAILED,
                None,
                format!(
                    "the serialized candidate holds {} {part} paragraphs, the prepared document {}",
                    reopened.paragraphs.len(),
                    post.len(),
                ),
            ));
        }
        for (node, paragraph) in post.iter().zip(&reopened.paragraphs) {
            let expected = project_paragraph(&tx.opened.dom, *node).text;
            if expected != paragraph.text {
                return Err(err(
                    codes::POSTCONDITION_FAILED,
                    None,
                    format!(
                        "the serialized candidate's paragraph {} reads differently than prepared",
                        paragraph.id,
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Compare the published redline's accepted text with the final clean copy.
fn check_redline_stories(
    tx: &Transaction<'_>,
    clean: &[u8],
    redline: &[u8],
) -> Result<(), EditError> {
    fn compare(
        part: &str,
        clean: &[crate::inspect::Paragraph],
        redline: &[crate::inspect::Paragraph],
    ) -> Result<(), EditError> {
        if clean.len() != redline.len() {
            return Err(err(
                codes::POSTCONDITION_FAILED,
                None,
                format!(
                    "the accepted redline's {part} paragraph {} differs: {} paragraphs, the clean candidate {}",
                    clean.len().min(redline.len()),
                    redline.len(),
                    clean.len(),
                ),
            ));
        }
        for (expected, actual) in clean.iter().zip(redline) {
            if expected.text != actual.text {
                return Err(err(
                    codes::POSTCONDITION_FAILED,
                    None,
                    format!(
                        "the accepted redline's {part} paragraph {} reads differently than the clean candidate",
                        expected.id,
                    ),
                ));
            }
        }
        Ok(())
    }

    // Under keep, even the clean copy retains the source's revisions.
    // Settle both copies for comparison without changing the published bytes.
    let accept = |bytes: &[u8]| {
        crate::document_comparer::accept_revisions(bytes)
            .map_err(|e| err(codes::POSTCONDITION_FAILED, None, e.to_string()))
    };
    let clean = accept(clean)?;
    let redline = accept(redline)?;
    let paragraphs = |bytes: &[u8]| {
        crate::inspect::paragraphs(bytes)
            .map_err(|e| err(codes::POSTCONDITION_FAILED, None, e.to_string()))
    };
    compare(
        &tx.opened.main,
        &paragraphs(&clean)?,
        &paragraphs(&redline)?,
    )?;
    if tx.written_stories.iter().all(|&story| story == 0) {
        return Ok(());
    }
    let stories = |bytes: &[u8]| {
        crate::inspect::stories(bytes)
            .map_err(|e| err(codes::POSTCONDITION_FAILED, None, e.to_string()))
    };
    let clean = stories(&clean)?;
    let redline = stories(&redline)?;
    for &story in &tx.written_stories {
        if story == 0 {
            continue;
        }
        let part = &tx.stories[story].part;
        let expected = clean.iter().find(|s| &s.part == part);
        let actual = redline.iter().find(|s| &s.part == part);
        let (Some(expected), Some(actual)) = (expected, actual) else {
            return Err(err(
                codes::POSTCONDITION_FAILED,
                None,
                format!("the accepted candidates lost the {part} story at paragraph 0"),
            ));
        };
        compare(part, &expected.paragraphs, &actual.paragraphs)?;
    }
    Ok(())
}

/// Serialize and check the prepared DOM before field updates, then check
/// the published redline against the final clean candidate.
fn produce(
    tx: &mut Transaction<'_>,
    emit: bool,
    role: PrimaryRole,
) -> Result<EditResult, EditError> {
    let plan = tx.plan;
    let (mut clean, mut marked) = tx.finish()?;
    check_serialized_stories(tx, &clean)?;
    // A reworded or deleted comment makes the comparer's base a copy; the
    // emitter writes those threads itself, as under keep.
    if plan.existing_revisions == ExistingRevisions::Keep
        || (emit
            && (tx.edits_only_comments()
                || matches!(tx.commented_base()?, std::borrow::Cow::Borrowed(_))))
    {
        let result = tracked::result(tx, clean)?;
        tx.check_redactions(&[&result.clean, &result.redline])?;
        if role == PrimaryRole::Redline {
            check_redline_stories(tx, &result.clean, &result.redline)?;
        }
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
    // Accept All gives back `clean`: the comparer leaves what the plan did
    // not touch as the source has it (#1868).
    let settings = WmlComparerSettings {
        author_for_revisions: plan.author.clone(),
        date_time_for_revisions: tx.date.clone(),
        preserve_untouched: true,
        ..WmlComparerSettings::default()
    };
    let revised = marked.as_deref().unwrap_or(&clean);
    let base = tx.commented_base()?;
    let mut redline =
        crate::document_comparer::compare_documents_for_edit(&base, revised, &settings)
            .map_err(|e| err("COMPARE_FAILED", None, e.to_string()))?;
    if tx
        .resolved
        .iter()
        .any(|(_, r)| matches!(r, Resolved::PageSetup { .. }))
    {
        redline = sections::record_mid_changes(&redline, &base, &plan.author, &tx.date)?;
    }
    redline = super::ranges::restore(&redline, &base, revised, &plan.author)?;
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
        // Under keep the deletion is written as given, and its comment
        // wraps the words named, its reply the struck paragraph.
        let keep = match tx.syntax {
            super::PlanSyntax::Flags | super::PlanSyntax::Typed => "--existing-revisions keep",
            super::PlanSyntax::V1 | super::PlanSyntax::V2 => r#""existing_revisions": "keep""#,
        };
        let message = match plan.operations[op].kind {
            OperationKind::Delete { .. } => format!(
                "the comparer deleted the same words elsewhere instead, so the comment would sit on text that stays; pass {keep} to write the deletion as given, or delete without a comment"
            ),
            OperationKind::ReplyComment { .. } => format!(
                "the comparer deleted an identical paragraph instead, so the reply would sit on text that stays; pass {keep} to write the deletion as given"
            ),
            _ => "the comparer deleted an identical paragraph instead, so the comment would sit on text that stays; delete without a comment, or comment on a neighbouring paragraph".to_string(),
        };
        let mut error = tx.conflict(op, &message);
        error.code = "UNSUPPORTED_STRUCTURE".into();
        error.outcomes[op].code = Some(error.code.clone());
        return Err(error);
    }
    // A deleted text's comment wraps its deletion, as a strike's does.
    let lifted: Vec<u32> = tx
        .deletion_comments
        .iter()
        .filter(|&&(_, op)| matches!(plan.operations[op].kind, OperationKind::Delete { .. }))
        .map(|&(id, _)| id)
        .collect();
    redline = lift_deletion_comments(&redline, &lifted)?;
    // Settings are not revisions: the redline takes them as they are.
    if let Some(request) = &tx.settings {
        redline = crate::settings::apply_settings_to_docx(&redline, request)
            .map_err(|m| err("PACKAGE_WRITE", None, m))?;
    }
    redline = super::replacement_order::typing_order(&redline)?;
    tx.check_redactions(&[&clean, &redline])?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline);
    report.fields = fields;
    if role == PrimaryRole::Redline {
        check_redline_stories(tx, &clean, &redline)?;
    }
    Ok(EditResult {
        clean,
        redline,
        report,
    })
}

/// An [`EditError`] as `super::err` builds it.
fn err(code: &str, operation: Option<&str>, message: impl Into<String>) -> EditError {
    super::err(code, operation, message)
}

/// Prepare a PlanV2 through the existing source/working-baseline adapter and
/// the same serialized candidate checks as v1. Both inline and saved v2
/// requests use this entry point; tracking never selects a different primary.
///
/// # Errors
/// Returns the adapter's source, scope, baseline and effect errors, candidate
/// validation failures, or `NO_CHANGES_REQUESTED` when required.
pub fn prepare_v2(
    source: &[u8],
    plan: &super::v2::PlanV2,
    options: &PrepareOptions,
) -> Result<PreparedEdit, EditError> {
    let known = SourceChecks::default();
    prepare_v2_once(source, plan, options, &known).map_err(|failure| {
        // A v2 plan's operations apply in order, each on what the ones
        // before it left: only prefixes are plans of their own, so only
        // the operation that brings the refusal in is named.
        let mut reruns = Reruns::new(|ops: &[usize]| {
            let part = super::v2::PlanV2 {
                operations: ops.iter().map(|&i| plan.operations[i].clone()).collect(),
                ..plan.clone()
            };
            prepare_v2_once(source, &part, options, &known)
                .err()
                .and_then(|f| f.refusal)
                .map(|(refusals, _)| refusals)
        })
        .knowing(plan.operations.len(), &failure);
        attributed(*failure, plan.operations.len(), false, &mut reruns)
    })
}

/// [`prepare_v2`] without attribution, as [`prepare_once`] is to
/// [`prepare`].
fn prepare_v2_once(
    source: &[u8],
    plan: &super::v2::PlanV2,
    options: &PrepareOptions,
    known: &SourceChecks,
) -> Result<PreparedEdit, Box<Failure>> {
    // A v2 plan's refusals quote its JSON, unless a typed command built it.
    let syntax = if options.syntax == super::PlanSyntax::Typed {
        super::PlanSyntax::Typed
    } else {
        super::PlanSyntax::V2
    };
    let applied = super::v2_apply::apply_quoting(source, plan, syntax)?;
    let result = EditResult {
        clean: applied.clean,
        redline: applied.redline,
        report: applied.report,
    };
    let primary = result.redline.clone();
    let unchanged = primary == source;
    let effect = if unchanged {
        Effect::Unchanged
    } else {
        Effect::Changed
    };
    let mut checks = screen_candidates(source, &result, known).map_err(|refusals| {
        Box::new(Failure {
            error: refusals[0].error.clone(),
            // A v2 operation is named by its pointer, `/operations/N`.
            refusal: Some((refusals, Vec::new())),
            advice: None,
        })
    })?;
    checks.operation_postconditions = true;
    if options.require_change && unchanged {
        return Err(err(
            codes::NO_CHANGES_REQUESTED,
            None,
            "the primary transaction has no document effect; --require-change was given",
        )
        .into());
    }
    Ok(PreparedEdit {
        source_sha256: source_sha256(source),
        request_sha256: source_sha256(
            serde_json::to_string(plan)
                .expect("plan serializes")
                .as_bytes(),
        ),
        primary_sha256: source_sha256(&primary),
        primary,
        primary_bytes: if unchanged {
            PrimaryBytes::Source
        } else {
            PrimaryBytes::Candidate
        },
        primary_effect: effect,
        result,
        checks,
        correspondence: Some(applied.correspondence),
        skipped: Vec::new(),
    })
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) mod tests {
    use super::super::{
        EditReport, EditResult, ExistingRevisions, ParagraphDelta, deeper_boundary_fixture,
    };
    use super::*;

    fn result_of(clean: Vec<u8>) -> EditResult {
        EditResult {
            clean: clean.clone(),
            redline: clean,
            report: EditReport {
                schema_version: 1,
                ok: true,
                source_sha256: String::new(),
                base_sha256: String::new(),
                guarded: false,
                author: "T".into(),
                date: "2026-10-17T12:00:00Z".into(),
                existing_revisions: ExistingRevisions::Refuse,
                resolved_revisions: Default::default(),
                removed_comments: Vec::new(),
                kept_comments: Vec::new(),
                paragraph_ids: Vec::new(),
                split_revisions: Vec::new(),
                paragraphs: ParagraphDelta::default(),
                operations: Vec::new(),
                comments_added: 0,
                revisions: Default::default(),
                fields: Vec::new(),
                skipped: 0,
                notes: Vec::new(),
                output_sha256: None,
            },
        }
    }

    /// A source the full validator cannot read must not become an empty
    /// baseline that calls every candidate finding introduced: with no
    /// trustworthy baseline nothing is claimed introduced, while the
    /// candidate's own validation still runs (fail-safe disclosure, not
    /// a fabricated verdict).
    #[test]
    fn unvalidatable_source_never_claims_findings_introduced() {
        let orphan = deeper_boundary_fixture::docx(
            r#"<w:p><w:r><w:t>x</w:t></w:r><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        );
        let findings = crate::validate::validate(&orphan).unwrap();
        assert!(
            findings.iter().any(|f| f.code == "COMMENT_ANCHOR_ORPHAN"),
            "{findings:?}"
        );
        let checks = check_candidates(b"not a zip at all", &result_of(orphan)).unwrap();
        assert!(checks.introduced.is_empty(), "{:?}", checks.introduced);
        assert!(checks.baseline.is_empty());
    }

    /// A paragraph of revisions: each `(kind, id)` around one word.
    fn revisions(records: &[(&str, u32)]) -> Vec<u8> {
        let body: String = records
            .iter()
            .map(|&(kind, id)| match kind {
                "customXmlInsRangeStart" => format!(
                    r#"<w:customXmlInsRangeStart w:id="{id}" w:author="A" w:date="2026-10-17T09:00:00Z"/><w:r><w:t>w</w:t></w:r><w:customXmlInsRangeEnd w:id="{id}"/>"#
                ),
                "rPrChange" => format!(
                    r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="A" w:date="2026-10-17T09:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>w</w:t></w:r>"#
                ),
                _ => format!(
                    r#"<w:{kind} w:id="{id}" w:author="A" w:date="2026-10-17T09:00:00Z"><w:r><w:{t}>w</w:{t}></w:r></w:{kind}>"#,
                    t = if kind == "del" { "delText" } else { "t" }
                ),
            })
            .collect();
        deeper_boundary_fixture::docx(&format!("<w:p>{body}</w:p>"))
    }

    fn gate(
        source: &[(&str, u32)],
        candidate: &[(&str, u32)],
    ) -> Result<CandidateChecks, EditError> {
        check_candidates(&revisions(source), &result_of(revisions(candidate)))
    }

    /// `result` refuses `record` (`w:ins w:id 5`), repeated in `part` of
    /// the clean copy.
    fn refused_as_duplicate(result: Result<CandidateChecks, EditError>, record: &str, part: &str) {
        let error = result.unwrap_err();
        assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
        assert!(
            error
                .message
                .contains("introduces a duplicate revision id in the clean copy: ")
                && error.message.contains(&format!(": {record} occurs "))
                && error.message.contains(&format!(" times in {part} (")),
            "{error:?}"
        );
    }

    #[test]
    fn a_candidate_may_not_introduce_a_duplicate_revision_id() {
        refused_as_duplicate(
            gate(&[("ins", 5), ("ins", 6)], &[("ins", 5), ("ins", 5)]),
            "w:ins w:id 5",
            "word/document.xml",
        );
        refused_as_duplicate(
            gate(&[("rPrChange", 5)], &[("rPrChange", 5), ("rPrChange", 5)]),
            "w:rPrChange w:id 5",
            "word/document.xml",
        );
    }

    #[test]
    fn a_candidate_may_not_add_an_occurrence_to_an_inherited_duplicate() {
        refused_as_duplicate(
            gate(
                &[("ins", 5), ("ins", 5)],
                &[("ins", 5), ("ins", 5), ("ins", 5)],
            ),
            "w:ins w:id 5",
            "word/document.xml",
        );
    }

    #[test]
    fn healing_one_duplicate_revision_id_hides_no_new_one() {
        refused_as_duplicate(
            gate(
                &[("ins", 5), ("ins", 5)],
                &[("ins", 5), ("ins", 7), ("ins", 7)],
            ),
            "w:ins w:id 7",
            "word/document.xml",
        );
    }

    #[test]
    fn a_custom_xml_range_start_may_not_take_a_duplicate_id() {
        // A range start and its end share an id; two starts may not.
        assert!(
            gate(
                &[("customXmlInsRangeStart", 5)],
                &[("customXmlInsRangeStart", 5)]
            )
            .is_ok()
        );
        // Their ends repeat too, and come first by name (Codex on #1666).
        refused_as_duplicate(
            gate(
                &[("customXmlInsRangeStart", 5), ("customXmlInsRangeStart", 6)],
                &[("customXmlInsRangeStart", 5), ("customXmlInsRangeStart", 5)],
            ),
            "w:customXmlInsRangeEnd w:id 5",
            "word/document.xml",
        );
    }

    /// `bytes` with its main part's WordprocessingML namespace made Strict's.
    fn strict(bytes: &[u8]) -> Vec<u8> {
        let mut pkg = crate::opc::PartFs::open(bytes).unwrap();
        let xml = pkg.part_string("word/document.xml").unwrap().replace(
            "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
            "http://purl.oclc.org/ooxml/wordprocessingml/main",
        );
        pkg.set_part("word/document.xml", xml.into_bytes());
        pkg.to_zip().unwrap()
    }

    #[test]
    fn a_strict_sources_duplicates_are_inherited() {
        // The edit writes a Strict source's revisions in Transitional.
        let source = strict(&revisions(&[("ins", 5), ("ins", 5)]));
        let candidate = revisions(&[("ins", 5), ("ins", 5)]);
        let result = check_candidates(&source, &result_of(candidate));
        assert!(result.is_ok(), "{result:?}");
    }

    /// `bytes` with its main part at `name`, wherever its relationship
    /// and content type point.
    fn main_named(bytes: &[u8], name: &str) -> Vec<u8> {
        const OFFICE_DOCUMENT: &str =
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
        let mut pkg = crate::opc::PartFs::open(bytes).unwrap();
        let xml = pkg.part_bytes("word/document.xml").unwrap().to_vec();
        pkg.remove_part("word/document.xml");
        pkg.set_part(name, xml);
        pkg.add_content_type_override(
            &format!("/{name}"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        );
        pkg.remove_package_relationships_by_type(OFFICE_DOCUMENT);
        pkg.add_package_relationship(OFFICE_DOCUMENT, name);
        pkg.to_zip().unwrap()
    }

    #[test]
    fn a_main_part_without_an_xml_extension_is_gated_too() {
        let source = main_named(&revisions(&[("ins", 5), ("ins", 6)]), "word/main.dat");
        let candidate = main_named(&revisions(&[("ins", 5), ("ins", 5)]), "word/main.dat");
        refused_as_duplicate(
            check_candidates(&source, &result_of(candidate)),
            "w:ins w:id 5",
            "word/main.dat",
        );
    }

    #[test]
    fn a_strict_main_part_without_an_xml_extension_is_read_as_transitional() {
        // Codex on #1666: its inherited duplicates stay inherited.
        let source = main_named(
            &strict(&revisions(&[("ins", 5), ("ins", 5)])),
            "word/main.dat",
        );
        let candidate = main_named(&revisions(&[("ins", 5), ("ins", 5)]), "word/main.dat");
        let result = check_candidates(&source, &result_of(candidate));
        assert!(result.is_ok(), "{result:?}");
    }

    fn prepared(source: &[u8], operation: &str) -> Result<PreparedEdit, EditError> {
        let json = format!(
            r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{operation}]}}"#
        );
        let plan = EditPlan::from_json(&json).unwrap();
        prepare(source, &plan, &PrepareOptions::default())
    }

    #[test]
    fn a_paragraph_marks_revision_id_survives_paragraph_edits() {
        // `w:pPr/w:rPr/w:ins`: a new paragraph made like it, one split off
        // it, and a replacement inside it publish.
        let source = deeper_boundary_fixture::docx(
            r#"<w:p><w:pPr><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr></w:pPr><w:r><w:t xml:space="preserve">abcd efgh</w:t></w:r></w:p><w:p><w:r><w:t>next</w:t></w:r></w:p>"#,
        );
        for operation in [
            r#"{"kind":"insert_paragraph","paragraph":{"index":1},"runs":[{"text":"b"}],"like":{"index":0}}"#,
            r#"{"kind":"insert","paragraph":"p0","after":"abcd","text":"\nnew para"}"#,
            r#"{"kind":"replace","paragraph":"p0","find":"cd","replacement":"X"}"#,
        ] {
            let result = prepared(&source, operation);
            assert!(result.is_ok(), "{operation}: {:?}", result.err());
        }
    }

    /// `culprits` against a refusal that comes back whenever the
    /// operations it is given include every set of one of `causes`,
    /// counting the reruns.
    fn culprits_of(
        count: usize,
        together: bool,
        causes: &[&[usize]],
    ) -> (Option<Vec<usize>>, usize) {
        let mut reruns = 0;
        let found = culprits(count, together, |ops| {
            reruns += 1;
            causes
                .iter()
                .any(|cause| cause.iter().all(|i| ops.contains(i)))
        });
        (found, reruns)
    }

    #[test]
    fn culprits_finds_one_operation_in_logarithmic_reruns() {
        for at in 0..64 {
            let (found, reruns) = culprits_of(64, true, &[&[at]]);
            assert_eq!(found, Some(vec![at]));
            // The empty plan, six halvings, and the operation alone.
            assert!(reruns <= 8, "{at}: {reruns}");
        }
        assert_eq!(culprits_of(1, true, &[&[0]]).0, Some(vec![0]));
    }

    #[test]
    fn culprits_names_the_operations_a_refusal_needs() {
        // The later one brings it in; the earlier ones it needs follow in
        // plan order.
        assert_eq!(culprits_of(10, true, &[&[2, 7]]).0, Some(vec![7, 2]));
        assert_eq!(culprits_of(10, true, &[&[5, 1, 8]]).0, Some(vec![8, 1, 5]));
        assert_eq!(culprits_of(2, true, &[&[0, 1]]).0, Some(vec![1, 0]));
        // Independent causes: the first one complete in plan order is
        // named, alone.
        assert_eq!(culprits_of(10, true, &[&[6], &[3]]).0, Some(vec![3]));
        // Without `together` (v2), only the operation that brings it in.
        assert_eq!(culprits_of(10, false, &[&[2, 7]]).0, Some(vec![7]));
    }

    #[test]
    fn culprits_seeks_each_further_operation_before_the_last_found() {
        // The operations before the last one found, with those found,
        // reproduce the refusal: the next search covers only them.
        let (found, reruns) = culprits_of(64, true, &[&[0, 1, 2, 3, 63]]);
        assert_eq!(found, Some(vec![63, 0, 1, 2, 3]));
        assert!(reruns <= 21, "{reruns}");
        let (found, reruns) = culprits_of(64, true, &[&[1, 3, 63]]);
        assert_eq!(found, Some(vec![63, 1, 3]));
        assert!(reruns <= 18, "{reruns}");
    }

    #[test]
    fn a_refusal_without_operations_names_none() {
        // The plan without operations meets it: settling, not an
        // operation, causes it.
        assert_eq!(culprits_of(5, true, &[&[]]).0, None);
        assert_eq!(culprits_of(0, true, &[]).0, None);
    }

    /// A failure the candidate checks did not raise keeps its error as it
    /// is, without a rerun; a refusal no operation causes stays the plan's.
    #[test]
    fn attribution_leaves_other_failures_alone() {
        let error = err("OVERLAPPING_EDITS", Some("op-2"), "two edits");
        let mut never = Reruns::new(|_: &[usize]| -> Option<Refused> {
            panic!("no rerun for a failure the candidate checks did not raise")
        });
        let kept = attributed(*Box::<Failure>::from(error.clone()), 3, true, &mut never);
        assert_eq!(kept, error);
        let refusal = refused(
            "redline",
            "BOOKMARK_CROSSES_REVISION",
            "word/document.xml",
            "Clause".into(),
            "the edit introduces it".into(),
        );
        let failure = Box::new(Failure {
            error: refusal.error.clone(),
            refusal: Some((vec![refusal.clone()], Vec::new())),
            advice: Some("drop update_fields"),
        });
        let mut always = Reruns::new(|_: &[usize]| Some(vec![refusal.clone()]));
        let kept = attributed(*failure, 3, true, &mut always);
        assert_eq!(kept.operation, None);
        assert_eq!(kept.message, "the edit introduces it; drop update_fields");
    }

    /// The operations are named before the advice that closes the message.
    #[test]
    fn named_operations_come_before_the_advice() {
        let refusal = refused(
            "redline",
            "RANGE_LOST",
            "",
            "bookmark B".into(),
            "Reject All drops bookmark B".into(),
        );
        let outcome = |id: &str, kind: &str, paragraph: Option<&str>| EditOutcome {
            id: id.into(),
            kind: kind.into(),
            status: String::new(),
            paragraph: paragraph.map(str::to_string),
            matches: 0,
            context: None,
            comment_id: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
            code: None,
            message: None,
        };
        let failure = Box::new(Failure {
            error: refusal.error.clone(),
            refusal: Some((
                vec![refusal.clone()],
                vec![
                    outcome("op-1", "replace", Some("p0")),
                    outcome("op-2", "insert", Some("p4")),
                    outcome("op-3", "page_setup", None),
                ],
            )),
            advice: Some("drop update_fields"),
        });
        let mut reruns = Reruns::new(|ops: &[usize]| {
            (ops.contains(&0) && ops.contains(&2)).then(|| vec![refusal.clone()])
        });
        let named = attributed(*failure, 3, true, &mut reruns);
        assert_eq!(named.operation.as_deref(), Some("op-3"));
        assert_eq!(
            named.message,
            "Reject All drops bookmark B; op-3 (page_setup) introduces it, together with op-1 (replace on p0); drop update_fields"
        );
    }

    /// A rerun reproduces a defect when it meets that one among others,
    /// never another defect of its code and part; each set of operations
    /// is prepared once.
    #[test]
    fn a_rerun_reproduces_the_defect_not_its_code() {
        let defect = |subject: &str| {
            refused(
                "redline",
                "DUPLICATE_REVISION_ID",
                "word/document.xml",
                subject.into(),
                format!("{subject} repeats"),
            )
        };
        let mut runs = 0;
        let mut reruns = Reruns::new(|ops: &[usize]| {
            runs += 1;
            let mut met = Vec::new();
            if ops.contains(&0) {
                met.push(defect("w:ins w:id 9"));
            }
            if ops.contains(&1) {
                met.push(defect("w:ins w:id 7"));
            }
            (!met.is_empty()).then_some(met)
        });
        let seven = defect("w:ins w:id 7");
        assert!(!reruns.reproduces(&[0], &seven));
        assert!(reruns.reproduces(&[1], &seven));
        assert!(reruns.reproduces(&[0, 1], &seven));
        assert!(!reruns.reproduces(&[0], &seven));
        drop(reruns);
        assert_eq!(runs, 3);
    }

    #[test]
    fn a_run_marks_revision_id_split_in_two_is_refused() {
        // A run's `w:rPr` holds no `w:ins` (OpenXmlValidator:
        // `Sch_InvalidElementContentExpectingComplex`). Splitting such a run
        // copies the mark's id: the candidate is refused, not published.
        let source = deeper_boundary_fixture::docx(
            r#"<w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p>"#,
        );
        let error = prepared(
            &source,
            r#"{"kind":"replace","paragraph":"p0","find":"bc","replacement":"X"}"#,
        )
        .err()
        .unwrap();
        assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
        assert!(
            error.message.contains("w:ins on a mark w:id 7"),
            "{error:?}"
        );
    }

    /// The KI-8 shape: the clean copy resolves the edit's insertion away
    /// and only the redline repeats an id. The refusal names the redline.
    #[test]
    fn a_duplicate_only_the_redline_carries_is_refused_by_its_artifact() {
        let source = revisions(&[("ins", 5), ("ins", 6)]);
        let mut result = result_of(revisions(&[("ins", 5), ("ins", 6)]));
        result.redline = revisions(&[("ins", 5), ("ins", 5)]);
        let error = check_candidates(&source, &result).unwrap_err();
        assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
        assert_eq!(
            error.message,
            "the edit introduces a duplicate revision id in the redline: w:ins w:id 5 occurs 2 times in word/document.xml (the source has 1)"
        );
    }

    #[test]
    fn an_unreadable_source_skips_the_duplicate_gate() {
        let candidate = revisions(&[("ins", 5), ("ins", 5)]);
        let result = check_candidates(b"not a zip at all", &result_of(candidate));
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn inherited_duplicates_and_ids_shared_across_kinds_publish() {
        let inherited = [("ins", 5), ("ins", 5)];
        assert!(gate(&inherited, &inherited).is_ok());
        assert!(gate(&inherited, &[("ins", 5)]).is_ok());
        let shared = [("ins", 5), ("del", 5), ("rPrChange", 5)];
        assert!(gate(&[("ins", 5)], &shared).is_ok());
    }

    #[test]
    fn verification_reports_only_completed_candidate_checks() {
        let skipped = CandidateChecks::default().verification();
        assert_eq!(skipped.integrity, "skipped");
        assert_eq!(skipped.operation_postconditions, "skipped");
        let source = deeper_boundary_fixture::docx("<w:p><w:r><w:t>old</w:t></w:r></w:p>");
        let plan = EditPlan::from_json(
            r#"{"schema_version":1,"author":"T","date":"2026-10-17T12:00:00Z","operations":[]}"#,
        )
        .unwrap();
        let checked = prepare(&source, &plan, &PrepareOptions::default())
            .unwrap()
            .checks
            .verification();
        assert_eq!(checked.integrity, "passed");
        assert_eq!(checked.operation_postconditions, "passed");
    }

    #[test]
    fn redline_only_defects_name_the_story_and_paragraph() {
        use deeper_boundary_fixture::{Part, R_NS, W_NS, docx_with_sect, para};

        let kinds = ["header", "footer", "footnotes", "endnotes"];
        let names = [
            "word/header1.xml",
            "word/footer1.xml",
            "word/footnotes.xml",
            "word/endnotes.xml",
        ];
        let roots = ["hdr", "ftr", "footnotes", "endnotes"];
        let xmls: Vec<_> = roots
            .iter()
            .map(|root| {
                let text = para("old");
                let content = match *root {
                    "footnotes" => format!(r#"<w:footnote w:id="1">{text}</w:footnote>"#),
                    "endnotes" => format!(r#"<w:endnote w:id="1">{text}</w:endnote>"#),
                    _ => text,
                };
                format!(r#"<w:{root} xmlns:w="{W_NS}">{content}</w:{root}>"#)
            })
            .collect();
        let rels: Vec<_> = kinds.iter().map(|k| format!("{R_NS}/{k}")).collect();
        let types: Vec<_> = kinds
            .iter()
            .map(|k| {
                format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{k}+xml")
            })
            .collect();
        let parts: Vec<_> = (0..kinds.len())
            .map(|i| Part {
                name: names[i],
                content_type: &types[i],
                rel_type: &rels[i],
                xml: &xmls[i],
            })
            .collect();
        let source = docx_with_sect(
            &(para("old")
                + r#"<w:p><w:r><w:footnoteReference w:id="1"/><w:endnoteReference w:id="1"/></w:r></w:p>"#),
            &parts,
            r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#,
        );
        let mut operations = vec![
            serde_json::json!({"kind":"replace", "paragraph":"p0", "find":"old", "replacement":"new"}),
        ];
        for story in crate::inspect::stories(&source).unwrap() {
            operations.push(serde_json::json!({"kind":"replace", "paragraph":story.paragraphs[0].id, "find":"old", "replacement":"new"}));
        }
        let plan = EditPlan::from_json(&serde_json::json!({
            "schema_version":1, "author":"T", "date":"2026-10-17T12:00:00Z", "operations":operations,
        }).to_string()).unwrap();
        let mut tx = Transaction::start(&source, &plan).unwrap();
        tx.resolve().unwrap();
        tx.apply().unwrap();
        let (clean, _) = tx.finish().unwrap();
        assert!(check_redline_stories(&tx, &clean, &clean).is_ok());
        let mut failures = Vec::new();
        for part in std::iter::once("word/document.xml").chain(names) {
            for defect in ["text", "count", "missing"] {
                if part == "word/document.xml" && defect == "missing" {
                    continue;
                }
                let mut pkg = crate::opc::PartFs::open(&clean).unwrap();
                let xml = pkg.part_string(part).unwrap();
                if defect == "missing" {
                    pkg.remove_related_part(
                        "word/document.xml",
                        &format!("rIdX{}", names.iter().position(|p| *p == part).unwrap()),
                    );
                } else {
                    let broken = if defect == "text" {
                        xml.replace("new", "WRONG")
                    } else {
                        xml.replace("</w:p>", "</w:p><w:p/>")
                    };
                    pkg.set_part(part, broken.into_bytes());
                }
                let redline = pkg.to_zip().unwrap();
                match check_redline_stories(&tx, &clean, &redline) {
                    Err(e)
                        if e.code == codes::POSTCONDITION_FAILED
                            && e.message.contains(part)
                            && e.message.contains("paragraph") => {}
                    outcome => failures.push(format!("{part} {defect}: {outcome:?}")),
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    thread_local! {
        /// How many times [`prepare_once`] ran on this test thread.
        pub(crate) static PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    /// `--skip-failed` attributes every defect of a refusal in one pass,
    /// each rerun once: a plan whose operations are each refused alone
    /// takes at most `2n + 1` preparations (the plan, its prefixes, each
    /// operation alone), not a bisection per retry (#1689 review).
    #[test]
    fn skip_failed_attributes_every_defect_of_a_refusal_at_once() {
        let n = 16;
        let body: String = (0..n)
            .map(|i| {
                format!(
                    r#"<w:p><w:r><w:rPr><w:ins w:id="{}" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p>"#,
                    100 + i
                )
            })
            .collect();
        let source = deeper_boundary_fixture::docx(&body);
        let operations: Vec<String> = (0..n)
            .map(|i| {
                format!(r#"{{"kind":"replace","paragraph":"p{i}","find":"bc","replacement":"X"}}"#)
            })
            .collect();
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{}]}}"#,
            operations.join(",")
        ))
        .unwrap();
        PREPARATIONS.with(|count| count.set(0));
        let options = PrepareOptions {
            skip_failed: true,
            ..PrepareOptions::default()
        };
        let error = prepare(&source, &plan, &options).unwrap_err();
        assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
        let failed = error
            .outcomes
            .iter()
            .filter(|o| o.status == "failed")
            .count();
        assert_eq!(failed, n, "{error:?}");
        // The plan, then each operation alone.
        let preparations = PREPARATIONS.with(std::cell::Cell::get);
        assert!(
            preparations <= n + 1,
            "{preparations} preparations for {n} operations"
        );
    }

    /// One refused operation among many is still found by bisection, in
    /// logarithmic reruns, and `--skip-failed` publishes the others.
    #[test]
    fn skip_failed_bisects_a_lone_defect() {
        let n = 16;
        let culprit = 11;
        let body: String = (0..n)
            .map(|i| {
                let mark = if i == culprit {
                    r#"<w:rPr><w:ins w:id="100" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr>"#
                } else {
                    ""
                };
                format!(r#"<w:p><w:r>{mark}<w:t>abcd</w:t></w:r></w:p>"#)
            })
            .collect();
        let source = deeper_boundary_fixture::docx(&body);
        let operations: Vec<String> = (0..n)
            .map(|i| {
                format!(r#"{{"kind":"replace","paragraph":"p{i}","find":"bc","replacement":"X"}}"#)
            })
            .collect();
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{}]}}"#,
            operations.join(",")
        ))
        .unwrap();
        PREPARATIONS.with(|count| count.set(0));
        let options = PrepareOptions {
            skip_failed: true,
            ..PrepareOptions::default()
        };
        let prepared = prepare(&source, &plan, &options).unwrap();
        let skipped: Vec<usize> = prepared.skipped.iter().map(|s| s.index).collect();
        assert_eq!(skipped, [culprit]);
        assert!(
            prepared.skipped[0]
                .message
                .contains("op-12 (replace on p11) introduces it"),
            "{:?}",
            prepared.skipped
        );
        // The plan, the plan without operations, four prefixes, the
        // culprit alone and the survivors.
        let preparations = PREPARATIONS.with(std::cell::Cell::get);
        assert!(preparations <= 8, "{preparations} preparations");
    }

    thread_local! {
        static SERIALIZED_CANDIDATE: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
    }

    // A deterministic serializer fake: only the current test thread sees it.
    pub(crate) fn serialized_candidate(bytes: Vec<u8>) -> Vec<u8> {
        SERIALIZED_CANDIDATE.with(|candidate| candidate.borrow().clone().unwrap_or(bytes))
    }

    #[test]
    fn review_11_prepare_v2_checks_serialized_text_like_v1() {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                SERIALIZED_CANDIDATE.with(|c| *c.borrow_mut() = None);
            }
        }
        let source = deeper_boundary_fixture::docx("<w:p><w:r><w:t>old</w:t></w:r></w:p>");
        let legacy = EditPlan::from_json(r#"{"schema_version":1,"author":"T","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{"kind":"replace","paragraph":"p0","find":"old","replacement":"new"}]}"#).unwrap();
        let view = super::super::for_edit::for_edit(
            &source,
            &super::super::for_edit::ForEditRequest {
                all: true,
                ..Default::default()
            },
        )
        .unwrap();
        let target = &view.paragraphs[0].target;
        let value = serde_json::json!({
            "schema_version": 2, "source": {"sha256": source_sha256(&source)},
            "author": "T", "date": "2026-10-17T12:00:00Z",
            "policy": {"track": "off", "existing_revisions": "keep", "text_view":"final"},
            "resolve_revisions":{"accept":[],"reject":[]}, "scope": {"kind":"all"},
            "operations": [{"kind":"replace", "target": {"short_id":target.short_id,"long_id":target.long_id}, "find":"old", "text":"new"}]
        });
        let plan = super::super::v2::PlanV2::from_json(&value.to_string()).unwrap();
        // The requested DOM reads "new", while the fake serializer emits "wrong".
        SERIALIZED_CANDIDATE.with(|c| {
            *c.borrow_mut() = Some(deeper_boundary_fixture::docx(
                "<w:p><w:r><w:t>wrong</w:t></w:r></w:p>",
            ));
        });
        let _reset = Reset;
        let v1 = prepare(&source, &legacy, &PrepareOptions::default());
        assert!(v1.is_err(), "v1 must detect the serializer disagreement");
        let expected = v1.unwrap_err();
        assert_eq!(expected.code, "POSTCONDITION_FAILED");
        for track in [super::super::v2::Track::Off, super::super::v2::Track::On] {
            let mut plan = plan.clone();
            plan.policy.track = track;
            let actual = prepare_v2(&source, &plan, &PrepareOptions::default());
            assert!(
                actual.is_err(),
                "prepare_v2 must refuse the same serialized-text disagreement as prepare"
            );
            assert_eq!(actual.unwrap_err().code, expected.code);
        }
    }
}
