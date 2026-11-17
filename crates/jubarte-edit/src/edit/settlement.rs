// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `resolve_revisions` settlement: selecting, accepting and rejecting
//! tracked changes before a plan runs, and the comments it would remove.

use super::*;

/// An unopenable source: admission refusals keep their own code
/// (`INPUT_LIMIT`, …), anything else is `INVALID_DOCUMENT`.
pub(super) fn open_error(error: crate::inspect::InspectError) -> EditError {
    match error {
        crate::inspect::InspectError::Admission(refused) => {
            err(refused.code(), None, refused.message)
        }
        other => err("INVALID_DOCUMENT", None, other.to_string()),
    }
}

/// A [`ChangeError`] of a `resolve_revisions` selection as the plan's error.
pub(super) fn selection_error(e: ChangeError, syntax: PlanSyntax, verb: &str) -> EditError {
    match e {
        ChangeError::UnknownChange(id) => err(
            "UNKNOWN_CHANGE",
            None,
            syntax.selection_message(format_args!("no tracked change {id}")),
        ),
        ChangeError::InvalidId(m) => err("INVALID_PLAN", None, format!("resolve_revisions: {m}")),
        ambiguous @ ChangeError::Ambiguous { .. } => err(
            "AMBIGUOUS_CHANGE",
            None,
            syntax.selection_message(ambiguous),
        ),
        part @ ChangeError::PartOfChange { .. } => err(
            "INCOMPLETE_CHANGE",
            None,
            syntax.incomplete_change(&part, verb),
        ),
        ChangeError::Package(m) => err("INVALID_DOCUMENT", None, m),
        exposed @ ChangeError::ExposedFinalMark { .. } => {
            err("EXPOSED_FINAL_MARK", None, exposed.to_string())
        }
    }
}

/// The ids of `listed` a `resolve_revisions` side selects, in document
/// order, a move's other side included.
pub(super) fn selected_ids(
    listed: &[Change],
    filter: Option<&ChangeFilter>,
    syntax: PlanSyntax,
) -> Result<Vec<String>, EditError> {
    let Some(filter) = filter else {
        return Ok(Vec::new());
    };
    let selection_error = |e| selection_error(e, syntax, "");
    let filter = &filter.normalized().map_err(selection_error)?;
    filter.check(listed).map_err(selection_error)?;
    let hit: Vec<&Change> = listed.iter().filter(|c| filter.matches(c)).collect();
    Ok(listed
        .iter()
        .filter(|c| {
            hit.iter()
                .any(|h| h.id == c.id || h.move_name.is_some() && h.move_name == c.move_name)
        })
        .map(|c| c.id.clone())
        .collect())
}

/// The ids `filter` itself selects in `listed`, before [`selected_ids`]
/// adds each move's other side: the ones a receipt names as resolved.
fn named_ids(listed: &[Change], filter: Option<&ChangeFilter>) -> Vec<String> {
    let Some(filter) = filter.and_then(|f| f.normalized().ok()) else {
        return Vec::new();
    };
    listed
        .iter()
        .filter(|c| filter.matches(c))
        .map(|c| c.id.clone())
        .collect()
}

