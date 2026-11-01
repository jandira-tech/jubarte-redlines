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

// Byte comparisons performed by [`v2_candidates`] on this thread. A
// linear-time overlapping search is `O(n+m)` (a KMP prefix-function table of
// the anchor plus one pass of the text); a `starts_with` re-scan at every char
// index is `O(n*m)`. The tests use the count to reject the quadratic path. It
// is per thread, so tests running side by side under `cargo test` do not
// disturb each other's count.
#[cfg(test)]
thread_local! {
    pub(super) static READING_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static ANCHOR_BYTE_PROBES: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// Count one byte comparison. Test-only; an empty inline function in
/// production builds, so the runtime cost is zero.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[inline]
fn probe_compare() {
    ANCHOR_BYTE_PROBES.with(|probes| probes.set(probes.get() + 1));
}

#[cfg(not(test))]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[inline]
fn probe_compare() {}

/// What `c` reads as, or `None` for a character that shows nothing.
pub(super) fn reads_as(c: char) -> Option<char> {
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
    #[cfg(test)]
    READING_CALLS.with(|calls| calls.set(calls.get() + 1));
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

/// A diagnostic occurrence, never a selected range.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NearOccurrence {
    /// The document's exact characters, suitable for copying into an anchor.
    pub text: String,
    /// The exact text quoted with the same escapes as anchor refusals.
    pub shown: String,
    /// UTF-8 byte offset in the searched logical paragraph text.
    pub offset: usize,
    /// Paragraph owning the offset, when a search spans several targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Report-only whitespace matches accompanying `ANCHOR_NOT_FOUND`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NearMatches {
    /// Complete occurrence count, including overlaps and entries beyond the cap.
    pub total: usize,
    /// At most five occurrences in source order. None authorizes an edit.
    pub occurrences: Vec<NearOccurrence>,
}

impl NearMatches {
    /// One extra refusal line. Exact text remains available in the JSON report;
    /// the terminal uses visible escapes for invisible characters.
    pub fn copy_hint(&self) -> String {
        let shown: Vec<_> = self
            .occurrences
            .iter()
            .map(|hit| hit.shown.as_str())
            .collect();
        format!(
            "near: {} (whitespace or zero-width characters differ); copy the text exactly",
            shown.join("; ")
        )
    }
}

/// Collapse nonempty Unicode whitespace runs, mapping each output byte back
/// to its complete input span. Only the document skips zero-width formats;
/// they are not whitespace and never disappear from the caller's anchor.
fn whitespace_reading(text: &str, document: bool) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(text.len());
    let mut from: Vec<(usize, usize)> = Vec::with_capacity(text.len());
    let mut in_space = false;
    for (start, c) in text.char_indices() {
        if document && matches!(c, '\u{200b}'..='\u{200d}' | '\u{2060}' | '\u{feff}') {
            continue;
        }
        let end = start + c.len_utf8();
        if c.is_whitespace() {
            if in_space {
                from.last_mut()
                    .expect("a whitespace run has a mapped byte")
                    .1 = end;
            } else {
                out.push(' ');
                from.push((start, end));
            }
            in_space = true;
        } else {
            out.push(c);
            from.extend(std::iter::repeat_n((start, end), c.len_utf8()));
            in_space = false;
        }
    }
    (out, from)
}

/// Explain a literal miss without selecting or editing anything. One or more
/// `char::is_whitespace` characters match one or more on the other side. All
/// other characters remain distinct, apart from document-only skipped formats.
/// Offsets map back to original UTF-8 bytes, never the collapsed text.
pub(super) fn near_matches(text: &str, anchor: &str) -> Option<NearMatches> {
    let (needle, _) = whitespace_reading(anchor, false);
    if needle.is_empty() {
        return None;
    }
    let (haystack, from) = whitespace_reading(text, true);
    let hits = v2_candidates(&haystack, &needle);
    if hits.is_empty() {
        return None;
    }
    let occurrences = hits
        .iter()
        .take(5)
        .map(|&(start, end)| {
            let offset = from[start].0;
            let exact = &text[offset..from[end - 1].1];
            NearOccurrence {
                text: exact.to_string(),
                shown: shown(exact),
                offset,
                target: None,
            }
        })
        .collect();
    Some(NearMatches {
        total: hits.len(),
        occurrences,
    })
}

