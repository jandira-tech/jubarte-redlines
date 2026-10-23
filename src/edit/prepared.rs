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
    EditError, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved, StoryPart,
    Transaction, check_update_fields, comment_holds_kept_text, comment_text_of, controls,
    lift_deletion_comments, revision_counts, sections, tracked, whole,
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
const BLOCKING_CODES: &[&str] = &["BOOKMARK_CROSSES_REVISION"];

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
    let effect = primary_effect(&tx, &result);
    let (primary, primary_bytes) = match effect {
        Effect::Unchanged => (source.to_vec(), PrimaryBytes::Source),
        _ => match options.role {
            PrimaryRole::Clean => (result.clean.clone(), PrimaryBytes::Candidate),
            PrimaryRole::Redline => (result.redline.clone(), PrimaryBytes::Candidate),
        },
    };
    let mut checks = check_candidates(source, &result)
        .and_then(|checks| {
            if compared_beside {
                check_rejected_ranges(source, &result.redline)?;
            }
            Ok(checks)
        })
        .map_err(|mut e| {
            if compared_beside && e.code == "CANDIDATE_INVALID" {
                e.message = format!(
                    "{}; update_fields compares an edit beside a bookmark, comment or permission range, so drop update_fields and refresh the fields with `jubarte convert FILE --update-fields -o OUT.docx` afterwards",
                    e.message
                );
            }
            e
        })?;
    checks.operation_postconditions = true;
    if options.require_change && effect == Effect::Unchanged {
        return Err(err(
            codes::NO_CHANGES_REQUESTED,
            None,
            "the primary transaction has no document effect; --require-change was given",
        ));
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
fn primary_effect(tx: &Transaction<'_>, result: &EditResult) -> Effect {
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
        return Effect::Changed;
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
            return Effect::Changed;
        }
    }
    // A text edit that meant to change the text is an effect even when the
    // engine wrote nothing for it.
    if expected_texts(tx, false).into_iter().any(|(p, e)| {
        matches!(e.scope, ExpectedScope::Paragraph) && e.text != tx.projections[p].text
    }) {
        return Effect::Changed;
    }
    Effect::Unchanged
}

