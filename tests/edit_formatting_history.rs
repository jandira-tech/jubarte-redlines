// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Another author's formatting history on a run that an edit splits (#1656).
//!
//! Alice made a run bold (`w:rPrChange`, italic before). Bob's edit splits
//! that run. Each retained fragment keeps Alice's record, under an id of
//! its own; the first fragment keeps the original id. Bob's own text never
//! carries Alice's history, in the redline or the clean copy (#1658).
mod common;

use common::docx::{Part, docx, docx_with_sect, part_string};
use jubarte::changes::{ChangeFilter, accept_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::namespaces::W;
use jubarte::validate::audit_tracked;
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

/// Bob's edit; both outputs must hold unique revision ids, and the
/// redline must hold Alice's history as she wrote it.
fn edit(source: &[u8], operations: Value) -> Vec<u8> {
    let new = new_words(&operations);
    let out = apply_plan(source, &plan(operations));
    let out = out.unwrap_or_else(|e| panic!("edit must succeed: {e:?}"));
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_only_alices_words(&alice_records(bytes), &new);
    }
    let audit = audit_tracked(source, &out.redline, BOB).unwrap();
    assert!(audit.is_empty(), "the edits audit: {audit:#?}");
    out.redline
}

fn dom(bytes: &[u8]) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    (dom, root)
}

/// Every XML part of the package, by name.
fn xml_parts(bytes: &[u8]) -> Vec<(String, String)> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
        .filter(|name| name.starts_with("word/") && name.ends_with(".xml"))
        .collect();
    names
        .into_iter()
        .map(|name| {
            let xml = part_string(bytes, &name).unwrap();
            (name, xml)
        })
        .collect()
}

/// Every revision record's `(kind, w:id)` occurs once in the whole package.
fn assert_unique_ids(bytes: &[u8]) {
    let mut seen: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (part, xml) in xml_parts(bytes) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else { continue };
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
                seen.entry((local.to_string(), id.to_string()))
                    .or_default()
                    .push(part.clone());
            }
        }
    }
    let repeated: Vec<_> = seen.iter().filter(|(_, parts)| parts.len() > 1).collect();
    assert!(repeated.is_empty(), "repeated revision ids {repeated:?}");
}

fn assert_validates(bytes: &[u8]) {
    let (passed, report) = validate_report(bytes);
    assert!(passed, "validate must pass: {report}");
    assert!(!report.contains("DUPLICATE_REVISION_ID"), "{report}");
}

/// Whether `jubarte validate` passes `bytes`, and its report.
fn validate_report(bytes: &[u8]) -> (bool, String) {
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
    (out.status.success(), report)
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
        let rpr = dom.parent(change).unwrap();
        assert!(
            dom.element(rpr, &W::name("b")).is_some(),
            "a fragment keeps the run's current formatting (bold)"
        );
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

/// The words Bob's operations write.
fn new_words(operations: &Value) -> Vec<String> {
    operations
        .as_array()
        .into_iter()
        .flatten()
        .filter(|op| matches!(op["kind"].as_str(), Some("replace" | "insert")))
        .filter_map(|op| op["replacement"].as_str().or(op["text"].as_str()))
        .filter(|words| !words.is_empty())
        .map(str::to_string)
        .collect()
}

/// None of Alice's records sits on Bob's new words, in either output
/// (#1658).
fn assert_only_alices_words(records: &[Record], new: &[String]) {
    for record in records {
        for words in new {
            assert!(
                !record.text.contains(words.as_str()),
                "r{} covers Bob's {words:?}: {:?}",
                record.id,
                record.text
            );
        }
    }
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
fn a_whole_word_replacement_gives_each_fragment_its_own_id() {
    let operations = json!([{"kind":"replace","paragraph":"p0","find":"beta",
        "replacement":"new","whole":true}]);
    check_split(
        &outside(TEXT),
        "5",
        "alpha ",
        operations,
        "[alpha new gamma]",
    );
}

#[test]
fn a_formatted_replacement_gives_each_fragment_its_own_id() {
    // `format` splits the runs around the new words a second time
    // (`format_range`).
    let operations = json!([{"kind":"replace","paragraph":"p0","find":"beta",
        "replacement":"new","format":{"underline":true}}]);
    check_split(
        &outside(TEXT),
        "5",
        "alpha ",
        operations,
        "[alpha new gamma]",
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

#[test]
fn a_deletion_inside_a_run_of_several_pieces_gives_each_fragment_its_own_id() {
    // A run of three text pieces between tabs: the deletion isolates the
    // middle one, with copies of the run on both sides of it.
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>alpha</w:t><w:tab/><w:t>beta</w:t><w:tab/><w:t>gamma</w:t></w:r><w:r><w:t>]</w:t></w:r></w:p>"#
    ));
    let operations = json!([{"kind":"delete","paragraph":"p0","find":"beta"}]);
    let out = edit(&source, operations);
    let records = alice_records(&out);
    assert_eq!(covered(&records), "alphabetagamma");
    assert!(records.len() > 1, "the deletion splits Alice's run");
    assert_eq!(records[0].id, "5", "the first fragment keeps the id");
    assert_eq!(records[0].text, "alpha");
}

#[test]
fn fresh_ids_stay_in_words_signed_32_bit_range() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r>{}<w:r><w:t>]</w:t></w:r></w:p>"#,
        formatted_run(2_147_483_647, TEXT)
    ));
    let operations = json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]);
    let out = edit(&source, operations);
    let (dom, root) = dom(&out);
    for node in dom.descendants(root, None) {
        if let Some(id) = dom.attribute(node, &W::id()) {
            let id: u64 = id.parse().unwrap();
            assert!(id <= i32::MAX as u64, "w:id {id} is past Word's range");
        }
    }
}

