<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Security policy

Security fixes target the latest engine release. Upgrade to the latest release;
older releases are not promised backports. This policy covers the Rust engine,
CLI and the Python/WASM bindings in this repository.

## Report a vulnerability privately

Use GitHub's [private vulnerability reporting form](https://github.com/jandira-tech/jubarte-redlines/security/advisories/new).
This repository has private vulnerability reporting enabled. Reports and
attachments sent through that form are private to the reporting participants
and maintainers until coordinated publication. Do not open a public issue for
an undisclosed vulnerability or upload confidential documents to public issues.

Include affected versions, platform, reproduction steps, impact and a minimal
non-confidential input. The initial acknowledgement target is **14 days or
less**. If no acknowledgement arrives, follow up in the same private report.
Maintainers triage confirmed critical findings immediately and prioritize a
fix; medium or higher exploitable vulnerabilities must not remain unpatched
for more than 60 days after public disclosure. These are policy commitments,
not claims about historical response times.

Coordinate a patch and disclosure with the reporter. Add regression tests,
rerun static and dynamic checks, and record affected/fixed versions and any
assigned CVE/GHSA identifiers in release notes and a public security advisory.
Never include the reporter's private documents or credentials in the advisory.
Rotate/revoke exposed credentials at the provider; removing a file alone does
not invalidate a credential in git history.

## Threat model and development

DOCX, legacy DOC, XML, embedded fonts and images are untrusted inputs. Risks
include ZIP decompression bombs, excessive nesting, malformed parser inputs,
algorithmic complexity, memory exhaustion, path traversal and accidental
network/file access through external relationships. Admission bounds archive
size, entries, inflated bytes and XML depth (`src/admission.rs`); the library
forbids unsafe Rust; external relationships are not fetched during document
processing. Run untrusted workloads in a process with memory/CPU limits: input
budgets do not prove protection against every denial of service. Do not
increase budgets silently to admit a malicious document.

Use existing cryptographic libraries, never custom security algorithms.
SHA-1/FNV content keys preserve the comparison algorithm's upstream behavior;
they are **not authentication, signatures or collision-resistant security
identities**. SHA-256 guards edit snapshots and release checksums. The CLI's
self-update uses the `self_update` crate with rustls HTTPS and supports zipsign
Ed25519 verification. See [SELF_UPDATE.md](docs/SELF_UPDATE.md) for whether a
release requires signatures and the risks of checksum-only releases. Download
archives and their checksums only over HTTPS; a checksum fetched over HTTP
without signature verification is not acceptable. The engine does not provide
user authentication, password storage, encryption or key agreement.

### Temporary dependency exceptions

`rdocx-opc` 0.2 requires patched quick-xml 0.41, fixing RUSTSEC-2026-0194
and RUSTSEC-2026-0195 in every binding and the public relationship parser.
There are no exceptions for these vulnerabilities in the advisory policy.
OPC package admission retains its 256-attribute budget as an additional
resource limit; a direct `Relationships::from_xml` call does not enforce
that package-specific limit. Callers still need process/input resource limits
for untrusted XML. The unmaintained
rustybuzz/ttf-parser exceptions track a separate migration, not a known
exploitable vulnerability; revisit them on each release.

## Security checks

Install `cargo-deny` 0.20.2 and Gitleaks 8.30.1, then run:

```sh
bash scripts/security-check.sh
cargo clippy --locked --all-targets --all-features -- -D warnings
```

The script audits every engine/binding and fuzz dependency graph and scans
reachable git history and current tracked/unignored files for secrets with redacted output.
The scheduled security workflow runs daily and on pushes/pull requests. Clippy
is the static source analyzer (beyond rustc's warnings); it also runs in CI and
before release. Fuzzing with sanitizers is dynamic analysis; ordinary tests
run with debug assertions. Run all fuzz targets before a major release:

```sh
./fuzz/seed.sh
cargo +nightly fuzz run admit -- -max_total_time=60
cargo +nightly fuzz run strict_to_transitional -- -max_total_time=60
cargo +nightly fuzz run compare -- -max_total_time=60
cargo +nightly fuzz run relationships -- -max_total_time=60
bash scripts/fuzz-native.sh compare -max_total_time=60
bash scripts/fuzz-native.sh relationships -max_total_time=60
```

A crash is a finding to triage, even when malformed input triggers it. Fix
confirmed exploitable medium/higher static or dynamic findings before release;
retain minimized regression inputs that are safe to publish. Any advisory
exception needs a reason, maintainer review and a review date. The release
script enforces dependency and secret checks even when ordinary gates are
skipped on a retry. Release reviewers must also inspect private/public reports
and the fuzz results: an automated clean scan cannot certify their absence.
