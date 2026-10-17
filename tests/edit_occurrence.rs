// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A repeated anchor is editable by `occurrence` (1-based); without it the
//! refusal says how many times the anchor occurs and how to pick one.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan, v2_candidates, v2_select, v2_select_all};
use jubarte::inspect::paragraphs;

fn source() -> Vec<u8> {
    docx(&para("fee fee fee"))
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","operations":{operations}}}"#
    ))
    .unwrap()
}

#[test]
fn the_second_occurrence_is_replaced() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"p0","find":"fee","occurrence":2,"replacement":"cost"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee cost fee");
    assert_eq!(out.report.operations[0].matches, 3);
}

#[test]
fn the_last_occurrence_is_deleted() {
    let plan = plan(r#"[{"kind":"delete","paragraph":"p0","find":" fee","occurrence":2}]"#);
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee fee");
    assert_eq!(out.report.operations[0].matches, 2);
}

#[test]
fn insert_after_and_before_take_an_occurrence() {
    let plan = plan(
        r#"[{"kind":"insert","paragraph":"p0","after":"fee","occurrence":1,"text":"!"},
            {"kind":"insert","paragraph":"p0","before":"fee","occurrence":3,"text":"?"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "fee! fee ?fee");
}

#[test]
fn a_comment_anchors_to_an_occurrence() {
    let plan =
        plan(r#"[{"kind":"comment","paragraph":"p0","find":"fee","occurrence":3,"text":"why?"}]"#);
    let out = apply_plan(&source(), &plan).unwrap();
    let op = &out.report.operations[0];
    assert_eq!(op.status, "ok");
    assert_eq!(op.matches, 3);
    assert_eq!(op.context.as_deref(), Some("fee fee {#fee}"));
}

#[test]
fn a_unique_anchor_accepts_occurrence_one() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"p0","find":"fee fee fee","occurrence":1,"replacement":"x"}]"#,
    );
    let out = apply_plan(&source(), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "x");
    assert_eq!(out.report.operations[0].matches, 1);
}

#[test]
fn an_out_of_range_occurrence_names_the_range() {
    let plan = plan(r#"[{"kind":"delete","paragraph":"p0","find":"fee","occurrence":4}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(
        e.message.contains("occurs 3 times") && e.message.contains("occurrence 1..=3"),
        "{}",
        e.message
    );
    assert_eq!(e.outcomes[0].matches, 3);
}

#[test]
fn an_occurrence_on_a_missing_anchor_is_still_not_found() {
    let plan = plan(r#"[{"kind":"delete","paragraph":"p0","find":"fum","occurrence":1}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert_eq!(e.outcomes[0].matches, 0);
}

#[test]
fn without_occurrence_the_refusal_says_how_to_fix_it() {
    let plan = plan(r#"[{"kind":"insert","paragraph":"p0","after":"fee","text":"!"}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(
        e.message.contains("occurs 3 times") && e.message.contains("set \"occurrence\" to 1..=3"),
        "{}",
        e.message
    );
}

#[test]
fn occurrence_zero_is_an_invalid_edit() {
    let plan =
        plan(r#"[{"kind":"comment","paragraph":"p0","find":"fee","occurrence":0,"text":"?"}]"#);
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "INVALID_EDIT");
    assert!(e.message.contains("1-based"), "{}", e.message);
}

#[test]
fn occurrence_round_trips_through_the_plan_json() {
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"p0","find":"fee","occurrence":2,"replacement":"cost"},
            {"kind":"delete","paragraph":"p0","find":"fee"}]"#,
    );
    let json: serde_json::Value = serde_json::from_str(&plan.to_json()).unwrap();
    assert_eq!(json["operations"][0]["occurrence"], 2);
    assert!(json["operations"][1].get("occurrence").is_none());
    assert_eq!(EditPlan::from_json(&plan.to_json()).unwrap(), plan);
}

#[test]
fn occurrence_without_an_anchor_is_an_invalid_edit() {
    for op in [
        r#"{"kind":"insert","paragraph":"p0","position":"end","occurrence":2,"text":"!"}"#,
        r#"{"kind":"comment","paragraph":"p0","occurrence":2,"text":"?"}"#,
        r#"{"kind":"comment","paragraph":"p0","through":"p0","occurrence":1,"text":"?"}"#,
    ] {
        let e = apply_plan(&source(), &plan(&format!("[{op}]"))).unwrap_err();
        assert_eq!(e.code, "INVALID_EDIT", "{op}");
        assert!(e.message.contains("occurrence"), "{op}: {}", e.message);
    }
}

/// The v2 rules for a literal anchor with several, overlapping candidates
/// (jubarte-edit-implementation-plan.md §5). v1 plan matching keeps its
/// own versioned behavior; the v2 functions are exact-text only.
#[test]
fn overlapping_literal_candidates_are_counted_and_batch_refused() {
    // Every exact start position counts, overlapping ones included.
    assert_eq!(v2_candidates("aaa", "aa"), [(0, 2), (1, 3)]);

    // Unqualified: refused with the candidate count, no range chosen.
    let e = v2_select("aaa", "aa", None).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(
        e.message.contains("occurs 2 times") && e.message.contains("set \"occurrence\" to 1..=2"),
        "{}",
        e.message
    );

    // An occurrence selects the corresponding range; replacing it with "X"
    // in the paragraph text gives the planned result.
    let replaced = |occurrence: usize| {
        let (start, end) = v2_select("aaa", "aa", Some(occurrence)).unwrap();
        format!("{}X{}", &"aaa"[..start], &"aaa"[end..])
    };
    assert_eq!(replaced(1), "Xa");
    assert_eq!(replaced(2), "aX");

    // all-matches at the complete count still refuses: the two ranges
    // overlap, and no disjoint subset is applied.
    let e = v2_select_all("aaa", "aa", 2).unwrap_err();
    assert_eq!(e.code, "OVERLAPPING_EDITS");
    assert!(e.message.contains("occurs 2 times"), "{}", e.message);

    // Non-overlapping candidates at the complete count all match.
    assert_eq!(
        v2_select_all("a-a-a", "a", 3).unwrap(),
        [(0, 1), (2, 3), (4, 5)]
    );
    // A wrong expected count refuses with the real candidate count.
    let e = v2_select_all("a-a-a", "a", 2).unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR");
    assert!(e.message.contains("occurs 3 times"), "{}", e.message);

    // v1 plan matching is unchanged: it counts overlapping candidates the
    // same way and an occurrence still picks one under its own rules.
    let plan = plan(
        r#"[{"kind":"replace","paragraph":"p0","find":"aa","occurrence":2,"replacement":"X"}]"#,
    );
    let out = apply_plan(&docx(&para("aaa")), &plan).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "aX");
    assert_eq!(out.report.operations[0].matches, 2);
}
