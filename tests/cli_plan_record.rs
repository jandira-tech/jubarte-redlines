// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit --plan` and `add --plan` with operation flags record into the plan
//! instead of editing; `--plan` alone applies it, and `--consume` deletes it
//! once applied.
#![cfg(feature = "cli")]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::docx::{docx, para};
use jubarte::cli::slash_path;
use jubarte::inspect::source_sha256;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn letter(dir: &Path, name: &str, last: &str) -> PathBuf {
    let body = para("Heading")
        + &para("The individual signs in his or her individual capacity.")
        + &para(last);
    let path = dir.join(name);
    std::fs::write(&path, docx(&body)).unwrap();
    path
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn plan(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join("plan.json")).unwrap()).unwrap()
}

/// clap's typed `--author` default (`InlineOptions`), not a guessed name.
fn typed_default_author() -> String {
    let cli = jubarte::cli::Cli::try_parse_from([
        "jubarte", "edit", "replace", "a.docx", "--find", "a", "--text", "b", "-o", "out.docx",
    ])
    .unwrap_or_else(|error| panic!("{error}"));
    let Some(jubarte::cli::Command::TypedEdit(typed)) = cli.command else {
        panic!("not a typed edit");
    };
    typed
        .action
        .inline()
        .expect("replace has inline options")
        .author
        .clone()
}

#[test]
fn occurrence_1894_replace_names_paragraphs_with_one_match_each() {
    replace_ambiguity_1894(
        false,
        "found 4: p6, p7, p40, p120; pick one with --at pN or --occurrence 1..=4",
    );
}

#[test]
fn occurrence_1894_replace_names_paragraphs_and_repeated_match_counts() {
    replace_ambiguity_1894(
        true,
        "found 5: p6 (2), p7, p40, p120; pick one with --at pN or --occurrence 1..=5",
    );
}

fn replace_ambiguity_1894(repeat: bool, expected: &str) {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let body: String = (0..=120)
        .map(|i| {
            para(match i {
                6 if repeat => "Affiliate Affiliate",
                6 | 7 | 40 | 120 => "Affiliate",
                _ => "Other text",
            })
        })
        .collect();
    std::fs::write(dir.join("a.docx"), docx(&body)).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "replace",
            "a.docx",
            "--find",
            "Affiliate",
            "--text",
            "Affiliatez",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 3, "{out}{err}");
    assert!(err.contains("AMBIGUOUS_ANCHOR"), "{err}");
    assert!(err.contains(expected), "{err}");
    assert!(!dir.join("out.docx").exists());
}

#[test]
fn edit_and_add_record_into_one_plan_that_edit_applies_and_consumes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let file = letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let before = std::fs::read(&file).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        out.trim(),
        "recorded 1 operation in plan.json: 1 operation, 0 revisions to accept, 0 to reject; \
         apply with: jubarte edit letter.docx --plan plan.json"
    );
    let (code, out, err) = run(
        dir,
        &[
            "add",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--comment",
            "--content",
            "Why 3?",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(
        out.starts_with("recorded 1 operation in plan.json: 2 operations"),
        "{out}"
    );
    // Recording edits nothing: FILE is as it was and no output bundle exists.
    assert_eq!(std::fs::read(&file).unwrap(), before);
    assert!(!dir.join("letter.edit").exists());
    // The plan reads as one written by hand: no null fields.
    let json = std::fs::read_to_string(dir.join("plan.json")).unwrap();
    assert!(!json.contains("null"), "{json}");
    let recorded = plan(dir);
    assert_eq!(recorded["operations"].as_array().unwrap().len(), 2);
    assert_eq!(recorded["source_sha256"], source_sha256(&before));
    assert_eq!(recorded["author"], "Jubarte");

    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--out-dir",
            "out",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(
        !dir.join("plan.json").exists(),
        "--consume deletes the plan"
    );
    let (_, view, _) = run(dir, &["read", "out/redline.docx"]);
    assert!(view.contains("{~~his or her~>their~~}{r1+2 @J}"), "{view}");
    assert!(view.contains("Why 3?"), "{view}");
}

#[test]
fn a_recording_that_does_not_apply_leaves_the_plan_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "Gone.",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let held = std::fs::read(dir.join("plan.json")).unwrap();
    let (code, out, _) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "nowhere",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 3, "{out}");
    assert!(out.contains(r#""status":"failed""#), "{out}");
    assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), held);
    let (code, out, _) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--dry-run",
            "-p",
            "p1",
            "--content",
            "Checked only.",
        ],
    );
    assert_eq!(code, 0);
    assert!(
        out.starts_with("would record 1 operation in plan.json: 2 operations"),
        "{out}"
    );
    assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), held);
}

#[test]
fn a_recording_refuses_what_applies_later_and_what_the_plan_names_otherwise() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    letter(dir, "other.docx", "Sections 4 and 5 survive.");
    let record = |extra: &[&str], file: &str| {
        let mut args = vec![
            "edit",
            file,
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ];
        args.extend_from_slice(extra);
        run(dir, &args)
    };
    for flag in [
        &["--out-dir", "d"][..],
        &["--pdf"],
        &["--consume"],
        &["--editing-mode"],
    ] {
        let (code, _, err) = record(flag, "letter.docx");
        assert_eq!(code, 2, "{flag:?}");
        assert!(
            err.contains(&format!(
                "{} takes effect when the plan is applied",
                flag[0]
            )),
            "{err}"
        );
        assert!(!dir.join("plan.json").exists(), "{flag:?}");
    }
    assert_eq!(record(&[], "letter.docx").0, 0);
    let (code, _, err) = record(&["--author", "Bob"], "letter.docx");
    assert_eq!(code, 2);
    assert!(
        err.contains(r#"the plan's author is "Jubarte"; pass --author "Jubarte""#),
        "{err}"
    );
    let (code, _, err) = record(&[], "other.docx");
    assert_eq!(code, 2);
    assert!(err.contains("recorded against another document"), "{err}");
    assert_eq!(plan(dir)["operations"].as_array().unwrap().len(), 1);
}

#[test]
fn consume_keeps_the_plan_until_it_has_been_applied() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    letter(dir, "other.docx", "Sections 4 and 5 survive.");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--dry-run",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("plan.json").exists(), "a dry run consumes nothing");
    // The plan is pinned to letter.docx: another file is refused as stale.
    let (code, _, _) = run(
        dir,
        &[
            "edit",
            "other.docx",
            "--plan",
            "plan.json",
            "--consume",
            "--out-dir",
            "o",
        ],
    );
    assert_eq!(code, 3);
    assert!(dir.join("plan.json").exists(), "a refused plan is kept");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--author",
            "Bob",
        ],
    );
    assert_eq!(code, 2);
    assert!(
        err.contains("--author belongs to recorded operations"),
        "{err}"
    );
    let (code, _, err) = run(
        dir,
        &["add", "letter.docx", "--plan", "plan.json", "--consume"],
    );
    assert_eq!(code, 0, "{err}");
    assert!(!dir.join("plan.json").exists());
    assert!(dir.join("letter_modified.docx").exists());
    assert!(
        !dir.join("letter.edit").exists(),
        "only --out-dir writes the bundle"
    );
}

/// #933: apply consumes a plan only — a plain apply records nothing into
/// the plan and deletes none of it.
#[test]
fn a_plain_apply_leaves_the_plan_as_it_was() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let held = std::fs::read(dir.join("plan.json")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("out.docx").is_file(), "the edit was written");
    assert!(dir.join("plan.json").is_file(), "the plan still exists");
    assert_eq!(
        std::fs::read(dir.join("plan.json")).unwrap(),
        held,
        "apply leaves the plan byte-identical"
    );
    let (_, view, _) = run(dir, &["read", "out.docx"]);
    assert!(view.contains("{~~his or her~>their~~}"), "{view}");
}

