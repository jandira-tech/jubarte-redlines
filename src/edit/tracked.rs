// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `existing_revisions: "keep"`: the redline by direct emission.
//!
//! The comparer accepts both inputs' revisions before it compares, so it
//! cannot keep another party's tracked changes. Under `keep` the plan's
//! resolved operations are replayed on a fresh copy of the base and written
//! as tracked changes by the plan's author and date, which is what Word does
//! when you type on a received redline with Track Changes on: new `w:ins`
//! and `w:del` runs, inserted and deleted paragraph marks, and `w:pPrChange`
//! for paragraph formatting. An edit inside another author's insertion
//! nests the deletion there and splits off the new words and remainder.
//! The editor's own insertion is edited in place.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::structural;
use super::visible;
use super::{
    EditError, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved, RevisionCounter,
    RevisionCounts, ScheduledEdit, Side, ThreadOp, Transaction, anchor_comment, anchor_span,
    apply_run_format, apply_text_edit, attach_segment, build_paragraph, comment_bounds,
    format_paragraph, insert_ppr_child, new_position, place_reply_markers, remove_comment_markers,
    runs_in_range, set_text, split_run_at, toc_paragraphs,
};
use crate::changes::{list_changes, list_logical_changes};
use crate::inspect::{Opened, Piece, project_paragraph};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XName};

/// The original identity of a pending insertion, retained across every split.
#[derive(Clone)]
struct InsertionOrigin {
    story: String,
    id: String,
    root: NodeId,
}

/// Revision attribution for the containers the emitter creates.
struct Stamp<'a> {
    author: &'a str,
    date: &'a str,
    ids: RevisionCounter,
    insertions: HashMap<NodeId, InsertionOrigin>,
}

impl Stamp<'_> {
    fn container(&mut self, dom: &mut Dom, name: XName) -> NodeId {
        let node = dom.new_element(name);
        let id = self.ids.allocate();
        dom.set_attribute_value(node, &W::id(), Some(&id.to_string()));
        dom.set_attribute_value(node, &W::author(), Some(self.author));
        dom.set_attribute_value(node, &W::date(), Some(self.date));
        node
    }

    fn insertion_origins(&self, dom: &Dom) -> BTreeMap<String, String> {
        self.insertions
            .iter()
            .filter_map(|(&node, origin)| {
                if !dom.ancestors(node, None).contains(&origin.root) {
                    return None;
                }
                let id = dom.attribute(node, &W::id())?;
                Some((
                    crate::ids::ChangeId::new(&origin.story, id).short(),
                    origin.id.clone(),
                ))
            })
            .collect()
    }
}

/// The operation kind of `resolved` when the emitter cannot write it as a
/// tracked change: such a plan goes through the comparer, or is refused
/// under `keep`, never left out of the redline.
fn untracked_kind(tx: &Transaction<'_>, operation: usize) -> Option<&'static str> {
    let kind = super::kind_name(&tx.plan.operations[operation].kind);
    (!super::operation_supports_keep(kind)).then_some(kind)
}

/// Whether the emitter writes every operation of `tx` as tracked changes.
pub(super) fn emits_all(tx: &Transaction<'_>) -> bool {
    tx.resolved
        .iter()
        .all(|(op, _)| untracked_kind(tx, *op).is_none())
}

/// The tracked formatting change (`w:rPrChange`) of the first run holding
/// `[start, end)` of `paragraph` that has one.
fn formatting_tracked(dom: &Dom, paragraph: NodeId, start: usize, end: usize) -> Option<NodeId> {
    project_paragraph(dom, paragraph)
        .segments
        .iter()
        .filter(|s| s.start < end && start < s.end)
        .find_map(|s| {
            let (Piece::Text { run, .. } | Piece::Glyph { run, .. }) = &s.piece;
            dom.element(*run, &W::r_pr())
                .and_then(|rpr| dom.element(rpr, &W::r_pr_change()))
        })
}

/// Refusals that only `keep` needs, checked on the untouched working source.
/// Each revision conflict retains its actual carriers for caller-specific advice.
pub(super) fn check(tx: &Transaction<'_>) -> Result<(), EditError> {
    if tx.plan.existing_revisions != ExistingRevisions::Keep {
        return Ok(());
    }
    tx.revision_refusal.take();
    let dom = &tx.opened.dom;
    let emptied = tx.emptied_paragraphs();
    let refuse = |op: usize, message: &str| {
        let mut error = tx.conflict(op, message);
        error.code = "UNSUPPORTED_STRUCTURE".into();
        error.outcomes[op].code = Some(error.code.clone());
        tx.attach_revision_refusal(error)
    };
    for (op, resolved) in &tx.resolved {
        if let Some(kind) = untracked_kind(tx, *op) {
            return Err(refuse(
                *op,
                &format!(
                    "under keep, {kind} cannot be written as a tracked change yet; apply it in a plan without existing_revisions keep"
                ),
            ));
        }
        // What is refused, then the revision in the way and each route
        // around it (#1108).
        let named = |lead: &str, revision: NodeId| tx.revision_hint(revision, lead);
        let message = match resolved {
            Resolved::DeleteParagraph { para } => {
                let node = tx.paragraph_nodes[*para];
                let replacement = tx.final_replacement(*para);
                if let Some(moved) = moved_in(dom, node) {
                    Some(named(
                        "under keep, a paragraph that holds a move cannot be deleted yet: the move is",
                        moved,
                    ))
                } else if let Some(foreign) = untrackable_content(dom, node) {
                    let name = dom.name(foreign).expect("an element");
                    Some(format!(
                        "under keep, a paragraph holding content other than runs ({} in {}) cannot be deleted as a tracked change yet; apply it in a plan without existing_revisions keep",
                        name.local_name(),
                        name.namespace_name()
                    ))
                } else if let Some(change) =
                    replacement.and_then(|op| record_replaced(tx, node, op))
                {
                    Some(named(
                        "under keep, a final paragraph whose formatting change is still tracked cannot be replaced by a paragraph formatted otherwise: that change is",
                        change,
                    ))
                } else if replacement.is_none()
                    && !emptied.contains(para)
                    && next_block(dom, node).is_none()
                    // A final paragraph's carrier keeps its own mark
                    // (`final_carrier`); this is the last paragraph of a
                    // content control or a custom XML element.
                    && tx.final_paragraph(node) != Some(node)
                {
                    previous_paragraph(dom, node)
                        .and_then(|p| mark_revision(dom, p, &DELETED))
                        .map(|mark| {
                            named(
                                "under keep, the last paragraph of a content control or a custom XML element cannot be deleted when the paragraph before it already has a deleted mark: that mark is",
                                mark,
                            )
                        })
                } else {
                    None
                }
            }
            // A table placed last leaves the final mark where it is: the
            // paragraph after it takes the inserted mark (CodeRabbit on
            // #1680).
            Resolved::InsertParagraph { .. }
                if appended_final(tx, resolved).is_some_and(|para| {
                    last_appended(tx, para) == Some(*op)
                        && !tx.resolved.iter().any(|(_, r)| {
                            matches!(r, Resolved::DeleteParagraph { para: p } if *p == para)
                        })
                }) =>
            {
                let para = appended_final(tx, resolved).expect("matched");
                let node = tx.paragraph_nodes[para];
                if let Some(mark) = mark_revision(dom, node, &TRACKED) {
                    Some(named(
                        "under keep, a paragraph cannot be added after a container's last paragraph whose mark is already tracked, since the inserted mark goes there: that mark is",
                        mark,
                    ))
                } else {
                    record_replaced(tx, node, *op).map(|change| {
                        named(
                            "under keep, a paragraph formatted otherwise cannot be added after a container's last paragraph whose formatting change is still tracked: that change is",
                            change,
                        )
                    })
                }
            }
            Resolved::MergeParagraphs { para, .. } => {
                mark_revision(dom, tx.paragraph_nodes[*para], &TRACKED).map(|mark| {
                    named(
                        "under keep, a paragraph whose mark is already tracked cannot be merged: that mark is",
                        mark,
                    )
                })
            }
            Resolved::FormatParagraph { para, .. } => {
                paragraph_format_change(dom, tx.paragraph_nodes[*para]).map(|change| {
                    named(
                        "under keep, a paragraph whose formatting change is still tracked cannot be formatted again: that change is",
                        change,
                    )
                })
            }
            Resolved::List { paras, .. } => paras
                .iter()
                .find_map(|&para| paragraph_format_change(dom, tx.paragraph_nodes[para]))
                .map(|change| {
                    named(
                        "under keep, a paragraph whose formatting change is still tracked cannot be numbered: that change is",
                        change,
                    )
                }),
            Resolved::FormatRun {
                para, start, end, ..
            } => formatting_tracked(dom, tx.paragraph_nodes[*para], *start, *end).map(|change| {
                named(
                    "under keep, text whose formatting change is still tracked cannot be formatted again: that change is",
                    change,
                )
            }),
            _ => None,
        };
        if let Some(message) = message {
            return Err(refuse(*op, &message));
        }
    }
    Ok(())
}

/// The result under `keep`: the clean copy as `finish` made it, the redline
/// by emission, and revision counts of the plan's own changes.
pub(super) fn result(tx: &Transaction<'_>, clean: Vec<u8>) -> Result<EditResult, EditError> {
    result_with_order(tx, clean, false)
}

/// The tracked primary's paragraph identity, for the v2 expectations
/// correspondence: `base` walks the replay's starting DOM (positionally the
/// transaction's own `paragraph_nodes`, both loaded from the same base
/// bytes), and `output` walks it again after the tracked emission — in the
/// checker's projection order, so output indices address the reopened
/// redline's paragraphs.
#[derive(Clone, Debug)]
pub(super) struct TrackedParagraphs {
    /// The replay's base paragraphs, one per transaction paragraph.
    pub(super) base: Vec<NodeId>,
    /// The emitted redline's paragraphs, in projection order.
    pub(super) output: Vec<NodeId>,
    /// Each `output` paragraph's final-view disposition, computed on the
    /// replay's own DOM before this call returns and that DOM is dropped.
    /// The correspondence reads the redline's marks here instead of
    /// reopening the emitted bytes (#1087); `None` when the caller asked
    /// for no final-view scan. These equal a reparse of the emitted
    /// redline because `finish` changes that DOM only through
    /// `apply_watermark`, which no v2 operation reaches; the regression
    /// net under `v2_apply`'s tests pins that equivalence.
    pub(super) output_states: Option<Vec<super::v2_apply::FinalView>>,
    /// Every surviving source insertion piece's emitted id -> source id.
    /// Split pieces share an origin even when a leading edit renumbers it.
    pub(super) insertion_origins: BTreeMap<String, String>,
}

/// As `result_v2`, also returning the tracked primary's paragraph walks
/// (see [`TrackedParagraphs`]).
pub(super) fn result_v2_with_paragraphs(
    tx: &Transaction<'_>,
    clean: Vec<u8>,
) -> Result<(EditResult, TrackedParagraphs), EditError> {
    let (redline, own, paragraphs) = redline(tx, true, true)?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline, &tx.plan.author, &tx.date, &own);
    report.split_revisions = splits(&paragraphs.insertion_origins);
    Ok((
        EditResult {
            clean,
            redline,
            report,
        },
        paragraphs,
    ))
}

fn result_with_order(
    tx: &Transaction<'_>,
    clean: Vec<u8>,
    array_order: bool,
) -> Result<EditResult, EditError> {
    let (redline, own, paragraphs) = redline(tx, array_order, false)?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline, &tx.plan.author, &tx.date, &own);
    report.split_revisions = splits(&paragraphs.insertion_origins);
    Ok(EditResult {
        clean,
        redline,
        report,
    })
}

