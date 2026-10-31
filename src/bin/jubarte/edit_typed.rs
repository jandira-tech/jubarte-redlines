// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Typed edit orchestration. Reuses candidate preparation and atomic publication.

use super::*;
use jubarte::cli::edit_actions::{EditAction, TypedEdit};
use jubarte::edit::{EditPlan, PrepareOptions, PreparedEdit, PrimaryRole, v2::PlanV2};
use jubarte::expect::{ExpectationReport, ReceiptMeta, evaluate_with_receipt, parse_expectations};
use jubarte::result::{
    Artifact, CommitState, Effect, Envelope, Error, MediaRole, NextAction, Phase, PredictedEffect,
    ResultData, Runtime, Source, Status,
};

type Failure = Box<Error>;
fn failure(code: &str, message: impl Into<String>) -> Failure {
    Box::new(
        Error::new(code)
            .with_message(message)
            .with_commit_state(CommitState::NotCommitted),
    )
}
fn edit_error(e: jubarte::edit::EditError) -> Failure {
    let mut error = failure(&e.code, e.message);
    error.operation = e.operation;
    error.near = e.near.map(|near| *near);
    error
}
fn io_error(message: String) -> Failure {
    failure("IO_ERROR", message)
}
/// A publication refusal or failure in the vocabulary of the typed and
/// template routes. The shared publisher names its refusals by a `CODE: `
/// prefix; `SOURCE_CHANGED` is these routes' `STALE_SOURCE`. Nothing is
/// published before the primary, so a failure that committed nothing is
/// `not_committed`; after a commit the primary's state is not reported back,
/// and stays `unknown`.
pub(super) fn publish_failure((_, message): (u8, String)) -> Failure {
    let code = match message.split_once(": ").map(|(code, _)| code) {
        Some(code @ ("OUTPUT_EXISTS" | "OUTPUT_ALIAS")) => code,
        Some("SOURCE_CHANGED") => "STALE_SOURCE",
        _ => "IO_ERROR",
    };
    let state = if message.contains("(already committed: ") {
        CommitState::Unknown
    } else {
        CommitState::NotCommitted
    };
    Box::new(
        Error::new(code)
            .with_message(message)
            .with_commit_state(state),
    )
}
fn read(path: &Path) -> Result<Vec<u8>, Failure> {
    std::fs::read(path).map_err(|e| io_error(format!("reading {}: {e}", path.display())))
}
fn alias(path: &Path, inputs: &[(&Path, &str)]) -> Result<(), Failure> {
    refuse_output_alias(path, inputs).map_err(|(_, message)| failure("OUTPUT_ALIAS", message))
}
fn writable(path: &Path, inputs: &[(&Path, &str)], force: bool) -> Result<(), Failure> {
    alias(path, inputs)?;
    ensure_writable(path, force).map_err(|error| {
        failure(
            "OUTPUT_EXISTS",
            error.message.trim_start_matches("OUTPUT_EXISTS: "),
        )
    })
}

enum Request {
    /// A v1 plan and the notes of the fields it read as others.
    V1(EditPlan, Vec<String>),
    V2(PlanV2),
}
impl Request {
    fn prepare(
        &self,
        source: &[u8],
        base: &PrepareOptions,
    ) -> Result<PreparedEdit, jubarte::edit::EditError> {
        match self {
            Self::V2(plan) => jubarte::edit::prepare_v2(source, plan, base),
            Self::V1(plan, notes) => {
                // The v1 typed route publishes the redline when the
                // transaction is tracked: ask `prepare` for the redline
                // role so its chosen-primary checks (serialized-story
                // postcondition included) describe the bytes this layer
                // writes, and the receipt's digest names those same
                // bytes. The library used to return clean and the CLI
                // swapped; that left a redline-only defect unchecked.
                let options = PrepareOptions {
                    role: PrimaryRole::Redline,
                    ..*base
                };
                let mut prepared = jubarte::edit::prepare(source, plan, &options)?;
                prepared.result.report.notes.clone_from(notes);
                Ok(prepared)
            }
        }
    }
    fn count(&self) -> usize {
        match self {
            Self::V1(p, _) => p.operations.len(),
            Self::V2(p) => p.operations.len(),
        }
    }
}

