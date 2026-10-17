// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Freeze original revision pairs for inline actions and generated plans.
use super::{for_edit::revision_pair, v2::Target};
use crate::{inspect::Opened, xmllinq::Dom};
fn refusal(code: &str, message: impl Into<String>) -> super::EditError {
    super::err(code, None, message)
}
fn pair(p: &super::for_edit::TargetPair) -> Target {
    Target {
        short_id: p.short_id.clone(),
        long_id: p.long_id.clone(),
    }
}
pub(super) fn decisions(source: &[u8], ids: &[String]) -> Result<Vec<Target>, super::EditError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let opened = Opened::open(source).map_err(|e| refusal("INVALID_DOCUMENT", e.to_string()))?;
    let mut inventory = Vec::new();
    for (part, _) in crate::revision_processor::revision_bearing_parts(&opened.pkg) {
        let Some(xml) = opened.pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            continue;
        };
        for node in dom.descendants(root, None) {
            if dom.name(node).is_some_and(|n| {
                crate::revision_processor::tracked_revisions_elements().contains(&n)
            }) {
                let stem = crate::ids::part_story(&part);
                let story = if part == opened.main { "body" } else { &stem };
                inventory.push(revision_pair(&dom, node, story, &part));
            }
        }
    }
    let mut selected = Vec::new();
    for id in ids {
        // A supplied long ID is resolved by exact identity, never reconstructed.
        let hits: Vec<_> = if id.contains(':') && inventory.iter().any(|p| p.long_id == *id) {
            inventory.iter().filter(|p| p.long_id == *id).collect()
        } else {
            let parsed = crate::ids::parse_change_ids(id)
                .map_err(|e| refusal("INVALID_ID", e.to_string()))?;
            let mut hits = Vec::new();
            for cid in parsed {
                let found: Vec<_> = inventory
                    .iter()
                    .filter(|p| p.short_id == cid.short())
                    .collect();
                if found.len() != 1 {
                    return Err(refusal(
                        "INVALID_TARGET",
                        format!("revision {id} does not identify one original-source carrier"),
                    ));
                }
                hits.extend(found);
            }
            hits
        };
        for hit in hits {
            let target = pair(hit);
            if !selected.contains(&target) {
                selected.push(target);
            }
        }
    }
    Ok(selected)
}
