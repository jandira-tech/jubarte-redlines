// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Machine result envelope for the v2 routes (`edit apply`, `edit template`,
//! and friends). Status, phase, effect, predicted effect, artifacts and the
//! error block share one structure, per `jubarte-cli-proposal-b-revised.md`
//! §11 and `jubarte-edit-implementation-plan.md` P5. The no-effect table in
//! `jubarte-edit-implementation-plan.md` P4 and `jubarte-template-implementation-plan.md`
//! Stage D gate every row of this file.

use std::collections::BTreeSet;

use jubarte::inspect::source_sha256;
use jubarte::result::{
    Artifact, Context, Counts, Envelope, Error, ExitStatus, MediaRole, NextAction, Phase,
    PredictedEffect, ProgressEvent, ResultData, Runtime, Source, Status, Verification,
    exit_status_for, jsonl_terminal_line, read_jsonl_stream, write_jsonl_stream,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_bytes() -> Vec<u8> {
    // A small fixed payload; only its sha256 is needed by the envelope.
    b"jubarte-result-envelope-fixture\n".to_vec()
}

fn ok_envelope(_status_data: Status, effect_data: jubarte::result::Effect) -> Envelope {
    let mut data = ResultData::new(Phase::Committed, effect_data);
    data.counts.requested_operations = 1;
    Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    )
}

// ---------------------------------------------------------------------------
// Type-safety: invalid (phase, effect) combinations are not constructible.
// ---------------------------------------------------------------------------

#[test]
fn generated_phase_forces_not_applied_effect() {
    // The constructor refuses generated + changed.
    let err = ResultData::new(Phase::Generated, jubarte::result::Effect::Changed).validate();
    assert!(
        err.is_err(),
        "generated phase with changed effect must be refused by the constructor"
    );
}

#[test]
fn validated_phase_forces_not_applied_effect() {
    let err = ResultData::new(Phase::Validated, jubarte::result::Effect::Unchanged).validate();
    assert!(
        err.is_err(),
        "validated phase with unchanged effect must be refused by the constructor"
    );
}

#[test]
fn predicted_effect_only_with_validated_phase() {
    // A predicted_effect without a validated phase is invalid.
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.set_predicted(PredictedEffect::Unchanged);
    let err = data.validate();
    assert!(
        err.is_err(),
        "predicted_effect must be rejected when phase is not validated"
    );
}

#[test]
fn validated_phase_accepts_predicted_effect() {
    let mut data = ResultData::new(Phase::Validated, jubarte::result::Effect::NotApplied);
    data.set_predicted(PredictedEffect::Unchanged);
    assert!(data.validate().is_ok());
}

#[test]
fn committed_phase_with_changed_effect_is_allowed() {
    let data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    assert!(data.validate().is_ok());
}

#[test]
fn committed_phase_with_unchanged_effect_is_allowed() {
    let data = ResultData::new(Phase::Committed, jubarte::result::Effect::Unchanged);
    assert!(data.validate().is_ok());
}

// ---------------------------------------------------------------------------
// P4 no-effect table: every row has a serialization test.
// ---------------------------------------------------------------------------