fn regeneration_action(
    error: &jubarte::edit::EditError,
    plan: &PlanV2,
    source: &Path,
    plan_path: &Path,
) -> Option<NextAction> {
    let mut seen = std::collections::BTreeSet::new();
    let targets: Vec<_> = error
        .outcomes
        .iter()
        .filter(|o| o.code.as_deref() == Some("WORKING_BASE_CHANGED"))
        .filter_map(|o| o.id.strip_prefix("/operations/")?.parse::<usize>().ok())
        .filter_map(|i| match plan.operations.get(i)? {
            jubarte::edit::v2::Operation::Rewrite { target, .. } => Some(&target.short_id),
            _ => None,
        })
        .filter(|id| seen.insert(*id))
        .map(String::as_str)
        .collect();
    if targets.is_empty() {
        return None;
    }
    let mut output = plan_path.with_extension("regenerated.json");
    let mut suffix = 1;
    while output.symlink_metadata().is_ok() {
        output = plan_path.with_extension(format!("regenerated-{suffix}.json"));
        suffix += 1;
    }
    let mut argv = vec![
        "jubarte".into(),
        "edit".into(),
        "template".into(),
        absolute(source),
        "--select".into(),
        targets.join(","),
    ];
    for (flag, decisions) in [
        ("--accept-change", &plan.resolve_revisions.accept),
        ("--reject-change", &plan.resolve_revisions.reject),
    ] {
        for target in decisions {
            argv.extend([flag.into(), target.short_id.clone()]);
        }
    }
    let policy = match plan.policy.existing_revisions {
        jubarte::edit::ExistingRevisions::Keep => "keep",
        jubarte::edit::ExistingRevisions::Accept => "accept",
        jubarte::edit::ExistingRevisions::Reject => "reject",
        jubarte::edit::ExistingRevisions::Refuse => "refuse",
    };
    let track = match plan.policy.track {
        jubarte::edit::v2::Track::On => "on",
        jubarte::edit::v2::Track::Off => "off",
    };
    argv.extend([
        "--existing-revisions".into(),
        policy.into(),
        "--track".into(),
        track.into(),
        format!("--author={}", plan.author),
        "--datetime".into(),
        plan.date.clone(),
        "--expect-sha256".into(),
        plan.source.sha256.clone(),
        "-o".into(),
        absolute(&output),
    ]);
    Some(
        NextAction::new(argv)
            .with_prerequisites(vec![format!(
                "the source file must still have the plan's recorded sha256 {} (--expect-sha256)",
                plan.source.sha256
            )])
            .with_purpose(
                "regenerate a new plan containing only the stale rows with their current working text; the original plan file is not changed; merge those rows into the original plan by hand",
            ),
    )
}

/// Read, bind and evaluate the optional expectation contract on the
/// prepared transaction's chosen primary bytes (V2-13, proposal §10).
///
/// A malformed file refuses `USAGE` (the malformed-request class, exit 64);
/// a `source.sha256` that does not name the plan's source digest refuses
/// `STALE_SOURCE`; and any failed or refused check refuses
/// `EXPECTATION_FAILED` before anything is written, with the bounded
/// report riding in the error block. On success the returned report
/// carries the output-bound copy of the evaluated expectations (the
/// caller's file is never rewritten).
fn evaluate_expectations(
    path: &Path,
    prepared: &PreparedEdit,
    source: &[u8],
) -> Result<ExpectationReport, Failure> {
    let bytes = read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| {
        failure(
            "USAGE",
            format!("--expect {}: not UTF-8: {e}", path.display()),
        )
    })?;
    let spec = parse_expectations(text)
        .map_err(|e| failure("USAGE", format!("--expect {}: {e}", path.display())))?;
    if spec.source.sha256 != prepared.source_sha256 {
        return Err(failure(
            "STALE_SOURCE",
            "expectations source.sha256 does not match the plan's source digest",
        ));
    }
    let Some(correspondence) = prepared.correspondence.as_ref() else {
        return Err(failure(
            "INTERNAL",
            "this request produced no source-to-output correspondence",
        ));
    };
    let report = evaluate_with_receipt(
        &spec,
        source,
        &prepared.primary,
        correspondence,
        ReceiptMeta {
            path: Some(path.display().to_string()),
            digest: Some(jubarte::expect::content_sha256(&bytes)),
            declared_provenance: None,
        },
    );
    if let Some(blocking) = report.blocking_failure() {
        let mut error = failure(
            "EXPECTATION_FAILED",
            format!(
                "expectation requirement {:?} on {} did not hold ({}); no output was published",
                blocking.key,
                blocking.target,
                match blocking.outcome {
                    jubarte::expect::Outcome::Failed => "check failed".to_string(),
                    _ => format!(
                        "could not evaluate: {}",
                        blocking
                            .reason
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    ),
                }
            ),
        );
        error.expectations = Some(report);
        return Err(error);
    }
    let mut report = report;
    report.output_bound =
        jubarte::expect::rebind_to_output(&spec, correspondence, &prepared.primary);
    Ok(report)
}

