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
use jubarte::comments::list_comments;
use jubarte::edit::v2::PlanV2;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

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
    with_p1(
        reply,
        &format!(
            r#"{starts}<w:r><w:t>The Provider shall perform the Services.</w:t></w:r>{ends}<w:del w:id="13" w:author="Dan" w:date="2026-10-18T12:02:00Z">{marks}</w:del>"#
        ),
    )
}

/// [`commented`]'s comment on "Kept.", its reference mark the start of
/// Dan's r13, which goes on to delete " Gone.".
fn mark_before_deleted_text() -> Vec<u8> {
    with_p1(
        false,
        &format!(
            r#"<w:commentRangeStart w:id="0"/><w:r><w:t>Kept.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:del w:id="13" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:r>{REF}<w:commentReference w:id="0"/></w:r><w:r><w:delText xml:space="preserve"> Gone.</w:delText></w:r></w:del>"#
        ),
    )
}

/// [`commented`]'s parts, its p1 holding `p1`.
fn with_p1(reply: bool, p1: &str) -> Vec<u8> {
    with_paragraphs(reply, &[p1])
}

/// [`commented`]'s parts with `paragraphs` between "Services" and "Next
/// paragraph.", each the content of one `w:p`.
fn with_paragraphs(reply: bool, paragraphs: &[&str]) -> Vec<u8> {
    let middle: String = paragraphs
        .iter()
        .map(|p| format!("<w:p>{p}</w:p>"))
        .collect();
    with_body(reply, &middle)
}

