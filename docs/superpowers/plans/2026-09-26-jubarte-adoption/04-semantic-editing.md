# Transactional DOCX Editing and the Acme Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let humans and agents propose, inspect, apply and verify Word-native edits without authoring OOXML or silently losing document features.

**Architecture:** Resolve all anchors against one immutable source snapshot, apply untracked edits to a copy, and use the existing comparer to generate tracked changes. Keep portable plan schemas and reports in Rust, with familiar typed builders in Python and TypeScript. Add structural operations only when their OOXML invariants and Word-oracle fixtures pass.

**Tech Stack:** Existing Rust DOM/OPC/comparer, SHA-256 source guards, versioned JSON Schema, PyO3, wasm-bindgen, existing renderer.

---

> **Status 2026-09-26 (branch `feat/agent-adoption`):** E1 and the core of
> E2/E3/E5 implemented in `src/edit.rs` with corrections; see
> [00-ASSESSMENT.md](00-ASSESSMENT.md) §3. Shipped plan schema v1:
> `replace`, `insert` (`after`/`before`/`position`), `delete`, `comment`,
> `insert_paragraph` (runs with `bold`/`italic`/`underline`/`highlight`,
> anchor `pPr` copied minus section break and revision marks),
> `delete_paragraph`; selectors `id`/`index`/`starts_with`/`contains`;
> `source_sha256` guard; `existing_revisions: refuse|accept|reject`;
> overlap detection; per-operation report with `to_jsonl()`.
> Corrections applied: (1) comments, including on text an earlier operation
> inserted, are authored in the clean copy and carried by the comparer
> (m35), so E5's provenance mapping is unnecessary; verified by
> `comments_on_source_text_and_on_inserted_text_survive_compare`;
> (2) patch 0005's "plain paragraph only" rule is replaced by a projection
> that ignores zero-width markers and refuses only ranges crossing opaque
> structures; (3) `expected_text` dropped in favor of the hash guard plus
> unique anchors. Not implemented: `merge_paragraphs`, `format_paragraph`
> (E4 paragraph formatting), rich formatting on inline `insert`/`replace`,
> `Preview.build`/`write_new_directory` (the CLI's `--out-dir` bundle covers
> the atomic-write need for now). Known renderer gap: balloons for comments
> anchored inside inserted runs are not painted (the comments are in the
> file). Verified: `tests/edit_plan.rs` (18), module unit tests.

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## Why this API can attract users

The product is a reviewable change transaction, not a bag of XML helpers. An agent can say “replace this exact clause in this paragraph,” inspect a preview, and return both an editable Word redline and a PDF. A different agent or a person can review the same serialized plan and apply it to the same source without replaying the chat. This makes Jubarte useful inside existing contract, policy and template workflows, with no requirement to adopt an agent framework.

A plain-text patch against a Markdown conversion cannot promise preservation of Word structures. A plan bound to the original package can. Conversely, this design must visibly refuse unsupported structures rather than imply it is a universal Word editor. Inspection, planned changes, completed edits, comparer revisions and rendered output are distinct artifacts.

## Tiered API, with one underlying plan

Tier 1: direct bytes functions for existing integrations. Tier 2: immutable `Document` values, typed options and structured reports. Tier 3: semantic `Draft` builder over a versioned portable `EditPlan`. No global mutable configuration and no undocumented parser behavior at any tier.

The following is the **target editing API**, distinct from implemented facade patch 0002. Types and methods are defined below; publish this example only when its vertical slice ships:

```python
from pathlib import Path
import jubarte_redlines as jubarte

source = jubarte.read("contract.docx")
snapshot = source.inspect(scope="body")
clause = snapshot.paragraphs.unique(starts_with="(a) Confidentiality.")
plan = jubarte.EditPlan.for_snapshot(snapshot, author="Legal review", date="2026-09-26T14:30:00Z")
plan = plan.replace(
    paragraph=clause.id,
    find="retained experts",
    with_="retained experts, court reporters and e-discovery vendors",
    expect=1,
)
preview = source.preview(plan)
preview.raise_for_errors()
bundle = preview.build(redline=True, pdf="word")
bundle.write_new_directory(Path("review-001"))
```

