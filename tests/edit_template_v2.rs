// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Source-bound v2 mixed transactions, using in-memory OOXML packages.
mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::edit::{v2::PlanV2, v2_apply::apply_plan};
use jubarte::inspect::{paragraphs, source_sha256};
use serde_json::{Value, json};

const STAMP: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
fn insertion(id: u32, text: &str) -> String {
    format!(r#"<w:ins w:id="{id}" {STAMP}><w:r><w:t>{text}</w:t></w:r></w:ins>"#)
}
fn source() -> Vec<u8> {
    docx(&format!(
        "<w:p><w:r><w:t xml:space=\"preserve\">Pay </w:t></w:r>{}<w:r><w:t xml:space=\"preserve\"> days, Delaware law.</w:t></w:r></w:p><w:p>{}</w:p>{}",
        insertion(7, "45"),
        insertion(8, "Unrelated"),
        para("Last.")
    ))
}
fn p(n: usize) -> Value {
    json!({"short_id":format!("p{n}"), "long_id":format!("document.w:p.j:short.p{n}")})
}
fn r(n: u32) -> Value {
    json!({"short_id":format!("r{n}"), "long_id":format!("document.w:ins.w:id.{n}")})
}
fn plan(bytes: &[u8], operations: Value) -> Value {
    let mut request = json!({"schema_version":2, "source":{"sha256":source_sha256(bytes)},
        "policy":{"track":"on","existing_revisions":"keep","text_view":"final"},
        "resolve_revisions":{"accept":[],"reject":[]}, "author":"Me","date":"2026-10-17T00:00:00Z",
        "scope":{"kind":"all"},"operations":[]});
    request["operations"] = operations;
    request
}
fn run(
    bytes: &[u8],
    plan: &Value,
) -> Result<jubarte::edit::v2_apply::V2EditResult, jubarte::edit::EditError> {
    apply_plan(bytes, &PlanV2::from_json(&plan.to_string()).unwrap())
}
fn refused(bytes: &[u8], plan: &Value) -> jubarte::edit::EditError {
    let result = run(bytes, plan);
    assert!(
        result.as_ref().ok().map(|out| &out.redline).is_none(),
        "refusal exposed primary bytes"
    );
    assert!(
        result.as_ref().ok().map(|out| &out.clean).is_none(),
        "refusal exposed candidate bytes"
    );
    result.unwrap_err()
}
fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}
fn rewrite(n: usize, before: &str, after: &str) -> Value {
    json!({"kind":"rewrite", "target":p(n),"before_sha256":source_sha256(before.as_bytes()),"text":after})
}
fn replace(n: usize, find: &str, text: &str) -> Value {
    json!({"kind":"replace","target":p(n),"find":find,"text":text})
}
fn mine() -> ChangeFilter {
    ChangeFilter {
        authors: Some(vec!["Me".into()]),
        ..ChangeFilter::default()
    }
}

#[test]
fn selected_revision_then_disjoint_edits_and_comment() {
    let s = source();
    let mut q = plan(
        &s,
        json!([replace(0,"45","60"),replace(0,"Delaware","New York"),
        {"kind":"comment","target":p(0),"text":"Agreed","find":" days, "}]),
    );
    q["resolve_revisions"]["accept"] = json!([r(7)]);
    let out = run(&s, &q).unwrap();
    assert_word_valid_package(&out.redline);
    assert_word_valid_package(&out.clean);
    assert_eq!(
        texts(&out.clean),
        ["Pay 60 days, New York law.", "Unrelated", "Last."]
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["Pay 45 days, Delaware law.", "Unrelated", "Last."]
    );
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        texts(&out.clean)
    );
    assert!(
        list_changes(&out.redline)
            .unwrap()
            .iter()
            .any(|c| c.id == "r8" && c.author.as_deref() == Some("Other"))
    );
    let comments = jubarte::comments::list_comments(&out.redline).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].anchor_text, " days, ");
    assert_eq!(out.report.resolved_revisions.accepted, ["r7"]);
    assert_eq!(out.report.source_sha256, source_sha256(&s));
}

