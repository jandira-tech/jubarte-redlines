// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PlanV2's source/scope/baseline adapter onto the existing transaction engine.
//! This module does no publication. `clean` means new edits applied directly;
//! it is never an implicitly all-accepted copy of retained negotiation history.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::v2::{self, AttachedComment, ContextKind, PlanV2, Scope, Target, Track};
use super::{
    EditError, EditOutcome, EditPlan, EditReport, ExistingRevisions, Loaded, Operation,
    OperationKind, ResolveRevisions, Resolved, ResolvedRevisions, Selector, Transaction, anchor,
    err, load_stories, open_error, origin, tracked,
};
use crate::changes::{Change, ChangeFilter, ChangeKind, MoveSide, Recorded};
use crate::ids::{ShortId, Step, Story};
use crate::inspect::{Opened, source_sha256};
use crate::namespaces::{W, W14};
use crate::xmllinq::{Dom, NodeId};

// The final-view states the last effectful `apply_plan` handed its
// expectations correspondence. Tests clear this, run an apply, and
// compare the captured states with a reparse of the emitted primary,
// reading exactly what production used (#1087).
#[cfg(test)]
thread_local! {
    static CORRESPONDENCE_STATES: std::cell::RefCell<Vec<FinalView>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Replace this thread's [`CORRESPONDENCE_STATES`] with the states the
/// correspondence is about to read.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
fn capture_correspondence_states(states: &[FinalView]) {
    CORRESPONDENCE_STATES.with(|cell| *cell.borrow_mut() = states.to_vec());
}

/// Take this thread's [`CORRESPONDENCE_STATES`], clearing the slot.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
fn correspondence_states() -> Vec<FinalView> {
    CORRESPONDENCE_STATES.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// In-memory v2 artifacts and disclosed settlement dependencies.
#[derive(Debug)]
pub struct V2EditResult {
    /// New edits applied directly; unselected history is retained under keep.
    pub clean: Vec<u8>,
    /// Primary document: new edits tracked when requested, otherwise direct.
    pub redline: Vec<u8>,
    /// Existing engine operation and selected-settlement report.
    pub report: EditReport,
    /// Revisions carried away by selected parents, separate from selections.
    pub dependent_revisions: Vec<String>,
    /// Comments carried away by selected settlement.
    pub dependent_comments: Vec<u32>,
    /// Explicit global-policy effects on history remaining after selections.
    pub remaining_revisions: ResolvedRevisions,
    /// Source paragraph → chosen-primary paragraph correspondence for the
    /// optional `--expect` evaluation (V2-13). The entries carry the
    /// source's own short ids and output indices addressing the reopened
    /// primary's projection; merged, split and removed targets refuse in
    /// the checker, never silently pass. When the primary is the exact
    /// source bytes this is the identity correspondence over the source.
    pub correspondence: crate::expect::SourceToOutputCorrespondence,
}

/// Apply a bound PlanV2 in memory, using one immutable working baseline.
///
/// All references and direct scope resolve against S. Selected decisions run
/// before the remaining-history policy. Exact text ranges resolve against W,
/// then reuse the engine's conflict checks, mutation and tracked emission.
/// The returned `redline` is the primary for either tracking policy. No-op
/// requests return original bytes. Serialized integrity/publication gates
/// belong to the preparation/publication layer, not to this adapter.
pub fn apply_plan(source: &[u8], plan: &PlanV2) -> Result<V2EditResult, EditError> {
    apply_quoting(source, plan, super::PlanSyntax::V2)
}

/// [`apply_plan`], its refusals quoting their routes in `syntax`: the
/// plan's JSON, or the options of the typed command that built it.
pub(super) fn apply_quoting(
    source: &[u8],
    plan: &PlanV2,
    syntax: super::PlanSyntax,
) -> Result<V2EditResult, EditError> {
    // The refusals record `syntax` themselves (`RevisionRefusal::new`).
    apply_bound(source, plan, syntax).map_err(|e| with_operation_targets(e, plan))
}

/// A refusal's comment route on the refused operation's own target: the
/// hint quotes a placeholder, since only the plan holds the target.
fn with_operation_targets(mut error: EditError, plan: &PlanV2) -> EditError {
    let placeholder = format!(r#""target":{}"#, super::hint::OPERATION_TARGET);
    let fill = |message: &mut String, pointer: Option<&str>| {
        if !message.contains(&placeholder) {
            return;
        }
        let target = pointer
            .and_then(|p| p.strip_prefix("/operations/"))
            .and_then(|i| i.parse::<usize>().ok())
            .and_then(|i| plan.operations.get(i))
            .and_then(|op| serde_json::to_string(op.target()).ok());
        if let Some(target) = target {
            *message = message.replace(&placeholder, &format!(r#""target":{target}"#));
        }
    };
    fill(&mut error.message, error.operation.as_deref());
    for outcome in &mut error.outcomes {
        if let Some(message) = &mut outcome.message {
            fill(message, Some(&outcome.id));
        }
    }
    error
}

fn apply_bound(
    source: &[u8],
    plan: &PlanV2,
    syntax: super::PlanSyntax,
) -> Result<V2EditResult, EditError> {
    plan.validate()?;
    if source_sha256(source) != plan.source.sha256 {
        return Err(err(
            "STALE_SOURCE",
            None,
            "source bytes do not match PlanV2 source.sha256",
        ));
    }
    let model = SourceModel::new(source)?;
    let Decisions {
        selection,
        explicit,
        carried,
    } = model.decisions(plan)?;
    let scope = model.scope(&plan.scope)?;
    model.check_context(plan)?;
    let mapped = plan
        .operations
        .iter()
        .enumerate()
        .map(|(i, op)| {
            model.operation(op, &scope).map_err(|mut e| {
                e.operation = Some(pointer(i));
                e
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut settlement = legacy(plan);
    settlement.resolve_revisions = Some(selection);
    // Reuse the original-source selection resolver before applying global policy.
    let (selected, mut selected_report, mut comment_changes) = super::resolve_selected(
        source,
        settlement.resolve_revisions.as_ref(),
        plan.removed_anchor_comments.unwrap_or_default(),
    )?;
    let selected = selected.as_deref().unwrap_or(source);
    let selected_left = changes(selected)?;
    let selected_left_ids: BTreeSet<_> = selected_left.iter().map(|c| c.id.as_str()).collect();
    selected_report.accepted.retain(|id| !carried.contains(id));
    selected_report.rejected.retain(|id| !carried.contains(id));
    let dependent_revisions = model
        .changes
        .iter()
        .filter(|c| {
            (!explicit.contains(&c.id) || carried.contains(&c.id))
                && !selected_left_ids.contains(c.id.as_str())
        })
        .map(|c| c.id.clone())
        .collect();
    let comments_left = comments(selected)?;
    let comments_left_ids: BTreeSet<_> = comments_left.iter().map(|c| c.id).collect();
    let dependent_comments = model
        .comments
        .iter()
        .filter(|c| !comments_left_ids.contains(&c.id))
        .map(|c| c.id)
        .collect();
    let mut remaining_revisions = ResolvedRevisions::default();
    let working = match plan.policy.existing_revisions {
        ExistingRevisions::Keep => selected.to_vec(),
        ExistingRevisions::Refuse => {
            if has_history(selected)? {
                return Err(err(
                    "EXISTING_REVISIONS",
                    None,
                    "refuse requires no remaining revision anywhere, including unknown revision forms",
                ));
            }
            selected.to_vec()
        }
        policy => {
            let resolution = if policy == ExistingRevisions::Accept {
                remaining_revisions.accepted = selected_left.iter().map(|c| c.id.clone()).collect();
                crate::revision_processor::Resolution::Accept
            } else {
                remaining_revisions.rejected = selected_left.iter().map(|c| c.id.clone()).collect();
                crate::revision_processor::Resolution::Reject
            };
            let (bytes, removed, kept) = crate::changes::resolve_for_edit(
                selected,
                &ChangeFilter::default(),
                resolution,
                plan.removed_anchor_comments.unwrap_or_default()
                    == super::RemovedAnchorComments::Keep,
            )
            .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
            comment_changes.extend(removed, kept, resolution, None);
            if has_history(&bytes)? {
                return Err(err(
                    "UNSUPPORTED_REVISIONS",
                    None,
                    "global policy cannot establish revision-free status for this document",
                ));
            }
            bytes
        }
    };
    let mut execution = legacy(plan);
    execution.operations = mapped.iter().map(|m| m.legacy.clone()).collect();
    // Always use the preserving engine path. Tracking controls only the new edits.
    let mut tx = Transaction::start(&working, &execution)?;
    tx.skip_comment_ids_of(source)?;
    tx.syntax = syntax;
    if selected_report.is_empty() && remaining_revisions.is_empty() {
        tx.origin = None;
    } else {
        settlement.existing_revisions = plan.policy.existing_revisions;
        let ids = tx.stories.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
        tx.origin = Some(origin::Origin::build(
            source,
            &settlement,
            &ids,
            &tx.paragraph_story,
        )?);
    }
    tx.source_sha256 = plan.source.sha256.clone();
    tx.resolved_revisions = selected_report;
    (tx.removed_comments, tx.kept_comments) = comment_changes.records(source)?;
    resolve(&mut tx, plan, &mapped)?;
    tracked::check(&tx)?;
    let effect = !tx.resolved.is_empty()
        || !tx.resolved_revisions.is_empty()
        || !remaining_revisions.is_empty();
    let edited_revisions = edited_revisions(&tx);
    tx.apply()?;
    super::prepared::check_dom_realized(&tx, true)?;
    let (clean, _) = tx.finish()?;
    super::prepared::check_serialized_stories(&tx, &clean)?;
    // The tracked primary is the replay's emission, so its paragraph walk
    // comes back with it — the final-view states too, scanned on the
    // replay's DOM before it is dropped; the direct primary is the serialized
    // transaction DOM, walked below. Either way the walk is taken from the
    // DOM that produced the primary bytes, not by re-matching text.
    let mut insertion_origins = BTreeMap::new();
    let (result, primary_base, primary_output, replay_states) = if plan.policy.track == Track::On {
        let (result, paragraphs) = tracked::result_v2_with_paragraphs(&tx, clean)?;
        insertion_origins = paragraphs.insertion_origins;
        (
            result,
            paragraphs.base,
            paragraphs.output,
            paragraphs.output_states,
        )
    } else {
        let report = tx.report(true);
        let base = tx.paragraph_nodes.clone();
        let output = tx.paragraph_walk();
        (
            super::EditResult {
                redline: clean.clone(),
                clean,
                report,
            },
            base,
            output,
            None,
        )
    };
    let mut report = result.report;
    report.existing_revisions = plan.policy.existing_revisions;
    report.guarded = true;
    // Tests count the packages the correspondence phase opens from here
    // on, and pin that count at zero (#1087).
    #[cfg(test)]
    crate::inspect::reset_reopened_packages();
    let (clean, redline, correspondence) = if effect {
        // The primary's final-view dispositions are read off the DOM that
        // produced those bytes — the replay's own scan for a tracked
        // primary (its DOM dies with the emission), the transaction's own
        // for a direct one — so the correspondence never reopens the
        // emitted zip (#1087). Sound because `finish` changes that DOM
        // only through `apply_watermark`, which no v2 operation reaches;
        // the regression net under this module's tests pins the
        // equivalence against a reparse.
        let states: Vec<FinalView> = replay_states.unwrap_or_else(|| {
            primary_output
                .iter()
                .map(|&node| paragraph_final_state(&tx.opened.dom, node))
                .collect()
        });
        #[cfg(test)]
        capture_correspondence_states(&states);
        let correspondence = expectation_correspondence(
            &model,
            &tx,
            &primary_base,
            &primary_output,
            &states,
            plan.policy.text_view,
        )?;
        (result.clean, result.redline, correspondence)
    } else {
        // A no-effect primary is the exact source bytes; the identity
        // correspondence over the source is the output walk.
        let identity = crate::expect::identity_correspondence(source).map_err(|e| {
            err(
                "INVALID_DOCUMENT",
                None,
                format!("cannot build expectations correspondence: {e}"),
            )
        })?;
        (source.to_vec(), source.to_vec(), identity)
    };
    preserve_direct_history(
        &working,
        &clean,
        &edited_revisions,
        &plan.author,
        &BTreeMap::new(),
    )?;
    let guard = if plan.policy.track == Track::On {
        preserve_history
    } else {
        preserve_direct_history
    };
    guard(
        &working,
        &redline,
        &edited_revisions,
        &plan.author,
        &insertion_origins,
    )?;
    Ok(V2EditResult {
        clean,
        redline,
        report,
        dependent_revisions,
        dependent_comments,
        remaining_revisions,
        correspondence,
    })
}

/// Map every source paragraph onto the chosen primary's projection, from
/// what preparation already knows: [`origin::Origin`] follows each source
/// paragraph through the plan's revision settlement by its tag (merges and
/// removals included), and the transaction's own paragraph nodes place the
/// survivors in the emitted primary. Exact one-to-one targets map
/// one-to-one; a settled paragraph several source paragraphs merged into
/// carries that count so the checker refuses them as merged; paragraphs
/// removed by settlement or by `delete_paragraph` map to nothing and
/// refuse as split. Inserted paragraphs are simply absent: no expectation
/// can reference them through this correspondence. On top of the
/// settlement mapping, the primary's own text view adjusts the survivors:
/// see [`refuse_removed_final_paragraphs`].
fn expectation_correspondence(
    model: &SourceModel,
    tx: &Transaction<'_>,
    primary_base: &[NodeId],
    primary_output: &[NodeId],
    primary_states: &[FinalView],
    view: v2::TextView,
) -> Result<crate::expect::SourceToOutputCorrespondence, EditError> {
    let mut position: std::collections::HashMap<NodeId, usize> =
        std::collections::HashMap::with_capacity(primary_output.len());
    for (index, &node) in primary_output.iter().enumerate() {
        position.insert(node, index);
    }
    let output_of = |working: usize| -> Option<usize> {
        primary_base
            .get(working)
            .and_then(|&node| position.get(&node).copied())
    };
    let ids = model.ids();
    let count = model.loaded.paragraph_nodes.len();
    let mut entries = Vec::with_capacity(count);
    match &tx.origin {
        Some(origin) => {
            // How many source paragraphs each settled paragraph received:
            // one kept plus any merged in. A many-to-one settled paragraph
            // refuses every member as merged.
            let mut members: BTreeMap<usize, usize> = BTreeMap::new();
            for source in 0..count {
                if let origin::Fate::Kept(post) | origin::Fate::Merged(post) = origin.fate(source) {
                    *members.entry(post).or_default() += 1;
                }
            }
            for source in 0..count {
                let id = ids.id(source);
                match origin.fate(source) {
                    origin::Fate::Gone => {
                        entries.push(crate::expect::CorrespondenceEntry::split(id));
                    }
                    origin::Fate::Kept(post) | origin::Fate::Merged(post) => {
                        match output_of(post) {
                            Some(output) if members[&post] == 1 => {
                                entries.push(crate::expect::CorrespondenceEntry::one_to_one(
                                    id, output,
                                ));
                            }
                            Some(output) => {
                                let many = vec![output; members[&post]];
                                entries.push(crate::expect::CorrespondenceEntry::merged(id, &many));
                            }
                            None => {
                                entries.push(crate::expect::CorrespondenceEntry::split(id));
                            }
                        }
                    }
                }
            }
        }
        None => {
            // No settlement: the working walk is the source's own, so the
            // transaction's paragraph nodes place each source paragraph.
            for source in 0..count {
                let id = ids.id(source);
                match output_of(source) {
                    Some(output) => {
                        entries.push(crate::expect::CorrespondenceEntry::one_to_one(id, output));
                    }
                    None => {
                        entries.push(crate::expect::CorrespondenceEntry::split(id));
                    }
                }
            }
        }
    }
    // The view the expectations judge in (this profile's schema admits
    // `final` only, the plan's own working projection). Under `final` a
    // paragraph whose mark is tracked-deleted is not a final-view
    // paragraph of its own; the mirror for tracked-inserted marks under
    // an `original` view lands with that view.
    match view {
        v2::TextView::Final => {
            refuse_removed_final_paragraphs(primary_states, &mut entries)?;
        }
    }
    crate::expect::SourceToOutputCorrespondence::from_entries(entries).map_err(|e| {
        err(
            "INTERNAL",
            None,
            format!("expectations correspondence: {e}"),
        )
    })
}

/// One emitted primary paragraph's final-view disposition: whether its
/// paragraph mark is tracked-deleted, and whether any of its content
/// survives the final projection.
#[cfg_attr(test, derive(PartialEq, Eq))]
#[derive(Clone, Debug)]
pub(super) struct FinalView {
    deleted_mark: bool,
    survives: bool,
}

/// Rewrite one-to-one entries whose output paragraph is not a paragraph of
/// the view the expectations judge in (#1084 review F2, proposal §10: "A
/// missing or unresolved mapped paragraph is not a passing absence
/// assertion"). A paragraph whose mark is tracked-deleted
/// (`w:pPr/w:rPr/w:del|moveFrom`, the elements [`tracked`] writes) does
/// not exist as its own final paragraph: with no surviving content it is
/// removed, so its sources map to nothing and refuse split; with
/// surviving content it merges into the next paragraph without a deleted
/// mark, and every source sharing that merged paragraph refuses merged
/// (the settlement merge encoding: one entry per source, the shared index
/// repeated). Judged structurally on the primary's own DOM — the same walk
/// and marks the caller holds from producing those bytes — never by
/// matching text and never by reopening the emitted zip.
fn refuse_removed_final_paragraphs(
    states: &[FinalView],
    entries: &mut [crate::expect::CorrespondenceEntry],
) -> Result<(), EditError> {
    // The first follower without a deleted mark: where a merging
    // paragraph's text lands in the final view.
    let chain_end = |start: usize| -> Option<usize> {
        (start + 1..states.len()).find(|&end| !states[end].deleted_mark)
    };
    let mut landing: Vec<Option<Option<usize>>> = Vec::with_capacity(entries.len());
    let mut redirected = Vec::with_capacity(entries.len());
    for entry in entries.iter() {
        landing.push(match entry.output.as_slice() {
            // Only exact one-to-one targets are rewritten; settlement
            // splits and merges already refuse honestly.
            [index] => {
                let landing = match &states[*index] {
                    FinalView {
                        deleted_mark: false,
                        ..
                    } => Some(Some(*index)),
                    FinalView {
                        deleted_mark: true,
                        survives: false,
                    } => Some(None),
                    FinalView {
                        deleted_mark: true,
                        survives: true,
                    } => Some(chain_end(*index)),
                };
                redirected.push(landing.is_some_and(|landing| landing != Some(*index)));
                landing
            }
            _ => {
                redirected.push(false);
                None
            }
        });
    }
    let mut shares = vec![0usize; states.len()];
    for landing in landing.iter().flatten().flatten() {
        shares[*landing] += 1;
    }
    for ((entry, landing), redirected) in entries.iter_mut().zip(landing).zip(redirected) {
        let Some(landing) = landing else { continue };
        let Some(final_index) = landing else {
            entry.output.clear();
            continue;
        };
        // A redirected source merged into another final paragraph: the
        // chain-end paragraph is not its own, even when that paragraph is
        // an insertion no source entry names, so it refuses merged — the
        // settlement encoding for a shared final paragraph.
        let group = shares[final_index].max(usize::from(redirected) + 1);
        entry.output = vec![final_index; group];
    }
    Ok(())
}

/// One emitted primary paragraph's final-view disposition, read off the DOM
/// that produced the primary's bytes.
pub(super) fn paragraph_final_state(dom: &Dom, paragraph: NodeId) -> FinalView {
    FinalView {
        deleted_mark: mark_deleted(dom, paragraph),
        survives: final_run(dom, paragraph),
    }
}

/// The emitted primary's final-view states, recomputed the way the
/// correspondence computed them before #1087: reopen the bytes and walk
/// their paragraphs again, in the checker's projection order (the body's,
/// then each story part's). `finish` changes the producing DOM only
/// through `apply_watermark`, which no v2 operation reaches, so these
/// must equal the states read off that DOM —
/// `final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary`
/// pins that equivalence.
#[cfg(test)]
fn reparsed_final_views(primary: &[u8]) -> Vec<FinalView> {
    let opened = Opened::open(primary).expect("reopen the emitted primary");
    let mut states: Vec<FinalView> = crate::inspect::body_paragraph_nodes(&opened.dom, opened.body)
        .into_iter()
        .map(|node| paragraph_final_state(&opened.dom, node))
        .collect();
    for (_, _, part) in opened.story_parts() {
        let (dom, _, root) = crate::inspect::parse_part(&opened.pkg, &part)
            .unwrap_or_else(|e| panic!("reparse {part}: {e}"));
        states.extend(
            crate::inspect::story_paragraph_nodes(&dom, root)
                .into_iter()
                .map(|node| paragraph_final_state(&dom, node)),
        );
    }
    states
}

/// Whether the paragraph's mark (`w:pPr/w:rPr`) is tracked-deleted
/// (`w:del` or `w:moveFrom`), the same elements `tracked::mark` writes.
fn mark_deleted(dom: &Dom, paragraph: NodeId) -> bool {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.elements(rpr, None).iter().any(|&child| {
                dom.name(child).is_some_and(|name| {
                    name.namespace_name() == W::URI
                        && matches!(name.local_name(), "del" | "moveFrom")
                })
            })
        })
}

/// Whether any run of the paragraph survives the final view: a run outside
/// every deletion container (`w:del`/`w:moveFrom`) that carries content
/// besides its own properties.
fn final_run(dom: &Dom, paragraph: NodeId) -> bool {
    dom.descendants(paragraph, Some(&W::r()))
        .iter()
        .any(|&run| {
            !inside_deletion(dom, paragraph, run)
                && dom
                    .elements(run, None)
                    .iter()
                    .any(|&child| !dom.name_is(child, &W::r_pr()))
        })
}

/// Whether `node` sits inside a deletion container between the paragraph
/// and itself.
fn inside_deletion(dom: &Dom, paragraph: NodeId, node: NodeId) -> bool {
    let mut cursor = node;
    while let Some(parent) = dom.parent(cursor) {
        if parent == paragraph {
            return false;
        }
        if dom.name(parent).is_some_and(|name| {
            name.namespace_name() == W::URI && matches!(name.local_name(), "del" | "moveFrom")
        }) {
            return true;
        }
        cursor = parent;
    }
    false
}

fn legacy(plan: &PlanV2) -> EditPlan {
    EditPlan {
        schema_version: super::SCHEMA_VERSION,
        source_sha256: None,
        author: plan.author.clone(),
        date: Some(plan.date.clone()),
        initials: None,
        resolve_revisions: None,
        removed_anchor_comments: plan.removed_anchor_comments.unwrap_or_default(),
        existing_revisions: ExistingRevisions::Keep,
        operations: Vec::new(),
        update_fields: false,
        rewrite_others_comments: plan.rewrite_others_comments.unwrap_or(false),
    }
}
fn changes(bytes: &[u8]) -> Result<Vec<Change>, EditError> {
    crate::changes::list_changes(bytes).map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))
}
fn comments(bytes: &[u8]) -> Result<Vec<crate::comments::CommentRecord>, EditError> {
    crate::comments::list_comments(bytes).map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))
}
fn pointer(i: usize) -> String {
    format!("/operations/{i}")
}
fn miss((code, message, _): origin::Miss) -> EditError {
    err(&code, None, message)
}

/// The revision inventory is intentionally supplemented by the admitted XML:
/// absence from list_changes is not proof of absence of an unknown revision.
fn has_history(bytes: &[u8]) -> Result<bool, EditError> {
    if !changes(bytes)?.is_empty() {
        return Ok(true);
    }
    let opened = Opened::open(bytes).map_err(open_error)?;
    let known = crate::revision_processor::tracked_revisions_elements();
    for part in opened.pkg.parts() {
        if !part.ends_with(".xml") {
            continue;
        }
        let Some(xml) = opened.pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            continue;
        };
        for node in std::iter::once(root).chain(dom.descendants(root, None)) {
            if let Some(name) = dom.name(node) {
                let local = name.local_name();
                if name.namespace_name() == W::URI
                    && (known.contains(&name)
                        || local.ends_with("Change")
                        || local.ends_with("PrChange"))
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub(super) struct SourceModel {
    opened: Opened,
    loaded: Loaded,
    changes: Vec<Change>,
    comments: Vec<crate::comments::CommentRecord>,
    index: SourceIndex,
}

/// Immutable identity facts for this apply's original source. Target checks
/// must not parse a part or walk all of its paragraphs again for each ID.
#[derive(Default)]
struct SourceIndex {
    revision_counts: BTreeMap<String, usize>,
    revision_carriers: BTreeMap<(String, String), Vec<String>>,
    /// Per story, occurrences of each native `w14:paraId` key under the
    /// shared native-key policy (shape, range, case-folded), matching what
    /// generation counts.
    paragraph_id_counts: Vec<BTreeMap<u32, usize>>,
}
impl SourceIndex {
    fn new(opened: &Opened, loaded: &Loaded, changes: &[Change]) -> Self {
        let mut index = Self::default();
        for change in changes {
            *index.revision_counts.entry(change.id.clone()).or_default() += 1;
        }
        for story in &loaded.stories {
            let mut counts = BTreeMap::new();
            for p in opened.dom.descendants(story.root, Some(&W::p())) {
                if let Some(key) = opened
                    .dom
                    .attribute(p, &W14::name("paraId"))
                    .and_then(super::native_keys::para_id)
                {
                    *counts.entry(key).or_default() += 1;
                }
            }
            index.paragraph_id_counts.push(counts);
        }
        // A repeated relationship still counts as repeated carrier evidence,
        // as before, but its XML is parsed only once.
        let mut parts = BTreeMap::<String, usize>::new();
        for (part, _) in crate::revision_processor::revision_bearing_parts(&opened.pkg) {
            *parts.entry(part).or_default() += 1;
        }
        for (part, count) in parts {
            if let Some(story) = loaded.stories.iter().find(|s| s.part == part) {
                if let Some(root) = opened.dom.root(story.document) {
                    index.add_revision_part(&opened.dom, root, &part, &opened.main, count);
                }
            } else if let Some(xml) = opened.pkg.part_string(&part) {
                let mut dom = Dom::new();
                let document = dom.parse_xdocument(&xml);
                if let Some(root) = dom.root(document) {
                    index.add_revision_part(&dom, root, &part, &opened.main, count);
                }
            }
        }
        index
    }

    fn add_revision_part(&mut self, dom: &Dom, root: NodeId, part: &str, main: &str, count: usize) {
        let mut stories = vec![crate::ids::part_story(part)];
        if part == main && stories[0] != "body" {
            stories.push("body".into());
        }
        let known = crate::revision_processor::tracked_revisions_elements();
        for node in dom.descendants(root, None) {
            let Some(name) = dom.name(node).filter(|name| known.contains(name)) else {
                continue;
            };
            let Some(wid) = dom.attribute(node, &W::id()) else {
                continue;
            };
            for story in &stories {
                let carrier = revision_long_id(part, name.local_name(), story, wid);
                self.revision_carriers
                    .entry((story.clone(), wid.to_string()))
                    .or_default()
                    .extend(std::iter::repeat_n(carrier, count));
            }
        }
    }
}
/// The long id of the revision carrier `w:{local}` with `w:id` `wid` in
/// `part`, as `story` prints it. It follows the shared native-key policy:
/// a `w:id` Word could have written (an `i32`) is its own form, anything
/// else travels as its printed `j:short`.
pub(super) fn revision_long_id(part: &str, local: &str, story: &str, wid: &str) -> String {
    if super::native_keys::revision_id(wid).is_some() {
        format!("{}.w:{local}.w:id.{}", part_token(part), encode(wid))
    } else {
        format!(
            "{}.w:{local}.j:short.{}",
            part_token(part),
            crate::ids::ChangeId::new(story, wid).short()
        )
    }
}

#[derive(Clone)]
enum Entity {
    Paragraph(usize),
    Container(Vec<usize>, Option<usize>),
    Revision(String),
    Comment(u32),
}
struct Decisions {
    selection: ResolveRevisions,
    explicit: BTreeSet<String>,
    carried: BTreeSet<String>,
}
struct Authority {
    all: bool,
    paragraphs: BTreeSet<usize>,
    comments: BTreeSet<u32>,
}
struct Mapped {
    legacy: Operation,
    paragraphs: Vec<usize>,
}
impl SourceModel {
    pub(super) fn new(bytes: &[u8]) -> Result<Self, EditError> {
        let mut opened = Opened::open(bytes).map_err(open_error)?;
        let loaded = load_stories(&mut opened)?;
        let changes = changes(bytes)?;
        let index = SourceIndex::new(&opened, &loaded, &changes);
        Ok(Self {
            opened,
            loaded,
            changes,
            comments: comments(bytes)?,
            index,
        })
    }
    fn ids(&self) -> origin::Ids<'_> {
        origin::Ids {
            dom: &self.opened.dom,
            body: self.opened.body,
            story_ids: self.loaded.stories.iter().map(|s| s.id.as_str()).collect(),
            paragraph_nodes: &self.loaded.paragraph_nodes,
            paragraph_story: &self.loaded.paragraph_story,
        }
    }
    fn entity(&self, target: &Target) -> Result<Entity, EditError> {
        let bad = || {
            err(
                "TARGET_ID_MISMATCH",
                None,
                format!(
                    "paired IDs do not identify one supported original entity: {}",
                    target.short_id
                ),
            )
        };
        // A change whose `w:id` has no short spelling (one Word never
        // writes, such as a value beyond `u32`) travels as its printed
        // transitional colon id; `change_id` accepts both spellings while
        // `ShortId` only parses the short one.
        if let Some(cid) = crate::ids::change_id(&target.short_id) {
            if self.index.revision_counts.get(&target.short_id) != Some(&1) {
                return Err(bad());
            }
            let carriers = self
                .index
                .revision_carriers
                .get(&(cid.story, cid.wid))
                .ok_or_else(bad)?;
            if carriers.len() != 1 || carriers[0] != target.long_id {
                return Err(bad());
            }
            return Ok(Entity::Revision(target.short_id.clone()));
        }
        let id = target.short_id.parse::<ShortId>().map_err(|_| bad())?;
        match id {
            ShortId::Change { .. } => Err(bad()),
            ShortId::Comment(n) => {
                if !self.comments.iter().any(|c| c.id == n) {
                    return Err(bad());
                }
                // Native comments must be unambiguous in the admitted family.
                let family =
                    crate::comments::CommentFamily::load(&self.opened.pkg, &self.opened.main)
                        .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
                if family.is_empty() || target.long_id != format!("comments.w:comment.w:id.{n}") {
                    return Err(bad());
                }
                Ok(Entity::Comment(n))
            }
            ShortId::Block {
                ref story,
                ref path,
            } => {
                let story = match story {
                    Story::Body => "body",
                    Story::Part(s) => s,
                };
                let si = self
                    .loaded
                    .stories
                    .iter()
                    .position(|s| s.id == story)
                    .ok_or_else(bad)?;
                let dom = &self.opened.dom;
                let container = self.container_node(si, path, &target.short_id);
                if let Some(node) = container {
                    self.check_pair(target, si, node, None)?;
                    let descendants = dom.descendants(node, Some(&W::p()));
                    let members = self
                        .loaded
                        .paragraph_nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| descendants.contains(p))
                        .map(|(g, _)| g)
                        .collect();
                    let first = if dom.name_is(node, &W::tc()) {
                        self.loaded
                            .paragraph_nodes
                            .iter()
                            .enumerate()
                            .find_map(|(g, &p)| {
                                (dom.ancestors(p, Some(&W::tc())).first() == Some(&node))
                                    .then_some(g)
                            })
                    } else {
                        None
                    };
                    return Ok(Entity::Container(members, first));
                }
                let g = self
                    .ids()
                    .position(&Selector::Name(target.short_id.clone()))
                    .map_err(miss)?
                    .map_err(|_| bad())?;
                self.check_pair(
                    target,
                    si,
                    self.loaded.paragraph_nodes[g],
                    Some(&self.ids().id(g)),
                )?;
                Ok(Entity::Paragraph(g))
            }
            _ => Err(bad()),
        }
    }
    fn container_node(
        &self,
        si: usize,
        path: &[Step],
        short_id: &str,
    ) -> Option<crate::xmllinq::NodeId> {
        let root = self.loaded.stories[si].root;
        let dom = &self.opened.dom;
        match path {
            [Step::Table(n), rest @ ..]
                if rest.is_empty()
                    || matches!(rest, [Step::Row(_)] | [Step::Row(_), Step::Cell(_)]) =>
            {
                let tables = crate::inspect::top_level_table_nodes(dom, root);
                let mut node = *tables.get(*n)?;
                for step in rest {
                    let (name, n) = match step {
                        Step::Row(n) => (W::tr(), n),
                        Step::Cell(n) => (W::tc(), n),
                        _ => return None,
                    };
                    node = *dom.elements(node, Some(&name)).get(*n)?;
                }
                Some(node)
            }
            path if path
                .last()
                .is_some_and(|step| matches!(step, Step::ContentControl(_)))
                && self.loaded.stories[si].id == "body" =>
            {
                let records = crate::inspect::collect_controls(dom, root);
                let nodes = crate::inspect::body_control_nodes(dom, root);
                records
                    .iter()
                    .position(|c| c.id == short_id)
                    .and_then(|n| nodes.get(n).copied())
            }
            _ => None,
        }
    }

    fn pair(
        &self,
        short_id: &str,
        story: usize,
        node: crate::xmllinq::NodeId,
        canonical: Option<&str>,
    ) -> Result<Target, EditError> {
        let dom = &self.opened.dom;
        let name = dom
            .name(node)
            .ok_or_else(|| err("TARGET_ID_MISMATCH", None, "missing entity"))?;
        let key = dom.attribute(node, &W14::name("paraId"));
        // The shared native-key policy decides whether this paraId is a
        // key at all (shape and range) and uniqueness is counted on the
        // case-folded value, exactly as generation counts it.
        let unique = key
            .and_then(super::native_keys::para_id)
            .is_some_and(|key| self.index.paragraph_id_counts[story].get(&key) == Some(&1));
        let (attribute, value) = if unique {
            ("w14:paraId", encode(key.unwrap_or_default()))
        } else {
            ("j:short", encode(canonical.unwrap_or(short_id)))
        };
        let long_id = format!(
            "{}.w:{}.{}.{}",
            part_token(&self.loaded.stories[story].part),
            name.local_name(),
            attribute,
            value
        );
        Ok(Target {
            short_id: short_id.to_string(),
            long_id,
        })
    }

    /// Resolve a printed source selector through the same compiled entity and
    /// descendant resolver used by apply's scope enforcement. With
    /// `open_ranges`, the transitional open-ended `pN-` / `-pN` spellings
    /// close against their story's ends, as `read --for-edit`'s help
    /// documents; plan surfaces pass `false` (closed ranges only).
    pub(super) fn selected_paragraphs(
        &self,
        spec: &str,
        open_ranges: bool,
    ) -> Result<std::collections::HashSet<usize>, EditError> {
        let mut out = std::collections::HashSet::new();
        for item in spec.split(',').map(str::trim) {
            let item = if open_ranges {
                self.close_open_range(item)?
            } else {
                item.to_string()
            };
            let parsed = item.parse::<ShortId>().map_err(|_| {
                err(
                    "USAGE",
                    None,
                    "--select requires printed source IDs or closed paragraph ranges",
                )
            })?;
            self.select_parsed(parsed, &mut out)?;
        }
        Ok(out)
    }

    /// The closed spelling of an open range: `p17-` closes against the
    /// last paragraph of its story and `-p3` against its first. Any other
    /// item is returned unchanged; a side past its story's end keeps the
    /// `past the last paragraph` refusal.
    fn close_open_range(&self, item: &str) -> Result<String, EditError> {
        let close = |first: String, last: &str| Ok(format!("{first}-{last}"));
        if let Some(first) = item.strip_suffix('-').filter(|f| !f.is_empty()) {
            if let Some(error) = self.past_end_usage(first) {
                return Err(error);
            }
            if let Some((story, _)) = crate::ids::paragraph_place(first) {
                let len = self.story_paragraph_count(&story);
                if let Some(last) = len.checked_sub(1) {
                    return close(first.to_string(), &format!("p{last}"));
                }
            }
            return Ok(item.to_string());
        }
        if let Some(last) = item.strip_prefix('-').filter(|l| !l.is_empty()) {
            if let Some(error) = self.past_end_usage(last) {
                return Err(error);
            }
            if let Some((story, _)) = crate::ids::paragraph_place(last) {
                let first = crate::ids::paragraph_id(&story, 0);
                return close(first, last);
            }
            return Ok(item.to_string());
        }
        Ok(item.to_string())
    }

    fn story_paragraph_count(&self, story: &str) -> usize {
        self.loaded
            .stories
            .iter()
            .position(|s| s.id == story)
            .map_or(0, |si| {
                self.loaded
                    .paragraph_story
                    .iter()
                    .filter(|pair| pair.0 == si)
                    .count()
            })
    }

    /// `USAGE` when `id` names a paragraph of a story this document has at
    /// an index past that story's end (`p99 is past the last paragraph
    /// pN`); `None` when `id` is not that shape. A selector past the end
    /// is a malformed selector, not a missing document anchor.
    fn past_end_usage(&self, id: &str) -> Option<EditError> {
        let (story, index) = crate::ids::paragraph_place(id)?;
        let si = self.loaded.stories.iter().position(|s| s.id == story)?;
        let last = self
            .loaded
            .paragraph_story
            .iter()
            .filter(|pair| pair.0 == si)
            .count()
            .saturating_sub(1);
        (index > last).then(|| {
            err(
                "USAGE",
                None,
                format!(
                    "{id} is past the last paragraph {}",
                    crate::ids::paragraph_id(&story, last)
                ),
            )
        })
    }

    /// The paragraphs one parsed selector names.
    fn select_parsed(
        &self,
        parsed: ShortId,
        out: &mut std::collections::HashSet<usize>,
    ) -> Result<(), EditError> {
        if let ShortId::Range { first, last } = parsed {
            let mut end = *first.clone();
            let ShortId::Block { ref mut path, .. } = end else {
                // The parser guarantees a range's first side is a block
                // today; a grammar that breaks that invariant must refuse
                // the selector, never panic on user input.
                return Err(err("USAGE", None, "ranges must begin at a block"));
            };
            let Some(step @ Step::Paragraph(_)) = path.last_mut() else {
                return Err(err("USAGE", None, "ranges require paragraph endpoints"));
            };
            *step = Step::Paragraph(last);
            let from = first.to_string();
            let to = end.to_string();
            let first = self.selector_pair(&from)?;
            let last = self.selector_pair(&to)?;
            let (Entity::Paragraph(a), Entity::Paragraph(b)) =
                (self.entity(&first)?, self.entity(&last)?)
            else {
                return Err(err("USAGE", None, "ranges require two paragraph IDs"));
            };
            if self.loaded.paragraph_story[a].0 != self.loaded.paragraph_story[b].0 || a > b {
                return Err(err(
                    "USAGE",
                    None,
                    "ranges must be ordered paragraphs in the same story",
                ));
            }
            out.extend(a..=b);
        } else {
            let pair = self.selector_pair(&parsed.to_string())?;
            match self.entity(&pair)? {
                Entity::Paragraph(g) => {
                    out.insert(g);
                }
                Entity::Container(members, _) => out.extend(members),
                _ => {
                    return Err(err(
                        "USAGE",
                        None,
                        "--select requires paragraphs or supported containers",
                    ));
                }
            }
        }
        Ok(())
    }

    fn selector_pair(&self, id: &str) -> Result<Target, EditError> {
        let bad = || {
            err(
                "USAGE",
                None,
                format!("{id} is not a supported original-source paragraph or container"),
            )
        };
        let ShortId::Block { story, path } = id.parse::<ShortId>().map_err(|_| bad())? else {
            return Err(bad());
        };
        let story = match &story {
            Story::Body => "body",
            Story::Part(stem) => stem.as_str(),
        };
        let si = self
            .loaded
            .stories
            .iter()
            .position(|s| s.id == story)
            .ok_or_else(bad)?;
        if let Some(node) = self.container_node(si, &path, id) {
            return self.pair(id, si, node, None);
        }
        let g = self
            .ids()
            .position(&Selector::Name(id.to_string()))
            .map_err(|m| self.selector_miss(id, m))?
            .map_err(|_| bad())?;
        let canonical = self.ids().id(g);
        self.pair(
            &canonical,
            si,
            self.loaded.paragraph_nodes[g],
            Some(&canonical),
        )
    }

    /// A missing selector keeps its anchor code unless it names a paragraph
    /// past its story's end, which is a malformed selector (`USAGE`), not a
    /// missing document anchor.
    fn selector_miss(&self, id: &str, (code, message, _): origin::Miss) -> EditError {
        self.past_end_usage(id)
            .unwrap_or_else(|| err(&code, None, message))
    }

    fn check_pair(
        &self,
        target: &Target,
        story: usize,
        node: NodeId,
        canonical: Option<&str>,
    ) -> Result<(), EditError> {
        let expected = self.pair(&target.short_id, story, node, canonical)?.long_id;
        if target.long_id != expected {
            return Err(err(
                "TARGET_ID_MISMATCH",
                None,
                format!(
                    "{} does not match its original-source long ID",
                    target.short_id
                ),
            ));
        }
        Ok(())
    }
    fn decisions(&self, plan: &PlanV2) -> Result<Decisions, EditError> {
        let select = |targets: &[Target]| -> Result<BTreeSet<String>, EditError> {
            let mut set = BTreeSet::new();
            for target in targets {
                let Entity::Revision(id) = self.entity(target)? else {
                    return Err(err(
                        "INVALID_TARGET",
                        None,
                        "revision decision requires a revision",
                    ));
                };
                set.insert(id.clone());
                let c = self
                    .changes
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or_else(|| err("UNKNOWN_CHANGE", None, id))?;
                if let Some(name) = &c.move_name {
                    set.extend(
                        self.changes
                            .iter()
                            .filter(|c| c.move_name.as_ref() == Some(name))
                            .map(|c| c.id.clone()),
                    );
                }
            }
            Ok(set)
        };
        let accept = select(&plan.resolve_revisions.accept)?;
        let reject = select(&plan.resolve_revisions.reject)?;
        let effects = |set: &BTreeSet<String>, accepted: bool| {
            let mut all = set.clone();
            let mut carried = BTreeSet::new();
            let mut removed: BTreeSet<String> = self
                .changes
                .iter()
                .filter(|c| {
                    set.contains(&c.id)
                        && match c.kind {
                            ChangeKind::Deletion => accepted,
                            ChangeKind::Insertion => !accepted,
                            ChangeKind::Move => matches!(
                                (c.move_side, accepted),
                                (Some(MoveSide::From), true) | (Some(MoveSide::To), false)
                            ),
                            ChangeKind::Formatting => false,
                            // Either way one side's text goes.
                            ChangeKind::Replacement => true,
                        }
                })
                .map(|c| c.id.clone())
                .collect();
            loop {
                let old = removed.len();
                for c in &self.changes {
                    if c.inside
                        .as_ref()
                        .is_some_and(|parent| removed.contains(parent))
                    {
                        all.insert(c.id.clone());
                        carried.insert(c.id.clone());
                        removed.insert(c.id.clone());
                    }
                }
                if removed.len() == old {
                    break;
                }
            }
            (all, carried)
        };
        let (accept_effects, accept_carried) = effects(&accept, true);
        let (reject_effects, reject_carried) = effects(&reject, false);
        if let Some(both) = accept_effects.intersection(&reject_effects).next() {
            return Err(err(
                "REVISION_CONFLICT",
                None,
                format!("opposite decisions overlap at {both}, including move/dependent effects"),
            ));
        }
        let explicit = accept.union(&reject).cloned().collect();
        let carried = accept_carried.union(&reject_carried).cloned().collect();
        let filter = |set: BTreeSet<String>| (!set.is_empty()).then(|| ChangeFilter::ids(set));
        Ok(Decisions {
            selection: ResolveRevisions {
                accept: filter(accept),
                reject: filter(reject),
                reject_visible: false,
            },
            explicit,
            carried,
        })
    }

    fn scope(&self, scope: &Scope) -> Result<Authority, EditError> {
        let mut authority = Authority {
            all: matches!(scope, Scope::All {}),
            paragraphs: BTreeSet::new(),
            comments: BTreeSet::new(),
        };
        if let Scope::Selected { targets } = scope {
            for t in targets {
                match self.entity(t)? {
                    Entity::Paragraph(g) => {
                        authority.paragraphs.insert(g);
                    }
                    Entity::Container(members, _) => authority.paragraphs.extend(members),
                    Entity::Comment(id) => {
                        authority.comments.insert(id);
                    }
                    Entity::Revision(_) => {
                        return Err(err(
                            "INVALID_TARGET",
                            None,
                            "revision decisions are separate from direct scope",
                        ));
                    }
                }
            }
        }
        Ok(authority)
    }
    fn check_context(&self, plan: &PlanV2) -> Result<(), EditError> {
        for context in plan.source_context.iter().flatten() {
            let entity = self.entity(&context.target)?;
            let actual = match (&context.kind, entity) {
                (ContextKind::Paragraph, Entity::Paragraph(g)) => Some(v2::logical_text(
                    &self.opened.dom,
                    self.loaded.paragraph_nodes[g],
                )?),
                (ContextKind::Comment, Entity::Comment(id)) => self
                    .comments
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.text.clone()),
                (ContextKind::Opaque, _) if context.text.is_none() => None,
                _ => {
                    return Err(err(
                        "UNSUPPORTED_CONTEXT",
                        None,
                        "cannot verify this original-source observation",
                    ));
                }
            };
            if actual != context.text {
                return Err(err(
                    "SOURCE_CONTEXT_CHANGED",
                    None,
                    "source_context must remain an immutable original-source observation",
                ));
            }
        }
        Ok(())
    }
    fn operation(&self, operation: &v2::Operation, scope: &Authority) -> Result<Mapped, EditError> {
        use v2::Operation as V;
        let target = match operation {
            V::Replace { target, .. }
            | V::Insert { target, .. }
            | V::Delete { target, .. }
            | V::Rewrite { target, .. }
            | V::InsertParagraph { target, .. }
            | V::DeleteParagraph { target, .. }
            | V::Comment { target, .. }
            | V::ReplyComment { target, .. }
            | V::ResolveComment { target, .. }
            | V::EditComment { target, .. }
            | V::DeleteComment { target, .. } => target,
        };
        let entity = self.entity(target)?;
        let mut paragraphs = Vec::new();
        let mut comment_id = None;
        match entity {
            Entity::Paragraph(g) => paragraphs.push(g),
            Entity::Container(_, first) => paragraphs.push(first.ok_or_else(|| {
                err(
                    "INVALID_TARGET",
                    None,
                    "operation requires a paragraph, or a cell with an own paragraph",
                )
            })?),
            Entity::Comment(id) => {
                comment_id = Some(id);
            }
            Entity::Revision(_) => {
                return Err(err(
                    "INVALID_TARGET",
                    None,
                    "revision is not a direct operation target",
                ));
            }
        }
        let paragraph = paragraphs
            .first()
            .map(|&g| Selector::Name(self.ids().id(g)));
        let occurrence = |n: Option<u64>| n.map(|n| usize::try_from(n).unwrap_or(usize::MAX));
        let attachment = |c: &Option<AttachedComment>| c.as_ref().map(|c| c.text.clone());
        let p = || {
            paragraph
                .clone()
                .ok_or_else(|| err("INVALID_TARGET", None, "operation requires a paragraph"))
        };
        let c = || {
            comment_id.ok_or_else(|| err("INVALID_TARGET", None, "operation requires a comment"))
        };
        let kind = match operation {
            V::Replace {
                find,
                text,
                occurrence: n,
                attached_comment,
                ..
            } => OperationKind::Replace {
                paragraph: p()?,
                find: find.clone(),
                replacement: text.clone(),
                occurrence: occurrence(*n),
                comment: attachment(attached_comment),
                format: None,
                whole: false,
            },
            V::Insert {
                text,
                before_text,
                after_text,
                edge,
                occurrence: n,
                attached_comment,
                ..
            } => OperationKind::Insert {
                paragraph: p()?,
                text: text.clone(),
                before: before_text.clone(),
                after: after_text.clone(),
                position: *edge,
                occurrence: occurrence(*n),
                comment: attachment(attached_comment),
                format: None,
            },
            V::Delete {
                find,
                occurrence: n,
                ..
            } => OperationKind::Delete {
                paragraph: p()?,
                find: find.clone(),
                occurrence: occurrence(*n),
                comment: None,
            },
            V::Rewrite { text, .. } => OperationKind::Rewrite {
                paragraph: p()?,
                text: text.clone(),
                comment: None,
            },
            V::InsertParagraph {
                position,
                text,
                attached_comment,
                ..
            } => OperationKind::InsertParagraph {
                paragraph: p()?,
                position: *position,
                runs: vec![super::RunSpec {
                    text: text.clone(),
                    ..Default::default()
                }],
                like: None,
                style: None,
                comment: attachment(attached_comment),
            },
            V::DeleteParagraph { .. } => OperationKind::DeleteParagraph {
                paragraph: p()?,
                comment: None,
            },
            V::Comment {
                text,
                find,
                occurrence: n,
                through,
                ..
            } => {
                let through = if let Some(target) = through {
                    let Entity::Paragraph(last) = self.entity(target)? else {
                        return Err(err("INVALID_TARGET", None, "through requires a paragraph"));
                    };
                    let first = *paragraphs.first().ok_or_else(|| {
                        err("INVALID_TARGET", None, "comment requires a paragraph")
                    })?;
                    if last < first
                        || self.loaded.paragraph_story[first].0
                            != self.loaded.paragraph_story[last].0
                    {
                        return Err(err(
                            "INVALID_EDIT",
                            None,
                            "through must follow target in the same story",
                        ));
                    }
                    paragraphs = (first..=last).collect();
                    Some(Selector::Name(self.ids().id(last)))
                } else {
                    None
                };
                OperationKind::Comment {
                    paragraph: p()?,
                    text: text.clone(),
                    find: find.clone(),
                    occurrence: occurrence(*n),
                    through,
                }
            }
            V::ReplyComment { text, .. } => OperationKind::ReplyComment {
                comment_id: c()?,
                text: text.clone(),
            },
            V::ResolveComment { done, .. } => OperationKind::ResolveComment {
                comment_id: c()?,
                done: *done,
            },
            V::EditComment { text, .. } => OperationKind::EditComment {
                comment_id: c()?,
                text: text.clone(),
            },
            V::DeleteComment {
                include_replies, ..
            } => {
                let id = c()?;
                let family =
                    crate::comments::CommentFamily::load(&self.opened.pkg, &self.opened.main)
                        .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
                if !include_replies && family.with_replies(id).len() > 1 {
                    return Err(err(
                        "COMMENT_HAS_REPLIES",
                        None,
                        "delete_comment requires include_replies for this thread",
                    ));
                }
                OperationKind::DeleteComment { comment_id: id }
            }
        };
        if !scope.all
            && (paragraphs.iter().any(|g| !scope.paragraphs.contains(g))
                || comment_id.is_some_and(|id| !scope.comments.contains(&id)))
        {
            return Err(err(
                "OUT_OF_SCOPE",
                None,
                "direct target or covered paragraph is outside original-source scope",
            ));
        }
        Ok(Mapped {
            legacy: Operation {
                id: None,
                group: None,
                kind,
            },
            paragraphs,
        })
    }
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':') {
                (b as char).to_string()
            } else {
                format!("-{b:02X}")
            }
        })
        .collect()
}
fn part_token(part: &str) -> String {
    let part = part.trim_start_matches('/');
    if let Some(stem) = part
        .strip_prefix("word/")
        .and_then(|p| p.strip_suffix(".xml"))
        && (matches!(
            stem,
            "document" | "footnotes" | "endnotes" | "comments" | "styles" | "numbering"
        ) || ["header", "footer"].iter().any(|prefix| {
            stem.strip_prefix(prefix)
                .is_some_and(|n| n.parse::<u32>().is_ok_and(|v| v.to_string() == n))
        }))
    {
        return stem.to_string();
    }
    let payload: String = part
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'_' {
                (b as char).to_string()
            } else {
                format!("-{b:02X}")
            }
        })
        .collect();
    format!("pkg:{payload}")
}

