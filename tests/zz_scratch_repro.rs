// Scratch reproductions for the PR #1689 review. NOT COMMITTED, NOT PUSHED.
//
// Run with:
//   nice -n 10 cargo nextest run --test zz_scratch_repro --no-capture

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","operations":[{operations}]}}"#
    ))
    .unwrap()
}

fn plan_keep(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Bob","date":"2026-10-17T12:00:00Z","existing_revisions":"keep","operations":[{operations}]}}"#
    ))
    .unwrap()
}

/// FINDING 1a: two different refusals (same code, same part) get one name.
///
/// p0 holds a marked run of id 9, p1 one of id 7. op-1 splits the run of
/// id 9 (the redline then holds that id twice), op-2 splits the run of id
/// 7. The plan's refusal is the *first* introduced duplicate the checker
/// reports, id 7 - op-2's - but the attribution names op-1.
#[test]
fn f1_two_refusals_one_name() {
    let source = docx(
        r#"<w:p><w:r><w:rPr><w:ins w:id="9" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p><w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>efgh</w:t></w:r></w:p>"#,
    );
    let p = plan_keep(
        r#"{"kind":"replace","paragraph":"p0","find":"bc","replacement":"X"},{"kind":"replace","paragraph":"p1","find":"fg","replacement":"Y"}"#,
    );
    let e: EditError = prepare(&source, &p, &DEFAULT).unwrap_err();
    // The refusal is about op-2's duplicate (id 7) ...
    assert!(e.message.contains("w:id 7"), "{e:?}");
    // ... and the blame is op-1, which duplicates id 9.
    assert_eq!(e.operation.as_deref(), Some("op-1"), "{e:?}");
    println!("F1a: {e}");
}

/// FINDING 1a': dropping the named operation does not clear the refusal:
/// the plan without op-1 is refused the same way.
#[test]
fn f1b_the_named_operation_is_not_the_cause() {
    let source = docx(
        r#"<w:p><w:r><w:rPr><w:ins w:id="9" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p><w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>efgh</w:t></w:r></w:p>"#,
    );
    // The plan the message tells the agent to write: op-2 alone.
    let p = plan_keep(r#"{"kind":"replace","paragraph":"p1","find":"fg","replacement":"Y"}"#);
    let e = prepare(&source, &p, &DEFAULT).unwrap_err();
    assert!(e.message.contains("w:id 7"), "{e:?}");
    println!("F1b: the plan without the named op-1 is still refused: {e}");
}

/// FINDING 1b: the same with the comparer route (BOOKMARK_CROSSES_REVISION).
///
/// p0 holds bookmark ClauseA, p1 holds ClauseB. op-1 edits p1's bookmark,
/// op-2 edits p0's, op-3 (the page setup) routes the plan through the
/// comparer. The refusal names ClauseA - op-2's - but the message names
/// op-1 (p1) as the operation it needs.
#[test]
fn f1c_bookmark_crossing_names_the_wrong_edit() {
    let source = docx(
        r#"<w:p><w:bookmarkStart w:id="7" w:name="ClauseA"/><w:r><w:t>alpha beta</w:t></w:r><w:bookmarkEnd w:id="7"/></w:p><w:p><w:bookmarkStart w:id="8" w:name="ClauseB"/><w:r><w:t>gamma delta</w:t></w:r><w:bookmarkEnd w:id="8"/></w:p>"#,
    );
    let p = plan(
        r#"{"kind":"replace","paragraph":"p1","find":"delta","replacement":"DELTA"},{"kind":"replace","paragraph":"p0","find":"beta","replacement":"BETA"},{"kind":"page_setup","section":"all","orientation":"landscape"}"#,
    );
    let e = prepare(&source, &p, &DEFAULT).unwrap_err();
    assert!(e.message.contains("ClauseA"), "{e:?}");
    assert!(
        e.message.contains("together with op-1 (replace on p1)"),
        "{e:?}"
    );
    println!("F1c: {e}");
}

/// FINDING 1c: the PlanV2 pointer names the wrong operation.
#[test]
fn f1d_v2_pointer_names_the_wrong_operation() {
    use jubarte::edit::for_edit::{ForEditRequest, for_edit};
    let source = docx(
        r#"<w:p><w:r><w:rPr><w:ins w:id="9" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p><w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>efgh</w:t></w:r></w:p>"#,
    );
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
            {"kind": "replace", "target": target("p0"), "find": "bc", "text": "X"},
            {"kind": "replace", "target": target("p1"), "find": "fg", "text": "Y"},
        ],
    });
    let plan = jubarte::edit::v2::PlanV2::from_json(&plan.to_string()).unwrap();
    let e = jubarte::edit::prepare_v2(&source, &plan, &DEFAULT).unwrap_err();
    // The refusal is op-1's (id 7) ...
    assert!(e.message.contains("w:id 7"), "{e:?}");
    // ... and the pointer names op-0.
    assert_eq!(e.operation.as_deref(), Some("/operations/0"), "{e:?}");
    println!("F1d: {e}");
}

/// FINDING 2: `--skip-failed` records the wrong reason for a skipped
/// operation: op-1 is skipped for a defect it did not cause.
#[test]
fn f2_skip_failed_records_the_wrong_reason() {
    let source = docx(
        r#"<w:p><w:r><w:rPr><w:ins w:id="9" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>abcd</w:t></w:r></w:p><w:p><w:r><w:rPr><w:ins w:id="7" w:author="Ann" w:date="2026-09-01T00:00:00Z"/></w:rPr><w:t>efgh</w:t></w:r></w:p><w:p><w:r><w:t>plain text</w:t></w:r></w:p>"#,
    );
    let p = plan_keep(
        r#"{"kind":"replace","paragraph":"p0","find":"bc","replacement":"X"},{"kind":"replace","paragraph":"p2","find":"plain","replacement":"PLAIN"},{"kind":"replace","paragraph":"p1","find":"fg","replacement":"Y"}"#,
    );
    let options = PrepareOptions {
        skip_failed: true,
        ..DEFAULT
    };
    let prepared = prepare(&source, &p, &options).expect("the plan publishes");
    assert_eq!(prepared.skipped.len(), 2, "{:?}", prepared.skipped);
    let first = &prepared.skipped[0];
    assert_eq!(first.index, 0, "{first:?}");
    // op-1 duplicates the id 9; the reason it is given names the id 7.
    assert!(first.message.contains("w:id 7"), "{first:?}");
    println!("F2: op-1 skipped with: {}", first.message);
}
