// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `strike` and `restore` (plan 2): rejecting another author's change
//! visibly, as a tracked change of our own that Word can show and undo.
mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{
    Change, ChangeFilter, ChangeKind, accept_changes, list_changes, reject_changes,
};
use jubarte::edit::{EditError, EditPlan, EditResult, RevisionSplit, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::process::Command;

const DATE: &str = "2026-10-18T10:00:00Z";
const OLD_DATE: &str = "2026-10-17T09:00:00Z";
const CAROL: &str = "Carol Diaz";
const ALMA: &str = "Alma Sato Reyes";
const STEP1: &[u8] = include_bytes!("fixtures/re-dialect/step1.docx");
const STEP2: &[u8] = include_bytes!("fixtures/re-dialect/step2-tracked.docx");
const ANNEX: &[u8] = include_bytes!("fixtures/re-dialect/annex.docx");

fn plan_with(request: &Value) -> EditPlan {
    let mut base = json!({"schema_version":1,"author":CAROL,"date":DATE,
        "existing_revisions":"keep","operations":[]});
    for (key, value) in request.as_object().unwrap() {
        base[key] = value.clone();
    }
    EditPlan::from_json(&base.to_string()).unwrap()
}
fn plan(operations: &Value) -> EditPlan {
    plan_with(&json!({ "operations": operations }))
}
fn strike(revision: &str) -> Value {
    json!([{"kind":"strike","revision":revision}])
}
fn restore(revision: &str) -> Value {
    json!([{"kind":"restore","revision":revision}])
}

fn validate_output(bytes: &[u8]) {
    assert_word_valid_package(bytes);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.docx");
    std::fs::write(&path, bytes).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "validate must pass: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
fn applied(source: &[u8], plan: &EditPlan) -> EditResult {
    let out = apply_plan(source, plan);
    assert!(out.is_ok(), "edit must succeed: {:?}", out.err());
    let out = out.unwrap();
    validate_output(&out.redline);
    validate_output(&out.clean);
    out
}
fn refused(source: &[u8], plan: &EditPlan) -> EditError {
    match apply_plan(source, plan) {
        Ok(_) => panic!("the plan must be refused"),
        Err(e) => e,
    }
}
fn doc_body(bytes: &[u8]) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&xml);
    let body = dom.descendants(doc, Some(&W::name("body")))[0];
    (dom, body)
}
fn element(dom: &Dom, root: NodeId, kind: &str, id: &str) -> NodeId {
    dom.descendants(root, Some(&W::name(kind)))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some(id))
        .unwrap_or_else(|| panic!("missing {kind}:{id}: {}", dom.serialize_element(root)))
}
fn stamped(dom: &Dom, node: NodeId, author: &str, date: &str) {
    assert_eq!(dom.attribute(node, &W::author()), Some(author));
    assert_eq!(dom.attribute(node, &W::date()), Some(date));
}
fn text(bytes: &[u8]) -> String {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}
fn all() -> ChangeFilter {
    ChangeFilter::default()
}
fn accepted(bytes: &[u8]) -> String {
    text(&accept_changes(bytes, &all()).unwrap())
}
fn rejected(bytes: &[u8]) -> String {
    text(&reject_changes(bytes, &all()).unwrap())
}
fn ids(id: &str) -> ChangeFilter {
    ChangeFilter::ids([id])
}

// ------------------------------------------------------------ strike

#[test]
fn strike_nests_our_deletion_inside_their_insertion() {
    let out = applied(STEP1, &plan(&strike("r1")));
    let (dom, body) = doc_body(&out.redline);
    let theirs = element(&dom, body, "ins", "1");
    stamped(&dom, theirs, ALMA, "2026-10-17T23:44:00Z");
    let children = dom.elements(theirs, None);
    assert_eq!(children.len(), 1, "{}", dom.serialize_element(theirs));
    let ours = children[0];
    assert!(
        dom.name_is(ours, &W::del()),
        "{}",
        dom.serialize_element(theirs)
    );
    stamped(&dom, ours, CAROL, DATE);
    assert_eq!(dom.attribute(ours, &W::id()), Some("2"));
    assert!(dom.descendants(theirs, Some(&W::t())).is_empty());
    assert_eq!(
        dom.value(ours),
        "The parties agree to pay the bankers a pro-rata amount by the end of the month."
    );
    // Their paragraph mark is not part of r1 and stays as it was.
    let mark = element(&dom, body, "ins", "0");
    stamped(&dom, mark, ALMA, "2026-10-17T23:44:00Z");
    assert!(dom.descendants(body, Some(&W::del())).len() == 1);
    // Every author's reading is the right one.
    assert_eq!(
        accepted(&out.redline),
        text(&accept_changes(&reject_changes(STEP1, &ids("r1")).unwrap(), &all()).unwrap())
    );
    assert_eq!(rejected(&out.redline), rejected(STEP1));
}