fn outcome(i: usize, kind: &OperationKind) -> EditOutcome {
    EditOutcome {
        id: pointer(i),
        kind: super::kind_name(kind).into(),
        status: "ok".into(),
        paragraph: None,
        matches: 0,
        context: None,
        comment_id: None,
        code: None,
        message: None,
        anchor_given: None,
        anchor_read_as: None,
        near: None,
    }
}
fn mapped_paragraph(tx: &Transaction<'_>, source: usize) -> Result<usize, EditError> {
    match &tx.origin {
        Some(origin) => origin.exact(source).map_err(miss),
        None => Ok(source),
    }
}
fn resolve(tx: &mut Transaction<'_>, plan: &PlanV2, mapped: &[Mapped]) -> Result<(), EditError> {
    // Hash every supplied row, including unchanged scaffolding, before dropping
    // any text component. Gather affected rows rather than reporting a guessed
    // post-settlement address or refreshing authored text.
    let mut stale = Vec::new();
    for (i, (operation, m)) in plan.operations.iter().zip(mapped).enumerate() {
        for &g in &m.paragraphs {
            mapped_paragraph(tx, g).map_err(|mut e| {
                // Exact correspondence still gates merges/splits. For a
                // removed comment target, keep Origin's settlement cause.
                if matches!(operation, v2::Operation::Comment { .. })
                    && let Some(origin) = &tx.origin
                    && let Err(cause) = origin.follow(g, true, |p| tx.paragraph_id(p), |_| None)
                {
                    e = miss(cause);
                    e.message.push_str(&format!(
                        "; selected accept={:?}, reject={:?}",
                        tx.resolved_revisions.accepted, tx.resolved_revisions.rejected
                    ));
                }
                e.operation = Some(pointer(i));
                e
            })?;
        }
        if let v2::Operation::Rewrite { before_sha256, .. } = operation {
            let p = mapped_paragraph(tx, m.paragraphs[0])?;
            let text = v2::logical_text(&tx.opened.dom, tx.paragraph_nodes[p])?;
            let hash = source_sha256(text.as_bytes());
            if hash != *before_sha256 {
                let mut o = outcome(i, &m.legacy.kind);
                o.status = "failed".into();
                o.code = Some("WORKING_BASE_CHANGED".into());
                o.paragraph = Some(tx.paragraph_id(p));
                o.context = Some(text.chars().take(512).collect());
                o.message = Some(format!(
                    "working before_sha256={hash}; policy={:?}; selected accept={:?}, reject={:?}; refresh/remove this row explicitly",
                    plan.policy.existing_revisions,
                    tx.resolved_revisions
                        .accepted
                        .iter()
                        .take(16)
                        .collect::<Vec<_>>(),
                    tx.resolved_revisions
                        .rejected
                        .iter()
                        .take(16)
                        .collect::<Vec<_>>()
                ));
                stale.push(o);
            }
        }
    }
    if let Some(first) = stale.first() {
        let mut e = err(
            "WORKING_BASE_CHANGED",
            Some(&first.id),
            first.message.clone().unwrap_or_default(),
        );
        e.outcomes = stale;
        return Err(e);
    }
    let mut rewrite_owners = BTreeMap::new();
    for (i, (operation, m)) in plan.operations.iter().zip(mapped).enumerate() {
        let mut o = outcome(i, &m.legacy.kind);
        tx.revision_refusal.take();
        let resolved = resolve_operation(tx, operation, m, &mut o).map_err(|e| {
            let mut e = tx.attach_revision_refusal(e);
            e.operation = Some(pointer(i));
            e.outcomes = tx.outcomes.clone();
            e
        })?;
        let resolved = match operation {
            v2::Operation::Replace { .. } => replacement_pieces(tx, i, resolved),
            _ => resolved,
        };
        if matches!(operation, v2::Operation::Rewrite { .. })
            && resolved.iter().any(|r| matches!(r, Resolved::Text { .. }))
        {
            let para = mapped_paragraph(tx, m.paragraphs[0])?;
            if rewrite_owners.insert(para, i).is_some() {
                return Err(err(
                    "CONFLICT",
                    Some(&pointer(i)),
                    "two changed rewrites own one working paragraph",
                ));
            }
        }
        tx.note_rewrites(tx.outcomes.len(), &resolved);
        tx.resolved.extend(resolved.into_iter().map(|r| (i, r)));
        tx.outcomes.push(o);
    }
    for &(i, ref resolved) in &tx.resolved {
        if let Resolved::Text { para, .. } = resolved
            && rewrite_owners.get(para).is_some_and(|&owner| owner != i)
        {
            return Err(err(
                "CONFLICT",
                Some(&pointer(i)),
                "changed full rewrite conflicts with another active text edit in its paragraph",
            ));
        }
    }
    check_boundary_ownership(tx)?;
    compose_threads(tx)?;
    if let Err(error) = tx.check_conflicts_v2() {
        return Err(diagnose_conflict(tx, error, plan.policy.track == Track::On));
    }
    check_comment_anchors(tx, plan.policy.track == Track::On)
}