/// zcode review 06b F3, F4, F6, F8, F9.
#[test]
fn render_flags_explicit_defaults_links_and_overwritten_plans_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Sections 1(g), 2(e), 3 survive.");
    let record = |extra: &[&str]| {
        let mut args = vec![
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p2",
            "--content",
            "New.",
        ];
        args.extend_from_slice(extra);
        run(dir, &args)
    };
    for flag in [&["--dpi", "300"][..], &["--revisions", "word"]] {
        let (code, _, err) = record(flag);
        assert_eq!(code, 2, "{flag:?}");
        assert!(
            err.contains(&format!(
                "{} takes effect when the plan is applied",
                flag[0]
            )),
            "{err}"
        );
    }
    assert_eq!(record(&[]).0, 0);
    // The default author spelled out is still a choice the plan names.
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "--author",
            "Jubarte",
        ],
    );
    assert_eq!(code, 2);
    assert!(
        err.contains("--author belongs to recorded operations"),
        "{err}"
    );
    // add without -p names -p, not --plan.
    let (code, _, err) = run(dir, &["add", "letter.docx"]);
    assert_eq!(code, 2);
    assert!(err.contains("--location <WHERE>"), "{err}");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("plan.json", dir.join("link.json")).unwrap();
        let (code, _, err) = run(
            dir,
            &["edit", "letter.docx", "--plan", "link.json", "--consume"],
        );
        assert_eq!(code, 2);
        assert!(
            err.contains("a symbolic link; pass the file it names"),
            "{err}"
        );
        // Recording through the link grows the file it names.
        let (code, _, err) = run(
            dir,
            &[
                "edit",
                "letter.docx",
                "--plan",
                "link.json",
                "-p",
                "p0",
                "--delete",
            ],
        );
        assert_eq!(code, 0, "{err}");
        assert!(
            dir.join("link.json")
                .symlink_metadata()
                .unwrap()
                .is_symlink()
        );
        assert_eq!(plan(dir)["operations"].as_array().unwrap().len(), 2);
    }
    // V2-05: a bundle member that names the plan is an input alias; the
    // publication is refused before anything is written, so --consume has
    // nothing to delete and the plan survives untouched.
    std::fs::create_dir(dir.join("out")).unwrap();
    std::fs::copy(dir.join("plan.json"), dir.join("out/report.jsonl")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "out/report.jsonl",
            "--out-dir",
            "out",
            "--force",
            "--consume",
        ],
    );
    assert_eq!(code, 2);
    assert!(err.contains("OUTPUT_ALIAS"), "{err}");
    assert!(!dir.join("out/redline.docx").exists());
    assert_eq!(
        std::fs::read(dir.join("out/report.jsonl")).unwrap(),
        std::fs::read(dir.join("plan.json")).unwrap()
    );
}

/// D16: `--plan` without a value is `<FILE's dir>/<stem>_plan.json`, for
/// recording, appending and applying; a Word document's name as its value
/// is FILE, swallowed.
#[test]
fn plan_without_a_value_is_file_plan_json() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    std::fs::create_dir(dir.join("d")).unwrap();
    letter(dir, "d/a.docx", "Sections 1(g), 2(e), 3 survive.");
    let (code, out, err) = run(
        dir,
        &["edit", "d/a.docx", "--plan", "-p", "p1", "--content", "X."],
    );
    assert_eq!(code, 0, "{err}");
    let plan = Path::new("d").join("a_plan.json");
    assert!(dir.join(&plan).is_file(), "{out}");
    assert!(
        out.trim_end()
            .ends_with("apply with: jubarte edit d/a.docx --plan"),
        "the apply line spells --plan bare: {out}"
    );
    assert!(out.contains(&format!(" in {}: ", plan.display())), "{out}");
    let (code, out, err) = run(
        dir,
        &["add", "d/a.docx", "--plan", "-p", "p2", "--content", "Y."],
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains(": 2 operations, "), "appended: {out}");
    let (code, _, err) = run(dir, &["edit", "d/a.docx", "--plan", "--consume", "-q"]);
    assert_eq!(code, 0, "{err}");
    assert!(!dir.join(&plan).exists(), "consumed");
    assert!(dir.join("d/a_modified.docx").is_file());

    // A named plan still works, both spellings, and its apply line names it.
    for named in [&["--plan", "x.json"][..], &["--plan=x.json"][..]] {
        let mut args = vec!["edit", "d/a.docx"];
        args.extend(named);
        args.extend(["-p", "p1", "--content", "Z."]);
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 0, "{named:?}: {err}");
        assert!(out.trim_end().ends_with("--plan x.json"), "{out}");
        std::fs::remove_file(dir.join("x.json")).unwrap();
    }

    // The value took FILE: say so, whichever way it is spelled.
    for swallowed in [
        &["edit", "--plan", "d/a.docx", "-p", "p1", "--content", "X."][..],
        &["add", "--plan", "d/a.docx", "-p", "p2", "--content", "X."][..],
        &["edit", "d/a.docx", "--plan=b.docx"][..],
    ] {
        let (code, _, err) = run(dir, swallowed);
        assert_eq!(code, 2, "{swallowed:?}: {err}");
        assert!(
            err.contains(
                "names a Word document; put FILE before --plan, or write --plan=PLAN.json"
            ),
            "{swallowed:?}: {err}"
        );
    }
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "--out-dir",
            "d/a.docx",
            "-p",
            "p1",
            "--content",
            "X.",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("put FILE before --out-dir, or write --out-dir=DIR"),
        "{err}"
    );
    // zcode-13c F2/F5, pi-13c F1: the `=` spelling without FILE, and a
    // name that is all extension.
    for swallowed in [
        &["edit", "--plan=d/a.docx", "-p", "p1", "--content", "X."][..],
        &[
            "edit",
            "d/a.docx",
            "--plan=.docx",
            "-p",
            "p1",
            "--content",
            "X.",
        ][..],
        &["add", "--out-dir=d/a.docx", "-p", "p2", "--content", "X."][..],
    ] {
        let (code, _, err) = run(dir, swallowed);
        assert_eq!(code, 2, "{swallowed:?}: {err}");
        assert!(
            err.contains("names a Word document"),
            "{swallowed:?}: {err}"
        );
    }
    // zcode-13c F1/F4: help, an unknown flag and `--` keep clap's own answer.
    let (code, out, _) = run(dir, &["edit", "--plan", "d/a.docx", "--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("Usage: jubarte edit"), "{out}");
    for other in [
        &["edit", "--plan", "d/a.docx", "--bogus"][..],
        &["edit", "--", "--plan", "d/a.docx"][..],
    ] {
        let (code, _, err) = run(dir, other);
        assert_eq!(code, 2, "{other:?}: {err}");
        assert!(!err.contains("names a Word document"), "{other:?}: {err}");
    }
    // zcode-13c F3: a bare --plan while recording reads bare in the refusal.
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "d/a.docx",
            "--plan",
            "--out-dir",
            "o",
            "-p",
            "p1",
            "--content",
            "X.",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("(jubarte edit FILE --plan), and is not recorded"),
        "{err}"
    );
}

/// #568: the usage line names `jubarte`, not the binary's own file name:
/// on windows-latest the help said `jubarte.exe edit` and broke the check
/// above. Pinned from another name for the program: a hard link beside
/// the binary, which every OS takes without privileges. #1082: the
/// system temp can sit on another volume, which windows-latest cannot
/// link across, so the link is made in the binary's own directory, and
/// a copy (the help reads argv[0], not the file's identity) is the
/// fallback where even that link cannot be made.
#[test]
fn the_help_names_jubarte_even_when_the_binary_is_jubarte_exe() {
    let dir = tempfile::tempdir_in(Path::new(BIN).parent().unwrap()).unwrap();
    let exe = dir.path().join("jubarte.exe");
    if let Err(err) = std::fs::hard_link(BIN, &exe) {
        assert_eq!(err.kind(), std::io::ErrorKind::CrossesDevices, "{err}");
        std::fs::copy(BIN, &exe).unwrap();
    }
    let out = Command::new(&exe)
        .args(["edit", "--help"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("Usage: jubarte edit"), "{help}");
}

#[test]
fn v1_plan_and_v2_schema_do_not_change_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let file = letter(dir, "letter.docx", "Last.");
    let original = std::fs::read(file).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "v1.json",
            "-p",
            "p0",
            "--content",
            "Legacy.",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let v1 = std::fs::read(dir.join("v1.json")).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "Typed.",
            "--datetime",
            "2026-10-17T07:00:00Z",
            "--save-plan",
            "v2.json",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let v2 = std::fs::read(dir.join("v2.json")).unwrap();
    let p = jubarte::edit::v2::PlanV2::from_json(std::str::from_utf8(&v2).unwrap()).unwrap();
    assert_eq!(p.author, "Jubarte");
    assert_eq!(p.source.sha256, source_sha256(&original));
    assert!(jubarte::edit::EditPlan::from_json(std::str::from_utf8(&v2).unwrap()).is_err());
    assert!(jubarte::edit::v2::PlanV2::from_json(std::str::from_utf8(&v1).unwrap()).is_err());
    assert_eq!(std::fs::read(dir.join("letter.docx")).unwrap(), original);
    for version in ["v1", "v2"] {
        let (code, out, err) = run(
            dir,
            &[
                "edit",
                "apply",
                "letter.docx",
                "--plan",
                &format!("{version}.json"),
                "-o",
                &format!("{version}.docx"),
                "--json",
            ],
        );
        assert_eq!(code, 0, "{out} {err}");
    }
    assert_eq!(std::fs::read(dir.join("v1.json")).unwrap(), v1);
    assert_eq!(std::fs::read(dir.join("v2.json")).unwrap(), v2);
}

