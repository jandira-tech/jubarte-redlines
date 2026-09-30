// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bookmarks through Accept All (and Reject All, which accepts the reversed
//! revisions), as Word keeps them.
//!
//! Word holds a bookmark as a range of characters, paragraph marks included.
//! Accepting a deletion removes characters, so a bookmark whose span held
//! characters that were all deleted is gone, both markers, while one that
//! keeps a character keeps both. An empty bookmark has no characters to lose:
//! it survives even inside deleted text, where the deletion was
//! (`_to_improve_accepted_changes`: ece42865e7, 221577c35b, 6fb9bbdb49).

use std::collections::{HashMap, HashSet};

use super::{TagType, descendant_and_self_tags, paragraph_mark_is_deleted_or_moved_from};
use crate::namespaces::{M, W};
use crate::xmllinq::{Dom, NodeId, XName};

/// Remove, before the accept transforms run, every bookmark whose span held
/// only deleted characters.
pub(super) fn drop_wholly_deleted_bookmarks(dom: &mut Dom, root: NodeId) {
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    if dom
        .find_descendant_element(root, Some(&start), |_| true)
        .is_none()
    {
        return;
    }
    let id = W::id();
    // Open bookmark id → (characters, deleted characters).
    let mut open: HashMap<String, (usize, usize)> = HashMap::new();
    let mut dropped: HashSet<String> = HashSet::new();
    for tag in descendant_and_self_tags(dom, root) {
        let e = tag.element;
        let Some(name) = dom.name(e) else {
            continue;
        };
        let deleted = if tag.tag_type == TagType::EndElement {
            if name != W::p() {
                continue;
            }
            paragraph_mark_is_deleted_or_moved_from(dom, e) || in_deleted_row(dom, e)
        } else if name == start {
            if let Some(b) = dom.attribute(e, &id) {
                open.insert(b.to_string(), (0, 0));
            }
            continue;
        } else if name == end {
            if let Some(b) = dom.attribute(e, &id)
                && let Some((chars, gone)) = open.remove(b)
                && chars > 0
                && gone == chars
            {
                dropped.insert(b.to_string());
            }
            continue;
        } else if is_character(dom, e, &name) {
            in_deleted_content(dom, e)
        } else {
            continue;
        };
        for (chars, gone) in open.values_mut() {
            *chars += 1;
            *gone += usize::from(deleted);
        }
    }
    if dropped.is_empty() {
        return;
    }
    for marker in [start, end] {
        for b in dom.descendants(root, Some(&marker)) {
            if dom.attribute(b, &id).is_some_and(|v| dropped.contains(v)) {
                dom.remove(b);
            }
        }
    }
}

/// Does `e` stand for at least one character of the story?
fn is_character(dom: &Dom, e: NodeId, name: &XName) -> bool {
    if name.namespace_name() == M::URI {
        return name.local_name() == "t" && !dom.value_str(e).is_empty();
    }
    if name.namespace_name() != W::URI {
        return false;
    }
    match name.local_name() {
        "t" | "delText" | "instrText" | "delInstrText" => !dom.value_str(e).is_empty(),
        // w:tab is also a tab stop in pPr/tabs; only the run's is a character.
        "tab" => dom.parent(e).and_then(|p| dom.name(p)) == Some(W::r()),
        "br" | "cr" | "sym" | "drawing" | "pict" | "object" | "noBreakHyphen" | "softHyphen"
        | "footnoteReference" | "endnoteReference" | "fldChar" | "ptab" => true,
        _ => false,
    }
}

/// Is `e` inside deleted or moved-from content, or a deleted row?
fn in_deleted_content(dom: &Dom, e: NodeId) -> bool {
    let (del, move_from) = (W::del(), W::move_from());
    dom.ancestors(e, None).into_iter().any(|a| {
        dom.name(a)
            .is_some_and(|n| n == del || n == move_from || (n == W::tr() && row_is_deleted(dom, a)))
    })
}

fn in_deleted_row(dom: &Dom, p: NodeId) -> bool {
    dom.ancestors(p, Some(&W::tr()))
        .into_iter()
        .any(|tr| row_is_deleted(dom, tr))
}

fn row_is_deleted(dom: &Dom, tr: NodeId) -> bool {
    dom.element(tr, &W::tr_pr())
        .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
}

/// Remove the bookmark markers left without their partner: Word's range model
/// has no half bookmark, and the accept transforms can drop one side with
/// the content around it (a deleted row, a moved-from run).
pub(super) fn drop_unpaired_bookmarks(dom: &mut Dom, root: NodeId) {
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    let id = W::id();
    let ids = |dom: &Dom, name: &XName| -> HashSet<String> {
        dom.descendants(root, Some(name))
            .into_iter()
            .filter_map(|b| dom.attribute(b, &id).map(str::to_string))
            .collect()
    };
    let (starts, ends) = (ids(dom, &start), ids(dom, &end));
    if starts == ends {
        return;
    }
    for (marker, partners) in [(start, &ends), (end, &starts)] {
        for b in dom.descendants(root, Some(&marker)) {
            if dom.attribute(b, &id).is_some_and(|v| !partners.contains(v)) {
                dom.remove(b);
            }
        }
    }
}
