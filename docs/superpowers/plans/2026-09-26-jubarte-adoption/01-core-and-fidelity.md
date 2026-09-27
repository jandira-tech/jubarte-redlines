# Safe Core, Inspection and Fidelity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose predictable document inspection without sacrificing existing Word behavior.

**Architecture:** Add checked XML admission around new reads and preserve the current DOM/comparer. Keep inspection facts distinct from layout facts and semantic edits.

**Tech Stack:** Rust, quick-xml 0.41, existing PartFs/Dom, cargo-llvm-cov

---

> **Status 2026-09-26 (branch `feat/agent-adoption`):** implemented with
> corrections; see [00-ASSESSMENT.md](00-ASSESSMENT.md) §3. Landed:
> parser progress guard and `validate_xml` (`src/xmllinq/parse.rs`), and
> `src/inspect.rs` with a richer read model than patch 0001 proposed:
> `Paragraph { index, id, text, style, numbered, in_table, page_break, runs,
> limitations }`, `summary`, `markdown`, `inspect_json`, `source_sha256`.
> Correction applied: `mc:AlternateContent`, `w:sym`, column breaks, fields,
> hyperlinks and content controls are per-paragraph **limitations**, not
> refusals (patch 0001 refused the document). Task C2 (ZIP admission limits)
> remains open and is still required before untrusted-upload use.
> Verified: `tests/inspect_paragraphs.rs` (13), module unit tests, clippy
> `-D warnings`.

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## Current-state correction

Initial findings were on `23f6e30`; current reviewed HEAD advanced to `722de2a`. Commit `cb33d11` already replaced missing-main/root/body and unsupported-content panics in `get_revisions`, and fixed an orphan-note comparison panic. Do not apply an old replacement of `document_comparer.rs`. Patch 0001 was generated from the newer source and adds checked reads rather than repeating those fixes. Re-run `git apply --check` because other work is active.

## Read model and limitations

`inspect::paragraphs(bytes)` returns `Vec<ParagraphInfo { index, text, page_break }>`. Index is zero-based and valid only for this snapshot. Text preserves tabs, line breaks, NBSP, soft/nonbreaking hyphens and Unicode; source-anchor text is never lowercased or whitespace-normalized. Paragraphs are in body XML order including table cells, not visual reading order. Textbox stories are omitted from this body API; the owner paragraph must not duplicate their text.

Inspection excludes `w:del` and `w:moveFrom` run content. It is a visible-run projection, **not** a promise of complete Word “final view”: deleted paragraph marks, field recalculation, hidden text and layout-dependent order need a richer story model. Field instructions are omitted; cached field-result text is not recomputed. `AlternateContent`, `altChunk`, subdocuments, font-dependent symbols and unsupported breaks produce an explicit error in this first read model.

`summary(bytes)` counts namespace-qualified XML elements and actual relationship targets, not string prefixes or guessed filenames. `revisions` is a raw carrier count, not the number of human-visible revision groups; inspection errors cannot become a zero count. Header/footer/comment/note discovery follows OPC relationships, including unusual valid target names. Page count is absent until the renderer computes it.

Before public release, add a story-aware v2 inspection DTO with `schema_version`, `source_sha256`, `stories`, opaque paragraph IDs, style/list context, raw text and capabilities. A bounded result window with `after` cursor prevents large documents from filling an agent context. Start with explicit `body` scope; do not expose an all-stories promise before headers, footers, notes and textboxes are individually covered.

## Task C1: checked XML and conservative inspection

**Files:** exact full contents/diffs in `patches/0001-checked-inspection.patch`: `Cargo.toml`, `src/lib.rs`, `src/xmllinq/parse.rs`, `src/document_comparer.rs`, `src/inspect.rs`. Unit tests live inside the new behavior modules.

- [ ] Review the patch against current HEAD; require no removal of current comparer fixes.

```sh
git apply --check docs/superpowers/plans/2026-09-26-jubarte-adoption/patches/0001-checked-inspection.patch
```

Expected: exit 0. This is applicability, not correctness.

