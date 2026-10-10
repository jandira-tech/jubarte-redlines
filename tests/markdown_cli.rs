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

#[test]
fn github_diff_is_text_only_for_docx_and_mixed_inputs() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    for (input, output) in [("old.md", "old.docx"), ("new.md", "new.docx")] {
        ok(&jubarte(&["convert", input, "-o", output], dir.path()));
    }
    for (old, new) in [
        ("old.md", "new.md"),
        ("old.docx", "new.docx"),
        ("old.docx", "new.md"),
    ] {
        let out = jubarte(
            &["diff", old, new, "--format", "github", "--context", "0"],
            dir.path(),
        );
        let patch = ok(&out);
        assert!(patch.starts_with("diff --git "), "{patch}");
        assert!(
            patch.contains("@@ ") && patch.contains("45 days"),
            "{patch}"
        );
        assert!(
            out.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!dir.path().join("old_v_new.docx").exists());
    }
}

#[test]
fn github_file_output_has_no_stdout_and_preserves_no_clobber() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    let args = [
        "diff",
        "old.md",
        "new.md",
        "--format",
        "unified",
        "-o",
        "changes.patch",
    ];
    let out = jubarte(&args, dir.path());
    assert!(ok(&out).is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("wrote changes.patch"));
    let patch = std::fs::read_to_string(dir.path().join("changes.patch")).unwrap();
    assert!(patch.starts_with("diff --git "));
    let refused = jubarte(&args, dir.path());
    assert_eq!(refused.status.code(), Some(1));
    assert!(refused.stdout.is_empty());
    assert!(failed(&refused).contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("changes.patch")).unwrap(),
        patch
    );
    assert!(ok(&jubarte(&[&args[..], &["--force"]].concat(), dir.path())).is_empty());
}

#[test]
fn github_render_outputs_fail_before_input_io() {
    let dir = tempfile::tempdir().unwrap();
    for extra in [
        vec!["--to", "docx"],
        vec!["--to", "pdf"],
        vec!["-o", "out.png"],
    ] {
        let args = [
            vec![
                "diff",
                "missing.docx",
                "also-missing.docx",
                "--format",
                "github",
            ],
            extra,
        ]
        .concat();
        let out = jubarte(&args, dir.path());
        assert_eq!(out.status.code(), Some(2));
        let stderr = failed(&out);
        assert!(stderr.contains("Usage:"), "{stderr}");
        assert!(!stderr.contains("reading"), "{stderr}");
        assert!(out.stdout.is_empty());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// `jubarte` with `user.name` set to `name` in the git configuration it
/// sees (none at all when `None`): the patch's default owner.
fn jubarte_as(args: &[&str], dir: &Path, name: Option<&str>) -> Output {
    let mut command = Command::new(BIN);
    command
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CEILING_DIRECTORIES", dir);
    if let Some(name) = name {
        command
            .env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "user.name")
            .env("GIT_CONFIG_VALUE_0", name);
    }
    command.output().unwrap()
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
    // Markdown to a Markdown file: refused over an existing one unless forced.
    let args = ["convert", "draft.md", "-t", "md", "-o", "out.md"];
    ok(&jubarte(
        &[&args[..], &["--track-changes", "accept"]].concat(),
        dir.path(),
    ));
    let out = || std::fs::read_to_string(dir.path().join("out.md")).unwrap();
    assert_eq!(out(), "Payment is due in 45 days.\n");
    let stderr = failed(&jubarte(&args, dir.path()));
    assert!(stderr.contains("already exists"), "{stderr}");
    ok(&jubarte(
        &[&args[..], &["--no-critic", "--force"]].concat(),
        dir.path(),
    ));
    assert_eq!(out(), DRAFT);
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
fn convert_writes_word_back_as_critic_markup() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    ok(&jubarte(&["convert", "draft.md"], dir.path()));
    let critic = "Payment is due in {~~30~>45~~}{>>Redline (1970-01-01T00:00:00Z)<<} days.\
                  {>>Redline (1970-01-01T00:00:00Z): Agreed on the call.<<}\n";
    // By default the Markdown names its pages; the text after the marker is
    // exactly what --no-page-markers writes.
    assert_eq!(
        ok(&jubarte(&["convert", "draft.docx", "-t", "md"], dir.path())),
        format!("<!-- page 1 of 1 -->\n\n{critic}")
    );
    assert_eq!(
        ok(&jubarte(
            &["convert", "draft.docx", "-t", "md", "--no-page-markers"],
            dir.path()
        )),
        critic
    );
    for (choice, text) in [("accept", "45"), ("reject", "30")] {
        assert_eq!(
            ok(&jubarte(
                &[
                    "convert",
                    "draft.docx",
                    "-t",
                    "md",
                    "--no-page-markers",
                    "--track-changes",
                    choice
                ],
                dir.path()
            )),
            format!("Payment is due in {text} days.\n")
        );
    }
    let stdout = ok(&jubarte(
        &[
            "convert",
            "draft.docx",
            "-o",
            "back.md",
            "--no-page-markers",
        ],
        dir.path(),
    ));
    assert!(stdout.contains("wrote back.md"), "{stdout}");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("back.md")).unwrap(),
        critic
    );
}

