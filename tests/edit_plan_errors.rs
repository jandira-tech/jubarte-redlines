// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A plan error teaches: it names the operation, its kind, the field, the
//! fields the kind takes and the nearest one. Three slips apply with a
//! note instead: `comment` on `delete` and `rewrite` (a real field), the
//! `comment` spelling of a comment's `text`, and `anchor` for `find`.

mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan_json, dispatched_operation_kinds};

const HEAD: &str = r#""schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z""#;

fn source() -> Vec<u8> {
    docx(r#"<w:p><w:r><w:t>Payment is due in thirty days.</w:t></w:r></w:p>"#)
}

fn plan(operations: &str) -> String {
    format!(r#"{{{HEAD},"operations":[{operations}]}}"#)
}

fn error(json: &str) -> String {
    EditPlan::from_json(json).unwrap_err().to_string()
}

fn kinds() -> String {
    dispatched_operation_kinds().join(", ")
}

#[test]
fn an_unknown_field_names_the_kind_the_fields_and_the_nearest() {
    let json = plan(
        r#"{"kind":"comment","paragraph":{"index":0},"text":"fine"},
           {"kind":"delete","paragraph":{"index":0},"find":"thirty","ocurrence":1}"#,
    );
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-2): delete: unknown field "ocurrence"; did you mean "occurrence"? fields: comment, find, occurrence, paragraph"#
    );
}

#[test]
fn an_operation_id_names_the_operation() {
    let json = plan(r#"{"id":"pay","kind":"delete","paragraph":{"index":0},"fnd":"thirty"}"#);
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (pay): delete: unknown field "fnd"; did you mean "find"? fields: comment, find, occurrence, paragraph"#
    );
}

#[test]
fn a_field_with_no_near_name_lists_the_fields() {
    let json = plan(r#"{"kind":"delete","paragraph":{"index":0},"find":"thirty","urgency":1}"#);
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): delete: unknown field "urgency"; fields: comment, find, occurrence, paragraph"#
    );
}

#[test]
fn text_on_replace_stays_an_error() {
    let json = plan(r#"{"kind":"replace","paragraph":{"index":0},"find":"thirty","text":"forty"}"#);
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): replace: unknown field "text"; did you mean "replacement"? fields: comment, find, format, occurrence, paragraph, replacement, whole"#
    );
}

#[test]
fn schema_version_is_required() {
    let json = r#"{"author":"Ann Counsel","operations":[]}"#;
    assert_eq!(
        error(json),
        "INVALID_PLAN: schema_version is required (1 for this build)"
    );
}

#[test]
fn characters_after_the_plan_say_it_is_one_object() {
    let json = format!("{}\n}}", plan(""));
    assert_eq!(
        error(&json),
        "INVALID_PLAN: the plan must be exactly one JSON object; characters follow it at line 2 column 1"
    );
}

#[test]
fn trailing_characters_give_where_they_start() {
    let one = plan("");
    for (tail, column) in [("  trailing junk", one.len() + 3), ("\0", one.len() + 1)] {
        assert_eq!(
            error(&format!("{one}{tail}")),
            format!(
                "INVALID_PLAN: the plan must be exactly one JSON object; characters follow it at line 1 column {column}"
            ),
            "{tail:?}"
        );
    }
}

/// A byte order mark before the plan is no character of it (RFC 8259
/// §8.1: a parser may ignore one), in the library and in `edit apply`.
#[test]
fn a_byte_order_mark_before_the_plan_is_ignored() {
    let json = format!("\u{feff}{}", plan(""));
    assert!(EditPlan::from_json(&json).is_ok(), "{}", error(&json));
    let v2 = r#"{"schema_version":2,"author":"Ann Counsel","operations":[]}"#;
    let message = |json: &str| {
        jubarte::edit::v2::PlanV2::from_json(json)
            .err()
            .map(|e| e.message)
    };
    assert_eq!(message(&format!("\u{feff}{v2}")), message(v2));
    let dir = tempfile::tempdir().unwrap();
    let (input, plan_file) = (dir.path().join("in.docx"), dir.path().join("plan.json"));
    std::fs::write(&input, source()).unwrap();
    std::fs::write(&plan_file, &json).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["edit", "apply"])
        .arg(&input)
        .arg("--plan")
        .arg(&plan_file)
        .arg("--dry-run")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("INVALID_PLAN"), "{stderr}");
}

