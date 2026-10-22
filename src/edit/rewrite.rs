// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The smallest text edits that make a paragraph read as new text, for the
//! `rewrite` operation, and that make a `replace`'s matched text read as its
//! replacement.

use std::ops::Range;

use similar::{Algorithm, DiffTag, capture_diff_slices};

use super::anchor::reads_as;
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

/// The old text in units, each keyed as it reads (a curly quote as the
/// straight one, a non-breaking space as a space, a zero-width character as
/// nothing), so a look-alike never differs: words and marks as they are,
/// any run of whitespace as one space when `collapse` (a tab or a line
/// break equals a space), symbols and text that reads as nothing left out.
fn units(text: &str, collapse: bool) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    let mut at = 0;
    for token in word_tokens(text) {
        let (start, end) = (at, at + token.len());
        at = end;
        let key: String = token.chars().filter_map(reads_as).collect();
        if token.chars().all(kept) || key.is_empty() {
            continue;
        }
        if collapse && token.chars().all(char::is_whitespace) {
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
        out.push(Unit { key, start, end });
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
/// Text of `old` that reads as `new`'s (a curly quote, a non-breaking
/// space) stays as it is.
pub(super) fn rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)> {
    ranges(old, new, true)
}

/// [`rewrite_ranges`] for a `replace`: whitespace compares as written, so
/// a double space or a tab the anchor names is replaced as asked, while
/// a look-alike of the replacement's character still stays.
pub(super) fn replace_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)> {
    ranges(old, new, false)
}

fn ranges(old: &str, new: &str, collapse: bool) -> Vec<(usize, usize, String)> {
    let olds = units(old, collapse);
    let news = units(new, collapse);
    let old_keys: Vec<&str> = olds.iter().map(|u| u.key.as_str()).collect();
    let new_keys: Vec<&str> = news.iter().map(|u| u.key.as_str()).collect();
    // Each op with the equal gaps a join took in: their units stay `old`'s.
    type Gaps = Vec<(Range<usize>, Range<usize>)>;
    let mut ops: Vec<(DiffTag, Range<usize>, Range<usize>, Gaps)> = Vec::new();
    for op in capture_diff_slices(Algorithm::Myers, &old_keys, &new_keys) {
        let (tag, old_range, new_range) = op.as_tag_tuple();
        if !collapse && tag != DiffTag::Equal {
            // A `replace` writes changes that only whitespace keeps apart
            // as one, as Word Compare shows them.
            let len = ops.len();
            let joins = match ops.as_slice() {
                [.., (a, ..), (DiffTag::Equal, gap, ..)] if *a != DiffTag::Equal => {
                    olds[gap.clone()].iter().all(|u| u.key.trim().is_empty())
                }
                [.., (a, ..)] => *a != DiffTag::Equal,
                [] => false,
            };
            if joins {
                let first = if ops[len - 1].0 == DiffTag::Equal {
                    len - 2
                } else {
                    len - 1
                };
                let mut drained = ops.drain(first..);
                let (_, old_start, new_start, mut gaps) = drained.next().expect("ops to join");
                gaps.extend(drained.map(|(_, o, n, _)| (o, n)));
                ops.push((
                    DiffTag::Replace,
                    old_start.start..old_range.end,
                    new_start.start..new_range.end,
                    gaps,
                ));
                continue;
            }
        }
        ops.push((tag, old_range, new_range, Vec::new()));
    }
    let mut edits = Vec::new();
    for (tag, old_range, new_range, gaps) in ops {
        if tag == DiffTag::Equal {
            for (o, n) in olds[old_range].iter().zip(&news[new_range]) {
                let wanted = &new[n.start..n.end];
                if collapse
                    && o.key == " "
                    && breaks(wanted).next().is_some()
                    && !breaks(wanted).eq(breaks(&old[o.start..o.end]))
                {
                    edits.extend(split_around_kept(old, o.start, o.end, wanted));
                }
            }
            continue;
        }
        // A gap a join took in reads as `new`'s and keeps `old`'s text.
        let replacement: String = new_range
            .clone()
            .map(|i| {
                gaps.iter().find(|(_, n)| n.contains(&i)).map_or(
                    &new[news[i].start..news[i].end],
                    |(o, n)| {
                        let u = &olds[o.start + (i - n.start)];
                        &old[u.start..u.end]
                    },
                )
            })
            .collect();
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
        edits.extend(split_around_kept(old, start, end, &replacement));
    }
    edits
}

