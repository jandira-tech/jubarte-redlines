// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Time scaling for pair() at the 4M boundary.
//! Scratch file - deleted at end of review.

use jubarte::markdown::diff_markdown;
use std::time::Instant;

fn make_input(n: usize) -> (String, String) {
    let old: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma delta epsilon zeta eta theta iota kappa\n"))
        .collect();
    let new: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma delta epsilon zeta eta theta iota LAMBDA\n"))
        .collect();
    (old, new)
}

#[test]
fn scaling_test() {
    for &n in &[100usize, 200, 500, 1000, 1500, 2000] {
        let (old, new) = make_input(n);
        let start = Instant::now();
        let _diff = diff_markdown(&old, &new);
        let elapsed = start.elapsed();
        eprintln!("n={n} (k*m={}) -> {elapsed:?}", n * n);
    }
}

#[test]
fn long_line_pairing() {
    // One very long line vs another very long line - tests LCS allocation path.
    let big_line_a = "alpha ".repeat(40) + "Z\n";
    let big_line_b = "alpha ".repeat(40) + "Q\n";
    let start = Instant::now();
    let _diff = diff_markdown(&big_line_a, &big_line_b);
    eprintln!("long-line pair: {:?}", start.elapsed());
}