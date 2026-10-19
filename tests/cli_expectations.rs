// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! V2-13 wiring (issue #845): `edit apply --expect` and `jubarte check`
//! through the real binary.
//!
//! The fixtures are the design examples (`docs/design/cli-v2/examples`),
//! already copied byte-for-byte into `tests/fixtures/edit-v2`. Per
//! `examples/README.md`: the edited, exact-replace and with-comment plans
//! satisfy `expectations.json`; the generated (unchanged scaffold) and
//! composed plans intentionally fail it.

mod common;

use std::path::Path;

use common::docx::{docx, para};
use jubarte::inspect::source_sha256;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

const EXAMPLE_SOURCE: &[u8] = include_bytes!("fixtures/edit-v2/example-source.docx");
const EXAMPLE_EXPECTATIONS: &str = include_str!("fixtures/edit-v2/expectations.json");
const PLAN_EDITED: &str = include_str!("fixtures/edit-v2/template.edited.json");
const PLAN_COMPOSED: &str = include_str!("fixtures/edit-v2/template.composed.json");
const PLAN_GENERATED: &str = include_str!("fixtures/edit-v2/template.generated.json");

fn stage(dir: &Path) {
    std::fs::write(dir.join("example-source.docx"), EXAMPLE_SOURCE).unwrap();
    std::fs::write(dir.join("expectations.json"), EXAMPLE_EXPECTATIONS).unwrap();
    std::fs::write(dir.join("template.edited.json"), PLAN_EDITED).unwrap();
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = std::process::Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn json_envelope(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("not one JSON object: {e}\n{stdout}"))
}

fn field<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
    value
        .pointer(pointer)
        .unwrap_or_else(|| panic!("missing {pointer} in {value}"))
}

/// A two-paragraph document whose final text already satisfies a
/// self-authored expectation contract, and that contract bound to the
/// document's own digest (the standalone-check shape).
fn satisfying_fixture(dir: &Path) {
    let body =
        para("Payment is due within 45 days.") + &para("The fee is due; the charge is payable.");
    let bytes = docx(&body);
    let digest = source_sha256(&bytes);
    std::fs::write(dir.join("satisfied.docx"), &bytes).unwrap();
    std::fs::write(
        dir.join("satisfied.expect.json"),
        format!(
            r#"{{
  "schema_version": 1,
  "source": {{"sha256": "{digest}"}},
  "text_view": "final",
  "requirements": [
    {{"key": "payment-period", "description": "45-day payment period.",
      "checks": [{{"kind": "text_equals", "target": {{"short_id": "p0", "long_id": "document.w:p.j:short.p0"}}, "text": "Payment is due within 45 days."}}]}},
    {{"key": "fee-wording", "description": "The charge wording.",
      "checks": [{{"kind": "text_contains", "target": {{"short_id": "p1", "long_id": "document.w:p.j:short.p1"}}, "text": "the charge is payable"}}]}},
    {{"key": "commercial-review", "description": "Commercially appropriate terms.", "checks": []}}
  ]
}}"#
        ),
    )
    .unwrap();
}

/// Expectations identical in shape to `satisfied.expect.json` but wanting a
/// different payment period (a failing executable check).
fn failing_fixture(dir: &Path) {
    let body =
        para("Payment is due within 45 days.") + &para("The fee is due; the charge is payable.");
    let bytes = docx(&body);
    let digest = source_sha256(&bytes);
    std::fs::write(dir.join("satisfied.docx"), &bytes).unwrap();
    std::fs::write(
        dir.join("failing.expect.json"),
        format!(
            r#"{{
  "schema_version": 1,
  "source": {{"sha256": "{digest}"}},
  "text_view": "final",
  "requirements": [
    {{"key": "payment-period", "description": "60-day payment period.",
      "checks": [{{"kind": "text_equals", "target": {{"short_id": "p0", "long_id": "document.w:p.j:short.p0"}}, "text": "Payment is due within 60 days."}}]}}
  ]
}}"#
        ),
    )
    .unwrap();
}

