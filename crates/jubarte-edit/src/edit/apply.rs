// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Applying resolved text edits to the copy: run text and marks, run
//! splits, and the revision records they hand on or drop.

use super::*;

/// Replace `[start, end)` of the paragraph's projection with `replacement`.
/// The replacement goes to the first piece of the range; the other pieces
/// lose their text; a tab, a line break or a non-breaking hyphen in the range
/// is removed. A page or column break is no text and stays.
pub(super) fn apply_text_edit(
    dom: &mut Dom,
    projection: &Projection,
    start: usize,
    end: usize,
    replacement: &str,
    attach_before: bool,
) {
    if start == end {
        let seg = attach_segment(projection, start, attach_before).expect("checked at resolution");
        // At a run edge the source range endpoints keep their side of the
        // words. A separate run lets both the clean copy and the comparer
        // retain that boundary instead of extending the source range.
        let (leaf, run) = match seg.piece {
            Piece::Text { t, run } => (t, run),
            Piece::Glyph { element, run } => (element, run),
        };
        let before = start == seg.start;
        let content: Vec<_> = dom
            .elements(run, None)
            .into_iter()
            .filter(|&n| !dom.name_is(n, &W::r_pr()))
            .collect();
        let at_run_edge = if before {
            content.first() == Some(&leaf)
        } else {
            start == seg.end && content.last() == Some(&leaf)
        };
        if at_run_edge {
            let anchor = tracked::insertion_boundary_anchor(dom, run, before);
            if anchor != run {
                let inserted = dom.new_element(W::r());
                if let Some(rpr) = dom.element(run, &W::r_pr()) {
                    let rpr = tracked::clean_rpr(dom, rpr);
                    dom.add(inserted, rpr);
                }
                let t = dom.new_element(W::t());
                dom.add(inserted, t);
                write_text(dom, t, replacement);
                if before {
                    dom.add_before_self(anchor, inserted);
                } else {
                    dom.add_after_self(anchor, inserted);
                }
                return;
            }
        }
        match seg.piece {
            Piece::Text { t, run } => {
                let value = dom.value(t);
                let at = start - seg.start;
                if !insert_outside_ranges(dom, run, t, at, value.len(), replacement) {
                    let updated = format!("{}{}{}", &value[..at], replacement, &value[at..]);
                    write_text_new_in(dom, t, &updated, at..at + replacement.len());
                }
            }
            // Beside a tab or a break: new text of the glyph's run, after it
            // when it holds the char before the point.
            Piece::Glyph { element, run } => {
                let (at, len) = (start - seg.start, seg.end - seg.start);
                if !insert_outside_ranges(dom, run, element, at, len, replacement) {
                    let t = dom.new_element(W::t());
                    if seg.end == start {
                        dom.add_after_self(element, t);
                    } else {
                        dom.add_before_self(element, t);
                    }
                    write_text(dom, t, replacement);
                }
            }
        }
        return;
    }
    let mut inside = inner_comment_references(dom, projection, start, end);
    inside.extend(closing_comment_references(dom, projection, start, end));
    let mut pending = Some(replacement);
    for seg in &projection.segments {
        if seg.end <= start || seg.start >= end {
            continue;
        }
        match seg.piece {
            Piece::Text { t, .. } => {
                let value = dom.value(t);
                let from = start.saturating_sub(seg.start).min(value.len());
                let to = end.saturating_sub(seg.start).min(value.len());
                let middle = pending.take().unwrap_or_default();
                let updated = format!("{}{}{}", &value[..from], middle, &value[to..]);
                write_text_new_in(dom, t, &updated, from..from + middle.len());
            }
            // One char, so inside the range it is covered whole.
            Piece::Glyph { element, .. } => {
                if let Some(text) = pending.take().filter(|text| !text.is_empty()) {
                    let t = dom.new_element(W::t());
                    dom.add_before_self(element, t);
                    write_text(dom, t, text);
                }
                dom.remove(element);
            }
        }
    }
    remove_comment_references(dom, &inside);
}

