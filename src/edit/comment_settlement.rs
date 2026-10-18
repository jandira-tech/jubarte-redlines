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

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct AnchorChanges {
    pub removed: Vec<String>,
    pub kept: Vec<String>,
}

impl AnchorChanges {
    pub fn extend(&mut self, removed: Vec<String>, kept: Vec<String>) {
        self.removed.extend(removed);
        self.kept.extend(kept);
    }

    pub fn records(
        &self,
        source: &[u8],
    ) -> Result<(Vec<RemovedComment>, Vec<KeptComment>), super::EditError> {
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
                        .collect();
                    (root, replies)
                })
                .collect::<Vec<_>>()
        };
        let removed = groups(&self.removed)
            .into_iter()
            .map(|(c, replies)| RemovedComment {
                id: format!("c{}", c.id),
                author: c.author.clone(),
                replies,
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

/// Settlement retains snapshot ids until all operations and tracked replays
/// finish. Renumber only revision-free outputs, and report newly written ids.
/// Thread settlement receipts always keep their source ids.
pub(super) fn renumber_outputs(
    clean: &mut Vec<u8>,
    redline: &mut Vec<u8>,
    report: &mut super::EditReport,
) -> Result<(), super::EditError> {
    fn renumber(
        bytes: &mut Vec<u8>,
    ) -> Result<std::collections::HashMap<String, String>, super::EditError> {
        let mut pkg = crate::opc::PartFs::open(bytes)
            .map_err(|e| super::err("INVALID_DOCUMENT", None, e.to_string()))?;
        let Some(ids) = crate::revision_processor::renumber_settled_annotations(&mut pkg) else {
            return Ok(std::collections::HashMap::new());
        };
        *bytes = pkg
            .to_zip()
            .map_err(|e| super::err("PACKAGE_WRITE", None, e.to_string()))?;
        Ok(ids)
    }
    renumber(clean)?;
    let ids = renumber(redline)?;
    for outcome in &mut report.operations {
        if let Some(id) = outcome.comment_id
            && let Some(new) = ids.get(&id.to_string())
        {
            outcome.comment_id = new.parse().ok();
        }
    }
    Ok(())
}