/// Accept, then reject, the changes `resolve_revisions` selects: the new
/// source (`None` when it selects nothing) and what was resolved.
pub(super) fn resolve_selected(
    source: &[u8],
    selection: Option<&ResolveRevisions>,
    policy: RemovedAnchorComments,
    syntax: PlanSyntax,
) -> Result<(Option<Vec<u8>>, ResolvedRevisions, AnchorChanges), EditError> {
    let mut comments = AnchorChanges::default();
    let mut resolved = ResolvedRevisions::default();
    let Some(selection) = selection else {
        return Ok((None, resolved, comments));
    };
    let change_err = |e| selection_error(e, syntax, "");
    // Each side names whole changes, checked on the source: an accept can
    // join what is left (#1261).
    for (side, verb) in [
        (selection.accept.as_ref(), "accept"),
        (selection.reject.as_ref(), "reject"),
    ] {
        if let Some(side) = side {
            crate::changes::check_whole_changes(source, side)
                .map_err(|e| selection_error(e, syntax, verb))?;
        }
    }
    let listed = crate::changes::list_changes(source).map_err(change_err)?;
    let accept = selected_ids(&listed, selection.accept.as_ref(), syntax)?;
    // A visible reject settles nothing: it is lowered to operations.
    let reject = if selection.reject_visible {
        Vec::new()
    } else {
        selected_ids(&listed, selection.reject.as_ref(), syntax)?
    };
    if let Some(both) = accept.iter().find(|id| reject.contains(id)) {
        return Err(err(
            "REVISION_CONFLICT",
            None,
            format!("resolve_revisions selects {both} to accept and to reject"),
        ));
    }
    if accept.is_empty() && reject.is_empty() {
        return Ok((None, resolved, comments));
    }
    let mut bytes = source.to_vec();
    if !accept.is_empty() {
        let (document, removed, kept) = crate::changes::resolve_for_edit(
            &bytes,
            &ChangeFilter::ids(accept.clone()),
            crate::revision_processor::Resolution::Accept,
            policy == RemovedAnchorComments::Keep,
        )
        .map_err(|e| selection_error(e, syntax, "accept"))?;
        comments.extend(
            removed,
            kept,
            crate::revision_processor::Resolution::Accept,
            Some(&named_ids(&listed, selection.accept.as_ref())),
        );
        bytes = document;
        resolved.accepted = accept;
    }
    // A change the accepted ones took along is gone, not rejected.
    let left = crate::changes::list_changes(&bytes).map_err(change_err)?;
    let reject: Vec<String> = reject
        .into_iter()
        .filter(|id| left.iter().any(|c| &c.id == id))
        .collect();
    if !reject.is_empty() {
        let (document, removed, kept) = crate::changes::resolve_for_edit(
            &bytes,
            &ChangeFilter::ids(reject.clone()),
            crate::revision_processor::Resolution::Reject,
            policy == RemovedAnchorComments::Keep,
        )
        .map_err(|e| selection_error(e, syntax, "reject"))?;
        comments.extend(
            removed,
            kept,
            crate::revision_processor::Resolution::Reject,
            Some(&named_ids(&listed, selection.reject.as_ref())),
        );
        bytes = document;
        resolved.rejected = reject;
    }
    Ok((Some(bytes), resolved, comments))
}

/// The document a plan edits once `resolve_revisions` and
/// `existing_revisions` have settled the tracked changes they select.
pub(super) struct Settled {
    /// The bytes the plan edits and the redline compares against.
    pub(super) base: Vec<u8>,
    pub(super) opened: Opened,
    pub(super) resolved_revisions: ResolvedRevisions,
    pub(super) comment_changes: AnchorChanges,
    /// Settling rewrote the document: paragraphs may have merged or gone.
    pub(super) changed: bool,
}

/// Whether `author` is the plan's author. A missing or empty author is not.
pub(super) fn owns_change(author: Option<&str>, actor: &str) -> bool {
    author.is_some_and(|name| !name.is_empty() && name == actor)
}

/// One sentence for a change the guard will not accept or reject.
pub(super) fn change_not_yours(change: &Change) -> String {
    let whose = match change.author.as_deref() {
        Some(name) if !name.is_empty() => format!("{} is by {name:?}", change.id),
        _ => format!("{} names no author", change.id),
    };
    let answer = match change.kind {
        ChangeKind::Deletion => {
            format!(
                "answer it visibly (-p {} --restore), or comment on it",
                change.id
            )
        }
        ChangeKind::Insertion => {
            format!("strike it (-p {} --strike), or comment on it", change.id)
        }
        _ => "comment on it (accepting or rejecting would erase their change)".to_string(),
    };
    format!("{whose}; {answer}")
}

/// `authors` in document order, then a nameless change, as one English list.
pub(super) fn join_authors(authors: &[String], nameless: bool) -> String {
    let mut parts: Vec<String> = authors.iter().map(|name| format!("{name:?}")).collect();
    if nameless {
        parts.push("one that names no author".to_string());
    }
    match parts.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let (last, rest) = parts.split_last().expect("at least three");
            format!("{} and {last}", rest.join(", "))
        }
    }
}

/// Whether settling `change` this way removes its content, and with it
/// every revision and comment anchor that content holds. Accepting a
/// deletion or a move-from, or rejecting an insertion or a move-to, does.
/// Accepting an insertion keeps the text, so a revision nested in it stays.
pub(super) fn removes_content(change: &Change, accept: bool) -> bool {
    matches!(
        (accept, change.kind, change.move_side),
        (true, ChangeKind::Deletion, _)
            | (false, ChangeKind::Insertion, _)
            | (true, ChangeKind::Move, Some(MoveSide::From))
            | (false, ChangeKind::Move, Some(MoveSide::To))
    )
}

/// The listed changes whose `inside` chain reaches `root`: settling `root`
/// so its content goes takes them along. A style record is not one.
pub(super) fn taken_along<'a>(listed: &'a [Change], root: &Change) -> Vec<&'a Change> {
    let by_id: HashMap<&str, &Change> = listed.iter().map(|c| (c.id.as_str(), c)).collect();
    let roots: HashSet<&str> = root.element_ids().collect();
    listed
        .iter()
        .filter(|change| {
            if roots.contains(change.id.as_str()) || change.id.starts_with(STYLES_STORY) {
                return false;
            }
            let mut cursor = change.inside.as_deref();
            let mut seen = HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    return false;
                }
                if roots.contains(id) {
                    return true;
                }
                cursor = by_id.get(id).and_then(|holder| holder.inside.as_deref());
            }
            false
        })
        .collect()
}