/// The comment references between the first and the last character of
/// `[start, end)`: a selection over them takes their mark along, and Word
/// then drops the comment (rule R1).
fn inner_comment_references(
    dom: &Dom,
    projection: &Projection,
    start: usize,
    end: usize,
) -> Vec<NodeId> {
    let node = |seg: &crate::inspect::Segment| match seg.piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut covered = projection
        .segments
        .iter()
        .filter(|seg| seg.end > start && seg.start < end);
    // One covered piece has nothing between its ends.
    let (Some(first), Some(last)) = (covered.next().map(node), covered.next_back().map(node))
    else {
        return Vec::new();
    };
    let Some(&paragraph) = dom.ancestors(first, Some(&W::p())).first() else {
        return Vec::new();
    };
    let order = dom.descendants(paragraph, None);
    let at = |n: NodeId| order.iter().position(|&o| o == n);
    let (Some(from), Some(to)) = (at(first), at(last)) else {
        return Vec::new();
    };
    dom.descendants(paragraph, Some(&W::comment_reference()))
        .into_iter()
        .filter(|&r| at(r).is_some_and(|i| from < i && i < to))
        .collect()
}

/// The references of the comments wholly inside `[start, end)` that close
/// where it ends: the edit deletes all their text, as it does a comment
/// whose reference stands between its first and last character, but the
/// reference follows the last one, where [`inner_comment_references`]
/// does not look. A comment that opens where the range opens is not one:
/// replaced exactly, it keeps the new text.
fn closing_comment_references(
    dom: &Dom,
    projection: &Projection,
    start: usize,
    end: usize,
) -> Vec<NodeId> {
    let node = |seg: &crate::inspect::Segment| match seg.piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut covered = projection
        .segments
        .iter()
        .filter(|seg| seg.end > start && seg.start < end);
    let (Some(first), Some(last)) = (covered.next(), covered.next_back()) else {
        return Vec::new();
    };
    if last.end != end {
        return Vec::new();
    }
    let Some(&paragraph) = dom.ancestors(node(first), Some(&W::p())).first() else {
        return Vec::new();
    };
    let order = dom.descendants(paragraph, None);
    let at = |n: NodeId| order.iter().position(|&o| o == n);
    let (Some(from), Some(to)) = (at(node(first)), at(node(last))) else {
        return Vec::new();
    };
    if from >= to {
        return Vec::new();
    }
    let next = projection
        .segments
        .iter()
        .find(|seg| seg.start >= end)
        .and_then(|seg| at(node(seg)))
        .unwrap_or(order.len())
        .max(to + 1);
    let marker = |local: &str, id: &str, range: std::ops::Range<usize>| {
        order[range]
            .iter()
            .any(|&n| dom.name_is(n, &W::name(local)) && dom.attribute(n, &W::id()) == Some(id))
    };
    let opened: Vec<&str> = order[from + 1..to]
        .iter()
        .filter(|&&n| dom.name_is(n, &W::comment_range_start()))
        .filter_map(|&n| dom.attribute(n, &W::id()))
        .filter(|id| marker("commentRangeEnd", id, to + 1..next))
        .collect();
    order[to + 1..next]
        .iter()
        .copied()
        .filter(|&n| {
            dom.name_is(n, &W::comment_reference())
                && dom
                    .attribute(n, &W::id())
                    .is_some_and(|id| opened.contains(&id))
        })
        .collect()
}

/// Remove comment references, with the run when it holds nothing else.
fn remove_comment_references(dom: &mut Dom, references: &[NodeId]) {
    for &reference in references {
        let run = dom.parent(reference).filter(|&r| {
            dom.name_is(r, &W::r())
                && dom
                    .elements(r, None)
                    .into_iter()
                    .all(|c| c == reference || dom.name_is(c, &W::r_pr()))
        });
        dom.remove(run.unwrap_or(reference));
    }
}

pub(super) fn set_text(dom: &mut Dom, t: NodeId, value: &str) {
    dom.set_value(t, value);
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
}

/// A new run for text typed into the empty `paragraph`, as Word types it:
/// formatted as the paragraph mark (its `w:pPr/w:rPr`, without the mark's
/// own revisions or `w:oMath`, which only a mark takes). Not yet in the
/// tree.
pub(super) fn mark_run(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    let run = dom.new_element(W::r());
    if let Some(rpr) = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    {
        let rpr = tracked::clean_rpr(dom, rpr);
        if let Some(math) = dom.element(rpr, &W::name("oMath")) {
            dom.remove(math);
        }
        if dom.has_elements(rpr) {
            dom.add(run, rpr);
        }
    }
    run
}

/// `text` written into `run`, a [`mark_run`] already in the tree.
pub(super) fn write_mark_run_text(dom: &mut Dom, run: NodeId, text: &str) {
    let t = dom.new_element(W::t());
    dom.add(run, t);
    write_text(dom, t, text);
}

/// Text typed into an empty paragraph, placed edit by edit in reverse
/// [`sort_edits`] order, so that each edge keeps plan order and what the
/// paragraph holds (a field, a bookmark's markers) stays between them: the
/// text joins no range it was not typed in (#1303).
#[derive(Default)]
pub(super) struct EmptyPlacement {
    /// The end text placed last, which the next end text goes before.
    next_end: Option<NodeId>,
}

