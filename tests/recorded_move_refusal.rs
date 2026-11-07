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

#[allow(clippy::needless_pass_by_value)]
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

#[test]
fn a_recorded_move_refusal_names_the_formatting_change() {
    let source = source();
    let err = apply_plan(
        &source,
        &plan(json!({"operations":[{"kind":"delete_paragraph","paragraph":"p0"}]})),
    )
    .unwrap_err();
    let first = err.message.lines().next().unwrap_or("");
    assert!(
        err.code == "UNSUPPORTED_STRUCTURE"
            && first.contains("r7,")
            && first.contains("reformatted")
            && !first.contains("r8")
            && err
                .message
                .contains(r#""resolve_revisions":{"accept":{"ids":["r7"]}}"#),
        "the refusal named the recorded move instead of the formatting change: {first}"
    );

    let settled = apply_plan(
        &source,
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
