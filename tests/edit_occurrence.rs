// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A repeated anchor is editable by `occurrence` (1-based); without it the
//! refusal says how many times the anchor occurs and how to pick one.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{
    AnchorRefusal, EditPlan, apply_plan, v2_candidates, v2_resolve, v2_resolve_all, v2_select,
    v2_select_all,
};
use jubarte::inspect::{paragraphs, source_sha256};
use jubarte::result::CommitState;

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

/// V2-11 (#841), edit plan §5: a wrong target and a repeated anchor return
/// bounded, actionable context. A wrong p15 gets p15's own text, bounded,
/// plus the paragraphs that hold the anchor elsewhere as read suggestions
/// only — never an automatic retarget — and a repeated anchor reports the
/// complete count with the anchor shown where it first matched.
#[test]
fn wrong_target_and_repeated_anchor_return_context() {
    let mut body = String::new();
    for i in 0..21 {
        let text = match i {
            15 => "Payment is due within thirty days; interest accrues after thirty days.",
            20 => "The aggregate liability cap of the Provider applies.",
            _ => "Filler paragraph to position the clauses.",
        };
        body.push_str(&para(text));
    }
    let source = docx(&body);
    let text: Vec<String> = paragraphs(&source)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    let p15 = text[15].clone();
    let digest = source_sha256(&source);

    // A wrong target: the anchor belongs to p20 but names p15.
    let refused = v2_resolve(
        &source,
        &p15,
        &AnchorRefusal {
            operation: Some("/operations/0"),
            target: "p15",
            anchor: "liability cap",
            path: Some("contract.docx"),
        },
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "ANCHOR_NOT_FOUND");
    assert_eq!(refused.operation.as_deref(), Some("/operations/0"));
    assert_eq!(refused.source_digest.as_deref(), Some(digest.as_str()));
    assert_eq!(refused.target.as_deref(), Some("p15"));
    assert_eq!(refused.matches, Some(0));
    assert_eq!(refused.retryable, Some(false));
    assert_eq!(refused.commit_state, CommitState::NotCommitted);
    // Usable p15 context: the actual target text, bounded, nothing matched.
    let context = refused.context.as_ref().unwrap();
    assert_eq!(context.matched, "");
    assert_eq!(context.after, "");
    assert!(context.before.starts_with("Payment is due"), "{context:?}");
    assert_eq!(context.before.chars().count(), 20, "{context:?}");
    assert!(context.truncated);
    // The anchor occurs elsewhere: named in the message, offered as reads.
    assert!(
        refused.message.contains("elsewhere") && refused.message.contains("p20"),
        "{}",
        refused.message
    );
    let read_p15 = refused
        .next_actions
        .iter()
        .find(|a| a.argv == ["jubarte", "read", "contract.docx", "-p", "p15"])
        .unwrap();
    assert!(
        read_p15
            .purpose
            .as_deref()
            .unwrap_or_default()
            .contains("p15")
    );
    let suggestion = refused
        .next_actions
        .iter()
        .find(|a| a.argv == ["jubarte", "read", "contract.docx", "-p", "p20"])
        .unwrap();
    assert!(
        suggestion
            .purpose
            .as_deref()
            .unwrap_or_default()
            .contains("suggestion"),
        "{:?}",
        suggestion.purpose
    );
    // Every continuation is a read of the bound source; no argv carries
    // document text and none of them edits anything.
    for action in &refused.next_actions {
        assert_eq!(action.argv[0], "jubarte");
        assert_eq!(action.argv[1], "read");
        for arg in &action.argv {
            assert!(
                !arg.contains("liability") && !arg.contains("Payment"),
                "{arg}"
            );
        }
    }

    // A repeated anchor in the target: the complete count, the anchor shown
    // where it first matched, and the occurrence that picks one.
    let refused = v2_resolve(
        &source,
        &p15,
        &AnchorRefusal {
            operation: Some("/operations/1"),
            target: "p15",
            anchor: "thirty",
            path: Some("contract.docx"),
        },
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "AMBIGUOUS_ANCHOR");
    assert_eq!(refused.target.as_deref(), Some("p15"));
    assert_eq!(refused.matches, Some(2));
    let context = refused.context.as_ref().unwrap();
    assert_eq!(context.matched, "thirty");
    assert!(context.before.ends_with("within "), "{context:?}");
    assert_eq!(context.before.chars().count(), 20, "{context:?}");
    assert!(context.after.starts_with(" days"), "{context:?}");
    assert_eq!(context.after.chars().count(), 20, "{context:?}");
    assert!(context.truncated);
    assert!(
        refused.message.contains("occurs 2 times")
            && refused.message.contains(r#"set "occurrence" to 1..=2"#),
        "{}",
        refused.message
    );
    assert_eq!(
        v2_resolve(
            &source,
            &p15,
            &AnchorRefusal {
                operation: None,
                target: "p15",
                anchor: "thirty",
                path: Some("contract.docx")
            },
            Some(2),
        )
        .unwrap(),
        (58, 64)
    );
    let refused = v2_resolve(
        &source,
        &p15,
        &AnchorRefusal {
            operation: None,
            target: "p15",
            anchor: "thirty",
            path: Some("contract.docx"),
        },
        Some(3),
    )
    .unwrap_err();
    assert_eq!(refused.code, "AMBIGUOUS_ANCHOR");
    assert_eq!(refused.matches, Some(2));
    assert!(
        refused
            .message
            .contains("occurrence 3 is outside occurrence 1..=2"),
        "{}",
        refused.message
    );

    // p20 is not edited: the engine refuses the wrong target, and the
    // refusal only suggests. Renaming the target is the caller's edit.
    let wrong = plan(
        r#"[{"kind":"replace","paragraph":"p15","find":"liability cap","replacement":"cap"}]"#,
    );
    assert_eq!(
        apply_plan(&source, &wrong).unwrap_err().code,
        "ANCHOR_NOT_FOUND"
    );
}

/// V2-11 (#841): the refusal of a target that does not exist, of an invalid
/// anchor, and the bounded all-matches refusals carry the same structured
/// facts, and a pathless refusal still binds the digest and says what its
/// continuation is missing.
#[test]
fn anchor_refusals_name_missing_targets_and_keep_argv_structured() {
    let source = docx(&format!("{}{}", para("one two"), para("three")));
    let working = paragraphs(&source).unwrap()[0].text.clone();

    let refused = v2_resolve(
        &source,
        &working,
        &AnchorRefusal {
            operation: Some("op-7"),
            target: "p99",
            anchor: "two",
            path: Some("small.docx"),
        },
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "ANCHOR_NOT_FOUND");
    assert_eq!(refused.operation.as_deref(), Some("op-7"));
    assert_eq!(refused.target.as_deref(), Some("p99"));
    assert_eq!(refused.matches, Some(0));
    assert!(refused.context.is_none());
    assert!(
        refused.message.contains("no paragraph p99") && refused.message.contains("2 paragraphs"),
        "{}",
        refused.message
    );
    assert_eq!(refused.next_actions.len(), 1);
    assert_eq!(
        refused.next_actions[0].argv,
        ["jubarte", "read", "small.docx", "--head", "20"]
    );

    let refused = v2_resolve(&source, &working, &AnchorRefusal::new("p0", ""), None).unwrap_err();
    assert_eq!(refused.code, "INVALID_EDIT");
    assert!(refused.message.contains("nonempty"), "{}", refused.message);
    assert_eq!(refused.matches, None);
    assert!(refused.context.is_none());

    let refused = v2_resolve_all(
        &source,
        &working,
        &AnchorRefusal {
            operation: None,
            target: "p0",
            anchor: "one",
            path: Some("small.docx"),
        },
        2,
    )
    .unwrap_err();
    assert_eq!(refused.code, "AMBIGUOUS_ANCHOR");
    assert_eq!(refused.matches, Some(1));
    assert!(
        refused
            .message
            .contains("occurs 1 times in p0, not the expected 2"),
        "{}",
        refused.message
    );
    assert_eq!(refused.context.as_ref().unwrap().matched, "one");
    let refused = v2_resolve_all(
        &source,
        "eee",
        &AnchorRefusal {
            operation: None,
            target: "p0",
            anchor: "ee",
            path: None,
        },
        2,
    )
    .unwrap_err();
    assert_eq!(refused.code, "OVERLAPPING_EDITS");
    assert_eq!(refused.matches, Some(2));
    assert_eq!(
        refused.source_digest.as_deref(),
        Some(source_sha256(&source).as_str())
    );
    let action = &refused.next_actions[0];
    assert_eq!(action.argv, ["jubarte", "read", "-p", "p0"]);
    assert!(
        action
            .prerequisites
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|p| p.contains("source file path")),
        "{:?}",
        action.prerequisites
    );
}