impl EmptyPlacement {
    /// Put `node` where its text was typed in `paragraph`: last for an
    /// insert at the end (before the end text already placed), else first,
    /// after the paragraph's properties (an insert at the start, a rewrite).
    pub(super) fn place(
        &mut self,
        dom: &mut Dom,
        paragraph: NodeId,
        node: NodeId,
        attach_before: bool,
    ) {
        if attach_before {
            match self.next_end {
                Some(next) => dom.add_before_self(next, node),
                None => dom.add(paragraph, node),
            }
            self.next_end = Some(node);
        } else if let Some(ppr) = dom.element(paragraph, &W::p_pr()) {
            dom.add_after_self(ppr, node);
        } else {
            dom.add_first(paragraph, node);
        }
    }
}

/// Sort a paragraph's edits for applying: by source range, then plan
/// order. In an `empty` paragraph every edit sits at its one point, and the
/// start edge's text comes before the end edge's (#1303).
pub(super) fn sort_edits(edits: &mut [ScheduledEdit], empty: bool) {
    edits.sort_by_key(|e| (e.start, e.end, empty && e.attach_before, e.op));
}

/// Emit a paragraph-edge insertion and the `delete` it touches as one whole
/// substitution (#1922): a delete that starts at the paragraph's first
/// character takes the `position: "start"` insertions, one that ends at its
/// last character the `position: "end"` ones. Deleting every visible run
/// must not leave such an insertion without a run to attach to. A `replace`
/// is never merged: it writes its own pieces, so the clean copy and the
/// redline keep the same runs. Insertions of different formatting cannot
/// share one inserted run, so that pair is refused. Original edits remain
/// available for each operation's comment and formatting coordinates.
pub(super) fn boundary_substitutions(
    tx: &Transaction<'_>,
    edits: &[ScheduledEdit],
    para: usize,
) -> Result<Vec<ScheduledEdit>, EditError> {
    let text_len = tx.projections[para].text.len();
    let is = |e: &ScheduledEdit, delete: bool| match tx.plan.operations[e.op].kind {
        OperationKind::Delete { .. } => delete && e.start != e.end,
        OperationKind::Insert { .. } => !delete && e.start == e.end,
        _ => false,
    };
    let mut consumed = HashSet::new();
    let mut substitutions = BTreeMap::new();
    for (index, removal) in edits.iter().enumerate() {
        if !is(removal, true) {
            continue;
        }
        let mut combined = removal.clone();
        let mut prefix = String::new();
        let mut suffix = String::new();
        for (i, insertion) in edits.iter().enumerate() {
            if !is(insertion, false) || consumed.contains(&i) {
                continue;
            }
            let before = removal.start == 0 && insertion.start == 0;
            let after = removal.end == text_len && insertion.start == text_len;
            if !before && !after {
                continue;
            }
            if combined.op != removal.op && combined.format != insertion.format {
                return Err(edge_conflict(tx, combined.op, insertion.op));
            }
            if before {
                prefix.push_str(&insertion.replacement);
            } else {
                suffix.push_str(&insertion.replacement);
            }
            // The insertion's kind and formatting make this a whole
            // substitution; a delete carries neither.
            combined.op = insertion.op;
            combined.format = insertion.format.clone();
            consumed.insert(i);
        }
        if combined.op != removal.op {
            combined.replacement = format!("{prefix}{suffix}");
            substitutions.insert(index, combined);
        }
    }
    Ok(edits
        .iter()
        .enumerate()
        .filter_map(|(i, edit)| {
            if consumed.contains(&i) {
                None
            } else {
                Some(substitutions.remove(&i).unwrap_or_else(|| edit.clone()))
            }
        })
        .collect())
}

/// `OVERLAPPING_EDITS` for two edge insertions of different formatting that
/// meet one delete (#1922). The redline replay has no outcomes, so the ids
/// fall back to the plan's own numbering instead of indexing them.
fn edge_conflict(tx: &Transaction<'_>, first: usize, second: usize) -> EditError {
    let id = |op: usize| {
        tx.outcomes.get(op).map_or_else(
            || {
                tx.plan.operations[op]
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("op-{}", op + 1))
            },
            |o| o.id.clone(),
        )
    };
    let message = format!(
        "{} and {} insert differently formatted text where one delete removes \
         the paragraph's edge; insert it as one text, or use one replace",
        id(first),
        id(second)
    );
    if tx.outcomes.get(second).is_some() {
        tx.conflict(second, &message)
    } else {
        err("OVERLAPPING_EDITS", Some(&id(second)), &message)
    }
}

