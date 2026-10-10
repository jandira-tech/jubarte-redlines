// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! An anchor copied out of the agent view, read as document text.

/// The ASCII punctuation a backslash escapes in CommonMark.
const ESCAPABLE: &str = r"\`*_{}[]()#+-.!|<>~=";

/// A private-use stand-in for an escaped character, so mark stripping never
/// sees it (Supplementary Private Use Area-A, U+F0000 onward).
fn hide(c: char) -> char {
    char::from_u32(0xF_0000 + u32::from(c)).unwrap_or(c)
}

fn reveal(c: char) -> char {
    match u32::from(c) {
        code @ 0xF_0000..=0xF_007F => char::from_u32(code - 0xF_0000).unwrap_or(c),
        _ => c,
    }
}

/// Removes a backslash escape before a Markdown punctuation character
/// (`\*` → `*`), as CommonMark reads it; other backslashes stay.
pub fn unescape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(&next) = chars.peek()
            && ESCAPABLE.contains(next)
        {
            out.push(next);
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

#[derive(Clone, Copy)]
enum Keep {
    Inner,
    Nothing,
    NewSide,
}

/// `open … close` spans replaced by what `keep` says; an unclosed `open`
/// stays as text.
fn strip_between(text: &str, open: &str, close: &str, keep: Keep) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        out.push_str(&rest[..start]);
        let inner_start = start + open.len();
        let Some(len) = rest[inner_start..].find(close) else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let inner = &rest[inner_start..inner_start + len];
        match keep {
            Keep::Inner => out.push_str(inner),
            Keep::Nothing => {}
            Keep::NewSide => out.push_str(inner.rsplit_once("~>").map_or(inner, |(_, new)| new)),
        }
        rest = &rest[inner_start + len + close.len()..];
    }
    out.push_str(rest);
    out
}

/// The text without a leading block mark: `# ` (one to six), `> `, a
/// bullet (`- `, `+ `, `* `) or an ordered marker (`1. `, `2) `).
fn without_block_mark(text: &str) -> &str {
    let trimmed = text.trim_start();
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if (1..=6).contains(&hashes) && trimmed.as_bytes().get(hashes) == Some(&b' ') {
        return &trimmed[hashes + 1..];
    }
    for bullet in ["> ", "- ", "+ ", "* "] {
        if let Some(rest) = trimmed.strip_prefix(bullet) {
            return rest;
        }
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=9).contains(&digits)
        && matches!(trimmed.as_bytes().get(digits), Some(b'.' | b')'))
        && trimmed.as_bytes().get(digits + 1) == Some(&b' ')
    {
        return &trimmed[digits + 2..];
    }
    text
}

/// The text with its emphasis marks dropped: `**`, `~~`, `==`, `<u>` and
/// `</u>` always; single `*` and `_` runs only when they pair up (`*x*`,
/// `_x_`, `__x__`), so `2 * 3`, `snake_case`, `snake__case` and
/// `foo__bar__` keep theirs.
fn without_emphasis(text: &str) -> String {
    let mut text = text.to_string();
    for mark in ["**", "~~", "==", "<u>", "</u>"] {
        text = text.replace(mark, "");
    }
    // A `*` with spaces on both sides (`2 * 3`) is text; the rest are marks
    // when they pair up.
    let chars: Vec<char> = text.chars().collect();
    let spaced = |i: usize| {
        i > 0 && chars[i - 1].is_whitespace() && chars.get(i + 1).is_some_and(|c| c.is_whitespace())
    };
    let star = |i: usize| chars[i] == '*' && !spaced(i);
    if (0..chars.len())
        .filter(|&i| star(i))
        .count()
        .is_multiple_of(2)
    {
        text = (0..chars.len())
            .filter(|&i| !star(i))
            .map(|i| chars[i])
            .collect();
    }
    // `_` runs pair as CommonMark pairs them: a run opens only with no
    // letter or digit before it and text after it, closes only with text
    // before it and no letter or digit after it, and closes the nearest
    // open run it may. Runs of unequal length use what they share and keep
    // the rest as text (`_foo__` is `foo_`). `snake__case` and `foo__bar__`
    // stay text.
    let chars: Vec<char> = text.chars().collect();
    let mut drop = vec![false; chars.len()];
    // (first, end of the unused delimiters, run length, can also close)
    let mut open: Vec<(usize, usize, usize, bool)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '_' {
            i += 1;
            continue;
        }
        let end = i + chars[i..].iter().take_while(|&&c| c == '_').count();
        let len = end - i;
        let before = i.checked_sub(1).map(|b| chars[b]);
        let after = chars.get(end).copied();
        let closes =
            before.is_some_and(|c| !c.is_whitespace()) && !after.is_some_and(char::is_alphanumeric);
        let opens =
            after.is_some_and(|c| !c.is_whitespace()) && !before.is_some_and(char::is_alphanumeric);
        // The first delimiter of this run not yet paired.
        let mut from = i;
        while closes && from < end {
            // CommonMark's rule of 3: when either run can both open and
            // close, their lengths may not sum to a multiple of 3 unless
            // both are multiples of 3.
            let Some(at) = open.iter().rposition(|&(_, _, opener, both)| {
                !((both || opens)
                    && (opener + len).is_multiple_of(3)
                    && !(opener.is_multiple_of(3) && len.is_multiple_of(3)))
            }) else {
                break;
            };
            let (a, z, opener, both) = open[at];
            // Both runs have two left: strong emphasis uses two each.
            let used = if z - a >= 2 && end - from >= 2 { 2 } else { 1 };
            drop[z - used..z].fill(true);
            drop[from..from + used].fill(true);
            from += used;
            // The runs between the pair stay text.
            open.truncate(at + 1);
            if z - used == a {
                open.pop();
            } else {
                open[at] = (a, z - used, opener, both);
            }
        }
        if opens && from < end {
            open.push((from, end, len, closes));
        }
        i = end;
    }
    if drop.contains(&true) {
        text = chars
            .iter()
            .zip(&drop)
            .filter(|(_, d)| !**d)
            .map(|(c, _)| *c)
            .collect();
    }
    text
}