/// A no-op PlanV2 (empty operations) bound to the given document's digest:
/// applying it is a valid unchanged state.
fn noop_plan(dir: &Path, docx_name: &str, plan_name: &str) {
    let digest = source_sha256(&std::fs::read(dir.join(docx_name)).unwrap());
    std::fs::write(
        dir.join(plan_name),
        format!(
            r#"{{
  "schema_version": 2,
  "source": {{"sha256": "{digest}"}},
  "policy": {{"track": "on", "existing_revisions": "keep", "text_view": "final"}},
  "resolve_revisions": {{"accept": [], "reject": []}},
  "author": "Jubarte",
  "date": "2026-10-17T07:00:00Z",
  "scope": {{"kind": "all"}},
  "operations": []
}}"#
        ),
    )
    .unwrap();
}

#[test]
fn edited_plan_satisfies_the_design_expectations() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "expectations.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "ok");
    assert_eq!(field(&envelope, "/data/effect"), "changed");
    assert_eq!(
        field(&envelope, "/data/verification/expectations"),
        "passed"
    );
    let receipt = field(&envelope, "/data/expectations/receipt");
    assert_eq!(field(receipt, "/schema_version"), 1);
    assert_eq!(field(receipt, "/checks_run"), 3);
    assert_eq!(field(receipt, "/checks_refused"), 0);
    assert_eq!(field(receipt, "/checks_unsupported"), 0);
    assert_eq!(field(receipt, "/requirements_unassessed"), 1);
    assert_eq!(
        field(receipt, "/source_sha256"),
        "54ef59e60b1986dd06131bfab3d6647d2f70b095ef3c09d698b4e221855c383e"
    );
    let digest = source_sha256(EXAMPLE_EXPECTATIONS.as_bytes());
    assert_eq!(field(receipt, "/expectations_sha256"), digest.as_str());
    assert!(
        field(receipt, "/expectations_path")
            .as_str()
            .is_some_and(|p| p.ends_with("expectations.json"))
    );
    assert_eq!(field(receipt, "/text_view"), "final");
    assert_eq!(
        field(&envelope, "/data/expectations/unassessed_requirements/0"),
        "commercial-review"
    );
    assert!(
        !out.contains("independent"),
        "the receipt never claims independence: {out}"
    );
    assert!(dir.path().join("out.docx").is_file(), "output published");
}

#[test]
fn missing_declared_literal_expectation_refuses_before_publication() {
    for plan in ["template.composed.json", "template.generated.json"] {
        let dir = tempfile::tempdir().unwrap();
        stage(dir.path());
        std::fs::write(
            dir.path().join(plan),
            if plan.contains("composed") {
                PLAN_COMPOSED
            } else {
                PLAN_GENERATED
            },
        )
        .unwrap();
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "apply",
                "example-source.docx",
                "--plan",
                plan,
                "--expect",
                "expectations.json",
                "-o",
                "out.docx",
                "--json",
            ],
        );
        assert_eq!(code, 3, "{plan}: exit {code}: {out} {err}");
        let envelope = json_envelope(&out);
        assert_eq!(field(&envelope, "/status"), "refused", "{plan}: {out}");
        assert_eq!(
            field(&envelope, "/error/code"),
            "EXPECTATION_FAILED",
            "{plan}"
        );
        // The bounded report rides in the refused envelope's error block.
        assert_eq!(
            field(&envelope, "/error/expectations/receipt/checks_run"),
            3
        );
        assert!(
            field(&envelope, "/error/expectations/checks")
                .as_array()
                .is_some_and(|checks| checks.iter().any(|c| c["outcome"] == "failed")),
            "{plan}: {out}"
        );
        assert!(
            !dir.path().join("out.docx").exists(),
            "{plan}: a failed expectation must block publication"
        );
    }
}

