// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What Word's own save drops from a story after Accept All / Reject All:
//!
//! - a `w:pStyle` naming a style that is not a paragraph style: Word reads it
//!   as no style (its own redlines carry `HeaderChar` there,
//!   `_to_improve_accepted_changes` 6fb9bbdb49, bc0135eaa1);
//! - an empty `w:rPr` (run or paragraph mark) and an empty `w:pPr`;
//! - the boundary between adjacent tables alike in every whole-table
//!   property: Word's model holds them as one table (R8, bench
//!   `rejected_tracking` 72cc9f4ac6, 3d4318d7e9; the 11 adjacent pairs in
//!   Word's own outputs all differ in `tblStyle`);
//! - a header or footer part no section references (R9, 205503ead9).

use std::collections::HashSet;

use super::comments::Parsed;
use crate::namespaces::{R, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

/// Tidy `story_parts` as Word saves them (module docs).
pub(super) fn tidy(pkg: &mut PartFs, story_parts: &[String]) {
    drop_unreferenced_headers_footers(pkg);
    let foreign = non_paragraph_styles(pkg);
    for part in story_parts {
        let Some(mut s) = Parsed::load(pkg, part) else {
            continue;
        };
        let mut changed = false;
        if !foreign.is_empty() {
            for ps in s.w("pStyle") {
                if s.dom
                    .attribute(ps, &W::val())
                    .is_some_and(|v| foreign.contains(v))
                {
                    s.dom.remove(ps);
                    changed = true;
                }
            }
        }
        let merged = super::merge_adjacent_tables_like_word(&mut s.dom, s.root);
        if merged != s.root {
            s.dom.replace_with(s.root, &[merged]);
            s.root = merged;
            changed = true;
        }
        changed |= drop_empty(&mut s.dom, s.root);
        if changed {
            s.store(pkg);
        }
    }
}

/// Remove the header and footer parts no section references any more.
fn drop_unreferenced_headers_footers(pkg: &mut PartFs) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(doc) = Parsed::load(pkg, &main) else {
        return;
    };
    let referenced: HashSet<String> = ["headerReference", "footerReference"]
        .into_iter()
        .flat_map(|local| doc.w(local))
        .filter_map(|r| doc.dom.attribute(r, &R::name("id")).map(str::to_string))
        .collect();
    let orphans: Vec<String> = pkg
        .read_rels_for(&main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| r.rel_type.ends_with("/header") || r.rel_type.ends_with("/footer"))
                .filter(|r| !referenced.contains(&r.id))
                .map(|r| r.id.clone())
                .collect()
        })
        .unwrap_or_default();
    for id in orphans {
        pkg.remove_related_part(&main, &id);
    }
}

/// Style ids of the character, table and numbering styles.
fn non_paragraph_styles(pkg: &PartFs) -> HashSet<String> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(styles) = pkg
        .read_rels_for(&main)
        .and_then(|rels| {
            rels.items
                .iter()
                .find(|r| r.rel_type.ends_with("/styles"))
                .map(|r| pkg.resolve_rel_target(&main, &r.target))
        })
        .and_then(|name| Parsed::load(pkg, &name))
    else {
        return HashSet::new();
    };
    let (ty, id) = (W::name("type"), W::name("styleId"));
    styles
        .w("style")
        .into_iter()
        .filter(|&st| {
            styles
                .dom
                .attribute(st, &ty)
                .is_some_and(|t| t != "paragraph")
        })
        .filter_map(|st| styles.dom.attribute(st, &id).map(str::to_string))
        .collect()
}

/// Remove the empty `w:rPr` of runs and paragraph marks, then the empty
/// `w:pPr` of paragraphs. Returns whether anything went.
fn drop_empty(dom: &mut Dom, root: NodeId) -> bool {
    let mut changed = false;
    let parent_is = |dom: &Dom, e: NodeId, names: &[_]| {
        dom.parent(e)
            .and_then(|p| dom.name(p))
            .is_some_and(|n| names.contains(&n))
    };
    for rpr in dom.descendants(root, Some(&W::r_pr())) {
        if dom.child_count(rpr) == 0 && parent_is(dom, rpr, &[W::r(), W::p_pr()]) {
            dom.remove(rpr);
            changed = true;
        }
    }
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        if dom.child_count(ppr) == 0 && parent_is(dom, ppr, &[W::p()]) {
            dom.remove(ppr);
            changed = true;
        }
    }
    changed
}