fn merged_source() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {STAMP}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>{}{}"#,
        para("Beta"),
        para("Survivor")
    ))
}
fn merge_plan(s: &[u8], ops: Value) -> Value {
    let mut q = plan(s, ops);
    q["resolve_revisions"]["accept"] = json!([{"short_id":"r1","long_id":"document.w:del.w:id.1"}]);
    q
}
#[test]
fn source_ids_survive_selected_settlement() {
    let s = merged_source();
    let q = merge_plan(&s, json!([replace(2, "Survivor", "Correct")]));
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), ["AlphaBeta", "Correct"]);
}
#[test]
fn rewrite_requires_one_to_one_working_paragraph() {
    let s = merged_source();
    let good = merge_plan(&s, json!([rewrite(2, "Survivor", "Correct")]));
    assert_eq!(
        texts(&run(&s, &good).unwrap().clean),
        ["AlphaBeta", "Correct"]
    );
    for n in [0, 1] {
        for op in [
            rewrite(n, "AlphaBeta", "Wrong"),
            replace(n, "Alpha", "Wrong"),
        ] {
            let e = run(&s, &merge_plan(&s, json!([op]))).unwrap_err();
            assert_eq!(e.code, "UNSUPPORTED_CORRESPONDENCE");
        }
    }
}
#[test]
fn revision_decisions_do_not_expand_body_scope() {
    let s = source();
    let mut q = plan(&s, json!([replace(0, "45", "60")]));
    q["resolve_revisions"]["accept"] = json!([r(7)]);
    q["scope"] = json!({"kind":"selected","targets":[p(2)]});
    assert_eq!(run(&s, &q).unwrap_err().code, "OUT_OF_SCOPE");
    q["operations"] = json!([{"kind":"comment","target":p(0),"through":p(2),"text":"Range"}]);
    q["scope"]["targets"] = json!([p(0), p(2)]);
    assert_eq!(run(&s, &q).unwrap_err().code, "OUT_OF_SCOPE");
    q["scope"]["targets"] = json!([p(0), p(1), p(2)]);
    assert!(run(&s, &q).is_ok());
}
#[test]
fn global_refuse_applies_to_review_only_and_noop() {
    let s = source();
    for ops in [
        json!([]),
        json!([rewrite(2, "Last.", "Last.")]),
        json!([{"kind":"comment","target":p(2),"text":"Agreed"}]),
    ] {
        let mut q = plan(&s, ops);
        q["policy"]["existing_revisions"] = json!("refuse");
        q["resolve_revisions"]["accept"] = json!([r(7)]);
        assert_eq!(run(&s, &q).unwrap_err().code, "EXISTING_REVISIONS");
        q["resolve_revisions"]["accept"] = json!([r(7), r(8)]);
        assert!(run(&s, &q).is_ok());
    }
}
#[test]
fn unchanged_scaffold_does_not_conflict_with_explicit_replace() {
    let s = docx(&para("Alpha Beta"));
    let mut row = rewrite(0, "Alpha Beta", "Alpha Beta");
    row["attached_comment"] = json!({"text":"Full paragraph"});
    let q = plan(&s, json!([row, replace(0, "Beta", "Gamma")]));
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), ["Alpha Gamma"]);
    let comments = jubarte::comments::list_comments(&out.redline).unwrap();
    assert_eq!(comments[0].anchor_text, "Alpha Gamma");
}
#[test]
fn template_rejection_invalidates_only_affected_baselines() {
    let s = source();
    let mut q = plan(
        &s,
        json!([
            rewrite(
                0,
                "Pay 45 days, Delaware law.",
                "Pay 45 days, Delaware law."
            ),
            rewrite(2, "Last.", "Last.")
        ]),
    );
    q["resolve_revisions"]["reject"] = json!([r(7)]);
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "WORKING_BASE_CHANGED");
    assert_eq!(e.operation.as_deref(), Some("/operations/0"));
    assert!(
        e.message
            .contains(&source_sha256(b"Pay  days, Delaware law."))
    );
    q["operations"] = json!([rewrite(2, "Last.", "Last.")]);
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.redline)[0], "Pay  days, Delaware law.");
    assert!(
        list_changes(&out.redline)
            .unwrap()
            .iter()
            .any(|c| c.id == "r8")
    );
}
#[test]
fn global_policy_runs_with_empty_operations() {
    let s = source();
    for (policy, first) in [
        ("accept", "Pay 45 days, Delaware law."),
        ("reject", "Pay  days, Delaware law."),
    ] {
        let mut q = plan(&s, json!([]));
        q["policy"]["existing_revisions"] = json!(policy);
        let out = run(&s, &q).unwrap();
        assert!(list_changes(&out.redline).unwrap().is_empty());
        assert_eq!(texts(&out.redline)[0], first);
    }
}
#[test]
fn direct_keep_preserves_unselected_history() {
    let s = source();
    let mut q = plan(&s, json!([replace(2, "Last.", "Direct.")]));
    q["policy"]["track"] = json!("off");
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.redline)[2], "Direct.");
    assert_eq!(
        list_changes(&out.redline).unwrap(),
        list_changes(&s).unwrap()
    );
}
#[test]
fn selected_decisions_conflict_after_move_expansion() {
    let s = docx(&format!(
        r#"<w:p><w:moveFromRangeStart w:id="0" w:name="m" {STAMP}/><w:moveFrom w:id="1" {STAMP}><w:r><w:t>A</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="0"/></w:p><w:p><w:moveToRangeStart w:id="2" w:name="m" {STAMP}/><w:moveTo w:id="3" {STAMP}><w:r><w:t>A</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="2"/></w:p>"#
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["accept"] =
        json!([{"short_id":"r1","long_id":"document.w:moveFrom.w:id.1"}]);
    q["resolve_revisions"]["reject"] =
        json!([{"short_id":"r3","long_id":"document.w:moveTo.w:id.3"}]);
    assert_eq!(run(&s, &q).unwrap_err().code, "REVISION_CONFLICT");
}
#[test]
fn desired_text_and_comments_never_settle_revisions() {
    let s = source();
    let q = plan(
        &s,
        json!([rewrite(0,"Pay 45 days, Delaware law.","Pay 45 days, Delaware law."),{"kind":"comment","target":p(2),"text":"Agreed"}]),
    );
    let out = run(&s, &q).unwrap();
    assert_eq!(
        list_changes(&out.redline).unwrap(),
        list_changes(&s).unwrap()
    );
}
#[test]
fn v2_literal_occurrences_are_exact_and_overlap_aware() {
    let s = docx(&para("aaa & a\u{a0}b"));
    let mut q = plan(&s, json!([replace(0, "aa", "X")]));
    assert_eq!(run(&s, &q).unwrap_err().code, "AMBIGUOUS_ANCHOR");
    for (n, want) in [(1, "Xa & a\u{a0}b"), (2, "aX & a\u{a0}b")] {
        q["operations"][0]["occurrence"] = json!(n);
        assert_eq!(texts(&run(&s, &q).unwrap().clean), [want]);
    }
    q["operations"] = json!([replace(0, "a b", "X")]);
    assert_eq!(run(&s, &q).unwrap_err().code, "ANCHOR_NOT_FOUND");
}

fn assert_max_occurrence_reports_ambiguity(mut operation: Value) {
    let s = docx(&para("two two"));
    operation["occurrence"] = json!(18446744073709551615_u64);
    let q = plan(&s, json!([operation]));
    let error = run(&s, &q).unwrap_err();
    assert_eq!(error.code, "AMBIGUOUS_ANCHOR");
    assert!(error.message.contains("occurs 2 times"), "{error:?}");
}

#[test]
fn v2_max_occurrence_replace_reports_full_candidate_count() {
    assert_max_occurrence_reports_ambiguity(replace(0, "two", "X"));
}

#[test]
fn v2_max_occurrence_insert_reports_full_candidate_count() {
    assert_max_occurrence_reports_ambiguity(
        json!({"kind":"insert","target":p(0),"before_text":"two","text":"X"}),
    );
}

#[test]
fn v2_max_occurrence_comment_reports_full_candidate_count() {
    assert_max_occurrence_reports_ambiguity(
        json!({"kind":"comment","target":p(0),"find":"two","text":"Note"}),
    );
}

#[test]
fn changed_rewrite_conflicts_with_other_active_text_in_its_paragraph() {
    let s = docx(&para("Alpha Beta"));
    let q = plan(
        &s,
        json!([
            rewrite(0, "Alpha Beta", "Alpha Gamma"),
            replace(0, "Alpha", "Delta")
        ]),
    );
    assert_eq!(run(&s, &q).unwrap_err().code, "CONFLICT");
}
#[test]
fn identical_replace_keeps_attachment_without_claiming_text() {
    let s = docx(&para("Alpha Beta"));
    let mut a = replace(0, "Beta", "Beta");
    a["attached_comment"] = json!({"text":"Keep this span"});
    let out = run(&s, &plan(&s, json!([a, replace(0, "Alpha", "Delta")]))).unwrap();
    assert_eq!(texts(&out.clean), ["Delta Beta"]);
    assert_eq!(
        jubarte::comments::list_comments(&out.redline).unwrap()[0].anchor_text,
        "Beta"
    );
}
#[test]
fn paired_ids_are_verified_in_original_source() {
    let s = source();
    let mut q = plan(&s, json!([replace(2, "Last.", "Wrong")]));
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.j:short.p0");
    assert_eq!(run(&s, &q).unwrap_err().code, "TARGET_ID_MISMATCH");
    q = plan(&s, json!([]));
    q["resolve_revisions"]["accept"] = json!([{"short_id":"r7","long_id":"document.w:ins.w:id.8"}]);
    assert_eq!(run(&s, &q).unwrap_err().code, "TARGET_ID_MISMATCH");
}
#[test]
fn tracked_boundaries_recover_effective_working_text() {
    let s = source();
    let mut q = plan(&s, json!([replace(0, "Delaware", "  A & B\tC\n ")]));
    q["resolve_revisions"]["accept"] = json!([r(7)]);
    let out = run(&s, &q).unwrap();
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["Pay 45 days, Delaware law.", "Unrelated", "Last."]
    );
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        ["Pay 45 days,   A & B\tC\n  law.", "Unrelated", "Last."]
    );
    let xml = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(xml.contains("A &amp; B"));
    assert!(xml.contains("xml:space=\"preserve\""));
}
#[test]
fn unknown_revision_forms_cannot_establish_revision_free() {
    let s = docx(&format!(
        r#"<w:p><w:pPr><w:futurePrChange w:id="50" {STAMP}/></w:pPr><w:r><w:t>A</w:t></w:r></w:p>"#
    ));
    let mut q = plan(&s, json!([]));
    q["policy"]["existing_revisions"] = json!("refuse");
    assert_eq!(run(&s, &q).unwrap_err().code, "EXISTING_REVISIONS");
}
#[test]
fn true_noop_retains_original_bytes() {
    let s = source();
    let out = run(&s, &plan(&s, json!([rewrite(2, "Last.", "Last.")]))).unwrap();
    assert_eq!(out.redline, s);
    assert_eq!(out.clean, s);
}

#[test]
fn compiled_scope_expands_table_descendants_without_id_arithmetic() {
    let s = docx(&format!(
        r#"{}<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{}{}</w:tc></w:tr></w:tbl>{}"#,
        para("Outside"),
        para("First"),
        para("Second"),
        para("Outside two")
    ));
    let mut q = plan(&s, json!([replace(2, "Second", "Changed")]));
    q["scope"] = json!({"kind":"selected","targets":[{"short_id":"t0","long_id":"document.w:tbl.j:short.t0"}]});
    assert_eq!(
        texts(&run(&s, &q).unwrap().clean),
        ["Outside", "First", "Changed", "Outside two"]
    );
    q["operations"] = json!([replace(3, "Outside", "Wrong")]);
    assert_eq!(run(&s, &q).unwrap_err().code, "OUT_OF_SCOPE");
}
#[test]
fn dependent_revision_and_comment_effects_are_disclosed() {
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="7" {STAMP}><w:ins w:id="8" {STAMP}><w:r><w:t>Nested</w:t></w:r></w:ins></w:ins></w:p>{}"#,
        para("Last")
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["reject"] = json!([r(7)]);
    let out = run(&s, &q).unwrap();
    assert_eq!(out.report.resolved_revisions.rejected, ["r7"]);
    // Dependencies are not mislabeled as explicitly selected decisions.
    assert_eq!(out.dependent_revisions, ["r8"]);
}
#[test]
fn opposite_decisions_on_a_removed_dependency_refuse() {
    let s = docx(&format!(
        r#"<w:p><w:del w:id="7" {STAMP}><w:ins w:id="8" {STAMP}><w:r><w:t>Nested</w:t></w:r></w:ins></w:del></w:p>{}"#,
        para("Last")
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["accept"] = json!([{"short_id":"r7","long_id":"document.w:del.w:id.7"}]);
    q["resolve_revisions"]["reject"] = json!([r(8)]);
    assert_eq!(run(&s, &q).unwrap_err().code, "REVISION_CONFLICT");
}
#[test]
fn original_source_context_is_checked_independently_of_working_text() {
    let s = source();
    let mut q = plan(&s, json!([]));
    q["source_context"] = json!([{"kind":"paragraph","target":p(0),"text":"Pay 45 days, Delaware law.","reason_codes":["CONTEXT"]}]);
    q["resolve_revisions"]["reject"] = json!([r(7)]);
    assert!(run(&s, &q).is_ok());
    q["source_context"][0]["text"] = json!("Edited context");
    assert_eq!(run(&s, &q).unwrap_err().code, "SOURCE_CONTEXT_CHANGED");
}
#[test]
fn rewrite_whitespace_and_soft_break_are_exact() {
    let s = docx(&para("Alpha Beta"));
    let out = run(
        &s,
        &plan(&s, json!([rewrite(0, "Alpha Beta", "Alpha\tBeta\n ")])),
    )
    .unwrap();
    assert_eq!(texts(&out.clean), ["Alpha\tBeta\n "]);
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["Alpha Beta"]
    );
}
#[test]
fn attached_comments_use_nonempty_result_anchors() {
    let s = docx(&para("Alpha Beta"));
    for (operation, want) in [
        (
            json!({"kind":"replace","target":p(0),"find":"Beta","text":"Gamma","attached_comment":{"text":"Note"}}),
            "Gamma",
        ),
        (
            json!({"kind":"rewrite","target":p(0),"before_sha256":source_sha256(b"Alpha Beta"),"text":"Alpha Gamma","attached_comment":{"text":"Note"}}),
            "Alpha Gamma",
        ),
        (
            json!({"kind":"insert","target":p(0),"edge":"end","text":" New","attached_comment":{"text":"Note"}}),
            " New",
        ),
        (
            json!({"kind":"insert_paragraph","target":p(0),"position":"after","text":"New paragraph","attached_comment":{"text":"Note"}}),
            "New paragraph",
        ),
    ] {
        let out = run(&s, &plan(&s, json!([operation]))).unwrap();
        assert_eq!(
            jubarte::comments::list_comments(&out.redline).unwrap()[0].anchor_text,
            want
        );
        for bytes in [&out.clean, &out.redline] {
            let comments = jubarte::comments::list_comments(bytes).unwrap();
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].anchor_text, want);
            assert_eq!(comments[0].author, "Me");
            assert_eq!(comments[0].initials.as_deref(), Some("M"));
            assert_eq!(comments[0].date.as_deref(), Some("2026-10-17T00:00:00Z"));
            assert_word_valid_package(bytes);
        }
    }
    let mut invalid = plan(&s, json!([rewrite(0, "Alpha Beta", "")]));
    invalid["operations"][0]["attached_comment"] = json!({"text":"Note"});
    assert!(PlanV2::from_json(&invalid.to_string()).is_err());
}
#[test]
fn source_binding_and_native_paragraph_identity_are_checked() {
    let s = docx(
        r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="ABCD1234"><w:r><w:t>Native</w:t></w:r></w:p>"#,
    );
    let mut q = plan(&s, json!([replace(0, "Native", "Changed")]));
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.w14:paraId.ABCD1234");
    assert_eq!(texts(&run(&s, &q).unwrap().clean), ["Changed"]);
    q["source"]["sha256"] = json!("0".repeat(64));
    assert_eq!(run(&s, &q).unwrap_err().code, "STALE_SOURCE");
}

