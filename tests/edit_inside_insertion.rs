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
fn another_author_deletion_has_no_editor_insertion() {
    split_case("beta", "", "alpha ", "beta", " gamma");
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
#[test]
fn moves_and_deletions_stay_uneditable() {
    for (kind, t) in [("del", "delText"), ("moveFrom", "delText"), ("moveTo", "t")] {
        let s = docx(&format!(
            r#"<w:p><w:{kind} w:id="6" w:author="Alice" w:date="{OLD_DATE}"><w:r><w:{t}>beta</w:{t}></w:r></w:{kind}></w:p>"#
        ));
        let e = apply_plan(&s, &plan("Carol Diaz", replace("beta", "new"))).unwrap_err();
        assert_eq!(
            e.code,
            if kind == "moveTo" {
                "UNSUPPORTED_STRUCTURE"
            } else {
                "ANCHOR_NOT_FOUND"
            }
        );
        if kind == "moveTo" {
            assert_eq!(
                e.message,
                r#"the text sits inside r6, a tracked move by "Alice"; {"resolve_revisions":{"accept":{"ids":["r6"]}}} keeps the moved content at its destination; {"resolve_revisions":{"reject":{"ids":["r6"]}}} restores the moved content at its origin; refresh the edit against the resulting document, or comment on the revision"#
            );
        }
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
