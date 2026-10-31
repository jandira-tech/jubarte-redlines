// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's edit-inside-insertion shapes, through every edit entrypoint.
mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};
use serde_json::{Value, json};
use std::process::Command;

const DATE: &str = "2026-10-18T10:00:00Z";
const OLD_DATE: &str = "2026-10-17T09:00:00Z";
const WORD: &[u8] = include_bytes!("fixtures/re-dialect/step2-tracked.docx");

fn source(author: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t>[</w:t></w:r><w:ins w:id="6" w:author="{author}" w:date="{OLD_DATE}"><w:r><w:rPr><w:b/></w:rPr><w:t>alpha beta gamma</w:t></w:r></w:ins><w:r><w:t>]</w:t></w:r></w:p>"#
    ))
}
fn plan(author: &str, operations: Value) -> EditPlan {
    let mut request = json!({"schema_version":1,"author":author,"date":DATE,
        "existing_revisions":"keep","operations":[]});
    request["operations"] = operations;
    EditPlan::from_json(&request.to_string()).unwrap()
}

fn replace(find: &str, text: &str) -> Value {
    json!([{"kind":"replace","paragraph":"p0","find":find,"replacement":text}])
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
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "validate must pass: {diagnostics}");
    assert!(
        !diagnostics.contains("DUPLICATE_REVISION_ID"),
        "{diagnostics}"
    );
}

fn run(source: &[u8], author: &str, operations: Value) -> Vec<u8> {
    let out = apply_plan(source, &plan(author, operations));
    assert!(out.is_ok(), "edit must succeed: {:?}", out.err());
    let out = out.unwrap();
    validate_output(&out.redline);
    validate_output(&out.clean);
    out.redline
}
fn tree(bytes: &[u8]) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&xml);
    let p = dom.descendants(doc, Some(&W::p()))[0];
    (dom, p)
}
fn text(bytes: &[u8]) -> String {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect::<Vec<_>>()
        .join("\n")
}
fn settled(bytes: &[u8], expected: &str, rejected: &str) {
    assert_eq!(
        text(&accept_changes(bytes, &ChangeFilter::default()).unwrap()),
        expected
    );
    assert_eq!(
        text(&reject_changes(bytes, &ChangeFilter::default()).unwrap()),
        rejected
    );
}
fn revision(
    dom: &Dom,
    p: NodeId,
    kind: &str,
    id: &str,
    author: &str,
    date: &str,
    value: &str,
) -> NodeId {
    let node = dom
        .descendants(p, Some(&W::name(kind)))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some(id))
        .unwrap_or_else(|| panic!("missing {kind}:{id}: {}", dom.serialize_element(p)));
    assert_eq!(dom.attribute(node, &W::author()), Some(author));
    assert_eq!(dom.attribute(node, &W::date()), Some(date));
    assert_eq!(dom.value(node), value);
    node
}
fn split_case(find: &str, replacement: &str, left: &str, deleted: &str, right: &str) {
    let out = run(&source("Alice"), "Carol Diaz", replace(find, replacement));
    let (dom, p) = tree(&out);
    let first = revision(
        &dom,
        p,
        "ins",
        "6",
        "Alice",
        OLD_DATE,
        &format!("{left}{deleted}"),
    );
    let del = revision(&dom, p, "del", "7", "Carol Diaz", DATE, deleted);
    assert_eq!(dom.parent(del), Some(first));
    assert!(dom.descendants(del, Some(&W::t())).is_empty());
    assert_eq!(dom.descendants(del, Some(&W::del_text())).len(), 1);
    let mut last = first;
    let next = if replacement.is_empty() {
        "8"
    } else {
        let ins = revision(&dom, p, "ins", "8", "Carol Diaz", DATE, replacement);
        assert_eq!(dom.next_element(first), Some(ins));
        assert_eq!(dom.parent(ins), Some(p));
        assert!(dom.descendants(ins, Some(&W::name("b"))).len() == 1);
        last = ins;
        "9"
    };
    if !right.is_empty() {
        let tail = revision(&dom, p, "ins", next, "Alice", OLD_DATE, right);
        assert_eq!(dom.next_element(last), Some(tail));
    }
    assert_eq!(
        dom.elements(p, Some(&W::ins())).len(),
        1 + usize::from(!replacement.is_empty()) + usize::from(!right.is_empty())
    );
    let expected = "alpha beta gamma".replacen(find, replacement, 1);
    settled(&out, &format!("[{expected}]"), "[]");
}
#[test]
fn another_author_replacement_splits_and_preserves_stamps() {
    split_case("beta", "new", "alpha ", "beta", " gamma");
}
#[test]
fn another_author_deletion_nests_without_splitting() {
    // Word nests a deletion inside another author's insertion and leaves
    // the insertion whole: annex.docx holds ASR's r4 inside AR's r3, with
    // AR's words before and after it in the one w:ins.
    let out = run(&source("Alice"), "Carol Diaz", replace("beta", ""));
    let (dom, p) = tree(&out);
    let theirs = revision(&dom, p, "ins", "6", "Alice", OLD_DATE, "alpha beta gamma");
    let del = revision(&dom, p, "del", "7", "Carol Diaz", DATE, "beta");
    assert_eq!(dom.parent(del), Some(theirs));
    assert!(dom.descendants(del, Some(&W::t())).is_empty());
    assert_eq!(dom.elements(p, Some(&W::ins())).len(), 1);
    settled(&out, "[alpha  gamma]", "[]");
}

