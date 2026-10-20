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
    /// Published comment id, for resolving the anchor through the shared map.
    #[serde(skip)]
    pub written_id: Option<u32>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct AnchorChanges {
    pub removed: Vec<String>,
    pub kept: Vec<String>,
    decisions: std::collections::HashMap<String, String>,
}

impl AnchorChanges {
    pub fn extend(
        &mut self,
        removed: Vec<String>,
        kept: Vec<String>,
        resolution: crate::revision_processor::Resolution,
    ) {
        let decision = match resolution {
            crate::revision_processor::Resolution::Accept => "acceptance",
            crate::revision_processor::Resolution::Reject => "rejection",
        };
        for id in &removed {
            self.decisions.insert(id.clone(), decision.into());
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
            crate::changes::list_changes_tolerant(source).unwrap_or_default()
        };
        let removed = groups(&self.removed)
            .into_iter()
            .map(|(c, replies)| {
                let decision = self.decisions.get(&c.id.to_string()).cloned();
                let id = format!("c{}", c.id);
                let why = why_removed(&holders, &id, &replies, decision.as_deref());
                RemovedComment {
                    id,
                    author: c.author.clone(),
                    replies,
                    why,
                    decision,
                }
            })
            .collect();
        let kept = groups(&self.kept)
            .into_iter()
            .map(|(c, replies)| KeptComment {
                id: format!("c{}", c.id),
                replies,
                anchor: "empty".into(),
                written_id: None,
            })
            .collect();
        Ok((removed, kept))
    }
}

/// `anchor removed by accepting r13`: the decision, and the source change
/// that held the mark of thread `id` (its own, else a reply's). A thread
/// whose whole range a change removed, its mark outside it, names no change.
fn why_removed(
    changes: &[crate::changes::Change],
    id: &str,
    replies: &[String],
    decision: Option<&str>,
) -> String {
    let verb = match decision {
        Some("rejection") => "rejecting",
        _ => "accepting",
    };
    let holder = std::iter::once(id)
        .chain(replies.iter().map(String::as_str))
        .find_map(|c| changes.iter().find(|ch| ch.holds_comments.iter().any(|h| h == c)));
    match holder {
        Some(change) => format!("anchor removed by {verb} {}", change.id),
        None => format!("anchor removed by {verb} the changes that held all its text"),
    }
}

/// Renumber the revision-free working document once, before deriving outputs.
/// Replay uses this allocation, never its own annotation walk.
pub(super) fn renumber_working(
    clean: &mut Vec<u8>,
    settled: bool,
) -> Result<Option<crate::revision_processor::AnnotationIds>, super::EditError> {
    if !settled {
        return Ok(None);
    }
    let mut pkg = crate::opc::PartFs::open(clean)
        .map_err(|e| super::err("INVALID_DOCUMENT", None, e.to_string()))?;
    let Some(ids) = crate::revision_processor::renumber_settled_annotations(&mut pkg) else {
        return Ok(None);
    };
    *clean = pkg
        .to_zip()
        .map_err(|e| super::err("PACKAGE_WRITE", None, e.to_string()))?;
    Ok(Some(ids))
}

pub(super) fn remap_report(
    report: &mut super::EditReport,
    ids: Option<&crate::revision_processor::AnnotationIds>,
) {
    let Some(ids) = ids else {
        return;
    };
    for outcome in &mut report.operations {
        if let Some(id) = outcome.comment_id
            && let Some(new) = ids.comments.get(&id.to_string())
        {
            outcome.comment_id = new.parse().ok();
        }
    }
    for kept in &mut report.kept_comments {
        kept.written_id = kept
            .id
            .strip_prefix('c')
            .and_then(|id| ids.comments.get(id))
            .and_then(|id| id.parse().ok());
    }
}
