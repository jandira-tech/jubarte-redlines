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

/// The change a patch names by its text, in any hunk.
fn change<'a>(patch: &'a jubarte::markdown::Patch, text: &str) -> &'a jubarte::markdown::Change {
    patch
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.changes)
        .find(|change| change.new == text || change.old == text)
        .unwrap_or_else(|| panic!("{text} in {patch:?}"))
}

/// `??` is the empty author's handle: its row names no one, and the mark
/// stays authorless rather than taking the patch owner, as the reader keeps
/// it. A row with no date leaves the mark undated, as the reader does.
#[test]
fn the_unnamed_handle_keeps_its_change_authorless() {
    let view = "---\nauthors:\n  ??: \"\"\n---\n\n{++x++}{r5 @??}\n";
    let patch = patch_critic(view, &options());
    assert!(quiet(&patch), "{patch:?}");
    let change = change(&patch, "x");
    assert_eq!(change.author, "", "{patch:?}");
    assert_eq!(change.date, "", "{patch:?}");
}

/// A known author with no date in the table: the change keeps no date, it
/// does not borrow the owner's.
#[test]
fn a_known_undated_author_does_not_take_the_owners_date() {
    let view = "---\nauthors:\n  A: Alice\n---\n\n{++x++}{r1 @A}\n";
    let patch = patch_critic(view, &options());
    let change = change(&patch, "x");
    assert_eq!(change.author, "Alice", "{patch:?}");
    assert_eq!(change.date, "", "{patch:?}");
}

/// A nested tag (`r3 @BO re r1 @AL`) names the inner change by its first
/// tag, and is no comment.
#[test]
fn a_nested_agent_tag_attributes_the_change_and_is_not_a_comment() {
    let view = "---\nauthors:\n  AL: Ana Lima  # 1 revision, 2026-09-29T14:05:00Z\n  BO: Bo Ode  # 1 revision, 2026-09-30T09:00:00Z\n---\n\n{--x--}{r3 @BO re r1 @AL}\n";
    let patch = patch_critic(view, &options());
    assert!(quiet(&patch), "{patch:?}");
    let change = change(&patch, "x");
    assert_eq!(change.author, "Bo Ode", "{patch:?}");
    assert_eq!(change.date, "2026-09-30T09:00:00Z", "{patch:?}");
}

/// An author dated `first..last`: the change written last takes the last
/// day, as the reader writes it; the others keep the first.
#[test]
fn a_dated_range_puts_its_last_day_on_the_change_written_last() {
    let view = "---\nauthors:\n  AL: Ann Lee  # 2 revisions, 2026-03-02..2026-03-04\n---\n\n{++x++}{r1 @AL} and {++z++}{r2 @AL}\n";
    let patch = patch_critic(view, &options());
    assert_eq!(
        change(&patch, "x").date,
        "2026-03-02T00:00:00Z",
        "{patch:?}"
    );
    assert_eq!(
        change(&patch, "z").date,
        "2026-03-04T23:59:59Z",
        "{patch:?}"
    );
}

/// Only the front matter holds the `authors:` table: a body paragraph that
/// looks like one names no one.
#[test]
fn a_body_authors_block_is_not_the_table() {
    let view = "---\nauthors:\n  A: Ann Counsel  # 1 revision, 2026-10-01T09:00:00Z\n---\n\nauthors:\n  A: Mallory\n\n{++x++}{r1 @A}\n";
    let patch = patch_critic(view, &options());
    assert_eq!(change(&patch, "x").author, "Ann Counsel", "{patch:?}");
}