fn check_hash(
    recorded: Option<&str>,
    expected: Option<&str>,
    source: &[u8],
    waiver: bool,
) -> Result<&'static str, Failure> {
    let digest = jubarte::inspect::source_sha256(source);
    if recorded.is_some_and(|hash| hash != digest)
        || expected.is_some_and(|hash| hash != digest)
        || recorded.zip(expected).is_some_and(|(a, b)| a != b)
    {
        return Err(failure(
            "STALE_SOURCE",
            "recorded source hash and --expect-sha256 must agree with the exact source bytes",
        ));
    }
    if recorded.is_some() || expected.is_some() {
        Ok("bound")
    } else if waiver {
        Ok("unbound_legacy")
    } else {
        Err(failure(
            "UNBOUND_LEGACY_PLAN",
            "legacy plan has no recorded digest; supply --expect-sha256 or explicitly --allow-unbound-plan",
        ))
    }
}

fn request(
    action: &EditAction,
    source: &[u8],
    payload: Option<&str>,
    plan_bytes: Option<&[u8]>,
) -> Result<(Request, &'static str), Failure> {
    let delivery = action.delivery();
    if let EditAction::Apply {
        allow_unbound_plan, ..
    } = action
    {
        let json = std::str::from_utf8(plan_bytes.expect("read plan"))
            .map_err(|e| failure("INVALID_PLAN", e.to_string()))?;
        let json = json.strip_prefix('\u{feff}').unwrap_or(json);
        // The v1 parser words a malformed plan as every v1 caller sees it.
        let version: serde_json::Value = serde_json::from_str(json).map_err(|e| {
            EditPlan::from_json(json)
                .err()
                .map_or_else(|| failure("INVALID_PLAN", e.to_string()), edit_error)
        })?;
        // So is a plan that is no object: an array is no plan without its
        // version.
        if !version.is_object() {
            return Err(EditPlan::from_json(json).err().map_or_else(
                || failure("INVALID_PLAN", "the plan must be exactly one JSON object"),
                edit_error,
            ));
        }
        if version.get("schema_version").is_none() {
            return Err(failure(
                "INVALID_PLAN",
                "schema_version is required (1 or 2 for edit apply)",
            ));
        }
        match version
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
        {
            Some(2) => {
                let plan = PlanV2::from_json(json).map_err(edit_error)?;
                if *allow_unbound_plan {
                    return Err(failure(
                        "INVALID_PLAN",
                        "--allow-unbound-plan is v1-only; PlanV2 requires its embedded source.sha256",
                    ));
                }
                check_hash(
                    Some(&plan.source.sha256),
                    delivery.expect_sha256.as_deref(),
                    source,
                    false,
                )?;
                Ok((Request::V2(plan), "bound"))
            }
            Some(1) => {
                let (mut plan, notes) = EditPlan::from_json_with_notes(json).map_err(edit_error)?;
                let binding = check_hash(
                    plan.source_sha256.as_deref(),
                    delivery.expect_sha256.as_deref(),
                    source,
                    *allow_unbound_plan,
                )?;
                if plan.source_sha256.is_none() {
                    plan.source_sha256.clone_from(&delivery.expect_sha256);
                }
                Ok((Request::V1(plan, notes), binding))
            }
            _ => Err(failure(
                "UNSUPPORTED_SCHEMA",
                "edit apply supports plan schema_version 1 or 2",
            )),
        }
    } else {
        let plan = jubarte::edit::inline::compile(source, action, payload)?;
        check_hash(
            Some(&plan.source.sha256),
            delivery.expect_sha256.as_deref(),
            source,
            false,
        )?;
        Ok((Request::V2(plan), "bound"))
    }
}

fn artifact(path: &Path, role: MediaRole, bytes: &[u8]) -> Artifact {
    let media = match role {
        MediaRole::Document => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        MediaRole::Patch => "text/plain",
        _ => "application/json",
    };
    Artifact::committed(absolute(path), role, media, bytes)
}
fn success(command: &str, source: &[u8], phase: Phase, effect: Effect) -> Envelope {
    Envelope::ok(
        command,
        Source::new(source),
        ResultData::new(phase, effect),
        Runtime::native(),
    )
    .expect("valid phase/effect")
}

