// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Annotation ids after Accept All / Reject All, as Word saves them.
//!
//! Word numbers bookmarks and comments from one counter, 0, 1, …, in document
//! order of their starts: a `bookmarkStart`, a `commentRangeStart`, or the
//! `commentReference` of a comment without a range (`_to_improve_accepted_
//! changes`: 2288f27be1 has comments 0 1, bookmarks 2–6, comments 7 8).
//! The counter runs on from the main document through the other stories;
//! `comments.xml` takes the new comment ids.

use std::collections::HashMap;

use super::comments::Parsed;
use crate::comparer::comments::FAMILY;
use crate::namespaces::W;
use crate::opc::PartFs;

/// Renumber the bookmarks and comments of `story_parts` (main first).
pub(super) fn renumber(pkg: &mut PartFs, story_parts: &[String]) {
    let id = W::id();
    let names = [
        "bookmarkStart",
        "bookmarkEnd",
        "commentRangeStart",
        "commentRangeEnd",
        "commentReference",
    ]
    .map(W::name);
    let [b_start, b_end, c_start, c_end, c_ref] = &names;
    let mut next = 0usize;
    let mut comment_ids: HashMap<String, String> = HashMap::new();
    for part in story_parts {
        let Some(mut s) = Parsed::load(pkg, part) else {
            continue;
        };
        let markers: Vec<_> = s
            .dom
            .descendants(s.root, None)
            .into_iter()
            .filter(|&e| s.dom.name(e).is_some_and(|n| names.contains(&n)))
            .collect();
        if markers.is_empty() {
            continue;
        }
        let mut bookmark_ids: HashMap<String, String> = HashMap::new();
        for &m in &markers {
            let name = s.dom.name(m).unwrap();
            let Some(old) = s.dom.attribute(m, &id).map(str::to_string) else {
                continue;
            };
            let map = if name == *b_start {
                &mut bookmark_ids
            } else if name == *c_start || name == *c_ref {
                &mut comment_ids
            } else {
                continue;
            };
            map.entry(old).or_insert_with(|| {
                next += 1;
                (next - 1).to_string()
            });
        }
        let mut changed = false;
        for m in markers {
            let name = s.dom.name(m).unwrap();
            let map = if name == *b_start || name == *b_end {
                &bookmark_ids
            } else if name == *c_start || name == *c_end || name == *c_ref {
                &comment_ids
            } else {
                continue;
            };
            let old = s.dom.attribute(m, &id).unwrap_or("");
            if let Some(new) = map.get(old)
                && new != old
            {
                let new = new.clone();
                s.dom.set_attribute_value(m, &id, Some(&new));
                changed = true;
            }
        }
        if changed {
            s.store(pkg);
        }
    }
    if comment_ids.iter().all(|(old, new)| old == new) {
        return;
    }
    let Some(mut comments) = Parsed::load(pkg, FAMILY[0].0) else {
        return;
    };
    for c in comments.w("comment") {
        let old = comments.dom.attribute(c, &id).unwrap_or("");
        if let Some(new) = comment_ids.get(old) {
            let new = new.clone();
            comments.dom.set_attribute_value(c, &id, Some(&new));
        }
    }
    comments.store(pkg);
}
