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
    EditError, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved, StoryPart,
    Transaction, check_update_fields, comment_holds_kept_text, comment_text_of, controls,
    revision_counts, sections, tracked, whole,
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
    /// Source paragraph → chosen-primary correspondence for `--expect`
    /// (V2-13). `None` on the v1 path, whose published primary is a
    /// comparer redline with no tracked correspondence: a v1 plan plus
    /// `--expect` refuses instead of guessing.
    pub correspondence: Option<crate::expect::SourceToOutputCorrespondence>,
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
    let mut checks = check_candidates(source, &result)?;
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

/// Findings on the input and what each serialized artifact adds. The
/// baseline discloses without blocking; an introduced blocking finding
/// fails preparation.
fn check_candidates(source: &[u8], result: &EditResult) -> Result<CandidateChecks, EditError> {
    // A source the validator cannot read gives no trustworthy baseline:
    // an empty one would call every candidate finding introduced, so with
    // no baseline nothing is claimed introduced (the candidate's own
    // validation below still gates what the edit produced).
    let baseline = validate::validate(source).ok();
    let mut introduced = Vec::new();
    let mut unresolved = Vec::new();
    for bytes in [&result.clean, &result.redline] {
        let findings = validate::validate(bytes).map_err(|e| {
            err(
                "CANDIDATE_INVALID",
                None,
                format!("the serialized candidate cannot be validated: {e}"),
            )
        })?;
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
    /// `para` → (first op id, `(start, end, replacement, op)` spans).
    type Spans = (String, Vec<(usize, usize, String, usize)>);
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
                ..
            } => {
                let entry = edits
                    .entry(*para)
                    .or_insert_with(|| (tx.outcomes[*op].id.clone(), Vec::new()));
                entry.1.push((*start, *end, replacement.clone(), *op));
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
        spans.sort_by_key(|&(start, ..)| start);
        let mut text = String::with_capacity(base.len());
        // The rewrite's own spans alone, composed on the base: a rewrite
        // beside co-resident edits must still realize its request, or the
        // failure is the rewrite's, not the composed text's.
        let mut own = String::with_capacity(base.len());
        let mut at = 0;
        let mut own_at = 0;
        for (start, end, replacement, span_op) in spans {
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

/// The application half of the old `apply_plan`, unchanged: the candidate
/// is produced exactly as before so the clean copy and redline stay
/// byte-identical.
fn produce(tx: &mut Transaction<'_>, emit: bool) -> Result<EditResult, EditError> {
    let plan = tx.plan;
    let (mut clean, mut marked) = tx.finish()?;
    // The candidate the DOM produced, checked before `update_fields`
    // (which legitimately rewrites it: filled TOC entries, NUMPAGES) or
    // the comparer ever sees it.
    check_serialized_stories(tx, &clean)?;
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
    let applied = super::v2_apply::apply_plan(source, plan)?;
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
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
                paragraph_ids: Vec::new(),
                paragraphs: ParagraphDelta::default(),
                operations: Vec::new(),
                comments_added: 0,
                revisions: Default::default(),
                fields: Vec::new(),
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