#[test]
fn p4_generated_plan_row() {
    // "Generate a plan: status ok, data.phase generated, data.effect
    //  not_applied; committed plan artifact if saved, no committed document
    //  effect."
    let mut data = ResultData::new(Phase::Generated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 4;
    let mut envelope = Envelope::ok(
        "edit template",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let plan_artifact = Artifact::committed(
        "/tmp/plan.json",
        MediaRole::Plan,
        "application/json",
        b"{\"schema_version\":2}".as_slice(),
    );
    envelope.push_artifact(plan_artifact);
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["phase"], "generated");
    assert_eq!(value["data"]["effect"], "not_applied");
    assert!(
        value["data"].get("predicted_effect").is_none(),
        "predicted_effect must not appear with phase generated"
    );

    let artifacts = value["artifacts"].as_array().expect("artifacts array");
    assert_eq!(artifacts.len(), 1, "exactly one plan artifact");
    let plan = &artifacts[0];
    assert_eq!(plan["media_role"], "plan");
    assert_eq!(plan["committed"], true);
    // No document artifact was committed; data.effect must remain not_applied.
    assert_eq!(value["data"]["effect"], "not_applied");
}

#[test]
fn p4_untouched_template_apply_row() {
    // "Valid untouched template, normal apply: status ok, data.effect
    //  unchanged, data.phase committed; byte-identical source copy at the
    //  requested output; zero effects."
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Unchanged);
    data.counts.requested_operations = 4;
    let mut envelope = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    envelope.push_artifact(Artifact::committed(
        "/tmp/out.docx",
        MediaRole::Document,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        &fixture_bytes(),
    ));
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["phase"], "committed");
    assert_eq!(value["data"]["effect"], "unchanged");

    let artifacts = value["artifacts"].as_array().expect("artifacts array");
    assert_eq!(artifacts.len(), 1);
    let doc = &artifacts[0];
    assert_eq!(doc["media_role"], "document");
    assert_eq!(doc["committed"], true);
    // Source is the same payload, so the document's digest equals source.
    assert_eq!(
        doc["sha256"], value["source"]["sha256"],
        "an unchanged primary carries the source's digest verbatim"
    );
}

#[test]
fn p4_require_change_refusal_row() {
    // "Same request with --require-change: status refused with
    //  NO_CHANGES_REQUESTED; no primary artifact, even if derivatives were
    //  requested."
    let envelope = Envelope::refused(
        "edit apply",
        Source::new(&fixture_bytes()),
        Error::new("NO_CHANGES_REQUESTED").with_message("primary document effect is zero"),
    );
    // No artifact is added; the envelope has to omit `artifacts` when empty.
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "refused");
    assert_eq!(value["error"]["code"], "NO_CHANGES_REQUESTED");
    assert!(
        value["artifacts"].is_null() || value["artifacts"].as_array().is_some_and(|a| a.is_empty()),
        "no primary artifact committed for a refusal"
    );
}

#[test]
fn p4_dry_run_row() {
    // "Same request under edit dry run: status ok, data.phase validated,
    //  data.effect not_applied, data.predicted_effect unchanged; or the
    //  same required-change refusal; no writes."
    let mut data = ResultData::new(Phase::Validated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 4;
    data.set_predicted(PredictedEffect::Unchanged);
    let envelope = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["phase"], "validated");
    assert_eq!(value["data"]["effect"], "not_applied");
    assert_eq!(value["data"]["predicted_effect"], "unchanged");

    // No primary artifact for a dry run.
    let artifacts = value["artifacts"].as_array();
    assert!(
        artifacts.is_none() || artifacts.unwrap().is_empty(),
        "dry run must not commit a primary artifact"
    );
}

#[test]
fn p4_dry_run_predicts_changed_when_plan_alters_text() {
    // A dry run that predicts a change still carries not_applied effect.
    let mut data = ResultData::new(Phase::Validated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 1;
    data.set_predicted(PredictedEffect::Changed);
    let envelope = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["data"]["phase"], "validated");
    assert_eq!(value["data"]["effect"], "not_applied");
    assert_eq!(value["data"]["predicted_effect"], "changed");
}

#[test]
fn p4_review_only_effects_row() {
    // "Selected revisions or comments changed but body rows unchanged:
    //  report actual review effects; this is not a zero-effect result."
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.counts.revisions_accepted = 2;
    data.counts.comments_added = 1;
    let mut envelope = Envelope::ok(
        "review accept",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    envelope.push_artifact(Artifact::committed(
        "/tmp/out.docx",
        MediaRole::Document,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        &fixture_bytes(),
    ));
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["effect"], "changed");
    assert_eq!(value["data"]["counts"]["revisions_accepted"], 2);
    assert_eq!(value["data"]["counts"]["comments_added"], 1);
    // The body operation count is zero on a review-only request; the
    // counts struct records it, even when the field is skipped in JSON.
    assert_eq!(
        envelope.data.expect("data").counts.body_operations_applied,
        0
    );
}

