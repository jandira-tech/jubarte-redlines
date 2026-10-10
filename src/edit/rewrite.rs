// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The smallest text edits that make a paragraph read as new text, for the
//! `rewrite` operation.

use similar::{Algorithm, DiffTag, capture_diff_slices};

use crate::util::word_tokens;

/// A character that holds a place in the projection but is no run text: a
/// symbol (`w:sym`, U+FFFC). It never takes part in an edit.
fn kept(c: char) -> bool {
    c == '\u{FFFC}'
}

/// A comparison unit of the old text: its key and its bytes.
struct Unit {
    key: String,
    start: usize,
    end: usize,
}

/// The old text in units: words and marks as they are, any run of
/// whitespace as one space (a tab or a line break equals a space), symbols
/// left out.
fn units(text: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    let mut at = 0;
    for token in word_tokens(text) {
        let (start, end) = (at, at + token.len());
        at = end;
        if token.chars().all(kept) {
            continue;
        }
        if token.chars().all(char::is_whitespace) {
            // Whitespace on both sides of a left-out symbol is one unit.
            if let Some(last) = out.last_mut()
                && last.key == " "
            {
                last.end = end;
                continue;
            }
            out.push(Unit {
                key: " ".to_string(),
                start,
                end,
            });
            continue;
        }
        out.push(Unit {
            key: token.to_string(),
            start,
            end,
        });
    }
    out
}

/// The tabs and line breaks of some whitespace, in order.
fn breaks(whitespace: &str) -> impl Iterator<Item = char> + '_ {
    whitespace.chars().filter(|c| matches!(c, '\t' | '\n'))
}

/// Byte ranges of `old` and their replacements that make it read as `new`,
/// in order and without overlaps, word by word. A tab or a line break of
/// `old` may be written as a space in `new` and stays; a `\t` or `\n` that
/// `new` writes where `old` has none is written. Symbols (U+FFFC) may be
/// left out of `new`: they stay where they are, and no edit covers one.
/// Words an edit replaces take the tabs and breaks between them along.
pub(super) fn rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)> {
    let olds = units(old);
    let news = units(new);
    let old_keys: Vec<&str> = olds.iter().map(|u| u.key.as_str()).collect();
    let new_keys: Vec<&str> = news.iter().map(|u| u.key.as_str()).collect();
    let mut edits = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &old_keys, &new_keys) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if tag == DiffTag::Equal {
            for (o, n) in olds[old_range].iter().zip(&news[new_range]) {
                let wanted = &new[n.start..n.end];
                if o.key == " "
                    && breaks(wanted).next().is_some()
                    && !breaks(wanted).eq(breaks(&old[o.start..o.end]))
                {
                    edits.extend(split_around_kept(old, o.start, o.end, wanted.to_string()));
                }
            }
            continue;
        }
        let replacement: String = news[new_range]
            .iter()
            .map(|u| &new[u.start..u.end])
            .collect::<String>()
            .replace('\u{FFFC}', "");
        let (start, end) = match (olds.get(old_range.start), old_range.is_empty()) {
            (Some(first), false) => (first.start, olds[old_range.end - 1].end),
            _ => {
                // An insertion: after the unit before it, or at the start.
                let at = old_range
                    .start
                    .checked_sub(1)
                    .and_then(|i| olds.get(i))
                    .map_or(0, |u| u.end);
                (at, at)
            }
        };
        edits.extend(split_around_kept(old, start, end, replacement));
    }
    edits
}