/// Reject All of `redline` gives the source back, so it keeps each of the
/// source's bookmarks (by name; Word's own `_GoBack` aside), comment ranges
/// and permission ranges. Compare places them by the words around them and
/// can carry one into an insertion, which Reject All then removes: the
/// TOC entry linking to a heading's `_Toc` bookmark reads "Error! Bookmark
/// not defined." (#1156).
fn check_rejected_ranges(source: &[u8], redline: &[u8]) -> Result<(), EditError> {
    let rejected = crate::document_comparer::reject_revisions(redline).map_err(|e| {
        err(
            "CANDIDATE_INVALID",
            None,
            format!("the redline cannot be rejected: {e}"),
        )
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
    Err(err(
        "CANDIDATE_INVALID",
        None,
        format!(
            "Reject All of the redline drops {} the source holds",
            lost.join(", ")
        ),
    ))
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
        for start in dom.descendants(root, Some(&W::name("bookmarkStart"))) {
            if let Some(name) = dom.attribute(start, &W::name("name"))
                && name != "_GoBack"
            {
                markers.bookmarks.insert(name.to_string());
            }
        }
        markers.comments += dom
            .descendants(root, Some(&W::name("commentRangeStart")))
            .len();
        markers.permissions += dom.descendants(root, Some(&W::name("permStart"))).len();
    }
    markers
}

/// Findings on the input and what each serialized artifact adds. The
/// baseline discloses without blocking; an introduced blocking finding
/// fails preparation.
fn check_candidates(source: &[u8], result: &EditResult) -> Result<CandidateChecks, EditError> {
    // A source the validator cannot read gives no trustworthy baseline:
    // an empty one would call every candidate finding introduced, so with
    // no baseline nothing is claimed introduced (the candidate's own
    // validation below still gates what the edit produced).
    let baseline = validate::validate(source).ok();
    // Counted only when a candidate repeats an id at all: its own
    // `DUPLICATE_REVISION_ID` reads the same parts, records and keys.
    let mut source_ids = None;
    let mut introduced = Vec::new();
    let mut unresolved = Vec::new();
    for (artifact, bytes) in [("clean copy", &result.clean), ("redline", &result.redline)] {
        let findings = validate::validate(bytes).map_err(|e| {
            err(
                "CANDIDATE_INVALID",
                None,
                format!("the serialized candidate cannot be validated: {e}"),
            )
        })?;
        if findings.iter().any(|f| f.code == "DUPLICATE_REVISION_ID")
            && let Some(source_ids) =
                source_ids.get_or_insert_with(|| validate::revision_id_occurrences(source).ok())
        {
            check_revision_ids(source_ids, artifact, bytes)?;
        }
        for finding in &findings {
            if !unresolved
                .iter()
                .any(|f| finding_key(f) == finding_key(finding))
            {
                unresolved.push(finding.clone());
            }
        }
        let Some(baseline) = &baseline else {
            continue;
        };
        for finding in introduced_findings(baseline, &findings) {
            if blocking(&finding) {
                return Err(err(
                    "CANDIDATE_INVALID",
                    None,
                    format!(
                        "the edit introduces {} in {}#{}: {}",
                        finding.code, finding.part, finding.location, finding.message
                    ),
                ));
            }
            if !introduced
                .iter()
                .any(|f| finding_key(f) == finding_key(&finding))
            {
                introduced.push(finding);
            }
        }
    }
    Ok(CandidateChecks {
        baseline: baseline.unwrap_or_default(),
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
/// opens it; publication refuses it. `artifact` names the candidate.
fn check_revision_ids(
    source: &HashMap<(String, String), usize>,
    artifact: &str,
    candidate: &[u8],
) -> Result<(), EditError> {
    let occurrences = validate::revision_id_occurrences(candidate).map_err(|e| {
        err(
            "CANDIDATE_INVALID",
            None,
            format!("the serialized candidate cannot be read: {e}"),
        )
    })?;
    let mut introduced: Vec<_> = occurrences
        .iter()
        .filter(|&(key, &count)| count > 1 && count > source.get(key).copied().unwrap_or(0))
        .collect();
    introduced.sort();
    let Some(((part, key), count)) = introduced.first() else {
        return Ok(());
    };
    let mut fields = key.splitn(3, ':');
    let (kind, mark, id) = (
        fields.next().unwrap_or_default(),
        fields.next() == Some("true"),
        fields.next().unwrap_or_default(),
    );
    let mark = if mark { " on a mark" } else { "" };
    let had = source
        .get(&(part.clone(), key.clone()))
        .copied()
        .unwrap_or(0);
    Err(err(
        "CANDIDATE_INVALID",
        None,
        format!(
            "the edit introduces a duplicate revision id in the {artifact}: w:{kind}{mark} w:id {id} occurs {count} times in {part} (the source has {had})"
        ),
    ))
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
fn expected_texts(tx: &Transaction<'_>, exact: bool) -> Vec<(usize, ExpectedText)> {
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
        spans.sort_by_key(|&(start, .., restore, after)| (start, !restore, empty && after));
        let mut text = String::with_capacity(base.len());
        // The rewrite's own spans alone, composed on the base: a rewrite
        // beside co-resident edits must still realize its request, or the
        // failure is the rewrite's, not the composed text's.
        let mut own = String::with_capacity(base.len());
        let mut at = 0;
        let mut own_at = 0;
        for (start, end, replacement, span_op, ..) in spans {
            text.push_str(&base[at..start]);
            text.push_str(&replacement);
            at = end;
            if Some(span_op) == rewrite_op {
                own.push_str(&base[own_at..start]);
                own.push_str(&replacement);
                own_at = end;
            }
        }
        text.push_str(&base[at..]);
        own.push_str(&base[own_at..]);
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
    out
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
    for (para, expected) in expected_texts(tx, exact) {
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
    if plan.existing_revisions == ExistingRevisions::Keep
        || (emit && matches!(tx.commented_base()?, std::borrow::Cow::Borrowed(_)))
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
    let settings = WmlComparerSettings {
        author_for_revisions: plan.author.clone(),
        date_time_for_revisions: tx.date.clone(),
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
    let mut checks = check_candidates(source, &result)?;
    checks.operation_postconditions = true;
    if options.require_change && unchanged {
        return Err(err(
            codes::NO_CHANGES_REQUESTED,
            None,
            "the primary transaction has no document effect; --require-change was given",
        ));
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
        refused_as_duplicate(
            gate(
                &[("customXmlInsRangeStart", 5), ("customXmlInsRangeStart", 6)],
                &[("customXmlInsRangeStart", 5), ("customXmlInsRangeStart", 5)],
            ),
            "w:customXmlInsRangeStart w:id 5",
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