/// Set the text of `t`, a `w:t` in a run, writing each `\t` as a `w:tab`
/// and each `\n` as a `w:br` of that run, with the text between them in
/// `w:t`s of its own; `t` itself goes when no text is left for it.
pub(super) fn write_text(dom: &mut Dom, t: NodeId, value: &str) {
    write_text_new_in(dom, t, value, 0..value.len());
}

/// [`write_text`] for `value` whose new text is `new` alone: the text kept
/// around it stays as the source wrote it, a tab typed in `w:t` included,
/// so an edit never restates what it did not touch.
fn write_text_new_in(dom: &mut Dom, t: NodeId, value: &str, new: std::ops::Range<usize>) {
    let cuts: Vec<usize> = value[new.clone()]
        .match_indices(['\t', '\n'])
        .map(|(at, _)| new.start + at)
        .collect();
    let Some(&first) = cuts.first() else {
        set_text(dom, t, value);
        return;
    };
    set_text(dom, t, &value[..first]);
    let mut last = t;
    for (i, &cut) in cuts.iter().enumerate() {
        let glyph = if value.as_bytes()[cut] == b'\t' {
            "tab"
        } else {
            "br"
        };
        let element = dom.new_element(W::name(glyph));
        dom.add_after_self(last, element);
        last = element;
        let piece = &value[cut + 1..cuts.get(i + 1).copied().unwrap_or(value.len())];
        if !piece.is_empty() {
            let next = dom.new_element(W::t());
            dom.add_after_self(last, next);
            set_text(dom, next, piece);
            last = next;
        }
    }
    if first == 0 {
        dom.remove(t);
    }
}

/// Split the run owning `seg` so that a run boundary falls at projection byte
/// `at` (inside `seg`). Returns nothing when `at` is already a boundary.
/// The right half's copy of a formatting history (another author's
/// `w:rPrChange`) takes a fresh id from `ids`; the left half, the
/// original run, keeps the original id (#1656).
pub(super) fn split_run_at(
    dom: &mut Dom,
    seg: &crate::inspect::Segment,
    at: usize,
    ids: &mut RevisionCounter,
) {
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
    refresh_run_history(dom, clone, ids);
    dom.add_after_self(run, clone);
}

/// Split `run` so that a run boundary falls before `child` (one of its
/// descendants): the original keeps the children before it, a copy takes
/// the child holding it and every child after. The copy's formatting
/// history takes fresh ids, as [`split_run_at`]'s does.
pub(super) fn split_run_before(
    dom: &mut Dom,
    run: NodeId,
    child: NodeId,
    ids: &mut RevisionCounter,
) {
    let Some(holder) = std::iter::once(child)
        .chain(dom.ancestors(child, None))
        .find(|&n| dom.parent(n) == Some(run))
    else {
        return;
    };
    let run_children = dom.elements(run, None);
    let Some(index) = run_children.iter().position(|&c| c == holder) else {
        return;
    };
    if run_children[..index]
        .iter()
        .all(|&c| dom.name_is(c, &W::r_pr()))
    {
        return;
    }
    let clone = dom.clone_subtree(run);
    for (i, child) in dom.elements(clone, None).into_iter().enumerate() {
        if i < index && !dom.name_is(child, &W::r_pr()) {
            dom.remove(child);
        }
    }
    for &child in &run_children[index..] {
        dom.remove(child);
    }
    refresh_run_history(dom, clone, ids);
    dom.add_after_self(run, clone);
}

