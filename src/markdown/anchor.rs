// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Two helpers the agent-view tooling uses to map between the
//! CriticMarkup-flavoured Markdown [`markdown::read`] prints and the
//! plain text a paragraph of the source document carries:
//!
//! - [`plain_anchor`] drops CriticMarkup marks, block markers and
//!   emphasis delimiters, resolving backslash escapes, so an id line
//!   `<!-- p3 -->` can be matched against the document's body text.
//! - [`unescape_markdown`] reverses CommonMark backslash escapes for
//!   the punctuation set both helpers share.
//!
//! They are the building blocks of the agent view's text-matching
//! passes: comparing id lines across revisions, scoring fuzzy anchors
//! for an editor's `--anchor` flag, and ranking repeated paragraphs by
//! similarity. They back the `jubarte read` path
//! ([`crate::markdown::read`]) and the bindings' text-projection
//! utilities.

/// The ASCII punctuation a backslash escapes in CommonMark.
const ESCAPABLE: &str = r"\`*_{}[]()#+-.!|<>~=&";

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

/// Whether `&` (just read) starts `&#32;`, the agent view's leading space;
/// the reference is consumed when it does.
fn space_reference(rest: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let found = rest.clone().take(4).eq("#32;".chars());
    if found {
        rest.nth(3);
    }
    found
}

/// Removes a backslash escape before a Markdown punctuation character
/// (`\*` → `*`), as CommonMark reads it; backslashes that do not precede
/// one of those characters are kept verbatim.
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
        // A line copied from the agent view: its leading space.
        if c == '&' && space_reference(&mut chars) {
            out.push(' ');
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
/// bullet (`- `, `+ `, `* `) or an ordered marker (`1. `, `2) `), and
/// without the spaces before it: the view trims a line's own edges, so a
/// leading space is its indent (a list item's continuation line).
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
    trimmed
}

/// Whitespace, or a `&#32;` [`plain_anchor`] hid.
fn blank(c: char) -> bool {
    c.is_whitespace() || c == hide(' ')
}

