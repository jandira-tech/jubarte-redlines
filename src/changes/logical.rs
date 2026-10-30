// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Logical changes (plan 11, #1261): the marks Word writes for one edit,
//! listed as one change.
//!
//! Word splits one typed insertion into several `w:ins` (each typing
//! session gets its own, with its own date), and records a replacement as
//! a deletion followed by an insertion. One author's touching marks of one
//! kind are one change, with the composite id the view prints
//! (`r7+8+9+10`); a deletion run and the same author's insertion run
//! beside it, in either order, are one replacement (`r3+4`, `r1+2`, its
//! id in reading order: Word usually writes the insertion first). An
//! insertion run and a deletion after it touch: nothing between them,
//! not even text another author inserted. Dates do not split a change.
//!
//! What separates one author's marks is text that was there before: text
//! no change touches, text another author deleted, and the text of an
//! insertion the marks were made inside (two deletions in someone's
//! insertion, with its text between them, are two changes). Text another
//! author inserted among the marks did not exist when they were made, so
//! it separates nothing: when an edit inside someone's insertion splits it
//! (plan 1, #1136), the pieces stay one change. A paragraph mark is content after the paragraph's text: a
//! change takes its own paragraph's inserted or deleted mark (`r5+6`) and
//! ends there. Word writes a mark of its own for every paragraph, so a
//! change that ran on into the next paragraph would make one change, with
//! a composite id hundreds of marks long, of a whole deleted clause list.
//! No change runs across a table, row, cell, content control or note
//! boundary, and moves, formatting changes and the marks in drawings stay
//! changes of their own. A hyperlink, smart tag, custom XML element or
//! simple field is walked through: its runs touch the runs beside it, as
//! Word lays them out.

use std::collections::{HashMap, HashSet};

use super::{Carrier, Change, ChangeError, ChangeFilter, ChangeKind, ChangeMember};
use crate::ids::{ShortId, Story};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// What the walk meets, in document order.
enum Event {
    /// Shown content, under the listed insertions, deletions and moves
    /// around it: indices into the story's carriers, outermost first.
    Atom(Vec<usize>),
    /// A boundary no change runs across.
    Break,
}

/// Which marks a change holds so far.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Insert,
    Delete,
    /// Deletions, then insertions.
    Replace,
    /// Insertions, then deletions: the order Word and the comparer
    /// usually write a replacement in.
    InsertDelete,
}

impl Phase {
    fn of(kind: ChangeKind) -> Self {
        if kind == ChangeKind::Deletion {
            Self::Delete
        } else {
            Self::Insert
        }
    }

    /// The phase after a mark of `kind`, or `None` when the mark starts a
    /// new change: a replacement holds one run of each kind, in either
    /// order.
    fn then(self, kind: ChangeKind) -> Option<Self> {
        let deletion = kind == ChangeKind::Deletion;
        match self {
            Self::Delete if deletion => Some(Self::Delete),
            Self::Delete => Some(Self::Replace),
            Self::Insert if deletion => Some(Self::InsertDelete),
            Self::Insert => Some(Self::Insert),
            Self::Replace => (!deletion).then_some(Self::Replace),
            Self::InsertDelete => deletion.then_some(Self::InsertDelete),
        }
    }
}

