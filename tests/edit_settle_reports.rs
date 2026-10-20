// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! What a settle removes, said before and after (#1159, plan 10): a change
//! names the comment marks it holds, the view shows them inside the
//! change, every removed thread says why, and a reply in a paragraph the
//! plan deletes is allowed while the deletion is tracked.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{changes_text, list_changes};
use jubarte::edit::v2::PlanV2;
use jubarte::comments::list_comments;
use jubarte::edit::{EditPlan, apply_plan};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

/// Bob's comment c0 on p1, and Carol's reply c1 when `reply`.
fn commented(reply: bool) -> Vec<u8> {
    let base = docx(
        &(para("Services")
            + &para("The Provider shall perform the Services.")
            + &para("Next paragraph.")),
    );
    let bob = plan(
        r#"{"schema_version":1,"author":"Bob","date":"2026-10-18T12:00:00Z","operations":[{"kind":"comment","paragraph":"p1","text":"Bob's note."}]}"#,
    );
    let out = apply_plan(&base, &bob).unwrap().clean;
    if !reply {
        return out;
    }
    let carol = plan(
        r#"{"schema_version":1,"author":"Carol","date":"2026-10-18T12:01:00Z","operations":[{"kind":"reply_comment","comment_id":"c0","text":"Carol agrees."}]}"#,
    );
    apply_plan(&out, &carol).unwrap().clean
}

const REF: &str = r#"<w:rPr><w:rStyle w:val="CommentReference"/></w:rPr>"#;

