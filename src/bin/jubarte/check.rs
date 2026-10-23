// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte check FILE --expect EXPECTATIONS.json` (V2-13, issue #845).
//!
//! The standalone expectation check of proposal §10 / edit plan P5: the
//! expectations' `source.sha256` must name FILE's own bytes (references
//! are bound to that output snapshot; pre-edit ids on an edited output
//! are not output ids), evaluation runs the identity correspondence over
//! FILE, and the final human line states the check's scope. Exit 0 when
//! every executable check passes, 2 when one fails, 3 for refused or
//! stale inputs, 64 for a malformed request. The other check profiles
//! are #961: without `--expect` the command refuses usage naming it.

use super::*;
use jubarte::expect::{
    CheckOutcome, ExpectationReport, Outcome, evaluate_with_receipt, identity_correspondence,
    parse_expectations,
};
use jubarte::result::{
    self, Effect, Envelope, Error, Phase, ResultData, ResultDataError, Runtime, Source, Status,
    Verification,
};

const COMMAND: &str = "check";

/// Print the envelope (`--json`) or the final scope line, and map the
/// proposal §12 exit code.
fn finish(
    envelope: Result<Envelope, ResultDataError>,
    line: Option<String>,
    json: bool,
    quiet: bool,
) -> ExitCode {
    let receipt = match envelope {
        Ok(receipt) => receipt,
        Err(e) => {
            eprintln!("error: invalid result envelope: {e}");
            return ExitCode::from(1);
        }
    };
    let code: u8 = result::exit_status_for_envelope(&receipt).into();
    if json {
        let rendered = format!(
            "{}\n",
            serde_json::to_string(&receipt).expect("receipt serializes")
        );
        if let Err(error) = write_stdout(rendered) {
            eprintln!("error: {error}");
            return ExitCode::from(1);
        }
        return ExitCode::from(code);
    }
    if let Some(line) = line {
        if !(quiet && code == 0)
            && let Err(error) = write_stdout(format!("{line}\n"))
        {
            eprintln!("error: {error}");
            return ExitCode::from(1);
        }
        return ExitCode::from(code);
    }
    if receipt.status != Status::Ok
        && let Some(error) = &receipt.error
    {
        eprintln!("error: {}: {}", error.code, error.message);
    }
    ExitCode::from(code)
}

/// The final human line that states the check's scope, e.g. "Expectation
/// checks passed: 3 run; 1 requirement not assessed."
fn scope_line(state: &str, report: &ExpectationReport, failed: usize) -> String {
    let receipt = &report.receipt;
    let mut line = format!("Expectation checks {state}: {} run", receipt.checks_run);
    if failed > 0 {
        line.push_str(&format!(", {failed} failed"));
    }
    let refused = receipt.checks_refused + receipt.checks_unsupported;
    if refused > 0 {
        line.push_str(&format!(", {refused} not evaluated"));
    }
    if receipt.requirements_unassessed > 0 {
        let n = receipt.requirements_unassessed;
        line.push_str(&format!(
            "; {n} requirement{} not assessed",
            if n == 1 { "" } else { "s" }
        ));
    }
    line.push('.');
    line
}

fn usage(source: Source, message: String) -> Result<Envelope, ResultDataError> {
    Ok(Envelope::new_error(
        COMMAND,
        source,
        Error::new("USAGE").with_message(message),
    ))
}

fn refused(source: Source, error: Error) -> Result<Envelope, ResultDataError> {
    Ok(Envelope::refused(COMMAND, source, error))
}

fn io_error(message: String) -> Error {
    Error::new("IO_ERROR").with_message(message)
}

/// The inspected file's own `Source` for failures that never read its
/// bytes: the user-named path, no invented digest (#1084 review F5).
fn path_only(file: &Path) -> Source {
    Source::from_digest("").with_path(absolute(file))
}

/// One named failing or refused check, for the error message.
fn first_of(report: &ExpectationReport, outcome: Outcome) -> Option<&CheckOutcome> {
    report.checks.iter().find(|c| c.outcome == outcome)
}

