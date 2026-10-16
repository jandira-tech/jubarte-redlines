// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Reflow heuristic edge cases.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

fn wrap(text: &str, width: usize) -> String {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.push(String::new());
    lines.join("\n")
}

fn round_trip(label: &str, old: &str, new: &str) -> String {
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    if acc != new || rej != old {
        panic!(
            "[{label}] round-trip failed\n  old: {old:?}\n  new: {new:?}\n  diff: {diff:?}\n  accept: {acc:?}\n  reject: {rej:?}"
        );
    }
    diff
}

#[test]
fn reflow_one_word_added_to_long_paragraph() {
    // The existing test scenario - sanity check.
    let base = "The quick brown fox jumps over the lazy dog and then runs around the old barn before sunset while the farmer watches quietly from the porch.";
    let added = base.replacen("lazy", "very lazy", 1);
    for width in [20, 24, 28, 32, 40, 48] {
        let old = wrap(base, width);
        let new = wrap(&added, width);
        let diff = round_trip(&format!("base-width-{width}"), &old, &new);
        eprintln!("width={width}: {diff}");
    }
}

#[test]
fn reflow_several_words_moved_across_lines() {
    // Multiple words shift lines: reflow should still work but the heuristic must allow up to 2 marks.
    let base = "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo";
    // Re-arrange: bravo and charlie move to next line.
    let rearranged = "alpha\nbravo charlie\ndelta echo foxtrot golf hotel india juliet kilo";
    let diff = round_trip("rearrange", base, rearranged);
    eprintln!("rearrange diff: {diff}");

    // 3+ marks (insertions+ deletions) would cause reflow to bail.
    let moved = "alpha bravo\ncharlie delta\necho foxtrot\ngolf hotel\nindia juliet kilo";
    let diff = round_trip("moved-3-word", moved, base);
    eprintln!("moved diff: {diff}");
}

#[test]
fn reflow_with_one_word_actually_changed() {
    // One word changed AND hard wrap shifts.
    let base = "the quick brown fox jumps over the lazy dog and runs to the porch";
    let added = "the quick red fox jumps over the very lazy dog and runs to the porch";
    for width in [20, 30, 40] {
        let old = wrap(base, width);
        let new = wrap(&added, width);
        let diff = round_trip(&format!("reflow-with-change-{width}"), &old, &new);
        eprintln!("reflow-change-{width}: {diff}");
    }
}

#[test]
fn reflow_similarity_below_threshold() {
    // Most words change: similarity drops below 0.8 -> reflow bails.
    let old = "alpha bravo charlie delta echo foxtrot golf hotel";
    let new = "xray yankee tango zulu uniform victor whiskey xray";
    let diff = round_trip("unrelated-paragraph", old, new);
    eprintln!("unrelated diff: {diff}");
}

#[test]
fn reflow_too_many_marks_bails() {
    // 3 marks should bail reflow (pair line-by-line).
    let old = "alpha bravo charlie delta echo foxtrot";
    let new = "alpha bravo charlie delta echo zulu";
    // The pair algorithm should word-diff this, since both are single lines.
    // To trigger reflow's bail logic, we need 3 marks in words_diff of flat.
    // This is harder. Let's just probe one word change in a wrap.
    let base = "alpha bravo charlie delta echo foxtrot";
    let added = "alpha bravo charlie delta zulu uniform";
    for width in [10, 12, 15] {
        let old = wrap(base, width);
        let new = wrap(&added, width);
        let diff = round_trip(&format!("two-word-{width}"), &old, &new);
        eprintln!("two-word-{width}: {diff}");
    }
}

#[test]
fn reflow_paragraph_with_block_marker_starts() {
    // Should NOT use reflow (has list/heading markers).
    let old = "# alpha bravo charlie delta echo foxtrot";
    let new = "# alpha bravo charlie delta echo zulu";
    let diff = round_trip("heading-no-reflow", old, new);
    eprintln!("heading diff: {diff}");

    let old = "- alpha bravo charlie delta echo foxtrot\n- last";
    let new = "- alpha bravo charlie delta echo zulu\n- last";
    let diff = round_trip("list-no-reflow", old, new);
    eprintln!("list diff: {diff}");
}