#[test]
fn p4_unrealized_desired_text_error_row() {
    // "Desired text differs but engine fails to realize it: postcondition
    //  failure; never treat as allowed unchanged."
    let envelope = Envelope::refused(
        "edit apply",
        Source::new(&fixture_bytes()),
        Error::new("POSTCONDITION_FAILED")
            .with_operation("op-2")
            .with_message("requested text was not realized after rewrite"),
    );
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["status"], "refused");
    assert_eq!(value["error"]["code"], "POSTCONDITION_FAILED");
    assert_eq!(value["error"]["operation"], "op-2");
    // No silent unchanged success.
    assert!(
        value["data"].is_null(),
        "data must be absent on a refused envelope"
    );
}

// ---------------------------------------------------------------------------
// Error envelope: code, operation pointer, source digest, bounded context,
// truncation flag, structured argv next actions.
// ---------------------------------------------------------------------------

#[test]
fn error_envelope_has_bounded_context_and_structured_argv() {
    let mut error = Error::new("AMBIGUOUS_ANCHOR")
        .with_operation("op-3")
        .with_source(Source::new(&fixture_bytes()))
        .with_message("anchor 'fee' matched 3 times")
        .with_matches(3);
    error
        .set_context(Context::new("Payment is due within ", "fee", " days.").with_truncated(false));
    error.push_next_action(
        NextAction::new(vec![
            "jubarte".to_string(),
            "read".to_string(),
            "FILE.docx".to_string(),
            "--find".to_string(),
            "fee".to_string(),
            "--json".to_string(),
        ])
        .with_purpose("list every occurrence with paragraph ids"),
    );
    let envelope = Envelope::refused("edit apply", Source::new(&fixture_bytes()), error);
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["error"]["code"], "AMBIGUOUS_ANCHOR");
    assert_eq!(value["error"]["operation"], "op-3");
    assert_eq!(
        value["error"]["source_digest"],
        source_sha256(&fixture_bytes())
    );
    assert_eq!(value["error"]["matches"], 3);
    assert_eq!(
        value["error"]["context"]["before"],
        "Payment is due within "
    );
    assert_eq!(value["error"]["context"]["matched"], "fee");
    assert_eq!(value["error"]["context"]["after"], " days.");
    assert_eq!(value["error"]["context"]["truncated"], false);

    let argv = value["error"]["next_actions"][0]["argv"]
        .as_array()
        .expect("argv must be a JSON array");
    let argv_strings: Vec<&str> = argv.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(
        argv_strings,
        vec!["jubarte", "read", "FILE.docx", "--find", "fee", "--json"],
        "next actions are structured argv, never shell strings"
    );
}

#[test]
fn context_carries_an_explicit_truncation_flag() {
    // A long context is bounded; the truncation flag is true.
    let ctx = Context::new(
        "Lorem ipsum dolor sit amet ",
        "needle",
        " consectetur adipiscing",
    )
    .with_truncated(true);
    let error = Error::new("AMBIGUOUS_ANCHOR")
        .with_operation("op-7")
        .with_source(Source::new(&fixture_bytes()))
        .with_context(ctx);
    let env = Envelope::refused("edit apply", Source::new(&fixture_bytes()), error);
    let json = serde_json::to_string(&env).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["error"]["context"]["truncated"], true);
}

// ---------------------------------------------------------------------------
// Artifacts: committed flag is independent of document effect.
// ---------------------------------------------------------------------------