#[test]
fn rejecting_our_strike_gives_their_insertion_back_intact() {
    let out = applied(STEP1, &plan(&strike("r1")));
    let ours = ChangeFilter {
        authors: Some(vec![CAROL.into()]),
        ..ChangeFilter::default()
    };
    let undone = reject_changes(&out.redline, &ours).unwrap();
    assert_eq!(theirs(&undone), theirs(STEP1));
    assert_eq!(text(&undone), text(STEP1));
}

#[test]
fn accepting_only_their_insertion_leaves_our_deletion_pending() {
    let out = applied(STEP1, &plan(&strike("r1")));
    let alma = ChangeFilter {
        authors: Some(vec![ALMA.into()]),
        ..ChangeFilter::default()
    };
    let only = accept_changes(&out.redline, &alma).unwrap();
    let left = list_changes(&only).unwrap();
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].kind, ChangeKind::Deletion);
    assert_eq!(left[0].author.as_deref(), Some(CAROL));
    assert_eq!(
        left[0].text,
        "The parties agree to pay the bankers a pro-rata amount by the end of the month."
    );
}

#[test]
fn strike_with_a_comment_anchors_it_on_the_struck_words() {
    let out = applied(
        STEP1,
        &plan(&json!([{"kind":"strike","revision":"r1","comment":"Not acceptable."}])),
    );
    let (dom, body) = doc_body(&out.redline);
    let theirs = element(&dom, body, "ins", "1");
    let start = dom.descendants(body, Some(&W::name("commentRangeStart")));
    let end = dom.descendants(body, Some(&W::name("commentRangeEnd")));
    assert_eq!((start.len(), end.len()), (1, 1));
    // The range holds the struck words: it opens before them and closes
    // after them.
    let p = dom.parent(theirs).unwrap();
    let order: Vec<NodeId> = dom.descendants(p, None);
    let at = |n: NodeId| order.iter().position(|&m| m == n).unwrap();
    let del = dom.descendants(theirs, Some(&W::del()))[0];
    assert!(at(start[0]) < at(del) && at(del) < at(end[0]));
    let comments = part_string(&out.redline, "word/comments.xml").unwrap();
    assert!(comments.contains("Not acceptable."), "{comments}");
    // The clean copy drops the struck words, and the comment on them.
    assert_eq!(part_string(&out.clean, "word/comments.xml"), None);
    assert!(!text(&out.clean).contains("Not acceptable."));
    assert_eq!(out.report.operations[0].comment_id, Some(0));
}

#[test]
fn strike_inside_a_split_insertion_struck_only_where_it_is_still_visible() {
    // r3 (AR) holds ASR's deletions r4 and r5: our deletion covers only
    // the words of r3 still visible.
    let out = applied(ANNEX, &plan(&strike("r3")));
    let (dom, body) = doc_body(&out.redline);
    let ours: Vec<NodeId> = dom
        .descendants(body, Some(&W::del()))
        .into_iter()
        .filter(|&d| dom.attribute(d, &W::author()) == Some(CAROL))
        .collect();
    assert!(!ours.is_empty());
    for del in &ours {
        let parent = dom.parent(*del).unwrap();
        assert!(
            dom.name_is(parent, &W::ins()),
            "{}",
            dom.serialize_element(parent)
        );
        assert!(dom.descendants(*del, Some(&W::del())).is_empty());
    }
    let struck: String = ours.iter().map(|&d| dom.value(d)).collect();
    assert_eq!(struck, "will pay the ");
    let before = accept_changes(&reject_changes(ANNEX, &ids("r3")).unwrap(), &all()).unwrap();
    assert_eq!(accepted(&out.redline), text(&before));
}