/// Replay `tx`'s resolved operations on a fresh copy of its base as tracked
/// changes. `tx` has been applied, so its outcomes hold the comment ids.
/// Returns the redline, the ids its own revisions took (copies of source
/// records excluded) and the replay's paragraph walks
/// ([`TrackedParagraphs`]); the walk is taken after the emission and
/// before `finish`. `finish` changes the DOM only through
/// `apply_watermark`, a step no v2 operation reaches, so the walk still
/// addresses the emitted redline's paragraphs; the regression net under
/// `v2_apply`'s tests pins that. When `final_views` is
/// set, each output paragraph's final-view disposition is scanned off the
/// replay's DOM here, while that DOM still lives — the one place the
/// emitted redline's marks can be read without reopening its bytes.
fn redline(
    tx: &Transaction<'_>,
    array_order: bool,
    final_views: bool,
) -> Result<(Vec<u8>, BTreeSet<u64>, TrackedParagraphs), EditError> {
    let plan = EditPlan {
        source_sha256: None,
        date: Some(tx.date.clone()),
        initials: Some(tx.initials.clone()),
        resolve_revisions: None,
        existing_revisions: ExistingRevisions::Keep,
        ..tx.plan.clone()
    };
    let mut t = Transaction::start(&tx.base, &plan)?;
    t.resolved = tx.resolved.clone();
    // A watermark is header content, not a change: written as is.
    t.watermark = tx.watermark.clone();
    // Settings are not revisions either.
    t.settings = tx.settings.clone();
    // The comments of commented paragraph deletions are written here, on
    // the deleted text, instead of into a commented base for the comparer.
    let mut comments = tx.comments.clone();
    for &(id, op) in &tx.deletion_comments {
        if let Some(text) = tx.deleted_text_comment(op) {
            comments.push((id, text.to_string()));
        }
    }
    comments.sort_by_key(|&(id, _)| id);
    let comment_ids: BTreeMap<usize, u32> = tx
        .outcomes
        .iter()
        .enumerate()
        .filter_map(|(op, outcome)| outcome.comment_id.map(|id| (op, id)))
        .collect();
    let mut insertions = HashMap::new();
    for story in &t.stories {
        for node in t.opened.dom.descendants(story.root, Some(&W::ins())) {
            if let Some(id) = t.opened.dom.attribute(node, &W::id()) {
                insertions.insert(
                    node,
                    InsertionOrigin {
                        story: story.id.clone(),
                        id: crate::ids::ChangeId::new(&story.id, id).short(),
                        root: story.root,
                    },
                );
            }
        }
    }
    // The replay starts from the settled base: the source's ids, settled
    // away or not, stay taken too.
    let mut used = used_ids(&t.opened, &comments);
    used.extend(&tx.used_revision_ids);
    let mut stamp = Stamp {
        author: &tx.plan.author,
        date: &tx.date,
        ids: RevisionCounter::new(used),
        insertions,
    };
    let tables = emit(&mut t, &comment_ids, &mut stamp, array_order);
    thread_markers(&mut t, &comment_ids);
    // 7. A paragraph after each new table, and between it and a table
    // before it: inserted paragraphs here.
    for table in tables {
        separate_tracked(&mut t.opened.dom, table, &mut stamp);
    }
    t.comments = comments;
    t.reply_parents = tx.reply_parents.clone();
    let base = t.paragraph_nodes.clone();
    let output = t.paragraph_walk();
    let output_states = final_views.then(|| {
        output
            .iter()
            .map(|&node| super::v2_apply::paragraph_final_state(&t.opened.dom, node))
            .collect()
    });
    let insertion_origins = stamp.insertion_origins(&t.opened.dom);
    let (redline, _) = t.finish()?;
    Ok((
        redline,
        stamp.ids.own().clone(),
        TrackedParagraphs {
            base,
            output,
            output_states,
            insertion_origins,
        },
    ))
}

/// The source insertions the emission split, from each emitted piece's
/// origin: a source id with more than one piece, or whose one piece took a
/// new id.
fn splits(origins: &BTreeMap<String, String>) -> Vec<super::RevisionSplit> {
    let mut pieces: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (emitted, source) in origins {
        pieces.entry(source).or_default().push(emitted);
    }
    // `r9` before `r10`: the number, then the text for an id without one.
    let order = |id: &str| {
        (
            crate::ids::change_id(id).and_then(|c| c.wid.parse::<u64>().ok()),
            id.to_string(),
        )
    };
    let mut out: Vec<super::RevisionSplit> = pieces
        .into_iter()
        .filter(|(source, to)| to.len() > 1 || to.first() != Some(source))
        .map(|(source, mut to)| {
            to.sort_by_key(|id| order(id));
            super::RevisionSplit {
                from: source.to_string(),
                to: to.into_iter().map(str::to_string).collect(),
            }
        })
        .collect();
    out.sort_by_key(|split| order(&split.from));
    out
}

/// Counts of the redline's logical changes holding a mark stamped with
/// `author` and `date` whose `w:id` is one of `own_ids`, the ids the edit's
/// own records took. A source change carrying the very same stamp, or a
/// copy of one a split made, has another id, so it does not count.
fn revision_counts(
    redline: &[u8],
    author: &str,
    date: &str,
    own_ids: &BTreeSet<u64>,
) -> RevisionCounts {
    let changes = list_changes(redline).unwrap_or_default();
    let own: HashSet<&str> = changes
        .iter()
        .filter(|c| {
            c.author.as_deref() == Some(author)
                && c.date.as_deref() == Some(date)
                && crate::ids::change_id(&c.id)
                    .and_then(|id| id.wid.parse::<u64>().ok())
                    .is_some_and(|id| own_ids.contains(&id))
        })
        .map(|c| c.id.as_str())
        .collect();
    let logical = list_logical_changes(redline).unwrap_or_default();
    RevisionCounts::from_changes(
        logical
            .iter()
            .filter(|c| c.element_ids().any(|id| own.contains(id))),
    )
}

/// Every `w:id` in the revision-bearing parts and among the plan's
/// comments, so a new revision id collides with none of them.
pub(super) fn used_ids(opened: &Opened, comments: &[(u32, String)]) -> HashSet<u64> {
    let mut used: HashSet<u64> = comments.iter().map(|&(id, _)| u64::from(id)).collect();
    for (part, _) in crate::revision_processor::revision_bearing_parts(&opened.pkg) {
        let Some(xml) = opened.pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            continue;
        };
        for node in dom.descendants(root, None) {
            if let Some(id) = dom
                .attribute(node, &W::id())
                .and_then(|v| v.parse::<u64>().ok())
            {
                used.insert(id);
            }
        }
    }
    used
}

