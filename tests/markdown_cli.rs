// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The CLI's Markdown paths, through the compiled binary: `convert` from
//! Markdown (pandoc style, CriticMarkup on by default), `diff` (pandiff
//! style) and the positional compare with Markdown documents.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::validity::assert_word_valid_package;
use jubarte::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use jubarte::document_comparer::{accept_revisions, get_revisions, reject_revisions};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

const OLD: &str = "# Terms\n\nPayment is due in 30 days.\n\n- Delivery\n- Warranty\n";
const NEW: &str = "# Terms\n\nPayment is due in 45 days.\n\n- Delivery\n- Returns\n- Warranty\n";
const DRAFT: &str = "Payment is due in {~~30~>45~~} days.{>>Agreed on the call.<<}\n";

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn failed(out: &Output) -> String {
    assert!(
        !out.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn seed(dir: &Path, files: &[(&str, &str)]) -> Vec<PathBuf> {
    files
        .iter()
        .map(|(name, text)| {
            let path = dir.join(name);
            std::fs::write(&path, text).unwrap();
            path
        })
        .collect()
}

fn texts(docx: &[u8]) -> Vec<String> {
    jubarte::inspect::paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn revision_kinds(docx: &[u8]) -> Vec<WmlComparerRevisionType> {
    get_revisions(docx, &WmlComparerSettings::default())
        .unwrap()
        .into_iter()
        .map(|r| r.revision_type)
        .collect()
}

#[test]
fn convert_writes_critic_markup_as_tracked_changes_next_to_the_markdown() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    let stdout = ok(&jubarte(
        &["convert", "draft.md", "--author", "Legal"],
        dir.path(),
    ));
    assert!(stdout.contains("draft.docx"), "{stdout}");
    let docx = std::fs::read(dir.path().join("draft.docx")).unwrap();
    assert_word_valid_package(&docx);
    let kinds = revision_kinds(&docx);
    assert!(kinds.contains(&WmlComparerRevisionType::Inserted));
    assert!(kinds.contains(&WmlComparerRevisionType::Deleted));
    assert!(
        common::docx::part_string(&docx, "word/comments.xml")
            .unwrap()
            .contains("Agreed on the call.")
    );
    assert!(
        common::docx::part_string(&docx, "word/document.xml")
            .unwrap()
            .contains("w:author=\"Legal\"")
    );
    assert_eq!(
        texts(&accept_revisions(&docx).unwrap()),
        ["Payment is due in 45 days."]
    );
    assert_eq!(
        texts(&reject_revisions(&docx).unwrap()),
        ["Payment is due in 30 days."]
    );
}

#[test]
fn convert_accepts_rejects_or_ignores_the_markup() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    // Markdown to Markdown, pandoc's --track-changes.
    assert_eq!(
        ok(&jubarte(
            &[
                "convert",
                "draft.md",
                "-t",
                "md",
                "--track-changes",
                "accept"
            ],
            dir.path()
        )),
        "Payment is due in 45 days.\n"
    );
    assert_eq!(
        ok(&jubarte(
            &[
                "convert",
                "draft.md",
                "--to",
                "markdown",
                "--track-changes",
                "reject"
            ],
            dir.path()
        )),
        "Payment is due in 30 days.\n"
    );
    // Word without tracked changes, and Word with the delimiters as text.
    ok(&jubarte(
        &[
            "convert",
            "draft.md",
            "-o",
            "clean.docx",
            "--track-changes",
            "accept",
        ],
        dir.path(),
    ));
    let clean = std::fs::read(dir.path().join("clean.docx")).unwrap();
    assert!(revision_kinds(&clean).is_empty());
    assert_eq!(texts(&clean), ["Payment is due in 45 days."]);
    ok(&jubarte(
        &["convert", "draft.md", "-o", "text.docx", "--no-critic"],
        dir.path(),
    ));
    let text = std::fs::read(dir.path().join("text.docx")).unwrap();
    assert_eq!(
        texts(&text),
        ["Payment is due in {~~30~>45~~} days.{>>Agreed on the call.<<}"]
    );
}

#[test]
fn convert_renders_markdown_to_pdf_with_the_changes_painted() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    let stdout = ok(&jubarte(
        &["convert", "draft.md", "-o", "draft.pdf"],
        dir.path(),
    ));
    assert!(stdout.contains("draft.pdf"), "{stdout}");
    let pdf = std::fs::read(dir.path().join("draft.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}

#[test]
fn convert_refuses_what_it_cannot_do_yet() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    ok(&jubarte(&["convert", "draft.md"], dir.path()));
    let stderr = failed(&jubarte(&["convert", "draft.docx", "-t", "md"], dir.path()));
    assert!(
        stderr.contains("Word to Markdown is not in this build yet"),
        "{stderr}"
    );
    let stderr = failed(&jubarte(
        &["convert", "draft.docx", "-o", "again.docx"],
        dir.path(),
    ));
    assert!(stderr.contains("already Word"), "{stderr}");
    // No clobbering without --force.
    let stderr = failed(&jubarte(&["convert", "draft.md"], dir.path()));
    assert!(stderr.contains("--force"), "{stderr}");
    ok(&jubarte(&["convert", "draft.md", "--force"], dir.path()));
}

#[test]
fn diff_prints_critic_markup_like_pandiff() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    let stdout = ok(&jubarte(&["diff", "old.md", "new.md"], dir.path()));
    assert_eq!(
        stdout,
        "# Terms\n\nPayment is due in {~~30~>45~~} days.\n\n- Delivery\n- {++Returns++}\n- Warranty\n"
    );
    ok(&jubarte(
        &["diff", "old.md", "new.md", "-o", "changes.md"],
        dir.path(),
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("changes.md")).unwrap(),
        stdout
    );
}

#[test]
fn diff_writes_a_word_redline_or_a_pdf() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    ok(&jubarte(
        &[
            "diff",
            "old.md",
            "new.md",
            "-o",
            "changes.docx",
            "-a",
            "Legal",
        ],
        dir.path(),
    ));
    let docx = std::fs::read(dir.path().join("changes.docx")).unwrap();
    assert_word_valid_package(&docx);
    assert_eq!(
        texts(&accept_revisions(&docx).unwrap()),
        [
            "Terms",
            "Payment is due in 45 days.",
            "Delivery",
            "Returns",
            "Warranty"
        ]
    );
    assert_eq!(
        texts(&reject_revisions(&docx).unwrap()),
        [
            "Terms",
            "Payment is due in 30 days.",
            "Delivery",
            "Warranty"
        ]
    );
    ok(&jubarte(
        &["diff", "old.md", "new.md", "-o", "changes.pdf"],
        dir.path(),
    ));
    assert!(
        std::fs::read(dir.path().join("changes.pdf"))
            .unwrap()
            .starts_with(b"%PDF")
    );
}

