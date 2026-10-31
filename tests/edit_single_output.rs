// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit` and `add` write one document, `FILE_modified.docx` beside FILE
//! (`-o` names another); `--out-dir` writes the review bundle instead.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn letter(dir: &Path) {
    let body = para("Heading") + &para("The fee is thirty dollars.") + &para("Last line.");
    std::fs::write(dir.join("a.docx"), docx(&body)).unwrap();
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

fn document_xml(path: &Path) -> String {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("word/document.xml").unwrap(), &mut xml)
        .unwrap();
    xml
}

/// The files in `dir` besides `a.docx`, sorted.
fn written(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n != "a.docx")
        .collect();
    names.sort();
    names
}

const EDIT: [&str; 8] = [
    "edit",
    "a.docx",
    "-p",
    "p1",
    "--content",
    "The fee is forty dollars.",
    "-a",
    "Jubarte",
];

/// #1894: these are process integration tests of the one-shot grammar.
fn repeated_fee(dir: &Path) {
    let text = "Préface with enough words to trim before the first fee is due; \
                many intervening words keep the two windows separate; \
                the second fee is waived with enough trailing words to trim.";
    std::fs::write(dir.join("a.docx"), docx(&(para(text) + &para("fee")))).unwrap();
}

#[test]
fn occurrence_1894_edit_selects_the_second_match_inside_its_group() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    repeated_fee(dir);
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p0",
            "--anchor",
            "fee",
            "--occurrence",
            "2",
            "--content",
            "cost",
            "-p",
            "p1",
            "--anchor",
            "fee",
            "--content",
            "charge",
            "--editing-mode",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    let bytes = std::fs::read(dir.join("a_modified.docx")).unwrap();
    let paragraphs = jubarte::inspect::paragraphs(&bytes).unwrap();
    assert!(paragraphs[0].text.contains("first fee is due"));
    assert!(paragraphs[0].text.contains("second cost is waived"));
    assert_eq!(paragraphs[1].text, "charge");
}

#[test]
fn occurrence_1894_add_comments_on_the_second_match() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    repeated_fee(dir);
    let (code, out, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p0",
            "--anchor",
            "fee",
            "--occurrence",
            "2",
            "--content",
            "Why waived?",
            "-a",
            "Jubarte",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    let bytes = std::fs::read(dir.join("a_modified.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&bytes).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].anchor_text, "fee");
    assert!(comments[0].before.ends_with("second "), "{:?}", comments[0]);
    let (_, view, _) = run(dir, &["read", "a_modified.docx"]);
    assert!(view.contains("first fee is due"), "{view}");
    assert!(view.contains("second {==fee==}"), "{view}");
}

#[test]
fn occurrence_1894_out_of_range_names_the_available_range() {
    for verb in ["edit", "add"] {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        repeated_fee(dir);
        let (code, out, err) = run(
            dir,
            &[
                verb,
                "a.docx",
                "-p",
                "p0",
                "--anchor",
                "fee",
                "--occurrence",
                "3",
                "--content",
                "cost",
            ],
        );
        assert_eq!(code, 3, "{out}{err}");
        assert!(err.contains("AMBIGUOUS_ANCHOR"), "{err}");
        assert!(
            err.contains("occurrence 3 is outside") && err.contains("1..=2"),
            "{err}"
        );
        assert!(written(dir).is_empty());
    }
}

#[test]
fn occurrence_1894_ambiguity_uses_flags_and_shows_each_context_window() {
    for verb in ["edit", "add"] {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        repeated_fee(dir);
        let (code, out, err) = run(
            dir,
            &[
                verb,
                "a.docx",
                "-p",
                "p0",
                "--anchor",
                "fee",
                "--content",
                "cost",
            ],
        );
        assert_eq!(code, 3, "{out}{err}");
        assert!(err.contains("set --occurrence 1..=2"), "{err}");
        assert!(!err.contains("set \"occurrence\""), "{err}");
        let lines: Vec<_> = err.lines().collect();
        assert!(
            lines.iter().any(|l| l.trim_start().starts_with("1: ")
                && l.contains("first fee is due")
                && l.contains('…')),
            "{err}"
        );
        assert!(
            lines.iter().any(|l| l.trim_start().starts_with("2: ")
                && l.contains("second fee is waived")
                && l.contains('…')),
            "{err}"
        );
        assert!(
            err.contains("lengthen --anchor") && err.contains("pass --occurrence"),
            "{err}"
        );
        assert!(written(dir).is_empty());
    }
}

#[test]
fn occurrence_1894_plan_keeps_1013_released_wording_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    repeated_fee(dir);
    std::fs::write(dir.join("plan.json"), r#"{"schema_version":1,"author":"A","operations":[{"kind":"replace","paragraph":"p0","find":"fee","replacement":"cost"}]}"#).unwrap();
    let (code, _, err) = run(dir, &["edit", "a.docx", "--plan", "plan.json"]);
    assert_eq!(code, 3, "{err}");
    assert_eq!(
        err,
        "error: plan refused: AMBIGUOUS_ANCHOR (op-1): \"fee\" occurs 2 times in the paragraph; set \"occurrence\" to 1..=2\n"
    );
}

#[test]
fn occurrence_1894_rejects_misplaced_duplicate_and_invalid_values() {
    for verb in ["edit", "add"] {
        for (flags, expected) in [
            (
                vec!["--occurrence", "2", "-p", "p0", "--anchor", "fee"],
                "before the first -p",
            ),
            (
                vec!["-p", "p0", "--occurrence", "2", "--anchor", "fee"],
                "after --anchor",
            ),
            (
                vec![
                    "-p",
                    "p0",
                    "--anchor",
                    "fee",
                    "--occurrence",
                    "1",
                    "--occurrence",
                    "2",
                ],
                "given twice",
            ),
            (
                vec!["-p", "p0", "--anchor", "fee", "--occurrence", "0"],
                "1-based",
            ),
            (
                vec!["-p", "p0", "--anchor", "fee", "--occurrence", "many"],
                "integer",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let dir = dir.path();
            repeated_fee(dir);
            let mut args = vec![verb, "a.docx"];
            args.extend(flags);
            args.extend(["--content", "cost"]);
            let (code, out, err) = run(dir, &args);
            assert_eq!(code, 2, "{out}{err}");
            assert!(err.contains(expected), "{err}");
            assert!(written(dir).is_empty());
        }
    }
}

#[test]
fn occurrence_1894_insert_delete_and_format_keep_the_selected_occurrence() {
    for (flags, expected) in [
        (vec!["--content", "fee!"], "second fee! is waived"),
        (vec!["--content", "!fee"], "second !fee is waived"),
        (vec!["--delete"], "second  is waived"),
        (vec!["--style", "bold"], "second **fee** is waived"),
        (
            vec!["--content", "fee", "--style", "bold"],
            "second **fee** is waived",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        repeated_fee(dir);
        let mut args = vec![
            "edit",
            "a.docx",
            "-p",
            "p0",
            "--anchor",
            "fee",
            "--occurrence=2",
        ];
        args.extend(flags);
        args.push("--editing-mode");
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 0, "{out}{err}");
        let (_, view, _) = run(dir, &["read", "a_modified.docx"]);
        assert!(view.contains("first fee is due"), "{view}");
        assert!(view.contains(expected), "{view}");
    }
}

#[test]
fn edit_writes_file_modified_docx_and_confirms_its_view() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, out, err) = run(dir, &EDIT);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx"]);
    let xml = document_xml(&dir.join("a_modified.docx"));
    assert!(xml.contains("<w:ins "), "a redline by default: {xml}");
    assert!(out.contains("\n+source: a_modified.docx\n"), "{out}");
    assert!(err.contains("wrote a_modified.docx (docx, "), "{err}");

    let (code, _, err) = run(dir, &EDIT);
    assert_eq!(code, 2, "an existing output is refused: {err}");
    assert!(err.contains("--force"), "{err}");
    let mut forced = EDIT.to_vec();
    forced.push("--force");
    let (code, _, err) = run(dir, &forced);
    assert_eq!(code, 0, "{err}");
}

/// D15/Q14: a FILE whose stem ends `_modified` keeps its name, so editing
/// it again replaces it, with --force; `a.docx` is never touched.
#[test]
fn a_modified_file_is_edited_in_place_with_force() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let before = std::fs::read(dir.join("a.docx")).unwrap();
    assert_eq!(run(dir, &EDIT).0, 0);
    let again = [
        "add",
        "a_modified.docx",
        "-p",
        "p2",
        "--content",
        "New line.",
        "-a",
        "Jubarte",
    ];
    let (code, _, err) = run(dir, &again);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("'a_modified.docx' already exists (use --force"),
        "{err}"
    );
    let mut forced = again.to_vec();
    forced.push("--force");
    let (code, out, err) = run(dir, &forced);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("+++ after a_modified.docx\n"), "{out}");
    assert!(
        out.lines()
            .any(|line| line.starts_with('+') && line.contains("New line.")),
        "{out}"
    );
    assert_eq!(written(dir), ["a_modified.docx"]);
    assert!(document_xml(&dir.join("a_modified.docx")).contains("New line."));
    assert_eq!(std::fs::read(dir.join("a.docx")).unwrap(), before);
}

