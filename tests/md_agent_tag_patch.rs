// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! An agent-view revision tag after a change names that change. It is not a
//! comment.

use jubarte::markdown::{Attribution, PatchOptions, patch_critic};

fn options() -> PatchOptions {
    PatchOptions {
        old_name: "a.md".to_string(),
        new_name: "b.md".to_string(),
        owner: Attribution {
            author: "Ada Owner".to_string(),
            date: "2026-09-30T14:05:00Z".to_string(),
        },
    }
}

fn quiet(patch: &jubarte::markdown::Patch) -> bool {
    patch.hunks.iter().all(|hunk| {
        hunk.comments.is_empty() && hunk.changes.iter().all(|change| change.comment.is_none())
    })
}

/// `{r1 @A}` after an insertion attributes that insertion and adds no comment.
/// A handle the `authors:` table names uses that person.
#[test]
fn an_agent_tag_attributes_the_change_and_is_not_a_comment() {
    let patch = patch_critic("{++x++}{r1 @A}\n", &options());
    assert!(quiet(&patch), "{patch:?}");
    let change = &patch.hunks[0].changes[0];
    assert_eq!(change.new, "x");
    assert_eq!(change.author, "Ada Owner");
    assert_eq!(change.date, "2026-09-30T14:05:00Z");

    let named = "---\nauthors:\n  A: Ann Counsel  # 1 revision, 2026-10-01T09:00:00Z\n---\n\n{++x++}{r1 @A}\n";
    let patch = patch_critic(named, &options());
    assert!(quiet(&patch), "{patch:?}");
    let change = patch
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.changes)
        .find(|change| change.new == "x")
        .expect("the insertion");
    assert_eq!(change.author, "Ann Counsel");
    assert_eq!(change.date, "2026-10-01T09:00:00Z");
}
