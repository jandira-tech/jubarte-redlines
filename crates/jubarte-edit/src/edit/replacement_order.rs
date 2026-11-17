// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The comparer models Compare Documents; an edit models typing over text.
//! Normalize only the edit's compared redline, after whole-span and range
//! restoration. Retained-history edits already emit deletion then insertion.
//! Every pair this pass meets is the edit's own: the compared base holds no
//! received revisions, because `existing_revisions` refuses, accepts or
//! rejects them first, and `keep` is emitted directly.

use crate::comparer::finalize::{fields_complete, fix_up_revision_ids, revision_wrapper};
use crate::inspect::Opened;
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

use super::{EditError, err};

/// Put all deleted pieces of an adjacent, equally attributed replacement
/// before its inserted pieces, preserving each side's formatting and order.
/// Hyperlinks containing only one revision kind travel with that side, as in
/// the comparer. Paragraph-mark properties stay in their schema order.
/// Every element is visited as a parent, so a pair inside a content control,
/// smart tag or simple field is reordered within it. Any other sibling (a
/// plain run, a bookmark or comment mark, a move) ends the group, and so does
/// a wrapper with no run or with an incomplete field.
fn reorder(dom: &mut Dom, root: NodeId) -> bool {
    let mut changed = false;
    for parent in dom.descendants_and_self(root, None) {
        let children = dom.elements(parent, None);
        let mut at = 0;
        while at < children.len() {
            let Some((_, author, date)) = revision_wrapper(dom, children[at]) else {
                at += 1;
                continue;
            };
            let start = at;
            let mut deleted = Vec::new();
            let mut inserted = Vec::new();
            let mut inverted = false;
            while let Some(&child) = children.get(at) {
                let Some((insertion, a, d)) = revision_wrapper(dom, child) else {
                    break;
                };
                if a != author
                    || d != date
                    || dom.descendants(child, Some(&W::r())).is_empty()
                    || !fields_complete(dom, child, &W::fld_char())
                {
                    break;
                }
                if insertion {
                    inserted.push(child);
                } else {
                    inverted |= !inserted.is_empty();
                    deleted.push(child);
                }
                at += 1;
            }
            if inverted {
                // Move each side before the original successor. Appending
                // at the end uses the same stable partition, without cloning.
                let next = children.get(at).copied();
                for child in deleted.into_iter().chain(inserted) {
                    dom.remove(child);
                    match next {
                        Some(next) => dom.add_before_self(next, child),
                        None => dom.add(parent, child),
                    }
                }
                changed = true;
            }
            at = at.max(start + 1);
        }
    }
    changed
}

/// The edit's compared output uses Word's typing order in every story, with
/// revision ids allocated in the resulting document order. The public
/// comparer never calls this pass. No-op documents retain their exact bytes.
pub(super) fn typing_order(bytes: &[u8]) -> Result<Vec<u8>, EditError> {
    let mut opened = Opened::open(bytes).map_err(super::open_error)?;
    let mut parts = vec![(opened.main.clone(), opened.document, opened.body)];
    for (_, _, part) in opened.story_parts() {
        let xml = opened
            .pkg
            .part_string(&part)
            .ok_or_else(|| err("INVALID_DOCUMENT", None, format!("missing story {part}")))?;
        let document = opened.dom.parse_xdocument(&xml);
        let root = opened
            .dom
            .root(document)
            .ok_or_else(|| err("INVALID_DOCUMENT", None, format!("missing root in {part}")))?;
        parts.push((part, document, root));
    }
    let mut changed = false;
    for &(_, _, root) in &parts {
        changed |= reorder(&mut opened.dom, root);
    }
    if !changed {
        return Ok(bytes.to_vec());
    }
    let roots: Vec<_> = parts.iter().map(|&(_, _, root)| root).collect();
    fix_up_revision_ids(&mut opened.dom, &roots);
    for (part, document, _) in parts {
        opened
            .pkg
            .set_part(&part, opened.dom.serialize_document(document).into_bytes());
    }
    opened
        .pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}