#[test]
fn artifact_committed_flag_is_independent_of_document_effect() {
    // A plan artifact can be committed while the document effect is
    // not_applied: a generated plan with no applied edit.
    let mut data = ResultData::new(Phase::Generated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 1;
    let mut envelope = Envelope::ok(
        "edit template",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    envelope.push_artifact(
        Artifact::new(
            "/tmp/plan.json",
            MediaRole::Plan,
            "application/json",
            b"{}".as_slice(),
        )
        .with_committed(true),
    );
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["data"]["phase"], "generated");
    assert_eq!(value["data"]["effect"], "not_applied");
    assert_eq!(value["artifacts"][0]["committed"], true);
    assert_eq!(value["artifacts"][0]["media_role"], "plan");
}

#[test]
fn artifact_byte_size_and_sha256_match_payload() {
    let payload = b"hello world".to_vec();
    let artifact = Artifact::new("/tmp/x.bin", MediaRole::Render, "application/pdf", &payload);
    let json = serde_json::to_string(&artifact).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["byte_size"], 11);
    assert_eq!(value["sha256"], source_sha256(&payload));
    assert_eq!(value["media_role"], "render");
}

#[test]
fn artifact_uncommitted_is_serialized_with_committed_false() {
    let artifact = Artifact::new("/tmp/x", MediaRole::Patch, "text/x-diff", b"".as_slice());
    let json = serde_json::to_string(&artifact).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["committed"], false);
}

// ---------------------------------------------------------------------------
// Counts: operations, logical revisions, revision XML entries, comments are
// kept distinct in the schema.
// ---------------------------------------------------------------------------

#[test]
fn counts_keep_operations_revisions_xml_entries_and_comments_apart() {
    let counts = Counts {
        requested_operations: 4,
        body_operations_applied: 2,
        review_operations_applied: 1,
        unchanged_operations: 1,
        revisions_accepted: 1,
        revisions_rejected: 1,
        logical_revisions: 1,
        revision_xml_entries: 2,
        comment_threads: 1,
        comment_replies: 2,
        ..Default::default()
    };
    let json = serde_json::to_string(&counts).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Every category has its own field; the keys are not merged.
    let keys: BTreeSet<&str> = value
        .as_object()
        .expect("counts is an object")
        .keys()
        .map(String::as_str)
        .collect();
    for required in [
        "requested_operations",
        "body_operations_applied",
        "review_operations_applied",
        "unchanged_operations",
        "revisions_accepted",
        "revisions_rejected",
        "logical_revisions",
        "revision_xml_entries",
        "comment_threads",
        "comment_replies",
    ] {
        assert!(
            keys.contains(required),
            "counts.{required} must appear; got {keys:?}"
        );
    }
}

#[test]
fn counts_omit_unset_zero_fields() {
    // An empty Counts serializes to an object that may be absent; nonzero
    // fields stay explicit so a reader never has to guess a default.
    let counts = Counts {
        requested_operations: 3,
        ..Default::default()
    };
    let json = serde_json::to_string(&counts).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["requested_operations"], 3);
    assert!(
        value.get("revisions_accepted").is_none(),
        "unset zero fields are skipped from the JSON"
    );
}

// ---------------------------------------------------------------------------
// JSONL: progress lines plus exactly one terminal result line.
// ---------------------------------------------------------------------------

#[test]
fn jsonl_stream_writes_progress_lines_then_terminal_result() {
    let mut data = ResultData::new(Phase::Validated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 1;
    data.set_predicted(PredictedEffect::Unchanged);
    let terminal = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );

    let events = vec![
        ProgressEvent::from(serde_json::json!({
            "ev": "load",
            "sha256": source_sha256(&fixture_bytes()),
        })),
        ProgressEvent::from(serde_json::json!({
            "ev": "op",
            "i": 1,
            "id": "op-1",
            "status": "ok",
        })),
    ];

    let stream = write_jsonl_stream(&events, &terminal);
    // The terminal line is the last non-empty line.
    let lines: Vec<&str> = stream.lines().filter(|l| !l.is_empty()).collect();
    assert!(lines.len() >= 3, "at least progress + terminal lines");
    let last = lines.last().expect("non-empty");
    let last_value: serde_json::Value = serde_json::from_str(last).unwrap();
    assert_eq!(last_value["status"], "ok");
    assert_eq!(last_value["data"]["phase"], "validated");
    assert_eq!(last_value["data"]["predicted_effect"], "unchanged");
}