#[test]
fn two_formattings_of_one_run_keep_one_record_of_its_old_properties() {
    let source = docx(r#"<w:p><w:r><w:t xml:space="preserve">alpha beta gamma</w:t></w:r></w:p>"#);
    let operations = json!([
        {"kind":"format_run","paragraph":"p0","find":"alpha beta gamma","format":{"bold":true}},
        {"kind":"format_run","paragraph":"p0","find":"alpha beta gamma","format":{"italic":true}}
    ]);
    let out = apply_plan(&source, &plan(operations)).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    let (dom, root) = dom(&out.redline);
    let records = dom.descendants(root, Some(&W::name("rPrChange")));
    assert_eq!(records.len(), 1, "{}", dom.serialize_element(root));
    let record = records[0];
    assert_eq!(dom.attribute(record, &W::author()), Some(BOB));
    let old = dom.element(record, &W::r_pr()).expect("old properties");
    assert!(
        dom.elements(old, None).is_empty(),
        "the run had no formatting"
    );
    let rpr = dom.parent(record).unwrap();
    assert!(dom.element(rpr, &W::name("b")).is_some());
    assert!(dom.element(rpr, &W::name("i")).is_some());
}

#[test]
fn a_copied_record_with_the_editors_own_stamp_is_not_counted_as_an_edit() {
    // Alice comments on her own formatted run, with the very stamp her
    // record has: the copies the split makes are not new formatting.
    let request = json!({"schema_version":1,"author":ALICE,"date":OLD,
        "existing_revisions":"keep",
        "operations":[{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]});
    let plan = EditPlan::from_json(&request.to_string()).unwrap();
    let out = apply_plan(&outside(TEXT), &plan).unwrap();
    assert_unique_ids(&out.redline);
    assert_eq!(
        out.report.revisions.format_changed, 0,
        "{:?}",
        out.report.revisions
    );
}

#[test]
fn overlapping_formattings_give_every_fragment_one_record_of_its_own() {
    // The second formatting splits the run the first one recorded.
    let source = docx(r#"<w:p><w:r><w:t xml:space="preserve">alpha beta gamma</w:t></w:r></w:p>"#);
    let operations = json!([
        {"kind":"format_run","paragraph":"p0","find":"alpha beta","format":{"bold":true}},
        {"kind":"format_run","paragraph":"p0","find":"beta gamma","format":{"italic":true}}
    ]);
    let out = apply_plan(&source, &plan(operations)).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    let (dom, root) = dom(&out.redline);
    let mut fragments = Vec::new();
    for record in dom.descendants(root, Some(&W::name("rPrChange"))) {
        assert_eq!(dom.attribute(record, &W::author()), Some(BOB));
        let old = dom.element(record, &W::r_pr()).expect("old properties");
        assert!(
            dom.elements(old, None).is_empty(),
            "the run had no formatting"
        );
        let rpr = dom.parent(record).unwrap();
        let run = dom.parent(rpr).unwrap();
        fragments.push((
            dom.descendants(run, Some(&W::t()))
                .iter()
                .map(|&t| dom.value(t))
                .collect::<String>(),
            dom.element(rpr, &W::name("b")).is_some(),
            dom.element(rpr, &W::name("i")).is_some(),
        ));
    }
    assert_eq!(
        fragments,
        [
            ("alpha ".to_string(), true, false),
            ("beta".to_string(), true, true),
            (" gamma".to_string(), false, true),
        ]
    );
}

#[test]
fn fresh_ids_skip_the_ids_other_parts_use() {
    // The header's insertion holds id 6, the next id after the body's: a
    // copy must not take it.
    let header = format!(
        r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="6" w:author="{ALICE}" w:date="{OLD}"><w:r><w:t>head</w:t></w:r></w:ins></w:p></w:hdr>"#
    );
    let source = docx_with_sect(
        &format!(
            r#"<w:p><w:r><w:t>[</w:t></w:r>{}<w:r><w:t>]</w:t></w:r></w:p>"#,
            formatted_run(5, TEXT)
        ),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let operations = json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]);
    let out = edit(&source, operations);
    let records = alice_records(&out);
    assert_eq!(covered(&records), TEXT);
    assert_eq!(records[0].id, "5");
    for record in &records[1..] {
        assert_ne!(record.id, "6", "a copy took the header's id");
    }
}

#[test]
fn a_split_beside_a_tab_keeps_the_id_on_the_first_fragment() {
    // The run holds `alpha`, a tab and `beta`: deleting the tab and `beta`
    // splits it into pieces copied on both sides of the original.
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>alpha</w:t><w:tab/><w:t>beta</w:t></w:r><w:r><w:t>]</w:t></w:r></w:p>"#
    ));
    let operations = json!([{"kind":"delete","paragraph":"p0","find":"\tbeta"}]);
    let out = edit(&source, operations);
    let records = alice_records(&out);
    assert!(records.len() > 1, "the edit splits Alice's run");
    assert_eq!(records[0].id, "5", "the first fragment keeps the id");
    assert_eq!(records[0].text, "alpha");
    assert_eq!(covered(&records), "alphabeta");
    assert_eq!(accepted(&out), "[alpha]");
}