/// [`commented`], with Dan's r13 a deletion of the thread's reference
/// marks alone: the text stays, the marks go when r13 is accepted.
fn mark_only_deletion(reply: bool) -> Vec<u8> {
    let mut pkg = jubarte::opc::PartFs::open(&commented(reply)).unwrap();
    let ids: &[u32] = if reply { &[0, 1] } else { &[0] };
    let starts: String = ids
        .iter()
        .map(|i| format!(r#"<w:commentRangeStart w:id="{i}"/>"#))
        .collect();
    let ends: String = ids
        .iter()
        .map(|i| format!(r#"<w:commentRangeEnd w:id="{i}"/>"#))
        .collect();
    let marks: String = ids
        .iter()
        .map(|i| format!(r#"<w:r>{REF}<w:commentReference w:id="{i}"/></w:r>"#))
        .collect();
    let body = format!(
        r#"<w:p><w:r><w:t>Services</w:t></w:r></w:p><w:p>{starts}<w:r><w:t>The Provider shall perform the Services.</w:t></w:r>{ends}<w:del w:id="13" w:author="Dan" w:date="2026-10-18T12:02:00Z">{marks}</w:del></w:p><w:p><w:r><w:t>Next paragraph.</w:t></w:r></w:p>"#
    );
    let xml = format!(
        r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    pkg.set_part("word/document.xml", xml.into_bytes());
    pkg.to_zip().unwrap()
}

/// Bob's insertion with his comment on it: rejecting r1 takes the thread.
fn inserted_comment() -> Vec<u8> {
    let base = docx(&(para("Services") + &para("The Provider shall perform the Services.")));
    let bob = plan(
        r#"{"schema_version":1,"author":"Bob","date":"2026-10-18T12:00:00Z","operations":[{"kind":"insert","paragraph":"p1","position":"end","text":" Bob added this sentence.","comment":"Bob's reason."}]}"#,
    );
    apply_plan(&base, &bob).unwrap().redline
}

fn holds(docx: &[u8], id: &str) -> (serde_json::Value, String) {
    let changes = list_changes(docx).unwrap();
    let change = changes.iter().find(|c| c.id == id).expect("change listed");
    let line = changes_text(&changes)
        .lines()
        .find(|l| l.starts_with(&format!("{id}\t")))
        .unwrap()
        .to_string();
    (serde_json::to_value(change).unwrap()["holds_comments"].clone(), line)
}

#[test]
fn changes_names_the_mark_a_deletion_holds() {
    let (json, line) = holds(&mark_only_deletion(false), "r13");
    assert_eq!(json, serde_json::json!(["c0"]));
    assert_eq!(line, "r13\tdeletion\ttext\tDan\tp1\t\"\"\tholds comment c0's mark");
    // A thread's reply has its own mark beside the parent's.
    let (json, line) = holds(&mark_only_deletion(true), "r13");
    assert_eq!(json, serde_json::json!(["c0", "c1"]));
    assert!(line.ends_with("\tholds comment marks c0, c1"), "{line}");
    // An insertion holding a mark loses it when rejected.
    let redline = inserted_comment();
    let changes = list_changes(&redline).unwrap();
    let held: Vec<_> = changes
        .iter()
        .filter(|c| !c.holds_comments.is_empty())
        .map(|c| (c.id.as_str(), c.holds_comments.clone()))
        .collect();
    assert_eq!(held, [("r1", vec!["c0".to_string()])], "{changes:?}");
    // A change that holds no mark says nothing.
    let plain = docx(&format!(
        r#"<w:p><w:del w:id="3" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:r><w:delText>Gone.</w:delText></w:r></w:del></w:p>"#
    ));
    let (json, line) = holds(&plain, "r3");
    assert_eq!(json, serde_json::Value::Null);
    assert!(line.ends_with("\"Gone.\""), "{line}");
}

#[test]
fn changes_cli_names_the_mark() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), mark_only_deletion(true)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["changes", "in.docx"])
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.starts_with("r13\tdeletion\ttext\tDan\tp1\t\"\"\tholds comment marks c0, c1\n"),
        "{text}"
    );
}

fn removed(docx: &[u8], fields: serde_json::Value) -> serde_json::Value {
    let mut value = serde_json::json!({"schema_version":1,"author":"Alice",
        "date":"2026-10-18T12:05:00Z","operations":[]});
    value.as_object_mut().unwrap().extend(fields.as_object().unwrap().clone());
    let out = apply_plan(docx, &plan(&value.to_string())).unwrap();
    assert_word_valid_package(&out.redline);
    let report = serde_json::to_value(&out.report).unwrap()["removed_comments"].clone();
    let summary: serde_json::Value =
        serde_json::from_str(out.report.to_jsonl().lines().last().unwrap()).unwrap();
    assert_eq!(summary["removed_comments"], report);
    report
}

#[test]
fn removed_comments_carry_why() {
    let thread = |why: &str| {
        serde_json::json!([{"id":"c0","author":"Bob","replies":["c1"],"why":why}])
    };
    assert_eq!(
        removed(
            &mark_only_deletion(true),
            serde_json::json!({"resolve_revisions":{"accept":{"ids":["r13"]}}})
        ),
        thread("anchor removed by accepting r13")
    );
    // Every remaining change accepted: the one that held the mark is named.
    assert_eq!(
        removed(
            &mark_only_deletion(true),
            serde_json::json!({"existing_revisions":"accept"})
        ),
        thread("anchor removed by accepting r13")
    );
    // Rejecting the insertion that holds the thread.
    assert_eq!(
        removed(
            &inserted_comment(),
            serde_json::json!({"resolve_revisions":{"reject":{"ids":["r1"]}}})
        ),
        serde_json::json!([{"id":"c0","author":"Bob","replies":[],"why":"anchor removed by rejecting r1"}])
    );
}

/// Alice deletes p1, whose text Bob commented on, and replies to Bob in
/// the same plan.
const DELETE_AND_REPLY: &str = r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:05:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p1"},{"kind":"reply_comment","comment_id":"c0","text":"Alice replies."}]}"#;

fn assert_thread_on_deleted_text(redline: &[u8]) {
    assert_word_valid_package(redline);
    let listed = list_comments(redline).unwrap();
    assert_eq!(listed.len(), 2, "{listed:?}");
    assert_eq!(listed[1].parent, Some(listed[0].id));
    assert_eq!(listed[1].text, "Alice replies.");
    let xml = part_string(redline, "word/document.xml").unwrap();
    // Both references sit in the deleted paragraph's struck text.
    let del = xml.find("<w:del ").unwrap();
    assert!(xml.find("commentReference").unwrap() > del, "{xml}");
    // Reject All brings the paragraph back with its thread; Accept All
    // removes both, as Word does.
    let rejected = jubarte::document_comparer::reject_revisions(redline).unwrap();
    assert_eq!(list_comments(&rejected).unwrap().len(), 2);
    assert_word_valid_package(&rejected);
    let accepted = jubarte::document_comparer::accept_revisions(redline).unwrap();
    assert!(list_comments(&accepted).unwrap().is_empty());
    assert_word_valid_package(&accepted);
}

/// Every policy for existing revisions: the comparer path (the default
/// and `accept`) and the direct one (`keep`) both put the reply there.
#[test]
fn reply_in_a_tracked_deleted_paragraph_is_allowed() {
    for policy in [None, Some("keep"), Some("accept")] {
        let json = policy.map_or_else(
            || DELETE_AND_REPLY.to_string(),
            |p| {
                DELETE_AND_REPLY.replace(
                    "\"operations\"",
                    &format!("\"existing_revisions\":\"{p}\",\"operations\""),
                )
            },
        );
        let out = apply_plan(&commented(false), &plan(&json)).unwrap();
        assert_thread_on_deleted_text(&out.redline);
        // The clean copy has neither the paragraph nor the thread.
        assert!(list_comments(&out.clean).unwrap().is_empty(), "{policy:?}");
    }
}

fn v2_delete_and_reply(source: &[u8], track: &str) -> serde_json::Value {
    serde_json::json!({"schema_version":2,
        "source":{"sha256":jubarte::inspect::source_sha256(source)},
        "policy":{"track":track,"existing_revisions":"keep","text_view":"final"},
        "resolve_revisions":{"accept":[],"reject":[]},"author":"Alice",
        "date":"2026-10-18T12:05:00Z","scope":{"kind":"all"},
        "operations":[
            {"kind":"delete_paragraph","target":{"short_id":"p1","long_id":"document.w:p.j:short.p1"}},
            {"kind":"reply_comment","target":{"short_id":"c0","long_id":"comments.w:comment.w:id.0"},"text":"Alice replies."}
        ]})
}

#[test]
fn reply_in_a_tracked_v2_deleted_paragraph_is_allowed() {
    let source = commented(false);
    let plan = PlanV2::from_json(&v2_delete_and_reply(&source, "on").to_string()).unwrap();
    let out = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap();
    assert_thread_on_deleted_text(&out.redline);
}

/// Editing mode deletes the paragraph outright, the thread's reference
/// with it: the reply has nothing to attach to. The refusal is the one an
/// untracked deletion of any comment's anchor gets.
#[test]
fn reply_in_an_editing_mode_deleted_paragraph_is_refused_with_comment_anchor_removed() {
    let source = commented(false);
    let plan = PlanV2::from_json(&v2_delete_and_reply(&source, "off").to_string()).unwrap();
    let error = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap_err();
    assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
    assert_eq!(error.operation.as_deref(), Some("/operations/0"), "{error}");
}