/// `[start, end)` of `old` with `replacement`, split so that no piece covers
/// a symbol. A replacement holding as many symbols as the range puts the
/// text between them in the matching gaps; any other goes, without its
/// symbols, to the first piece, and the other pieces are deleted.
fn split_around_kept(
    old: &str,
    start: usize,
    end: usize,
    replacement: &str,
) -> Vec<(usize, usize, String)> {
    // The gaps between the kept chars of the range, empty ones too.
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    let mut gap_start = start;
    for (offset, c) in old[start..end].char_indices() {
        if kept(c) {
            let at = start + offset;
            gaps.push((gap_start, at));
            gap_start = at + c.len_utf8();
        }
    }
    gaps.push((gap_start, end));
    // The symbols themselves are never text.
    let parts: Vec<&str> = replacement.split(kept).collect();
    if parts.len() == gaps.len() {
        return gaps
            .into_iter()
            .zip(parts)
            .filter(|&((s, e), text)| old[s..e] != *text)
            .map(|((s, e), text)| (s, e, text.to_string()))
            .collect();
    }
    let replacement: String = parts.concat();
    let mut pieces: Vec<(usize, usize)> = gaps.into_iter().filter(|(s, e)| s < e).collect();
    if pieces.is_empty() {
        pieces.push((end, end));
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
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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
    fn look_alikes_read_as_the_plain_character_and_stay() {
        let old = "The \u{201C}Seller\u{201D} shall\u{A0}deliver";
        let edits = rewrite_ranges(old, "The \"Seller\" must deliver");
        assert_eq!(
            apply(old, &edits),
            "The \u{201C}Seller\u{201D} must\u{A0}deliver"
        );
        assert_eq!(edits.len(), 1, "{edits:?}");
        // A zero-width space reads as nothing: it is no word of its own.
        assert!(rewrite_ranges("a\u{200B} b", "a b").is_empty());
    }

    #[test]
    fn replace_ranges_compare_whitespace_as_written() {
        assert_eq!(replace_ranges("a  b", "a b"), [(1, 3, " ".to_string())]);
        assert_eq!(replace_ranges("a\tb", "a b"), [(1, 2, " ".to_string())]);
        assert!(replace_ranges("a\u{A0}b", "a b").is_empty());
        let old = "price is ten dollars";
        assert_eq!(
            replace_ranges(old, "price is twelve dollars"),
            [(9, 12, "twelve".to_string())]
        );
    }

    /// Changes a whitespace gap joins into one keep the gap as the
    /// document writes it: a non-breaking space the replacement types as a
    /// plain one stays (#1384 review).
    #[test]
    fn a_joined_replacement_keeps_its_gaps_look_alike() {
        for (old, new, want) in [
            ("Section\u{A0}5", "Article 6", "Article\u{A0}6"),
            ("ten\u{A0}dollars", "twelve euros", "twelve\u{A0}euros"),
            (
                "a ten\u{A0}dollars b",
                "a twelve euros b",
                "a twelve\u{A0}euros b",
            ),
        ] {
            let edits = replace_ranges(old, new);
            assert_eq!(apply(old, &edits), want, "{edits:?}");
            assert_eq!(edits.len(), 1, "{edits:?}");
        }
    }

    #[test]
    fn replace_ranges_join_changes_only_whitespace_separates() {
        // Every word changes: one edit, as Word Compare shows it, not two
        // pairs around a kept space.
        assert_eq!(
            replace_ranges("ten dollars", "twelve euros"),
            [(0, 11, "twelve euros".to_string())]
        );
        let old = "The cap is";
        let edits = replace_ranges(old, "It is");
        assert_eq!(apply(old, &edits), "It is");
        assert_eq!(edits.len(), 1, "{edits:?}");
        // A kept word between two changes still splits them.
        assert_eq!(
            replace_ranges("pay ten big dollars", "pay twelve big euros"),
            [(4, 7, "twelve".to_string()), (12, 19, "euros".to_string())]
        );
    }

    #[test]
    fn everything_can_go() {
        let old = "gone words";
        let edits = rewrite_ranges(old, "");
        assert_eq!(apply(old, &edits), "");
    }
}
