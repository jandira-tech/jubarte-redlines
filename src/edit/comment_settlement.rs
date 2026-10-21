// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Thread-level receipts for revision settlement, in the source's cN ids.

use serde::{Deserialize, Serialize};

/// A thread removed when settlement removes its anchor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovedComment {
    /// Source comment id, as used in plans.
    pub id: String,
    /// Root comment's author.
    pub author: String,
    /// Removed replies, in comment order.
    pub replies: Vec<String>,
    /// Why the thread went: `anchor removed by accepting r13`, naming the
    /// change that held its mark in the source (#1159).
    #[serde(default)]
    pub why: String,
    /// The pass that removed this thread; presentation metadata, not a source id.
    #[serde(skip)]
    pub decision: Option<String>,
}

/// A thread rescued onto an empty anchor by an edit plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeptComment {
    /// Source comment id, as used in plans.
    pub id: String,
    /// Kept replies, in comment order.
    pub replies: Vec<String>,
    /// The new anchor kind (`empty`).
    pub anchor: String,
}

use crate::revision_processor::Resolution;

/// The pass that removed a thread: how it resolved, and the change ids it
/// selected (`None` for a blanket pass over every change left).
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pass {
    resolution: Resolution,
    settled: Option<Vec<String>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct AnchorChanges {
    pub removed: Vec<String>,
    pub kept: Vec<String>,
    passes: std::collections::HashMap<String, Pass>,
}

impl AnchorChanges {
    /// Record a pass's receipts: `settled` names the ids it selected, or
    /// `None` when it resolved every change left.
    pub fn extend(
        &mut self,
        removed: Vec<String>,
        kept: Vec<String>,
        resolution: Resolution,
        settled: Option<&[String]>,
    ) {
        let pass = Pass {
            resolution,
            settled: settled.map(<[String]>::to_vec),
        };
        for id in &removed {
            self.passes.insert(id.clone(), pass.clone());
        }
        self.removed.extend(removed);
        self.kept.extend(kept);
        // Rescue is provisional until every selected and blanket pass finishes.
        self.kept.retain(|id| !self.removed.contains(id));
    }

    pub fn records(
        &self,
        source: &[u8],
    ) -> Result<(Vec<RemovedComment>, Vec<KeptComment>), super::EditError> {
        assert!(
            self.removed.iter().all(|id| !self.kept.contains(id)),
            "a comment cannot be both removed and kept"
        );
        if self.removed.is_empty() && self.kept.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }
        let comments = crate::comments::list_comments(source)
            .map_err(|e| super::err("INVALID_DOCUMENT", None, e.to_string()))?;
        let groups = |ids: &[String]| {
            comments
                .iter()
                .filter(|c| {
                    ids.contains(&c.id.to_string())
                        && !c.parent.is_some_and(|p| ids.contains(&p.to_string()))
                })
                .map(|root| {
                    let mut family = vec![root.id];
                    loop {
                        let before = family.len();
                        for c in &comments {
                            if ids.contains(&c.id.to_string())
                                && !family.contains(&c.id)
                                && c.parent.is_some_and(|p| family.contains(&p))
                            {
                                family.push(c.id);
                            }
                        }
                        if before == family.len() {
                            break;
                        }
                    }
                    let replies = comments
                        .iter()
                        .filter(|c| c.id != root.id && family.contains(&c.id))
                        .map(|c| format!("c{}", c.id))
                        .collect::<Vec<String>>();
                    (root, replies)
                })
                .collect::<Vec<_>>()
        };
        let holders = if self.removed.is_empty() {
            Vec::new()
        } else {
            crate::changes::list_changes_tolerant(source)
                .map_err(|e| super::err("INVALID_DOCUMENT", None, e.to_string()))?
        };
        let removed = groups(&self.removed)
            .into_iter()
            .map(|(c, replies)| {
                let pass = self
                    .passes
                    .get(&c.id.to_string())
                    .expect("every removed thread is recorded with its pass");
                let id = format!("c{}", c.id);
                let thread: Vec<String> = std::iter::once(id.clone())
                    .chain(replies.iter().cloned())
                    .collect();
                let why = crate::changes::why_removed(
                    &holders,
                    &thread,
                    pass.resolution,
                    pass.settled.as_deref(),
                );
                RemovedComment {
                    id,
                    author: c.author.clone(),
                    replies,
                    why,
                    decision: Some(
                        match pass.resolution {
                            Resolution::Accept => "acceptance",
                            Resolution::Reject => "rejection",
                        }
                        .into(),
                    ),
                }
            })
            .collect();
        let kept = groups(&self.kept)
            .into_iter()
            .map(|(c, replies)| KeptComment {
                id: format!("c{}", c.id),
                replies,
                anchor: "empty".into(),
            })
            .collect();
        Ok((removed, kept))
    }
}