#[test]
fn an_id_is_a_string() {
    let json = plan(r#"{"id":42,"kind":"delete","paragraph":{"index":0},"find":"thirty"}"#);
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): id is a string, not a number: write "42""#
    );
    let json = plan(r#"{"id":{"n":1},"kind":"delete","paragraph":{"index":0},"find":"thirty"}"#);
    assert_eq!(
        error(&json),
        "INVALID_PLAN (op-1): id is a string, not an object"
    );
}

#[test]
fn a_paragraph_number_that_is_no_index_says_what_one_is() {
    for n in ["3.5", "-1"] {
        let json = plan(&format!(
            r#"{{"kind":"delete","paragraph":{n},"find":"thirty"}}"#
        ));
        assert_eq!(
            error(&json),
            r#"INVALID_PLAN (op-1): delete: paragraph is a selector, not a number: write {"index": N}, N a whole number from 0"#,
            "{n}"
        );
    }
}

#[test]
fn an_alias_beside_the_field_it_reads_as_says_to_drop_one() {
    let json = plan(
        r#"{"kind":"replace","paragraph":{"index":0},"anchor":"thirty","find":"thirty","replacement":"forty"}"#,
    );
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): replace: "anchor" is read as "find", which the operation gives too: drop one"#
    );
}

#[test]
fn an_unknown_kind_lists_the_kinds() {
    let json =
        plan(r#"{"kind":"replce","paragraph":{"index":0},"find":"thirty","replacement":"forty"}"#);
    assert_eq!(
        error(&json),
        format!(
            r#"INVALID_PLAN (op-1): unknown kind "replce"; did you mean "replace"? kinds: {}"#,
            kinds()
        )
    );
}

#[test]
fn a_missing_kind_lists_the_kinds() {
    let json = plan(r#"{"paragraph":{"index":0},"find":"thirty"}"#);
    assert_eq!(
        error(&json),
        format!("INVALID_PLAN (op-1): kind is required; kinds: {}", kinds())
    );
}

#[test]
fn ids_given_as_a_string_say_they_are_an_array() {
    for side in ["accept", "reject"] {
        let json = format!(
            r#"{{{HEAD},"resolve_revisions":{{"{side}":{{"ids":"r1"}}}},"operations":[]}}"#
        );
        assert_eq!(
            error(&json),
            format!(r#"INVALID_PLAN: resolve_revisions.{side}.ids is an array: write ["r1"]"#),
            "{side}"
        );
    }
}

#[test]
fn a_numeric_paragraph_stays_an_error_that_shows_a_selector() {
    let json = plan(r#"{"kind":"delete","paragraph":3,"find":"thirty"}"#);
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): delete: paragraph is a selector, not a number: write {"index": 3}"#
    );
}

#[test]
fn position_with_after_stays_an_error() {
    let json = plan(
        r#"{"kind":"insert","paragraph":{"index":0},"after":"due","position":"end","text":" soon"}"#,
    );
    let message = apply_plan_json(&source(), &json).unwrap_err().to_string();
    assert!(
        message.contains("insert needs exactly one of after, before, position"),
        "{message}"
    );
}

/// Through the comparer (a clean source) and emitted directly (`keep`),
/// a deleted text's comment wraps its deletion.
#[test]
fn comment_on_delete_anchors_to_the_deleted_text() {
    let op = r#"{"kind":"delete","paragraph":{"index":0},"find":"thirty ","comment":"Net 30 is too short."}"#;
    let compared = plan(op);
    let kept = compared.replacen('{', r#"{"existing_revisions":"keep","#, 1);
    for json in [compared, kept] {
        comment_wraps_the_deletion(&json);
    }
}

fn comment_wraps_the_deletion(json: &str) {
    let result = apply_plan_json(&source(), json).unwrap();
    assert!(result.report.notes.is_empty(), "{:?}", result.report.notes);
    let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
    let at = |needle: &str| {
        body.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing from {body}"))
    };
    let start = at("<w:commentRangeStart");
    let del = at("<w:del ");
    let deleted = at("<w:delText");
    let close = at("</w:del>");
    let end = at("<w:commentRangeEnd");
    let reference = at("<w:commentReference");
    assert!(
        start < del && del < deleted && deleted < close && close < end && end < reference,
        "{body}"
    );
    assert!(body[deleted..close].contains("thirty"), "{body}");
    let comments = part_string(&result.redline, "word/comments.xml").expect("word/comments.xml");
    assert!(comments.contains("Net 30 is too short."), "{comments}");
    // Rejecting the deletion gives the comment its words back. Word's
    // Accept All drops the comment with them, as the clean copy does; ours
    // keeps it for now (#1295).
    let filter = ChangeFilter::default();
    let accepted = accept_changes(&result.redline, &filter).unwrap();
    let accepted = part_string(&accepted, "word/document.xml").expect("word/document.xml");
    assert!(!accepted.contains("thirty"), "{accepted}");
    let rejected = reject_changes(&result.redline, &filter).unwrap();
    let rejected = part_string(&rejected, "word/document.xml").expect("word/document.xml");
    let at = |needle: &str| {
        rejected
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} missing from {rejected}"))
    };
    assert!(
        at("<w:commentRangeStart") < at("thirty") && at("thirty") < at("<w:commentRangeEnd"),
        "{rejected}"
    );
    assert_word_valid_package(&result.redline);
    assert_word_valid_package(&result.clean);
}

/// The words comment `id`'s range holds in `body`, deleted or kept.
fn wrapped(body: &str, id: &str) -> String {
    let start = body
        .find(&format!(r#"<w:commentRangeStart w:id="{id}""#))
        .unwrap_or_else(|| panic!("no range start {id} in {body}"));
    let end = body
        .find(&format!(r#"<w:commentRangeEnd w:id="{id}""#))
        .unwrap_or_else(|| panic!("no range end {id} in {body}"));
    let mut words = String::new();
    let mut rest = &body[start..end];
    while let Some(open) = rest.find("<w:t").or_else(|| rest.find("<w:delText")) {
        let next = [rest.find("<w:t"), rest.find("<w:delText")]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(open);
        let after = &rest[next..];
        let text = &after[after.find('>').unwrap() + 1..];
        let stop = text.find('<').unwrap();
        if !after.starts_with("<w:tab") && !after[..after.find('>').unwrap()].ends_with('/') {
            words.push_str(&text[..stop]);
        }
        rest = &text[stop..];
    }
    words
}

/// Two deletions side by side, each with a comment. The comparer writes
/// them as one deletion; each comment still wraps its own words, and no
/// comment marker sits inside a deletion, so Accept All gives both comments
/// one fate (#1295).
#[test]
fn adjacent_deletions_each_keep_their_comment() {
    let compared = plan(
        r#"{"kind":"delete","paragraph":{"index":0},"find":"is ","comment":"first"},{"kind":"delete","paragraph":{"index":0},"find":"due ","comment":"second"}"#,
    );
    let kept = compared.replacen('{', r#"{"existing_revisions":"keep","#, 1);
    for json in [compared, kept] {
        let result = apply_plan_json(&source(), &json).unwrap();
        let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
        assert_eq!(wrapped(&body, "0"), "is ", "{json}: {body}");
        assert_eq!(wrapped(&body, "1"), "due ", "{json}: {body}");
        for deletion in body.split("<w:del ").skip(1) {
            let inside = &deletion[..deletion.find("</w:del>").unwrap()];
            assert!(!inside.contains("<w:comment"), "{json}: {body}");
        }
        let filter = ChangeFilter::default();
        let accepted = accept_changes(&result.redline, &filter).unwrap();
        let comments = part_string(&accepted, "word/comments.xml").unwrap_or_default();
        assert_eq!(
            comments.contains(">first<"),
            comments.contains(">second<"),
            "{json}: {comments}"
        );
        let accepted = part_string(&accepted, "word/document.xml").expect("word/document.xml");
        assert_eq!(
            accepted.matches("<w:commentRangeStart").count(),
            accepted.matches("<w:commentReference").count(),
            "{json}: {accepted}"
        );
        let rejected = reject_changes(&result.redline, &filter).unwrap();
        let rejected = part_string(&rejected, "word/document.xml").expect("word/document.xml");
        assert_eq!(wrapped(&rejected, "0"), "is ", "{json}: {rejected}");
        assert_eq!(wrapped(&rejected, "1"), "due ", "{json}: {rejected}");
        assert_word_valid_package(&result.redline);
        assert_word_valid_package(&result.clean);
    }
    // A comment on the middle one of three deletions that touch splits the
    // comparer's one deletion on both sides of it.
    let json = plan(
        r#"{"kind":"delete","paragraph":{"index":0},"find":"is "},{"kind":"delete","paragraph":{"index":0},"find":"due ","comment":"middle"},{"kind":"delete","paragraph":{"index":0},"find":"in "}"#,
    );
    let result = apply_plan_json(&source(), &json).unwrap();
    let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
    assert_eq!(wrapped(&body, "0"), "due ", "{body}");
    assert_eq!(body.matches("<w:del ").count(), 3, "{body}");
    for deletion in body.split("<w:del ").skip(1) {
        let inside = &deletion[..deletion.find("</w:del>").unwrap()];
        assert!(!inside.contains("<w:comment"), "{body}");
    }
    let filter = ChangeFilter::default();
    let rejected = reject_changes(&result.redline, &filter).unwrap();
    assert_eq!(
        wrapped(
            &part_string(&rejected, "word/document.xml").expect("word/document.xml"),
            "0"
        ),
        "due "
    );
    assert_word_valid_package(&result.redline);
}

#[test]
fn comment_on_rewrite_anchors_to_the_rewritten_paragraph() {
    let json = plan(
        r#"{"kind":"rewrite","paragraph":{"index":0},"text":"Payment is due in forty days.","comment":"Matches the term sheet."}"#,
    );
    let result = apply_plan_json(&source(), &json).unwrap();
    assert!(result.report.notes.is_empty(), "{:?}", result.report.notes);
    let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
    let start = body.find("<w:commentRangeStart").expect("range start");
    let end = body.find("<w:commentRangeEnd").expect("range end");
    let del = body.find("<w:del ").expect("deletion");
    let ins = body.find("<w:ins ").expect("insertion");
    assert!(start < del.min(ins) && del.max(ins) < end, "{body}");
    assert_word_valid_package(&result.redline);
}

#[test]
fn alias_text_on_comment_op_applies_with_a_note() {
    for kind in ["comment", "reply_comment"] {
        let op = if kind == "comment" {
            r#"{"kind":"comment","paragraph":{"index":0},"find":"thirty","comment":"Why thirty?"}"#
        } else {
            r#"{"kind":"reply_comment","comment_id":0,"comment":"Why thirty?"}"#
        };
        let (parsed, notes) = EditPlan::from_json_with_notes(&plan(op)).unwrap();
        assert_eq!(
            notes,
            [format!(
                r#"op-1 ({kind}): field "comment" is read as "text""#
            )],
            "{kind}"
        );
        let canonical = serde_json::to_value(&parsed.operations[0]).unwrap();
        assert_eq!(canonical["text"], "Why thirty?", "{kind}");
        assert!(canonical.get("comment").is_none(), "{kind}");
    }
    let result = apply_plan_json(
        &source(),
        &plan(
            r#"{"kind":"comment","paragraph":{"index":0},"find":"thirty","comment":"Why thirty?"}"#,
        ),
    )
    .unwrap();
    assert_eq!(
        result.report.notes,
        [r#"op-1 (comment): field "comment" is read as "text""#]
    );
    let summary = result
        .report
        .to_jsonl()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|event| event["ev"] == "summary")
        .expect("summary event");
    assert_eq!(
        summary["notes"],
        serde_json::json!([r#"op-1 (comment): field "comment" is read as "text""#])
    );
    let comments = part_string(&result.redline, "word/comments.xml").expect("word/comments.xml");
    assert!(comments.contains("Why thirty?"), "{comments}");
}

#[test]
fn alias_text_given_with_text_stays_an_error() {
    let json = plan(
        r#"{"kind":"comment","paragraph":{"index":0},"find":"thirty","text":"One","comment":"Two"}"#,
    );
    assert_eq!(
        error(&json),
        r#"INVALID_PLAN (op-1): comment: "comment" is read as "text", which the operation gives too: drop one"#
    );
}

#[test]
fn alias_anchor_applies_with_a_note() {
    for (kind, op) in [
        (
            "replace",
            r#"{"kind":"replace","paragraph":{"index":0},"anchor":"thirty","replacement":"forty"}"#,
        ),
        (
            "delete",
            r#"{"kind":"delete","paragraph":{"index":0},"anchor":"thirty "}"#,
        ),
        (
            "comment",
            r#"{"kind":"comment","paragraph":{"index":0},"anchor":"thirty","text":"Why?"}"#,
        ),
    ] {
        let result = apply_plan_json(&source(), &plan(op)).unwrap();
        assert_eq!(
            result.report.notes,
            [format!(
                r#"op-1 ({kind}): field "anchor" is read as "find""#
            )],
            "{kind}"
        );
        assert!(result.report.ok, "{kind}");
        assert_word_valid_package(&result.redline);
    }
}

#[test]
fn anchor_is_no_alias_on_other_kinds() {
    let json = plan(r#"{"kind":"insert","paragraph":{"index":0},"anchor":"due","text":" soon"}"#);
    let message = error(&json);
    assert!(
        message.starts_with(r#"INVALID_PLAN (op-1): insert: unknown field "anchor"; "#),
        "{message}"
    );
}

fn jubarte(args: &[&str], dir: &std::path::Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run jubarte")
}

#[test]
fn the_cli_prints_the_teaching_error_and_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), source()).unwrap();
    std::fs::write(
        dir.path().join("plan.json"),
        plan(
            r#"{"kind":"comment","paragraph":{"index":0},"text":"fine"},
                {"kind":"delete","paragraph":{"index":0},"find":"thirty","ocurrence":1}"#,
        ),
    )
    .unwrap();
    let out = jubarte(
        &["edit", "in.docx", "--plan", "plan.json", "-o", "out.docx"],
        dir.path(),
    );
    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(r#"error: INVALID_PLAN (op-2): delete: unknown field "ocurrence"; did you mean "occurrence"? fields: comment, find, occurrence, paragraph"#),
        "{stderr}"
    );
    assert!(!dir.path().join("out.docx").exists());
}

#[test]
fn the_cli_notes_an_alias_on_stderr_and_in_the_summary() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), source()).unwrap();
    std::fs::write(
        dir.path().join("plan.json"),
        plan(
            r#"{"kind":"replace","paragraph":{"index":0},"anchor":"thirty","replacement":"forty"}"#,
        ),
    )
    .unwrap();
    let out = jubarte(
        &["edit", "in.docx", "--plan", "plan.json", "-o", "out.docx"],
        dir.path(),
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(
        stderr.contains(r#"note: op-1 (replace): field "anchor" is read as "find""#),
        "{stderr}"
    );
    let dry = jubarte(
        &["edit", "in.docx", "--plan", "plan.json", "--dry-run"],
        dir.path(),
    );
    let stdout = String::from_utf8_lossy(&dry.stdout);
    let summary = stdout
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|event| event["ev"] == "summary")
        .expect("summary event");
    assert_eq!(
        summary["notes"],
        serde_json::json!([r#"op-1 (replace): field "anchor" is read as "find""#])
    );
}

/// `edit apply` reads a plan that is no JSON object as the first-version
/// parser does: the plan is one object, not an array, a string or a
/// number, rather than an object missing its `schema_version`.
#[test]
fn edit_apply_says_a_plan_is_one_object() {
    let dir = tempfile::tempdir().unwrap();
    let (input, plan_file) = (dir.path().join("in.docx"), dir.path().join("plan.json"));
    std::fs::write(&input, source()).unwrap();
    for (json, what) in [
        ("[]", "an array"),
        (r#""plan""#, "a string"),
        ("3", "a number"),
    ] {
        std::fs::write(&plan_file, json).unwrap();
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["edit", "apply"])
            .arg(&input)
            .arg("--plan")
            .arg(&plan_file)
            .arg("--dry-run")
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(&format!(
                "the plan must be exactly one JSON object, not {what}"
            )),
            "{json}: {stderr}"
        );
        assert!(
            !stderr.contains("schema_version is required"),
            "{json}: {stderr}"
        );
    }
}

/// A plan that settles revisions names paragraphs as read: a deletion's
/// comment lands on the paragraph the deletion does, though p1 vanishes
/// when the plan accepts.
#[test]
fn a_deletions_comment_follows_the_paragraph_as_read() {
    let rev = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>Zero.</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:del w:id="2" {rev}/></w:rPr></w:pPr><w:del w:id="3" {rev}><w:r><w:delText>One gone.</w:delText></w:r></w:del></w:p><w:p><w:r><w:t>Alpha delta.</w:t></w:r></w:p><w:p><w:r><w:t>Gamma delta.</w:t></w:r></w:p><w:p><w:r><w:t>Five.</w:t></w:r></w:p>"#
    ));
    let texts = |bytes: &[u8]| {
        jubarte::inspect::paragraphs(bytes)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>()
    };
    let accept = |op: &str| {
        format!(
            r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","existing_revisions":"accept","operations":[{op}]}}"#
        )
    };
    for (op, clean, covered) in [
        (
            r#"{"kind":"delete","paragraph":"p3","find":"delta","comment":"why"}"#,
            vec!["Zero.", "Alpha delta.", "Gamma .", "Five."],
            "delta",
        ),
        (
            r#"{"kind":"delete_paragraph","paragraph":"p3","comment":"why"}"#,
            vec!["Zero.", "Alpha delta.", "Five."],
            "Gamma delta.",
        ),
    ] {
        let result = apply_plan_json(&source, &accept(op)).unwrap_or_else(|e| panic!("{op}: {e}"));
        assert_eq!(texts(&result.clean), clean, "{op}");
        let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
        assert_eq!(wrapped(&body, "0"), covered, "{op}: {body}");
        // The paragraph that holds the range is the "Gamma" one.
        let start = body.find(r#"<w:commentRangeStart w:id="0""#).unwrap();
        let open = body[..start].rfind("<w:p>").unwrap();
        let close = start + body[start..].find("</w:p>").unwrap();
        assert!(!body[open..start].contains("</w:p>"), "{op}: {body}");
        assert!(body[open..close].contains("Gamma"), "{op}: {body}");
        assert_word_valid_package(&result.redline);
    }
}

/// When the comparer deletes identical words elsewhere, the refusal says
/// so, and gives the route that writes the deletion
/// directly: `keep`, under which the comment wraps the words named.
#[test]
fn a_deletions_comment_on_repeated_words_gives_the_direct_route() {
    let source =
        docx(r#"<w:p><w:r><w:t xml:space="preserve">net net thirty days net</w:t></w:r></w:p>"#);
    let op = r#"{"kind":"delete","paragraph":"p0","find":"net ","occurrence":2,"comment":"why"}"#;
    let compared = plan(op);
    let e = apply_plan_json(&source, &compared).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(
        e.message,
        r#"the comparer deleted the same words elsewhere instead, so the comment would sit on text that stays; pass "existing_revisions": "keep" to write the deletion as given, or delete without a comment"#
    );
    // A flags-built plan gets the flag.
    let options = jubarte::edit::PrepareOptions {
        syntax: jubarte::edit::PlanSyntax::Flags,
        ..jubarte::edit::PrepareOptions::default()
    };
    let e = jubarte::edit::prepare(&source, &EditPlan::from_json(&compared).unwrap(), &options)
        .unwrap_err();
    assert!(
        e.message
            .ends_with("pass --existing-revisions keep to write the deletion as given, or delete without a comment"),
        "{e}"
    );
    let kept = compared.replacen('{', r#"{"existing_revisions":"keep","#, 1);
    let result = apply_plan_json(&source, &kept).unwrap();
    let body = part_string(&result.redline, "word/document.xml").expect("word/document.xml");
    assert_eq!(wrapped(&body, "0"), "net ", "{body}");
    let start = body.find(r#"<w:commentRangeStart w:id="0""#).unwrap();
    assert!(
        body[..start].contains(">net <"),
        "the first net stays before the range: {body}"
    );
    assert_word_valid_package(&result.redline);
}
