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
//! for paragraph formatting. The other party's markup is never touched.
//! Resolution already refuses ranges inside a revision, so every range here
//! holds only runs that are direct children of their paragraph.

use std::collections::{BTreeMap, BTreeSet};

use super::structural;
use super::{
    EditError, EditPlan, EditResult, ExistingRevisions, OperationKind, Resolved, RevisionCounts,
    ScheduledEdit, Side, ThreadOp, Transaction, anchor_comment, anchor_span, apply_run_format,
    apply_text_edit, attach_segment, build_paragraph, format_paragraph, insert_ppr_child,
    new_position, place_reply_markers, remove_comment_markers, set_text, split_run_at,
    toc_paragraphs,
};
use crate::changes::{ChangeKind, list_changes};
use crate::inspect::{Opened, Piece, project_paragraph};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XName};

/// Revision attribution for the containers the emitter creates.
struct Stamp<'a> {
    author: &'a str,
    date: &'a str,
    next_id: u64,
}

impl Stamp<'_> {
    fn container(&mut self, dom: &mut Dom, name: XName) -> NodeId {
        let node = dom.new_element(name);
        dom.set_attribute_value(node, &W::id(), Some(&self.next_id.to_string()));
        self.next_id += 1;
        dom.set_attribute_value(node, &W::author(), Some(self.author));
        dom.set_attribute_value(node, &W::date(), Some(self.date));
        node
    }
}

/// Refusals that only `keep` needs, checked on the untouched source after
/// resolution: a paragraph-level change on top of another revision of the
/// same paragraph mark or properties.
pub(super) fn check(tx: &Transaction<'_>) -> Result<(), EditError> {
    if tx.plan.existing_revisions != ExistingRevisions::Keep {
        return Ok(());
    }
    let dom = &tx.opened.dom;
    for (op, resolved) in &tx.resolved {
        let message = match resolved {
            Resolved::DeleteParagraph { para } => {
                let node = tx.paragraph_nodes[*para];
                if holds_revision(dom, node) {
                    Some(
                        "under keep, a paragraph that holds tracked changes cannot be deleted; resolve them first with resolve_revisions",
                    )
                } else if dom
                    .next_element(node)
                    .is_none_or(|n| !dom.name_is(n, &W::p()))
                    && previous_paragraph(dom, node).is_some_and(|p| mark_has(dom, p, &DELETED))
                {
                    Some(
                        "under keep, the last paragraph cannot be deleted when the paragraph before it already has a deleted mark; resolve that change first",
                    )
                } else {
                    None
                }
            }
            Resolved::MergeParagraphs { para, .. }
                if mark_has(dom, tx.paragraph_nodes[*para], &TRACKED) =>
            {
                Some(
                    "under keep, a paragraph whose mark is already tracked cannot be merged; resolve that change first with resolve_revisions",
                )
            }
            Resolved::FormatParagraph { para, .. }
                if dom
                    .element(tx.paragraph_nodes[*para], &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
                    .is_some() =>
            {
                Some(
                    "under keep, a paragraph whose formatting change is still tracked cannot be formatted again; resolve that change first with resolve_revisions",
                )
            }
            Resolved::List { paras, .. }
                if paras.iter().any(|&para| {
                    dom.element(tx.paragraph_nodes[para], &W::p_pr())
                        .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
                        .is_some()
                }) =>
            {
                Some(
                    "under keep, a paragraph whose formatting change is still tracked cannot be numbered; resolve that change first with resolve_revisions",
                )
            }
            _ => None,
        };
        if let Some(message) = message {
            let mut error = tx.conflict(*op, message);
            error.code = "UNSUPPORTED_STRUCTURE".into();
            error.outcomes[*op].code = Some(error.code.clone());
            return Err(error);
        }
    }
    Ok(())
}

/// The result under `keep`: the clean copy as `finish` made it, the redline
/// by emission, and revision counts of the plan's own changes.
pub(super) fn result(tx: &Transaction<'_>, clean: Vec<u8>) -> Result<EditResult, EditError> {
    let redline = redline(tx)?;
    let mut report = tx.report(true);
    report.paragraphs.to = crate::inspect::paragraphs(&clean)
        .map(|p| p.len())
        .unwrap_or(report.paragraphs.from);
    report.revisions = revision_counts(&redline, &tx.plan.author, &tx.date);
    Ok(EditResult {
        clean,
        redline,
        report,
    })
}

/// Replay `tx`'s resolved operations on a fresh copy of its base as tracked
/// changes. `tx` has been applied, so its outcomes hold the comment ids.
fn redline(tx: &Transaction<'_>) -> Result<Vec<u8>, EditError> {
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
        if let Some(text) = tx.deletion_comment(op) {
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
    let mut stamp = Stamp {
        author: &tx.plan.author,
        date: &tx.date,
        next_id: first_free_id(&t.opened, &comments),
    };
    let tables = emit(&mut t, &comment_ids, &mut stamp);
    thread_markers(&mut t, &comment_ids);
    // 7. A paragraph after each new table, and between it and a table
    // before it: inserted paragraphs here.
    for table in tables {
        separate_tracked(&mut t.opened.dom, table, &mut stamp);
    }
    t.comments = comments;
    t.reply_parents = tx.reply_parents.clone();
    let (redline, _) = t.finish()?;
    Ok(redline)
}

/// Counts of the redline's changes by `author` on `date`.
fn revision_counts(redline: &[u8], author: &str, date: &str) -> RevisionCounts {
    let mut counts = RevisionCounts::default();
    let changes = list_changes(redline).unwrap_or_default();
    for change in changes
        .iter()
        .filter(|c| c.author.as_deref() == Some(author) && c.date.as_deref() == Some(date))
    {
        match change.kind {
            ChangeKind::Insertion => counts.inserted += 1,
            ChangeKind::Deletion => counts.deleted += 1,
            ChangeKind::Move => counts.moved += 1,
            ChangeKind::Formatting => counts.format_changed += 1,
        }
        counts.total += 1;
    }
    counts
}

/// One past the highest `w:id` in every revision-bearing part and among the
/// plan's comments, so a new revision id collides with nothing.
fn first_free_id(opened: &Opened, comments: &[(u32, String)]) -> u64 {
    let mut max: Option<u64> = comments.iter().map(|&(id, _)| u64::from(id)).max();
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
                max = max.max(Some(id));
            }
        }
    }
    max.map_or(0, |m| m + 1)
}

