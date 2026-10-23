<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Jubarte 0.13 benchmark fixes — implementation plan

## 1. Summary and deliverables

**Goal:** eliminate reproduced revision corruption, fix compound cell edits, and resolve remaining CLI and schema-hygiene findings without changing Word-faithful behavior.

**Baseline:** assessed checkout `ed635d691a3142273031799819cd5b775a86fc9a`. Recheck the implementation branch before editing; the previous assessment confirmed that its five relevant implementation files matched GitHub main at `422162ac`.

**Architecture:** keep fixes in the Rust engine and shared preparation pipeline. Preserve existing plan schemas and machine-output formats. Rebuild consumers from the fixed source.

**Planned document:** `docs/superpowers/plans/2026-10-23-fixing-issues.md`, this file (drafted as `2026-10-23-jubarte-0.13-benchmark-fixes.md`).

**GitHub deliverable:** one tracking issue titled **`[0.13][Urgent] Fix reproduced revision corruption and remaining benchmark regressions`**, labeled `bug` and `Urgent`, with the tasks and corrected evidence below. Link existing owners, including #1250 and the closed #1153 follow-up.

The tracking issue is #1656. Each task below lands as its own pull request, stacked on the one before.

## 2. P0 — Revision identity, history preservation and publication

### Task 1: reproduce formatting-history corruption deterministically

- [x] Add `tests/edit_formatting_history.rs`, using the existing in-memory DOCX builder.
- [x] Construct a run containing current formatting plus Alice’s `rPrChange`, with fixed author/date and recorded old properties.
- [x] Cover replacement and insertion inside that run, both outside and inside Alice’s tracked insertion.
- [x] Include insertions exactly at run boundaries as negative controls.
- [x] Cover comment anchoring that splits the same run, multiple splits in one transaction, and non-ASCII text.
- [x] Assert unique revision IDs, preserved attribution and old properties, expected accepted text, and restoration of original history after rejecting only Bob’s edits.

Retain the five supplied plans as integration reproductions. Before task 2, ops 0, 1 and 4 failed validation with `DUPLICATE_REVISION_ID` and `FOREIGN_AUTHOR`. After task 2 only `FOREIGN_AUTHOR` remains: the history audit compares formatting records by id, so a copy's fresh id looks like new foreign history until task 3 (#1659). Controls 2 and 3 pass.

### Task 2: allocate IDs when retained runs split

Change the private run-splitting helpers in `src/edit.rs` and their tracked-emission callers.

- [x] Thread a transaction-owned monotonic revision counter through helpers that retain both fragments.
- [x] Seed it above existing annotation IDs, using the repository’s existing allocation helpers.
- [x] Keep the original formatting record’s ID on the original fragment; allocate fresh IDs to formatting records copied onto retained fragments.
- [x] Preserve author, date and recorded property payload on those fragments.
- [ ] Continue using cleaned run properties for newly authored text, so it cannot acquire another author’s formatting history. Not done by the split's fresh ids: owned by #1658.
- [x] Update all callers, including comment-range splitting and typed editing, rather than fixing only the reported replace path.

Do not settle foreign revisions, discard their history, or attribute their retained history to the reviewing author.

### Task 3: compare formatting history by coverage and properties

Extend the private audit representation in `src/changes.rs` and its comparison in `src/validate.rs`.

- [x] Represent run-formatting history with its recorded property payload and covered content interval within the aligned story/paragraph.
- [x] Normalize property payloads independently of revision IDs and XML prefix spelling.
- [x] Treat touching fragments as equivalent only when attribution, date, property payload and continuous content coverage agree.
- [x] Preserve boundaries around unrelated text, revisions, paragraphs and stories.
- [x] Use the same equivalence rules in typed-edit history checks.

Add negative tests for changed old properties, changed author/date, missing fragments, reordered coverage, and identical-looking records belonging to different paragraphs.

Done in #1661 (#1659): `tests/validate_formatting_history.rs` and the `history_guard_*` unit tests in `src/edit/v2_apply.rs`.

**Acceptance:** the duplicate-ID finding and the false appearance of newly authored foreign history are both cleared. Task 2's fresh IDs clear the first. Comparing by coverage and properties clears the second, because a fragment under a fresh ID is no longer new history. History that new text genuinely inherited is #1658's to remove. Genuine history changes remain detectable.

### Task 4: block newly introduced duplicate IDs before publication

Update shared candidate checks in `src/edit/prepared.rs`.

- [ ] Check clean and tracked serialized candidates.
- [ ] Compare revision occurrences by part, record type and ID. A candidate must not introduce an additional same-kind duplicate occurrence.
- [ ] Keep inherited duplicates separate from newly introduced duplicates; healing one ID must not conceal a collision under another.
- [ ] Retain valid cross-kind ID sharing.
- [ ] Refuse with `CANDIDATE_INVALID` before writing outputs; attach the responsible operation when provenance identifies it.
- [ ] Keep `DUPLICATE_REVISION_ID` classified as non-Word-fatal. Publication policy and Word-opening behavior are separate concerns.

Repair must retain its existing nonzero result when validation against the original still fails. Its help and completion text must distinguish structural repair from history verification.

## 3. P1 — Compound edits, diagnostics and CLI usability

### Task 5: validate cell topology for the entire transaction