/// V2-11 (#841): candidate matches elsewhere are bounded — a handful of
/// paragraphs in the message and as read continuations, with the count of
/// what the cap cut — and they are all of it: suggestions, never a
/// selection.
#[test]
fn elsewhere_suggestions_are_bounded_and_read_only() {
    let mut body = String::new();
    for _ in 0..8 {
        body.push_str(&para("the needle appears here"));
    }
    body.push_str(&para("nothing"));
    let source = docx(&body);
    let working = paragraphs(&source).unwrap()[8].text.clone();

    let refused = v2_resolve(
        &source,
        &working,
        &AnchorRefusal {
            operation: None,
            target: "p8",
            anchor: "needle",
            path: Some("many.docx"),
        },
        None,
    )
    .unwrap_err();
    assert_eq!(refused.code, "ANCHOR_NOT_FOUND");
    assert_eq!(refused.matches, Some(0));
    for id in ["p0", "p1", "p2", "p3", "p4"] {
        assert!(
            refused.message.contains(&format!("{id} (1)")),
            "{}",
            refused.message
        );
    }
    assert!(
        refused.message.contains("+3 more paragraphs") && !refused.message.contains("p5 ("),
        "{}",
        refused.message
    );
    // The target read plus the five suggested reads, nothing else.
    assert_eq!(refused.next_actions.len(), 6);
    assert_eq!(
        refused.next_actions[0].argv,
        ["jubarte", "read", "many.docx", "-p", "p8"]
    );
    for action in refused.next_actions.iter().skip(1) {
        assert_eq!(action.argv[0], "jubarte");
        assert_eq!(action.argv[1], "read");
        assert_ne!(action.argv[4], "p8");
        assert!(
            action
                .purpose
                .as_deref()
                .unwrap_or_default()
                .contains("suggestion"),
            "{:?}",
            action.purpose
        );
    }
}