fn prepare_and_publish(
    action: &EditAction,
    source: &[u8],
    command: &str,
) -> Result<(Envelope, bool, Option<String>), Failure> {
    let delivery = action.delivery();
    let payload_bytes = action
        .payload()
        .and_then(|p| p.text_file.as_ref())
        .map(|path| read(path))
        .transpose()?;
    let payload = payload_bytes
        .as_deref()
        .map(std::str::from_utf8)
        .transpose()
        .map_err(|e| failure("INVALID_UTF8", e.to_string()))?
        .or_else(|| action.payload().and_then(|p| p.text.as_deref()));
    let plan_path = match action {
        EditAction::Apply { plan, .. } => Some(plan.as_path()),
        _ => None,
    };
    let plan_bytes = plan_path.map(read).transpose()?;
    let mut inputs = vec![(action.file(), "the source")];
    if let Some(plan) = plan_path {
        inputs.push((plan, "the plan"));
    }
    if let Some(path) = action.payload().and_then(|p| p.text_file.as_deref()) {
        inputs.push((path, "the text payload"));
    }
    if let Some(path) = &delivery.expect {
        inputs.push((path, "the expectations"));
    }
    let save = action.inline().and_then(|i| i.save_plan.as_deref());
    let paths: Vec<_> = [delivery.output.as_deref(), delivery.report.as_deref(), save]
        .into_iter()
        .flatten()
        .collect();
    for (i, path) in paths.iter().enumerate() {
        alias(path, &inputs)?;
        for other in &paths[..i] {
            if same_path(path, other) {
                return Err(failure(
                    "OUTPUT_ALIAS",
                    "two output roles name the same file",
                ));
            }
        }
    }
    if let Some(dir) = &delivery.out_dir {
        // Bundles must be new. A directory containing an input cannot be replaced.
        if dir.symlink_metadata().is_ok() {
            return Err(failure(
                "OUTPUT_EXISTS",
                "--out-dir requires a new directory; --force replaces only an explicitly named output file",
            ));
        }
    }
    let (mut request, binding) = request(action, source, payload, plan_bytes.as_deref())?;
    // The flag and JUBARTE_KEEP_OTHERS only turn the guard on. A plan that
    // already says "others": "keep" stays guarded when neither is set.
    if delivery.keep_others || super::env_keep_others() {
        match &mut request {
            Request::V2(plan) => plan.others = Some(jubarte::edit::Others::Keep),
            Request::V1(plan, _) => plan.others = jubarte::edit::Others::Keep,
        }
    }
    if delivery.expect.is_some() && matches!(&request, Request::V1(..)) {
        // The v1 primary is a comparer redline with no tracked
        // source-to-output correspondence; guessing one could grade the
        // wrong paragraph, so the combination refuses.
        return Err(failure(
            "UNSUPPORTED_EXPECTATION",
            "--expect requires a PlanV2 request in this layer; re-save the plan as v2 (edit template / --save-plan) or use jubarte check on the output",
        ));
    }
    // A refusal quotes its routes, and records its choices, in the syntax
    // the caller wrote: the typed options, or the plan's JSON.
    let (syntax, revision_syntax) = if action.inline().is_some() {
        (
            jubarte::edit::PlanSyntax::Typed,
            jubarte::edit::refusal::RevisionSyntax::TypedAction,
        )
    } else if matches!(request, Request::V2(_)) {
        (
            jubarte::edit::PlanSyntax::V2,
            jubarte::edit::refusal::RevisionSyntax::V2Plan,
        )
    } else {
        (
            jubarte::edit::PlanSyntax::V1,
            jubarte::edit::refusal::RevisionSyntax::V1Plan,
        )
    };
    let prepared = request
        .prepare(
            source,
            &PrepareOptions {
                require_change: delivery.require_change,
                exact_text: matches!(request, Request::V2(_)),
                role: PrimaryRole::Clean,
                syntax,
                ..PrepareOptions::default()
            },
        )
        .map_err(|e| {
            let next = match (&request, plan_path) {
                (Request::V2(plan), Some(path)) if e.code == "WORKING_BASE_CHANGED" => {
                    regeneration_action(&e, plan, action.file(), path)
                }
                _ => None,
            };
            let e = e.with_revision_syntax(revision_syntax);
            let actions = e.revision_next_actions(
                &absolute(action.file()),
                &jubarte::inspect::source_sha256(source),
            );
            let mut error = edit_error(e);
            for action in actions {
                error.push_next_action(action);
            }
            if let Some(next) = next {
                error.push_next_action(next);
            }
            error
        })?;
    // Optional expectations (V2-13): evaluate on the prepared transaction's
    // chosen primary bytes, reopened, through the source-to-output
    // correspondence the transaction built. Any failed executable check and
    // any refused check blocks publication; unassessed natural-language
    // requirements never do. A dry run evaluates the same way.
    let expectations = match &delivery.expect {
        Some(path) => Some(evaluate_expectations(path, &prepared, source)?),
        None => None,
    };
    let phase = if save.is_some() {
        Phase::Generated
    } else if delivery.dry_run {
        Phase::Validated
    } else {
        Phase::Committed
    };
    let effect = if phase == Phase::Committed {
        prepared.primary_effect
    } else {
        Effect::NotApplied
    };
    let mut receipt = success(command, source, phase, effect);
    receipt.source.path = Some(absolute(action.file()));
    receipt
        .effective_options
        .insert("source_binding".into(), binding.into());
    receipt.effective_options.insert(
        "request_sha256".into(),
        prepared.request_sha256.clone().into(),
    );
    // `--in-place` publishes FILE itself. `-o` stays unset, so the alias
    // check above does not treat the source as an output.
    let document = if delivery.in_place {
        Some(action.file().to_path_buf())
    } else {
        delivery.output.clone()
    };
    let report_path = delivery.report.clone().or_else(|| {
        if delivery.out_dir.is_some()
            && (delivery.emit.is_empty() || delivery.emit.iter().any(|r| r == "report"))
        {
            delivery.out_dir.as_ref().map(|dir| dir.join("report.json"))
        } else if delivery.emit.iter().any(|r| r == "report") {
            document
                .as_ref()
                .map(|path| path.with_extension("report.json"))
        } else {
            None
        }
    });
    let (author, datetime, track, existing_revisions) = match &request {
        Request::V2(plan) => (
            plan.author.clone(),
            plan.date.clone(),
            serde_json::to_value(plan.policy.track).expect("track"),
            serde_json::to_value(plan.policy.existing_revisions).expect("policy"),
        ),
        Request::V1(..) => (
            prepared.result.report.author.clone(),
            prepared.result.report.date.clone(),
            serde_json::json!("on"),
            serde_json::to_value(prepared.result.report.existing_revisions).expect("policy"),
        ),
    };
    let emit = if delivery.emit.is_empty() {
        if save.is_some() {
            vec![]
        } else if delivery.out_dir.is_some() {
            vec!["document", "patch", "report"]
        } else {
            vec!["document"]
        }
    } else {
        delivery.emit.iter().map(String::as_str).collect()
    };
    for (key, value) in [
        ("author", serde_json::json!(author)),
        ("datetime", serde_json::json!(datetime)),
        ("track", track),
        ("existing_revisions", existing_revisions),
        ("emit", serde_json::json!(emit)),
        ("force", serde_json::json!(delivery.force)),
        ("dry_run", serde_json::json!(delivery.dry_run)),
        (
            "report",
            serde_json::json!(report_path.as_deref().map(absolute)),
        ),
    ] {
        receipt.effective_options.insert(key.into(), value);
    }
    for finding in &prepared.checks.introduced {
        receipt.warnings.push(format!(
            "{} in {}#{}: {}",
            finding.code, finding.part, finding.location, finding.message
        ));
    }
    let data = receipt.data.as_mut().expect("data");
    data.verification = prepared.checks.verification();
    data.removed_comments = prepared.result.report.removed_comments.clone();
    data.kept_comments = prepared.result.report.kept_comments.clone();
    if let Some(report) = &expectations {
        data.verification.expectations = "passed".to_string();
        data.expectations = Some(report.clone());
    }
    data.counts.requested_operations = request.count();
    data.counts.body_operations_applied = prepared
        .result
        .report
        .operations
        .iter()
        .filter(|op| !op.kind.contains("comment") && op.status == "ok")
        .count();
    data.counts.review_operations_applied = prepared
        .result
        .report
        .operations
        .iter()
        .filter(|op| op.kind.contains("comment") && op.status == "ok")
        .count();
    data.counts.unchanged_operations = request
        .count()
        .saturating_sub(prepared.result.report.operations.len());
    data.counts.comments_added = prepared.result.report.comments_added;
    data.counts.revisions_accepted = prepared.result.report.resolved_revisions.accepted.len();
    data.counts.revisions_rejected = prepared.result.report.resolved_revisions.rejected.len();
    // Count the published primary. v2 publishes the redline (track off
    // leaves its revision total at zero); v1 apply asks for that role too.
    let markup = jubarte::cli::wrote_markup_detail(
        prepared.result.report.revisions.total,
        prepared.result.report.comments_added,
        &prepared.result.report.author,
    );
    let redacted = prepared
        .result
        .report
        .operations
        .iter()
        .any(|op| op.kind == "redact");
    if delivery.dry_run {
        data.set_predicted(if prepared.primary_effect == Effect::Changed {
            PredictedEffect::Changed
        } else {
            PredictedEffect::Unchanged
        });
        return Ok((receipt, redacted, markup));
    }
    // #1997: a write that signs markup needs a spelled -a (or
    // JUBARTE_AUTHOR). It refuses before anything is published. Saving a
    // plan signs nothing yet; `edit apply` carries the plan's own author.
    if markup.is_some() && save.is_none() && !action.author_was_given() {
        return Err(failure("AUTHOR_REQUIRED", author_required_message(source)));
    }
    let mut outputs = Vec::<(PathBuf, Vec<u8>, MediaRole)>::new();
    if let Some(path) = save {
        let Request::V2(plan) = &request else {
            unreachable!()
        };
        let mut bytes = serde_json::to_vec_pretty(plan).expect("plan serializes");
        bytes.push(b'\n');
        outputs.push((path.into(), bytes, MediaRole::Plan));
    } else if let Some(path) = &document {
        outputs.push((path.clone(), prepared.primary.clone(), MediaRole::Document));
        if delivery.emit.iter().any(|r| r == "patch") {
            let patch = jubarte::text_diff::diff_documents(
                jubarte::markdown::Source::Docx(source),
                jubarte::markdown::Source::Docx(&prepared.primary),
                &jubarte::text_diff::UnifiedOptions::default(),
            )
            .map_err(io_error)?;
            outputs.push((
                path.with_extension("diff"),
                patch.into_bytes(),
                MediaRole::Patch,
            ));
        }
    } else if let Some(dir) = &delivery.out_dir {
        outputs.push((
            dir.join("document.docx"),
            prepared.primary.clone(),
            MediaRole::Document,
        ));
        if delivery.emit.is_empty() || delivery.emit.iter().any(|r| r == "patch") {
            let patch = jubarte::text_diff::diff_documents(
                jubarte::markdown::Source::Docx(source),
                jubarte::markdown::Source::Docx(&prepared.primary),
                &jubarte::text_diff::UnifiedOptions::default(),
            )
            .map_err(io_error)?;
            outputs.push((
                dir.join("changes.diff"),
                patch.into_bytes(),
                MediaRole::Patch,
            ));
        }
    }
    for (path, bytes, role) in &outputs {
        receipt.artifacts.push(artifact(path, *role, bytes));
    }
    if let Some(path) = report_path.as_ref() {
        if outputs.iter().any(|(other, _, _)| same_path(path, other)) {
            return Err(failure(
                "OUTPUT_ALIAS",
                "report aliases another output role",
            ));
        }
        let mut bytes = serde_json::to_vec_pretty(&receipt).expect("receipt serializes");
        bytes.push(b'\n');
        receipt
            .artifacts
            .push(artifact(path, MediaRole::Report, &bytes));
        outputs.push((path.clone(), bytes, MediaRole::Report));
    }
    // Recheck immutable input bytes and all path identities at the commit boundary.
    if read(action.file())? != source {
        return Err(failure("STALE_SOURCE", "source changed during preparation"));
    }
    if let Some(path) = plan_path
        && Some(read(path)?) != plan_bytes
    {
        return Err(failure("STALE_PLAN", "plan changed during preparation"));
    }
    if let Some(path) = action.payload().and_then(|p| p.text_file.as_deref())
        && Some(read(path)?) != payload_bytes
    {
        return Err(failure(
            "STALE_RESOURCE",
            "text payload changed during preparation",
        ));
    }
    // The in-place document is the source, so only that role skips the
    // alias check. A plan, patch or report that names FILE still refuses.
    // `--in-place` replaces the input only. A patch or report still needs
    // `--force`.
    let replaces = |path: &Path, role: MediaRole| {
        delivery.force
            || (delivery.in_place && role == MediaRole::Document && same_path(path, action.file()))
    };
    for (i, (path, _, role)) in outputs.iter().enumerate() {
        let alias_inputs: Vec<(&Path, &str)> = if delivery.in_place
            && *role == MediaRole::Document
            && same_path(path, action.file())
        {
            inputs
                .iter()
                .copied()
                .filter(|(input, _)| !same_path(input, action.file()))
                .collect()
        } else {
            inputs.clone()
        };
        writable(path, &alias_inputs, replaces(path, *role))?;
        for (other, _, _) in &outputs[..i] {
            if same_path(path, other) {
                return Err(failure(
                    "OUTPUT_ALIAS",
                    "two output roles name the same file",
                ));
            }
        }
    }
    if let Some(dir) = &delivery.out_dir {
        // Keep bundle publication a single atomic directory rename.
        if delivery.report.is_some() {
            return Err(failure(
                "USAGE",
                "use the bundle's report.json or a separate -o document with --report",
            ));
        }
        let stage = stage_bundle_dir(dir).map_err(io_error)?;
        let manifest =
            serde_json::json!({"schema_version":1,"artifacts":receipt.artifacts}).to_string();
        let mut files: Vec<_> = outputs
            .iter()
            .map(|(path, bytes, _)| {
                (
                    stage.join(path.file_name().expect("bundle member")),
                    bytes.as_slice(),
                )
            })
            .collect();
        files.push((stage.join("manifest.json"), manifest.as_bytes()));
        if let Err(error) = publish_paths(&files, false) {
            let _ = std::fs::remove_dir_all(&stage);
            return Err(io_error(error.to_string()));
        }
        publish_dir(&stage, dir, false, || {
            for (path, _, _) in &outputs {
                refuse_output_alias(path, &inputs)?;
            }
            refuse_bundle_input_containment(dir, &inputs)?;
            recheck_edit_source(action.file(), source)
        })
        .map_err(publish_failure)?;
    } else {
        let mut files: Vec<_> = outputs
            .iter()
            .map(|(path, bytes, _)| (path.clone(), bytes.as_slice()))
            .collect();
        // The input is last, so a companion that cannot be committed leaves it.
        if delivery.in_place {
            files.sort_by_key(|(path, _)| same_path(path, action.file()));
        }
        publish_files(
            &files,
            &Recheck {
                force: delivery.force,
                replace_through: &|path| delivery.in_place && same_path(path, action.file()),
                alias: &|path| {
                    let filtered: Vec<(&Path, &str)> =
                        if delivery.in_place && same_path(path, action.file()) {
                            inputs
                                .iter()
                                .copied()
                                .filter(|(input, _)| !same_path(input, action.file()))
                                .collect()
                        } else {
                            inputs.clone()
                        };
                    refuse_output_alias(path, &filtered)
                },
                source: &|| recheck_edit_source(action.file(), source),
            },
        )
        .map_err(publish_failure)?;
    }
    Ok((receipt, redacted, markup))
}