#[test]
fn word_nests_a_deletion_inside_an_insertion_without_splitting_it() {
    let annex = include_bytes!("fixtures/re-dialect/annex.docx");
    let mut dom = Dom::new();
    let xml = part_string(annex, "word/document.xml").unwrap();
    let doc = dom.parse_xdocument(&xml);
    let r3 = dom
        .descendants(doc, Some(&W::ins()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some("3"))
        .unwrap();
    let kinds: Vec<String> = dom
        .elements(r3, None)
        .into_iter()
        .map(|n| dom.name(n).unwrap().local_name().to_string())
        .filter(|local| local != "proofErr")
        .collect();
    assert_eq!(
        kinds,
        ["del", "r", "r", "del"],
        "{}",
        dom.serialize_element(r3)
    );
}
#[test]
fn replacement_at_start_has_no_empty_prefix() {
    split_case("alpha", "new", "", "alpha", " beta gamma");
}
#[test]
fn replacement_at_end_has_no_empty_tail() {
    split_case("gamma", "new", "alpha beta ", "gamma", "");
}
#[test]
fn replacement_of_whole_insertion_has_no_empty_tail() {
    split_case("alpha beta gamma", "new", "", "alpha beta gamma", "");
}
fn insertion_case(position: &Value, left: &str, right: &str) {
    let mut op = json!({"kind":"insert","paragraph":"p0","text":"NEW"});
    for (key, value) in position.as_object().unwrap() {
        op[key] = value.clone();
    }
    let out = run(&source("Alice"), "Carol Diaz", json!([op]));
    let (dom, p) = tree(&out);
    let ins = revision(&dom, p, "ins", "7", "Carol Diaz", DATE, "NEW");
    assert_eq!(dom.parent(ins), Some(p));
    assert!(dom.descendants(p, Some(&W::del())).is_empty());
    if !left.is_empty() {
        revision(&dom, p, "ins", "6", "Alice", OLD_DATE, left);
    }
    if !right.is_empty() {
        revision(&dom, p, "ins", "8", "Alice", OLD_DATE, right);
    }
    assert_eq!(
        dom.elements(p, Some(&W::ins())).len(),
        1 + usize::from(!left.is_empty()) + usize::from(!right.is_empty())
    );
    settled(&out, &format!("[{left}NEW{right}]"), "[]");
}
#[test]
fn pure_insertion_splits_without_a_deletion() {
    insertion_case(&json!({"after":"beta"}), "alpha beta", " gamma");
}
#[test]
fn pure_insertion_at_start_leaves_no_empty_piece() {
    insertion_case(&json!({"before":"alpha"}), "", "alpha beta gamma");
}
#[test]
fn pure_insertion_at_end_leaves_no_empty_piece() {
    insertion_case(&json!({"after":"gamma"}), "alpha beta gamma", "");
}
fn own(ops: Value, expected: &str) {
    let out = run(&source("Carol Diaz"), "Carol Diaz", ops);
    let (dom, p) = tree(&out);
    revision(&dom, p, "ins", "6", "Carol Diaz", OLD_DATE, expected);
    assert_eq!(dom.elements(p, Some(&W::ins())).len(), 1);
    assert!(dom.descendants(p, Some(&W::del())).is_empty());
    settled(&out, &format!("[{expected}]"), "[]");
}
#[test]
fn own_insertion_replacement_edits_in_place() {
    own(replace("beta", "new"), "alpha new gamma");
}
#[test]
fn own_insertion_deletion_removes_words_without_marks() {
    own(replace("beta", ""), "alpha  gamma");
}
#[test]
fn own_insertion_addition_keeps_id_and_date() {
    own(
        json!([{"kind":"insert","paragraph":"p0","after":"beta","text":"NEW"}]),
        "alpha betaNEW gamma",
    );
}
#[test]
fn own_insertion_deleted_whole_leaves_no_empty_piece() {
    let out = run(
        &source("Carol Diaz"),
        "Carol Diaz",
        replace("alpha beta gamma", ""),
    );
    let (dom, p) = tree(&out);
    assert!(dom.elements(p, Some(&W::ins())).is_empty());
    settled(&out, "[]", "[]");
}
fn word_case(
    find: &str,
    replacement: &str,
    original_id: &str,
    old_author: &str,
    prefix_and_deleted: &str,
    deleted: &str,
    tail: Option<(&str, &str)>,
) {
    let out = run(WORD, "Carol Diaz", replace(find, replacement));
    let (dom, p) = tree(&out);
    let original = dom
        .descendants(p, Some(&W::ins()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some(original_id))
        .unwrap();
    assert_eq!(dom.attribute(original, &W::author()), Some(old_author));
    let (source_dom, source_p) = tree(WORD);
    let source_ins = source_dom
        .descendants(source_p, Some(&W::ins()))
        .into_iter()
        .find(|&n| source_dom.attribute(n, &W::id()) == Some(original_id))
        .unwrap();
    assert_eq!(
        dom.attribute(original, &W::date()),
        source_dom.attribute(source_ins, &W::date())
    );
    assert_eq!(dom.value(original), prefix_and_deleted);
    let del = revision(&dom, p, "del", "7", "Carol Diaz", DATE, deleted);
    assert_eq!(dom.parent(del), Some(original));
    assert_eq!(dom.descendants(del, Some(&W::del_text())).len(), 1);
    let ins = revision(&dom, p, "ins", "8", "Carol Diaz", DATE, replacement);
    assert_eq!(dom.next_element(original), Some(ins));
    if let Some((id, value)) = tail {
        revision(
            &dom,
            p,
            "ins",
            id,
            old_author,
            dom.attribute(original, &W::date()).unwrap(),
            value,
        );
    }
    let accepted = text(&accept_changes(WORD, &ChangeFilter::default()).unwrap()).replacen(
        find,
        replacement,
        1,
    );
    settled(
        &out,
        &accepted,
        &text(&reject_changes(WORD, &ChangeFilter::default()).unwrap()),
    );
}
#[test]
fn word_fixture_modify_inside_insertion() {
    word_case(
        "their",
        "its",
        "2",
        "Bob",
        "and their",
        "their",
        Some(("9", " affiliates ")),
    );
}
#[test]
fn word_fixture_modify_substitution() {
    word_case(
        "insurers",
        "underwriters",
        "5",
        "Bob",
        "insurers",
        "insurers",
        None,
    );
}
#[test]
fn word_fixture_add_inside_insertion() {
    let out = run(WORD, "Carol Diaz", replace("month.", "month, in cash."));
    let (dom, p) = tree(&out);
    revision(&dom, p, "ins", "7", "Carol Diaz", DATE, ", in cash");
    let old = dom
        .descendants(p, Some(&W::ins()))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::id()) == Some("6"))
        .unwrap();
    revision(
        &dom,
        p,
        "ins",
        "8",
        "Alma Sato Reyes",
        dom.attribute(old, &W::date()).unwrap(),
        ".",
    );
    assert_eq!(dom.attribute(old, &W::author()), Some("Alma Sato Reyes"));
    assert_eq!(dom.attribute(old, &W::date()), Some("2026-10-17T23:44:00Z"));
    assert!(dom.value(old).ends_with("month"));
    let accepted = text(&accept_changes(WORD, &ChangeFilter::default()).unwrap())
        .replace("month.", "month, in cash.");
    settled(
        &out,
        &accepted,
        &text(&reject_changes(WORD, &ChangeFilter::default()).unwrap()),
    );
}
fn refused_inside(kind: &str, t: &str) -> jubarte::edit::EditError {
    let s = docx(&format!(
        r#"<w:p><w:{kind} w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:{t}>beta</w:{t}></w:r></w:{kind}></w:p>"#
    ));
    apply_plan(&s, &plan("Carol Diaz", replace("beta", "new"))).unwrap_err()
}
#[test]
fn edit_inside_their_deletion_is_refused() {
    let e = refused_inside("del", "delText");
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(
        e.message
            .starts_with(r#"the text is inside r6, "beta" deleted by "Alice"."#),
        "{e}"
    );
    assert!(!e.message.contains("settle it first"), "{e}");
}
#[test]
fn edit_inside_move_is_refused_until_probed() {
    for (kind, t, verb) in [
        ("moveFrom", "delText", "moved away"),
        ("moveTo", "t", "moved here"),
    ] {
        let e = refused_inside(kind, t);
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{kind}: {e}");
        assert!(
            e.message.starts_with(&format!(
                r#"the text is inside r6, "beta" {verb} by "Alice"."#
            )),
            "{kind}: {e}"
        );
    }
}
fn cli_route(route: &str) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), source("Alice")).unwrap();
    let cli = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap()
    };
    let output = match route {
        "flags" => cli(&[
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--anchor",
            "beta",
            "--content",
            "new",
            "-a",
            "Carol Diaz",
            "-d",
            DATE,
            "-o",
            "out.docx",
        ]),
        "v1" => {
            std::fs::write(
                dir.path().join("plan.json"),
                plan("Carol Diaz", replace("beta", "new")).to_json(),
            )
            .unwrap();
            cli(&["edit", "in.docx", "--plan", "plan.json", "-o", "out.docx"])
        }
        _ => {
            let generated = cli(&[
                "edit",
                "template",
                "in.docx",
                "-a",
                "Carol Diaz",
                "-d",
                DATE,
                "-o",
                "-",
            ]);
            assert!(
                generated.status.success(),
                "{}",
                String::from_utf8_lossy(&generated.stderr)
            );
            let mut q: Value = serde_json::from_slice(&generated.stdout).unwrap();
            if route == "template" {
                q["operations"][0]["text"] = json!("[alpha new gamma]");
            } else {
                q["operations"] = json!([{"kind":"replace","target":q["operations"][0]["target"],"find":"beta","text":"new"}]);
            }
            std::fs::write(dir.path().join("plan.json"), q.to_string()).unwrap();
            cli(&[
                "edit",
                "apply",
                "in.docx",
                "--plan",
                "plan.json",
                "-o",
                "out.docx",
            ])
        }
    };
    assert!(
        output.status.success(),
        "{route}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = std::fs::read(dir.path().join("out.docx")).unwrap();
    assert_word_valid_package(&out);
    settled(&out, "[alpha new gamma]", "[]");
    let validation = cli(&["validate", "out.docx"]);
    assert!(
        validation.status.success(),
        "{}",
        String::from_utf8_lossy(&validation.stderr)
    );
    assert!(!String::from_utf8_lossy(&validation.stdout).contains("DUPLICATE_REVISION_ID"));
}
#[test]
fn flags_route() {
    cli_route("flags");
}
#[test]
fn v1_json_route() {
    cli_route("v1");
}
#[test]
fn plan_v2_route() {
    cli_route("v2");
}
#[test]
fn template_route() {
    cli_route("template");
}