#[test]
fn convert_refuses_what_it_cannot_do_yet() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    ok(&jubarte(&["convert", "draft.md"], dir.path()));
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
    let stdout = ok(&jubarte(
        &["diff", "old.md", "new.md", "--format", "critic"],
        dir.path(),
    ));
    assert_eq!(
        stdout,
        "# Terms\n\nPayment is due in {~~30~>45~~} days.\n\n- Delivery\n- {++Returns++}\n- Warranty\n"
    );
    // Written to a file, the patch is still printed.
    let printed = ok(&jubarte(
        &["diff", "old.md", "new.md", "-o", "changes.md"],
        dir.path(),
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("changes.md")).unwrap(),
        stdout
    );
    assert!(
        printed.starts_with("--- a/old.md\n+++ b/new.md\t"),
        "{printed}"
    );
}

const OWNER: [&str; 4] = ["-a", "Arthur Rodrigues", "-d", "2026-09-30T14:05:00Z"];

#[test]
fn diff_prints_a_patch_of_the_changed_paragraphs_by_default() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    let args = [&["diff", "old.md", "new.md"][..], &OWNER].concat();
    let stdout = ok(&jubarte(&args, dir.path()));
    assert!(
        stdout.starts_with(
            "--- a/old.md\n+++ b/new.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
             @@ [line:3] @@\nPayment is due in [-30-]{+45+} days.\n"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("{+Returns+}"), "{stdout}");
    assert!(!stdout.contains("# Terms"), "{stdout}");
    // Nothing changed, nothing printed.
    let same = [&["diff", "old.md", "old.md"][..], &OWNER].concat();
    assert_eq!(ok(&jubarte(&same, dir.path())), "");
}

#[test]
fn diff_wraps_at_72_columns_unless_told_otherwise() {
    let dir = tempfile::tempdir().unwrap();
    let long = "word ".repeat(40);
    seed(
        dir.path(),
        &[
            ("old.md", &format!("{long}old.\n")),
            ("new.md", &format!("{long}new.\n")),
        ],
    );
    let lines = |columns: Option<&str>| {
        let mut args = [&["diff", "old.md", "new.md"][..], &OWNER].concat();
        if let Some(columns) = columns {
            args.extend(["--columns", columns]);
        }
        let stdout = ok(&jubarte(&args, dir.path()));
        stdout
            .lines()
            .skip(3)
            .map(|l| l.chars().count())
            .collect::<Vec<_>>()
    };
    let wrapped = lines(None);
    assert!(
        wrapped.len() > 1 && wrapped.iter().all(|&n| n <= 72),
        "{wrapped:?}"
    );
    let narrow = lines(Some("40"));
    assert!(
        narrow.len() > wrapped.len() && narrow.iter().all(|&n| n <= 40),
        "{narrow:?}"
    );
    assert_eq!(lines(Some("0")).len(), 1);
}

#[test]
fn the_owner_is_git_user_name_or_redline_and_the_date_now() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    let header = |name: Option<&str>| {
        let stdout = ok(&jubarte_as(&["diff", "old.md", "new.md"], dir.path(), name));
        stdout.lines().nth(1).unwrap().to_string()
    };
    let named = header(Some("Ana Lima"));
    let fields: Vec<&str> = named.split('\t').collect();
    assert_eq!(fields[..2], ["+++ b/new.md", "Ana Lima"], "{named}");
    // YYYY-MM-DDTHH:MM:SSZ, this year or later.
    let date = fields[2];
    assert_eq!(date.len(), 20, "{date}");
    assert!(date.ends_with('Z') && &date[10..11] == "T", "{date}");
    assert!(date[..4].parse::<u32>().unwrap() >= 2026, "{date}");
    assert!(header(None).contains("\tRedline\t"));
}