fn comment_source() -> Vec<u8> {
    let s = docx(&para("Alpha Beta"));
    let q=jubarte::edit::EditPlan::from_json(r#"{"schema_version":1,"author":"Other","date":"2026-09-01T00:00:00Z","existing_revisions":"keep","operations":[{"kind":"comment","paragraph":"p0","text":"Original"}]}"#).unwrap();
    jubarte::edit::apply_plan(&s, &q).unwrap().redline
}
fn c(n: u32) -> Value {
    json!({"short_id":format!("c{n}"),"long_id":format!("comments.w:comment.w:id.{n}")})
}
#[test]
fn comment_mutations_and_final_thread_state_compose() {
    let s = comment_source();
    let q = plan(
        &s,
        json!([
            {"kind":"resolve_comment","target":c(0),"done":true},
            {"kind":"reply_comment","target":c(0),"text":"First"},
            {"kind":"reply_comment","target":c(0),"text":"Second"}
        ]),
    );
    let out = run(&s, &q).unwrap();
    let comments = jubarte::comments::list_comments(&out.redline).unwrap();
    assert_eq!(
        comments.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
        ["Original", "First", "Second"]
    );
    assert!(comments.iter().all(|c| c.done));
    let mut duplicate = q.clone();
    duplicate["operations"]
        .as_array_mut()
        .unwrap()
        .push(json!({"kind":"resolve_comment","target":c(0),"done":false}));
    assert_eq!(run(&s, &duplicate).unwrap_err().code, "CONFLICT");
    let mut deletion = plan(
        &out.redline,
        json!([{"kind":"delete_comment","target":c(0),"include_replies":false}]),
    );
    assert_eq!(
        run(&out.redline, &deletion).unwrap_err().code,
        "COMMENT_HAS_REPLIES"
    );
    deletion["operations"][0]["include_replies"] = json!(true);
    assert!(
        jubarte::comments::list_comments(&run(&out.redline, &deletion).unwrap().redline)
            .unwrap()
            .is_empty()
    );
}
#[test]
fn comment_delete_conflicts_with_other_thread_mutations() {
    let s = comment_source();
    for operation in [
        json!({"kind":"reply_comment","target":c(0),"text":"Reply"}),
        json!({"kind":"edit_comment","target":c(0),"text":"Changed"}),
        json!({"kind":"resolve_comment","target":c(0),"done":true}),
    ] {
        let q = plan(
            &s,
            json!([{"kind":"delete_comment","target":c(0),"include_replies":true},operation]),
        );
        assert_eq!(run(&s, &q).unwrap_err().code, "CONFLICT");
    }
}
#[test]
fn settlement_discloses_comments_removed_with_selected_parent() {
    use common::docx::{Part, docx_with};
    let comment = r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="0" w:author="Other"><w:p><w:r><w:t>Old note</w:t></w:r></w:p></w:comment></w:comments>"#;
    let s = docx_with(
        &format!(
            r#"<w:p><w:ins w:id="7" {STAMP}><w:commentRangeStart w:id="0"/><w:r><w:t>Inserted</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:ins></w:p>{}"#,
            para("Last")
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: comment,
        }],
    );
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["reject"] = json!([r(7)]);
    let out = run(&s, &q).unwrap();
    assert_eq!(out.dependent_comments, [0]);
    for operation in [
        json!({"kind":"reply_comment","target":c(0),"text":"Reply"}),
        json!({"kind":"edit_comment","target":c(0),"text":"Changed"}),
        json!({"kind":"resolve_comment","target":c(0),"done":true}),
        json!({"kind":"delete_comment","target":c(0),"include_replies":true}),
    ] {
        q["operations"] = json!([operation]);
        let error = run(&s, &q).unwrap_err();
        assert_eq!(error.code, "UNKNOWN_COMMENT");
        assert!(
            error
                .message
                .contains("gone once the plan's revisions are settled"),
            "{error}"
        );
        assert_eq!(error.operation.as_deref(), Some("/operations/0"));
    }
}
#[test]
fn direct_scope_and_decision_target_types_are_not_interchangeable() {
    let s = source();
    for target in [
        p(0),
        json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"}),
    ] {
        let mut q = plan(&s, json!([]));
        q["resolve_revisions"]["accept"] = json!([target]);
        assert_eq!(run(&s, &q).unwrap_err().code, "INVALID_TARGET");
    }
    let mut q = plan(&s, json!([]));
    q["scope"] = json!({"kind":"selected","targets":[r(7)]});
    assert_eq!(run(&s, &q).unwrap_err().code, "INVALID_TARGET");
    q = plan(&s, json!([{"kind":"comment","target":r(7),"text":"Wrong"}]));
    assert_eq!(run(&s, &q).unwrap_err().code, "INVALID_TARGET");
}
#[test]
fn stale_rows_are_all_reported_before_text_conflicts() {
    let s = docx(&(para("Alpha") + &para("Beta")));
    let q = plan(
        &s,
        json!([
            rewrite(0, "Wrong", "Alpha"),
            rewrite(1, "Wrong", "Beta"),
            replace(0, "Alpha", "Gamma")
        ]),
    );
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "WORKING_BASE_CHANGED");
    assert_eq!(
        e.outcomes.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(),
        ["/operations/0", "/operations/1"]
    );
    assert_eq!(e.outcomes[0].context.as_deref(), Some("Alpha"));
}
#[test]
fn exact_insert_delete_and_rewrite_compose_with_scope() {
    let s = docx(&(para("a a a") + &para("Separate")));
    let q = plan(
        &s,
        json!([
            {"kind":"insert","target":p(0),"before_text":"a","occurrence":2,"text":"before "},
            {"kind":"insert","target":p(0),"after_text":"a","occurrence":2,"text":" after"},
            {"kind":"delete","target":p(0),"find":"a","occurrence":3},
            rewrite(1,"Separate","Rewritten")
        ]),
    );
    assert_eq!(
        texts(&run(&s, &q).unwrap().clean),
        ["a before a after ", "Rewritten"]
    );
}
#[test]
fn unsupported_tracked_span_is_never_accepted_implicitly() {
    let s = source();
    for track in ["on", "off"] {
        let mut q = plan(&s, json!([replace(0, "45", "60")]));
        q["policy"]["track"] = json!(track);
        assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_STRUCTURE");
    }
}
#[test]
fn container_scope_is_distinct_from_a_paragraph_target() {
    let s = docx(&format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}"#,
        para("Cell"),
        para("Last")
    ));
    let mut q = plan(&s, json!([replace(0, "Cell", "Wrong")]));
    q["operations"][0]["target"] = json!({"short_id":"t0","long_id":"document.w:tbl.j:short.t0"});
    assert_eq!(run(&s, &q).unwrap_err().code, "INVALID_TARGET");
    q["operations"][0]["target"] =
        json!({"short_id":"t0.row0.cell0","long_id":"document.w:tc.j:short.t0.row0.cell0"});
    assert_eq!(texts(&run(&s, &q).unwrap().clean), ["Wrong", "Last"]);
}
#[test]
fn compiled_scope_expands_block_content_controls() {
    let s = docx(&format!(
        r#"{}<w:sdt><w:sdtPr/><w:sdtContent>{}{}</w:sdtContent></w:sdt>{}"#,
        para("Outside"),
        para("Inside one"),
        para("Inside two"),
        para("After")
    ));
    let mut q = plan(&s, json!([replace(2, "two", "changed")]));
    q["scope"] = json!({"kind":"selected","targets":[{"short_id":"cc0","long_id":"document.w:sdt.j:short.cc0"}]});
    assert_eq!(
        texts(&run(&s, &q).unwrap().clean),
        ["Outside", "Inside one", "Inside changed", "After"]
    );
    q["operations"] = json!([replace(3, "After", "Wrong")]);
    assert_eq!(run(&s, &q).unwrap_err().code, "OUT_OF_SCOPE");
}
#[test]
fn deeply_nested_dependency_conflicts_are_detected() {
    let s = docx(&format!(
        r#"<w:p><w:del w:id="7" {STAMP}><w:ins w:id="8" {STAMP}><w:ins w:id="9" {STAMP}><w:r><w:t>Nested</w:t></w:r></w:ins></w:ins></w:del></w:p>{}"#,
        para("Last")
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["accept"] = json!([{"short_id":"r7","long_id":"document.w:del.w:id.7"}]);
    q["resolve_revisions"]["reject"] = json!([r(9)]);
    assert_eq!(run(&s, &q).unwrap_err().code, "REVISION_CONFLICT");
}
#[test]
fn global_refuse_includes_headers_and_styles_outside_body_scope() {
    use common::docx::{Part, docx_with_sect};
    for (name, typ, rel, xml, refs) in [
        (
            "word/header1.xml",
            "header",
            "header",
            format!(
                r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p>{}</w:p></w:hdr>"#,
                insertion(90, "Header history")
            ),
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        ),
        (
            "word/styles.xml",
            "styles",
            "styles",
            format!(
                r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:rPrChange w:id="90" {STAMP}><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style></w:styles>"#
            ),
            "",
        ),
    ] {
        let content_type =
            format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{typ}+xml");
        let rel_type =
            format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel}");
        let s = docx_with_sect(
            &para("Body"),
            &[Part {
                name,
                content_type: &content_type,
                rel_type: &rel_type,
                xml: &xml,
            }],
            refs,
        );
        let mut q = plan(&s, json!([]));
        q["scope"] = json!({"kind":"selected","targets":[p(0)]});
        q["policy"]["existing_revisions"] = json!("refuse");
        assert_eq!(run(&s, &q).unwrap_err().code, "EXISTING_REVISIONS");
        q["policy"]["existing_revisions"] = json!("accept");
        let out = run(&s, &q).unwrap();
        assert!(list_changes(&out.redline).unwrap().is_empty());
        assert!(!out.remaining_revisions.accepted.is_empty());
    }
}
#[test]
fn unknown_revision_policy_refuses_instead_of_dropping_it() {
    let s = docx(&format!(
        r#"<w:p><w:pPr><w:futurePrChange w:id="50" {STAMP}/></w:pPr><w:r><w:t>A</w:t></w:r></w:p>"#
    ));
    for policy in ["accept", "reject"] {
        let mut q = plan(&s, json!([]));
        q["policy"]["existing_revisions"] = json!(policy);
        assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_REVISIONS");
    }
}
#[test]
fn direct_edits_in_a_paragraph_with_history_preserve_it() {
    let s = source();
    let mut q = plan(&s, json!([replace(0, "Delaware", "New York")]));
    q["policy"]["track"] = json!("off");
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.redline)[0], "Pay 45 days, New York law.");
    assert_eq!(
        list_changes(&out.redline).unwrap(),
        list_changes(&s).unwrap()
    );
}
#[test]
fn an_explicitly_named_dependency_is_reported_as_carried() {
    let s = docx(&format!(
        r#"<w:p><w:ins w:id="7" {STAMP}><w:ins w:id="8" {STAMP}><w:r><w:t>Nested</w:t></w:r></w:ins></w:ins></w:p>{}"#,
        para("Last")
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["reject"] = json!([r(7), r(8)]);
    let out = run(&s, &q).unwrap();
    assert_eq!(out.report.resolved_revisions.rejected, ["r7"]);
    assert_eq!(out.dependent_revisions, ["r8"]);
}
#[test]
fn source_ids_survive_deleted_tables_without_retargeting() {
    let table = |deleted: bool| {
        let pr = if deleted {
            format!(r#"<w:trPr><w:del w:id="40" {STAMP}/></w:trPr>"#)
        } else {
            String::new()
        };
        let run = if deleted {
            format!(r#"<w:del w:id="41" {STAMP}><w:r><w:delText>Gone</w:delText></w:r></w:del>"#)
        } else {
            "<w:r><w:t>Target</w:t></w:r>".into()
        };
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr>{pr}<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p>{run}</w:p></w:tc></w:tr></w:tbl>"#
        )
    };
    let s = docx(&(table(true) + &para("Outside") + &table(false) + &para("Last")));
    let mut q = plan(&s, json!([replace(2, "Target", "Changed")]));
    q["resolve_revisions"]["accept"] =
        json!([{"short_id":"r40","long_id":"document.w:del.w:id.40"}]);
    q["operations"][0]["target"] =
        json!({"short_id":"t1.row0.cell0","long_id":"document.w:tc.j:short.t1.row0.cell0"});
    assert_eq!(
        texts(&run(&s, &q).unwrap().clean),
        ["Outside", "Changed", "Last"]
    );
    q["operations"] = json!([rewrite(0, "Gone", "Wrong")]);
    assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_CORRESPONDENCE");
}

#[test]
fn comment_boundary_inside_replacement_refuses_atomically() {
    let s = docx(&para("Alpha Beta"));
    for existing in [false, true] {
        let comment = json!({"kind":"comment","target":p(0),"find":"pha Be","text":"Boundary"});
        let source = if existing {
            run(&s, &plan(&s, json!([comment.clone()]))).unwrap().clean
        } else {
            s.clone()
        };
        let mut ops = vec![replace(0, "Alpha", "A")];
        if !existing {
            ops.push(comment);
        }
        // A refused transaction exposes no candidate bytes to publish.
        let error = refused(&source, &plan(&source, json!(ops)));
        assert_eq!(error.code, "COMMENT_ANCHOR_UNMAPPABLE", "{error}");
        assert!(error.message.contains("boundary"), "{error}");
        assert_eq!(
            error.operation.as_deref(),
            Some(if existing {
                "/operations/0"
            } else {
                "/operations/1"
            })
        );
    }
}

#[test]
fn whole_comment_anchor_deletion_refuses_without_preservation() {
    let s = docx(&(para("Alpha Beta") + &para("Last")));
    let comment = json!({"kind":"comment","target":p(0),"find":"Beta","text":"Anchor"});
    let with = run(&s, &plan(&s, json!([comment.clone()]))).unwrap().clean;
    for existing in [false, true] {
        for edit in [
            json!({"kind":"delete","target":p(0),"find":"Beta"}),
            json!({"kind":"delete_paragraph","target":p(0)}),
        ] {
            let source = if existing { &with } else { &s };
            let mut ops = vec![edit];
            if !existing {
                ops.push(comment.clone());
            }
            let mut q = plan(source, json!(ops));
            // Tracking off removes the original anchor from the primary.
            // Existing tracked deletion preservation has its own positive test.
            q["policy"]["track"] = json!("off");
            let error = run(source, &q).unwrap_err();
            assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
            assert_eq!(
                error.operation.as_deref(),
                Some(if existing {
                    "/operations/0"
                } else {
                    "/operations/1"
                })
            );
            assert!(
                error.message.contains("entire comment anchor")
                    && error.message.contains("no supported preservation rule"),
                "{error}"
            );
        }
    }
}

#[test]
fn comment_anchor_preservation_supports_replacement_and_containing_ranges() {
    let s = docx(&para("Alpha Beta Omega"));
    for find in ["Beta", "Alpha Beta Omega"] {
        let comment = json!({"kind":"comment","target":p(0),"find":find,"text":"Keep"});
        let with = run(&s, &plan(&s, json!([comment.clone()]))).unwrap().clean;
        for existing in [false, true] {
            let source = if existing { &with } else { &s };
            let mut ops = vec![replace(0, "Beta", "Gamma")];
            if !existing {
                ops.push(comment.clone());
            }
            for track in ["on", "off"] {
                let mut q = plan(source, json!(ops));
                q["policy"]["track"] = json!(track);
                let out = run(source, &q).unwrap();
                for bytes in [&out.clean, &out.redline] {
                    let comments = jubarte::comments::list_comments(bytes).unwrap();
                    assert_eq!(comments.len(), 1);
                    // Existing tracked anchors also retain the deleted original
                    // text in the package. Accept only our revisions to observe
                    // the final anchor, independently of that display detail.
                    let final_bytes = accept_changes(bytes, &mine()).unwrap();
                    assert_eq!(
                        jubarte::comments::list_comments(&final_bytes).unwrap()[0].anchor_text,
                        find.replace("Beta", "Gamma")
                    );
                    assert_word_valid_package(bytes);
                }
            }
        }
    }
}

#[test]
fn duplicate_comment_edits_and_thread_state_aliases_conflict() {
    let s = comment_source();
    let with = run(
        &s,
        &plan(
            &s,
            json!([{"kind":"reply_comment","target":c(0),"text":"Existing reply"}]),
        ),
    )
    .unwrap()
    .clean;
    for ops in [
        json!([{"kind":"edit_comment","target":c(0),"text":"First"},{"kind":"edit_comment","target":c(0),"text":"Second"}]),
        json!([{"kind":"resolve_comment","target":c(0),"done":true},{"kind":"resolve_comment","target":c(1),"done":true}]),
        json!([{"kind":"resolve_comment","target":c(1),"done":false},{"kind":"resolve_comment","target":c(0),"done":true}]),
    ] {
        let error = refused(&with, &plan(&with, ops.clone()));
        assert_eq!(error.code, "CONFLICT");
        assert_eq!(error.operation.as_deref(), Some("/operations/1"));
        assert!(
            error.message.contains(if ops[1]["kind"] == "edit_comment" {
                "edits a comment another operation edits"
            } else {
                "only one final state"
            }),
            "{error}"
        );
    }
}

#[test]
fn deleting_a_reply_conflicts_with_mutations_on_its_thread() {
    let s = comment_source();
    let with = run(
        &s,
        &plan(
            &s,
            json!([{"kind":"reply_comment","target":c(0),"text":"Existing reply"}]),
        ),
    )
    .unwrap()
    .clean;
    for target in [c(0), c(1)] {
        for op in [
            json!({"kind":"edit_comment","target":target,"text":"Changed"}),
            json!({"kind":"reply_comment","target":target,"text":"New"}),
            json!({"kind":"resolve_comment","target":target,"done":true}),
        ] {
            for reverse in [false, true] {
                let mut ops = vec![
                    json!({"kind":"delete_comment","target":c(1),"include_replies":false}),
                    op.clone(),
                ];
                if reverse {
                    ops.reverse();
                }
                let error = refused(&with, &plan(&with, json!(ops)));
                assert_eq!(error.code, "CONFLICT");
                assert_eq!(
                    error.operation.as_deref(),
                    Some(if reverse {
                        "/operations/0"
                    } else {
                        "/operations/1"
                    })
                );
                assert!(
                    error
                        .message
                        .contains("delete_comment conflicts with another mutation of its thread"),
                    "{error}"
                );
            }
        }
    }
}

#[test]
fn replies_preserve_prior_state_and_sibling_parentage() {
    let s = comment_source();
    for done in [false, true] {
        let with = run(
            &s,
            &plan(
                &s,
                json!([
                    {"kind":"reply_comment","target":c(0),"text":"Existing reply"},
                    {"kind":"resolve_comment","target":c(0),"done":done}
                ]),
            ),
        )
        .unwrap()
        .clean;
        for final_state in [None, Some(false), Some(true)] {
            let mut ops = vec![
                json!({"kind":"reply_comment","target":c(1),"text":"First"}),
                json!({"kind":"reply_comment","target":c(0),"text":"Second"}),
            ];
            if let Some(state) = final_state {
                ops.insert(
                    0,
                    json!({"kind":"resolve_comment","target":c(1),"done":state}),
                );
            }
            let out = run(&with, &plan(&with, json!(ops))).unwrap();
            for bytes in [&out.clean, &out.redline] {
                let comments = jubarte::comments::list_comments(bytes).unwrap();
                assert_eq!(
                    comments.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
                    ["Original", "Existing reply", "First", "Second"]
                );
                assert!(
                    comments
                        .iter()
                        .all(|c| c.done == final_state.unwrap_or(done)),
                    "{comments:?}"
                );
                assert!(
                    comments[1..].iter().all(|c| c.parent == Some(0)),
                    "{comments:?}"
                );
                assert_word_valid_package(bytes);
            }
        }
    }
}

#[test]
fn comment_attribution_is_preserved_or_taken_from_plan() {
    let s = comment_source();
    let original = jubarte::comments::list_comments(&s).unwrap().remove(0);
    let out = run(
        &s,
        &plan(
            &s,
            json!([
                {"kind":"edit_comment","target":c(0),"text":"Updated"},
                {"kind":"reply_comment","target":c(0),"text":"Reply <literal> & text"},
                {"kind":"comment","target":p(0),"find":"Alpha","text":"Standalone"}
            ]),
        ),
    )
    .unwrap();
    for bytes in [&out.clean, &out.redline] {
        let comments = jubarte::comments::list_comments(bytes).unwrap();
        let edited = comments.iter().find(|c| c.id == 0).unwrap();
        assert_eq!(edited.text, "Updated");
        assert_eq!(
            (&edited.author, &edited.date, &edited.initials),
            (&original.author, &original.date, &original.initials)
        );
        for comment in comments.iter().filter(|c| c.id != 0) {
            assert_eq!(comment.author, "Me");
            assert_eq!(comment.initials.as_deref(), Some("M"));
            assert_eq!(comment.date.as_deref(), Some("2026-10-17T00:00:00Z"));
        }
        assert!(comments.iter().any(|c| c.text == "Reply <literal> & text"));
        assert_word_valid_package(bytes);
    }
}

#[test]
fn unchanged_text_attachments_keep_exact_span_and_plan_attribution() {
    let s = docx(&para("Alpha Beta"));
    let mut unchanged = rewrite(0, "Alpha Beta", "Alpha Beta");
    unchanged["attached_comment"] = json!({"text":"<literal> & note"});
    let mut identity = replace(0, "Beta", "Beta");
    identity["attached_comment"] = json!({"text":"<literal> & note"});
    for (op, want) in [(unchanged, "Alpha Beta"), (identity, "Beta")] {
        let out = run(&s, &plan(&s, json!([op]))).unwrap();
        assert!(list_changes(&out.redline).unwrap().is_empty());
        assert_eq!(out.report.comments_added, 1);
        for bytes in [&out.clean, &out.redline] {
            let comments = jubarte::comments::list_comments(bytes).unwrap();
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].anchor_text, want);
            assert_eq!(comments[0].text, "<literal> & note");
            assert_eq!(comments[0].author, "Me");
            assert_eq!(comments[0].initials.as_deref(), Some("M"));
            assert_eq!(comments[0].date.as_deref(), Some("2026-10-17T00:00:00Z"));
            assert_eq!(texts(bytes), ["Alpha Beta"]);
        }
    }
}

#[test]
fn all_attachment_kinds_refuse_empty_results() {
    let s = docx(&para("Alpha Beta"));
    for mut op in [
        replace(0, "Beta", ""),
        rewrite(0, "Alpha Beta", ""),
        json!({"kind":"insert","target":p(0),"edge":"end","text":""}),
        json!({"kind":"insert_paragraph","target":p(0),"position":"after","text":""}),
    ] {
        op["attached_comment"] = json!({"text":"Note"});
        let error = PlanV2::from_json(&plan(&s, json!([op])).to_string()).unwrap_err();
        assert_eq!(error.code, "INVALID_PLAN");
        assert!(error.message.contains("nonempty"), "{error}");
    }
}

#[test]
fn rewrite_attachment_covers_complete_result_at_both_edges() {
    for (before, after) in [
        ("Alpha Beta", "Prefix Alpha Beta Suffix"),
        ("Alpha Beta", "Completely new"),
    ] {
        let s = docx(&para(before));
        let mut op = rewrite(0, before, after);
        op["attached_comment"] = json!({"text":"Complete paragraph"});
        for track in ["on", "off"] {
            let mut q = plan(&s, json!([op]));
            q["policy"]["track"] = json!(track);
            let out = run(&s, &q).unwrap();
            for bytes in [&out.clean, &out.redline] {
                let final_bytes = accept_changes(bytes, &mine()).unwrap();
                let comments = jubarte::comments::list_comments(&final_bytes).unwrap();
                assert_eq!(comments.len(), 1);
                assert_eq!(comments[0].anchor_text, after, "{before:?} -> {after:?}");
                assert_word_valid_package(bytes);
            }
        }
    }
}

#[test]
fn unchanged_attachment_refuses_when_composition_removes_its_anchor() {
    let s = docx(&para("Alpha Beta"));
    let mut op = replace(0, "Beta", "Beta");
    op["attached_comment"] = json!({"text":"Keep"});
    let error = run(
        &s,
        &plan(
            &s,
            json!([op, {"kind":"delete","target":p(0),"find":"Beta"}]),
        ),
    )
    .unwrap_err();
    assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
    assert_eq!(error.operation.as_deref(), Some("/operations/0"));
    assert!(error.message.contains("entire comment anchor"), "{error}");
    let mut op = rewrite(0, "Alpha Beta", "Alpha Beta");
    op["attached_comment"] = json!({"text":"Keep"});
    let error = run(
        &s,
        &plan(
            &s,
            json!([op, {"kind":"delete","target":p(0),"find":"Alpha Beta"}]),
        ),
    )
    .unwrap_err();
    assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
    assert_eq!(error.operation.as_deref(), Some("/operations/0"));
    assert!(error.message.contains("entire comment anchor"), "{error}");
}

#[test]
fn unknown_comment_targets_refuse_before_any_candidate() {
    let s = comment_source();
    for op in [
        json!({"kind":"edit_comment","target":c(99),"text":"Changed"}),
        json!({"kind":"reply_comment","target":c(99),"text":"Reply"}),
        json!({"kind":"resolve_comment","target":c(99),"done":true}),
        json!({"kind":"delete_comment","target":c(99),"include_replies":true}),
    ] {
        let error = refused(&s, &plan(&s, json!([replace(0, "Beta", "Gamma"), op])));
        assert_eq!(error.code, "TARGET_ID_MISMATCH");
        assert_eq!(error.operation.as_deref(), Some("/operations/1"));
    }
}

#[test]
fn empty_source_paragraph_keeps_existing_structure_refusal() {
    // The existing engine deliberately gates insertion into an empty source
    // paragraph. An attachment must not bypass that structural restriction.
    let s = docx(&para(""));
    let mut op = rewrite(0, "", "New paragraph text");
    op["attached_comment"] = json!({"text":"Complete paragraph"});
    let error = run(&s, &plan(&s, json!([op]))).unwrap_err();
    assert_eq!(error.code, "UNSUPPORTED_STRUCTURE");
    assert!(error.message.contains("empty paragraph"), "{error}");
}

#[test]
fn explicit_comment_deletion_authorizes_anchor_removal() {
    let s = comment_source();
    let out = run(
        &s,
        &plan(
            &s,
            json!([
                {"kind":"delete","target":p(0),"find":"Alpha Beta"},
                {"kind":"delete_comment","target":c(0),"include_replies":true}
            ]),
        ),
    )
    .unwrap();
    for bytes in [&out.clean, &out.redline] {
        assert!(jubarte::comments::list_comments(bytes).unwrap().is_empty());
        assert_word_valid_package(bytes);
    }
}

#[test]
fn existing_containing_comment_survives_interior_paragraph_deletion() {
    let s = docx(&(para("First") + &para("Middle") + &para("Last")));
    let with = run(
        &s,
        &plan(
            &s,
            json!([
                {"kind":"comment","target":p(0),"through":p(2),"text":"All"}
            ]),
        ),
    )
    .unwrap()
    .clean;
    let out = run(
        &with,
        &plan(
            &with,
            json!([
                {"kind":"delete_paragraph","target":p(1)}
            ]),
        ),
    )
    .unwrap();
    for bytes in [&out.clean, &out.redline] {
        let final_bytes = accept_changes(bytes, &mine()).unwrap();
        let comments = jubarte::comments::list_comments(&final_bytes).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].anchor_text, "First\nLast");
        assert_word_valid_package(bytes);
    }
}

