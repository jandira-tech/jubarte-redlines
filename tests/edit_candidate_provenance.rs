// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A refused candidate (`CANDIDATE_INVALID`) names the operation that
//! causes the refusal, and the operations it needs, when the plan's own
//! operations cause it (#1681, task 6 of #1656): the agent learns which
//! operation to restate, and `--skip-failed` leaves that one out.

mod common;

use common::docx::docx;
use jubarte::edit::{EditError, EditPlan, PlanSyntax, PrepareOptions, PrimaryRole, prepare};

const DEFAULT: PrepareOptions = PrepareOptions {
    require_change: false,
    exact_text: false,
    role: PrimaryRole::Clean,
    syntax: PlanSyntax::V1,
    skip_failed: false,
};

/// A run whose own `w:rPr` holds a `w:ins` (no schema allows it there):
/// splitting the run copies the record's id, so the candidate repeats it.
/// A second, plain paragraph follows.
fn marked_run() -> Vec<u8> {
    docx(
        r#"<w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p><w:p><w:r><w:t>next words</w:t></w:r></w:p>"#,
    )
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{operations}]}}"#
    ))
    .unwrap()
}

/// Splits the marked run: the refused operation.
const SPLIT: &str = r#"{"kind":"replace","paragraph":"p0","find":"bc","replacement":"X"}"#;
/// A plain replacement in the other paragraph: publishes on its own.
const HARMLESS: &str = r#"{"kind":"replace","paragraph":"p1","find":"next","replacement":"later"}"#;

fn refused(source: &[u8], operations: &str) -> EditError {
    let error = prepare(source, &plan(operations), &DEFAULT).unwrap_err();
    assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
    error
}

#[test]
fn the_operation_that_repeats_an_id_is_named() {
    let error = refused(&marked_run(), SPLIT);
    assert_eq!(error.operation.as_deref(), Some("op-1"), "{error:?}");
    assert!(
        error.message.contains("w:ins on a mark w:id 7")
            && error.message.contains("op-1 (replace on p0)"),
        "{error:?}"
    );
}

#[test]
fn an_operation_before_the_cause_takes_no_blame() {
    let error = refused(&marked_run(), &format!("{HARMLESS},{SPLIT}"));
    assert_eq!(error.operation.as_deref(), Some("op-2"), "{error:?}");
    assert!(
        error.message.contains("op-2 (replace on p0)") && !error.message.contains("op-1"),
        "{error:?}"
    );
    // After the cause, too.
    let error = refused(&marked_run(), &format!("{SPLIT},{HARMLESS}"));
    assert_eq!(error.operation.as_deref(), Some("op-1"), "{error:?}");
    assert!(!error.message.contains("op-2"), "{error:?}");
}

#[test]
fn a_plans_own_operation_ids_name_it() {
    let error = refused(
        &marked_run(),
        &format!(
            r#"{},{}"#,
            HARMLESS.replacen('{', r#"{"id":"keep-me","#, 1),
            SPLIT.replacen('{', r#"{"id":"split-it","#, 1)
        ),
    );
    assert_eq!(error.operation.as_deref(), Some("split-it"), "{error:?}");
    assert!(
        error.message.contains("split-it (replace on p0)"),
        "{error:?}"
    );
}

/// Neither operation alone is refused: the replacement alone is emitted
/// and keeps the bookmark's end outside it; the page setup sends the plan
/// through the comparer, which takes the end into the insertion. The page
/// setup brings the refusal in, so it is the operation named; the
/// replacement it needs is named beside it.
#[test]
fn operations_refused_together_are_named_together() {
    let source = docx(
        r#"<w:p><w:bookmarkStart w:id="7" w:name="Clause"/><w:r><w:t>alpha beta</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p>"#,
    );
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Ed Itor","date":"2026-10-17T12:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"beta","replacement":"BETA"},{"kind":"page_setup","section":"all","orientation":"landscape"}]}"#,
    )
    .unwrap();
    let error = prepare(&source, &plan, &DEFAULT).unwrap_err();
    assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
    assert!(
        error.message.contains("BOOKMARK_CROSSES_REVISION"),
        "{error:?}"
    );
    assert_eq!(error.operation.as_deref(), Some("op-2"), "{error:?}");
    assert!(
        error.message.contains("op-2 (page_setup)")
            && error.message.contains("together with op-1 (replace on p0)"),
        "{error:?}"
    );
}

/// `--skip-failed` leaves the named operation out and publishes the rest.
#[test]
fn skip_failed_leaves_the_named_operation_out() {
    let options = PrepareOptions {
        skip_failed: true,
        ..DEFAULT
    };
    let prepared = prepare(
        &marked_run(),
        &plan(&format!("{HARMLESS},{SPLIT}")),
        &options,
    )
    .expect("the harmless operation publishes");
    assert_eq!(prepared.skipped.len(), 1, "{:?}", prepared.skipped);
    let skipped = &prepared.skipped[0];
    assert_eq!(skipped.index, 1, "{skipped:?}");
    assert_eq!(skipped.code, "CANDIDATE_INVALID", "{skipped:?}");
    let texts: Vec<String> = jubarte::inspect::paragraphs(&prepared.primary)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts, ["abcd", "later words"]);
}

/// A PlanV2 names the operation by its pointer: its operations apply in
/// order, so the one that brings the refusal in is named alone.
#[test]
fn a_v2_plan_names_the_operation_by_its_pointer() {
    use jubarte::edit::for_edit::{ForEditRequest, for_edit};
    let source = marked_run();
    let view = for_edit(
        &source,
        &ForEditRequest {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    let target = |short: &str| {
        let row = view
            .paragraphs
            .iter()
            .find(|row| row.target.short_id == short)
            .unwrap_or_else(|| panic!("no for-edit row {short}"));
        serde_json::json!({"short_id": row.target.short_id, "long_id": row.target.long_id})
    };
    let plan = serde_json::json!({
        "schema_version": 2,
        "source": {"sha256": jubarte::inspect::source_sha256(&source)},
        "policy": {"track": "on", "existing_revisions": "keep", "text_view": "final"},
        "resolve_revisions": {"accept": [], "reject": []},
        "author": "Bob",
        "date": "2026-10-17T12:00:00Z",
        "scope": {"kind": "all"},
        "operations": [
            {"kind": "replace", "target": target("p1"), "find": "next", "text": "later"},
            {"kind": "replace", "target": target("p0"), "find": "bc", "text": "X"},
        ],
    });
    let plan = jubarte::edit::v2::PlanV2::from_json(&plan.to_string()).unwrap();
    let error = jubarte::edit::prepare_v2(&source, &plan, &DEFAULT).unwrap_err();
    assert_eq!(error.code, "CANDIDATE_INVALID", "{error:?}");
    assert_eq!(
        error.operation.as_deref(),
        Some("/operations/1"),
        "{error:?}"
    );
    assert!(
        error.message.contains("; /operations/1 introduces it")
            && !error.message.contains("/operations/0"),
        "{error:?}"
    );
}