#[test]
fn typed_actions_and_saved_plans_prepare_identical_documents() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "fee fee");
    std::fs::write(dir.join("exact.txt"), "new\u{a0}text\t\n\n").unwrap();
    for (i, action) in [
        vec![
            "replace",
            "--at",
            "p2",
            "--find",
            "fee",
            "--occurrence",
            "2",
            "--text-file",
            "exact.txt",
        ],
        vec!["replace", "--find", "Heading", "--text", "Title"],
        vec!["rewrite", "--at", "p0", "--text-file", "exact.txt"],
        vec![
            "insert",
            "--kind",
            "text",
            "--at",
            "p0",
            "--after-text",
            "Heading",
            "--text-file",
            "exact.txt",
        ],
        vec![
            "insert",
            "--kind",
            "paragraph",
            "--at",
            "p0",
            "--position",
            "after",
            "--text-file",
            "exact.txt",
        ],
        vec!["delete", "--at", "p2", "--find", "fee", "--occurrence", "2"],
        vec!["delete", "--at", "p0", "--whole-block"],
    ]
    .iter()
    .enumerate()
    {
        let inline = format!("inline-{i}.docx");
        let applied = format!("applied-{i}.docx");
        let saved = format!("plan-{i}.json");
        let args = [
            vec!["edit"],
            action.clone(),
            vec![
                "letter.docx",
                "--track",
                "off",
                "--datetime",
                "2026-10-17T07:00:00Z",
            ],
        ]
        .concat();
        for extra in [vec!["-o", &inline], vec!["--save-plan", &saved]] {
            let (code, out, err) = run(dir, &[args.clone(), extra].concat());
            assert_eq!(code, 0, "{action:?}: {out} {err}");
        }
        let bytes = std::fs::read(dir.join(&saved)).unwrap();
        let plan: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        if i == 0 || (2..=4).contains(&i) {
            assert_eq!(plan["operations"][0]["text"], "new\u{a0}text\t\n\n");
        }
        if i == 6 {
            assert_eq!(plan["operations"][0]["kind"], "delete_paragraph");
        }
        let (code, out, err) = run(
            dir,
            &[
                "edit",
                "apply",
                "letter.docx",
                "--plan",
                &saved,
                "-o",
                &applied,
            ],
        );
        assert_eq!(code, 0, "{out} {err}");
        assert_eq!(std::fs::read(dir.join(&saved)).unwrap(), bytes);
        let output = std::fs::read(dir.join(&inline)).unwrap();
        common::validity::assert_word_valid_package(&output);
        let paragraphs = jubarte::inspect::paragraphs(&output).unwrap();
        let (paragraph, expected) = match i {
            0 => (2, "fee new\u{a0}text\t\n\n"),
            1 => (0, "Title"),
            2 => (0, "new\u{a0}text\t\n\n"),
            3 => (0, "Headingnew\u{a0}text\t\n\n"),
            4 => (1, "new\u{a0}text\t\n\n"),
            5 => (2, "fee "),
            6 => (0, "The individual signs in his or her individual capacity."),
            _ => unreachable!(),
        };
        assert_eq!(paragraphs[paragraph].text, expected, "{action:?}");
        assert_eq!(
            std::fs::read(dir.join(&inline)).unwrap(),
            std::fs::read(dir.join(&applied)).unwrap(),
            "{action:?}"
        );
    }
}

#[test]
fn new_apply_requires_legacy_binding_and_never_waives_v2_binding() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let file = letter(dir, "letter.docx", "Last.");
    let digest = source_sha256(&std::fs::read(file).unwrap());
    let legacy = serde_json::json!({"schema_version": 1, "author": "Recorded", "operations": []});
    std::fs::write(dir.join("legacy.json"), legacy.to_string()).unwrap();
    let base = vec![
        "edit",
        "apply",
        "letter.docx",
        "--plan",
        "legacy.json",
        "--dry-run",
        "--json",
    ];
    let (code, out, err) = run(dir, &base);
    assert_eq!(code, 3, "{out} {err}");
    assert!(out.contains("UNBOUND_LEGACY_PLAN"), "{out}");
    for (extra, binding) in [
        (vec!["--allow-unbound-plan"], "unbound_legacy"),
        (vec!["--expect-sha256", &digest], "bound"),
    ] {
        let (code, out, err) = run(dir, &[base.clone(), extra].concat());
        assert_eq!(code, 0, "{out} {err}");
        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(receipt["effective_options"]["source_binding"], binding);
    }
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "x",
            "--save-plan",
            "v2.json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    for extra in [
        vec!["--allow-unbound-plan"],
        vec!["--expect-sha256", &"0".repeat(64)],
    ] {
        let (code, out, err) = run(
            dir,
            &[
                vec![
                    "edit",
                    "apply",
                    "letter.docx",
                    "--plan",
                    "v2.json",
                    "--dry-run",
                    "--json",
                ],
                extra,
            ]
            .concat(),
        );
        assert_ne!(code, 0, "{out} {err}");
    }
    let mut bound = legacy;
    bound["source_sha256"] = "0".repeat(64).into();
    std::fs::write(dir.join("legacy.json"), bound.to_string()).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            base,
            vec!["--allow-unbound-plan", "--expect-sha256", &digest],
        ]
        .concat(),
    );
    assert_eq!(code, 3, "{out} {err}");
}

#[test]
fn typed_routes_protect_plan_payload_and_source_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    std::fs::write(dir.join("payload.txt"), "new").unwrap();
    let inline = vec![
        "edit",
        "rewrite",
        "letter.docx",
        "--at",
        "p0",
        "--text-file",
        "payload.txt",
    ];
    for path in ["letter.docx", "./letter.docx", "payload.txt"] {
        let before = std::fs::read(dir.join(path)).unwrap();
        for output in ["-o", "--save-plan"] {
            let (code, out, err) = run(
                dir,
                &[inline.clone(), vec![output, path, "--force", "--json"]].concat(),
            );
            assert_ne!(code, 0, "{out} {err}");
            assert!(
                out.contains("OUTPUT_ALIAS") || err.contains("OUTPUT_ALIAS"),
                "{out} {err}"
            );
            assert_eq!(std::fs::read(dir.join(path)).unwrap(), before);
        }
    }
    let (code, out, err) = run(dir, &[inline, vec!["--save-plan", "plan.json"]].concat());
    assert_eq!(code, 0, "{out} {err}");
    let before = std::fs::read(dir.join("plan.json")).unwrap();
    std::fs::hard_link(dir.join("plan.json"), dir.join("hard.json")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("plan.json", dir.join("sym.json")).unwrap();
    for path in ["plan.json", "./plan.json", "hard.json"]
        .into_iter()
        .chain(if cfg!(unix) { Some("sym.json") } else { None })
    {
        let (code, out, err) = run(
            dir,
            &[
                "edit",
                "apply",
                "letter.docx",
                "--plan",
                "plan.json",
                "-o",
                path,
                "--force",
                "--json",
            ],
        );
        assert_ne!(code, 0, "{out} {err}");
        assert!(
            out.contains("OUTPUT_ALIAS") || err.contains("OUTPUT_ALIAS"),
            "{out} {err}"
        );
    }
    assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), before);
}

fn plan_with_moved_source(dir: &Path) -> Vec<u8> {
    let file = letter(dir, "a.docx", "Last.");
    let before = std::fs::read(&file).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "rewrite",
            "a.docx",
            "--at",
            "p0",
            "--text",
            "Moved heading.",
            "--track",
            "off",
            "--datetime",
            "2026-10-17T12:00:00Z",
            "--save-plan",
            "plan.json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(plan(dir)["source"]["sha256"], source_sha256(&before));
    std::fs::create_dir(dir.join("moved")).unwrap();
    std::fs::copy(&file, dir.join("moved/b.docx")).unwrap();
    std::fs::remove_file(file).unwrap();
    before
}

