// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! One opt-in guard for another author's markup (#1892, #1890).
//!
//! Off (the default), accepting or rejecting their change and deleting or
//! resolving their comment are allowed. On (`--keep-others`,
//! `JUBARTE_KEEP_OTHERS=1`, or `"others": "keep"`), those are refused.
//! The top-level `accept` and `reject` commands are not part of the guard.

mod common;

use common::docx::{Part, W_NS, docx, docx_with};
use jubarte::comments::list_comments;
use jubarte::edit::EditPlan;
use jubarte::inspect::source_sha256;
use serde_json::{Value, json};
use std::path::Path;
use std::process::{Command, Output};

const DATE: &str = "2026-10-18T12:00:00Z";
const KEEP: &str = concat!(
    r#"add "existing_revisions": "keep" to the plan "#,
    "(it keeps every tracked change, yours and theirs; accept and reject resolve them all first)"
);

fn deleted(author: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:del w:id="2" w:author="{author}" w:date="{DATE}"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t> kept</w:t></w:r></w:p>"#
    ))
}

fn inserted(author: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:ins w:id="3" w:author="{author}" w:date="{DATE}"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#
    ))
}

fn plain() -> Vec<u8> {
    docx(r#"<w:p><w:r><w:t>Hello.</w:t></w:r></w:p>"#)
}

const COMMENTS: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";

fn bobs_comment() -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Bob" w:date="{DATE}" w:initials="B"><w:p><w:r><w:t>Too low.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>The cap is 10.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

fn plan(value: &Value) -> EditPlan {
    EditPlan::from_json(&value.to_string()).unwrap()
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn run_env(dir: &Path, key: &str, value: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .env(key, value)
        .args(args)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn edit_accepts_another_authors_change_by_default() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), deleted("Trainer 1 - AgentCo")).unwrap();
    let output = run(
        dir.path(),
        &[
            "edit", "in.docx", "-p", "r2", "--accept", "--author", "Alice", "--quiet", "-o",
            "out.docx",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(dir.path().join("out.docx").is_file());
}

#[test]
fn edit_keep_others_refuses_accept_of_another_authors_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), deleted("Trainer 1 - AgentCo")).unwrap();
    let output = run(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "-p",
            "r2",
            "--accept",
            "--author",
            "Alice",
            "--keep-others",
            "--quiet",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.contains("CHANGE_NOT_YOURS"), "{err}");
    assert!(
        err.contains(
            r#"r2 is by "Trainer 1 - AgentCo"; answer it visibly (-p r2 --restore), or comment on it"#
        ),
        "{err}"
    );
    assert!(!err.contains("--accept"), "{err}");
    assert!(!dir.path().join("out.docx").exists());
}

#[test]
fn edit_keep_others_env_refuses_reject_of_another_authors_insertion() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), inserted("Trainer 1 - AgentCo")).unwrap();
    let output = run_env(
        dir.path(),
        "JUBARTE_KEEP_OTHERS",
        "1",
        &[
            "edit", "in.docx", "-p", "r3", "--reject", "--author", "Alice", "--quiet", "-o",
            "out.docx",
        ],
    );
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.contains("CHANGE_NOT_YOURS"), "{err}");
    assert!(
        err.contains(
            r#"r3 is by "Trainer 1 - AgentCo"; strike it (-p r3 --strike), or comment on it"#
        ),
        "{err}"
    );
    // Only the exact value "1" arms the guard.
    let loose = run_env(
        dir.path(),
        "JUBARTE_KEEP_OTHERS",
        "true",
        &[
            "edit",
            "in.docx",
            "-p",
            "r3",
            "--reject",
            "--author",
            "Alice",
            "--quiet",
            "-o",
            "loose.docx",
        ],
    );
    assert_eq!(loose.status.code(), Some(0), "{}", stderr(&loose));
}

