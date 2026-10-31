// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment id is an identifier, and a comment speaks for its author
//! (#1264). Settling revisions never renumbers the comments that stay, so
//! a reply written after a bare accept lands in the thread the agent read.
//! `edit_comment`, `delete_comment` and `resolve_comment` on another
//! author's comment are allowed and noted (#1892). `"others": "keep"`
//! refuses them (`COMMENT_NOT_YOURS`), and so does deleting your own
//! comment whose thread holds another author's reply. The old
//! `rewrite_others_comments` field is accepted and does not bypass that.

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

fn plan(json: &serde_json::Value) -> EditPlan {
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
    plan(&value)
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

const REFUSAL: &str = r#"c0 is by "Bob"; reply instead (reply_comment)"#;

fn kept(mut plan: EditPlan) -> EditPlan {
    plan.others = jubarte::edit::Others::Keep;
    plan
}

#[test]
fn editing_another_authors_comment_is_allowed_unless_others_is_keep() {
    for rewrite in [None, Some(false)] {
        let out = apply_plan(&bobs_comment(), &edit_c0("Alice", rewrite)).unwrap();
        assert_eq!(
            out.report.notes,
            [r#"op-1 (edit_comment): c0 is by "Bob"; their words were changed under their name"#],
            "{rewrite:?}"
        );
        assert_eq!(list_comments(&out.redline).unwrap()[0].text, "Reworded.");
    }
    let error = apply_plan(&bobs_comment(), &kept(edit_c0("Alice", None))).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        ("COMMENT_NOT_YOURS", REFUSAL),
        "{error}"
    );
    let error = apply_plan(&bobs_comment(), &kept(edit_c0("Alice", Some(true)))).unwrap_err();
    assert_eq!(error.code, "COMMENT_NOT_YOURS");
    assert!(
        !error.message.contains("rewrite_others_comments"),
        "{error}"
    );
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
        r#"<w:p><w:commentRangeStart w:id="0"/><w:commentRangeStart w:id="1"/><w:r><w:t>The cap is 10.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="0"/></w:r><w:r><w:commentReference w:id="1"/></w:r></w:p>"#,
        &[(0, "Alice", "Too low."), (1, "Bob", "Agreed.")],
    );
    let value = serde_json::json!({
        "schema_version": 1,
        "author": "Alice",
        "operations": [{"kind": "edit_comment", "comment_id": 1, "text": "Reworded."}],
    });
    let out = apply_plan(&source, &plan(&value)).unwrap();
    assert!(
        out.report
            .notes
            .iter()
            .any(|n| n.contains(r#"c1 is by "Bob""#)),
        "{:?}",
        out.report.notes
    );
    let error = apply_plan(&source, &kept(plan(&value))).unwrap_err();
    assert_eq!(error.code, "COMMENT_NOT_YOURS");
    assert!(
        error
            .message
            .starts_with(r#"c1 is by "Bob"; reply instead"#),
        "{error}"
    );
    // Alice's own root stays hers to edit, guard or not.
    apply_plan(&source, &kept(edit_c0("Alice", None))).unwrap();
}

#[test]
fn a_v2_plan_allows_another_authors_comment_unless_others_is_keep() {
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
    let out = jubarte::edit::v2_apply::apply_plan(&source, &value(None)).unwrap();
    assert_eq!(
        out.report.notes,
        [
            r#"/operations/0 (edit_comment): c0 is by "Bob"; their words were changed under their name"#
        ]
    );
    let mut guarded = serde_json::json!({
        "schema_version": 2,
        "source": {"sha256": jubarte::inspect::source_sha256(&source)},
        "policy": {"track": "on", "existing_revisions": "keep", "text_view": "final"},
        "resolve_revisions": {"accept": [], "reject": []},
        "author": "Alice",
        "date": "2026-10-20T09:00:00Z",
        "others": "keep",
        "rewrite_others_comments": true,
        "scope": {"kind": "all"},
        "operations": [{
            "kind": "edit_comment",
            "target": {"short_id": "c0", "long_id": "comments.w:comment.w:id.0"},
            "text": "Reworded.",
        }],
    });
    let error = jubarte::edit::v2_apply::apply_plan(
        &source,
        &jubarte::edit::v2::PlanV2::from_json(&guarded.to_string()).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        ("COMMENT_NOT_YOURS", REFUSAL)
    );
    assert!(
        !error.message.contains("rewrite_others_comments"),
        "{error}"
    );
    guarded
        .as_object_mut()
        .unwrap()
        .remove("rewrite_others_comments");
    let error = jubarte::edit::v2_apply::apply_plan(
        &source,
        &jubarte::edit::v2::PlanV2::from_json(&guarded.to_string()).unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.code, "COMMENT_NOT_YOURS");
    let out = jubarte::edit::v2_apply::apply_plan(&source, &value(Some(true))).unwrap();
    let listed = list_comments(&out.redline).unwrap();
    assert_eq!(
        (listed[0].text.as_str(), listed[0].author.as_str()),
        ("Reworded.", "Bob")
    );
    // A v2 report notes the rewrite as a v1 report does.
    assert_eq!(
        out.report.notes,
        [
            r#"/operations/0 (edit_comment): c0 is by "Bob"; their words were changed under their name"#
        ]
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
fn the_flags_allow_another_authors_comment_unless_keep_others() {
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
    let allowed = edit(&[], "allowed.docx");
    assert_eq!(allowed.status.code(), Some(0), "{allowed:?}");
    let written = std::fs::read(dir.path().join("allowed.docx")).unwrap();
    assert_word_valid_package(&written);
    let listed = list_comments(&written).unwrap();
    assert_eq!(
        (listed[0].text.as_str(), listed[0].author.as_str()),
        ("Reworded.", "Bob")
    );
    // The old flag is still accepted and still allows the reword.
    let flagged = edit(&["--rewrite-others-comments"], "flagged.docx");
    assert_eq!(flagged.status.code(), Some(0), "{flagged:?}");

    let refused = edit(&["--keep-others"], "refused.docx");
    assert_eq!(refused.status.code(), Some(3), "{refused:?}");
    let stderr = String::from_utf8(refused.stderr).unwrap();
    assert!(
        stderr.contains(
            r#"COMMENT_NOT_YOURS (op-1): c0 is by "Bob"; reply instead (-p c0 --reply TEXT)"#
        ),
        "{stderr}"
    );
    assert!(!stderr.contains("rewrite-others-comments"), "{stderr}");
    assert!(!dir.path().join("refused.docx").exists());
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
        &plan(&serde_json::json!({
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
        &plan(&serde_json::json!({
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

/// c0 on p0 whose `w:comment` carries `author_attribute` verbatim (none,
/// or an empty `w:author`).
fn unnamed_comment(author_attribute: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0"{author_attribute} w:date="2026-10-18T12:00:00Z"><w:p><w:r><w:t>Too low.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        &format!("<w:p>{}</w:p>", commented(0, "The cap is 10.")),
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

#[test]
fn a_comment_that_names_no_author_is_not_yours_either() {
    // Only a comment that names the plan's author is its own: with no
    // name (scrubbed, or written by a tool) nobody can say whose it is.
    for attribute in ["", r#" w:author="""#] {
        let source = unnamed_comment(attribute);
        let out = apply_plan(&source, &edit_c0("Alice", None)).unwrap();
        assert_eq!(
            out.report.notes,
            ["op-1 (edit_comment): c0 names no author; its words were changed"],
            "{attribute:?}"
        );
        let error = apply_plan(&source, &kept(edit_c0("Alice", None))).unwrap_err();
        assert_eq!(
            (error.code.as_str(), error.message.as_str()),
            (
                "COMMENT_NOT_YOURS",
                r#"c0 names no author; reply instead (reply_comment)"#
            ),
            "{attribute:?}: {error}"
        );
        let out = apply_plan(&source, &edit_c0("Alice", Some(true))).unwrap();
        assert_eq!(
            out.report.notes,
            ["op-1 (edit_comment): c0 names no author; its words were changed"],
            "{attribute:?}"
        );
        for bytes in [&out.clean, &out.redline] {
            assert_word_valid_package(bytes);
            assert_eq!(list_comments(bytes).unwrap()[0].text, "Reworded.");
        }
    }
}

/// A v1 plan by `author` of one thread operation `op`, `rewrite` as given.
fn thread_plan(author: &str, op: &serde_json::Value, rewrite: Option<bool>) -> EditPlan {
    let mut value = serde_json::json!({
        "schema_version": 1,
        "author": author,
        "date": "2026-10-20T09:00:00Z",
        "operations": [op],
    });
    if let Some(rewrite) = rewrite {
        value["rewrite_others_comments"] = rewrite.into();
    }
    plan(&value)
}

/// Alice's c0 on p0 with Bob's reply c1, a thread as `edit` writes one.
fn alices_thread_with_bobs_reply() -> Vec<u8> {
    let reply = apply_plan(
        &with_comments(
            &format!("<w:p>{}</w:p>", commented(0, "The cap is 10.")),
            &[(0, "Alice", "Too low.")],
        ),
        &thread_plan(
            "Bob",
            &serde_json::json!({"kind": "reply_comment", "comment_id": 0, "text": "Agreed."}),
            None,
        ),
    )
    .unwrap()
    .redline;
    let listed = list_comments(&reply).unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|c| (c.id, c.author.as_str(), c.parent))
            .collect::<Vec<_>>(),
        [(0, "Alice", None), (1, "Bob", Some(0))]
    );
    reply
}

#[test]
fn deleting_or_resolving_another_authors_comment_is_allowed_unless_others_is_keep() {
    let cases = [
        (
            serde_json::json!({"kind": "delete_comment", "comment_id": 0}),
            "their comment was deleted",
        ),
        (
            serde_json::json!({"kind": "resolve_comment", "comment_id": 0}),
            "their comment was resolved",
        ),
        (
            serde_json::json!({"kind": "resolve_comment", "comment_id": 0, "done": false}),
            "their comment was reopened",
        ),
    ];
    for (op, noted) in cases {
        let out = apply_plan(&bobs_comment(), &thread_plan("Alice", &op, None)).unwrap();
        let kind = op["kind"].as_str().unwrap();
        assert_eq!(
            out.report.notes,
            [format!(r#"op-1 ({kind}): c0 is by "Bob"; {noted}"#)],
            "{op}"
        );
        for bytes in [&out.clean, &out.redline] {
            assert_word_valid_package(bytes);
        }
        let error = apply_plan(
            &bobs_comment(),
            &kept(thread_plan("Alice", &op, Some(true))),
        )
        .unwrap_err();
        assert_eq!(
            (error.code.as_str(), error.message.as_str()),
            ("COMMENT_NOT_YOURS", REFUSAL),
            "{op}: {error}"
        );
        assert!(
            !error.message.contains("rewrite_others_comments"),
            "{error}"
        );
    }
}

#[test]
fn deleting_or_resolving_your_own_comment_needs_no_flag() {
    for op in [
        serde_json::json!({"kind": "delete_comment", "comment_id": 0}),
        serde_json::json!({"kind": "resolve_comment", "comment_id": 0}),
    ] {
        let out = apply_plan(&bobs_comment(), &thread_plan("Bob", &op, None)).unwrap();
        assert!(out.report.notes.is_empty(), "{op}: {:?}", out.report.notes);
        let listed = list_comments(&out.redline).unwrap();
        match op["kind"].as_str().unwrap() {
            "delete_comment" => assert!(listed.is_empty(), "{listed:?}"),
            _ => assert!(listed[0].done, "{listed:?}"),
        }
    }
}

#[test]
fn deleting_your_own_thread_would_take_another_authors_reply() {
    let source = alices_thread_with_bobs_reply();
    let delete = serde_json::json!({"kind": "delete_comment", "comment_id": 0});
    let out = apply_plan(&source, &thread_plan("Alice", &delete, None)).unwrap();
    assert_eq!(
        out.report.notes,
        [r#"op-1 (delete_comment): c1 is by "Bob"; their reply was deleted with c0"#]
    );
    assert!(list_comments(&out.redline).unwrap().is_empty());
    let error = apply_plan(&source, &kept(thread_plan("Alice", &delete, Some(true)))).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        (
            "COMMENT_NOT_YOURS",
            r#"deleting c0 deletes c1, a reply by "Bob"; resolve c0 instead (resolve_comment)"#
        )
    );
    assert!(
        !error.message.contains("rewrite_others_comments"),
        "{error}"
    );
    // Resolving or reopening her own thread is Alice's to do. Word marks the
    // whole thread done or open again, Bob's reply included. That erases
    // nothing, so the guard allows it.
    let out = apply_plan(
        &source,
        &kept(thread_plan(
            "Alice",
            &serde_json::json!({"kind": "resolve_comment", "comment_id": 0}),
            None,
        )),
    )
    .unwrap();
    assert!(out.report.notes.is_empty(), "{:?}", out.report.notes);
    let resolved = list_comments(&out.clean).unwrap();
    assert!(
        resolved.len() == 2 && resolved.iter().all(|c| c.done),
        "resolve marks Alice's thread and Bob's reply done: {resolved:?}"
    );
    let reopened = apply_plan(
        &out.clean,
        &kept(thread_plan(
            "Alice",
            &serde_json::json!({"kind": "resolve_comment", "comment_id": 0, "done": false}),
            None,
        )),
    )
    .unwrap();
    assert!(
        reopened.report.notes.is_empty(),
        "{:?}",
        reopened.report.notes
    );
    let open = list_comments(&reopened.clean).unwrap();
    assert!(
        open.len() == 2 && open.iter().all(|c| !c.done),
        "reopen marks Alice's thread and Bob's reply open: {open:?}"
    );
    // Bob may delete his own reply: it holds no one else's words.
    apply_plan(
        &source,
        &kept(thread_plan(
            "Bob",
            &serde_json::json!({"kind": "delete_comment", "comment_id": 1}),
            None,
        )),
    )
    .unwrap();
}

#[test]
fn the_flags_allow_delete_and_resolve_unless_keep_others() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), bobs_comment()).unwrap();
    for flag in ["--delete", "--resolve"] {
        let edit = |extra: &[&str], output: &str| {
            let mut args = vec![
                "edit", "in.docx", "-p", "c0", flag, "--author", "Alice", "--quiet", "-o", output,
            ];
            args.extend_from_slice(extra);
            run(dir.path(), &args)
        };
        let allowed = edit(&[], "allowed.docx");
        assert_eq!(allowed.status.code(), Some(0), "{flag}: {allowed:?}");
        assert_word_valid_package(&std::fs::read(dir.path().join("allowed.docx")).unwrap());
        let refused = edit(&["--keep-others", "--force"], "refused.docx");
        assert_eq!(refused.status.code(), Some(3), "{flag}: {refused:?}");
        let stderr = String::from_utf8(refused.stderr).unwrap();
        assert!(
            stderr.contains(
                r#"COMMENT_NOT_YOURS (op-1): c0 is by "Bob"; reply instead (-p c0 --reply TEXT)"#
            ),
            "{flag}: {stderr}"
        );
        assert!(!stderr.contains("rewrite-others-comments"), "{stderr}");
    }
}

#[test]
fn a_comment_that_names_no_author_is_not_yours_to_delete_either() {
    let source = unnamed_comment("");
    let delete = serde_json::json!({"kind": "delete_comment", "comment_id": 0});
    let out = apply_plan(&source, &thread_plan("Alice", &delete, None)).unwrap();
    assert_eq!(
        out.report.notes,
        ["op-1 (delete_comment): c0 names no author; it was deleted"]
    );
    assert!(list_comments(&out.redline).unwrap().is_empty());
    let error = apply_plan(&source, &kept(thread_plan("Alice", &delete, None))).unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.message.as_str()),
        (
            "COMMENT_NOT_YOURS",
            r#"c0 names no author; reply instead (reply_comment)"#
        )
    );
}