- [ ] Extract the `checked_xml_tests` and inspection fixture assertions into the target source as a red step, with the proposed symbols declared only when needed for compilation. Run the focused tests with coverage. Expected: unresolved new `inspect`/`validate_xml` symbols before implementation. Never run a known infinite-loop baseline without an external process deadline.
- [ ] Apply the remaining patch exactly as supplied. The parser must advance or leave its attribute loop; the checked path rejects incomplete structures, multiple roots, DTDs, unresolved entities and nesting >256 before recursive DOM parsing. This is structural admission, not an OOXML schema validator.
- [ ] Run the focused gates sequentially from the canonical repository root:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo +nightly llvm-cov --branch --lib inspect:: --json --output-path target/inspect-coverage.json
cargo +nightly llvm-cov --branch --lib checked_xml_tests --json --output-path target/xml-coverage.json
cargo run --bin jubarte -- --help
```

Expected: format/clippy clean; all selected tests pass; coverage files include actual line and branch counters; CLI exits 0. Pin the tested nightly date in CI after verifying compatibility, rather than claiming stable Rust line coverage is branch coverage. Report `Coverage: X% lines, Y% branches` from actual file counters. Target inspection ≥90% lines/85% branches; any uncovered refusal path needs an assertion, not an exclusion.

- [ ] Run the existing malformed CLI, strict namespace and package-validity suites with coverage before committing. Keep the Rust 1.88 compile gate separately: nightly coverage must not raise the library's MSRV.

```sh
cargo +nightly llvm-cov --branch --test m_cli_no_panic --test m8_strict --test m_validity_ring1 --json --output-path target/foundation-regression-coverage.json
cargo +1.88 check --all-features
git add Cargo.toml Cargo.lock src/lib.rs src/inspect.rs src/xmllinq/parse.rs src/document_comparer.rs
git commit -m "feat(inspect): add checked body projection and package facts"
```

`Cargo.lock` must be resolved by the implementation's sequential Cargo operation, even when promoting an existing dev dependency adds no new version. Never claim a patch's syntax or apply check proves compilation.

## Task C2: resource admission before general upload support

**Files:** create `src/admission.rs`; modify `src/lib.rs`, `src/inspect.rs`, new edit entry points and new language facade entry points; tests beside `admission.rs`.

The complete contract to implement is `InputLimits { max_compressed_bytes: u64, max_entries: usize, max_part_bytes: u64, max_uncompressed_bytes: u64, max_xml_depth: usize }` and `admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, AdmissionError>`. Initial configurable defaults: 64 MiB input, 10,000 entries, 64 MiB per inflated part, 256 MiB total inflated bytes, XML depth 256. These defaults are a product policy to benchmark, not proof against every denial of service.

Read the central directory before inflation; reject duplicate canonical entry paths, absolute/traversal paths, encryption and unsupported compression. Do not extract to disk. During inflation use a counting reader capped at `remaining_budget + 1`; central-directory sizes alone are untrusted. Validate CRC/stream termination. Limit XML before DOM recursion. Count UTF-8 bytes for byte budgets, not Python characters or JS UTF-16 units. Reject unsupported package kinds using declared content types/relationships, not filename extension alone. Do not execute macros, resolve external URLs or load external entities.

- [ ] Add in-memory ZIP fixtures for oversized advertised size, actual expansion exceeding advertised allowance, duplicate paths, `../` paths, unsupported encryption/compression, missing main part and valid boundary sizes.
- [ ] Assert exact error codes (`INPUT_LIMIT`, `DUPLICATE_PART`, `UNSUPPORTED_PACKAGE`, `INVALID_XML`) and zero document mutation. Test a declared high-compression XML part with a tiny budget; do not allocate the full payload to test a bound.
- [ ] Implement limits at the reader layer before `PartFs::open` and strict translation, which otherwise inflate unbounded input. Keep legacy APIs behavior stable until explicit versioned adoption of admission limits; new safe APIs must use them from day one.
- [ ] Run coverage for admission and edit together, then the same-input legacy regressions. Require ≥90/85 line/branch on policy logic. Keep resource stress tests in integration lanes under process/memory limits; they are not wall-clock unit tests.
- [ ] Commit admission separately from inspection and from any rendering change.

This task is a fully specified engineering work item, not claimed as implemented by patch 0001. The patch cannot be advertised as a comprehensive untrusted-upload solution until this task passes.

## Fidelity matrix and evidence ownership

| Concern | Existing representative tests / evidence | New API invariant |
|---|---|---|
| Word markup/order | `tests/m18_ppr_first.rs`, `m26_paragraph_mark_order.rs`, `m_numpr_child_order.rs` | No new invalid child ordering |
| Fields | `tests/m33_fields.rs`, `m474_real_field_ins_keeps_heading_ppr.rs` | Refuse unsupported edits across field boundaries; preserve untouched fields |
| Moves/format changes | `tests/m4g_moves_format.rs`, `m149_formatting_rpr_change.rs` | Preserve explicit supported compare preset defaults |
| Headers, notes, comments | `tests/m21_header_footer_diff.rs`, `m4h4_footnotes.rs`, `m35_comments.rs` | Discover related parts, never silently flatten unselected stories |
| Strict namespaces | `tests/m8_strict.rs`, `m24_strict_namespaces.rs` | Inspection normalize in memory; editing preservation policy explicit |
| Drawings/media | `tests/m10_deleted_drawing.rs`, `m74_opaque_vml_pict.rs` | Hash untouched binary payloads, preserve relationships |
| PDF conversion | `tests/convert_docx_to_pdf.rs`, `RESULTS.md`, Word oracle | Same fonts/options/corpus and separate render gate |
| Public surface | `tests/m7_cli.rs`, `m_cli_no_panic.rs`, binding consumer tests | old exports/defaults remain available |

The external benchmark owns runs/reports, but current source owns `jubarte-wasm`. Read its current README and benchmark AGENTS before rebuilding. Rebuild from one clean source commit, pin copied native binary hash/generated WASM hash, require equal per-document `script_redlines` scores, then run performance. Do not build a historical source copy, patch generated glue, or compare numbers from different corpora as a speed claim.

For edit acceptance, compare accepted/rejected semantic content, property/relationship structure and unmodified binary payloads. ZIP byte identity is not a required invariant; map iteration/compression order can vary. For each intentional normalization maintain a named assertion and rationale rather than a broad XML-ignore rule.
