// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! More targeted probes against HEAD diff.
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
    println!("round-trip: {}", if acc == new && rej == old { "OK" } else { "FAIL" });
}

#[test]
fn change_outside_code_with_cm_inside() {
    dump(
        "change-outside-cm-inside",
        "call `{++}` now\n",
        "say `{++}` tomorrow\n",
    );
    dump(
        "two-code-spans-cm",
        "see `{++}` and `{--}` here\n",
        "see `{++}` and `{--}` there\n",
    );
}

#[test]
fn image_in_link_with_escaped_alt() {
    // A change inside an image alt that has delimiter-like text.
    // The image atom should wrap the whole image; the delimiters in alt get escaped.
    dump(
        "img alt contains cm-like text",
        "see ![{++}](a.png) here\n",
        "see ![{--}](a.png) here\n",
    );
}

#[test]
fn code_span_with_newline_inside() {
    dump(
        "code span newline, change outside",
        "a `b\nc` d\n",
        "a `b\nc` e\n",
    );
    dump(
        "code span newline, change inside",
        "a `b\nc` d\n",
        "a `b\nx` d\n",
    );
}

#[test]
fn reflow_word_moves_across_lines() {
    // Without the heuristic, line pairing would rewrite each wrapped word.
    let base = "the quick brown fox jumps over the lazy dog and then runs around the old barn before sunset";
    let rearranged = "the quick brown fox jumps over the lazy dog and\nthen runs around\nthe old barn before sunset";
    dump("reflow-moves", base, rearranged);
}

#[test]
fn reflow_paragraph_with_one_block_change() {
    // 5-line paragraph with the middle line changed.
    let base = "alpha bravo charlie\ndelta echo foxtrot\ngolf hotel india\njuliet kilo lima\nmike november oscar";
    let changed = "alpha bravo charlie\ndelta echo FOXTROT\ngolf hotel india\njuliet kilo lima\nmike november oscar";
    dump("reflow-mid-change", base, changed);
}

#[test]
fn many_atoms_paragraph() {
    // Multiple emphasis spans with changes.
    let base = "*alpha* *bravo* *charlie* *delta*";
    let changed = "*alpha* *BRAVO* *charlie* *delta*";
    dump("many-emph", base, changed);
}

#[test]
fn image_change_wraps_whole() {
    dump(
        "image-target",
        "see ![alt](old.png) end\n",
        "see ![alt](new.png) end\n",
    );
    dump(
        "image-alt-and-target",
        "see ![old alt](old.png) end\n",
        "see ![new alt](new.png) end\n",
    );
}

#[test]
fn list_pairing_with_long_lines() {
    let old = "1. alpha bravo charlie delta echo\n2. alpha bravo charlie delta echo\n3. alpha bravo charlie delta echo\n";
    let new = "1. ALPHA bravo charlie delta echo\n2. alpha bravo charlie delta echo\n3. alpha bravo charlie delta echo\n";
    dump("list-pair-long", old, new);
}

#[test]
fn identical_unchanged() {
    dump(
        "identical with cm",
        "see `{++}` here\n",
        "see `{++}` here\n",
    );
    dump(
        "identical with $",
        "see $5 here\n",
        "see $5 here\n",
    );
    dump(
        "identical with image",
        "see ![a](b.png) here\n",
        "see ![a](b.png) here\n",
    );
}

#[test]
fn empty_diff() {
    let empty_old = "";
    let empty_new = "";
    dump("empty-empty", empty_old, empty_new);
    let empty_old = "";
    let new = "hello\n";
    dump("empty-new", empty_old, new);
    let old = "hello\n";
    let empty_new = "";
    dump("old-empty", old, empty_new);
}

#[test]
fn escape_only_opener_or_closer() {
    // Test escape of just opener or just closer of a delimiter pair.
    dump("just-opener", "x {++ y\n", "x {++ z\n");
    dump("just-closer", "x y ++} z\n", "x y ++\\} z\n");
}

#[test]
fn empty_line_among_changes() {
    // When a blank line falls between changes.
    let old = "alpha\n\nbeta\n";
    let new = "ALPHA\n\nbeta\n";
    dump("blank-between-changes", old, new);
}

#[test]
fn many_paragraphs_with_changes() {
    let old = "para one.\n\npara two.\n\npara three.\n\npara four.\n";
    let new = "para one.\n\nPARA TWO.\n\npara three changed.\n\npara four.\n";
    dump("many-paras-changed", old, new);
}

#[test]
fn trailing_newline_difference() {
    dump("trailing-newline", "hello", "hello\n");
    dump("trailing-newline-2", "hello\n", "hello");
}