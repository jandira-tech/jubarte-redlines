// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Performance probes at MAX_PAIRINGS = 4_000_000 boundary.
//! Scratch file - deleted at end of review.

use jubarte::markdown::{TrackChanges, diff_markdown, resolve_critic};
use std::time::Instant;

#[test]
fn probe_at_exactly_4_000_000_bound() {
    // k = m = 2000: k*m = 4_000_000 exactly.
    let n = 2000usize;
    let old: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma\n"))
        .collect();
    let new: String = (0..n)
        .map(|i| format!("row {i} alpha beta delta\n"))
        .collect();
    let start = Instant::now();
    let diff = diff_markdown(&old, &new);
    let elapsed = start.elapsed();
    eprintln!("k=m=2000 diff took {elapsed:?}, len = {}", diff.len());
    // Accept should round-trip.
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    assert_eq!(acc, new);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(rej, old);
}

#[test]
fn probe_just_over_bound_falls_back() {
    // k = 2001, m = 2000: k*m = 4_002_000 > 4_000_000.
    let n = 2000usize;
    let old: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma\n"))
        .collect();
    let mut new = old.clone();
    new.push_str("row 2000 alpha beta gamma\n");
    let start = Instant::now();
    let diff = diff_markdown(&old, &new);
    let elapsed = start.elapsed();
    eprintln!("k=2001, m=2000 diff took {elapsed:?}, len = {}", diff.len());
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    assert_eq!(acc, new);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(rej, old);
}

#[test]
fn probe_reflow_quadratic_check() {
    // Many lines, all but identical content -> reflow kicks in.
    let base = "alpha bravo charlie delta echo foxtrot golf hotel india juliet";
    let added = base.replacen("alpha", "alpha X", 1);
    let wrap = |text: &str, width: usize| -> String {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines.push(String::new());
        lines.join("\n")
    };
    for &width in &[20usize, 24, 40] {
        for copies in [1usize, 5, 20, 100] {
            let old_paras = std::iter::repeat(wrap(base, width))
                .take(copies)
                .collect::<Vec<_>>()
                .join("\n\n");
            let new_paras = std::iter::repeat(wrap(&added, width))
                .take(copies)
                .collect::<Vec<_>>()
                .join("\n\n");
            let start = Instant::now();
            let diff = diff_markdown(&old_paras, &new_paras);
            let elapsed = start.elapsed();
            eprintln!(
                "width={width} copies={copies} ({} lines) -> {elapsed:?}",
                old_paras.matches('\n').count()
            );
            let acc = resolve_critic(&diff, TrackChanges::Accept);
            assert_eq!(acc, new_paras, "width={width} copies={copies}");
        }
    }
}