#[test]
fn editing_mode_writes_the_clean_document() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.push("--editing-mode");
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    let xml = document_xml(&dir.join("a_modified.docx"));
    assert!(!xml.contains("<w:ins "), "{xml}");
    assert!(xml.contains("forty"), "{xml}");
}

#[test]
fn add_writes_file_modified_docx_too() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p2",
            "--content",
            "New line.",
            "-a",
            "Jubarte",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx"]);
}

#[test]
fn out_dir_writes_the_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut bare = EDIT.to_vec();
    bare.push("--out-dir");
    let (code, out, err) = run(dir, &bare);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a.edit"]);
    for name in ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"] {
        assert!(
            dir.join("a.edit").join(name).is_file(),
            "a.edit holds {name}"
        );
    }
    assert!(out.contains("\n+source: redline.docx\n"), "{out}");

    let mut named = EDIT.to_vec();
    named.extend(["--out-dir", "r"]);
    let (code, _, err) = run(dir, &named);
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("r/redline.docx").is_file());
}

#[test]
fn o_names_the_one_document_and_refuses_the_rest() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    for bad in [
        &["-o", "b.docx", "--out-dir", "r"][..],
        &["-o", "b.pdf"][..],
        &["-o", "a.docx"][..],
        &["--out-dir", "x.docx"][..],
    ] {
        let mut args = EDIT.to_vec();
        args.extend(bad);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{bad:?}: {err}");
        assert!(written(dir).is_empty(), "{bad:?} wrote nothing");
    }
    let mut args = EDIT.to_vec();
    args.extend(["-o", "b.docx"]);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["b.docx"]);
    assert!(out.contains("\n+source: b.docx\n"), "{out}");
}

#[test]
fn pdf_lands_beside_the_document() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["--pdf", "-q"]);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx", "a_modified.pdf"]);
    assert!(out.is_empty() && err.is_empty(), "-q: {out}{err}");
}

#[test]
fn a_refused_plan_writes_nothing_and_prints_its_report() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "not in the paragraph",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 3, "{out}{err}");
    assert!(written(dir).is_empty(), "{:?}", written(dir));
    assert!(out.contains("not in the paragraph"), "{out}");
}

/// zcode-13b F2: the plan runs before the outputs are checked, so a refused
/// plan prints its report even when the output already exists.
#[test]
fn a_refused_plan_reports_even_over_an_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::write(dir.join("a_modified.docx"), b"kept").unwrap();
    std::fs::create_dir(dir.join("a.edit")).unwrap();
    for extra in [&[][..], &["--out-dir"][..]] {
        let mut args = vec![
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "absent words",
            "--content",
            "x",
        ];
        args.extend(extra);
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 3, "{extra:?}: {out}{err}");
        assert!(out.contains("absent words"), "{extra:?}: {out}");
    }
    assert_eq!(std::fs::read(dir.join("a_modified.docx")).unwrap(), b"kept");
}

/// pi-13b F1: the bundle's `wrote` line and notes are status lines, on
/// stderr (D6); its summary and the view stay on stdout.
#[test]
fn the_bundle_says_wrote_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["--out-dir", "r"]);
    let (code, out, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert!(err.starts_with("wrote r (4 files: "), "{err}");
    assert!(
        out.starts_with("{") && out.contains("\n+source: redline.docx\n"),
        "{out}"
    );
    assert!(!out.contains("\nwrote "), "{out}");
}

/// #568: the view's `source:` line and the bundle's report name the
/// written files with `/` on every OS — an agent reads them back —
/// whatever spelling FILE was given in. `d\a.docx` is a Windows path;
/// on a Unix it is one odd file name, so the test runs on both.
#[test]
fn diff_labels_spell_paths_with_forward_slashes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::create_dir_all(dir.join("d")).unwrap();
    std::fs::rename(dir.join("a.docx"), dir.join("d\\a.docx")).unwrap();
    let edit = [
        "edit",
        "d\\a.docx",
        "-p",
        "p1",
        "--content",
        "New line.",
        "-a",
        "Jubarte",
    ];
    let (code, out, err) = run(dir, &edit);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("+++ after d/a_modified.docx\n"), "{out}");
    let mut bundle = edit.to_vec();
    bundle.push("--out-dir");
    let (code, out, err) = run(dir, &bundle);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\n+source: redline.docx\n"), "{out}");
    let report = std::fs::read_to_string(dir.join("d\\a.edit").join("report.jsonl")).unwrap();
    assert!(report.contains("\"dir\":\"d/a.edit\""), "{report}");
}

/// zcode-13b F1: the advice names the extension as typed.
#[test]
fn a_text_output_is_refused_naming_its_own_extension() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["-o", "v.txt"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("`jubarte read FILE_modified.docx -o v.txt` writes .txt from it"),
        "{err}"
    );
}

/// The sha256 of a file, for "the input is untouched" assertions.
fn sha256(path: &Path) -> String {
    jubarte::inspect::source_sha256(&std::fs::read(path).unwrap())
}

/// V2-05: an output that spells the source another way is the same file;
/// it is refused with a stable code before anything is written, even with
/// `--force`, and the source keeps its bytes.
#[test]
fn an_output_spelling_the_source_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::create_dir(dir.join("sub")).unwrap();
    let source = sha256(&dir.join("a.docx"));
    let absolute = dir.join("a.docx");
    for spelling in [
        "a.docx",
        "./a.docx",
        "sub/../a.docx",
        "A.docx",
        "a.DOCX",
        "a.docx/",
        absolute.to_str().unwrap(),
    ] {
        let mut args = EDIT.to_vec();
        args.extend(["-o", spelling, "--force"]);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{spelling}: {err}");
        assert!(err.contains("OUTPUT_ALIAS"), "{spelling}: {err}");
        assert_eq!(sha256(&dir.join("a.docx")), source, "{spelling}");
    }
}

/// V2-05: a symlink or a hard link to the source is the same file too.
#[cfg(unix)]
#[test]
fn an_output_linked_to_the_source_is_refused() {
    use std::os::unix::fs::MetadataExt;
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let source = sha256(&dir.join("a.docx"));
    let source_ino = std::fs::metadata(dir.join("a.docx")).unwrap().ino();
    std::os::unix::fs::symlink("a.docx", dir.join("link.docx")).unwrap();
    std::fs::hard_link(dir.join("a.docx"), dir.join("hard.docx")).unwrap();
    for link in ["link.docx", "hard.docx"] {
        let mut args = EDIT.to_vec();
        args.extend(["-o", link, "--force"]);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{link}: {err}");
        assert!(err.contains("OUTPUT_ALIAS"), "{link}: {err}");
    }
    // Neither link nor the shared inode was touched.
    assert!(
        dir.join("link.docx")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(sha256(&dir.join("a.docx")), source);
    assert_eq!(
        std::fs::metadata(dir.join("hard.docx")).unwrap().ino(),
        source_ino
    );
}

/// V2-05: the plan is an input of the command as well; `-o` naming it
/// through a hard link is refused and the plan keeps its bytes. (`-o`
/// takes only a .docx name, so a .json plan is aliased through a link.)
#[test]
fn an_output_spelling_the_plan_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let plan = sha256(&dir.join("plan.json"));
    std::fs::hard_link(dir.join("plan.json"), dir.join("out.docx")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--force",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_ALIAS"), "{err}");
    assert_eq!(sha256(&dir.join("plan.json")), plan);
}

/// V2-05: a bundle member that is an input (here the plan, living at the
/// report's path) refuses the publication; nothing is written.
#[test]
fn a_bundle_member_aliasing_the_plan_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "-p",
            "p1",
            "--content",
            "x",
        ],
    );
    assert_eq!(code, 0, "{err}");
    std::fs::create_dir(dir.join("out")).unwrap();
    std::fs::copy(dir.join("plan.json"), dir.join("out/report.jsonl")).unwrap();
    let plan = sha256(&dir.join("out/report.jsonl"));
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "out/report.jsonl",
            "--consume",
            "--out-dir",
            "out",
            "--force",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_ALIAS"), "{err}");
    // The destination kept what it held: the plan, byte for byte, alone.
    assert_eq!(sha256(&dir.join("out/report.jsonl")), plan);
    assert_eq!(
        std::fs::read_dir(dir.join("out")).unwrap().count(),
        1,
        "no staged or committed file may appear in out/"
    );
}

/// V2-05: a forced bundle publishes a new directory wholesale over its own
/// earlier bundle; an existing destination without `--force` refuses.
#[test]
fn a_forced_bundle_replaces_its_earlier_bundle_wholesale() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["--out-dir", "r"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    let first = sha256(&dir.join("r/report.jsonl"));
    // Without --force the taken destination refuses, untouched.
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_EXISTS"), "{err}");
    assert!(err.contains("already exists (use --force"), "{err}");
    assert_eq!(sha256(&dir.join("r/report.jsonl")), first);
    // With --force the earlier bundle is replaced by a new directory.
    args.push("--force");
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("r/redline.docx").exists());
    assert!(dir.join("r/report.jsonl").exists());
}

