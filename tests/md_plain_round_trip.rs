// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The plain Markdown round trip: `jubarte convert --commonmark` docx → md
//! → docx → md prints the same Markdown both times. `md_fixed_point` checks
//! the agent view (`jubarte read`, and `convert`'s default); this is the
//! CommonMark for readers outside the dialect, which prints fields as their
//! text and no id lines. Page markers are left out: the layout is outside
//! the dialect (A2, 2026-10-16).

use std::path::Path;
use std::process::Command;

const EDGES: &str = "tests/fixtures/md-roundtrip/edges";

fn convert(dir: &Path, from: &str, to: &str) {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["-q", "convert", from, "-o", to, "--force"])
        .args(
            to.ends_with(".md")
                .then_some(["--no-page-markers", "--commonmark"])
                .into_iter()
                .flatten(),
        )
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

#[test]
fn an_undated_author_stays_the_change_s_author() {
    // `{>>r75 Di Eng<<}`: the number says the note names who made the
    // change, so it is no comment, and the change gets no date.
    let (first, second) = twice("edge-dates.docx");
    assert!(first.contains("{++whenever++}{>>r75 Di Eng<<}"), "{first}");
    assert_eq!(first, second);
}

#[test]
fn an_empty_author_stays_empty() {
    let (first, second) = twice("edge-authorless.docx");
    assert!(
        first.contains("{++new++}{>>r5 (1970-01-01T00:00:00Z)<<}"),
        "{first}"
    );
    assert_eq!(first, second);
}

#[test]
fn a_note_without_a_number_after_a_change_keeps_its_old_meaning() {
    // Hand-written CriticMarkup carries no numbers: a bare note after a
    // change is still a comment on it, and `(date)` alone still takes the
    // default author.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.md"),
        "One {++new++}{>>why<<} and {++two++}{>>(2026-01-02T03:04:05Z)<<}.\n",
    )
    .unwrap();
    convert(dir.path(), "a.md", "b.docx");
    convert(dir.path(), "b.docx", "b.md");
    let back = std::fs::read_to_string(dir.path().join("b.md")).unwrap();
    assert!(back.contains(": why<<}"), "{back}");
    assert!(back.contains("{++two++}{>>r"), "{back}");
    assert!(
        back.contains(" Redline (2026-01-02T03:04:05Z)<<}"),
        "{back}"
    );
}

#[test]
fn a_code_block_followed_by_a_tracked_paragraph_is_a_fixed_point() {
    // The closing fence and the `{++` used to land on one line (```{++),
    // which no Markdown reads as a closing fence: the block stayed open,
    // swallowed the tracked paragraph, and the next generation wrapped one
    // more backtick around the lot (#587).
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.md"),
        "```\nsome code\n```\n\n{++real change++}\n",
    )
    .unwrap();
    convert(dir.path(), "a.md", "a.docx");
    convert(dir.path(), "a.docx", "one.md");
    convert(dir.path(), "one.md", "b.docx");
    convert(dir.path(), "b.docx", "two.md");
    let read = |f: &str| std::fs::read_to_string(dir.path().join(f)).unwrap();
    let (one, two) = (read("one.md"), read("two.md"));
    assert!(
        one.contains("```\nsome code\n```\n\n{++real change++}"),
        "{one}"
    );
    assert_eq!(one, two);
}

#[test]
fn word_converts_to_the_agent_view_unless_commonmark_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(EDGES)
        .join("edge-dates.docx");
    std::fs::copy(source, dir.path().join("a.docx")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "-q",
            "convert",
            "a.docx",
            "-o",
            "view.md",
            "--no-page-markers",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let view = std::fs::read_to_string(dir.path().join("view.md")).unwrap();
    let read = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["read", "a.docx", "--no-page-markers"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(view, String::from_utf8(read.stdout).unwrap());
    assert!(view.starts_with("---\n"), "{view}");
    assert!(view.contains("{++whenever++}{>>r75 @DE<<}"), "{view}");
    convert(dir.path(), "a.docx", "plain.md");
    let plain = std::fs::read_to_string(dir.path().join("plain.md")).unwrap();
    assert!(!plain.contains("<!-- p0 -->"), "{plain}");
    assert!(plain.contains("{++whenever++}{>>r75 Di Eng<<}"), "{plain}");
}