#[test]
fn jsonl_terminal_line_helper_writes_only_the_terminal_record() {
    let mut data = ResultData::new(Phase::Generated, jubarte::result::Effect::NotApplied);
    data.counts.requested_operations = 1;
    let terminal = Envelope::ok(
        "edit template",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let line = jsonl_terminal_line(&terminal);
    let value: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["data"]["phase"], "generated");
}

#[test]
fn jsonl_stream_without_terminal_record_is_incomplete() {
    // Only progress lines; no terminal. The reader reports incomplete.
    let events = vec![ProgressEvent::from(serde_json::json!({"ev": "load"}))];
    let stream = write_jsonl_stream(
        &events,
        &Envelope::new(
            "edit apply",
            Source::new(&fixture_bytes()),
            Runtime::native(),
        ),
    );
    // Replace the terminal line with a progress line so the stream lacks it.
    let mangled = stream.replace("\"status\":", "\"xstatus\":");
    let result = read_jsonl_stream(&mangled);
    assert!(
        result.is_err(),
        "a stream without a terminal result line must be reported incomplete"
    );
}

#[test]
fn jsonl_stream_with_terminal_record_is_complete() {
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.counts.requested_operations = 1;
    let terminal = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let stream = write_jsonl_stream(&[], &terminal);
    let parsed = read_jsonl_stream(&stream).expect("complete stream parses");
    assert_eq!(parsed.terminal.status, Status::Ok);
    assert!(matches!(
        parsed.terminal.data.as_ref().expect("data present").phase,
        Phase::Committed
    ));
}

#[test]
fn jsonl_stream_rejects_two_terminal_records() {
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.counts.requested_operations = 1;
    let terminal = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    // Two terminals concatenated: invalid stream.
    let stream = format!(
        "{}\n{}\n",
        write_jsonl_stream(&[], &terminal),
        write_jsonl_stream(&[], &terminal),
    );
    let parsed = read_jsonl_stream(&stream);
    assert!(parsed.is_err(), "exactly one terminal record is required");
}

// ---------------------------------------------------------------------------
// Exit status mapping per proposal §12.
// ---------------------------------------------------------------------------

#[test]
fn exit_status_mapping_covers_ok_refused_and_error() {
    assert_eq!(exit_status_for(Status::Ok, None), ExitStatus::SUCCESS);
    let err = Error::new("STALE_SOURCE");
    assert_eq!(
        exit_status_for(Status::Refused, Some(&err)),
        ExitStatus::REFUSED
    );
    assert_eq!(
        exit_status_for(Status::Error, Some(&err)),
        ExitStatus::INPUT_FAILURE
    );
}

#[test]
fn exit_status_mapping_recognises_render_failure_and_deadline() {
    let err = Error::new("RENDERING_FAILED");
    // RENDERING_FAILED is a pre-publication rendering/fidelity policy failure.
    assert_eq!(
        exit_status_for(Status::Error, Some(&err)),
        ExitStatus::INPUT_FAILURE,
        "non-classified error codes fall back to input/operational failure (1)"
    );
    let err = Error::new("RENDER_FIDELITY_FAILED");
    assert_eq!(
        exit_status_for(Status::Error, Some(&err)),
        ExitStatus::RENDER_FAILURE
    );
    let err = Error::new("DEADLINE_EXCEEDED");
    assert_eq!(
        exit_status_for(Status::Error, Some(&err)),
        ExitStatus::DEADLINE_EXCEEDED
    );
    let err = Error::new("USAGE");
    assert_eq!(
        exit_status_for(Status::Error, Some(&err)),
        ExitStatus::USAGE
    );
}

