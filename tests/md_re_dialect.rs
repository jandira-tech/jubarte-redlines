// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The attribution syntax settled on 2026-10-18 (#1135), and edits inside
//! pending insertions (#1136), on Word-made files.
//!
//! Each `tests/fixtures/re-dialect/F.docx` sits beside `F.md`, the view it
//! must print: `{++text++}{r1 @ASR}`, `re rN @X` for a deletion inside an
//! insertion, `{~~old~>new~~}{rD+I @X}` for a deletion followed directly by
//! an insertion from the same author, and `¶{rN @X}` for a revised paragraph
//! mark. `edits/NAME.md` is the body after one `jubarte edit` as Carol Diaz.
//! `scripts/re-dialect-read.sh` and `scripts/re-dialect-edit.sh` run the
//! same commands and print the diffs.

use std::path::Path;
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/re-dialect");
const EDITOR: [&str; 4] = ["-a", "Carol Diaz", "-d", "2026-10-18T10:00:00Z"];

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

fn ok(out: &Output, what: &str) -> String {
    assert!(
        out.status.success(),
        "{what} failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{name}")).unwrap()
}

/// Everything after the YAML header: the header names the file and its
/// sha256, which change with every write.
fn body(view: &str) -> &str {
    let (_, rest) = view.split_once("---\n").expect("header opens");
    let (_, body) = rest.split_once("---\n").expect("header closes");
    body
}

fn copy_in(dir: &Path, name: &str) {
    std::fs::copy(format!("{FIXTURES}/{name}"), dir.join(name)).unwrap();
}

/// `jubarte F.docx > a.md`, `jubarte F.docx -o b.md` and
/// `jubarte convert F.docx -o c.md` all print `F.md`.
fn reads_as_its_view(name: &str) {
    let dir = tempfile::tempdir().unwrap();
    let docx = format!("{name}.docx");
    copy_in(dir.path(), &docx);
    let expected = fixture(&format!("{name}.md"));
    let a = ok(&jubarte(&[&docx], dir.path()), "read");
    ok(
        &jubarte(&["-q", &docx, "-o", "b.md"], dir.path()),
        "read -o",
    );
    ok(
        &jubarte(&["-q", "convert", &docx, "-o", "c.md"], dir.path()),
        "convert",
    );
    let b = std::fs::read_to_string(dir.path().join("b.md")).unwrap();
    let c = std::fs::read_to_string(dir.path().join("c.md")).unwrap();
    assert_eq!(a, expected, "jubarte {docx} > a.md");
    assert_eq!(b, expected, "jubarte {docx} -o b.md");
    assert_eq!(c, expected, "jubarte convert {docx} -o c.md");
}

/// `F.md` written to a docx and read back keeps its body.
fn round_trips(name: &str) {
    let dir = tempfile::tempdir().unwrap();
    let md = format!("{name}.md");
    copy_in(dir.path(), &md);
    ok(
        &jubarte(&["-q", "convert", &md, "-o", "back.docx"], dir.path()),
        "convert md",
    );
    let back = ok(&jubarte(&["back.docx"], dir.path()), "read back");
    let expected = fixture(&md);
    assert_eq!(body(&back), body(&expected), "{md} -> docx -> md");
}

/// One `jubarte edit` on `FIXTURE.docx` as Carol Diaz reads back as
/// `edits/NAME.md`.
fn edits_as(name: &str, fixture_name: &str, args: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    let docx = format!("{fixture_name}.docx");
    copy_in(dir.path(), &docx);
    let mut all = vec!["-q", "edit", docx.as_str()];
    all.extend_from_slice(args);
    all.extend_from_slice(&EDITOR);
    all.extend_from_slice(&["-o", "out.docx"]);
    ok(&jubarte(&all, dir.path()), &format!("edit {args:?}"));
    let view = ok(&jubarte(&["out.docx"], dir.path()), "read the edit");
    assert_eq!(
        body(&view),
        fixture(&format!("edits/{name}.md")),
        "jubarte edit {docx} {args:?}"
    );
}

#[test]
fn step0_reads_as_its_view() {
    reads_as_its_view("step0");
}

#[test]
fn step1_reads_as_its_view() {
    reads_as_its_view("step1");
}

#[test]
fn step2_tracked_reads_as_its_view() {
    reads_as_its_view("step2-tracked");
}

#[test]
fn step2_untracked_reads_as_its_view() {
    reads_as_its_view("step2-untracked");
}

#[test]
fn annex_reads_as_its_view() {
    reads_as_its_view("annex");
}

#[test]
fn step0_round_trips() {
    round_trips("step0");
}

#[test]
fn step1_round_trips() {
    round_trips("step1");
}

#[test]
fn step2_tracked_round_trips() {
    round_trips("step2-tracked");
}

#[test]
fn step2_untracked_round_trips() {
    round_trips("step2-untracked");
}

#[test]
fn annex_round_trips() {
    round_trips("annex");
}

#[test]
fn edit_modifies_inside_another_authors_insertion() {
    edits_as(
        "modify-inside-insertion",
        "step2-tracked",
        &["-p", "p0", "--anchor", "their", "--content", "its"],
    );
}

#[test]
fn edit_adds_inside_another_authors_insertion() {
    edits_as(
        "add-inside-insertion",
        "step2-tracked",
        &[
            "-p",
            "p0",
            "--anchor",
            "month.",
            "--content",
            "month, in cash.",
        ],
    );
}

#[test]
fn edit_modifies_the_new_side_of_a_substitution() {
    edits_as(
        "modify-substitution",
        "step2-tracked",
        &[
            "-p",
            "p0",
            "--anchor",
            "insurers",
            "--content",
            "underwriters",
        ],
    );
}

#[test]
fn edit_rejects_a_substitution_inside_an_insertion() {
    edits_as(
        "reject-substitution",
        "step2-tracked",
        &["-p", "r4+5", "--reject"],
    );
}

#[test]
fn edit_accepts_a_substitution_inside_an_insertion() {
    edits_as(
        "accept-substitution",
        "step2-tracked",
        &["-p", "r4+5", "--accept"],
    );
}

#[test]
fn edit_modifies_untracked_text() {
    edits_as(
        "modify-plain-text",
        "step2-untracked",
        &[
            "-p",
            "p0",
            "--anchor",
            "insurers",
            "--content",
            "underwriters",
        ],
    );
}

#[test]
fn edit_rejects_a_deletion_nested_in_an_insertion() {
    edits_as("reject-nested-deletion", "annex", &["-p", "r4", "--reject"]);
}