#[test]
fn edit_plan_others_keep_refuses_resolve_revisions_of_their_change() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "resolve_revisions": {"accept": {"ids": ["r2"]}},
        "operations": []
    });
    let err =
        jubarte::edit::apply_plan(&deleted("Trainer 1 - AgentCo"), &plan(&value)).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS");
    assert!(
        err.message.contains(
            r#"r2 is by "Trainer 1 - AgentCo"; answer it visibly (-p r2 --restore), or comment on it"#
        ),
        "{err}"
    );
}

#[test]
fn edit_keep_others_allows_accepting_your_own_change() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "resolve_revisions": {"accept": {"ids": ["r2"]}},
        "operations": []
    });
    jubarte::edit::apply_plan(&deleted("Alice"), &plan(&value)).unwrap();
}

#[test]
fn edit_keep_others_refuses_the_default_author_against_a_foreign_change() {
    let value = json!({
        "schema_version": 1,
        "author": "Jubarte",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "resolve_revisions": {"reject": {"ids": ["r2"]}},
        "operations": []
    });
    let err = jubarte::edit::apply_plan(&deleted("Alice"), &plan(&value)).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    // A change signed Jubarte is the default author's, guard or not.
    let own = json!({
        "schema_version": 1,
        "author": "Jubarte",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "resolve_revisions": {"accept": {"ids": ["r2"]}},
        "operations": []
    });
    jubarte::edit::apply_plan(&deleted("Jubarte"), &plan(&own)).unwrap();
}

#[test]
fn edit_keep_others_allows_a_visible_reject() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "resolve_revisions": {"reject": {"ids": ["r2"], "visible": true}},
        "operations": []
    });
    jubarte::edit::apply_plan(&deleted("Trainer 1 - AgentCo"), &plan(&value)).unwrap();
}

#[test]
fn edit_existing_revisions_accept_of_their_changes_is_change_not_yours() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "accept",
        "others": "keep",
        "operations": []
    });
    let err =
        jubarte::edit::apply_plan(&deleted("Trainer 1 - AgentCo"), &plan(&value)).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(
        err.message.contains(
            r#""existing_revisions": "accept" would erase changes by "Trainer 1 - AgentCo"; "#
        ),
        "{err}"
    );
    assert!(err.message.contains(KEEP), "{err}");
    // A file that holds only your changes may still be accepted.
    let own = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "accept",
        "others": "keep",
        "operations": []
    });
    jubarte::edit::apply_plan(&deleted("Alice"), &plan(&own)).unwrap();
}

#[test]
fn edit_existing_revisions_refusal_names_keep_with_the_guard_off_and_on() {
    for others in [None, Some("keep")] {
        let mut value = json!({
            "schema_version": 1,
            "author": "Alice",
            "date": DATE,
            "operations": [{"kind": "comment", "paragraph": "p0", "find": "kept", "text": "note"}]
        });
        if let Some(others) = others {
            value["others"] = others.into();
        }
        let err = jubarte::edit::apply_plan(&deleted("Bob"), &plan(&value)).unwrap_err();
        assert_eq!(err.code, "EXISTING_REVISIONS", "{err}");
        assert!(err.message.contains(KEEP), "{others:?} {err}");
    }
}

#[test]
fn edit_existing_revisions_flag_beside_plan_names_the_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), plain()).unwrap();
    std::fs::write(
        dir.path().join("plan.json"),
        r#"{"schema_version":1,"author":"Alice","operations":[]}"#,
    )
    .unwrap();
    let output = run(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "--plan",
            "plan.json",
            "--existing-revisions",
            "keep",
        ],
    );
    let err = stderr(&output);
    assert!(err.contains(KEEP), "{err}");
    assert!(
        err.contains("--existing-revisions belongs to recorded operations"),
        "{err}"
    );
    let author = run(
        dir.path(),
        &["edit", "in.docx", "--plan", "plan.json", "--author", "Ada"],
    );
    let author_err = stderr(&author);
    assert!(
        author_err.contains("--author belongs to recorded operations; the plan names its own"),
        "{author_err}"
    );
    assert!(!author_err.contains("existing_revisions"), "{author_err}");
}