/// The text with its emphasis marks dropped: `**`, `~~`, `==`, `<u>`,
/// `<mark>` (with or without the `class="hl-…"` colour the view prints
/// for a highlight that is not yellow), `<s>` and their closers always; single `*` and `_` runs only when they pair up (`*x*`,
/// `_x_`, `__x__`), so `2 * 3`, `snake_case`, `snake__case` and
/// `foo__bar__` keep theirs.
fn without_emphasis(text: &str) -> String {
    let mut text = text.to_string();
    for mark in [
        "**", "~~", "==", "<u>", "</u>", "<mark>", "</mark>", "<s>", "</s>",
    ] {
        text = text.replace(mark, "");
    }
    // `<mark class="hl-cyan">`: the open tag the view writes for a
    // highlight that is not yellow.
    while let Some(at) = text.find("<mark class=\"") {
        let end = text[at..]
            .find("\">")
            .map_or(text.len(), |found| at + found + 2);
        text.replace_range(at..end, "");
    }
    // A `*` with spaces on both sides (`2 * 3`) is text; the rest are marks
    // when they pair up.
    let chars: Vec<char> = text.chars().collect();
    let spaced =
        |i: usize| i > 0 && blank(chars[i - 1]) && chars.get(i + 1).is_some_and(|&c| blank(c));
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
        let closes = before.is_some_and(|c| !blank(c)) && !after.is_some_and(char::is_alphanumeric);
        let opens = after.is_some_and(|c| !blank(c)) && !before.is_some_and(char::is_alphanumeric);
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

/// The document text an agent-view fragment stands for:
///
/// - attribution notes and comment text dropped (`{>>r0 @AC<<}` etc.);
/// - insertions unwrapped (`{++x++}` → `x`) and comment scopes too
///   (`{==x==}`);
/// - deletions removed (`{--x--}`);
/// - substitutions become their new side (`{~~old~>new~~}` → `new`);
/// - a block mark that leads the line (`#`–`######`, `> `, `- `, `+ `,
///   `* `, `1. `, `2) `, after any leading spaces) dropped;
/// - emphasis marks (`**`, `~~`, `==`, `<u>`, `<mark>` (with or without
///   its `class="hl-…"` colour), `<s>`, paired `*`/`_` runs) dropped;
/// - backslash escapes (`\*`, `\[`, ...) resolved.
///
/// A mark inside a change mark (`{++1. one++}`) is the change's own
/// text and stays, as does an escaped mark (`\*\*`). Reads the way
/// pulldown-cmark reads the same source.
pub fn plain_anchor(text: &str) -> String {
    let mut hidden = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        // The view's hard break: `\` before a newline is the newline, a
        // character of the paragraph rather than the end of a block.
        if c == '\\' && chars.peek() == Some(&'\n') {
            hidden.push(hide('\n'));
            chars.next();
            // A list item's next line is indented under its text; the
            // view trims the document's own line edges, so the spaces are
            // the view's.
            while chars.peek() == Some(&' ') {
                chars.next();
            }
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
        // Hidden, so no block mark or edge trim takes it for the view's.
        if c == '&' && space_reference(&mut chars) {
            hidden.push(hide(' '));
            continue;
        }
        hidden.push(c);
    }
    // The agent view writes a block mark at the start of its line, ahead
    // of the line's change marks, and escapes a literal one there, so marks
    // go first, line by line: a `1. ` an insertion wraps is the document's
    // text, and one after an opener left on the line above is a list's.
    // (The plain critic view escapes nothing: a literal `1. ` at the start
    // of a line copied from it is read as a mark.)
    let blocks: Vec<&str> = hidden.split('\n').map(without_block_mark).collect();
    let mut text = strip_between(&blocks.join("\n"), "{>>", "<<}", Keep::Nothing);
    text = strip_between(&text, "{--", "--}", Keep::Nothing);
    text = strip_between(&text, "{~~", "~~}", Keep::NewSide);
    text = strip_between(&text, "{++", "++}", Keep::Inner);
    text = strip_between(&text, "{==", "==}", Keep::Inner);
    // A newline left at either end separates this paragraph from the one
    // an opener or closer came from; it is not the paragraph's text.
    let text = without_emphasis(text.trim_matches('\n'));
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
            ("<mark>secret</mark>", "secret"),
            // A highlight that is not yellow opens with its colour
            // (any case the writer reads back).
            ("<mark class=\"hl-cyan\">secret</mark>", "secret"),
            ("<mark class=\"hl-Cyan\">secret</mark> tail", "secret tail"),
            ("A <mark class=\"hl-cyan\">-</mark> B", "A - B"),
            ("<s>secret</s>", "secret"),
            ("{==keep it secret.==}{>>c5 @AC: Cap?<<}", "keep it secret."),
            ("keep {++it++}{>>r0 @AC<<} secret", "keep it secret"),
            ("keep {~~that~>it~~}{>>#0 @AC<<} secret", "keep it secret"),
            ("keep it {--very --}{>>r1 @AC<<}secret", "keep it secret"),
            ("\\*\\*stars\\*\\*", "**stars**"),
            ("1. one", "one"),
            // A block mark leads the line, outside the change marks: typed
            // `1. ` an insertion wraps is text (bench 1aff6b41d6).
            ("{++1. one++}{>>r0 @AC<<}", "1. one"),
            ("1. {++one++}{>>r0 @AC<<}", "one"),
            ("{++\n\n1. item two++}{>>r0 @AC<<}", "item two"),
            ("one{++\n\n## Two++}{>>r0 @AC<<}", "one\n\nTwo"),
            ("a\\\n1\\. b", "a\n1. b"),
            ("a\\\n", "a\n"),
            ("# {~~Old~>New~~}{>>#1 @AC<<}", "New"),
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
            // The agent view writes a paragraph's leading space as `&#32;`.
            ("alpha\\\n&#32;beta", "alpha\n beta"),
            ("&#32;lead", " lead"),
            ("&#32;- item", " - item"),
            ("a &#32;b", "a  b"),
            ("&#32", "&#32"),
            // CodeRabbit #484: a `&#32;` is a space to the emphasis rules too.
            ("2&#32;*&#32;3&#32;*&#32;4", "2 * 3 * 4"),
            ("a&#32;_b_&#32;c", "a b c"),
            // The view escapes a literal reference the document holds.
            (
                "Code \\&#32; and \\&amp; literal.",
                "Code &#32; and &amp; literal.",
            ),
        ] {
            assert_eq!(plain_anchor(anchor), plain, "{anchor}");
        }
    }

    /// The text pulldown-cmark, the crate's Markdown reader, keeps of `s`.
    fn reader_text(s: &str) -> String {
        pulldown_cmark::Parser::new(s)
            .filter_map(|event| match event {
                pulldown_cmark::Event::Text(text) => Some(text.into_string()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn underscore_emphasis_reads_as_the_markdown_reader_reads_it() {
        // pi review of #394: the inputs it traced against CommonMark. In
        // `_x(__(y)__)__` the last run closes on the first, since CommonMark's
        // openers_bottom also keys on whether the closer can open (markdown-it
        // agrees: `<em>x(<strong>(y)</strong>)</em>_`).
        for s in [
            "_x(__(y)__)__",
            "_foo__",
            "__foo_",
            "___foo___",
            "_foo___",
            "__a_ b__",
            "___foo___bar___",
            "_foo___bar___baz_",
            "_a _b_ c_",
            "_(__foo__)_",
            "-_ x__-",
            "-___-___-",
            "___foo__",
            "__foo___",
            "foo_bar_baz_",
            "snake__case",
            "__init__",
            "foo-_(bar)_",
            "foo__bar__",
            "__x__y",
            "a _b_ c",
        ] {
            assert_eq!(plain_anchor(s), reader_text(s), "{s}");
        }
    }

    #[test]
    fn unescape_markdown_resolves_punctuation_escapes_only() {
        assert_eq!(unescape_markdown("\\*a\\_b\\\\ \\n \\é"), "*a_b\\ \\n \\é");
        assert_eq!(unescape_markdown("trailing \\"), "trailing \\");
        assert_eq!(unescape_markdown("&#32;lead and &#32"), " lead and &#32");
        assert_eq!(unescape_markdown("\\&#32;"), "&#32;");
    }
}
