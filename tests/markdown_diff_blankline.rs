// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Minimal blank-line investigation.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

fn dump(label: &str, old: &str, new: &str) {
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("--- {label} ---");
    println!("old:    {old:?}");
    println!("new:    {new:?}");
    println!("diff:   {diff:?}");
    println!("accept: {acc:?}");
    println!("reject: {rej:?}");
    let ok = acc == new && rej == old;
    println!("round-trip: {}", if ok { "OK" } else { "FAIL" });
}

#[test]
fn blank_line_minimal() {
    dump("blank between paras", "alpha\n\nbeta\n", "alpha\nbeta\n");
    dump("blank added", "alpha\nbeta\n", "alpha\n\nbeta\n");
    dump("blank removed", "alpha\n\nbeta\n", "alpha\nbeta\n");
    dump("two blanks", "alpha\n\n\nbeta\n", "alpha\nbeta\n");
}

#[test]
fn blank_line_in_long_para() {
    // Blank line inside what looks like a reflow paragraph.
    dump(
        "blank-inside-block-1",
        "alpha bravo\ncharlie delta\n\necho foxtrot\n",
        "alpha bravo charlie\ndelta\necho foxtrot\n",
    );
    dump(
        "blank-inside-block-2",
        "alpha bravo\ncharlie delta\n\necho foxtrot\n",
        "alpha bravo\ncharlie delta\nfoo bar\necho foxtrot\n",
    );
}

#[test]
fn blank_line_in_hunk() {
    dump("between-eq", "x\n\nbeta\ny\n", "x\nalpha\n\nbeta\ny\n");
}