#[test]
fn moved_source_applies_its_plan_and_still_refuses_output_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = plan_with_moved_source(dir);
    let plan_before = std::fs::read(dir.join("plan.json")).unwrap();
    let base = vec![
        "edit",
        "apply",
        "moved/b.docx",
        "--plan",
        "plan.json",
        "--json",
    ];
    for dry_run in [true, false] {
        let mut args = base.clone();
        if dry_run {
            args.push("--dry-run");
        } else {
            args.extend(["-o", "out.docx"]);
        }
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 0, "{out} {err}");
        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(receipt["status"], "ok");
        assert_eq!(receipt["source"]["sha256"], source_sha256(&before));
        // #568: the receipt's path is the canonical one an agent reads
        // back — compared as strings in the convention's spelling, since
        // `Path` equality on Windows would equate a `//?/C:/…` string with
        // the verbatim path while an agent could not use it as written.
        let shown = receipt["source"]["path"].as_str().unwrap();
        assert_eq!(
            shown,
            slash_path(&dir.join("moved/b.docx").canonicalize().unwrap())
        );
        // #1222: a verbatim prefix spelled out is `//?/` — not a path.
        assert!(!shown.contains("//?"), "{shown}");
        assert_eq!(receipt["effective_options"]["source_binding"], "bound");
        assert_eq!(receipt["data"]["counts"]["body_operations_applied"], 1);
        if dry_run {
            assert_eq!(receipt["data"]["phase"], "validated");
            assert_eq!(receipt["data"]["effect"], "not_applied");
            assert_eq!(receipt["data"]["predicted_effect"], "changed");
            assert!(!dir.join("out.docx").exists());
            assert!(receipt.get("artifacts").is_none());
        } else {
            assert_eq!(receipt["data"]["phase"], "committed");
            assert_eq!(receipt["data"]["effect"], "changed");
            let output = std::fs::read(dir.join("out.docx")).unwrap();
            common::validity::assert_word_valid_package(&output);
            assert_eq!(
                jubarte::inspect::paragraphs(&output).unwrap()[0].text,
                "Moved heading."
            );
        }
    }
    std::fs::hard_link(dir.join("moved/b.docx"), dir.join("moved/source-link.docx")).unwrap();
    std::fs::hard_link(dir.join("plan.json"), dir.join("moved/plan-link.json")).unwrap();
    for path in [
        "moved/b.docx",
        "moved/./b.docx",
        "moved/source-link.docx",
        "plan.json",
        "./plan.json",
        "moved/plan-link.json",
    ] {
        let args = [base.clone(), vec!["-o", path, "--force"]].concat();
        let (code, out, err) = run(dir, &args);
        assert_ne!(code, 0, "{path}: {out} {err}");
        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(receipt["error"]["code"], "OUTPUT_ALIAS");
        assert_eq!(std::fs::read(dir.join("moved/b.docx")).unwrap(), before);
        assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), plan_before);
    }
}

#[test]
fn moved_source_with_one_changed_byte_refuses_stale_source() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let mut changed = plan_with_moved_source(dir);
    let plan_before = std::fs::read(dir.join("plan.json")).unwrap();
    // Change a ZIP timestamp without changing the document text.
    changed[10] ^= 1;
    assert_eq!(
        jubarte::inspect::paragraphs(&changed).unwrap()[0].text,
        "Heading"
    );
    std::fs::write(dir.join("moved/b.docx"), &changed).unwrap();
    for dry_run in [true, false] {
        let mut args = vec![
            "edit",
            "apply",
            "moved/b.docx",
            "--plan",
            "plan.json",
            "--json",
        ];
        if dry_run {
            args.push("--dry-run");
        } else {
            args.extend(["-o", "out.docx"]);
        }
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 3, "{out} {err}");
        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(receipt["status"], "refused");
        assert_eq!(receipt["error"]["code"], "STALE_SOURCE");
        assert!(!dir.join("out.docx").exists());
        assert_eq!(std::fs::read(dir.join("moved/b.docx")).unwrap(), changed);
        assert_eq!(std::fs::read(dir.join("plan.json")).unwrap(), plan_before);
    }
}

#[test]
fn action_named_files_execute_the_unchanged_legacy_route() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    for name in ["replace", "rewrite", "insert", "delete", "apply"] {
        letter(dir, name, "Last.");
        let (code, out, err) = run(
            dir,
            &[
                "edit",
                name,
                "-p",
                "p0",
                "--content",
                "Legacy.",
                "-o",
                &format!("{name}-out.docx"),
            ],
        );
        assert_eq!(code, 0, "{name}: {out} {err}");
        assert!(out.contains("Legacy."), "{out}");
    }
}

#[test]
fn saved_plan_freezes_policy_targets_and_time_without_delivery_options() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "Heading",
            "--text",
            "Title",
            "--author",
            "Task Author",
            "--existing-revisions",
            "reject",
            "--track",
            "off",
            "--save-plan",
            "plan.json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value = plan(dir);
    assert_eq!(value["author"], "Task Author");
    assert_eq!(value["policy"]["track"], "off");
    assert_eq!(value["policy"]["existing_revisions"], "reject");
    assert_eq!(value["operations"][0]["target"]["short_id"], "p0");
    assert!(value["date"].as_str().is_some_and(|s| s.ends_with('Z')));
    for key in [
        "output",
        "out_dir",
        "report",
        "emit",
        "expect",
        "expect_sha256",
        "dry_run",
    ] {
        assert!(value.get(key).is_none(), "{value}");
    }
    let digest = value["source"]["sha256"].as_str().unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "plan.json",
            "--expect-sha256",
            digest,
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let result: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(result["data"]["phase"], "validated");
    assert_eq!(result["data"]["effect"], "not_applied");
    assert_eq!(result["data"]["predicted_effect"], "changed");
    assert!(!dir.join("letter_modified.docx").exists());
    assert_eq!(std::fs::read_dir(dir).unwrap().count(), 2);
    let mut missing_hash = value;
    missing_hash["source"]
        .as_object_mut()
        .unwrap()
        .remove("sha256");
    std::fs::write(dir.join("plan.json"), missing_hash.to_string()).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "plan.json",
            "--allow-unbound-plan",
            "--dry-run",
            "--json",
        ],
    );
    assert_ne!(code, 0, "{out} {err}");
}

#[test]
fn typed_delivery_reports_and_bundles_are_explicit_and_inputs_remain_immutable() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let base = vec![
        "edit",
        "rewrite",
        "letter.docx",
        "--at",
        "p0",
        "--text",
        "Changed.",
    ];
    let (code, out, err) = run(
        dir,
        &[base.clone(), vec!["--out-dir", "bundle", "--jsonl"]].concat(),
    );
    assert_eq!(code, 0, "{out} {err}");
    let parsed = jubarte::result::read_jsonl_stream(&out).unwrap();
    assert_eq!(parsed.terminal.status, jubarte::result::Status::Ok);
    assert!(
        parsed
            .terminal
            .artifacts
            .iter()
            .any(|a| a.media_role == jubarte::result::MediaRole::Report && a.committed)
    );
    for file in [
        "document.docx",
        "changes.diff",
        "report.json",
        "manifest.json",
    ] {
        assert!(dir.join("bundle").join(file).is_file(), "{file}");
    }
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("bundle/report.json")).unwrap()).unwrap();
    assert_eq!(report["data"]["phase"], "committed");
    assert_eq!(report["data"]["verification"]["integrity"], "passed");
    let (code, out, err) = run(
        dir,
        &[
            base.clone(),
            vec![
                "-o",
                "single.docx",
                "--emit",
                "document,patch,report",
                "--json",
            ],
        ]
        .concat(),
    );
    assert_eq!(code, 0, "{out} {err}");
    for file in ["single.docx", "single.diff", "single.report.json"] {
        assert!(dir.join(file).is_file(), "{file}");
    }
    let (code, out, err) = run(
        dir,
        &[
            base.clone(),
            vec![
                "-o",
                "report-target.docx",
                "--report",
                "letter.docx",
                "--force",
                "--json",
            ],
        ]
        .concat(),
    );
    assert_ne!(code, 0, "{out} {err}");
    assert!(out.contains("OUTPUT_ALIAS"), "{out}");
    let (code, out, err) = run(
        dir,
        &[
            base.clone(),
            vec!["--out-dir", "bundle", "--force", "--json"],
        ]
        .concat(),
    );
    assert_ne!(code, 0, "{out} {err}");
    assert!(out.contains("OUTPUT_EXISTS"), "{out}");
    let (code, out, err) = run(dir, &[base, vec!["-o", "single.docx", "--json"]].concat());
    assert_ne!(code, 0, "{out} {err}");
}

