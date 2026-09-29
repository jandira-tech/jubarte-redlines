// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What Word's own save drops from a story after Accept All / Reject All:
//!
//! - a `w:pStyle` naming a style that is not a paragraph style: Word reads it
//!   as no style (its own redlines carry `HeaderChar` there,
//!   `_to_improve_accepted_changes` 6fb9bbdb49, bc0135eaa1);
//! - an empty `w:rPr` (run or paragraph mark) and an empty `w:pPr`.

use std::collections::HashSet;

use super::comments::Parsed;
use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

/// Tidy `story_parts` as Word saves them (module docs).
pub(super) fn tidy(pkg: &mut PartFs, story_parts: &[String]) {
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
        changed |= drop_empty(&mut s.dom, s.root);
        if changed {
            s.store(pkg);
        }
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
