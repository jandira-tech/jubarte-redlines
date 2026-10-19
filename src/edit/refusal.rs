// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Review choices retained as data until the caller's grammar is known.

use serde::{Deserialize, Serialize};

use super::{EditError, Transaction, v2::Target};
use crate::namespaces::W;
use crate::result::NextAction;
use crate::xmllinq::NodeId;

/// The syntax the caller can use to resolve a blocking revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionSyntax {
    /// Legacy JSON filter selections.
    #[default]
    V1Plan,
    /// Source-bound paired-target decision arrays.
    V2Plan,
    /// Legacy `-p r7 --accept` / `--reject` groups.
    LegacyFlags,
    /// Typed inline `--accept-change` / `--reject-change` options.
    TypedAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Revision {
    target: Target,
    kind: String,
    author: String,
    accept: String,
    reject: String,
}

/// The blocking carriers and both consequences, independent of message text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionRefusal {
    reason: String,
    revisions: Vec<Revision>,
    syntax: RevisionSyntax,
}

impl RevisionRefusal {
    pub(super) fn new(tx: &Transaction<'_>, reason: &str, nodes: &[NodeId]) -> Self {
        let dom = &tx.opened.dom;
        let mut revisions = Vec::new();
        for &node in nodes {
            let ancestors = dom.ancestors(node, None);
            let story = tx
                .stories
                .iter()
                .find(|s| ancestors.contains(&s.root))
                .expect("a blocking revision belongs to a loaded story");
            let pair = super::for_edit::revision_pair(dom, node, &story.id, &story.part);
            let target = Target {
                short_id: pair.short_id,
                long_id: pair.long_id,
            };
            if revisions.iter().any(|r: &Revision| r.target == target) {
                continue;
            }
            let local = dom
                .name(node)
                .expect("revision element")
                .local_name()
                .to_string();
            let mark = dom.parent(node).is_some_and(|rpr| {
                dom.name_is(rpr, &W::r_pr())
                    && dom
                        .parent(rpr)
                        .is_some_and(|ppr| dom.name_is(ppr, &W::p_pr()))
            });
            let (kind, accept, reject) = match (local.as_str(), mark) {
                ("ins", true) => (
                    "insertion",
                    "keeps the proposed paragraph break",
                    "removes the proposed paragraph break",
                ),
                ("ins", false) => (
                    "insertion",
                    "keeps the proposed content as plain text",
                    "removes the proposed content",
                ),
                ("del", true) => (
                    "deletion",
                    "removes the deleted paragraph break",
                    "restores the deleted paragraph break",
                ),
                ("del", false) => (
                    "deletion",
                    "removes the deleted content",
                    "restores the deleted content",
                ),
                ("moveFrom" | "moveTo", _) => (
                    "move",
                    "keeps the moved content at its destination",
                    "restores the moved content at its origin",
                ),
                _ => (
                    "formatting",
                    "keeps the proposed formatting",
                    "restores the previous formatting",
                ),
            };
            revisions.push(Revision {
                target,
                kind: kind.into(),
                author: dom.attribute(node, &W::author()).unwrap_or_default().into(),
                accept: accept.into(),
                reject: reject.into(),
            });
        }
        Self {
            reason: reason.into(),
            revisions,
            syntax: RevisionSyntax::V1Plan,
        }
    }

    fn consequence(&self, accept: bool) -> String {
        self.revisions
            .iter()
            .map(|r| {
                let effect = if accept { &r.accept } else { &r.reject };
                if self.revisions.len() == 1 {
                    effect.clone()
                } else {
                    format!("{effect} for {}", r.target.short_id)
                }
            })
            .collect::<Vec<_>>()
            .join(" and ")
    }
}

impl EditError {
    /// Render all revision refusals, including operation outcomes, for a caller.
    #[must_use]
    pub fn with_revision_syntax(mut self, syntax: RevisionSyntax) -> Self {
        // The message's routes are already in the caller's syntax
        // (`hint::PlanSyntax`); the data records the same choice.
        for refusal in self.revision_refusals.iter_mut() {
            refusal.syntax = syntax;
        }
        self
    }

    /// Two explicit decisions as executable template-generation argument arrays.
    /// The templates go to stdout; source documents are never changed here.
    #[must_use]
    pub fn revision_next_actions(&self, source_path: &str, source_digest: &str) -> Vec<NextAction> {
        // Only the refusal the message names: a revision another operation
        // met must not answer for a refusal of a different kind.
        let Some(refusal) = self.revision_refusals.first().filter(|refusal| {
            self.message.starts_with(&refusal.reason)
                && refusal
                    .revisions
                    .iter()
                    .all(|r| self.message.contains(&r.target.short_id))
        }) else {
            return Vec::new();
        };
        [true, false].into_iter().map(|accept| {
            let key = if accept { "accept" } else { "reject" };
            let mut argv = vec!["jubarte".into(), "edit".into(), "template".into(), source_path.into(),
                "--expect-sha256".into(), source_digest.into()];
            for revision in &refusal.revisions {
                argv.extend([format!("--{key}-change"), revision.target.short_id.clone()]);
            }
            argv.extend(["-o".into(), "-".into()]);
            NextAction::new(argv).with_purpose(format!(
                "{}; generate a fresh plan and deliberately author the refreshed edit before applying it",
                refusal.consequence(accept)
            ))
        }).collect()
    }
}

impl Transaction<'_> {
    pub(super) fn attach_revision_refusal(&self, mut error: EditError) -> EditError {
        if let Some(refusal) = self.revision_refusal.take() {
            let mut refusals = error.revision_refusals.into_vec();
            refusals.push(refusal);
            error.revision_refusals = refusals.into_boxed_slice();
        }
        error
    }
}
