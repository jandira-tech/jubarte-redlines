// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

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
    EditError, EditOutcome, EditPlan, ExistingRevisions, OperationKind, Resolved, Side,
    Transaction, check_comment, comment_marker, comment_reference_run, editable_insertion, err,
    excerpt, run_of,
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
pub(super) fn lower(
    source: &[u8],
    plan: &EditPlan,
    syntax: super::PlanSyntax,
) -> Result<Option<EditPlan>, EditError> {
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
        crate::changes::check_whole_changes(source, reject)
            .map_err(|e| super::selection_error(e, syntax, "reject"))?;
    }
    let listed = crate::changes::list_changes(source)
        .map_err(|e| super::selection_error(e, syntax, "reject"))?;
    let ids = super::selected_ids(&listed, selection.reject.as_ref(), syntax)?;
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

/// Whether the visible reject of `revision` is a `restore`: of a deletion
/// or of a move's source. An id `listed` lacks is struck (the strike then
/// refuses it as unknown).
fn restores(listed: &[Change], revision: &str) -> bool {
    listed.iter().find(|c| c.id == revision).is_some_and(|c| {
        matches!(
            (c.kind, c.move_side),
            (ChangeKind::Deletion, _) | (ChangeKind::Move, Some(MoveSide::From))
        )
    })
}

/// The visible reject of `revision`: a `restore` of a deletion or of a
/// move's source, a `strike` of anything else.
pub(super) fn reject_kind(
    listed: &[Change],
    revision: String,
    comment: Option<String>,
) -> OperationKind {
    if restores(listed, &revision) {
        OperationKind::Restore { revision, comment }
    } else {
        OperationKind::Strike { revision, comment }
    }
}

