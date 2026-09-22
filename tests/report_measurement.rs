// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan Step 1 / goal gate: `planning/report.md` section 16 must carry the
//! sample50 and both-set (76 and 398) figures from a real convert+score run.

use std::fs;
use std::path::PathBuf;

fn report() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("planning/report.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Section 16 only: figures elsewhere in the report must not satisfy it.
fn section_16(body: &str) -> &str {
    let start = body
        .find("\n## 16. ")
        .expect("report.md has a section 16 heading");
    let rest = &body[start + 1..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 3);
    &rest[..end]
}

/// The pipe-table row whose first cell starts with `set`.
fn set_row<'a>(section: &'a str, set: &str) -> Vec<&'a str> {
    section
        .lines()
        .find(|l| l.starts_with(&format!("| {set}")))
        .unwrap_or_else(|| panic!("section 16 has a {set:?} row"))
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

#[test]
fn report_contains_sample50_and_both_set_figures() {
    let body = report();
    let s16 = section_16(&body);
    // (set, n, baseline, now, delta, failures) in that row's own cells.
    for (set, n, base, now, delta) in [
        ("sample50", "50", "37.57", "43.63", "+6.07"),
        ("docxide-pdf 76 fixtures", "76", "13.74", "28.27", "+14.53"),
        ("neurotic 398 corpus", "398", "53.10", "56.38", "+3.28"),
    ] {
        let row = set_row(s16, set);
        assert_eq!(row[1], n, "{set} n");
        assert!(row[2].starts_with(base), "{set} baseline {base}: {row:?}");
        assert!(row[3].starts_with(now), "{set} now {now}: {row:?}");
        assert_eq!(row[4], delta, "{set} delta");
        assert_eq!(row[7], "0", "{set} convert failures");
    }
    for needle in [
        "(not `--bless`ed)",
        "Rounding: every figure is rounded on its own",
        "### 398-set rows that dropped >1.0 Jaccard (68)",
        "`source__mcdoc` 68.28→12.82 (−55.47)",
        "SmartArt",
        "hyphenation",
        "lastRenderedPageBreak",
    ] {
        assert!(s16.contains(needle), "section 16 must contain {needle:?}");
    }
}

#[test]
fn the_398_drop_list_has_as_many_stems_as_its_heading_counts() {
    let body = report();
    let s16 = section_16(&body);
    let list = s16
        .split("--compare tools/convert_baseline_398.tsv`:")
        .nth(1)
        .and_then(|rest| rest.split("\n### ").next())
        .expect("398 stem list");
    let stems: Vec<&str> = list
        .split(',')
        .map(|s| s.trim().trim_end_matches('.'))
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(stems.len(), 68, "heading says 68; list has {}", stems.len());
    assert!(stems.iter().all(|s| s.starts_with("source")), "{stems:?}");
}

#[test]
fn section_16_slice_excludes_other_sections() {
    let body = "# r\n## 15. X\n76 fixtures\n## 16. Post\nfigures\n## 17. Next\nlater\n";
    assert_eq!(section_16(body), "## 16. Post\nfigures");
}