fn diagnose_conflict(tx: &Transaction<'_>, mut error: EditError, tracked: bool) -> EditError {
    // Keep real text/thread conflicts; specialize only engine anchor conflicts.
    if error.code == super::COMMENT_ANCHOR_CONFLICT
        && let Err(anchor) = check_comment_anchors(tx, tracked)
    {
        return anchor;
    }
    if error.code == "OVERLAPPING_EDITS" {
        error.code = "CONFLICT".into();
        for outcome in &mut error.outcomes {
            if outcome.code.as_deref() == Some("OVERLAPPING_EDITS") {
                outcome.code = Some("CONFLICT".into());
            }
        }
    }
    error
}

/// Same-position inserts share array order only when they agree on the
/// source side that owns them. A side removed by another edit cannot lend
/// its run/formatting to an insertion without an explicit ownership rule.
fn check_boundary_ownership(tx: &Transaction<'_>) -> Result<(), EditError> {
    let mut paragraphs = BTreeMap::<usize, Vec<_>>::new();
    // A replacement split into word-level pieces owns its whole match, as
    // before the split: its pieces are not insertions of their own. A
    // range's side never matters below.
    for (&i, &(para, start, end)) in &tx.claims {
        paragraphs
            .entry(para)
            .or_default()
            .push((start, end, true, i));
    }
    for (i, r) in &tx.resolved {
        if let Resolved::Text {
            para,
            start,
            end,
            attach_before,
            ..
        } = r
            && !tx.claims.contains_key(i)
        {
            paragraphs
                .entry(*para)
                .or_default()
                .push((*start, *end, *attach_before, *i));
        }
    }
    for ranges in paragraphs.values_mut() {
        ranges.sort_by_key(|&(start, end, _, i)| (start, end, i));
        for &[(s0, e0, a0, i0), (s1, e1, a1, i1)] in ranges.array_windows() {
            let different_sides = s0 == e0 && s1 == e1 && s0 == s1 && a0 != a1;
            let removed_right = s0 == e0 && s0 == s1 && s1 < e1 && !a0;
            let removed_left = s0 < e0 && s1 == e1 && e0 == s1 && a1;
            if i0 != i1 && (different_sides || removed_right || removed_left) {
                return Err(err(
                    "CONFLICT",
                    Some(&pointer(i1)),
                    "ambiguous insertion boundary ownership; use one explicit text edit",
                ));
            }
        }
    }
    Ok(())
}

