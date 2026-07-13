//! Integration tests for the `redline` binary — drives the compiled CLI to cover
//! the file-I/O `run` path (default output naming, --force, --quiet, errors) and
//! confirms the produced docx is ooxmlsdk-loadable.

mod common;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const ORIG: &str = "tests/fixtures/redline/original.docx";
const MOD: &str = "tests/fixtures/redline/modified.docx";

fn tmpdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Copy the two redline fixtures into `dir` as `a.docx` / `b.docx`.
fn seed(dir: &Path) -> (PathBuf, PathBuf) {
    let a = dir.join("a.docx");
    let b = dir.join("b.docx");
    std::fs::copy(ORIG, &a).unwrap();
    std::fs::copy(MOD, &b).unwrap();
    (a, b)
}

fn assert_loadable(path: &Path) {
    let bytes = std::fs::read(path).unwrap();
    let doc =
        ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument::new(Cursor::new(bytes))
            .unwrap();
    assert!(
        doc.main_document_part().is_ok(),
        "output must be a valid wordprocessing doc"
    );
}

#[test]
fn default_output_name_in_original_dir_and_loadable() {
    let dir = tmpdir();
    let (a, b) = seed(dir.path());
    let out = Command::new(BIN).arg(&a).arg(&b).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let expected = dir.path().join("a_v_b.docx");
    assert!(expected.exists(), "default output a_v_b.docx should exist");
    assert!(String::from_utf8_lossy(&out.stdout).contains("a_v_b.docx"));
    assert_loadable(&expected);
}

#[test]
fn explicit_flags_output_author_and_quiet() {
    let dir = tmpdir();
    let (a, b) = seed(dir.path());
    let out_path = dir.path().join("redline.docx");
    let out = Command::new(BIN)
        .args(["-b"])
        .arg(&a)
        .args(["-m"])
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .args(["--author", "Legal", "--quiet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty(), "--quiet suppresses the success line");
    assert_loadable(&out_path);
}

#[test]
fn refuses_to_overwrite_without_force_then_force_succeeds() {
    let dir = tmpdir();
    let (a, b) = seed(dir.path());
    let out_path = dir.path().join("o.docx");
    // first run creates it
    let first = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .output()
        .unwrap();
    assert!(first.status.success());
    // second run without --force fails (exit 1) and does not clobber
    let second = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&second.stderr).contains("already exists"));
    // third run with --force overwrites
    let third = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .arg("--force")
        .output()
        .unwrap();
    assert!(third.status.success());
    assert_loadable(&out_path);
}

#[test]
fn missing_input_file_errors() {
    let dir = tmpdir();
    let (_, b) = seed(dir.path());
    let out = Command::new(BIN)
        .arg(dir.path().join("nope.docx"))
        .arg(&b)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("reading"));
}

#[test]
fn missing_arguments_exit_code_2() {
    let out = Command::new(BIN).arg(ORIG).output().unwrap();
    assert_eq!(out.status.code(), Some(2), "missing MODIFIED → usage error");
    assert!(String::from_utf8_lossy(&out.stderr).contains("missing MODIFIED"));
}

#[test]
fn help_and_version_exit_zero() {
    for flag in ["--help", "-h", "--version", "-V"] {
        let out = Command::new(BIN).arg(flag).output().unwrap();
        assert!(out.status.success(), "{flag} should exit 0");
        assert!(!out.stdout.is_empty(), "{flag} should print to stdout");
    }
}

// --- gems from recipe PR #60 (revisions subcommand) ---