#[test]
fn typed_invalid_inputs_and_require_change_never_publish() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    std::fs::write(dir.join("bad.txt"), [255, 254]).unwrap();
    for (args, code) in [
        (
            vec![
                "rewrite",
                "letter.docx",
                "--at",
                "p0",
                "--text-file",
                "bad.txt",
            ],
            "INVALID_UTF8",
        ),
        (
            vec![
                "rewrite",
                "letter.docx",
                "--at",
                "p0",
                "--text-file",
                "missing.txt",
            ],
            "IO_ERROR",
        ),
        (
            vec!["rewrite", "missing.docx", "--at", "p0", "--text", "x"],
            "IO_ERROR",
        ),
        (
            vec![
                "rewrite",
                "letter.docx",
                "--at",
                "p0",
                "--text",
                "Heading",
                "--require-change",
            ],
            "NO_CHANGES_REQUESTED",
        ),
    ] {
        let (status, out, err) = run(
            dir,
            &[vec!["edit"], args, vec!["-o", "absent.docx", "--json"]].concat(),
        );
        assert_ne!(status, 0, "{out} {err}");
        assert!(out.contains(code), "{out} {err}");
        assert!(!dir.join("absent.docx").exists());
    }
    for (name, json, code) in [
        ("bad.json", "{", "INVALID_PLAN"),
        (
            "unknown.json",
            r#"{"schema_version":99}"#,
            "UNSUPPORTED_SCHEMA",
        ),
    ] {
        std::fs::write(dir.join(name), json).unwrap();
        let (status, out, err) = run(
            dir,
            &[
                "edit",
                "apply",
                "letter.docx",
                "--plan",
                name,
                "--dry-run",
                "--json",
            ],
        );
        assert_ne!(status, 0, "{out} {err}");
        assert!(out.contains(code), "{out} {err}");
    }
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "Heading",
            "-o",
            "unchanged.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(
        std::fs::read(dir.join("letter.docx")).unwrap(),
        std::fs::read(dir.join("unchanged.docx")).unwrap()
    );
    assert!(out.contains(r#""effect":"unchanged""#), "{out}");
}

#[test]
fn typed_recognized_usage_errors_use_the_requested_result_protocol() {
    let dir = tempfile::tempdir().unwrap();
    for mode in ["--json", "--jsonl"] {
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "rewrite",
                "missing.docx",
                "--text",
                "x",
                "--dry-run",
                mode,
            ],
        );
        assert_eq!(code, 64, "{out} {err}");
        let receipt = if mode == "--json" {
            serde_json::from_str::<jubarte::result::Envelope>(&out).unwrap()
        } else {
            jubarte::result::read_jsonl_stream(&out).unwrap().terminal
        };
        assert_eq!(receipt.error.unwrap().code, "USAGE");
        assert!(err.is_empty(), "{err}");
    }
}

#[test]
fn review_01_usage_protocol_respects_hoisting_values_and_separator() {
    let dir = tempfile::tempdir().unwrap();
    for prefix in [vec![], vec!["-q"]] {
        for mode in ["--json", "--jsonl"] {
            let args = [
                prefix.clone(),
                vec![
                    "edit", "replace", "f.docx", "--find", "x", "--text", "y", mode,
                ],
            ]
            .concat();
            let (code, out, err) = run(dir.path(), &args);
            assert_eq!(code, 64, "{args:?}: {out} {err}");
            assert!(out.contains("USAGE"), "{out}");
            assert!(err.is_empty(), "{err}");
        }
    }
    for tail in [
        vec!["--", "--json"],
        vec!["--text", "--json"],
        vec!["--text-file", "--jsonl"],
    ] {
        let args = [vec!["edit", "replace", "f.docx", "--find", "x"], tail].concat();
        let (code, out, err) = run(dir.path(), &args);
        assert_eq!(code, 2, "{args:?}: {out} {err}");
        assert!(out.is_empty(), "{args:?}: {out}");
    }
}

#[test]
fn review_04_dry_run_conflicts_with_destinations() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    for destination in ["-o", "--out-dir"] {
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "rewrite",
                "letter.docx",
                "--at",
                "p0",
                "--text",
                "new",
                "--dry-run",
                destination,
                "unused",
                "--json",
            ],
        );
        assert_eq!(code, 64, "{out} {err}");
        assert!(out.contains("USAGE"), "{out}");
        assert!(!dir.path().join("unused").exists());
    }
}

#[test]
fn review_05_anchor_codes_agree_across_scopes() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    for scope in [vec![], vec!["--at", "p2"]] {
        for (selection, expected) in [
            (vec!["--occurrence", "2"], "AMBIGUOUS_ANCHOR"),
            (
                vec!["--all-matches", "--expect-count", "1"],
                "ANCHOR_NOT_FOUND",
            ),
        ] {
            let find = if selection[0] == "--all-matches" {
                "absent"
            } else {
                "fee"
            };
            let args = [
                vec![
                    "edit",
                    "replace",
                    "letter.docx",
                    "--find",
                    find,
                    "--text",
                    "x",
                    "--dry-run",
                    "--json",
                ],
                scope.clone(),
                selection,
            ]
            .concat();
            let (code, out, err) = run(dir.path(), &args);
            assert_ne!(code, 0, "{out} {err}");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&out).unwrap()["error"]["code"],
                expected,
                "{out} {err}"
            );
        }
    }
}

#[test]
fn review_06_unknown_story_names_inventory() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--story",
            "header99",
            "--find",
            "fee",
            "--text",
            "x",
            "--dry-run",
            "--json",
        ],
    );
    assert_ne!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["error"]["code"], "ANCHOR_NOT_FOUND");
    assert_eq!(value["error"]["target"], "header99");
    assert!(
        value["error"]["message"].as_str().unwrap().contains("body"),
        "{out}"
    );
    assert!(out.contains("header99"), "{out}");
}

#[test]
fn review_07_effective_options_resolve_policy_and_delivery() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "new",
            "--author",
            "Alice",
            "--datetime",
            "2026-10-17T12:00:00Z",
            "--track",
            "off",
            "--existing-revisions",
            "keep",
            "--force",
            "-o",
            "out.docx",
            "--emit",
            "document",
            "--report",
            "receipt.json",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let options = &value["effective_options"];
    for (key, expected) in [
        ("author", serde_json::json!("Alice")),
        ("datetime", serde_json::json!("2026-10-17T12:00:00Z")),
        ("track", serde_json::json!("off")),
        ("existing_revisions", serde_json::json!("keep")),
        ("force", serde_json::json!(true)),
        ("dry_run", serde_json::json!(false)),
        ("emit", serde_json::json!(["document"])),
    ] {
        assert_eq!(options[key], expected, "{key}: {out}");
    }
    assert!(
        options["report"]
            .as_str()
            .unwrap()
            .ends_with("receipt.json")
    );
    assert!(options["source_binding"].is_string());
    assert!(options["request_sha256"].is_string());
}

#[cfg(unix)]
#[test]
fn review_08_broken_symlink_bundle_is_output_exists() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    std::os::unix::fs::symlink("missing", dir.path().join("bundle")).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "new",
            "--out-dir",
            "bundle",
            "--json",
        ],
    );
    assert_ne!(code, 0, "{out} {err}");
    assert!(out.contains("OUTPUT_EXISTS"), "{out} {err}");
}

#[test]
fn review_09_missing_legacy_action_filename_suggests_typed_apply() {
    let dir = tempfile::tempdir().unwrap();
    for action in ["apply", "replace"] {
        let (code, out, err) = run(dir.path(), &["edit", action, "--plan", "p.json"]);
        assert_ne!(code, 0, "{out} {err}");
        assert!(
            err.contains("jubarte edit apply FILE --plan PLAN"),
            "{out} {err}"
        );
        // Creating the legacy FILE must retain the legacy grammar.
        letter(dir.path(), action, "fee");
        let (code, out, err) = run(dir.path(), &["edit", action, "--plan", "p.json"]);
        assert_ne!(code, 0, "{out} {err}");
        assert!(
            !err.contains("jubarte edit apply FILE --plan PLAN"),
            "{out} {err}"
        );
    }
}

