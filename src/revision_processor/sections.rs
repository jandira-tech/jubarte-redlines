// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Section headers and footers through Accept All and Reject All, as Word
//! keeps them.
//!
//! A section without header or footer references shows its predecessor's:
//! Word's "link to previous". Resolving a section break away (accepting its
//! deleted paragraph mark, rejecting its inserted one) removes that
//! predecessor, so Word writes its references into the next section when
//! that section has none of its own (`_to_improve_accepted_changes`
//! 29e3872eed, 4eff11f045; bench `rejected_tracking` 589b832f86,
//! 7787357743, ecad91f16e). A next section with a reference of its own keeps
//! just its own (205503ead9: its header stays, the break's footer goes).

use super::paragraph_mark_is_deleted_or_moved_from;
use crate::namespaces::{R, W};
use crate::xmllinq::{Dom, NodeId};

/// Which revisions the package resolution removes.
#[derive(Clone, Copy)]
pub(super) enum Resolution {
    Accept,
    Reject,
}

/// Before the revisions are resolved: give each section that follows a
/// vanishing section break, and has no header or footer reference, the
/// break's references.
pub(super) fn carry_vanishing_section_references(
    dom: &mut Dom,
    root: NodeId,
    resolution: Resolution,
) {
    let sect_pr = W::sect_pr();
    let sections = dom.descendants(root, Some(&sect_pr));
    let vanishing = |dom: &Dom, s: NodeId| {
        dom.parent(s)
            .filter(|&ppr| dom.name(ppr) == Some(W::p_pr()))
            .and_then(|ppr| dom.parent(ppr))
            .is_some_and(|p| match resolution {
                Resolution::Accept => paragraph_mark_is_deleted_or_moved_from(dom, p),
                Resolution::Reject => paragraph_mark_is_inserted_or_moved_to(dom, p),
            })
    };
    for pair in sections.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        if !vanishing(dom, from) || !references(dom, to).is_empty() {
            continue;
        }
        // Schema order: the references lead the sectPr.
        for r in references(dom, from).into_iter().rev() {
            let c = dom.clone_subtree(r);
            dom.add_first(to, c);
        }
    }
}

fn paragraph_mark_is_inserted_or_moved_to(dom: &Dom, p: NodeId) -> bool {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.element(rpr, &W::ins()).is_some() || dom.element(rpr, &W::move_to()).is_some()
        })
}

/// The header and footer references of a sectPr.
fn references(dom: &Dom, sect: NodeId) -> Vec<NodeId> {
    dom.elements(sect, None)
        .into_iter()
        .filter(|&e| {
            dom.name(e)
                .is_some_and(|n| n == W::name("headerReference") || n == W::name("footerReference"))
        })
        .filter(|&e| dom.attribute(e, &R::name("id")).is_some())
        .collect()
}
