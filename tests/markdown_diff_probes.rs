// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Adversarial probes against `diff_markdown` for issues #521, #522, #523.
//! Scratch file — deleted at end of review. Not part of tracked tests.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};

/// accept(diff) == new and reject(diff) == old, with marker-level print on failure.
fn round_trip(label: &str, old: &str, new: &str) -> Result<String, String> {
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    if acc == new && rej == old {
        Ok(diff)
    } else {
        let mut msg = format!(
            "[{label}] round-trip mismatch\n  old: {old:?}\n  new: {new:?}\n  diff: {diff:?}\n  accept -> {acc:?}\n  reject -> {rej:?}"
        );
        if acc != new {
            msg.push_str(&format!("\n  expected accept: {new:?}"));
        }
        if rej != old {
            msg.push_str(&format!("\n  expected reject: {old:?}"));
        }
        Err(msg)
    }
}

#[test]
fn probe_code_spans_round_trip() {
    // Different backtick widths.
    for (label, old, new) in [
        ("single backtick inner change", "a `foo` b\n", "a `bar` b\n"),
        ("double backtick inner change", "a ``foo`` b\n", "a ``bar`` b\n"),
        ("triple backtick inner change", "a ```foo`` b\n", "a ```bar`` b\n"),
        ("different widths back to back", "a `foo` and ``bar`` here\n", "a `qux` and ``bar`` here\n"),
        ("nested-like single backtick", "a `b` c\n", "a `b` d\n"),
    ] {
        let _ = round_trip(label, old, new);
    }
}

#[test]
fn probe_dollar_sign_not_math() {
    // "$5 and $6" — these are not math, must stay literal.
    let cases: &[(&str, &str, &str)] = &[
        ("dollar figures", "$5 and $6 today\n", "$7 and $6 today\n"),
        ("dollar only", "$5\n", "$7\n"),
        ("dollar sign in money", "is $5 today\n", "is $9 today\n"),
        ("dollar sign in money 2", "is $5 today\n", "is $5 tomorrow\n"),
    ];
    for (label, old, new) in cases {
        let _ = round_trip(label, old, new);
    }
}

#[test]
fn probe_escaped_dollar() {
    // "\$5" — escaped dollar is literal text, not math.
    let _ = round_trip("escaped dollar", &nl(r"\$5 today"), &nl(r"\$6 today"));
}

#[test]
fn probe_display_math_across_lines() {
    let old = "Area is\n$$\na+b\n$$\nnow.\n";
    let new = "Area is\n$$\na+c\n$$\nnow.\n";
    let _ = round_trip("display math multi-line", old, new);
}

#[test]
fn probe_code_span_containing_criticmarkup() {
    let old = "use `{++}` here.\n";
    let new = "use `{--}` here.\n";
    let _ = round_trip("cm inside code", old, new);
    // Double backtick enclosing cm delimiters.
    let old = "use ``{++}`` here.\n";
    let new = "use ``{--}`` here.\n";
    let _ = round_trip("cm inside double code", old, new);
}

fn nl(s: &str) -> String {
    let mut s = s.to_string();
    s.push('\n');
    s
}

#[test]
fn probe_lines_with_backslash_runs() {
    let a1 = nl(r"\body");
    let a2 = nl(r"\changed");
    let b1 = nl(r"\\body");
    let b2 = nl(r"\\changed");
    let c1 = nl(r"a\`b c");
    let c2 = nl(r"a\`d c");
    let d1 = nl(r"a\\`b c");
    let d2 = nl(r"a\\`d c");
    let cases: Vec<(&str, &str, &str)> = vec![
        ("odd backslash before delim", &a1, &a2),
        ("even backslashes before delim", &b1, &b2),
        ("backslash before backtick", &c1, &c2),
        ("double backslash before backtick", &d1, &d2),
    ];
    for (label, old, new) in &cases {
        let _ = round_trip(label, old, new);
    }
}

