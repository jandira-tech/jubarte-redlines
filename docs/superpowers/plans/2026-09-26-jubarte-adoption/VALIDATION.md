# Planning Bundle Validation and Remaining Engineering Gates

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

This document reports what was actually checked while preparing the plan. It is not a product release certificate. All product changes remain proposed patch artifacts; no product source/license change or publication was performed by this planning work.

## Patch inventory

Apply in numeric order after reviewing current source status. Patches 0001–0004 and 0006–0007 are independent of each other's source semantics; patch 0005 expects 0001's inspection/export/dependency changes. Never `git am` an old attached patch on top of the superseding inspection patch.

| Patch | Concrete contents | Verification achieved | Remaining release work |
|---|---|---|---|
| `0001-checked-inspection.patch` | parser progress, structural XML checks, conservative body inspection, relationship-aware summary, Rust tests | sequential Git apply check; Rust syntax/format parsing for inspected proposed files | actual compile/clippy/coverage, broader malformed-input/namespace/resource admission and existing tolerance regressions |
| `0002-python-document-api.patch` | immutable Document/options/results, additive exports, tests/config | 33 tests passed against installed native 0.9.2; 100% lines/branches for two Python modules | rebuilt binding, oldest supported Python, all wheel platforms, mypy/pyright distribution checks |
| `0003-javascript-api.patch` | typed ESM facades/declarations for Node/browser/slim, export map, 27 wrapper tests and locked private tools | 27 wrapper tests plus one installed-WASM integration passed; strict TS consumer passed; 100% lines/branches for facade.mjs | packed artifact across supported runtimes, browser initialization/loading, future worker client lifecycle |
| `0004-adoption-recipes.patch` | complete python-docx/npm-docx→PDF recipes and opt-in experiment ledger | both recipes produced PDFs from installed 0.9.2 packages | observed independent users; visual/layout checks and clean-environment support matrix |
| `0005-guarded-text-edit.patch` | source-hash guard, exact match/overlap validation, conservative run-preserving text edits and Rust tests | sequential apply check after 0001; rustfmt parsed source | compile/clippy/coverage, full refusal/correctness matrix, admission limits, shared binding/service integration, Word validation |
| `0006-agent-skill.patch` | original scoped skill and complete current-API recipes | source/reference review and patch applicability | actual Codex/Claude routing/task evaluation and packaged distribution |
| `0007-local-playground.patch` | local file→worker→PDF/redline preview/download/cancel prototype, Vite console-pipe config | Vite production build succeeded | browser task smoke, sample-first flow, resource limits, accessibility, request-body privacy check, supported-browser matrix |

These are reviewable reference patches, not seven production-ready commits. The complete higher-level editing API in plan 04 requires the explicitly specified service, binding and structural/comment milestones; its examples must not be advertised as available today. The source patches intentionally retain the current license until a rights-cleared migration is implemented.

## Actual checks and scope

### Python facade

A temporary CPython 3.14.2 environment installed `jubarte-redlines==0.9.2`, pytest 9.1.1, pytest-cov 7.1.0 and coverage 7.16.1. The proposed Python files were loaded from a temporary package alongside the installed native extension. No local Rust binding rebuild was performed.

Result: **33 passed**. Coverage: **100% lines, 100% branches** for `document.py` and `models.py` only (119 executable statements, 34 branches). PDF/font and filesystem tests are identified as integration tests. In-memory OPC compare tests call the real deterministic engine; no generic mocks were used.

The first run found an incorrect byte-for-byte ZIP assertion. Inspection showed equal decompressed part contents and equal revision records with different archive bytes/order. The test was strengthened to compare the exact decompressed part map plus revision metadata, keeping coverage intact. This is why the plan avoids ZIP byte-identity promises.

`research/python-coverage.json` preserves the actual coverage report. Its temporary paths identify the reference package used; they are not canonical product paths.

### JavaScript/TypeScript

The wrapper tests ran in an isolated npm project with Vitest 5.0.2/v8 coverage. **27 unit tests passed** using one owned deterministic engine fake. An additional **one integration test passed** against the installed `jubarte-wasm@0.9.2` package with the proposed facade/export overlay: compare, typed revisions, accept/reject, PDF signature and slim capability.

Coverage: **100% lines, 100% branches** for the local `api/facade.mjs` reference (50 lines, 47 branches). This does not measure generated glue/WASM, browser entrypoints or Rust coverage. Strict TypeScript `NodeNext` consumer compilation succeeded, including negative checks for missing custom palette, unsupported date option and absent slim PDF export.

Node emitted an experimental localStorage warning from the test/runtime environment; the run passed. The warning is retained as an observation, not suppressed or interpreted as a document-engine failure.

`research/javascript-coverage-summary.json` preserves the actual summary. The integration test was run against the reference facade overlay, not a newly published package.

### Library/recipe smoke

- Published native Python import succeeded and reported version 0.9.2.
- `python-docx` generation→Jubarte PDF produced a nonempty 18,136-byte PDF in a temporary output path.
- npm `docx` generation→Jubarte WASM PDF produced a nonempty 7,248-byte PDF in a temporary output path.
- Output recipes use exclusive file creation, so a retry does not silently overwrite an existing artifact.

These are startup/integration smoke results. They do not establish visual equivalence, page fidelity or feature coverage. No claim was made that the synthetic documents reproduce the unavailable original Acme DOCX.

