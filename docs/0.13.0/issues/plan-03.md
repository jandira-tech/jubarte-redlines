@coderabbitai plan against `src/edit.rs` (`apply_plan`, `Transaction::resolve`, `check_conflicts`, `EditReport`), `src/bin/jubarte.rs` (`run_edit`, `write_edited`), `src/cli.rs` (`EditOptions`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 3: `--skip-failed`, the `next:` line and the output digest

**Stack.** Step E3 of the jubarte 0.13 contract-redlining plan; branch `feat/edit-skip-failed`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 3.

**Problem.** A plan is all-or-nothing; agents dry-ran a 30-minute plan
nine times and wrote once at 60.3 minutes (brief items 6–7). A clean
dry-run prints the report and nothing else (`run_edit`,
`src/bin/jubarte.rs:953-955`).

**Design (§A-detail 3, A).**

- `edit FILE --plan P --skip-failed`: every op that passes is applied;
  the failed ones are left out; exit 0 when at least one landed; the
  report marks each op `ok` / `skipped`; stderr: `skipped 2 of 14
  operations; the fixes are in P.failed.json`. `P.failed.json` is a
  complete plan (same `author`, `date`, `existing_revisions`;
  `source_sha256` of the new output) holding only the failed ops, each
  with a `why` field the parser ignores.
- Optional `"group": "cap"` on any op: a group stands or falls together
  under `--skip-failed`. An op with a `comment` is one unit already.
- `resolve_revisions` errors and `STALE_SOURCE` are never skipped; a plan
  where every op failed exits 3 and writes nothing.
- After a clean `--dry-run`: stderr `next: jubarte edit contract.docx
  --plan plan.json -o contract.docx --force` (the same argv without
  `--dry-run`; `--quiet` drops it).
- After every write: stderr `sha256: <hex>` and `output_sha256` in the
  report's `summary` event.

Why op granularity is safe: `check_conflicts` (`src/edit.rs:3988`)
already refuses overlapping ranges, double deletes and dependent targets,
so survivors are pairwise independent; `resolve_revisions` is applied as
one unit before them.

**Files.** `src/edit.rs` (`EditPlan` 64: `group` in the common key list
of `check_operation_keys` ~1407; `EditOutcome.status` already knows
`skipped` 865; `EditReport` 947: `output_sha256: Option<String>`,
`skipped: usize`; `apply_plan` 1565 → `apply_plan_with(source, plan,
&ApplyOptions { skip_failed })`; `Transaction::resolve` collects per-op
failures instead of returning the first; `check_conflicts` runs on the
survivors; a new `EditResult.failed_plan: Option<EditPlan>`),
`src/bin/jubarte.rs` (`EditJob` + `run_edit` 946: write
`P.failed.json`, the `skipped` line, the `next:` line, the `sha256:`
line after `write_edited`), `src/cli.rs` (`EditOptions.skip_failed`,
help), `docs/MARKDOWN.md` / `capabilities`, `tests/edit_skip_failed.rs`,
`tests/cli_parser.rs`.

**Steps.**

- [ ] Red: `skip_failed_writes_the_survivors_byte_for_byte` — a 5-op
      plan with op 3 bad (`ANCHOR_NOT_FOUND`) under `--skip-failed`
      produces the same bytes as the 4-op plan.
- [ ] Red: `failed_plan_applies_cleanly_on_the_output` — `P.failed.json`
      carries the new sha256 and the bad op with `why`; fixed, it applies.
- [ ] Red: `a_group_falls_together`, `resolve_revisions_failure_is_never_skipped`,
      `all_failed_exits_3_and_writes_nothing`, `stale_source_is_never_skipped`.
- [ ] Red: `dry_run_prints_the_next_command` (stderr line; absent with
      `--quiet`; absent when the dry-run failed) and
      `every_write_prints_the_sha256` (stderr and `summary` event).
- [ ] Property: for random disjoint op sets, `apply(skip_failed(P))`
      then `apply(P.failed fixed)` equals `apply(P fixed)`.
- [ ] Implement; docs; `capabilities` lists `group` and `--skip-failed`.
- [ ] fmt, clippy, `cargo nextest run --test edit_skip_failed --test
      cli_parser`, `--help` smoke; commit.

**Acceptance.** Default stays atomic; `--skip-failed` lands survivors;
`P.failed.json` is a runnable plan; the two stderr lines appear exactly
as specified; `report.ok` is false whenever an op was skipped.

**Depends on.** Nothing in the stack; placed after plan 2 so
`strike`/`restore` are covered by the group logic.

**Related.** B2, brief items 6–7.

---