#[test]
fn diff_takes_no_quiet_flag() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    let stderr = failed(&jubarte(&["diff", "-q", "old.md", "new.md"], dir.path()));
    assert!(stderr.contains("'-q'"), "{stderr}");
}

#[test]
fn diff_with_a_word_side_prints_paragraph_ids_and_says_what_it_wrote_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("old.md", OLD), ("new.md", NEW)]);
    ok(&jubarte(
        &["convert", "old.md", "-o", "contract.docx"],
        dir.path(),
    ));
    let args = [&["diff", "contract.docx", "new.md"][..], &OWNER].concat();
    let out = jubarte(&args, dir.path());
    let stdout = ok(&out);
    assert!(
        stdout.starts_with("--- a/contract.docx\n+++ b/new.md\tArthur Rodrigues\t"),
        "{stdout}"
    );
    assert!(
        stdout.contains("@@ [body:p:1] @@\nPayment is due in [-30-]{+45+} days.\n"),
        "{stdout}"
    );
    assert!(!stdout.contains("{>>Arthur Rodrigues ("), "{stdout}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("contract_v_new.docx"), "{stderr}");
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

/// A patch printed beside a PDF or PNG output is printed only once the
/// output is written: a refused output prints no patch, and what was
/// written is said on stderr so stdout holds the patch alone.
#[test]
fn diff_prints_the_patch_only_after_its_pdf_or_png_is_written() {
    let dir = tempfile::tempdir().unwrap();
    seed(
        dir.path(),
        &[
            ("old.md", OLD),
            ("new.md", NEW),
            ("taken.pdf", "keep"),
            ("pages-page-01.png", "keep"),
        ],
    );
    for output in ["taken.pdf", "pages.png"] {
        let out = jubarte(&["diff", "old.md", "new.md", "-o", output], dir.path());
        assert!(failed(&out).contains("already exists"), "{output}");
        assert!(out.stdout.is_empty(), "{output}: {:?}", out.stdout);
    }
    assert_eq!(
        std::fs::read(dir.path().join("taken.pdf")).unwrap(),
        b"keep"
    );

    for (output, written) in [
        ("fresh.pdf", "fresh.pdf"),
        ("fresh.png", "fresh-page-01.png"),
    ] {
        let out = jubarte(&["diff", "old.md", "new.md", "-o", output], dir.path());
        let stdout = ok(&out);
        assert!(
            stdout.starts_with("--- a/old.md\n+++ b/new.md\t"),
            "{stdout}"
        );
        assert!(!stdout.contains("wrote"), "{stdout}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("wrote"), "{output}: {stderr}");
        assert!(dir.path().join(written).exists(), "{written}");
    }
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
    let out = jubarte(&["diff", "contract.docx", "new.md"], dir.path());
    ok(&out);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("contract_v_new.docx"), "{stderr}");
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
    ok(&jubarte(&["compare", "old.md", "new.md"], dir.path()));
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

#[test]
fn convert_resolves_word_to_word_and_renders_markdown_to_png() {
    let dir = tempfile::tempdir().unwrap();
    seed(dir.path(), &[("draft.md", DRAFT)]);
    ok(&jubarte(&["convert", "draft.md"], dir.path()));
    let stderr = failed(&jubarte(
        &[
            "convert",
            "draft.docx",
            "-t",
            "docx",
            "--track-changes",
            "accept",
        ],
        dir.path(),
    ));
    assert!(stderr.contains("--output is required"), "{stderr}");
    // Word to PDF with a choice renders the accepted (or rejected) document.
    let stdout = ok(&jubarte(
        &["convert", "draft.docx", "--track-changes", "reject"],
        dir.path(),
    ));
    assert!(stdout.contains("draft.pdf"), "{stdout}");
    ok(&jubarte(
        &[
            "convert",
            "draft.docx",
            "-o",
            "accepted.docx",
            "--track-changes",
            "accept",
        ],
        dir.path(),
    ));
    ok(&jubarte(
        &[
            "convert",
            "draft.docx",
            "-t",
            "docx",
            "-o",
            "rejected.docx",
            "--track-changes",
            "reject",
        ],
        dir.path(),
    ));
    let accepted = std::fs::read(dir.path().join("accepted.docx")).unwrap();
    let rejected = std::fs::read(dir.path().join("rejected.docx")).unwrap();
    assert_eq!(texts(&accepted), ["Payment is due in 45 days."]);
    assert_eq!(texts(&rejected), ["Payment is due in 30 days."]);
    let stdout = ok(&jubarte(
        &["convert", "draft.md", "-t", "png", "--dpi", "20"],
        dir.path(),
    ));
    assert!(stdout.contains("PNG page"), "{stdout}");
    assert!(dir.path().join("draft-page-01.png").exists());
}

#[test]
fn inputs_are_told_apart_by_extension_then_by_their_bytes() {
    let dir = tempfile::tempdir().unwrap();
    // No extension: Markdown; a .bin that is a zip: Word; a BOM is dropped.
    std::fs::write(dir.path().join("NOTES"), "\u{FEFF}Notes {++added++}.\n").unwrap();
    ok(&jubarte(
        &["convert", "NOTES", "-o", "notes.docx"],
        dir.path(),
    ));
    std::fs::copy(dir.path().join("notes.docx"), dir.path().join("notes.bin")).unwrap();
    assert!(
        ok(&jubarte(
            &["convert", "notes.bin", "-t", "md", "--no-page-markers"],
            dir.path()
        ))
        .starts_with("Notes {++added++}"),
        "a .bin holding a zip reads as Word"
    );
    assert_eq!(
        ok(&jubarte(
            &["convert", "NOTES", "-t", "md", "--track-changes", "accept"],
            dir.path()
        )),
        "Notes added.\n"
    );
    // --from wins over the extension.
    std::fs::write(dir.path().join("plain.txt"), "Plain.\n").unwrap();
    ok(&jubarte(
        &["convert", "plain.txt", "-f", "markdown", "-o", "plain.docx"],
        dir.path(),
    ));
    assert_eq!(
        texts(&std::fs::read(dir.path().join("plain.docx")).unwrap()),
        ["Plain."]
    );
    // Not UTF-8, and not an input format.
    std::fs::write(dir.path().join("bad.md"), [0xFF, 0xFE, 0x00]).unwrap();
    let stderr = failed(&jubarte(&["convert", "bad.md"], dir.path()));
    assert!(stderr.contains("must be UTF-8"), "{stderr}");
    let stderr = failed(&jubarte(&["convert", "plain.txt", "-f", "pdf"], dir.path()));
    assert!(stderr.contains("not inputs"), "{stderr}");
    let stderr = failed(&jubarte(
        &["diff", "plain.txt", "NOTES", "-f", "png"],
        dir.path(),
    ));
    assert!(
        stderr.contains("invalid value 'png'") && stderr.contains("docx, md, markdown"),
        "{stderr}"
    );
}

#[test]
fn images_are_read_next_to_the_markdown_with_escapes_decoded() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("figs")).unwrap();
    let mut png = Vec::new();
    {
        use image::ImageEncoder as _;
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&[0u8; 16], 4, 4, image::ExtendedColorType::L8)
            .unwrap();
    }
    std::fs::write(dir.path().join("figs").join("my chart.png"), &png).unwrap();
    seed(
        dir.path(),
        &[(
            "doc.md",
            "![local](figs/my%20chart.png) ![remote](https://example.com/a.png) ![inline](data:image/png;base64,AAAA)\n",
        )],
    );
    let out = jubarte(&["convert", "doc.md"], dir.path());
    ok(&out);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.matches("warning:").count(), 2, "{stderr}");
    let docx = std::fs::read(dir.path().join("doc.docx")).unwrap();
    assert!(
        common::docx::part_string(&docx, "word/document.xml")
            .unwrap()
            .contains("descr=\"local\"")
    );
    // --resource-path points elsewhere.
    seed(dir.path(), &[("other.md", "![local](my%20chart.png)\n")]);
    let out = jubarte(
        &["convert", "other.md", "--resource-path", "figs"],
        dir.path(),
    );
    ok(&out);
    assert!(!String::from_utf8_lossy(&out.stderr).contains("warning:"));
}