#[test]
fn reflow_inside_indented_block() {
    // 4-space indented -> code block. Should NOT use reflow.
    let old = "    alpha bravo charlie\n    delta echo\n";
    let new = "    alpha bravo charlie\n    delta zulu\n";
    let diff = round_trip("code-block-no-reflow", old, new);
    eprintln!("code-block diff: {diff:?}");
}

#[test]
fn reflow_indented_continuation() {
    // 4-space indented continuation lines are code blocks, not reflow.
    let old = "first paragraph\n    indented\n    continuation\n";
    let new = "first paragraph changed\n    indented\n    continuation\n";
    let diff = round_trip("indented-cont", old, new);
    eprintln!("indented-cont diff: {diff:?}");
}

#[test]
fn reflow_long_wrap_with_punctuation() {
    // Punctuation tokens shouldn't break the heuristic.
    let base = "He paid $5,000,000 for the 3.5% stake; the buyer (Buyer Corp.) said: \"Sale.\"";
    let added = "He paid $5,000,000 for the 4.25% stake; the buyer (Buyer Corp.) said: \"Sale.\"";
    for width in [30, 40, 50] {
        let old = wrap(base, width);
        let new = wrap(&added, width);
        let diff = round_trip(&format!("pun-{width}"), &old, &new);
        eprintln!("pun-{width}: {diff}");
    }
}

#[test]
fn reflow_empty_line_inside_paragraph_block() {
    // A blank line should break the paragraph (reflow only sees one block).
    let old = "alpha bravo\ncharlie delta\n\necho foxtrot\n";
    let new = "alpha bravo charlie\ndelta\necho foxtrot\n";
    let diff = round_trip("blank-inside-block", old, new);
    eprintln!("blank-inside diff: {diff:?}");
}

#[test]
fn reflow_just_rewrap() {
    // Pure rewrap, no content change. Should not introduce marks.
    let old = "alpha bravo\ncharlie delta";
    let new = "alpha\nbravo charlie\ndelta";
    let diff = diff_markdown(old, new);
    eprintln!("reflow-just-rewrap:\n{diff:?}");
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
}

#[test]
fn reflow_one_word_moved() {
    // One word moves from one line to another.
    let old = "alpha bravo charlie\ndelta echo foxtrot";
    let new = "alpha bravo\ncharlie delta echo foxtrot";
    let diff = diff_markdown(old, new);
    eprintln!("reflow-moved:\n{diff}");
}

#[test]
fn reflow_two_words_moved() {
    // Two words move.
    let old = "alpha bravo charlie\ndelta echo foxtrot\ngolf hotel";
    let new = "alpha\nbravo charlie delta\necho foxtrot\ngolf hotel";
    let diff = diff_markdown(old, new);
    eprintln!("reflow-two-moved:\n{diff:?}");
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
    assert_eq!(acc, new, "accept mismatch\ndiff: {diff}");
    assert_eq!(rej, old, "reject mismatch\ndiff: {diff}");
}

#[test]
fn reflow_two_independent_paragraphs() {
    // Two independent paragraphs, each with a rewrap.
    let old = "first\n\nsecond";
    let new = "fi\nrst\n\nsec\nond";
    let diff = diff_markdown(old, new);
    eprintln!("reflow-two-paras:\n{diff}");
}

#[test]
fn reflow_similar_but_independent_lines() {
    // Independent lines, each similar but not a reflow.
    let old = "- alpha bravo\n- charlie delta";
    let new = "- alpha bravo\n- charlie delta changed";
    let diff = diff_markdown(old, new);
    eprintln!("reflow-list-lines:\n{diff}");
}