#[test]
fn comment_mapping_does_not_mask_text_or_thread_conflicts() {
    let s = comment_source();
    for ops in [
        json!([replace(0, "Alpha Beta", "X"), replace(0, "Beta", "Y")]),
        json!([
            {"kind":"delete","target":p(0),"find":"Alpha Beta"},
            {"kind":"edit_comment","target":c(0),"text":"First"},
            {"kind":"edit_comment","target":c(0),"text":"Second"}
        ]),
    ] {
        let thread = ops.as_array().unwrap().len() == 3;
        let error = refused(&s, &plan(&s, ops));
        assert_eq!(error.code, "CONFLICT", "{error}");
        assert_eq!(
            error.operation.as_deref(),
            Some(if thread {
                "/operations/2"
            } else {
                "/operations/1"
            })
        );
        assert!(
            error.message.contains(if thread {
                "edits a comment another operation edits"
            } else {
                "overlaps an earlier edit"
            }),
            "{error}"
        );
    }
}

#[test]
fn existing_comment_anchor_survives_tracked_deletion() {
    let s = comment_source();
    for edit in [
        json!({"kind":"delete","target":p(0),"find":"Alpha Beta"}),
        json!({"kind":"delete","target":p(0),"find":"Alpha"}),
    ] {
        let out = run(&s, &plan(&s, json!([edit]))).unwrap();
        assert_word_valid_package(&out.redline);
        let comments = jubarte::comments::list_comments(&out.redline).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "Original");
        assert_eq!(comments[0].author, "Other");
        let rejected = reject_changes(&out.redline, &mine()).unwrap();
        assert_eq!(
            jubarte::comments::list_comments(&rejected).unwrap()[0].anchor_text,
            "Alpha Beta"
        );
        let xml = part_string(&out.redline, "word/document.xml").unwrap();
        let start = xml.find("<w:commentRangeStart").unwrap();
        let end = xml.find("<w:commentRangeEnd").unwrap();
        let deletion = xml.find("<w:delText").unwrap();
        assert!(start < deletion && deletion < end, "{xml}");
    }
    let s = docx(&(para("Alpha Beta") + &para("Last")));
    let with = run(
        &s,
        &plan(
            &s,
            json!([
                {"kind":"comment","target":p(0),"text":"Original"}
            ]),
        ),
    )
    .unwrap()
    .clean;
    let out = run(
        &with,
        &plan(
            &with,
            json!([
                {"kind":"delete_paragraph","target":p(0)}
            ]),
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.redline);
    assert_eq!(
        jubarte::comments::list_comments(&out.redline)
            .unwrap()
            .len(),
        1
    );
    let rejected = reject_changes(&out.redline, &mine()).unwrap();
    assert_eq!(
        jubarte::comments::list_comments(&rejected).unwrap()[0].anchor_text,
        "Alpha Beta"
    );
}