/// The logical changes of one story, in document order: each group of
/// `carriers` folded into one change, every other listed carrier as it is.
pub(super) fn logical(dom: &Dom, root: NodeId, carriers: &[Carrier]) -> Vec<Change> {
    let groups = groups(dom, root, carriers);
    let mut group_of: HashMap<usize, usize> = HashMap::new();
    for (g, members) in groups.iter().enumerate() {
        for &m in members {
            group_of.insert(m, g);
        }
    }
    let ids: Vec<String> = groups
        .iter()
        .map(|members| composite(members.iter().map(|&m| listed(carriers, m))))
        .collect();
    // A member's id names its change wherever another change sits inside it.
    let renamed: HashMap<&str, &str> = groups
        .iter()
        .zip(&ids)
        .flat_map(|(members, id)| {
            members
                .iter()
                .map(move |&m| (listed(carriers, m).id.as_str(), id.as_str()))
        })
        .collect();
    let rename = |inside: &Option<String>| {
        inside
            .as_deref()
            .map(|id| renamed.get(id).copied().unwrap_or(id).to_string())
    };
    let mut out = Vec::new();
    for (i, carrier) in carriers.iter().enumerate() {
        let Some(change) = &carrier.change else {
            continue;
        };
        let Some(&g) = group_of.get(&i) else {
            let mut change = change.clone();
            change.inside = rename(&change.inside);
            out.push(change);
            continue;
        };
        // A group takes the place of its first member in reading order,
        // not of its paragraph mark, which the XML writes first.
        if groups[g][0] != i {
            continue;
        }
        let members: Vec<&Change> = groups[g].iter().map(|&m| listed(carriers, m)).collect();
        let mut change = folded(&ids[g], &members);
        // Inside a change only when every member is (two deletions, one in
        // someone's insertion and one after it, are not); a replacement is
        // inside what its old side is, as the view's `re` names it.
        let old_side = change.kind == ChangeKind::Replacement;
        let insides: Vec<Option<String>> = members
            .iter()
            .filter(|m| !old_side || m.kind == ChangeKind::Deletion)
            .map(|m| rename(&m.inside).filter(|inside| inside != &ids[g]))
            .collect();
        change.inside = insides
            .iter()
            .all(|inside| *inside == insides[0])
            .then(|| insides[0].clone())
            .flatten();
        out.push(change);
    }
    out
}

fn listed(carriers: &[Carrier], i: usize) -> &Change {
    carriers[i]
        .change
        .as_ref()
        .expect("a group holds listed carriers")
}

/// The groups of two or more carriers, each in reading order (a
/// paragraph's mark after its text).
fn groups(dom: &Dom, root: NodeId, carriers: &[Carrier]) -> Vec<Vec<usize>> {
    let wrappers: HashMap<NodeId, usize> = carriers
        .iter()
        .enumerate()
        .filter(|(_, c)| c.change.as_ref().is_some_and(wraps))
        .map(|(i, c)| (c.node, i))
        .collect();
    // A `w:id` the story repeats names more than one element: a composite
    // of it could not say which, so those stay apart.
    let mut seen = HashSet::new();
    let repeated: HashSet<&str> = carriers
        .iter()
        .filter_map(|c| c.change.as_ref())
        .map(|c| c.id.as_str())
        .filter(|id| !seen.insert(*id))
        .collect();
    let mut events = Vec::new();
    walk(dom, root, &wrappers, &mut Vec::new(), &mut events);
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut phases: Vec<Phase> = Vec::new();
    // The carriers each group's marks sit inside: content they hold was
    // there when the marks were made.
    let mut within: Vec<HashSet<usize>> = Vec::new();
    let mut group_of: HashMap<usize, usize> = HashMap::new();
    // Whether content the group's author did not mark came after its last
    // mark: an insertion and a deletion after it are one replacement only
    // when they touch.
    let mut apart: Vec<bool> = Vec::new();
    // Each author's change the walk is still adding to.
    let mut open: HashMap<Option<&str>, usize> = HashMap::new();
    for event in &events {
        let stack = match event {
            Event::Break => {
                open.clear();
                continue;
            }
            Event::Atom(stack) => stack,
        };
        // Each author's outermost carrier around this content.
        let mut here: Vec<(Option<&str>, usize)> = Vec::new();
        for &c in stack {
            let author = listed(carriers, c).author.as_deref();
            if !here.iter().any(|(a, _)| *a == author) {
                here.push((author, c));
            }
        }
        // Content another author inserted after a change's marks were made
        // separates nothing, even inside the insertion the marks sit in,
        // and nor does a later edit of it; anything that was there before
        // does: original text, and the text of an insertion the marks were
        // made inside.
        open.retain(|author, g| {
            here.iter().any(|(a, _)| a == author)
                || stack.iter().any(|c| {
                    listed(carriers, *c).kind == ChangeKind::Insertion && !within[*g].contains(c)
                })
        });
        for (author, &g) in &open {
            if !here.iter().any(|(a, _)| a == author) {
                apart[g] = true;
            }
        }
        for (author, c) in here {
            let change = listed(carriers, c);
            if !groupable(change) || repeated.contains(change.id.as_str()) {
                open.remove(&author);
                continue;
            }
            if let Some(&g) = group_of.get(&c) {
                // More content of a carrier already placed.
                apart[g] = false;
                open.insert(author, g);
                continue;
            }
            let joined = open.get(&author).and_then(|&g| {
                let phase = phases[g].then(change.kind)?;
                let touches =
                    !(apart[g] && phase == Phase::InsertDelete && phases[g] == Phase::Insert);
                touches.then_some((g, phase))
            });
            let g = match joined {
                Some((g, phase)) => {
                    phases[g] = phase;
                    groups[g].push(c);
                    g
                }
                None => {
                    groups.push(vec![c]);
                    phases.push(Phase::of(change.kind));
                    within.push(HashSet::new());
                    apart.push(false);
                    groups.len() - 1
                }
            };
            apart[g] = false;
            let outside = stack.iter().take_while(|&&s| s != c);
            within[g].extend(outside);
            group_of.insert(c, g);
            open.insert(author, g);
        }
    }
    groups.retain(|members| members.len() > 1);
    groups
}

