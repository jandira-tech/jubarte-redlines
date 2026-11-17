// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision wrappers: insertion-before-deletion order, adjacent-revision coalescing, move simplification.

use super::*;

/// Word emits the INSERTION (new text) before the DELETION (old text) at a
/// replacement site; our LCS produces delete-then-insert. Swap an adjacent
/// same-author/date `w:del` immediately followed by `w:ins` into `w:ins` then
/// `w:del`, matching Word's order. Text-preserving: each of the delText / ins-text
/// streams keeps its own order (only their interleaving changes). Recurses.
///
/// A wrapper holding part of a field stays put: swapping a deleted field's
/// `end` with an inserted field's `begin` crosses the two fields, and Word
/// crashed opening the redline (English pair 57f96361×3832d290). A wrapper
/// whose fields are complete swaps whole: Word's redline of super_editor
/// hyperlink_multiple_runs × diff_before10 (corpus a098bd5ad9) writes the
/// inserted "One two" before the deleted HYPERLINK field.
/// Match Word's replacement order inside `w:r` and `w:p`: insertions
/// (`w:ins`) come before deletions (`w:del`).
///
/// When the LCS pairs a fully-deleted run with a fully-inserted
/// replacement run, the rebuilder may keep `w:del` ahead of `w:ins`
/// at a child position. Word always writes `ins+del`. The pass
/// reorders only at positions where two immediate siblings are one
/// each; deeper reordering is left for the multi-paragraph peel
/// ladder.
pub fn reorder_replacements_ins_before_del(dom: &mut Dom, node: NodeId) {
    let fld_char = W::fld_char();
    let mut i = 0usize;
    loop {
        let kids = dom.nodes(node);
        if i + 1 >= kids.len() {
            break;
        }
        let (a, b) = (kids[i], kids[i + 1]);
        let is_replacement = matches!(
            (revision_wrapper(dom, a), revision_wrapper(dom, b)),
            (Some((false, author_a, date_a)), Some((true, author_b, date_b)))
                if author_a == author_b && date_a == date_b
        ) && fields_complete(dom, a, &fld_char)
            && fields_complete(dom, b, &fld_char);
        if is_replacement {
            dom.remove(b);
            dom.add_before_self(a, b); // ins (b) now precedes del (a)
            i += 2;
        } else {
            i += 1;
        }
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            reorder_replacements_ins_before_del(dom, c);
        }
    }
}