#[test]
fn edit_top_level_accept_still_accepts_another_authors_change_when_keep_others_is_set() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), deleted("Trainer 1 - AgentCo")).unwrap();
    let output = run_env(
        dir.path(),
        "JUBARTE_KEEP_OTHERS",
        "1",
        &["accept", "in.docx", "-o", "out.docx"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn edit_top_level_reject_still_rejects_another_authors_change_when_keep_others_is_set() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), inserted("Trainer 1 - AgentCo")).unwrap();
    let output = run_env(
        dir.path(),
        "JUBARTE_KEEP_OTHERS",
        "1",
        &["reject", "in.docx", "-o", "out.docx"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn edit_deleting_another_authors_comment_is_allowed_by_default() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "operations": [{"kind": "delete_comment", "comment_id": 0}]
    });
    let out = jubarte::edit::apply_plan(&bobs_comment(), &plan(&value)).unwrap();
    assert!(list_comments(&out.clean).unwrap().is_empty());
    assert!(
        out.report
            .notes
            .iter()
            .any(|n| n.contains(r#"c0 is by "Bob""#)),
        "{:?}",
        out.report.notes
    );
}

#[test]
fn edit_keep_others_refuses_another_authors_comment_without_the_rewrite_flag() {
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "rewrite_others_comments": true,
        "operations": [{"kind": "delete_comment", "comment_id": 0}]
    });
    let err = jubarte::edit::apply_plan(&bobs_comment(), &plan(&value)).unwrap_err();
    assert_eq!(err.code, "COMMENT_NOT_YOURS", "{err}");
    assert_eq!(
        err.message,
        r#"c0 is by "Bob"; reply instead (reply_comment)"#
    );
    assert!(!err.message.contains("rewrite_others_comments"), "{err}");
}

#[test]
fn edit_rewrite_others_comments_does_not_bypass_keep_others_on_the_command_line() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), bobs_comment()).unwrap();
    let output = run(
        dir.path(),
        &[
            "edit",
            "in.docx",
            "-p",
            "c0",
            "--delete",
            "--author",
            "Alice",
            "--keep-others",
            "--rewrite-others-comments",
            "--quiet",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.contains("COMMENT_NOT_YOURS"), "{err}");
    assert!(
        err.contains(r#"c0 is by "Bob"; reply instead (-p c0 --reply TEXT)"#),
        "{err}"
    );
    assert!(!err.contains("rewrite-others-comments"), "{err}");
}

#[test]
fn edit_add_help_offers_keep_others_and_hides_the_old_flag() {
    let add = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["add", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(add.stdout).unwrap();
    assert!(help.contains("--keep-others"), "{help}");
    assert!(!help.contains("rewrite-others-comments"), "{help}");
    let edit = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["edit", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(edit.stdout).unwrap();
    assert!(help.contains("--keep-others"), "{help}");
    assert!(!help.contains("rewrite-others-comments"), "{help}");
}

#[test]
fn edit_add_keep_others_records_the_plan_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), plain()).unwrap();
    let output = run(
        dir.path(),
        &[
            "add",
            "in.docx",
            "--plan",
            "plan.json",
            "-p",
            "p0",
            "--content",
            "More.",
            "--keep-others",
            "--quiet",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let recorded: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("plan.json")).unwrap())
            .unwrap();
    assert_eq!(recorded["others"], "keep");
}

