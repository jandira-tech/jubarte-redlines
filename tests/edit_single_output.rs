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

const EDIT: [&str; 6] = [
    "edit",
    "a.docx",
    "-p",
    "p1",
    "--content",
    "The fee is forty dollars.",
];

#[test]
fn edit_writes_file_modified_docx_and_prints_its_view() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let (code, out, err) = run(dir, &EDIT);
    assert_eq!(code, 0, "{err}");
    assert_eq!(written(dir), ["a_modified.docx"]);
    let xml = document_xml(&dir.join("a_modified.docx"));
    assert!(xml.contains("<w:ins "), "a redline by default: {xml}");
    assert!(out.contains("\nsource: a_modified.docx\n"), "{out}");
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
    assert!(out.contains("\nsource: a_modified.docx\n"), "{out}");
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
        &["add", "a.docx", "-p", "p2", "--content", "New line."],
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
    assert!(out.contains("\nsource: a.edit/redline.docx\n"), "{out}");

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
    assert!(out.contains("\nsource: b.docx\n"), "{out}");
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
        out.starts_with("{") && out.contains("\nsource: r/redline.docx\n"),
        "{out}"
    );
    assert!(!out.contains("\nwrote "), "{out}");
}

/// #568: the view's `source:` line and the bundle's report name the
/// written files with `/` on every OS — an agent reads them back —
/// whatever spelling FILE was given in. `d\a.docx` is a Windows path;
/// on a Unix it is one odd file name, so the test runs on both.
#[test]
fn the_views_source_line_spells_paths_with_forward_slashes() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    std::fs::create_dir_all(dir.join("d")).unwrap();
    std::fs::rename(dir.join("a.docx"), dir.join("d\\a.docx")).unwrap();
    let edit = ["edit", "d\\a.docx", "-p", "p1", "--content", "New line."];
    let (code, out, err) = run(dir, &edit);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\nsource: d/a_modified.docx\n"), "{out}");
    let mut bundle = edit.to_vec();
    bundle.push("--out-dir");
    let (code, out, err) = run(dir, &bundle);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\nsource: d/a.edit/redline.docx\n"), "{out}");
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
    for spelling in ["a.docx", "./a.docx", "sub/../a.docx"] {
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
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("already exists"), "{err}");
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
            "--out-dir",
            "r",
            "--force",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("OUTPUT_DIR_NOT_EMPTY"), "{err}");
    assert_eq!(sha256(&dir.join("r/src/a.docx")), source);
}

/// V2-05: a failure before the commit writes neither the output nor its
/// staging file.
#[cfg(unix)]
#[test]
fn a_failed_commit_leaves_no_output_or_staging_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path();
    letter(dir);
    let ro = dir.join("ro");
    std::fs::create_dir(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    let mut args = EDIT.to_vec();
    args.extend(["-o", "ro/out.docx"]);
    let (code, _, err) = run(dir, &args);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("ro/out.docx"), "{err}");
    let left: Vec<_> = std::fs::read_dir(&ro).unwrap().collect();
    assert!(left.is_empty(), "no output and no staging file: {left:?}");
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).unwrap();
}
