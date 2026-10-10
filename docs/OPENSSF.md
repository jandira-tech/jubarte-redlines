<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# OpenSSF Passing evidence review

Review date: 2026-10-06. Source: the supplied project 15267 questionnaire and
[Passing criteria](https://www.bestpractices.dev/en/criteria/0).
An unknown answer is missing certification evidence, not automatically a defect.
This ledger covers every unknown item supplied. Repository changes must be
merged before their `main` URLs can be used as public badge evidence. This file
does not submit answers or claim a passing badge.

**Evidence** means a concrete implementation/document exists; **review** means
maintainer evidence or a measured run is still required; **N/A candidate** needs
its stated applicability confirmed by the badge owner. No CPE was established
in this review; a CPE is optional, and none is invented.

## Basics

| Criterion | Disposition and evidence |
|---|---|
| description_good | Evidence: README introduction describes comparison, tracked changes and rendering. |
| interact | Fixed: README links installation, issues, CONTRIBUTING and SECURITY. |
| contribution_requirements | Fixed: [CONTRIBUTING](../CONTRIBUTING.md) describes PRs, licensing, Word parity, rustfmt, strict Clippy and tests. Public URL after merge: https://github.com/jandira-tech/jubarte-redlines/blob/main/CONTRIBUTING.md |
| documentation_interface | Evidence: README CLI examples and `--help`; Rust input/output types at https://docs.rs/jubarte-redlines ; binding references in jubarte-python/README.md and jubarte-wasm/README.md. |
| english | Evidence: English documentation; contribution policy explicitly accepts English reports/comments. |
| maintained | Evidence: active interim commits and releases; review the public history at certification time. |

## Change control

| Criterion | Disposition and evidence |
|---|---|
| repo_interim | Evidence: public git history and reviewed PRs between releases, https://github.com/jandira-tech/jubarte-redlines/commits/main/ |
| version_unique | Evidence: Cargo.toml version, VERSIONING.md and release.sh registry/version checks. |
| version_semver | Evidence: VERSIONING.md explicitly specifies SemVer, including pre-1.0 rules. |
| version_tags | Evidence: release.sh creates annotated `vX.Y.Z` tags; public tags at https://github.com/jandira-tech/jubarte-redlines/tags |
| release_notes_vulns | Fixed policy: CONTRIBUTING requires public vulnerability identifiers and upgrade guidance. Unreleased Security entry records RUSTSEC-2026-0194 dependency mitigation. Review past engine CVE/GHSA assignments before claiming compliance or N/A; a dependency advisory alone is not an engine CVE. |

## Reporting

| Criterion | Disposition and evidence |
|---|---|
| report_process / osps_do_02_01 | Fixed discoverability and required URL: https://github.com/jandira-tech/jubarte-redlines/issues ; CONTRIBUTING requests version, platform and a minimal reproduction. |
| report_tracker | Evidence: GitHub Issues enabled (verified via API). |
| report_responses | Review: API returned no public issues in this repository; no response percentage can be computed. Check other reporting channels and the required 2–12 month window before answering. |
| enhancement_responses | Review: same empty tracker history; do not invent an acknowledgement rate. |
| report_archive | Evidence and required URL: https://github.com/jandira-tech/jubarte-redlines/issues?q=is%3Aissue ; includes closed reports/responses. |
| vulnerability_report_process | Fixed: https://github.com/jandira-tech/jubarte-redlines/blob/main/SECURITY.md after merge. |
| vulnerability_report_private | Evidence: GitHub private vulnerability reporting enabled; SECURITY links https://github.com/jandira-tech/jubarte-redlines/security/advisories/new and explains privacy. |
| vulnerability_report_response | N/A supported: authenticated published/triage/draft/closed advisory queries are empty, and on 2026-10-08 the maintainer confirmed no reports outside GitHub. See OPENSSF_VERIFICATION.md. |

## Quality

| Criterion | Disposition and evidence |
|---|---|
| build | Evidence: Cargo.toml/Cargo.lock, `cargo build --locked --all-features`; CI/package and release builds. |
| build_common_tools | Evidence: Rust/Cargo. |
| build_floss_tools | Evidence: Rust/Cargo/LLVM; Microsoft Word is an oracle for separate fidelity integration runs, not a build dependency. |
| test | Evidence: public Rust suite, CI and CONTRIBUTING commands. |
| test_invocation | Evidence: standard `cargo test --all-features`; coverage invocation documented. |
| test_most | Verified recommendation: full candidate suite passes 4,479 tests with 92.66% line and 77.98% branch coverage, including core API and Word-corpus regressions. Original maintainer Unmet answer is preserved; this measurement supports Met. |
| test_continuous_integration | Evidence: .github/workflows/ci.yml on main pushes/PRs. |
| test_policy | Fixed public contributor policy: tests for major new behavior and regressions. |
| tests_are_added | Evidence: recent source changes include colocated tests and tests/input_admission.rs; this change adds the OPC attribute-budget regression. Review major-change PRs, not just a policy statement. |
| tests_documented_added | Fixed in CONTRIBUTING. |
| warnings | Evidence: rustc warnings, Cargo lint policy and strict Clippy CI. |
| warnings_fixed | Evidence: `cargo clippy --all-targets --all-features -- -D warnings` verified locally; rerun for each candidate. |
| warnings_strict | Evidence: CI and release use `-D warnings`; unsafe source forbidden, targeted indexing/overflow lints on untrusted-input modules. |

## Security

| Criterion | Disposition and evidence |
|---|---|
| know_secure_design | Review: a primary developer must attest to the exact criterion; SECURITY's threat model is supporting material, not proof of personal knowledge. |
| know_common_errors | Review: developer attestation needed; SECURITY documents document/parser risks and mitigations. |
| crypto_published | Evidence: RustCrypto SHA-256, rustls TLS and zipsign Ed25519 via dependencies. SHA-1 comparison keys are non-security hashes; see SECURITY. |
| crypto_call | Evidence: sha1/sha2/self_update dependencies provide cryptographic functions. |
| crypto_floss | Evidence: cryptographic dependencies are FLOSS; Cargo license policy. |
| crypto_keylength | Verified: default rustls/ring suites and certificate keys exceed the criterion's minimum; see OPENSSF_VERIFICATION.md. Signing support is not a claim that released artifacts are signed. |
| crypto_working | Evidence: no broken algorithm used as a security mechanism; SHA-1 comparison risk explicitly documented. Review release trust configuration. |
| crypto_weaknesses | Evidence: SHA-256 snapshot/checksum guard, rustls transport, Ed25519 signature support. SHA-1 content equality is not security identity. |
| crypto_pfs | Verified: default TLS uses ephemeral key agreement; see provider and resumption scope in OPENSSF_VERIFICATION.md. |
| crypto_password_storage | N/A candidate: no external-user password authentication/storage. GitHub/registry tokens are not passwords stored for authenticating engine users. |
| crypto_random | Verified: delegated TLS randomness uses ring SystemRandom backed by the operating system. Met is appropriate for the default CLI. |
| delivery_unsigned | Evidence: update/install use HTTPS; SECURITY forbids unsigned HTTP checksum delivery. Signing rollout is documented in docs/SELF_UPDATE.md; HTTPS alone meets this criterion. |
| vulnerabilities_fixed_60_days | Source fix: rdocx-opc 0.2 requires patched quick-xml 0.41 in every binding, including direct public parser calls; XML advisory exceptions removed. Publish the corrected release before attesting that released results are fixed. |
| vulnerabilities_critical_fixed | Fixed policy: immediately triage critical findings and prioritize fixes; past response/fix history needs maintainer review. |
| no_leaked_credentials | Evidence: Gitleaks reachable-history scan found no leaks (1,832 commits); GitHub secret scanning enabled. New daily/PR/release scans cover history and local changes. A scanner cannot prove a credential is valid or detect every secret. |

## Analysis

| Criterion | Disposition and evidence |
|---|---|
| static_analysis | Evidence: Clippy is a static source analyzer beyond rustc warnings, applied in CI and release.sh. Dependency and secret checks now also enforced before release. |
| static_analysis_common_vulnerabilities | Evidence: Clippy correctness/suspicious and targeted indexing/arithmetic lints; cargo-deny RustSec vulnerability checks. Dependency analysis supplements source analysis. |
| static_analysis_fixed | Fixed triage policy: confirmed medium/higher exploitable findings block release. Exceptions documented in deny.toml/SECURITY, not represented as fixed upstream. |
| static_analysis_often | Evidence: Clippy every push/PR; security checks daily and every main push/PR. |
| dynamic_analysis | Evidence: four libFuzzer targets, executed candidate runs, and pre-major-release checklist requirement; see OPENSSF_VERIFICATION.md. |
| dynamic_analysis_unsafe | Review: own Rust library forbids unsafe code, but CLI ships the C mimalloc allocator. Do not claim blanket N/A; validate sanitizer coverage of shipped native dependencies or document the safe-only build under assessment. |
| dynamic_analysis_enable_assertions | Evidence: debug Rust tests; cargo-fuzz targets use sanitizers/debug assertions. |
| dynamic_analysis_fixed | Fixed triage policy: retain reproducers and fix confirmed medium/higher exploitable findings before release. Review crash artifacts for each candidate. |

## Remaining owner evidence

Developer knowledge is attested in the dated answer file. On 2026-10-08 the
maintainer confirmed no reports/findings outside GitHub; authenticated advisory
queries were also empty. Release the corrected parser and use the current
verification record for sanitizer scope and coverage. The badge owner must
enter supported URLs/justifications in project 15267. Live badge answers and
repository settings were not edited.

## Local validation of this change

- Strict Clippy (`--all-targets --all-features -- -D warnings`) and rustfmt pass.
- 1,021 library tests and 16 admission integration tests pass (one unrelated
  library test remains ignored). Nightly LLVM branch coverage includes both
  suites. Coverage: 87.13% lines, 57.61% branches for the OPC module. This is
  not a full-repository coverage claim; the selected suites cover 57.00% lines
  and 36.92% branches overall. Broad branch coverage remains review work.
- 101 release-script checks pass after regenerating all four published READMEs.
  Python test-harness coverage: 99.54% lines, 92.50% branches; this measures
  the Python harness, not Bash production coverage.
- CLI `--help` smoke passes. All four dependency advisory scans pass with the
  documented exceptions. Reachable-history and publishable-file secret scans
  find no leaks. REUSE lint, workflow YAML parsing, Bash syntax and whitespace
  checks pass.
- Hosted CI and release-candidate fuzzing are not claimed as executed locally.

## Interactive maintainer attestations

From the repository root, run:

```sh
python3 scripts/openssf_attest.py
```

The script asks your name, then walks through 17 remaining attestations one at
a time. Enter accepts the displayed status and justification; you can choose
`met`, `unmet`, `?`, or `n/a` where permitted. `skip` defers an item and `quit`
(or Ctrl-C) stops. Knowledge criteria suggest Met only for the displayed
personal confirmations; historical and unmeasured claims default to Unknown.
The password-storage criterion suggests N/A only if its displayed scope is true.
No answer is accepted without your input, and positive/N/A answers require a
justification. Keep private reports and credentials out of saved justifications.

Each completed answer is saved to `out/openssf-attestations.json` (ignored by
Git). Run again to resume; `--review` revisits saved answers, and `--show` prints
answers for copying into the badge form. `--list` previews all questions without
saving. You may use `--output PATH` for another answer file. An assessment has a
fixed date: on a later day, pass its original `--date YYYY-MM-DD` to resume or
use a fresh output file for a new assessment. Nothing is uploaded or submitted.

Verification: six deterministic interaction/local-persistence integration tests
pass. Coverage: 99.14% lines, 98.00% branches of `scripts/openssf_attest.py`.
CLI `--help` and question-preview smoke checks pass.

## Maintainer answers recorded on 2026-10-07

Arthur Souza Rodrigues completed the interactive review. The
[dated answer file](openssf-attestations-2026-10-07.json) preserves all 17
answers and justifications exactly as entered, including Unknown and Unmet
answers. These are maintainer attestations, not additional automated
verification. They have not been submitted to the badge site. The earlier
criterion review remains the technical evidence and limitations for evaluating
these answers.

## Follow-up verification on 2026-10-07

[OPENSSF_VERIFICATION.md](OPENSSF_VERIFICATION.md) records source and dependency
inspection, authenticated advisory queries and an executed hosted fuzz run.
Its recommendations supersede the earlier provisional crypto/dynamic-analysis
labels above. It identifies an unresolved affected public XML API and separates
that blocker from guarded document-opening paths. Original maintainer answers
remain unchanged; private-report history and full native allocator analysis
still require evidence.
