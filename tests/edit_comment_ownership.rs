// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment id is an identifier, and a comment speaks for its author
//! (#1264). Settling revisions never renumbers the comments that stay, so
//! a reply written after a bare accept lands in the thread the agent read;
//! `edit_comment` on another author's comment is refused unless the plan
//! asks to rewrite it under their name.

mod common;

use common::docx::{Part, W_NS, docx_with};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, reject_changes};
use jubarte::comments::list_comments;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use std::path::Path;
use std::process::{Command, Output};

const BOB: &str = r#"w:author="Bob" w:date="2026-10-18T12:00:00Z""#;

const COMMENTS: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";

/// A paragraph whose whole text `text` carries comment `id`.
fn commented(id: u32, text: &str) -> String {
    format!(
        r#"<w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
    )
}

/// `body` with one comment definition per `(id, author, text)`.
fn with_comments(body: &str, definitions: &[(u32, &str, &str)]) -> Vec<u8> {
    let definitions: String = definitions
        .iter()
        .map(|(id, author, text)| {
            format!(
                r#"<w:comment w:id="{id}" w:author="{author}" w:date="2026-10-18T12:00:00Z" w:initials="{}"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#,
                &author[..1]
            )
        })
        .collect();
    let comments = format!(r#"<w:comments xmlns:w="{W_NS}">{definitions}</w:comments>"#);
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

/// Bob's c0 on p0.
fn bobs_comment() -> Vec<u8> {
    with_comments(
        &format!("<w:p>{}</w:p>", commented(0, "The cap is 10.")),
        &[(0, "Bob", "Too low.")],
    )
}

/// c0 sits on text Bob inserted (r1), c5 on plain text, and a bookmark
/// with id 42 follows: accepting keeps both comments, rejecting removes c0.
fn sparse_ids() -> Vec<u8> {
    with_comments(
        &format!(
            r#"<w:p><w:ins w:id="1" {BOB}>{}</w:ins></w:p><w:p>{}</w:p><w:p><w:bookmarkStart w:id="42" w:name="kept"/><w:r><w:t>Bookmarked.</w:t></w:r><w:bookmarkEnd w:id="42"/></w:p>"#,
            commented(0, "New words."),
            commented(5, "Target."),
        ),
        &[
            (0, "Bob", "On the insertion."),
            (5, "Carol", "On the target."),
        ],
    )
}

fn plan(json: serde_json::Value) -> EditPlan {
    EditPlan::from_json(&json.to_string()).unwrap()
}

fn edit_c0(author: &str, rewrite: Option<bool>) -> EditPlan {
    let mut value = serde_json::json!({
        "schema_version": 1,
        "author": author,
        "date": "2026-10-20T09:00:00Z",
        "operations": [{"kind": "edit_comment", "comment_id": 0, "text": "Reworded."}],
    });
    if let Some(rewrite) = rewrite {
        value["rewrite_others_comments"] = rewrite.into();
    }
    plan(value)
}

/// The comment ids `bytes` holds, ascending.
fn ids(bytes: &[u8]) -> Vec<u32> {
    let mut ids: Vec<u32> = list_comments(bytes).unwrap().iter().map(|c| c.id).collect();
    ids.sort_unstable();
    ids
}

fn bookmark_id(bytes: &[u8]) -> String {
    let xml = common::docx::part_string(bytes, "word/document.xml").unwrap();
    let start = xml.find("<w:bookmarkStart ").unwrap();
    let id = &xml[start..][xml[start..].find("w:id=\"").unwrap() + 6..];
    id[..id.find('"').unwrap()].to_string()
}

const REFUSAL: &str = r#"c0 is by "Bob"; reply instead (reply_comment), or pass "rewrite_others_comments": true to change their words under their name"#;

#[test]
fn editing_another_authors_comment_is_refused() {
    let error = apply_plan(&bobs_comment(), &edit_c0("Alice", None)).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        ("COMMENT_NOT_YOURS", REFUSAL),
        "{error}"
    );
    // The default is the refusal, said or not.
    let error = apply_plan(&bobs_comment(), &edit_c0("Alice", Some(false))).unwrap_err();
    assert_eq!(error.code, "COMMENT_NOT_YOURS");
}

#[test]
fn editing_your_own_comment_needs_no_flag() {
    let out = apply_plan(&bobs_comment(), &edit_c0("Bob", None)).unwrap();
    assert!(out.report.notes.is_empty(), "{:?}", out.report.notes);
    for bytes in [&out.clean, &out.redline] {
        assert_word_valid_package(bytes);
        assert_eq!(list_comments(bytes).unwrap()[0].text, "Reworded.");
    }
}

#[test]
fn rewrite_others_comments_true_allows_it_with_a_note() {
    let out = apply_plan(&bobs_comment(), &edit_c0("Alice", Some(true))).unwrap();
    assert_eq!(
        out.report.notes,
        [r#"op-1 (edit_comment): c0 is by "Bob"; their words were changed under their name"#]
    );
    for bytes in [&out.clean, &out.redline] {
        assert_word_valid_package(bytes);
        let listed = list_comments(bytes).unwrap();
        assert_eq!(
            (listed[0].text.as_str(), listed[0].author.as_str()),
            ("Reworded.", "Bob")
        );
    }
}

#[test]
fn a_reply_by_another_author_is_theirs_too() {
    let source = with_comments(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/><w:r><w:t>The cap is 10.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="0"/></w:r><w:r><w:commentReference w:id="1"/></w:r></w:p>"#
        ),
        &[(0, "Alice", "Too low."), (1, "Bob", "Agreed.")],
    );
    let refused = apply_plan(
        &source,
        &plan(serde_json::json!({
            "schema_version": 1,
            "author": "Alice",
            "operations": [{"kind": "edit_comment", "comment_id": 1, "text": "Reworded."}],
        })),
    )
    .unwrap_err();
    assert_eq!(refused.code, "COMMENT_NOT_YOURS");
    assert!(
        refused.message.starts_with(r#"c1 is by "Bob";"#),
        "{refused}"
    );
    // Alice's own root stays hers to edit.
    apply_plan(&source, &edit_c0("Alice", None)).unwrap();
}

#[test]
fn a_v2_plan_is_refused_in_its_own_words() {
    let source = bobs_comment();
    let value = |rewrite: Option<bool>| {
        let mut value = serde_json::json!({
            "schema_version": 2,
            "source": {"sha256": jubarte::inspect::source_sha256(&source)},
            "policy": {"track": "on", "existing_revisions": "keep", "text_view": "final"},
            "resolve_revisions": {"accept": [], "reject": []},
            "author": "Alice",
            "date": "2026-10-20T09:00:00Z",
            "scope": {"kind": "all"},
            "operations": [{
                "kind": "edit_comment",
                "target": {"short_id": "c0", "long_id": "comments.w:comment.w:id.0"},
                "text": "Reworded.",
            }],
        });
        if let Some(rewrite) = rewrite {
            value["rewrite_others_comments"] = rewrite.into();
        }
        jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap()
    };
    let error = jubarte::edit::v2_apply::apply_plan(&source, &value(None)).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        ("COMMENT_NOT_YOURS", REFUSAL)
    );
    let out = jubarte::edit::v2_apply::apply_plan(&source, &value(Some(true))).unwrap();
    let listed = list_comments(&out.redline).unwrap();
    assert_eq!(
        (listed[0].text.as_str(), listed[0].author.as_str()),
        ("Reworded.", "Bob")
    );
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_flags_refuse_with_exit_3_and_name_their_own_route() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), bobs_comment()).unwrap();
    let edit = |extra: &[&str], output: &str| {
        let mut args = vec![
            "edit",
            "in.docx",
            "-p",
            "c0",
            "--content",
            "Reworded.",
            "--author",
            "Alice",
            "--quiet",
            "-o",
            output,
        ];
        args.extend_from_slice(extra);
        run(dir.path(), &args)
    };
    let refused = edit(&[], "refused.docx");
    assert_eq!(refused.status.code(), Some(3), "{refused:?}");
    let stderr = String::from_utf8(refused.stderr).unwrap();
    assert!(
        stderr.contains(r#"COMMENT_NOT_YOURS (op-1): c0 is by "Bob"; reply instead (-p c0 --reply TEXT), or pass --rewrite-others-comments to change their words under their name"#),
        "{stderr}"
    );
    assert!(!dir.path().join("refused.docx").exists());

    let allowed = edit(&["--rewrite-others-comments"], "allowed.docx");
    assert_eq!(allowed.status.code(), Some(0), "{allowed:?}");
    let written = std::fs::read(dir.path().join("allowed.docx")).unwrap();
    assert_word_valid_package(&written);
    let listed = list_comments(&written).unwrap();
    assert_eq!(
        (listed[0].text.as_str(), listed[0].author.as_str()),
        ("Reworded.", "Bob")
    );
}