/// The listed carriers whose content the walk descends: run-level
/// insertions, deletions and moves, and paragraph marks.
fn wraps(change: &Change) -> bool {
    matches!(
        change.kind,
        ChangeKind::Insertion | ChangeKind::Deletion | ChangeKind::Move
    ) && matches!(change.target, "text" | "paragraph_mark")
}

/// Whether a carrier can join others: an insertion or deletion of text or
/// of a paragraph mark whose id has a short spelling (a composite holds
/// short ids only).
fn groupable(change: &Change) -> bool {
    matches!(change.kind, ChangeKind::Insertion | ChangeKind::Deletion)
        && matches!(change.target, "text" | "paragraph_mark")
        && single(&change.id).is_some()
}

/// The story and `w:id` of a one-change short id.
fn single(id: &str) -> Option<(Story, u32)> {
    match id.parse::<ShortId>() {
        Ok(ShortId::Change { story, wids }) if wids.len() == 1 => Some((story, wids[0])),
        _ => None,
    }
}

/// The id of changes `members` together: `r7+8+9+10`, `header1.r1+2`.
fn composite<'a>(members: impl IntoIterator<Item = &'a Change>) -> String {
    joined(members.into_iter().map(|member| member.id.as_str()))
}

/// The composite id of the marks `ids` names, in their order: every mark
/// of a change is in one story, the first's.
pub(super) fn joined<'a>(ids: impl IntoIterator<Item = &'a str>) -> String {
    let mut story = None;
    let mut wids = Vec::new();
    for (s, wid) in ids.into_iter().filter_map(single) {
        story.get_or_insert(s);
        wids.push(wid);
    }
    ShortId::Change {
        story: story.unwrap_or(Story::Body),
        wids,
    }
    .to_string()
}

/// One change standing for `members`, in reading order.
fn folded(id: &str, members: &[&Change]) -> Change {
    let first = members[0];
    let kind = if members.iter().all(|m| m.kind == first.kind) {
        first.kind
    } else {
        ChangeKind::Replacement
    };
    let target = if members.iter().all(|m| m.target == "paragraph_mark") {
        "paragraph_mark"
    } else {
        "text"
    };
    let text_of = |kind: ChangeKind| -> String {
        members
            .iter()
            .filter(|m| m.kind == kind)
            .map(|m| {
                if m.target == "paragraph_mark" {
                    "\n"
                } else {
                    m.text.as_str()
                }
            })
            .collect()
    };
    let text = if kind == ChangeKind::Replacement {
        format!(
            "{} → {}",
            text_of(ChangeKind::Deletion),
            text_of(ChangeKind::Insertion)
        )
    } else {
        text_of(kind)
    };
    let mut holds_comments: Vec<String> = Vec::new();
    for c in members.iter().flat_map(|m| &m.holds_comments) {
        if !holds_comments.contains(c) {
            holds_comments.push(c.clone());
        }
    }
    Change {
        id: id.to_string(),
        r#ref: id.to_string(),
        kind,
        target,
        author: first.author.clone(),
        date: earliest(members),
        text,
        paragraph: first.paragraph.clone(),
        move_name: None,
        move_side: None,
        inside: None,
        holds_comments,
        members: members
            .iter()
            .map(|m| ChangeMember {
                id: m.id.clone(),
                kind: m.kind,
                target: m.target,
                date: m.date.clone(),
                text: m.text.clone(),
            })
            .collect(),
    }
}

