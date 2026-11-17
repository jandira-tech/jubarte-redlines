// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn a_tie_drops_the_first_sides_word() {
    assert_eq!(
        word_lcs_indices(&["a", "b", "c"], &["b", "a", "c"]),
        [(0, 1), (2, 2)]
    );
}

#[test]
fn empty_and_disjoint_sides_match_nothing() {
    assert!(word_lcs_indices(&[], &["a"]).is_empty());
    assert!(word_lcs_indices(&["a"], &["b"]).is_empty());
}

#[test]
fn boilerplate_and_short_words_are_skipped_and_indices_map_back() {
    let a: Vec<String> = ["the", "limits", "is", "clear"].map(String::from).into();
    let b: Vec<String> = ["the", "bounds", "is", "clear"].map(String::from).into();
    assert_eq!(word_lcs_indices_eligible(&a, &b, &["clear"]), [(2, 2)]);
}

/// The table as two-dimensional `usize` rows, as before the flat `u32`
/// table: the same recurrence and the same tie-break.
fn reference(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 0..n {
        for j in 0..m {
            dp[i + 1][j + 1] = if a[i] == b[j] {
                dp[i][j] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (n, m, Vec::new());
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            out.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    out.reverse();
    out
}

#[test]
fn the_flat_table_matches_the_two_dimensional_one() {
    const WORDS: [&str; 4] = ["a", "b", "c", "d"];
    let mut seed = 0x2545_f491_u64;
    let mut next = |bound: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % bound as u64) as usize
    };
    for _ in 0..2000 {
        let a: Vec<&str> = (0..next(9)).map(|_| WORDS[next(4)]).collect();
        let b: Vec<&str> = (0..next(9)).map(|_| WORDS[next(4)]).collect();
        assert_eq!(word_lcs_indices(&a, &b), reference(&a, &b), "{a:?} {b:?}");
    }
}