/// The steps of `Transaction::apply`, in its order, as tracked changes.
/// Returns the new tables, which get their separating paragraphs last.
fn emit(
    t: &mut Transaction<'_>,
    comment_ids: &BTreeMap<usize, u32>,
    stamp: &mut Stamp<'_>,
    array_order: bool,
) -> Vec<NodeId> {
    // Read off the source's structure, before any step changes it: the
    // container-final paragraphs whose mark stays (Word never deletes it).
    let emptied = t.emptied_paragraphs();
    let finals: BTreeSet<usize> = t
        .resolved
        .iter()
        .filter_map(|(_, r)| match r {
            Resolved::DeleteParagraph { para }
                if t.final_paragraph(t.paragraph_nodes[*para])
                    == Some(t.paragraph_nodes[*para]) =>
            {
                Some(*para)
            }
            _ => None,
        })
        .collect();
    // The operation whose new paragraph takes each final mark that stays,
    // and the paragraph that carries each other final mark away.
    let replacements: BTreeMap<usize, usize> = finals
        .iter()
        .filter_map(|&para| t.final_replacement(para).map(|op| (para, op)))
        .collect();
    let carriers: BTreeMap<usize, NodeId> = finals
        .iter()
        .filter_map(|&para| t.final_carrier(para).map(|carrier| (para, carrier)))
        .collect();
    // The container-final paragraphs that stay, with blocks placed after
    // them (#1669).
    let appended: BTreeSet<usize> = t
        .resolved
        .iter()
        .filter_map(|(_, r)| appended_final(t, r))
        .filter(|para| !finals.contains(para))
        .collect();
    // 0. Run formatting, on source offsets (no text edit overlaps it), each
    // run's old properties in `w:rPrChange`.
    for (_, r) in &t.resolved {
        let Resolved::FormatRun {
            para,
            start,
            end,
            format,
        } = r
        else {
            continue;
        };
        let node = t.paragraph_nodes[*para];
        track_run_format(&mut t.opened.dom, node, (*start, *end), format, stamp);
    }
    // 1. Text edits and their comments, paragraph by paragraph.
    let mut by_para: BTreeMap<usize, Vec<ScheduledEdit>> = BTreeMap::new();
    let mut comment_ranges: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    let mut strike_comments: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    for (i, r) in &t.resolved {
        match r {
            Resolved::StrikeComment { para, start, end } => strike_comments
                .entry(*para)
                .or_default()
                .push((*start, *end, *i)),
            Resolved::Text {
                para,
                start,
                end,
                replacement,
                attach_before,
                comment,
                format,
            } => by_para.entry(*para).or_default().push(ScheduledEdit {
                start: *start,
                end: *end,
                op: *i,
                replacement: replacement.clone(),
                attach_before: *attach_before,
                comment: comment.clone(),
                format: format.clone(),
            }),
            Resolved::CommentRange {
                para, start, end, ..
            } => comment_ranges
                .entry(*para)
                .or_default()
                .push((*start, *end, *i)),
            _ => {}
        }
    }
    let touched: BTreeSet<usize> = by_para
        .keys()
        .chain(comment_ranges.keys())
        .copied()
        .collect();
    for para in touched {
        let node = t.paragraph_nodes[para];
        let dom = &mut t.opened.dom;
        let mut edits = by_para.remove(&para).unwrap_or_default();
        let mut empty = project_paragraph(dom, node)
            .text
            .is_empty()
            .then(super::EmptyPlacement::default);
        super::sort_edits(&mut edits, empty.is_some());
        // In reverse, so earlier source offsets stay valid: a deletion
        // leaves the projection and an insertion joins it, exactly as the
        // plain edit changes the clean copy.
        // Emit a simultaneous V2 insertion group once, using its agreed
        // original attachment side. Otherwise the first replayed insertion
        // becomes a non-direct segment and can hide the source insertion
        // point from the next replay (notably at paragraph start). Keep the
        // original edits below for each comment's separate resulting range.
        let emission = if array_order {
            coalesce_insertions(&edits)
        } else {
            edits.clone()
        };
        // A strike's comment, on the words before they are struck: their
        // text leaves the projection with the deletion.
        for (start, end, i) in strike_comments.remove(&para).unwrap_or_default() {
            if let Some(&id) = comment_ids.get(&i) {
                anchor_comment(dom, node, start, end, id, &mut stamp.ids);
            }
        }
        for edit in emission.iter().rev() {
            if matches!(
                t.plan.operations[edit.op].kind,
                OperationKind::Redact { .. }
            ) {
                // A redaction is no change: its blocks replace the text
                // outright, as in the clean copy.
                let projection = project_paragraph(dom, node);
                apply_text_edit(
                    dom,
                    &projection,
                    edit.start,
                    edit.end,
                    &edit.replacement,
                    edit.attach_before,
                );
                super::clean_new_words(
                    dom,
                    node,
                    edit.start,
                    edit.start + edit.replacement.len(),
                    &mut stamp.ids,
                );
            } else {
                let pieces = if matches!(
                    t.plan.operations[edit.op].kind,
                    OperationKind::Replace { whole: false, .. }
                ) {
                    insertion_replacement(dom, node, edit)
                } else {
                    vec![edit.clone()]
                };
                for piece in pieces.iter().rev() {
                    emit_text(dom, node, piece, stamp, empty.as_mut());
                }
            }
        }
        let mut pending: Vec<(usize, usize, usize)> = Vec::new();
        for edit in &edits {
            if edit.comment.is_some() {
                let s = new_position(&edits, edit.start, true, Some(edit.op));
                pending.push((s, s + edit.replacement.len(), edit.op));
            }
        }
        for (start, end, i) in comment_ranges.remove(&para).unwrap_or_default() {
            let (s, e) = comment_bounds(&edits, start, end, i);
            pending.push((s, e, i));
        }
        pending.sort_by_key(|&(s, _, i)| (s, i));
        for (start, end, i) in pending {
            if let Some(&id) = comment_ids.get(&i) {
                anchor_comment(dom, node, start, end, id, &mut stamp.ids);
            }
        }
    }
    // 1b. Comments over several paragraphs, in their edited text.
    for (i, r) in &t.resolved {
        if let Resolved::CommentSpan { para, last, .. } = r
            && let Some(&id) = comment_ids.get(i)
        {
            let (first, last) = (t.paragraph_nodes[*para], t.paragraph_nodes[*last]);
            anchor_span(&mut t.opened.dom, first, last, id, &mut stamp.ids);
        }
    }
    // 1c. Restores: their deletion stays, and our insertion of a copy of
    // its runs follows it.
    for (i, r) in &t.resolved {
        let Resolved::Restore { para, wid, .. } = r else {
            continue;
        };
        let node = t.paragraph_nodes[*para];
        let dom = &mut t.opened.dom;
        let Some(del) = visible::revision_in(dom, node, "del", wid) else {
            continue;
        };
        let ours = stamp.container(dom, W::ins());
        for copy in visible::copy_runs(dom, del) {
            dom.add(ours, copy);
        }
        let runs = dom.elements(ours, None);
        if let (Some(&first), Some(&last), Some(&id)) =
            (runs.first(), runs.last(), comment_ids.get(i))
        {
            visible::comment_around(dom, first, last, id);
        }
        place_beside(dom, del, ours, stamp);
    }
    // 2. Paragraph and table insertions: every run, row and paragraph mark
    // inserted.
    let mut last_after: BTreeMap<usize, NodeId> = BTreeMap::new();
    // Each insertion's last new block.
    let mut inserted: BTreeMap<usize, NodeId> = BTreeMap::new();
    let mut tables: Vec<NodeId> = Vec::new();
    for (i, r) in &t.resolved {
        let (anchor, side, news, commented) = match r {
            Resolved::InsertParagraph {
                anchor,
                side,
                runs,
                like,
                style,
                comment,
                toc,
            } => {
                let like_node = t.paragraph_nodes[*like];
                let dom = &mut t.opened.dom;
                let news = match toc {
                    Some(toc) => toc_paragraphs(dom, toc),
                    None => vec![build_paragraph(dom, like_node, runs, style.as_deref())],
                };
                for &new in &news {
                    insert_paragraph_content(dom, new, stamp);
                }
                (*anchor, *side, news, comment.is_some())
            }
            Resolved::InsertTable {
                anchor,
                side,
                rows,
                header_row,
                widths,
                style,
                ..
            } => {
                let anchor_node = t.paragraph_nodes[*anchor];
                let dom = &mut t.opened.dom;
                let new =
                    structural::build_table(dom, anchor_node, rows, *header_row, widths, style);
                insert_table_content(dom, new, stamp);
                tables.push(new);
                (*anchor, *side, vec![new], false)
            }
            _ => continue,
        };
        let anchor_node = t.paragraph_nodes[anchor];
        let dom = &mut t.opened.dom;
        for &new in &news {
            match side {
                Side::After => {
                    let prev = last_after.get(&anchor).copied().unwrap_or(anchor_node);
                    dom.add_after_self(prev, new);
                    last_after.insert(anchor, new);
                }
                Side::Before => {
                    dom.add_before_self(anchor_node, new);
                }
            }
        }
        let new = news[news.len() - 1];
        inserted.insert(*i, new);
        if commented && let Some(&id) = comment_ids.get(i) {
            let projection = project_paragraph(dom, new);
            anchor_comment(dom, new, 0, projection.text.len(), id, &mut stamp.ids);
        }
    }
    // 2b. The last new paragraph after a container's last paragraph takes
    // the final mark, unrevised, and the inserted mark is the one before
    // it, as Word's Compare writes an append: a revision on a table cell's
    // final mark hangs Word's Accept All and Reject All (#1669). In the
    // body, whose final mark Word resolves, a mark that closes a section
    // stays: the record cannot hold the break (`CT_PPrBase` has no
    // `w:sectPr`), so Reject All would drop it (codex on #1680).
    for para in appended {
        let Some(&last) = last_after.get(&para) else {
            continue;
        };
        let dom = &mut t.opened.dom;
        let old = t.paragraph_nodes[para];
        let closes_section = dom
            .parent(old)
            .is_some_and(|up| dom.name_is(up, &W::body()))
            && dom
                .element(old, &W::p_pr())
                .is_some_and(|ppr| dom.element(ppr, &W::sect_pr()).is_some());
        if dom.name_is(last, &W::p()) && !closes_section {
            rotate_final_mark(dom, old, last, stamp);
        }
    }
    // 3. Paragraph formatting, with the old properties in `w:pPrChange`.
    for (_, r) in &t.resolved {
        let Resolved::FormatParagraph {
            para,
            style,
            alignment,
            spacing,
        } = r
        else {
            continue;
        };
        let node = t.paragraph_nodes[*para];
        track_properties(&mut t.opened.dom, node, stamp, |dom| {
            format_paragraph(dom, node, style.as_deref(), *alignment, *spacing);
        });
    }
    // 3b. Lists, numbered as `Transaction::apply` numbers them, each
    // paragraph's old properties in `w:pPrChange`.
    let (mut next_abstract, mut next_num) = structural::next_numbering_ids(&t.opened);
    for (_, r) in &t.resolved {
        let Resolved::List {
            paras,
            kind,
            level,
            join,
            style,
            add_style,
        } = r
        else {
            continue;
        };
        let num_id = match join {
            Some(num_id) => num_id.clone(),
            None => {
                let (abstracts, nums) = &mut t.new_numbering;
                abstracts.push(crate::markdown::xml::abstract_num(
                    next_abstract,
                    kind.format(),
                ));
                nums.push(crate::markdown::xml::num(next_num, next_abstract, None));
                next_abstract += 1;
                next_num += 1;
                (next_num - 1).to_string()
            }
        };
        for &para in paras {
            let node = t.paragraph_nodes[para];
            let mut styled = false;
            track_properties(&mut t.opened.dom, node, stamp, |dom| {
                styled = structural::number_paragraph(dom, node, *level, &num_id, style);
            });
            if styled && *add_style {
                t.needed_styles.insert(style.clone());
            }
        }
    }
    // 4. Merges: the head's mark deleted, the separator inserted.
    let mut merges: Vec<(usize, String)> = t
        .resolved
        .iter()
        .filter_map(|(_, r)| match r {
            Resolved::MergeParagraphs {
                para, separator, ..
            } => Some((*para, separator.clone())),
            _ => None,
        })
        .collect();
    merges.sort_by_key(|&(para, _)| para);
    for (para, separator) in merges {
        let head = t.paragraph_nodes[para];
        let dom = &mut t.opened.dom;
        if !separator.is_empty() {
            let run = dom.new_element(W::r());
            let last_rpr = dom
                .elements(head, Some(&W::r()))
                .last()
                .and_then(|&r| dom.element(r, &W::r_pr()));
            if let Some(rpr) = last_rpr {
                let rpr = clean_rpr(dom, rpr);
                dom.add(run, rpr);
            }
            let text = dom.new_element(W::t());
            set_text(dom, text, &separator);
            dom.add(run, text);
            let ins = stamp.container(dom, W::ins());
            dom.add(ins, run);
            dom.add(head, ins);
        }
        mark(dom, head, W::del(), stamp);
    }
    // 5. Paragraph deletions: every run deleted, and a deleted mark.
    for (i, r) in &t.resolved {
        let Resolved::DeleteParagraph { para } = r else {
            continue;
        };
        let node = t.paragraph_nodes[*para];
        let dom = &mut t.opened.dom;
        if let Some(&id) = comment_ids.get(i) {
            let projection = project_paragraph(dom, node);
            anchor_comment(dom, node, 0, projection.text.len(), id, &mut stamp.ids);
        }
        delete_runs(dom, node, stamp);
        // A container's final mark stays, as Word keeps it: emptied, or
        // holding the new paragraphs' last (#1668).
        if emptied.contains(para) {
            continue;
        }
        if let Some(&last) = replacements.get(para).and_then(|op| inserted.get(op)) {
            keep_final_mark(dom, node, last, stamp);
            continue;
        }
        // A paragraph before another or before a table takes a deleted
        // mark of its own, as Word's Compare writes it. The mark of the
        // last paragraph of a container cannot go: its content joins the
        // paragraph before it, whose mark is deleted.
        let carrier = if let Some(next) = next_block(dom, node) {
            // Between a table the plan inserts and another table, the
            // paragraph stays, emptied: it is the paragraph the clean copy
            // puts between them (glm-5.3 on #1670, round 8).
            let before = super::previous_block(dom, node).filter(|&b| dom.name_is(b, &W::tbl()));
            let separates = dom.name_is(next, &W::tbl())
                && before.is_some_and(|b| tables.contains(&b) || tables.contains(&next));
            (!separates).then_some(node)
        } else if let Some(&carrier) = carriers.get(para) {
            Some(carrier)
        } else {
            let mut cursor = previous_paragraph(dom, node);
            while let Some(p) = cursor.filter(|&p| mark_has(dom, p, &DELETED)) {
                cursor = previous_paragraph(dom, p);
            }
            cursor
        };
        if let Some(carrier) = carrier {
            mark(dom, carrier, W::del(), stamp);
        }
    }
    tables
}

fn coalesce_insertions(edits: &[ScheduledEdit]) -> Vec<ScheduledEdit> {
    let mut emission: Vec<ScheduledEdit> = Vec::new();
    for edit in edits {
        if let Some(previous) = emission.last_mut()
            && previous.start == previous.end
            && edit.start == edit.end
            && previous.start == edit.start
            && previous.attach_before == edit.attach_before
            && previous.format == edit.format
        {
            previous.replacement.push_str(&edit.replacement);
        } else {
            emission.push(edit.clone());
        }
    }
    emission
}

/// `last`, the new paragraph that replaces the container-final `old`, takes
/// `old`'s place and mark, as Word writes text typed over a final
/// paragraph: `old` holds `last`'s inserted content before its own deleted
/// content, and `last`'s properties, the old ones in a `w:pPrChange` when
/// they differ; its mark takes `last`'s formatting, the old in a
/// `w:rPrChange`. A change record already there stays: `check` refuses,
/// under keep, the plans that would have to replace it. The other new
/// paragraphs keep their inserted marks before it.
fn keep_final_mark(dom: &mut Dom, old: NodeId, last: NodeId, stamp: &mut Stamp<'_>) {
    let content: Vec<NodeId> = dom
        .nodes(last)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    let first = dom
        .nodes(old)
        .into_iter()
        .find(|&c| !dom.name_is(c, &W::p_pr()));
    for c in content {
        dom.remove(c);
        match first {
            Some(first) => dom.add_before_self(first, c),
            None => dom.add(old, c),
        }
    }
    let wanted = bare_properties(dom, last);
    if serialized(dom, &wanted) != serialized(dom, &bare_properties(dom, old)) {
        track_properties(dom, old, stamp, |dom| {
            let ppr = paragraph_properties(dom, old);
            for child in dom.elements(ppr, None) {
                if !dom.name_is(child, &W::r_pr())
                    && !dom.name_is(child, &W::sect_pr())
                    && !dom.name_is(child, &W::p_pr_change())
                {
                    dom.remove(child);
                }
            }
            for &child in &wanted {
                dom.remove(child);
                insert_ppr_child(dom, ppr, child);
            }
        });
    }
    let wanted = mark_formatting(dom, last);
    let had = mark_formatting(dom, old);
    if serialized(dom, &wanted) != serialized(dom, &had) {
        let rpr = mark_properties(dom, old);
        // A record already there holds the mark's original formatting.
        let change = dom.element(rpr, &W::r_pr_change());
        let before = dom.new_element(W::r_pr());
        for &child in &had {
            dom.remove(child);
            dom.add(before, child);
        }
        for child in wanted {
            dom.remove(child);
            match change {
                Some(change) => dom.add_before_self(change, child),
                None => dom.add(rpr, child),
            }
        }
        if change.is_none() {
            let record = stamp.container(dom, W::r_pr_change());
            dom.add(record, before);
            dom.add(rpr, record);
        }
    }
    // `old` takes `last`'s place when `last` follows it; placed before
    // paragraphs the plan deletes, `last` leaves `old` where it is, after
    // them (codex on #1670, round 8).
    if precedes(dom, old, last) {
        dom.remove(old);
        dom.add_after_self(last, old);
    }
    dom.remove(last);
}

