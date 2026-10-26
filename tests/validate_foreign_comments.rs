// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `validate --original --author NAME` reads comments as well as
//! revisions (#1846): another author's comment that is gone, reworded,
//! resolved or reopened, and a new comment or reply not signed NAME, are
//! `FOREIGN_COMMENT` findings. NAME's own comments change freely, and a
//! comment keeps its identity when its id is renumbered.

mod common;

use common::docx::{Part, W_NS, docx_with};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::validate::{Finding, audit_tracked};

const COMMENTS: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";

/// One paragraph per `(id, author, text)`, each carrying its comment; the
/// paragraphs read `Clause 1.`, `Clause 2.` whatever the ids.
fn commented(comments: &[(u32, &str, &str)]) -> Vec<u8> {
    let body: String = comments
        .iter()
        .enumerate()
        .map(|(n, (id, _, _))| {
            format!(
                r#"<w:p><w:commentRangeStart w:id="{id}"/><w:r><w:t>Clause {}.</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r></w:p>"#,
                n + 1
            )
        })
        .collect();
    let definitions: String = comments
        .iter()
        .map(|(id, author, text)| {
            format!(
                r#"<w:comment w:id="{id}" w:author="{author}" w:date="2026-10-18T12:00:00Z"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
            )
        })
        .collect();
    docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &format!(r#"<w:comments xmlns:w="{W_NS}">{definitions}</w:comments>"#),
        }],
    )
}

/// `source` edited by `author` with `operations`, opted in to change
/// other authors' comments; the clean copy.
fn edited(source: &[u8], author: &str, operations: &serde_json::Value) -> Vec<u8> {
    let plan = serde_json::json!({
        "schema_version": 1,
        "author": author,
        "date": "2026-10-20T09:00:00Z",
        "rewrite_others_comments": true,
        "operations": operations,
    });
    apply_plan(source, &EditPlan::from_json(&plan.to_string()).unwrap())
        .unwrap()
        .clean
}

/// The `FOREIGN_COMMENT` messages of auditing `edited` against `original`
/// for `author`.
fn foreign(original: &[u8], edited: &[u8], author: &str) -> Vec<String> {
    let findings = audit_tracked(original, edited, author).unwrap();
    for f in &findings {
        assert!(
            f.code == "FOREIGN_COMMENT",
            "only comments changed, yet {f:?}"
        );
        assert_eq!((f.set.as_str(), f.severity.as_str()), ("edits", "error"));
        assert_eq!(f.part, "word/comments.xml");
    }
    findings.into_iter().map(|f: Finding| f.message).collect()
}

fn bobs() -> Vec<u8> {
    commented(&[(0, "Bob", "Too low.")])
}

#[test]
fn another_authors_comment_gone_reworded_or_resolved_is_a_finding() {
    for (operation, message) in [
        (
            serde_json::json!({"kind": "delete_comment", "comment_id": 0}),
            r#"c0 by "Bob" is gone: reply to their comment instead of deleting it"#,
        ),
        (
            serde_json::json!({"kind": "edit_comment", "comment_id": 0, "text": "Fine."}),
            r#"c0 by "Bob" was reworded: reply to their comment instead of changing its words"#,
        ),
        (
            serde_json::json!({"kind": "resolve_comment", "comment_id": 0}),
            r#"c0 by "Bob" was resolved: reply to their comment instead of resolving it"#,
        ),
    ] {
        let mine = edited(&bobs(), "Me", &serde_json::json!([operation]));
        assert_eq!(foreign(&bobs(), &mine, "Me"), [message], "{operation}");
    }
}