/// All aliases of a thread share one deletion/conflict domain and one final
/// state. The engine writes replies before state changes. Internal state
/// preservation reuses that path without inventing a public operation/result.
fn compose_threads(tx: &mut Transaction<'_>) -> Result<(), EditError> {
    use super::ThreadOp;
    let family = tx.family.as_ref().expect("loaded in start");
    let mut states = BTreeSet::new();
    let mut replies = BTreeMap::new();
    let deleted: Vec<_> = tx
        .resolved
        .iter()
        .filter_map(|(i, r)| match r {
            Resolved::Thread {
                op: ThreadOp::Delete { id },
                ..
            } => Some((*i, family.thread_root(*id))),
            _ => None,
        })
        .collect();
    for (i, r) in &tx.resolved {
        let Resolved::Thread { op, .. } = r else {
            continue;
        };
        let root = family.thread_root(op.target());
        if deleted.iter().any(|(j, gone)| j != i && *gone == root) {
            return Err(comment_error(
                tx,
                *i,
                "CONFLICT",
                "delete_comment conflicts with another mutation of its thread",
            ));
        }
        match op {
            ThreadOp::Resolve { .. } if !states.insert(root) => {
                return Err(comment_error(
                    tx,
                    *i,
                    "CONFLICT",
                    "only one final state is allowed per thread",
                ));
            }
            ThreadOp::Reply { .. } => {
                replies.entry(root).or_insert(*i);
            }
            _ => {}
        }
    }
    let prior: Vec<_> = replies
        .into_iter()
        .filter(|(root, _)| !states.contains(root))
        .map(|(root, i)| {
            (
                i,
                root,
                family.thread_state(root).expect("resolved thread").1,
            )
        })
        .collect();
    for (_, r) in &mut tx.resolved {
        if let Resolved::Thread {
            op: ThreadOp::Resolve { id, .. },
            ..
        } = r
        {
            *id = family.thread_root(*id);
        }
    }
    for (i, id, done) in prior {
        tx.resolved.push((
            i,
            Resolved::Thread {
                op: ThreadOp::Resolve { id, done },
                stories: Vec::new(),
                anchor: None,
            },
        ));
    }
    Ok(())
}

fn comment_error(tx: &Transaction<'_>, i: usize, code: &str, message: &str) -> EditError {
    let mut error = tx.conflict(i, message);
    error.code = code.into();
    error.outcomes[i].code = Some(code.into());
    error
}

type AnchorEdits<'a> = BTreeMap<usize, Vec<(usize, &'a Resolved)>>;

/// A supported range keeps its endpoints through composition. An exact
/// nonempty replacement is a defined correspondence (the replacement is the
/// anchor); containing/disjoint ranges retain their surviving content. No
/// rule maps a boundary strictly inside removed text or invents a neighbor
/// for an emptied anchor. An existing anchor may instead retain its original
/// markers on pure tracked deletion. Check existing and requested anchors.
fn check_comment_anchors(tx: &Transaction<'_>, tracked: bool) -> Result<(), EditError> {
    let mut edits = AnchorEdits::new();
    for (i, r) in &tx.resolved {
        if let Resolved::Text { para, .. } | Resolved::DeleteParagraph { para } = r {
            edits.entry(*para).or_default().push((*i, r));
        }
    }
    // Thread-only, comment-only and unchanged plans cannot change an
    // existing anchor. In particular they need no document mark scan.
    if edits.is_empty() {
        return Ok(());
    }
    let family = tx.family.as_ref().expect("loaded in start");
    let deleted_comments: BTreeSet<_> = tx
        .resolved
        .iter()
        .filter_map(|(_, r)| match r {
            Resolved::Thread {
                op: super::ThreadOp::Delete { id },
                ..
            } => Some(family.with_replies(*id)),
            _ => None,
        })
        .flatten()
        .collect();
    let ids = family.order();
    let anchors = if ids.is_empty() {
        BTreeMap::new()
    } else {
        tx.comment_anchors()
    };
    for id in ids {
        if !deleted_comments.contains(&id)
            && let Some(&covered) = anchors.get(&id.to_string())
        {
            check_comment_anchor(tx, &edits, covered, None, false, tracked)?;
        }
    }
    for (i, r) in &tx.resolved {
        let covered = match r {
            Resolved::CommentRange {
                para, start, end, ..
            } => super::Covered {
                first: *para,
                start: *start,
                last: *para,
                end: *end,
            },
            Resolved::CommentSpan { para, last, .. } => super::Covered {
                first: *para,
                start: 0,
                last: *last,
                end: tx.projections[*last].text.len(),
            },
            _ => continue,
        };
        check_comment_anchor(
            tx,
            &edits,
            covered,
            Some(*i),
            matches!(r, Resolved::CommentSpan { .. }),
            false,
        )?;
    }
    Ok(())
}

