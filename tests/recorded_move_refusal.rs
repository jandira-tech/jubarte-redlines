// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A move marker stored inside a formatting change belongs to that change.
//! Deleting the paragraph under keep must name the formatting change, and
//! the accept route that refusal prints must run.

mod common;

use common::docx::docx;
use jubarte::changes::list_changes;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use serde_json::json;

const DATE: &str = "2026-10-18T10:00:00Z";

fn source() -> Vec<u8> {
    docx(
        r#"<w:p><w:pPr><w:pPrChange w:id="7" w:author="B"><w:pPr><w:rPr><w:moveTo w:id="8" w:author="B"/></w:rPr></w:pPr></w:pPrChange></w:pPr><w:r><w:t>x</w:t></w:r></w:p><w:p><w:r><w:t>y</w:t></w:r></w:p>"#,
    )
}

fn plan(extra: serde_json::Value) -> EditPlan {
    let mut request = json!({
        "schema_version": 1,
        "author": "Carol Diaz",
        "date": DATE,
        "existing_revisions": "keep",
        "operations": []
    });
    for (key, value) in extra.as_object().expect("object") {
        request[key] = value.clone();
    }
    EditPlan::from_json(&request.to_string()).unwrap()
}

/// The refusal names `id` (and only it) and prints its accept route.
fn assert_refusal_names(source: &[u8], id: &str, not: &[&str]) {
    let err = apply_plan(
        source,
        &plan(json!({"operations":[{"kind":"delete_paragraph","paragraph":"p0"}]})),
    )
    .unwrap_err();
    let first = err.message.lines().next().unwrap_or("");
    assert_eq!(err.code, "UNSUPPORTED_STRUCTURE", "{err:?}");
    assert!(
        first.contains(&format!("{id},")) && first.contains("reformatted"),
        "the refusal did not name {id}: {first}"
    );
    for other in not {
        assert!(!first.contains(other), "the refusal named {other}: {first}");
    }
    assert!(
        err.message.contains(&format!(
            r#""resolve_revisions":{{"accept":{{"ids":["{id}"]}}}}"#
        )),
        "{}",
        err.message
    );
    assert!(
        list_changes(source).unwrap().iter().any(|c| c.id == id),
        "the named change is not one the listing offers"
    );
}

#[test]
fn a_recorded_move_refusal_names_the_formatting_change() {
    assert_refusal_names(&source(), "r7", &["r8"]);
}

#[test]
fn accepting_the_named_formatting_change_settles_it() {
    let settled = apply_plan(
        &source(),
        &plan(json!({"resolve_revisions":{"accept":{"ids":["r7"]}}})),
    )
    .expect("the refusal's accept of the formatting change runs");
    let text: Vec<_> = paragraphs(&settled.redline)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(text, ["x", "y"]);
    assert!(
        list_changes(&settled.redline)
            .unwrap()
            .iter()
            .all(|c| c.id != "r7")
    );
}

/// A formatting history recorded inside another is not listed on its own
/// (`in_recorded_properties`): the refusal names the outer change.
#[test]
fn a_move_in_nested_formatting_history_names_the_outer_change() {
    let source = docx(
        r#"<w:p><w:pPr><w:pPrChange w:id="7" w:author="B"><w:pPr><w:rPr><w:rPrChange w:id="9" w:author="B"><w:rPr><w:moveTo w:id="8" w:author="B"/></w:rPr></w:rPrChange></w:rPr></w:pPr></w:pPrChange></w:pPr><w:r><w:t>x</w:t></w:r></w:p><w:p><w:r><w:t>y</w:t></w:r></w:p>"#,
    );
    assert_refusal_names(&source, "r7", &["r8", "r9"]);
}
