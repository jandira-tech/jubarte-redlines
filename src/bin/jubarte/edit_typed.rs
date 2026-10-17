// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Typed edit orchestration. Reuses candidate preparation and atomic publication.

use super::*;
use jubarte::cli::edit_actions::{EditAction, TypedEdit};
use jubarte::edit::{EditPlan, PrepareOptions, PreparedEdit, v2::PlanV2};
use jubarte::result::{
    Artifact, CommitState, Effect, Envelope, Error, MediaRole, Phase, PredictedEffect, ResultData,
    Runtime, Source, Status,
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
    error
}
fn io_error(message: String) -> Failure {
    failure("IO_ERROR", message)
}
/// A publication refusal or failure in this route's vocabulary. The shared
/// publisher names its refusals by a `CODE: ` prefix; `SOURCE_CHANGED` is
/// this route's `STALE_SOURCE`. Nothing is published before the primary, so
/// a failure that committed nothing is `not_committed`; after a commit the
/// primary's state is not reported back, and stays `unknown`.
fn publish_failure((_, message): (u8, String)) -> Failure {
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
    ensure_writable(path, force).map_err(|message| failure("OUTPUT_EXISTS", message))
}

enum Request {
    V1(EditPlan),
    V2(PlanV2),
}
impl Request {
    fn prepare(&self, source: &[u8], options: &PrepareOptions) -> Result<PreparedEdit, Failure> {
        match self {
            Self::V2(plan) => jubarte::edit::prepare_v2(source, plan, options).map_err(edit_error),
            Self::V1(plan) => {
                let mut prepared =
                    jubarte::edit::prepare(source, plan, options).map_err(edit_error)?;
                // v1's saved request is a tracked edit. Candidate checks covered
                // both serialized representations; publish its redline as v1 does.
                if prepared.primary_effect == Effect::Changed {
                    prepared.primary = prepared.result.redline.clone();
                    prepared.primary_sha256 = jubarte::inspect::source_sha256(&prepared.primary);
                }
                Ok(prepared)
            }
        }
    }
    fn count(&self) -> usize {
        match self {
            Self::V1(p) => p.operations.len(),
            Self::V2(p) => p.operations.len(),
        }
    }
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
        let version: serde_json::Value =
            serde_json::from_str(json).map_err(|e| failure("INVALID_PLAN", e.to_string()))?;
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
                let mut plan = EditPlan::from_json(json).map_err(edit_error)?;
                let binding = check_hash(
                    plan.source_sha256.as_deref(),
                    delivery.expect_sha256.as_deref(),
                    source,
                    *allow_unbound_plan,
                )?;
                if plan.source_sha256.is_none() {
                    plan.source_sha256.clone_from(&delivery.expect_sha256);
                }
                Ok((Request::V1(plan), binding))
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

fn absolute(path: &Path) -> String {
    let path = std::fs::canonicalize(path)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path));
    slash_path(&path)
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
) -> Result<Envelope, Failure> {
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
    if delivery.expect.is_some() {
        return Err(failure(
            "UNSUPPORTED_EXPECTATION",
            "this build has no typed expectation evaluator; no contract was silently ignored",
        ));
    }
    let (request, binding) = request(action, source, payload, plan_bytes.as_deref())?;
    let prepared = request.prepare(
        source,
        &PrepareOptions {
            require_change: delivery.require_change,
            exact_text: matches!(request, Request::V2(_)),
        },
    )?;
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
    let report_path = delivery.report.clone().or_else(|| {
        if delivery.out_dir.is_some()
            && (delivery.emit.is_empty() || delivery.emit.iter().any(|r| r == "report"))
        {
            delivery.out_dir.as_ref().map(|dir| dir.join("report.json"))
        } else if delivery.emit.iter().any(|r| r == "report") {
            delivery
                .output
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
        Request::V1(_) => (
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
    if delivery.dry_run {
        data.set_predicted(if prepared.primary_effect == Effect::Changed {
            PredictedEffect::Changed
        } else {
            PredictedEffect::Unchanged
        });
        return Ok(receipt);
    }
    let mut outputs = Vec::<(PathBuf, Vec<u8>, MediaRole)>::new();
    if let Some(path) = save {
        let Request::V2(plan) = &request else {
            unreachable!()
        };
        let mut bytes = serde_json::to_vec_pretty(plan).expect("plan serializes");
        bytes.push(b'\n');
        outputs.push((path.into(), bytes, MediaRole::Plan));
    } else if let Some(path) = &delivery.output {
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
    for (i, (path, _, _)) in outputs.iter().enumerate() {
        writable(path, &inputs, delivery.force)?;
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
        for (path, bytes, _) in &outputs {
            let name = path.file_name().expect("bundle member");
            if let Err(e) = std::fs::write(stage.join(name), bytes) {
                let _ = std::fs::remove_dir_all(&stage);
                return Err(io_error(e.to_string()));
            }
        }
        let manifest = serde_json::json!({"schema_version":1,"artifacts":receipt.artifacts});
        if let Err(e) = std::fs::write(stage.join("manifest.json"), manifest.to_string()) {
            let _ = std::fs::remove_dir_all(&stage);
            return Err(io_error(e.to_string()));
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
        let files: Vec<_> = outputs
            .iter()
            .map(|(path, bytes, _)| (path.clone(), bytes.as_slice()))
            .collect();
        publish_files(
            &files,
            &Recheck {
                force: delivery.force,
                alias: &|path| refuse_output_alias(path, &inputs),
                source: &|| recheck_edit_source(action.file(), source),
            },
        )
        .map_err(publish_failure)?;
    }
    Ok(receipt)
}

pub(super) fn run(typed: &TypedEdit, quiet: bool) -> ExitCode {
    let action = &typed.action;
    let command = format!("edit {}", action.name());
    let source = read(action.file());
    let receipt = match source.as_ref() {
        Ok(source) => prepare_and_publish(action, source, &command).unwrap_or_else(|error| {
            if matches!(
                error.code.as_str(),
                "IO_ERROR"
                    | "OUTPUT_EXISTS"
                    | "USAGE"
                    | "OUTPUT_ALIAS"
                    | "INVALID_UTF8"
                    | "UNSUPPORTED_EXPECTATION"
            ) {
                Envelope::new_error(&command, Source::new(source), *error)
            } else {
                Envelope::refused(&command, Source::new(source), *error)
            }
        }),
        Err(error) => Envelope::new_error(&command, Source::from_digest(""), *error.clone()),
    };
    let code: u8 = jubarte::result::exit_status_for_envelope(&receipt).into();
    let delivery = action.delivery();
    if !delivery.json && !delivery.jsonl && receipt.status != Status::Ok {
        let error = receipt.error.as_ref().expect("failure error");
        eprintln!("error: {}: {}", error.code, error.message);
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
        let paths = receipt
            .artifacts
            .iter()
            .map(|a| a.path.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}: {outcome}{}\n",
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
    if !(quiet && !delivery.json && !delivery.jsonl && code == 0)
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
    let Some((action, jsonl)) = jubarte::cli::typed_edit_protocol(&args) else {
        return false;
    };
    let receipt = Envelope::new_error(
        format!("edit {action}"),
        Source::from_digest(""),
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