#[test]
fn edit_hint_names_the_cost_and_next_actions_mark_accept_and_reject_destructive() {
    let source = deleted("Trainer 1 - AgentCo");
    let value = json!({
        "schema_version": 1,
        "author": "Carol Diaz",
        "date": DATE,
        "existing_revisions": "keep",
        "operations": [{"kind": "replace", "paragraph": "p0", "find": "old", "replacement": "new"}]
    });
    let err = jubarte::edit::apply_plan(&source, &plan(&value)).unwrap_err();
    assert!(
        err.message
            .contains(r#"let it go (erases "Trainer 1 - AgentCo"'s deletion):"#),
        "{err}"
    );
    assert!(
        err.message
            .contains(r#"put it back silently (erases "Trainer 1 - AgentCo"'s deletion):"#),
        "{err}"
    );
    let actions = err.revision_next_actions("in.docx", &source_sha256(&source));
    let listed = serde_json::to_value(&actions).unwrap();
    assert!(
        listed[0]["argv"]
            .as_array()
            .unwrap()
            .iter()
            .any(|arg| arg == "--restore"),
        "{listed}"
    );
    assert_eq!(listed[1]["destructive"], true, "{listed}");
    assert_eq!(listed[2]["destructive"], true, "{listed}");
    assert!(
        listed[2]["purpose"]
            .as_str()
            .unwrap()
            .contains("rejecting erases their change"),
        "{listed}"
    );
    assert!(
        !listed[2]["purpose"]
            .as_str()
            .unwrap()
            .contains("restores the deleted content"),
        "{listed}"
    );
}

#[test]
fn edit_hint_guard_on_offers_no_accept_or_reject_for_another_authors_deletion() {
    let source = deleted("Trainer 1 - AgentCo");
    let value = json!({
        "schema_version": 1,
        "author": "Carol Diaz",
        "date": DATE,
        "existing_revisions": "keep",
        "others": "keep",
        "operations": [{"kind": "replace", "paragraph": "p0", "find": "old", "replacement": "new"}]
    });
    let err = jubarte::edit::apply_plan(&source, &plan(&value)).unwrap_err();
    assert!(!err.message.contains("let it go"), "{err}");
    assert!(!err.message.contains("put it back silently"), "{err}");
    assert!(err.message.contains("restore"), "{err}");
    let actions = err.revision_next_actions("in.docx", &source_sha256(&source));
    assert!(
        actions
            .iter()
            .any(|action| action.argv.iter().any(|arg| arg == "--restore")),
        "{actions:?}"
    );
    assert!(
        actions.iter().all(|action| !action
            .argv
            .iter()
            .any(|arg| arg.contains("accept") || arg.contains("reject"))),
        "{actions:?}"
    );
}

/// Alice's insertion `r3` whose run records Bob's formatting change `r4`.
fn alice_insertion_holding_bobs_formatting() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:ins w:id="3" w:author="Alice" w:date="{DATE}"><w:r><w:rPr><w:b/><w:rPrChange w:id="4" w:author="Bob" w:date="{DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>new</w:t></w:r></w:ins></w:p>"#
    ))
}

fn reject_r3(others: Option<&str>) -> Value {
    let mut value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "resolve_revisions": {"reject": {"ids": ["r3"]}},
        "operations": []
    });
    if let Some(others) = others {
        value["others"] = others.into();
    }
    value
}

