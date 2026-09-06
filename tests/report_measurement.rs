// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan Step 1 / goal gate: `planning/report.md` must carry the sample50 and
//! both-set (76 and 398) figures from a real convert+score run.

use std::fs;
use std::path::PathBuf;

#[test]
fn report_contains_sample50_and_both_set_figures() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("planning/report.md");
    let body = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    for needle in [
        "sample50",
        "76 fixtures",
        "398 corpus",
        "43.63",
        "28.27",
        "56.38",
        "lastRenderedPageBreak",
        "SmartArt",
        "hyphenation",
    ] {
        assert!(
            body.contains(needle),
            "planning/report.md must contain {needle:?} (sample50 / both-set / parked)"
        );
    }
}
