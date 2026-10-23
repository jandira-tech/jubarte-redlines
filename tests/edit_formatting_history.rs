// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Another author's formatting history on a run that an edit splits (#1656).
//!
//! Alice made a run bold (`w:rPrChange`, italic before). Bob's edit splits
//! that run. Each retained fragment keeps Alice's record, under an id of
//! its own; the first fragment keeps the original id. Bob's own text never
//! carries Alice's history.
mod common;

use common::docx::{docx, part_string};
use jubarte::changes::{ChangeFilter, accept_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::process::Command;

const OLD: &str = "2026-10-17T09:00:00Z";
const DATE: &str = "2026-10-23T10:00:00Z";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const TEXT: &str = "alpha beta gamma";

/// A run Alice made bold; it was italic before.
fn formatted_run(id: u32, text: &str) -> String {
    format!(
        r#"<w:r><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>"#
    )
}

/// `[` + Alice's formatted run + `]`; the run's formatting record is `w:id` 5.
fn outside(text: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r>{}<w:r><w:t>]</w:t></w:r></w:p>"#,
        formatted_run(5, text)
    ))
}

/// The same run inside Alice's tracked insertion (`w:ins` 6); the run's
/// formatting record is `w:id` 7.
fn inside(text: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:ins w:id="6" w:author="{ALICE}" w:date="{OLD}">{}</w:ins><w:r><w:t>]</w:t></w:r></w:p>"#,
        formatted_run(7, text)
    ))
}

fn plan(operations: Value) -> EditPlan {
    let mut request = json!({"schema_version":1,"author":BOB,"date":DATE,
        "existing_revisions":"keep"});
    request["operations"] = operations;
    EditPlan::from_json(&request.to_string()).unwrap()
}

/// Bob's edit; both outputs must hold unique revision ids.
fn edit(source: &[u8], operations: Value) -> Vec<u8> {
    let out = apply_plan(source, &plan(operations));
    let out = out.unwrap_or_else(|e| panic!("edit must succeed: {e:?}"));
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    alice_records(&out.clean);
    out.redline
}

fn dom(bytes: &[u8]) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    (dom, root)
}

/// Every revision record's `(kind, w:id)` occurs once.
fn assert_unique_ids(bytes: &[u8]) {
    let (dom, root) = dom(bytes);
    let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
    for node in dom.descendants(root, None) {
        let Some(name) = dom.name(node) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name();
        if !matches!(
            local,
            "ins" | "del" | "rPrChange" | "pPrChange" | "moveFrom" | "moveTo"
        ) {
            continue;
        }
        if let Some(id) = dom.attribute(node, &W::id()) {
            *seen.entry((local.to_string(), id.to_string())).or_default() += 1;
        }
    }
    let repeated: Vec<_> = seen.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        repeated.is_empty(),
        "repeated revision ids {repeated:?}: {}",
        dom.serialize_element(root)
    );
}

fn assert_validates(bytes: &[u8]) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.docx");
    std::fs::write(&path, bytes).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!report.contains("DUPLICATE_REVISION_ID"), "{report}");
}

/// One formatting record: its id and the text of the run it sits on.
struct Record {
    id: String,
    text: String,
}

/// Alice's formatting records in document order. Every `w:rPrChange` must
/// be Alice's, with her date and her old properties (italic).
fn alice_records(bytes: &[u8]) -> Vec<Record> {
    let (dom, root) = dom(bytes);
    let mut out = Vec::new();
    for change in dom.descendants(root, Some(&W::name("rPrChange"))) {
        assert_eq!(dom.attribute(change, &W::author()), Some(ALICE));
        assert_eq!(dom.attribute(change, &W::date()), Some(OLD));
        let old: Vec<String> = dom
            .descendants(change, None)
            .into_iter()
            .filter_map(|n| dom.name(n).map(|q| q.local_name().to_string()))
            .collect();
        assert_eq!(
            old,
            ["rPr", "i"],
            "old properties of {:?}",
            dom.attribute(change, &W::id())
        );
        let run = dom
            .ancestors(change, None)
            .into_iter()
            .find(|&a| dom.name_is(a, &W::r()))
            .expect("a run formatting record sits in a run");
        let in_bobs_insertion = dom
            .ancestors(run, None)
            .into_iter()
            .any(|a| dom.name_is(a, &W::ins()) && dom.attribute(a, &W::author()) == Some(BOB));
        assert!(
            !in_bobs_insertion,
            "Bob's inserted text carries Alice's history"
        );
        let text: String = dom
            .descendants(run, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            .map(|n| dom.value(n))
            .collect();
        out.push(Record {
            id: dom.attribute(change, &W::id()).unwrap().to_string(),
            text,
        });
    }
    out
}

/// The text Alice's records sit on, in document order, deleted text
/// included.
fn covered(records: &[Record]) -> String {
    records.iter().map(|r| r.text.as_str()).collect()
}

fn text(bytes: &[u8]) -> String {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn accepted(bytes: &[u8]) -> String {
    text(&accept_changes(bytes, &ChangeFilter::default()).unwrap())
}

fn rejected_bob(bytes: &[u8]) -> Vec<u8> {
    let filter = ChangeFilter {
        authors: Some(vec![BOB.to_string()]),
        ..ChangeFilter::default()
    };
    reject_changes(bytes, &filter).unwrap()
}

/// The common checks for an edit that splits Alice's run: the original
/// record stays on the first fragment, Alice's records cover the text she
/// formatted, the accepted text is `expected`, and rejecting Bob's edits
/// restores her text and her history.
fn check_split(source: &[u8], original_id: &str, first: &str, operations: Value, expected: &str) {
    let out = edit(source, operations);
    let records = alice_records(&out);
    assert_eq!(covered(&records), TEXT, "Alice's records cover her text");
    assert!(records.len() > 1, "the edit splits Alice's run");
    assert_eq!(
        records[0].id, original_id,
        "the first fragment keeps the id"
    );
    assert_eq!(records[0].text, first);
    assert_eq!(accepted(&out), expected);
    let restored = rejected_bob(&out);
    assert_unique_ids(&restored);
    assert_eq!(text(&restored), format!("[{TEXT}]"));
    let records = alice_records(&restored);
    assert_eq!(
        covered(&records),
        TEXT,
        "rejecting Bob restores Alice's history"
    );
}

fn replace(find: &str, replacement: &str) -> Value {
    json!([{"kind":"replace","paragraph":"p0","find":find,"replacement":replacement}])
}

fn insert(after: &str, text: &str) -> Value {
    json!([{"kind":"insert","paragraph":"p0","after":after,"text":text}])
}

#[test]
fn a_replacement_outside_an_insertion_gives_each_fragment_its_own_id() {
    check_split(
        &outside(TEXT),
        "5",
        "alpha ",
        replace("beta", "new"),
        "[alpha new gamma]",
    );
}

#[test]
fn an_insertion_outside_an_insertion_gives_each_fragment_its_own_id() {
    check_split(
        &outside(TEXT),
        "5",
        "alpha",
        insert("alpha", " new"),
        "[alpha new beta gamma]",
    );
}

#[test]
fn a_replacement_inside_alices_insertion_gives_each_fragment_its_own_id() {
    check_split(
        &inside(TEXT),
        "7",
        "alpha ",
        replace("beta", "new"),
        "[alpha new gamma]",
    );
}

#[test]
fn an_insertion_inside_alices_insertion_gives_each_fragment_its_own_id() {
    check_split(
        &inside(TEXT),
        "7",
        "alpha",
        insert("alpha", " new"),
        "[alpha new beta gamma]",
    );
}

#[test]
fn a_comment_that_splits_the_run_gives_each_fragment_its_own_id() {
    let operations = json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]);
    check_split(
        &outside(TEXT),
        "5",
        "alpha ",
        operations,
        &format!("[{TEXT}]"),
    );
}

