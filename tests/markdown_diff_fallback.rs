// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Investigate the position-pairing fallback when MAX_PAIRINGS is exceeded.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

#[test]
fn fallback_reorders_lines_incorrectly() {
    // Old: [a, b, c]. New: [b, a, c].
    // Position fallback pairs (a,b), (b,a)? No wait, position fallback pairs by position: (a,b), (b,a), (c,c).
    // That would diff a vs b and b vs a — incorrect!
    let old = "a\nb\nc\n";
    let new = "b\na\nc\n";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("old: {old:?}");
    println!("new: {new:?}");
    println!("diff: {diff:?}");
    println!("accept: {acc:?}");
    println!("reject: {rej:?}");
    let ok = acc == new && rej == old;
    println!("round-trip: {}", if ok { "OK" } else { "FAIL" });
}

#[test]
fn fallback_at_bound() {
    // k=2001, m=2000: just over the 4_000_000 bound.
    // Make every line different so position fallback is unambiguously wrong.
    let n = 100usize;
    let old: String = (0..n)
        .map(|i| format!("row_old_{i} alpha beta\n"))
        .collect();
    let new: String = (0..n)
        .map(|i| format!("row_new_{i} alpha beta\n"))
        .collect();
    let diff = diff_markdown(&old, &new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("fallback acc == new: {}", acc == new);
    println!("fallback rej == old: {}", rej == old);
}

#[test]
fn fallback_only_lines_no_block_marker() {
    // Smaller test - position fallback for many lines.
    let old: String = (0..2200)
        .map(|i| format!("row_{i} alpha beta\n"))
        .collect();
    let new: String = (0..2200)
        .map(|i| format!("row_{i} alpha beta changed\n"))
        .collect();
    // k*m ≈ 4_840_000 - over 4M bound, but small enough to finish.
    let diff = diff_markdown(&old, &new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("2200 lines fallback acc==new: {}", acc == new);
    println!("2200 lines fallback rej==old: {}", rej == old);
}