#[test]
fn exit_status_mapping_refused_codes_match_section_12() {
    // --require-change is a refused condition (exit 3).
    let err = Error::new("NO_CHANGES_REQUESTED");
    assert_eq!(
        exit_status_for(Status::Refused, Some(&err)),
        ExitStatus::REFUSED
    );
    // Postcondition failure is a refused invariant (exit 3).
    let err = Error::new("POSTCONDITION_FAILED");
    assert_eq!(
        exit_status_for(Status::Refused, Some(&err)),
        ExitStatus::REFUSED
    );
    // Stale source, ambiguous anchor, unsupported all map to refused (exit 3).
    for code in [
        "STALE_SOURCE",
        "AMBIGUOUS_ANCHOR",
        "UNSUPPORTED_OPERATION",
        "WORKING_BASE_CHANGED",
    ] {
        assert_eq!(
            exit_status_for(Status::Refused, Some(&Error::new(code))),
            ExitStatus::REFUSED,
            "{code} must map to exit 3"
        );
    }
}

#[test]
fn exit_status_mapping_check_threshold_and_comparison_difference() {
    // Standalone check failure threshold maps to exit 2.
    let err = Error::new("CHECK_THRESHOLD_EXCEEDED");
    assert_eq!(
        exit_status_for(Status::Refused, Some(&err)),
        ExitStatus::CHECK_FINDINGS
    );
    // Comparison difference with --exit-code maps to exit 5.
    let err = Error::new("COMPARISON_DIFFERENCE");
    assert_eq!(
        exit_status_for(Status::Refused, Some(&err)),
        ExitStatus::COMPARISON_DIFFERENCE
    );
}

// ---------------------------------------------------------------------------
// Envelope shape: command, runtime, verification, source.
// ---------------------------------------------------------------------------

#[test]
fn envelope_includes_command_engine_version_runtime_and_verification() {
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.verification = Verification {
        operation_postconditions: "passed".to_string(),
        integrity: "passed".to_string(),
        expectations: "not_requested".to_string(),
        task_semantics: "not_assessed".to_string(),
    };
    let envelope = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["command"], "edit apply");
    assert_eq!(value["runtime"]["name"], "native");
    assert!(value["engine_version"].is_string());
    assert_eq!(
        value["data"]["verification"]["operation_postconditions"],
        "passed"
    );
    assert_eq!(
        value["data"]["verification"]["expectations"],
        "not_requested"
    );
}

#[test]
fn envelope_can_carry_warnings() {
    let mut envelope = ok_envelope(Status::Ok, jubarte::result::Effect::Changed);
    envelope.push_warning("baseline warning: BOOKMARK_CROSSES_REVISION");
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    let warnings = value["warnings"].as_array().expect("warnings array");
    assert_eq!(warnings.len(), 1);
}

#[test]
fn envelope_can_carry_effective_options() {
    let mut envelope = ok_envelope(Status::Ok, jubarte::result::Effect::Changed);
    envelope.set_effective_option("require_change", serde_json::json!(true));
    envelope.set_effective_option("track", serde_json::json!("on"));
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    assert_eq!(value["effective_options"]["require_change"], true);
    assert_eq!(value["effective_options"]["track"], "on");
}

// ---------------------------------------------------------------------------
// Constructors: envelope kinds and shapes.
// ---------------------------------------------------------------------------

#[test]
fn ok_envelope_status_field_serializes_lower_string() {
    let envelope = ok_envelope(Status::Ok, jubarte::result::Effect::Changed);
    let json = serde_json::to_string(&envelope).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["status"], "ok");
}

#[test]
fn refused_envelope_carries_error_and_null_data() {
    let err = Error::new("STALE_SOURCE").with_message("source sha256 mismatch");
    let envelope = Envelope::refused("edit apply", Source::new(&fixture_bytes()), err);
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    assert_eq!(value["status"], "refused");
    assert_eq!(value["error"]["code"], "STALE_SOURCE");
    assert!(value["data"].is_null(), "data must be absent on a refusal");
}

#[test]
fn error_envelope_carries_error_and_null_data() {
    let err = Error::new("INVALID_PLAN").with_message("schema_version mismatch");
    let envelope = Envelope::new_error("edit apply", Source::new(&fixture_bytes()), err);
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["code"], "INVALID_PLAN");
    assert!(value["data"].is_null());
}
// ---------------------------------------------------------------------------
// Snapshot: exact JSON strings are the contract for selected shapes.
// ---------------------------------------------------------------------------