/// The container-final paragraph that `r` places a block after, if any.
fn appended_final(tx: &Transaction<'_>, r: &Resolved) -> Option<usize> {
    let (Resolved::InsertParagraph {
        anchor,
        side: Side::After,
        ..
    }
    | Resolved::InsertTable {
        anchor,
        side: Side::After,
        ..
    }) = r
    else {
        return None;
    };
    let node = tx.paragraph_nodes[*anchor];
    (tx.final_paragraph(node) == Some(node)).then_some(*anchor)
}

/// The operation whose block lands last after the container-final
/// `para`: blocks placed after one paragraph follow each other in plan
/// order.
fn last_appended(tx: &Transaction<'_>, para: usize) -> Option<usize> {
    tx.resolved
        .iter()
        .filter(|(_, r)| appended_final(tx, r) == Some(para))
        .map(|(op, _)| *op)
        .last()
}

/// `last`, the new paragraph placed last after the container-final `old`,
/// takes the final mark: its inserted mark goes, and `old`'s mark is the
/// inserted one. Rejected, `old`'s words join `last`'s mark, so that mark
/// records `old`'s properties and mark formatting where they differ from
/// its own; a change record `old` already holds moves to it (`check`
/// refuses, under keep, the plans whose new paragraph that record would
/// misdescribe).
fn rotate_final_mark(dom: &mut Dom, old: NodeId, last: NodeId, stamp: &mut Stamp<'_>) {
    if let Some(inserted) = mark_revision(dom, last, &["ins"]) {
        dom.remove(inserted);
    }
    let had = bare_properties(dom, old);
    if let Some(change) = paragraph_format_change(dom, old) {
        dom.remove(change);
        let ppr = paragraph_properties(dom, last);
        insert_ppr_child(dom, ppr, change);
    } else if serialized(dom, &bare_properties(dom, last)) != serialized(dom, &had) {
        let before = dom.new_element(W::p_pr());
        for child in had {
            let copy = dom.clone_subtree(child);
            dom.add(before, copy);
        }
        let record = stamp.container(dom, W::p_pr_change());
        dom.add(record, before);
        let ppr = paragraph_properties(dom, last);
        insert_ppr_child(dom, ppr, record);
    }
    let had = mark_formatting(dom, old);
    let ppr = paragraph_properties(dom, old);
    let change = dom
        .element(ppr, &W::r_pr())
        .and_then(|rpr| dom.element(rpr, &W::r_pr_change()));
    if let Some(change) = change {
        dom.remove(change);
        let rpr = mark_properties(dom, last);
        dom.add(rpr, change);
    } else if serialized(dom, &mark_formatting(dom, last)) != serialized(dom, &had) {
        let before = dom.new_element(W::r_pr());
        for child in had {
            let copy = dom.clone_subtree(child);
            dom.add(before, copy);
        }
        let record = stamp.container(dom, W::r_pr_change());
        dom.add(record, before);
        let rpr = mark_properties(dom, last);
        dom.add(rpr, record);
    }
    mark(dom, old, W::ins(), stamp);
}

/// Whether `a` comes before `b` in document order.
fn precedes(dom: &Dom, a: NodeId, b: NodeId) -> bool {
    let chain = dom.ancestors_and_self(a, None);
    dom.ancestors_and_self(b, None)
        .into_iter()
        .any(|up| dom.nodes_before_self(up).iter().any(|n| chain.contains(n)))
}

/// `paragraph`'s properties without the mark's own (`w:rPr`), its section
/// or a change record; none at all reads as an empty `w:pPr`.
fn bare_properties(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .map(|ppr| dom.elements(ppr, None))
        .unwrap_or_default()
        .into_iter()
        .filter(|&child| {
            !dom.name_is(child, &W::r_pr())
                && !dom.name_is(child, &W::sect_pr())
                && !dom.name_is(child, &W::p_pr_change())
        })
        .collect()
}

/// The formatting of `paragraph`'s mark (`w:pPr/w:rPr`), without its
/// revisions or a change record.
fn mark_formatting(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .map(|rpr| dom.elements(rpr, None))
        .unwrap_or_default()
        .into_iter()
        .filter(|&child| !is_one_of(dom, child, &MARK_RECORDS))
        .collect()
}

/// What a paragraph mark's `w:rPr` holds besides its formatting.
const MARK_RECORDS: [&str; 5] = ["ins", "del", "moveFrom", "moveTo", "rPrChange"];

/// `children` as XML, to compare property sets.
fn serialized(dom: &Dom, children: &[NodeId]) -> Vec<String> {
    children.iter().map(|&c| dom.serialize_element(c)).collect()
}

/// Wrap a new paragraph's content in one `w:ins` and mark it inserted.
fn insert_paragraph_content(dom: &mut Dom, paragraph: NodeId, stamp: &mut Stamp<'_>) {
    let content: Vec<NodeId> = dom
        .elements(paragraph, None)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    if let Some(&first) = content.first() {
        let ins = stamp.container(dom, W::ins());
        dom.add_before_self(first, ins);
        for c in content {
            dom.remove(c);
            dom.add(ins, c);
        }
    }
    mark(dom, paragraph, W::ins(), stamp);
}

/// Mark every row of a new table inserted (`w:trPr/w:ins`, after the row's
/// other properties) and every cell paragraph's runs and mark inserted.
fn insert_table_content(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>) {
    for row in dom.elements(table, Some(&W::tr())) {
        let properties = match dom.element(row, &W::tr_pr()) {
            Some(properties) => properties,
            None => {
                let properties = dom.new_element(W::tr_pr());
                match dom.element(row, &W::name("tblPrEx")) {
                    Some(exceptions) => dom.add_after_self(exceptions, properties),
                    None => dom.add_first(row, properties),
                }
                properties
            }
        };
        let ins = stamp.container(dom, W::ins());
        dom.add(properties, ins);
        for paragraph in dom.descendants(row, Some(&W::p())) {
            insert_paragraph_content(dom, paragraph, stamp);
        }
    }
}

/// `structural::separate`, with each paragraph it adds marked inserted.
fn separate_tracked(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>) {
    let parent = dom.parent(table);
    let before: Vec<NodeId> = parent
        .map(|p| dom.elements(p, Some(&W::p())))
        .unwrap_or_default();
    structural::separate(dom, table);
    if let Some(parent) = parent {
        for paragraph in dom.elements(parent, Some(&W::p())) {
            if !before.contains(&paragraph) {
                mark(dom, paragraph, W::ins(), stamp);
            }
        }
    }
}

/// Run `change` on `paragraph`'s properties and track it: a `w:pPrChange`
/// holding the old `w:pPr`, unless the plan already tracked a change there
/// (that one holds the original).
fn track_properties(
    dom: &mut Dom,
    paragraph: NodeId,
    stamp: &mut Stamp<'_>,
    change: impl FnOnce(&mut Dom),
) {
    let tracked = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
        .is_some();
    let old = (!tracked).then(|| {
        let old = match dom.element(paragraph, &W::p_pr()) {
            Some(ppr) => dom.clone_subtree(ppr),
            None => dom.new_element(W::p_pr()),
        };
        for child in dom.elements(old, None) {
            if dom.name_is(child, &W::r_pr())
                || dom.name_is(child, &W::sect_pr())
                || dom.name_is(child, &W::p_pr_change())
            {
                dom.remove(child);
            }
        }
        old
    });
    change(dom);
    if let Some(old) = old {
        let ppr = paragraph_properties(dom, paragraph);
        let record = stamp.container(dom, W::p_pr_change());
        dom.add(record, old);
        insert_ppr_child(dom, ppr, record);
    }
}

/// Format `range` of `paragraph` as `Transaction::apply` does, each changed
/// run's old properties in a `w:rPrChange`, the last child of its `w:rPr`.
fn track_run_format(
    dom: &mut Dom,
    paragraph: NodeId,
    (start, end): (usize, usize),
    format: &super::RunFormat,
    stamp: &mut Stamp<'_>,
) {
    for run in runs_in_range(dom, paragraph, start, end, &mut stamp.ids) {
        let rpr = match dom.element(run, &W::r_pr()) {
            Some(rpr) => rpr,
            None => {
                let rpr = dom.new_element(W::r_pr());
                dom.add_first(run, rpr);
                rpr
            }
        };
        // `check` refuses formatting over a source record, so a record here
        // is an earlier operation's: it already holds the run's old
        // properties, and a second one would copy it, id and all (#1656).
        if dom.element(rpr, &W::r_pr_change()).is_some() {
            apply_run_format(dom, rpr, format);
            continue;
        }
        let old = dom.clone_subtree(rpr);
        apply_run_format(dom, rpr, format);
        if dom.serialize_element(old) == dom.serialize_element(rpr) {
            if dom.elements(rpr, None).is_empty() {
                dom.remove(rpr);
            }
            continue;
        }
        let record = stamp.container(dom, W::r_pr_change());
        dom.add(record, old);
        dom.add(rpr, record);
    }
}

/// Reply markers beside their comment's, and deleted comments' markers
/// removed, as `Transaction::apply` does last.
fn thread_markers(t: &mut Transaction<'_>, comment_ids: &BTreeMap<usize, u32>) {
    let roots: Vec<NodeId> = t.stories.iter().map(|s| s.root).collect();
    for (i, r) in &t.resolved {
        let Resolved::Thread { op, .. } = r else {
            continue;
        };
        match op {
            ThreadOp::Reply { parent, .. } => {
                if let Some(&id) = comment_ids.get(i) {
                    place_reply_markers(&mut t.opened.dom, &roots, *parent, id);
                }
            }
            ThreadOp::Delete { id } => {
                let gone: Vec<String> = t
                    .family
                    .as_ref()
                    .map(|family| family.with_replies(*id))
                    .unwrap_or_default()
                    .iter()
                    .map(u32::to_string)
                    .collect();
                remove_comment_markers(&mut t.opened.dom, &roots, &gone);
            }
            ThreadOp::Resolve { .. } | ThreadOp::Edit { .. } => {}
        }
    }
}

/// Reuse the word diff for default replacements inside an insertion. For
/// example, `month.` -> `month, in cash.` only inserts `, in cash` and
/// preserves both the word and period. A `whole` replacement bypasses this.
fn insertion_replacement(dom: &Dom, paragraph: NodeId, edit: &ScheduledEdit) -> Vec<ScheduledEdit> {
    // A formatted replacement formats every word of it, the ones it keeps
    // too, as the clean copy does: one deletion, one insertion.
    if edit.format.is_some() {
        return vec![edit.clone()];
    }
    let Some(pieces) = insertion_pieces(dom, paragraph, edit.start, edit.end, &edit.replacement)
    else {
        return vec![edit.clone()];
    };
    pieces
        .into_iter()
        .map(|(start, end, replacement)| ScheduledEdit {
            start,
            end,
            replacement,
            ..edit.clone()
        })
        .collect()
}

/// The word-level edits, in projection offsets, that make `start..end`
/// read as `replacement` when the range lies in one insertion; `None` when
/// it reaches outside one, or the word diff cannot write it exactly.
pub(super) fn insertion_pieces(
    dom: &Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    replacement: &str,
) -> Option<Vec<(usize, usize, String)>> {
    let projection = project_paragraph(dom, paragraph);
    let touched: Vec<_> = projection
        .segments
        .iter()
        .filter(|s| s.start < end && start < s.end)
        .collect();
    let ins = touched
        .first()
        .and_then(|s| super::editable_insertion(dom, &s.piece));
    if ins.is_none()
        || !touched
            .iter()
            .all(|s| super::editable_insertion(dom, &s.piece) == ins)
    {
        return None;
    }
    let old = &projection.text[start..end];
    let ranges = super::rewrite::rewrite_ranges(old, replacement);
    let mut rewritten = old.to_string();
    for (from, to, words) in ranges.iter().rev() {
        rewritten.replace_range(*from..*to, words);
    }
    // The rewrite diff normalizes whitespace. An exact replacement still
    // has to write every requested byte, including whitespace-only edits.
    if rewritten != replacement {
        return None;
    }
    Some(
        ranges
            .into_iter()
            .map(|(from, to, words)| (start + from, start + to, words))
            .collect(),
    )
}