#[test]
fn glyphs_inside_an_insertion_are_deleted_with_their_run_properties() {
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:rPr><w:b/></w:rPr><w:t>alpha</w:t><w:tab/><w:t>beta</w:t></w:r></w:ins></w:p>"#
    ));
    let out = run(&s, "Carol Diaz", replace("\t", "NEW"));
    settled(&out, "alphaNEWbeta", "");
    let (dom, p) = tree(&out);
    let del = revision(&dom, p, "del", "7", "Carol Diaz", DATE, "");
    assert_eq!(dom.descendants(del, Some(&W::name("tab"))).len(), 1);
    assert_eq!(dom.descendants(del, Some(&W::name("b"))).len(), 1);
}

#[test]
fn typing_next_to_a_glyph_in_an_insertion_keeps_the_attachment() {
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:rPr><w:b/></w:rPr><w:t>alpha</w:t><w:tab/></w:r><w:r><w:t>beta</w:t></w:r></w:ins></w:p>"#
    ));
    let out = run(
        &s,
        "Carol Diaz",
        json!([{"kind":"insert","paragraph":"p0","after":"\t","text":"NEW"}]),
    );
    settled(&out, "alpha\tNEWbeta", "");
    let (dom, p) = tree(&out);
    let ins = revision(&dom, p, "ins", "7", "Carol Diaz", DATE, "NEW");
    assert_eq!(
        dom.descendants(ins, Some(&W::name("b"))).len(),
        1,
        "new text takes the glyph's formatting"
    );
}

#[test]
fn crossing_ones_own_insertion_removes_ones_words_without_a_deletion() {
    let out = run(
        &source("Carol Diaz"),
        "Carol Diaz",
        replace("[alpha", "NEW"),
    );
    settled(&out, "NEW beta gamma]", "[]");
    let (dom, p) = tree(&out);
    for del in dom.descendants(p, Some(&W::del())) {
        assert!(
            !dom.value(del).contains("alpha"),
            "own inserted words must disappear"
        );
    }
}

#[test]
fn whole_replacement_inside_insertion_keeps_the_full_anchor() {
    let mut ops = replace("alpha beta", "alpha NEW beta");
    ops[0]["whole"] = json!(true);
    let out = run(&source("Alice"), "Carol Diaz", ops);
    let (dom, p) = tree(&out);
    revision(&dom, p, "del", "7", "Carol Diaz", DATE, "alpha beta");
    revision(&dom, p, "ins", "8", "Carol Diaz", DATE, "alpha NEW beta");
    settled(&out, "[alpha NEW beta gamma]", "[]");
}

#[test]
fn whitespace_only_replacement_inside_insertion_is_exact() {
    let out = run(
        &source("Alice"),
        "Carol Diaz",
        replace("alpha beta", "alpha  beta"),
    );
    settled(&out, "[alpha  beta gamma]", "[]");
}

#[test]
fn crossing_out_of_ones_own_insertion_also_removes_ones_words() {
    let out = run(
        &source("Carol Diaz"),
        "Carol Diaz",
        replace("gamma]", "NEW"),
    );
    settled(&out, "[alpha beta NEW", "[]");
    let (dom, p) = tree(&out);
    for del in dom.descendants(p, Some(&W::del())) {
        assert!(
            !dom.value(del).contains("gamma"),
            "own inserted words must disappear"
        );
    }
}

#[test]
fn plan_v2_split_preserves_a_nested_sources_deletion_in_the_tail() {
    use jubarte::edit::{
        for_edit::ForEditRequest,
        template::{TemplateRequest, generate},
        v2::PlanV2,
        v2_apply,
    };
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t>alpha</w:t></w:r><w:del w:id="5" w:author="Bob" w:date="{OLD_DATE}"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t> beta</w:t></w:r></w:ins></w:p>"#
    ));
    let generated = generate(
        &s,
        &TemplateRequest {
            view: ForEditRequest::default(),
            author: "Carol Diaz".into(),
            date: DATE.into(),
        },
    )
    .unwrap();
    let mut q: Value = serde_json::from_str(&generated.plan.to_json()).unwrap();
    q["operations"] = json!([{"kind":"replace","target":q["operations"][0]["target"],"find":"alpha","text":"NEW"}]);
    let out = v2_apply::apply_plan(&s, &PlanV2::from_json(&q.to_string()).unwrap());
    assert!(
        out.is_ok(),
        "splitting a source insertion must preserve its old nested deletion: {:?}",
        out.err()
    );
    let out = out.unwrap();
    validate_output(&out.redline);
    validate_output(&out.clean);
    settled(&out.redline, "NEW beta", "");
    let (dom, p) = tree(&out.redline);
    let del = revision(&dom, p, "del", "5", "Bob", OLD_DATE, "old");
    let tail = revision(&dom, p, "ins", "9", "Alice", OLD_DATE, "old beta");
    assert_eq!(dom.parent(del), Some(tail));
}