TypeScript target uses the same nouns and normalized wire schema:

```typescript
import { createEditor } from "jubarte-wasm/editor";
const editor = await createEditor();
try {
  const snapshot = await editor.inspect(sourceBytes, { scope: "body" });
  const clause = snapshot.paragraphs.unique({ startsWith: "(a) Confidentiality." });
  const plan = editor.plan(snapshot, { author: "Legal review", date: "2026-09-26T14:30:00Z" })
    .replace({ paragraph: clause.id, find: "retained experts", replacement: "retained experts and court reporters", expect: 1 });
  const preview = await editor.preview(sourceBytes, plan);
  preview.raiseForErrors();
  const bundle = await preview.build({ redline: true, pdf: "word" });
  download(bundle.redline, "redline.docx");
} finally {
  await editor.dispose();
}
function download(bytes: Uint8Array, name: string): void {
  const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)]));
  const link = document.createElement("a");
  link.href = url; link.download = name; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
```

The download recipe is a browser example, not a unit test; use an appropriate user gesture/lifecycle in the actual app. File output is a separate operation after preview, never an implicit context-manager effect.

## Portable contract

`Snapshot` contains `schema_version:1`, source SHA-256, ordered `Paragraph` records and warnings. `Paragraph` contains opaque `id`, `story`, `part`, `index`, `text`, `style_id`, `numbering`, `editable_operations` and `limitations`. `ParagraphCollection.unique` is a client convenience that requires exactly one matching paragraph and returns it; zero/multiple matches are typed errors. It does not mutate or find “the nearest” text.

`EditPlan` contains schema version, source hash, author, timestamp, `existing_revisions` policy and ordered operations. `for_snapshot`/`plan` capture the hash and immutable metadata. Each builder returns a new plan. User-visible ordering establishes report order; resolution remains source-relative. Wire field names use snake_case; TypeScript/Python builders translate language spelling only, not document semantics.

A minimal exact plan:

```json
{
  "schema_version": 1,
  "source_sha256": "4b4dd3c5015e2a62c5cf5c673f04e4a679c188f8ed238b8fce0cc6e81a43d8e9",
  "author": "Legal review",
  "date": "2026-09-26T14:30:00Z",
  "existing_revisions": "refuse",
  "operations": [
    {"id":"op-1","kind":"replace","paragraph":"body:p:18","find":"retained experts","replacement":"retained experts and court reporters"}
  ]
}
```

The hash above illustrates the field shape, not the supplied Acme file's hash. Production plans must derive it from actual bytes; no hardcoded sample hash may pass validation.

`Preview` contains `ok`, immutable per-operation outcomes, warnings, capabilities and input/plan digests. It stores the computed edited snapshot; `build` reuses that snapshot, not a second anchor-resolution pass. `raise_for_errors` raises an aggregate `EditPlanError` with structured details. `build` returns `ReviewBundle` with clean DOCX, optional redline/PDFs and report; it must not write. A schema/code capability mismatch fails before edits. `write_new_directory` creates a staged sibling directory, writes all requested files, then exposes a complete bundle only if every requested generation succeeded. Refuse an existing destination. Single-file rename atomicity does not imply cross-filesystem or multi-file atomicity; keep staging on the same filesystem.

`EditOutcome` has `operation_id`, status, match count, source anchors, resulting anchors if stable, warnings and error code. A failed plan returns no success DOCX. Do not promise one comparer revision per requested operation: a replace can generate multiple insert/delete/move/format records. Report the plan outcomes and resulting Word revisions separately.

## Anchor and conflict rules