### Local playground prototype

Vite 7.3.6 built the proposed browser prototype successfully with its worker/WASM asset import. The observed uncompressed WASM asset was about **12.04 MB**, gzip about **5.44 MB**. This is a useful cold-start/distribution measurement for that exact installed artifact, not the draft's older size estimate.

The prototype includes console-pipe using its documented default export. Browser interaction, preview/download, cancel, network-body checks and accessibility remain explicit verification work; a successful build does not prove those behaviors. It is not deployed and has no analytics/upload code.

### Applicability and repository scope

All seven patches passed sequential `git apply --cached --check` in a **temporary Git index** populated from the then-current HEAD. Product files and the live index were not patched. Exact source revision and per-patch statuses are in `research/patch-applicability.json`. Re-run before implementation because this is a shared, actively changing checkout.

REUSE baseline was clean when inspected. Newly authored plan/patch assets carry or are covered by the repository's current first-party attribution rules; the final lint result is recorded after artifact creation. No Apache license conversion was made. A REUSE success checks declared licensing completeness, not legal authority to relicense.

## Completion audit of the planning scope

| Requirement | Evidence | Status |
|---|---|---|
| Distinguish attachment content from instructions | master attachment disposition; Acme operation map; skill untrusted-content rule | addressed |
| Understand current source/bindings | master snapshot, current-state correction, source file maps | addressed, refresh required before implementation |
| Feasibility/desirability scores 0–1 | 18 technical proposals + 14 growth proposals with definitions | addressed |
| Familiar Python/JS examples | plans 02/03/04/07, complete recipe/facade patches | addressed; current vs target APIs labeled |
| Preserve current functionality | additive contracts, fidelity matrix, regression/release gates | planned; product gates are not falsely claimed passed |
| Apache migration | rights inventory, exact scope/manifest changes, notice/artifact checks | planned; legal clearance and migration not performed |
| Actual code changes proposed | seven unified patch artifacts plus detailed later-stage contracts | reference implementation provided for initial vertical slices; later capabilities require remaining implementation work |
| Complete Acme workflow | all 14 operations mapped to core changes and assertions | fully scoped; exact original layout remains unverified without DOCX |
| Increase adoption beyond another API wrapper | growth program, concrete launch assets, partner drafts, demo prototype, integration recipes and retention experiments | addressed as a strategy/assets; actual users are not invented |
| Bite-sized engineer handoff | file maps, task steps, test/commit commands and patch source | initial slices actionable; complex later milestones need finer code-complete decomposition before execution |
| No unauthorized product/publishing/outreach changes | product-source status; only plan directory/master created | maintained |

## Honest outstanding work in the plan itself

This is a substantially expanded working plan, written incrementally. Before calling the entire requested plan implementation-ready under the writing-plans skill, finish these concrete planning gaps:

1. Expand the complex later milestones (admission budgets, shared edit service/bindings, structural Acme operations/comments, worker client, release manifest/CI) into smaller steps with complete code/test patches, not only the current detailed contracts.
2. Add complete acquisition prototype patches for document regression CI and review receipts, with deterministic tests and runnable user examples.
3. Verify the new Rust reference modules through an appropriate isolated review/build procedure that respects the canonical-checkout and single-Cargo rules; do not apply product code to an active shared tree merely to make this planning report greener.
4. Complete browser smoke for the proposed local playground using console-pipe, or keep its UI behavior explicitly unverified; strengthen the sample-first adoption flow.
5. Re-run the author self-review for type/signature consistency, complete patch dependencies, stale source facts and every local artifact link after those additions.

The active goal remains open while these gaps are being addressed. The planning work is not blocked by pending publication, outreach, or licensing authority: those are execution gates, and useful preparation can continue.


## Implementation record, 2026-09-26 (branch `feat/agent-adoption`)

Implemented in a git worktree of `892ddbc` (user instruction; overrides the
canonical-checkout note above), Rust 1.95, in an 8 GB container with Cargo
serialized (`-j1`) and `ooxmlsdk` built without debuginfo through an
uncommitted local `.cargo/config.toml`. Everything below was compiled and
run there; see [00-ASSESSMENT.md](00-ASSESSMENT.md) §7 for the numbers.

| Patch | Disposition |
|---|---|
| 0001 | superseded by `src/xmllinq/parse.rs` (guard + `validate_xml`) and `src/inspect.rs` (richer, non-refusing read model) |
| 0002 | applied, then extended (`document.py`, `models.py`, `__main__.py`, stubs) |
| 0005 | superseded by `src/edit.rs` (plan schema v1, selectors, comments, paragraph operations, report) |
| 0006 | superseded by `skills/jubarte-documents/SKILL.md` |
| 0003, 0004, 0007 | untouched (TypeScript, recipes, playground: out of this scope) |

Gates run: `cargo fmt --check`; `cargo clippy --all-targets --all-features
-- -D warnings`; `cargo test` for the library, the binary and the suites
`inspect_paragraphs`, `edit_plan`, `convert_docx_to_png`, `m_cli_agent`,
`m7_cli`, `m_cli_no_panic`, `convert_docx_to_pdf`, `convert_revision_palette`;
`jubarte --help`; `pytest --cov=jubarte_redlines --cov-branch`.