#[test]
fn a_replacement_with_a_comment_gives_each_fragment_its_own_id() {
    let operations = json!([{"kind":"replace","paragraph":"p0","find":"beta",
        "replacement":"new","comment":"why"}]);
    check_split(
        &outside(TEXT),
        "5",
        "alpha ",
        operations,
        "[alpha new gamma]",
    );
}

#[test]
fn a_deletion_gives_each_fragment_its_own_id() {
    let operations = json!([{"kind":"delete","paragraph":"p0","find":"beta"}]);
    check_split(&outside(TEXT), "5", "alpha ", operations, "[alpha  gamma]");
}

#[test]
fn several_splits_in_one_transaction_give_every_fragment_its_own_id() {
    let operations = json!([
        {"kind":"insert","paragraph":"p0","after":"alpha","text":" one"},
        {"kind":"replace","paragraph":"p0","find":"gamma","replacement":"delta"},
        {"kind":"comment","paragraph":"p0","find":"beta","text":"note"}
    ]);
    check_split(
        &outside(TEXT),
        "5",
        "alpha",
        operations,
        "[alpha one beta delta]",
    );
}

#[test]
fn several_splits_inside_alices_insertion_give_every_fragment_its_own_id() {
    let operations = json!([
        {"kind":"insert","paragraph":"p0","after":"alpha","text":" one"},
        {"kind":"replace","paragraph":"p0","find":"gamma","replacement":"delta"}
    ]);
    check_split(
        &inside(TEXT),
        "7",
        "alpha",
        operations,
        "[alpha one beta delta]",
    );
}

#[test]
fn a_split_in_non_ascii_text_gives_each_fragment_its_own_id() {
    let text = "álpha βeta gämma";
    let out = edit(&outside(text), replace("βeta", "ñew"));
    let records = alice_records(&out);
    assert_eq!(covered(&records), text);
    assert!(records.len() > 1);
    assert_eq!(records[0].id, "5");
    assert_eq!(records[0].text, "álpha ");
    assert_eq!(accepted(&out), "[álpha ñew gämma]");
    let restored = rejected_bob(&out);
    assert_unique_ids(&restored);
    assert_eq!(covered(&alice_records(&restored)), text);
    assert_eq!(self::text(&restored), format!("[{text}]"));
}

/// An insertion at a run boundary splits nothing: Alice's one record stays
/// as it was, and Bob's text takes none of her history.
fn check_boundary(source: &[u8], original_id: &str, operations: Value, expected: &str) {
    let out = edit(source, operations);
    let records = alice_records(&out);
    assert_eq!(records.len(), 1, "nothing was split");
    assert_eq!(records[0].id, original_id);
    assert_eq!(records[0].text, TEXT);
    assert_eq!(accepted(&out), expected);
}

#[test]
fn an_insertion_before_the_formatted_run_splits_nothing() {
    check_boundary(&outside(TEXT), "5", insert("[", "X"), "[Xalpha beta gamma]");
}

#[test]
fn an_insertion_after_the_formatted_run_splits_nothing() {
    check_boundary(
        &outside(TEXT),
        "5",
        insert("gamma", "X"),
        "[alpha beta gammaX]",
    );
}

#[test]
fn an_insertion_after_alices_insertion_splits_nothing() {
    check_boundary(
        &inside(TEXT),
        "7",
        insert("gamma", "X"),
        "[alpha beta gammaX]",
    );
}