#[test]
fn valid_unchanged_state_satisfies_expectations() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    noop_plan(dir.path(), "satisfied.docx", "noop.json");
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "satisfied.docx",
            "--plan",
            "noop.json",
            "--expect",
            "satisfied.expect.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/data/effect"), "unchanged");
    assert_eq!(
        field(&envelope, "/data/verification/expectations"),
        "passed"
    );
    assert_eq!(field(&envelope, "/data/expectations/receipt/checks_run"), 2);
    assert_eq!(
        field(
            &envelope,
            "/data/expectations/receipt/requirements_unassessed"
        ),
        1
    );
    let published = std::fs::read(dir.path().join("out.docx")).unwrap();
    let source = std::fs::read(dir.path().join("satisfied.docx")).unwrap();
    assert_eq!(
        published, source,
        "an unchanged primary is the source bytes"
    );
}

#[test]
fn source_digest_mismatch_refuses_stale() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let stale = EXAMPLE_EXPECTATIONS.replace(
        "54ef59e60b1986dd06131bfab3d6647d2f70b095ef3c09d698b4e221855c383e",
        &"0".repeat(64),
    );
    std::fs::write(dir.path().join("stale.expect.json"), stale).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "stale.expect.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 3, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "refused");
    assert_eq!(field(&envelope, "/error/code"), "STALE_SOURCE");
    assert!(!dir.path().join("out.docx").exists());
}

#[test]
fn output_aliasing_the_expectations_file_refuses() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "expectations.json",
            "-o",
            "expectations.json",
            "--json",
        ],
    );
    assert_eq!(code, 1, "exit {code}: {out} {err}");
    assert_eq!(field(&json_envelope(&out), "/error/code"), "OUTPUT_ALIAS");
    assert_eq!(
        std::fs::read(dir.path().join("expectations.json")).unwrap(),
        EXAMPLE_EXPECTATIONS.as_bytes(),
        "the caller's expectations file is never rewritten"
    );

    #[cfg(unix)]
    {
        let dir = tempfile::tempdir().unwrap();
        stage(dir.path());
        std::os::unix::fs::symlink(
            dir.path().join("expectations.json"),
            dir.path().join("alias.json"),
        )
        .unwrap();
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "apply",
                "example-source.docx",
                "--plan",
                "template.edited.json",
                "--expect",
                "expectations.json",
                "-o",
                "alias.json",
                "--json",
            ],
        );
        assert_eq!(code, 1, "symlink alias: exit {code}: {out} {err}");
        assert_eq!(field(&json_envelope(&out), "/error/code"), "OUTPUT_ALIAS");
    }
}

#[test]
fn malformed_expectations_refuses_usage() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    for (name, body) in [
        ("not-json.expect.json", "{not json"),
        (
            "bad-schema.expect.json",
            r#"{"schema_version": 1, "source": {"sha256": "not-a-digest"}, "text_view": "final", "requirements": []}"#,
        ),
    ] {
        std::fs::write(dir.path().join(name), body).unwrap();
        let (code, out, err) = run(
            dir.path(),
            &[
                "edit",
                "apply",
                "example-source.docx",
                "--plan",
                "template.edited.json",
                "--expect",
                name,
                "-o",
                "out.docx",
                "--json",
            ],
        );
        assert_eq!(code, 64, "{name}: exit {code}: {out} {err}");
        let envelope = json_envelope(&out);
        assert_eq!(field(&envelope, "/status"), "error", "{name}: {out}");
        assert_eq!(field(&envelope, "/error/code"), "USAGE", "{name}");
        assert!(!dir.path().join("out.docx").exists(), "{name}");
    }
}

#[test]
fn dry_run_evaluates_expectations_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "expectations.json",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/data/phase"), "validated");
    assert_eq!(field(&envelope, "/data/predicted_effect"), "changed");
    assert_eq!(
        field(&envelope, "/data/verification/expectations"),
        "passed"
    );
    assert_eq!(field(&envelope, "/data/expectations/receipt/checks_run"), 3);
    let mut written: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    written.sort();
    assert_eq!(
        written,
        [
            "example-source.docx",
            "expectations.json",
            "template.edited.json"
        ],
        "dry run writes nothing"
    );
}