#[test]
fn a_new_comment_cannot_anchor_to_text_deleted_in_the_same_plan() {
    let s = docx(&(para("Alpha Beta") + &para("Last")));
    for edit in [
        json!({"kind":"delete","target":p(0),"find":"Beta"}),
        json!({"kind":"delete_paragraph","target":p(0)}),
    ] {
        let error = run(
            &s,
            &plan(
                &s,
                json!([
                    edit,
                    {"kind":"comment","target":p(0),"find":"Beta","text":"New"}
                ]),
            ),
        )
        .unwrap_err();
        assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED");
        assert_eq!(error.operation.as_deref(), Some("/operations/1"));
        assert!(error.message.contains("entire comment anchor"), "{error}");
    }
}

#[test]
fn delete_paragraph_maps_source_target_through_settlement() {
    let s = merged_source();
    let q = merge_plan(&s, json!([{"kind":"delete_paragraph","target":p(2)}]));
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), ["AlphaBeta"]);
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        ["AlphaBeta"]
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["AlphaBeta", "Survivor"]
    );
    assert_eq!(out.report.operations[0].kind, "delete_paragraph");
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    for n in [0, 1] {
        let q = merge_plan(&s, json!([{"kind":"delete_paragraph","target":p(n)}]));
        assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_CORRESPONDENCE");
    }
}

