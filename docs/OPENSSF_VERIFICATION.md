<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# OpenSSF follow-up verification — 2026-10-08

## Current source fix and owner confirmation

The follow-up changes on `c/address-openssf-findings` upgrade rdocx-opc from
0.1 to 0.2 in every engine/binding graph. Version 0.2 requires patched
quick-xml 0.41. Its relationship and content-type parser source is identical
to 0.1.2, retaining the existing entity-decoding and explicit-close adapters
and identical-duplicate relationship repair. Later upstream releases change
those semantics, so this fix uses the smaller compatible security upgrade.
The WASM-only vendored patch and both XML advisory exceptions are removed.

The dependency advisory gate failed with RUSTSEC-2026-0194/0195 before the
upgrade. Direct public-parser regressions now exercise 16,384 distinct
attributes, late duplicates, required-attribute duplicates, round trips,
lookup and relationship ID generation. No elapsed-time assertion or weakened
duplicate checks are used.

On 2026-10-08 Arthur Souza Rodrigues answered: "None received outside GitHub"
when asked about vulnerability reports and confirmed medium/high static or
fuzzing findings outside GitHub. The repository's authenticated published,
triage, draft and closed advisory queries were repeated and all returned
empty lists; public Issues excluding PRs were also empty. This resolves the
external-report uncertainty from the October 7 assessment.

| Criterion | Current recommendation | Basis |
| --- | --- | --- |
| vulnerability_report_response | N/A | No reports in the reviewed GitHub history or outside GitHub, as confirmed by the maintainer. |
| release_notes_vulns | N/A for reviewed release history | No assigned vulnerabilities in project results were found. Dependency fixes are recorded with their RustSec identifiers in CHANGELOG.md. |
| vulnerabilities_fixed_60_days | Source fix complete; release pending | The affected parser is removed from all graphs, including the public API. Merge and publish a corrected release before claiming that released results are fixed. |
| static_analysis_fixed | Met for the reviewed findings | The confirmed XML advisory findings are fixed without advisory exemptions; original maintainer timeliness attestation remains recorded. |
| dynamic_analysis_fixed | N/A if candidate analysis remains clean | No confirmed exploitable dynamic findings in the reviewed report history; maintainer confirms none outside GitHub. Execute/review current candidate runs before submission. |
| crypto_keylength, crypto_pfs, crypto_random | Met | The verified default TLS configuration below is unaffected by this OPC upgrade. |
| crypto_password_storage | N/A | No authentication-password storage for external users. |
| test_most | Met recommendation based on measured coverage | Full candidate suite: 4,479 passed, five existing ignored; 92.66% lines and 77.98% branches. Core API, accept/reject Word-corpus and rendering regressions passed. The original Unmet answer is preserved for the owner to review. |

Native dynamic-analysis execution and final verification results are recorded
below once the candidate checks finish. Original maintainer answers remain
unchanged; these are dated recommendations for the badge owner.

## Historical snapshot — 2026-10-07, before the fix

Assessment of commit `fb0d2c2eb7ff9cbb6978d20a0acc3706969bc2df` and its
resolved dependency graphs. These recommendations supplement the
[original maintainer answers](openssf-attestations-2026-10-07.json), which
remain unchanged. Nothing has been submitted to the badge site.

### Historical recommendations

| Criterion | Recommendation | Verified evidence or remaining limitation |
| --- | --- | --- |
| crypto_keylength | Met | The default CLI updater uses rustls/ring TLS 1.2/1.3. Its supported suites use AES-128/256 or ChaCha20, SHA-256/384, and ephemeral X25519/P-256/P-384 key agreement. RSA certificate verification accepts keys of at least 2048 bits. These exceed the criterion's 112-bit minimum security strength; shorter alternatives are absent from this provider. |
| crypto_pfs | Met | The configured provider offers ephemeral key agreement, with no static RSA key transport. TLS 1.3 permits PSK_DHE and excludes PSK-only key exchange. This does not assert fresh key agreement on every TLS 1.2 resumed connection. |
| crypto_random | Met | TLS key generation and randomness use ring's SystemRandom, backed by the operating system through getrandom. Delegated cryptographic randomness makes Met more appropriate than the original N/A answer. |
| crypto_password_storage | N/A | The engine and CLI do not authenticate external users or store their authentication passwords. This supports N/A instead of the original Met answer. |
| release_notes_vulns | N/A, within reviewed history | No assigned vulnerabilities in the project results were found in the repository advisories, global package-advisory queries, or changelog. Changelog advisory references concern dependencies. Recheck if an assignment exists outside these sources. |
| vulnerability_report_response | Unknown pending maintainer confirmation | GitHub advisory queries found no reports, including triage, draft and closed states. Reports outside GitHub are not observable from this review. If there were none in the last six months, N/A is appropriate; otherwise verify every initial response was within 14 days. |
| vulnerabilities_fixed_60_days | Unmet for the full public library API | The public Relationships::from_xml re-export bypasses the engine attribute guard and calls the vulnerable upstream parser. See the blocker below. |
| dynamic_analysis | Met for this candidate's engine source | All three hosted libFuzzer targets succeeded on an identical engine source tree. This is not retrospective evidence for earlier releases. |
| dynamic_analysis_unsafe | Unknown for the complete default CLI | Fuzzing disables engine default features, excluding the CLI's C mimalloc allocator. The run does not establish routine sanitizer coverage of that shipped component. |
| dynamic_analysis_enable_assertions | Met for the reviewed fuzz configuration | CI uses cargo-fuzz's default debug-assertion configuration and address sanitizer; no optimization flag disabling assertions or sanitizer override was passed. |
| dynamic_analysis_fixed | Unknown for historical findings | No exploitable sanitizer findings were identified in the inspected run. This alone cannot establish timely remediation of all previous or private dynamic-analysis findings. |
| test_most | Unmet, as attested | No new whole-project coverage claim is established. This SUGGESTED criterion may remain Unmet without blocking Passing. |