#[test]
fn a_struck_insertion_is_ok_with_a_note() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:del w:id="7" w:author="{CAROL}" w:date="{OLD_DATE}"><w:r><w:delText>alpha</w:delText></w:r></w:del></w:ins><w:r><w:t>]</w:t></w:r></w:p>"#
    ));
    let out = applied(&source, &plan(&strike("r6")));
    let outcome = &out.report.operations[0];
    assert_eq!(outcome.status, "ok");
    assert_eq!(outcome.message.as_deref(), Some("already struck"));
    let (dom, body) = doc_body(&out.redline);
    assert_eq!(dom.descendants(body, Some(&W::del())).len(), 1);
}

#[test]
fn strike_of_our_own_insertion_is_refused() {
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="{CAROL}" w:date="{OLD_DATE}"><w:r><w:t>alpha</w:t></w:r></w:ins></w:p>"#
    ));
    let e = refused(&source, &plan(&strike("r6")));
    assert_eq!(e.code, "OWN_REVISION");
    assert_eq!(
        e.message,
        r#"r6 is by "Carol Diaz"; delete the text instead"#
    );
    let e = refused(&del_source(CAROL), &plan(&restore("r12")));
    assert_eq!(e.code, "OWN_REVISION");
    assert_eq!(e.message, r#"r12 is by "Carol Diaz"; reject it instead"#);
}

#[test]
fn strike_and_restore_take_the_kind_they_name() {
    let e = refused(&del_source("Bob Lee"), &plan(&strike("r12")));
    assert_eq!(e.code, "INVALID_EDIT");
    assert_eq!(
        e.message,
        "r12 is a deletion, not an insertion: restore it instead"
    );
    let e = refused(STEP1, &plan(&restore("r1")));
    assert_eq!(e.code, "INVALID_EDIT");
    assert_eq!(
        e.message,
        "r1 is an insertion, not a deletion: strike it instead"
    );
    let e = refused(STEP1, &plan(&strike("r99")));
    assert_eq!(e.code, "UNKNOWN_CHANGE");
    assert_eq!(e.message, "no tracked change r99");
    let e = refused(STEP2, &plan(&strike("r4+5")));
    assert_eq!(e.code, "INVALID_EDIT");
    assert_eq!(
        e.message,
        "strike takes one revision; r4+5 names 2 (r4, r5)"
    );
}

#[test]
fn a_paragraph_mark_is_refused_with_the_route_that_takes_it() {
    let e = refused(STEP1, &plan(&strike("r0")));
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(
        e.message,
        "r0 is an inserted paragraph mark; strike takes inserted text only: delete paragraph p0 to strike it with its mark"
    );
    let deleted_mark = docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="9" w:author="Bob Lee" w:date="{OLD_DATE}"/></w:rPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p><w:p><w:r><w:t>two</w:t></w:r></w:p>"#
    ));
    let e = refused(&deleted_mark, &plan(&restore("r9")));
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(
        e.message,
        "r9 is a deleted paragraph mark; restore takes deleted text only: reject r9 to put the paragraph break back"
    );
}

#[test]
fn restoring_a_move_source_is_refused() {
    let moved = docx(&format!(
        r#"<w:p><w:moveFromRangeStart w:id="1" w:name="m1"/><w:moveFrom w:id="7" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:t>moved</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="1"/><w:r><w:t> stays</w:t></w:r></w:p><w:p><w:r><w:t>end </w:t></w:r><w:moveToRangeStart w:id="2" w:name="m1"/><w:moveTo w:id="8" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:t>moved</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="2"/></w:p>"#
    ));
    let e = refused(&moved, &plan(&restore("r7")));
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(
        e.message,
        "r7 is the source of a move: restoring it would duplicate the moved words; reject the move (r7) instead"
    );
}