#[test]
fn diff_and_compare_take_word_against_markdown() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    // A Word original, edited as Markdown.
    ok(&jubarte(
        &["convert", "old.md", "-o", "contract.docx"],
        dir.path(),
    ));
    let stdout = ok(&jubarte(&["diff", "contract.docx", "new.md"], dir.path()));
    assert!(stdout.contains("contract_v_new.docx"), "{stdout}");
    let redline = std::fs::read(dir.path().join("contract_v_new.docx")).unwrap();
    assert_word_valid_package(&redline);
    assert_eq!(
        texts(&accept_revisions(&redline).unwrap()),
        [
            "Terms",
            "Payment is due in 45 days.",
            "Delivery",
            "Returns",
            "Warranty"
        ]
    );
    // Markdown output needs Markdown on both sides for now.
    let stderr = failed(&jubarte(
        &["diff", "contract.docx", "new.md", "-t", "md"],
        dir.path(),
    ));
    assert!(
        stderr.contains("Markdown output needs both documents in Markdown"),
        "{stderr}"
    );
    // The positional compare takes Markdown too.
    ok(&jubarte(&["old.md", "new.md"], dir.path()));
    assert_word_valid_package(&std::fs::read(dir.path().join("old_v_new.docx")).unwrap());
    ok(&jubarte(
        &["old.md", "new.md", "-o", "changes.md"],
        dir.path(),
    ));
    assert!(
        std::fs::read_to_string(dir.path().join("changes.md"))
            .unwrap()
            .contains("{++Returns++}")
    );
}

#[test]
fn help_names_the_markdown_commands() {
    let dir = tempfile::tempdir().unwrap();
    for (args, needle) in [
        (&["--help"][..], "diff"),
        (
            &["diff", "--help"][..],
            "git config --global difftool.jubarte.cmd",
        ),
        (&["convert", "--help"][..], "--track-changes"),
        (&["convert", "--help"][..], "--reference-doc"),
    ] {
        let stdout = ok(&jubarte(args, dir.path()));
        assert!(stdout.contains(needle), "{args:?}: {stdout}");
    }
}