/// V2-05: a bundle is never merged into a directory, and `--force` never
/// deletes a file the bundle did not write: a destination holding one
/// refuses with `OUTPUT_DIR_NOT_EMPTY`, every file in it untouched.
#[test]
fn a_forced_bundle_refuses_a_directory_holding_files_it_did_not_write() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let mut args = EDIT.to_vec();
    args.extend(["--out-dir", "r"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    std::fs::write(dir.join("r/notes.txt"), "the user's notes").unwrap();
    let report = sha256(&dir.join("r/report.jsonl"));
    args.push("--force");
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_DIR_NOT_EMPTY"), "{err}");
    assert!(
        err.contains("notes.txt"),
        "the refusal names the file: {err}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("r/notes.txt")).unwrap(),
        "the user's notes"
    );
    assert_eq!(sha256(&dir.join("r/report.jsonl")), report);
    // Nothing staged survives the refusal.
    assert_eq!(
        std::fs::read_dir(dir)
            .unwrap()
            .filter(|e| e
                .as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains("jubarte-tmp"))
            .count(),
        0
    );
}

/// V2-05: an input inside the forced destination (here in a subdirectory,
/// which the input-directory check does not cover) is never deleted by the
/// wholesale replacement.
#[test]
fn a_forced_bundle_never_deletes_an_input_inside_the_destination() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    std::fs::create_dir_all(dir.join("r/src")).unwrap();
    letter(&dir.join("r/src"));
    let source = sha256(&dir.join("r/src/a.docx"));
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "r/src/a.docx",
            "-p",
            "p1",
            "--content",
            "The fee is forty dollars.",
            "-a",
            "Jubarte",
            "--out-dir",
            "r",
            "--force",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_ALIAS"), "{err}");
    assert!(err.contains("r/src/a.docx"), "{err}");
    assert_eq!(sha256(&dir.join("r/src/a.docx")), source);
}

/// V2-05: a failure before the commit writes neither the output nor its
/// staging file. The output's parent is a regular file, so staging fails
/// with "Not a directory" for every user: a read-only directory would not
/// stop root, and the self-hosted CI runner runs as root.
#[test]
fn a_failed_commit_leaves_no_output_or_staging_file() {
    fn names(dir: &Path) -> Vec<std::ffi::OsString> {
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        names
    }
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::write(dir.join("plain"), b"a file, not a directory").unwrap();
    let before = names(dir);
    let mut args = EDIT.to_vec();
    args.extend(["-o", "plain/out.docx"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("plain/out.docx"), "{err}");
    assert_eq!(names(dir), before, "no output and no staging file");
    assert_eq!(
        std::fs::read(dir.join("plain")).unwrap(),
        b"a file, not a directory"
    );
}

/// C3/M18: one stable refusal and status for occupied destinations.
#[test]
fn occupied_outputs_share_the_output_exists_code() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    std::fs::write(dir.join("b.docx"), b"kept").unwrap();
    std::fs::create_dir(dir.join("r")).unwrap();
    for extra in [["-o", "b.docx"], ["--out-dir", "r"]] {
        let mut args = EDIT.to_vec();
        args.extend(extra);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{err}");
        assert!(err.contains("OUTPUT_EXISTS:"), "{err}");
        assert!(err.contains("already exists (use --force"), "{err}");
    }
    assert_eq!(std::fs::read(dir.join("b.docx")).unwrap(), b"kept");
    assert_eq!(std::fs::read_dir(dir.join("r")).unwrap().count(), 0);
}

/// M15: advice to force an output must never precede input protection.
#[test]
fn alias_refusals_precede_no_clobber_in_both_paths() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    std::fs::create_dir(dir.join("r")).unwrap();
    std::fs::hard_link(dir.join("a.docx"), dir.join("r/redline.docx")).unwrap();
    let source = sha256(&dir.join("a.docx"));
    for extra in [["-o", "a.docx"], ["--out-dir", "r"]] {
        let mut args = EDIT.to_vec();
        args.extend(extra);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{err}");
        assert!(err.contains("OUTPUT_ALIAS:"), "{err}");
        assert!(!err.contains("use --force"), "{err}");
    }
    assert_eq!(sha256(&dir.join("a.docx")), source);
}

/// C9: a PDF derivative cannot replace the primary DOCX.
#[test]
fn outputs_aliasing_each_other_write_nothing() {
    for name in ["r.pdf", "r.PDF"] {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        letter(dir);
        let mut args = EDIT.to_vec();
        args.extend(["-o", name, "--pdf"]);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 2, "{err}");
        assert!(err.contains("OUTPUT_ALIAS:"), "{err}");
        assert!(err.contains(name) && err.contains("r.pdf"), "{err}");
        assert!(written(dir).is_empty(), "{:?}", written(dir));
    }
}

/// I11: force accepts an empty existing directory, even without a report.
#[test]
fn force_replaces_an_empty_non_bundle_directory() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    std::fs::create_dir(dir.join("r")).unwrap();
    let mut args = EDIT.to_vec();
    args.extend(["--out-dir", "r", "--force"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 0, "{err}");
    assert!(dir.join("r/redline.docx").is_file());
    assert_eq!(std::fs::read_dir(dir.join("r")).unwrap().count(), 4);
}

