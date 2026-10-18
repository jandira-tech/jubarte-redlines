// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Old attribution wrappers fail with a migration message at both entry points.
mod common;
use jubarte::markdown::{DocxOptions, markdown_to_docx};
use std::process::Command;

fn assert_refused(markdown: &str, old: &str, new: &str) {
    let result = markdown_to_docx(markdown, &DocxOptions::default());
    assert!(result.is_err(), "old attribution must fail");
    let error = result.unwrap_err().to_string();
    for message in ["line 2", old, new, "{>>c1 @X: r1 @ASR<<}"] {
        assert!(error.contains(message), "missing {message:?}: {error}");
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("old.md"), markdown).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["convert", "old.md", "-o", "back.docx"])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "old attribution converted successfully"
    );
    assert!(
        !dir.path().join("back.docx").exists(),
        "refused input must not produce a document"
    );
    let error = String::from_utf8(output.stderr).unwrap();
    for message in ["line 2", old, new, "{>>c1 @X: r1 @ASR<<}"] {
        assert!(error.contains(message), "missing {message:?}: {error}");
    }
}

#[test]
fn old_attribution_is_refused_after_each_closer_and_emphasis() {
    for mark in ["{++new++}", "{--old--}", "{~~old~>new~~}"] {
        for emphasis in ["", "**", "_", "`", "*_`"] {
            for (old, new) in [
                ("{>>r1 @ASR<<}", "{r1 @ASR}"),
                ("{>>header1.r1 @ASR<<}", "{header1.r1 @ASR}"),
                ("{>>r1+2 @ASR<<}", "{r1+2 @ASR}"),
                (
                    "{>>r1+2+3 @ASR on r4 @BO on header1.r5 @AC<<}",
                    "{r1+2+3 @ASR re r4 @BO re header1.r5 @AC}",
                ),
            ] {
                assert_refused(&format!("Intro\n{mark}{emphasis}{old}\n"), old, new);
            }
        }
    }
}

#[test]
fn explicit_free_and_nonadjacent_old_looking_comments_still_read() {
    for (source, comment) in [
        ("{++new++}{r1 Ann Lee}{>>c5 @AC: note<<}", Some("note")),
        ("{++new++}{r1 Ann Lee}{>>c1 @X: r1 @ASR<<}", Some("r1 @ASR")),
        ("{++new++}{>>a free comment<<}", Some("a free comment")),
        ("text {>>r1 @ASR<<}", Some("r1 @ASR")),
        ("{++new++} {>>r1 @ASR<<}", Some("r1 @ASR")),
        (r"{++new++}\{>>r1 @ASR<<}", None),
        ("`{++new++}{>>r1 @ASR<<}`", None),
        ("{>>quoted {++new++}{>>r1 @ASR<<}", Some("r1 @ASR")),
    ] {
        let docx = markdown_to_docx(source, &DocxOptions::default())
            .expect(source)
            .docx;
        assert!(common::docx::part_string(&docx, "word/document.xml").is_some());
        if let Some(text) = comment {
            let comments =
                common::docx::part_string(&docx, "word/comments.xml").expect("comment part");
            assert!(comments.contains(text), "{source}: {comments}");
        }
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("comments.md"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .current_dir(dir.path())
            .args(["convert", "comments.md", "-o", "back.docx"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