#[test]
fn delete_paragraph_with_kept_tracked_changes_refuses() {
    let s = source();
    let q = plan(&s, json!([{"kind":"delete_paragraph","target":p(0)}]));
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(e.operation.as_deref(), Some("/operations/0"));
    assert!(
        e.message
            .contains("holds tracked changes cannot be deleted")
    );
}

#[test]
fn delete_paragraph_leaves_valid_containers() {
    let s = docx(&(para("First") + &para("Last")));
    let q = plan(
        &s,
        json!([
            {"kind":"delete_paragraph","target":p(0)},
            {"kind":"delete_paragraph","target":p(1)}
        ]),
    );
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "CONFLICT");
    assert!(e.message.contains("without a paragraph"));

    let s = docx(&format!(
        r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr/>{}</w:tc></w:tr></w:tbl>{}"#,
        para("Cell first") + &para("Cell last"),
        para("Outside")
    ));
    let q = plan(
        &s,
        json!([
            {"kind":"delete_paragraph","target":p(0)},
            {"kind":"delete_paragraph","target":p(1)}
        ]),
    );
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "CONFLICT");
    assert!(e.message.contains("table cell without a closing paragraph"));
}

#[test]
fn text_insertions_at_same_source_position_keep_array_order() {
    let s = docx(&para("a a a"));
    for track in ["on", "off"] {
        for anchor in ["after_text", "before_text"] {
            let mut ops = json!([
                {"kind":"insert","target":p(0),"occurrence":1,"text":"1"},
                {"kind":"insert","target":p(0),"occurrence":1,"text":"2"}
            ]);
            for op in ops.as_array_mut().unwrap() {
                op[anchor] = json!("a");
            }
            let mut q = plan(&s, ops);
            q["policy"]["track"] = json!(track);
            let out = run(&s, &q).unwrap();
            let want = if anchor == "after_text" {
                "a12 a a"
            } else {
                "12a a a"
            };
            assert_eq!(texts(&out.clean), [want]);
            assert_eq!(
                texts(&accept_changes(&out.redline, &mine()).unwrap()),
                [want]
            );
            if track == "on" {
                assert_eq!(
                    texts(&reject_changes(&out.redline, &mine()).unwrap()),
                    ["a a a"]
                );
            }
            assert_word_valid_package(&out.redline);
        }
    }
}

#[test]
fn paragraph_insertions_at_same_edge_keep_array_order() {
    let s = docx(&para("Anchor"));
    for side in ["before", "after"] {
        for track in ["on", "off"] {
            let mut q = plan(
                &s,
                json!([
                    {"kind":"insert_paragraph","target":p(0),"position":side,"text":"First"},
                    {"kind":"insert_paragraph","target":p(0),"position":side,"text":"Second"}
                ]),
            );
            q["policy"]["track"] = json!(track);
            let out = run(&s, &q).unwrap();
            let want = if side == "after" {
                ["Anchor", "First", "Second"]
            } else {
                ["First", "Second", "Anchor"]
            };
            assert_eq!(texts(&out.clean), want);
            assert_eq!(texts(&accept_changes(&out.redline, &mine()).unwrap()), want);
            if track == "on" {
                assert_eq!(
                    texts(&reject_changes(&out.redline, &mine()).unwrap()),
                    ["Anchor"]
                );
            }
            assert_word_valid_package(&out.redline);
        }
    }
}

#[test]
fn insertion_inside_removed_text_refuses_ambiguous_ownership() {
    let s = docx(&para("Alpha"));
    let q = plan(
        &s,
        json!([
            replace(0, "lph", "X"),
            {"kind":"insert","target":p(0),"after_text":"Al","text":"Y"}
        ]),
    );
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "CONFLICT");
    assert_eq!(e.operation.as_deref(), Some("/operations/1"));
}

#[test]
fn insertions_with_conflicting_boundary_ownership_refuse() {
    let s = docx(&para("Alpha Beta"));
    for operations in [
        json!([
            {"kind":"insert","target":p(0),"after_text":"Alpha ","text":"1"},
            {"kind":"insert","target":p(0),"before_text":"Beta","text":"2"}
        ]),
        json!([replace(0,"Beta","Gamma"), {"kind":"insert","target":p(0),"before_text":"Beta","text":"X"}]),
        json!([replace(0,"Alpha","Gamma"), {"kind":"insert","target":p(0),"after_text":"Alpha","text":"X"}]),
    ] {
        let e = run(&s, &plan(&s, operations)).unwrap_err();
        assert_eq!(e.code, "CONFLICT");
        assert!(e.message.contains("boundary ownership"));
    }
}