#[test]
fn reopening_another_authors_comment_is_a_finding() {
    let resolved = edited(
        &bobs(),
        "Bob",
        &serde_json::json!([{"kind": "resolve_comment", "comment_id": 0}]),
    );
    let reopened = edited(
        &resolved,
        "Me",
        &serde_json::json!([{"kind": "resolve_comment", "comment_id": 0, "done": false}]),
    );
    assert_eq!(
        foreign(&resolved, &reopened, "Me"),
        [r#"c0 by "Bob" was reopened: reply to their comment instead of reopening it"#]
    );
}

#[test]
fn a_new_comment_not_signed_by_the_author_is_a_finding() {
    let signed = |author: &str| {
        edited(
            &bobs(),
            author,
            &serde_json::json!([
                {"kind": "comment", "paragraph": "p0", "find": "Clause", "text": "Our note."},
                {"kind": "reply_comment", "comment_id": 0, "text": "Our reply."},
            ]),
        )
    };
    // In the order `jubarte comments` lists them: c2 replies in Bob's
    // thread, before c1.
    assert_eq!(
        foreign(&bobs(), &signed("Jubarte"), "Me"),
        [
            r#"c2 is new and signed "Jubarte", not "Me""#,
            r#"c1 is new and signed "Jubarte", not "Me""#,
        ]
    );
    assert!(foreign(&bobs(), &signed("Me"), "Me").is_empty());
}

#[test]
fn the_authors_own_comments_change_freely() {
    let source = commented(&[(0, "Me", "Mine."), (1, "Bob", "Theirs.")]);
    for operation in [
        serde_json::json!({"kind": "delete_comment", "comment_id": 0}),
        serde_json::json!({"kind": "edit_comment", "comment_id": 0, "text": "Mine, reworded."}),
        serde_json::json!({"kind": "resolve_comment", "comment_id": 0}),
        serde_json::json!({"kind": "reply_comment", "comment_id": 1, "text": "Agreed."}),
    ] {
        let mine = edited(&source, "Me", &serde_json::json!([operation]));
        assert!(foreign(&source, &mine, "Me").is_empty(), "{operation}");
    }
}

#[test]
fn a_thread_follows_its_root() {
    // Bob replied to Me's c0. Resolving Me's thread marks Bob's reply done
    // too, which is Me's to do; deleting the thread deletes Bob's reply.
    let source = edited(
        &commented(&[(0, "Me", "Mine.")]),
        "Bob",
        &serde_json::json!([{"kind": "reply_comment", "comment_id": 0, "text": "No."}]),
    );
    let resolved = edited(
        &source,
        "Me",
        &serde_json::json!([{"kind": "resolve_comment", "comment_id": 0}]),
    );
    assert!(foreign(&source, &resolved, "Me").is_empty());
    let deleted = edited(
        &source,
        "Me",
        &serde_json::json!([{"kind": "delete_comment", "comment_id": 0}]),
    );
    assert_eq!(
        foreign(&source, &deleted, "Me"),
        [r#"c1 by "Bob" is gone: reply to their comment instead of deleting it"#]
    );
}

#[test]
fn a_renumbered_comment_keeps_its_identity() {
    let original = commented(&[(0, "Bob", "Too low."), (1, "Carol", "Why?")]);
    // The same comments under other ids, as another tool may save them.
    let renumbered = commented(&[(7, "Bob", "Too low."), (9, "Carol", "Why?")]);
    assert!(foreign(&original, &renumbered, "Me").is_empty());
    // A reworded one still pairs, by author and date.
    let reworded = commented(&[(7, "Bob", "Too low."), (9, "Carol", "Why not?")]);
    assert_eq!(
        foreign(&original, &reworded, "Me"),
        [r#"c9 by "Carol" was reworded: reply to their comment instead of changing its words"#]
    );
}

#[test]
fn a_comment_that_names_no_author_is_another_authors() {
    let original = commented(&[(0, "", "Unsigned.")]);
    let mine = edited(
        &original,
        "Me",
        &serde_json::json!([{"kind": "delete_comment", "comment_id": 0}]),
    );
    assert_eq!(
        foreign(&original, &mine, "Me"),
        ["c0, which names no author, is gone: reply to it instead of deleting it"]
    );
    // A new one is not the author's either: Carol's c1 gave way to it.
    let carols = commented(&[(0, "Bob", "Too low."), (1, "Carol", "Why?")]);
    let unsigned = commented(&[(0, "Bob", "Too low."), (1, "", "Why?")]);
    assert_eq!(
        foreign(&carols, &unsigned, "Me"),
        [
            r#"c1 by "Carol" is gone: reply to their comment instead of deleting it"#,
            r#"c1 is new and names no author, not "Me""#,
        ]
    );
}

#[test]
fn the_cli_fails_on_a_foreign_comment() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("counter.docx"), bobs()).unwrap();
    let mine = edited(
        &bobs(),
        "Me",
        &serde_json::json!([{"kind": "resolve_comment", "comment_id": 0}]),
    );
    std::fs::write(dir.path().join("mine.docx"), mine).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "validate",
            "mine.docx",
            "--original",
            "counter.docx",
            "--author",
            "Me",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2), "{stdout}");
    assert!(
        stdout.contains(r#"FOREIGN_COMMENT"#) && stdout.contains(r#"c0 by "Bob" was resolved"#),
        "{stdout}"
    );
}

/// The input holds no comment finding of its own, so against `--baseline`
/// every `FOREIGN_COMMENT` is the edit's: new, and the run fails.
#[test]
fn a_foreign_comment_is_new_against_the_baseline() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), bobs()).unwrap();
    let mine = edited(
        &bobs(),
        "Me",
        &serde_json::json!([{"kind": "delete_comment", "comment_id": 0}]),
    );
    std::fs::write(dir.path().join("out.docx"), mine).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "validate",
            "out.docx",
            "--baseline",
            "in.docx",
            "--author",
            "Me",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2), "{stdout}");
    assert!(stdout.starts_with("new: "), "{stdout}");
    assert!(stdout.contains(r#"c0 by "Bob" is gone"#), "{stdout}");
    assert!(stdout.contains("already in the input: none"), "{stdout}");
}