#[test]
fn review_10_text_file_rejects_cr_and_leading_bom_with_offsets() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    for (payload, offset, character) in [("é\r\nb", "1", "U+000D"), ("\u{feff}text", "0", "U+FEFF")]
    {
        std::fs::write(dir.path().join("payload.txt"), payload).unwrap();
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "rewrite",
                "letter.docx",
                "--at",
                "p0",
                "--text-file",
                "payload.txt",
                "--track",
                "off",
                "-o",
                "out.docx",
                "--json",
            ],
        );
        assert_ne!(code, 0, "{out} {err}");
        assert!(out.contains("INVALID_EDIT"), "{out} {err}");
        assert!(
            out.contains(character) && out.contains(&format!("character offset {offset}")),
            "{out} {err}"
        );
        if character == "U+000D" {
            assert!(out.contains("LF"), "{out}");
        }
        assert!(!dir.path().join("out.docx").exists());
    }
    std::fs::write(dir.path().join("payload.txt"), "a\u{feff}b").unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text-file",
            "payload.txt",
            "--track",
            "off",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let paragraphs =
        jubarte::inspect::paragraphs(&std::fs::read(dir.path().join("out.docx")).unwrap()).unwrap();
    assert_eq!(paragraphs[0].text, "a\u{feff}b");
}

#[test]
fn review_02_receipt_discloses_computed_candidate_findings() {
    let dir = tempfile::tempdir().unwrap();
    let source = docx(
        &(para("fee")
            + r#"<w:p><w:r><w:t>note</w:t></w:r><w:r><w:commentReference w:id="0"/></w:r></w:p>"#),
    );
    std::fs::write(dir.path().join("letter.docx"), &source).unwrap();
    let baseline = jubarte::validate::validate(&source).unwrap();
    assert!(!baseline.is_empty());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "--track",
            "off",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let verification = &value["data"]["verification"];
    assert_eq!(
        verification["baseline_findings"],
        serde_json::to_value(&baseline).unwrap(),
        "{out}"
    );
    let candidate =
        jubarte::validate::validate(&std::fs::read(dir.path().join("out.docx")).unwrap()).unwrap();
    assert_eq!(
        verification["unresolved_findings"],
        serde_json::to_value(&candidate).unwrap(),
        "{out}"
    );
    assert_eq!(verification["introduced_findings"], serde_json::json!([]));
    assert_eq!(verification["integrity"], "passed");
    assert_eq!(verification["operation_postconditions"], "passed");
}

#[test]
fn review_10_inline_hash_preconditions_cover_all_actions() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let source = std::fs::read(dir.path().join("letter.docx")).unwrap();
    let hash = source_sha256(&source);
    for action in [
        vec!["replace", "--find", "fee", "--text", "cost"],
        vec!["rewrite", "--at", "p0", "--text", "new"],
        vec![
            "insert", "--kind", "text", "--at", "p0", "--edge", "end", "--text", "new",
        ],
        vec!["delete", "--at", "p2", "--find", "fee"],
    ] {
        for (digest, expected) in [
            (hash.as_str(), 0),
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                3,
            ),
        ] {
            let args = [
                vec!["edit"],
                action.clone(),
                vec![
                    "letter.docx",
                    "--expect-sha256",
                    digest,
                    "--dry-run",
                    "--json",
                ],
            ]
            .concat();
            let (code, out, err) = run(dir.path(), &args);
            assert_eq!(code, expected, "{action:?}: {out} {err}");
            if expected != 0 {
                assert!(out.contains("STALE_SOURCE"), "{out}");
            }
        }
    }
    assert_eq!(
        std::fs::read(dir.path().join("letter.docx")).unwrap(),
        source
    );
}

#[test]
fn review_10_derived_patch_alias_refuses_before_publication() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    std::fs::write(dir.path().join("out.diff"), "cost").unwrap();
    let original = std::fs::read(dir.path().join("letter.docx")).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text-file",
            "out.diff",
            "-o",
            "out.docx",
            "--emit",
            "patch",
            "--force",
            "--json",
        ],
    );
    assert_ne!(code, 0, "{out} {err}");
    assert!(out.contains("OUTPUT_ALIAS"), "{out} {err}");
    assert!(!dir.path().join("out.docx").exists());
    assert_eq!(std::fs::read(dir.path().join("out.diff")).unwrap(), b"cost");
    assert_eq!(
        std::fs::read(dir.path().join("letter.docx")).unwrap(),
        original
    );
}

#[test]
fn review_10_unbound_v1_publishes_with_disclosed_waiver() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let plan = br#"{"schema_version":1,"author":"Legacy","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p2","find":"fee","replacement":"cost"}]}"#;
    std::fs::write(dir.path().join("v1.json"), plan).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "v1.json",
            "--allow-unbound-plan",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        value["effective_options"]["source_binding"],
        "unbound_legacy"
    );
    assert_eq!(value["data"]["phase"], "committed");
    assert_eq!(value["effective_options"]["author"], "Legacy");
    assert_eq!(value["effective_options"]["track"], "on");
    assert_eq!(std::fs::read(dir.path().join("v1.json")).unwrap(), plan);
    let paragraphs =
        jubarte::inspect::paragraphs(&std::fs::read(dir.path().join("out.docx")).unwrap()).unwrap();
    assert_eq!(paragraphs[2].text, "cost");
}

/// #1094: for a changed v1 typed apply, the receipt's document artifact
/// carries the published bytes' sha256, and the SHA the receipt cites
/// (in `effective_options.request_sha256` plus the artifact's own field)
/// agrees with what's actually written to disk. Before the fix the CLI
/// swapped the chosen primary to redline after `prepare` returned; the
/// library today owns the choice and names the redline bytes directly.
#[test]
fn review_typed_v1_apply_receipt_describes_the_redline_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "fee");
    let plan = br#"{"schema_version":1,"author":"Legacy","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p2","find":"fee","replacement":"cost"}]}"#;
    std::fs::write(dir.join("v1.json"), plan).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "v1.json",
            "--allow-unbound-plan",
            "-o",
            "out.docx",
            "--emit",
            "document,report",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let bytes = std::fs::read(dir.join("out.docx")).unwrap();
    let actual_sha = source_sha256(&bytes);
    // The receipt's document artifact sha256 is the sha256 of the
    // bytes actually written: not the source's, not the clean copy's.
    let documents: Vec<&serde_json::Value> = value["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["media_role"] == "document")
        .collect();
    assert_eq!(documents.len(), 1, "{value}");
    assert_eq!(documents[0]["sha256"], actual_sha, "{value}");
    assert_eq!(documents[0]["committed"], true, "{value}");
    // The published bytes are the redline: paragraph 2 reads "cost" in
    // accepted terms and the package carries a tracked-changes mark.
    let paragraphs = jubarte::inspect::paragraphs(&bytes).unwrap();
    assert_eq!(paragraphs[2].text, "cost");
    let pkg = jubarte::opc::PartFs::open(&bytes).unwrap();
    let document_xml = pkg
        .part_bytes("word/document.xml")
        .expect("word/document.xml in the published package");
    let document_xml = std::str::from_utf8(document_xml).unwrap();
    assert!(
        document_xml.contains("<w:ins ") || document_xml.contains("<w:del "),
        "v1's published primary must remain a tracked-changes redline; got: {document_xml}"
    );
}

/// The first stdout line of a typed edit, the human receipt.
fn receipt_line(out: &str) -> &str {
    out.lines().next().unwrap_or(out)
}