/// C7/C8: existing link chains and symlink-to-plan aliases are refused.
#[cfg(unix)]
#[test]
fn chained_source_and_symlinked_plan_aliases_are_refused() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let source = sha256(&dir.join("a.docx"));
    std::os::unix::fs::symlink("a.docx", dir.join("link2.docx")).unwrap();
    std::os::unix::fs::symlink("link2.docx", dir.join("link1.docx")).unwrap();
    let mut args = EDIT.to_vec();
    args.extend(["-o", "link1.docx", "--force"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_ALIAS:"), "{err}");
    assert_eq!(sha256(&dir.join("a.docx")), source);
    let mut record = EDIT.to_vec();
    record.extend(["--plan", "plan.json"]);
    assert_eq!(run(dir, &record).0, 0);
    let plan = sha256(&dir.join("plan.json"));
    std::os::unix::fs::symlink("plan.json", dir.join("plan.docx")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "-o",
            "plan.docx",
            "--force",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_ALIAS:"), "{err}");
    assert_eq!(sha256(&dir.join("plan.json")), plan);
}

/// I6/C2 pin: a member alias is an input; non-member links remain foreign.
#[cfg(unix)]
#[test]
fn bundle_member_and_foreign_links_keep_their_refusal_classes() {
    for name in ["redline.docx", "foreign-hard.docx", "foreign-sym.docx"] {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        letter(dir);
        std::fs::create_dir(dir.join("r")).unwrap();
        let source = sha256(&dir.join("a.docx"));
        if name == "foreign-sym.docx" {
            std::os::unix::fs::symlink("../a.docx", dir.join("r").join(name)).unwrap();
        } else {
            std::fs::hard_link(dir.join("a.docx"), dir.join("r").join(name)).unwrap();
        }
        let (code, _, err) = run(
            dir,
            &[
                "edit",
                "r/../a.docx",
                "-p",
                "p1",
                "--content",
                "changed",
                "-a",
                "Jubarte",
                "--out-dir",
                "r",
                "--force",
            ],
        );
        assert_eq!(code, 2, "{err}");
        let expected = if name == "redline.docx" {
            "OUTPUT_ALIAS:"
        } else {
            "OUTPUT_DIR_NOT_EMPTY:"
        };
        assert!(err.contains(expected), "{err}");
        assert_eq!(sha256(&dir.join("a.docx")), source);
        assert_eq!(std::fs::read_dir(dir.join("r")).unwrap().count(), 1);
    }
}

/// Debug-build-only publication synchronization. The child connects after
/// staging and after its final collision check; the parent releases it only
/// after making the exact filesystem change under test. These are integration
/// tests: no sleeps or probabilistic racing writers determine the result.
#[cfg(unix)]
fn publication_hook(
    dir: &Path,
    args: &[std::ffi::OsString],
    mut at: impl FnMut(&str, u32),
) -> std::process::Output {
    use std::os::unix::net::UnixListener;
    let socket = dir.join("hook.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let mut child = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .env("JUBARTE_TEST_PUBLICATION_HOOK", &socket)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // Polling is only for child exit / a bounded synchronization timeout;
    // the callback always runs while the child is blocked on the socket.
    listener.set_nonblocking(true).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match listener.accept() {
            Ok((stream, _)) => serve_hook(stream, &mut at),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
                if std::time::Instant::now() >= deadline {
                    child.kill().unwrap();
                    panic!("publication hook timed out");
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(e) => panic!("{e}"),
        }
    }
    let output = child.wait_with_output().unwrap();
    std::fs::remove_file(socket).unwrap();
    output
}

/// One hook connection: the child's `PHASE PID` line, the callback, then
/// the byte that releases the child. On macOS a stream accepted from the
/// non-blocking listener is non-blocking too (BSD sockets inherit
/// `O_NONBLOCK`; Linux's do not): a child that has connected but not yet
/// written read as `WouldBlock` (#1859). The read blocks, bounded by its
/// timeout.
#[cfg(unix)]
fn serve_hook(mut stream: std::os::unix::net::UnixStream, at: &mut impl FnMut(&str, u32)) {
    use std::io::{BufRead, Write};
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let mut line = String::new();
    std::io::BufReader::new(&mut stream)
        .read_line(&mut line)
        .unwrap();
    let (phase, pid) = line.trim().split_once(' ').unwrap();
    at(phase, pid.parse().unwrap());
    stream.write_all(b"x").unwrap();
}

/// #1859: a child that connects before it writes its line is waited for.
/// The listener polls without blocking; the connection must still block.
#[cfg(unix)]
#[test]
fn the_hook_waits_for_a_line_written_after_connecting() {
    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("late.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let client = std::thread::spawn(move || {
        let mut stream = UnixStream::connect(&socket).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        stream.write_all(b"staged 42\n").unwrap();
        let mut released = [0u8; 1];
        stream.read_exact(&mut released).unwrap();
        released[0]
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() > deadline {
                    panic!("timeout waiting for connection");
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(e) => panic!("{e}"),
        }
    };
    let mut seen = Vec::new();
    serve_hook(stream, &mut |phase, pid| {
        seen.push((phase.to_string(), pid));
    });
    assert_eq!(seen, [("staged".to_string(), 42)]);
    assert_eq!(client.join().unwrap(), b'x');
}

#[cfg(unix)]
fn hook_args(extra: &[&str]) -> Vec<std::ffi::OsString> {
    EDIT.iter()
        .chain(extra)
        .map(std::ffi::OsString::from)
        .collect()
}

/// C1: changing the named source never substitutes unprepared bytes.
#[cfg(unix)]
#[test]
fn a_source_changed_after_preparation_refuses_without_publishing() {
    for extra in [&["-o", "out.docx"][..], &["--out-dir", "r"][..]] {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        letter(dir);
        let replacement = docx(&para("A different named source."));
        let mut injected = false;
        let out = publication_hook(dir, &hook_args(extra), |phase, _| {
            if phase == "staged" {
                std::fs::write(dir.join("a.docx"), &replacement).unwrap();
                injected = true;
            }
        });
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(injected, "the source mutation ran");
        assert_eq!(out.status.code(), Some(3), "{err}");
        assert!(err.contains("SOURCE_CHANGED:"), "{err}");
        assert_eq!(std::fs::read(dir.join("a.docx")).unwrap(), replacement);
        assert!(written(dir).is_empty(), "{:?}", written(dir));
    }
}

/// C1 holds for the typed edit routes: they publish through the same
/// recheck, so a source changed after staging publishes nothing, in either
/// delivery.
#[cfg(unix)]
#[test]
fn a_typed_route_refuses_a_source_changed_after_staging() {
    for extra in [&["-o", "out.docx"][..], &["--out-dir", "r"][..]] {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        letter(dir);
        let replacement = docx(&para("A different named source."));
        let mut injected = false;
        let args: Vec<std::ffi::OsString> = [
            "edit", "replace", "--find", "thirty", "--text", "forty", "a.docx", "--track", "off",
            "--json",
        ]
        .iter()
        .chain(extra)
        .map(std::ffi::OsString::from)
        .collect();
        let out = publication_hook(dir, &args, |phase, _| {
            if phase == "staged" {
                std::fs::write(dir.join("a.docx"), &replacement).unwrap();
                injected = true;
            }
        });
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(injected, "the source mutation ran: {stdout}");
        let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).expect("envelope");
        assert_eq!(envelope["error"]["code"], "STALE_SOURCE", "{stdout}");
        assert_eq!(
            envelope["error"]["commit_state"], "not_committed",
            "{stdout}"
        );
        assert_eq!(std::fs::read(dir.join("a.docx")).unwrap(), replacement);
        assert!(written(dir).is_empty(), "{:?}", written(dir));
    }
}

/// The template route publishes its plan through the same recheck: a source
/// changed after staging publishes no plan.
#[cfg(unix)]
#[test]
fn edit_template_refuses_a_source_changed_after_staging() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let replacement = docx(&para("A different named source."));
    let mut injected = false;
    let args: Vec<std::ffi::OsString> = ["edit", "template", "a.docx", "-o", "plan.json", "--json"]
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let out = publication_hook(dir, &args, |phase, _| {
        if phase == "staged" {
            std::fs::write(dir.join("a.docx"), &replacement).unwrap();
            injected = true;
        }
    });
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(injected, "the source mutation ran: {stdout}");
    let envelope: serde_json::Value = serde_json::from_str(stdout.trim()).expect("envelope");
    assert_eq!(envelope["error"]["code"], "STALE_SOURCE", "{stdout}");
    assert_eq!(
        envelope["error"]["commit_state"], "not_committed",
        "{stdout}"
    );
    assert!(written(dir).is_empty(), "{:?}", written(dir));
}

/// C1: an entry created after the last taken check must not be clobbered.
#[cfg(unix)]
#[test]
fn a_destination_created_after_the_check_is_not_clobbered() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let mut injected = false;
    let out = publication_hook(dir, &hook_args(&["-o", "out.docx"]), |phase, _| {
        if phase == "checked" {
            std::fs::write(dir.join("out.docx"), b"late output").unwrap();
            injected = true;
        }
    });
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(injected, "the late destination was created");
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("OUTPUT_EXISTS:"), "{err}");
    assert_eq!(std::fs::read(dir.join("out.docx")).unwrap(), b"late output");
    assert_eq!(written(dir), ["out.docx"]);
}

/// C1: the final alias recheck catches a link introduced after staging.
#[cfg(unix)]
#[test]
fn an_alias_introduced_after_staging_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let source = sha256(&dir.join("a.docx"));
    let mut injected = false;
    let out = publication_hook(
        dir,
        &hook_args(&["-o", "out.docx", "--force"]),
        |phase, _| {
            if phase == "staged" {
                std::os::unix::fs::symlink("a.docx", dir.join("out.docx")).unwrap();
                injected = true;
            }
        },
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(injected);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("OUTPUT_ALIAS:"), "{err}");
    assert!(
        dir.join("out.docx")
            .symlink_metadata()
            .unwrap()
            .is_symlink()
    );
    assert_eq!(sha256(&dir.join("a.docx")), source);
}

/// C1: a symlink racing after the recheck is replaced by rename; the
/// source inode is never written through (the review's attack was wrong).
#[cfg(unix)]
#[test]
fn force_rename_replaces_a_late_symlink_without_touching_its_input() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let source = sha256(&dir.join("a.docx"));
    let mut injected = false;
    let out = publication_hook(
        dir,
        &hook_args(&["-o", "out.docx", "--force"]),
        |phase, _| {
            if phase == "checked" {
                std::os::unix::fs::symlink("a.docx", dir.join("out.docx")).unwrap();
                injected = true;
            }
        },
    );
    assert!(injected);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert!(
        !dir.join("out.docx")
            .symlink_metadata()
            .unwrap()
            .is_symlink()
    );
    assert_eq!(sha256(&dir.join("a.docx")), source);
    assert!(document_xml(&dir.join("out.docx")).contains("forty"));
}

/// C5: a stale aside belonging to a reused PID remains intact.
#[cfg(unix)]
#[test]
fn a_forced_bundle_probes_past_an_occupied_aside_name() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    std::fs::create_dir(dir.join("r")).unwrap();
    let mut aside = None;
    let out = publication_hook(
        dir,
        &hook_args(&["--out-dir", "r", "--force"]),
        |phase, pid| {
            if phase == "staged" {
                let path = dir.join(format!(".r.jubarte-tmp-{pid}-100"));
                std::fs::create_dir(&path).unwrap();
                std::fs::write(path.join("kept"), b"old residue").unwrap();
                aside = Some(path);
            }
        },
    );
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        std::fs::read(aside.unwrap().join("kept")).unwrap(),
        b"old residue"
    );
    assert!(dir.join("r/redline.docx").is_file());
    assert_eq!(written(dir).len(), 2, "only bundle and preserved residue");
}

/// M4: a destination's raw filename bytes survive in its staging name.
#[cfg(unix)]
#[test]
fn staging_names_preserve_non_utf8_destination_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    letter(dir);
    let name = std::ffi::OsString::from_vec(b"r-\xff".to_vec());
    // APFS rejects invalid UTF-8 directory entries. On such a filesystem
    // byte preservation means refusing the exact name before staging,
    // rather than creating a different name with replacement characters.
    let accepts_raw_name = match std::fs::create_dir(dir.join(&name)) {
        Ok(()) => {
            std::fs::remove_dir(dir.join(&name)).unwrap();
            true
        }
        Err(e) if e.raw_os_error() == Some(92) => false, // macOS EILSEQ
        Err(e) => panic!("probing raw filename support: {e}"),
    };
    let mut args = hook_args(&["--out-dir"]);
    args.push(name.clone());
    let mut exact = false;
    let mut visited = false;
    let out = publication_hook(dir, &args, |phase, pid| {
        if phase == "staged" {
            let mut stage = std::ffi::OsString::from(".");
            stage.push(&name);
            stage.push(format!(".jubarte-tmp-{pid}-0"));
            exact = dir.join(stage).is_dir();
            visited = true;
        }
    });
    if accepts_raw_name {
        assert!(visited, "{out:?}");
        assert!(
            exact,
            "staging must preserve the filename bytes, without U+FFFD"
        );
        assert_eq!(out.status.code(), Some(0), "{out:?}");
        assert!(dir.join(name).join("redline.docx").is_file());
    } else {
        assert!(
            !visited,
            "an unrepresentable staging name must be refused, never replaced with U+FFFD"
        );
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("Illegal byte sequence"),
            "{out:?}"
        );
        assert!(written(dir).is_empty(), "{:?}", written(dir));
    }
}

