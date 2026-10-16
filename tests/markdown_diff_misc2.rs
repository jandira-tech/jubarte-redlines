// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Specific attack vectors from the prompt.
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

fn nl(s: &str) -> String {
    let mut s = s.to_string();
    s.push('\n');
    s
}

#[test]
fn code_backticks_different_lengths() {
    // Different code-span widths.
    for (label, old, new) in [
        ("single", "a `f` b\n", "a `g` b\n"),
        ("double", "a ``f`` b\n", "a ``g`` b\n"),
        ("triple", "a ```f`` b\n", "a ```g`` b\n"),
        ("quad", "a ````f``` b\n", "a ````g``` b\n"),
    ] {
        let _ = rt(label, old, new);
    }
}

#[test]
fn nested_image_in_link_change_alt() {
    let _ = rt(
        "nested-img-link-alt",
        "see [![cat](a.png)](https://x) here\n",
        "see [![dog](a.png)](https://x) here\n",
    );
}

#[test]
fn nested_image_in_link_change_target() {
    let _ = rt(
        "nested-img-link-target",
        "see [![cat](a.png)](https://x) here\n",
        "see [![cat](b.png)](https://x) here\n",
    );
}

#[test]
fn dollar_not_math_various() {
    // $5 and $6 — writer uses $ as currency, not math.
    let _ = rt("dollar-pair", "$5 and $6 today\n", "$7 and $6 today\n");
    let _ = rt("dollar-only", "$5 today\n", "$7 today\n");
    let _ = rt("dollar-mid", "is $5 today\n", "is $5 tomorrow\n");
}

#[test]
fn escaped_dollar() {
    let _ = rt("escaped-dollar", &nl(r"\$5 today"), &nl(r"\$6 today"));
    let _ = rt("escaped-dollar-only", &nl(r"\$5"), &nl(r"\$6"));
}

#[test]
fn display_math_multiline() {
    let old = "Area is\n$$\na+b\n$$\nnow.\n";
    let new = "Area is\n$$\na+c\n$$\nnow.\n";
    let _ = rt("display-math-ml", old, new);
}

#[test]
fn code_with_cm_delim_inside() {
    let _ = rt(
        "cm-in-code-1",
        "use `{++}` here.\n",
        "use `{--}` here.\n",
    );
    let _ = rt(
        "cm-in-code-double",
        "use ``{++}`` here.\n",
        "use ``{--}`` here.\n",
    );
}

#[test]
fn backslash_runs() {
    let _ = rt("odd-bs", &nl(r"\body"), &nl(r"\changed"));
    let _ = rt("even-bs", &nl(r"\\body"), &nl(r"\\changed"));
    let _ = rt("triple-bs", &nl(r"\\\{body"), &nl(r"\\\{changed"));
}

#[test]
fn list_marker_variants() {
    let _ = rt("star-bullet", "* a\n* c\n", "* a\n* b\n* c\n");
    let _ = rt("plus-bullet", "+ a\n+ c\n", "+ a\n+ b\n+ c\n");
    let _ = rt("dash-bullet", "- a\n- c\n", "- a\n+ b\n+ c\n");
    let _ = rt("9-to-10", "9. nine\n", "10. ten\n");
    let _ = rt("numbered-9", "9. nine\n", "9. nineb\n");
}

#[test]
fn hard_wrapped_many_moves() {
    let old = "alpha bravo charlie delta echo foxtrot golf hotel india juliet\n\
               kilo lima mike november oscar papa quebec romeo sierra\n\
               tango uniform victor whiskey xray yankee zulu\n";
    let new = "alpha bravo charlie delta echo foxtrot golf hotel india\n\
               juliet kilo lima mike november oscar papa quebec romeo\n\
               sierra tango uniform victor whiskey xray yankee zulu\n";
    let _ = rt("hard-wrap-many-moves", old, new);
}

#[test]
fn empty_line_deletion_in_hunk() {
    // Empty line in old but not new — covered already, but test here.
    let _ = rt("blank-del", "alpha\n\nbeta\n", "alpha\nbeta\n");
    let _ = rt("blank-add", "alpha\nbeta\n", "alpha\n\nbeta\n");
}

#[test]
fn escape_specific_cases() {
    // Odd/even backslashes before delimiters.
    let _ = rt("odd-before-{++", &nl(r"\{++"), &nl(r"\{--"));
    let _ = rt("even-before-{++", &nl(r"\\{++"), &nl(r"\\{--"));
    let _ = rt("triple-before-{++", &nl(r"\\\{++"), &nl(r"\\\{--"));

    let _ = rt("tilde-0", &nl("a~>b"), &nl("a~>c"));
    let _ = rt("tilde-1", &nl(r"a\~>b"), &nl(r"a\~>c"));
    let _ = rt("tilde-2", &nl(r"a\\~>b"), &nl(r"a\\~>c"));

    // Code span skips escape.
    let _ = rt("cm-in-code-id", &nl("a `{++}` b"), &nl("a `{--}` b"));
}

#[test]
fn code_spans_spanning_lines() {
    let _ = rt(
        "code-span-ml",
        "a `b\nc` d\n",
        "a `b\nc` e\n",
    );
    let _ = rt(
        "code-span-ml-2",
        "a `b\nc` d\n",
        "a `b\nx` d\n",
    );
}

#[test]
fn double_backtick_with_inner() {
    let _ = rt(
        "double-bt-inner",
        "a ``b `c`` d\n",
        "a ``b `x`` d\n",
    );
}

#[test]
fn image_alt_with_cm_text() {
    // Image alt contains `{++}` which must be escaped on output.
    let _ = rt(
        "img-alt-cm",
        "see ![a {++} b](x.png) here\n",
        "see ![a {--} b](x.png) here\n",
    );
}