#[test]
fn accept_all_keeps_comment_ids() {
    let all = ChangeFilter::default();
    for accepted in [
        accept_revisions(&sparse_ids()).unwrap(),
        accept_changes(&sparse_ids(), &all).unwrap(),
    ] {
        assert_word_valid_package(&accepted);
        assert_eq!(ids(&accepted), [0, 5]);
        assert_eq!(bookmark_id(&accepted), "42");
    }
}

#[test]
fn reject_all_keeps_comment_ids() {
    let all = ChangeFilter::default();
    for rejected in [
        reject_revisions(&sparse_ids()).unwrap(),
        reject_changes(&sparse_ids(), &all).unwrap(),
    ] {
        assert_word_valid_package(&rejected);
        // c0 went with Bob's insertion; c5 is still c5.
        assert_eq!(ids(&rejected), [5]);
        assert_eq!(bookmark_id(&rejected), "42");
    }
}

#[test]
fn a_reply_after_a_bare_accept_lands_in_the_thread_read() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), sparse_ids()).unwrap();
    let accepted = run(dir.path(), &["accept", "in.docx", "-o", "accepted.docx"]);
    assert_eq!(accepted.status.code(), Some(0), "{accepted:?}");
    let replied = run(
        dir.path(),
        &[
            "edit",
            "accepted.docx",
            "-p",
            "c5",
            "--reply",
            "Fine.",
            "--author",
            "Alice",
            "--quiet",
            "-o",
            "replied.docx",
        ],
    );
    assert_eq!(replied.status.code(), Some(0), "{replied:?}");
    let listed = list_comments(&std::fs::read(dir.path().join("replied.docx")).unwrap()).unwrap();
    let reply = listed.iter().find(|c| c.text == "Fine.").unwrap();
    let parent = listed.iter().find(|c| Some(c.id) == reply.parent).unwrap();
    assert_eq!(parent.text, "On the target.");
}