/// clap's one-shot `--author` default (`EditOptions`), not a guessed name.
fn default_author() -> String {
    let json = jubarte::cli::parse_json(
        &[
            "edit".into(),
            "a.docx".into(),
            "-p".into(),
            "p0".into(),
            "--content".into(),
            "x".into(),
        ],
        "jubarte",
        &["edit".into()],
    );
    serde_json::from_str::<serde_json::Value>(&json).unwrap()["args"]["author"]
        .as_str()
        .expect("author default")
        .to_owned()
}

fn wrote_docx(err: &str) -> &str {
    err.lines()
        .find(|line| line.starts_with("wrote ") && line.contains(" (docx, "))
        .unwrap_or(err)
}

fn assert_wrote_markup(err: &str, file: &str, detail: &str) {
    let line = wrote_docx(err);
    let prefix = format!("wrote {file} (docx, ");
    let suffix = format!(" bytes; {detail})");
    assert!(
        line.starts_with(&prefix) && line.ends_with(&suffix),
        "expected `{prefix}N{suffix}`\n{err}"
    );
}

/// #1891: a one-shot edit with `-a` names that author and the change count.
#[test]
fn edit_wrote_line_names_the_given_author_and_change_count() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_wrote_markup(&err, "out.docx", r#"1 change by "GiantCo""#);
    assert!(
        !err.contains("warning: no -a given"),
        "explicit -a is not a warning: {err}"
    );
}

/// #1997: a one-shot write that adds markup needs `-a` (or
/// `JUBARTE_AUTHOR`): the refusal is `AUTHOR_REQUIRED`, exit 2, and names
/// the authors the document's markup already carries. The document is
/// untouched.
#[test]
fn edit_adding_markup_without_an_author_is_author_required() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let before = std::fs::read(dir.join("a.docx")).unwrap();
    for args in [
        // One-shot edit adding a tracked change.
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-o",
            "out.docx",
        ][..],
        // One-shot add adding a comment.
        &[
            "add",
            "a.docx",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "-o",
            "out.docx",
        ][..],
        // The bundle refuses the same way.
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--out-dir",
            "bundle",
        ][..],
        // --in-place refuses before touching the input.
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--in-place",
        ][..],
    ] {
        let (code, _, err) = run(dir, args);
        assert_eq!(code, 2, "{args:?}\n{err}");
        assert!(
            err.contains("AUTHOR_REQUIRED: sign the new markup with -a NAME"),
            "{args:?}\n{err}"
        );
        assert!(
            !err.contains("this document has markup by"),
            "a markup-free letter names no author: {args:?}\n{err}"
        );
        assert_eq!(
            std::fs::read(dir.join("a.docx")).unwrap(),
            before,
            "{args:?} changed the input"
        );
        assert!(!dir.join("out.docx").exists() && !dir.join("bundle").exists());
    }
    // A document that already has markup names who signed it.
    let redline = redline(dir);
    std::fs::write(dir.join("redline.docx"), &redline).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "redline.docx",
            "-p",
            "p2",
            "--anchor",
            "Last",
            "--content",
            "First",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains(
            r#"AUTHOR_REQUIRED: sign the new markup with -a NAME (this document has markup by "GiantCo")"#
        ),
        "{err}"
    );
    assert!(!dir.join("out.docx").exists());
}

/// #1997: `JUBARTE_AUTHOR`, set and non-empty, counts as `-a`; an empty one
/// does not.
#[test]
fn jubarte_author_env_signs_the_new_markup() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let out = Command::new(BIN)
        .current_dir(dir)
        .args([
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-o",
            "out.docx",
        ])
        .env("JUBARTE_AUTHOR", "Trainer 1 - AgentCo")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code().unwrap_or(-1), 0, "{err}");
    assert_wrote_markup(&err, "out.docx", r#"1 change by "Trainer 1 - AgentCo""#);
    assert!(!err.contains("no -a given"), "{err}");

    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let out = Command::new(BIN)
        .current_dir(dir)
        .args([
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-o",
            "out.docx",
        ])
        .env("JUBARTE_AUTHOR", "")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code().unwrap_or(-1), 2, "{err}");
    assert!(err.contains("AUTHOR_REQUIRED"), "{err}");
}

/// #1997: `-a` beats `JUBARTE_AUTHOR`, and a write that adds no markup
/// (`--accept`/`--reject` only) needs neither.
#[test]
fn author_flag_beats_env_and_resolutions_need_no_author() {
    const TRACKED: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/agent-view/received.docx"
    );
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    std::fs::copy(TRACKED, dir.join("in.docx")).unwrap();
    let out = Command::new(BIN)
        .current_dir(dir)
        .args([
            "edit",
            "in.docx",
            "-p",
            "r0",
            "--accept",
            "-a",
            "GiantCo",
            "-o",
            "accepted.docx",
        ])
        .env("JUBARTE_AUTHOR", "Trainer 1 - AgentCo")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code().unwrap_or(-1), 0, "{err}");
    // The spelled -a wins over the environment when markup is signed.
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let out = Command::new(BIN)
        .current_dir(dir)
        .args([
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ])
        .env("JUBARTE_AUTHOR", "Trainer 1 - AgentCo")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code().unwrap_or(-1), 0, "{err}");
    assert_wrote_markup(&err, "out.docx", r#"1 change by "GiantCo""#);
}