#[test]
fn probe_list_markers() {
    let cases: &[(&str, &str, &str)] = &[
        ("star bullet", "* a\n* c\n", "* a\n* b\n* c\n"),
        ("plus bullet", "+ a\n+ c\n", "+ a\n+ b\n+ c\n"),
        ("dash bullet", "- a\n- c\n", "- a\n- b\n- c\n"),
        ("9 to 10 digit", "9. ninth\n", "10. tenth\n"),
        ("9 to 9 numbered", "9. nine\n", "9. nineb\n"),
    ];
    for (label, old, new) in cases {
        let _ = round_trip(label, old, new);
    }
}

#[test]
fn probe_hard_wrapped_paragraph_with_many_moves() {
    // Several words move across lines.
    let old = "alpha bravo charlie delta echo foxtrot golf hotel india juliet\n\
               kilo lima mike november oscar papa quebec romeo sierra\n\
               tango uniform victor whiskey xray yankee zulu\n";
    let new = "alpha bravo charlie delta echo foxtrot golf hotel india\n\
               juliet kilo lima mike november oscar papa quebec romeo\n\
               sierra tango uniform victor whiskey xray yankee zulu\n";
    let _ = round_trip("hard-wrap multi-move", old, new);
}

#[test]
fn probe_empty_lines_in_hunk() {
    let old = "alpha\n\nbeta\n";
    let new = "alpha\n\nbeta changed\n";
    let _ = round_trip("blank line in hunk", old, new);
}

#[test]
fn probe_escape_change_backslash_parity() {
    // 1 backslash before `{++` — round-trip only; the diff can differ.
    let _ = round_trip("odd bs before cm", &nl(r"\{++"), &nl(r"\{--"));
    // 2 backslashes — should NOT add a third.
    let _ = round_trip("even bs before cm", &nl(r"\\{++"), &nl(r"\\{--"));
    // 3 backslashes — round-trip.
    let _ = round_trip("triple bs before cm", &nl(r"\\\{++"), &nl(r"\\\{--"));
    // ~> preceded by 0 backslashes — round-trip.
    let _ = round_trip("tilde 0", &nl("a~>b"), &nl("a~>c"));
    // ~> preceded by 1 backslash.
    let _ = round_trip("tilde 1", &nl(r"a\~>b"), &nl(r"a\~>c"));
    // ~> preceded by 2 backslashes.
    let _ = round_trip("tilde 2", &nl(r"a\\~>b"), &nl(r"a\\~>c"));
    // Word-level change after odd backslash.
    let _ = round_trip("body w 1 backslash", &nl(r"\body"), &nl(r"\changed"));
    // Body change after even backslashes.
    let _ = round_trip("body w 2 backslashes", &nl(r"\\body"), &nl(r"\\changed"));
}

#[test]
fn probe_code_spans_spanning_lines() {
    // A code span that spans a single line break (allowed by CommonMark).
    let old = "a `b\nc` d\n";
    let new = "a `b\nx` d\n";
    let _ = round_trip("multiline code span", old, new);
}

#[test]
fn probe_double_backtick_code_spans() {
    // Double backticks — content may contain single backticks.
    let old = "a ``b `c`` d\n";
    let new = "a ``b `x`` d\n";
    let _ = round_trip("double backtick with inner", old, new);
}

#[test]
fn probe_image_inside_link_change() {
    // Image alt changes, link wraps it.
    let old = "see [![cat](a.png)](https://x) here\n";
    let new = "see [![dog](a.png)](https://x) here\n";
    let _ = round_trip("image-in-link alt change", old, new);
    // Image target changes.
    let old = "see [![cat](a.png)](https://x) here\n";
    let new = "see [![cat](b.png)](https://x) here\n";
    let _ = round_trip("image-in-link target change", old, new);
    // Link URL changes only.
    let old = "see [![cat](a.png)](https://x) here\n";
    let new = "see [![cat](a.png)](https://y) here\n";
    let _ = round_trip("image-in-link url change", old, new);
}

#[test]
fn probe_image_alt_text_containing_delim() {
    // If the alt text contains `{++}`, it must NOT be escaped.
    let old = "see ![a {++} b](x.png) here\n";
    let new = "see ![a {--} b](x.png) here\n";
    let _ = round_trip("image alt contains delim", old, new);
}