/// Emit one text edit: the range's runs in `w:del`, then the replacement in
/// a `w:ins` run formatted like the first deleted run, or like the run the
/// insertion attaches to. `empty` places the text of a paragraph that had
/// none.
fn emit_text(
    dom: &mut Dom,
    paragraph: NodeId,
    edit: &ScheduledEdit,
    stamp: &mut Stamp<'_>,
    empty: Option<&mut super::EmptyPlacement>,
) {
    if let Some(placement) = empty {
        // Typed into an empty paragraph: a run formatted as the mark, in
        // our insertion, at its edge. Placed in reverse order, no text is
        // before it yet, so its words start at 0.
        if edit.replacement.is_empty() {
            return;
        }
        let ins = stamp.container(dom, W::ins());
        placement.place(dom, paragraph, ins, edit.attach_before);
        let run = super::mark_run(dom, paragraph);
        dom.add(ins, run);
        super::write_mark_run_text(dom, run, &edit.replacement);
        if let Some(format) = &edit.format {
            super::format_range(
                dom,
                paragraph,
                0,
                edit.replacement.len(),
                format,
                &mut stamp.ids,
            );
        }
        return;
    }
    let projection = project_paragraph(dom, paragraph);
    let touched: Vec<_> = if edit.start == edit.end {
        attach_segment(&projection, edit.start, edit.attach_before)
            .into_iter()
            .collect()
    } else {
        projection
            .segments
            .iter()
            .filter(|s| s.start < edit.end && edit.start < s.end)
            .collect()
    };
    // Word treats typing in one's own insertion as a correction to that
    // insertion, preserving its original id and timestamp.
    if let Some(ins) = touched
        .first()
        .and_then(|s| super::editable_insertion(dom, &s.piece))
        && dom.attribute(ins, &W::author()) == Some(stamp.author)
        && touched
            .iter()
            .all(|s| super::editable_insertion(dom, &s.piece) == Some(ins))
    {
        apply_text_edit(
            dom,
            &projection,
            edit.start,
            edit.end,
            &edit.replacement,
            edit.attach_before,
        );
        super::clean_new_words(
            dom,
            paragraph,
            edit.start,
            edit.start + edit.replacement.len(),
            &mut stamp.ids,
        );
        if let Some(format) = &edit.format {
            super::format_range(
                dom,
                paragraph,
                edit.start,
                edit.start + edit.replacement.len(),
                format,
                &mut stamp.ids,
            );
        }
        remove_empty_insertion(dom, ins);
        return;
    }
    let (start, end) = (edit.start, edit.end);
    let mut replacement = edit.replacement.as_str();
    // This edit's own deletions, in document order.
    let mut ours = Vec::new();
    // `None` places the insertion right after `anchor`, `Some(true)` before.
    let (template, anchor, before) = if start < end {
        let runs = isolate_range(dom, paragraph, (start, end), &mut stamp.ids);
        let mut groups: Vec<Vec<NodeId>> = Vec::new();
        for &run in &runs {
            match groups.last_mut() {
                Some(group) if dom.next_element(*group.last().expect("nonempty")) == Some(run) => {
                    group.push(run);
                }
                _ => groups.push(vec![run]),
            }
        }
        let mut last_del = None;
        for (index, group) in groups.into_iter().enumerate() {
            if let Some(ins) = dom.parent(group[0]).filter(|&n| dom.name_is(n, &W::ins()))
                && dom.attribute(ins, &W::author()) == Some(stamp.author)
            {
                let projection = project_paragraph(dom, paragraph);
                let pieces: Vec<_> = projection
                    .segments
                    .iter()
                    .filter(|s| group.contains(&super::run_of(&s.piece)))
                    .collect();
                if let (Some(first), Some(last)) = (pieces.first(), pieces.last()) {
                    let text = if index == 0 { replacement } else { "" };
                    apply_text_edit(dom, &projection, first.start, last.end, text, true);
                    super::clean_new_words(
                        dom,
                        paragraph,
                        first.start,
                        first.start + text.len(),
                        &mut stamp.ids,
                    );
                    if index == 0 {
                        if let Some(format) = &edit.format {
                            super::format_range(
                                dom,
                                paragraph,
                                first.start,
                                first.start + text.len(),
                                format,
                                &mut stamp.ids,
                            );
                        }
                        replacement = "";
                    }
                }
                remove_empty_insertion(dom, ins);
                continue;
            }
            let del = stamp.container(dom, W::del());
            dom.add_before_self(group[0], del);
            for run in group {
                dom.remove(run);
                dom.add(del, run);
                rename_deleted_text(dom, run);
            }
            ours.push(del);
            last_del = Some(del);
        }
        (runs.first().copied(), last_del, false)
    } else {
        let projection = project_paragraph(dom, paragraph);
        let Some(seg) = attach_segment(&projection, start, edit.attach_before).cloned() else {
            return;
        };
        split_run_at(dom, &seg, start, &mut stamp.ids);
        let projection = project_paragraph(dom, paragraph);
        let text_piece = |s: &crate::inspect::Segment| {
            if !s.direct && super::editable_insertion(dom, &s.piece).is_none() {
                return None;
            }
            Some(match s.piece {
                Piece::Text { t, run } => (t, run),
                Piece::Glyph { element, run } => (element, run),
            })
        };
        let left = projection
            .segments
            .iter()
            .rev()
            .find(|s| s.end == start && s.start < s.end)
            .and_then(text_piece);
        let right = projection
            .segments
            .iter()
            .find(|s| s.start == start && s.start < s.end)
            .and_then(text_piece);
        let (chosen, before) = match (left, right) {
            (Some(l), _) if edit.attach_before => (l, false),
            (_, Some(r)) => (r, true),
            (Some(l), None) => (l, false),
            (None, None) => return,
        };
        let (t, run) = chosen;
        isolate(dom, run, t, &mut stamp.ids);
        let anchor = insertion_boundary_anchor(dom, run, before);
        (Some(run), Some(anchor), before)
    };
    // A deletion stays inside the source insertion, which stays whole, as
    // Word leaves it: only the editor's words split it. Range endpoints stay
    // beside their words; the remaining children follow the editor's words.
    let split = anchor
        .filter(|_| !replacement.is_empty())
        .and_then(|anchor| {
            let ins = dom.parent(anchor).filter(|&n| dom.name_is(n, &W::ins()))?;
            let children = dom.elements(ins, None);
            let at = children.iter().position(|&n| n == anchor)? + usize::from(!before);
            let at = insertion_split_boundary(dom, &children, at, before);
            if at == 0 {
                // There is no first piece to keep. The complete source
                // insertion becomes the newly numbered trailing piece.
                return Some((ins, true, Some(ins)));
            }
            let tail = if at < children.len() {
                let tail = dom.clone_subtree(ins);
                for child in dom.nodes(tail) {
                    dom.remove(child);
                }
                for &child in &children[at..] {
                    dom.remove(child);
                    dom.add(tail, child);
                }
                Some(tail)
            } else {
                None
            };
            Some((ins, false, tail))
        });
    let (anchor, before) = split.map_or((anchor, before), |(ins, before, _)| (Some(ins), before));
    // New words meet no range they were not typed in: past the openers
    // before them, past the closers after them (#1156). A replacement takes
    // the place of words its ranges cover, and stays beside its deletion,
    // past the end of a comment its deletion holds whole: Accept removes
    // that comment, and the new words must not be in it.
    let anchor = anchor.map(|anchor| {
        if start == end {
            outside_ranges(dom, anchor, before)
        } else if before || replacement.is_empty() {
            anchor
        } else {
            let (last, references) = past_deleted_comments(dom, anchor, &ours);
            // Their references go with the deleted text, as deleting the
            // whole commented text writes them.
            let mut last = last;
            for run in references {
                let del = stamp.container(dom, W::del());
                dom.add_before_self(run, del);
                dom.remove(run);
                dom.add(del, run);
                if last == run {
                    last = del;
                }
            }
            last
        }
    });
    let mut last = anchor;
    if !replacement.is_empty() {
        let run = dom.new_element(W::r());
        let rpr = template
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|rpr| clean_rpr(dom, rpr));
        let rpr = match (rpr, &edit.format) {
            (Some(rpr), _) => Some(rpr),
            (None, Some(_)) => Some(dom.new_element(W::r_pr())),
            (None, None) => None,
        };
        if let Some(rpr) = rpr {
            if let Some(format) = &edit.format {
                apply_run_format(dom, rpr, format);
            }
            if dom.elements(rpr, None).is_empty() {
                dom.remove(rpr);
            } else {
                dom.add(run, rpr);
            }
        }
        let text = dom.new_element(W::t());
        set_text(dom, text, replacement);
        dom.add(run, text);
        let ins = stamp.container(dom, W::ins());
        dom.add(ins, run);
        match anchor {
            Some(anchor) if before => dom.add_before_self(anchor, ins),
            Some(anchor) => dom.add_after_self(anchor, ins),
            None => dom.add(paragraph, ins),
        }
        last = Some(ins);
    }
    if let Some((source, _, Some(tail))) = split {
        if let Some(origin) = stamp.insertions.get(&source).cloned() {
            stamp.insertions.insert(tail, origin);
        }
        // Allocate after the deletion and editor insertion, using the same
        // package-wide allocator. The source attribution stays on the tail.
        let id = stamp.ids.allocate();
        dom.set_attribute_value(tail, &W::id(), Some(&id.to_string()));
        dom.remove(tail);
        dom.add_after_self(last.expect("a split has a first piece"), tail);
    }
}

/// The sibling an insertion at `run`'s edge goes beside: past the range
/// markers that open before the run (`before`) or close after it, so those
/// ranges keep their source endpoints. `run` itself when none sit there.
pub(super) fn insertion_boundary_anchor(dom: &Dom, run: NodeId, before: bool) -> NodeId {
    let Some(parent) = dom.parent(run) else {
        return run;
    };
    let children = dom.elements(parent, None);
    let Some(index) = children.iter().position(|&n| n == run) else {
        return run;
    };
    let at = insertion_split_boundary(dom, &children, index + usize::from(!before), before);
    children[if before { at } else { at - 1 }]
}

/// The range openers an insertion can hold: they travel with the words to
/// their right.
pub(super) const RANGE_STARTS: [&str; 5] = [
    "bookmarkStart",
    "commentRangeStart",
    "permStart",
    "moveFromRangeStart",
    "moveToRangeStart",
];
/// The range closers that travel with the words to their left; a comment's
/// end also takes its reference, so it is read on its own.
pub(super) const RANGE_ENDS: [&str; 4] = [
    "bookmarkEnd",
    "permEnd",
    "moveFromRangeEnd",
    "moveToRangeEnd",
];

/// A comment range's closer: it travels with the words to its left, and
/// takes its reference along.
pub(super) const COMMENT_RANGE_END: &str = "commentRangeEnd";

/// Whether `node` is a `w:` element named one of `locals`.
fn is_one_of(dom: &Dom, node: NodeId, locals: &[&str]) -> bool {
    dom.name(node)
        .is_some_and(|n| n.namespace_name() == W::URI && locals.contains(&n.local_name()))
}

/// The sibling of `anchor` that new words go `before` (or after) so they
/// join no range they meet: the first of the openers right before
/// `anchor`, or the last of the closers right after it, a comment's end
/// with its reference. `anchor` itself when no marker is in the way. Body-
/// level markers are no siblings of a run and stay where they are.
pub(super) fn outside_ranges(dom: &Dom, anchor: NodeId, before: bool) -> NodeId {
    let Some(parent) = dom.parent(anchor) else {
        return anchor;
    };
    let children = dom.elements(parent, None);
    let Some(index) = children.iter().position(|&n| n == anchor) else {
        return anchor;
    };
    let at = insertion_split_boundary(dom, &children, index + usize::from(!before), before);
    if before {
        children[at]
    } else {
        children[at - 1]
    }
}