fn missing(haystack: &str, anchor: &str) -> EditError {
    let mut error = err(
        "ANCHOR_NOT_FOUND",
        None,
        format!("{} occurs 0 times in the text", shown(anchor)),
    );
    error.near = near_matches(haystack, anchor).map(Box::new);
    error
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
    let needle = anchor.as_bytes();
    let text = haystack.as_bytes();
    let m = needle.len();
    // KMP prefix function: `pi[i]` is the length of the longest proper
    // prefix of `needle[..=i]` that is also a suffix of it.
    let mut pi = vec![0usize; m];
    {
        let mut k = 0usize;
        for (i, &b) in needle.iter().enumerate().skip(1) {
            while k > 0 && needle[k] != b {
                probe_compare();
                k = pi[k - 1];
            }
            if k > 0 {
                probe_compare();
            }
            if needle[k] == b {
                k += 1;
            }
            pi[i] = k;
        }
    }
    // KMP search. A valid UTF-8 anchor begins with a lead byte, so a
    // byte-level match against valid UTF-8 text always lands on a char
    // boundary; that is exactly what callers rely on.
    let mut hits = Vec::new();
    let mut k = 0usize;
    for (i, &b) in text.iter().enumerate() {
        while k > 0 {
            probe_compare();
            if needle[k] == b {
                break;
            }
            k = pi[k - 1];
        }
        if k == 0 {
            probe_compare();
            if needle[0] == b {
                k = 1;
            }
        } else {
            k += 1;
        }
        if k == m {
            hits.push((i + 1 - m, i + 1));
            k = pi[k - 1];
        }
    }
    hits
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
    match (candidates.as_slice(), occurrence) {
        ([], _) => Err(missing(haystack, anchor)),
        ([one], None) => Ok(*one),
        (many, None) => {
            let shown = shown(anchor);
            let n = many.len();
            Err(err(
                "AMBIGUOUS_ANCHOR",
                None,
                format!("{shown} occurs {n} times in the text; set \"occurrence\" to 1..={n}"),
            ))
        }
        (many, Some(k)) if k <= many.len() => Ok(many[k - 1]),
        (many, Some(k)) => {
            let shown = shown(anchor);
            let n = many.len();
            Err(err(
                "AMBIGUOUS_ANCHOR",
                None,
                format!(
                    "{shown} occurs {n} times in the text; occurrence {k} is outside occurrence 1..={n}"
                ),
            ))
        }
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
    let n = candidates.len();
    if candidates.is_empty() {
        return Err(missing(haystack, anchor));
    }
    if n != expected {
        let shown = shown(anchor);
        return Err(err(
            "AMBIGUOUS_ANCHOR",
            None,
            format!("{shown} occurs {n} times in the text, not the expected {expected}"),
        ));
    }
    // Equal-length ranges are sorted by start, so any overlap appears
    // between neighbours. Equality at the boundary is not overlap.
    if candidates.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        let shown = shown(anchor);
        return Err(err(
            "OVERLAPPING_EDITS",
            None,
            format!(
                "{shown} occurs {n} times and those ranges overlap, so none of them is replaced"
            ),
        ));
    }
    Ok(candidates)
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
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

    /// The quadratic reference: `starts_with` re-scanned at every char
    /// index. Kept private so the equivalence test can prove the new
    /// linear search returns identical byte ranges on every fixture.
    fn naive_v2_candidates(haystack: &str, anchor: &str) -> Vec<(usize, usize)> {
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

    #[test]
    fn v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus() {
        // Covers: overlaps, no overlap, multibyte, astral, touching
        // ranges, empty, single char, anchor longer than text, multiple
        // disjoint matches, anchor at the start / middle / end.
        let corpus: &[(&str, &str)] = &[
            // Overlaps.
            ("aaa", "aa"),
            ("aaaa", "aa"),
            ("ababab", "abab"),
            ("abcabcabc", "abcabc"),
            // No overlap.
            ("abc", "b"),
            ("", "a"),
            ("a", "aa"),
            ("abc", ""),
            ("", ""),
            // Single-char matches, disjoint.
            ("a-a-a", "a"),
            ("xxxx", "x"),
            // Multibyte anchors / haystacks.
            ("ééé", "éé"),
            ("ééé", "é"),
            ("café", "fé"),
            // Astral (4-byte UTF-8) anchors and haystacks.
            ("𐀀𐀀𐀀", "𐀀𐀀"),
            ("𐀀𐀀𐀀", "𐀀"),
            // Touching ranges (disjoint, not overlap).
            ("éé", "é"),
            ("abcd", "ab"),
            ("abcdef", "cd"),
            // Anchor at start / middle / end.
            ("abcabc", "abc"),
            ("xyzabcxyz", "abc"),
            ("abc", "abc"),
            // Mixed multibyte + ASCII.
            ("aébéc", "éb"),
            // Repeated anchor with one-off shift.
            ("abababab", "abab"),
            // Long with no match.
            ("the quick brown fox", "cat"),
        ];
        for &(text, anchor) in corpus {
            assert_eq!(
                v2_candidates(text, anchor),
                naive_v2_candidates(text, anchor),
                "text={text:?}, anchor={anchor:?}"
            );
        }
    }

    #[test]
    fn v2_overlapping_search_probes_in_linear_time() {
        // Pathological input: 200k of `a` against a 100k `a` anchor. The
        // naive scan re-runs `starts_with` at every char index, so it
        // performs ~2·10¹⁰ byte comparisons before deciding AMBIGUOUS.
        // The linear search does at most `c·(n+m)` probes for a small
        // constant `c`; the assertion would time out long before failing
        // on the naive scan, so it gates the asymptotic improvement.
        let n: usize = 200_000;
        let m: usize = 100_000;
        let haystack = "a".repeat(n);
        let anchor = "a".repeat(m);
        let before = ANCHOR_BYTE_PROBES.with(std::cell::Cell::get);
        let hits = v2_candidates(&haystack, &anchor);
        let probes = ANCHOR_BYTE_PROBES.with(std::cell::Cell::get) - before;
        // Sanity: the result is the well-known all-overlapping match.
        assert_eq!(hits.len(), n - m + 1);
        assert_eq!(hits[0], (0, m));
        assert_eq!(hits[hits.len() - 1], (n - m, n));
        // Linear-time bound: a small constant times (n + m). The naive
        // scan is Θ(n·m), which dwarfs any linear bound by ~5 orders.
        const C: usize = 4;
        assert!(
            probes <= C * (n + m),
            "linear search used {probes} probes for n={n}, m={m}; \
             expected at most {bound}",
            bound = C * (n + m)
        );
    }

    #[test]
    fn v2_overlap_message_states_the_condition_without_naming_a_flag() {
        // The library error names the overlap condition itself, not a
        // CLI flag. PlanV2 has no `all_matches` field; the message must
        // remain intelligible for Python, WASM, and JSON callers.
        let e = v2_select_all("aaa", "aa", 2).unwrap_err();
        assert_eq!(e.code, "OVERLAPPING_EDITS");
        assert!(
            e.message.contains("occurs 2 times")
                && e.message.contains("those ranges overlap")
                && e.message.contains("so none of them is replaced"),
            "{}",
            e.message
        );
        assert!(
            !e.message.contains("all-matches"),
            "message must not name the CLI flag: {}",
            e.message
        );
    }
}