#[test]
fn template_composes_a_leading_addition_and_an_inner_replacement() {
    use jubarte::edit::{
        for_edit::ForEditRequest,
        template::{TemplateRequest, generate},
        v2::PlanV2,
        v2_apply,
    };
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t>alpha beta gamma</w:t></w:r></w:ins></w:p>"#
    ));
    let generated = generate(
        &s,
        &TemplateRequest {
            view: ForEditRequest::default(),
            author: "Carol Diaz".into(),
            date: DATE.into(),
        },
    )
    .unwrap();
    let mut q: Value = serde_json::from_str(&generated.plan.to_json()).unwrap();
    q["operations"][0]["text"] = json!("PRE alpha NEW gamma");
    let out = v2_apply::apply_plan(&s, &PlanV2::from_json(&q.to_string()).unwrap());
    assert!(
        out.is_ok(),
        "composed edits inside one source insertion must succeed: {:?}",
        out.err()
    );
    let out = out.unwrap();
    validate_output(&out.redline);
    validate_output(&out.clean);
    settled(&out.redline, "PRE alpha NEW gamma", "");
    let (dom, p) = tree(&out.redline);
    let source_pieces: Vec<_> = dom
        .elements(p, Some(&W::ins()))
        .into_iter()
        .filter(|&n| dom.attribute(n, &W::author()) == Some("Alice"))
        .collect();
    assert_eq!(
        source_pieces
            .iter()
            .map(|&n| dom.value(n))
            .collect::<Vec<_>>(),
        ["alpha beta", " gamma"]
    );
    for piece in source_pieces {
        assert_eq!(dom.attribute(piece, &W::date()), Some(OLD_DATE));
    }
    let mut ids: Vec<_> = dom
        .descendants(p, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del()))
        .map(|n| dom.attribute(n, &W::id()).unwrap().parse::<u32>().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, [7, 8, 9, 10, 11]);
}

/// Three separate runs make every marker's position independent of run splitting.
fn range_source(kind: &str, word: usize) -> Vec<u8> {
    use common::docx::{Part, docx_with};
    let (start, end, reference) = match kind {
        "bookmark" => (
            r#"<w:bookmarkStart w:id="40" w:name="Words"/>"#,
            r#"<w:bookmarkEnd w:id="40"/>"#,
            "",
        ),
        "comment" => (
            r#"<w:commentRangeStart w:id="0"/>"#,
            r#"<w:commentRangeEnd w:id="0"/>"#,
            r#"<w:r><w:commentReference w:id="0"/></w:r>"#,
        ),
        "permission" => (
            r#"<w:permStart w:id="40" w:edGrp="everyone"/>"#,
            r#"<w:permEnd w:id="40"/>"#,
            "",
        ),
        _ => unreachable!(),
    };
    let mut content = String::new();
    for (index, text) in ["alpha ", "beta", " gamma"].into_iter().enumerate() {
        if index == word {
            content.push_str(start);
        }
        content.push_str(&format!(
            r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#
        ));
        if index == word {
            content.push_str(end);
            content.push_str(reference);
        }
    }
    let body = format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}">{content}</w:ins></w:p>"#
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="0" w:author="Reviewer" w:date="{OLD_DATE}"><w:p><w:r><w:t>Existing range</w:t></w:r></w:p></w:comment></w:comments>"#,
        W::URI
    );
    let extras = if kind == "comment" {
        vec![Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }]
    } else {
        vec![]
    };
    docx_with(&body, &extras)
}

fn child_labels(dom: &Dom, parent: NodeId) -> Vec<String> {
    dom.elements(parent, None)
        .into_iter()
        .map(|node| {
            assert_eq!(dom.parent(node), Some(parent));
            if dom.name_is(node, &W::r()) {
                if let Some(reference) = dom.element(node, &W::name("commentReference")) {
                    assert_eq!(dom.parent(reference), Some(node));
                    format!(
                        "commentReference:{}",
                        dom.attribute(reference, &W::id()).unwrap()
                    )
                } else {
                    format!("r:{}", dom.value(node))
                }
            } else if dom.name_is(node, &W::del()) {
                format!("del:{}", dom.value(node))
            } else {
                format!(
                    "{}:{}",
                    dom.name(node).unwrap().local_name(),
                    dom.attribute(node, &W::id()).unwrap()
                )
            }
        })
        .collect()
}

fn range_labels(kind: &str, text: &str) -> Vec<String> {
    if kind == "bookmark" || kind == "permission" {
        let (start, end) = if kind == "bookmark" {
            ("bookmarkStart", "bookmarkEnd")
        } else {
            ("permStart", "permEnd")
        };
        vec![
            format!("{start}:40"),
            format!("r:{text}"),
            format!("{end}:40"),
        ]
    } else {
        vec![
            "commentRangeStart:0".into(),
            format!("r:{text}"),
            "commentRangeEnd:0".into(),
            "commentReference:0".into(),
        ]
    }
}

fn replacement_keeps_ranges(kind: &str, word: usize) {
    let out = run(
        &range_source(kind, word),
        "Carol Diaz",
        replace("beta", "NEW"),
    );
    let (dom, p) = tree(&out);
    let first_id = if kind == "comment" { 7 } else { 41 };
    let head = revision(&dom, p, "ins", "6", "Alice", OLD_DATE, "alpha beta");
    let inserted = revision(
        &dom,
        p,
        "ins",
        &(first_id + 1).to_string(),
        "Carol Diaz",
        DATE,
        "NEW",
    );
    let tail = revision(
        &dom,
        p,
        "ins",
        &(first_id + 2).to_string(),
        "Alice",
        OLD_DATE,
        " gamma",
    );
    assert_eq!(dom.elements(p, None), [head, inserted, tail]);
    let deletion = revision(
        &dom,
        p,
        "del",
        &first_id.to_string(),
        "Carol Diaz",
        DATE,
        "beta",
    );
    assert_eq!(dom.parent(deletion), Some(head));
    let mut expected_head = if word == 0 {
        range_labels(kind, "alpha ")
    } else {
        vec!["r:alpha ".into()]
    };
    if word == 1 {
        let mut marked = range_labels(kind, "beta");
        marked[1] = "del:beta".into();
        expected_head.extend(marked);
    } else {
        expected_head.push("del:beta".into());
    }
    assert_eq!(
        child_labels(&dom, head),
        expected_head,
        "{kind} around word {word}"
    );
    let expected_tail = if word == 2 {
        range_labels(kind, " gamma")
    } else {
        vec!["r: gamma".into()]
    };
    assert_eq!(child_labels(&dom, tail), expected_tail);
    assert_eq!(child_labels(&dom, inserted), ["r:NEW"]);
    settled(&out, "alpha NEW gamma", "");
}

