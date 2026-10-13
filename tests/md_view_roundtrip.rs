// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The view (`jubarte read`) read back as Markdown: what the view writes,
//! `convert X.md -o X.docx` reads (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md).

use std::path::Path;
use std::process::Command;

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// `received.docx` → its view → a docx again.
fn round_trip(dir: &Path) -> (Vec<u8>, Vec<u8>) {
    let source = std::fs::read("tests/fixtures/agent-view/received.docx").unwrap();
    std::fs::write(dir.join("received.docx"), &source).unwrap();
    let view = jubarte(dir, &["read", "received.docx"]);
    std::fs::write(dir.join("view.md"), view).unwrap();
    jubarte(dir, &["convert", "view.md", "-o", "back.docx"]);
    (source, std::fs::read(dir.join("back.docx")).unwrap())
}

/// Each change's kind, author, date and text.
fn changes(docx: &[u8]) -> Vec<String> {
    jubarte::changes::list_changes(docx)
        .unwrap()
        .into_iter()
        .map(|c| format!("{:?} {:?} {:?} {:?}", c.kind, c.author, c.date, c.text))
        .collect()
}

/// A revision's note (`{>>r1+2 @AC<<}`) names its author through the
/// header's `authors:` table, and its date: it never becomes a comment.
#[test]
fn revision_notes_keep_their_author_and_date() {
    let dir = tempfile::tempdir().unwrap();
    let (source, back) = round_trip(dir.path());
    assert_eq!(changes(&back), changes(&source));
}

/// A comment's note (`{>>c5 @AC: text<<}`, `{>>c6 @AS re c5: text<<}`)
/// keeps its author, initials, date and text, and nothing else turns into a
/// comment.
#[test]
fn comment_notes_keep_their_author_initials_and_date() {
    let dir = tempfile::tempdir().unwrap();
    let (source, back) = round_trip(dir.path());
    let comments = |docx: &[u8]| -> Vec<String> {
        jubarte::comments::list_comments(docx)
            .unwrap()
            .into_iter()
            .map(|c| format!("{} {:?} {:?} {:?}", c.author, c.initials, c.date, c.text))
            .collect()
    };
    assert_eq!(comments(&back), comments(&source));
}