/// Run the check. The expectations' `source.sha256` must name the
/// inspected file's own bytes; evaluation uses the identity
/// correspondence on FILE (the inspected file IS the reopened output).
pub(super) fn run(file: &Path, expect: Option<&Path>, json: bool, quiet: bool) -> ExitCode {
    let Some(expect_path) = expect else {
        return finish(
            usage(
                path_only(file),
                "check requires --expect FILE in this build; the integrity, quality and layout \
                 profiles are not implemented yet (#961)"
                    .to_string(),
            ),
            None,
            json,
            quiet,
        );
    };
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(e) => {
            let error = io_error(format!("reading {}: {e}", file.display()));
            return finish(
                Ok(Envelope::new_error(COMMAND, path_only(file), error)),
                None,
                json,
                quiet,
            );
        }
    };
    let source = Source::new(&bytes).with_path(absolute(file));
    let contract = match std::fs::read(expect_path) {
        Ok(contract) => contract,
        Err(e) => {
            // An unreadable --expect file is an input/I/O failure (§12:
            // exit 1), the same class `edit apply --expect` reports.
            let error = io_error(format!("reading {}: {e}", expect_path.display()));
            return finish(
                Ok(Envelope::new_error(COMMAND, source.clone(), error)),
                None,
                json,
                quiet,
            );
        }
    };
    let parse_or_usage = |message: String| usage(source.clone(), message);
    let text = match std::str::from_utf8(&contract) {
        Ok(text) => text,
        Err(e) => {
            return finish(
                parse_or_usage(format!(
                    "--expect {} is not UTF-8: {e}",
                    expect_path.display()
                )),
                None,
                json,
                quiet,
            );
        }
    };
    let spec = match parse_expectations(text) {
        Ok(spec) => spec,
        Err(e) => {
            return finish(
                parse_or_usage(format!("--expect {}: {e}", expect_path.display())),
                None,
                json,
                quiet,
            );
        }
    };
    // The expectations hash and references must be bound to the inspected
    // file's own snapshot: an expectation file written for the pre-edit
    // source refuses as stale on an edited output.
    if spec.source.sha256 != source.sha256 {
        return finish(
            refused(
                source.clone(),
                Error::new("STALE_SOURCE").with_message(
                    "expectations source.sha256 does not name the inspected file's bytes; \
                     references are bound to that output snapshot",
                ),
            ),
            None,
            json,
            quiet,
        );
    }
    let correspondence = match identity_correspondence(&bytes) {
        Ok(correspondence) => correspondence,
        Err(e) => {
            let error = Error::new("INVALID_DOCUMENT")
                .with_message(format!("cannot project {}: {e}", file.display()));
            return finish(
                Ok(Envelope::new_error(COMMAND, source.clone(), error)),
                None,
                json,
                quiet,
            );
        }
    };
    let report = evaluate_with_receipt(
        &spec,
        &bytes,
        &bytes,
        &correspondence,
        jubarte::expect::ReceiptMeta {
            path: Some(expect_path.display().to_string()),
            digest: Some(jubarte::expect::content_sha256(&contract)),
            declared_provenance: None,
        },
    );
    let failed = report
        .checks
        .iter()
        .filter(|c| c.outcome == Outcome::Failed)
        .count();
    let not_evaluated = report.receipt.checks_refused + report.receipt.checks_unsupported;
    let verification = Verification {
        expectations: if failed > 0 {
            "failed".to_string()
        } else if not_evaluated > 0 {
            "unresolved".to_string()
        } else {
            "passed".to_string()
        },
        ..Default::default()
    };
    let mut data = ResultData::new(Phase::Inspected, Effect::NotApplied);
    data.verification = verification;
    data.expectations = Some(report.clone());
    if failed > 0 || not_evaluated > 0 {
        let state = if failed > 0 { "failed" } else { "refused" };
        let line = scope_line(state, &report, failed);
        let (code, message) = if failed > 0 {
            let first = first_of(&report, Outcome::Failed).expect("failed count is nonzero");
            (
                result::codes::CHECK_THRESHOLD_EXCEEDED,
                format!(
                    "{} of {} expectation checks failed; first: requirement {:?} on {} ({:?})",
                    failed, report.receipt.checks_run, first.key, first.target, first.kind
                ),
            )
        } else {
            let first = first_of(&report, Outcome::Refused).expect("refused count is nonzero");
            (
                "EXPECTATION_REFUSED",
                format!(
                    "{} expectation checks could not be evaluated; first: requirement {:?} on {} \
                     ({:?}): {}",
                    not_evaluated,
                    first.key,
                    first.target,
                    first.kind,
                    first
                        .reason
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                ),
            )
        };
        let mut error = Error::new(code).with_message(message);
        error.expectations = Some(report);
        return finish(
            Envelope::refused(COMMAND, source, error).with_failure_data(data),
            Some(line),
            json,
            quiet,
        );
    }
    finish(
        Envelope::ok(COMMAND, source, data, Runtime::native()),
        Some(scope_line("passed", &report, 0)),
        json,
        quiet,
    )
}