#[test]
fn an_empty_deletion_names_the_comment_it_holds() {
    let body = format!(
        r#"<w:p><w:commentRangeStart w:id="4"/><w:r><w:t>words</w:t></w:r><w:commentRangeEnd w:id="4"/><w:del w:id="13" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:commentReference w:id="4"/></w:r></w:del></w:p>"#
    );
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:comment w:id="4" w:author="Ann" w:date="2026-10-01T00:00:00Z" w:initials="A"><w:p><w:r><w:t>Why?</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let rel = format!("{R_NS}/comments");
    let source = docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: &rel,
            xml: &comments,
        }],
    );
    let e = refused(&source, &plan(&restore("r13")));
    assert_eq!(e.code, "EMPTY_REVISION");
    assert_eq!(
        e.message,
        "r13 holds no text, only comment c4's mark; reject r13 to keep c4"
    );
}

#[test]
fn strike_needs_kept_revisions() {
    for mode in ["accept", "reject"] {
        let e = refused(
            STEP1,
            &plan_with(&json!({"existing_revisions":mode,"operations":strike("r1")})),
        );
        assert_eq!(e.code, "INVALID_PLAN", "{mode}");
        assert_eq!(
            e.message,
            "strike needs existing_revisions keep (the revision would already be settled)"
        );
    }
}

#[test]
fn a_revision_settled_and_struck_is_a_conflict() {
    let e = refused(
        STEP2,
        &plan_with(
            &json!({"resolve_revisions":{"reject":{"ids":["r5"]}},"operations":strike("r5")}),
        ),
    );
    assert_eq!(e.code, "REVISION_CONFLICT");
    assert_eq!(
        e.message,
        "r5 is both settled by resolve_revisions and struck by op-1"
    );
}

// ------------------------------------------------------------ restore

fn del_source(author: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r><w:del w:id="12" w:author="{author}" w:date="{OLD_DATE}"><w:r><w:rPr><w:b/></w:rPr><w:delText>use, store, copy</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> end</w:t></w:r></w:p>"#
    ))
}