#[test]
fn formatting_over_another_authors_record_is_refused() {
    // The record already holds the run's old properties: a second record
    // over it would copy Alice's, so `keep` refuses the formatting.
    let operations =
        json!([{"kind":"format_run","paragraph":"p0","find":"beta","format":{"italic":true}}]);
    let error = apply_plan(&outside(TEXT), &plan(operations)).unwrap_err();
    assert_eq!(error.code, "UNSUPPORTED_STRUCTURE", "{error:?}");
}

#[test]
fn a_bookmark_at_the_top_of_words_range_does_not_push_ids_past_it() {
    // Bookmark ids are not revision ids, but the counter avoids them too:
    // past the top it takes the lowest free id instead.
    let source = docx(&format!(
        r#"<w:p><w:bookmarkStart w:id="2147483647" w:name="top"/><w:r><w:t>[</w:t></w:r>{}<w:r><w:t>]</w:t></w:r><w:bookmarkEnd w:id="2147483647"/></w:p>"#,
        formatted_run(5, TEXT)
    ));
    let operations = json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]);
    let out = edit(&source, operations);
    let records = alice_records(&out);
    assert_eq!(covered(&records), TEXT);
    for record in &records {
        let id: u64 = record.id.parse().unwrap();
        assert!(id <= 2_147_483_647, "w:id {id} is past Word's range");
    }
}

#[test]
fn an_id_the_settling_removed_names_no_new_record() {
    // Accepting Alice's insertion r6 takes the source's highest id; the
    // report names r6 as resolved, so no copy in the output may reuse it.
    let source = docx(&format!(
        r#"<w:p>{}<w:ins w:id="6" w:author="{ALICE}" w:date="{OLD}"><w:r><w:t>!</w:t></w:r></w:ins></w:p>"#,
        formatted_run(5, TEXT)
    ));
    let mut request = json!({"schema_version":1,"author":BOB,"date":DATE,
        "existing_revisions":"keep","resolve_revisions":{"accept":{"ids":["r6"]}}});
    request["operations"] =
        json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"note"}]);
    let out = apply_plan(&source, &EditPlan::from_json(&request.to_string()).unwrap())
        .unwrap_or_else(|e| panic!("edit must succeed: {e:?}"));
    for bytes in [&out.redline, &out.clean] {
        let records = alice_records(bytes);
        assert!(records.len() > 1, "the comment splits Alice's run");
        for record in &records {
            assert_ne!(
                record.id, "6",
                "a copy took the id the report names as resolved"
            );
        }
    }
}

/// Bob's typed (v2) plan for `source`: the template's own binding, with
/// `operations` aimed at its first paragraph.
fn edit_v2(
    source: &[u8],
    operations: impl Fn(&Value) -> Value,
) -> jubarte::edit::v2_apply::V2EditResult {
    let out = try_edit_v2(source, operations);
    let out = out.unwrap_or_else(|e| panic!("the typed edit must keep Alice's history: {e:?}"));
    let audit = audit_tracked(source, &out.redline, BOB).unwrap();
    assert!(audit.is_empty(), "the typed edit audits: {audit:#?}");
    out
}

/// Bob's typed edit, its target the generated plan's first paragraph.
fn try_edit_v2(
    source: &[u8],
    operations: impl Fn(&Value) -> Value,
) -> Result<jubarte::edit::v2_apply::V2EditResult, jubarte::edit::EditError> {
    try_edit_v2_tracking(source, "on", operations)
}

/// [`try_edit_v2`] with the plan's `track` policy.
fn try_edit_v2_tracking(
    source: &[u8],
    track: &str,
    operations: impl Fn(&Value) -> Value,
) -> Result<jubarte::edit::v2_apply::V2EditResult, jubarte::edit::EditError> {
    use jubarte::edit::{
        for_edit::ForEditRequest,
        template::{TemplateRequest, generate},
        v2::PlanV2,
        v2_apply,
    };
    let generated = generate(
        source,
        &TemplateRequest {
            view: ForEditRequest::default(),
            author: BOB.into(),
            date: DATE.into(),
        },
    )
    .unwrap();
    let mut q: Value = serde_json::from_str(&generated.plan.to_json()).unwrap();
    q["operations"] = operations(&q["operations"][0]["target"]);
    q["policy"]["track"] = json!(track);
    v2_apply::apply_plan(source, &PlanV2::from_json(&q.to_string()).unwrap())
}

#[test]
fn a_typed_comment_that_splits_the_run_keeps_alices_history() {
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"comment","target":target,"find":"beta","text":"note"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    let records = alice_records(&out.redline);
    assert!(records.len() > 1, "the comment splits Alice's run");
    assert_eq!(records[0].id, "5");
    assert_eq!(covered(&records), TEXT);
}

#[test]
fn a_typed_replacement_that_splits_the_run_keeps_alices_history() {
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"replace","target":target,"find":"beta","text":"new"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    let records = alice_records(&out.redline);
    assert_eq!(records[0].id, "5");
    assert_eq!(covered(&records), TEXT);
    assert_eq!(accepted(&out.redline), "[alpha new gamma]");
}

/// Every run formatting record's id and the text of its run, in document order.
fn formatting_records(bytes: &[u8]) -> Vec<(String, String)> {
    let (dom, root) = dom(bytes);
    dom.descendants(root, Some(&W::name("rPrChange")))
        .into_iter()
        .filter_map(|change| {
            // A paragraph mark's record sits in no run.
            let run = dom
                .ancestors(change, None)
                .into_iter()
                .find(|&a| dom.name_is(a, &W::r()))?;
            let text = dom
                .descendants(run, None)
                .into_iter()
                .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
                .map(|n| dom.value(n))
                .collect();
            Some((dom.attribute(change, &W::id()).unwrap().to_string(), text))
        })
        .collect()
}