/// Opening markers travel with the words to their right, closing markers
/// with the words to their left. A comment reference stays with its end.
fn insertion_split_boundary(dom: &Dom, children: &[NodeId], mut at: usize, before: bool) -> usize {
    if before {
        while at > 0 && is_one_of(dom, children[at - 1], &RANGE_STARTS) {
            at -= 1;
        }
        return at;
    }
    let mut closed_comments = BTreeSet::new();
    while let Some(&child) = children.get(at) {
        if dom.name_is(child, &W::name(COMMENT_RANGE_END)) {
            if let Some(id) = dom.attribute(child, &W::id()) {
                closed_comments.insert(id);
            }
        } else if !is_one_of(dom, child, &RANGE_ENDS) {
            if !dom.name_is(child, &W::r()) {
                break;
            }
            let references = dom.elements(child, Some(&W::comment_reference()));
            if references.is_empty()
                || references.iter().any(|&reference| {
                    dom.attribute(reference, &W::id())
                        .is_none_or(|id| !closed_comments.contains(id))
                })
                || dom.elements(child, None).iter().any(|&element| {
                    !dom.name_is(element, &W::r_pr())
                        && !dom.name_is(element, &W::comment_reference())
                })
            {
                break;
            }
        }
        at += 1;
    }
    at
}

/// The last sibling after deletion `del` that closes a comment opened
/// inside the deletions `ours` of one edit, after its first deleted word:
/// its end and the run with its reference (`del` itself when none follows),
/// and those reference runs. A comment opened on kept text, or on the
/// edit's first word (the replaced text is the commented text), keeps the
/// new words, in Word's order.
fn past_deleted_comments(dom: &Dom, del: NodeId, ours: &[NodeId]) -> (NodeId, Vec<NodeId>) {
    let Some(parent) = dom.parent(del) else {
        return (del, Vec::new());
    };
    let children = dom.elements(parent, None);
    let Some(index) = children.iter().position(|&n| n == del) else {
        return (del, Vec::new());
    };
    let opened_in_deletion = |id: &str| {
        let mut back = children[..index].iter().rev();
        let found = back.by_ref().find_map(|&c| {
            if dom.name_is(c, &W::comment_range_start()) && dom.attribute(c, &W::id()) == Some(id) {
                Some(true)
            } else if ours.contains(&c) || is_one_of(dom, c, &RANGE_STARTS) {
                None
            } else {
                Some(false)
            }
        }) == Some(true);
        found
            && back
                .find(|&&c| !is_one_of(dom, c, &RANGE_STARTS))
                .is_some_and(|c| ours.contains(c))
    };
    let mut closed = BTreeSet::new();
    let mut last = del;
    let mut references_passed = Vec::new();
    for &child in &children[index + 1..] {
        let is_end = dom.name_is(child, &W::name(COMMENT_RANGE_END));
        let passes = if is_end {
            dom.attribute(child, &W::id())
                .filter(|id| opened_in_deletion(id))
                .is_some_and(|id| closed.insert(id))
        } else {
            let references = dom.elements(child, Some(&W::comment_reference()));
            dom.name_is(child, &W::r())
                && !references.is_empty()
                && references.iter().all(|&r| {
                    dom.attribute(r, &W::id())
                        .is_some_and(|id| closed.contains(id))
                })
                && dom
                    .elements(child, None)
                    .iter()
                    .all(|&e| dom.name_is(e, &W::r_pr()) || dom.name_is(e, &W::comment_reference()))
        };
        if !passes {
            break;
        }
        if !is_end {
            references_passed.push(child);
        }
        last = child;
    }
    (last, references_passed)
}

/// Our insertion `ours` right after deletion `del`. A deletion inside an
/// insertion splits it there, as typing does in Word: ours follows the
/// piece that holds the deletion, and the rest follows ours under a new id
/// with the source attribution.
fn place_beside(dom: &mut Dom, del: NodeId, ours: NodeId, stamp: &mut Stamp<'_>) {
    let Some(host) = dom.parent(del).filter(|&p| dom.name_is(p, &W::ins())) else {
        dom.add_after_self(del, ours);
        return;
    };
    let children = dom.elements(host, None);
    let at = children
        .iter()
        .position(|&n| n == del)
        .map_or(children.len(), |i| i + 1);
    let at = insertion_split_boundary(dom, &children, at, false);
    dom.add_after_self(host, ours);
    if at < children.len() {
        let tail = dom.clone_subtree(host);
        for child in dom.nodes(tail) {
            dom.remove(child);
        }
        for &child in &children[at..] {
            dom.remove(child);
            dom.add(tail, child);
        }
        if let Some(origin) = stamp.insertions.get(&host).cloned() {
            stamp.insertions.insert(tail, origin);
        }
        let id = stamp.ids.allocate();
        dom.set_attribute_value(tail, &W::id(), Some(&id.to_string()));
        dom.add_after_self(ours, tail);
    }
}

/// Empty run shells must not keep an otherwise deleted own insertion alive.
pub(super) fn remove_empty_insertion(dom: &mut Dom, ins: NodeId) {
    for run in dom.elements(ins, Some(&W::r())) {
        if dom.elements(run, None).iter().all(|&c| {
            dom.name_is(c, &W::r_pr()) || (dom.name_is(c, &W::t()) && dom.value(c).is_empty())
        }) {
            dom.remove(run);
        }
    }
    if dom.elements(ins, None).is_empty() {
        dom.remove(ins);
    }
}

/// Split runs so each text or glyph of `[start, end)` sits alone in its
/// run (beside the run's `w:rPr`); return the runs in document order.
fn isolate_range(
    dom: &mut Dom,
    paragraph: NodeId,
    (start, end): (usize, usize),
    ids: &mut RevisionCounter,
) -> Vec<NodeId> {
    for at in [start, end] {
        let projection = project_paragraph(dom, paragraph);
        if let Some(seg) = projection
            .segments
            .iter()
            .find(|s| s.start < at && at < s.end)
            .cloned()
        {
            split_run_at(dom, &seg, at, ids);
        }
    }
    loop {
        let projection = project_paragraph(dom, paragraph);
        let pieces: Vec<(NodeId, NodeId)> = projection
            .segments
            .iter()
            .filter(|s| s.start >= start && s.end <= end && s.start < s.end)
            .map(|s| match s.piece {
                Piece::Text { t, run } => (t, run),
                Piece::Glyph { element, run } => (element, run),
            })
            .collect();
        match pieces.iter().find(|&&(t, run)| !alone(dom, run, t)) {
            Some(&(t, run)) => isolate(dom, run, t, ids),
            None => {
                let mut runs: Vec<NodeId> = Vec::new();
                for (_, run) in pieces {
                    if !runs.contains(&run) {
                        runs.push(run);
                    }
                }
                return runs;
            }
        }
    }
}

/// Whether `t` is the only child of `run` besides its `w:rPr`.
fn alone(dom: &Dom, run: NodeId, t: NodeId) -> bool {
    dom.elements(run, None)
        .into_iter()
        .all(|c| c == t || dom.name_is(c, &W::r_pr()))
}

/// Move the children of `run` before `t` into a copy of the run placed
/// before it, and those after `t` into a copy placed after it; each copy
/// keeps the run's `w:rPr`. The first fragment keeps the ids of the run's
/// formatting history; the others take fresh ids from `ids` (#1656).
fn isolate(dom: &mut Dom, run: NodeId, t: NodeId, ids: &mut RevisionCounter) {
    let children = dom.elements(run, None);
    let Some(index) = children.iter().position(|&c| c == t) else {
        return;
    };
    let content = |dom: &Dom, c: NodeId| !dom.name_is(c, &W::r_pr());
    for (keep_before, place_before) in [(true, true), (false, false)] {
        // `keep_before`: the copy keeps the children before `t`.
        let side: Vec<NodeId> = children
            .iter()
            .enumerate()
            .filter(|&(i, &c)| (if keep_before { i < index } else { i > index }) && content(dom, c))
            .map(|(_, &c)| c)
            .collect();
        if side.is_empty() {
            continue;
        }
        let copy = dom.clone_subtree(run);
        for (i, c) in dom.elements(copy, None).into_iter().enumerate() {
            let kept = if keep_before { i < index } else { i > index };
            if !kept && content(dom, c) {
                dom.remove(c);
            }
        }
        if place_before {
            dom.add_before_self(run, copy);
            // The copy is now the first fragment: the original's record
            // becomes the copy.
            super::refresh_run_history(dom, run, ids);
        } else {
            super::refresh_run_history(dom, copy, ids);
            dom.add_after_self(run, copy);
        }
    }
    // Both copies need the original child indexes. Removing the left
    // side before cloning the right shifts those indexes and loses text.
    for child in children {
        if child != t && content(dom, child) {
            dom.remove(child);
        }
    }
}

/// `w:t` to `w:delText` and `w:instrText` to `w:delInstrText` in a run that
/// now sits inside a `w:del`.
fn rename_deleted_text(dom: &mut Dom, run: NodeId) {
    for child in dom.elements(run, None) {
        if dom.name_is(child, &W::t()) {
            dom.set_name(child, W::del_text());
        } else if dom.name_is(child, &W::instr_text()) {
            dom.set_name(child, W::del_instr_text());
        }
    }
}

/// Every run of `container`, at any depth of hyperlinks, content controls,
/// smart tags and simple fields, inside a `w:del`; consecutive runs share one.
/// Under `keep` (#1136), another author's insertion keeps its words with
/// the deletion nested in it, the plan author's own pending insertion goes,
/// and a deletion stays as it is.
fn delete_runs(dom: &mut Dom, container: NodeId, stamp: &mut Stamp<'_>) {
    let mut group: Vec<NodeId> = Vec::new();
    for child in dom.elements(container, None) {
        if dom.name_is(child, &W::r()) {
            group.push(child);
            continue;
        }
        wrap_deleted(dom, std::mem::take(&mut group), stamp);
        let Some(name) = dom.name(child).filter(|n| n.namespace_name() == W::URI) else {
            continue;
        };
        match name.local_name() {
            "hyperlink" | "smartTag" | "customXml" | "dir" | "bdo" | "fldSimple" => {
                delete_runs(dom, child, stamp);
            }
            "sdt" => {
                if let Some(content) = dom.element(child, &W::sdt_content()) {
                    delete_runs(dom, content, stamp);
                }
            }
            "ins" if dom.attribute(child, &W::author()) == Some(stamp.author) => {
                remove_own_insertion(dom, child, stamp);
            }
            "ins" => delete_runs(dom, child, stamp),
            _ => {}
        }
    }
    wrap_deleted(dom, group, stamp);
}

/// What a run may hold and still go outright: text Word types, nothing a
/// field or a note needs.
const TYPED: [&str; 9] = [
    "rPr",
    "t",
    "tab",
    "br",
    "cr",
    "sym",
    "noBreakHyphen",
    "softHyphen",
    "lastRenderedPageBreak",
];

/// The plan author's own pending insertion, deleted: its typed runs go
/// outright, a run holding a field character or a reference is deleted
/// inside it so no field or note loses a part, and the markers it held
/// (bookmarks, comment ranges) stay where they were.
fn remove_own_insertion(dom: &mut Dom, ins: NodeId, stamp: &mut Stamp<'_>) {
    for run in dom.elements(ins, Some(&W::r())) {
        if dom
            .elements(run, None)
            .into_iter()
            .all(|c| is_one_of(dom, c, &TYPED))
        {
            dom.remove(run);
        }
    }
    delete_runs(dom, ins, stamp);
    let left = dom.elements(ins, None);
    if left.iter().all(|&c| {
        is_one_of(dom, c, &RANGE_STARTS)
            || is_one_of(dom, c, &RANGE_ENDS)
            || is_one_of(dom, c, &[COMMENT_RANGE_END])
    }) {
        for marker in left {
            dom.remove(marker);
            dom.add_before_self(ins, marker);
        }
        dom.remove(ins);
    }
}

fn wrap_deleted(dom: &mut Dom, group: Vec<NodeId>, stamp: &mut Stamp<'_>) {
    let Some(&first) = group.first() else {
        return;
    };
    let del = stamp.container(dom, W::del());
    dom.add_before_self(first, del);
    for run in group {
        dom.remove(run);
        dom.add(del, run);
        rename_deleted_text(dom, run);
    }
}