#[test]
fn restore_copies_their_deleted_runs_as_our_insertion_after_it() {
    let source = del_source("Bob Lee");
    let out = applied(&source, &plan(&restore("r12")));
    let (src_dom, src_body) = doc_body(&source);
    let (dom, body) = doc_body(&out.redline);
    let theirs = element(&dom, body, "del", "12");
    // Their deletion is untouched.
    assert_eq!(
        dom.serialize_element(theirs),
        src_dom.serialize_element(element(&src_dom, src_body, "del", "12"))
    );
    let ours = dom.next_element(theirs).unwrap();
    assert!(
        dom.name_is(ours, &W::ins()),
        "{}",
        dom.serialize_element(body)
    );
    stamped(&dom, ours, CAROL, DATE);
    assert_eq!(dom.attribute(ours, &W::id()), Some("13"));
    let runs = dom.elements(ours, Some(&W::r()));
    assert_eq!(runs.len(), 1);
    let rpr = dom
        .element(runs[0], &W::r_pr())
        .expect("the deleted run's rPr");
    let kept: Vec<String> = dom
        .elements(rpr, None)
        .into_iter()
        .map(|n| dom.name(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(kept, ["b"]);
    assert_eq!(dom.value(ours), "use, store, copy");
    assert!(dom.descendants(ours, Some(&W::del_text())).is_empty());
    assert_eq!(accepted(&out.redline), "keep use, store, copy end");
    assert_eq!(rejected(&out.redline), "keep use, store, copy end");
    // The clean copy: their deletion still pending, our words plain.
    let (clean, clean_body) = doc_body(&out.clean);
    let theirs = element(&clean, clean_body, "del", "12");
    let next = clean.next_element(theirs).unwrap();
    assert!(
        clean.name_is(next, &W::r()),
        "{}",
        clean.serialize_element(clean_body)
    );
    assert_eq!(clean.value(next), "use, store, copy");
    assert_eq!(
        out.report.operations[0].context.as_deref(),
        Some(r#"{+restored "use, store, copy"}"#)
    );
}

#[test]
fn restore_inside_their_insertion_lands_beside_it() {
    // r4 (B) deleted "bankers" inside ASR's r3: our words cannot sit
    // inside ASR's insertion, so they follow it.
    let out = applied(STEP2, &plan(&restore("r4")));
    let (dom, body) = doc_body(&out.redline);
    let del = element(&dom, body, "del", "4");
    let host = dom.parent(del).unwrap();
    assert_eq!(dom.attribute(host, &W::id()), Some("3"));
    let ours = dom.next_element(host).unwrap();
    assert!(dom.name_is(ours, &W::ins()));
    stamped(&dom, ours, CAROL, DATE);
    assert_eq!(dom.value(ours), "bankers");
    assert!(dom.name_is(dom.parent(ours).unwrap(), &W::p()));
    let before = accept_changes(&reject_changes(STEP2, &ids("r4")).unwrap(), &all()).unwrap();
    assert_eq!(accepted(&out.redline), text(&before));
}

#[test]
fn restore_with_a_comment_anchors_it_on_our_words() {
    let out = applied(
        &del_source("Bob Lee"),
        &plan(&json!([{"kind":"restore","revision":"r12","comment":"Agreed in round 1."}])),
    );
    let (dom, body) = doc_body(&out.redline);
    let ours = element(&dom, body, "ins", "13");
    let start = dom.descendants(body, Some(&W::name("commentRangeStart")))[0];
    let end = dom.descendants(body, Some(&W::name("commentRangeEnd")))[0];
    let order: Vec<NodeId> = dom.descendants(body, None);
    let at = |n: NodeId| order.iter().position(|&m| m == n).unwrap();
    let words = dom.descendants(ours, Some(&W::t()))[0];
    assert!(at(start) < at(words) && at(words) < at(end));
    let comments = part_string(&out.clean, "word/comments.xml").unwrap();
    assert!(comments.contains("Agreed in round 1."), "{comments}");
}

// ------------------------------------------------------- reject visible

#[test]
fn reject_visible_picks_by_kind() {
    let out = applied(
        STEP2,
        &plan_with(&json!({"resolve_revisions":{"reject":{"ids":["r4+5"],"visible":true}}})),
    );
    assert!(
        out.report.resolved_revisions.is_empty(),
        "{:?}",
        out.report.resolved_revisions
    );
    let kinds: Vec<&str> = out
        .report
        .operations
        .iter()
        .map(|o| o.kind.as_str())
        .collect();
    assert_eq!(kinds, ["restore", "strike"]);
    let (dom, body) = doc_body(&out.redline);
    let insurers = element(&dom, body, "ins", "5");
    let struck = dom.elements(insurers, None);
    assert_eq!(struck.len(), 1);
    assert!(dom.name_is(struck[0], &W::del()));
    stamped(&dom, struck[0], CAROL, DATE);
    let restored: Vec<NodeId> = dom
        .descendants(body, Some(&W::ins()))
        .into_iter()
        .filter(|&n| dom.attribute(n, &W::author()) == Some(CAROL))
        .collect();
    assert_eq!(restored.len(), 1);
    assert_eq!(dom.value(restored[0]), "bankers");
    let settled = reject_changes(STEP2, &ChangeFilter::ids(["r4", "r5"])).unwrap();
    assert_eq!(accepted(&out.redline), accepted(&settled));
}

#[test]
fn reject_visible_is_only_for_reject() {
    let e = EditPlan::from_json(
        &json!({"schema_version":1,"author":CAROL,"existing_revisions":"keep",
            "resolve_revisions":{"accept":{"ids":["r1"],"visible":true}},"operations":[]})
        .to_string(),
    )
    .unwrap_err();
    assert!(e.message.contains("visible"), "{}", e.message);
}

// ---------------------------------------------------------- properties

/// The other authors' changes as `changes` lists them: what a strike or a
/// restore must leave as it found it.
type Theirs = BTreeMap<String, (ChangeKind, Option<String>, Option<String>, String)>;

fn theirs(bytes: &[u8]) -> Theirs {
    joined(bytes, &[])
}

/// The other authors' changes of `bytes` by source id: each piece the
/// redline split an insertion into (`splits`, from the report) joins the
/// insertion it was, its text appended in document order.
fn joined(bytes: &[u8], splits: &[RevisionSplit]) -> Theirs {
    let origin = |id: &str| {
        splits
            .iter()
            .find(|s| s.to.iter().any(|t| t == id))
            .map_or_else(|| id.to_string(), |s| s.from.clone())
    };
    let mut out = Theirs::new();
    for c in list_changes(bytes).unwrap() {
        if c.author.as_deref() == Some(CAROL) {
            continue;
        }
        let c: Change = c;
        out.entry(origin(&c.id))
            .and_modify(|(.., text)| text.push_str(&c.text))
            .or_insert((c.kind, c.author, c.date, c.text));
    }
    out
}

fn property_family(name: &str, source: &[u8]) {
    let changes = list_changes(source).unwrap();
    let mut checked = 0;
    for change in changes.iter().filter(|c| c.target == "text") {
        let (op, settle) = match change.kind {
            ChangeKind::Insertion => (strike(&change.id), ids(&change.id)),
            ChangeKind::Deletion => (restore(&change.id), ids(&change.id)),
            _ => continue,
        };
        let out = applied(source, &plan(&op));
        let ours = ChangeFilter {
            authors: Some(vec![CAROL.into()]),
            ..ChangeFilter::default()
        };
        // (1) Rejecting our change gives their changes back as they were.
        let undone = reject_changes(&out.redline, &ours).unwrap();
        assert_eq!(
            joined(&undone, &out.report.split_revisions),
            theirs(source),
            "{name} {}",
            change.id
        );
        assert_eq!(text(&undone), text(source), "{name} {}", change.id);
        // (2), (3) Accept All reads as Word's Reject would have left it.
        let settled = reject_changes(source, &settle).unwrap();
        assert_eq!(
            accepted(&out.redline),
            accepted(&settled),
            "{name} {}",
            change.id
        );
        checked += 1;
    }
    assert!(checked >= 1, "{name}: no text change to strike or restore");
}

#[test]
fn step1_strikes_and_restores_read_as_rejects() {
    property_family("step1", STEP1);
}

#[test]
fn step2_strikes_and_restores_read_as_rejects() {
    property_family("step2", STEP2);
}

#[test]
fn annex_strikes_and_restores_read_as_rejects() {
    property_family("annex", ANNEX);
}

/// A `-p AT --reject` group, `--visible` or not.
fn reject_group(at: &str, visible: bool) -> jubarte::edit::flags::FlagOp {
    jubarte::edit::flags::FlagOp {
        at: at.into(),
        reject: true,
        visible,
        ..jubarte::edit::flags::FlagOp::default()
    }
}

#[test]
fn reject_visible_flags_pick_by_kind_per_group() {
    use jubarte::edit::OperationKind;
    use jubarte::edit::flags::{Verb, plan_from_flags};
    let built = plan_from_flags(
        Verb::Edit,
        &[reject_group("r4", true), reject_group("r5", true)],
        CAROL,
        Some(DATE),
        None,
        STEP2,
    )
    .unwrap()
    .plan;
    assert!(built.resolve_revisions.is_none(), "{built:?}");
    let kinds: Vec<OperationKind> = built.operations.iter().map(|o| o.kind.clone()).collect();
    assert_eq!(
        kinds,
        [
            OperationKind::Restore {
                revision: "r4".into(),
                comment: None
            },
            OperationKind::Strike {
                revision: "r5".into(),
                comment: None
            },
        ]
    );
    // A silent reject beside a visible one stays a silent reject.
    let mixed = plan_from_flags(
        Verb::Edit,
        &[reject_group("r4", true), reject_group("r5", false)],
        CAROL,
        Some(DATE),
        None,
        STEP2,
    )
    .unwrap()
    .plan;
    assert_eq!(mixed.operations.len(), 1);
    assert_eq!(
        mixed.resolve_revisions.as_ref().unwrap().reject,
        Some(ChangeFilter::ids(["r5"]))
    );
    let out = applied(STEP2, &mixed);
    let settled = reject_changes(STEP2, &ChangeFilter::ids(["r4", "r5"])).unwrap();
    assert_eq!(accepted(&out.redline), accepted(&settled));
}

fn cli(dir: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let says = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{args:?}: {says}");
    says
}

#[test]
fn the_cli_strikes_restores_and_rejects_visibly() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), STEP2).unwrap();
    let settled = reject_changes(STEP2, &ChangeFilter::ids(["r4", "r5"])).unwrap();
    for (out, groups) in [
        (
            "flags.docx",
            &[
                "-p",
                "r5",
                "--strike",
                "--content",
                "Not agreed.",
                "-p",
                "r4",
                "--restore",
            ][..],
        ),
        ("visible.docx", &["-p", "r4+5", "--reject", "--visible"][..]),
    ] {
        let mut args = vec!["edit", "in.docx"];
        args.extend_from_slice(groups);
        args.extend_from_slice(&["-a", CAROL, "-d", DATE, "-o", out]);
        cli(dir.path(), &args);
        let bytes = std::fs::read(dir.path().join(out)).unwrap();
        validate_output(&bytes);
        assert_eq!(accepted(&bytes), accepted(&settled), "{out}");
        // Their changes stay, beside ours.
        let theirs: Vec<String> = list_changes(&bytes)
            .unwrap()
            .into_iter()
            .filter(|c| c.author.as_deref() != Some(CAROL))
            .map(|c| c.id)
            .collect();
        for id in ["r4", "r5"] {
            assert!(theirs.iter().any(|t| t == id), "{out}: {theirs:?}");
        }
    }
}