/// The document text an agent-view fragment stands for: attribution and
/// comment notes dropped, insertions and highlights unwrapped, deletions
/// removed, substitutions replaced by their new side, a leading block mark
/// (`# `, `> `, `- `, `1. `) dropped, emphasis marks dropped, backslash
/// escapes resolved. An escaped mark (`\*\*`) is text and stays.
pub fn plain_anchor(text: &str) -> String {
    let mut hidden = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        // The view's hard break: `\` before a newline is the newline.
        if c == '\\' && chars.peek() == Some(&'\n') {
            continue;
        }
        if c == '\\'
            && let Some(&next) = chars.peek()
            && ESCAPABLE.contains(next)
        {
            hidden.push(hide(next));
            chars.next();
            continue;
        }
        hidden.push(c);
    }
    let mut text = strip_between(&hidden, "{>>", "<<}", Keep::Nothing);
    text = strip_between(&text, "{--", "--}", Keep::Nothing);
    text = strip_between(&text, "{~~", "~~}", Keep::NewSide);
    text = strip_between(&text, "{++", "++}", Keep::Inner);
    text = strip_between(&text, "{==", "==}", Keep::Inner);
    let text = without_emphasis(without_block_mark(&text));
    text.chars().map(reveal).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_anchor_drops_the_marks_the_agent_view_adds() {
        for (anchor, plain) in [
            ("# Chapter 1", "Chapter 1"),
            ("**secret**", "secret"),
            ("<u>secret</u>", "secret"),
            (
                "{==keep it secret.==}{>>#c5 @AC: Cap?<<}",
                "keep it secret.",
            ),
            ("keep {++it++}{>>#0 @AC<<} secret", "keep it secret"),
            ("keep {~~that~>it~~}{>>#0 @AC<<} secret", "keep it secret"),
            ("keep it {--very --}{>>#1 @AC<<}secret", "keep it secret"),
            ("\\*\\*stars\\*\\*", "**stars**"),
            ("1. one", "one"),
            ("- item", "item"),
            ("a_b", "a_b"),
            ("*x*", "x"),
            ("2 * 3", "2 * 3"),
            ("2 * 3 * 4", "2 * 3 * 4"),
            ("*a* times 2 * 3", "a times 2 * 3"),
            ("Intro\\\n\\# Not", "Intro\n# Not"),
            ("\\\\*", "\\*"),
            ("snake_case and _x_", "snake_case and x"),
            // pi review av4 F7: CommonMark keeps an underscore run inside a
            // word, so `snake__case` is text; `__init__` is bold `init`.
            ("snake__case", "snake__case"),
            ("a__b__c", "a__b__c"),
            ("__init__", "init"),
            ("__bold__ and snake__case", "bold and snake__case"),
            // CodeRabbit #392: an intraword run cannot open, so the trailing
            // run closes nothing and both stay text (CommonMark).
            ("foo__bar__", "foo__bar__"),
            ("__x__y", "__x__y"),
            ("a _b_ c", "a b c"),
            ("_a_ and foo__bar__", "a and foo__bar__"),
            // CodeRabbit #392 (2): runs of unequal length pair what they can
            // and keep the rest as text, as CommonMark does.
            ("_foo__", "foo_"),
            ("__foo_", "_foo"),
            ("___foo___", "foo"),
            ("_foo___", "foo__"),
            ("unclosed {++ insert", "unclosed {++ insert"),
            ("\\# 1", "# 1"),
        ] {
            assert_eq!(plain_anchor(anchor), plain, "{anchor}");
        }
    }

    #[test]
    fn unescape_markdown_resolves_punctuation_escapes_only() {
        assert_eq!(unescape_markdown("\\*a\\_b\\\\ \\n \\é"), "*a_b\\ \\n \\é");
        assert_eq!(unescape_markdown("trailing \\"), "trailing \\");
    }
}