#[test]
fn diff_takes_a_reference_and_can_read_critic_markup() {
    let dir = tempfile::tempdir().unwrap();
    seed(
        dir.path(),
        &[
            ("old.md", "Text {++kept++}.\n"),
            ("new.md", "Text {++kept++} and more.\n"),
        ],
    );
    // By default the documents are text: the delimiters are compared too.
    let stdout = ok(&jubarte(
        &["diff", "old.md", "new.md", "--format", "critic"],
        dir.path(),
    ));
    assert_eq!(stdout, "Text \\{++kept++\\}{++ and more++}.\n");
    let reference = std::fs::canonicalize("tests/fixtures/redline-inpi/original-new.docx").unwrap();
    ok(&jubarte(
        &[
            "diff",
            "old.md",
            "new.md",
            "--critic",
            "--reference-doc",
            reference.to_str().unwrap(),
            "-o",
            "d.docx",
        ],
        dir.path(),
    ));
    let docx = std::fs::read(dir.path().join("d.docx")).unwrap();
    assert_word_valid_package(&docx);
    assert!(common::docx::part_string(&docx, "word/header1.xml").is_some());
    ok(&jubarte(
        &["diff", "old.md", "new.md", "-t", "png"],
        dir.path(),
    ));
    assert!(dir.path().join("old_v_new-page-01.png").exists());
    // Two Word documents cannot give Markdown yet.
    ok(&jubarte(&["convert", "old.md", "-o", "a.docx"], dir.path()));
    ok(&jubarte(&["convert", "new.md", "-o", "b.docx"], dir.path()));
    let stderr = failed(&jubarte(
        &["a.docx", "b.docx", "-o", "changes.md"],
        dir.path(),
    ));
    assert!(
        stderr.contains("needs both documents in Markdown"),
        "{stderr}"
    );
}

