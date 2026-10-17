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

use std::collections::{BTreeMap, HashSet};

use super::{
    EditError, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved, Transaction,
    check_update_fields, comment_holds_kept_text, comment_text_of, revision_counts, sections,
    tracked, whole,
};
use crate::comparer::WmlComparerSettings;
use crate::inspect::{body_paragraph_nodes, project_paragraph, source_sha256};
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
}

/// What to check beyond the shared candidate checks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    check_update_fields(plan)?;
    let mut tx = Transaction::start(source, plan)?;
    tx.resolve()?;
    tracked::check(&tx)?;
    let emit = tx.emits_directly()?;
    tx.apply()?;
    check_dom_realized(&tx, options.exact_text)?;
    let result = produce(&mut tx, emit)?;
    let effect = primary_effect(&tx, &result);
    let (primary, primary_bytes) = match effect {
        Effect::Unchanged => (source.to_vec(), PrimaryBytes::Source),
        _ => (result.clean.clone(), PrimaryBytes::Candidate),
    };
    let checks = check_candidates(source, &result)?;
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
    if expected_texts(tx, false)
        .into_iter()
        .any(|(p, e)| e.text != tx.projections[p].text)
    {
        return Effect::Changed;
    }
    Effect::Unchanged
}

/// Findings on the input and what each serialized artifact adds. The
/// baseline discloses without blocking; an introduced blocking finding
/// fails preparation.
fn check_candidates(source: &[u8], result: &EditResult) -> Result<CandidateChecks, EditError> {
    let baseline = validate::validate(source).unwrap_or_default();
    let mut introduced = Vec::new();
    for bytes in [&result.clean, &result.redline] {
        let findings = validate::validate(bytes).map_err(|e| {
            err(
                "CANDIDATE_INVALID",
                None,
                format!("the serialized candidate cannot be validated: {e}"),
            )
        })?;
        for finding in introduced_findings(&baseline, &findings) {
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
        baseline,
        introduced,
    })
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
}

/// The exact text each paragraph must read as in the candidate: resolution
/// composed over the base projection, or, under `exact`, the `rewrite`'s
/// requested text itself — so a requested change the algorithm normalizes
/// away is detected instead of passing as a no-op.
fn expected_texts(tx: &Transaction<'_>, exact: bool) -> BTreeMap<usize, ExpectedText> {
    /// `para` → (first op id, `(start, end, replacement)` spans).
    type Spans = (String, Vec<(usize, usize, String)>);
    let mut edits: BTreeMap<usize, Spans> = BTreeMap::new();
    let mut merges = Vec::new();
    let mut filled: HashSet<usize> = HashSet::new();
    for (op, resolved) in &tx.resolved {
        match resolved {
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                ..
            } => {
                let entry = edits
                    .entry(*para)
                    .or_insert_with(|| (tx.outcomes[*op].id.clone(), Vec::new()));
                entry.1.push((*start, *end, replacement.clone()));
            }
            Resolved::MergeParagraphs {
                para,
                next,
                separator,
            } => {
                merges.push((*para, *next, separator.clone(), tx.outcomes[*op].id.clone()));
            }
            // A fill replaces its control's projected text, which the
            // composition cannot predict; the fill's own resolution already
            // refused the bad cases, so these paragraphs carry no composed
            // postcondition.
            Resolved::FillControl { paragraphs, .. } => {
                filled.extend(paragraphs.iter().copied());
            }
            _ => {}
        }
    }
    let mut expected = BTreeMap::new();
    for (para, (op, mut spans)) in edits {
        if exact && let Some(text) = rewrite_text(tx, para) {
            expected.insert(
                para,
                ExpectedText {
                    op,
                    text,
                    ignore_symbols: true,
                },
            );
            continue;
        }
        let base = &tx.projections[para].text;
        spans.sort_by_key(|&(start, ..)| start);
        let mut text = String::with_capacity(base.len());
        let mut at = 0;
        for (start, end, replacement) in spans {
            text.push_str(&base[at..start]);
            text.push_str(&replacement);
            at = end;
        }
        text.push_str(&base[at..]);
        expected.insert(
            para,
            ExpectedText {
                op,
                text,
                ignore_symbols: false,
            },
        );
    }
    // A rewrite that resolved to no edits still has a postcondition.
    if exact {
        for (i, op) in tx.plan.operations.iter().enumerate() {
            let OperationKind::Rewrite { paragraph, text } = &op.kind else {
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
            },
        );
        expected.remove(&para);
    }
    for para in filled {
        expected.remove(&para);
    }
    expected
}

/// The text a `rewrite` on `para` asks for, or `None` when no rewrite op
/// resolved onto it (`cN.text` rewrites excluded: their target is the
/// covered span, not the paragraph).
fn rewrite_text(tx: &Transaction<'_>, para: usize) -> Option<String> {
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
        (outcome_paragraph(tx, i) == Some(para)).then(|| text.clone())
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
fn check_dom_realized(tx: &Transaction<'_>, exact: bool) -> Result<(), EditError> {
    for (para, expected) in expected_texts(tx, exact) {
        if !attached(tx, para) {
            // Merged-away or deleted paragraphs carry no text postcondition;
            // merge survivors are checked on their own entry.
            continue;
        }
        let node = tx.paragraph_nodes[para];
        let realized = project_paragraph(&tx.opened.dom, node).text;
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

/// The serialized body's paragraphs read exactly as the DOM they were
/// written from: a character the writer dropped or restated is
/// `POSTCONDITION_FAILED`.
fn check_serialized_body(tx: &Transaction<'_>, clean: &[u8]) -> Result<(), EditError> {
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
    Ok(())
}

/// The application half of the old `apply_plan`, unchanged: the candidate
/// is produced exactly as before so the clean copy and redline stay
/// byte-identical.
fn produce(tx: &mut Transaction<'_>, emit: bool) -> Result<EditResult, EditError> {
    let plan = tx.plan;
    let (mut clean, mut marked) = tx.finish()?;
    // The candidate the DOM produced, checked before `update_fields`
    // (which legitimately rewrites it: filled TOC entries, NUMPAGES) or
    // the comparer ever sees it.
    check_serialized_body(tx, &clean)?;
    if plan.existing_revisions == ExistingRevisions::Keep
        || (emit && matches!(tx.commented_base()?, std::borrow::Cow::Borrowed(_)))
    {
        let result = tracked::result(tx, clean)?;
        tx.check_redactions(&[&result.clean, &result.redline])?;
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
        crate::document_comparer::compare_documents_with_settings(&base, revised, &settings)
            .map_err(|e| err("COMPARE_FAILED", None, e.to_string()))?;
    if tx
        .resolved
        .iter()
        .any(|(_, r)| matches!(r, Resolved::PageSetup { .. }))
    {
        redline = sections::record_mid_changes(&redline, &base, &plan.author, &tx.date)?;
    }
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
    // Settings are not revisions: the redline takes them as they are.
    if let Some(request) = &tx.settings {
        redline = crate::settings::apply_settings_to_docx(&redline, request)
            .map_err(|m| err("PACKAGE_WRITE", None, m))?;
    }
    tx.check_redactions(&[&clean, &redline])?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline);
    report.fields = fields;
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