#[test]
fn split_keeps_bookmark_around_words_before_anchor() {
    replacement_keeps_ranges("bookmark", 0);
}
#[test]
fn split_keeps_bookmark_around_anchor() {
    replacement_keeps_ranges("bookmark", 1);
}
#[test]
fn split_keeps_bookmark_around_words_after_anchor() {
    replacement_keeps_ranges("bookmark", 2);
}
#[test]
fn split_keeps_comment_around_words_before_anchor() {
    replacement_keeps_ranges("comment", 0);
}
#[test]
fn split_keeps_comment_around_anchor() {
    replacement_keeps_ranges("comment", 1);
}
#[test]
fn split_keeps_comment_around_words_after_anchor() {
    replacement_keeps_ranges("comment", 2);
}

fn insertion_keeps_range(kind: &str, before: bool) {
    let mut operation = json!({"kind":"insert","paragraph":"p0","text":"NEW"});
    operation[if before { "before" } else { "after" }] = json!("beta");
    let out = run(&range_source(kind, 1), "Carol Diaz", json!([operation]));
    let (dom, p) = tree(&out);
    let first_id = if kind == "comment" { 7 } else { 41 };
    let head = revision(
        &dom,
        p,
        "ins",
        "6",
        "Alice",
        OLD_DATE,
        if before { "alpha " } else { "alpha beta" },
    );
    let inserted = revision(
        &dom,
        p,
        "ins",
        &first_id.to_string(),
        "Carol Diaz",
        DATE,
        "NEW",
    );
    let tail = revision(
        &dom,
        p,
        "ins",
        &(first_id + 1).to_string(),
        "Alice",
        OLD_DATE,
        if before { "beta gamma" } else { " gamma" },
    );
    assert_eq!(dom.elements(p, None), [head, inserted, tail]);
    let mut expected_head = vec!["r:alpha ".into()];
    let mut expected_tail = vec!["r: gamma".into()];
    if before {
        expected_tail.splice(0..0, range_labels(kind, "beta"));
    } else {
        expected_head.extend(range_labels(kind, "beta"));
    }
    assert_eq!(
        child_labels(&dom, head),
        expected_head,
        "{kind} before={before}"
    );
    assert_eq!(child_labels(&dom, tail), expected_tail);
    assert_eq!(child_labels(&dom, inserted), ["r:NEW"]);
    settled(
        &out,
        if before {
            "alpha NEWbeta gamma"
        } else {
            "alpha betaNEW gamma"
        },
        "",
    );
}

#[test]
fn split_insertion_before_bookmark_keeps_new_words_outside() {
    insertion_keeps_range("bookmark", true);
}
#[test]
fn split_insertion_after_bookmark_keeps_new_words_outside() {
    insertion_keeps_range("bookmark", false);
}
#[test]
fn split_insertion_before_comment_keeps_new_words_outside() {
    insertion_keeps_range("comment", true);
}
#[test]
fn split_insertion_after_comment_keeps_new_words_outside() {
    insertion_keeps_range("comment", false);
}
// Codex P2 on #1143: a permission range split like a bookmark, not left
// open in the head around the new words.
#[test]
fn split_keeps_permission_around_words_before_anchor() {
    replacement_keeps_ranges("permission", 0);
}
#[test]
fn split_keeps_permission_around_anchor() {
    replacement_keeps_ranges("permission", 1);
}
#[test]
fn split_keeps_permission_around_words_after_anchor() {
    replacement_keeps_ranges("permission", 2);
}
#[test]
fn split_insertion_before_permission_keeps_new_words_outside() {
    insertion_keeps_range("permission", true);
}
#[test]
fn split_insertion_after_permission_keeps_new_words_outside() {
    insertion_keeps_range("permission", false);
}

/// The text of every run with italic on, in document order.
fn italic_text(bytes: &[u8]) -> String {
    let (dom, p) = tree(bytes);
    dom.descendants(p, Some(&W::r()))
        .into_iter()
        .filter(|&r| {
            dom.element(r, &W::r_pr())
                .is_some_and(|rpr| dom.element(rpr, &W::name("i")).is_some())
        })
        .map(|r| dom.value(r))
        .collect()
}

#[test]
fn a_formatted_replacement_formats_every_word_it_keeps() {
    // Codex P2 on #1143: the word split kept "beta" as it was and set the
    // format only on the words it inserted, so Accept All differed from the
    // clean copy.
    let out = apply_plan(
        &source("Alice"),
        &plan(
            "Carol Diaz",
            json!([{"kind":"replace","paragraph":"p0","find":"beta","replacement":"beta delta","format":{"italic":true}}]),
        ),
    )
    .unwrap();
    validate_output(&out.redline);
    let accepted = accept_changes(&out.redline, &ChangeFilter::default()).unwrap();
    assert_eq!(text(&accepted), "[alpha beta delta gamma]");
    assert_eq!(italic_text(&out.clean), "beta delta");
    assert_eq!(italic_text(&accepted), "beta delta");
}

#[test]
fn own_insertion_deleted_whole_leaves_no_empty_carrier_in_the_clean_copy() {
    // Codex P2 on #1143: the direct path cleared the text but kept the
    // `w:ins` around an empty run.
    let out = apply_plan(
        &source("Carol Diaz"),
        &plan("Carol Diaz", replace("alpha beta gamma", "")),
    )
    .unwrap();
    validate_output(&out.clean);
    for bytes in [&out.clean, &out.redline] {
        let (dom, p) = tree(bytes);
        assert!(
            dom.elements(p, Some(&W::ins())).is_empty(),
            "{}",
            dom.serialize_element(p)
        );
    }
}