Developer knowledge, enhancement response history and earlier remediation
timeliness remain maintainer attestations. Published policies establish intended
behavior, not proof of historical compliance. Empty GitHub Issues results
support the maintainer's statement that there were no public bug reports, but
cannot reveal reports received elsewhere.

## Default updater cryptography

The resolved chain is self_update 1.3.0 → ureq 3.4.2 → rustls 0.23.45 →
ring 0.17.14. `cargo tree -e features -i rustls` confirms the default CLI
selects the rustls/ring transport. Source inspection covered:

- self_update's `src/http_client/ureq.rs`: rustls provider selection.
- ureq's `src/tls/rustls.rs`: ring default provider and supported protocol
  versions; ordinary certificate verification is retained.
- rustls's `src/crypto/ring/mod.rs` and `kx.rs`: suites, verification
  algorithms, groups and ephemeral-key generation.
- rustls's `src/client/hs.rs`: TLS 1.3 PSK_DHE selection.
- ring's `src/rand.rs`: operating-system randomness.

The project does not replace this provider, disable certificate verification,
or enable weak suites. The strength comparison uses the criterion's
[NIST reference](https://www.nist.gov/publications/recommendation-key-management-part-1-general-revision-3).

`src/update.rs` currently has an empty RELEASE_KEYS list and no first signed
release. Ed25519 verification support therefore does **not** establish that
published releases are currently signed. The active updater trust mechanism
uses HTTPS; checksum-only delivery retains the risks documented in
[SELF_UPDATE.md](SELF_UPDATE.md).

## Unresolved public API vulnerability

[RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194.html)
was publicly issued on July 2, 2026, has CVSS 7.5, and is fixed in quick-xml
0.41.0 or later. The assessment date is more than 60 days after disclosure.

Native, Python and in-process binding graphs retain rdocx-opc 0.1.2 with
quick-xml 0.37.5. `src/opc/mod.rs` publicly re-exports Relationships;
upstream `src/relationship.rs::Relationships::from_xml` iterates
`.attributes()` with the vulnerable default duplicate checks. A caller can
pass untrusted XML directly through this public API without entering
check_opc_attributes. This is a source-level reachability finding, not a
claim that a particular deployed service accepts that input.

The engine's document-opening and set-part guards remain useful mitigations,
but cannot justify marking this criterion Met for the entire exposed API.
A complete fix must use a patched parser across those dependency graphs or
make every exposed affected parse path enforce an effective bound.

Fresh cargo-deny scans were also run with advisory exceptions removed:

| Graph | Findings |
| --- | --- |
| Root, Python, in-process | RUSTSEC-2026-0194, RUSTSEC-2026-0195; unmaintained rustybuzz/ttf-parser notices |
| WASM | Unmaintained rustybuzz/ttf-parser notices; no quick-xml vulnerability findings |

RUSTSEC-2026-0195 concerns NsReader, which the inspected rdocx-opc code does
not use. The unmaintained notices alone do not establish exploitable
vulnerabilities. A normal policy scan passes only with the documented
exceptions, so its success is not evidence that the public API issue is fixed.

## GitHub report and assignment review

Authenticated queries had repository admin permissions. The correct endpoint
is `repos/jandira-tech/jubarte-redlines/security-advisories`, rather than
`security/advisories`. The default list and explicit triage, draft and closed
queries returned no advisories. Public Issues excluding pull requests were
also empty. Global advisory queries returned no results for Rust packages
jubarte-redlines/jubarte, npm jubarte-wasm, or pip jubarte-redlines.

These observations support the scoped release-note recommendation above.
They do not prove the absence of external reports, undisclosed findings, or
assignments under another package identity.

## Executed dynamic analysis

[CI run 37627577114](https://github.com/jandira-tech/jubarte-redlines/actions/runs/37627577114),
job 112813220545, used commit
`8dbfb80d018e67ef0d9dd9f6663201de60c8e79f`. Its engine source, Cargo inputs
and fuzz targets are identical to the assessed commit. Each target step
individually succeeded; the job's continue-on-error setting was not used as
proof of success.

| Target | Completed executions | Reported duration |
| --- | ---: | ---: |
| admit | 63,918 | 61 seconds |
| strict_to_transitional | 62,643 | 61 seconds |
| compare | 62 | 172 seconds |

The logs contained no sanitizer error report or crash summary. These short
seeded runs do not prove absence of defects or broad branch coverage.
`fuzz/Cargo.toml` uses default-features = false, so evidence excludes the
default CLI allocator. Confirm allocator instrumentation and execute an
appropriate native-path analysis before claiming that scope is covered.

## Badge submission

Use the supported recommendations and evidence URLs when completing project
15267. First resolve the public API blocker and the remaining historical
questions. Under the [Passing rules](https://www.bestpractices.dev/en/criteria_discussion),
MUST criteria need Met or an allowed, justified N/A; SHOULD may be Unmet
with justification, and SUGGESTED must be considered. Repository evidence
and saved answers do not automatically update the badge form.