/// The steps of `Transaction::apply`, in its order, as tracked changes.
/// Returns the new tables, which get their separating paragraphs last.
fn emit(
    t: &mut Transaction<'_>,
    comment_ids: &BTreeMap<usize, u32>,
    stamp: &mut Stamp<'_>,
) -> Vec<NodeId> {
    // 1. Text edits and their comments, paragraph by paragraph.
    let mut by_para: BTreeMap<usize, Vec<ScheduledEdit>> = BTreeMap::new();
    let mut comment_ranges: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    for (i, r) in &t.resolved {
        match r {
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
        edits.sort_by_key(|e| (e.start, e.end, e.op));
        // In reverse, so earlier source offsets stay valid: a deletion
        // leaves the projection and an insertion joins it, exactly as the
        // plain edit changes the clean copy.
        for edit in edits.iter().rev() {
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
            } else {
                emit_text(dom, node, edit, stamp);
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
            let s = new_position(&edits, start, true, None);
            let e = new_position(&edits, end, false, None);
            pending.push((s, e, i));
        }
        pending.sort_by_key(|&(s, _, i)| (s, i));
        for (start, end, i) in pending {
            if let Some(&id) = comment_ids.get(&i) {
                anchor_comment(dom, node, start, end, id);
            }
        }
    }
    // 1b. Comments over several paragraphs, in their edited text.
    for (i, r) in &t.resolved {
        if let Resolved::CommentSpan { para, last, .. } = r
            && let Some(&id) = comment_ids.get(i)
        {
            let (first, last) = (t.paragraph_nodes[*para], t.paragraph_nodes[*last]);
            anchor_span(&mut t.opened.dom, first, last, id);
        }
    }
    // 2. Paragraph and table insertions: every run, row and paragraph mark
    // inserted.
    let mut last_after: BTreeMap<usize, NodeId> = BTreeMap::new();
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
                Side::Before => dom.add_before_self(anchor_node, new),
            }
        }
        let new = news[news.len() - 1];
        if commented && let Some(&id) = comment_ids.get(i) {
            let projection = project_paragraph(dom, new);
            anchor_comment(dom, new, 0, projection.text.len(), id);
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
            anchor_comment(dom, node, 0, projection.text.len(), id);
        }
        delete_runs(dom, node, stamp);
        // The mark of the last paragraph of a container cannot go: its
        // content joins the paragraph before it, whose mark is deleted.
        let carrier = if dom
            .next_element(node)
            .is_some_and(|n| dom.name_is(n, &W::p()))
        {
            Some(node)
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
        let properties = match dom.element(row, &W::name("trPr")) {
            Some(properties) => properties,
            None => {
                let properties = dom.new_element(W::name("trPr"));
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

/// Emit one text edit: the range's runs in `w:del`, then the replacement in
/// a `w:ins` run formatted like the first deleted run, or like the run the
/// insertion attaches to.
fn emit_text(dom: &mut Dom, paragraph: NodeId, edit: &ScheduledEdit, stamp: &mut Stamp<'_>) {
    let (start, end) = (edit.start, edit.end);
    // `None` places the insertion right after `anchor`, `Some(true)` before.
    let (template, anchor, before) = if start < end {
        let runs = isolate_range(dom, paragraph, start, end);
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
        for group in groups {
            let del = stamp.container(dom, W::del());
            dom.add_before_self(group[0], del);
            for run in group {
                dom.remove(run);
                dom.add(del, run);
                rename_deleted_text(dom, run);
            }
            last_del = Some(del);
        }
        (runs.first().copied(), last_del, false)
    } else {
        let projection = project_paragraph(dom, paragraph);
        let Some(seg) = attach_segment(&projection, start, edit.attach_before).cloned() else {
            return;
        };
        split_run_at(dom, &seg, start);
        let projection = project_paragraph(dom, paragraph);
        let text_piece = |s: &crate::inspect::Segment| match s.piece {
            Piece::Text { t, run } if s.direct => Some((t, run)),
            _ => None,
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
        isolate(dom, run, t);
        (Some(run), Some(run), before)
    };
    if edit.replacement.is_empty() {
        return;
    }
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
    set_text(dom, text, &edit.replacement);
    dom.add(run, text);
    let ins = stamp.container(dom, W::ins());
    dom.add(ins, run);
    match anchor {
        Some(anchor) if before => dom.add_before_self(anchor, ins),
        Some(anchor) => dom.add_after_self(anchor, ins),
        None => dom.add(paragraph, ins),
    }
}

/// Split runs so that each `w:t` of `[start, end)` sits alone in its run
/// (beside the run's `w:rPr`); the runs, in document order.
fn isolate_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId> {
    for at in [start, end] {
        let projection = project_paragraph(dom, paragraph);
        if let Some(seg) = projection
            .segments
            .iter()
            .find(|s| s.start < at && at < s.end)
            .cloned()
        {
            split_run_at(dom, &seg, at);
        }
    }
    loop {
        let projection = project_paragraph(dom, paragraph);
        let pieces: Vec<(NodeId, NodeId)> = projection
            .segments
            .iter()
            .filter(|s| s.start >= start && s.end <= end && s.start < s.end)
            .filter_map(|s| match s.piece {
                Piece::Text { t, run } => Some((t, run)),
                Piece::Glyph { .. } => None,
            })
            .collect();
        match pieces.iter().find(|&&(t, run)| !alone(dom, run, t)) {
            Some(&(t, run)) => isolate(dom, run, t),
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
/// keeps the run's `w:rPr`.
fn isolate(dom: &mut Dom, run: NodeId, t: NodeId) {
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
        } else {
            dom.add_after_self(run, copy);
        }
        for c in side {
            dom.remove(c);
        }
    }
}

/// `w:t` to `w:delText` and `w:instrText` to `w:delInstrText` in a run that
/// now sits inside a `w:del`.
fn rename_deleted_text(dom: &mut Dom, run: NodeId) {
    for child in dom.elements(run, None) {
        if dom.name_is(child, &W::t()) {
            dom.set_name(child, W::del_text());
        } else if dom.name_is(child, &W::name("instrText")) {
            dom.set_name(child, W::name("delInstrText"));
        }
    }
}

/// Every run of `container`, at any depth of hyperlinks, content controls,
/// smart tags and simple fields, inside a `w:del`; consecutive runs share one.
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
            _ => {}
        }
    }
    wrap_deleted(dom, group, stamp);
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
const TRACKED: [&str; 4] = ["ins", "del", "moveFrom", "moveTo"];

/// Whether `paragraph`'s mark (`w:pPr/w:rPr`) carries one of `locals`.
fn mark_has(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> bool {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.elements(rpr, None).into_iter().any(|c| {
                dom.name(c).is_some_and(|n| {
                    n.namespace_name() == W::URI && locals.contains(&n.local_name())
                })
            })
        })
}

/// Whether `paragraph` holds an inserted, deleted or moved run or mark.
fn holds_revision(dom: &Dom, paragraph: NodeId) -> bool {
    dom.descendants(paragraph, None).into_iter().any(|n| {
        dom.name(n)
            .is_some_and(|n| n.namespace_name() == W::URI && TRACKED.contains(&n.local_name()))
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
    let ppr = paragraph_properties(dom, paragraph);
    let rpr = match dom.element(ppr, &W::r_pr()) {
        Some(rpr) => rpr,
        None => {
            let rpr = dom.new_element(W::r_pr());
            insert_ppr_child(dom, ppr, rpr);
            rpr
        }
    };
    let node = stamp.container(dom, name);
    match dom.element(rpr, &W::ins()) {
        Some(ins) if deleting => dom.add_after_self(ins, node),
        _ => dom.add_first(rpr, node),
    }
}

/// A copy of a run's `w:rPr` for new text: without another revision's
/// format change or mark revisions, whose ids must stay unique.
fn clean_rpr(dom: &mut Dom, rpr: NodeId) -> NodeId {
    let copy = dom.clone_subtree(rpr);
    for child in dom.elements(copy, None) {
        if dom.name_is(child, &W::r_pr_change())
            || dom.name_is(child, &W::ins())
            || dom.name_is(child, &W::del())
        {
            dom.remove(child);
        }
    }
    copy
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

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
        isolate(&mut dom, run, t);
        let runs = dom.elements(p, Some(&W::r()));
        assert_eq!(runs.len(), 3);
        assert!(alone(&dom, runs[1], t));
        for r in runs {
            assert!(dom.element(r, &W::r_pr()).is_some(), "each copy keeps rPr");
        }
        // Already alone: nothing changes.
        let before = dom.elements(p, None).len();
        isolate(&mut dom, run, t);
        assert_eq!(dom.elements(p, None).len(), before);
    }

    #[test]
    fn a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled() {
        let mut dom = Dom::new();
        let p = paragraph(&mut dom, "<w:r><w:t>x</w:t></w:r>");
        let mut stamp = Stamp {
            author: "Me",
            date: "2026-10-02T00:00:00Z",
            next_id: 5,
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
        assert_eq!(stamp.next_id, 7);
        assert!(mark_has(&dom, p, &DELETED));
        assert!(!holds_revision(&dom, dom.elements(p, Some(&W::r()))[0]));
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
            next_id: 0,
        };
        delete_runs(&mut dom, p, &mut stamp);
        assert_eq!(
            dom.elements(p, Some(&W::del())).len(),
            1,
            "a and b share one"
        );
        assert_eq!(dom.descendants(p, Some(&W::del())).len(), 3);
        assert_eq!(dom.descendants(p, Some(&W::del_text())).len(), 3);
        assert_eq!(dom.descendants(p, Some(&W::name("delInstrText"))).len(), 1);
        assert!(dom.descendants(p, Some(&W::t())).is_empty());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod nested_run_deletion_source_boundary_tests {
    use super::*;
    fn semantic(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return dom.text_value(node).unwrap_or_default().into();
        }
        let name = dom.name(node).unwrap();
        let mut attrs = dom
            .attributes(node)
            .into_iter()
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
                v.clone(),
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
                next_id: 100,
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
                assert!(dom.descendants(run, Some(&W::name("instrText"))).is_empty());
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tracked_source_property_boundary_tests {
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
                next_id: 50,
            };
            track_properties(&mut dom, p, &mut stamp, |dom| {
                let ppr = paragraph_properties(dom, p);
                let jc = dom.new_element(W::name("jc"));
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
            assert!(dom.element(old, &W::name("jc")).is_none());
            if shell.contains("BodyText") {
                assert_eq!(
                    dom.attribute(dom.element(old, &W::p_style()).unwrap(), &W::val()),
                    Some("BodyText")
                );
                assert_eq!(
                    dom.attribute(
                        dom.element(old, &W::name("spacing")).unwrap(),
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
                let jc = dom.element(ppr, &W::name("jc")).unwrap();
                dom.set_attribute_value(jc, &W::val(), Some("center"));
            });
            assert_eq!(dom.serialize_element(record), frozen_record);
            assert_eq!(stamp.next_id, 51);
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