/// The visible reject of the change the marks `ids` make: one operation
/// per kind ([`reject_kind`]), the kind of the first mark first, and
/// `comment` on that one.
pub(super) fn reject_kinds(
    listed: &[Change],
    ids: &[String],
    comment: Option<String>,
) -> Vec<OperationKind> {
    let (back, out): (Vec<String>, Vec<String>) =
        ids.iter().cloned().partition(|id| restores(listed, id));
    let mut kinds = [(true, back), (false, out)];
    if !ids.first().is_some_and(|id| restores(listed, id)) {
        kinds.swap(0, 1);
    }
    let mut comment = comment;
    kinds
        .into_iter()
        .filter(|(_, marks)| !marks.is_empty())
        .map(|(restore, marks)| {
            let revision = crate::changes::composite_id(&marks);
            let comment = comment.take();
            if restore {
                OperationKind::Restore { revision, comment }
            } else {
                OperationKind::Strike { revision, comment }
            }
        })
        .collect()
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

/// The refusal of a comment outside the body.
const BODY_ONLY: &str =
    "comments are supported in the body only: Word cannot anchor one in a header, footer or note";

impl Transaction<'_> {
    /// Resolve a `strike` or a `restore` of `revision`, a change as
    /// `changes` lists it: the text deletions that strike its insertion,
    /// or the copy that restores its deletion.
    pub(super) fn resolve_visible(
        &mut self,
        id: &str,
        visible: Visible,
        revision: &str,
        comment: Option<&str>,
    ) -> Result<(Vec<Resolved>, EditOutcome), Failure> {
        let outcome = EditOutcome {
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
            candidates: Vec::new(),
        };
        let fail =
            |code: &str, message: String| Box::new((err(code, Some(id), message), outcome.clone()));
        let verb = visible.verb();
        let ids = crate::ids::parse_change_ids(revision.trim())
            .map_err(|e| fail("INVALID_EDIT", format!("{verb}: {e}")))?;
        if let Some(note) = comment {
            check_comment(note).map_err(|m| fail("INVALID_EDIT", m))?;
        }
        let listed = crate::changes::list_changes(&self.base)
            .map_err(|e| fail("INVALID_DOCUMENT", e.to_string()))?;
        match ids.as_slice() {
            [one] => self.resolve_mark(id, visible, one, &listed, comment, outcome),
            marks => self.resolve_marks(id, visible, marks, &listed, comment, outcome),
        }
    }

    /// The refusal of `change` when it is the plan author's own: the way to
    /// undo it is theirs to take, not a strike or a restore.
    fn own_revision(&self, visible: Visible, change: &Change) -> Option<String> {
        let author = change
            .author
            .as_deref()
            .filter(|&a| a == self.plan.author)?;
        let instead = match (visible, change.kind) {
            (Visible::Strike, ChangeKind::Insertion) => "delete the text instead",
            _ => "reject it instead",
        };
        Some(format!("{} is by {author:?}; {instead}", change.id))
    }

    /// Resolve a `strike` or a `restore` of a change of several `marks`
    /// (`r10+11`): each mark of the operation's kind as it would be alone,
    /// in reading order, and one comment over the answer. The marks of the
    /// other kind (a replacement's deletion under a strike) stay as they
    /// are, and the outcome says so.
    fn resolve_marks(
        &mut self,
        id: &str,
        visible: Visible,
        marks: &[crate::ids::ChangeId],
        listed: &[Change],
        comment: Option<&str>,
        mut outcome: EditOutcome,
    ) -> Result<(Vec<Resolved>, EditOutcome), Failure> {
        let fail = |code: &str, message: String, outcome: &EditOutcome| {
            Box::new((err(code, Some(id), message), outcome.clone()))
        };
        let shorts: Vec<String> = marks.iter().map(crate::ids::ChangeId::short).collect();
        let revision = crate::changes::composite_id(&shorts);
        // A mark named twice would be answered twice: its words struck or
        // restored once per mention.
        if let Some(twice) = shorts
            .iter()
            .enumerate()
            .find_map(|(i, s)| shorts[..i].contains(s).then_some(s))
        {
            return Err(fail(
                "INVALID_EDIT",
                format!("{revision} names {twice} twice"),
                &outcome,
            ));
        }
        let (noun, opposite, other, other_verb) = match visible {
            Visible::Strike => ("insertion", ChangeKind::Deletion, "deletion", "restore"),
            Visible::Restore => ("deletion", ChangeKind::Insertion, "insertion", "strike"),
        };
        // Every mark is known and none is ours, whichever its kind.
        let mut answered = Vec::new();
        let mut left = Vec::new();
        for (mark, short) in marks.iter().zip(&shorts) {
            let Some(at) = listed.iter().position(|c| &c.id == short) else {
                return Err(fail(
                    "UNKNOWN_CHANGE",
                    format!("no tracked change {short}"),
                    &outcome,
                ));
            };
            if let Some(message) = self.own_revision(visible, &listed[at]) {
                return Err(fail("OWN_REVISION", message, &outcome));
            }
            if listed[at].kind == opposite {
                left.push(short.clone());
            } else {
                answered.push((at, mark));
            }
        }
        if answered.is_empty() {
            return Err(fail(
                "INVALID_EDIT",
                format!("{revision} holds no {noun}: {other_verb} it instead"),
                &outcome,
            ));
        }
        answered.sort_by_key(|&(at, _)| at);
        let mut resolved = Vec::new();
        let mut paragraphs: Vec<String> = Vec::new();
        let mut contexts = Vec::new();
        let mut done = Vec::new();
        for (_, mark) in answered {
            let (answer, said) =
                self.resolve_mark(id, visible, mark, listed, comment, outcome.clone())?;
            if let Some(paragraph) = said.paragraph
                && !paragraphs.contains(&paragraph)
            {
                paragraphs.push(paragraph);
            }
            if !answer.is_empty() {
                done.push(mark.short());
                contexts.extend(said.context);
            }
            resolved.extend(answer);
        }
        outcome.paragraph = paragraphs.first().cloned();
        if comment.is_some()
            && let [first, .., last] = paragraphs.as_slice()
        {
            let between = &paragraphs[1..paragraphs.len() - 1];
            let named = between
                .iter()
                .fold(first.clone(), |all, p| format!("{all}, {p}"));
            return Err(fail(
                "INVALID_EDIT",
                format!(
                    "{revision} has marks in {named} and {last}; a comment answers one paragraph's: give each paragraph's marks their own {}",
                    visible.verb()
                ),
                &outcome,
            ));
        }
        // The other kind's marks stay, whether or not this answer did
        // anything: a second strike of a replacement still names its deletion.
        let rest = (!left.is_empty()).then(|| {
            format!(
                "{} is its {other}: {other_verb} it too to reject the whole change",
                crate::changes::composite_id(&left)
            )
        });
        if resolved.is_empty() {
            outcome.message = Some(match rest {
                Some(rest) => format!("already struck; {rest}"),
                None => "already struck".into(),
            });
            return Ok((Vec::new(), outcome));
        }
        outcome.matches = 1;
        outcome.context = Some(contexts.join(" "));
        if let Some(rest) = rest {
            outcome.message = Some(format!(
                "{} {}; {rest}",
                visible.past(),
                crate::changes::composite_id(&done),
            ));
        }
        Ok((one_comment(resolved), outcome))
    }

    /// Resolve a `strike` or a `restore` of the single mark `one`.
    fn resolve_mark(
        &mut self,
        id: &str,
        visible: Visible,
        one: &crate::ids::ChangeId,
        listed: &[Change],
        comment: Option<&str>,
        mut outcome: EditOutcome,
    ) -> Result<(Vec<Resolved>, EditOutcome), Failure> {
        let fail = |code: &str, message: String, outcome: &EditOutcome| {
            Box::new((err(code, Some(id), message), outcome.clone()))
        };
        let verb = visible.verb();
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
        let unknown = || format!("no tracked change {short}");
        let Some(change) = listed.iter().find(|c| c.id == short) else {
            return Err(fail("UNKNOWN_CHANGE", unknown(), &outcome));
        };
        // Our own revision first, whatever its kind.
        if let Some(message) = self.own_revision(visible, change) {
            return Err(fail("OWN_REVISION", message, &outcome));
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
                    "{short} is the source of a move: restoring it would duplicate the moved words; comment on it instead"
                ),
            )),
            (Visible::Strike, ChangeKind::Move, Some(MoveSide::To)) => Some((
                "UNSUPPORTED_STRUCTURE",
                format!(
                    "{short} is the destination of a move: striking it would lose the moved words; comment on it instead"
                ),
            )),
            (_, ChangeKind::Move, _) => Some((
                "INVALID_EDIT",
                format!(
                    "{short} is a move, not {}: comment on it instead",
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
                    "{short} is a formatting change; {verb} takes {} text: comment on it instead",
                    visible.takes()
                ),
            )),
        };
        if let Some((code, message)) = wrong_kind {
            return Err(fail(code, message, &outcome));
        }
        let unsupported = |message: String| fail("UNSUPPORTED_STRUCTURE", message, &outcome);
        match change.target {
            "text" | "paragraph_mark" => {}
            other => {
                return Err(unsupported(format!(
                    "{short} is {} {}; {verb} takes {} text only: comment on it instead",
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
        if change.target == "paragraph_mark" {
            return self.resolve_visible_mark(id, visible, story, one, comment, outcome);
        }
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
                format!("{short} is not in a paragraph jubarte edits"),
                &outcome,
            ));
        };
        outcome.paragraph = Some(self.paragraph_id(first_para));
        let unsupported = |message: String| fail("UNSUPPORTED_STRUCTURE", message, &outcome);
        if comment.is_some() && self.paragraph_story[first_para].0 != 0 {
            return Err(fail("COMMENT_NOT_IN_BODY", BODY_ONLY.into(), &outcome));
        }
        match visible {
            Visible::Strike => {
                let mut resolved = Vec::new();
                let mut struck = String::new();
                let mut commented = None;
                for &ins in &nodes {
                    if let Some(what) = unsupported_child(dom, ins, &["del"]) {
                        return Err(unsupported(format!(
                            "{short} holds {what}; strike takes plain inserted text only: comment on it instead"
                        )));
                    }
                    let Some(para) = paragraph_of(ins) else {
                        return Err(unsupported(format!(
                            "{short} is not in a paragraph jubarte edits"
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
                                "{short} holds a field's result; strike takes plain inserted text only: comment on it instead"
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
                        "{short} is split over several places; restore takes one deleted stretch: comment on it instead"
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
                        "{short} sits inside {inside}; restore takes deleted text in a paragraph only: comment on it instead"
                    )));
                }
                let marks: Vec<String> = dom
                    .descendants(del, Some(&W::comment_reference()))
                    .into_iter()
                    .filter_map(|r| dom.attribute(r, &W::id()).map(|n| format!("c{n}")))
                    .collect();
                if !holds_words(dom, del) {
                    let message = match marks.as_slice() {
                        [] => format!("{short} holds no text: nothing to restore"),
                        [one] => format!(
                            "{short} holds no text, only comment {one}'s mark: nothing to restore"
                        ),
                        many => format!(
                            "{short} holds no text, only the marks of comments {}: nothing to restore",
                            many.join(", ")
                        ),
                    };
                    return Err(fail("EMPTY_REVISION", message, &outcome));
                }
                if let Some(what) = unsupported_child(dom, del, &[]) {
                    return Err(unsupported(format!(
                        "{short} holds {what}; restore takes plain deleted text only: comment on it instead"
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

    /// Resolve a `strike` or a `restore` of paragraph mark `one`, in
    /// `story`. A strike joins its paragraph to the next one: our deletion
    /// joins their insertion in the mark. A restore puts right after its
    /// paragraph an empty one of ours, with its properties and mark
    /// formatting, whose mark is our insertion. Accepted, either reads as
    /// their change rejected. A comment sits on the last word before the
    /// break.
    fn resolve_visible_mark(
        &self,
        id: &str,
        visible: Visible,
        story: usize,
        one: &crate::ids::ChangeId,
        comment: Option<&str>,
        mut outcome: EditOutcome,
    ) -> Result<(Vec<Resolved>, EditOutcome), Failure> {
        let short = one.short();
        let fail = |code: &str, message: String, outcome: &EditOutcome| {
            Box::new((err(code, Some(id), message), outcome.clone()))
        };
        let dom = &self.opened.dom;
        let local_name = match visible {
            Visible::Strike => "ins",
            Visible::Restore => "del",
        };
        let in_mark = |n: NodeId| {
            dom.parent(n).is_some_and(|rpr| {
                dom.name_is(rpr, &W::r_pr())
                    && dom
                        .parent(rpr)
                        .is_some_and(|ppr| dom.name_is(ppr, &W::p_pr()))
            })
        };
        let Some(para) = dom
            .descendants(self.stories[story].root, Some(&W::name(local_name)))
            .into_iter()
            .filter(|&n| dom.attribute(n, &W::id()) == Some(one.wid.as_str()) && in_mark(n))
            .find_map(|n| {
                let p = *dom.ancestors(n, Some(&W::p())).first()?;
                self.paragraph_nodes.iter().position(|&m| m == p)
            })
        else {
            return Err(fail(
                "UNSUPPORTED_STRUCTURE",
                format!("{short} is not in a paragraph jubarte edits"),
                &outcome,
            ));
        };
        outcome.paragraph = Some(self.paragraph_id(para));
        if comment.is_some() && self.paragraph_story[para].0 != 0 {
            return Err(fail("COMMENT_NOT_IN_BODY", BODY_ONLY.into(), &outcome));
        }
        let unsupported = |message: String| fail("UNSUPPORTED_STRUCTURE", message, &outcome);
        let what = match visible {
            Visible::Strike => "an inserted",
            Visible::Restore => "a deleted",
        };
        let node = self.paragraph_nodes[para];
        let ppr = dom.element(node, &W::p_pr());
        if ppr.is_some_and(|ppr| dom.element(ppr, &W::sect_pr()).is_some()) {
            let cannot = match visible {
                Visible::Strike => "striking it would drop the section break",
                Visible::Restore => "restore cannot put a section break back",
            };
            return Err(unsupported(format!(
                "{short} is {what} paragraph mark that ends a section; {cannot}: comment on it instead"
            )));
        }
        let Ok(next) = self.merge_partner(para) else {
            let only = match visible {
                Visible::Strike => "strike joins two paragraphs only",
                Visible::Restore => "restore parts two paragraphs only",
            };
            return Err(unsupported(format!(
                "{short} is {what} paragraph mark with no paragraph right after it; {only}: comment on it instead"
            )));
        };
        let struck = ppr
            .and_then(|ppr| dom.element(ppr, &W::r_pr()))
            .is_some_and(|rpr| dom.element(rpr, &W::del()).is_some());
        if visible == Visible::Strike && struck {
            outcome.message = Some("already struck".into());
            return Ok((Vec::new(), outcome));
        }
        let text = &self.projections[para].text;
        let chars: Vec<char> = text.chars().collect();
        let tail: String = chars[chars.len().saturating_sub(20)..].iter().collect();
        let (edit, mark) = match visible {
            Visible::Strike => (
                Resolved::MergeParagraphs {
                    para,
                    next,
                    separator: String::new(),
                    strike: true,
                },
                "{-¶}",
            ),
            Visible::Restore => (
                Resolved::InsertParagraph {
                    anchor: para,
                    side: Side::After,
                    runs: Vec::new(),
                    like: para,
                    style: None,
                    comment: None,
                    toc: None,
                    keep_mark: true,
                },
                "{+¶}",
            ),
        };
        outcome.matches = 1;
        outcome.context = Some(format!(
            "{tail}{mark}{}",
            excerpt(&self.projections[next].text, 20)
        ));
        // The last word before the break, or the break itself in an empty
        // paragraph.
        let words = text.trim_end();
        let start = words
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace())
            .map_or(0, |(i, c)| i + c.len_utf8());
        let comment = comment.map(|note| Resolved::CommentRange {
            para,
            start,
            end: words.len(),
            text: note.to_string(),
        });
        Ok((std::iter::once(edit).chain(comment).collect(), outcome))
    }
}

/// `resolved`, the answers to the marks of one change, with one comment:
/// the strikes' comments joined into one from the first struck word to the
/// last, only the first restore's (the copies of all of them anchor it),
/// and a paragraph mark's only when no words carry one.
fn one_comment(resolved: Vec<Resolved>) -> Vec<Resolved> {
    let words = resolved.iter().any(|r| {
        matches!(
            r,
            Resolved::StrikeComment { .. }
                | Resolved::Restore {
                    comment: Some(_),
                    ..
                }
        )
    });
    let mut out: Vec<Resolved> = Vec::with_capacity(resolved.len());
    let mut struck: Option<usize> = None;
    let mut restored = false;
    let mut marked = false;
    for r in resolved {
        match r {
            Resolved::StrikeComment { para, start, end } => match struck {
                Some(at) => {
                    if let Resolved::StrikeComment {
                        start: first,
                        end: last,
                        ..
                    } = &mut out[at]
                    {
                        *first = (*first).min(start);
                        *last = (*last).max(end);
                    }
                }
                None => {
                    struck = Some(out.len());
                    out.push(Resolved::StrikeComment { para, start, end });
                }
            },
            Resolved::Restore {
                para,
                wid,
                at,
                text,
                comment,
            } => {
                let comment = comment.filter(|_| !restored);
                restored |= comment.is_some();
                out.push(Resolved::Restore {
                    para,
                    wid,
                    at,
                    text,
                    comment,
                });
            }
            Resolved::CommentRange { .. } if words || marked => {}
            r @ Resolved::CommentRange { .. } => {
                marked = true;
                out.push(r);
            }
            r => out.push(r),
        }
    }
    out
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod visible_branch_tests {
    use super::super::deeper_boundary_fixture::{Part, docx, docx_with_sect, part_string};
    use super::super::{EditResult, apply_plan};
    use super::*;

    const INS: &str = r#"<w:ins w:id="1" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:t>added words</w:t></w:r></w:ins>"#;
    const DEL: &str = r#"<w:del w:id="2" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:rPr><w:b/><w:ins w:id="8" w:author="Bo" w:date="2026-01-01T00:00:00Z"/></w:rPr><w:delText>old</w:delText><w:tab/><w:delText>words</w:delText></w:r></w:del>"#;

    fn body() -> String {
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">Lead </w:t></w:r>{INS}<w:r><w:t xml:space="preserve"> mid </w:t></w:r>{DEL}<w:r><w:t xml:space="preserve"> end.</w:t></w:r></w:p>"#
        )
    }

    fn plan(extra: serde_json::Value) -> EditPlan {
        let mut value = serde_json::json!({
            "schema_version": 1,
            "author": "Ed",
            "date": "2026-02-02T00:00:00Z",
            "existing_revisions": "keep",
            "operations": [],
        });
        let serde_json::Value::Object(extra) = extra else {
            panic!("plan fields are an object");
        };
        for (k, v) in extra {
            value[k.as_str()] = v;
        }
        EditPlan::from_json(&value.to_string()).unwrap()
    }

    fn ok(source: &[u8], extra: serde_json::Value) -> EditResult {
        apply_plan(source, &plan(extra)).unwrap()
    }

    fn refused(source: &[u8], extra: serde_json::Value) -> EditError {
        match apply_plan(source, &plan(extra)) {
            Ok(_) => panic!("expected a refusal"),
            Err(e) => e,
        }
    }

    fn document(result: &EditResult) -> String {
        part_string(&result.redline, "word/document.xml").unwrap()
    }

    #[test]
    fn a_visible_reject_lowers_to_a_strike_and_a_restore() {
        let source = docx(&body());
        let result = ok(
            &source,
            serde_json::json!({"resolve_revisions": {"reject": {"ids": ["r1", "r2"], "visible": true}}}),
        );
        let xml = document(&result);
        // Ed's deletion sits inside Bo's insertion; Ed's insertion copies
        // Bo's deleted words, the tab included, and keeps the bold.
        assert!(xml.contains(r#"w:author="Ed""#), "{xml}");
        let changes = crate::changes::list_changes(&result.redline).unwrap();
        let ed: Vec<_> = changes
            .iter()
            .filter(|c| c.author.as_deref() == Some("Ed"))
            .collect();
        assert!(
            ed.iter()
                .any(|c| c.kind == ChangeKind::Deletion && c.text.contains("added words")),
            "{ed:?}"
        );
        assert!(
            ed.iter()
                .any(|c| c.kind == ChangeKind::Insertion && c.text.contains("old")),
            "{ed:?}"
        );
        // Bo's changes stay.
        assert!(changes.iter().any(|c| c.author.as_deref() == Some("Bo")));
    }

    #[test]
    fn a_visible_reject_needs_keep_and_known_ids() {
        let source = docx(&body());
        let e = refused(
            &source,
            serde_json::json!({
                "existing_revisions": "refuse",
                "resolve_revisions": {"reject": {"ids": ["r1"], "visible": true}}
            }),
        );
        assert_eq!(e.code, "INVALID_PLAN");
        assert!(
            e.message.contains("needs existing_revisions keep"),
            "{}",
            e.message
        );
        let e = refused(
            &source,
            serde_json::json!({"resolve_revisions": {"reject": {"ids": ["r9"], "visible": true}}}),
        );
        assert_eq!(e.code, "UNKNOWN_CHANGE", "{}", e.message);
        // A visible reject that also accepts keeps the accept side.
        let both = docx(&format!(
            r#"{}<w:p><w:ins w:id="3" w:author="Cy" w:date="2026-01-01T00:00:00Z"><w:r><w:t>more</w:t></w:r></w:ins></w:p>"#,
            body()
        ));
        let result = ok(
            &both,
            serde_json::json!({"resolve_revisions": {
                "accept": {"ids": ["r3"]},
                "reject": {"ids": ["r1"], "visible": true}
            }}),
        );
        assert_eq!(result.report.resolved_revisions.accepted, ["r3"]);
        assert!(result.report.resolved_revisions.rejected.is_empty());
    }

    #[test]
    fn strike_and_restore_operations_take_comments() {
        let source = docx(&body());
        let result = ok(
            &source,
            serde_json::json!({"operations": [
                {"kind":"strike","revision":"r1","comment":"not agreed"},
                {"kind":"restore","revision":"r2","comment":"keep these"}
            ]}),
        );
        let comments = crate::comments::list_comments(&result.redline).unwrap();
        let texts: Vec<&str> = comments.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"not agreed"), "{texts:?}");
        assert!(texts.contains(&"keep these"), "{texts:?}");
        assert!(comments.iter().all(|c| c.author == "Ed"));
    }

    #[test]
    fn strike_and_restore_refuse_the_wrong_kind_and_content() {
        let source = docx(&body());
        for (op, said) in [
            (
                serde_json::json!({"kind":"strike","revision":"r2"}),
                "r2 is a deletion, not an insertion",
            ),
            (
                serde_json::json!({"kind":"restore","revision":"r1"}),
                "r1 is an insertion, not a deletion",
            ),
        ] {
            let e = refused(&source, serde_json::json!({"operations": [op]}));
            assert_eq!(e.code, "INVALID_EDIT");
            assert!(e.message.starts_with(said), "{}", e.message);
        }
        // A deletion holding a field is no plain text to restore.
        let field = docx(
            r#"<w:p><w:r><w:t>a </w:t></w:r><w:del w:id="4" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:delText xml:space="preserve">see </w:delText></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText>PAGE</w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del></w:p>"#,
        );
        let e = refused(
            &field,
            serde_json::json!({"operations": [{"kind":"restore","revision":"r4"}]}),
        );
        assert!(e.message.contains("a field"), "{}", e.message);
        // A deletion with no words has nothing to restore.
        let empty = docx(
            r#"<w:p><w:r><w:t>a</w:t></w:r><w:del w:id="4" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del></w:p>"#,
        );
        let e = refused(
            &empty,
            serde_json::json!({"operations": [{"kind":"restore","revision":"r4"}]}),
        );
        assert_eq!(e.message, "r4 holds no text: nothing to restore");
        // A plan that does not keep revisions refuses a strike outright.
        let e = refused(
            &source,
            serde_json::json!({
                "existing_revisions": "accept",
                "operations": [{"kind":"strike","revision":"r1"}]
            }),
        );
        assert_eq!(e.code, "INVALID_PLAN");
        assert!(e.message.contains("already be settled"), "{}", e.message);
    }

    #[test]
    fn a_header_strike_takes_no_comment() {
        let header = format!(
            r#"<w:hdr xmlns:w="{}"><w:p><w:r><w:t xml:space="preserve">Head </w:t></w:r>{INS}</w:p></w:hdr>"#,
            W::URI
        );
        let source = docx_with_sect(
            "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        );
        let e = refused(
            &source,
            serde_json::json!({"operations": [{"kind":"strike","revision":"header1.r1","comment":"no"}]}),
        );
        assert_eq!(e.code, "COMMENT_NOT_IN_BODY");
        let result = ok(
            &source,
            serde_json::json!({"operations": [{"kind":"strike","revision":"header1.r1"}]}),
        );
        let header = part_string(&result.redline, "word/header1.xml").unwrap();
        assert!(header.contains(r#"w:author="Ed""#), "{header}");
    }

    #[test]
    fn restored_words_only_hold_plain_glyphs() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:p xmlns:w="{}"><w:del w:id="1"><w:r><w:delText/></w:r></w:del><w:del w:id="2"><w:r><w:tab/></w:r></w:del><w:del w:id="3"><w:bookmarkStart w:id="0" w:name="b"/><w:r><w:delText>x</w:delText></w:r></w:del><w:ins w:id="4"><w:del w:id="5"><w:r><w:delText>y</w:delText></w:r></w:del></w:ins><w:del w:id="6"><w:hyperlink><w:r><w:delText>z</w:delText></w:r></w:hyperlink></w:del><w:del w:id="7"><w:sdt/></w:del><w:del w:id="8"><w:fldSimple/></w:del><w:del w:id="9"><w:r><w:drawing/></w:r></w:del></w:p>"#,
            W::URI
        ));
        let p = dom.root(doc).unwrap();
        let dels = dom.descendants(p, Some(&W::del()));
        let ok: Vec<bool> = dels.iter().map(|&d| restorable(&dom, d, 1)).collect();
        assert_eq!(ok, [false, true, true, true, false, false, false, false]);
        assert!(!restorable(&dom, dels[2], 2));
        assert_eq!(unsupported_child(&dom, dels[4], &[]), Some("a hyperlink"));
        assert_eq!(
            unsupported_child(&dom, dels[5], &[]),
            Some("a content control")
        );
        assert_eq!(unsupported_child(&dom, dels[6], &[]), Some("a field"));
        assert_eq!(
            unsupported_child(&dom, dels[7], &[]),
            Some("a picture or an object")
        );
    }
}
