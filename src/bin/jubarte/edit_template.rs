// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Read-only template generation and atomic publication of the plan artifact.
use super::*;
use jubarte::cli::edit_template::TemplateOptions;
use jubarte::edit::{
    ExistingRevisions,
    for_edit::ForEditRequest,
    template::{self, TemplateRequest},
};
use jubarte::result::{
    Artifact, CommitState, Effect, Envelope, Error, MediaRole, Phase, ResultData, Source, Status,
};

type Failure = Box<Error>;
fn failure(code: &str, message: impl Into<String>) -> Failure {
    Box::new(
        Error::new(code)
            .with_message(message)
            .with_commit_state(CommitState::NotCommitted),
    )
}
fn read(path: &Path) -> Result<Vec<u8>, Failure> {
    std::fs::read(path).map_err(|e| failure("IO_ERROR", format!("reading {}: {e}", path.display())))
}
fn alias(path: &Path, input: &Path) -> Result<(), Failure> {
    refuse_output_alias(path, &[(input, "the source")]).map_err(|(_, m)| failure("OUTPUT_ALIAS", m))
}
fn absolute(path: &Path) -> String {
    slash_path(
        &std::fs::canonicalize(path)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path)),
    )
}
fn generate(
    options: &TemplateOptions,
    source: &[u8],
) -> Result<(Envelope, Option<Vec<u8>>), Failure> {
    if options
        .expect_sha256
        .as_ref()
        .is_some_and(|hash| *hash != jubarte::inspect::source_sha256(source))
    {
        return Err(failure(
            "STALE_SOURCE",
            "--expect-sha256 does not match the inspected source bytes",
        ));
    }
    let output = options.output.as_deref().filter(|p| *p != Path::new("-"));
    let paths: Vec<_> = [output, options.report.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    for (i, path) in paths.iter().enumerate() {
        alias(path, &options.file)?;
        if paths[..i].iter().any(|p| same_path(p, path)) {
            return Err(failure(
                "OUTPUT_ALIAS",
                "plan and report name the same file",
            ));
        }
    }
    let generated = template::generate(
        source,
        &TemplateRequest {
            view: ForEditRequest {
                select: options.select.clone(),
                story: Some(options.story.clone()),
                find: options.find.clone(),
                regex: options.regex.clone(),
                ignore_case: options.ignore_case,
                changed: options.changed,
                by_any: options.by.clone(),
                context: options.context,
                accept: options.accept_change.clone(),
                reject: options.reject_change.clone(),
                existing_revisions: match options.existing_revisions.as_str() {
                    "keep" => ExistingRevisions::Keep,
                    "accept" => ExistingRevisions::Accept,
                    "reject" => ExistingRevisions::Reject,
                    _ => ExistingRevisions::Refuse,
                },
                track: options.track == "on",
                ..Default::default()
            },
            author: options.author.clone(),
            date: options
                .datetime
                .clone()
                .unwrap_or_else(jubarte::convert::utc_now_iso8601),
        },
    )
    .map_err(|e| failure(&e.code, e.message))?;
    let mut data = ResultData::new(Phase::Generated, Effect::NotApplied);
    data.generation =
        Some(serde_json::to_value(&generated.summary).expect("generation serializes"));
    let mut receipt = Envelope::ok(
        "edit template",
        Source::new(source),
        data,
        jubarte::result::Runtime::native(),
    )
    .expect("generated/not_applied");
    receipt.source.path = Some(absolute(&options.file));
    receipt.effective_options.insert(
        "scope".into(),
        serde_json::to_value(&generated.plan.scope).expect("scope serializes"),
    );
    receipt.effective_options.insert(
        "policy".into(),
        serde_json::to_value(&generated.plan.policy).expect("policy serializes"),
    );
    receipt.effective_options.insert(
        "resolve_revisions".into(),
        serde_json::to_value(&generated.plan.resolve_revisions).expect("decisions serialize"),
    );
    if options.dry_run {
        return Ok((receipt, None));
    }
    let mut bytes = serde_json::to_vec_pretty(&generated.plan).expect("PlanV2 serializes");
    bytes.push(b'\n');
    if output.is_none() && options.report.is_none() {
        return Ok((receipt, Some(bytes)));
    }
    // Protect the exact source snapshot and path identities again at publication.
    if read(&options.file)? != source {
        return Err(failure("STALE_SOURCE", "source changed during generation"));
    }
    for path in &paths {
        alias(path, &options.file)?;
        ensure_writable(path, options.force).map_err(|m| failure("OUTPUT_EXISTS", m))?;
    }
    if let Some(output) = output {
        if options
            .report
            .as_ref()
            .is_some_and(|p| same_path(p, output))
        {
            return Err(failure(
                "OUTPUT_ALIAS",
                "plan and report name the same file",
            ));
        }
        receipt.artifacts.push(Artifact::committed(
            absolute(output),
            MediaRole::Plan,
            "application/json",
            &bytes,
        ));
    }
    let report = options.report.as_ref().map(|path| {
        let mut bytes = serde_json::to_vec_pretty(&receipt).expect("receipt serializes");
        bytes.push(b'\n');
        receipt.artifacts.push(Artifact::committed(
            absolute(path),
            MediaRole::Report,
            "application/json",
            &bytes,
        ));
        bytes
    });
    let mut outputs = Vec::new();
    if let Some(output) = output {
        outputs.push((output.to_path_buf(), bytes.as_slice()));
    }
    if let Some((path, bytes)) = options.report.as_ref().zip(report.as_ref()) {
        outputs.push((path.clone(), bytes.as_slice()));
    }
    publish_files(&outputs).map_err(|message| {
        Box::new(
            Error::new("IO_ERROR")
                .with_message(message)
                .with_commit_state(CommitState::Unknown),
        )
    })?;
    Ok((receipt, output.is_none().then_some(bytes)))
}

pub(super) fn run(options: &TemplateOptions, quiet: bool) -> ExitCode {
    let source = read(&options.file);
    let outcome = source
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|source| generate(options, source));
    let (receipt, raw) = match outcome {
        Ok(ok) => ok,
        Err(error) => {
            let source = source
                .as_ref()
                .map_or_else(|_| Source::from_digest(""), |s| Source::new(s));
            let receipt = if matches!(
                error.code.as_str(),
                "IO_ERROR" | "OUTPUT_ALIAS" | "OUTPUT_EXISTS" | "USAGE"
            ) {
                Envelope::new_error("edit template", source, *error)
            } else {
                Envelope::refused("edit template", source, *error)
            };
            (receipt, None)
        }
    };
    let code: u8 = jubarte::result::exit_status_for_envelope(&receipt).into();
    let rendered = if let Some(raw) = raw {
        String::from_utf8(raw).expect("JSON is UTF-8")
    } else if options.jsonl {
        jubarte::result::write_jsonl_stream(&[], &receipt)
    } else if options.json {
        format!(
            "{}\n",
            serde_json::to_string(&receipt).expect("receipt serializes")
        )
    } else if receipt.status != Status::Ok {
        let error = receipt.error.as_ref().expect("failure error");
        eprintln!("error: {}: {}", error.code, error.message);
        return ExitCode::from(code);
    } else if quiet {
        return ExitCode::from(0);
    } else {
        let generation = receipt
            .data
            .as_ref()
            .and_then(|d| d.generation.as_ref())
            .expect("generation");
        let paths = receipt
            .artifacts
            .iter()
            .map(|a| a.path.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "edit template: {} rows, {} conditional, {} read-only paragraphs, {} opaque regions; no document changes applied{}\n",
            generation["rows_generated"],
            generation["coverage"]["conditional_paragraphs"],
            generation["read_only_paragraphs"],
            generation["coverage"]["opaque_regions"],
            if paths.is_empty() {
                String::new()
            } else {
                format!("; wrote {paths}")
            }
        )
    };
    if let Err(error) = write_stdout(rendered) {
        eprintln!(
            "error: {error}; committed artifacts: {}",
            receipt
                .artifacts
                .iter()
                .filter(|a| a.committed)
                .map(|a| format!("{} ({})", a.path, a.sha256))
                .collect::<Vec<_>>()
                .join(", ")
        );
        return ExitCode::from(1);
    }
    ExitCode::from(code)
}

/// Recognized machine mode applies even to invalid generator syntax.
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
    let routed = jubarte::cli::edit_actions::route(&args, &jubarte::cli::top_level_command());
    if !routed.iter().any(|a| a == "__edit-template") {
        return false;
    }
    let jsonl = argv.iter().any(|a| a == "--jsonl");
    if !jsonl && !argv.iter().any(|a| a == "--json") {
        return false;
    }
    let receipt = Envelope::new_error(
        "edit template",
        Source::from_digest(""),
        *failure(
            "USAGE",
            error
                .to_string()
                .replace("__edit-template", "edit template"),
        ),
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