/// V2-11 (#841): a refusal without a source path keeps saying what its
/// continuations are missing — on the target read and on every suggestion.
#[test]
fn a_pathless_miss_keeps_its_prerequisites_on_every_continuation() {
    let source = docx(&format!(
        "{}{}",
        para("the needle appears here"),
        para("other")
    ));
    let working = paragraphs(&source).unwrap()[1].text.clone();
    let refused = v2_resolve(
        &source,
        &working,
        &AnchorRefusal {
            operation: None,
            target: "p1",
            anchor: "needle",
            path: None,
        },
        None,
    )
    .unwrap_err();
    assert_eq!(refused.next_actions.len(), 2);
    for action in &refused.next_actions {
        assert_eq!(action.argv[0], "jubarte");
        assert_eq!(action.argv[1], "read");
        assert!(!action.argv.iter().any(|arg| arg.contains("needle")));
        let prerequisites = action.prerequisites.as_deref().unwrap_or_default();
        assert!(
            prerequisites.iter().any(|p| p.contains("source file path")),
            "{prerequisites:?}"
        );
    }
    let suggestion = &refused.next_actions[1];
    assert_eq!(suggestion.argv, ["jubarte", "read", "-p", "p0"]);
    assert!(
        suggestion
            .prerequisites
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|p| p.contains("re-targeting")),
        "{:?}",
        suggestion.prerequisites
    );
}
