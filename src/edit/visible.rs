// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `strike` and `restore` (plan 2): another author's change rejected
//! visibly, as a tracked change of the plan's author that Word shows under
//! that name and that Reject takes back.
//!
//! `strike` of their insertion is our deletion of its visible words,
//! nested inside it: it resolves to the text deletions the emitter already
//! nests (#1136). `restore` of their deletion leaves it as it is and
//! inserts a copy of its runs right after it, `w:delText` read as `w:t`
//! and the run formatting kept.

use super::{
    EditError, EditOutcome, EditPlan, ExistingRevisions, OperationKind, Resolved, Transaction,
    check_comment, comment_marker, comment_reference_run, editable_insertion, err, excerpt, run_of,
};
use crate::changes::{Change, ChangeKind, MoveSide};
use crate::inspect::project_paragraph;
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// Which visible reject an operation asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Visible {
    Strike,
    Restore,
}

impl Visible {
    /// The operation kind's name.
    pub(super) fn verb(self) -> &'static str {
        match self {
            Self::Strike => "strike",
            Self::Restore => "restore",
        }
    }

    fn past(self) -> &'static str {
        match self {
            Self::Strike => "struck",
            Self::Restore => "restored",
        }
    }

    /// What it takes, as the refusals say it.
    fn takes(self) -> &'static str {
        match self {
            Self::Strike => "inserted",
            Self::Restore => "deleted",
        }
    }
}

/// The visible reject `kind` asks for, with its revision and comment.
pub(super) fn of(kind: &OperationKind) -> Option<(Visible, &str, Option<&str>)> {
    match kind {
        OperationKind::Strike { revision, comment } => {
            Some((Visible::Strike, revision, comment.as_deref()))
        }
        OperationKind::Restore { revision, comment } => {
            Some((Visible::Restore, revision, comment.as_deref()))
        }
        _ => None,
    }
}

/// `INVALID_PLAN` for a strike or a restore in a plan that does not keep
/// the source's revisions: the change it names would already be settled.
pub(super) fn check_keep(plan: &EditPlan) -> Result<(), EditError> {
    if plan.existing_revisions == ExistingRevisions::Keep {
        return Ok(());
    }
    let Some((visible, ..)) = plan.operations.iter().find_map(|op| of(&op.kind)) else {
        return Ok(());
    };
    let why = match plan.existing_revisions {
        ExistingRevisions::Accept | ExistingRevisions::Reject => {
            " (the revision would already be settled)"
        }
        ExistingRevisions::Refuse | ExistingRevisions::Keep => "",
    };
    Err(err(
        "INVALID_PLAN",
        None,
        format!("{} needs existing_revisions keep{why}", visible.verb()),
    ))
}

/// `plan` with `resolve_revisions.reject.visible` lowered: each change the
/// reject side selects becomes a `strike` (an insertion) or a `restore` (a
/// deletion) after the plan's operations, in document order, and the
/// reject side settles nothing. `None` when the plan asks for none.
pub(super) fn lower(source: &[u8], plan: &EditPlan) -> Result<Option<EditPlan>, EditError> {
    let Some(selection) = plan.resolve_revisions.as_ref().filter(|s| s.reject_visible) else {
        return Ok(None);
    };
    if plan.existing_revisions != ExistingRevisions::Keep {
        return Err(err(
            "INVALID_PLAN",
            None,
            "resolve_revisions.reject.visible needs existing_revisions keep: the rejected changes stay, beside ours",
        ));
    }
    if let Some(reject) = &selection.reject {
        crate::changes::check_whole_changes(source, reject).map_err(super::selection_error)?;
    }
    let listed = crate::changes::list_changes(source).map_err(super::selection_error)?;
    let ids = super::selected_ids(&listed, selection.reject.as_ref())?;
    let mut lowered = plan.clone();
    lowered.resolve_revisions = selection
        .accept
        .clone()
        .map(|accept| super::ResolveRevisions {
            accept: Some(accept),
            reject: None,
            reject_visible: false,
        });
    for id in ids {
        if listed.iter().any(|c| c.id == id) {
            let kind = reject_kind(&listed, id, None);
            lowered.operations.push(super::Operation {
                id: None,
                group: None,
                kind,
            });
        }
    }
    Ok(Some(lowered))
}

