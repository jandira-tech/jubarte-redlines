// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `convert --page letter|a4` for Markdown written to Word, and
//! `text --track-changes all|accept|reject`, through the compiled binary.

mod common;

use std::path::Path;
use std::process::{Command, Output};

use common::docx::part_string;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
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

fn page_width(dir: &Path, docx: &str) -> String {
    let xml = part_string(&std::fs::read(dir.join(docx)).unwrap(), "word/document.xml").unwrap();
    let at = xml.find("<w:pgSz ").expect("a page size");
    let tag = &xml[at..at + xml[at..].find('>').unwrap()];
    let width = tag.find("w:w=\"").unwrap() + 5;
    tag[width..width + tag[width..].find('"').unwrap()].to_string()
}

#[test]
fn convert_writes_markdown_on_letter_or_a4() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("draft.md"), DRAFT).unwrap();

    ok(&jubarte(
        &["convert", "draft.md", "-o", "default.docx"],
        dir.path(),
    ));
    ok(&jubarte(
        &[
            "convert",
            "draft.md",
            "--page",
            "letter",
            "-o",
            "letter.docx",
        ],
        dir.path(),
    ));
    ok(&jubarte(
        &["convert", "draft.md", "--page", "a4", "-o", "a4.docx"],
        dir.path(),
    ));
    assert_eq!(page_width(dir.path(), "default.docx"), "12240");
    assert_eq!(page_width(dir.path(), "letter.docx"), "12240");
    assert_eq!(page_width(dir.path(), "a4.docx"), "11906");

    let out = jubarte(&["convert", "draft.md", "--page", "legal"], dir.path());
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("legal"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn convert_warns_when_a_reference_overrides_the_page() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("draft.md"), DRAFT).unwrap();
    let reference =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/redline-inpi/original-new.docx");
    let out = jubarte(
        &[
            "convert",
            "draft.md",
            "--page",
            "a4",
            "--reference-doc",
            reference.to_str().unwrap(),
            "-o",
            "out.docx",
        ],
        dir.path(),
    );
    ok(&out);
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .contains("warning: page size a4 ignored: the reference document's page setup is used"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(page_width(dir.path(), "out.docx"), "11906");
}

#[test]
fn text_reads_tracked_changes_as_critic_markup_or_resolves_them() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("draft.md"), DRAFT).unwrap();
    ok(&jubarte(&["convert", "draft.md"], dir.path()));

    // Without the flag: tracked changes inline, with the paragraph ids an
    // edit plan uses.
    let plain = ok(&jubarte(&["text", "draft.docx"], dir.path()));
    assert!(plain.contains("<!-- p0"), "{plain}");

    let all = ok(&jubarte(
        &["text", "draft.docx", "--track-changes", "all"],
        dir.path(),
    ));
    assert_eq!(all, plain, "all is the default");
    assert!(all.contains("{~~30~>45~~}"), "{all}");
    assert!(all.contains("Agreed on the call."), "{all}");

    let accepted = ok(&jubarte(
        &["text", "draft.docx", "--track-changes", "accept"],
        dir.path(),
    ));
    assert!(
        accepted.contains("\nPayment is due in 45 days."),
        "{accepted}"
    );
    assert!(!accepted.contains("{~~"), "{accepted}");
    let rejected = ok(&jubarte(
        &["text", "draft.docx", "--track-changes", "reject"],
        dir.path(),
    ));
    assert!(
        rejected.contains("\nPayment is due in 30 days."),
        "{rejected}"
    );
    assert!(!rejected.contains("{~~"), "{rejected}");
}

#[test]
fn help_describes_the_new_flags() {
    let dir = tempfile::tempdir().unwrap();
    let convert = ok(&jubarte(&["convert", "--help"], dir.path()));
    assert!(convert.contains("--page"), "{convert}");
    let text = ok(&jubarte(&["text", "--help"], dir.path()));
    assert!(text.contains("--track-changes"), "{text}");
    assert!(text.contains("<!-- pN -->"), "{text}");
}
