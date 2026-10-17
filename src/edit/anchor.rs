// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Anchors that differ from the paragraph only in characters a reader
//! cannot tell apart: a non-breaking space typed as a space, a curly quote
//! as a straight one, a non-breaking hyphen as a hyphen, a zero-width
//! character left out. Such an anchor is found, and an anchor that is not
//! found is echoed with those characters escaped.
//!
//! v2 literal anchors ([`v2_candidates`], [`v2_select`],
//! [`v2_select_all`]) match the exact logical text only: every character
//! distinct (a space is not U+00A0, a double space is not a single one),
//! with no lookalike reading and no Markdown-mark fallback. Those v1
//! fallbacks stay in [`reading_hits`] under their versioned contract.

use super::{EditError, err};

/// What `c` reads as, or `None` for a character that shows nothing.
fn reads_as(c: char) -> Option<char> {
    match c {
        '\u{A0}' | '\u{2007}' | '\u{202F}' | '\u{2000}'..='\u{200A}' => Some(' '),
        // Curled and other look-alike quotes read as the straight one. A
        // backtick-like mark (`\u{2035}`, `\u{B4}`) does not: the straight
        // quotes and the backtick are what a shell quotes with.
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' | '\u{02B9}'
        | '\u{02BB}' | '\u{02BC}' | '\u{FF07}' => Some('\''),
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}' | '\u{2036}'
        | '\u{02BA}' | '\u{02EE}' | '\u{FF02}' | '\u{301D}' | '\u{301E}' | '\u{301F}'
        | '\u{05F4}' => Some('"'),
        '\u{2010}' | '\u{2011}' => Some('-'),
        '\u{AD}' | '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FEFF}' => None,
        c => Some(c),
    }
}

/// `text` as it reads, with the byte range of `text` each byte of it
/// comes from.
fn reading(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(text.len());
    let mut from = Vec::with_capacity(text.len());
    for (i, c) in text.char_indices() {
        if let Some(r) = reads_as(c) {
            out.push(r);
            from.extend(std::iter::repeat_n((i, i + c.len_utf8()), r.len_utf8()));
        }
    }
    (out, from)
}

/// Byte ranges of `text` that read as `find`, in order (overlapping ones
/// too); none when `find` reads as nothing.
pub(super) fn reading_hits(text: &str, find: &str) -> Vec<(usize, usize)> {
    let (needle, _) = reading(find);
    if needle.is_empty() {
        return Vec::new();
    }
    let (haystack, from) = reading(text);
    haystack
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| haystack[i..].starts_with(&needle))
        .map(|i| (from[i].0, from[i + needle.len() - 1].1))
        .collect()
}