/// The visible reject of `revision`: a `restore` of a deletion or of a
/// move's source, a `strike` of anything else, an id `listed` lacks
/// included (the strike then refuses it as unknown).
pub(super) fn reject_kind(
    listed: &[Change],
    revision: String,
    comment: Option<String>,
) -> OperationKind {
    let restores = listed.iter().find(|c| c.id == revision).is_some_and(|c| {
        matches!(
            (c.kind, c.move_side),
            (ChangeKind::Deletion, _) | (ChangeKind::Move, Some(MoveSide::From))
        )
    });
    if restores {
        OperationKind::Restore { revision, comment }
    } else {
        OperationKind::Strike { revision, comment }
    }
}

/// The run content a struck or restored run may hold: text and the
/// characters typed beside it. Anything else (a field, a picture, a note
/// or comment mark) is not plain text.
const PLAIN: [&str; 10] = [
    "rPr",
    "t",
    "delText",
    "tab",
    "br",
    "cr",
    "sym",
    "noBreakHyphen",
    "softHyphen",
    "lastRenderedPageBreak",
];

/// Markers a revision may hold beside its runs; they are not its words.
const MARKERS: [&str; 10] = [
    "bookmarkStart",
    "bookmarkEnd",
    "commentRangeStart",
    "commentRangeEnd",
    "permStart",
    "permEnd",
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
];

/// `node`'s local name, empty for a text node.
fn local(dom: &Dom, node: NodeId) -> String {
    dom.name(node)
        .map(|n| n.local_name().to_string())
        .unwrap_or_default()
}

/// What in `run` is not plain text, as a refusal names it.
fn not_plain(dom: &Dom, run: NodeId) -> Option<&'static str> {
    dom.elements(run, None).into_iter().find_map(|child| {
        let name = local(dom, child);
        if PLAIN.contains(&name.as_str()) {
            return None;
        }
        Some(match name.as_str() {
            "fldChar" | "instrText" | "delInstrText" => "a field",
            "drawing" | "pict" | "object" => "a picture or an object",
            "footnoteReference" | "endnoteReference" => "a note reference",
            "commentReference" => "a comment's mark",
            _ => "content that is not plain text",
        })
    })
}

/// What among `container`'s children is not a run of plain text, a marker
/// or one of `nested`, as a refusal names it.
fn unsupported_child(dom: &Dom, container: NodeId, nested: &[&str]) -> Option<&'static str> {
    dom.elements(container, None)
        .into_iter()
        .find_map(|child| match local(dom, child).as_str() {
            "r" => not_plain(dom, child),
            "proofErr" => None,
            n if MARKERS.contains(&n) || nested.contains(&n) => None,
            "hyperlink" => Some("a hyperlink"),
            "fldSimple" => Some("a field"),
            "sdt" => Some("a content control"),
            _ => Some("content that is not plain text"),
        })
}

/// Whether a deletion whose parent is `parent` sits where restore puts
/// text back: in a paragraph, or in an insertion directly inside one.
fn in_paragraph(dom: &Dom, parent: NodeId) -> bool {
    dom.name_is(parent, &W::p()) || dom.name_is(container(dom, parent), &W::p())
}

/// What holds a deletion whose parent is `parent`: that parent, or past an
/// insertion the one holding it.
fn container(dom: &Dom, parent: NodeId) -> NodeId {
    if dom.name_is(parent, &W::ins()) {
        dom.parent(parent).unwrap_or(parent)
    } else {
        parent
    }
}

/// Whether `del` holds words to put back: deleted text, or a glyph typed
/// in one of its runs (a tab, a break, a symbol, a hyphen).
fn holds_words(dom: &Dom, del: NodeId) -> bool {
    dom.descendants(del, None).into_iter().any(|n| {
        let name = local(dom, n);
        let typed = matches!(
            name.as_str(),
            "tab" | "br" | "cr" | "sym" | "noBreakHyphen" | "softHyphen"
        ) && dom.parent(n).is_some_and(|p| dom.name_is(p, &W::r()));
        (name == "delText" && !dom.value(n).is_empty()) || typed
    })
}