#[test]
fn an_edit_that_settles_keeps_the_ids_it_reports() {
    // Accepting r1 inside the plan and replying to c5: the output still
    // calls the target c5, and the reply's id is the one reported.
    let out = apply_plan(
        &sparse_ids(),
        &plan(serde_json::json!({
            "schema_version": 1,
            "author": "Alice",
            "date": "2026-10-20T09:00:00Z",
            "existing_revisions": "accept",
            "operations": [{"kind": "reply_comment", "comment_id": 5, "text": "Fine."}],
        })),
    )
    .unwrap();
    for bytes in [&out.clean, &out.redline] {
        assert_word_valid_package(bytes);
        let listed = list_comments(bytes).unwrap();
        let reply = listed.iter().find(|c| c.text == "Fine.").unwrap();
        assert_eq!(reply.parent, Some(5), "{listed:?}");
        assert_eq!(out.report.operations[0].comment_id, Some(reply.id));
    }
    assert_eq!(ids(&out.clean), [0, 5, 6]);
    // The clean copy keeps the bookmark's id; the comparer numbers the
    // redline's bookmarks itself, and nothing addresses a bookmark by id.
    assert_eq!(bookmark_id(&out.clean), "42");
}

#[test]
fn a_new_comment_never_takes_an_id_the_plan_removed() {
    // Rejecting r1 removes c0; with c5 the highest left, a new comment
    // must not come back as c0 under the same report that names c0 removed.
    let source = with_comments(
        &format!(
            r#"<w:p>{}</w:p><w:p><w:ins w:id="1" {BOB}>{}</w:ins></w:p><w:p><w:r><w:t>Plain.</w:t></w:r></w:p>"#,
            commented(5, "Target."),
            commented(9, "New words."),
        ),
        &[
            (5, "Carol", "On the target."),
            (9, "Bob", "On the insertion."),
        ],
    );
    let out = apply_plan(
        &source,
        &plan(serde_json::json!({
            "schema_version": 1,
            "author": "Alice",
            "date": "2026-10-20T09:00:00Z",
            "existing_revisions": "reject",
            "operations": [{"kind": "comment", "paragraph": "p2", "text": "New note."}],
        })),
    )
    .unwrap();
    assert_eq!(out.report.removed_comments[0].id, "c9");
    let added = out.report.operations[0].comment_id.unwrap();
    assert!(added > 9, "the new comment reuses a removed id: c{added}");
    assert_eq!(ids(&out.clean), [5, added]);
}