#[test]
fn words_between_deleted_field_markers_stay_the_field_result() {
    // Codex P2 on #1143 read these words as plain text, since the view
    // skips the deleted markers. Word 16 does not: Accept All of this very
    // source drops "alpha beta gamma" with the field, so an edit there is
    // an edit of the field result, refused as one.
    let source = docx(&format!(
        r#"<w:p><w:del w:id="1" w:author="Bob" w:date="{OLD_DATE}"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText xml:space="preserve"> TITLE </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r></w:del><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t>alpha beta gamma</w:t></w:r></w:ins><w:del w:id="2" w:author="Bob" w:date="{OLD_DATE}"><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del></w:p>"#
    ));
    let e = apply_plan(&source, &plan("Carol Diaz", replace("beta", "delta"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(e.message.contains("the result of a field"), "{e}");
}

#[test]
fn word_fixture_split_keeps_existing_nested_deletion_in_order() {
    let (source_dom, source_p) = tree(WORD);
    let original = source_dom
        .descendants(source_p, Some(&W::del()))
        .into_iter()
        .find(|&node| source_dom.attribute(node, &W::id()) == Some("4"))
        .unwrap();
    let original_xml = source_dom.serialize_element(original);
    let out = run(WORD, "Carol Diaz", replace("pay", "remit"));
    let (dom, p) = tree(&out);
    let head = revision(
        &dom,
        p,
        "ins",
        "3",
        "Alma Sato Reyes",
        "2026-10-17T23:44:00Z",
        "agree to pay",
    );
    let deletion = revision(&dom, p, "del", "7", "Carol Diaz", DATE, "pay");
    assert_eq!(dom.parent(deletion), Some(head));
    assert_eq!(child_labels(&dom, head), ["r:agree to ", "del:pay"]);
    let inserted = revision(&dom, p, "ins", "8", "Carol Diaz", DATE, "remit");
    let tail = revision(
        &dom,
        p,
        "ins",
        "9",
        "Alma Sato Reyes",
        "2026-10-17T23:44:00Z",
        " the bankers",
    );
    assert_eq!(dom.parent(head), Some(p));
    assert_eq!(dom.parent(inserted), Some(p));
    assert_eq!(dom.parent(tail), Some(p));
    assert_eq!(dom.next_element(head), Some(inserted));
    assert_eq!(dom.next_element(inserted), Some(tail));
    let old = revision(
        &dom,
        p,
        "del",
        "4",
        "Bob",
        "2026-10-17T23:46:00Z",
        "bankers",
    );
    assert_eq!(dom.parent(old), Some(tail));
    assert_eq!(dom.serialize_element(old), original_xml);
    assert_eq!(child_labels(&dom, tail), ["r: the ", "del:bankers"]);
    assert_eq!(child_labels(&dom, inserted), ["r:remit"]);
    let accepted =
        text(&accept_changes(WORD, &ChangeFilter::default()).unwrap()).replace("pay", "remit");
    let rejected = text(&reject_changes(WORD, &ChangeFilter::default()).unwrap());
    settled(&out, &accepted, &rejected);
}

/// Word 16: Alma Sato Reyes inserted the paragraph and its mark (r0, r1).
const STEP1: &[u8] = include_bytes!("fixtures/re-dialect/step1.docx");
const ALMA: &str = "Alma Sato Reyes";

fn delete_p0(source: &[u8], author: &str) -> jubarte::edit::EditResult {
    let out = apply_plan(
        source,
        &plan(
            author,
            json!([{"kind":"delete_paragraph","paragraph":"p0"}]),
        ),
    );
    assert!(out.is_ok(), "the delete must land: {:?}", out.err());
    let out = out.unwrap();
    validate_output(&out.redline);
    validate_output(&out.clean);
    out
}

/// The `w:pPr/w:rPr` revisions of `p`, as `local:author`.
fn mark_labels(dom: &Dom, p: NodeId) -> Vec<String> {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .map(|rpr| {
            dom.elements(rpr, None)
                .into_iter()
                .filter(|&c| dom.name_is(c, &W::ins()) || dom.name_is(c, &W::del()))
                .map(|c| {
                    format!(
                        "{}:{}",
                        dom.name(c).unwrap().local_name(),
                        dom.attribute(c, &W::author()).unwrap()
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn delete_paragraph_under_keep_nests_in_their_insertion() {
    let out = delete_p0(STEP1, "Carol Diaz");
    let (dom, p) = tree(&out.redline);
    let theirs = revision(
        &dom,
        p,
        "ins",
        "1",
        ALMA,
        "2026-10-17T23:44:00Z",
        "The parties agree to pay the bankers a pro-rata amount by the end of the month.",
    );
    let del = dom.elements(theirs, Some(&W::del()));
    assert_eq!(del.len(), 1, "{}", dom.serialize_element(p));
    assert_eq!(dom.attribute(del[0], &W::author()), Some("Carol Diaz"));
    assert!(dom.descendants(theirs, Some(&W::t())).is_empty());
    assert!(
        dom.elements(p, Some(&W::r())).is_empty(),
        "no plain run is left: {}",
        dom.serialize_element(p)
    );
    assert_eq!(
        mark_labels(&dom, p),
        [format!("ins:{ALMA}"), "del:Carol Diaz".into()]
    );
    settled(&out.redline, "", "");
}

#[test]
fn delete_paragraph_of_own_insertion_removes_it_outright() {
    let out = delete_p0(STEP1, ALMA);
    let (dom, p) = tree(&out.redline);
    assert!(
        dom.descendants(p, Some(&W::ins()))
            .into_iter()
            .all(|n| dom.parent(n).is_some_and(|r| dom.name_is(r, &W::r_pr()))),
        "only the mark keeps its insertion: {}",
        dom.serialize_element(p)
    );
    assert!(dom.descendants(p, Some(&W::del_text())).is_empty());
    assert!(dom.descendants(p, Some(&W::t())).is_empty());
    assert_eq!(
        mark_labels(&dom, p),
        [format!("ins:{ALMA}"), format!("del:{ALMA}")]
    );
    settled(&out.redline, "", "");
}

#[test]
fn delete_paragraph_under_keep_leaves_their_deletion_and_nests_their_insertion() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">plain </w:t></w:r><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t xml:space="preserve">added </w:t></w:r></w:ins><w:del w:id="7" w:author="Bob" w:date="{OLD_DATE}"><w:r><w:delText xml:space="preserve">gone </w:delText></w:r></w:del><w:ins w:id="8" w:author="Carol Diaz" w:date="{OLD_DATE}"><w:r><w:t>mine</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>next</w:t></w:r></w:p>"#
    ));
    let out = delete_p0(&source, "Carol Diaz");
    let (dom, p) = tree(&out.redline);
    let labels: Vec<String> = dom
        .elements(p, None)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .map(|c| {
            let inner: Vec<String> = dom
                .elements(c, None)
                .into_iter()
                .map(|n| dom.name(n).unwrap().local_name().to_string())
                .collect();
            format!(
                "{}:{}:{}",
                dom.name(c).unwrap().local_name(),
                dom.attribute(c, &W::author()).unwrap_or("-"),
                inner.join("+")
            )
        })
        .collect();
    assert_eq!(
        labels,
        ["del:Carol Diaz:r", "ins:Alice:del", "del:Bob:r",],
        "{}",
        dom.serialize_element(p)
    );
    assert_eq!(mark_labels(&dom, p), ["del:Carol Diaz"]);
    settled(&out.redline, "next", "plain gone \nnext");
}

#[test]
fn delete_paragraph_keeps_their_formatting_change_inside_their_insertion() {
    // Alice inserted bold words and tracked the bolding (r7) inside r6.
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:rPr><w:b/><w:rPrChange w:id="7" w:author="Alice" w:date="{OLD_DATE}"><w:rPr/></w:rPrChange></w:rPr><w:t>bold words</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>next</w:t></w:r></w:p>"#
    ));
    let out = delete_p0(&source, "Carol Diaz");
    let (dom, p) = tree(&out.redline);
    let theirs = revision(&dom, p, "ins", "6", "Alice", OLD_DATE, "bold words");
    let ours = dom.elements(theirs, Some(&W::del()));
    assert_eq!(ours.len(), 1, "{}", dom.serialize_element(p));
    assert_eq!(dom.attribute(ours[0], &W::author()), Some("Carol Diaz"));
    let change = dom.descendants(ours[0], Some(&W::r_pr_change()));
    assert_eq!(change.len(), 1, "{}", dom.serialize_element(p));
    assert_eq!(dom.attribute(change[0], &W::author()), Some("Alice"));
    assert_eq!(dom.attribute(change[0], &W::id()), Some("7"));
    settled(&out.redline, "next", "\nnext");
    // Rejecting only ours gives the source back, bolding change and all.
    let ours_only = ChangeFilter {
        authors: Some(vec!["Carol Diaz".into()]),
        ..ChangeFilter::default()
    };
    let back = reject_changes(&out.redline, &ours_only).unwrap();
    assert_eq!(text(&back), "bold words\nnext");
    let (dom, p) = tree(&back);
    let theirs = revision(&dom, p, "ins", "6", "Alice", OLD_DATE, "bold words");
    assert!(dom.descendants(theirs, Some(&W::del())).is_empty());
    assert_eq!(
        dom.descendants(theirs, Some(&W::r_pr_change())).len(),
        1,
        "{}",
        dom.serialize_element(p)
    );
}

#[test]
fn delete_paragraph_holding_a_move_is_refused_with_the_move_named() {
    let source = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">alpha </w:t></w:r><w:moveTo w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t>beta</w:t></w:r></w:moveTo></w:p><w:p><w:r><w:t>next</w:t></w:r></w:p>"#
    ));
    let e = apply_plan(
        &source,
        &plan(
            "Carol Diaz",
            json!([{"kind":"delete_paragraph","paragraph":"p0"}]),
        ),
    )
    .unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert_eq!(
        e.message.lines().next().unwrap(),
        r#"under keep, a paragraph that holds a move cannot be deleted yet: the move is r6, "beta" moved here by "Alice"."#
    );
    assert!(
        e.message
            .contains(r#"keep the move (erases "Alice"'s move):"#),
        "{e}"
    );
}

#[test]
fn report_lists_the_split_pieces() {
    // Carol's replacement splits Alice's r6: its head keeps r6, its tail
    // after Carol's r7 (deletion) and r8 (insertion) takes r9.
    let out = apply_plan(
        &source("Alice"),
        &plan("Carol Diaz", replace("beta", "NEW")),
    )
    .unwrap();
    let pieces = &out.report.split_revisions;
    assert_eq!(pieces.len(), 1, "{pieces:?}");
    assert_eq!(pieces[0].from, "r6");
    assert_eq!(pieces[0].to, ["r6", "r9"]);
    let summary: Value =
        serde_json::from_str(out.report.to_jsonl().lines().last().unwrap()).unwrap();
    assert_eq!(summary["ev"], "summary");
    assert_eq!(
        summary["split_revisions"],
        json!([{"from":"r6","to":["r6","r9"]}])
    );
}

#[test]
fn an_edit_that_splits_nothing_reports_no_pieces() {
    let out = apply_plan(
        &source("Carol Diaz"),
        &plan("Carol Diaz", replace("beta", "NEW")),
    )
    .unwrap();
    assert!(out.report.split_revisions.is_empty());
    assert!(!out.report.to_jsonl().contains("split_revisions"));
}

#[test]
fn own_insertion_is_edited_in_place() {
    // Word 16's Step 1: Alma Sato Reyes typed the sentence as r1; she now
    // corrects her own word, which Word folds into r1.
    let out = run(STEP1, ALMA, replace("bankers", "insurers"));
    let (dom, p) = tree(&out);
    revision(
        &dom,
        p,
        "ins",
        "1",
        ALMA,
        "2026-10-17T23:44:00Z",
        "The parties agree to pay the insurers a pro-rata amount by the end of the month.",
    );
    assert_eq!(dom.elements(p, Some(&W::ins())).len(), 1);
    assert!(dom.descendants(p, Some(&W::del())).is_empty());
}

#[test]
fn the_change_listing_names_the_insertion_our_deletion_sits_in() {
    let out = run(&source("Alice"), "Carol Diaz", replace("beta", "NEW"));
    let changes = jubarte::changes::list_changes(&out).unwrap();
    let ours = changes
        .iter()
        .find(|c| c.author.as_deref() == Some("Carol Diaz") && c.text == "beta")
        .unwrap_or_else(|| panic!("{changes:#?}"));
    assert_eq!(ours.inside.as_deref(), Some("r6"), "{ours:#?}");
}

/// Each change of `bytes` by an author other than `except`, as
/// `kind target author: text`, with neighbouring pieces of one change kind
/// and author joined: a split insertion reads as the one it was.
fn others(bytes: &[u8], except: &str) -> Vec<String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for c in jubarte::changes::list_changes(bytes).unwrap() {
        let author = c.author.clone().unwrap_or_default();
        if author == except {
            continue;
        }
        let key = format!("{:?} {} {author}", c.kind, c.target);
        match out.last_mut() {
            Some((last, text)) if *last == key => text.push_str(&c.text),
            _ => out.push((key, c.text.clone())),
        }
    }
    out.into_iter().map(|(k, t)| format!("{k}: {t}")).collect()
}

/// Every edit of `kind` the step family's first paragraph takes once: each
/// word that occurs once replaced, deleted, or followed by an insertion.
fn step_edits(visible: &str, kind: &str) -> Vec<Value> {
    let mut edits = Vec::new();
    for word in visible.split_whitespace() {
        let word = word.trim_end_matches('.');
        if word.is_empty() || visible.matches(word).count() != 1 {
            continue;
        }
        edits.push(match kind {
            "replace" => json!({"kind":"replace","paragraph":"p0","find":word,"replacement":"XX"}),
            "delete" => json!({"kind":"replace","paragraph":"p0","find":word,"replacement":""}),
            _ => json!({"kind":"insert","paragraph":"p0","after":word,"text":" YY"}),
        });
    }
    edits
}

const STEP2_UNTRACKED: &[u8] = include_bytes!("fixtures/re-dialect/step2-untracked.docx");

/// Each edit of `kind` on `source` under `keep`: rejecting Carol gives the
/// source's text and leaves the other authors' changes as they were, and
/// accepting everything gives the same edit made on the accepted source.
fn step_family(name: &str, source: &[u8], kind: &str) {
    let carol = ChangeFilter {
        authors: Some(vec!["Carol Diaz".into()]),
        ..ChangeFilter::default()
    };
    let visible = paragraphs(source).unwrap()[0].text.clone();
    let plain = accept_changes(source, &ChangeFilter::default()).unwrap();
    let edits = step_edits(&visible, kind);
    assert!(edits.len() > 6, "{name}: only {} edits", edits.len());
    for edit in edits {
        let what = format!("{name}: {edit}");
        let out = apply_plan(source, &plan("Carol Diaz", json!([edit.clone()])))
            .unwrap_or_else(|e| panic!("{what}: {e}"));
        assert_word_valid_package(&out.redline);
        let rejected = reject_changes(&out.redline, &carol).unwrap();
        assert_eq!(text(&rejected), text(source), "{what}: reject ours");
        assert_eq!(
            others(&rejected, "Carol Diaz"),
            others(source, "Carol Diaz"),
            "{what}: their changes"
        );
        let mut direct = json!({"schema_version":1,"author":"Carol Diaz","date":DATE,
            "operations":[]});
        direct["operations"] = json!([edit]);
        let edited =
            apply_plan(&plain, &EditPlan::from_json(&direct.to_string()).unwrap()).unwrap();
        assert_eq!(
            text(&accept_changes(&out.redline, &ChangeFilter::default()).unwrap()),
            text(&edited.clean),
            "{what}: accept all"
        );
    }
}

#[test]
fn step1_replacements_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step1", STEP1, "replace");
}
#[test]
fn step1_deletions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step1", STEP1, "delete");
}
#[test]
fn step1_insertions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step1", STEP1, "insert");
}
#[test]
fn step2_tracked_replacements_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-tracked", WORD, "replace");
}
#[test]
fn step2_tracked_deletions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-tracked", WORD, "delete");
}
#[test]
fn step2_tracked_insertions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-tracked", WORD, "insert");
}
#[test]
fn step2_untracked_replacements_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-untracked", STEP2_UNTRACKED, "replace");
}
#[test]
fn step2_untracked_deletions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-untracked", STEP2_UNTRACKED, "delete");
}
#[test]
fn step2_untracked_insertions_reject_to_the_source_and_accept_to_the_plain_edit() {
    step_family("step2-untracked", STEP2_UNTRACKED, "insert");
}