/// The earliest member date that reads as an instant; else the first one.
fn earliest(members: &[&Change]) -> Option<String> {
    let key = |date: &str| {
        crate::convert::timestamp_order_key(date)
            .map(|(instant, fraction)| (instant, format!("{fraction:0<9}")))
    };
    members
        .iter()
        .filter_map(|m| m.date.as_deref())
        .filter_map(|date| Some((key(date)?, date)))
        .min_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, date)| date.to_string())
        .or_else(|| members.iter().find_map(|m| m.date.clone()))
}

/// Walk `node`'s children in document order, with `stack` the listed
/// carriers around them.
fn walk(
    dom: &Dom,
    node: NodeId,
    wrappers: &HashMap<NodeId, usize>,
    stack: &mut Vec<usize>,
    out: &mut Vec<Event>,
) {
    for child in dom.elements(node, None) {
        let Some(name) = dom.name(child) else {
            continue;
        };
        if name.namespace_name() != W::URI {
            // mc:AlternateContent, math, and the like: shown, not walked.
            out.push(Event::Atom(stack.clone()));
            continue;
        }
        if let Some(&c) = wrappers.get(&child) {
            stack.push(c);
            walk(dom, child, wrappers, stack, out);
            stack.pop();
            continue;
        }
        match name.local_name() {
            "p" => {
                walk(dom, child, wrappers, stack, out);
                // The paragraph's mark comes after its text, and no change
                // runs on past it into the next paragraph.
                let depth = stack.len();
                stack.extend(mark_carriers(dom, child, wrappers));
                out.push(Event::Atom(stack.clone()));
                stack.truncate(depth);
                out.push(Event::Break);
            }
            // What content runs through.
            "body" | "r" | "hyperlink" | "smartTag" | "customXml" | "fldSimple" | "sdtContent"
            | "dir" | "bdo" | "ins" | "del" | "moveFrom" | "moveTo" => {
                walk(dom, child, wrappers, stack, out);
            }
            // Boundaries no change runs across.
            "tbl" | "tr" | "tc" | "sdt" | "footnote" | "endnote" | "comment" | "txbxContent" => {
                out.push(Event::Break);
                walk(dom, child, wrappers, stack, out);
                out.push(Event::Break);
            }
            // Properties and markers: nothing shown.
            "pPr"
            | "rPr"
            | "sectPr"
            | "tblPr"
            | "tblPrEx"
            | "tblGrid"
            | "trPr"
            | "tcPr"
            | "sdtPr"
            | "sdtEndPr"
            | "customXmlPr"
            | "smartTagPr"
            | "bookmarkStart"
            | "bookmarkEnd"
            | "commentRangeStart"
            | "commentRangeEnd"
            | "commentReference"
            | "proofErr"
            | "permStart"
            | "permEnd"
            | "moveFromRangeStart"
            | "moveFromRangeEnd"
            | "moveToRangeStart"
            | "moveToRangeEnd"
            | "lastRenderedPageBreak"
            | "customXmlInsRangeStart"
            | "customXmlInsRangeEnd"
            | "customXmlDelRangeStart"
            | "customXmlDelRangeEnd"
            | "customXmlMoveFromRangeStart"
            | "customXmlMoveFromRangeEnd"
            | "customXmlMoveToRangeStart"
            | "customXmlMoveToRangeEnd" => {}
            // Text that holds none, an empty run's: nothing shown, as the
            // view prints nothing for it.
            "t" | "delText" if empty_text(dom, child) => {}
            // Text, tabs, breaks, symbols, references, fields, drawings.
            _ => out.push(Event::Atom(stack.clone())),
        }
    }
}

/// Whether text element `node` holds no character.
fn empty_text(dom: &Dom, node: NodeId) -> bool {
    dom.nodes(node)
        .into_iter()
        .all(|n| dom.text_value(n).is_none_or(str::is_empty))
}