#[test]
fn editing_ones_own_insertion_keeps_new_words_out_of_alices_record() {
    // Bob inserted the words; Alice then formatted them. Bob's replacement
    // inside his own insertion is a correction of it, but his new word is
    // not text Alice formatted.
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:ins w:id="6" w:author="{BOB}" w:date="{OLD}">{}</w:ins><w:r><w:t>]</w:t></w:r></w:p>"#,
        formatted_run(7, TEXT)
    ));
    let out = apply_plan(&source, &plan(replace("beta", "new"))).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        let records = formatting_records(bytes);
        assert_eq!(records[0].0, "7", "the first fragment keeps the id");
        for (id, text) in &records {
            assert!(
                !text.contains("new"),
                "r{id} covers Bob's new word: {text:?}"
            );
        }
    }
    assert_eq!(accepted(&out.redline), "[alpha new gamma]");
}

#[test]
fn new_words_in_a_run_without_a_record_stay_in_that_run() {
    let source = docx(
        r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">alpha beta gamma</w:t></w:r></w:p>"#,
    );
    let out = apply_plan(&source, &plan(insert("alpha", " new"))).unwrap();
    let (dom, root) = dom(&out.clean);
    let runs = dom.descendants(root, Some(&W::r()));
    assert_eq!(runs.len(), 1, "{}", dom.serialize_element(root));
    assert_eq!(text(&out.clean), "alpha new beta gamma");
}

/// `(id, text)` pairs, for comparing with [`formatting_records`].
fn pairs(records: &[(&str, &str)]) -> Vec<(String, String)> {
    records
        .iter()
        .map(|&(id, text)| (id.to_string(), text.to_string()))
        .collect()
}

#[test]
fn new_words_at_the_start_of_alices_run_leave_her_id_on_her_words() {
    // The split leaves the original run with Bob's word and copies the
    // rest off it: Alice's words, in the copy, keep her record's id.
    let source = outside(TEXT);
    edit(&source, replace("alpha", "new"));
    let out = apply_plan(&source, &plan(replace("alpha", "new"))).unwrap();
    assert_eq!(
        formatting_records(&out.clean),
        pairs(&[("5", " beta gamma")])
    );
    assert_eq!(
        formatting_records(&out.redline)[0],
        pairs(&[("5", "alpha")])[0]
    );
}

#[test]
fn a_typed_replacement_of_alices_first_words_keeps_her_history() {
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"replace","target":target,"find":"alpha","text":"new"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_only_alices_words(&alice_records(bytes), &["new".to_string()]);
    }
    assert_eq!(
        formatting_records(&out.clean),
        pairs(&[("5", " beta gamma")])
    );
    assert_eq!(covered(&alice_records(&out.redline)), TEXT);
}

#[test]
fn a_typed_replacement_of_all_of_alices_words_takes_her_record_out_of_the_clean_copy() {
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"replace","target":target,"find":TEXT,"text":"new"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_only_alices_words(&alice_records(bytes), &["new".to_string()]);
    }
    assert!(formatting_records(&out.clean).is_empty());
    assert_eq!(covered(&alice_records(&out.redline)), TEXT);
    assert_eq!(accepted(&out.redline), "[new]");
}

#[test]
fn a_typed_insertion_inside_alices_run_keeps_her_history() {
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"insert","target":target,"text":" new","after_text":"alpha"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_only_alices_words(&alice_records(bytes), &["new".to_string()]);
    }
    assert_eq!(covered(&alice_records(&out.clean)), TEXT);
    assert_eq!(formatting_records(&out.clean)[0].0, "5");
    assert_eq!(accepted(&out.redline), "[alpha new beta gamma]");
}

/// Alice's run r5 with its words in two `w:t`: `aa` and ` bb`.
fn two_texts() -> String {
    formatted_run(5, "aa").replace(
        "</w:t></w:r>",
        r#"</w:t><w:t xml:space="preserve"> bb</w:t></w:r>"#,
    )
}

#[test]
fn new_words_that_end_where_a_text_ends_leave_the_next_text_its_record() {
    let source = docx(&format!("<w:p>{}</w:p>", two_texts()));
    edit(&source, replace("aa", "XX"));
    let out = apply_plan(&source, &plan(replace("aa", "XX"))).unwrap();
    assert_eq!(formatting_records(&out.clean), pairs(&[("5", " bb")]));
    assert_eq!(covered(&alice_records(&out.redline)), "aa bb");
    let typed = edit_v2(
        &source,
        |target| json!([{"kind":"replace","target":target,"find":"aa","text":"XX"}]),
    );
    assert_eq!(formatting_records(&typed.clean), pairs(&[("5", " bb")]));
}