/// Produce a redline .docx via the plain (subcommand-less) compare surface,
/// for use as input to the `revisions` subcommand tests below.
fn make_redline(dir: &Path, author: &str) -> PathBuf {
    let (a, b) = seed(dir);
    let out_path = dir.join("redline.docx");
    let out = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .args(["--author", author, "--quiet"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out_path
}

/// Prior behavior path: adding the `revisions` subcommand does not disturb
/// the plain compare surface — a normal two-file invocation (whose filenames
/// don't collide with the subcommand name) still runs the compare job, not
/// `run_revisions`.
#[test]
fn plain_compare_surface_still_works_alongside_revisions_subcommand() {
    let dir = tmpdir();
    let (a, b) = seed(dir.path());
    let out = Command::new(BIN).arg(&a).arg(&b).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(dir.path().join("a_v_b.docx").exists());
}

/// D.6 — `redline revisions <file that isn't a docx>` fails (exit code 1)
/// rather than panicking, exercising the `get_revisions failed` error path.
#[test]
fn revisions_subcommand_invalid_docx_errors() {
    let dir = tmpdir();
    let bogus = dir.path().join("bogus.docx");
    std::fs::write(&bogus, b"not a zip").unwrap();
    let out = Command::new(BIN)
        .args(["revisions"])
        .arg(&bogus)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

/// D.6 — the `--json` writer escapes embedded quotes in field values (the
/// `esc` closure in `run_revisions`), so the emitted line stays valid
/// JSON-shaped text rather than corrupting the record.
#[test]
fn revisions_subcommand_json_escapes_quotes_in_author() {
    let dir = tmpdir();
    let (a, b) = seed(dir.path());
    let out_path = dir.path().join("redline.docx");
    let cmp = Command::new(BIN)
        .arg(&a)
        .arg(&b)
        .args(["-o"])
        .arg(&out_path)
        .args(["--author", "Legal \"Team\"", "--quiet"])
        .output()
        .unwrap();
    assert!(cmp.status.success());

    let json = Command::new(BIN)
        .args(["revisions"])
        .arg(&out_path)
        .args(["--json"])
        .output()
        .unwrap();
    assert!(json.status.success());
    let json_out = String::from_utf8_lossy(&json.stdout);
    assert!(
        json_out.contains("Legal \\\"Team\\\""),
        "embedded quotes must be backslash-escaped: {json_out}"
    );
}

/// D.6 — `redline revisions <file>` (human summary) and `--json` report the
/// SAME non-zero revision count for a redline known to carry tracked changes
/// (m5_document_comparer.rs's `compare_documents_produces_redline_zip`
/// confirms this fixture pair emits `w:ins`/`w:del`), with each output mode's
/// per-revision row shaped as documented.
#[test]
fn revisions_subcommand_lists_tracked_changes_in_both_output_modes() {
    let dir = tmpdir();
    let redline_path = make_redline(dir.path(), "Tester");

    let human = Command::new(BIN)
        .args(["revisions"])
        .arg(&redline_path)
        .output()
        .unwrap();
    assert!(
        human.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&human.stderr)
    );
    let human_out = String::from_utf8_lossy(&human.stdout).into_owned();
    let mut lines: Vec<&str> = human_out.lines().collect();
    let summary = lines.pop().expect("summary line present");
    assert!(
        summary.ends_with("revision(s)"),
        "last line is the count summary: {summary:?}"
    );
    let count: usize = summary.split_whitespace().next().unwrap().parse().unwrap();
    assert!(
        count > 0,
        "the redline fixture pair must carry tracked changes: {human_out}"
    );
    assert_eq!(
        lines.len(),
        count,
        "one row per revision before the summary"
    );
    for line in &lines {
        // type \t author \t part \t {preview:?}
        // This 4-field count assumes the fixture's revision authors contain no
        // literal tab (they don't). The human row format is tab-delimited by
        // contract, so a tab inside an author would need run_revisions to escape
        // it (e.g. Debug-format the author) before this could stay parseable.
        assert_eq!(
            line.split('\t').count(),
            4,
            "row has 4 tab-separated fields: {line:?}"
        );
    }

    let json = Command::new(BIN)
        .args(["revisions"])
        .arg(&redline_path)
        .args(["--json"])
        .output()
        .unwrap();
    assert!(
        json.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json.stderr)
    );
    let json_out = String::from_utf8_lossy(&json.stdout).into_owned();
    let json_lines: Vec<&str> = json_out.lines().collect();
    assert_eq!(
        json_lines.len(),
        count,
        "same revision count in both output modes"
    );
    for line in &json_lines {
        assert!(
            line.starts_with('{') && line.ends_with('}'),
            "JSON-lines row: {line:?}"
        );
        assert!(line.contains("\"type\":"));
        assert!(line.contains("\"author\":\"Tester\""));
        assert!(line.contains("\"part\":\"word/document.xml\""));
    }
}

/// D.6 — `redline revisions <missing file>` fails like the plain compare
/// surface's missing-input case: exit code 1, stderr names the read failure.
#[test]
fn revisions_subcommand_missing_file_errors() {
    let dir = tmpdir();
    let out = Command::new(BIN)
        .args(["revisions"])
        .arg(dir.path().join("nope.docx"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("reading"));
}

// --- gems from recipe PR #71 (word-visual default CLI parity), adapted to main's Option threshold ---

fn compare_with(settings: &jubarte::comparer::WmlComparerSettings) -> Vec<u8> {
    let orig = std::fs::read(ORIG).unwrap();
    let modi = std::fs::read(MOD).unwrap();
    jubarte::document_comparer::compare_documents_with_settings(&orig, &modi, settings).unwrap()
}

/// Build settings matching CLI `run()` for (`powertools_faithful`, optional threshold).
fn expected_settings(
    powertools_faithful: bool,
    cli_detail_threshold: Option<f64>,
) -> jubarte::comparer::WmlComparerSettings {
    let base = if powertools_faithful {
        jubarte::comparer::WmlComparerSettings::powertools_faithful()
    } else {
        jubarte::comparer::WmlComparerSettings::default()
    };
    jubarte::comparer::WmlComparerSettings {
        author_for_revisions: "Redline".to_string(),
        date_time_for_revisions: "1970-01-01T00:00:00Z".to_string(),
        detail_threshold: cli_detail_threshold.unwrap_or(base.detail_threshold),
        ..base
    }
}

/// Run the binary against ORIG/MOD with extra flags; return produced bytes.
fn run_cli(out_path: &Path, extra: &[&str]) -> Vec<u8> {
    let out = Command::new(BIN)
        .arg(ORIG)
        .arg(MOD)
        .args(["-o"])
        .arg(out_path)
        .arg("--quiet")
        .args(extra)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read(out_path).unwrap()
}

#[test]
fn default_cli_run_matches_word_visual_library_default() {
    let dir = tmpdir();
    let out_path = dir.path().join("out.docx");
    let cli_bytes = run_cli(&out_path, &[]);
    let lib_bytes = compare_with(&expected_settings(false, None));
    common::assert_docx_structurally_eq(&cli_bytes, &lib_bytes);
    assert_loadable(&out_path);
}

#[test]
fn detail_threshold_override_layers_on_word_visual_default() {
    let dir = tmpdir();
    let out_path = dir.path().join("out.docx");
    let cli_bytes = run_cli(&out_path, &["--detail-threshold", "0.3"]);
    let lib_bytes = compare_with(&expected_settings(false, Some(0.3)));
    common::assert_docx_structurally_eq(&cli_bytes, &lib_bytes);
    assert_loadable(&out_path);
}

#[test]
fn powertools_faithful_flag_matches_library_preset() {
    let dir = tmpdir();
    let out_path = dir.path().join("out.docx");
    let cli_bytes = run_cli(&out_path, &["--powertools-faithful"]);
    let lib_bytes = compare_with(&expected_settings(true, None));
    common::assert_docx_structurally_eq(&cli_bytes, &lib_bytes);
    assert_loadable(&out_path);
}

#[test]
fn powertools_faithful_with_explicit_threshold_uses_override() {
    let dir = tmpdir();
    let out_path = dir.path().join("out.docx");
    let cli_bytes = run_cli(
        &out_path,
        &["--powertools-faithful", "--detail-threshold", "0.3"],
    );
    let lib_bytes = compare_with(&expected_settings(true, Some(0.3)));
    common::assert_docx_structurally_eq(&cli_bytes, &lib_bytes);
    assert_loadable(&out_path);
}
