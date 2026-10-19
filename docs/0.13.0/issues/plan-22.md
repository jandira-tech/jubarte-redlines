@coderabbitai plan against `src/cli.rs` (the `help` hoisting at 355, the subcommand enum), `src/capabilities.rs` (352, 466), `src/validate.rs` (codes table 18-70), `src/edit.rs` (`EditError`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 22: help that fits a clock

**Stack.** Step E10 of the jubarte 0.13 contract-redlining plan; branch `feat/help-plan-errors`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 22.

**Problem.** The 63 KB help-all is the entry point; 21–25 minutes to the
first edit is mostly reading (brief item 23).

**Design (§A-detail 22).** `jubarte help plan` prints the plan schema
from `capabilities` (`src/capabilities.rs:352`) with one example per
kind, under 6 KB; `jubarte help errors` lists every code
(`src/validate.rs:18-70` and the edit codes) with one line and the usual
fix; `jubarte help TASK` stays clap's. The help-all stays as the
reference; the skill (plan 23) stops pointing at it first.

**Files.** `src/cli.rs` (clap's `help` subcommand is hoisted at 355;
add a `Help { topic: Option<String> }` subcommand with
`disable_help_subcommand`, delegating unknown topics to clap's
`find_subcommand(..).print_help()`), `src/capabilities.rs` (examples per
kind beside the field table from plan 8), `src/edit.rs` (`EditError`
1085: a `pub const CODES: &[(&str, &str)]` with the one-line fix),
`src/bin/jubarte.rs`, `tests/cli_help_plan.rs` (new), `tests/cli_parser.rs`.

**Steps.**

- [ ] Red: `help_plan_is_under_6kb_and_shows_every_kind`,
      `help_errors_lists_every_code_once` (the test enumerates the
      validator table and `EditError::CODES`), `help_task_is_claps`,
      `quiet_help_still_prints`.
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test cli_help_plan --test
      cli_parser`, `jubarte --help`, `jubarte help plan | wc -c`;
      commit on `feat/help-plan-errors`.

**Acceptance.** `jubarte help plan` prints one valid example per kind
that `edit --dry-run` accepts (the test runs each); every code in a
refusal has a line in `help errors`.

**Depends on.** Plan 8 (one field table), plan 2 (new kinds).

**Related.** brief item 23.

---