Extend `tests/edit_empty_paragraph.rs` and update paragraph resolution/conflict checking.

- [ ] Move the “last paragraph” decision out of individual-operation resolution.
- [ ] Compute surviving and inserted paragraphs per original container after resolving the complete plan.
- [ ] Permit paragraph insertion before or after an original paragraph that the same plan deletes, using that original paragraph’s stable position as the insertion boundary.
- [ ] Remove the separate deleted-anchor refusal for this supported paragraph-insertion case.
- [ ] Preserve existing restrictions for image/table insertion and unrelated overlaps.
- [ ] Verify final container structure after application and serialization.

Test insertion before/after the old paragraph, both operation orders, multiple inserted paragraphs, deletion-only cells, and body/header/footer containers.

**Acceptance:** replacing a cell’s sole paragraph through a compound plan succeeds; deleting all resulting paragraphs still fails atomically. Clean, tracked, Accept All and Reject All outputs retain valid required containers.

### Task 6: improve actionable diagnostics

- [ ] Retain operation provenance through run splitting and structural scheduling.
- [ ] Populate the existing primary `operation` field when a candidate finding maps to one operation.
- [ ] Name additional conflicting operation IDs and targets in the message.
- [ ] Keep document-level failures document-level when attribution is genuinely unavailable.
- [ ] Preserve existing error codes and exit-status contracts.

Keep the historical panic as an unreproduced lead. Record its event and missing-plan limitation; do not claim an off-by-one fix without recovering a failing case.

### Task 7: make supported CLI combinations discoverable

- [ ] Replace `batches` with `operations` in help.
- [ ] Show one complete legacy plan and the separate PlanV2 template/apply workflow.
- [ ] Accept repeated `read -p` selectors by combining them with existing comma-list semantics.
- [ ] Support `read --tables -o FILE`, producing the same table inventory as stdout.
- [ ] Add `changes --author NAME` using existing author-filter semantics.
- [ ] Preserve JSONL output for changes/comments and the snapshot object for `read --json`.
- [ ] Keep stale-source checks and input/output alias protection.

Update the repository’s skill document and CLI documentation with output chaining, explicit delivery paths, exit-status checking, and validation against the preserved original. Do not edit frozen benchmark task installations.

These are additive CLI capabilities. Plan schema versions and document-format contracts remain unchanged; document any public CLI argument-structure changes in Rust.

## 4. P2 — Schema hygiene with corrected ownership

Extend the comparer’s shared finalization and schema-order tests.

- [ ] Remove an empty `w:shd/@w:fill`, preserving other shading attributes.
- [ ] Remove undeclared `w:gridCol/@w:type`, preserving column widths and extension metadata.
- [ ] Add schema-derived `tblCellMar` child ordering: `top`, `start`, `left`, `bottom`, `end`, `right`.
- [ ] Apply the same ordering to document and style parts.
- [ ] Assert unchanged text, widths, effective shading and revision attribution.

Use small in-memory probes first, then the public lane fixtures.

Record these assessment corrections in the tracking issue:

- Page-margin defects are inherited; do not invent header/footer/gutter defaults.
- Missing header-table properties and the textbox namespace occur in source documents; do not describe them as newly introduced defects.
- Preserve `mc:AlternateContent`; validator preprocessing must not erase its branches.
- Preserve the reported math-revision shape pending a Word-oracle comparison. Schema rejection alone does not establish a Word defect.

Any confirmed Word/schema divergence belongs in `docs/WORD_DIFFERENCES.md`.

## 5. Verification, sequencing and completion

Implement in this order: **regressions → ID allocation → history comparison → publication gate → compound topology → CLI/docs → schema hygiene**. Use focused commits and inline execution; no delegation is required.

For each behavioral change, demonstrate the new regression failing before the fix and passing afterward. Use in-memory fixtures for deterministic tests; filesystem and CLI probes belong in integration tests.

Run Cargo commands sequentially. Plain `cargo` goes through mbx's shim (AGENTS.md); the coverage run needs the nightly toolchain:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo +nightly llvm-cov nextest --branch \
  --test edit_formatting_history \
  --test edit_inside_insertion \
  --test edit_empty_paragraph \
  --test validate_public \
  --test validate_cli \
  --test read_options_json \
  --test cli_parser \
  --test schema_consistency
cargo run --bin jubarte -- --help
```

Use the nightly coverage toolchain for branch coverage. Report achieved line and branch coverage, targeting **90% lines and 85% branches for new critical logic**, without lowering established thresholds.

Before completion:

- [ ] Replay all five supplied operations against pristine originals; all must validate without introduced revision/history errors.
- [ ] Verify repair’s structural and history results separately.
- [ ] Check changed OOXML shapes through the benchmark’s Word scripts: normal opening, Accept All, Reject All and rejection of the reviewing author’s edits.
- [ ] Run OpenXmlValidator and MCE-aware schema checks on representative comparison outputs.
- [ ] Rebuild native/WASM consumers from the same commit and require matching affected-case behavior.
- [ ] Keep historical benchmark arms unchanged.
- [ ] Save the implementation plan and publish the single prioritized GitHub issue, including reproduction hashes, corrected classifications, task checkboxes and verification boundaries.
- [ ] Report the issue URL, implemented commit, coverage, Word results and remaining unreproduced leads.

No release publication or merge is part of this plan.
