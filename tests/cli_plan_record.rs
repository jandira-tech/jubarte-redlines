// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit --plan` and `add --plan` with operation flags record into the plan
//! instead of editing; `--plan` alone applies it, and `--consume` deletes it
//! once applied.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::docx::{docx, para};
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
    assert_eq!(recorded["author"], "Redline");

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
    assert!(view.contains("{~~his or her~>their~~}"), "{view}");
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
        err.contains(r#"the plan's author is "Redline"; pass --author "Redline""#),
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
            "Redline",
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
/// above. Pinned from a hard link under another name, which every OS
/// takes without privileges.
#[test]
fn the_help_names_jubarte_even_when_the_binary_is_jubarte_exe() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("jubarte.exe");
    std::fs::hard_link(BIN, &exe).unwrap();
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