/// Alice's run `text` (none: no `w:t`) with a tab after it, then `]`.
fn text_and_tab(text: &str) -> Vec<u8> {
    let run = formatted_run(5, text)
        .replace(r#"<w:t xml:space="preserve"></w:t>"#, "")
        .replace("</w:t></w:r>", "</w:t><w:tab/></w:r>")
        .replace("</w:rPr></w:r>", "</w:rPr><w:tab/></w:r>");
    docx(&format!("<w:p>{run}<w:r><w:t>]</w:t></w:r></w:p>"))
}

#[test]
fn new_words_after_a_tab_leave_alices_record_on_her_run() {
    // The new words go in a `w:t` of their own beside the tab, inside
    // Alice's run: the run splits there, and only they lose the record.
    for (words, after) in [("alpha", "alpha\t"), ("", "\t")] {
        let source = text_and_tab(words);
        let operations = insert(after, " new");
        edit(&source, operations.clone());
        let out = apply_plan(&source, &plan(operations)).unwrap();
        assert_eq!(formatting_records(&out.clean), pairs(&[("5", words)]));
        let typed = edit_v2(
            &source,
            |target| json!([{"kind":"insert","target":target,"text":" new","after_text":after}]),
        );
        assert_eq!(formatting_records(&typed.clean), pairs(&[("5", words)]));
    }
}

#[test]
fn a_correction_of_ones_own_insertion_leaves_the_next_text_its_record() {
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="{BOB}" w:date="{OLD}">{}</w:ins></w:p>"#,
        two_texts()
    ));
    let out = apply_plan(&source, &plan(replace("aa", "XX"))).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_eq!(formatting_records(bytes), pairs(&[("5", " bb")]));
    }
}

#[test]
fn a_rewrite_of_two_words_takes_both_out_of_alices_record() {
    // Two fragments of one operation, around a two-byte character: each
    // is cleaned where it landed.
    let source = docx(&format!("<w:p>{}</w:p>", formatted_run(5, "aa é bb tail")));
    let operations = json!([{"kind":"rewrite","paragraph":"p0","text":"XXXX é YY tail"}]);
    edit(&source, operations.clone());
    let out = apply_plan(&source, &plan(operations)).unwrap();
    assert_eq!(text(&out.clean), "XXXX é YY tail");
    assert_only_alices_words(
        &alice_records(&out.clean),
        &["XXXX".to_string(), "YY".to_string()],
    );
    assert_eq!(covered(&alice_records(&out.clean)), " é  tail");
}

/// The local names of `run`'s properties.
fn property_names(dom: &Dom, run: NodeId) -> Vec<String> {
    dom.element(run, &W::r_pr())
        .map(|rpr| {
            dom.elements(rpr, None)
                .into_iter()
                .filter_map(|n| dom.name(n).map(|q| q.local_name().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn new_words_beside_a_run_without_text_leave_that_run_alone() {
    // A run holding only Word's rendered page break projects no text: it
    // is no copy of Alice's run, and her record never moves onto it.
    let other = r#"<w:r><w:rPr><w:sz w:val="28"/></w:rPr><w:lastRenderedPageBreak/></w:r>"#;
    let source = docx(&format!(
        r#"<w:p>{}{other}<w:r><w:t xml:space="preserve"> tail</w:t></w:r></w:p>"#,
        formatted_run(5, "alpha")
    ));
    let typed = edit_v2(
        &source,
        |target| json!([{"kind":"replace","target":target,"find":"alpha","text":"new"}]),
    );
    let out = apply_plan(&source, &plan(replace("alpha", "new"))).unwrap();
    for bytes in [&out.clean, &typed.clean] {
        assert!(formatting_records(bytes).is_empty());
        let (dom, root) = dom(bytes);
        let br = dom.descendants(root, Some(&W::name("lastRenderedPageBreak")))[0];
        assert_eq!(property_names(&dom, dom.parent(br).unwrap()), ["sz"]);
        let new = dom
            .descendants(root, Some(&W::t()))
            .into_iter()
            .find(|&t| dom.value(t) == "new")
            .unwrap();
        assert_eq!(property_names(&dom, dom.parent(new).unwrap()), ["b"]);
    }
}

#[test]
fn new_words_take_no_empty_properties() {
    // Alice's record is the run's only property.
    let source = docx(&format!(
        "<w:p>{}</w:p>",
        formatted_run(5, TEXT).replace("<w:b/>", "")
    ));
    let out = apply_plan(&source, &plan(replace("alpha", "new"))).unwrap();
    let (dom, root) = dom(&out.clean);
    let new = dom
        .descendants(root, Some(&W::t()))
        .into_iter()
        .find(|&t| dom.value(t) == "new")
        .unwrap();
    assert!(dom.element(dom.parent(new).unwrap(), &W::r_pr()).is_none());
}

#[test]
fn two_adjacent_replacements_of_alices_first_words_leave_her_id_on_the_rest() {
    // Applied last to first: the second replacement leaves the id on
    // "alpha" and copies " gamma" off with a fresh one; the first then
    // takes "alpha" too, and the id moves on to Alice's words, as one
    // replacement of "alpha beta" moves it (Codex on #1664).
    let out = edit_v2(&outside(TEXT), |target| {
        json!([
            {"kind":"replace","target":target,"find":"alpha","text":"A"},
            {"kind":"replace","target":target,"find":" beta","text":" B"}
        ])
    });
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(formatting_records(&out.clean), pairs(&[("5", " gamma")]));
    assert_eq!(covered(&alice_records(&out.redline)), TEXT);
    assert_eq!(accepted(&out.redline), "[A B gamma]");
    let operations = json!([
        {"kind":"replace","paragraph":"p0","find":"alpha","replacement":"A"},
        {"kind":"replace","paragraph":"p0","find":" beta","replacement":" B"}
    ]);
    // v1 diffs " beta" against " B": the space stays Alice's, between
    // Bob's words, and keeps the id; " gamma" is a fragment of its own.
    edit(&outside(TEXT), operations.clone());
    let v1 = apply_plan(&outside(TEXT), &plan(operations)).unwrap();
    let records = formatting_records(&v1.clean);
    assert_eq!(records[0], pairs(&[("5", " ")])[0], "{records:?}");
    assert_eq!(
        records
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        [" ", " gamma"]
    );
}

#[test]
fn a_typed_deletion_of_all_of_alices_words_takes_her_record_out_of_the_clean_copy() {
    // No words left, no record: an empty run keeps no formatting history
    // (Codex on #1664).
    let out = edit_v2(
        &outside(TEXT),
        |target| json!([{"kind":"delete","target":target,"find":TEXT}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert!(
        formatting_records(&out.clean).is_empty(),
        "{:?}",
        formatting_records(&out.clean)
    );
    assert_eq!(covered(&alice_records(&out.redline)), TEXT);
    assert_eq!(accepted(&out.redline), "[]");
}

#[test]
fn a_deletion_of_alices_first_words_after_a_split_leaves_her_id_on_the_rest() {
    // The replacement, applied first, copies " gamma" off with a fresh id;
    // the deletion then empties "alpha": the id moves on to Alice's words
    // (Codex on #1664, round 5).
    let out = edit_v2(&outside(TEXT), |target| {
        json!([
            {"kind":"delete","target":target,"find":"alpha"},
            {"kind":"replace","target":target,"find":" beta","text":" B"}
        ])
    });
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(formatting_records(&out.clean), pairs(&[("5", " gamma")]));
    assert_eq!(covered(&alice_records(&out.redline)), TEXT);
    assert_eq!(accepted(&out.redline), "[ B gamma]");
}

#[test]
fn a_deletion_that_leaves_a_page_break_leaves_it_alices_record() {
    // A page break is content the record covers; deleting the words
    // before it does not take the record off it (Codex on #1664, round 5).
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>abc</w:t><w:br w:type="page"/></w:r><w:r><w:t>]</w:t></w:r></w:p>"#
    ));
    let out = apply_plan(
        &source,
        &plan(json!([{"kind":"delete","paragraph":"p0","find":"abc"}])),
    )
    .unwrap();
    let (dom, root) = dom(&out.clean);
    let breaks = dom.descendants(root, Some(&W::name("br")));
    assert_eq!(breaks.len(), 1);
    let run = dom.parent(breaks[0]).unwrap();
    let rpr = dom
        .element(run, &W::name("rPr"))
        .expect("the break's run keeps its properties");
    let record = dom.element(rpr, &W::name("rPrChange"));
    assert!(
        record.is_some_and(|r| dom.attribute(r, &W::id()) == Some("5")),
        "{}",
        dom.serialize_element(run)
    );
}

#[test]
fn a_handed_on_id_keeps_its_spelling() {
    // `w:id="005"`: the id that moves on to Alice's words is the source's,
    // as it is spelled (Codex on #1664, round 5).
    let source = docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="005" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t xml:space="preserve">{TEXT}</w:t></w:r><w:r><w:t>]</w:t></w:r></w:p>"#
    ));
    let out = edit_v2(&source, |target| {
        json!([
            {"kind":"replace","target":target,"find":"alpha","text":"A"},
            {"kind":"replace","target":target,"find":" beta","text":" B"}
        ])
    });
    assert_eq!(formatting_records(&out.clean), pairs(&[("005", " gamma")]));
}

/// Alice's insertion: her formatted run `alpha beta` (record `w:id` 5) and
/// a plain run ` gamma`.
fn alices_two_runs() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:ins w:id="7" w:author="{ALICE}" w:date="{OLD}">{}<w:r><w:t xml:space="preserve"> gamma</w:t></w:r></w:ins></w:p>"#,
        formatted_run(5, "alpha beta")
    ))
}