fn check_comment_anchor(
    tx: &Transaction<'_>,
    edits: &AnchorEdits<'_>,
    covered: super::Covered,
    owner: Option<usize>,
    full_paragraphs: bool,
    mut preserve_deleted: bool,
) -> Result<(), EditError> {
    let mut remaining = 0_i128;
    let mut original = 0;
    let mut changed = None;
    let mut boundary = None;
    let mut removed_paragraph = None;
    for para in covered.first..=covered.last {
        let start = if para == covered.first {
            covered.start
        } else {
            0
        };
        let end = if para == covered.last {
            covered.end
        } else {
            tx.projections[para].text.len()
        };
        original += end - start;
        if let Some((i, _)) = edits
            .get(&para)
            .into_iter()
            .flatten()
            .find(|(_, r)| matches!(r, Resolved::DeleteParagraph { .. }))
        {
            changed = Some(*i);
            // Existing and requested containing ranges keep both markers
            // when only an interior paragraph disappears.
            if para == covered.first || para == covered.last {
                removed_paragraph = Some(*i);
            }
            continue;
        }
        let mut length = (end - start) as i128;
        for &(i, r) in edits.get(&para).into_iter().flatten() {
            let Resolved::Text {
                start: s,
                end: e,
                replacement,
                ..
            } = r
            else {
                continue;
            };
            // Only an existing anchor on pure tracked deletion has this
            // preservation rule. A replacement or insertion touching it
            // needs a correspondence into nonempty resulting content.
            if !replacement.is_empty() && *s <= end && *e >= start {
                preserve_deleted = false;
            }
            if *s < end && *e > start {
                changed = Some(i);
                length -= (end.min(*e) - start.max(*s)) as i128;
            }
            // Replacement characters have a correspondence only when
            // their complete source interval belongs to the anchor. Range
            // anchors exclude edge insertions; paragraph anchors include them.
            let contained = *s >= start && *e <= end;
            let owns_insertion = full_paragraphs || (start < *s && *s < end);
            if contained && (*e > *s || owns_insertion) {
                length += replacement.len() as i128;
            }
            if (*s < start && start < *e) || (*s < end && end < *e) {
                boundary = Some(i);
            }
        }
        remaining += length;
    }
    if let Some(i) = changed
        && remaining <= 0
        && (original > 0 || removed_paragraph.is_some())
    {
        // The preserving emitter leaves the original markers around the
        // deleted runs/paragraph. They still anchor the original comment in
        // the tracked primary; rejecting the deletion restores its source
        // range. New comments cannot acquire this deleted-content anchor.
        if preserve_deleted {
            return Ok(());
        }
        return Err(comment_error(
            tx,
            owner.unwrap_or(i),
            "COMMENT_ANCHOR_REMOVED",
            "composed edits remove the entire comment anchor; no supported preservation rule applies",
        ));
    }
    if let Some(i) = boundary.or(removed_paragraph) {
        return Err(comment_error(
            tx,
            owner.unwrap_or(i),
            "COMMENT_ANCHOR_UNMAPPABLE",
            "comment boundary has no supported mapping through removed text or paragraphs",
        ));
    }
    Ok(())
}

fn resolve_operation(
    tx: &Transaction<'_>,
    operation: &v2::Operation,
    m: &Mapped,
    o: &mut EditOutcome,
) -> Result<Vec<Resolved>, EditError> {
    use v2::Operation as V;
    let p = m
        .paragraphs
        .first()
        .map(|&g| mapped_paragraph(tx, g))
        .transpose()?;
    let exact = |p| v2::logical_text(&tx.opened.dom, tx.paragraph_nodes[p]);
    let occurrence = |n: Option<u64>| n.map(|n| usize::try_from(n).unwrap_or(usize::MAX));
    let note = |c: &Option<AttachedComment>| c.as_ref().map(|c| c.text.clone());
    let p = p.unwrap_or(0);
    // A miss that reaches into another author's deletion names it (#1108).
    let select = |old: &str, needle: &str, n: Option<u64>| {
        anchor::v2_select(old, needle, occurrence(n)).map_err(|e| {
            match (e.code == "ANCHOR_NOT_FOUND")
                .then(|| tx.removed_anchor_in(p, needle))
                .flatten()
            {
                Some(hint) => err("UNSUPPORTED_STRUCTURE", None, hint),
                None => e,
            }
        })
    };
    o.paragraph = (!m.paragraphs.is_empty()).then(|| tx.paragraph_id(p));
    o.matches = 1;
    match operation {
        V::Replace {
            find,
            text,
            occurrence: n,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let (start, end) = select(&old, find, *n)?;
            if &old[start..end] == text {
                comment_component(tx, p, start, end, note(attached_comment))
            } else {
                Ok(vec![text_component(
                    tx,
                    p,
                    start,
                    end,
                    text,
                    note(attached_comment),
                    true,
                )?])
            }
        }
        V::Delete {
            find,
            occurrence: n,
            ..
        } => {
            let old = exact(p)?;
            let (start, end) = select(&old, find, *n)?;
            Ok(vec![text_component(tx, p, start, end, "", None, true)?])
        }
        V::Insert {
            text,
            before_text,
            after_text,
            edge,
            occurrence: n,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let (at, attach) = match (before_text, after_text, edge) {
                (Some(anchor), None, None) => (select(&old, anchor, *n)?.0, false),
                (None, Some(anchor), None) => (select(&old, anchor, *n)?.1, true),
                (None, None, Some(super::Edge::Start)) => (0, false),
                (None, None, Some(super::Edge::End)) => (old.len(), true),
                _ => {
                    return Err(err(
                        "INVALID_EDIT",
                        None,
                        "insert requires exactly one anchor",
                    ));
                }
            };
            Ok(vec![text_component(
                tx,
                p,
                at,
                at,
                text,
                note(attached_comment),
                attach,
            )?])
        }
        V::Rewrite {
            text,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let mut resolved = Vec::new();
            for (start, end, replacement) in exact_rewrite_ranges(&old, text) {
                resolved.push(text_component(tx, p, start, end, &replacement, None, true)?);
            }
            if let Some(note) = note(attached_comment) {
                // Reuse the engine's full-paragraph anchoring after text
                // composition, including characters inserted at either edge.
                comment_component(tx, p, 0, old.len(), Some(note.clone()))?;
                resolved.push(Resolved::CommentSpan {
                    para: p,
                    last: p,
                    text: note,
                });
            }
            Ok(resolved)
        }
        V::Comment {
            text,
            find,
            occurrence: n,
            through: None,
            ..
        } => {
            let (start, end) = if let Some(find) = find {
                let old = exact(p)?;
                select(&old, find, *n)?
            } else {
                (0, tx.projections[p].text.len())
            };
            comment_component(tx, p, start, end, Some(text.clone()))
        }
        V::ReplyComment { .. }
        | V::ResolveComment { .. }
        | V::EditComment { .. }
        | V::DeleteComment { .. } => {
            let (r, resolved) = tx
                .resolve_thread(&o.id, &m.legacy.kind)
                .map_err(|boxed| boxed.0)?;
            *o = EditOutcome {
                kind: o.kind.clone(),
                status: "ok".into(),
                ..resolved
            };
            Ok(vec![r])
        }
        _ => {
            let (r, resolved) = tx
                .resolve_one(&o.id, &m.legacy.kind)
                .map_err(|boxed| boxed.0)?;
            *o = EditOutcome {
                kind: o.kind.clone(),
                status: "ok".into(),
                ..resolved
            };
            Ok(vec![r])
        }
    }
}
/// A replacement inside one insertion as the word-level edits the redline
/// replays it as ([`tracked::insertion_pieces`]): the clean copy then keeps
/// the words the redline keeps, in their runs and with their formatting
/// history (MiniMax on #1664). Each piece is checked where it lands, as a
/// v1 `replace`'s are; its comment covers the whole match, and its pieces
/// conflict as the whole match does.
fn replacement_pieces(
    tx: &mut Transaction<'_>,
    i: usize,
    resolved: Vec<Resolved>,
) -> Vec<Resolved> {
    let (para, start, end, replacement, comment) = match resolved.as_slice() {
        [
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                comment,
                format: None,
                ..
            },
        ] => (*para, *start, *end, replacement.clone(), comment.clone()),
        _ => return resolved,
    };
    let node = tx.paragraph_nodes[para];
    let Some(pieces) = tracked::insertion_pieces(&tx.opened.dom, node, start, end, &replacement)
    else {
        return resolved;
    };
    if pieces == [(start, end, replacement)] {
        return resolved;
    }
    let Some(mut out) = tx.checked_pieces(para, pieces) else {
        return resolved;
    };
    tx.claims.insert(i, (para, start, end));
    if let Some(text) = comment {
        out.push(Resolved::CommentRange {
            para,
            start,
            end,
            text,
        });
    }
    out
}
fn comment_component(
    tx: &Transaction<'_>,
    p: usize,
    start: usize,
    end: usize,
    note: Option<String>,
) -> Result<Vec<Resolved>, EditError> {
    let Some(text) = note else {
        return Ok(Vec::new());
    };
    if tx.paragraph_story[p].0 != 0 {
        return Err(err(
            "COMMENT_NOT_IN_BODY",
            None,
            "comments are supported in the body only",
        ));
    }
    super::check_comment(&text).map_err(|m| err("INVALID_EDIT", None, m))?;
    Ok(vec![Resolved::CommentRange {
        para: p,
        start,
        end,
        text,
    }])
}
fn text_component(
    tx: &Transaction<'_>,
    p: usize,
    start: usize,
    end: usize,
    text: &str,
    note: Option<String>,
    attach: bool,
) -> Result<Resolved, EditError> {
    super::check_run_text(text).map_err(|m| err("INVALID_EDIT", None, m))?;
    if let Some(note) = &note {
        if tx.paragraph_story[p].0 != 0 {
            return Err(err(
                "COMMENT_NOT_IN_BODY",
                None,
                "comments are supported in the body only",
            ));
        }
        super::check_comment(note).map_err(|m| err("INVALID_EDIT", None, m))?;
    }
    let projection = &tx.projections[p];
    if start == end {
        tx.check_insert_position(projection, start, attach)
            .map_err(|m| {
                err(
                    "UNSUPPORTED_INSERTION_SIDE",
                    None,
                    format!("{}{m}", super::refusal::INSERTION_SIDE_PREFIX),
                )
            })?;
    } else {
        tx.check_range(projection, start, end)
            .map_err(|m| err("UNSUPPORTED_STRUCTURE", None, m))?;
    }
    Ok(Resolved::Text {
        para: p,
        start,
        end,
        replacement: text.into(),
        comment: note,
        attach_before: attach,
        format: None,
    })
}

/// The same diff library as the legacy rewrite adapter, with exact character
/// keys so whitespace/control changes cannot disappear through normalization.
fn exact_rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)> {
    let old_chars = old.chars().collect::<Vec<_>>();
    let new_chars = new.chars().collect::<Vec<_>>();
    let offsets = old
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(old.len()))
        .collect::<Vec<_>>();
    similar::capture_diff_slices(similar::Algorithm::Myers, &old_chars, &new_chars)
        .into_iter()
        .filter_map(|op| {
            let (tag, from, to) = op.as_tag_tuple();
            (tag != similar::DiffTag::Equal).then(|| {
                (
                    offsets[from.start],
                    offsets[from.end],
                    new_chars[to].iter().collect(),
                )
            })
        })
        .collect()
}

/// What the resolved operations reach, snapshotted before the
/// transaction applies: the stored projection nodes themselves are mutated
/// by `apply`.
#[derive(Debug, Default)]
struct Edited {
    /// The revisions, by short id, whose content may change: insertions a
    /// text operation touches, and every revision of a deleted paragraph.
    ids: BTreeSet<String>,
    /// Each run formatting change a text operation reaches, by short id,
    /// with what the operations take from its run. A direct edit may take
    /// those words out from under such a record, nothing else; new words
    /// never join it (#1658).
    rewritten: BTreeMap<String, Rewrite>,
}

/// What the operations leave of a run formatting change's run.
#[derive(Debug, Default)]
struct Rewrite {
    /// The run's words outside every operation's range, as its formatting
    /// change lists them.
    words: String,
    /// How many of the run's characters the operations take: one per
    /// character of text, one per atom ([`crate::changes::atom`]), as
    /// [`Recorded::characters`] counts them.
    taken: usize,
}

/// The revisions the resolved operations reach: only insertion content a
/// text operation touches may change, a deleted paragraph reaches every
/// revision it holds, and a text operation may take words out of the run
/// formatting changes it reaches.
fn edited_revisions(tx: &Transaction<'_>) -> Edited {
    let mut ids = BTreeSet::new();
    let dom = &tx.opened.dom;
    // Each paragraph's operation ranges; the runs of each formatting
    // change the operations reach; the ids paragraph deletions reach.
    let mut ranges = BTreeMap::<usize, Vec<(usize, usize)>>::new();
    let mut recorded = BTreeMap::<String, Vec<(usize, NodeId)>>::new();
    let mut deleted = BTreeSet::new();
    for (_, resolved) in &tx.resolved {
        // A deleted paragraph deletes inside every insertion of its runs
        // (#1136). A direct deletion takes the paragraph, and so every
        // revision in it, mark included.
        if let Resolved::DeleteParagraph { para } = resolved {
            let story = &tx.stories[tx.paragraph_story[*para].0].id;
            let node = tx.paragraph_nodes[*para];
            for revision in dom.descendants(node, None) {
                if super::hint::Held::of(dom, revision).is_some()
                    && let Some(wid) = dom.attribute(revision, &W::id())
                {
                    let id = crate::ids::ChangeId::new(story, wid).short();
                    deleted.insert(id.clone());
                    ids.insert(id);
                }
            }
            continue;
        }
        let Resolved::Text {
            para,
            start,
            end,
            attach_before,
            ..
        } = resolved
        else {
            continue;
        };
        let projection = &tx.projections[*para];
        ranges.entry(*para).or_default().push((*start, *end));
        let touched: Vec<_> = if start == end {
            super::attach_segment(projection, *start, *attach_before)
                .into_iter()
                .collect()
        } else {
            projection
                .segments
                .iter()
                .filter(|s| s.start < *end && *start < s.end)
                .collect()
        };
        let story = &tx.stories[tx.paragraph_story[*para].0].id;
        for segment in touched {
            if let Some(ins) = super::editable_insertion(dom, &segment.piece)
                && let Some(wid) = dom.attribute(ins, &W::id())
            {
                ids.insert(crate::ids::ChangeId::new(story, wid).short());
            }
            let run = super::run_of(&segment.piece);
            if let Some(record) = dom
                .element(run, &W::r_pr())
                .and_then(|rpr| dom.element(rpr, &W::r_pr_change()))
                && let Some(wid) = dom.attribute(record, &W::id())
            {
                let runs = recorded
                    .entry(crate::ids::ChangeId::new(story, wid).short())
                    .or_default();
                if !runs.contains(&(*para, run)) {
                    runs.push((*para, run));
                }
            }
        }
    }
    // An id two reached runs share, or a paragraph deletion reaches too,
    // keeps every word.
    let rewritten = recorded
        .into_iter()
        .filter(|(id, runs)| runs.len() == 1 && !deleted.contains(id))
        .map(|(id, runs)| {
            let (para, run) = runs[0];
            let rewrite = rewrite(dom, &tx.projections[para], run, &ranges[&para]);
            (id, rewrite)
        })
        .collect();
    Edited { ids, rewritten }
}

/// What `ranges` (projection bytes) take from `run`: its words outside
/// them, its `w:t` and `w:delText` text in document order as its
/// formatting change lists it, and how many characters and atoms inside.
fn rewrite(
    dom: &Dom,
    projection: &crate::inspect::Projection,
    run: NodeId,
    ranges: &[(usize, usize)],
) -> Rewrite {
    use crate::inspect::Piece;
    let inside = |at: usize| ranges.iter().any(|&(start, end)| start <= at && at < end);
    let mut out = Rewrite::default();
    for node in dom.descendants(run, None) {
        if !dom.name_is(node, &W::t()) && !dom.name_is(node, &W::del_text()) {
            continue;
        }
        let value = dom.value(node);
        let piece = Piece::Text { t: node, run };
        let Some(segment) = projection.segments.iter().find(|s| s.piece == piece) else {
            out.words.push_str(&value);
            continue;
        };
        for (i, c) in value.char_indices() {
            if inside(segment.start + i) {
                out.taken += 1;
            } else {
                out.words.push(c);
            }
        }
    }
    out.taken += projection
        .segments
        .iter()
        .filter(|s| inside(s.start))
        .filter(|s| match s.piece {
            Piece::Glyph {
                element,
                run: owner,
            } => {
                owner == run
                    && dom
                        .name(element)
                        .is_some_and(|name| crate::changes::atom(&name).is_some())
            }
            Piece::Text { .. } => false,
        })
        .count();
    out
}

/// Tracked edits retain another author's words, in order, either alive
/// or in the editor's nested deletions. Split pieces retain the source stamp.
fn preserve_history(
    working: &[u8],
    candidate: &[u8],
    edited: &Edited,
    author: &str,
    insertion_origins: &BTreeMap<String, String>,
) -> Result<(), EditError> {
    compare_history(working, candidate, edited, author, insertion_origins, true)
}

/// The direct artifact intentionally applies new text without revision marks.
/// Its resolved text is checked by check_dom_realized; this guard also keeps
/// unedited history and every retained source stamp/dependency intact.
fn preserve_direct_history(
    working: &[u8],
    candidate: &[u8],
    edited: &Edited,
    author: &str,
    insertion_origins: &BTreeMap<String, String>,
) -> Result<(), EditError> {
    compare_history(working, candidate, edited, author, insertion_origins, false)
}