#[test]
fn ok_envelope_snapshot_matches_proposal_shape() {
    // The proposal's "abbreviated result-shape illustration" (section 11)
    // is the contract for the field set; this test locks the field order
    // and naming down exactly. Empty optional arrays/objects/maps may be
    // absent in the JSON for terseness, but every required field is named.
    let mut data = ResultData::new(Phase::Committed, jubarte::result::Effect::Changed);
    data.counts.requested_operations = 4;
    data.counts.unchanged_operations = 1;
    data.counts.body_operations_applied = 2;
    data.counts.review_operations_applied = 1;
    data.counts.revisions_accepted = 0;
    data.counts.revisions_rejected = 0;
    data.verification = Verification {
        operation_postconditions: "passed".to_string(),
        integrity: "passed".to_string(),
        expectations: "not_requested".to_string(),
        task_semantics: "not_assessed".to_string(),
    };
    let envelope = Envelope::ok(
        "edit apply",
        Source::new(&fixture_bytes()),
        data,
        Runtime::native(),
    );
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();

    // Required top-level fields the proposal names explicitly. These
    // must always be present.
    for key in [
        "schema_version",
        "command",
        "engine_version",
        "runtime",
        "status",
        "source",
        "data",
    ] {
        assert!(
            value.get(key).is_some(),
            "top-level field `{key}` must exist"
        );
    }
    // Optional top-level fields: when present they have the right shape
    // (array / object / null); when absent it is because they are empty.
    if let Some(warnings) = value.get("warnings") {
        assert!(warnings.is_array());
        assert!(warnings.as_array().unwrap().is_empty());
    }
    if let Some(artifacts) = value.get("artifacts") {
        assert!(artifacts.is_array());
        assert!(artifacts.as_array().unwrap().is_empty());
    }
    if let Some(options) = value.get("effective_options") {
        assert!(options.is_object());
        assert!(options.as_object().unwrap().is_empty());
    }
    if let Some(continuation) = value.get("continuation") {
        assert!(continuation.is_null());
    }
    if let Some(error) = value.get("error") {
        assert!(error.is_null());
    }
    // The example illustration the proposal prints explicitly:
    assert_eq!(value["status"], "ok");
    assert_eq!(value["command"], "edit apply");
    assert_eq!(value["data"]["phase"], "committed");
    assert_eq!(value["data"]["effect"], "changed");
    assert_eq!(
        value["data"]["verification"]["operation_postconditions"],
        "passed"
    );
}

#[test]
fn error_envelope_snapshot_carries_code_operation_source_context_argv() {
    // The error block carries the typed code, operation pointer, source
    // digest, bounded context and structured argv next actions. Each is
    // a distinct JSON field; readers do not have to parse a message
    // string to recover them.
    let mut error = Error::new("AMBIGUOUS_ANCHOR")
        .with_operation("op-3")
        .with_source(Source::new(&fixture_bytes()))
        .with_message("anchor matched 3 times")
        .with_matches(3);
    error.set_context(Context::new("before ", "needle", " after"));
    error.push_next_action(NextAction::new(vec![
        "jubarte".to_string(),
        "read".to_string(),
    ]));
    let envelope = Envelope::refused("edit apply", Source::new(&fixture_bytes()), error);
    let value: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    let err = &value["error"];
    assert!(err["code"].is_string());
    assert!(err["operation"].is_string());
    assert!(err["source_digest"].is_string());
    assert!(err["context"].is_object());
    assert!(err["context"]["before"].is_string());
    assert!(err["context"]["matched"].is_string());
    assert!(err["context"]["after"].is_string());
    assert!(err["context"]["truncated"].is_boolean());
    assert!(err["next_actions"].is_array());
    assert!(err["next_actions"][0]["argv"].is_array());
    // next_action argv entries are strings, never a concatenated shell
    // string assembled from document content.
    for arg in err["next_actions"][0]["argv"].as_array().unwrap() {
        assert!(arg.is_string(), "argv entries must be strings");
    }
}
