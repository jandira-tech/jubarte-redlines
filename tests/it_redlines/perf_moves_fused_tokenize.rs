// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! MOVES-FUSE-01 — the fused word-count + tokenize pass must be
//! byte-for-byte equivalent to calling `count_words` and `tokenize`
//! separately.
//!
//! `precompute_block` used to walk `collapsed` twice — once through
//! `count_words`, once through `tokenize` — and each walk ran
//! `split_by_chars`, which allocates an intermediate `Vec<String>` plus one
//! `String` per segment. `count_words_and_tokenize` fuses both into a single
//! pass with no intermediate `Vec`.
//!
//! The two reference functions are faithful ports of PowerTools' `CountWords`
//! and `TokenizeForComparison`, so the fused version is only ever allowed to
//! be an optimization of them — never a reinterpretation. This is the
//! differential test that pins that down: any drift in separator handling,
//! empty-segment filtering, dedup, or case folding shows up here.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::moves::{count_words, count_words_and_tokenize, tokenize};

/// Inputs chosen to exercise every branch of `split_by_chars` + the two
/// filters layered on top of it.
fn corpus() -> Vec<&'static str> {
    vec![
        // Degenerate.
        "",
        " ",
        "-",
        "   ",
        "---",
        // Leading / trailing / adjacent separators — `split_by_chars` keeps
        // empty segments, and both consumers filter them out.
        "alpha",
        " alpha",
        "alpha ",
        " alpha ",
        "alpha  beta",
        "alpha--beta",
        "-alpha-beta-",
        // Dedup: `tokenize` collapses repeats, `count_words` does NOT.
        "alpha alpha alpha",
        "alpha beta alpha",
        // Every separator in the default set, including the CJK ones.
        "a-b)c(d;e,f（g）h，i、j；k。l：m的n",
        // Case folding — only applies when `case_insensitive`.
        "Alpha ALPHA alpha AlPhA",
        // Multi-byte / expanding uppercase: `ß`.to_uppercase() is "SS", so the
        // folded token is longer than its source. Also a dotted-i and an
        // already-uppercase Greek sigma.
        "straße STRASSE Straße",
        "İstanbul istanbul",
        "ΣΊΣΥΦΟΣ σίσυφος",
        // Mixed scripts around a CJK separator char.
        "中文的英文",
        // Punctuation that is NOT a separator stays inside the token.
        "e.g. i.e. foo/bar baz_qux",
    ]
}

fn settings_with(case_insensitive: bool) -> WmlComparerSettings {
    WmlComparerSettings {
        case_insensitive,
        ..WmlComparerSettings::default()
    }
}

#[test]
fn fused_matches_separate_count_words_and_tokenize() {
    for case_insensitive in [false, true] {
        let settings = settings_with(case_insensitive);
        for text in corpus() {
            let want_words = count_words(text, &settings);
            let want_tokens = tokenize(text, &settings);

            let (got_words, got_tokens) = count_words_and_tokenize(text, &settings);

            assert_eq!(
                got_words, want_words,
                "word count drifted for {text:?} (case_insensitive={case_insensitive})"
            );
            assert_eq!(
                got_tokens, want_tokens,
                "token set drifted for {text:?} (case_insensitive={case_insensitive})"
            );
        }
    }
}

/// The separator list is configurable, so the fusion must read it from
/// `settings` rather than hardcoding whitespace.
#[test]
fn fused_honours_custom_separators() {
    let settings = WmlComparerSettings {
        word_separators: vec!['|', '@'],
        case_insensitive: false,
        ..WmlComparerSettings::default()
    };

    for text in ["a|b@c", "|||", "a b|c", " spaced | out ", ""] {
        let (got_words, got_tokens) = count_words_and_tokenize(text, &settings);
        assert_eq!(
            got_words,
            count_words(text, &settings),
            "word count drifted for {text:?} under custom separators"
        );
        assert_eq!(
            got_tokens,
            tokenize(text, &settings),
            "token set drifted for {text:?} under custom separators"
        );
    }
}

/// Guards the specific asymmetry that makes the fusion non-trivial: a word
/// repeated N times contributes N to the count but 1 to the token set.
#[test]
fn fused_preserves_count_dedup_asymmetry() {
    let settings = settings_with(false);
    let (words, tokens) = count_words_and_tokenize("x x x x x", &settings);
    assert_eq!(words, 5, "count_words must NOT dedup");
    assert_eq!(tokens.len(), 1, "tokenize MUST dedup");
}