#[test]
fn edit_keep_others_refuses_rejecting_your_change_that_holds_their_revision() {
    let source = alice_insertion_holding_bobs_formatting();
    let listed = jubarte::changes::list_changes(&source).unwrap();
    let nested = listed.iter().find(|c| c.id == "r4").expect("r4 is listed");
    assert_eq!(nested.author.as_deref(), Some("Bob"), "{listed:?}");
    assert_eq!(nested.inside.as_deref(), Some("r3"), "{listed:?}");
    let err = jubarte::edit::apply_plan(&source, &plan(&reject_r3(Some("keep")))).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(
        err.message.contains(r#"r4 is by "Bob""#),
        "the refusal names the nested revision: {err}"
    );
    assert!(
        !err.message.contains(r#"r3 is by "Alice""#),
        "Alice's own insertion is not the refusal: {err}"
    );
    // Accepting her insertion keeps the text, so Bob's formatting stays.
    let mut accept = reject_r3(Some("keep"));
    accept["resolve_revisions"] = json!({"accept": {"ids": ["r3"]}});
    let out = jubarte::edit::apply_plan(&source, &plan(&accept)).unwrap();
    let kept = jubarte::changes::list_changes(&out.clean).unwrap();
    assert!(
        kept.iter()
            .any(|c| c.id == "r4" && c.author.as_deref() == Some("Bob")),
        "{kept:?}"
    );
    // Her own nested formatting is hers to settle with the insertion.
    let own = docx(&format!(
        r#"<w:p><w:ins w:id="3" w:author="Alice" w:date="{DATE}"><w:r><w:rPr><w:b/><w:rPrChange w:id="4" w:author="Alice" w:date="{DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>new</w:t></w:r></w:ins></w:p>"#
    ));
    jubarte::edit::apply_plan(&own, &plan(&reject_r3(Some("keep")))).unwrap();
    // The guard off still rejects the insertion and takes Bob's revision with it.
    let out = jubarte::edit::apply_plan(&source, &plan(&reject_r3(None))).unwrap();
    let gone = jubarte::changes::list_changes(&out.clean).unwrap();
    assert!(
        gone.iter().all(|c| c.id != "r4"),
        "guard off removes the nested revision: {gone:?}"
    );
}

/// Alice's insertion `r3` whose content holds Bob's comment anchor `c0`.
fn alice_insertion_holding_bobs_comment() -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Bob" w:date="{DATE}" w:initials="B"><w:p><w:r><w:t>Too low.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        &format!(
            r#"<w:p><w:ins w:id="3" w:author="Alice" w:date="{DATE}"><w:commentRangeStart w:id="0"/><w:r><w:t>new</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:ins></w:p>"#
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

#[test]
fn edit_keep_others_refuses_rejecting_your_change_that_holds_their_comment() {
    let source = alice_insertion_holding_bobs_comment();
    let listed = jubarte::changes::list_changes(&source).unwrap();
    let insertion = listed.iter().find(|c| c.id == "r3").expect("r3 is listed");
    assert_eq!(insertion.holds_comments, ["c0"], "{listed:?}");
    let err = jubarte::edit::apply_plan(&source, &plan(&reject_r3(Some("keep")))).unwrap_err();
    assert_eq!(err.code, "COMMENT_NOT_YOURS", "{err}");
    assert!(
        err.message
            .contains(r#"rejecting r3 would delete c0, which is by "Bob""#),
        "{err}"
    );
    let out = jubarte::edit::apply_plan(&source, &plan(&reject_r3(None))).unwrap();
    assert!(
        list_comments(&out.clean).unwrap().is_empty(),
        "guard off still deletes the comment the rejected insertion held"
    );
}

/// Alice's own body insertion, plus Carol's deletion inside Bob's comment.
fn body_change_and_comment_deletion() -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Bob" w:date="{DATE}" w:initials="B"><w:p><w:del w:id="9" w:author="Carol" w:date="{DATE}"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t>Too low.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        &format!(
            r#"<w:p><w:ins w:id="3" w:author="Alice" w:date="{DATE}"><w:r><w:t>new</w:t></w:r></w:ins><w:commentRangeStart w:id="0"/><w:r><w:t>The cap is 10.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

fn comment_deletion_only() -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Bob" w:date="{DATE}" w:initials="B"><w:p><w:del w:id="9" w:author="Carol" w:date="{DATE}"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t>Too low.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>The cap is 10.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

fn v2_accept(source: &[u8], others: Option<&str>) -> jubarte::edit::v2::PlanV2 {
    let mut value = json!({
        "schema_version": 2,
        "source": {"sha256": source_sha256(source)},
        "policy": {"track": "on", "existing_revisions": "accept", "text_view": "final"},
        "resolve_revisions": {"accept": [], "reject": []},
        "author": "Alice",
        "date": DATE,
        "scope": {"kind": "all"},
        "operations": []
    });
    if let Some(others) = others {
        value["others"] = others.into();
    }
    jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap()
}

fn comment_still_has_carols_deletion(bytes: &[u8]) -> bool {
    common::docx::part_string(bytes, "word/comments.xml")
        .is_some_and(|xml| xml.contains("w:del") && xml.contains("Carol"))
}

#[test]
fn edit_keep_others_sees_a_revision_inside_a_comment() {
    let source = body_change_and_comment_deletion();
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "accept",
        "others": "keep",
        "operations": []
    });
    let err = jubarte::edit::apply_plan(&source, &plan(&value)).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(err.message.contains(r#""Carol""#), "{err}");
    assert!(err.message.contains(KEEP), "{err}");
    let err = jubarte::edit::v2_apply::apply_plan(&source, &v2_accept(&source, Some("keep")))
        .unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(err.message.contains(r#""Carol""#), "{err}");
    // Guard off, the same settlement still accepts Carol's deletion.
    let mut allow = value.clone();
    allow.as_object_mut().unwrap().remove("others");
    let out = jubarte::edit::apply_plan(&source, &plan(&allow)).unwrap();
    assert!(
        !comment_still_has_carols_deletion(&out.clean),
        "guard off erases the comment-body deletion"
    );
    // A comment-only file lists `comments.r9`. v1 accept with the guard on
    // refuses to erase Carol; with the guard off it settles the deletion.
    let only = comment_deletion_only();
    let err = jubarte::edit::apply_plan(&only, &plan(&value)).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(err.message.contains(r#""Carol""#), "{err}");
    let mut allow_only = value.clone();
    allow_only.as_object_mut().unwrap().remove("others");
    let settled = jubarte::edit::apply_plan(&only, &plan(&allow_only)).unwrap();
    assert!(
        !comment_still_has_carols_deletion(&settled.clean),
        "v1 accept settles a comment-only revision"
    );
    // v2 accept does flatten that comment body, so the guard refuses it.
    let err =
        jubarte::edit::v2_apply::apply_plan(&only, &v2_accept(&only, Some("keep"))).unwrap_err();
    assert_eq!(err.code, "CHANGE_NOT_YOURS", "{err}");
    assert!(err.message.contains(r#""Carol""#), "{err}");
}

#[test]
fn edit_next_action_restore_carries_the_effective_author() {
    let source = deleted("Jubarte");
    let value = json!({
        "schema_version": 1,
        "author": "Alice",
        "date": DATE,
        "existing_revisions": "keep",
        "operations": [{"kind": "replace", "paragraph": "p0", "find": "old", "replacement": "new"}]
    });
    let err = jubarte::edit::apply_plan(&source, &plan(&value)).unwrap_err();
    let actions = err.revision_next_actions("in.docx", &source_sha256(&source));
    let argv = &actions[0].argv;
    let author = argv.iter().position(|arg| arg == "--author").unwrap();
    let output = argv.iter().position(|arg| arg == "-o").unwrap();
    assert!(author + 1 < output, "{argv:?}");
    assert_eq!(argv[author + 1], "Alice", "{argv:?}");
    assert!(!argv.iter().any(|arg| arg == "--keep-others"), "{argv:?}");
    let mut guarded = value.clone();
    guarded["others"] = "keep".into();
    let err = jubarte::edit::apply_plan(&source, &plan(&guarded)).unwrap_err();
    let actions = err.revision_next_actions("in.docx", &source_sha256(&source));
    let argv = &actions[0].argv;
    assert!(
        argv.windows(2).any(|pair| pair == ["--author", "Alice"]),
        "{argv:?}"
    );
    assert!(argv.iter().any(|arg| arg == "--keep-others"), "{argv:?}");
    let output = argv.iter().position(|arg| arg == "-o").unwrap();
    let kept = argv.iter().position(|arg| arg == "--keep-others").unwrap();
    assert!(kept < output, "{argv:?}");
    assert!(argv.iter().any(|arg| arg == "--restore"), "{argv:?}");
}

fn v2_refuse(source: &[u8], others: Option<&str>) -> jubarte::edit::v2::PlanV2 {
    let mut value = json!({
        "schema_version": 2,
        "source": {"sha256": source_sha256(source)},
        "policy": {"track": "on", "existing_revisions": "refuse", "text_view": "final"},
        "resolve_revisions": {"accept": [], "reject": []},
        "author": "Alice",
        "date": DATE,
        "scope": {"kind": "all"},
        "operations": []
    });
    if let Some(others) = others {
        value["others"] = others.into();
    }
    jubarte::edit::v2::PlanV2::from_json(&value.to_string()).unwrap()
}

#[test]
fn plan_v2_existing_revisions_refuse_names_keep() {
    let source = deleted("Bob");
    for others in [None, Some("keep")] {
        let err =
            jubarte::edit::v2_apply::apply_plan(&source, &v2_refuse(&source, others)).unwrap_err();
        assert_eq!(err.code, "EXISTING_REVISIONS", "{others:?} {err}");
        assert!(
            err.message.contains(
                "refuse requires no remaining revision anywhere, including unknown revision forms"
            ),
            "{others:?} {err}"
        );
        assert!(err.message.contains(KEEP), "{others:?} {err}");
    }
}

#[test]
fn typed_edit_keep_others_refuses_another_authors_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), inserted("Bob")).unwrap();
    let replace = [
        "edit",
        "replace",
        "in.docx",
        "--find",
        "new",
        "--text",
        "newer",
        "--accept-change",
        "r3",
        "-a",
        "Alice",
        "-o",
        "out.docx",
    ];
    let output = run_env(dir.path(), "JUBARTE_KEEP_OTHERS", "1", &replace);
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let err = stderr(&output);
    assert!(err.contains("CHANGE_NOT_YOURS"), "{err}");
    assert!(err.contains(r#"r3 is by "Bob""#), "{err}");
    assert!(!dir.path().join("out.docx").exists());
    let loose = run_env(dir.path(), "JUBARTE_KEEP_OTHERS", "true", &replace);
    assert_eq!(loose.status.code(), Some(0), "{}", stderr(&loose));
    let mut flagged = replace.to_vec();
    flagged.insert(flagged.len() - 2, "--keep-others");
    let _ = std::fs::remove_file(dir.path().join("out.docx"));
    let output = run(dir.path(), &flagged);
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("CHANGE_NOT_YOURS"),
        "{}",
        stderr(&output)
    );

    let plan_json = r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","existing_revisions":"keep","resolve_revisions":{"accept":{"ids":["r3"]}},"operations":[]}"#;
    std::fs::write(dir.path().join("plan.json"), plan_json).unwrap();
    let apply = [
        "edit",
        "apply",
        "in.docx",
        "--plan",
        "plan.json",
        "--allow-unbound-plan",
        "-o",
        "applied.docx",
    ];
    let output = run_env(dir.path(), "JUBARTE_KEEP_OTHERS", "1", &apply);
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("CHANGE_NOT_YOURS"),
        "{}",
        stderr(&output)
    );
    let mut flagged = apply.to_vec();
    flagged.insert(flagged.len() - 2, "--keep-others");
    let output = run(dir.path(), &flagged);
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("CHANGE_NOT_YOURS"),
        "{}",
        stderr(&output)
    );

    let template = ["edit", "template", "in.docx", "--select", "p0", "-o", "-"];
    let generated = run_env(dir.path(), "JUBARTE_KEEP_OTHERS", "1", &template);
    assert_eq!(generated.status.code(), Some(0), "{}", stderr(&generated));
    let json = String::from_utf8(generated.stdout).unwrap();
    assert!(json.contains(r#""others": "keep""#), "{json}");
    let plain = run(dir.path(), &template);
    assert_eq!(plain.status.code(), Some(0), "{}", stderr(&plain));
    let json = String::from_utf8(plain.stdout).unwrap();
    assert!(!json.contains(r#""others""#), "{json}");
    let mut flagged = template.to_vec();
    flagged.insert(flagged.len() - 2, "--keep-others");
    let generated = run(dir.path(), &flagged);
    assert_eq!(generated.status.code(), Some(0), "{}", stderr(&generated));
    let json = String::from_utf8(generated.stdout).unwrap();
    assert!(json.contains(r#""others": "keep""#), "{json}");

    for action in [
        "replace", "rewrite", "insert", "delete", "apply", "template",
    ] {
        let help = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["edit", action, "--help"])
            .output()
            .unwrap();
        let text = String::from_utf8(help.stdout).unwrap();
        assert!(
            text.contains("--keep-others"),
            "{action} help omits --keep-others:\n{text}"
        );
    }
}