/// Whether restore takes `del`, a `w:del` its revision spreads over
/// `stretches` places: one stretch of plain deleted words in a paragraph.
/// A refusal hint offers restore only where it would land, by the checks
/// restore itself makes.
pub(super) fn restorable(dom: &Dom, del: NodeId, stretches: usize) -> bool {
    stretches == 1
        && dom.name_is(del, &W::del())
        && dom.parent(del).is_some_and(|p| in_paragraph(dom, p))
        && holds_words(dom, del)
        && unsupported_child(dom, del, &[]).is_none()
}

/// Detached copies of `del`'s runs as inserted text: `w:delText` read as
/// `w:t`, the run formatting without another revision's marks.
pub(super) fn copy_runs(dom: &mut Dom, del: NodeId) -> Vec<NodeId> {
    let mut copies = Vec::new();
    for run in dom.elements(del, Some(&W::r())) {
        let copy = dom.clone_subtree(run);
        for child in dom.elements(copy, None) {
            if dom.name_is(child, &W::del_text()) {
                dom.set_name(child, W::t());
            } else if dom.name_is(child, &W::r_pr()) {
                let clean = super::tracked::clean_rpr(dom, child);
                dom.add_before_self(child, clean);
                dom.remove(child);
            }
        }
        copies.push(copy);
    }
    copies
}

/// The run-level revision `local` (`ins`, `del`) with `w:id` `wid` in
/// `paragraph`.
pub(super) fn revision_in(dom: &Dom, paragraph: NodeId, local: &str, wid: &str) -> Option<NodeId> {
    dom.descendants(paragraph, Some(&W::name(local)))
        .into_iter()
        .find(|&n| {
            dom.attribute(n, &W::id()) == Some(wid)
                && dom.parent(n).is_some_and(|p| !dom.name_is(p, &W::r_pr()))
        })
}

/// Comment `id` from `first` to `last`, siblings in that order: the start
/// marker before `first`, the end marker and the reference run after
/// `last`. Inside our insertion they go with it when it is rejected.
pub(super) fn comment_around(dom: &mut Dom, first: NodeId, last: NodeId, id: u32) {
    let id = id.to_string();
    let start = comment_marker(dom, "commentRangeStart", &id);
    let end = comment_marker(dom, "commentRangeEnd", &id);
    let reference = comment_reference_run(dom, &id);
    dom.add_before_self(first, start);
    dom.add_after_self(last, end);
    dom.add_after_self(end, reference);
}

/// The clean copy's restore: plain copies of their deletion `wid`'s runs
/// right after it, which stays. Returns the first and last copy.
pub(super) fn restore_plain(
    dom: &mut Dom,
    paragraph: NodeId,
    wid: &str,
) -> Option<(NodeId, NodeId)> {
    let del = revision_in(dom, paragraph, "del", wid)?;
    let copies = copy_runs(dom, del);
    let mut last = del;
    for &copy in &copies {
        dom.add_after_self(last, copy);
        last = copy;
    }
    Some((*copies.first()?, last))
}

/// The projection offset at `node` in paragraph `para`: the end of the
/// last projected piece before it.
fn offset_before(tx: &Transaction<'_>, para: usize, node: NodeId) -> usize {
    let dom = &tx.opened.dom;
    let order = dom.descendants(tx.paragraph_nodes[para], None);
    let index = |n: NodeId| order.iter().position(|&m| m == n);
    let at = index(node);
    tx.projections[para]
        .segments
        .iter()
        .filter(|s| index(run_of(&s.piece)) < at)
        .map(|s| s.end)
        .max()
        .unwrap_or(0)
}

type Failure = Box<(EditError, EditOutcome)>;

