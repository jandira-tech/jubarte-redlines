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
size, entries, inflated bytes and XML depth (`src/admission.rs`). Package
admission and the checked XML APIs enforce tag policy with the active
`InputLimits` before DOM construction (#650). `InputLimits` defaults
are `max_element_attributes = 1024` (namespace declarations included),
`max_attribute_name_bytes = 1024` (qualified UTF-8 name),
`max_attribute_value_bytes = 67108864` (64 MiB, raw and decoded), and
`max_start_tag_bytes = 134217728` (128 MiB, including spaces, quotes and
`<`, `>`, `/` delimiters). Each bound is inclusive and configurable,
including through `InputLimitOverrides`; comparison budgets keep these defaults.
Admission returns the existing `InvalidXml` / `INVALID_XML` refusal for
tag-policy violations, with the part, element and limit. Checked library
parsing returns the same policy message; its raw XML API has no part name.
`validate_xml` and `try_parse_xdocument` use the defaults; their
`*_with_limits` variants use the supplied limits, as does package admission.
The infallible `parse_xdocument` does not enforce input policy: it preserves
attribute order and keeps the last value at the first position for repeated
expanded names, with linear bulk insertion. Use the checked APIs for untrusted
XML. The Markdown converter checks the same policy before constructing tree
attributes, using `MarkdownOptions::input_limits` (defaults when omitted).
Pass the admission limits through this field when rendering a package admitted
under raised limits. Policy violations refuse the whole conversion with
`INVALID_XML` and the part name, including auxiliary parts otherwise silently
skipped by best-effort rendering. Ordinary malformed auxiliary XML retains its
existing tolerance; this is not full XML admission. CLI `read` and `validate`
also enforce admission.
Aliased attributes sharing a namespace URI and local name are malformed XML;
unprefixed attributes remain in no namespace. The independent OPC metadata
attribute cap stays at 256. These are XML resource/namespace checks, not claims
about which files Word will open. The library
forbids unsafe Rust; external relationships are not fetched during document
processing. Run untrusted workloads in a process with memory/CPU limits: input
budgets do not prove protection against every denial of service. Do not
increase budgets silently to admit a malicious document.

Use existing cryptographic libraries, never custom security algorithms.
SHA-1/FNV content keys preserve the comparison algorithm's upstream behavior;
they are **not authentication, signatures or collision-resistant security
identities**. SHA-256 guards edit snapshots and release checksums. The CLI's
self-update uses the `self_update` crate with rustls HTTPS and supports zipsign
Ed25519 verification (jubarte's own check, `src/update/zipsign.rs`, on
ed25519-dalek 3). See [SELF_UPDATE.md](docs/SELF_UPDATE.md) for whether a
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

### Recorded dependency advisories

Advisories this repository has answered, kept here so a gap in the automated
watch cannot hide one. `scripts/test_oracle_dependency.py` fails while an
advisory named there is missing from this section.

**GHSA-m665-v7pm-7grj** — zip-rs/zip2 repository advisory (critical,
published 2026-10-17): "Parser Differential & Security Control Bypass via
Dual-Header Inconsistencies (Zip Cloaking) Leading to Arbitrary File
Overwrite & RCE". Affected `zip <= 9.0.4`; patched in 9.0.5. The manifest
floor and every committed lockfile hold 9.0.5.

Call paths here: the engine reads packages in memory through `ZipArchive`
and writes them with `ZipWriter`. Every production package open goes
through `crate::opc::open_archive` (`src/opc/archive.rs`), which reads
each entry's raw name from its central record and from its local file
header: a local header that names the entry differently, or is missing,
fails the open and every caller reports `ZIP_HEADER_MISMATCH`. zip
9.0.5's own `ZipArchive::validate_header_consistency` is not used: it
also refuses a local time or a central-only Unicode Path field, which hide
no content and which real Word-opened inputs carry. The same pass (#644)
compares the remaining content fields — compression method, flag bits 0
and 3 (bit 11, the UTF-8-name flag, only when a raw name byte is 0x80 or
above — an ASCII name decodes the same either way), and with bit 3 clear
the CRC-32 and ZIP64-resolved sizes — as `ZIP_HEADER_MISMATCH`, refuses entries whose extents overlap or reach
into the central directory and data descriptors that disagree with the
central record as `INVALID_PACKAGE`, and validates directory payloads
through the bounded CRC stream. Admission keeps the typed
`AdmissionErrorKind::ZipHeaderMismatch` refusal for the same check, and
existing duplicate-part identity checks retain `DUPLICATE_PART`. The
`self_update` updater path is covered below.

`jubarte debug`, the maintainer triage tool, is the one deliberate
exception: it opens through `open_archive_unchecked` and prints the
disagreement as a `zip-header-mismatch` finding — and each #644 defect
as a `zip-extent`, `zip-descriptor`, `zip-directory-payload` or
`zip-entry-read` finding —
instead of refusing, so a file every reader rejects can still be inspected.

The Windows updater delegates unpacking to `self_update::Extract::extract_file`;
its ZIP arm opens a `ZipArchive`, selects the binary with `by_name`, checks
`enclosed_name`, and copies that entry. It uses neither archive-wide
`ZipArchive::extract` nor the streaming reader. Our code in `src/update.rs`
configures the updater and its signature policy; it does not unpack ZIPs.

The version bump alone does **not** reject conflicting headers:
`ZipArchive::new`/`by_name` still trust the central directory by default.
9.0.5 adds opt-in consistency checks to both random-access and streaming
readers; the delegated updater's `extract_file` does not opt in. Its
single-entry central-directory lookup does not cross into a streaming
extractor. See the [repository advisory](https://github.com/zip-rs/zip2/security/advisories/GHSA-m665-v7pm-7grj).

**ZIP-16 — symlink and special file metadata** (#639): the same
`zip_defects` pass reads the raw central record's `version_made_by` and
external attributes. When the host byte is Unix (3) or OS X (19), the
mode `external attributes >> 16` must record a regular file — or a
directory on a `/`-terminated name — and on any host the DOS directory
attribute `0x10` demands the same. A symlink, device, FIFO, socket, any
other non-zero type, or a directory type on a file name is refused as
`UNSUPPORTED_PACKAGE` (the encryption/unsupported-compression class), and
`jubarte debug` reports it as a `zip-special-entry` finding. Permission,
setuid, setgid and sticky bits never matter, and the check reads the raw
fields itself because zip's `is_symlink` is a bit-mask test that reads a
mode only for the Unix host. For the updater, `self_update`'s single-file
zip path already refuses a symlink entry (`extract_file`), but its tar
path delegates to `tar`'s `unpack_in`, which restores a symlink whose
target is not confined to the staging directory; `src/update.rs`
registers a `verify_binary` hook that lstats the extracted binary and
refuses anything that is not a regular file before the swap.

Why the watch stayed green: this is a repository advisory on zip-rs/zip2,
not an entry in the RustSec database `cargo-deny` reads
(`scripts/security-check.sh`), so the daily advisory watch reported
nothing until the version was checked by hand.

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
