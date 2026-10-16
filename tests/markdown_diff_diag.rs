// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Diagnostic tests.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

#[test]
fn diag_multiline_code_span() {
    let old = "a `b\nc` d\n";
    let new = "a `b\nx` d\n";
    let diff = diff_markdown(old, new);
    println!("old: {old:?}");
    println!("new: {new:?}");
    println!("diff: {diff:?}");
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
}

#[test]
fn diag_blank_line_in_hunk() {
    let old = "alpha\n\nbeta\n";
    let new = "alpha\nbeta\n";
    let diff = diff_markdown(old, new);
    println!("old: {old:?}");
    println!("new: {new:?}");
    println!("diff: {diff:?}");
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
}

#[test]
fn diag_just_opener() {
    let old = "x {++ y\n";
    let new = "x {++ z\n";
    let diff = diff_markdown(old, new);
    println!("old: {old:?}");
    println!("new: {new:?}");
    println!("diff: {diff:?}");
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
}