impl Transaction<'_> {
    /// Resolve a `strike` or a `restore` of `revision`: the text deletions
    /// that strike their insertion, or the copy that restores their
    /// deletion.
    pub(super) fn resolve_visible(
        &mut self,
        id: &str,
        visible: Visible,
        revision: &str,
        comment: Option<&str>,
    ) -> Result<(Vec<Resolved>, EditOutcome), Failure> {
        let mut outcome = EditOutcome {
            id: id.to_string(),
            kind: String::new(),
            status: String::new(),
            paragraph: None,
            matches: 0,
            context: None,
            comment_id: None,
            code: None,
            message: None,
            anchor_given: None,
            anchor_read_as: None,
            near: None,
        };
        let fail = |code: &str, message: String, outcome: &EditOutcome| {
            Box::new((err(code, Some(id), message), outcome.clone()))
        };
        let verb = visible.verb();
        let ids = crate::ids::parse_change_ids(revision.trim())
            .map_err(|e| fail("INVALID_EDIT", format!("{verb}: {e}"), &outcome))?;
        let [one] = ids.as_slice() else {
            let names: Vec<String> = ids.iter().map(crate::ids::ChangeId::short).collect();
            return Err(fail(
                "INVALID_EDIT",
                format!(
                    "{verb} takes one revision; {} names {} ({})",
                    revision.trim(),
                    names.len(),
                    names.join(", ")
                ),
                &outcome,
            ));
        };
        let short = one.short();
        let settled = &self.resolved_revisions;
        if settled.accepted.contains(&short) || settled.rejected.contains(&short) {
            return Err(fail(
                "REVISION_CONFLICT",
                format!(
                    "{short} is both settled by resolve_revisions and {} by {id}",
                    visible.past()
                ),
                &outcome,
            ));
        }
        if let Some(note) = comment {
            check_comment(note).map_err(|m| fail("INVALID_EDIT", m, &outcome))?;
        }
        let unknown = || format!("no tracked change {short}");
        let listed = crate::changes::list_changes(&self.base)
            .map_err(|e| fail("INVALID_DOCUMENT", e.to_string(), &outcome))?;
        let Some(change) = listed.into_iter().find(|c| c.id == short) else {
            return Err(fail("UNKNOWN_CHANGE", unknown(), &outcome));
        };
        // Our own revision first, whatever its kind: the way to undo it is
        // ours to take, not a strike or a restore.
        if let Some(author) = change.author.as_deref().filter(|&a| a == self.plan.author) {
            let instead = match (visible, change.kind) {
                (Visible::Strike, ChangeKind::Insertion) => "delete the text instead",
                _ => "reject it instead",
            };
            return Err(fail(
                "OWN_REVISION",
                format!("{short} is by {author:?}; {instead}"),
                &outcome,
            ));
        }
        let wrong_kind = match (visible, change.kind, change.move_side) {
            (Visible::Strike, ChangeKind::Insertion, _)
            | (Visible::Restore, ChangeKind::Deletion, _) => None,
            (Visible::Strike, ChangeKind::Deletion, _) => Some((
                "INVALID_EDIT",
                format!("{short} is a deletion, not an insertion: restore it instead"),
            )),
            (Visible::Restore, ChangeKind::Insertion, _) => Some((
                "INVALID_EDIT",
                format!("{short} is an insertion, not a deletion: strike it instead"),
            )),
            (Visible::Restore, ChangeKind::Move, Some(MoveSide::From)) => Some((
                "UNSUPPORTED_STRUCTURE",
                format!(
                    "{short} is the source of a move: restoring it would duplicate the moved words; reject the move ({short}) instead"
                ),
            )),
            (Visible::Strike, ChangeKind::Move, Some(MoveSide::To)) => Some((
                "UNSUPPORTED_STRUCTURE",
                format!(
                    "{short} is the destination of a move: striking it would lose the moved words; reject the move ({short}) instead"
                ),
            )),
            (_, ChangeKind::Move, _) => Some((
                "INVALID_EDIT",
                format!(
                    "{short} is a move, not {}: reject the move ({short}) instead",
                    match visible {
                        Visible::Strike => "an insertion",
                        Visible::Restore => "a deletion",
                    }
                ),
            )),
            (_, ChangeKind::Replacement, _) => Some((
                "INVALID_EDIT",
                format!(
                    "{short} is a replacement: strike its insertion or restore its deletion by its own id"
                ),
            )),
            (_, ChangeKind::Formatting, _) => Some((
                "INVALID_EDIT",
                format!(
                    "{short} is a formatting change; {verb} takes {} text: reject {short} to undo it (no visible reject of formatting yet; it leaves no trace)",
                    visible.takes()
                ),
            )),
        };
        if let Some((code, message)) = wrong_kind {
            return Err(fail(code, message, &outcome));
        }
        let unsupported = |message: String| fail("UNSUPPORTED_STRUCTURE", message, &outcome);
        match change.target {
            "text" => {}
            "paragraph_mark" => {
                let paragraph = change.paragraph.as_deref().unwrap_or("its paragraph");
                return Err(unsupported(match visible {
                    Visible::Strike => format!(
                        "{short} is an inserted paragraph mark; strike takes inserted text only: delete paragraph {paragraph} to strike it with its mark"
                    ),
                    Visible::Restore => format!(
                        "{short} is a deleted paragraph mark; restore takes deleted text only: reject {short} to put the paragraph break back"
                    ),
                }));
            }
            other => {
                return Err(unsupported(format!(
                    "{short} is {} {}; {verb} takes {} text only: reject {short} instead",
                    match visible {
                        Visible::Strike => "an inserted",
                        Visible::Restore => "a deleted",
                    },
                    other.replace('_', " "),
                    visible.takes()
                )));
            }
        }
        let Some(story) = self.stories.iter().position(|s| s.id == one.story) else {
            return Err(fail("UNKNOWN_CHANGE", unknown(), &outcome));
        };
        let local_name = match visible {
            Visible::Strike => "ins",
            Visible::Restore => "del",
        };
        let dom = &self.opened.dom;
        let nodes: Vec<NodeId> = dom
            .descendants(self.stories[story].root, Some(&W::name(local_name)))
            .into_iter()
            .filter(|&n| {
                dom.attribute(n, &W::id()) == Some(one.wid.as_str())
                    && dom.parent(n).is_some_and(|p| !dom.name_is(p, &W::r_pr()))
            })
            .collect();
        let paragraph_of = |node: NodeId| {
            dom.ancestors(node, Some(&W::p()))
                .first()
                .and_then(|p| self.paragraph_nodes.iter().position(|n| n == p))
        };
        let Some(first_para) = nodes.iter().find_map(|&n| paragraph_of(n)) else {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("{short} is not in a paragraph jubarte edits; reject {short} instead"),
                &outcome,
            ));
        };
        outcome.paragraph = Some(self.paragraph_id(first_para));
        let unsupported = |message: String| fail("UNSUPPORTED_STRUCTURE", message, &outcome);
        if comment.is_some() && self.paragraph_story[first_para].0 != 0 {
            return Err(fail(
                "COMMENT_NOT_IN_BODY",
                "comments are supported in the body only: Word cannot anchor one in a header, footer or note".into(),
                &outcome,
            ));
        }
        match visible {
            Visible::Strike => {
                let mut resolved = Vec::new();
                let mut struck = String::new();
                let mut commented = None;
                for &ins in &nodes {
                    if let Some(what) = unsupported_child(dom, ins, &["del"]) {
                        return Err(unsupported(format!(
                            "{short} holds {what}; strike takes plain inserted text only: reject {short} instead, or comment on it"
                        )));
                    }
                    let Some(para) = paragraph_of(ins) else {
                        return Err(unsupported(format!(
                            "{short} is not in a paragraph jubarte edits; reject {short} instead"
                        )));
                    };
                    let projection = &self.projections[para];
                    // One range per stretch of adjacent runs: another
                    // author's deletion between two runs stays outside ours.
                    let mut ranges: Vec<(usize, usize, NodeId)> = Vec::new();
                    for segment in &projection.segments {
                        let run = run_of(&segment.piece);
                        if dom.parent(run) != Some(ins) {
                            continue;
                        }
                        if editable_insertion(dom, &segment.piece) != Some(ins) {
                            return Err(unsupported(format!(
                                "{short} holds a field's result; strike takes plain inserted text only: reject {short} instead, or comment on it"
                            )));
                        }
                        match ranges.last_mut() {
                            Some((_, end, last))
                                if *end == segment.start
                                    && (*last == run || dom.next_element(*last) == Some(run)) =>
                            {
                                *end = segment.end;
                                *last = run;
                            }
                            _ => ranges.push((segment.start, segment.end, run)),
                        }
                    }
                    let (Some(first), Some(last)) = (ranges.first(), ranges.last()) else {
                        continue;
                    };
                    if comment.is_some() && commented.is_none() {
                        commented = Some(Resolved::StrikeComment {
                            para,
                            start: first.0,
                            end: last.1,
                        });
                    }
                    for &(start, end, _) in &ranges {
                        struck.push_str(&projection.text[start..end]);
                        resolved.push(Resolved::Text {
                            para,
                            start,
                            end,
                            replacement: String::new(),
                            comment: None,
                            attach_before: false,
                            format: None,
                        });
                    }
                }
                if resolved.is_empty() {
                    outcome.message = Some("already struck".into());
                    return Ok((Vec::new(), outcome));
                }
                outcome.matches = 1;
                outcome.context = Some(format!("{{-struck {:?}}}", excerpt(&struck, 60)));
                resolved.extend(commented);
                Ok((resolved, outcome))
            }
            Visible::Restore => {
                let &[del] = nodes.as_slice() else {
                    return Err(unsupported(format!(
                        "{short} is split over several places; restore takes one deleted stretch: reject {short} instead"
                    )));
                };
                let parent = dom.parent(del).expect("a run-level deletion has a parent");
                if !in_paragraph(dom, parent) {
                    let inside = match local(dom, container(dom, parent)).as_str() {
                        "hyperlink" => "a hyperlink",
                        "sdtContent" => "a content control",
                        "fldSimple" => "a field",
                        _ => "markup that is not a paragraph",
                    };
                    return Err(unsupported(format!(
                        "{short} sits inside {inside}; restore takes deleted text in a paragraph only: reject {short} instead"
                    )));
                }
                let marks: Vec<String> = dom
                    .descendants(del, Some(&W::comment_reference()))
                    .into_iter()
                    .filter_map(|r| dom.attribute(r, &W::id()).map(|n| format!("c{n}")))
                    .collect();
                if !holds_words(dom, del) {
                    let message = match marks.as_slice() {
                        [] => format!("{short} holds no text; reject {short} instead"),
                        [one] => format!(
                            "{short} holds no text, only comment {one}'s mark; reject {short} to keep {one}"
                        ),
                        many => format!(
                            "{short} holds no text, only the marks of comments {}; reject {short} to keep them",
                            many.join(", ")
                        ),
                    };
                    return Err(fail("EMPTY_REVISION", message, &outcome));
                }
                if let Some(what) = unsupported_child(dom, del, &[]) {
                    return Err(unsupported(format!(
                        "{short} holds {what}; restore takes plain deleted text only: reject {short} instead"
                    )));
                }
                let at = offset_before(self, first_para, del);
                // The copy's text as the projection reads it, so the
                // postcondition expects exactly what lands.
                let dom = &mut self.opened.dom;
                let holder = dom.new_element(W::p());
                for copy in copy_runs(dom, del) {
                    dom.add(holder, copy);
                }
                let text = project_paragraph(dom, holder).text;
                outcome.matches = 1;
                outcome.context = Some(format!("{{+restored {:?}}}", excerpt(&text, 60)));
                Ok((
                    vec![Resolved::Restore {
                        para: first_para,
                        wid: one.wid.clone(),
                        at,
                        text,
                        comment: comment.map(str::to_string),
                    }],
                    outcome,
                ))
            }
        }
    }
}