/// Every field char under `node` belongs to a field that begins and ends
/// under it: no `fldChar` at all, or begins and ends that nest and balance.
#[doc(hidden)]
pub fn fields_complete(dom: &Dom, node: NodeId, fld_char: &XName) -> bool {
    let mut depth = 0i32;
    for fc in dom.descendants(node, Some(fld_char)) {
        match dom.attribute(fc, &W::name("fldCharType")) {
            Some("begin") => depth += 1,
            Some("end") => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

/// A node that is one revision for the replacement swap: a `w:ins`
/// (`true`) or `w:del` (`false`) wrapper, or a `w:hyperlink` whose element
/// children are all one of them by one author and date (Word's redline of
/// super_editor hyperlink_multiple_runs × diff_before10, corpus a098bd5ad9,
/// writes "One two" inserted before the deleted "Click here now" link;
/// the swap saw a hyperlink, not a deletion, and left the link first).
#[doc(hidden)]
pub fn revision_wrapper(dom: &Dom, node: NodeId) -> Option<(bool, String, String)> {
    if !dom.is_element(node) {
        return None;
    }
    let own = |n: NodeId| {
        let kind = match dom.name(n).as_ref() {
            Some(name) if *name == W::ins() => true,
            Some(name) if *name == W::del() => false,
            _ => return None,
        };
        Some((
            kind,
            dom.attribute(n, &W::author()).unwrap_or("").to_string(),
            dom.attribute(n, &W::date()).unwrap_or("").to_string(),
        ))
    };
    if let Some(rev) = own(node) {
        return Some(rev);
    }
    if dom.name(node).as_ref() != Some(&W::hyperlink()) {
        return None;
    }
    let mut found: Option<(bool, String, String)> = None;
    for child in dom.nodes(node) {
        if !dom.is_element(child) {
            continue;
        }
        let rev = own(child)?;
        match &found {
            None => found = Some(rev),
            Some(first) if *first == rev => {}
            Some(_) => return None,
        }
    }
    found
}

/// Merge consecutive sibling `w:ins` (and consecutive `w:del`) wrappers that
/// share the same `w:author` and `w:date` into a single wrapper, moving the
/// later wrapper's child runs into the first and removing the emptied wrapper.
/// Runs are kept intact (per-run formatting preserved); only the wrappers merge.
/// Word never emits adjacent same-status revision wrappers; this matches that
/// (PowerTools/ours otherwise inflate the w:ins/w:del element count ~2x).
/// Recurses through the whole tree. Text-preserving (the redlined text is unchanged).
/// Merge adjacent same-status `w:ins` / `w:del` wrappers under
/// `node` into one.
///
/// Word never emits two adjacent same-status revisions; the
/// redline pipeline can otherwise inflate the element count by ~2x.
/// Text-preserving — golden text-parity is unaffected. Walks the
/// children of `node` once, so a deep tree needs
/// [`coalesce_adjacent_runs`] or similar to repeat at lower levels.
pub fn coalesce_adjacent_revisions(dom: &mut Dom, node: NodeId) {
    let ins = W::ins();
    let del = W::del();
    let kids = dom.nodes(node);
    let mut prev: Option<NodeId> = None;
    for c in kids {
        let is_rev =
            dom.is_element(c) && matches!(dom.name(c), Some(ref n) if *n == ins || *n == del);
        if is_rev {
            if let Some(p) = prev {
                let same = dom.name(p) == dom.name(c)
                    && dom.attribute(p, &W::author()).map(|s| s.to_string())
                        == dom.attribute(c, &W::author()).map(|s| s.to_string())
                    && dom.attribute(p, &W::date()).map(|s| s.to_string())
                        == dom.attribute(c, &W::date()).map(|s| s.to_string());
                if same {
                    for gc in dom.nodes(c) {
                        dom.remove(gc);
                        dom.add(p, gc);
                    }
                    dom.remove(c);
                    continue; // prev unchanged; it may absorb the next sibling too
                }
            }
            prev = Some(c);
        } else {
            prev = None; // any non-revision sibling breaks adjacency
        }
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            coalesce_adjacent_revisions(dom, c);
        }
    }
}

/// M4.F — `SimplifyMoveMarkupToDelIns` (:3309), gated by settings.simplify_move_markup
/// (default off). Converts native move markup to del/ins for Word compatibility
/// (Issue #96): w:moveFrom→w:del, w:moveTo→w:ins (keeping author/date/id +
/// children), and removes all move range markers. Transform over `root`.
/// Convert native move markup (`w:moveFrom` / `w:moveTo`) into
/// `w:del` / `w:ins` (`SimplifyMoveMarkupToDelIns`,
/// WmlComparer.cs:2874-2878).
///
/// The `w:moveFrom` / `w:moveTo` half is rewritten to its
/// `W:author` / `W:date` / `W:id` (matching the C# `?.XAttribute`
/// pattern); stray attributes such as `rsid*`, `pt:Unid`, and any
/// future vendor attrs do not leak through. Gated on
/// [`WmlComparerSettings::simplify_move_markup`] (issue #96
/// workaround); default off.
pub fn simplify_move_markup_to_del_ins(dom: &mut Dom, root: NodeId) -> NodeId {
    simplify_move_transform(dom, root)
}

pub(super) fn simplify_move_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    let name = dom.name(node).unwrap();
    let ranges = [
        W::move_from_range_start(),
        W::move_from_range_end(),
        W::move_to_range_start(),
        W::move_to_range_end(),
    ];
    let new_name = if name == W::move_from() {
        W::del()
    } else if name == W::move_to() {
        W::ins()
    } else {
        name.clone()
    };
    let ne = dom.new_element(new_name);
    // Oracle (WmlComparer.cs:2874-2878) propagates only w:author, w:date, w:id
    // — but only when rewriting w:moveFrom/w:moveTo. Recreated descendant
    // elements (w:r, w:p, …) must keep all of their attributes.
    if name == W::move_from() || name == W::move_to() {
        copy_move_id_attrs(dom, node, ne);
    } else {
        copy_attrs(dom, node, ne, false);
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            let cn = dom.name(c).unwrap();
            if ranges.contains(&cn) {
                continue; // drop range markers
            }
            let t = simplify_move_transform(dom, c);
            dom.add(ne, t);
        } else {
            let cc = dom.clone_subtree(c);
            dom.add(ne, cc);
        }
    }
    ne
}