fn compare_history(
    working: &[u8],
    candidate: &[u8],
    edited: &Edited,
    author: &str,
    insertion_origins: &BTreeMap<String, String>,
    tracked: bool,
) -> Result<(), EditError> {
    let rewrites = &edited.rewritten;
    let edited = &edited.ids;
    let failure = || {
        err(
            "REVISION_PRESERVATION_FAILED",
            None,
            "candidate changed revision history retained in W",
        )
    };
    // Each document's changes and its formatting changes' records (the old
    // properties and, a run's, the characters covered), from one parse.
    // The comment marks a revision holds are the comments' history, not
    // the revision's: a comment added or removed beside one moves them.
    let history = |bytes, skip: &dyn Fn(&Change) -> bool| {
        crate::changes::changes_and_records(bytes, skip)
            .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))
            .map(|(listed, records)| {
                let listed = listed
                    .into_iter()
                    .map(|change| Change {
                        holds_comments: Vec::new(),
                        ..change
                    })
                    .collect::<Vec<_>>();
                (listed, records)
            })
    };
    let (source, mut old_records) = history(working, &|_| false)?;
    let ids: BTreeSet<_> = source.iter().map(|c| c.id.clone()).collect();
    // The editor's own new revisions, in a tracked candidate.
    let editors = |c: &Change, kind: ChangeKind| {
        tracked && c.kind == kind && !ids.contains(&c.id) && c.author.as_deref() == Some(author)
    };
    // A tracked candidate's characters count where the source has them:
    // the editor's new insertions count for nothing.
    let (listed, mut new_records) = history(candidate, &|c| editors(c, ChangeKind::Insertion))?;
    // Accepting the editor's deletion takes what it holds along, as that
    // deletion's words go: what it holds depends on what the deletion sits in.
    let deletions: BTreeMap<String, Option<String>> = listed
        .iter()
        .filter(|c| editors(c, ChangeKind::Deletion))
        .map(|c| (c.id.clone(), c.inside.clone()))
        .collect();
    let dependency = |mut inside: Option<String>| {
        for _ in 0..=deletions.len() {
            match inside.as_ref().and_then(|id| deletions.get(id)) {
                Some(outer) => inside.clone_from(outer),
                None => break,
            }
        }
        inside
    };
    // Ids may repeat, so each formatting change takes its next record in
    // order.
    let next = |records: &mut HashMap<String, Vec<Recorded>>, c: &Change| {
        (c.kind == ChangeKind::Formatting)
            .then(|| {
                records
                    .get_mut(&c.id)
                    .filter(|r| !r.is_empty())
                    .map(|r| r.remove(0))
            })
            .flatten()
    };
    let mut before: Vec<(Change, Option<Recorded>)> = source
        .into_iter()
        .map(|c| {
            let record = next(&mut old_records, &c);
            (c, record)
        })
        .collect();
    // A run's formatting record, which an operation's words may leave.
    let run_record = |c: &Change, record: Option<&Recorded>| {
        c.kind == ChangeKind::Formatting && record.is_some_and(|r| r.characters.is_some())
    };
    let mut run_records = HashMap::<String, usize>::new();
    for (c, record) in &before {
        if run_record(c, record.as_ref()) {
            *run_records.entry(c.id.clone()).or_default() += 1;
        }
    }
    // A tracked candidate keeps every word, deleted ones in deletions. An
    // id two run records share names neither (`DUPLICATE_REVISION_ID`).
    let rewritten = |c: &Change, record: Option<&Recorded>| {
        (!tracked && run_record(c, record) && run_records.get(&c.id) == Some(&1))
            .then(|| rewrites.get(&c.id))
            .flatten()
    };
    let mut after: Vec<(Change, Option<Recorded>)> = Vec::new();
    let mut seen_insertions = BTreeSet::new();
    let mut insertion_text = BTreeMap::<String, String>::new();
    for mut change in listed {
        if let Some(origin) = change
            .inside
            .as_ref()
            .and_then(|id| insertion_origins.get(id))
        {
            change.inside = Some(origin.clone());
        }
        // Old nested revisions keep their own attribution. A new deletion
        // inside a retained insertion must belong to this editor.
        if tracked
            && !ids.contains(&change.id)
            && change.kind == ChangeKind::Deletion
            && change.inside.as_ref().is_some_and(|id| edited.contains(id))
            && change.author.as_deref() != Some(author)
        {
            return Err(failure());
        }
        change.inside = dependency(change.inside.take());
        if change.kind == ChangeKind::Insertion
            && change.target == "text"
            && let Some(origin) = insertion_origins
                .get(&change.id)
                .or_else(|| ids.get(&change.id))
        {
            let original = before
                .iter()
                .map(|(old, _)| old)
                .find(|old| old.id == *origin && old.kind == change.kind)
                .ok_or_else(failure)?;
            if change.id != *origin && !edited.contains(origin) {
                return Err(failure());
            }
            change.id.clone_from(origin);
            change.r#ref.clone_from(origin);
            // Every fragment retains the source stamp/dependency. Content
            // changes are authorized only on the resolved insertion ids.
            let mut metadata = change.clone();
            metadata.text.clone_from(&original.text);
            metadata.paragraph.clone_from(&original.paragraph);
            if metadata != *original {
                return Err(failure());
            }
            // Change::text includes surviving text and nested delText in
            // document order, including pre-existing nested deletions.
            insertion_text
                .entry(origin.clone())
                .or_default()
                .push_str(&change.text);
            if !seen_insertions.insert(origin.clone()) {
                continue;
            }
        }
        let record = next(&mut new_records, &change);
        // Another author's run formatting change under a fresh id is a
        // fragment of one the edit split: it joins the fragment before it
        // when both recorded the same author, date, paragraph, dependency and
        // old properties, and their characters touch (#1656). In a direct
        // candidate the new words of an operation that reached the record
        // may sit between them (#1658).
        if !ids.contains(&change.id)
            && change.kind == ChangeKind::Formatting
            && change.author.as_deref() != Some(author)
            && let Some((payload, Some((start, end)))) =
                record.as_ref().map(|r| (&r.payload, r.characters))
        {
            let Some((last, Some(joined))) = after
                .iter_mut()
                .rfind(|entry| entry.0.kind == ChangeKind::Formatting)
            else {
                return Err(failure());
            };
            let reached = rewritten(last, Some(joined)).is_some();
            let Some(covered) = joined.characters.as_mut() else {
                return Err(failure());
            };
            if last.author != change.author
                || last.date != change.date
                || last.paragraph != change.paragraph
                || last.inside != change.inside
                || joined.payload != *payload
                || (covered.1 != start && !reached)
            {
                return Err(failure());
            }
            last.text.push_str(&change.text);
            // How many characters the fragments cover; across the new
            // words of a direct edit, not their span.
            covered.1 += end - start;
            continue;
        }
        if ids.contains(&change.id) {
            after.push((change, record));
        }
    }
    if tracked {
        for (original, _) in &before {
            if original.kind == ChangeKind::Insertion
                && original.target == "text"
                && edited.contains(&original.id)
                && original.author.as_deref() != Some(author)
                && insertion_text.get(&original.id) != Some(&original.text)
            {
                return Err(failure());
            }
        }
    }
    let kept = |c: &Change, record: Option<&Recorded>| {
        after
            .iter()
            .find(|(new, r)| {
                new.id == c.id
                    && new.kind == c.kind
                    && run_record(new, r.as_ref()) == run_record(c, record)
            })
            .map(|(new, _)| new)
    };
    before.retain_mut(|(c, record)| {
        if let Some(rewrite) = rewritten(c, record.as_ref()) {
            // The record keeps the words and atoms the operations left it,
            // and goes with the last of them.
            let covered = record.as_mut().and_then(|r| r.characters.as_mut());
            let left = covered.map(|(start, end)| {
                *end = *start + (*end - *start).saturating_sub(rewrite.taken);
                *end - *start
            });
            let remains = !rewrite.words.is_empty() || left.unwrap_or(0) > 0;
            if remains && kept(c, record.as_ref()).is_some() {
                c.text.clone_from(&rewrite.words);
            }
            return remains;
        }
        if !edited.contains(&c.id) {
            return true;
        }
        if c.kind == ChangeKind::Insertion && c.target == "text" {
            if let Some(new) = kept(c, record.as_ref()) {
                c.text.clone_from(&new.text);
            } else if c.author.as_deref() == Some(author) || !tracked {
                // Deleting one's entire insertion removes its carrier; so
                // does a direct edit that takes every word of another
                // author's (`check_dom_realized` holds the text).
                return false;
            }
        } else if !tracked && kept(c, record.as_ref()).is_none() {
            // A direct paragraph deletion takes the paragraph's other
            // revisions with it; a tracked one keeps them (#1136).
            return false;
        }
        true
    });
    // A tracked candidate keeps each record over the characters it covered;
    // a direct one moves them with the edit, so only how many compare.
    for (c, record) in before.iter_mut().chain(after.iter_mut()) {
        c.paragraph = None;
        if !tracked && let Some((start, end)) = record.as_mut().and_then(|r| r.characters.as_mut())
        {
            (*start, *end) = (0, *end - *start);
        }
    }
    if before != after {
        return Err(failure());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::deeper_boundary_fixture::{docx, para};
    use crate::edit::for_edit::{ForEditRequest, ForEditRow, for_edit};

    impl From<BTreeSet<String>> for Edited {
        fn from(ids: BTreeSet<String>) -> Self {
            Self {
                ids,
                ..Self::default()
            }
        }
    }

    #[test]
    fn history_guard_rejects_words_dropped_without_a_nested_deletion() {
        let package = |text: &str| {
            docx(&format!(
                r#"<w:p><w:ins w:id="6" w:author="Alice"><w:r><w:t>{text}</w:t></w:r></w:ins></w:p>"#
            ))
        };
        let result = preserve_history(
            &package("old"),
            &package("new"),
            &Edited::from(BTreeSet::from(["r6".into()])),
            "Carol Diaz",
            &BTreeMap::from([("r6".into(), "r6".into())]),
        );
        assert_eq!(
            result.err().map(|error| error.code),
            Some("REVISION_PRESERVATION_FAILED".into()),
            "another author's words must survive inside an editor deletion"
        );
    }

    #[test]
    fn history_guard_ignores_a_comment_mark_inside_a_kept_revision() {
        // A comment added or removed beside another author's insertion moves
        // the mark the change holds; the revision's history is the same.
        let package = |mark: &str| {
            docx(&format!(
                r#"<w:p><w:ins w:id="6" w:author="Alice"><w:r><w:t>kept</w:t></w:r>{mark}</w:ins></w:p>"#
            ))
        };
        let mark = r#"<w:r><w:commentReference w:id="0"/></w:r>"#;
        for (working, candidate) in [(package(""), package(mark)), (package(mark), package(""))] {
            for guard in [preserve_history, preserve_direct_history] {
                let result = guard(
                    &working,
                    &candidate,
                    &Edited::default(),
                    "Carol Diaz",
                    &BTreeMap::new(),
                );
                assert!(result.is_ok(), "{result:?}");
            }
        }
    }

    #[test]
    fn history_guard_checks_all_fragments_and_the_nested_deletion_author() {
        let insertion = |id: u32, content: &str| {
            format!(r#"<w:ins w:id="{id}" w:author="Alice">{content}</w:ins>"#)
        };
        let run = |text: &str| format!("<w:r><w:t>{text}</w:t></w:r>");
        let deletion = |author: &str, text: &str| {
            format!(
                r#"<w:del w:id="7" w:author="{author}"><w:r><w:delText>{text}</w:delText></w:r></w:del>"#
            )
        };
        let source = docx(&format!(
            "<w:p>{}</w:p>",
            insertion(6, &run("alpha beta gamma"))
        ));
        let head = format!("{}{}", run("alpha "), deletion("Carol Diaz", "beta"));
        let editor = r#"<w:ins w:id="8" w:author="Carol Diaz"><w:r><w:t>new</w:t></w:r></w:ins>"#;
        let origins = BTreeMap::from([("r6".into(), "r6".into()), ("r9".into(), "r6".into())]);
        for (content, valid) in [
            (
                format!(
                    "{}{editor}{}",
                    insertion(6, &head),
                    insertion(9, &run(" gamma"))
                ),
                true,
            ),
            (format!("{}{editor}", insertion(6, &head)), false),
            (
                format!("{}{}", insertion(9, &run(" gamma")), insertion(6, &head)),
                false,
            ),
            (
                insertion(
                    6,
                    &format!(
                        "{}{}{}",
                        run("changed "),
                        deletion("Carol Diaz", "beta"),
                        run(" gamma")
                    ),
                ),
                false,
            ),
            (
                insertion(
                    6,
                    &format!(
                        "{}{}{}",
                        run("alpha "),
                        deletion("Bob", "beta"),
                        run(" gamma")
                    ),
                ),
                false,
            ),
            (
                format!(
                    "{}{}",
                    insertion(6, &format!("{}{}", run("alpha "), run(" gamma"))),
                    deletion("Carol Diaz", "beta")
                ),
                false,
            ),
        ] {
            let result = preserve_history(
                &source,
                &docx(&format!("<w:p>{content}</w:p>")),
                &Edited::from(BTreeSet::from(["r6".into()])),
                "Carol Diaz",
                &origins,
            );
            assert_eq!(result.is_ok(), valid, "{content}: {result:?}");
            if let Err(error) = result {
                assert_eq!(error.code, "REVISION_PRESERVATION_FAILED");
            }
        }
        // The guard also protects an insertion that was not split.
        let nested = insertion(
            6,
            &format!(
                "{}{}{}",
                run("alpha "),
                deletion("Carol Diaz", "beta"),
                run(" gamma")
            ),
        );
        assert!(
            preserve_history(
                &source,
                &docx(&format!("<w:p>{nested}</w:p>")),
                &Edited::from(BTreeSet::from(["r6".into()])),
                "Carol Diaz",
                &BTreeMap::new()
            )
            .is_ok()
        );
    }

    #[test]
    fn history_guard_joins_the_fragments_of_a_split_formatting_record() {
        // Alice made a run bold (italic before); Carol's edit split it. Each
        // fragment carries Alice's record under its own id (#1656).
        let formatted = |id: u32, author: &str, old: &str, text: &str| {
            format!(
                r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="{author}" w:date="2026-10-17T09:00:00Z"><w:rPr>{old}</w:rPr></w:rPrChange></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>"#
            )
        };
        let alice = |id: u32, text: &str| formatted(id, "Alice", "<w:i/>", text);
        let plain = |text: &str| format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#);
        let carol = r#"<w:ins w:id="20" w:author="Carol Diaz" w:date="2026-10-23T10:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins>"#;
        let source = docx(&format!("<w:p>{}</w:p>", alice(5, "alpha beta gamma")));
        for (content, valid) in [
            (
                format!("{}{}", alice(5, "alpha "), alice(9, "beta gamma")),
                true,
            ),
            (
                format!("{}{carol}{}", alice(5, "alpha "), alice(9, "beta gamma")),
                true,
            ),
            (
                format!(
                    "{}{}{}",
                    alice(5, "alpha "),
                    alice(9, "beta"),
                    alice(10, " gamma")
                ),
                true,
            ),
            // The fragment recorded other old properties.
            (
                format!(
                    "{}{}",
                    alice(5, "alpha "),
                    formatted(9, "Alice", r#"<w:u w:val="single"/>"#, "beta gamma")
                ),
                false,
            ),
            // The original record's own old properties changed.
            (
                formatted(5, "Alice", r#"<w:u w:val="single"/>"#, "alpha beta gamma"),
                false,
            ),
            // Another author's record.
            (
                format!(
                    "{}{}",
                    alice(5, "alpha "),
                    formatted(9, "Dan", "<w:i/>", "beta gamma")
                ),
                false,
            ),
            // A fragment lost its record.
            (
                format!(
                    "{}{}{}",
                    alice(5, "alpha "),
                    plain("beta"),
                    alice(9, " gamma")
                ),
                false,
            ),
            // The record spread over text it never covered.
            (
                format!("{}{}", alice(5, "alpha beta gamma"), alice(9, " more")),
                false,
            ),
        ] {
            let result = preserve_history(
                &source,
                &docx(&format!("<w:p>{content}</w:p>")),
                &Edited::default(),
                "Carol Diaz",
                &BTreeMap::new(),
            );
            assert_eq!(result.is_ok(), valid, "{content}: {result:?}");
            if let Err(error) = result {
                assert_eq!(error.code, "REVISION_PRESERVATION_FAILED");
            }
        }
    }

    /// Alice's run formatting change `id` over `text`: bold now, italic
    /// before. `deleted` spells the words as `w:delText`.
    fn alice_formatted(id: u32, text: &str, deleted: bool) -> String {
        let t = if deleted { "delText" } else { "t" };
        format!(
            r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:{t} xml:space="preserve">{text}</w:{t}></w:r>"#
        )
    }

    fn plain_run(text: &str) -> String {
        format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
    }

    /// Carol's new revision `kind` (`ins` or `del`) around `content`.
    fn carols(kind: &str, id: u32, content: &str) -> String {
        format!(
            r#"<w:{kind} w:id="{id}" w:author="Carol Diaz" w:date="2026-10-23T10:00:00Z">{content}</w:{kind}>"#
        )
    }

    fn paragraph(content: &str) -> Vec<u8> {
        docx(&format!("<w:p>{content}</w:p>"))
    }

    #[test]
    fn history_guard_joins_fragments_only_where_they_touch_in_the_source() {
        // `abXb`: Alice's record moves from the first `ab` onto `a` and the
        // last `b`, around unrelated words (#1656).
        let source = paragraph(&[alice_formatted(5, "ab", false), plain_run("Xb")].concat());
        let moved = paragraph(
            &[
                alice_formatted(5, "a", false),
                plain_run("bX"),
                alice_formatted(9, "b", false),
            ]
            .concat(),
        );
        // A record of Alice's under a fresh id that touches none of hers is
        // new history, not a fragment.
        let spread = paragraph(
            &[
                alice_formatted(5, "ab", false),
                plain_run("X"),
                alice_formatted(9, "b", false),
            ]
            .concat(),
        );
        for candidate in [moved, spread] {
            for guard in [preserve_history, preserve_direct_history] {
                let result = guard(
                    &source,
                    &candidate,
                    &Edited::default(),
                    "Carol Diaz",
                    &BTreeMap::new(),
                );
                assert_eq!(
                    result.map_err(|e| e.code).unwrap_err(),
                    "REVISION_PRESERVATION_FAILED"
                );
            }
        }
    }

    #[test]
    fn history_guard_keeps_a_record_out_of_the_editors_new_insertion() {
        // Rejecting Carol's insertion would take part of Alice's history.
        let source = paragraph(&alice_formatted(5, "ab", false));
        let candidate = paragraph(
            &[
                alice_formatted(5, "a", false),
                carols("ins", 20, &alice_formatted(9, "b", false)),
            ]
            .concat(),
        );
        let result = preserve_history(
            &source,
            &candidate,
            &Edited::default(),
            "Carol Diaz",
            &BTreeMap::new(),
        );
        assert_eq!(
            result.map_err(|e| e.code).unwrap_err(),
            "REVISION_PRESERVATION_FAILED"
        );
    }

    #[test]
    fn history_guard_keeps_a_record_whose_words_the_editor_deleted() {
        // Carol's tracked deletion holds a fragment of Alice's record: the
        // record covers the same words, deleted ones included.
        let source = paragraph(&alice_formatted(5, "alpha beta gamma", false));
        for content in [
            [
                carols("del", 20, &alice_formatted(5, "alpha ", true)),
                alice_formatted(9, "beta gamma", false),
            ]
            .concat(),
            [
                alice_formatted(5, "alpha ", false),
                carols("del", 20, &alice_formatted(9, "beta", true)),
                alice_formatted(10, " gamma", false),
            ]
            .concat(),
        ] {
            let result = preserve_history(
                &source,
                &paragraph(&content),
                &Edited::default(),
                "Carol Diaz",
                &BTreeMap::new(),
            );
            assert!(result.is_ok(), "{content}: {result:?}");
        }
    }

    #[test]
    fn history_guard_tells_an_insertion_from_a_formatting_change_with_its_id() {
        // An insertion and a run formatting change may share `w:id` 5; Carol
        // edits only the insertion.
        let insertion = |content: &str| {
            format!(
                r#"<w:ins w:id="5" w:author="Alice" w:date="2026-10-17T09:00:00Z">{content}</w:ins>"#
            )
        };
        let source = paragraph(
            &[
                alice_formatted(5, "foo", false),
                insertion(&plain_run("old")),
            ]
            .concat(),
        );
        let edited = Edited::from(BTreeSet::from(["r5".to_string()]));
        let direct = paragraph(
            &[
                alice_formatted(5, "foo", false),
                insertion(&plain_run("new")),
            ]
            .concat(),
        );
        let result =
            preserve_direct_history(&source, &direct, &edited, "Carol Diaz", &BTreeMap::new());
        assert!(result.is_ok(), "{result:?}");
        let tracked = paragraph(
            &[
                alice_formatted(5, "foo", false),
                insertion(&carols(
                    "del",
                    20,
                    r#"<w:r><w:delText>old</w:delText></w:r>"#,
                )),
                carols("ins", 21, &plain_run("new")),
            ]
            .concat(),
        );
        let result = preserve_history(
            &source,
            &tracked,
            &edited,
            "Carol Diaz",
            &BTreeMap::from([("r5".into(), "r5".into())]),
        );
        assert!(result.is_ok(), "{result:?}");
        // The formatting change itself was not edited: its words stay.
        let moved = paragraph(
            &[
                plain_run("foo"),
                insertion(&plain_run("new")),
                alice_formatted(5, "", false),
            ]
            .concat(),
        );
        let result =
            preserve_direct_history(&source, &moved, &edited, "Carol Diaz", &BTreeMap::new());
        assert_eq!(
            result.map_err(|e| e.code).unwrap_err(),
            "REVISION_PRESERVATION_FAILED"
        );
    }

    #[test]
    fn history_guard_reads_each_record_where_ids_repeat() {
        // Two of Alice's records share `w:id` 5; the candidate rewrites the
        // first one's old properties.
        let record = |old: &str, text: &str| {
            format!(
                r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:rPr>{old}</w:rPr></w:rPrChange></w:rPr><w:t>{text}</w:t></w:r>"#
            )
        };
        let source = paragraph(&[record("<w:i/>", "one"), record("<w:u/>", "two")].concat());
        let candidate = paragraph(&[record("<w:caps/>", "one"), record("<w:u/>", "two")].concat());
        for guard in [preserve_history, preserve_direct_history] {
            assert!(
                guard(
                    &source,
                    &source,
                    &Edited::default(),
                    "Carol Diaz",
                    &BTreeMap::new()
                )
                .is_ok()
            );
            let result = guard(
                &source,
                &candidate,
                &Edited::default(),
                "Carol Diaz",
                &BTreeMap::new(),
            );
            assert_eq!(
                result.map_err(|e| e.code).unwrap_err(),
                "REVISION_PRESERVATION_FAILED"
            );
        }
    }

    #[test]
    fn history_guard_keeps_a_record_off_a_neighbouring_drawing() {
        // Alice's record covers a tab; the candidate spreads it over the
        // drawing beside it, under a fresh id.
        let record = |id: u32| {
            format!(
                r#"<w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr>"#
            )
        };
        let tab = format!("<w:r>{}<w:tab/></w:r>", record(5));
        let source = paragraph(&format!("{tab}<w:r><w:drawing/></w:r>"));
        let candidate = paragraph(&format!("{tab}<w:r>{}<w:drawing/></w:r>", record(9)));
        for guard in [preserve_history, preserve_direct_history] {
            let result = guard(
                &source,
                &candidate,
                &Edited::default(),
                "Carol Diaz",
                &BTreeMap::new(),
            );
            assert_eq!(
                result.map_err(|e| e.code).unwrap_err(),
                "REVISION_PRESERVATION_FAILED"
            );
        }
    }

    #[test]
    fn history_guard_keeps_a_paragraph_records_old_properties() {
        let aligned = |jc: &str| {
            docx(&format!(
                r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="6" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:pPr><w:jc w:val="{jc}"/></w:pPr></w:pPrChange></w:pPr><w:r><w:t>alpha</w:t></w:r></w:p>"#
            ))
        };
        for guard in [preserve_history, preserve_direct_history] {
            let result = guard(
                &aligned("left"),
                &aligned("right"),
                &Edited::default(),
                "Carol Diaz",
                &BTreeMap::new(),
            );
            assert_eq!(
                result.map_err(|e| e.code).unwrap_err(),
                "REVISION_PRESERVATION_FAILED"
            );
        }
    }

    #[test]
    fn history_guard_keeps_unedited_content_and_all_source_attribution() {
        let package = |first: &str, second: &str, author: &str| {
            docx(&format!(
                r#"<w:p><w:ins w:id="6" w:author="{author}" w:date="2026-10-17T09:00:00Z"><w:r><w:t>{first}</w:t></w:r></w:ins><w:ins w:id="9" w:author="Bob" w:date="2026-10-17T09:00:00Z"><w:r><w:t>{second}</w:t></w:r></w:ins></w:p>"#
            ))
        };
        let source = package("old", "unrelated", "Alice");
        let edited = Edited::from(BTreeSet::from(["r6".into()]));
        for candidate in [
            package("new", "changed", "Alice"),
            package("new", "unrelated", "Bob"),
        ] {
            assert_eq!(
                preserve_history(
                    &source,
                    &candidate,
                    &edited,
                    "Carol Diaz",
                    &BTreeMap::from([("r6".into(), "r6".into())])
                )
                .unwrap_err()
                .code,
                "REVISION_PRESERVATION_FAILED"
            );
        }
        assert!(
            preserve_history(
                &source,
                &package("new", "unrelated", "Alice"),
                &edited,
                "Alice",
                &BTreeMap::from([("r6".into(), "r6".into())])
            )
            .is_ok()
        );
    }

    /// An effectful v2 apply builds its expectations correspondence without
    /// reopening the emitted primary: the final-view dispositions come from
    /// the DOM that produced the bytes, so no apply pays a second admission,
    /// unzip and parse of its own output for a correspondence only `--expect`
    /// reads (#1087).
    #[test]
    fn effectful_apply_never_reopens_its_primary_for_the_correspondence() {
        let source = docx(&format!(
            "{}{}{}",
            para("Alpha."),
            para("Beta."),
            para("Gamma.")
        ));
        let hash = crate::inspect::source_sha256(&source);
        for track in ["on", "off"] {
            let plan = PlanV2::from_json(&format!(
                r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{hash}"}},
  "policy": {{"track": "{track}", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "T",
  "date": "2026-10-18T00:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": [
    {{"kind": "delete_paragraph", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}}}
  ]
}}"#
            ))
            .unwrap();
            let applied = apply_plan(&source, &plan).unwrap();
            assert_ne!(
                applied.redline, source,
                "track {track}: the delete must take effect for the counter window"
            );
            assert_eq!(
                crate::inspect::REOPENED_PACKAGES.with(|count| count.get()),
                0,
                "track {track}: the correspondence reopened the emitted primary"
            );
        }
    }

    /// The regression net behind #1087: the final-view states `apply_plan`
    /// hands the correspondence, read off the DOM that produced the
    /// primary, must equal a reparse of the emitted primary itself. That
    /// holds because `finish` changes the producing DOM only through
    /// `apply_watermark`, which no v2 operation reaches; a future
    /// DOM-mutating step in `finish` would silently misalign the
    /// correspondence's output indices, and this test must catch it first.
    ///
    /// The fixtures together carry headers and footers (Strict01, f4),
    /// footnotes and endnotes (Strict01, f4), existing comments (all),
    /// text boxes (Strict01, hard-tables), existing tracked changes (all)
    /// with deleted paragraph marks (Strict01, hard-tables), content
    /// controls (Strict01) and tables (all), nested ones included
    /// (hard-tables). Every plan kind runs on every fixture under both
    /// tracking policies, so the tracked replay's scan and the direct
    /// DOM walk are each pinned. The reparse runs here, after `apply_plan`
    /// returns — never inside the counted phase of
    /// `effectful_apply_never_reopens_its_primary_for_the_correspondence`.
    #[test]
    fn final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary() {
        // One (fixture, kind → target row) net per shape family; rows are
        // the paired ids `read --for-edit` prints for the frozen fixture.
        const STRICT01: &[u8] = include_bytes!("../../tests/fixtures/strict/Strict01.docx");
        const HARD_TABLES: &[u8] =
            include_bytes!("../../tests/fixtures/from-docx/critic/synthetic/hard-tables.docx");
        const F4_ORIGINAL: &[u8] = include_bytes!("../../tests/fixtures/f4/original.docx");
        type Net = (
            &'static str,
            &'static [u8],
            &'static [(&'static str, &'static str)],
        );
        const NET: &[Net] = &[
            (
                "strict01",
                STRICT01,
                &[
                    ("delete_paragraph", "p2"),
                    ("replace", "p31"),                // a table cell's paragraph
                    ("insert_paragraph", "footnotes"), // a footnote story
                    ("rewrite", "p50"),                // a table cell's paragraph
                    ("comment", "p47"),                // a paragraph holding a text box
                ],
            ),
            (
                "hard-tables",
                HARD_TABLES,
                &[
                    ("delete_paragraph", "p0"),
                    ("replace", "p0"),
                    ("insert_paragraph", "p0"),
                    ("rewrite", "p2"),  // a nested table cell's paragraph
                    ("comment", "p14"), // a paragraph holding a text box
                ],
            ),
            (
                "f4-original",
                F4_ORIGINAL,
                &[
                    ("delete_paragraph", "p0"),
                    ("replace", "p0"),
                    ("insert_paragraph", "header1"), // a header story
                    ("rewrite", "p0"),
                    ("comment", "p10"), // a table cell's paragraph
                ],
            ),
        ];
        for (name, source, kinds) in NET {
            let view = for_edit(
                source,
                &ForEditRequest {
                    all: true,
                    ..Default::default()
                },
            )
            .unwrap();
            for (kind, row_id) in *kinds {
                let row = view
                    .paragraphs
                    .iter()
                    .find(|row| &row.target.short_id == row_id)
                    .unwrap_or_else(|| panic!("{name}: no for-edit row {row_id}"));
                for track in ["on", "off"] {
                    let operation = net_operation(kind, row);
                    let plan = PlanV2::from_json(&net_plan(source, track, &operation)).unwrap();
                    let _ = correspondence_states(); // clear any earlier capture
                    let applied = apply_plan(source, &plan)
                        .unwrap_or_else(|e| panic!("{name}/{kind} track {track}: {e}"));
                    // The redline is the primary under tracking, the clean
                    // copy otherwise, exactly how `apply_plan` picks it.
                    let primary = if track == "on" {
                        &applied.redline
                    } else {
                        &applied.clean
                    };
                    let states = correspondence_states();
                    let reparsed = reparsed_final_views(primary);
                    assert!(
                        !states.is_empty(),
                        "{name}/{kind} track {track}: no states captured"
                    );
                    assert_eq!(
                        states, reparsed,
                        "{name}/{kind} track {track}: the DOM-read states diverge from a reparse"
                    );
                }
            }
        }
    }

    /// One single-operation PlanV2 for the equivalence net.
    fn net_plan(source: &[u8], track: &str, operation: &serde_json::Value) -> String {
        serde_json::json!({
            "schema_version": 2,
            "source": {"sha256": source_sha256(source)},
            "policy": {"track": track, "existing_revisions": "keep", "text_view": "final"},
            "resolve_revisions": {"accept": [], "reject": []},
            "author": "T",
            "date": "2026-10-18T00:00:00Z",
            "scope": {"kind": "all"},
            "operations": [operation],
        })
        .to_string()
    }

    /// One operation of the equivalence net, addressed at `row`'s paired ids.
    fn net_operation(kind: &str, row: &ForEditRow) -> serde_json::Value {
        let target = serde_json::json!({
            "short_id": row.target.short_id,
            "long_id": row.target.long_id,
        });
        match kind {
            "delete_paragraph" => serde_json::json!({
                "kind": "delete_paragraph",
                "target": target,
            }),
            "replace" => {
                // The row's first word: a literal anchor a replace admits.
                let find = row
                    .working
                    .as_ref()
                    .and_then(|working| working.value.text.as_deref())
                    .and_then(|text| text.split_whitespace().next())
                    .unwrap_or_else(|| panic!("{} has no literal to replace", row.target.short_id));
                serde_json::json!({
                    "kind": "replace",
                    "target": target,
                    "find": find,
                    "text": "X",
                })
            }
            "insert_paragraph" => serde_json::json!({
                "kind": "insert_paragraph",
                "target": target,
                "position": "after",
                "text": "New paragraph.",
            }),
            "rewrite" => {
                let before = row
                    .working
                    .as_ref()
                    .and_then(|working| working.before_sha256.as_deref())
                    .unwrap_or_else(|| panic!("{} has no working hash", row.target.short_id));
                serde_json::json!({
                    "kind": "rewrite",
                    "target": target,
                    "before_sha256": before,
                    "text": "Rewritten.",
                })
            }
            "comment" => serde_json::json!({
                "kind": "comment",
                "target": target,
                "text": "A note.",
            }),
            other => panic!("unknown plan kind {other}"),
        }
    }

    /// A style's formatting record and a comment body's share `w:id` 3 but
    /// not a story: removing the comment leaves the style's record its own
    /// (Jules, #1661, one parse for listing and records).
    #[test]
    fn a_comment_bodys_formatting_record_stands_in_for_no_listed_change() {
        use crate::edit::deeper_boundary_fixture::{Part, docx_with};
        let styles = format!(
            r#"<w:styles xmlns:w="{}"><w:style w:type="character" w:styleId="S"><w:name w:val="S"/><w:rPr><w:b/><w:rPrChange w:id="3" w:author="Ann" w:date="2026-09-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:style></w:styles>"#,
            W::URI
        );
        let comments = format!(
            r#"<w:comments xmlns:w="{}"><w:comment w:id="0" w:author="Cy"><w:p><w:r><w:rPr><w:i/><w:rPrChange w:id="3" w:author="Cy" w:date="2026-09-01T00:00:00Z"><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#,
            W::URI
        );
        let styles_part = Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        };
        let comments_part = Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        };
        let source = docx_with(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Text</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
            &[styles_part, comments_part],
        );
        let styles_part = Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        };
        let candidate = docx_with(r#"<w:p><w:r><w:t>Text</w:t></w:r></w:p>"#, &[styles_part]);
        for tracked in [false, true] {
            let result = compare_history(
                &source,
                &candidate,
                &Edited::default(),
                "Bob",
                &BTreeMap::new(),
                tracked,
            );
            assert!(result.is_ok(), "tracked {tracked}: {result:?}");
        }
    }

    fn many_comment_source() -> Vec<u8> {
        use crate::edit::deeper_boundary_fixture::{Part, docx_with};
        let mut starts = String::new();
        let mut ends = String::new();
        let mut notes = String::new();
        for id in 0..8 {
            starts.push_str(&format!(r#"<w:commentRangeStart w:id="{id}"/>"#));
            ends.push_str(&format!(
                r#"<w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
            ));
            notes.push_str(&format!(r#"<w:comment w:id="{id}" w:author="Other"><w:p><w:r><w:t>Note {id}</w:t></w:r></w:p></w:comment>"#));
        }
        let comments = format!(r#"<w:comments xmlns:w="{}">{notes}</w:comments>"#, W::URI);
        docx_with(
            &format!(
                "<w:p>{starts}<w:r><w:t>First</w:t></w:r></w:p>{}<w:p><w:r><w:t>Last</w:t></w:r>{ends}</w:p>",
                para("Middle")
            ),
            &[Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            }],
        )
    }

    #[test]
    fn a_range_first_side_that_is_not_a_block_refuses_usage_not_a_panic() {
        // The parser guarantees a range's first side is a block today; a
        // future grammar that breaks that invariant must refuse the
        // selector, never panic on user input.
        let source = docx(&para("Alpha"));
        let model = SourceModel::new(&source).unwrap();
        let mut out = std::collections::HashSet::new();
        let parsed = ShortId::Range {
            first: Box::new(ShortId::Comment(5)),
            last: 3,
        };
        let error = model.select_parsed(parsed, &mut out).unwrap_err();
        assert_eq!(error.code, "USAGE");
        assert!(out.is_empty());
    }

    #[test]
    fn thread_only_and_unchanged_plans_do_not_scan_comment_anchors() {
        let source = many_comment_source();
        let plan = EditPlan::from_json(
            r#"{"schema_version":1,"author":"Me","date":"2026-10-17T00:00:00Z","operations":[]}"#,
        )
        .unwrap();
        let mut tx = Transaction::start(&source, &plan).unwrap();
        for thread_only in [false, true] {
            if thread_only {
                tx.resolved.push((
                    0,
                    Resolved::Thread {
                        op: super::super::ThreadOp::Resolve { id: 0, done: true },
                        stories: vec![0],
                        anchor: None,
                    },
                ));
            }
            super::super::COMMENT_MARK_SCANS.with(|scans| scans.set(0));
            check_comment_anchors(&tx, true).unwrap();
            assert_eq!(
                super::super::COMMENT_MARK_SCANS.with(|scans| scans.get()),
                0
            );
        }
    }

    #[test]
    fn existing_comment_anchors_scan_each_paragraph_once_per_apply() {
        let source = many_comment_source();
        let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","date":"2026-10-17T00:00:00Z","operations":[{"kind":"replace","paragraph":"p1","find":"Middle","replacement":"Changed"}]}"#).unwrap();
        let mut tx = Transaction::start(&source, &plan).unwrap();
        let (resolved, outcome) = tx
            .resolve_one("/operations/0", &plan.operations[0].kind)
            .unwrap();
        tx.resolved.push((0, resolved));
        tx.outcomes.push(outcome);
        super::super::COMMENT_MARK_SCANS.with(|scans| scans.set(0));
        check_comment_anchors(&tx, true).unwrap();
        assert_eq!(
            super::super::COMMENT_MARK_SCANS.with(|scans| scans.get()),
            3
        );
    }

    #[test]
    fn v2_comment_diagnosis_ignores_engine_message_wording() {
        let source = docx(&para("Alpha Beta"));
        let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","date":"2026-10-17T00:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"Alpha","replacement":"A"},{"kind":"comment","paragraph":"p0","find":"pha Be","text":"Boundary"}]}"#).unwrap();
        let mut tx = Transaction::start(&source, &plan).unwrap();
        for (i, op) in plan.operations.iter().enumerate() {
            let (resolved, outcome) = tx.resolve_one(&pointer(i), &op.kind).unwrap();
            tx.resolved.push((i, resolved));
            tx.outcomes.push(outcome);
        }
        let legacy = tx.check_conflicts().unwrap_err();
        assert_eq!(legacy.code, "OVERLAPPING_EDITS");
        let mut engine = tx.check_conflicts_v2().unwrap_err();
        assert_eq!(engine.code, super::super::COMMENT_ANCHOR_CONFLICT);
        engine.message = "Reworded engine detail".into();
        let error = diagnose_conflict(&tx, engine, true);
        assert_eq!(error.code, "COMMENT_ANCHOR_UNMAPPABLE");
        assert_eq!(error.operation.as_deref(), Some("/operations/1"));
        assert!(error.message.contains("boundary"));
    }

    #[test]
    fn comment_inventory_preserves_first_complete_ranges_and_offsets() {
        let source = docx(
            r#"<w:p><w:commentRangeEnd w:id="0"/><w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="0"/><w:r><w:t>é</w:t><w:tab/></w:r><w:commentRangeStart w:id="1"/><w:commentRangeStart/></w:p><w:p><w:r><w:t>Beta</w:t></w:r><w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:commentRangeStart w:id="2"/><w:commentRangeEnd w:id="3"/></w:p><w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Later</w:t></w:r><w:commentRangeEnd w:id="0"/></w:p>"#,
        );
        let plan = EditPlan::from_json(
            r#"{"schema_version":1,"author":"Me","date":"2026-10-17T00:00:00Z","operations":[]}"#,
        )
        .unwrap();
        let tx = Transaction::start(&source, &plan).unwrap();
        let anchors = tx.comment_anchors();
        assert_eq!(anchors.len(), 2);
        for (id, start) in [(0, 0), (1, 3)] {
            let range = anchors[&id.to_string()];
            assert_eq!(
                (range.first, range.start, range.last, range.end),
                (0, start, 1, 4)
            );
            let legacy = tx.comment_text(id).unwrap();
            assert_eq!(
                (range.first, range.start, range.last, range.end),
                (legacy.first, legacy.start, legacy.last, legacy.end)
            );
        }
        assert!(tx.comment_text(2).is_err());
        assert!(tx.comment_text(3).is_err());
    }

    /// The reach of operations that take `taken` characters from record
    /// `id`'s run and leave it `words`.
    fn rewriting(id: &str, words: &str, taken: usize) -> Edited {
        let rewrite = Rewrite {
            words: words.to_string(),
            taken,
        };
        Edited {
            rewritten: BTreeMap::from([(id.to_string(), rewrite)]),
            ..Edited::default()
        }
    }

    fn refused(result: Result<(), EditError>) {
        assert_eq!(
            result.map_err(|e| e.code).unwrap_err(),
            "REVISION_PRESERVATION_FAILED"
        );
    }

    #[test]
    fn history_guard_lets_a_direct_edit_take_words_out_of_a_record_it_reached() {
        // Carol's direct replace of `beta` inside Alice's run r5: her new
        // word sits in a run of its own between the two fragments (#1658).
        let source = paragraph(&alice_formatted(5, "alpha beta gamma", false));
        let edited = rewriting("r5", "alpha  gamma", 4);
        let fragments = |middle: &str, last: &str| {
            paragraph(
                &[
                    alice_formatted(5, "alpha ", false),
                    middle.to_string(),
                    last.to_string(),
                ]
                .concat(),
            )
        };
        let direct = fragments(&plain_run("new"), &alice_formatted(9, " gamma", false));
        let guard = |candidate: &[u8], edited: &Edited| {
            preserve_direct_history(&source, candidate, edited, "Carol Diaz", &BTreeMap::new())
        };
        let result = guard(&direct, &edited);
        assert!(result.is_ok(), "{result:?}");
        // Not reached: the fragments must touch.
        refused(guard(&direct, &Edited::default()));
        // A tracked candidate keeps every word.
        refused(preserve_history(
            &source,
            &direct,
            &edited,
            "Carol Diaz",
            &BTreeMap::new(),
        ));
        // The new word joined the record, or a copy of it.
        refused(guard(
            &paragraph(&alice_formatted(5, "alpha new gamma", false)),
            &edited,
        ));
        refused(guard(
            &fragments("", &alice_formatted(9, "new gamma", false)),
            &edited,
        ));
        // The record lost a word the operation did not take.
        refused(guard(
            &fragments(&plain_run("new"), &alice_formatted(9, " gam", false)),
            &edited,
        ));
        // Its old properties changed.
        let underlined =
            alice_formatted(9, " gamma", false).replace("<w:i/>", r#"<w:u w:val="single"/>"#);
        refused(guard(&fragments(&plain_run("new"), &underlined), &edited));
        // It vanished while it still had words.
        refused(guard(&paragraph(&plain_run("alpha new gamma")), &edited));
    }

    #[test]
    fn history_guard_lets_a_record_go_with_its_last_word() {
        let source = paragraph(&alice_formatted(5, "beta", false));
        let edited = rewriting("r5", "", 4);
        let guard = |candidate: &[u8]| {
            preserve_direct_history(&source, candidate, &edited, "Carol Diaz", &BTreeMap::new())
        };
        let result = guard(&paragraph(&plain_run("new")));
        assert!(result.is_ok(), "{result:?}");
        refused(guard(&paragraph(&alice_formatted(5, "new", false))));
    }

    #[test]
    fn history_guard_rewrites_only_a_formatting_changes_words() {
        // Alice's insertion shares `w:id` 5 with the record Carol's
        // operation reached; the insertion's words are not the record's.
        let insertion = |text: &str| {
            format!(
                r#"<w:ins w:id="5" w:author="Alice" w:date="2026-10-17T09:00:00Z">{}</w:ins>"#,
                plain_run(text)
            )
        };
        let source = paragraph(&[alice_formatted(5, "foo", false), insertion("old")].concat());
        let candidate = paragraph(&[alice_formatted(5, "foo", false), insertion("new")].concat());
        refused(preserve_direct_history(
            &source,
            &candidate,
            &rewriting("r5", "foo", 0),
            "Carol Diaz",
            &BTreeMap::new(),
        ));
    }

    /// A paragraph of `words` whose justification Alice changed, in record `id`.
    fn alice_justified(id: u32, words: &str) -> String {
        format!(
            r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="{id}" w:author="Alice" w:date="2026-10-17T09:00:00Z"><w:pPr/></w:pPrChange></w:pPr>{}</w:p>"#,
            plain_run(words)
        )
    }

    #[test]
    fn history_guard_rewrites_only_the_run_record_an_operation_reached() {
        // A `w:pPrChange` may share `w:id` 5 with Alice's run record: the
        // two are told apart by type. Carol's direct edit reached the run.
        let other = alice_justified(5, "other words");
        let source = docx(
            &[
                other.clone(),
                format!("<w:p>{}</w:p>", alice_formatted(5, "alpha beta", false)),
            ]
            .concat(),
        );
        let guard = |candidate: &str, edited: &Edited| {
            preserve_direct_history(
                &source,
                &docx(&[other.clone(), format!("<w:p>{candidate}</w:p>")].concat()),
                edited,
                "Carol Diaz",
                &BTreeMap::new(),
            )
        };
        let replaced = [alice_formatted(5, "alpha ", false), plain_run("new")].concat();
        let result = guard(&replaced, &rewriting("r5", "alpha ", 4));
        assert!(result.is_ok(), "{result:?}");
        // The run record goes with its last word; the paragraph's stays.
        let result = guard(&plain_run("new"), &rewriting("r5", "", 10));
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn history_guard_rewrites_no_run_record_whose_id_repeats() {
        // Two run records share `w:id` 5 (`DUPLICATE_REVISION_ID`). Carol
        // took all of the first's words: the leniency cannot tell which
        // record the operation reached, so neither may go.
        let source = docx(
            &[
                format!("<w:p>{}</w:p>", alice_formatted(5, "foo", false)),
                format!("<w:p>{}</w:p>", alice_formatted(5, "bar", false)),
            ]
            .concat(),
        );
        let guard = |second: &str| {
            preserve_direct_history(
                &source,
                &docx(
                    &[
                        format!("<w:p>{}</w:p>", plain_run("new")),
                        format!("<w:p>{second}</w:p>"),
                    ]
                    .concat(),
                ),
                &rewriting("r5", "", 3),
                "Carol Diaz",
                &BTreeMap::new(),
            )
        };
        refused(guard(&plain_run("bar")));
        refused(guard(&alice_formatted(5, "bar", false)));
    }

    #[test]
    fn history_guard_refuses_a_record_left_on_no_words() {
        // Carol deleted every word of Alice's run: the record goes with
        // them, so a candidate that keeps it on an emptied run is refused
        // (Codex on #1664).
        let source = docx(&format!(
            "<w:p>{}</w:p>",
            alice_formatted(5, "alpha beta", false)
        ));
        let guard = |candidate: &str| {
            preserve_direct_history(
                &source,
                &docx(&format!("<w:p>{candidate}</w:p>")),
                &rewriting("r5", "", 10),
                "Carol Diaz",
                &BTreeMap::new(),
            )
        };
        refused(guard(&alice_formatted(5, "", false)));
        let result = guard("");
        assert!(result.is_ok(), "{result:?}");
    }

    /// Alice's run `text` followed, inside the run, by a tab.
    fn alice_with_tab(id: u32, text: &str) -> String {
        alice_formatted(id, text, false).replace("</w:t></w:r>", "</w:t><w:tab/></w:r>")
    }

    #[test]
    fn history_guard_counts_the_atoms_a_rewritten_record_covers() {
        // Alice's `abc`, then a plain tab: Carol's direct replace of `b`
        // takes one character of her record, and the tab was never hers.
        let tab = "<w:r><w:tab/></w:r>".to_string();
        let source = paragraph(&[alice_formatted(5, "abc", false), tab.clone()].concat());
        let edited = rewriting("r5", "ac", 1);
        let guard = |candidate: &[u8]| {
            preserve_direct_history(&source, candidate, &edited, "Carol Diaz", &BTreeMap::new())
        };
        let head = [alice_formatted(5, "a", false), plain_run("X")].concat();
        let kept = paragraph(&[head.clone(), alice_formatted(9, "c", false), tab].concat());
        let result = guard(&kept);
        assert!(result.is_ok(), "{result:?}");
        refused(guard(&paragraph(&[head, alice_with_tab(9, "c")].concat())));
    }

    #[test]
    fn history_guard_keeps_a_record_while_an_atom_of_its_run_remains() {
        // Carol replaces all of Alice's words; the tab of her run stays hers.
        let source = paragraph(&alice_with_tab(5, "abc"));
        let edited = rewriting("r5", "", 3);
        let guard = |candidate: &[u8]| {
            preserve_direct_history(&source, candidate, &edited, "Carol Diaz", &BTreeMap::new())
        };
        let tab_only = alice_with_tab(5, "").replace(r#"<w:t xml:space="preserve"></w:t>"#, "");
        let result = guard(&paragraph(&[plain_run("X"), tab_only].concat()));
        assert!(result.is_ok(), "{result:?}");
        refused(guard(&paragraph(
            &[plain_run("X"), "<w:r><w:tab/></w:r>".to_string()].concat(),
        )));
    }
}
