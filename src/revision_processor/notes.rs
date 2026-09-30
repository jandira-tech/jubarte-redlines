// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Footnotes and endnotes through Accept All / Reject All, as Word keeps
//! them: a note lives by its reference mark. Once the resolved revisions
//! took the reference (an inserted reference rejected, a deleted one
//! accepted), the note goes; Word's separator notes (`w:type`) stay
//! (bench `rejected_tracking`: 546a6e0c15, 5620c6bac3).

use std::collections::HashSet;

use super::comments::Parsed;
use crate::namespaces::W;
use crate::opc::PartFs;

/// (relationship type suffix, note element, reference element).
const KINDS: [(&str, &str, &str); 2] = [
    ("/footnotes", "footnote", "footnoteReference"),
    ("/endnotes", "endnote", "endnoteReference"),
];

/// Remove the notes no story references any more.
pub(super) fn prune_orphan_notes(pkg: &mut PartFs, story_parts: &[String]) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let parts: Vec<(String, &str, &str)> = pkg
        .read_rels_for(&main)
        .map(|rels| {
            KINDS
                .iter()
                .filter_map(|&(rel_type, note, reference)| {
                    rels.items
                        .iter()
                        .find(|r| r.rel_type.ends_with(rel_type))
                        .map(|r| (pkg.resolve_rel_target(&main, &r.target), note, reference))
                })
                .collect()
        })
        .unwrap_or_default();
    let id = W::id();
    for (part, note, reference) in parts {
        let Some(mut notes) = Parsed::load(pkg, &part) else {
            continue;
        };
        let referenced: HashSet<String> = story_parts
            .iter()
            .filter_map(|p| Parsed::load(pkg, p))
            .flat_map(|s| {
                s.w(reference)
                    .into_iter()
                    .filter_map(|r| s.dom.attribute(r, &id).map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut changed = false;
        for n in notes.w(note) {
            let separator = notes.dom.attribute(n, &W::name("type")).is_some();
            if !separator
                && !notes
                    .dom
                    .attribute(n, &id)
                    .is_some_and(|v| referenced.contains(v))
            {
                notes.dom.remove(n);
                changed = true;
            }
        }
        if changed {
            notes.store(pkg);
        }
    }
}