/// #1891: typed edits name who signed on the human receipt.
#[test]
fn typed_replace_receipt_names_the_given_author() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let line = receipt_line(&out);
    assert!(
        line.contains(r#"; 1 change by "GiantCo""#) && !line.contains("no -a given"),
        "{line}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: without `-a`, the receipt names the default and stderr warns.
#[test]
fn typed_replace_receipt_without_author_warns() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let author = typed_default_author();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let line = receipt_line(&out);
    assert!(
        line.contains(&format!(r#"; 1 change by "{author}": no -a given"#)),
        "{line}"
    );
    assert!(
        err.contains(&format!(
            r#"warning: no -a given; the new markup is signed "{author}""#
        )),
        "{err}"
    );
}

/// #1891: `-a` with the default name is explicit.
#[test]
fn typed_explicit_default_author_is_not_a_missing_author_warning() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let author = typed_default_author();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "--author",
            &author,
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let line = receipt_line(&out);
    assert!(
        line.contains(&format!(r#"; 1 change by "{author}""#)) && !line.contains("no -a given"),
        "{line}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: a comment on the replacement counts beside the change.
#[test]
fn typed_replace_receipt_counts_a_change_and_a_comment() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "--comment",
            "Cap it",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(
        receipt_line(&out).contains(r#"; 1 change, 1 comment by "GiantCo""#),
        "{out}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: insert and delete use the same receipt detail.
#[test]
fn typed_insert_and_delete_receipts_name_the_author() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "insert",
            "letter.docx",
            "--kind",
            "text",
            "--at",
            "p2",
            "--edge",
            "end",
            "--text",
            " more",
            "-a",
            "GiantCo",
            "-o",
            "inserted.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(
        receipt_line(&out).contains(r#"; 1 change by "GiantCo""#),
        "{out}"
    );
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "delete",
            "letter.docx",
            "--at",
            "p2",
            "--find",
            "fee",
            "-a",
            "GiantCo",
            "-o",
            "deleted.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(
        receipt_line(&out).contains(r#"; 1 change by "GiantCo""#),
        "{out}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: `--track off` publishes the clean copy, so the receipt names no
/// tracked change.
#[test]
fn typed_track_off_receipt_names_no_tracked_change() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "--track",
            "off",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let line = receipt_line(&out);
    // "changed" is the outcome word. The markup clause is what must be absent.
    assert!(!line.contains("change by"), "{line}");
    assert!(!line.contains("no -a given"), "{line}");
    assert!(!err.contains("warning: no -a given"), "{err}");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(
        std::fs::read(dir.path().join("out.docx")).unwrap(),
    ))
    .unwrap();
    let mut document = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("word/document.xml").unwrap(),
        &mut document,
    )
    .unwrap();
    assert!(!document.contains("<w:ins "), "{document}");
    assert!(document.contains("cost"), "{document}");
}

/// #1891: `--json` / `--jsonl` keep the receipt shape. Stdout is the machine
/// stream (proposal §12: structured output on stdout, diagnostics on stderr),
/// so the default-author warning stays on stderr and out of the JSON.
#[test]
fn typed_json_receipt_keeps_its_shape_and_warns_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let author = typed_default_author();
    let warning = format!(r#"warning: no -a given; the new markup is signed "{author}""#);
    for mode in ["--json", "--jsonl"] {
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "replace",
                "letter.docx",
                "--find",
                "fee",
                "--text",
                "cost",
                "-o",
                &format!("out{mode}.docx"),
                mode,
            ],
        );
        assert_eq!(code, 0, "{mode}: {out} {err}");
        assert!(!out.contains("warning:"), "{mode}: {out}");
        assert!(!out.contains("no -a given"), "{mode}: {out}");
        assert!(err.contains(&warning), "{mode}: {err}");
        let value = if mode == "--json" {
            serde_json::from_str::<serde_json::Value>(&out).unwrap()
        } else {
            let parsed = jubarte::result::read_jsonl_stream(&out).unwrap();
            serde_json::to_value(parsed.terminal).unwrap()
        };
        assert_eq!(
            value["effective_options"]["author"], author,
            "{mode}: {out}"
        );
        let keys: Vec<&str> = value["effective_options"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        for key in [
            "author",
            "datetime",
            "dry_run",
            "emit",
            "existing_revisions",
            "force",
            "report",
            "request_sha256",
            "source_binding",
            "track",
        ] {
            assert!(keys.contains(&key), "{mode} missing {key}: {keys:?}");
        }
        assert!(
            !keys.contains(&"author_given"),
            "{mode} grew a receipt field: {keys:?}"
        );
    }
}

/// #1891: `-q` drops the status warning, as it drops the human receipt.
/// `--json` still prints the envelope.
#[test]
fn typed_quiet_json_does_not_warn() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "-q",
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(
        serde_json::from_str::<serde_json::Value>(&out).is_ok(),
        "{out}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
    assert!(!out.contains("warning:"), "{out}");
}

/// #1891: recording a plan without `-a` says the default is what it signs.
#[test]
fn record_without_author_warns_that_the_plan_signs_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let author = typed_default_author();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(out.contains("recorded 1 operation"), "{out}");
    assert_eq!(plan(dir)["author"], author);
    assert!(
        err.contains(&format!(
            r#"warning: no -a given; the plan signs its markup "{author}""#
        )),
        "{err}"
    );
    // Applying the plan treats its author as explicit: the wrote line names
    // them, with no missing-author warning.
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let wrote = err
        .lines()
        .find(|line| line.starts_with("wrote "))
        .unwrap_or(&err);
    assert!(
        wrote.contains(&format!(r#"; 1 change by "{author}""#)) && !wrote.contains("no -a given"),
        "{err}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: `-a` on a recording is the plan's author, and the warning stays off.
/// `edit apply` then names that author without calling it a missing `-a`.
#[test]
fn record_with_author_does_not_warn_and_apply_names_them() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "-a",
            "GiantCo",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(plan(dir)["author"], "GiantCo");
    assert!(!err.contains("warning: no -a given"), "{err}");
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(
        receipt_line(&out).contains(r#"; 1 change by "GiantCo""#)
            && !receipt_line(&out).contains("no -a given"),
        "{out}"
    );
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: a dry run still chooses the default author, so it warns too.
#[test]
fn record_dry_run_without_author_warns() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let author = typed_default_author();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
            "--dry-run",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(out.contains("would record"), "{out}");
    assert!(!dir.join("plan.json").exists());
    assert!(
        err.contains(&format!(
            r#"warning: no -a given; the plan signs its markup "{author}""#
        )),
        "{err}"
    );
}

/// #1891: `-q` drops the recording warning with the summary.
#[test]
fn record_quiet_without_author_does_not_warn() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir, "letter.docx", "Last.");
    let (code, out, err) = run(
        dir,
        &[
            "-q",
            "edit",
            "letter.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--anchor",
            "his or her",
            "--content",
            "their",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert!(out.is_empty(), "{out}");
    assert!(!err.contains("warning: no -a given"), "{err}");
    assert_eq!(plan(dir)["author"], typed_default_author());
}

#[test]
fn review_10_all_actions_help_and_human_success_and_usage_exit_two() {
    let dir = tempfile::tempdir().unwrap();
    for action in ["replace", "rewrite", "insert", "delete", "apply"] {
        let (code, out, err) = run(dir.path(), &["edit", action, "--help"]);
        assert_eq!(code, 0, "{action}: {out} {err}");
        assert!(out.contains(&format!("jubarte edit {action}")), "{out}");
        assert!(!out.contains("__edit-v2"), "{out}");
        assert!(err.is_empty(), "{err}");
    }
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
        ],
    );
    assert_eq!(code, 2, "{out} {err}");
    assert!(out.is_empty());
    assert!(err.contains("writes require"), "{err}");
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let author = typed_default_author();
    let summary = format!(
        "edit replace: changed; wrote {}; 1 change by \"{author}\": no -a given\n",
        slash_path(&dir.path().join("out.docx").canonicalize().unwrap())
    );
    assert!(
        out.starts_with(&format!("{summary}SUCCESS:\n~~~~diff\n")),
        "{out}"
    );
    assert!(
        err.contains(&format!(
            r#"warning: no -a given; the new markup is signed "{author}""#
        )),
        "{err}"
    );
    assert!(
        out.contains("--- before letter.docx\n+++ after out.docx\n@@ "),
        "{out}"
    );
    assert!(
        out.contains("\n-fee\n")
            && out
                .lines()
                .any(|line| line.starts_with('+') && line.contains("{~~fee~>cost~~}")),
        "{out}"
    );
    assert!(out.ends_with("~~~~\n"), "{out}");
}

/// #1244: the summary path comes from `absolute()`, which the CLI calls
/// before publication. `canonicalize(path)` fails while the file is still
/// unwritten, so the fallback must canonicalize the parent and rejoin the
/// file name — never just `current_dir().join(path)`. A `./out.docx`
/// output pins the difference on every host: the old code lets the `./`
/// through to the summary, the new code resolves it.
#[test]
fn review_11_wrote_summary_canonicalizes_a_relative_output() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "replace",
            "letter.docx",
            "--find",
            "fee",
            "--text",
            "cost",
            "-o",
            "./out.docx",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let canonical = dir.path().canonicalize().unwrap().join("out.docx");
    let author = typed_default_author();
    let expected_summary = format!(
        "edit replace: changed; wrote {}; 1 change by \"{author}\": no -a given\n",
        slash_path(&canonical)
    );
    assert!(out.starts_with(&expected_summary), "{out}");
    // The user-facing `./out.docx` echo in the diff header (`+++ after
    // ./out.docx`) stays verbatim, but the agent-facing summary line on
    // stdout must not carry it: a `./` survives the old code's
    // `current_dir().join(path)` fallback because `Path::join` keeps
    // literal path components as written.
    let summary_line = out
        .lines()
        .next()
        .expect("summary line is the first line of stdout");
    assert!(
        !summary_line.contains("./"),
        "the `./` must not survive into the summary line: {summary_line}"
    );
}

#[test]
fn review_02_every_introduced_nonblocking_finding_is_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let source = docx(
        r#"<w:p><w:commentRangeStart w:id="9"/><w:r><w:t>remove</w:t></w:r></w:p><w:p><w:r><w:t>keep</w:t></w:r><w:commentRangeEnd w:id="9"/></w:p>"#,
    );
    std::fs::write(dir.path().join("letter.docx"), &source).unwrap();
    let baseline = jubarte::validate::validate(&source).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "delete",
            "letter.docx",
            "--at",
            "p0",
            "--whole-block",
            "--track",
            "off",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let candidate =
        jubarte::validate::validate(&std::fs::read(dir.path().join("out.docx")).unwrap()).unwrap();
    let introduced = jubarte::edit::introduced_findings(&baseline, &candidate);
    assert!(
        !introduced.is_empty(),
        "fixture must introduce a nonblocking finding: {baseline:?} -> {candidate:?}"
    );
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        value["data"]["verification"]["introduced_findings"],
        serde_json::to_value(&introduced).unwrap()
    );
    for finding in introduced {
        assert!(!finding.word_fatal);
        assert!(
            value["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w.as_str().unwrap().contains(&finding.code)),
            "{out}"
        );
    }
}

#[test]
fn review_10_leading_bom_text_file_is_an_encoding_mark() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    std::fs::write(dir.path().join("payload.txt"), "\u{feff}text").unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text-file",
            "payload.txt",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_ne!(code, 0, "{out} {err}");
    assert!(
        out.contains("INVALID_EDIT")
            && out.contains("U+FEFF")
            && out.contains("character offset 0"),
        "{out} {err}"
    );
    assert!(!dir.path().join("out.docx").exists());
}

#[test]
fn review_07_defaults_and_saved_plan_options_are_resolved() {
    let dir = tempfile::tempdir().unwrap();
    letter(dir.path(), "letter.docx", "fee");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "new",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let options = &value["effective_options"];
    assert_eq!(options["author"], "Jubarte");
    assert!(options["datetime"].as_str().unwrap().ends_with('Z'));
    assert_eq!(options["track"], "on");
    assert_eq!(options["existing_revisions"], "keep");
    assert_eq!(options["emit"], serde_json::json!(["document"]));
    assert_eq!(options["force"], false);
    assert_eq!(options["dry_run"], true);
    assert!(options["report"].is_null());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "rewrite",
            "letter.docx",
            "--at",
            "p0",
            "--text",
            "new",
            "--author",
            "Frozen",
            "--datetime",
            "2026-10-17T12:00:00Z",
            "--track",
            "off",
            "--save-plan",
            "v2.json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "letter.docx",
            "--plan",
            "v2.json",
            "--out-dir",
            "bundle",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    let options = &value["effective_options"];
    assert_eq!(options["author"], "Frozen");
    assert_eq!(options["datetime"], "2026-10-17T12:00:00Z");
    assert_eq!(options["track"], "off");
    assert_eq!(
        options["emit"],
        serde_json::json!(["document", "patch", "report"])
    );
    assert!(
        options["report"]
            .as_str()
            .unwrap()
            .ends_with("bundle/report.json")
    );
}

fn typed_v1_tracked_apply(source: &[u8], mut plan: serde_json::Value) -> Result<Vec<u8>, String> {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    plan["schema_version"] = serde_json::json!(1);
    plan["source_sha256"] = serde_json::json!(source_sha256(source));
    plan["author"] = serde_json::json!("Editor");
    plan["date"] = serde_json::json!("2026-10-17T12:00:00Z");
    std::fs::write(dir.join("source.docx"), source).unwrap();
    std::fs::write(dir.join("v1.json"), plan.to_string()).unwrap();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "apply",
            "source.docx",
            "--plan",
            "v1.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    if code != 0 {
        assert!(!dir.join("out.docx").exists());
        return Err(format!("exit {code}: {out} {err}"));
    }
    let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
    let bytes = std::fs::read(dir.join("out.docx")).unwrap();
    let artifacts: Vec<_> = receipt["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["media_role"] == "document")
        .collect();
    assert_eq!(artifacts.len(), 1, "{receipt}");
    assert_eq!(artifacts[0]["sha256"], source_sha256(&bytes));
    assert_eq!(artifacts[0]["committed"], true);
    assert!(!jubarte::changes::list_changes(&bytes).unwrap().is_empty());
    Ok(bytes)
}

#[test]
fn review_typed_v1_tracked_apply_refreshes_stale_fields() {
    let source = docx(
        &(para("One")
            + r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#
            + r#"<w:p><w:r><w:t xml:space="preserve">Pages: </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>9</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#),
    );
    let result = typed_v1_tracked_apply(
        &source,
        serde_json::json!({
            "update_fields": true,
            "operations": [{"kind":"replace", "paragraph":"p0", "find":"One", "replacement":"Uno"}],
        }),
    );
    assert!(
        result.is_ok(),
        "stale field refresh must succeed: {result:?}"
    );
    let accepted = jubarte::document_comparer::accept_revisions(&result.unwrap()).unwrap();
    let texts: Vec<_> = jubarte::inspect::paragraphs(&accepted)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts, ["Uno", "", "Pages: 2"]);
}

#[test]
fn review_typed_v1_tracked_paragraph_edits_publish_verified_bytes() {
    let revision = r#"<w:p><w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>Earlier insertion.</w:t></w:r></w:ins></w:p>"#;
    let mut failures = Vec::new();
    for keep in [false, true] {
        let source =
            docx(&(para("alpha beta") + &para("Kept.") + if keep { revision } else { "" }));
        for (name, operation, mut expected) in [
            (
                "insert",
                serde_json::json!({"kind":"insert_paragraph", "paragraph":"p0", "position":"after", "runs":[{"text":"Inserted."}]}),
                vec!["alpha beta", "Inserted.", "Kept."],
            ),
            (
                "delete",
                serde_json::json!({"kind":"delete_paragraph", "paragraph":"p0"}),
                vec!["Kept."],
            ),
            (
                "replace",
                serde_json::json!({"kind":"replace", "paragraph":"p0", "find":"alpha", "replacement":"ALPHA"}),
                vec!["ALPHA beta", "Kept."],
            ),
        ] {
            let mut plan = serde_json::json!({"operations": [operation]});
            if keep {
                plan["existing_revisions"] = serde_json::json!("keep");
                expected.push("Earlier insertion.");
            }
            match typed_v1_tracked_apply(&source, plan) {
                Ok(bytes) => {
                    let accepted = jubarte::document_comparer::accept_revisions(&bytes).unwrap();
                    let texts: Vec<_> = jubarte::inspect::paragraphs(&accepted)
                        .unwrap()
                        .into_iter()
                        .map(|p| p.text)
                        .collect();
                    assert_eq!(texts, expected, "{name}, keep={keep}");
                    if keep {
                        assert!(
                            jubarte::changes::list_changes(&bytes)
                                .unwrap()
                                .iter()
                                .any(|c| c.author.as_deref() == Some("Other"))
                        );
                    }
                }
                Err(error) => failures.push(format!("{name}, keep={keep}: {error}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[cfg(unix)]
#[test]
fn audit_d13_recording_never_opens_a_predictable_partial_link() {
    let dir = tempfile::tempdir().unwrap();
    let plan = dir.path().join("plan.json");
    let victim = dir.path().join("victim.json");
    std::fs::write(&victim, "keep").unwrap();
    std::os::unix::fs::symlink(&victim, jubarte::edit::record::partial_path(&plan)).unwrap();
    jubarte::edit::record::write_plan(&plan, "{}\n").unwrap();
    assert_eq!(std::fs::read_to_string(victim).unwrap(), "keep");
    assert!(!plan.symlink_metadata().unwrap().is_symlink());
}

#[test]
fn audit_d13_plan_precommit_error_preserves_io_kind() {
    let dir = tempfile::tempdir().unwrap();
    let error =
        jubarte::edit::record::write_plan(&dir.path().join("missing/plan.json"), "{}").unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    assert!(!dir.path().join("missing").exists());
}