/// Alice's formatting records once Bob's changes are accepted.
fn bob_accepted(redline: &[u8]) -> Vec<(String, String)> {
    let filter = ChangeFilter {
        authors: Some(vec![BOB.to_string()]),
        ..ChangeFilter::default()
    };
    formatting_records(&accept_changes(redline, &filter).unwrap())
}

/// The text between the paragraph's comment range markers.
fn commented(bytes: &[u8]) -> String {
    let (dom, root) = dom(bytes);
    let mut inside = false;
    let mut out = String::new();
    for node in dom.descendants(root, None) {
        if dom.name_is(node, &W::name("commentRangeStart")) {
            inside = true;
        } else if dom.name_is(node, &W::name("commentRangeEnd")) {
            inside = false;
        } else if inside && dom.name_is(node, &W::t()) {
            out.push_str(&dom.value(node));
        }
    }
    out
}

#[test]
fn a_typed_replacement_inside_alices_insertion_leaves_her_record_on_the_words_it_keeps() {
    // The redline replays the replacement word by word inside Alice's
    // insertion: `alpha beta` stays her run, with her record. The clean
    // copy keeps the same words, and so her record (MiniMax on #1664).
    let source = alices_two_runs();
    let out = edit_v2(
        &source,
        |target| json!([{"kind":"replace","target":target,"find":TEXT,"text":"alpha beta delta"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(bob_accepted(&out.redline), pairs(&[("5", "alpha beta")]));
    assert_eq!(
        formatting_records(&out.clean),
        pairs(&[("5", "alpha beta")])
    );
    assert_eq!(text(&out.clean), "alpha beta delta");
    let v1 = apply_plan(&source, &plan(replace(TEXT, "alpha beta delta"))).unwrap();
    assert_eq!(formatting_records(&v1.clean), pairs(&[("5", "alpha beta")]));
}

#[test]
fn a_typed_replacement_with_a_comment_inside_alices_insertion_comments_all_its_words() {
    let out = edit_v2(&alices_two_runs(), |target| {
        json!([{"kind":"replace","target":target,"find":TEXT,"text":"alpha beta delta",
            "attached_comment":{"text":"why"}}])
    });
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(commented(&out.clean), "alpha beta delta");
    assert_eq!(commented(&out.redline), "alpha beta delta");
    assert_eq!(
        formatting_records(&out.clean),
        pairs(&[("5", "alpha beta")])
    );
    assert_eq!(bob_accepted(&out.redline), pairs(&[("5", "alpha beta")]));
}

#[test]
fn typed_replacements_inside_one_insertion_that_overlap_still_conflict() {
    // The first replacement writes back `alpha beta` and changes `gamma`;
    // the second changes `beta`. Their found texts overlap.
    let Err(error) = try_edit_v2(&alices_two_runs(), |target| {
        json!([
            {"kind":"replace","target":target,"find":TEXT,"text":"alpha beta delta"},
            {"kind":"replace","target":target,"find":"beta","text":"BETA"}
        ])
    }) else {
        panic!("overlapping replacements conflict");
    };
    assert_eq!(error.code, "CONFLICT", "{error:?}");
}

#[test]
fn a_record_left_on_a_tab_keeps_its_id() {
    // Alice's record covers `a` and the tab after it. Bob's new words take
    // `a`; the tab is what the record still covers, and it keeps the id
    // (MiniMax on #1664, round 5).
    let source = docx(&format!(
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="5" w:author="{ALICE}" w:date="{OLD}"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>a</w:t><w:tab/></w:r></w:p>"#
    ));
    let out = apply_plan(&source, &plan(replace("a", "xy"))).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    let (dom, root) = dom(&out.clean);
    let tabs = dom.descendants(root, Some(&W::name("tab")));
    let run = dom.parent(tabs[0]).unwrap();
    let record = dom
        .element(run, &W::r_pr())
        .and_then(|rpr| dom.element(rpr, &W::r_pr_change()));
    assert!(
        record.is_some_and(|r| dom.attribute(r, &W::id()) == Some("5")),
        "{}",
        dom.serialize_element(run)
    );
    assert_eq!(formatting_records(&out.clean), pairs(&[("5", "")]));
}

#[test]
fn a_paragraph_marks_record_with_the_same_id_does_not_hold_alices_id_back() {
    // The paragraph mark's record shares w:id 5 with the run's: only run
    // records hold the run's lineage, so the id still moves on to Alice's
    // words (glm on #1664, round 5).
    let mark = format!(
        r#"<w:pPr><w:rPr><w:rPrChange w:id="5" w:author="{ALICE}" w:date="{OLD}"><w:rPr/></w:rPrChange></w:rPr></w:pPr>"#
    );
    let source = docx(&format!(
        r#"<w:p>{mark}<w:r><w:t>[</w:t></w:r>{}<w:r><w:t>]</w:t></w:r></w:p>"#,
        formatted_run(5, TEXT)
    ));
    let out = edit_v2(&source, |target| {
        json!([
            {"kind":"replace","target":target,"find":"alpha","text":"A"},
            {"kind":"replace","target":target,"find":" beta","text":" B"}
        ])
    });
    // The mark and the run share an id in the source already: the outputs
    // keep that duplicate and add none.
    let duplicates = |bytes: &[u8]| {
        validate_report(bytes)
            .1
            .matches("DUPLICATE_REVISION_ID")
            .count()
    };
    assert_eq!(duplicates(&source), 1);
    for bytes in [&out.redline, &out.clean] {
        assert_eq!(duplicates(bytes), 1, "{}", validate_report(bytes).1);
    }
    assert_eq!(formatting_records(&out.clean), pairs(&[("5", " gamma")]));
}

#[test]
fn a_typed_replacement_of_alices_first_words_inside_her_insertion_agrees_in_both_outputs() {
    // One formatted run inside Alice's insertion; the replacement keeps
    // `alpha `: the clean copy keeps her record where the accepted redline
    // does (crush on #1664, round 5).
    let out = edit_v2(
        &inside(TEXT),
        |target| json!([{"kind":"replace","target":target,"find":"alpha beta","text":"alpha new"}]),
    );
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(text(&out.clean), "[alpha new gamma]");
    let clean = formatting_records(&out.clean);
    assert_eq!(bob_accepted(&out.redline), clean);
    assert_eq!(covered(&alice_records(&out.clean)), "alpha  gamma");
    assert_eq!(clean[0], pairs(&[("7", "alpha ")])[0], "{clean:?}");
}

#[test]
fn an_untracked_correction_of_ones_own_insertion_keeps_alices_record_on_the_words_it_keeps() {
    // Bob corrects his own insertion, which holds Alice's formatted run.
    // Untracked, the typed replacement splits word by word as the tracked
    // replay does: ` beta ` stays Alice's, with her record, as in v1
    // (#1662's untracked acceptance). The tracked plan is #1662 itself.
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="{BOB}" w:date="{OLD}">{}</w:ins></w:p>"#,
        formatted_run(7, TEXT)
    ));
    let out = try_edit_v2_tracking(
        &source,
        "off",
        |target| json!([{"kind":"replace","target":target,"find":TEXT,"text":"X beta Y"}]),
    )
    .unwrap();
    assert_validates(&out.clean);
    assert_eq!(text(&out.clean), "X beta Y");
    assert_eq!(formatting_records(&out.clean), pairs(&[("7", " beta ")]));
    let v1 = apply_plan(&source, &plan(replace(TEXT, "X beta Y"))).unwrap();
    assert_eq!(
        formatting_records(&v1.clean),
        formatting_records(&out.clean)
    );
}

/// Alice's insertion `alpha beta`, after `before` (raw XML) in one paragraph.
fn alices_insertion_after(before: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p>{before}<w:ins w:id="7" w:author="{ALICE}" w:date="{OLD}"><w:r><w:t>alpha beta</w:t></w:r></w:ins></w:p>"#
    ))
}