- IDs are valid for one exact byte snapshot, not a persistent document identity across Word saves. Validate source SHA-256 first, then full paragraph/range expectation.
- Exact raw Unicode is the source coordinate system. The core may store byte offsets, but never expose them as JS string indexes. Emoji, combining accents, RTL, NBSP and tabs have dedicated fixtures.
- `find` must be nonempty. Default occurrence count is exactly one; count overlapping candidates too. `aaa` contains two occurrences of `aa`, so it is ambiguous.
- Resolve every operation against the untouched source before mutation. Overlapping replacements/deletions fail; identical offsets for two inserts require explicit ordered grouping, not arbitrary sort order.
- Paragraph deletion conflicts with every edit/comment anchored inside it. Merge conflicts with edits crossing its deleted boundary unless a single structural group explicitly owns them. An edit inside a field/hyperlink/content-control/bookmark range requires supported span rules; never split an opaque structure accidentally.
- “First”/“each” are explicit selection modes with report cardinality, not hidden fallbacks. Defer broad `*_all` aliases until repeated user need justifies their semantics. No fuzzy/autocorrected legal edits by default.
- `existing_revisions="refuse"` is the new-editor default. Explicit accept/reject creates a new base snapshot with a new hash, records the action, and compares against that chosen base. This is not preservation of old revision history.

## Task E1: implement the conservative source-guarded text core

**Files:** `src/edit.rs`, `src/lib.rs`, `Cargo.toml`, lockfile; complete reference source/tests in `patches/0005-guarded-text-edit.patch`, applied after 0001.

Reference public types are `TextEditPlan { source_sha256, edits }` and `TextEdit { paragraph_index, expected_text, find, replacement }`. `source_sha256(bytes)` creates the snapshot guard; `apply_text_edits(bytes, &plan)` returns new DOCX bytes or `EditError {code, operation, message}`. This lower-level vertical slice intentionally does not yet define the full fluent API above.