#[test]
fn same_position_insertions_retain_each_attached_comment() {
    let s = docx(&para("Alpha"));
    let q = plan(
        &s,
        json!([
            {"kind":"insert","target":p(0),"edge":"start","text":"one ","attached_comment":{"text":"First"}},
            {"kind":"insert","target":p(0),"edge":"start","text":"two ","attached_comment":{"text":"Second"}}
        ]),
    );
    let out = run(&s, &q).unwrap();
    for bytes in [&out.redline, &out.clean] {
        assert_word_valid_package(bytes);
        let comments = jubarte::comments::list_comments(bytes).unwrap();
        assert_eq!(
            comments
                .iter()
                .map(|c| (c.text.as_str(), c.anchor_text.as_str()))
                .collect::<Vec<_>>(),
            [("First", "one "), ("Second", "two ")]
        );
    }
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        ["one two Alpha"]
    );
}

#[test]
fn rewrite_to_empty_text_preserves_paragraph_mark() {
    let s = docx(&para("Alpha Beta"));
    for track in ["on", "off"] {
        let mut q = plan(&s, json!([rewrite(0, "Alpha Beta", "")]));
        q["policy"]["track"] = json!(track);
        let out = run(&s, &q).unwrap();
        assert_eq!(texts(&out.clean), [""]);
        assert_eq!(texts(&accept_changes(&out.redline, &mine()).unwrap()), [""]);
        if track == "on" {
            assert_eq!(
                texts(&reject_changes(&out.redline, &mine()).unwrap()),
                ["Alpha Beta"]
            );
        }
        assert!(
            jubarte::comments::list_comments(&out.redline)
                .unwrap()
                .is_empty()
        );
        assert_word_valid_package(&out.clean);
        assert_word_valid_package(&out.redline);
    }
}

#[test]
fn same_revision_accepted_and_rejected_refuses_before_preparation() {
    let s = source();
    // A stale rewrite would fail during preparation if decision conflict did not win.
    let mut q = plan(&s, json!([rewrite(2, "Wrong baseline", "Changed")]));
    q["resolve_revisions"] = json!({"accept":[r(7)],"reject":[r(7)]});
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "REVISION_CONFLICT");
    assert_eq!(e.operation, None);
    assert!(e.message.contains("r7"));
}

#[test]
fn comment_direct_scope_does_not_follow_paragraph_scope() {
    let s = comment_source();
    for op in [
        json!({"kind":"reply_comment","target":c(0),"text":"Reply"}),
        json!({"kind":"edit_comment","target":c(0),"text":"Changed"}),
        json!({"kind":"resolve_comment","target":c(0),"done":true}),
        json!({"kind":"delete_comment","target":c(0),"include_replies":true}),
    ] {
        let mut q = plan(&s, json!([op]));
        q["scope"] = json!({"kind":"selected","targets":[p(0)]});
        let e = run(&s, &q).unwrap_err();
        assert_eq!(e.code, "OUT_OF_SCOPE");
        assert_eq!(e.operation.as_deref(), Some("/operations/0"));
        q["scope"]["targets"] = json!([c(0)]);
        assert_word_valid_package(&run(&s, &q).unwrap().redline);
    }
}

#[test]
fn comment_source_context_is_checked_against_original_text() {
    let s = comment_source();
    let mut q = plan(
        &s,
        json!([{"kind":"edit_comment","target":c(0),"text":"New text"}]),
    );
    q["source_context"] =
        json!([{"kind":"comment","target":c(0),"text":"Original","reason_codes":["CONTEXT"]}]);
    let out = run(&s, &q).unwrap();
    assert_eq!(
        jubarte::comments::list_comments(&out.redline).unwrap()[0].text,
        "New text"
    );
    q["source_context"][0]["text"] = json!("Changed observation");
    assert_eq!(run(&s, &q).unwrap_err().code, "SOURCE_CONTEXT_CHANGED");
}

#[test]
fn opaque_source_context_passes_without_text() {
    let s = comment_source();
    let mut q = plan(&s, json!([]));
    q["source_context"] = json!([
        {"kind":"opaque","target":p(0),"reason_codes":["OPAQUE"]},
        {"kind":"opaque","target":c(0),"reason_codes":["OPAQUE"]}
    ]);
    assert_eq!(run(&s, &q).unwrap().redline, s);
    q["source_context"][0]["text"] = json!("Unsupported text");
    assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_CONTEXT");
}

#[test]
fn source_context_kind_target_mismatch_refuses() {
    let s = comment_source();
    for (kind, target) in [("paragraph", c(0)), ("comment", p(0))] {
        let mut q = plan(&s, json!([]));
        q["source_context"] =
            json!([{"kind":kind,"target":target,"text":"Original","reason_codes":["CONTEXT"]}]);
        assert_eq!(run(&s, &q).unwrap_err().code, "UNSUPPORTED_CONTEXT");
    }
}

#[test]
fn two_changed_rewrites_on_one_paragraph_refuse() {
    let s = docx(&para("Alpha"));
    let q = plan(
        &s,
        json!([rewrite(0, "Alpha", "Beta"), rewrite(0, "Alpha", "Gamma")]),
    );
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "CONFLICT");
    assert_eq!(e.operation.as_deref(), Some("/operations/1"));
    assert!(e.message.contains("two changed rewrites"));
}

#[test]
fn overlapping_literal_edits_refuse_without_primary() {
    let s = docx(&para("Alpha"));
    let q = plan(&s, json!([replace(0, "lph", "X"), replace(0, "pha", "Y")]));
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "CONFLICT");
    assert_eq!(e.operation.as_deref(), Some("/operations/1"));
    assert!(e.message.contains("overlaps an earlier edit"));
    assert!(
        e.outcomes
            .iter()
            .all(|o| o.code.as_deref() != Some("OVERLAPPING_EDITS"))
    );
}

#[test]
fn explicit_insert_never_flips_requested_tracked_side() {
    let s = source();
    for track in ["on", "off"] {
        for anchor in ["after_text", "before_text"] {
            let mut op = json!({"kind":"insert","target":p(0),"text":"X"});
            // Both requested sides touch Other's insertion; the opposite side is direct.
            op[anchor] = json!("45");
            let mut q = plan(&s, json!([op]));
            q["policy"]["track"] = json!(track);
            let e = run(&s, &q).unwrap_err();
            assert_eq!(e.code, "UNSUPPORTED_INSERTION_SIDE");
            assert_eq!(e.operation.as_deref(), Some("/operations/0"));
            assert!(e.message.contains("tracked"));
        }
    }
}

#[test]
fn explicit_insert_keeps_supported_side_beside_tracked_text() {
    let s = source();
    for (anchor, literal, want) in [
        ("after_text", "Pay ", "Pay X45 days, Delaware law."),
        ("before_text", " days", "Pay 45X days, Delaware law."),
    ] {
        let mut op = json!({"kind":"insert","target":p(0),"text":"X"});
        op[anchor] = json!(literal);
        let out = run(&s, &plan(&s, json!([op]))).unwrap();
        assert_eq!(texts(&out.clean)[0], want);
        assert_eq!(
            texts(&reject_changes(&out.redline, &mine()).unwrap()),
            texts(&s)
        );
        assert_word_valid_package(&out.redline);
    }
}

#[test]
fn many_revision_decisions_match_global_policy_and_original_scope() {
    let mut body = String::new();
    for n in 0..64 {
        let native = n + 1;
        body.push_str(&format!(r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="{native:08X}">{}</w:p>"#, insertion(n, &format!("Item {n}"))));
    }
    let s = docx(&body);
    for policy in ["accept", "reject"] {
        let mut selected = plan(&s, json!([]));
        selected["resolve_revisions"][policy] = (0..64).map(r).collect();
        selected["scope"] = json!({"kind":"selected","targets":(0..64).map(|n| {let native = n + 1; json!({"short_id":format!("p{n}"),"long_id":format!("document.w:p.w14:paraId.{native:08X}")})}).collect::<Vec<_>>()});
        let mut global = plan(&s, json!([]));
        global["policy"]["existing_revisions"] = json!(policy);
        let selected_out = run(&s, &selected).unwrap();
        let global_out = run(&s, &global).unwrap();
        assert_eq!(texts(&selected_out.redline), texts(&global_out.redline));
        assert_eq!(
            part_string(&selected_out.redline, "word/document.xml"),
            part_string(&global_out.redline, "word/document.xml")
        );
        assert_eq!(
            selected_out.report.resolved_revisions.accepted.len()
                + selected_out.report.resolved_revisions.rejected.len(),
            64
        );
        assert!(list_changes(&selected_out.redline).unwrap().is_empty());
        assert_word_valid_package(&selected_out.redline);
    }
}

#[test]
fn direct_edits_after_global_accept_or_reject_resolve_only_old_history() {
    let s = source();
    let s = run(
        &s,
        &plan(
            &s,
            json!([{"kind":"comment","target":p(2),"text":"Keep this thread"}]),
        ),
    )
    .unwrap()
    .redline;
    let old_comments = jubarte::comments::list_comments(&s).unwrap();
    for policy in ["accept", "reject"] {
        let mut q = plan(&s, json!([replace(0, "Delaware", "New York")]));
        q["policy"]["track"] = json!("off");
        q["policy"]["existing_revisions"] = json!(policy);
        let out = run(&s, &q).unwrap();
        let want = if policy == "accept" {
            ["Pay 45 days, New York law.", "Unrelated", "Last."]
        } else {
            ["Pay  days, New York law.", "", "Last."]
        };
        assert_eq!(texts(&out.redline), want);
        assert_eq!(out.redline, out.clean);
        assert!(list_changes(&out.redline).unwrap().is_empty());
        assert!(out.report.resolved_revisions.is_empty());
        let resolved = if policy == "accept" {
            &out.remaining_revisions.accepted
        } else {
            &out.remaining_revisions.rejected
        };
        assert_eq!(resolved, &["r7", "r8"]);
        assert!(out.dependent_comments.is_empty());
        assert_eq!(
            jubarte::comments::list_comments(&out.redline).unwrap(),
            old_comments
        );
        assert_word_valid_package(&out.redline);
    }
}

#[test]
fn source_identity_inventory_keeps_story_and_duplicate_distinctions() {
    use common::docx::{Part, docx_with_sect};
    let native = |text: &str| {
        format!(
            r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="ABCD1234">{text}</w:p>"#
        )
    };
    let header = format!(
        r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}</w:hdr>"#,
        native(&insertion(7, "Header"))
    );
    let s = docx_with_sect(
        &(native(&insertion(7, "Body")) + &native("<w:r><w:t>Duplicate native ID</w:t></w:r>")),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let mut q = plan(&s, json!([replace(1, "Duplicate", "Changed")]));
    q["resolve_revisions"]["accept"] =
        json!([r(7), {"short_id":"header1.r7","long_id":"header1.w:ins.w:id.7"}]);
    q["source_context"] = json!([{"kind":"paragraph","target":{"short_id":"header1","long_id":"header1.w:p.w14:paraId.ABCD1234"},"text":"Header","reason_codes":["CONTEXT"]}]);
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), ["Body", "Changed native ID"]);
    assert!(
        list_changes(&out.redline)
            .unwrap()
            .iter()
            .all(|c| c.author.as_deref() == Some("Me"))
    );
    // Duplicate native IDs in the body use the short-path fallback. A header's
    // equal native ID remains unique within its own story.
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.w14:paraId.ABCD1234");
    assert_eq!(run(&s, &q).unwrap_err().code, "TARGET_ID_MISMATCH");

    let s = docx(&format!(
        "<w:p>{}{}</w:p>",
        insertion(7, "First"),
        insertion(7, "Second")
    ));
    let mut q = plan(&s, json!([]));
    q["resolve_revisions"]["accept"] = json!([r(7)]);
    assert_eq!(run(&s, &q).unwrap_err().code, "TARGET_ID_MISMATCH");
}