/// `text` quoted, with every character that looks like another or shows
/// nothing written as `\u{…}`, so an agent sees why its anchor missed.
pub(super) fn shown(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() || reads_as(c) != Some(c) && !c.is_alphanumeric() => {
                out.push_str(&format!("\\u{{{:x}}}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Every exact occurrence of `anchor` in `haystack`, including overlaps,
/// as UTF-8 byte ranges in start order. Both ends are character boundaries.
/// Matching preserves exact characters and whitespace, without normalization
/// or Markdown fallback. An empty anchor produces no candidates; selection
/// refuses it as `INVALID_EDIT`.
pub fn v2_candidates(haystack: &str, anchor: &str) -> Vec<(usize, usize)> {
    if anchor.is_empty() {
        return Vec::new();
    }
    haystack
        .char_indices()
        .map(|(i, _)| i)
        .filter(|&i| haystack[i..].starts_with(anchor))
        .map(|i| (i, i + anchor.len()))
        .collect()
}

/// Select the unique v2 literal candidate, or the 1-based `occurrence`
/// within the complete candidate list, including overlapping ranges.
///
/// # Errors
///
/// Reuses [`EditError`] codes: `INVALID_EDIT` for an empty anchor or occurrence
/// zero, `ANCHOR_NOT_FOUND` for no candidates, and `AMBIGUOUS_ANCHOR` for
/// multiple unqualified candidates or an out-of-range occurrence. Match
/// refusals report the complete candidate count. No operation is bound yet.
pub fn v2_select(
    haystack: &str,
    anchor: &str,
    occurrence: Option<usize>,
) -> Result<(usize, usize), EditError> {
    if anchor.is_empty() {
        return Err(err("INVALID_EDIT", None, "anchor must be nonempty"));
    }
    if occurrence == Some(0) {
        return Err(err("INVALID_EDIT", None, "occurrence is 1-based"));
    }
    let candidates = v2_candidates(haystack, anchor);
    let shown = shown(anchor);
    match (candidates.as_slice(), occurrence) {
        ([], _) => Err(err(
            "ANCHOR_NOT_FOUND",
            None,
            format!("{shown} occurs 0 times in the text"),
        )),
        ([one], None) => Ok(*one),
        (many, None) => Err(err(
            "AMBIGUOUS_ANCHOR",
            None,
            format!(
                "{shown} occurs {n} times in the text; set \"occurrence\" to 1..={n}",
                n = many.len()
            ),
        )),
        (many, Some(k)) if k <= many.len() => Ok(many[k - 1]),
        (many, Some(k)) => Err(err(
            "AMBIGUOUS_ANCHOR",
            None,
            format!(
                "{shown} occurs {n} times in the text; occurrence {k} is outside occurrence 1..={n}",
                n = many.len()
            ),
        )),
    }
}

/// Select every exact v2 candidate, requiring the complete count to equal
/// `expected` and every range to be disjoint. Touching ranges are allowed.
/// Overlapping ranges never yield a greedy subset.
///
/// # Errors
///
/// Reuses [`EditError`] codes: `INVALID_EDIT` for an empty anchor,
/// `ANCHOR_NOT_FOUND` for no candidates (even when `expected` is zero),
/// `AMBIGUOUS_ANCHOR` for a count mismatch, then `OVERLAPPING_EDITS` for
/// overlapping ranges. Match refusals report the complete candidate count.
pub fn v2_select_all(
    haystack: &str,
    anchor: &str,
    expected: usize,
) -> Result<Vec<(usize, usize)>, EditError> {
    if anchor.is_empty() {
        return Err(err("INVALID_EDIT", None, "anchor must be nonempty"));
    }
    let candidates = v2_candidates(haystack, anchor);
    let shown = shown(anchor);
    let n = candidates.len();
    if candidates.is_empty() {
        return Err(err(
            "ANCHOR_NOT_FOUND",
            None,
            format!("{shown} occurs 0 times in the text"),
        ));
    }
    if n != expected {
        return Err(err(
            "AMBIGUOUS_ANCHOR",
            None,
            format!("{shown} occurs {n} times in the text, not the expected {expected}"),
        ));
    }
    // Equal-length ranges are sorted by start, so any overlap appears
    // between neighbours. Equality at the boundary is not overlap.
    if candidates.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(err(
            "OVERLAPPING_EDITS",
            None,
            format!(
                "{shown} occurs {n} times and those ranges overlap; all-matches refuses them all"
            ),
        ));
    }
    Ok(candidates)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn text_is_found_by_what_it_reads_as() {
        let text =
            "\u{201D} shall mean (1)\u{A0}any officer\u{2019}s attorney\u{2011}in\u{2011}fact";
        let hit = |find: &str| {
            reading_hits(text, find)
                .into_iter()
                .map(|(s, e)| &text[s..e])
                .collect::<Vec<_>>()
        };
        assert_eq!(
            hit("\" shall mean (1) any officer's"),
            ["\u{201D} shall mean (1)\u{A0}any officer\u{2019}s"]
        );
        assert_eq!(hit("attorney-in-fact"), ["attorney\u{2011}in\u{2011}fact"]);
        assert!(hit("attorney in fact").is_empty());
        // A zero-width character reads as nothing, on either side.
        assert_eq!(reading_hits("ab\u{200B}c", "abc"), [(0, 6)]);
        assert_eq!(reading_hits("abc", "a\u{AD}bc"), [(0, 3)]);
        assert!(reading_hits("abc", "\u{200B}").is_empty());
        // Repeated text overlaps, as the exact search counts it.
        assert_eq!(reading_hits("aaa", "aa"), [(0, 2), (1, 3)]);
        // Every character that reads as a quote is one; a backtick-like
        // mark is not, and a straight quote finds only itself or a curled one.
        for (quote, straight) in [
            ('\u{201C}', '"'),
            ('\u{201D}', '"'),
            ('\u{201E}', '"'),
            ('\u{201F}', '"'),
            ('\u{2033}', '"'),
            ('\u{2036}', '"'),
            ('\u{02BA}', '"'),
            ('\u{02EE}', '"'),
            ('\u{FF02}', '"'),
            ('\u{301D}', '"'),
            ('\u{301E}', '"'),
            ('\u{301F}', '"'),
            ('\u{05F4}', '"'),
            ('\u{2018}', '\''),
            ('\u{2019}', '\''),
            ('\u{02B9}', '\''),
            ('\u{02BB}', '\''),
            ('\u{02BC}', '\''),
            ('\u{FF07}', '\''),
        ] {
            let text = format!("say {quote}hi{quote}");
            let find = format!("say {straight}hi{straight}");
            assert_eq!(reading_hits(&text, &find).len(), 1, "{quote:?}");
        }
        for mark in ['`', '\u{2035}', '\u{B4}'] {
            assert!(
                reading_hits(&format!("it{mark}s"), "it's").is_empty(),
                "{mark:?}"
            );
        }
        assert!(reading_hits("say \"hi\"", "say 'hi'").is_empty());
    }

    #[test]
    fn lookalikes_and_invisible_characters_are_shown_escaped() {
        assert_eq!(
            shown("(1)\u{A0}any \u{2019}x\u{2019} a\u{2011}b\u{200B}"),
            r#""(1)\u{a0}any \u{2019}x\u{2019} a\u{2011}b\u{200b}""#
        );
        assert_eq!(shown("say \"hi\"\t\\"), r#""say \"hi\"\t\\""#);
        assert_eq!(shown("café naïve"), r#""café naïve""#);
    }

    #[test]
    fn v2_counts_every_exact_start_position_including_overlaps() {
        assert_eq!(v2_candidates("aaa", "aa"), [(0, 2), (1, 3)]);
        assert_eq!(v2_candidates("a-a-a", "a"), [(0, 1), (2, 3), (4, 5)]);
        assert_eq!(v2_candidates("abc", "abc"), [(0, 3)]);
        // Exact characters only: no lookalike reading, no whitespace folding.
        assert!(v2_candidates("a\u{A0}b", "a b").is_empty());
        assert!(v2_candidates("a  b", "a b").is_empty());
        assert!(v2_candidates("\u{201C}x\u{201D}", "\"x\"").is_empty());
        // Multi-byte anchors land on character boundaries.
        assert_eq!(v2_candidates("ééé", "éé"), [(0, 4), (2, 6)]);
        // Nothing to find, and an empty anchor has no candidates.
        assert!(v2_candidates("abc", "z").is_empty());
        assert!(v2_candidates("abc", "").is_empty());
        assert!(v2_candidates("", "a").is_empty());
        assert!(v2_candidates("a", "aa").is_empty());
        assert!(v2_candidates("", "").is_empty());
    }

    #[test]
    fn v2_selects_a_unique_anchor_and_refuses_with_the_count_when_repeated() {
        assert_eq!(v2_select("a-a", "-", None).unwrap(), (1, 2));
        let e = v2_select("aaa", "aa", None).unwrap_err();
        assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
        assert_eq!(e.operation, None);
        assert!(
            e.message.contains("occurs 2 times")
                && e.message.contains("set \"occurrence\" to 1..=2"),
            "{}",
            e.message
        );
        assert_eq!(
            v2_select("abc", "z", None).unwrap_err().code,
            "ANCHOR_NOT_FOUND"
        );
    }

    #[test]
    fn v2_occurrence_picks_the_nth_candidate_and_refuses_past_the_last() {
        assert_eq!(v2_select("aaa", "aa", Some(1)).unwrap(), (0, 2));
        assert_eq!(v2_select("aaa", "aa", Some(2)).unwrap(), (1, 3));
        // A unique anchor takes occurrence 1.
        assert_eq!(v2_select("abc", "b", Some(1)).unwrap(), (1, 2));
        let e = v2_select("aaa", "aa", Some(3)).unwrap_err();
        assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
        assert!(
            e.message.contains("occurs 2 times") && e.message.contains("occurrence 3"),
            "{}",
            e.message
        );
        let e = v2_select("aaa", "aa", Some(0)).unwrap_err();
        assert_eq!(e.code, "INVALID_EDIT");
        assert!(e.message.contains("1-based"), "{}", e.message);
    }

    #[test]
    fn v2_all_matches_takes_the_complete_disjoint_candidate_set() {
        assert_eq!(
            v2_select_all("a-a-a", "a", 3).unwrap(),
            [(0, 1), (2, 3), (4, 5)]
        );
        // A wrong expected count reports the real one.
        let e = v2_select_all("a-a-a", "a", 2).unwrap_err();
        assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
        assert!(
            e.message.contains("occurs 3 times") && e.message.contains("expected 2"),
            "{}",
            e.message
        );
        // Overlapping candidates are refused together, even at the right
        // count: no disjoint subset is kept.
        let e = v2_select_all("aaa", "aa", 2).unwrap_err();
        assert_eq!(e.code, "OVERLAPPING_EDITS");
        assert!(
            e.message.contains("occurs 2 times") && e.message.contains("overlap"),
            "{}",
            e.message
        );
        assert_eq!(
            v2_select_all("abc", "z", 1).unwrap_err().code,
            "ANCHOR_NOT_FOUND"
        );
        assert_eq!(
            v2_select_all("abc", "z", 0).unwrap_err().code,
            "ANCHOR_NOT_FOUND"
        );
        // Touching ranges are disjoint, even when the anchor has several bytes.
        assert_eq!(v2_select_all("éé", "é", 2).unwrap(), [(0, 2), (2, 4)]);
        assert_eq!(
            v2_select_all("aaaa", "aa", 3).unwrap_err().code,
            "OVERLAPPING_EDITS"
        );
        // The expected count is checked against every candidate, never a subset.
        let e = v2_select_all("aaaa", "aa", 2).unwrap_err();
        assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
        assert!(e.message.contains("occurs 3 times"), "{}", e.message);
        assert_eq!(v2_select_all("abc", "b", 1).unwrap(), [(1, 2)]);
        assert_eq!(
            v2_select_all("abc", "b", 0).unwrap_err().code,
            "AMBIGUOUS_ANCHOR"
        );
    }

    #[test]
    fn v2_empty_anchor_is_an_invalid_edit() {
        for refused in [
            v2_select("abc", "", None).unwrap_err(),
            v2_select("abc", "", Some(1)).unwrap_err(),
            v2_select_all("abc", "", 1).unwrap_err(),
        ] {
            assert_eq!(refused.code, "INVALID_EDIT");
            assert!(refused.message.contains("nonempty"), "{}", refused.message);
        }
    }

    #[test]
    fn v2_literal_anchors_preserve_exact_characters() {
        for anchor in [
            "a b", "a  b", "a\u{a0}b", "a\tb", "a\nb", "**x**", "e\u{301}", "é",
        ] {
            assert_eq!(v2_select(anchor, anchor, None).unwrap(), (0, anchor.len()));
        }
        for (text, anchor) in [
            ("a\u{a0}b", "a b"),
            ("a  b", "a b"),
            ("a b", "a  b"),
            ("a b", "a\u{a0}b"),
            ("a\tb", "a b"),
            ("a\nb", "a b"),
            ("x", "**x**"),
            ("é", "e\u{301}"),
            ("e\u{301}", "é"),
            ("a\u{200b}b", "ab"),
            ("A", "a"),
        ] {
            assert_eq!(
                v2_select(text, anchor, None).unwrap_err().code,
                "ANCHOR_NOT_FOUND",
                "{text:?}, {anchor:?}"
            );
        }
    }

    #[test]
    fn v2_selection_uses_utf8_byte_ranges() {
        for (occurrence, expected, range) in [(1, "Xé", (0, 4)), (2, "éX", (2, 6))] {
            let mut text = String::from("ééé");
            let (start, end) = v2_select(&text, "éé", Some(occurrence)).unwrap();
            assert_eq!((start, end), range);
            assert!(text.is_char_boundary(start) && text.is_char_boundary(end));
            text.replace_range(start..end, "X");
            assert_eq!(text, expected);
        }
        let e = v2_select_all("ééé", "éé", 2).unwrap_err();
        assert_eq!(e.code, "OVERLAPPING_EDITS");
        let mut text = String::from("a-a-a");
        for (start, end) in v2_select_all(&text, "a", 3).unwrap().into_iter().rev() {
            text.replace_range(start..end, "XX");
        }
        assert_eq!(text, "XX-XX-XX");
    }

    #[test]
    fn v2_missing_anchors_refuse_with_zero_candidates() {
        for e in [
            v2_select("abc", "z", None).unwrap_err(),
            v2_select("abc", "z", Some(1)).unwrap_err(),
            v2_select_all("abc", "z", 0).unwrap_err(),
            v2_select_all("abc", "z", 1).unwrap_err(),
            v2_select("", "z", None).unwrap_err(),
        ] {
            assert_eq!(e.code, "ANCHOR_NOT_FOUND");
            assert!(e.message.contains("0 times"), "{}", e.message);
            assert_eq!(e.operation, None);
            assert!(e.outcomes.is_empty());
        }
    }

    #[test]
    fn v2_zero_occurrence_is_invalid_even_without_candidates() {
        for text in ["aaa", "", "bbb"] {
            assert_eq!(
                v2_select(text, "aa", Some(0)).unwrap_err().code,
                "INVALID_EDIT"
            );
        }
        let e = v2_select("abc", "b", Some(usize::MAX)).unwrap_err();
        assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
        assert!(e.message.contains("occurs 1 times"), "{}", e.message);
    }
}