/// The text of the runs inside `ancestor` elements (`w:hyperlink`,
/// `w:moveTo`).
fn text_under(bytes: &[u8], ancestor: &str) -> String {
    let (dom, root) = dom(bytes);
    dom.descendants(root, Some(&W::t()))
        .into_iter()
        .filter(|&t| {
            dom.ancestors(t, None)
                .into_iter()
                .any(|a| dom.name_is(a, &W::name(ancestor)))
        })
        .map(|t| dom.value(t))
        .collect()
}

#[test]
fn a_typed_replacements_comment_holds_its_new_edge_words_in_both_outputs() {
    // The word diff leaves `new ` or ` new` as an insertion at the match's
    // edge; the comment holds it in the redline as in the clean copy (codex,
    // glm-5.3, MiniMax and crush on #1664, round 6).
    for replacement in ["new alpha beta", "alpha beta new"] {
        let out = edit_v2(&alices_insertion_after(""), |target| {
            json!([{"kind":"replace","target":target,"find":"alpha beta","text":replacement,
                "attached_comment":{"text":"why"}}])
        });
        for bytes in [&out.redline, &out.clean] {
            assert_unique_ids(bytes);
            assert_validates(bytes);
        }
        assert_eq!(commented(&out.clean), replacement);
        assert_eq!(commented(&out.redline), replacement);
    }
}

