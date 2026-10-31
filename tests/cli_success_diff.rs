// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration contract for confirmations rendered from published bytes.
mod common;
use common::docx::{docx, para};
use std::path::Path;
use std::process::{Command, Output};
const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn run(dir: &Path, args: &[&str]) -> Output {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}
fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    std::fs::create_dir(p.join("inputs")).unwrap();
    std::fs::write(p.join("inputs/in.docx"), docx(&para("Pay thirty dollars."))).unwrap();
    std::fs::write(p.join("b.docx"), docx(&para("Pay forty dollars."))).unwrap();
    std::fs::write(p.join("annex.docx"), docx(&para("Annex."))).unwrap();
    std::fs::write(p.join("tracked.docx"), docx(r#"<w:p><w:r><w:t xml:space="preserve">Pay </w:t></w:r><w:ins w:id="1" w:author="Alice" w:date="2026-10-17T00:00:00Z"><w:r><w:t>forty</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> dollars.</w:t></w:r></w:p>"#)).unwrap();
    std::fs::write(p.join("in.md"), "Pay {++forty++} dollars.\n").unwrap();
    std::fs::copy(p.join("in.md"), p.join("raw.docx")).unwrap();
    std::fs::copy(p.join("tracked.docx"), p.join("tracked.md")).unwrap();
    dir
}
fn read(dir: &Path, file: &str) -> String {
    String::from_utf8(run(dir, &["read", file]).stdout).unwrap()
}
fn check(args: &[&str], input: &str, outputs: &[&str], removed: &str, added: &str) {
    let dir = setup();
    let p = dir.path();
    let markdown_input = matches!(args[0], "convert" | "compare")
        && (input.ends_with(".md") || args.windows(2).any(|pair| pair == ["--from", "md"]));
    let before = if markdown_input {
        std::fs::read_to_string(p.join(input)).unwrap()
    } else {
        read(p, input)
    };
    let out = run(p, args);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout.lines().filter(|line| *line == "SUCCESS:").count(),
        outputs.len(),
        "{stdout}"
    );
    assert_eq!(
        stdout.matches("~~~~diff\n").count(),
        outputs.len(),
        "{stdout}"
    );
    assert_eq!(
        stdout.lines().filter(|line| *line == "~~~~").count(),
        outputs.len(),
        "{stdout}"
    );
    for output in outputs {
        let after = read(p, output);
        assert!(
            stdout.contains(&format!("--- before {input}\n+++ after {output}\n@@ ")),
            "{stdout}"
        );
        // Independent GNU/BSD diff oracle: the input view uses read's basename,
        // whereas the labels preserve the complete command-line spelling.
        std::fs::write(p.join("before.md"), &before).unwrap();
        std::fs::write(p.join("after.md"), &after).unwrap();
        let diff = Command::new("diff")
            .current_dir(p)
            .args([
                "-u",
                "--label",
                &format!("before {input}"),
                "--label",
                &format!("after {output}"),
                "before.md",
                "after.md",
            ])
            .output()
            .unwrap();
        assert_eq!(diff.status.code(), Some(1));
        let expected = String::from_utf8(diff.stdout).unwrap();
        assert!(
            stdout.contains(&format!("SUCCESS:\n~~~~diff\n{expected}~~~~\n")),
            "{stdout}\nexpected:\n{expected}"
        );
    }
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('-') && l.contains(removed)),
        "missing removed {removed}: {stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with('+') && l.contains(added)),
        "missing added {added}: {stdout}"
    );
    // Repeat the exact operation with quiet and a permitted replacement.
    let quiet = run(p, &[args, &["--force", "-q"]].concat());
    assert!(quiet.stdout.is_empty(), "{:?}", quiet.stdout);
    assert!(
        quiet.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&quiet.stderr)
    );
}
#[test]
fn accept() {
    check(
        &["accept", "tracked.docx", "-o", "out.docx"],
        "tracked.docx",
        &["out.docx"],
        "{++forty++}",
        "Pay forty dollars.",
    );
}
#[test]
fn reject() {
    check(
        &["reject", "tracked.docx", "-o", "out.docx"],
        "tracked.docx",
        &["out.docx"],
        "{++forty++}",
        "Pay  dollars.",
    );
}
#[test]
fn scrub() {
    check(
        &[
            "scrub",
            "tracked.docx",
            "--author-alias",
            "Bob",
            "-o",
            "out.docx",
        ],
        "tracked.docx",
        &["out.docx"],
        "Alice",
        "Bob",
    );
}
#[test]
fn append() {
    check(
        &["append", "inputs/in.docx", "annex.docx", "-o", "out.docx"],
        "inputs/in.docx",
        &["out.docx"],
        "source: in.docx",
        "Annex.",
    );
}
#[test]
fn compare() {
    check(
        &["compare", "inputs/in.docx", "b.docx", "-o", "out.docx"],
        "inputs/in.docx",
        &["out.docx"],
        "Pay thirty dollars.",
        "forty",
    );
}
#[test]
fn convert_markdown() {
    check(
        &["convert", "in.md", "-o", "out.docx"],
        "in.md",
        &["out.docx"],
        "Pay {++forty++} dollars.",
        "forty",
    );
}
#[test]
fn edit_flags_single() {
    check(
        &[
            "edit",
            "inputs/in.docx",
            "-p",
            "p0",
            "--content",
            "Pay forty dollars.",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "Pay thirty dollars.",
        "forty",
    );
}
#[test]
fn edit_flags_bundle() {
    check(
        &[
            "edit",
            "inputs/in.docx",
            "-p",
            "p0",
            "--content",
            "Pay forty dollars.",
            "-a",
            "Jubarte",
            "--out-dir",
            "bundle",
        ],
        "inputs/in.docx",
        &["bundle/clean.docx", "bundle/redline.docx"],
        "Pay thirty dollars.",
        "forty",
    );
}
#[test]
fn add() {
    check(
        &[
            "add",
            "inputs/in.docx",
            "-p",
            "p0",
            "--content",
            "Annex.",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "source: in.docx",
        "Annex.",
    );
}
#[test]
fn typed_replace() {
    check(
        &[
            "edit",
            "replace",
            "inputs/in.docx",
            "--at",
            "p0",
            "--find",
            "thirty",
            "--text",
            "forty",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "Pay thirty dollars.",
        "forty",
    );
}
#[test]
fn edit_template_then_apply() {
    // Template generation writes JSON only; the document-writing path is apply.
    let dir = setup();
    let p = dir.path();
    let generated = run(
        p,
        &["edit", "template", "inputs/in.docx", "-o", "plan.json"],
    );
    assert!(!String::from_utf8_lossy(&generated.stdout).contains("SUCCESS:"));
    let mut plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(p.join("plan.json")).unwrap()).unwrap();
    plan["operations"][0]["text"] = "Pay forty dollars.".into();
    std::fs::write(p.join("plan.json"), serde_json::to_vec(&plan).unwrap()).unwrap();
    let out = run(
        p,
        &[
            "edit",
            "apply",
            "inputs/in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
        ],
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains("SUCCESS:\n~~~~diff\n--- before inputs/in.docx\n+++ after out.docx\n@@ "),
        "{text}"
    );
    assert!(text.contains("-Pay thirty dollars."), "{text}");
    assert!(
        text.lines()
            .any(|l| l.starts_with('+') && l.contains("Pay {~~thi~>fo~~}")),
        "{text}"
    );
    assert!(text.ends_with("~~~~\n"));
    let quiet = run(
        p,
        &[
            "edit",
            "apply",
            "inputs/in.docx",
            "--plan",
            "plan.json",
            "-o",
            "out.docx",
            "--force",
            "-q",
        ],
    );
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());
}
#[test]
fn unchanged_accept_still_confirms_source_and_hash() {
    let dir = setup();
    let out = run(dir.path(), &["accept", "inputs/in.docx", "-o", "out.docx"]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("SUCCESS:\n~~~~diff\n"), "{text}");
    assert!(
        text.contains("-source: in.docx\n+source: out.docx\n")
            || text.contains("-source: in.docx\n-sha256:"),
        "{text}"
    );
}
#[test]
fn equal_views_have_no_differences_body() {
    let dir = setup();
    let p = dir.path();
    std::fs::create_dir(p.join("after")).unwrap();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/re-dialect/step0.docx"
        ),
        p.join("step0.docx"),
    )
    .unwrap();
    let out = run(p, &["accept", "step0.docx", "-o", "after/step0.docx"]);
    assert_eq!(read(p, "step0.docx"), read(p, "after/step0.docx"));
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "SUCCESS:\n~~~~diff\nno differences\n~~~~\n"
    );
}
#[test]
fn convert_word() {
    check(
        &[
            "convert",
            "tracked.docx",
            "--track-changes",
            "accept",
            "-o",
            "out.docx",
        ],
        "tracked.docx",
        &["out.docx"],
        "{++forty++}",
        "Pay forty dollars.",
    );
}
#[test]
fn convert_refresh_fields() {
    check(
        &[
            "convert",
            "inputs/in.docx",
            "--update-fields",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "source: in.docx",
        "source: out.docx",
    );
}
#[test]
fn typed_rewrite() {
    check(
        &[
            "edit",
            "rewrite",
            "inputs/in.docx",
            "--at",
            "p0",
            "--text",
            "Pay forty dollars.",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "Pay thirty dollars.",
        "Pay {~~thi~>fo~~}",
    );
}
#[test]
fn typed_insert() {
    check(
        &[
            "edit",
            "insert",
            "inputs/in.docx",
            "--kind",
            "paragraph",
            "--position",
            "after",
            "--at",
            "p0",
            "--text",
            "Annex.",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "source: in.docx",
        "Annex.",
    );
}
#[test]
fn typed_delete() {
    check(
        &[
            "edit",
            "delete",
            "inputs/in.docx",
            "--at",
            "p0",
            "--find",
            "thirty",
            "-a",
            "Jubarte",
            "-o",
            "out.docx",
        ],
        "inputs/in.docx",
        &["out.docx"],
        "Pay thirty dollars.",
        "Pay {--thirty--}",
    );
}
#[test]
fn refresh_fields_quiet() {
    let dir = setup();
    let out = run(
        dir.path(),
        &[
            "convert",
            "inputs/in.docx",
            "--update-fields",
            "-o",
            "out.docx",
            "-q",
        ],
    );
    assert!(
        out.stdout.is_empty() && out.stderr.is_empty(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn dash_is_a_literal_docx_output() {
    // This writer has never streamed DOCX to stdout; a dash names a file.
    check(
        &["accept", "tracked.docx", "-o", "-"],
        "tracked.docx",
        &["-"],
        "{++forty++}",
        "Pay forty dollars.",
    );
}

#[test]
fn convert_explicit_markdown_input() {
    check(
        &["convert", "raw.docx", "--from", "md", "-o", "out.docx"],
        "raw.docx",
        &["out.docx"],
        "Pay {++forty++} dollars.",
        "forty",
    );
}
#[test]
fn accept_word_package_with_markdown_extension() {
    check(
        &["accept", "tracked.md", "-o", "out.docx"],
        "tracked.md",
        &["out.docx"],
        "{++forty++}",
        "Pay forty dollars.",
    );
}
