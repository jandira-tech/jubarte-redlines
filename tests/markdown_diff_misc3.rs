// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Multi-line reflow and various edge cases.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

fn rt(label: &str, old: &str, new: &str) -> Result<String, String> {
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    if acc == new && rej == old {
        Ok(diff)
    } else {
        let mut msg = format!("[{label}] FAIL\n  diff: {diff:?}\n  acc: {acc:?}\n  rej: {rej:?}");
        if acc != new {
            msg.push_str(&format!("\n  want acc: {new:?}"));
        }
        if rej != old {
            msg.push_str(&format!("\n  want rej: {old:?}"));
        }
        Err(msg)
    }
}

#[test]
fn code_span_across_lines() {
    // The diff must wrap the WHOLE code span (which spans a line break).
    let _ = rt(
        "code-across-lines-change-inside",
        "see `b\nc` d\n",
        "see `b\nx` d\n",
    );
}

#[test]
fn reflow_with_code_span_change() {
    // Hard wrap with a code span change inside.
    let old = "alpha bravo\ncharlie `delta echo` foxtrot\ngolf hotel";
    let new = "alpha bravo\ncharlie `delta ECHO` foxtrot\ngolf hotel";
    let _ = rt("reflow-with-code-change", old, new);
}

#[test]
fn reflow_code_span_across_lines() {
    let old = "alpha bravo\ncharlie `d\ne` foxtrot\ngolf hotel";
    let new = "alpha bravo\ncharlie `d\nx` foxtrot\ngolf hotel";
    let _ = rt("reflow-with-ml-code", old, new);
}

#[test]
fn reflow_with_emphasis_change() {
    let old = "alpha *bravo* charlie\ndelta echo foxtrot\ngolf hotel";
    let new = "alpha *bravo* CHARLIE\ndelta echo foxtrot\ngolf hotel";
    let _ = rt("reflow-with-emph", old, new);
}

#[test]
fn reflow_with_link_change() {
    let old = "alpha [bravo](a) charlie\ndelta echo foxtrot\ngolf hotel";
    let new = "alpha [bravo](A) charlie\ndelta echo foxtrot\ngolf hotel";
    let _ = rt("reflow-with-link", old, new);
}

#[test]
fn reflow_single_line_change() {
    // 1-line paragraph in old, 2-line in new. Word moved.
    let old = "alpha bravo charlie delta echo foxtrot";
    let new = "alpha bravo\ncharlie delta echo foxtrot";
    let _ = rt("reflow-1-to-2", old, new);
}

#[test]
fn reflow_two_lines_to_one() {
    let old = "alpha bravo\ncharlie delta echo foxtrot";
    let new = "alpha bravo charlie delta echo foxtrot";
    let _ = rt("reflow-2-to-1", old, new);
}

#[test]
fn reflow_indented_lines_count() {
    // 4-space indented lines are code blocks (per CommonMark).
    let old = "    alpha bravo\n    charlie delta\n";
    let new = "    alpha bravo\n    charlie delta changed\n";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("indented: acc={acc:?} rej={rej:?}");
    println!("want acc: {new:?}");
    println!("want rej: {old:?}");
    assert_eq!(acc, new);
    assert_eq!(rej, old);
}

#[test]
fn similar_but_different_lines() {
    // Many lines, each with one word change. Should NOT use reflow (positions differ).
    let old = "first row alpha\nsecond row beta\nthird row gamma\nfourth row delta\nfifth row epsilon";
    let new = "first row ALPHA\nsecond row beta\nthird row GAMMA\nfourth row delta\nfifth row epsilon";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    println!("similar-lines diff: {diff:?}");
    assert_eq!(acc, new);
}

#[test]
fn indented_then_plain() {
    // A line with 4-space indent is a code block. Reflow shouldn't see the
    // whole thing as one paragraph.
    let old = "alpha bravo\n    indented\ncharlie delta";
    let new = "alpha bravo\n    indented\ncharlie changed";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("indented-then-plain diff: {diff:?}");
    println!("acc: {acc:?}");
    println!("rej: {rej:?}");
    assert_eq!(acc, new);
    assert_eq!(rej, old);
}

#[test]
fn image_in_long_paragraph() {
    // Image inside a paragraph with rewrap.
    let old = "see ![pic](a.png) here today\nand tomorrow we will go";
    let new = "see ![pic](a.png) here today\nand TOMORROW we will go";
    let _ = rt("img-in-para", old, new);
}

#[test]
fn just_text_marks_through_reflow() {
    let old = "alpha bravo\ncharlie delta";
    let new = "alpha\nbravo charlie\ndelta";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    println!("reflow diff: {diff:?}");
    assert_eq!(acc, new);
    assert_eq!(rej, old);
}