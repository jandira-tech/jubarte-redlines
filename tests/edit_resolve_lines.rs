// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `edit -p rN --accept/--reject` prints the resolved changes, not the view.

mod common;

fn received_path() -> String {
    format!(
        "{}/tests/fixtures/agent-view/received.docx",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn edit_cli(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .args(args)
        .args(["--datetime", "2026-10-11T00:00:00Z"])
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn edit_accept_prints_the_resolved_insertion_not_the_view() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r0", "--accept", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        stdout, "accepted r0 in p3: \"{++quarterly++}\" > \"quarterly\"\n",
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.starts_with("wrote out.docx (docx, "), "{stderr}");
    assert!(!stdout.contains("source:"), "{stdout}");
}

#[test]
fn edit_reject_prints_the_resolved_insertion_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r0", "--reject", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        stdout, "rejected r0 in p3: \"{++quarterly++}\" > \"\"\n",
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(stderr.starts_with("wrote out.docx (docx, "), "{stderr}");
}

#[test]
fn edit_accept_prints_a_substitution_pair_as_one_line() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r1+2", "--accept", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        stdout, "accepted r1+2 in p3: \"{~~weekly~>monthly~~}\" > \"monthly\"\n",
        "{stdout}"
    );
}

#[test]
fn edit_reject_prints_a_single_deletion_half() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r3", "--reject", "-o", "out.docx"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        stdout, "rejected r3 in p5: \"{--thirty--}\" > \"thirty\"\n",
        "{stdout}"
    );
}

#[test]
fn edit_accept_of_an_unknown_id_keeps_its_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(received_path(), dir.path().join("received.docx")).unwrap();
    let out = edit_cli(
        dir.path(),
        &["received.docx", "-p", "r999", "--accept", "-o", "out.docx"],
    );
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        stdout.contains("UNKNOWN_CHANGE") && stdout.contains("no tracked change r999"),
        "{stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        stderr.contains("error: plan refused: UNKNOWN_CHANGE"),
        "{stderr}"
    );
}

#[test]
fn unchanged_text_between_resolved_changes_prevents_a_substitution() {
    let source = common::docx::docx(
        r#"<w:p><w:del w:id="1" w:author="A"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t> kept </w:t></w:r><w:ins w:id="2" w:author="A"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#,
    );
    let lines = jubarte::edit::resolution::resolution_lines(
        &source,
        &jubarte::edit::ResolvedRevisions {
            accepted: vec!["r1".into(), "r2".into()],
            rejected: vec![],
        },
    )
    .unwrap();
    assert_eq!(
        lines,
        [
            "accepted r1 in p0: \"{--old--}\" > \"\"",
            "accepted r2 in p0: \"{++new++}\" > \"new\"",
        ]
    );
}
