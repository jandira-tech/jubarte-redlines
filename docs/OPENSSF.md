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
| vulnerability_report_response | Review: SECURITY sets ≤14 days acknowledgement; private report dates in the past six months require maintainer review. No reports may justify N/A, but public API access cannot prove no private reports. |

## Quality

| Criterion | Disposition and evidence |
|---|---|
| build | Evidence: Cargo.toml/Cargo.lock, `cargo build --locked --all-features`; CI/package and release builds. |
| build_common_tools | Evidence: Rust/Cargo. |
| build_floss_tools | Evidence: Rust/Cargo/LLVM; Microsoft Word is an oracle for separate fidelity integration runs, not a build dependency. |
| test | Evidence: public Rust suite, CI and CONTRIBUTING commands. |
| test_invocation | Evidence: standard `cargo test --all-features`; coverage invocation documented. |
| test_most | Review: CI enforces ≥80% lines. Line coverage does not prove most branches/input fields; inspect measured branch results and coverage gaps. |
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
| crypto_keylength | Review/N/A candidate: no engine encryption/key agreement; TLS handled by rustls, optional release signatures Ed25519. Confirm deployed release key and transport policy; do not blanket-mark all crypto N/A because self-update uses TLS. |
| crypto_working | Evidence: no broken algorithm used as a security mechanism; SHA-1 comparison risk explicitly documented. Review release trust configuration. |
| crypto_weaknesses | Evidence: SHA-256 snapshot/checksum guard, rustls transport, Ed25519 signature support. SHA-1 content equality is not security identity. |
| crypto_pfs | Review: rustls transport configuration belongs to self_update/ureq; engine itself implements no key agreement. |
| crypto_password_storage | N/A candidate: no external-user password authentication/storage. GitHub/registry tokens are not passwords stored for authenticating engine users. |
| crypto_random | N/A candidate for engine key/nonce generation: no such engine mechanism; TLS randomness delegated to rustls. Release private keys are generated by zipsign outside the repository. |
| delivery_unsigned | Evidence: update/install use HTTPS; SECURITY forbids unsigned HTTP checksum delivery. Signing rollout is documented in docs/SELF_UPDATE.md; HTTPS alone meets this criterion. |
| vulnerabilities_fixed_60_days | Review: XML attribute DoS mitigated before engine parsing; NsReader advisory is unreachable in rdocx-opc. Scan all binding graphs and review exceptions/private reports before attesting no remaining medium/higher engine vulnerabilities. Re-exported upstream XML parser remains affected when called directly. |
| vulnerabilities_critical_fixed | Fixed policy: immediately triage critical findings and prioritize fixes; past response/fix history needs maintainer review. |
| no_leaked_credentials | Evidence: Gitleaks reachable-history scan found no leaks (1,832 commits); GitHub secret scanning enabled. New daily/PR/release scans cover history and local changes. A scanner cannot prove a credential is valid or detect every secret. |

## Analysis

| Criterion | Disposition and evidence |
|---|---|
| static_analysis | Evidence: Clippy is a static source analyzer beyond rustc warnings, applied in CI and release.sh. Dependency and secret checks now also enforced before release. |
| static_analysis_common_vulnerabilities | Evidence: Clippy correctness/suspicious and targeted indexing/arithmetic lints; cargo-deny RustSec vulnerability checks. Dependency analysis supplements source analysis. |
| static_analysis_fixed | Fixed triage policy: confirmed medium/higher exploitable findings block release. Exceptions documented in deny.toml/SECURITY, not represented as fixed upstream. |
| static_analysis_often | Evidence: Clippy every push/PR; security checks daily and every main push/PR. |
| dynamic_analysis | Evidence: three libFuzzer targets and pre-major-release contributor/checklist requirement. Review candidate fuzz run/crash evidence before certification. |
| dynamic_analysis_unsafe | Review: own Rust library forbids unsafe code, but CLI ships the C mimalloc allocator. Do not claim blanket N/A; validate sanitizer coverage of shipped native dependencies or document the safe-only build under assessment. |
| dynamic_analysis_enable_assertions | Evidence: debug Rust tests; cargo-fuzz targets use sanitizers/debug assertions. |
| dynamic_analysis_fixed | Fixed triage policy: retain reproducers and fix confirmed medium/higher exploitable findings before release. Review crash artifacts for each candidate. |

## Remaining owner evidence

Supply developer secure-design/common-error attestations, review private-report
acknowledgements and publicly known engine vulnerabilities, measure relevant
branch coverage, and confirm sanitizer coverage of the default CLI's native
allocator. The badge owner should then enter supported URLs/justifications in
project 15267. Live badge answers and repository settings were not edited.

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