#[test]
fn comment_through_settlement_removed_paragraph_reports_selected_cause() {
    // A removed row really removes its paragraph. Rejecting only inline
    // insertion text leaves an empty paragraph and cannot exercise Gone.
    let removed = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:trPr><w:del w:id="7" {STAMP}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:del w:id="8" {STAMP}><w:r><w:delText>Gone</w:delText></w:r></w:del></w:p></w:tc></w:tr></w:tbl>"#
    );
    let s = docx(&(removed.clone() + &para("Last")));
    for (first, last) in [(0, 1), (0, 0)] {
        let mut q = plan(
            &s,
            json!([
                {"kind":"comment","target":p(first),"through":p(last),"text":"Range"}
            ]),
        );
        q["resolve_revisions"]["accept"] =
            json!([{"short_id":"r7","long_id":"document.w:del.w:id.7"}]);
        let error = refused(&s, &q);
        assert_eq!(error.code, "PARAGRAPH_RESOLVED", "{error}");
        assert_eq!(error.operation.as_deref(), Some("/operations/0"));
        assert!(
            error
                .message
                .contains("gone once the plan's revisions are settled"),
            "{error}"
        );
        assert!(
            error.message.contains("accept") && error.message.contains("r7"),
            "{error}"
        );
    }
    // The removed paragraph can also be interior to the requested range.
    let s = docx(&(para("First") + &removed + &para("Last")));
    let mut q = plan(
        &s,
        json!([{"kind":"comment","target":p(0),"through":p(2),"text":"Range"}]),
    );
    q["resolve_revisions"]["accept"] = json!([{"short_id":"r7","long_id":"document.w:del.w:id.7"}]);
    let error = refused(&s, &q);
    assert_eq!(error.code, "PARAGRAPH_RESOLVED", "{error}");
    assert!(
        error.message.contains("gone") && error.message.contains("r7"),
        "{error}"
    );
    assert_eq!(error.operation.as_deref(), Some("/operations/0"));
    // Rejecting a selected inserted row exercises the opposite decision and
    // the standalone (no through) target through the same removal diagnosis.
    let inserted = removed
        .replace("w:delText", "w:t")
        .replace("w:del", "w:ins");
    let s = docx(&(inserted + &para("Last")));
    let mut q = plan(&s, json!([{"kind":"comment","target":p(0),"text":"Range"}]));
    q["resolve_revisions"]["reject"] = json!([r(7)]);
    let error = refused(&s, &q);
    assert_eq!(error.code, "PARAGRAPH_RESOLVED", "{error}");
    assert_eq!(error.operation.as_deref(), Some("/operations/0"));
    assert!(
        error.message.contains("gone")
            && error.message.contains("reject")
            && error.message.contains("r7"),
        "{error}"
    );
}

#[test]
fn multi_paragraph_comment_maps_edits_in_other_covered_paragraphs() {
    let s = docx(&(para("First") + &para("Middle") + &para("Last")));
    let comment = json!({"kind":"comment","target":p(0),"through":p(2),"text":"All"});
    let with = run(&s, &plan(&s, json!([comment.clone()]))).unwrap().clean;
    for existing in [false, true] {
        let source = if existing { &with } else { &s };
        let mut ops = vec![replace(0, "First", "Start"), replace(2, "Last", "End")];
        if !existing {
            ops.push(comment.clone());
        }
        for track in ["on", "off"] {
            let mut q = plan(source, json!(ops));
            q["policy"]["track"] = json!(track);
            let out = run(source, &q).unwrap();
            for bytes in [&out.clean, &out.redline] {
                let final_bytes = accept_changes(bytes, &mine()).unwrap();
                let comments = jubarte::comments::list_comments(&final_bytes).unwrap();
                assert_eq!(comments.len(), 1);
                assert_eq!(comments[0].anchor_text, "Start\nMiddle\nEnd");
                assert_word_valid_package(bytes);
            }
        }
    }
}

#[test]
fn comment_end_inside_replacement_refuses_atomically() {
    let s = docx(&para("Alpha Beta Omega"));
    let comment = json!({"kind":"comment","target":p(0),"find":"Beta Ome","text":"End boundary"});
    let with = run(&s, &plan(&s, json!([comment.clone()]))).unwrap().clean;
    for existing in [false, true] {
        let source = if existing { &with } else { &s };
        let mut ops = vec![replace(0, "a Omega", "X")];
        if !existing {
            ops.push(comment.clone());
        }
        let error = refused(source, &plan(source, json!(ops)));
        assert_eq!(error.code, "COMMENT_ANCHOR_UNMAPPABLE", "{error}");
        assert_eq!(
            error.operation.as_deref(),
            Some(if existing {
                "/operations/0"
            } else {
                "/operations/1"
            })
        );
        assert!(
            error.message.contains("boundary") && error.message.contains("removed text"),
            "{error}"
        );
    }
}

#[test]
fn zero_width_comment_on_deleted_empty_paragraph_refuses() {
    let s = docx(&(para("") + &para("Last")));
    for track in ["on", "off"] {
        let mut q = plan(
            &s,
            json!([
                {"kind":"delete_paragraph","target":p(0)},
                {"kind":"comment","target":p(0),"text":"Empty anchor"}
            ]),
        );
        q["policy"]["track"] = json!(track);
        let error = refused(&s, &q);
        assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
        assert_eq!(error.operation.as_deref(), Some("/operations/1"));
        assert!(error.message.contains("entire comment anchor"), "{error}");
    }
}

#[test]
fn adjacent_replacement_disables_tracked_anchor_deletion_preservation() {
    let s = docx(&para("Alpha Beta"));
    let with = run(
        &s,
        &plan(
            &s,
            json!([{"kind":"comment","target":p(0),"find":"Beta","text":"Keep"}]),
        ),
    )
    .unwrap()
    .clean;
    for reverse in [false, true] {
        let mut ops = vec![
            replace(0, "Alpha ", "X "),
            json!({"kind":"delete","target":p(0),"find":"Beta"}),
        ];
        if reverse {
            ops.reverse();
        }
        let error = refused(&with, &plan(&with, json!(ops)));
        assert_eq!(error.code, "COMMENT_ANCHOR_REMOVED", "{error}");
        assert_eq!(
            error.operation.as_deref(),
            Some(if reverse {
                "/operations/0"
            } else {
                "/operations/1"
            })
        );
        assert!(
            error.message.contains("no supported preservation rule"),
            "{error}"
        );
    }
}

#[test]
fn requested_containing_comment_survives_interior_paragraph_deletion() {
    let s = docx(&(para("First") + &para("Middle") + &para("Last")));
    for track in ["on", "off"] {
        for reverse in [false, true] {
            let mut ops = vec![
                json!({"kind":"delete_paragraph","target":p(1)}),
                json!({"kind":"comment","target":p(0),"through":p(2),"text":"All"}),
            ];
            if reverse {
                ops.reverse();
            }
            let mut q = plan(&s, json!(ops));
            q["policy"]["track"] = json!(track);
            let out = run(&s, &q).unwrap();
            for bytes in [&out.clean, &out.redline] {
                let final_bytes = accept_changes(bytes, &mine()).unwrap();
                let comments = jubarte::comments::list_comments(&final_bytes).unwrap();
                assert_eq!(comments.len(), 1);
                assert_eq!(comments[0].anchor_text, "First\nLast");
                assert_word_valid_package(bytes);
            }
        }
    }
}
