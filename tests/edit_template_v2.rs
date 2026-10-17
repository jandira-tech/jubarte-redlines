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
        assert_word_valid_package(&out.redline);
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
    q["operations"] = json!([{"kind":"reply_comment","target":c(0),"text":"Reply"}]);
    assert_eq!(run(&s, &q).unwrap_err().code, "UNKNOWN_COMMENT");
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
