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
/// [`plan`] that may change another author's comment: the comment tests
/// that call it edit "Other"'s `c0` on purpose (#1264).
fn rewriting_plan(bytes: &[u8], operations: Value) -> Value {
    let mut request = plan(bytes, operations);
    request["rewrite_others_comments"] = true.into();
    request
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
fn rewrite_survives_empty_cell_repair_without_a_vanished_source() {
    let s = docx(&format!(
        r#"{}<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="1000"/></w:tblGrid>
        <w:tr><w:tc><w:tcPr/>
        <w:customXmlDelRangeStart w:id="9" {STAMP}/>
        <w:sdt><w:sdtPr><w:id w:val="10"/></w:sdtPr><w:sdtContent/></w:sdt>
        <w:customXmlDelRangeEnd w:id="9"/>
        </w:tc></w:tr></w:tbl>{}<w:p>{}</w:p>"#,
        para("Before"),
        para("After"),
        insertion(7, "Trigger")
    ));
    assert_eq!(texts(&s), ["Before", "After", "Trigger"]);
    let mut q = plan(&s, json!([rewrite(0, "Before", "Correct")]));
    q["policy"]["existing_revisions"] = json!("accept");
    let result = run(&s, &q);
    assert!(result.is_ok(), "rewrite failed: {result:?}");
    let out = result.unwrap();
    assert_eq!(texts(&out.clean), ["Correct", "", "After", "Trigger"]);
    assert_word_valid_package(&out.redline);
    assert_word_valid_package(&out.clean);
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
        r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="1234ABCD"><w:r><w:t>Native</w:t></w:r></w:p>"#,
    );
    let mut q = plan(&s, json!([replace(0, "Native", "Changed")]));
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.w14:paraId.1234ABCD");
    assert_eq!(texts(&run(&s, &q).unwrap().clean), ["Changed"]);
    q["source"]["sha256"] = json!("0".repeat(64));
    assert_eq!(run(&s, &q).unwrap_err().code, "STALE_SOURCE");
    // An out-of-range paraId is not a native key: [MS-DOCX] keeps paraIds
    // below 0x80000000, which is exactly the shared key policy, so only the
    // `j:short` pairing identifies such a paragraph on both sides.
    q["source"]["sha256"] = json!(source_sha256(&s));
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.w14:paraId.ABCD1234");
    assert_eq!(run(&s, &q).unwrap_err().code, "TARGET_ID_MISMATCH");
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
        let q = rewriting_plan(
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
fn editing_a_tracked_span_preserves_pending_revisions() {
    let s = source();
    for track in ["on", "off"] {
        let mut q = plan(&s, json!([replace(0, "45", "60")]));
        q["policy"]["track"] = json!(track);
        let out = run(&s, &q);
        assert!(out.is_ok(), "edit must succeed: {:?}", out.err());
        let out = out.unwrap();
        assert_word_valid_package(&out.redline);
        assert_eq!(texts(&out.clean)[0], "Pay 60 days, Delaware law.");
        assert!(
            list_changes(&out.redline)
                .unwrap()
                .iter()
                .any(|c| c.author.as_deref() == Some("Other"))
        );
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
        &rewriting_plan(
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
        let error = refused(&with, &rewriting_plan(&with, ops.clone()));
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
        &rewriting_plan(
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
                let error = refused(&with, &rewriting_plan(&with, json!(ops)));
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
        &rewriting_plan(
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
fn empty_source_paragraph_takes_the_text_and_its_attached_comment() {
    // An empty source paragraph takes text as Word types it (#1263), and
    // the attached comment covers the new words.
    let s = docx(&para(""));
    let mut op = rewrite(0, "", "New paragraph text");
    op["attached_comment"] = json!({"text":"Complete paragraph"});
    let out = run(&s, &plan(&s, json!([op]))).unwrap();
    assert_eq!(texts(&out.clean), ["New paragraph text"]);
    let comments = jubarte::comments::list_comments(&out.redline).unwrap();
    assert_eq!(
        comments
            .iter()
            .map(|c| (c.text.as_str(), c.anchor_text.as_str()))
            .collect::<Vec<_>>(),
        [("Complete paragraph", "New paragraph text")]
    );
    assert_word_valid_package(&out.redline);
    assert_word_valid_package(&out.clean);
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
        let error = refused(&s, &rewriting_plan(&s, ops));
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
fn delete_paragraph_with_kept_tracked_changes_nests_in_their_insertion() {
    // #1136: the paragraph goes; Other's "45" keeps its insertion with the
    // deletion nested in it.
    let s = source();
    let q = plan(&s, json!([{"kind":"delete_paragraph","target":p(0)}]));
    let out = run(&s, &q).unwrap();
    assert_word_valid_package(&out.redline);
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(
        red.contains(r#"<w:ins w:id="7" w:author="Other""#)
            && red.contains("<w:delText>45</w:delText>"),
        "{red}"
    );
    let rejected = reject_changes(
        &out.redline,
        &ChangeFilter {
            authors: Some(vec![out.report.author.clone()]),
            ..ChangeFilter::default()
        },
    )
    .unwrap();
    assert_eq!(
        paragraphs(&rejected).unwrap()[0].text,
        paragraphs(&s).unwrap()[0].text
    );
}

#[test]
fn delete_paragraph_holding_a_move_names_it_in_v2_syntax() {
    let s = docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Pay </w:t></w:r><w:moveTo w:id="7" {STAMP}><w:r><w:t>45</w:t></w:r></w:moveTo></w:p>{}"#,
        para("Last.")
    ));
    let q = plan(&s, json!([{"kind":"delete_paragraph","target":p(0)}]));
    let e = run(&s, &q).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert_eq!(e.operation.as_deref(), Some("/operations/0"));
    assert!(
        e.message.starts_with(
            r#"under keep, a paragraph that holds a move cannot be deleted yet: the move is r7, "45" moved here by "Other"."#
        ),
        "{}",
        e.message
    );
    assert!(
        e.message
            .contains(r#"{"short_id":"r7","long_id":"document.w:moveTo.w:id.7"}"#),
        "{}",
        e.message
    );
}

#[test]
fn delete_paragraph_leaves_valid_containers() {
    // As Word does: deleting every paragraph of the body or of a cell
    // empties the last one, whose mark stays.
    let s = docx(&(para("First") + &para("Last")));
    let q = plan(
        &s,
        json!([
            {"kind":"delete_paragraph","target":p(0)},
            {"kind":"delete_paragraph","target":p(1)}
        ]),
    );
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), [""]);
    assert_eq!(texts(&accept_changes(&out.redline, &mine()).unwrap()), [""]);
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["First", "Last"]
    );
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);

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
    let out = run(&s, &q).unwrap();
    assert_eq!(texts(&out.clean), ["", "Outside"]);
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        ["", "Outside"]
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        ["Cell first", "Cell last", "Outside"]
    );
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
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
        let mut q = rewriting_plan(&s, json!([op]));
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
    let mut q = rewriting_plan(
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
fn explicit_insert_preserves_requested_side_inside_tracked_text() {
    let s = source();
    for track in ["on", "off"] {
        for (anchor, expected) in [
            ("after_text", "Pay 45X days, Delaware law."),
            ("before_text", "Pay X45 days, Delaware law."),
        ] {
            let mut op = json!({"kind":"insert","target":p(0),"text":"X"});
            op[anchor] = json!("45");
            let mut q = plan(&s, json!([op]));
            q["policy"]["track"] = json!(track);
            let out = run(&s, &q);
            assert!(
                out.is_ok(),
                "requested side must be editable: {:?}",
                out.err()
            );
            let out = out.unwrap();
            assert_eq!(texts(&out.clean)[0], expected);
            assert_word_valid_package(&out.redline);
            if track == "on" {
                assert_eq!(
                    texts(&reject_changes(&out.redline, &mine()).unwrap()),
                    texts(&s)
                );
            }
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
    // In `1..=0x7FFFFFFF` ([MS-DOCX]): a native key under the shared policy.
    let native = |text: &str| {
        format!(
            r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="1234ABCD">{text}</w:p>"#
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
    q["source_context"] = json!([{"kind":"paragraph","target":{"short_id":"header1","long_id":"header1.w:p.w14:paraId.1234ABCD"},"text":"Header","reason_codes":["CONTEXT"]}]);
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
    q["operations"][0]["target"]["long_id"] = json!("document.w:p.w14:paraId.1234ABCD");
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

// V2-12: exercise generation and consumption through the production CLI.
mod generator {
    use super::*;
    use std::{
        path::Path,
        process::{Command, Output},
    };
    const EXAMPLE: &[u8] = include_bytes!("fixtures/edit-v2/example-source.docx");
    const DATE: &str = "2026-10-17T07:00:00Z";

    fn cli(args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .output()
            .unwrap()
    }
    fn value(output: &Output) -> Value {
        serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "{e}: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        })
    }
    fn ok(output: &Output) -> Value {
        assert!(
            output.status.success(),
            "stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        value(output)
    }
    fn path(p: &Path) -> &str {
        p.to_str().unwrap()
    }
    fn generate(source: &[u8], flags: &[&str]) -> Value {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, source).unwrap();
        let mut args = vec![
            "edit",
            "template",
            path(&input),
            "-o",
            "-",
            "--datetime",
            DATE,
        ];
        args.extend_from_slice(flags);
        ok(&cli(&args))
    }
    fn saved(dir: &Path, source: &[u8]) -> (std::path::PathBuf, std::path::PathBuf, Value) {
        let input = dir.join("source.docx");
        let plan = dir.join("edits.json");
        std::fs::write(&input, source).unwrap();
        let receipt = ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&plan),
            "--datetime",
            DATE,
            "--json",
        ]));
        (input, plan, receipt)
    }
    #[test]
    fn generate_unchanged_plain_paragraph_matches_design() {
        let actual = generate(EXAMPLE, &[]);
        let expected: Value =
            serde_json::from_str(include_str!("fixtures/edit-v2/template.generated.json")).unwrap();
        assert_eq!(
            actual, expected,
            "fixed datetime and identical fixture bytes require exact design parity"
        );
        assert!(!actual.to_string().contains("run_operation"));
    }
    #[test]
    fn generate_change_apply_inspect_documented_final_text() {
        let dir = tempfile::tempdir().unwrap();
        let (input, plan_path, receipt) = saved(dir.path(), EXAMPLE);
        assert_eq!(receipt["data"]["phase"], "generated");
        assert_eq!(receipt["data"]["effect"], "not_applied");
        assert_eq!(receipt["artifacts"][0]["media_role"], "plan");
        assert_eq!(
            receipt["artifacts"][0]["sha256"],
            source_sha256(&std::fs::read(&plan_path).unwrap())
        );
        let mut plan: Value = serde_json::from_slice(&std::fs::read(&plan_path).unwrap()).unwrap();
        plan["operations"][0]["text"] = json!("Payment is due within 45 days.");
        plan["operations"][1]["text"] = json!("The fee is due; the charge is payable.");
        let anchor = plan["operations"][0]["target"].clone();
        plan["operations"].as_array_mut().unwrap().push(json!({"kind":"insert_paragraph", "target":anchor, "position":"after", "text":"Invoices must state the payment due date."}));
        std::fs::write(&plan_path, plan.to_string()).unwrap();
        let output = dir.path().join("revised.docx");
        let result = ok(&cli(&[
            "edit",
            "apply",
            path(&input),
            "--plan",
            path(&plan_path),
            "-o",
            path(&output),
            "--json",
        ]));
        assert_eq!(result["data"]["effect"], "changed");
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(
            texts(&bytes),
            [
                "Payment is due within 45 days.",
                "Invoices must state the payment due date.",
                "The fee is due; the charge is payable.",
                "One  space.",
                "Notice is due within 30\u{a0}days."
            ]
        );
        assert_word_valid_package(&bytes);
        let changes = list_changes(&bytes).unwrap();
        assert!(!changes.is_empty());
        assert!(
            changes
                .iter()
                .all(|c| c.author.as_deref() == Some("Jubarte"))
        );
        let xml = part_string(&bytes, "word/document.xml").unwrap();
        assert!(xml.contains("w:ins") && xml.contains("w:del"));
        assert_eq!(std::fs::read(&input).unwrap(), EXAMPLE);
        assert_eq!(
            std::fs::read_to_string(&plan_path).unwrap(),
            plan.to_string()
        );
    }
    #[test]
    fn unchanged_copy_and_require_change_are_distinct() {
        let dir = tempfile::tempdir().unwrap();
        let (input, plan, _) = saved(dir.path(), EXAMPLE);
        let output = dir.path().join("copy.docx");
        let args = [
            "edit",
            "apply",
            path(&input),
            "--plan",
            path(&plan),
            "-o",
            path(&output),
            "--json",
        ];
        let receipt = ok(&cli(&args));
        assert_eq!(receipt["data"]["phase"], "committed");
        assert_eq!(receipt["data"]["effect"], "unchanged");
        assert_eq!(std::fs::read(&output).unwrap(), EXAMPLE);
        std::fs::remove_file(&output).unwrap();
        let mut required = args.to_vec();
        required.push("--require-change");
        let refusal = cli(&required);
        assert!(!refusal.status.success());
        assert_eq!(value(&refusal)["error"]["code"], "NO_CHANGES_REQUESTED");
        assert!(!output.exists());
    }
    #[test]
    fn selection_and_source_context_remain_bound_to_original() {
        let plan = generate(EXAMPLE, &["--select", "p0,p0-p1", "--context", "1"]);
        assert_eq!(plan["operations"].as_array().unwrap().len(), 2);
        assert_eq!(plan["scope"]["kind"], "selected");
        assert_eq!(plan["scope"]["targets"], json!([p(0), p(1)]));
        assert_eq!(plan["source_context"][0]["target"], p(2));
        assert_eq!(plan["source_context"][0]["text"], "One  space.");
        let mut q = plan.clone();
        q["source_context"][0]["text"] = json!("Changed observation");
        assert_eq!(run(EXAMPLE, &q).unwrap_err().code, "SOURCE_CONTEXT_CHANGED");
        q.as_object_mut().unwrap().remove("source_context");
        assert_eq!(run(EXAMPLE, &q).unwrap().redline, EXAMPLE);
        // Supplied coverage cannot authorize a direct target outside scope.
        q["coverage"]["eligible_paragraphs"] = json!(999);
        q["operations"]
            .as_array_mut()
            .unwrap()
            .push(replace(3, "Notice", "Wrong"));
        assert_eq!(run(EXAMPLE, &q).unwrap_err().code, "OUT_OF_SCOPE");
    }
    #[test]
    fn table_traversal_deduplicates_canonical_cell_paragraphs() {
        let s = docx(
            &(para("Before")
                + r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>Cell A</w:t></w:r></w:p><w:p><w:r><w:t>Cell B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
                + &para("After")),
        );
        let q = generate(&s, &["--select", "t0,t0.row0.cell0,p1,p2"]);
        assert_eq!(q["operations"].as_array().unwrap().len(), 2);
        assert_eq!(q["operations"][0]["target"], p(1));
        assert_eq!(q["operations"][1]["target"], p(2));
        let mut q = q;
        q["operations"][1]["text"] = json!("New cell");
        assert_eq!(
            texts(&run(&s, &q).unwrap().clean),
            ["Before", "Cell A", "New cell", "After"]
        );
    }
    #[test]
    fn conditional_rows_use_shared_readiness_and_mixed_executor() {
        let s = source();
        let dir = tempfile::tempdir().unwrap();
        let (input, plan_path, receipt) = saved(dir.path(), &s);
        assert_eq!(
            receipt["data"]["generation"]["coverage"]["conditional_paragraphs"],
            2
        );
        let mut q: Value = serde_json::from_slice(&std::fs::read(plan_path).unwrap()).unwrap();
        assert_eq!(q["operations"][0]["text"], "Pay 45 days, Delaware law.");
        q["operations"][0]["text"] = json!("Pay 60 days, Delaware law.");
        let untampered = run(&s, &q);
        assert!(
            untampered.is_ok(),
            "editing a pending insertion must succeed"
        );
        q["coverage"]["conditional_paragraphs"] = json!(0);
        let out = run(&s, &q).unwrap();
        assert_eq!(
            texts(&out.clean),
            texts(&untampered.unwrap().clean),
            "coverage is observation only"
        );
        let q = generate(&s, &["--select", "p0", "--accept-change", "r7"]);
        assert_eq!(q["resolve_revisions"]["accept"], json!([r(7)]));
        let mut q = q;
        q["operations"].as_array_mut().unwrap().extend([replace(0,"45","60"), json!({"kind":"replace","target":p(0),"find":"Delaware","text":"New York","attached_comment":{"text":"Agreed"}})]);
        let out = run(&s, &q).unwrap();
        assert_eq!(texts(&out.clean)[0], "Pay 60 days, New York law.");
        assert_eq!(
            jubarte::comments::list_comments(&out.redline)
                .unwrap()
                .len(),
            1
        );
        assert!(
            list_changes(&out.redline)
                .unwrap()
                .iter()
                .any(|c| c.id == "r8")
        );
        assert_eq!(std::fs::read(input).unwrap(), s);
    }
    #[test]
    fn source_context_survives_changed_decisions_and_stale_baselines_refuse() {
        let s = source();
        let mut q = generate(&s, &["--find", "Last", "--context", "2"]);
        q["resolve_revisions"]["reject"] = json!([r(7)]);
        assert!(
            run(&s, &q).is_ok(),
            "original source observations do not become working observations"
        );
        let mut q = generate(&s, &["--select", "p0"]);
        q["resolve_revisions"]["reject"] = json!([r(7)]);
        assert_eq!(run(&s, &q).unwrap_err().code, "WORKING_BASE_CHANGED");
    }

    fn arguments<'a>(argv: &[&'a str], flag: &str) -> Vec<&'a str> {
        let mut args = argv.iter().copied();
        let mut values = Vec::new();
        while let Some(arg) = args.next() {
            let (key, value) = arg
                .split_once('=')
                .unwrap_or_else(|| (arg, args.next().expect("flag must have a value")));
            if key == flag {
                values.push(value);
            }
        }
        values
    }

    #[test]
    fn arguments_keeps_equals_values_separate_from_following_flags() {
        let argv = ["--author=John=Doe", "--expect-sha256", "digest"];
        assert_eq!(arguments(&argv, "--author"), ["John=Doe"]);
        assert_eq!(arguments(&argv, "--expect-sha256"), ["digest"]);
        assert!(arguments(&argv, "--track").is_empty());
    }

    fn regenerate_stale_rows(s: &[u8], q: &Value, ids: &str, expected: &[&str]) -> Value {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source ' $(literal).docx");
        let plan_path = dir.path().join("edits ; draft.json");
        let previous_regeneration = plan_path.with_extension("regenerated.json");
        std::fs::write(&previous_regeneration, "previous recovery").unwrap();
        let output = dir.path().join("revised.docx");
        std::fs::write(&input, s).unwrap();
        let original_plan = q.to_string();
        std::fs::write(&plan_path, &original_plan).unwrap();
        let out = cli(&[
            "edit",
            "apply",
            path(&input),
            "--plan",
            path(&plan_path),
            "-o",
            path(&output),
            "--json",
        ]);
        assert_eq!(out.status.code(), Some(3));
        let receipt = value(&out);
        let error = &receipt["error"];
        assert_eq!(error["code"], "WORKING_BASE_CHANGED");
        assert_eq!(error["commit_state"], "not_committed");
        assert!(
            error["message"]
                .as_str()
                .unwrap()
                .contains(&source_sha256(expected[0].as_bytes()))
        );
        assert!(!output.exists());
        let actions = error["next_actions"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(actions.len(), 1, "{error}");
        let action = &actions[0];
        let argv: Vec<_> = action["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a.as_str().unwrap())
            .collect();
        assert_eq!(&argv[..3], ["jubarte", "edit", "template"]);
        assert_eq!(
            Path::new(argv[3]).canonicalize().unwrap(),
            input.canonicalize().unwrap()
        );
        let arguments = |flag| arguments(&argv[4..], flag);
        assert_eq!(arguments("--select"), [ids]);
        for (key, flag) in [("accept", "--accept-change"), ("reject", "--reject-change")] {
            let decisions: Vec<_> = q["resolve_revisions"][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["short_id"].as_str().unwrap())
                .collect();
            assert_eq!(arguments(flag), decisions);
        }
        assert_eq!(
            arguments("--existing-revisions"),
            [q["policy"]["existing_revisions"].as_str().unwrap()]
        );
        let policy_index = argv
            .iter()
            .position(|arg| *arg == "--existing-revisions")
            .unwrap();
        let author = format!("--author={}", q["author"].as_str().unwrap());
        assert_eq!(
            &argv[policy_index..argv.len() - 2],
            [
                "--existing-revisions",
                q["policy"]["existing_revisions"].as_str().unwrap(),
                "--track",
                q["policy"]["track"].as_str().unwrap(),
                &author,
                "--datetime",
                q["date"].as_str().unwrap(),
                "--expect-sha256",
                q["source"]["sha256"].as_str().unwrap(),
            ]
        );
        assert_eq!(arguments("--author"), [q["author"].as_str().unwrap()]);
        assert_eq!(
            arguments("--expect-sha256"),
            [q["source"]["sha256"].as_str().unwrap()]
        );
        assert!(
            !argv
                .iter()
                .any(|arg| matches!(*arg, "--force" | "--accept-all" | "--accept-all-changes"))
        );
        let new_paths = arguments("-o");
        assert_eq!(new_paths.len(), 1);
        let regenerated_path = Path::new(new_paths[0]);
        assert!(!regenerated_path.exists());
        assert_ne!(regenerated_path, plan_path);
        let generated = cli(&argv[1..]);
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );
        let regenerated: Value =
            serde_json::from_slice(&std::fs::read(regenerated_path).unwrap()).unwrap();
        let rows = regenerated["operations"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len());
        let targets: Vec<_> = rows
            .iter()
            .map(|row| row["target"]["short_id"].as_str().unwrap())
            .collect();
        assert_eq!(targets.join(","), ids);
        for (row, text) in rows.iter().zip(expected) {
            assert_eq!(row["text"], *text);
            assert_eq!(row["before_sha256"], source_sha256(text.as_bytes()));
        }
        for field in ["source", "policy", "resolve_revisions", "author", "date"] {
            assert_eq!(
                regenerated[field], q[field],
                "regeneration must preserve {field}"
            );
        }
        assert!(run(s, &regenerated).is_ok());
        assert_eq!(std::fs::read(&input).unwrap(), s);
        assert_eq!(std::fs::read_to_string(&plan_path).unwrap(), original_plan);
        assert_eq!(
            std::fs::read_to_string(previous_regeneration).unwrap(),
            "previous recovery"
        );
        assert!(!output.exists());
        action.clone()
    }

    #[test]
    fn working_base_changed_regenerates_only_stale_payment_rows() {
        let s = docx(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">Payment is due within </w:t></w:r><w:del w:id="6" {STAMP}><w:r><w:delText>30</w:delText></w:r></w:del>{}<w:r><w:t> days.</w:t></w:r></w:p><w:p>{}</w:p>{}"#,
            insertion(7, "45"),
            insertion(8, "Unrelated"),
            para("Last.")
        ));
        let mut q = generate(&s, &["--author=-Reviewer"]);
        q["operations"][0]["text"] = json!("Authored payment terms");
        q["resolve_revisions"]["reject"] = json!([
            {"short_id":"r6","long_id":"document.w:del.w:id.6"}, r(7)
        ]);
        q["resolve_revisions"]["accept"] = json!([r(8)]);
        for policy in ["keep", "accept", "refuse"] {
            q["policy"]["existing_revisions"] = json!(policy);
            let action = regenerate_stale_rows(&s, &q, "p0", &["Payment is due within 30 days."]);
            assert_eq!(
                action["purpose"],
                "regenerate a new plan containing only the stale rows with their current working text; the original plan file is not changed; merge those rows into the original plan by hand"
            );
        }
    }

    #[test]
    fn working_base_changed_regeneration_keeps_source_ids_after_merge() {
        let s = docx(&format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {STAMP}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>{}<w:p>{}</w:p><w:p>{}</w:p>{}"#,
            para("Beta"),
            insertion(7, "First"),
            insertion(8, "Second"),
            para("Last.")
        ));
        let mut q = generate(
            &s,
            &[
                "--select", "p2-p4", "--track", "off", "--author", "Reviewer",
            ],
        );
        q["resolve_revisions"]["accept"] =
            json!([{"short_id":"r1","long_id":"document.w:del.w:id.1"}]);
        q["policy"]["existing_revisions"] = json!("reject");
        let duplicate = q["operations"][0].clone();
        q["operations"].as_array_mut().unwrap().push(duplicate);
        let action = regenerate_stale_rows(&s, &q, "p2,p3", &["", ""]);
        assert_eq!(
            action["prerequisites"],
            json!([format!(
                "the source file must still have the plan's recorded sha256 {} (--expect-sha256)",
                q["source"]["sha256"].as_str().unwrap()
            )])
        );
    }

    #[test]
    fn working_base_changed_regenerates_story_qualified_header_rows() {
        use common::docx::{Part, docx_with_sect};
        let header = format!(
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}<w:p>{}</w:p><w:p>{}</w:p><w:p>{}</w:p></w:hdr>"#,
            para("Header"),
            insertion(7, "First"),
            insertion(8, "Second"),
            insertion(9, "Third")
        );
        let s = docx_with_sect(
            &para("Body"),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        );
        let mut q = generate(&s, &["--story", "header1", "--author=John=Doe"]);
        q["resolve_revisions"]["reject"] = json!(
            (7..=9)
                .map(|n| json!({
                    "short_id": format!("header1.r{n}"),
                    "long_id": format!("header1.w:ins.w:id.{n}")
                }))
                .collect::<Vec<_>>()
        );
        regenerate_stale_rows(&s, &q, "header1.p1,header1.p2,header1.p3", &["", "", ""]);
    }
    #[test]
    fn unrepresentable_and_removed_rows_are_inventory_not_rewrites() {
        let s = docx(
            &(r#"<w:p><w:r><w:t>Text</w:t><w:drawing/></w:r></w:p>"#.to_string() + &para("Plain")),
        );
        let q = generate(&s, &[]);
        assert_eq!(q["operations"].as_array().unwrap().len(), 1);
        assert_eq!(q["coverage"]["opaque_regions"], 1);
        assert!(
            q["source_context"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["kind"] == "opaque" && c["target"] == p(0))
        );
        assert_eq!(run(&s, &q).unwrap().redline, s);
        let only_opaque = generate(&s, &["--select", "p0"]);
        assert_eq!(only_opaque["operations"], json!([]));
        assert_eq!(only_opaque["coverage"]["eligible_paragraphs"], 0);
        let s = docx(&format!(
            "{}<w:p><w:pPr><w:rPr><w:del w:id=\"8\" {STAMP}/></w:rPr></w:pPr><w:del w:id=\"9\" {STAMP}><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>{}",
            para("start"),
            para("end")
        ));
        let q = generate(&s, &["--existing-revisions", "accept"]);
        // The following paragraph loses its exact tag too: the strict V2-06
        // correspondence excludes it even though its text still says "end".
        assert_eq!(q["operations"].as_array().unwrap().len(), 1);
        assert_eq!(q["operations"][0]["target"], p(0));
        assert!(q["source_context"].as_array().unwrap().iter().any(|c| {
            c["target"] == p(2)
                && c["reason_codes"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("merged"))
        }));
        assert!(q["source_context"].as_array().unwrap().iter().any(|c| {
            c["target"] == p(1)
                && c["reason_codes"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("removed"))
        }));
    }
    #[test]
    fn plan_publication_receipt_dry_run_and_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let (input, plan, _) = saved(dir.path(), EXAMPLE);
        let before = std::fs::read(&plan).unwrap();
        let receipt = ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&plan),
            "--dry-run",
            "--json",
        ]));
        assert_eq!(receipt["data"]["phase"], "generated");
        assert_eq!(receipt["data"]["effect"], "not_applied");
        assert!(
            serde_json::from_value::<jubarte::result::Envelope>(receipt.clone())
                .unwrap()
                .artifacts
                .is_empty()
        );
        assert_eq!(std::fs::read(&plan).unwrap(), before);
        let receipt = cli(&["edit", "template", path(&input), "--dry-run", "--jsonl"]);
        assert!(receipt.status.success());
        let stream =
            jubarte::result::read_jsonl_stream(std::str::from_utf8(&receipt.stdout).unwrap())
                .unwrap();
        assert_eq!(
            stream.terminal.data.unwrap().phase,
            Some(jubarte::result::Phase::Generated)
        );
        let result = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&plan),
            "--json",
        ]);
        assert_eq!(value(&result)["error"]["code"], "OUTPUT_EXISTS");
        ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&plan),
            "--force",
            "--json",
        ]));
        assert!(std::fs::read_dir(dir.path()).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .contains("jubarte-tmp")
        }));
    }
    #[test]
    fn empty_selection_and_output_alias_never_publish() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        let output = dir.path().join("plan.json");
        std::fs::write(&input, EXAMPLE).unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "--find",
            "absent",
            "-o",
            path(&output),
            "--json",
        ]);
        assert_eq!(value(&out)["error"]["code"], "EMPTY_SELECTION");
        assert!(!output.exists());
        let alias = dir.path().join("alias.json");
        std::fs::hard_link(&input, &alias).unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&alias),
            "--force",
            "--json",
        ]);
        assert_eq!(value(&out)["error"]["code"], "OUTPUT_ALIAS");
        assert_eq!(std::fs::read(input).unwrap(), EXAMPLE);
    }
    #[test]
    fn generator_output_forms_reject_activation_and_plan_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, EXAMPLE).unwrap();
        for extra in [
            vec!["-o", "-", "--json"],
            vec!["-o", "-", "--jsonl"],
            vec!["-o", "-", "--dry-run"],
            vec!["--runs", "--json"],
            vec!["--scaffold", "delete", "--json"],
            vec!["--format", "jsonl", "--json"],
            vec!["--json"],
        ] {
            let mut args = vec!["edit", "template", path(&input)];
            args.extend(extra);
            let out = cli(&args);
            assert!(!out.status.success());
            if args.contains(&"--json") || args.contains(&"--jsonl") {
                let receipt = if args.contains(&"--jsonl") {
                    serde_json::to_value(
                        jubarte::result::read_jsonl_stream(
                            std::str::from_utf8(&out.stdout).unwrap(),
                        )
                        .unwrap()
                        .terminal,
                    )
                    .unwrap()
                } else {
                    value(&out)
                };
                assert_eq!(receipt["error"]["code"], "USAGE");
            }
        }
    }
    #[test]
    fn generator_covers_all_stories_and_complete_long_selected_scope() {
        use common::docx::{Part, docx_with_sect};
        let s = docx_with_sect(
            &para("Body"),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Header</w:t></w:r></w:p></w:hdr>"#,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        );
        let q = generate(&s, &[]);
        assert_eq!(q["scope"], json!({"kind":"all"}));
        assert_eq!(q["operations"].as_array().unwrap().len(), 2);
        let q = generate(&s, &["--story", "header1"]);
        assert_eq!(q["operations"].as_array().unwrap().len(), 1);
        assert_eq!(q["operations"][0]["text"], "Header");
        assert_eq!(q["scope"]["kind"], "selected");
        assert_eq!(run(&s, &q).unwrap().redline, s);
        let q = generate(&s, &["--select", "header1"]);
        assert_eq!(q["operations"].as_array().unwrap().len(), 1);
        let s = docx(
            &(0..130)
                .map(|n| para(&format!("{n}: {}", "words ".repeat(100))))
                .collect::<String>(),
        );
        let q = generate(&s, &[]);
        assert_eq!(
            q["operations"].as_array().unwrap().len(),
            130,
            "read presentation limits cannot truncate a plan"
        );
        assert_eq!(
            q["operations"][129]["text"],
            format!("129: {}", "words ".repeat(100))
        );
        assert_eq!(q["coverage"]["traversal_complete"], true);
    }
    #[test]
    fn design_edited_example_remains_valid_for_bound_source() {
        for (sample, expected) in [
            (
                include_str!("fixtures/edit-v2/template.edited.json"),
                "Payment is due within 45 days.",
            ),
            (
                include_str!("fixtures/edit-v2/template.exact-replace.json"),
                "Payment is due within 45 days.",
            ),
            (
                include_str!("fixtures/edit-v2/template.with-comment.json"),
                "Payment is due within 45 days.",
            ),
            (
                include_str!("fixtures/edit-v2/template.composed.json"),
                "Payment is due within 30 days.",
            ),
        ] {
            let q: Value = serde_json::from_str(sample).unwrap();
            let output = run(EXAMPLE, &q).unwrap();
            assert_eq!(texts(&output.clean)[0], expected);
            assert_word_valid_package(&output.redline);
        }
    }
    #[test]
    fn merged_selection_is_read_only_and_removed_rows_request_no_mutation() {
        let s = merged_source();
        let q = generate(&s, &["--select", "p0,p1", "--accept-change", "r1"]);
        assert_eq!(q["operations"], json!([]));
        assert_eq!(q["coverage"]["context_paragraphs"], 2);
        assert!(q["source_context"].as_array().unwrap().iter().any(|c| {
            c["reason_codes"]
                .as_array()
                .unwrap()
                .contains(&json!("merged"))
        }));
        let mut q = generate(EXAMPLE, &[]);
        q["operations"] = json!([]);
        assert_eq!(run(EXAMPLE, &q).unwrap().redline, EXAMPLE);
        let mut q = generate(EXAMPLE, &["--select", "p0"]);
        q["operations"][0]["text"] = json!("");
        let out = run(EXAMPLE, &q).unwrap();
        assert_eq!(texts(&out.clean).len(), 4);
        assert_eq!(texts(&out.clean)[0], "");
    }
    #[test]
    fn opaque_neighbour_context_is_not_an_exact_text_observation() {
        let s = docx(
            &(r#"<w:p><w:r><w:t>Opaque</w:t><w:drawing/></w:r></w:p>"#.to_string()
                + &para("Selected")),
        );
        let q = generate(&s, &["--select", "p1", "--context", "1"]);
        assert_eq!(
            run(&s, &q).unwrap().redline,
            s,
            "a generated plan must verify all its own observations"
        );
        assert!(
            q["source_context"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["target"] == p(0) && c["kind"] == "opaque" && c.get("text").is_none())
        );
        assert_eq!(q["coverage"]["opaque_regions"], 1);
    }
    #[test]
    fn repeated_authors_union_original_marks_before_settlement() {
        let s = docx(&format!(
            "<w:p>{}</w:p><w:p>{}</w:p>{}",
            insertion(7, "Other"),
            insertion(8, "Third").replace("Other", "Third"),
            para("Plain")
        ));
        let q = generate(
            &s,
            &[
                "--changed",
                "--by",
                "Other",
                "--by",
                "Third",
                "--existing-revisions",
                "accept",
            ],
        );
        assert_eq!(q["operations"].as_array().unwrap().len(), 2);
        assert_eq!(q["scope"]["targets"], json!([p(0), p(1)]));
        assert_eq!(q["coverage"]["conditional_paragraphs"], 0);
    }
    #[test]
    fn original_comment_bodies_are_inventory_and_never_mutations() {
        let s = comment_source();
        let q = generate(&s, &[]);
        let context = q["source_context"].as_array().unwrap();
        let comment = context
            .iter()
            .find(|c| c["kind"] == "comment")
            .expect("existing comments are source context");
        assert_eq!(
            comment["text"],
            jubarte::comments::list_comments(&s).unwrap()[0].text
        );
        assert_eq!(run(&s, &q).unwrap().redline, s);
        let mut changed = q.clone();
        changed["source_context"][0]["text"] = json!("Changed comment observation");
        assert_eq!(
            run(&s, &changed).unwrap_err().code,
            "SOURCE_CONTEXT_CHANGED"
        );
        let mut removed = q;
        removed.as_object_mut().unwrap().remove("source_context");
        assert_eq!(run(&s, &removed).unwrap().redline, s);
    }
    #[test]
    fn generated_baselines_are_source_specific_and_readiness_is_not_plan_authority() {
        let s = docx(&para("Different source.\t\nExact whitespace."));
        let mut q = generate(&s, &[]);
        assert_eq!(
            q["operations"][0]["text"],
            "Different source.\t\nExact whitespace."
        );
        assert_eq!(
            q["operations"][0]["before_sha256"],
            source_sha256("Different source.\t\nExact whitespace.".as_bytes())
        );
        assert_eq!(q["source"]["sha256"], source_sha256(&s));
        assert_eq!(run(&s, &q).unwrap().redline, s);
        q["operations"][0]["readiness"] = json!({"status":"available", "editable":true});
        assert_eq!(
            PlanV2::from_json(&q.to_string()).unwrap_err().code,
            "INVALID_PLAN"
        );
    }
    #[test]
    fn human_receipt_report_and_quiet_generation_keep_protocols_distinct() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        let output = dir.path().join("plan.json");
        let report = dir.path().join("receipt.json");
        std::fs::write(&input, EXAMPLE).unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&output),
            "--report",
            path(&report),
        ]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("4 rows, 0 conditional"));
        assert!(
            !text.contains("Payment is due"),
            "receipt must not dump the plan"
        );
        let receipt: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        assert_eq!(receipt["data"]["effect"], "not_applied");
        assert_eq!(
            receipt["artifacts"][0]["sha256"],
            source_sha256(&std::fs::read(&output).unwrap())
        );
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&output),
            "--force",
            "--quiet",
        ]);
        assert!(out.status.success());
        assert!(out.stdout.is_empty());
        let out = cli(&["edit", "template", path(&input), "-o", "-", "--quiet"]);
        assert_eq!(ok(&out)["operations"].as_array().unwrap().len(), 4);
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "--dry-run",
            "--report",
            path(&report),
            "--json",
        ]);
        assert!(out.status.success());
        assert_eq!(value(&out)["data"]["phase"], "generated");
        let stale = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&output),
            "--force",
            "--expect-sha256",
            &"0".repeat(64),
            "--json",
        ]);
        assert_eq!(value(&stale)["error"]["code"], "STALE_SOURCE");
        let alias = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&output),
            "--report",
            path(&output),
            "--json",
        ]);
        assert_eq!(value(&alias)["error"]["code"], "OUTPUT_ALIAS");
        let missing = cli(&[
            "edit",
            "template",
            "/nonexistent/v2-12.docx",
            "--dry-run",
            "--json",
        ]);
        assert_eq!(value(&missing)["error"]["code"], "IO_ERROR");
    }

    #[test]
    fn generated_revision_pairs_are_resolved_not_guessed() {
        let s = source();
        let short = generate(&s, &["--accept-change", "r7", "--track", "off"]);
        let long = generate(
            &s,
            &["--accept-change", "document.w:ins.w:id.7", "--track", "off"],
        );
        assert_eq!(short, long);
        assert_eq!(short["policy"]["track"], "off");
        assert_eq!(texts(&run(&s, &short).unwrap().redline), texts(&s));
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, &s).unwrap();
        for (id, code) in [("r999", "INVALID_TARGET"), ("ps0", "INVALID_ID")] {
            let out = cli(&[
                "edit",
                "template",
                path(&input),
                "--accept-change",
                id,
                "--dry-run",
                "--json",
            ]);
            assert_eq!(value(&out)["error"]["code"], code);
        }
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "--existing-revisions",
            "refuse",
            "--dry-run",
            "--json",
        ]);
        assert_eq!(value(&out)["error"]["code"], "EXISTING_REVISIONS");
    }
    #[test]
    fn raw_plan_stdout_and_explicit_report_have_separate_destinations() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        let report = dir.path().join("receipt.json");
        std::fs::write(&input, EXAMPLE).unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            "-",
            "--report",
            path(&report),
        ]);
        let plan = ok(&out);
        assert_eq!(plan["operations"].as_array().unwrap().len(), 4);
        let receipt: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        assert_eq!(receipt["data"]["phase"], "generated");
        assert_eq!(receipt["data"]["effect"], "not_applied");
        assert!(
            receipt.get("artifacts").is_none(),
            "stdout is not a committed plan file"
        );
    }
    #[test]
    fn compact_receipt_bounds_blocker_references_and_discloses_truncation() {
        let s = docx(&format!(
            "<w:p>{}</w:p>",
            (0..20).map(|n| insertion(n, "x")).collect::<String>()
        ));
        let dir = tempfile::tempdir().unwrap();
        let (_, _, receipt) = saved(dir.path(), &s);
        let facts = &receipt["data"]["generation"];
        let row = &facts["working_readiness"][0];
        assert_eq!(row["blocker_count"], 20);
        assert_eq!(row["blockers"].as_array().unwrap().len(), 8);
        assert!(row["readiness"].as_array().unwrap().iter().all(|r| {
            r.get("blockers")
                .is_none_or(|b| b.as_array().unwrap().len() <= 8)
        }));
        assert_eq!(facts["readiness_complete"], false);
    }

    /// A source whose paragraphs carry the given `w14:paraId` values (plus
    /// one plain paragraph), for the shared native-key policy.
    fn paraid_source(ids: &[&str]) -> Vec<u8> {
        let body: String = ids
            .iter()
            .map(|id| {
                format!(
                    r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="{id}"><w:r><w:t>Paragraph {id}</w:t></w:r></w:p>"#
                )
            })
            .chain(std::iter::once(para("Plain")))
            .collect();
        docx(&body)
    }

    #[test]
    fn native_paraid_templates_apply_untouched_under_the_shared_key_policy() {
        // In range (1..=0x7FFFFFFF, the [MS-DOCX] rule): the native paraId
        // stays the long key on both sides, and the untouched plan applies
        // as a byte-identical copy.
        let in_range = paraid_source(&["1234ABCD"]);
        let q = generate(&in_range, &[]);
        assert_eq!(
            q["operations"][0]["target"]["long_id"],
            "document.w:p.w14:paraId.1234ABCD"
        );
        let out =
            run(&in_range, &q).unwrap_or_else(|e| panic!("an untouched template must apply: {e}"));
        assert_eq!(out.redline, in_range);
        assert_eq!(out.clean, in_range);

        // Out of range: the generator must fall back to `j:short`, and that
        // untouched plan must still apply as a byte-identical copy.
        let out_of_range = paraid_source(&["ABCD1234"]);
        let q = generate(&out_of_range, &[]);
        assert_eq!(
            q["operations"][0]["target"]["long_id"], "document.w:p.j:short.p0",
            "a paraId beyond 0x7FFFFFFF is not a native key"
        );
        let out = run(&out_of_range, &q)
            .unwrap_or_else(|e| panic!("an untouched template must apply: {e}"));
        assert_eq!(out.redline, out_of_range);
        assert_eq!(out.clean, out_of_range);

        // Case-colliding values are one key counted twice, so both fall
        // back to `j:short`; that untouched plan must apply too.
        let colliding = paraid_source(&["1234ABCD", "1234abcd"]);
        let q = generate(&colliding, &[]);
        assert_eq!(
            q["operations"][0]["target"]["long_id"],
            "document.w:p.j:short.p0"
        );
        assert_eq!(
            q["operations"][1]["target"]["long_id"],
            "document.w:p.j:short.p1"
        );
        let out =
            run(&colliding, &q).unwrap_or_else(|e| panic!("an untouched template must apply: {e}"));
        assert_eq!(out.redline, colliding);
        assert_eq!(out.clean, colliding);
    }

    #[test]
    fn revision_wid_beyond_i32_round_trips_the_shared_policy() {
        let s = docx(&format!(
            r#"<w:p><w:r><w:t xml:space="preserve">Pay </w:t></w:r><w:ins w:id="5000000000" {STAMP}><w:r><w:t>45</w:t></w:r></w:ins><w:r><w:t> days.</w:t></w:r></w:p>"#
        ));
        let q = generate(&s, &["--accept-change", "body:rev:5000000000"]);
        assert_eq!(
            q["resolve_revisions"]["accept"][0],
            json!({"short_id": "body:rev:5000000000",
                    "long_id": "document.w:ins.j:short.body:rev:5000000000"})
        );
        let result = run(&s, &q);
        assert!(
            result.is_ok(),
            "the applier must accept the pair its generator froze: {:?}",
            result.err()
        );
        assert_eq!(texts(&result.unwrap().clean), ["Pay 45 days."]);
    }

    #[test]
    fn template_select_keeps_closed_ranges_only() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, EXAMPLE).unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "--select",
            "p1-",
            "--dry-run",
            "--json",
        ]);
        assert_eq!(
            value(&out)["error"]["code"],
            "USAGE",
            "proposal §2 allows only closed ranges for --select"
        );
    }

    #[test]
    fn an_out_of_range_selector_is_usage_not_a_missing_anchor() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, EXAMPLE).unwrap();
        for spec in ["p99", "p0-p99"] {
            let out = cli(&[
                "edit",
                "template",
                path(&input),
                "--select",
                spec,
                "--dry-run",
                "--json",
            ]);
            let error = value(&out);
            assert_eq!(error["error"]["code"], "USAGE", "{spec}");
            assert!(
                error["error"]["message"]
                    .as_str()
                    .is_some_and(|m| m.contains("past the last paragraph")),
                "{spec}: {}",
                error["error"]["message"]
            );
        }
    }

    #[test]
    fn dry_run_receipt_reports_scope_count_and_digest_without_the_scaffold() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, EXAMPLE).unwrap();
        let receipt = ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "--select",
            "p0-p1",
            "--dry-run",
            "--json",
        ]));
        let scope = &receipt["effective_options"]["scope"];
        assert_eq!(scope["kind"], "selected");
        assert_eq!(
            scope["targets"],
            json!(2),
            "the receipt carries a count, not the target list"
        );
        let plan = generate(EXAMPLE, &["--select", "p0-p1"]);
        let digest = source_sha256(serde_json::to_string(&plan["scope"]).unwrap().as_bytes());
        assert_eq!(scope["sha256"], json!(digest));
        let all = ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "--dry-run",
            "--json",
        ]));
        let scope = &all["effective_options"]["scope"];
        assert_eq!(scope["kind"], "all");
        assert!(scope.get("targets").is_none());
        let every = generate(EXAMPLE, &[]);
        assert_eq!(
            scope["sha256"],
            json!(source_sha256(
                serde_json::to_string(&every["scope"]).unwrap().as_bytes()
            ))
        );
    }

    #[test]
    fn identical_inputs_generate_byte_identical_plans() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, EXAMPLE).unwrap();
        let first = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            "-",
            "--datetime",
            DATE,
            "--select",
            "p0-p1",
        ]);
        let second = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            "-",
            "--datetime",
            DATE,
            "--select",
            "p0-p1",
        ]);
        assert!(first.status.success() && second.status.success());
        assert_eq!(
            first.stdout, second.stdout,
            "the same timestamp and source require byte-identical plans"
        );
        let a = dir.path().join("a.json");
        let b = dir.path().join("b.json");
        let first_saved = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&a),
            "--datetime",
            DATE,
        ]);
        let second_saved = cli(&[
            "edit",
            "template",
            path(&input),
            "-o",
            path(&b),
            "--datetime",
            DATE,
        ]);
        assert!(first_saved.status.success() && second_saved.status.success());
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    }

    #[test]
    fn dry_run_writes_neither_plan_nor_report_file() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        let plan = dir.path().join("plan.json");
        let report = dir.path().join("receipt.json");
        std::fs::write(&input, EXAMPLE).unwrap();
        std::fs::write(&plan, b"stale plan").unwrap();
        std::fs::write(&report, b"stale report").unwrap();
        let out = cli(&[
            "edit",
            "template",
            path(&input),
            "--dry-run",
            "-o",
            path(&plan),
            "--report",
            path(&report),
            "--json",
        ]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(std::fs::read(&plan).unwrap(), b"stale plan");
        assert_eq!(std::fs::read(&report).unwrap(), b"stale report");
        let absent = dir.path().join("absent.json");
        ok(&cli(&[
            "edit",
            "template",
            path(&input),
            "--dry-run",
            "--report",
            path(&absent),
            "--json",
        ]));
        assert!(!absent.exists(), "a dry-run writes no report file");
    }

    #[test]
    fn readiness_examples_stop_at_sixteen_rows_but_the_plan_keeps_every_one() {
        let s = docx(
            &(0..17)
                .map(|n| para(&format!("Row {n}")))
                .collect::<String>(),
        );
        let dir = tempfile::tempdir().unwrap();
        let (_, plan_path, receipt) = saved(dir.path(), &s);
        let facts = &receipt["data"]["generation"];
        assert_eq!(facts["working_readiness"].as_array().unwrap().len(), 16);
        assert_eq!(facts["readiness_complete"], false);
        assert_eq!(facts["coverage"]["eligible_paragraphs"], 17);
        let plan: Value = serde_json::from_slice(&std::fs::read(plan_path).unwrap()).unwrap();
        assert_eq!(
            plan["operations"].as_array().unwrap().len(),
            17,
            "presentation limits never truncate the plan"
        );
    }

    #[test]
    fn template_usage_branches_refuse_with_usage() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("source.docx");
        std::fs::write(&input, source()).unwrap();
        let code = |args: &[&str]| value(&cli(args))["error"]["code"].clone();
        assert_eq!(
            code(&[
                "edit",
                "template",
                path(&input),
                "--story",
                "xyz",
                "--dry-run",
                "--json",
            ]),
            json!("USAGE")
        );
        for id in ["c5", "s0", "r7"] {
            assert_eq!(
                code(&[
                    "edit",
                    "template",
                    path(&input),
                    "--select",
                    id,
                    "--dry-run",
                    "--json",
                ]),
                json!("USAGE"),
                "--select of {id}"
            );
        }
        let both = cli(&[
            "edit",
            "template",
            path(&input),
            "--find",
            "x",
            "--regex",
            "y",
            "--dry-run",
            "--json",
        ]);
        assert!(!both.status.success(), "--find with --regex must refuse");
    }
}

#[test]
fn deleting_all_of_their_insertion_lands_tracked_and_direct() {
    // The direct copy has no carrier left for Other's "45"; the tracked one
    // keeps it with the deletion nested in it.
    let s = source();
    for track in ["on", "off"] {
        let mut q = plan(&s, json!([replace(0, "45", "")]));
        q["policy"]["track"] = json!(track);
        let out = run(&s, &q).unwrap_or_else(|e| panic!("track {track}: {e}"));
        assert_word_valid_package(&out.redline);
        assert_eq!(
            paragraphs(&out.clean).unwrap()[0].text,
            "Pay  days, Delaware law.",
            "track {track}"
        );
    }
}