/// [`commented`]'s parts with the body `middle` between "Services" and
/// "Next paragraph.".
fn with_body(reply: bool, middle: &str) -> Vec<u8> {
    let mut pkg = jubarte::opc::PartFs::open(&commented(reply)).unwrap();
    let body = format!(
        r#"<w:p><w:r><w:t>Services</w:t></w:r></w:p>{middle}<w:p><w:r><w:t>Next paragraph.</w:t></w:r></w:p>"#
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
    (
        serde_json::to_value(change).unwrap()["holds_comments"].clone(),
        line,
    )
}

#[test]
fn changes_names_the_mark_a_deletion_holds() {
    let (json, line) = holds(&mark_only_deletion(false), "r13");
    assert_eq!(json, serde_json::json!(["c0"]));
    assert_eq!(
        line,
        "r13\tdeletion\ttext\tDan\tp1\t\"\"\tholds comment c0's mark"
    );
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
    let plain = docx(
        r#"<w:p><w:del w:id="3" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:r><w:delText>Gone.</w:delText></w:r></w:del></w:p>"#,
    );
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

/// The agent view's p1, id line and text.
fn view_p1(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    let view = read(docx, &options).unwrap().markdown;
    view.lines()
        .skip_while(|line| !line.starts_with("<!-- p1"))
        .take(2)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn view_shows_the_mark_inside_the_deletion() {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    for (source, shown) in [
        (
            mark_only_deletion(false),
            "<!-- p1 rev-del r13 @D -->\n{==The Provider shall perform the Services.==}{>>c0 @B in r13: Bob's note.<<}",
        ),
        (
            mark_only_deletion(true),
            "<!-- p1 rev-del r13 @D -->\n{==The Provider shall perform the Services.==}{>>c0 @B in r13: Bob's note.<<}{>>c1 @C re c0 in r13: Carol agrees.<<}",
        ),
        (
            mark_before_deleted_text(),
            "<!-- p1 -->\n{==Kept.==}{>>c0 @B in r13: Bob's note.<<}{-- Gone.--}{r13 @D}",
        ),
    ] {
        assert_eq!(view_p1(&source), shown);
        // Converted back, r13 holds the same marks, so accepting it still
        // takes the thread; the comments keep their text and replies.
        let view = read(&source, &options).unwrap().markdown;
        let back = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert_eq!(holds(&back.docx, "r13"), holds(&source, "r13"), "{shown}");
        let thread = |docx: &[u8]| {
            list_comments(docx)
                .unwrap()
                .into_iter()
                .map(|c| (c.id, c.text, c.parent))
                .collect::<Vec<_>>()
        };
        assert_eq!(thread(&back.docx), thread(&source), "{shown}");
        assert_eq!(view_p1(&back.docx), shown);
        assert_word_valid_package(&back.docx);
    }
}

#[test]
fn a_note_inside_its_deletion_needs_no_in() {
    // The whole range and the mark in r13: the note prints inside it.
    let source = with_p1(
        false,
        &format!(
            r#"<w:r><w:t xml:space="preserve">Kept. </w:t></w:r><w:del w:id="13" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:commentRangeStart w:id="0"/><w:r><w:delText>Gone.</w:delText></w:r><w:commentRangeEnd w:id="0"/><w:r>{REF}<w:commentReference w:id="0"/></w:r></w:del>"#
        ),
    );
    assert_eq!(
        view_p1(&source),
        "<!-- p1 -->\nKept. {--{==Gone.==}{>>c0 @B: Bob's note.<<}--}{r13 @D}"
    );
}

#[test]
fn a_note_in_a_change_the_paragraph_lacks_keeps_its_mark_outside() {
    let view = "---\nauthors:\n  B: Bob # 1 comment, 2026-10-18T12:00:00Z\n---\n\n{==Kept.==}{>>c0 @B in r13: Bob's note.<<}\n";
    let back = markdown_to_docx(view, &DocxOptions::default()).unwrap();
    assert_eq!(
        back.warnings,
        ["c0 is in r13, which its paragraph does not have: its mark stays after its text"]
    );
    let comments = list_comments(&back.docx).unwrap();
    assert_eq!(comments[0].text, "Bob's note.");
    assert!(list_changes(&back.docx).unwrap().is_empty());
}

fn removed(docx: &[u8], fields: &serde_json::Value) -> serde_json::Value {
    let mut value = serde_json::json!({"schema_version":1,"author":"Alice",
        "date":"2026-10-18T12:05:00Z","operations":[]});
    value
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    let out = apply_plan(docx, &plan(&value.to_string())).unwrap();
    assert_word_valid_package(&out.redline);
    let report = serde_json::to_value(&out.report).unwrap()["removed_comments"].clone();
    let summary: serde_json::Value =
        serde_json::from_str(out.report.to_jsonl().lines().last().unwrap()).unwrap();
    // The report leaves out a receipt with nothing in it; the summary line
    // lists it empty.
    if report.is_null() {
        assert_eq!(summary["removed_comments"], serde_json::json!([]));
        return serde_json::json!([]);
    }
    assert_eq!(summary["removed_comments"], report);
    report
}

#[test]
fn removed_comments_carry_why() {
    let thread =
        |why: &str| serde_json::json!([{"id":"c0","author":"Bob","replies":["c1"],"why":why}]);
    assert_eq!(
        removed(
            &mark_only_deletion(true),
            &serde_json::json!({"resolve_revisions":{"accept":{"ids":["r13"]}}})
        ),
        thread("anchor removed by accepting r13")
    );
    // Every remaining change accepted: the one that held the mark is named.
    assert_eq!(
        removed(
            &mark_only_deletion(true),
            &serde_json::json!({"existing_revisions":"accept"})
        ),
        thread("anchor removed by accepting r13")
    );
    // Rejecting the insertion that holds the thread.
    assert_eq!(
        removed(
            &inserted_comment(),
            &serde_json::json!({"resolve_revisions":{"reject":{"ids":["r1"]}}})
        ),
        serde_json::json!([{"id":"c0","author":"Bob","replies":[],"why":"anchor removed by rejecting r1"}])
    );
}

/// Dan's insertion r20, Eve's deletion r21 of its end inside it, and c0's
/// mark in r21: accepting r21 or rejecting r20 takes the thread, accepting
/// r20 or rejecting r21 keeps it.
fn nested_holders() -> Vec<u8> {
    with_p1(
        false,
        &format!(
            r#"<w:commentRangeStart w:id="0"/><w:r><w:t xml:space="preserve">Kept. </w:t></w:r><w:ins w:id="20" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:r><w:t xml:space="preserve">Typed </w:t></w:r><w:del w:id="21" w:author="Eve" w:date="2026-10-18T12:03:00Z"><w:r><w:delText>then struck</w:delText></w:r><w:commentRangeEnd w:id="0"/><w:r>{REF}<w:commentReference w:id="0"/></w:r></w:del></w:ins>"#
        ),
    )
}

/// Dan's move of "Moved words." from p1 (r7) to p2 (r8), c0 on the moved-to
/// words: rejecting either side takes the moved-to side and the thread.
fn moved_thread() -> Vec<u8> {
    with_paragraphs(
        false,
        &[
            r#"<w:r><w:t xml:space="preserve">Lead. </w:t></w:r><w:moveFromRangeStart w:id="90" w:author="Dan" w:date="2026-10-18T12:02:00Z" w:name="mv1"/><w:moveFrom w:id="7" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:r><w:delText>Moved words.</w:delText></w:r></w:moveFrom><w:moveFromRangeEnd w:id="90"/>"#,
            &format!(
                r#"<w:moveToRangeStart w:id="91" w:author="Dan" w:date="2026-10-18T12:02:00Z" w:name="mv1"/><w:moveTo w:id="8" w:author="Dan" w:date="2026-10-18T12:02:00Z"><w:commentRangeStart w:id="0"/><w:r><w:t>Moved words.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r>{REF}<w:commentReference w:id="0"/></w:r></w:moveTo><w:moveToRangeEnd w:id="91"/>"#
            ),
        ],
    )
}

/// `why` names a change the pass settled whose resolution removes its
/// content: not an enclosing change of the other kind the plan never
/// touched, and the side of a move the plan named, not the side it took
/// along (#1342 review).
#[test]
fn why_names_the_change_the_pass_settled() {
    let why = |docx: &[u8], fields: serde_json::Value| {
        let report = removed(docx, &fields);
        report
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["why"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    let select = |verb: &str, id: &str| serde_json::json!({"resolve_revisions":{verb:{"ids":[id]}},"existing_revisions":"keep"});
    let nested = nested_holders();
    assert_eq!(
        why(&nested, select("accept", "r21")),
        ["anchor removed by accepting r21"]
    );
    assert_eq!(
        why(&nested, select("reject", "r20")),
        ["anchor removed by rejecting r20"]
    );
    assert!(why(&nested, select("accept", "r20")).is_empty());
    assert!(why(&nested, select("reject", "r21")).is_empty());
    // A blanket pass names the change of its kind that held the mark.
    assert_eq!(
        why(&nested, serde_json::json!({"existing_revisions":"accept"})),
        ["anchor removed by accepting r21"]
    );
    assert_eq!(
        why(&nested, serde_json::json!({"existing_revisions":"reject"})),
        ["anchor removed by rejecting r20"]
    );
    // r20 accepted by id, then the rest rejected: the blanket rejection of
    // r21 keeps its text, so the thread stays.
    assert!(
        why(
            &nested,
            serde_json::json!({"resolve_revisions":{"accept":{"ids":["r20"]}},"existing_revisions":"reject"})
        )
        .is_empty()
    );
    let moved = moved_thread();
    assert_eq!(
        why(&moved, select("reject", "r7")),
        ["anchor removed by rejecting r7"]
    );
    assert_eq!(
        why(&moved, select("reject", "r8")),
        ["anchor removed by rejecting r8"]
    );
    assert!(why(&moved, select("accept", "r7")).is_empty());
}

/// Alice deletes p1, whose text Bob commented on, and replies to Bob in
/// the same plan.
const DELETE_AND_REPLY: &str = r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:05:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p1"},{"kind":"reply_comment","comment_id":"c0","text":"Alice replies."}]}"#;

fn assert_thread_on_deleted_text(redline: &[u8]) {
    assert_thread_of_on_deleted_text(redline, 2);
}

/// Whether the `<w:commentReference w:id="{id}"/>` of `xml` sits inside a
/// `w:del`.
fn reference_is_deleted(xml: &str, id: u32) -> bool {
    let at = xml
        .find(&format!(r#"<w:commentReference w:id="{id}""#))
        .unwrap_or_else(|| panic!("c{id} has a reference: {xml}"));
    let before = &xml[..at];
    // A paragraph mark's `<w:del …/>` opens nothing.
    let opened = before
        .match_indices("<w:del ")
        .filter(|&(i, _)| !before[i..].split('>').next().unwrap().ends_with('/'))
        .count();
    opened > before.matches("</w:del>").count()
}

/// The thread of `count` comments, Alice's reply last, on p1's struck
/// text: every reference inside the deletion, the new reply's own too.
fn assert_thread_of_on_deleted_text(redline: &[u8], count: usize) {
    assert_word_valid_package(redline);
    let listed = list_comments(redline).unwrap();
    assert_eq!(listed.len(), count, "{listed:?}");
    let reply = listed.last().unwrap();
    assert_eq!(reply.parent, Some(listed[0].id));
    assert_eq!(reply.text, "Alice replies.");
    let xml = part_string(redline, "word/document.xml").unwrap();
    for c in &listed {
        assert!(
            reference_is_deleted(&xml, c.id),
            "c{} outside the deletion: {xml}",
            c.id
        );
    }
    // Reject All brings the paragraph back with its thread; Accept All
    // removes it all, as Word does.
    let rejected = jubarte::document_comparer::reject_revisions(redline).unwrap();
    assert_eq!(list_comments(&rejected).unwrap().len(), count);
    assert_word_valid_package(&rejected);
    let accepted = jubarte::document_comparer::accept_revisions(redline).unwrap();
    assert!(list_comments(&accepted).unwrap().is_empty());
    assert_word_valid_package(&accepted);
}

/// Every policy for existing revisions: the comparer path (the default
/// and `accept`) and the direct one (`keep`) both put the reply there, on
/// a thread with no reply yet and on one Carol already replied to.
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
        let out = apply_plan(&commented(true), &plan(&json)).unwrap();
        assert_thread_of_on_deleted_text(&out.redline, 3);
        assert!(list_comments(&out.clean).unwrap().is_empty(), "{policy:?}");
    }
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
    v2_delete_and_reply_under(source, track, "keep")
}

fn v2_delete_and_reply_under(source: &[u8], track: &str, existing: &str) -> serde_json::Value {
    serde_json::json!({"schema_version":2,
    "source":{"sha256":jubarte::inspect::source_sha256(source)},
    "policy":{"track":track,"existing_revisions":existing,"text_view":"final"},
    "resolve_revisions":{"accept":[],"reject":[]},"author":"Alice",
    "date":"2026-10-18T12:05:00Z","scope":{"kind":"all"},
    "operations":[
        {"kind":"delete_paragraph","target":{"short_id":"p1","long_id":"document.w:p.j:short.p1"}},
        {"kind":"reply_comment","target":{"short_id":"c0","long_id":"comments.w:comment.w:id.0"},"text":"Alice replies."}
    ]})
}

/// Both v2 routes: the direct one (`keep`) and the comparer's (`accept`).
#[test]
fn reply_in_a_tracked_v2_deleted_paragraph_is_allowed() {
    for existing in ["keep", "accept"] {
        let source = commented(false);
        let json = v2_delete_and_reply_under(&source, "on", existing).to_string();
        let plan = PlanV2::from_json(&json).unwrap();
        let out = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap();
        assert_thread_on_deleted_text(&out.redline);
    }
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
    // The deletion is what removes the anchor: alone, it is refused the same.
    let mut alone = v2_delete_and_reply(&source, "off");
    alone["operations"].as_array_mut().unwrap().truncate(1);
    let plan = PlanV2::from_json(&alone.to_string()).unwrap();
    let error = jubarte::edit::v2_apply::apply_plan(&source, &plan).unwrap_err();
    assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
    assert_eq!(error.operation.as_deref(), Some("/operations/0"), "{error}");
}

/// The v1 face of the refusal above: `--editing-mode` publishes the clean
/// copy, where the paragraph and c0's thread are gone, so a reply to c0 in
/// the same plan would land nowhere. The shared hook refuses it before
/// anything is written, for the CLI and the Python and npm bindings alike.
#[test]
fn editing_mode_refuses_a_reply_on_a_paragraph_the_plan_deletes() {
    let source = commented(false);
    let message = jubarte::edit::flags::editing_mode_conflict(&plan(DELETE_AND_REPLY), &source)
        .expect("editing mode loses the reply");
    assert!(
        message.contains("c0") && message.contains("p1"),
        "{message}"
    );
    // The reply alone, or the deletion alone, has nothing to lose.
    for alone in [
        r#"{"kind":"delete_paragraph","paragraph":"p1"}"#,
        r#"{"kind":"reply_comment","comment_id":"c0","text":"Alice replies."}"#,
    ] {
        let one = plan(&format!(
            r#"{{"schema_version":1,"author":"Alice","date":"2026-10-18T12:05:00Z","operations":[{alone}]}}"#
        ));
        assert_eq!(
            jubarte::edit::flags::editing_mode_conflict(&one, &source),
            None,
            "{alone}"
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::write(&input, &source).unwrap();
    let output = dir.path().join("out.docx");
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .arg(&input)
        .args(["--editing-mode", "-a", "Alice", "-p", "p1", "--delete"])
        .args(["-p", "c0", "--reply", "Alice replies."])
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(run.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("--editing-mode"), "{stderr}");
    assert!(!output.exists(), "a refused edit writes nothing");
    // Tracked, the same flags keep the reply on the struck paragraph.
    let tracked = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .arg(&input)
        .args(["-a", "Alice", "-p", "p1", "--delete"])
        .args(["-p", "c0", "--reply", "Alice replies."])
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        tracked.status.success(),
        "{}",
        String::from_utf8_lossy(&tracked.stderr)
    );
    assert_thread_on_deleted_text(&std::fs::read(&output).unwrap());
}

/// Two identical paragraphs, Bob's comment on the first: deleting it and
/// replying to Bob, the comparer strikes the twin instead, so the reply
/// would sit on text that stays. The refusal says so about the reply and
/// names the policy that writes the deletion as given; under it the
/// thread goes on the struck paragraph.
#[test]
fn a_reply_the_comparer_would_leave_on_kept_text_is_refused_with_keep() {
    let base = docx(
        &(para("Lead.") + &para("Same words here.") + &para("Same words here.") + &para("Tail.")),
    );
    let bob = plan(
        r#"{"schema_version":1,"author":"Bob","date":"2026-10-18T12:00:00Z","operations":[{"kind":"comment","paragraph":"p1","text":"Bob's note."}]}"#,
    );
    let source = apply_plan(&base, &bob).unwrap().clean;
    let error = apply_plan(&source, &plan(DELETE_AND_REPLY)).unwrap_err();
    assert_eq!(error.code, "UNSUPPORTED_STRUCTURE", "{error}");
    assert!(
        error
            .message
            .contains("the reply would sit on text that stays")
            && error.message.contains(r#""existing_revisions": "keep""#),
        "{error}"
    );
    let keep = DELETE_AND_REPLY.replace(
        "\"operations\"",
        "\"existing_revisions\":\"keep\",\"operations\"",
    );
    let out = apply_plan(&source, &plan(&keep)).unwrap();
    assert_thread_on_deleted_text(&out.redline);
    let xml = part_string(&out.redline, "word/document.xml").unwrap();
    let first = xml.find("Same words here.").unwrap();
    assert!(
        xml[..first].contains("<w:del "),
        "the first copy is struck: {xml}"
    );
}

/// The comments whose mark change `id` holds, as `changes --json` lists them.
fn held(docx: &[u8], id: &str) -> Vec<String> {
    list_changes(docx)
        .unwrap()
        .into_iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("{id} listed"))
        .holds_comments
}

const DAN: &str = r#"w:author="Dan" w:date="2026-10-18T12:02:00Z""#;

/// A change holds the marks in the content it removes: a deleted row the
/// marks in its cells, the row of a nested table only those in it, not the
/// outer cell's around it.
#[test]
fn a_deleted_row_holds_the_marks_in_its_own_cells() {
    let cell = |content: &str| {
        format!(r#"<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{content}</w:tc>"#)
    };
    let table = |rows: &str| {
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid>{rows}</w:tbl>"#
        )
    };
    let marked = |id: u32, text: &str| {
        format!(
            r#"<w:p><w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r>{REF}<w:commentReference w:id="{id}"/></w:r></w:p>"#
        )
    };
    let inner = table(&format!(
        r#"<w:tr><w:trPr><w:del w:id="41" {DAN}/></w:trPr>{}</w:tr>"#,
        cell("<w:p><w:r><w:t>Inner.</w:t></w:r></w:p>")
    ));
    let body = table(&format!(
        r#"<w:tr><w:trPr><w:del w:id="40" {DAN}/></w:trPr>{}</w:tr><w:tr>{}</w:tr>"#,
        cell(&marked(0, "Row text.")),
        cell(&format!("{}{inner}<w:p/>", marked(1, "Outer cell.")))
    ));
    let docx = with_body(true, &body);
    assert_eq!(held(&docx, "r40"), ["c0"]);
    assert!(held(&docx, "r41").is_empty(), "{:?}", held(&docx, "r41"));
}

/// A deleted paragraph mark and a formatting change remove no content, so
/// they hold no mark, whatever the paragraph or run holds.
#[test]
fn a_paragraph_mark_or_formatting_change_holds_no_mark() {
    let docx = with_p1(
        false,
        &format!(
            r#"<w:pPr><w:rPr><w:del w:id="42" {DAN}/></w:rPr></w:pPr><w:commentRangeStart w:id="0"/><w:r><w:t>Text.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/><w:b/><w:rPrChange w:id="43" {DAN}><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr></w:rPrChange></w:rPr><w:commentReference w:id="0"/></w:r>"#
        ),
    );
    assert!(held(&docx, "r42").is_empty());
    assert!(held(&docx, "r43").is_empty());
}

/// A receipt written before `why` existed still reads, its `why` empty.
#[test]
fn a_receipt_without_why_reads_with_an_empty_one() {
    let old: jubarte::edit::RemovedComment =
        serde_json::from_str(r#"{"id":"c0","author":"Bob","replies":["c1"]}"#).unwrap();
    assert_eq!(old.why, "");
    assert_eq!(old.replies, ["c1"]);
}