#[test]
fn text_diff_views_accept_history_only_when_requested() {
    let dir = tempfile::tempdir().unwrap();
    seed(
        dir.path(),
        &[
            ("a.md", "Fee {--old--}{++same++}.\nDue 30 days.\n"),
            ("b.md", "Fee same.\nDue 60 days.\n"),
        ],
    );
    let args = ["diff", "a.md", "b.md", "--format", "word", "--full-lines"];
    let text = ok(&jubarte(&args, dir.path()));
    assert!(
        (text.contains("{--30--}{++60++}") || text.contains("{~~30~>60~~}")),
        "{text}"
    );
    assert!(!text.contains("old") && !text.contains("Fee"), "{text}");
    assert!(!dir.path().join("a_v_b.docx").exists());
    let normal = ok(&jubarte(
        &[
            "diff",
            "a.md",
            "b.md",
            "--format",
            "normal",
            "--accept-changes",
            "--full-lines",
        ],
        dir.path(),
    ));
    assert_eq!(normal, "2c2\n< Due 30 days.\n---\n> Due 60 days.\n");
    let context = ok(&jubarte(
        &["diff", "a.md", "b.md", "--format", "context", "-U0"],
        dir.path(),
    ));
    assert!(context.contains("! Fee {--old--}{++same++}."), "{context}");
    let side = ok(&jubarte(
        &["diff", "a.md", "b.md", "--format", "side-by-side"],
        dir.path(),
    ));
    assert!(side.contains('|') && side.contains("{--old--}"), "{side}");
}

#[test]
fn github_display_clips_around_unicode_change_and_full_lines_restores_content() {
    let dir = tempfile::tempdir().unwrap();
    let old = format!("{}old{}\n", "é".repeat(120), "Z".repeat(120));
    let new = old.replace("old", "new");
    seed(dir.path(), &[("a.md", &old), ("b.md", &new)]);
    let base = ["diff", "a.md", "b.md", "--format", "github"];
    let clipped = ok(&jubarte(&base, dir.path()));
    assert!(
        clipped.contains('…') && clipped.contains("old") && clipped.contains("new"),
        "{clipped}"
    );
    assert!(!clipped.contains(&"é".repeat(100)));
    let full = ok(&jubarte(
        &[&base[..], &["--full-lines"]].concat(),
        dir.path(),
    ));
    assert!(full.contains(&old) && full.contains(&new), "{full}");
}