/// One edit by Carol under keep on `source`: both outputs Word-valid, and
/// rejecting Carol gives the source's text with the other authors' changes
/// as they were.
fn rejects_to_the_source(what: &str, source: &[u8], edit: &Value) -> Vec<u8> {
    let carol = ChangeFilter {
        authors: Some(vec!["Carol Diaz".into()]),
        ..ChangeFilter::default()
    };
    let out = apply_plan(source, &plan("Carol Diaz", json!([edit])))
        .unwrap_or_else(|e| panic!("{what}: {e}"));
    validate_output(&out.redline);
    validate_output(&out.clean);
    let rejected = reject_changes(&out.redline, &carol).unwrap();
    assert_eq!(text(&rejected), text(source), "{what}: reject ours");
    assert_eq!(
        others(&rejected, "Carol Diaz"),
        others(source, "Carol Diaz"),
        "{what}: their changes"
    );
    out.redline
}

#[test]
fn the_other_text_operations_land_inside_their_insertion() {
    let theirs = source("Alice");
    for (what, edit) in [
        (
            "format_run",
            json!({"kind":"format_run","paragraph":"p0","find":"beta","format":{"italic":true}}),
        ),
        (
            "replace with a comment",
            json!({"kind":"replace","paragraph":"p0","find":"beta","replacement":"BETA","comment":"Why?"}),
        ),
        (
            "comment",
            json!({"kind":"comment","paragraph":"p0","find":"beta","text":"Why?"}),
        ),
        (
            "a range into it",
            json!({"kind":"replace","paragraph":"p0","find":"[alpha","replacement":"X"}),
        ),
        (
            "a range out of it",
            json!({"kind":"replace","paragraph":"p0","find":"gamma]","replacement":"X"}),
        ),
    ] {
        rejects_to_the_source(what, &theirs, &edit);
    }
}