/// #1891: `add --comment` counts comments, not changes.
#[test]
fn add_comment_wrote_line_counts_the_comment() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_wrote_markup(&err, "out.docx", r#"1 comment by "GiantCo""#);
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: a write that adds both a change and a comment names both.
#[test]
fn mixed_write_counts_changes_and_comments() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p2",
            "--content",
            "New line.",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_wrote_markup(&err, "out.docx", r#"1 change, 1 comment by "GiantCo""#);
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: `--accept` / `--reject` add no markup: today's wrote line, no warning.
#[test]
fn accept_or_reject_only_keeps_todays_wrote_line() {
    const TRACKED: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/agent-view/received.docx"
    );
    for verb in ["--accept", "--reject"] {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        std::fs::copy(TRACKED, dir.join("in.docx")).unwrap();
        let (code, _, err) = run(
            dir,
            &["edit", "in.docx", "-p", "r0", verb, "-o", "out.docx"],
        );
        assert_eq!(code, 0, "{verb}: {err}");
        let line = wrote_docx(&err);
        assert!(
            line.starts_with("wrote out.docx (docx, ") && line.ends_with(" bytes)"),
            "{verb}: {err}"
        );
        assert!(!line.contains(';'), "{verb}: {err}");
        assert!(!err.contains("warning: no -a given"), "{verb}: {err}");
    }
}

/// #1891: `-a` with the default name still counts as explicit: no warning.
#[test]
fn explicit_default_author_is_not_a_missing_author_warning() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let author = default_author();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            &author,
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_wrote_markup(&err, "out.docx", &format!(r#"1 change by "{author}""#));
    assert!(!err.contains("no -a given"), "{err}");
}

/// #1891: `--editing-mode` publishes the clean copy. The internal redline's
/// tracked changes are not in that file, so the wrote line names no change
/// and does not warn.
#[test]
fn editing_mode_wrote_line_ignores_the_internal_redline() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--editing-mode",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = wrote_docx(&err);
    assert!(
        line.starts_with("wrote out.docx (docx, ") && line.ends_with(" bytes)"),
        "{err}"
    );
    assert!(!line.contains(';'), "{err}");
    assert!(!err.contains("warning: no -a given"), "{err}");
    let xml = document_xml(&dir.join("out.docx"));
    assert!(xml.contains("forty"), "{xml}");
    assert!(!xml.contains("<w:ins "), "{xml}");
    assert!(!xml.contains("<w:del "), "{xml}");
}

/// #1997: a comment `--editing-mode` keeps in the clean copy is markup
/// too: it needs `-a` like the redline's. With `-a` the write counts it.
#[test]
fn editing_mode_wrote_line_counts_a_published_comment() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "--editing-mode",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("AUTHOR_REQUIRED"), "{err}");
    assert!(!dir.join("out.docx").exists());

    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "--editing-mode",
            "-a",
            "GiantCo",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let published = std::fs::read(dir.join("out.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&published).unwrap();
    assert_eq!(comments.len(), 1, "{comments:?}");
    assert_eq!(comments[0].text, "Cap the fee?");
    assert_wrote_markup(&err, "out.docx", r#"1 comment by "GiantCo""#);
    let xml = document_xml(&dir.join("out.docx"));
    assert!(!xml.contains("<w:ins "), "{xml}");
}

/// #1891: a comment on deleted text lives in the redline only. Editing mode
/// publishes the clean copy, which has neither the change nor the comment.
#[test]
fn editing_mode_ignores_a_comment_the_clean_copy_drops() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::write(
        dir.join("plan.json"),
        r#"{"schema_version":1,"author":"GiantCo","operations":[{"kind":"delete","paragraph":"p1","find":"thirty","comment":"why"}]}"#,
    )
    .unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "--editing-mode",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let published = std::fs::read(dir.join("out.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&published).unwrap();
    assert!(
        comments.is_empty(),
        "the clean copy kept a deletion comment: {comments:?}"
    );
    let line = wrote_docx(&err);
    assert!(
        line.starts_with("wrote out.docx (docx, ") && line.ends_with(" bytes)"),
        "{err}"
    );
    assert!(!line.contains(';'), "{err}");
    assert!(!err.contains("warning: no -a given"), "{err}");
    assert!(
        !document_xml(&dir.join("out.docx")).contains("thirty"),
        "{err}"
    );
}

/// #1891: a replace whose comment the clean copy keeps counts the comment,
/// not the internal redline's tracked change. The plan names the author.
#[test]
fn editing_mode_counts_a_comment_on_text_it_publishes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::write(
        dir.join("plan.json"),
        r#"{"schema_version":1,"author":"GiantCo","operations":[{"kind":"replace","paragraph":"p1","find":"thirty","replacement":"forty","comment":"Cap it"}]}"#,
    )
    .unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "--plan",
            "plan.json",
            "--editing-mode",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let published = std::fs::read(dir.join("out.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&published).unwrap();
    assert_eq!(comments.len(), 1, "{comments:?}");
    assert_eq!(comments[0].author, "GiantCo");
    assert_wrote_markup(&err, "out.docx", r#"1 comment by "GiantCo""#);
    assert!(!err.contains("no -a given"), "{err}");
    let xml = document_xml(&dir.join("out.docx"));
    assert!(xml.contains("forty") && !xml.contains("<w:ins "), "{xml}");
}

/// #1891: `--out-dir` names who signed, the same detail as `-o`.
#[test]
fn out_dir_wrote_line_names_the_given_author() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "--out-dir",
            "r",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = err
        .lines()
        .find(|line| line.starts_with("wrote r ("))
        .unwrap_or(&err);
    assert!(line.contains("4 files:"), "{line}");
    assert!(line.ends_with(r#"; 1 change by "GiantCo")"#), "{line}");
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1997: an `--out-dir` write without `-a` is `AUTHOR_REQUIRED` too.
#[test]
fn out_dir_wrote_line_without_author_is_author_required() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--out-dir",
            "r",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("AUTHOR_REQUIRED: sign the new markup with -a NAME"),
        "{err}"
    );
    assert!(!dir.join("r").exists(), "nothing was written");
}

/// #1891: an editing-mode bundle publishes the clean copy. A comment it
/// keeps counts; the internal redline's change does not.
#[test]
fn out_dir_editing_mode_counts_a_published_comment_and_no_change() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--editing-mode",
            "--out-dir",
            "plain",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let plain = err
        .lines()
        .find(|line| line.starts_with("wrote plain ("))
        .unwrap_or(&err);
    assert!(plain.contains("2 files:"), "{plain}");
    assert!(!plain.contains(';'), "{plain}");
    assert!(!err.contains("warning: no -a given"), "{err}");

    let (code, _, err) = run(
        dir,
        &[
            "add",
            "a.docx",
            "-p",
            "p1",
            "--comment",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
            "--editing-mode",
            "--out-dir",
            "noted",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let noted = std::fs::read(dir.join("noted/clean.docx")).unwrap();
    assert_eq!(jubarte::comments::list_comments(&noted).unwrap().len(), 1);
    let line = err
        .lines()
        .find(|line| line.starts_with("wrote noted ("))
        .unwrap_or(&err);
    assert!(line.ends_with(r#"; 1 comment by "GiantCo")"#), "{line}");
    assert!(!err.contains("warning: no -a given"), "{err}");
}

/// #1891: a dry-run JSONL envelope still carries `author`.
#[test]
fn dry_run_jsonl_keeps_the_author_field() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let author = default_author();
    let (code, out, err) = run(
        dir,
        &[
            "edit",
            "a.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--dry-run",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    let load: serde_json::Value = serde_json::from_str(out.lines().next().expect("jsonl")).unwrap();
    assert_eq!(load["ev"], "load", "{out}");
    assert_eq!(load["author"], author, "{out}");
}

/// #1888: a letter named `contract.docx`, so the wrote line matches the flag's spelling.
fn contract(dir: &Path) -> Vec<u8> {
    letter(dir);
    let path = dir.join("contract.docx");
    std::fs::rename(dir.join("a.docx"), &path).unwrap();
    std::fs::read(&path).unwrap()
}

fn sha8(bytes: &[u8]) -> String {
    jubarte::inspect::source_sha256(bytes)[..8].to_string()
}

fn wrote_any(err: &str) -> &str {
    err.lines()
        .find(|line| line.starts_with("wrote "))
        .unwrap_or(err)
}

fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

/// A redline of the letter, written beside `source.docx` as `contract.docx`.
fn redline(dir: &Path) -> Vec<u8> {
    letter(dir);
    std::fs::rename(dir.join("a.docx"), dir.join("source.docx")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "source.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "-o",
            "contract.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    std::fs::read(dir.join("contract.docx")).unwrap()
}

/// #1888: `edit --in-place` and `add --in-place` replace the input.
#[test]
fn in_place_edit_and_add_replace_the_input() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let prefix = sha8(&before);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "--in-place",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = wrote_any(&err);
    assert!(
        line.starts_with(&format!(
            "wrote contract.docx in place (replaced sha256 {prefix}\u{2026})"
        )),
        "{line}"
    );
    assert!(line.contains("; 1 change by \"GiantCo\""), "{line}");
    assert!(!line.contains("(docx,"), "{line}");
    assert!(!err.contains("warning: no -a given"), "{err}");
    let edited = std::fs::read(dir.join("contract.docx")).unwrap();
    assert_ne!(edited, before);
    assert!(document_xml(dir.join("contract.docx").as_path()).contains("forty"));
    assert_eq!(names(dir), ["contract.docx"]);

    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let prefix = sha8(&before);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
            "--in-place",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = wrote_any(&err);
    assert!(
        line.starts_with(&format!(
            "wrote contract.docx in place (replaced sha256 {prefix}\u{2026})"
        )),
        "{line}"
    );
    assert!(line.contains("; 1 comment by \"GiantCo\""), "{line}");
    assert!(!line.contains("(docx,"), "{line}");
    assert_ne!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_eq!(names(dir), ["contract.docx"]);
}

/// #1997: `--in-place` refuses a markup-adding write without `-a`,
/// before the input is touched.
#[test]
fn in_place_edit_without_an_author_is_author_required() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "--in-place",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("AUTHOR_REQUIRED"), "{err}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
}

/// #1888: `accept --in-place` and `reject --in-place` replace the input.
#[test]
fn in_place_accept_and_reject_replace_the_input() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let redline = redline(dir);
    std::fs::write(dir.join("accept.docx"), &redline).unwrap();
    std::fs::write(dir.join("reject.docx"), &redline).unwrap();
    let accept_prefix = sha8(&redline);
    let (code, _, err) = run(dir, &["accept", "accept.docx", "--in-place"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        wrote_any(&err),
        format!("wrote accept.docx in place (replaced sha256 {accept_prefix}\u{2026})")
    );
    assert!(!err.contains("(docx,"), "{err}");
    assert!(!err.contains("deprecated"), "{err}");
    let accepted = document_xml(&dir.join("accept.docx"));
    assert!(accepted.contains("forty"), "{accepted}");
    assert!(!accepted.contains("<w:ins"), "{accepted}");
    let (code, _, err) = run(dir, &["accept", "contract.docx", "-o", "accepted.docx"]);
    assert_eq!(code, 0, "{err}");
    assert!(!err.contains("deprecated"), "{err}");
    assert_eq!(
        std::fs::read(dir.join("accept.docx")).unwrap(),
        std::fs::read(dir.join("accepted.docx")).unwrap()
    );

    let (code, _, err) = run(dir, &["reject", "reject.docx", "--in-place"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(
        wrote_any(&err),
        format!("wrote reject.docx in place (replaced sha256 {accept_prefix}\u{2026})")
    );
    let rejected = document_xml(&dir.join("reject.docx"));
    assert!(rejected.contains("thirty"), "{rejected}");
    assert!(!rejected.contains("forty"), "{rejected}");
    assert!(!rejected.contains("<w:del"), "{rejected}");
    assert!(!dir.join("accept_modified.docx").exists());
    assert!(!dir.join("reject_modified.docx").exists());
}

/// #1888: a refused edit or a failed check leaves the input byte-identical.
#[test]
fn in_place_refusal_and_failed_check_leave_the_input_byte_identical() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p99",
            "--content",
            "x",
            "--in-place",
        ],
    );
    assert_eq!(code, 3, "{err}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert!(!dir.join("contract_modified.docx").exists());

    let plan = r#"{"schema_version":1,"author":"GiantCo","operations":[{"kind":"replace","paragraph":"p1","find":"thirty","replacement":"thirty"}]}"#;
    std::fs::write(dir.join("plan.json"), plan).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "--plan",
            "plan.json",
            "--require-change",
            "--in-place",
        ],
    );
    assert_eq!(code, 3, "{err}");
    assert!(err.contains("NO_CHANGES_REQUESTED"), "{err}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert!(!dir.join("contract_modified.docx").exists());
}

/// #1888: `--in-place` with `-o` or `--out-dir` is a usage error.
#[test]
fn in_place_conflicts_with_output_and_out_dir() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let cases: &[&[&str]] = &[
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--in-place",
            "-o",
            "out.docx",
        ],
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--in-place",
            "--out-dir",
        ],
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--in-place",
            "--out-dir",
            "bundle",
        ],
        &[
            "add",
            "contract.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--in-place",
            "-o",
            "out.docx",
        ],
        &[
            "add",
            "contract.docx",
            "-p",
            "p1",
            "--content",
            "x",
            "--in-place",
            "--out-dir",
            "bundle",
        ],
        &["accept", "contract.docx", "--in-place", "-o", "out.docx"],
        &["reject", "contract.docx", "--in-place", "-o", "out.docx"],
    ];
    for args in cases {
        let (code, _, err) = run(dir, args);
        assert_eq!(code, 2, "{args:?}\n{err}");
        assert!(
            err.contains("cannot be used with") && err.contains("--in-place"),
            "{args:?}\n{err}"
        );
        assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    }
    assert_eq!(names(dir), ["contract.docx"]);
}

/// #1998: `accept`/`reject -o INPUT` refuses and names `--in-place`,
/// with or without `--force`. The 0.16.0 deprecation window is over.
#[test]
fn accept_reject_o_naming_the_input_refuses_in_favour_of_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let redline = redline(dir);
    let refusal = "OUTPUT_ALIAS: the output 'contract.docx' is the same file as the input 'contract.docx'; use --in-place";

    for args in [
        &["accept", "contract.docx", "-o", "contract.docx"][..],
        &["accept", "contract.docx", "-o", "contract.docx", "--force"][..],
        &["reject", "contract.docx", "-o", "contract.docx"][..],
        &["reject", "contract.docx", "-o", "contract.docx", "--force"][..],
        // Another spelling of the same file is the same refusal.
        &[
            "accept",
            "contract.docx",
            "-o",
            "./contract.docx",
            "--force",
        ][..],
    ] {
        std::fs::write(dir.join("contract.docx"), &redline).unwrap();
        let (code, _, err) = run(dir, args);
        assert_eq!(code, 2, "{args:?}\n{err}");
        assert!(
            err.contains("OUTPUT_ALIAS") && err.contains("; use --in-place"),
            "{args:?}\n{err}"
        );
        assert!(
            std::fs::read(dir.join("contract.docx")).unwrap() == redline,
            "{args:?} changed the input"
        );
    }
    std::fs::write(dir.join("contract.docx"), &redline).unwrap();
    let (code, _, err) = run(dir, &["accept", "contract.docx", "-o", "contract.docx"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains(refusal), "{err}");

    // -q quiets status, never the error.
    std::fs::write(dir.join("quiet.docx"), &redline).unwrap();
    let (code, _, err) = run(
        dir,
        &["-q", "accept", "quiet.docx", "-o", "quiet.docx", "--force"],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains(
            "the output 'quiet.docx' is the same file as the input 'quiet.docx'; use --in-place"
        ),
        "{err}"
    );

    // `-o` naming another file still writes it.
    std::fs::write(dir.join("contract.docx"), &redline).unwrap();
    let (code, _, err) = run(dir, &["accept", "contract.docx", "-o", "other.docx"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), redline);

    // `--in-place` is the spelled-out route the refusal names.
    std::fs::write(dir.join("contract.docx"), &redline).unwrap();
    let (code, _, err) = run(dir, &["accept", "contract.docx", "--in-place"]);
    assert_eq!(code, 0, "{err}");
    let accepted = document_xml(&dir.join("contract.docx"));
    assert!(
        accepted.contains("forty") && !accepted.contains("<w:ins"),
        "{accepted}"
    );
}

/// #1888: the default sibling says the input was left unchanged.
#[test]
fn default_output_says_the_input_is_unchanged() {
    let note = "; contract.docx is unchanged (--in-place updates it)";
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = wrote_any(&err);
    assert!(line.ends_with(note), "{line}");
    assert!(line.contains("(docx, "), "{line}");
    assert!(line.contains("; 1 change by \"GiantCo\")"), "{line}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_ne!(
        std::fs::read(dir.join("contract_modified.docx")).unwrap(),
        before
    );

    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "-o",
            "named.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let named = wrote_any(&err);
    assert!(!named.contains("unchanged"), "{named}");
    assert!(named.contains("(docx, "), "{named}");

    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let (code, _, err) = run(
        dir,
        &[
            "add",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "Cap the fee?",
            "-a",
            "GiantCo",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let line = wrote_any(&err);
    assert!(line.ends_with(note), "{line}");
    assert!(line.contains("; 1 comment by \"GiantCo\")"), "{line}");
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert!(dir.join("contract_modified.docx").is_file());
}

/// #1888: an input whose stem already ends `_modified` is the default output.
#[test]
fn modified_stem_default_omits_the_unchanged_note() {
    for (verb, content) in [
        ("edit", &["--anchor", "thirty", "--content", "forty"][..]),
        (
            "add",
            &["--anchor", "thirty", "--content", "Cap the fee?"][..],
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        letter(dir);
        std::fs::rename(dir.join("a.docx"), dir.join("contract_modified.docx")).unwrap();
        let before = std::fs::read(dir.join("contract_modified.docx")).unwrap();
        let mut args = vec![
            verb,
            "contract_modified.docx",
            "-p",
            "p1",
            "--force",
            "-a",
            "GiantCo",
        ];
        args.extend(content);
        let (code, _, err) = run(dir, &args);
        assert_eq!(code, 0, "{verb}: {err}");
        let line = wrote_any(&err);
        assert!(line.contains("(docx, "), "{verb}: {line}");
        assert!(!line.contains("unchanged"), "{verb}: {line}");
        assert!(!line.contains("in place (replaced"), "{verb}: {line}");
        assert!(!dir.join("contract_modified_modified.docx").exists());
        assert_ne!(
            std::fs::read(dir.join("contract_modified.docx")).unwrap(),
            before
        );
    }
}

/// #1888: `--in-place` is named in the opening of edit/add help and sits on `-o`.
#[test]
fn in_place_sits_beside_output_in_edit_and_add_help() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let help_of = |args: &[&str]| {
        let (code, out, err) = run(dir, args);
        assert_eq!(code, 0, "{out}{err}");
        if out.contains("Usage:") { out } else { err }
    };
    for task in ["edit", "add"] {
        let help = help_of(&[task, "--help"]);
        let about = help.split("Usage:").next().unwrap_or(&help);
        assert!(
            about.contains("--in-place"),
            "{task} about does not name --in-place:\n{about}"
        );
        assert!(
            help.contains("Left unchanged unless --in-place"),
            "{task} still says the file is never modified:\n{help}"
        );
        assert_flag_before_output(&help, task);
    }
    for task in ["accept", "reject"] {
        let help = help_of(&[task, "--help"]);
        assert_flag_before_output(&help, task);
    }
}

fn assert_flag_before_output(help: &str, task: &str) {
    let flag = help
        .find("\n      --in-place\n")
        .unwrap_or_else(|| panic!("{task} help has no --in-place option:\n{help}"));
    let output = help
        .find("\n  -o, --output ")
        .unwrap_or_else(|| panic!("{task} help has no -o option:\n{help}"));
    assert!(
        flag < output && output - flag < 800,
        "{task}: --in-place at {flag}, -o at {output}"
    );
}

/// #1888: the shared parser names the input as the edit/add output.
#[test]
fn parse_json_in_place_names_the_input() {
    let json = jubarte::cli::parse_json(
        &[
            "edit".into(),
            "contract.docx".into(),
            "--in-place".into(),
            "-p".into(),
            "p1".into(),
            "--content".into(),
            "x".into(),
        ],
        "jubarte",
        &[
            "edit".into(),
            "add".into(),
            "accept".into(),
            "reject".into(),
        ],
    );
    let edit: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(edit["exit_code"], 0, "{json}");
    assert_eq!(edit["args"]["output"], "contract.docx", "{json}");
    assert_eq!(edit["args"]["out_dir"], serde_json::Value::Null, "{json}");
    assert_eq!(edit["args"]["in_place"], true, "{json}");

    let json = jubarte::cli::parse_json(
        &["accept".into(), "contract.docx".into(), "--in-place".into()],
        "jubarte",
        &["accept".into()],
    );
    let accept: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(accept["exit_code"], 0, "{json}");
    assert_eq!(accept["args"]["in_place"], true, "{json}");
    assert_eq!(accept["args"]["output"], "contract.docx", "{json}");
    assert_eq!(accept["args"]["output_format"], "docx", "{json}");

    // A DOCX whose name is not .docx is still a DOCX when replaced in place.
    let json = jubarte::cli::parse_json(
        &["accept".into(), "contract.json".into(), "--in-place".into()],
        "jubarte",
        &["accept".into(), "reject".into()],
    );
    let named: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(named["exit_code"], 0, "{json}");
    assert_eq!(named["args"]["output"], "contract.json", "{json}");
    assert_eq!(named["args"]["output_format"], "docx", "{json}");
    let json = jubarte::cli::parse_json(
        &["reject".into(), "notes.md".into(), "--in-place".into()],
        "jubarte",
        &["reject".into()],
    );
    let markdown: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(markdown["args"]["output"], "notes.md", "{json}");
    assert_eq!(markdown["args"]["output_format"], "docx", "{json}");
}

/// #1888: `accept`/`reject --in-place` always write a DOCX, whatever the name.
#[test]
fn in_place_resolution_of_a_misnamed_docx_stays_a_docx() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let packed = redline(dir);
    for (name, cmd) in [("contract.json", "accept"), ("contract.md", "reject")] {
        std::fs::write(dir.join(name), &packed).unwrap();
        let prefix = sha8(&packed);
        let (code, _, err) = run(dir, &[cmd, name, "--in-place"]);
        assert_eq!(code, 0, "{name}: {err}");
        let bytes = std::fs::read(dir.join(name)).unwrap();
        assert!(
            bytes.starts_with(b"PK\x03\x04"),
            "{name} was overwritten as text: {}",
            String::from_utf8_lossy(&bytes[..bytes.len().min(24)])
        );
        assert!(
            wrote_any(&err).starts_with(&format!(
                "wrote {name} in place (replaced sha256 {prefix}\u{2026})"
            )),
            "{err}"
        );
        assert!(!err.contains("(json") && !err.contains("(md,"), "{err}");
    }
}

/// #1888: `--in-place` overwrites only the input. A PDF, a PNG page and a
/// failed-op plan that already exist still need `--force`.
#[test]
fn in_place_does_not_clobber_companions_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    std::fs::write(dir.join("contract.pdf"), b"keep-pdf").unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "Jubarte",
            "--in-place",
            "--pdf",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("OUTPUT_EXISTS") && err.contains("contract.pdf"),
        "{err}"
    );
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_eq!(
        std::fs::read(dir.join("contract.pdf")).unwrap(),
        b"keep-pdf"
    );

    std::fs::write(dir.join("contract-page-01.png"), b"keep-png").unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "Jubarte",
            "--in-place",
            "--png",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("OUTPUT_EXISTS") && err.contains("contract-page-01.png"),
        "{err}"
    );
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_eq!(
        std::fs::read(dir.join("contract-page-01.png")).unwrap(),
        b"keep-png"
    );

    let plan = r#"{"schema_version":1,"author":"GiantCo","operations":[{"kind":"replace","paragraph":"p1","find":"thirty","replacement":"forty"},{"kind":"replace","paragraph":"p1","find":"no-such-anchor","replacement":"x"}]}"#;
    std::fs::write(dir.join("plan.json"), plan).unwrap();
    std::fs::write(dir.join("plan.failed.json"), b"keep-plan").unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "--plan",
            "plan.json",
            "--skip-failed",
            "--in-place",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("OUTPUT_EXISTS") && err.contains("plan.failed.json"),
        "{err}"
    );
    assert_eq!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_eq!(
        std::fs::read(dir.join("plan.failed.json")).unwrap(),
        b"keep-plan"
    );

    std::fs::remove_file(dir.join("contract.pdf")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "--in-place",
            "--pdf",
            "--force",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert_ne!(std::fs::read(dir.join("contract.docx")).unwrap(), before);
    assert_ne!(
        std::fs::read(dir.join("contract.pdf")).unwrap(),
        b"keep-pdf"
    );
    assert!(
        wrote_any(&err).contains("in place (replaced sha256 "),
        "{err}"
    );
}

/// #1888: a companion that cannot be committed does not leave the input replaced.
#[cfg(unix)]
#[test]
fn in_place_unwritable_companion_leaves_the_input() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let before = contract(dir);
    let args: Vec<std::ffi::OsString> = [
        "edit",
        "contract.docx",
        "-p",
        "p1",
        "--anchor",
        "thirty",
        "--content",
        "forty",
        "-a",
        "Jubarte",
        "--in-place",
        "--pdf",
    ]
    .iter()
    .map(std::ffi::OsString::from)
    .collect();
    let mut blocked = false;
    let out = publication_hook(dir, &args, |phase, _| {
        // After the existence check, at the first commit. Making the PDF a
        // directory here fails that commit; the input must still be uncommitted.
        if phase == "checked" && !blocked {
            let pdf = dir.join("contract.pdf");
            std::fs::create_dir(&pdf).unwrap();
            std::fs::write(pdf.join("blocked"), b"x").unwrap();
            blocked = true;
        }
    });
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(blocked, "the companion was made unwritable: {err}");
    assert_ne!(out.status.code(), Some(0), "{err}");
    assert_eq!(
        std::fs::read(dir.join("contract.docx")).unwrap(),
        before,
        "{err}"
    );
}