/// Revision elements of a paragraph mark that delete it.
const DELETED: [&str; 2] = ["del", "moveFrom"];
/// Every revision element a paragraph mark can carry.
pub(super) const TRACKED: [&str; 4] = ["ins", "del", "moveFrom", "moveTo"];

/// Whether `paragraph`'s mark (`w:pPr/w:rPr`) carries one of `locals`.
fn mark_has(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> bool {
    mark_revision(dom, paragraph, locals).is_some()
}

/// The revision among `locals` that `paragraph`'s mark (`w:pPr/w:rPr`)
/// carries.
fn mark_revision(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> Option<NodeId> {
    let rpr = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))?;
    dom.elements(rpr, None).into_iter().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && locals.contains(&n.local_name()))
    })
}

/// `paragraph`'s tracked properties change (`w:pPr/w:pPrChange`).
fn paragraph_format_change(dom: &Dom, paragraph: NodeId) -> Option<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
}

/// The change record of the container-final `paragraph` that operation
/// `op`'s new paragraph would have to replace when it takes the final mark
/// (`keep_final_mark`): its `w:pPrChange` when the new properties differ,
/// or its mark's `w:rPrChange` when the mark's formatting goes (a new
/// paragraph's mark carries none, as `build_paragraph` writes it).
fn record_replaced(tx: &Transaction<'_>, paragraph: NodeId, op: usize) -> Option<NodeId> {
    let dom = &tx.opened.dom;
    let Some((
        _,
        Resolved::InsertParagraph {
            like, style, toc, ..
        },
    )) = tx.resolved.iter().find(|(i, _)| *i == op)
    else {
        return None;
    };
    let like = tx.paragraph_nodes[*like];
    let style_of = |p: NodeId| {
        dom.element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::p_style()))
            .and_then(|s| dom.attribute(s, &W::val()))
    };
    let unstyled = |p: NodeId| -> Vec<String> {
        let children: Vec<NodeId> = bare_properties(dom, p)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_style()))
            .collect();
        serialized(dom, &children)
    };
    // A table of contents ends on its field paragraph, which has no
    // properties (`toc_paragraphs`).
    let same = match toc {
        Some(_) => style_of(paragraph).is_none() && unstyled(paragraph).is_empty(),
        None => {
            style.as_deref().or(style_of(like)) == style_of(paragraph)
                && unstyled(like) == unstyled(paragraph)
        }
    };
    let properties = paragraph_format_change(dom, paragraph).filter(|_| !same);
    let mark = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .and_then(|rpr| dom.element(rpr, &W::r_pr_change()))
        .filter(|_| !mark_formatting(dom, paragraph).is_empty());
    properties.or(mark)
}

/// The first element `paragraph` holds, at the levels `delete_runs` walks,
/// that the replay cannot delete as a tracked change: one that is no
/// WordprocessingML (an equation, `m:oMath`), or a subdocument (`w:subDoc`,
/// crush on #1670, round 8).
fn untrackable_content(dom: &Dom, container: NodeId) -> Option<NodeId> {
    dom.elements(container, None).into_iter().find_map(|child| {
        let name = dom.name(child)?;
        if name.namespace_name() != W::URI {
            return Some(child);
        }
        match name.local_name() {
            "subDoc" => Some(child),
            "hyperlink" | "smartTag" | "customXml" | "dir" | "bdo" | "fldSimple" | "ins" => {
                untrackable_content(dom, child)
            }
            "sdt" => dom
                .element(child, &W::sdt_content())
                .and_then(|content| untrackable_content(dom, content)),
            _ => None,
        }
    })
}

/// The first move (`w:moveFrom`, `w:moveTo`) `node` holds, in its runs or
/// its mark.
fn moved_in(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.descendants(node, None)
        .into_iter()
        .find(|&n| is_one_of(dom, n, &["moveFrom", "moveTo"]))
}

/// The block right after `node` among its siblings, past the range markers
/// and proofing marks between: a paragraph, a table, a block content
/// control or a custom XML element. Word's Compare deletes the mark of a
/// paragraph deleted before any of them (codex on #1670, round 8).
fn next_block(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.nodes_after_self(node).into_iter().find(|&n| {
        dom.name(n).is_some_and(|name| {
            name.namespace_name() == W::URI
                && matches!(name.local_name(), "p" | "tbl" | "sdt" | "customXml")
        })
    })
}

/// The paragraph right before `node` among its siblings.
fn previous_paragraph(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.nodes_before_self(node)
        .into_iter()
        .rev()
        .find(|&n| dom.is_element(n))
        .filter(|&n| dom.name_is(n, &W::p()))
}

/// `paragraph`'s mark properties (`w:pPr/w:rPr`), created when missing.
fn mark_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    let ppr = paragraph_properties(dom, paragraph);
    match dom.element(ppr, &W::r_pr()) {
        Some(rpr) => rpr,
        None => {
            let rpr = dom.new_element(W::r_pr());
            insert_ppr_child(dom, ppr, rpr);
            rpr
        }
    }
}

/// `paragraph`'s `w:pPr`, created when missing.
fn paragraph_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    }
}

/// Mark `paragraph`'s mark inserted or deleted (`w:pPr/w:rPr/w:ins|w:del`,
/// first in `w:rPr` as the schema orders them, a deletion after an
/// insertion). A mark already deleted stays as it is.
fn mark(dom: &mut Dom, paragraph: NodeId, name: XName, stamp: &mut Stamp<'_>) {
    let deleting = name == W::del();
    if deleting && mark_has(dom, paragraph, &DELETED) {
        return;
    }
    let rpr = mark_properties(dom, paragraph);
    let node = stamp.container(dom, name);
    match dom.element(rpr, &W::ins()) {
        Some(ins) if deleting => dom.add_after_self(ins, node),
        _ => dom.add_first(rpr, node),
    }
}

/// A copy of a run's `w:rPr` for new text: without another revision's
/// format change or mark revisions, whose ids must stay unique.
pub(super) fn clean_rpr(dom: &mut Dom, rpr: NodeId) -> NodeId {
    let copy = dom.clone_subtree(rpr);
    strip_records(dom, copy);
    copy
}