#[test]
fn a_strike_listed_before_the_restore_beside_it_composes() {
    // r4 ("bankers", deleted) sits right before r5 ("insurers", inserted).
    let out = applied(
        STEP2,
        &plan(&json!([
            {"kind":"strike","revision":"r5","comment":"Not agreed."},
            {"kind":"restore","revision":"r4"}
        ])),
    );
    let settled = reject_changes(STEP2, &ChangeFilter::ids(["r4", "r5"])).unwrap();
    assert_eq!(accepted(&out.redline), accepted(&settled));
}

#[test]
fn a_restore_inside_another_edit_is_refused_not_composed() {
    let e = apply_plan(
        STEP2,
        &plan(&json!([
            {"kind":"restore","revision":"r4"},
            {"kind":"replace","paragraph":"p0","find":"the insurers","replacement":"the brokers"}
        ])),
    )
    .unwrap_err();
    assert!(
        ["OVERLAPPING_EDITS", "UNSUPPORTED_STRUCTURE"].contains(&e.code.as_str()),
        "{e}"
    );
}

/// CodeRabbit on #1286: a deletion in an insertion is in a paragraph only
/// when that insertion is.
#[test]
fn restore_refuses_a_deletion_whose_insertion_sits_in_a_container() {
    for (open, close, inside) in [
        (
            r##"<w:hyperlink w:anchor="x">"##,
            "</w:hyperlink>",
            "a hyperlink",
        ),
        (
            "<w:sdt><w:sdtContent>",
            "</w:sdtContent></w:sdt>",
            "a content control",
        ),
    ] {
        let source = docx(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>{open}<w:ins w:id="6" w:author="Bob Lee" w:date="{OLD_DATE}"><w:del w:id="7" w:author="{ALMA}" w:date="{OLD_DATE}"><w:r><w:delText>alpha</w:delText></w:r></w:del></w:ins>{close}</w:p>"#
        ));
        let e = refused(&source, &plan(&restore("r7")));
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{inside}");
        assert_eq!(
            e.message,
            format!(
                "r7 sits inside {inside}; restore takes deleted text in a paragraph only: reject r7 instead"
            )
        );
    }
}

