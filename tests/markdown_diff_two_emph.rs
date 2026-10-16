// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Two emphasis atoms in one line.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

#[test]
fn two_emph_change_one() {
    let old = "*foo* and *bar* today\n";
    let new = "*FOO* and *bar* today\n";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("two-emph-one-change:");
    println!("old:    {old:?}");
    println!("new:    {new:?}");
    println!("diff:   {diff:?}");
    println!("accept: {acc:?}");
    println!("reject: {rej:?}");
    println!("round-trip: {}", if acc == new && rej == old { "OK" } else { "FAIL" });
}

#[test]
fn two_emph_change_both() {
    let old = "*foo* and *bar* today\n";
    let new = "*FOO* and *BAR* today\n";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("two-emph-both-change:");
    println!("old:    {old:?}");
    println!("new:    {new:?}");
    println!("diff:   {diff:?}");
    println!("accept: {acc:?}");
    println!("reject: {rej:?}");
    println!("round-trip: {}", if acc == new && rej == old { "OK" } else { "FAIL" });
}

#[test]
fn three_emph_one_change() {
    let old = "*a* *b* *c*\n";
    let new = "*A* *b* *c*\n";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("three-emph-one-change:");
    println!("diff: {diff:?}");
    println!("acc/rej: {acc:?} / {rej:?}");
    println!("round-trip: {}", if acc == new && rej == old { "OK" } else { "FAIL" });
}