- [ ] Apply the new unit test module first and run a coverage-enabled compile to observe missing `edit` symbols.
- [ ] Apply the full reference implementation and dependency/export hunks. It resolves all matches and overlap conflicts before mutation, applies in reverse source order, preserves run properties/unmodified text and lets existing compare produce tracked changes.
- [ ] Extend its in-memory tests to cover every documented refusal: stale source/full paragraph, missing/duplicate/overlapping anchor, empty find, invalid controls, direct split runs, emoji/combining accents, multiple edits in one run, deletion to empty text, unsupported field/hyperlink/SDT/bookmark/section, revisions in header/notes, signed/macro package, and an unchanged opaque media payload.
- [ ] Verify with sequential commands and report actual branch counters:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo +nightly llvm-cov --branch --lib edit:: --json --output-path target/edit-coverage.json
cargo run --bin jubarte -- --help
```

Target ≥90% lines/85% branches for edit/resolution code. The supplied Rust reference patch is not claimed compiled or release-ready in this planning task; see VALIDATION. Admission limits from C2 and the expanded tests above are explicit prerequisites for exposing it to untrusted documents.

- [ ] Add an integration fixture comparing source to edited output with explicit author/date. Assert accept(redline) matches intended edited structure/text; reject(redline) matches the chosen base; untouched payloads/relationships satisfy preservation checks. Validate with `tests/common/validity.rs` and Word-open oracle.
- [ ] Commit narrowly with `feat(edit): add guarded exact text transactions`.

## Task E2: expose one complete inspect→edit→redline vertical slice

**Files:** create `src/agent_service.rs`, schema `schemas/edit-plan-v1.json`, `schemas/edit-report-v1.json`; modify both binding source files and Python/JS typed facade files. Tests in Rust service module and corresponding binding suites.

The service owns: schema decoding (reject unknown operation kind and version), limits, hash check, existing-revision policy, operation resolution/application, comparer call, renderer call when requested and report assembly. Bindings convert typed values; neither binding implements its own XML locator. Use serializable DTOs isolated from internal DOM structures and generated schemas from the Rust model where practical.

Sequence: admit → inspect/capability check → validate plan → resolve → apply to copy → package checks → compare → accepted/rejected consistency checks → optional render/report. An error before completion returns diagnostic outcomes and no success bundle. Keep `on_invalid="warn"` out of the public writer: warning mode must never authorize knowingly invalid DOCX output.

- [ ] Define fixtures containing version mismatch, unknown kind, stale source and a valid replace plan; verify both languages receive the same code/outcome count.
- [ ] Implement Rust service and expose thin new PyO3/WASM exports without changing old functions. JSON objects must reject unknown/misspelled fields where omission could change meaning.
- [ ] Implement immutable builders and preview/report value types from the contracts above. Keep constructor/serialization tests pure; use actual engine integration only in integration tests.
- [ ] Install actual wheel/tarball outside the source checkout and run matching Python/TS use cases, including a failed plan that writes no files.
- [ ] Release the exact-edit capability as experimental with its explicit supported structure table. Do not market the complete Acme workflow until E3–E6 pass.

## Complete Acme operation map

All 14 operations from the attachment are accounted for. Text is example material, not legal advice or an instruction to change a real contract. The actual `.docx` was not attached; text-derived synthetic fixtures prove operation behavior, not that original layout.

| # | Requested operation | Core change required | Acceptance assertion |
|---:|---|---|---|
| 1 | Insert recipients after `retained experts, ` in confidentiality; attach comment | exact insert span + operation-result comment anchor | one scoped match; original comma/spacing preserved; comment visible in Word |
| 2 | Define “Permitted Recipient” after scoped purpose phrase | exact insert with paragraph selector | duplicate phrase elsewhere unchanged |
| 3 | Append responsibility sentence at confidentiality paragraph end | explicit paragraph-end insertion | inherits last live text run style; no new paragraph mark |
| 4 | Delete onward-disclosure paragraph including mark | structural delete with section/relationship checks | paragraph disappears from clean; redline marks deletion; adjacent numbering stable |
| 5 | Insert AI Services paragraph with bold heading | paragraph insertion + mixed run builder + pPr policy | correct position, heading bold only, numbering/indent preserved |
| 6 | Add `2(c), ` to survival list | exact scoped insert | only intended list changes |
| 7 | Add `7(c), ` to survival list | exact insert resolved before #6 | source-relative anchors remain valid after first insertion |
| 8 | Comment on survival clause | source-range comment | anchors exact source range after compare remapping |
| 9 | Add `email ` before address phrase | exact insert | spacing exact and rest of clause unchanged |
| 10 | Replace courier language with text/bold highlighted placeholder + comment | rich replacement span with declared formatting | original runs around range preserved; only placeholder bold/yellow |
| 11 | Add courier timing qualifier | exact insert | no duplicated punctuation; one match |
| 12 | Merge jury-waiver heading and body | merge within same story/container and section policy | one clean paragraph with first pPr, text boundary spacing specified |
| 13 | Replace “his or her” with neutral phrase | exact replacement | unique signature-block match; no global pronoun rewrite |
| 14 | Single-space drafting notes to end | paragraph-range formatting | only target pPr changes; headings, text and preceding section retained |

## Task E3: structural paragraph operations

**Files:** split `src/edit.rs` into `src/edit/{mod,model,resolve,text,paragraph,report}.rs` only at this milestone; create `tests/edit_paragraphs.rs`.

`InsertParagraph` requires an existing anchor ID, before/after side, explicit run list and paragraph property policy (`copy_anchor` or named style). Never copy section breaks, bookmarks, comment IDs, drawing relationship references, numbering definitions or revision IDs blindly. Clone only allowed pPr properties; allocate identifiers/remap relationships when an allowed operation introduces them.

`DeleteParagraph` refuses the only required paragraph of a table cell unless it replaces it with the required empty paragraph under a defined policy. Refuse paragraph section breaks in v1. A deletion cannot discard referenced comments/notes/images without reconciling their references. `MergeParagraphs` requires adjacent paragraphs in the same body/cell/story, no section or field boundary, explicit separator string, and `keep_format="first"|"second"`; paragraph formatting policy does not erase run formatting.

- [ ] Add synthetic cases for #4/#5/#12 plus a table-cell last paragraph, section break, bookmark crossing, and a field split across paragraphs.
- [ ] Implement each operation in a separate commit, with a source-span conflict rule and exact Word-valid child ordering test.
- [ ] Verify accepted/rejected redline behavior and Word-open oracle after each operation; compare lexical text alone is insufficient.

## Task E4: rich runs and paragraph formatting

**Files:** `src/edit/{model,format,resolve}.rs`, `tests/edit_format.rs`; binding builder types and schemas in the same commit.

`Run` has text and explicit property changes, e.g. `{bold:true, highlight:"yellow"}`. Unspecified properties inherit from the selected source run; false means clear the property rather than absent. Named styles must exist and match expected style type. Validate highlight/color/font sizes/units with typed enums/numbers; never accept arbitrary raw XML through semantic operations.

`FormatParagraph` takes a range of paragraph IDs and a typed `ParagraphFormat`. For line spacing, distinguish `multiple:1.0` from `exact_points:12`; never encode either as a unitless floating field. Preserve unsupported pPr children and ordering. For #14, source-range end is explicit end-of-story, not a text substring that can appear in a hidden comment.

- [ ] Assert #5 bold range and #10 yellow placeholder with unmodified surrounding run properties.
- [ ] Assert #14 changes only requested spacing, with correct OOXML units and tracked `pPrChange` behavior through compare.
- [ ] Reject unknown styles, invalid units, conflicting range edits and control characters before output.

## Task E5: comments after comparison

**Files:** `src/edit/comments.rs`, `tests/edit_comments.rs`, binding schema/models.

Comments anchored to newly inserted content need a stable mapping from operation result spans into the redline. Do not locate inserted text afterward by a global substring search; repeated text makes that unsafe. Carry provenance anchors through applying and comparing, or deliberately restrict supported comment spans until mapping is available. A comment on source text has a separate anchor kind.

Start with classic comments if that meets the Word oracle; do not manufacture threaded-comment extension parts merely because the draft lists them. If extensions are used, assign coherent IDs and relationships/content types and preserve existing threads/people metadata. Allocate comment IDs disjoint from existing ones, balance range start/end/reference markers, and enforce XML child ordering.

- [ ] Test a comment on an inserted phrase, a comment on unchanged source text, existing comments, repeated identical text, deleted anchor and a range crossing unsupported containers.
- [ ] Prove visible comment anchors in Word and package validation, not only existence of `comments.xml`.
- [ ] Only then enable `comment=` sugar on operations and the standalone `Comment` operation.

## Task E6: atomic writing, preview and replay

**Files:** Python `bundle.py`, JS `node-files.mjs` (no filesystem in browser core), shared report schema and integration tests.

Draft states: OPEN → PREVIEWED → BUILT → WRITTEN or DISCARDED; mutation after preview creates a new plan and invalidates cached preview. Exceptions and context-manager exit discard pending work by default. Writing requires a successful preview for the identical source/plan/settings digest. A user can serialize a plan, inspect it elsewhere and replay; any hash mismatch is a conflict requiring fresh inspection.

Generate all requested bytes before exposing the final directory. Stage outputs next to the destination; use fixed safe filenames owned by the library, reject traversal and pre-existing target, and clean only this invocation's staging directory on failure. Do not claim a global transaction over arbitrary user-chosen paths; offer a bundle directory API with clear filesystem guarantees.

Integration assertions: failure on the second render leaves no final bundle; existing destination remains unchanged; replay on identical source yields same semantic results; stale source fails; no output file exists after discard; log/receipt excludes text by default. Test filesystem behavior with an in-memory filesystem interface in unit tests and temporary real directories in integration tests.

## Expansion and launch gate

The exact-edit beta does not substitute for the full requested Acme workflow. Ship the complete workflow only after all 14 rows pass, with an actual DOCX layout fixture when available, Word validity/open evidence, accepted/rejected structure checks, comments anchored correctly, same-source native/WASM redline parity and PDF font/report evidence. Until then, list each unavailable operation in machine-readable capabilities so agents can choose supported tasks without hallucinating methods.