/// The listed carriers on paragraph `p`'s mark, in order.
fn mark_carriers(dom: &Dom, p: NodeId, wrappers: &HashMap<NodeId, usize>) -> Vec<usize> {
    dom.element(p, &W::p_pr())
        .and_then(|p_pr| dom.element(p_pr, &W::r_pr()))
        .map(|r_pr| {
            dom.elements(r_pr, None)
                .into_iter()
                .filter_map(|m| wrappers.get(&m).copied())
                .collect()
        })
        .unwrap_or_default()
}

/// Refuse a selection that names only some of a logical change's marks:
/// the marks `filter` selects (its ids, [`ChangeFilter::normalized`], with
/// its authors and kinds), taken together, must be every mark of each
/// change they touch, or one whole side of a replacement (rejecting only
/// its deletion keeps both texts, as Word allows). A filter that selects
/// none of a change's marks leaves it alone.
///
/// # Errors
///
/// [`ChangeError::PartOfChange`] for the first change, in document order,
/// that the ids name only part of.
pub(super) fn check_whole(filter: &ChangeFilter, logical: &[Change]) -> Result<(), ChangeError> {
    let Some(named) = &filter.ids else {
        return Ok(());
    };
    for change in logical.iter().filter(|c| !c.members.is_empty()) {
        // The marks the whole filter selects: its authors and kinds too.
        let by = filter
            .authors
            .as_ref()
            .is_none_or(|authors| change.author.as_ref().is_some_and(|a| authors.contains(a)));
        let hit: Vec<&ChangeMember> = change
            .members
            .iter()
            .filter(|m| {
                by && named.contains(&m.id)
                    && filter
                        .kinds
                        .as_ref()
                        .is_none_or(|kinds| kinds.contains(&m.kind))
            })
            .collect();
        if hit.is_empty() || hit.len() == change.members.len() {
            continue;
        }
        let side = |kind: ChangeKind| -> Vec<&ChangeMember> {
            change.members.iter().filter(|m| m.kind == kind).collect()
        };
        let mut part_of = None;
        if change.kind == ChangeKind::Replacement {
            for kind in [ChangeKind::Deletion, ChangeKind::Insertion] {
                let whole_side = side(kind);
                if hit.iter().all(|m| m.kind == kind) {
                    if hit.len() == whole_side.len() {
                        part_of = Some(None);
                    } else {
                        part_of = Some(Some((composite_of(&whole_side), kind)));
                    }
                }
            }
        }
        let side = match part_of {
            // One whole side of a replacement.
            Some(None) => continue,
            Some(side) => side,
            None => None,
        };
        return Err(ChangeError::PartOfChange {
            named: composite_of(&hit),
            whole: change.id.clone(),
            side,
        });
    }
    Ok(())
}

fn composite_of(members: &[&ChangeMember]) -> String {
    joined(members.iter().map(|member| member.id.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replacement_holds_one_run_of_each_kind() {
        use ChangeKind::{Deletion, Insertion};
        assert!(Phase::Insert.then(Deletion) == Some(Phase::InsertDelete));
        assert!(Phase::InsertDelete.then(Deletion) == Some(Phase::InsertDelete));
        assert!(Phase::InsertDelete.then(Insertion).is_none());
        assert!(Phase::Replace.then(Deletion).is_none());
        assert!(Phase::Delete.then(Insertion) == Some(Phase::Replace));
        assert!(Phase::Replace.then(Insertion) == Some(Phase::Replace));
        assert!(Phase::Delete.then(Deletion) == Some(Phase::Delete));
        assert!(Phase::Insert.then(Insertion) == Some(Phase::Insert));
    }

    fn change(id: &str) -> Change {
        Change {
            id: id.to_string(),
            r#ref: id.to_string(),
            kind: ChangeKind::Insertion,
            target: "text",
            author: None,
            date: None,
            text: String::new(),
            paragraph: None,
            move_name: None,
            move_side: None,
            inside: None,
            holds_comments: Vec::new(),
            members: Vec::new(),
        }
    }

    #[test]
    fn a_composite_keeps_its_story_and_reading_order() {
        let (a, b, c) = (
            change("header1.r6"),
            change("header1.r5"),
            change("header1.r11"),
        );
        assert_eq!(composite([&a, &b, &c]), "header1.r6+5+11");
        assert_eq!(composite([&change("r3"), &change("r4")]), "r3+4");
    }
}
