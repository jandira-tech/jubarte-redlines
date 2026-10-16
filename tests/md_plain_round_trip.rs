// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The plain Markdown round trip: `jubarte convert` docx → md → docx → md
//! prints the same Markdown both times. `md_fixed_point` checks the agent
//! view (`jubarte read`); this is the Markdown a person converts to, which
//! prints fields as their text and carries no ids. Page markers are left
//! out: the layout is outside the dialect (A2, 2026-10-16).

use std::path::Path;
use std::process::Command;

const EDGES: &str = "tests/fixtures/md-roundtrip/edges";

fn convert(dir: &Path, from: &str, to: &str) {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["-q", "convert", from, "-o", to, "--force"])
        .args(to.ends_with(".md").then_some("--no-page-markers"))
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "convert {from} -o {to}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The Markdown of `name` after one pass and after two.
fn twice(name: &str) -> (String, String) {
    let dir = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(EDGES).join(name);
    std::fs::copy(source, dir.path().join("a.docx")).unwrap();
    convert(dir.path(), "a.docx", "a.md");
    convert(dir.path(), "a.md", "b.docx");
    convert(dir.path(), "b.docx", "b.md");
    let read = |f: &str| std::fs::read_to_string(dir.path().join(f)).unwrap();
    (read("a.md"), read("b.md"))
}

#[test]
fn a_cross_reference_target_survives_the_round_trip() {
    // The REF fields print as their text, so the rebuilt file has the
    // bookmark and nothing naming it; its anchor still prints.
    let (first, second) = twice("edge-anchors.docx");
    assert!(
        first.contains("<a id=\"_Ref100\"></a>Clause 4 sets the fee."),
        "{first}"
    );
    assert_eq!(first, second);
}
