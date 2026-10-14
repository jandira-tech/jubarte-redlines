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
    // The source sits apart from the Markdown, so automatic anchoring (spec
    // §5.3) can never make this test measure nothing.
    let source = std::fs::read("tests/fixtures/agent-view/received.docx").unwrap();
    std::fs::create_dir_all(dir.join("source")).unwrap();
    std::fs::write(dir.join("source/received.docx"), &source).unwrap();
    let view = jubarte(dir, &["read", "source/received.docx"]);
    std::fs::write(dir.join("view.md"), view).unwrap();
    // The epoch pinned: unpinned, an undated change is dated now.
    jubarte(
        dir,
        &[
            "convert",
            "view.md",
            "-o",
            "back.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
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

/// An author with several dates is listed as a day range (`2016-07-03..
/// 2016-07-03`), which names no date: their notes are written undated,
/// never with the conversion's 1970-01-01 (pi review of #462: 21 of 22
/// such corpus files were re-dated to the epoch).
#[test]
fn an_author_with_several_dates_is_never_dated_1970() {
    let dir = tempfile::tempdir().unwrap();
    let view = "---\nview: tracked\ncomments: 2 threads open\nauthors:\n  MW: Michael Williamson           # 2 comments, 2016-07-03..2016-07-03\n---\n\n<!-- p0 -->\nA tachyon{== walks==}{>>c0 @MW: A tachyon walks into a bar.<<} in.{== Fin==}{>>c1 @MW: Fin.<<}\n";
    std::fs::write(dir.path().join("view.md"), view).unwrap();
    jubarte(
        dir.path(),
        &[
            "convert",
            "view.md",
            "-o",
            "back.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let back = std::fs::read(dir.path().join("back.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&back).unwrap();
    assert_eq!(comments.len(), 2, "{comments:?}");
    for c in comments {
        assert_eq!(c.author, "Michael Williamson");
        assert!(
            !c.date.as_deref().unwrap_or_default().starts_with("1970"),
            "{c:?}"
        );
    }
}