#[test]
fn check_requires_expect_and_names_it() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let (code, out, err) = run(dir.path(), &["check", "satisfied.docx", "--json"]);
    assert_eq!(code, 64, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "error");
    assert_eq!(field(&envelope, "/error/code"), "USAGE");
    assert!(
        field(&envelope, "/error/message")
            .as_str()
            .is_some_and(|m| m.contains("--expect")),
        "the usage error names the supported option: {out}"
    );
    let (_, help, _) = run(dir.path(), &["check", "--help"]);
    assert!(
        help.contains("--expect"),
        "check --help names --expect: {help}"
    );
}

#[test]
fn unreadable_expectations_file_is_an_io_failure_not_a_refusal() {
    // Proposal §12: exit 1 is "Input, operational or I/O failure"; an
    // unreadable --expect file is not a refusal. Both `check` and
    // `edit apply` must agree (#1084 review F1).
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "no-such.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 1, "check: exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "error", "check: {out}");
    assert_eq!(field(&envelope, "/error/code"), "IO_ERROR", "check: {out}");

    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "no-such.expect.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 1, "apply: exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "error", "apply: {out}");
    assert_eq!(field(&envelope, "/error/code"), "IO_ERROR", "apply: {out}");
    assert!(
        !dir.path().join("out.docx").exists(),
        "apply: nothing is published"
    );
}

#[test]
fn v1_plan_with_expect_refuses_with_unresolved_verification() {
    // A v1 plan plus --expect refuses UNSUPPORTED_EXPECTATION, but the
    // caller did supply expectations: the receipt must say so instead of
    // claiming they were never requested (#1084 review F4).
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let digest = source_sha256(&std::fs::read(dir.path().join("satisfied.docx")).unwrap());
    std::fs::write(
        dir.path().join("legacy.json"),
        format!(r#"{{"schema_version": 1, "author": "Recorded", "source_sha256": "{digest}", "operations": []}}"#),
    )
    .unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "satisfied.docx",
            "--plan",
            "legacy.json",
            "--expect",
            "satisfied.expect.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 3, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "refused");
    assert_eq!(field(&envelope, "/error/code"), "UNSUPPORTED_EXPECTATION");
    assert_eq!(
        field(&envelope, "/data/verification/expectations"),
        "unresolved",
        "the receipt records the refused expectation contract: {out}"
    );
    assert!(
        !dir.path().join("out.docx").exists(),
        "the refusal publishes nothing"
    );
}

/// #1244: `absolute()` canonicalizes the parent of a file that does not
/// exist yet, so Windows' 8.3 short names expand. A parent that does not
/// exist either cannot be canonicalized; the path must still come back
/// absolute (proposal §11), not as the relative spelling the caller typed.
#[test]
fn error_envelopes_name_a_missing_file_in_a_missing_directory_absolutely() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "nodir/missing.docx",
            "--expect",
            "satisfied.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 1, "check read: exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    let path = field(&envelope, "/source/path").as_str().expect("path");
    assert!(
        std::path::Path::new(path).is_absolute() && path.ends_with("nodir/missing.docx"),
        "a missing file in a missing directory is still named absolutely: {path}"
    );
}