/// One sentence for a comment a content-removing decision would delete.
fn comment_not_yours(verb: &str, change_id: &str, comment_id: &str, author: &str) -> String {
    let whose = if author.is_empty() {
        format!("{comment_id}, which names no author")
    } else {
        format!("{comment_id}, which is by {author:?}")
    };
    format!("{verb} {change_id} would delete {whose}")
}

/// Whether `author` wrote the comment. An empty author is not the plan's.
fn owns_comment(author: &str, actor: &str) -> bool {
    !author.is_empty() && author == actor
}

/// `record` replies to `root`, walking `parent` and stopping on a cycle.
fn replies_to(
    record: &crate::comments::CommentRecord,
    root: u32,
    by_id: &HashMap<u32, &crate::comments::CommentRecord>,
) -> bool {
    let mut cursor = record.parent;
    let mut seen = HashSet::new();
    while let Some(id) = cursor {
        if !seen.insert(id) {
            return false;
        }
        if id == root {
            return true;
        }
        cursor = by_id.get(&id).and_then(|comment| comment.parent);
    }
    false
}

/// Refuse when a content-removing decision would delete a comment the plan's
/// author does not own, the comment itself or a reply in its thread. The
/// anchor's removal deletes the thread.
pub(super) fn guard_removed_comments(
    source: &[u8],
    actor: &str,
    removals: &[(&Change, bool, Vec<&Change>)],
) -> Result<(), EditError> {
    let mut held: Vec<(&str, &str, bool)> = Vec::new();
    for (change, accepting, nested) in removals {
        for id in change
            .holds_comments
            .iter()
            .chain(nested.iter().flat_map(|inner| inner.holds_comments.iter()))
        {
            if held.iter().any(|(_, have, _)| *have == id) {
                continue;
            }
            held.push((change.id.as_str(), id.as_str(), *accepting));
        }
    }
    if held.is_empty() {
        return Ok(());
    }
    let records = crate::comments::list_comments(source)
        .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
    let by_id: HashMap<u32, &crate::comments::CommentRecord> =
        records.iter().map(|record| (record.id, record)).collect();
    for (change_id, comment_id, accepting) in held {
        let Some(root) = records.iter().find(|record| record.r#ref == comment_id) else {
            continue;
        };
        let verb = if accepting { "accepting" } else { "rejecting" };
        if !owns_comment(&root.author, actor) {
            return Err(err(
                "COMMENT_NOT_YOURS",
                None,
                comment_not_yours(verb, change_id, comment_id, &root.author),
            ));
        }
        for reply in &records {
            if reply.id == root.id || !replies_to(reply, root.id, &by_id) {
                continue;
            }
            if owns_comment(&reply.author, actor) {
                continue;
            }
            return Err(err(
                "COMMENT_NOT_YOURS",
                None,
                comment_not_yours(verb, change_id, &reply.r#ref, &reply.author),
            ));
        }
    }
    Ok(())
}

/// An author the wholesale settlement would erase, in first-seen order.
pub(super) fn note_foreign(
    change: &Change,
    actor: &str,
    authors: &mut Vec<String>,
    nameless: &mut bool,
) {
    if change.id.starts_with(STYLES_STORY) || owns_change(change.author.as_deref(), actor) {
        return;
    }
    match change.author.as_deref() {
        Some(name) if !name.is_empty() => {
            if !authors.iter().any(|have: &String| have == name) {
                authors.push(name.to_string());
            }
        }
        _ => *nameless = true,
    }
}

/// The blockers a refusal names: the first [`BLOCKERS_SHOWN`] revision
/// ids, then a count of the rest (`jubarte changes` lists them all).
fn blocker_list(revisions: &[String]) -> String {
    let shown = revisions
        .iter()
        .take(BLOCKERS_SHOWN)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    match revisions.len().saturating_sub(BLOCKERS_SHOWN) {
        0 => shown,
        rest => format!("{shown} and {rest} more"),
    }
}

const BLOCKERS_SHOWN: usize = 20;

/// Settle `source` (opened as `probe`) as `plan` asks: accept and reject
/// what `resolve_revisions` selects, then apply `existing_revisions` to the
/// changes left.
pub(super) fn settle(
    source: &[u8],
    plan: &EditPlan,
    probe: Opened,
    syntax: PlanSyntax,
) -> Result<Settled, EditError> {
    let original = source;
    if plan.others.keeps() {
        let wholesale = match plan.existing_revisions {
            ExistingRevisions::Accept => Some("accept"),
            ExistingRevisions::Reject => Some("reject"),
            ExistingRevisions::Keep | ExistingRevisions::Refuse => None,
        };
        guard_keep_others(
            source,
            &plan.author,
            plan.resolve_revisions.as_ref(),
            wholesale,
            false,
            syntax,
        )?;
    }
    let (resolved_source, resolved_revisions, mut comment_changes) = resolve_selected(
        source,
        plan.resolve_revisions.as_ref(),
        plan.removed_anchor_comments,
        syntax,
    )?;
    let (source, probe) = match &resolved_source {
        Some(bytes) => (bytes.as_slice(), Opened::open(bytes).map_err(open_error)?),
        None => (source, probe),
    };
    // The listed changes of the body, headers, footers and notes; a style's
    // recorded formatting does not make the text tracked. A refusal names
    // those revisions: they are the blockers, and no rewrite hash is returned.
    let revisions: Vec<String> = crate::changes::changes_in(&probe.pkg)
        .into_iter()
        .filter(|c| !c.id.starts_with(STYLES_STORY))
        .map(|c| c.id)
        .collect();
    let has_revisions = !revisions.is_empty();
    let (base, opened, flattened) = match (has_revisions, plan.existing_revisions) {
        (false, _) | (true, ExistingRevisions::Keep) => (source.to_vec(), probe, false),
        (true, ExistingRevisions::Refuse) => {
            return Err(err(
                "EXISTING_REVISIONS",
                None,
                format!(
                    "the document already holds tracked changes; blockers: {}; {KEEP_REVISIONS}",
                    blocker_list(&revisions)
                ),
            ));
        }
        (true, policy) => {
            let resolution = if policy == ExistingRevisions::Accept {
                crate::revision_processor::Resolution::Accept
            } else {
                crate::revision_processor::Resolution::Reject
            };
            let (flattened, removed, kept) = crate::changes::resolve_for_edit(
                source,
                &ChangeFilter::default(),
                resolution,
                plan.removed_anchor_comments == RemovedAnchorComments::Keep,
            )
            .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
            comment_changes.extend(removed, kept, resolution, None);
            let reopened = Opened::open(&flattened).map_err(open_error)?;
            (flattened, reopened, true)
        }
    };
    let changed = resolved_source.is_some() || flattened;
    // Accepts, rejects and the global policy settle in separate passes;
    // only the settled document may not expose a final mark (#2096).
    if changed {
        crate::changes::check_exposed_final_marks(original, &base)
            .map_err(|e| selection_error(e, syntax, ""))?;
    }
    Ok(Settled {
        base,
        opened,
        resolved_revisions,
        comment_changes,
        changed,
    })
}

/// A document's stories and its addressable paragraphs.
pub(super) struct Loaded {
    /// The body first, then every header, footer and notes part.
    pub(super) stories: Vec<StoryPart>,
    /// The body's paragraphs, then each story's.
    pub(super) paragraph_nodes: Vec<NodeId>,
    /// `(story, index in that story)` per entry of `paragraph_nodes`.
    pub(super) paragraph_story: Vec<(usize, usize)>,
}

/// The body and every header, footer and notes part of `opened`, parsed
/// into its DOM, with every addressable paragraph.
pub(super) fn load_stories(opened: &mut Opened) -> Result<Loaded, EditError> {
    let mut stories = vec![StoryPart {
        id: BODY_STORY.to_string(),
        part: opened.main.clone(),
        document: opened.document,
        root: opened.body,
    }];
    for (id, _, part) in opened.story_parts() {
        let invalid = |m: String| err("INVALID_DOCUMENT", None, format!("{part}: {m}"));
        let xml = opened
            .pkg
            .part_string(&part)
            .ok_or_else(|| invalid("missing part".into()))?;
        crate::xmllinq::parse::validate_xml(&xml).map_err(|e| invalid(e.to_string()))?;
        let document = opened.dom.parse_xdocument(&xml);
        let root = opened
            .dom
            .root(document)
            .ok_or_else(|| invalid("missing XML root".into()))?;
        stories.push(StoryPart {
            id,
            part,
            document,
            root,
        });
    }
    let mut paragraph_nodes = Vec::new();
    let mut paragraph_story = Vec::new();
    for (index, story) in stories.iter().enumerate() {
        let nodes = if index == 0 {
            crate::inspect::body_paragraph_nodes(&opened.dom, story.root)
        } else {
            crate::inspect::story_paragraph_nodes(&opened.dom, story.root)
        };
        for (local, node) in nodes.into_iter().enumerate() {
            paragraph_nodes.push(node);
            paragraph_story.push((index, local));
        }
    }
    Ok(Loaded {
        stories,
        paragraph_nodes,
        paragraph_story,
    })
}