/// Remove the format change and mark revisions from `rpr`.
pub(super) fn strip_records(dom: &mut Dom, rpr: NodeId) {
    for child in dom.elements(rpr, None) {
        if dom.name_is(child, &W::r_pr_change())
            || dom.name_is(child, &W::ins())
            || dom.name_is(child, &W::del())
            || dom.name_is(child, &W::move_from())
            || dom.name_is(child, &W::move_to())
        {
            dom.remove(child);
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    /// A counter whose next id is `next`.
    pub(super) fn counter_at(next: u64) -> RevisionCounter {
        match next.checked_sub(1) {
            Some(below) => RevisionCounter::new(HashSet::from([below])),
            None => RevisionCounter::default(),
        }
    }

    fn paragraph(dom: &mut Dom, xml: &str) -> NodeId {
        let document = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}">{xml}</w:p>"#, W::URI));
        dom.root(document).expect("root")
    }

    #[test]
    fn isolate_moves_neighbours_into_copies() {
        let mut dom = Dom::new();
        let p = paragraph(
            &mut dom,
            "<w:r><w:rPr><w:b/></w:rPr><w:tab/><w:t>a</w:t><w:br/></w:r>",
        );
        let run = dom.elements(p, Some(&W::r()))[0];
        let t = dom.element(run, &W::t()).expect("t");
        isolate(&mut dom, run, t, &mut RevisionCounter::default());
        let runs = dom.elements(p, Some(&W::r()));
        assert_eq!(runs.len(), 3);
        assert!(alone(&dom, runs[1], t));
        for r in runs {
            assert!(dom.element(r, &W::r_pr()).is_some(), "each copy keeps rPr");
        }
        // Already alone: nothing changes.
        let before = dom.elements(p, None).len();
        isolate(&mut dom, run, t, &mut RevisionCounter::default());
        assert_eq!(dom.elements(p, None).len(), before);
    }

    #[test]
    fn a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled() {
        let mut dom = Dom::new();
        let p = paragraph(&mut dom, "<w:r><w:t>x</w:t></w:r>");
        let mut stamp = Stamp {
            author: "Me",
            date: "2026-10-02T00:00:00Z",
            ids: counter_at(5),
            insertions: HashMap::new(),
        };
        mark(&mut dom, p, W::ins(), &mut stamp);
        mark(&mut dom, p, W::del(), &mut stamp);
        mark(&mut dom, p, W::del(), &mut stamp);
        let rpr = dom
            .element(dom.element(p, &W::p_pr()).expect("pPr"), &W::r_pr())
            .expect("rPr");
        let names: Vec<_> = dom
            .elements(rpr, None)
            .into_iter()
            .map(|c| dom.name(c).expect("name").local_name().to_string())
            .collect();
        assert_eq!(names, ["ins", "del"]);
        assert_eq!(stamp.ids.own(), &BTreeSet::from([5, 6]));
        assert!(mark_has(&dom, p, &DELETED));
        assert!(moved_in(&dom, dom.elements(p, Some(&W::r()))[0]).is_none());
    }

    #[test]
    fn delete_runs_groups_siblings_and_recurses_into_hyperlinks() {
        let mut dom = Dom::new();
        let p = paragraph(
            &mut dom,
            r#"<w:r><w:t>a</w:t></w:r><w:r><w:t>b</w:t></w:r><w:bookmarkStart w:id="0" w:name="x"/><w:hyperlink><w:r><w:instrText>c</w:instrText></w:r></w:hyperlink><w:sdt><w:sdtContent><w:r><w:t>d</w:t></w:r></w:sdtContent></w:sdt>"#,
        );
        let mut stamp = Stamp {
            author: "Me",
            date: "d",
            ids: counter_at(0),
            insertions: HashMap::new(),
        };
        delete_runs(&mut dom, p, &mut stamp);
        assert_eq!(
            dom.elements(p, Some(&W::del())).len(),
            1,
            "a and b share one"
        );
        assert_eq!(dom.descendants(p, Some(&W::del())).len(), 3);
        assert_eq!(dom.descendants(p, Some(&W::del_text())).len(), 3);
        assert_eq!(dom.descendants(p, Some(&W::del_instr_text())).len(), 1);
        assert!(dom.descendants(p, Some(&W::t())).is_empty());
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod nested_run_deletion_source_boundary_tests {
    use super::tests::counter_at;
    use super::*;
    fn semantic(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return dom.text_value(node).unwrap_or_default().into();
        }
        let name = dom.name(node).unwrap();
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(n, _)| {
                n.namespace_name() != "http://www.w3.org/2000/xmlns/" && n.local_name() != "xmlns"
            })
            .collect::<Vec<_>>();
        if attrs.is_empty()
            && dom.nodes(node).is_empty()
            && (name == W::r_pr() || name == W::p_pr())
        {
            return String::new();
        }
        attrs.sort_by_key(|(n, v)| {
            (
                n.namespace_name().to_string(),
                n.local_name().to_string(),
                *v,
            )
        });
        format!(
            "{:?}{attrs:?}[{}]",
            name,
            dom.nodes(node)
                .into_iter()
                .map(|n| semantic(dom, n))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("|")
        )
    }
    #[test]
    fn deleting_supported_nested_run_containers_preserves_complete_rejected_sources() {
        let runs = "<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>owned text</w:t><w:tab/><w:br w:type='page'/></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>second run</w:t></w:r>";
        let field = "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:rPr><w:color w:val='234567'/></w:rPr><w:instrText xml:space='preserve'> REF Clause </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>cached clause</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>";
        for content in [
            runs.to_owned(),
            format!("<w:hyperlink w:anchor='Clause'>{runs}</w:hyperlink>"),
            format!("<w:smartTag w:uri='urn:source' w:element='clause'>{runs}</w:smartTag>"),
            format!("<w:customXml w:uri='urn:source' w:element='clause'>{runs}</w:customXml>"),
            format!("<w:dir w:val='rtl'>{runs}</w:dir>"),
            format!("<w:bdo w:val='rtl'>{runs}</w:bdo>"),
            format!("<w:fldSimple w:instr=' DATE '>{runs}</w:fldSimple>"),
            format!(
                "<w:sdt><w:sdtPr><w:alias w:val='Clause'/><w:tag w:val='owner'/><w:id w:val='31'/><w:richText/></w:sdtPr><w:sdtContent>{runs}</w:sdtContent></w:sdt>"
            ),
            "<w:sdt><w:sdtPr><w:tag w:val='empty-owner'/><w:richText/></w:sdtPr></w:sdt>"
                .to_owned(),
            field.to_owned(),
            format!(
                "<w:hyperlink w:anchor='Clause'><w:sdt><w:sdtPr><w:tag w:val='nested-owner'/><w:id w:val='32'/><w:richText/></w:sdtPr><w:sdtContent>{runs}</w:sdtContent></w:sdt></w:hyperlink>"
            ),
        ] {
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&format!("<w:document xmlns:w='{}'><w:body><w:p><w:pPr><w:spacing w:after='120'/></w:pPr><w:bookmarkStart w:id='40' w:name='Clause'/>{content}<w:bookmarkEnd w:id='40'/></w:p><w:p><w:r><w:t>independent tail</w:t></w:r></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI));
            let root = dom.root(document).unwrap();
            let expected = semantic(&dom, root);
            let paragraph = dom.descendants(root, Some(&W::p()))[0];
            let source_runs = dom.descendants(paragraph, Some(&W::r()));
            let mut stamp = Stamp {
                author: "New owner",
                date: "2001-02-03T04:05:06Z",
                ids: counter_at(100),
                insertions: HashMap::new(),
            };
            delete_runs(&mut dom, paragraph, &mut stamp);
            mark(&mut dom, paragraph, W::del(), &mut stamp);
            for run in source_runs {
                let owner = dom.parent(run).unwrap();
                assert!(dom.name_is(owner, &W::del()), "run owner: {content}");
                assert_eq!(dom.attribute(owner, &W::author()), Some("New owner"));
                assert_eq!(
                    dom.attribute(owner, &W::date()),
                    Some("2001-02-03T04:05:06Z")
                );
                assert!(dom.descendants(run, Some(&W::t())).is_empty());
                assert!(dom.descendants(run, Some(&W::instr_text())).is_empty());
            }
            let result = crate::revision_processor::reject_revisions_document(&mut dom, root);
            assert_eq!(
                semantic(&dom, result),
                expected,
                "nested container complete source recovery: {content}"
            );
        }
    }
}
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tracked_source_property_boundary_tests {
    use super::tests::counter_at;
    use super::*;

    fn paragraph(dom: &mut Dom, content: &str) -> NodeId {
        let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}">{content}</w:p>"#, W::URI));
        dom.root(doc).unwrap()
    }

    #[test]
    fn property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source() {
        for shell in [
            "",
            "<w:pPr/>",
            "<w:pPr><w:pStyle w:val=\"BodyText\"/><w:spacing w:after=\"240\"/><w:rPr><w:b/><w:ins w:id=\"7\" w:author=\"Old\" w:date=\"2026-01-01T00:00:00Z\"/></w:rPr><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr>",
        ] {
            let mut dom = Dom::new();
            let p = paragraph(
                &mut dom,
                &format!("{shell}<w:r><w:rPr><w:i/></w:rPr><w:t>owned</w:t></w:r>"),
            );
            let run = dom.element(p, &W::r()).unwrap();
            let frozen_run = dom.serialize_element(run);
            let original_ppr = dom.element(p, &W::p_pr());
            let mark = original_ppr
                .and_then(|node| dom.element(node, &W::r_pr()))
                .map(|node| dom.serialize_element(node));
            let section = original_ppr
                .and_then(|node| dom.element(node, &W::sect_pr()))
                .map(|node| dom.serialize_element(node));
            let mut stamp = Stamp {
                author: "Ada",
                date: "2026-10-09T12:34:00Z",
                ids: counter_at(50),
                insertions: HashMap::new(),
            };
            track_properties(&mut dom, p, &mut stamp, |dom| {
                let ppr = paragraph_properties(dom, p);
                let jc = dom.new_element(W::jc_el());
                dom.set_attribute_value(jc, &W::val(), Some("right"));
                insert_ppr_child(dom, ppr, jc);
            });
            let ppr = dom.element(p, &W::p_pr()).unwrap();
            let record = dom.element(ppr, &W::p_pr_change()).unwrap();
            assert_eq!(dom.attribute(record, &W::id()), Some("50"));
            assert_eq!(dom.attribute(record, &W::author()), Some("Ada"));
            assert_eq!(
                dom.attribute(record, &W::date()),
                Some("2026-10-09T12:34:00Z")
            );
            let old = dom.element(record, &W::p_pr()).unwrap();
            assert!(dom.element(old, &W::r_pr()).is_none());
            assert!(dom.element(old, &W::sect_pr()).is_none());
            assert!(dom.element(old, &W::jc_el()).is_none());
            if shell.contains("BodyText") {
                assert_eq!(
                    dom.attribute(dom.element(old, &W::p_style()).unwrap(), &W::val()),
                    Some("BodyText")
                );
                assert_eq!(
                    dom.attribute(
                        dom.element(old, &W::spacing_el()).unwrap(),
                        &W::name("after")
                    ),
                    Some("240")
                );
                assert_eq!(dom.elements(old, None).len(), 2);
            } else {
                assert!(dom.elements(old, None).is_empty());
            }
            assert_eq!(
                dom.element(ppr, &W::r_pr())
                    .map(|node| dom.serialize_element(node)),
                mark
            );
            assert_eq!(
                dom.element(ppr, &W::sect_pr())
                    .map(|node| dom.serialize_element(node)),
                section
            );
            assert_eq!(dom.serialize_element(run), frozen_run);
            let frozen_record = dom.serialize_element(record);
            track_properties(&mut dom, p, &mut stamp, |dom| {
                let ppr = paragraph_properties(dom, p);
                let jc = dom.element(ppr, &W::jc_el()).unwrap();
                dom.set_attribute_value(jc, &W::val(), Some("center"));
            });
            assert_eq!(dom.serialize_element(record), frozen_record);
            assert_eq!(stamp.ids.own(), &BTreeSet::from([50]));
            assert_eq!(dom.elements(ppr, Some(&W::p_pr_change())).len(), 1);
            assert_eq!(dom.serialize_element(run), frozen_run);
        }
    }

    #[test]
    fn inserted_run_format_clones_drop_only_foreign_change_records() {
        let mut dom = Dom::new();
        let p = paragraph(
            &mut dom,
            r#"<w:r><w:rPr><w:b/><w:i/><w:color w:val="123456"/><w:sz w:val="24"/><w:rPrChange w:id="3" w:author="Old" w:date="2026-01-01T00:00:00Z"><w:rPr><w:u w:val="single"/></w:rPr></w:rPrChange></w:rPr><w:t>owned</w:t></w:r>"#,
        );
        let rpr = dom
            .element(dom.element(p, &W::r()).unwrap(), &W::r_pr())
            .unwrap();
        let frozen = dom.serialize_element(p);
        let copy = clean_rpr(&mut dom, rpr);
        let expected_p = paragraph(
            &mut dom,
            r#"<w:r><w:rPr><w:b/><w:i/><w:color w:val="123456"/><w:sz w:val="24"/></w:rPr></w:r>"#,
        );
        let expected = dom
            .element(dom.element(expected_p, &W::r()).unwrap(), &W::r_pr())
            .unwrap();
        assert_eq!(dom.serialize_element(copy), dom.serialize_element(expected));
        assert_eq!(dom.serialize_element(p), frozen);
        let second = clean_rpr(&mut dom, copy);
        assert_eq!(
            dom.serialize_element(second),
            dom.serialize_element(expected)
        );
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod public_adjacent_run_replacement_owner_tests {
    use super::*;
    use crate::changes::ChangeKind;
    use crate::edit::deeper_boundary_fixture::docx;

    fn tree(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return dom.text_value(node).unwrap_or_default().into();
        }
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(n, _)| {
                n.namespace_name() != "http://www.w3.org/2000/xmlns/" && n.local_name() != "xmlns"
            })
            .collect::<Vec<_>>();
        attrs.sort_by_key(|(n, v)| (n.namespace_name().to_owned(), n.local_name().to_owned(), *v));
        format!(
            "{:?}{attrs:?}[{}]",
            dom.name(node),
            dom.nodes(node)
                .into_iter()
                .map(|n| tree(dom, n))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
    fn source_events(bytes: &[u8]) -> Vec<String> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("/word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let mut out = Vec::new();
        for child in dom.elements(body, None) {
            if !dom.name_is(child, &W::p()) {
                out.push(tree(&dom, child));
                continue;
            }
            out.push("paragraph".into());
            for node in dom.elements(child, None) {
                if !dom.name_is(node, &W::r()) {
                    out.push(tree(&dom, node));
                    continue;
                }
                let props = dom
                    .element(node, &W::r_pr())
                    .map(|n| tree(&dom, n))
                    .unwrap_or_default();
                for content in dom.elements(node, None) {
                    if dom.name_is(content, &W::r_pr()) {
                        continue;
                    }
                    if dom.name_is(content, &W::t()) {
                        out.extend(dom.value(content).chars().map(|ch| format!("{ch}:{props}")));
                    } else {
                        out.push(format!("{}:{props}", tree(&dom, content)));
                    }
                }
            }
            out.push("end paragraph".into());
        }
        out
    }
    #[test]
    fn public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner() {
        let ppr = "<w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr>";
        let first = "<w:rPr><w:b/><w:color w:val='123456'/><w:sz w:val='22'/></w:rPr>";
        for second in [
            first,
            "<w:rPr><w:i/><w:color w:val='654321'/><w:sz w:val='28'/></w:rPr>",
        ] {
            for split_text in [false, true] {
                let lhs = if split_text {
                    "<w:t>αfirst</w:t><w:t>part</w:t>"
                } else {
                    "<w:t>αfirstpart</w:t>"
                };
                let body = format!(
                    "<w:p>{ppr}<w:r>{first}{lhs}</w:r><w:r>{second}<w:t>secondΖ</w:t></w:r><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:tab/><w:t>tail</w:t><w:br/></w:r></w:p>"
                );
                let source = docx(&body);
                let frozen = source.clone();
                let plan = r#"{"schema_version":1,"author":"Source owner","date":"2001-02-03T04:05:06Z","existing_revisions":"keep","operations":[{"kind":"replace","paragraph":"p0","find":"firstpartsecond","replacement":"owned replacement"}]}"#;
                let out = crate::edit::apply_plan_json(&source, plan).unwrap();
                let expected = docx(&format!(
                    "<w:p>{ppr}<w:r>{first}<w:t>αowned replacement</w:t></w:r><w:r>{second}<w:t>Ζ</w:t></w:r><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:tab/><w:t>tail</w:t><w:br/></w:r></w:p>"
                ));
                let accepted = crate::changes::accept_changes(
                    &out.redline,
                    &crate::changes::ChangeFilter::default(),
                )
                .unwrap();
                let rejected = crate::changes::reject_changes(
                    &out.redline,
                    &crate::changes::ChangeFilter::default(),
                )
                .unwrap();
                assert_eq!(
                    source_events(&rejected),
                    source_events(&source),
                    "all rejected glyphs/properties/controls/section owners"
                );
                assert_eq!(
                    source_events(&accepted),
                    source_events(&expected),
                    "replacement inherits first deleted source format only"
                );
                assert_eq!(source_events(&out.clean), source_events(&expected));
                assert_eq!(source, frozen);
                let changes = crate::changes::list_changes(&out.redline).unwrap();
                assert_eq!(
                    changes
                        .iter()
                        .map(|c| (c.kind, c.text.as_str(), c.author.as_deref()))
                        .collect::<Vec<_>>(),
                    [
                        (
                            ChangeKind::Deletion,
                            "firstpartsecond",
                            Some("Source owner")
                        ),
                        (
                            ChangeKind::Insertion,
                            "owned replacement",
                            Some("Source owner")
                        )
                    ]
                );
            }
        }
    }
}