#[test]
fn a_typed_replacements_new_first_words_stay_out_of_the_hyperlink_before_it() {
    // A hyperlink ends where the match starts: the new first words go
    // inside Alice's insertion, as v1 puts them (codex, glm-5.3 and MiniMax
    // on #1664, round 6).
    let source = alices_insertion_after(
        r#"<w:bookmarkStart w:id="0" w:name="dest"/><w:bookmarkEnd w:id="0"/><w:hyperlink w:anchor="dest"><w:r><w:t>L</w:t></w:r></w:hyperlink>"#,
    );
    let out = edit_v2(
        &source,
        |target| json!([{"kind":"replace","target":target,"find":"alpha beta","text":"new alpha beta"}]),
    );
    let v1 = apply_plan(&source, &plan(replace("alpha beta", "new alpha beta"))).unwrap();
    for bytes in [&out.redline, &out.clean, &v1.redline, &v1.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_eq!(text_under(bytes, "hyperlink"), "L");
    }
    assert_eq!(text(&out.clean), "Lnew alpha beta");
    assert_eq!(accepted(&out.redline), text(&out.clean));
}

#[test]
fn a_typed_replacements_new_first_words_stay_out_of_the_move_before_it() {
    // Carol's moved text ends where the match starts: the new first words
    // go inside Alice's insertion, not into Carol's move (glm-5.3 on #1664,
    // round 6).
    let source = docx(&format!(
        r#"<w:p><w:moveToRangeStart w:id="1" w:author="Carol" w:date="{OLD}" w:name="move1"/><w:moveTo w:id="2" w:author="Carol" w:date="{OLD}"><w:r><w:t>M</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="1"/><w:ins w:id="7" w:author="{ALICE}" w:date="{OLD}"><w:r><w:t>alpha beta</w:t></w:r></w:ins></w:p><w:p><w:moveFromRangeStart w:id="3" w:author="Carol" w:date="{OLD}" w:name="move1"/><w:moveFrom w:id="4" w:author="Carol" w:date="{OLD}"><w:r><w:t>M</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="3"/><w:r><w:t>tail</w:t></w:r></w:p>"#
    ));
    let out = edit_v2(
        &source,
        |target| json!([{"kind":"replace","target":target,"find":"alpha beta","text":"new alpha beta"}]),
    );
    let v1 = apply_plan(&source, &plan(replace("alpha beta", "new alpha beta"))).unwrap();
    for bytes in [&out.redline, &out.clean, &v1.redline, &v1.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
        assert_eq!(text_under(bytes, "moveTo"), "M");
    }
    assert_eq!(accepted(&out.redline), text(&out.clean));
}

#[test]
fn a_split_typed_replacement_claims_its_whole_match_at_a_boundary() {
    // The replacement keeps `alpha beta` and adds ` delta` at its end; a
    // second operation inserts at that end, on the other side. The whole
    // match is the first operation's claim, which the insertion only
    // touches, as before the split (MiniMax on #1664, round 6).
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="7" w:author="{ALICE}" w:date="{OLD}"><w:r><w:t>alpha beta</w:t></w:r></w:ins><w:r><w:t>gamma</w:t></w:r></w:p>"#
    ));
    let out = edit_v2(&source, |target| {
        json!([
            {"kind":"replace","target":target,"find":"alpha beta","text":"alpha beta delta"},
            {"kind":"insert","target":target,"text":"Y","before_text":"gamma"}
        ])
    });
    for bytes in [&out.redline, &out.clean] {
        assert_unique_ids(bytes);
        assert_validates(bytes);
    }
    assert_eq!(text(&out.clean), "alpha beta deltaYgamma");
    assert_eq!(accepted(&out.redline), text(&out.clean));
}