pub(super) fn run(typed: &TypedEdit, quiet: bool) -> ExitCode {
    let action = &typed.action;
    let command = format!("edit {}", action.name());
    let source = read(action.file());
    if let Ok(source) = &source {
        super::warn_nested_text_boxes(source);
    }
    let (receipt, redacted, markup) = match source.as_ref() {
        Ok(source) => prepare_and_publish(action, source, &command).unwrap_or_else(|error| {
            let operational = matches!(
                error.code.as_str(),
                "IO_ERROR"
                    | "OUTPUT_EXISTS"
                    | "USAGE"
                    | "OUTPUT_ALIAS"
                    | "INVALID_UTF8"
                    | "AUTHOR_REQUIRED"
            );
            let mut receipt = if operational {
                Envelope::new_error(&command, Source::new(source), *error)
            } else {
                Envelope::refused(&command, Source::new(source), *error)
            };
            // The caller supplied an expectation contract this request
            // cannot run (v1 plan): the receipt says so instead of
            // claiming none was requested (#1084 review F4).
            if !operational
                && receipt
                    .error
                    .as_ref()
                    .is_some_and(|e| e.code == "UNSUPPORTED_EXPECTATION")
                && let Some(data) = receipt.data.as_mut()
            {
                data.verification.expectations = "unresolved".to_string();
            }
            (receipt, false, None)
        }),
        Err(error) => (
            Envelope::new_error(
                &command,
                Source::from_digest("").with_path(absolute(action.file())),
                *error.clone(),
            ),
            false,
            None,
        ),
    };
    let code = edit_envelope_exit(&receipt);
    let delivery = action.delivery();
    if !delivery.json && !delivery.jsonl && receipt.status != Status::Ok {
        let error = receipt.error.as_ref().expect("failure error");
        eprintln!("error: {}: {}", error.code, error.message);
        if let Some(near) = &error.near {
            eprintln!("{}", near.copy_hint());
        }
        return ExitCode::from(code);
    }
    let rendered = if delivery.jsonl {
        jubarte::result::write_jsonl_stream(&[], &receipt)
    } else if delivery.json {
        format!(
            "{}\n",
            serde_json::to_string(&receipt).expect("receipt serializes")
        )
    } else if receipt.status == Status::Ok {
        let data = receipt.data.as_ref().expect("success data");
        let outcome = match data.phase {
            Some(Phase::Generated) => "saved plan",
            Some(Phase::Validated) => "validated",
            _ if data.effect == Effect::Unchanged => "unchanged",
            _ => "changed",
        };
        let replaced = source
            .as_ref()
            .map(|bytes| jubarte::inspect::source_sha256(bytes))
            .unwrap_or_default();
        let paths = receipt
            .artifacts
            .iter()
            .map(|artifact| {
                if delivery.in_place && artifact.media_role == MediaRole::Document {
                    let prefix = replaced.get(..8).unwrap_or(replaced.as_str());
                    format!(
                        "{} in place (replaced sha256 {prefix}\u{2026})",
                        artifact.path
                    )
                } else {
                    artifact.path.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        // An expectations note that stays honest about what ran and what
        // stayed unassessed (proposal §10's completion reporting).
        let expectations = data
            .expectations
            .as_ref()
            .map(|report| {
                let receipt = &report.receipt;
                let mut note = format!("; {} expectation checks run", receipt.checks_run);
                if receipt.requirements_unassessed > 0 {
                    note.push_str(&format!(
                        ", {} requirement{} not assessed",
                        receipt.requirements_unassessed,
                        if receipt.requirements_unassessed == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ));
                }
                note
            })
            .unwrap_or_default();
        let authored = if paths.is_empty() || !matches!(data.phase, Some(Phase::Committed)) {
            String::new()
        } else {
            markup
                .as_deref()
                .map(|detail| format!("; {detail}"))
                .unwrap_or_default()
        };
        format!(
            "{}: {outcome}{}{authored}{expectations}\n",
            receipt.command,
            if paths.is_empty() {
                String::new()
            } else {
                format!("; wrote {paths}")
            }
        )
    } else {
        format!(
            "{}: {}\n",
            receipt.error.as_ref().expect("error").code,
            receipt.error.as_ref().expect("error").message
        )
    };
    if (!quiet || delivery.json || delivery.jsonl || code != 0)
        && let Err(error) = write_stdout(rendered)
    {
        eprintln!(
            "error: {error}; commit state is {}",
            if receipt
                .data
                .as_ref()
                .is_some_and(|d| d.phase == Some(Phase::Committed))
            {
                "committed"
            } else {
                "see receipt"
            }
        );
        return ExitCode::from(1);
    }
    if code == 0
        && !quiet
        && !delivery.json
        && !delivery.jsonl
        && receipt
            .data
            .as_ref()
            .is_some_and(|d| d.phase == Some(Phase::Committed))
        && let Ok(source) = &source
    {
        let path = if delivery.in_place {
            Some(action.file().to_path_buf())
        } else {
            delivery
                .output
                .clone()
                .or_else(|| delivery.out_dir.as_ref().map(|d| d.join("document.docx")))
        };
        if let Some(path) = path {
            // Read the committed artifact, not the prepared transaction.
            let published = std::fs::read(&path);
            match published {
                Ok(bytes) => {
                    let data = receipt.data.as_ref().expect("success data");
                    for line in super::settlement_notes(
                        &data.removed_comments,
                        &data.kept_comments,
                        &bytes,
                        source,
                        "settlement",
                    ) {
                        edit_status(format_args!("{line}"));
                    }
                    if let Err(error) =
                        print_edit_success(action.file(), source, &path, &bytes, redacted)
                    {
                        eprintln!(
                            "error: committed {}, but reporting the result failed: {error}",
                            path.display()
                        );
                        return ExitCode::from(1);
                    }
                }
                Err(error) => edit_status(format_args!(
                    "warning: the published document cannot be read: {error}"
                )),
            }
        }
    }
    ExitCode::from(code)
}

/// Emit only recognized typed-route usage failures in the requested protocol.
pub(super) fn usage(argv: &[String], error: &clap::Error) -> bool {
    if matches!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
    ) {
        return false;
    }
    let args: Vec<std::ffi::OsString> = std::iter::once("jubarte")
        .chain(argv.iter().map(String::as_str))
        .map(Into::into)
        .collect();
    let Some((action, jsonl, file)) = jubarte::cli::typed_edit_protocol(&args) else {
        return false;
    };
    // Name the file the tokens carried when they named one: a usage
    // failure the caller cannot re-associate with a file is
    // undiagnosable (#1084 review F5).
    let source = match file {
        Some(file) => Source::from_digest("").with_path(absolute(Path::new(&file))),
        None => Source::from_digest(""),
    };
    let receipt = Envelope::new_error(
        format!("edit {action}"),
        source,
        *failure("USAGE", error.to_string().replace("__edit-v2", "edit")),
    );
    let rendered = if jsonl {
        jubarte::result::write_jsonl_stream(&[], &receipt)
    } else {
        format!(
            "{}\n",
            serde_json::to_string(&receipt).expect("receipt serializes")
        )
    };
    if let Err(error) = write_stdout(rendered) {
        eprintln!("error: {error}");
    }
    true
}