/// Our own revision is named first, whatever it is, with the way to undo
/// it: not a strike sent to restore and refused again there.
#[test]
fn our_own_revision_of_any_kind_is_refused_as_ours() {
    let e = refused(&del_source(CAROL), &plan(&strike("r12")));
    assert_eq!(e.code, "OWN_REVISION");
    assert_eq!(e.message, r#"r12 is by "Carol Diaz"; reject it instead"#);
    let e = refused(&formatted(CAROL), &plan(&strike("r8")));
    assert_eq!(e.code, "OWN_REVISION");
    assert_eq!(e.message, r#"r8 is by "Carol Diaz"; reject it instead"#);
}

fn formatted(author: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="8" w:author="{author}" w:date="{OLD_DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>bold</w:t></w:r></w:p>"#
    ))
}

/// A formatting change has no visible reject: the refusal says the route
/// it offers leaves no trace.
#[test]
fn a_formatting_change_is_refused_with_the_silent_route_named_as_such() {
    for (op, verb, takes) in [
        (strike("r8"), "strike", "inserted"),
        (restore("r8"), "restore", "deleted"),
    ] {
        let e = refused(&formatted("Bob Lee"), &plan(&op));
        assert_eq!(e.code, "INVALID_EDIT");
        assert_eq!(
            e.message,
            format!(
                "r8 is a formatting change; {verb} takes {takes} text: reject r8 to undo it (no visible reject of formatting yet; it leaves no trace)"
            )
        );
    }
}