#[test]
fn a_range_over_two_authors_insertions_lands_in_both() {
    let source = docx(&format!(
        r#"<w:p><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t xml:space="preserve">alpha </w:t></w:r></w:ins><w:ins w:id="7" w:author="Bob" w:date="{OLD_DATE}"><w:r><w:t>beta</w:t></w:r></w:ins></w:p>"#
    ));
    let redline = rejects_to_the_source(
        "two owners",
        &source,
        &json!({"kind":"replace","paragraph":"p0","find":"alpha beta","replacement":"X"}),
    );
    settled(&redline, "X", "");
}

#[test]
fn the_text_a_comment_covers_is_edited_inside_their_insertion() {
    let commented = apply_plan(
        &source("Alice"),
        &plan(
            "Ann",
            json!([{"kind":"comment","paragraph":"p0","find":"beta","text":"Check"}]),
        ),
    )
    .unwrap()
    .redline;
    rejects_to_the_source(
        "c0.text",
        &commented,
        &json!({"kind":"replace","paragraph":"c0.text","find":"beta","replacement":"BETA"}),
    );
}

#[test]
fn a_redaction_inside_their_insertion_removes_the_words_from_both_outputs() {
    let out = apply_plan(
        &source("Alice"),
        &plan(
            "Carol Diaz",
            json!([{"kind":"redact","paragraph":"p0","find":"beta"}]),
        ),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    validate_output(&out.redline);
    validate_output(&out.clean);
    for bytes in [&out.redline, &out.clean] {
        assert!(!text(bytes).contains("beta"), "{}", text(bytes));
    }
}

#[test]
fn a_hyperlink_names_itself_around_their_insertion() {
    let source = docx(&format!(
        r#"<w:p><w:bookmarkStart w:id="0" w:name="top"/><w:bookmarkEnd w:id="0"/><w:hyperlink w:anchor="top"><w:ins w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:t>link text</w:t></w:r></w:ins></w:hyperlink></w:p>"#
    ));
    let e = apply_plan(&source, &plan("Carol Diaz", replace("link", "LINK"))).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{e}");
    assert!(e.message.contains("inside a hyperlink"), "{e}");
    assert!(!e.message.contains("r6"), "{e}");
}
