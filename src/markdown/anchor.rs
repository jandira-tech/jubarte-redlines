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

/// The text with its emphasis marks dropped: `**`, `__`, `~~`, `==`, `<u>`
/// and `</u>` always; single `*` and word-edge `_` only when they pair up
/// (`*x*`, `_x_`), so `2 * 3` and `snake_case` keep theirs.
fn without_emphasis(text: &str) -> String {
    let mut text = text.to_string();
    for mark in ["**", "__", "~~", "==", "<u>", "</u>"] {
        text = text.replace(mark, "");
    }
    if text.matches('*').count().is_multiple_of(2) {
        text = text.replace('*', "");
    }
    let chars: Vec<char> = text.chars().collect();
    let inner = |i: usize| {
        i > 0
            && chars[i - 1].is_alphanumeric()
            && chars.get(i + 1).is_some_and(|c| c.is_alphanumeric())
    };
    let edges: Vec<usize> = (0..chars.len())
        .filter(|&i| chars[i] == '_' && !inner(i))
        .collect();
    if edges.len().is_multiple_of(2) {
        text = chars
            .iter()
            .enumerate()
            .filter(|(i, _)| !edges.contains(i))
            .map(|(_, c)| c)
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
            ("\\\\*", "\\*"),
            ("snake_case and _x_", "snake_case and x"),
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