/// Fields come later (plan 2 ships plain text first): a strike of their
/// inserted field is refused with the route that takes it.
#[test]
fn striking_their_inserted_field_is_refused_for_now() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">page </w:t></w:r><w:ins w:id="6" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:ins></w:p>"#
    ));
    let e = refused(&source, &plan(&strike("r6")));
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(
        e.message,
        "r6 holds a field; strike takes plain inserted text only: reject r6 instead, or comment on it"
    );
}

/// `--reject --visible` of a move's source asks for a restore, which a
/// move does not take: the refusal names the silent route.
#[test]
fn a_visible_reject_of_a_move_source_is_refused_with_the_move_route() {
    let moved = docx(&format!(
        r#"<w:p><w:moveFromRangeStart w:id="1" w:name="m1"/><w:moveFrom w:id="7" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:t>moved</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="1"/><w:r><w:t> stays</w:t></w:r></w:p><w:p><w:r><w:t>end </w:t></w:r><w:moveToRangeStart w:id="2" w:name="m1"/><w:moveTo w:id="8" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:t>moved</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="2"/></w:p>"#
    ));
    let e = refused(
        &moved,
        &plan_with(&json!({
            "resolve_revisions":{"reject":{"ids":["r7"],"visible":true}},
            "operations":[]
        })),
    );
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(
        e.message,
        "r7 is the source of a move: restoring it would duplicate the moved words; reject the move (r7) instead"
    );
}

#[test]
fn restore_needs_kept_revisions() {
    for mode in ["accept", "reject"] {
        let e = refused(
            &del_source("Bob Lee"),
            &plan_with(&json!({"existing_revisions":mode,"operations":restore("r12")})),
        );
        assert_eq!(e.code, "INVALID_PLAN", "{mode}");
        assert_eq!(
            e.message,
            "restore needs existing_revisions keep (the revision would already be settled)"
        );
    }
}

/// A visible reject with no filter is every revision, as a silent one is.
#[test]
fn a_visible_reject_with_no_filter_takes_every_revision() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r><w:ins w:id="6" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:t>alpha</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> and </w:t></w:r><w:del w:id="7" w:author="Bob Lee" w:date="{OLD_DATE}"><w:r><w:delText>beta</w:delText></w:r></w:del></w:p>"#
    ));
    let out = applied(
        &source,
        &plan_with(&json!({"resolve_revisions":{"reject":{"visible":true}},"operations":[]})),
    );
    let kinds: Vec<&str> = out
        .report
        .operations
        .iter()
        .map(|o| o.kind.as_str())
        .collect();
    assert_eq!(kinds, ["strike", "restore"]);
    assert_eq!(accepted(&out.redline), "keep  and beta");
}

/// The agent view of a strike and a restore: our deletion inside their
/// insertion, and our insertion beside their deletion, by our author.
#[test]
fn changes_lists_the_strike_inside_theirs_and_the_restore_beside_theirs() {
    let out = applied(STEP1, &plan(&strike("r1")));
    let listed = list_changes(&out.redline).unwrap();
    let ours: Vec<_> = listed
        .iter()
        .filter(|c| c.author.as_deref() == Some(CAROL))
        .collect();
    assert_eq!(ours.len(), 1, "{listed:?}");
    assert_eq!(ours[0].kind, ChangeKind::Deletion);
    assert_eq!(ours[0].inside.as_deref(), Some("r1"));
    let out = applied(&del_source("Bob Lee"), &plan(&restore("r12")));
    let listed = list_changes(&out.redline).unwrap();
    let ours: Vec<_> = listed
        .iter()
        .filter(|c| c.author.as_deref() == Some(CAROL))
        .collect();
    assert_eq!(ours.len(), 1, "{listed:?}");
    assert_eq!(ours[0].kind, ChangeKind::Insertion);
    assert_eq!(ours[0].inside, None);
    assert_eq!(ours[0].text, "use, store, copy");
}