/// #1888: accept/reject recheck the source at the commit boundary.
#[cfg(unix)]
#[test]
fn in_place_accept_rechecks_the_source_before_replacing() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let packed = redline(dir);
    std::fs::write(dir.join("contract.docx"), &packed).unwrap();
    let replacement = docx(&para("A different named source."));
    let args: Vec<std::ffi::OsString> = ["accept", "contract.docx", "--in-place"]
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let mut injected = false;
    let out = publication_hook(dir, &args, |phase, _| {
        if phase == "staged" {
            std::fs::write(dir.join("contract.docx"), &replacement).unwrap();
            injected = true;
        }
    });
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(injected, "the source mutation ran: {err}");
    assert_eq!(out.status.code(), Some(3), "{err}");
    assert!(err.contains("SOURCE_CHANGED:"), "{err}");
    assert!(
        err.contains("contract.docx") && err.contains("nothing was published"),
        "{err}"
    );
    assert_eq!(
        std::fs::read(dir.join("contract.docx")).unwrap(),
        replacement
    );
    assert!(!err.contains("in place"), "{err}");
}

/// #1888: `--in-place` replaces the symlink's target and keeps its mode.
#[cfg(unix)]
#[test]
fn in_place_follows_a_symlink_and_keeps_its_mode() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    let before = contract(dir);
    let real = dir.join("real.docx");
    std::fs::rename(dir.join("contract.docx"), &real).unwrap();
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o640)).unwrap();
    symlink("real.docx", dir.join("contract.docx")).unwrap();
    let prefix = sha8(&before);
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "contract.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "--in-place",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let link = dir.join("contract.docx");
    assert!(
        link.symlink_metadata().unwrap().file_type().is_symlink(),
        "the link was replaced by a regular file: {err}"
    );
    assert_eq!(std::fs::read_link(&link).unwrap(), Path::new("real.docx"));
    let mode = std::fs::metadata(&real).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640, "mode {mode:o}");
    assert_ne!(std::fs::read(&real).unwrap(), before);
    assert!(
        wrote_any(&err).starts_with(&format!(
            "wrote contract.docx in place (replaced sha256 {prefix}\u{2026})"
        )),
        "{err}"
    );

    // `redline()` always writes `contract.docx`. That name is the symlink
    // this test already published, so the accept fixture uses another name.
    letter(dir);
    std::fs::rename(dir.join("a.docx"), dir.join("source.docx")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit",
            "source.docx",
            "-p",
            "p1",
            "--anchor",
            "thirty",
            "--content",
            "forty",
            "-a",
            "GiantCo",
            "-o",
            "packed.docx",
        ],
    );
    assert_eq!(code, 0, "{err}");
    let packed = std::fs::read(dir.join("packed.docx")).unwrap();
    let accept_real = dir.join("accept-real.docx");
    std::fs::write(&accept_real, &packed).unwrap();
    std::fs::set_permissions(&accept_real, std::fs::Permissions::from_mode(0o604)).unwrap();
    symlink("accept-real.docx", dir.join("accept.docx")).unwrap();
    let accept_prefix = sha8(&packed);
    let (code, _, err) = run(dir, &["accept", "accept.docx", "--in-place"]);
    assert_eq!(code, 0, "{err}");
    assert!(
        dir.join("accept.docx")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink(),
        "{err}"
    );
    let mode = std::fs::metadata(&accept_real)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o604, "mode {mode:o}");
    assert!(
        std::fs::read(&accept_real)
            .unwrap()
            .starts_with(b"PK\x03\x04"),
        "target is not a docx"
    );
    assert_ne!(std::fs::read(&accept_real).unwrap(), packed);
    assert!(
        wrote_any(&err).starts_with(&format!(
            "wrote accept.docx in place (replaced sha256 {accept_prefix}\u{2026})"
        )),
        "{err}"
    );
}