/// `[start, end)` of `old` with `replacement`, split so that no piece covers
/// a symbol: the replacement goes to the first piece, the other pieces are
/// deleted.
fn split_around_kept(
    old: &str,
    start: usize,
    end: usize,
    replacement: String,
) -> Vec<(usize, usize, String)> {
    let mut pieces: Vec<(usize, usize)> = Vec::new();
    let mut piece_start = start;
    for (offset, c) in old[start..end].char_indices() {
        if kept(c) {
            let at = start + offset;
            if piece_start < at {
                pieces.push((piece_start, at));
            }
            piece_start = at + c.len_utf8();
        }
    }
    if piece_start < end || pieces.is_empty() {
        pieces.push((piece_start.min(end), end));
    }
    let mut replacement = Some(replacement);
    pieces
        .into_iter()
        .filter_map(|(s, e)| {
            let text = replacement.take().unwrap_or_default();
            (s < e || !text.is_empty()).then_some((s, e, text))
        })
        .collect()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn apply(old: &str, edits: &[(usize, usize, String)]) -> String {
        let mut out = old.to_string();
        for (start, end, text) in edits.iter().rev() {
            out.replace_range(*start..*end, text);
        }
        out
    }

    #[test]
    fn one_word_is_one_edit() {
        let old = "The fee is ten dollars.";
        let edits = rewrite_ranges(old, "The fee is twelve dollars.");
        assert_eq!(edits, [(11, 14, "twelve".to_string())]);
        assert_eq!(apply(old, &edits), "The fee is twelve dollars.");
    }

    #[test]
    fn a_decomposed_word_is_replaced_whole_with_its_accent() {
        let old = "Le cafe\u{301} est chaud";
        let edits = rewrite_ranges(old, "Le the\u{301} est chaud");
        // One edit over "cafe" + U+0301 (4 + 2 bytes), not over "cafe" with
        // the accent left behind as a stray mark.
        assert_eq!(edits, [(3, 9, "the\u{301}".to_string())]);
        assert_eq!(apply(old, &edits), "Le the\u{301} est chaud");
        assert!(rewrite_ranges(old, old).is_empty());
    }

    #[test]
    fn insertions_at_the_start_and_end() {
        assert_eq!(rewrite_ranges("b", "a b"), [(0, 0, "a ".to_string())]);
        assert_eq!(rewrite_ranges("a b", "a b c"), [(3, 3, " c".to_string())]);
        assert_eq!(rewrite_ranges("", "x"), [(0, 0, "x".to_string())]);
    }

    #[test]
    fn unchanged_text_and_whitespace_of_another_kind_need_nothing() {
        assert!(rewrite_ranges("same text", "same text").is_empty());
        assert!(rewrite_ranges("1.\tDefinitions", "1. Definitions").is_empty());
        assert!(rewrite_ranges("a  b", "a b").is_empty());
        assert!(rewrite_ranges("line\nbreak", "line break").is_empty());
    }

    #[test]
    fn tabs_breaks_and_symbols_stay() {
        let old = "1.\tDefinitions apply";
        let edits = rewrite_ranges(old, "1. Definitions now apply");
        assert_eq!(apply(old, &edits), "1.\tDefinitions now apply");

        let old = "x \u{FFFC} y";
        assert!(rewrite_ranges(old, "x y").is_empty());
        let edits = rewrite_ranges(old, "x z");
        assert_eq!(apply(old, &edits), "x \u{FFFC} z");

        // Words deleted take the tab between them along; a symbol stays.
        let old = "a\tb c";
        assert_eq!(apply(old, &rewrite_ranges(old, "c")), "c");
        let old = "a \u{FFFC} b c";
        assert_eq!(apply(old, &rewrite_ranges(old, "c")), "\u{FFFC}c");
    }

    #[test]
    fn new_text_writes_its_tabs_and_breaks() {
        let edits = rewrite_ranges("a", "a\tb\nc");
        assert_eq!(apply("a", &edits), "a\tb\nc");
        // Whitespace the old text has as spaces becomes the tab or the
        // break asked for; a tab asked for where one is needs nothing.
        assert_eq!(rewrite_ranges("a b", "a\tb"), [(1, 2, "\t".to_string())]);
        assert_eq!(rewrite_ranges("a\tb", "a\nb"), [(1, 2, "\n".to_string())]);
        assert_eq!(
            rewrite_ranges("a \tb", "a\t\tb"),
            [(1, 3, "\t\t".to_string())]
        );
        assert!(rewrite_ranges("a\tb", "a\tb").is_empty());
        assert!(rewrite_ranges("a \t b", "a\tb").is_empty());
    }

    #[test]
    fn everything_can_go() {
        let old = "gone words";
        let edits = rewrite_ranges(old, "");
        assert_eq!(apply(old, &edits), "");
    }
}