#[test]
fn error_envelopes_name_the_file_the_caller_supplied() {
    // Proposal §11: the envelope's `source` keeps the path. An error the
    // user cannot re-associate with a file is undiagnosable (#1084 review
    // F5): read failures and recognized-mode usage failures carry the
    // user-named path even when the bytes were never read (no digest
    // then).
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    // check: the inspected FILE cannot be read.
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "missing.docx",
            "--expect",
            "satisfied.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 1, "check read: exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/error/code"), "IO_ERROR");
    let path = field(&envelope, "/source/path").as_str().expect("path");
    assert!(
        std::path::Path::new(path).is_absolute() && path.ends_with("missing.docx"),
        "check read failure names the file: {path}"
    );
    assert_eq!(
        field(&envelope, "/source/sha256"),
        "",
        "the bytes were never read; no digest is invented"
    );
    // check: usage failure (no --expect) still names the inspected file.
    let (code, out, err) = run(dir.path(), &["check", "missing.docx", "--json"]);
    assert_eq!(code, 64, "check usage: exit {code}: {out} {err}");
    assert_eq!(field(&json_envelope(&out), "/error/code"), "USAGE");
    assert!(
        field(&json_envelope(&out), "/source/path")
            .as_str()
            .is_some_and(|p| p.ends_with("missing.docx")),
        "check usage failure names the file"
    );
    // edit apply: the source read fails.
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "missing.docx",
            "--plan",
            "legacy.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 1, "apply read: exit {code}: {out} {err}");
    assert!(
        field(&json_envelope(&out), "/source/path")
            .as_str()
            .is_some_and(|p| p.ends_with("missing.docx")),
        "apply read failure names the file"
    );
    // edit replace / edit template: recognized machine mode on a malformed
    // command line still names the file operand the tokens carried.
    for (code, out, err) in [
        run(
            dir.path(),
            &[
                "edit",
                "replace",
                "missing.docx",
                "--no-such-flag",
                "--json",
            ],
        ),
        run(
            dir.path(),
            &[
                "edit",
                "template",
                "missing.docx",
                "--no-such-flag",
                "--json",
            ],
        ),
    ] {
        assert_eq!(code, 64, "usage: exit {code}: {out} {err}");
        assert!(
            field(&json_envelope(&out), "/source/path")
                .as_str()
                .is_some_and(|p| p.ends_with("missing.docx")),
            "usage failure names the file operand: {out}"
        );
    }
}

/// A passing `check` names its file the way its failures do: one absolute
/// path, whichever way the command ends (proposal §11, evidence review:
/// receipts carry actual absolute paths).
#[test]
fn check_names_its_file_by_one_absolute_path_on_success_and_failure() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "satisfied.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let passed = field(&json_envelope(&out), "/source/path").clone();
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "absent.json",
            "--json",
        ],
    );
    assert_eq!(code, 1, "exit {code}: {out} {err}");
    let failed = field(&json_envelope(&out), "/source/path").clone();
    let path = passed.as_str().expect("path");
    assert!(std::path::Path::new(path).is_absolute(), "{path}");
    assert_eq!(passed, failed);
}

#[test]
fn check_pass_line_states_scope() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "satisfied.expect.json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    assert_eq!(
        out.trim_end(),
        "Expectation checks passed: 2 run; 1 requirement not assessed.",
        "the final human line states the check's scope"
    );
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "satisfied.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/command"), "check");
    assert_eq!(field(&envelope, "/status"), "ok");
    assert_eq!(field(&envelope, "/data/phase"), "inspected");
    assert_eq!(
        field(&envelope, "/data/verification/expectations"),
        "passed"
    );
    assert_eq!(field(&envelope, "/data/expectations/receipt/checks_run"), 2);
}

#[test]
fn check_failing_expectation_exits_two() {
    let dir = tempfile::tempdir().unwrap();
    failing_fixture(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "failing.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 2, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "refused");
    assert_eq!(field(&envelope, "/error/code"), "CHECK_THRESHOLD_EXCEEDED");
    let (_, human, _) = run(
        dir.path(),
        &["check", "satisfied.docx", "--expect", "failing.expect.json"],
    );
    assert_eq!(
        human.trim_end(),
        "Expectation checks failed: 1 run, 1 failed.",
        "the failure line states the scope"
    );
}

#[test]
fn check_stale_digest_refuses() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    let stale = std::fs::read_to_string(dir.path().join("satisfied.expect.json"))
        .unwrap()
        .replace(
            &source_sha256(&std::fs::read(dir.path().join("satisfied.docx")).unwrap()),
            &"0".repeat(64),
        );
    std::fs::write(dir.path().join("stale.expect.json"), stale).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "stale.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 3, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "refused");
    assert_eq!(field(&envelope, "/error/code"), "STALE_SOURCE");
}

#[test]
fn check_malformed_expectations_exits_sixty_four() {
    let dir = tempfile::tempdir().unwrap();
    satisfying_fixture(dir.path());
    std::fs::write(
        dir.path().join("bad.expect.json"),
        "{\"schema_version\": 2}",
    )
    .unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "satisfied.docx",
            "--expect",
            "bad.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 64, "exit {code}: {out} {err}");
    assert_eq!(field(&json_envelope(&out), "/error/code"), "USAGE");
}