/// Take the new words at projection offsets `[start, end)` of `paragraph`
/// out of the formatting records of the runs they were written into: they
/// go in runs of their own without a record, and the fragments around them
/// keep theirs. New words are no one's earlier formatting (#1658). Runs
/// without a record are left as they are. A record keeps its id on the
/// words it covered: where the new words open a run, the copy split off
/// after them takes the run's own properties.
pub(super) fn clean_new_words(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    ids: &mut RevisionCounter,
) {
    let recorded = |dom: &Dom, run: NodeId| {
        dom.element(run, &W::r_pr())
            .and_then(|rpr| dom.element(rpr, &W::r_pr_change()))
            .is_some()
    };
    if start >= end
        || !project_paragraph(dom, paragraph)
            .segments
            .iter()
            .any(|s| s.start < end && start < s.end && recorded(dom, run_of(&s.piece)))
    {
        return;
    }
    // Every run before the splits, those without projected text included:
    // a run after the new words that is not one of them is a copy.
    let originals: HashSet<NodeId> = dom
        .descendants(paragraph, Some(&W::r()))
        .into_iter()
        .collect();
    let mut stripped = Vec::new();
    for run in runs_in_range(dom, paragraph, start, end, ids) {
        let Some(mut rpr) = dom.element(run, &W::r_pr()) else {
            continue;
        };
        if originals.contains(&run)
            && let Some(rest) = dom
                .next_element(run)
                .filter(|&n| dom.name_is(n, &W::r()) && !originals.contains(&n))
            && let Some(copied) = dom.element(rest, &W::r_pr())
        {
            dom.remove(rpr);
            dom.remove(copied);
            dom.add_first(rest, rpr);
            dom.add_first(run, copied);
            rpr = copied;
        }
        stripped.extend(strip_run_records(dom, rpr));
    }
    hand_on_ids(dom, paragraph, &stripped, ids);
}

/// Strip the records in a run's properties `rpr`, and the properties when
/// nothing else is left in them: each formatting record's id, as a number
/// and as spelled.
fn strip_run_records(dom: &mut Dom, rpr: NodeId) -> Vec<(u64, String)> {
    let ids = dom
        .elements(rpr, Some(&W::r_pr_change()))
        .into_iter()
        .filter_map(|record| {
            let spelled = dom.attribute(record, &W::id())?;
            Some((record_id(dom, record)?, spelled.to_string()))
        })
        .collect();
    tracked::strip_records(dom, rpr);
    if dom.elements(rpr, None).is_empty() {
        dom.remove(rpr);
    }
    ids
}

/// A formatting record's `w:id`, as a number.
fn record_id(dom: &Dom, record: NodeId) -> Option<u64> {
    dom.attribute(record, &W::id())?.parse().ok()
}

/// A record that lost all its words leaves its id to the first copy an
/// earlier split of the transaction made of it: the id stays, as spelled,
/// on what the record still covers (words, or an atom such as a tab, as
/// `changes::run_characters` counts them), as one edit over the
/// same words leaves it (Codex and MiniMax on #1664).
fn hand_on_ids(
    dom: &mut Dom,
    paragraph: NodeId,
    stripped: &[(u64, String)],
    ids: &RevisionCounter,
) {
    for (id, spelled) in stripped {
        let original = ids.original(*id);
        // Run records only: a paragraph mark's record is no copy of a run's,
        // whatever its id.
        let records: Vec<NodeId> = dom
            .descendants(paragraph, Some(&W::r_pr_change()))
            .into_iter()
            .filter(|&record| {
                dom.parent(record)
                    .and_then(|rpr| dom.parent(rpr))
                    .is_some_and(|run| dom.name_is(run, &W::r()))
            })
            .collect();
        if records
            .iter()
            .any(|&record| record_id(dom, record) == Some(original))
        {
            continue;
        }
        if let Some(heir) = records.into_iter().find(|&record| {
            record_id(dom, record).is_some_and(|other| ids.original(other) == original)
        }) {
            let spelled = if *id == original {
                spelled.clone()
            } else {
                original.to_string()
            };
            dom.set_attribute_value(heir, &W::id(), Some(&spelled));
        }
    }
}

/// Take the formatting records off the `reached` runs an edit left
/// without a character the history guard counts
/// ([`crate::changes::run_has_characters`]): a record goes with the last
/// of its words, and its id with it to a copy that still covers some
/// (Codex on #1664).
pub(super) fn drop_emptied_records(
    dom: &mut Dom,
    paragraph: NodeId,
    reached: &[NodeId],
    ids: &RevisionCounter,
) {
    let mut stripped = Vec::new();
    for &run in reached {
        if crate::changes::run_has_characters(dom, run) {
            continue;
        }
        let Some(rpr) = dom.element(run, &W::r_pr()) else {
            continue;
        };
        stripped.extend(strip_run_records(dom, rpr));
    }
    hand_on_ids(dom, paragraph, &stripped, ids);
}

/// Fresh ids for the formatting history in `run`'s own `w:rPr`, for a run
/// that is a copy of another. Content the copy took over (a text box in a
/// drawing) moved rather than doubled, so its records keep their ids.
pub(super) fn refresh_run_history(dom: &mut Dom, run: NodeId, ids: &mut RevisionCounter) {
    if let Some(rpr) = dom.element(run, &W::r_pr()) {
        crate::revision_processor::refresh_property_ids(dom, rpr, |old| ids.copy(old));
    }
}
