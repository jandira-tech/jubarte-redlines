@coderabbitai plan against `src/validate.rs` (`Finding`, `fails`, `findings_text`, `audit_tracked`), `src/cli.rs` (`Validate`), `src/bin/jubarte.rs` (`run_validate`, the three `warning:` print sites); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 6: `validate --baseline`, and warnings printed once

**Stack.** Step E4 of the jubarte 0.13 contract-redlining plan; branch `feat/validate-baseline`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 6.

**Problem.** `COMMENT_METADATA_MISMATCH` is in the inputs; the skill
demands `no findings`; `read`/`changes`/`comments` repeat the input's
warnings on every call (brief items 9–10); `validate --original INPUT
--author US` on an untouched input reports 518 `FOREIGN_AUTHOR` (#1192, fixed on the stack by PR #1197).

**Design (§A-detail 6, A).**

- `validate OUT --baseline IN`: validate both; match findings by identity
  `(code, part, entity id or paragraph id, message with ids normalised)`;
  print `new:` findings first, then `already in the input: N (not
  yours)`; exit 2 only for new findings; `--json` keeps one finding per
  line with `"status": "new" | "baseline"`, and a trailing `resolved`
  line listing baseline findings the output no longer has. `--rules
  edits --baseline IN` treats every revision the baseline holds as not
  foreign (calls PR #1197's `check_foreign_revisions` with IN as the
  original; no second matcher).
- `read`, `changes`, `comments`: the input's warnings collapse to one
  stderr line `input has 2 validity warnings; jubarte validate FILE lists
  them`; `--quiet` drops it.
- Plain `validate`'s exit code on warnings is #1150's decision; this plan
  does not change it.

**Files.** `src/validate.rs` (`Finding` 87: `fn identity(&self) ->
FindingKey`; new `pub fn baseline(new: Vec<Finding>, base: Vec<Finding>)
-> Baselined { new, baseline, resolved }`; `findings_text` 359 variant;
`fails` 345 over `new` only), `src/cli.rs` (`Validate` ~1205-1265:
`baseline: Option<PathBuf>`, conflicts with `--repair`), `src/bin/jubarte.rs`
(`ValidateJob`, `run_validate` 2320; warning sites 863, 1308, 1999 → one
counter per run), `tests/validate_baseline.rs` (new), `tests/validate_cli.rs`,
`tests/agent_text_view.rs`.

**Steps.**

- [ ] Red: `baseline_hides_the_inputs_findings` (input with a
      `COMMENT_METADATA_MISMATCH`; output edited by jubarte: `new: none`,
      exit 0), `baseline_reports_a_new_finding_even_at_the_same_count`
      (one baseline finding gone, one new: exit 2, both named),
      `baseline_json_marks_status`, `edits_rule_under_baseline_ignores_inherited_revisions`.
- [ ] Red: `read_prints_warnings_once` (stderr has one `input has N
      validity warnings` line), `quiet_drops_it`; same for `changes` and
      `comments`.
- [ ] Implement identity, baseline diff, CLI, the single warning line.
- [ ] fmt, clippy, `cargo nextest run --test validate_baseline --test
      validate_cli --test agent_text_view`, `jubarte validate --help`;
      commit on `feat/validate-baseline`.

**Acceptance.** `validate out.docx --baseline in.docx` on every bench
input/output pair prints `new: none` for the inherited findings; a
finding counts as new only by identity, never by count.

**Related.** #1192, #1150, B4, brief items 9–10.

---