#[test]
fn check_with_pre_edit_ids_on_edited_output_refuses() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    // Publish the tracked edited output...
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "-o",
            "out.docx",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    // ...and check it with the SOURCE-bound expectations: the digest names
    // another snapshot, so the run refuses stale instead of grading the
    // output with pre-edit references.
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "out.docx",
            "--expect",
            "expectations.json",
            "--json",
        ],
    );
    assert_eq!(code, 3, "exit {code}: {out} {err}");
    assert_eq!(field(&json_envelope(&out), "/error/code"), "STALE_SOURCE");

    // A digest rebound to the edited output but naming a paragraph the
    // output no longer has (p1 was rewritten in place, so target p5) is a
    // missing target, never a passing absence.
    let output_digest = source_sha256(&std::fs::read(dir.path().join("out.docx")).unwrap());
    let rebound = EXAMPLE_EXPECTATIONS
        .replace(
            "54ef59e60b1986dd06131bfab3d6647d2f70b095ef3c09d698b4e221855c383e",
            &output_digest,
        )
        .replace("\"short_id\": \"p1\"", "\"short_id\": \"p5\"")
        .replace("j:short.p1\"", "j:short.p5\"");
    std::fs::write(dir.path().join("rebound.expect.json"), rebound).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "check",
            "out.docx",
            "--expect",
            "rebound.expect.json",
            "--json",
        ],
    );
    assert_eq!(code, 3, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    assert_eq!(field(&envelope, "/status"), "refused");
    assert_eq!(field(&envelope, "/error/code"), "EXPECTATION_REFUSED");
}

#[test]
fn apply_receipt_exposes_output_bound_expectations() {
    let dir = tempfile::tempdir().unwrap();
    stage(dir.path());
    let (code, out, err) = run(
        dir.path(),
        &[
            "edit",
            "apply",
            "example-source.docx",
            "--plan",
            "template.edited.json",
            "--expect",
            "expectations.json",
            "-o",
            "out.docx",
            "--json",
        ],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    let envelope = json_envelope(&out);
    let rebound = field(&envelope, "/data/expectations/output_bound").clone();
    assert!(
        rebound.is_object(),
        "the receipt carries an output-bound copy: {out}"
    );
    let output_digest = source_sha256(&std::fs::read(dir.path().join("out.docx")).unwrap());
    assert_eq!(rebound["source"]["sha256"], output_digest.as_str());
    // p1 shifted to p2 beside the inserted paragraph.
    assert_eq!(
        rebound["requirements"][0]["checks"][0]["target"]["short_id"],
        "p0"
    );
    assert_eq!(
        rebound["requirements"][1]["checks"][0]["target"]["short_id"],
        "p2"
    );
    assert_eq!(
        rebound["requirements"][1]["checks"][1]["target"]["short_id"],
        "p2"
    );
    // The copy is a schema-valid expectations document...
    std::fs::write(
        dir.path().join("rebound.json"),
        serde_json::to_string_pretty(&rebound).unwrap(),
    )
    .unwrap();
    // ...that keeps the unassessed natural-language requirement as
    // unassessed, with its empty checks list (#1084 review F6): dropping
    // it would make a downstream pass look cleaner than it was.
    let commercial = rebound["requirements"]
        .as_array()
        .expect("requirements array")
        .iter()
        .find(|r| r["key"] == "commercial-review")
        .expect("the unassessed requirement survives the rebind");
    assert_eq!(
        commercial["checks"].as_array().map(Vec::len),
        Some(0),
        "the unassessed requirement keeps its empty checks list"
    );
    // ...that standalone check accepts on the published output, still
    // reporting the requirement as not assessed.
    let (code, out, err) = run(
        dir.path(),
        &["check", "out.docx", "--expect", "rebound.json"],
    );
    assert_eq!(code, 0, "exit {code}: {out} {err}");
    assert_eq!(
        out.trim_end(),
        "Expectation checks passed: 3 run; 1 requirement not assessed.",
        "the rebound copy still reports the unassessed requirement"
    );
}
