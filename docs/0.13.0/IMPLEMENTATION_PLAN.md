# jubarte 0.13: implementation plan for contract redlining

Date: 2026-10-19. Source checkout `~/temp/T/1_jubarte-redlines` at `59d8d96c`
(origin/main `b86fe5ac`); line numbers below are from that commit unless a
branch is named. Inputs: `QUESTION_FOR_LARGE_MODEL.md` (the brief, items
1–23), `PROPOSALS_PLAN.md` (the chosen alternative per change, §A–§E) and
`AGENT_VIEW_BY_ALTERNATIVE.md` (what the agent sees). One plan per change,
numbered as in `PROPOSALS_PLAN.md` §A; the stack order is §E's.

## Goal

An agent redlines a counterparty's draft in place, as Word would: it edits
inside their insertions, strikes what it rejects visibly, lands work as it
goes, and reads a 1,000-revision contract from two screens. Every output
opens in Word without a prompt, and Accept All / Reject All / reject-ours
give the right text for every author.

## Rules for the implementer

- Branch per plan, stacked on the previous plan's branch in the order of
  the table below; one PR per branch; never merge unless Arthur says so.
- Red first: every plan starts with a failing test that quotes the exact
  error text or XML shape; implement; run the named tests; then
  `cargo fmt`, `cargo clippy --all-targets -- -D warnings` (no
  `#[allow]`), `cargo nextest run --test NAME`, and `jubarte --help`.
- Cargo runs through the mbx shim, one Cargo process per checkout.
- Word is the oracle. Every new XML shape is opened in Word alone, under a
  fresh name, normal / Accept All / Reject All, through
  `scripts/word_pdf.py` from the bench root; a shape Word refuses becomes a
  Ring-1 check in `src/validate.rs` with a broken probe in
  `tests/m_validity_ring1.rs` before the fix lands. A Word behaviour that
  is worse than ours is recorded in `docs/WORD_DIFFERENCES.md`.
- Fixtures: the `Step_0_Clean.docx`, `Step_1_Tracking.docx`,
  `Step_2_With_Tracking_Against_Tracking.docx` and
  `Step_2_Without_Tracking_Against_Tracking.docx` files now untracked at
  the repository root move to `tests/fixtures/redline/step/` with a
  `REUSE.toml` entry (plan 1 does this). Bench case-folder inputs stay out
  of the repository; synthesize the shape from the `Step_2` family instead.
- Docs land with the code: `docs/MARKDOWN.md`, `docs/DIALECT.md`,
  `docs/WORD_DIFFERENCES.md`, the `edit --help` text and
  `capabilities` when a field or code is added. No model identifiers in
  commits, PR text or issues.
- CodeRabbit reviews one PR per hour; ask for a "full review" once the PR
  is complete, not per push.

## Stack order (PR over PR)

| step | branch | plans | stacks on |
|---|---|---|---|
| E1 | `fix/edit-inside-ins-1136` (exists) | 1, 4 | origin/main |
| E2 | `feat/edit-strike-restore` | 2 | E1 |
| E3 | `feat/edit-skip-failed` | 3 | E2 |
| E4 | `fix/read-option-conflicts-1158`, `feat/validate-baseline`, `fix/markers-beside-insertion-1156` | 5, 6, 7 | E3 |
| E5 | `feat/plan-errors-aliases`, `fix/settled-id-messages` | 8, 18 | E4 |
| E6 | `fix/settle-reports-1159`, `fix/insert-empty-paragraph`, `fix/stable-comment-ids` | 10, 13, 14 | E5 |
| E7 | `feat/one-change-notion`, `fix/view-attribution-1111` | 11, 17 | E6 |
| E8 | `fix/replace-keeps-formatting` | 12 | E7 |
| E9 | `fix/structural-direct-emission`, `fix/numbering-side-effects`, `fix/paragraph-beside-deleted-mark`, `fix/changes-tabs-b12` | 15, 16, 19, 20 | E8 |
| E10 | `feat/summary-reads`, `feat/help-plan-errors`; bench: skill template | 9, 22, 23 | E9 |
| E11 | `feat/compare-three-way` | 21 | E10 |

Word experiments E1–E7 (`PROPOSALS_PLAN.md` §D) gate the shapes they
name; each plan says which one it needs. Run them first, from the bench
root, and keep the files beside the `Step_2` fixtures.

---

## Plan 1: edits inside another author's insertion (land #1136)

**Problem.** 111 of 146 refusals in 0.12.0 are
`UNSUPPORTED_STRUCTURE: the text sits inside rN, a tracked insertion by
"…"; settle it first` (brief items 1–3). `check_range` /
`opaque_structure` (`src/edit.rs:3863-3960`) and `check_insert_position`
(`src/edit.rs:3965`) treat any `w:ins`, `w:moveTo` or `w:del` ancestor as
opaque; `tracked::check` (`src/edit/tracked.rs:87-110`) refuses
`delete_paragraph` of a paragraph holding any revision under `keep`.

**Branch.** `fix/edit-inside-ins-1136` exists (commits `c7cddbc1`,
`23c11a3d`); it writes Word's nested shape and adds
`insertion_split_boundary` (`src/edit/tracked.rs:1150` on the branch) and
`tests/edit_inside_insertion.rs`. This plan finishes it.

**Design (PROPOSALS_PLAN §A-detail 1, alternative A).** No new flag or
field. Under `existing_revisions: keep`:

- a range inside another author's `w:ins`: the deleted words stay inside
  their insertion as our `w:del`; our words are a sibling `w:ins` after it;
  the remainder of their insertion is a new `w:ins` with their author and
  date and a new `w:id` (the `Step_2` fixture pins this; one `w:id` on
  two elements would trip `DUPLICATE_REVISION_ID`);
- our own pending insertion is edited in place, no nested mark;
- `delete_paragraph` of a paragraph holding revisions: plain runs get our
  `w:del`; their `w:ins` runs get a nested `w:del`; their `w:del` runs stay;
  our own `w:ins` runs are removed outright; the mark gets `w:del` in
  `w:pPr/w:rPr` (beside their `w:ins` when the mark is theirs, pending E1);
- comments and bookmarks in the range stay where they are; a field is
  deleted whole or the existing field-crossing refusal fires;
- still refused: inside their `w:del` (plan 4's message), inside
  `w:moveTo`/`w:moveFrom` until E2, a field result inside their insertion.

**Files.** `src/edit/tracked.rs` (`check` 87, `emit_text` 735,
`isolate_range` 829, `delete_runs` 926, `wrap_deleted` 952,
`holds_revision` 984, `mark` 1015, `clean_rpr` 1038), `src/edit.rs`
(`check_range` 3863, `opaque_structure` ~3900, `check_insert_position`
3965, `paragraph_ids` in `EditReport` 947), `tests/edit_inside_insertion.rs`,
`tests/edit_keep_revisions.rs`, `tests/edit_opaque_refusals.rs`,
`docs/MARKDOWN.md` (the body writer's nesting note, ~line 575),
`REUSE.toml`.

**Steps.**

- [ ] Rebase `fix/edit-inside-ins-1136` on `origin/main` (`b86fe5ac`);
      run `cargo nextest run --test edit_inside_insertion --test
      edit_keep_revisions --test edit_opaque_refusals`; all green before
      any new work.
- [ ] Move the four `Step_*.docx` files to `tests/fixtures/redline/step/`,
      add the `REUSE.toml` entry, run `uv tool run --from
      'reuse[charset-normalizer]' reuse lint`.
- [ ] Red: `own_insertion_is_edited_in_place` — author A inserts
      "bankers", the same author replaces it with "insurers" under `keep`;
      expect the `w:ins` text changed, no nested `w:del`, one `w:ins`.
- [ ] Red: `delete_paragraph_under_keep_nests_in_their_insertion` on the
      `Step_1_Tracking.docx` paragraph that holds A's insertion: expect
      plain runs in our `w:del`, A's `w:ins` runs with a nested `w:del`,
      the mark's `w:rPr` carrying our `w:del`.
- [ ] Red: `edit_inside_their_deletion_is_refused` expecting the plan-4
      message (write the assertion against the message plan 4 defines, and
      land plan 4 in this PR).
- [ ] Red: `edit_inside_move_is_refused_until_probed` (keeps today's
      refusal for `w:moveTo`/`w:moveFrom`, message names the move id).
- [ ] Red: `report_lists_the_split_pieces` — after a split, the report's
      summary names the new `w:id`s under the source id (`r3 -> r3+6`),
      as `paragraph_ids` names moved paragraphs today.
- [ ] Implement: lift the `w:ins` ancestor refusal in `check_range` and
      `check_insert_position` when the plan keeps revisions; route the
      insertion-ancestor case through `emit_text`'s nested path; add the
      in-place path for the plan author's own insertion; extend
      `tracked::check`'s `DeleteParagraph` arm to emit instead of refuse;
      add the split listing to the report.
- [ ] Property test on the step family: for every edit, rejecting our
      author gives the source's text and the other author's `changes`
      listing unchanged; `accept_all(output)` equals `accept_all` of the
      same edit on the plain text.
- [ ] Word: open each new output (own-insertion edit, paragraph delete
      with their mark) normal / Accept All / Reject All; run E1 for the
      mark shape; record the result in the test name or
      `docs/WORD_DIFFERENCES.md`.
- [ ] `cargo fmt`, clippy `-D warnings`, the three test files, `jubarte
      --help`; commit; mark the PR ready after Word opened every shape.

**Acceptance.** The `UNSUPPORTED_STRUCTURE … tracked insertion` refusal
no longer fires for any range inside a `w:ins`; the two `Step_2` fixtures
are reproduced byte-for-byte in element order and attributes except
`w:date`; `jubarte validate` on every output is clean; `changes` after
the edit lists our `w:del` as `inside rN`.

**Depends on.** E1 (mark shape) for the paragraph-delete case only; ship
the text cases first if E1 is late and keep that one refusal.

**Related.** #1136, #1134 (nested `w:ins` inside `w:del` uses
`w:delText`), #1111, brief items 1–3.

---

## Plan 2: `strike` and `restore` (visible rejects)

**Problem.** `resolve_revisions.reject` is Word's Reject: the
counterparty's insertion vanishes, their deletion silently comes back.
Ten delivered documents show the right text with no trace of the
decision (brief items 4–5, `C-invisible-settle/`).

**Design (§A-detail 2, A).** Two operation kinds, by our author:

```json
{"kind": "strike",  "revision": "r366", "comment": "Not acceptable: parents are not parties."}
{"kind": "restore", "revision": "r12",  "comment": "This carve-out was agreed in round 1."}
```

CLI: `jubarte edit FILE -p r366 --strike [--content "comment"]`,
`-p r12 --restore`; `--reject --visible` picks strike or restore by the
revision's kind. JSON: `resolve_revisions.reject.visible: true` does the
same for every id in that list. `resolve_revisions.reject` without
`visible` is unchanged.

OOXML. `strike` of their insertion: our `w:del` (new `w:id`, our author,
our date) wrapping the runs inside their `w:ins`, text as `w:delText`.
`restore` of their deletion: their `w:del` untouched; our `w:ins` right
after it holding a copy of the runs with `w:delText` → `w:t` and the same
`w:rPr`. Moves: `strike` of a `w:moveTo` nests `w:del` inside it after E2;
`restore` of a `w:moveFrom` is refused (restoring moved text duplicates
it). Marks: an inserted mark inside the struck range gets `w:del` beside
their `w:ins` in `w:rPr` (E1); a deleted mark to restore is a new paragraph
whose mark carries our `w:ins` (E4). Rows: `w:trPr/w:ins` by them gets
`w:trPr/w:del` by us. Fields: struck whole. Bookmarks, comments: unmoved.

Refusals (exact text): `OWN_REVISION: r40 is by "Reviewing Counsel
(AgentCo)"; delete the text instead`; `EMPTY_REVISION: r13 holds no text,
only comment c4's mark; reject r13 to keep c4`; a revision in both
`resolve_revisions` and a strike/restore op: existing
`REVISION_CONFLICT`; `strike`/`restore` in a plan whose
`existing_revisions` is `accept` or `reject`: `INVALID_PLAN: strike
needs existing_revisions keep (the revision would already be settled)`.
A strike of an insertion that already holds our nested deletion of the
same text: `ok` with `message: "already struck"`, not an error.

**Files.** `src/edit.rs` (`OperationKind` 171 + `check_operation_keys`
allowed lists ~1440; `Resolved` 1718: add `Strike { revision, comment }`
and `Restore { revision, comment }`; resolution in the `OperationKind`
match ~3030; `check_conflicts` 3988 for the `REVISION_CONFLICT` case;
`EditOutcome.context` `{-struck "…"}` / `{+restored "…"}`),
`src/edit/tracked.rs` (`check` 87, `emit` 278: new arms reusing
`wrap_deleted` 952 with the `w:ins` as container, `mark` 1015, `clean_rpr`
1038), `src/edit/flags.rs` (`FlagOp`: `strike`, `restore`, `visible`;
parsing 235-260), `src/cli.rs` (`EditOptions` flags and help),
`src/capabilities.rs` (kinds list), `docs/MARKDOWN.md`, `tests/edit_strike_restore.rs`,
`tests/cli_parser.rs`, `tests/m_validity_ring1.rs` (probe: nested
`w:del` with `w:t` is Word-invalid per #1134; make sure `ring1` already
fires, else add it).

**Steps.**

- [ ] Red: `strike_nests_our_deletion_inside_their_insertion` on
      `Step_1_Tracking.docx`: XML equals the shape above modulo dates.
- [ ] Red: `restore_copies_their_deleted_runs_as_our_insertion_after_it`
      (bold `w:rPr` preserved; their `w:del` untouched byte-for-byte).
- [ ] Red: the four refusal tests with the exact messages, and
      `already_struck_is_ok_with_a_note`.
- [ ] Red: `reject_visible_picks_by_kind` (CLI parse and JSON).
- [ ] Red: properties on the step family: (1) rejecting our author on
      `strike(x)` leaves the other author's elements identical to `x`;
      (2) `accept_all(strike(ins))` text equals `accept_all(reject(ins))`;
      (3) `accept_all(restore(del))` text equals `accept_all(reject(del))`;
      (4) `jubarte validate` clean on every output.
- [ ] Implement kinds, resolution, conflicts, emission, flags, help,
      capabilities, docs.
- [ ] Word: open strike and restore outputs normal / Accept All / Reject
      All / reject only our author / accept only their author (five
      renders each); run E2 and E4 before enabling moves and marks, else
      refuse those with the exact alternative.
- [ ] fmt, clippy, `cargo nextest run --test edit_strike_restore --test
      cli_parser --test m_validity_ring1`, `--help` smoke; commit.

**Acceptance.** Both JSON kinds and both flags work on text revisions;
`changes` on the output lists our deletion `inside r366` and our
insertion after `r12`; Word's four readings in §A-detail 2 hold on the
fixtures; the hint of plan 4 offers `restore` first.

**Depends on.** Plan 1 (nested emission), E1/E2/E4 for marks and moves.

**Related.** #1108 (hint), #1134, brief items 4–5, B1.

---

## Plan 3: `--skip-failed`, the `next:` line and the output digest

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

## Plan 4: refusal hints that name both decisions and the visible route (#1108)

**Problem.** The hint names accept first, says nothing of the cost,
quotes v1 syntax to v2 plans, never mentions the visible route.

**Design (§A-detail 4, A).** After plan 1 the hint fires only for
deletions, moves and fields. Template, one message, four routes:

```
UNSUPPORTED_STRUCTURE (op-2): the text is inside r12, text "Trainer 1 - LargeCo" deleted.
  put it back visibly:  {"kind":"restore","revision":"r12"}            (their deletion stays, your insertion shows)
  let it go:            "resolve_revisions":{"accept":{"ids":["r12"]}} (their words leave the document for good)
  put it back silently: "resolve_revisions":{"reject":{"ids":["r12"]}} (their deletion disappears, no trace)
  or comment on it:     {"kind":"comment","paragraph":"p160","find":"…","text":"…"}
```

For an insertion that remains opaque (moves until E2) the first line is
`strike`. The fragments come from one `PlanSyntax` enum (v1 today; the v2
branches add their variant) so the message never quotes a syntax the
build does not accept.

**Files.** `src/edit.rs` (`opaque_structure` ~3900 and the message built
in `check_range`; new `fn settle_hint(kind, id, author, text, paragraph,
syntax: PlanSyntax) -> String`), `src/edit/tracked.rs` (`check` 87 uses
the same builder), `tests/edit_opaque_refusals.rs` (update the quoted
text), `tests/edit_refusal_hints.rs` (new), `docs/MARKDOWN.md` (the
errors section).

**Steps.**

- [ ] Red: `deletion_hint_offers_restore_first_then_both_settles_then_comment`
      asserting the exact four-line text on a `Step_1` deletion.
- [ ] Red: `move_hint_offers_strike_first`; `hint_quotes_the_active_plan_syntax`
      (one variant now; the test enumerates `PlanSyntax`).
- [ ] Implement the builder; replace every call site that builds the old
      `settle it first` text (grep `settle it first`).
- [ ] fmt, clippy, `cargo nextest run --test edit_opaque_refusals --test
      edit_refusal_hints`; commit on `fix/edit-inside-ins-1136`.

**Acceptance.** No refusal text says `settle it first`; every
`UNSUPPORTED_STRUCTURE` for a revision names the revision, its author,
its text, and the four routes with their consequence in parentheses.

**Related.** #1108, B1.

---

## Plan 5: option conflicts, and JSON-shaped usage errors (#1158)

**Problem.** `read --no-page-markers --json` was refused 165 times in 78
trials; `--tables -p t0` 9 times; `--track-changes accepted --json` 7
(brief item 8). The refusal is a clap usage error on stderr, so `2>&1 |
jq` sees a parse error and the agent retries blind.

**Design (§A-detail 5, A).**

- `--no-page-markers` with `--json` or `--tables`: accepted, no-op,
  byte-identical output; help says `ignored with --json and --tables`.
- `--tables -p t0`: selects the table (closes #1161; `--tables` and
  `--json` number tables the same way, #1173).
- `--track-changes accepted|rejected` with `--json`: works; each
  paragraph's `text` is the settled view and `summary.track_changes`
  records the choice.
- With `--json` on any command, a usage or runtime error prints on stdout
  as one line `{"error":{"code":"USAGE","message":"…","usage":"jubarte
  read [OPTIONS] <FILE>"}}` and the same text on stderr; exit code
  unchanged.
- Audit every `conflicts_with` in `src/cli.rs` (86, 94, 104, 126, 135,
  168, 186): keep only pairs whose combination is ambiguous (`-p` with
  `--head`/`--tail` stays; `--json` with `--comments`/`--dates` becomes a
  no-op with a help note).

**Files.** `src/cli.rs` (`ReadArgs` 50-137; `read_output`/`compare_output`
~2458-2560; the clap error handler in `src/bin/jubarte.rs` `main`:
detect `--json` in argv before parsing and print the object),
`src/inspect.rs` (`Summary` 94: `track_changes: Option<String>`),
`tests/cli_parser.rs`, `tests/agent_text_view.rs`,
`tests/read_options_json.rs` (new).

**Steps.**

- [ ] Red: `no_page_markers_with_json_is_byte_identical`,
      `tables_with_paragraph_selects_the_table`,
      `track_changes_accepted_with_json_settles_text`,
      `usage_error_with_json_is_an_object_on_stdout` (exit 2 kept),
      `help_says_ignored_with_json`.
- [ ] Implement the clap changes, the snapshot's settled text, the error
      object; update `--help` lines.
- [ ] fmt, clippy, the three test files, `jubarte read --help` smoke;
      commit on `fix/read-option-conflicts-1158`.

**Acceptance.** No clap conflict remains for the three combinations;
`jubarte read x.docx --json --bogus | jq .error.code` prints `"USAGE"`.

**Related.** #1158, #1161, #1173, brief item 8.

---

## Plan 6: `validate --baseline`, and warnings printed once

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

## Plan 7: markers stay beside a new insertion (#1156)

**Problem.** An `insert` at a paragraph edge wraps the adjacent
`bookmarkEnd` (and, at the start, body-level markers) inside the new
`w:ins`; Accept All then extends the bookmark over the new text and the
endpoints no longer match the source (#827's contract). The brief's code
name `BOOKMARK_CROSSES_REVISION` does not exist in 0.12.0; the shape does.

**Design (§A-detail 7, A).** `insertion_split_boundary`
(`src/edit/tracked.rs:1150` on the plan-1 branch) already states the
rule: opening markers travel with the words to their right, closing
markers with the words to their left, body-level markers stay at body
level. Apply it to the edge-insert path for `bookmarkStart/End`,
`commentRangeStart/End`, `permStart/End` (share the `RANGE_ENDS` list at
`src/edit/controls.rs:560` with its `RANGE_STARTS` twin). Source
position wins (#1156 choice A): the bookmark does not grow over the new
text.

**Files.** `src/edit/tracked.rs` (`emit_text` 735: the attach position
for an insertion at offset 0 or at the end; `insertion_split_boundary`),
`src/edit/controls.rs` (export the marker name lists),
`tests/edit_bookmark_preservation.rs` (new; every row of #1156's table),
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: one test per row of #1156's table (insert at start / end /
      beside `bookmarkStart` / beside `bookmarkEnd` / body-level markers /
      comment range ends / permission range ends): the marker is a
      sibling of the new `w:ins`, never its child.
- [ ] Red: property — for any insert, the multiset of (marker, offset in
      source text) is unchanged.
- [ ] Implement with the boundary rule; keep the comparer path's
      behaviour pinned by the same tests (run both modes).
- [ ] Word: open the start-of-paragraph and end-of-paragraph shapes;
      Accept All; check the bookmark's extent in Word's bookmark dialog.
- [ ] fmt, clippy, `cargo nextest run --test edit_bookmark_preservation`;
      commit on `fix/markers-beside-insertion-1156`.

**Acceptance.** No `w:ins` written by `insert` contains a range marker
it did not create; #827's endpoint test passes on every row.

**Related.** #1156, #827, brief item 11.

---

## Plan 8: plan schema errors that teach, and three aliases

**Problem.** `INVALID_PLAN unknown field "comment"` on `delete`;
`comment` instead of `text` on comment ops; missing `schema_version`;
trailing characters; no valid fields listed (brief item 12).
`EditPlan::from_json` (`src/edit.rs:1379`) maps serde's text straight
through; `check_operation_keys` (1405) knows the allowed keys per kind
but prints none.

**Design (§A-detail 8, A).**

- `INVALID_PLAN (op-4, reply_comment): unknown field "comment"; did you
  mean "text"? fields: comment_id, text` — op index, kind, offending
  field, the allowed list, the nearest by edit distance (a 20-line
  Levenshtein in `src/edit.rs`, no new dependency).
- Missing `schema_version`: `INVALID_PLAN: schema_version is required (1
  for this build)`. Trailing characters: `INVALID_PLAN: the plan must be
  exactly one JSON object; characters follow it at line 6 column 58`.
  Unknown kind: list the kinds. `ids` as a string: say it is an array.
- Aliases, each adding a `note:` line in the report (`EditReport.notes:
  Vec<String>`, printed in the `summary` event and on stderr):
  1. `comment` on `delete`, `rewrite`, `strike`, `restore`: a real
     field — a comment anchored to the deleted (or rewritten) text.
     `Resolved::Text` already carries `comment`; the emitter anchors the
     range around the `w:del`.
  2. `comment` → `text` on `comment` and `reply_comment`.
  3. `anchor` → `find` on `replace`, `delete`, `comment`.
- Stay errors: `text` on `replace`, `position` with `after`, numeric
  `paragraph`, unknown `kind`, `ids` as a string.

**Files.** `src/edit.rs` (`from_json` 1379, `check_operation_keys` 1405
and the allowed table ~1440, `OperationKind::Delete`/`Rewrite` gain
`comment`, `EditReport.notes`), `src/edit/tracked.rs` (comment range
around a deletion), `src/capabilities.rs` (fields per kind come from the
same table), `tests/edit_plan_errors.rs` (new), `tests/edit_plan.rs`,
`tests/edit_structured_comments.rs`, `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: one test per message above, asserting the exact text.
- [ ] Red: `comment_on_delete_anchors_to_the_deleted_text` (XML:
      `commentRangeStart`, our `w:del`, `commentRangeEnd`, reference run);
      `alias_text_on_comment_op_applies_with_a_note`;
      `alias_anchor_applies_with_a_note`; `text_on_replace_stays_an_error`.
- [ ] Implement; make `capabilities` and `check_operation_keys` read one
      table so help and errors cannot drift.
- [ ] Word: open the comment-on-deletion output; the balloon sits on the
      struck text.
- [ ] fmt, clippy, `cargo nextest run --test edit_plan_errors --test
      edit_plan --test edit_structured_comments`; commit on
      `feat/plan-errors-aliases`.

**Acceptance.** Every `INVALID_PLAN` for a field names the kind, the
field, the valid fields and the nearest one; the three aliases apply
with a note; the five listed errors stay errors.

**Related.** B6, brief item 12.

---

## Plan 9: summary-first reading

**Problem.** Agents page `read` five to ten times on turn 2–4 documents;
median first edit at 21–25 minutes (brief item 13). `read --changed
--by` exists but prints whole paragraphs with marks.

**Design (§A-detail 9, A).** Three read-only views, deterministic, each
with `--json`:

- `changes --summary`: counts by author × kind, then by section (nearest
  heading above, with its paragraph range): `§8 Limitation of Liability
  (p150-p168): LargeCo 12 ins, 4 del, 3 comments; ids r401-r428`.
- `read --outline`: headings with paragraph ranges and per-section counts
  of pending changes and open threads, nothing else.
- `changes --brief [--by A] [--section "8"]`: one line per logical change
  (plan 11's grouping): `r401+402 ins p152 §8.1 "and each of their
  respective parents…" (62 chars)`.
- Sections come from heading styles (`Heading1`–`Heading6` or the
  `styles:` legend) or outline levels; without them, one section `body`.

**Files.** `src/cli.rs` (`Changes { file, json }` 652: add `summary`,
`brief`, `by`, `section`; `ReadArgs`: `outline`), `src/changes.rs`
(`pub fn summarize(changes, sections) -> ChangeSummary`; `brief_text`),
`src/inspect.rs` (`Paragraph.style` → `pub fn outline(snapshot) ->
Vec<Section { heading, range, changes, threads }>`), `src/bin/jubarte.rs`
(`run_changes` 399), `tests/changes_summary.rs` (new),
`tests/agent_text_view.rs`, `docs/MARKDOWN.md`, `docs/DIALECT.md`
(one line each), bench skill (plan 23).

**Steps.**

- [ ] Red: `summary_groups_by_author_kind_and_section` on a fixture with
      three headings and two authors; `summary_without_headings_is_one_body_section`;
      `outline_lists_headings_ranges_and_counts`; `brief_is_one_line_per_logical_change`
      (uses plan 11's `r7+8+9+10`); each with a `--json` twin.
- [ ] Implement; `--summary`, `--brief` and `--outline` are text by
      default, JSON objects with `--json`.
- [ ] fmt, clippy, `cargo nextest run --test changes_summary --test
      agent_text_view`, `jubarte changes --help`; commit on
      `feat/summary-reads`.

**Acceptance.** On a 1,000-revision fixture `changes --summary` and
`read --outline` each fit in 80 lines; every id range they print is
accepted by `read -p` and `changes --brief --section`.

**Depends on.** Plan 11 (grouping).

**Related.** B5, brief item 13.

---

## Plan 10: report what a settle removes; replies in tracked-deleted text (#1159)

**Problem.** Accepting r13 (a deletion holding only c4's reference mark)
removes c4 and its reply c5, as Word does. On the stack, PR #1146
performs that removal (replies included) and warns, and PR #1191 reports
every removed thread on the edit paths (`removed_comments: [{"id","author",
"replies"}]`), corrects the `UNKNOWN_COMMENT` advice and adds the opt-in
plan field `removed_anchor_comments: "keep"`. What the stack lacks:
`changes` does not say that r13 holds c4's mark; the view hides the
mark; a reply to a comment whose paragraph the plan deletes is refused
as `OVERLAPPING_EDITS` (`src/edit.rs:4047-4056`), the wrong code (brief
item 14; mid-session item 10).

**Design (§A-detail 10, A, cut down to what PRs #1146 and #1191 lack).**

- `changes` prints `r13 deletion (holds comment c4's mark)`; the view
  shows the mark inside the deletion.
- The report keeps PR #1191's shape and adds `why`:
  `removed_comments: [{"id":"c4","author":"…","replies":["c5"],"why":
  "anchor removed by accepting r13"}]`.
- Reply to a comment in a paragraph the plan deletes: allowed when the
  deletion is tracked (the reference stays in deleted text; Word shows
  the thread); refused with `COMMENT_ANCHOR_DELETED` only in editing
  mode.
- Do not re-implement the removal, the warning, the `UNKNOWN_COMMENT`
  advice or the keep field: they are PR #1191's. Branch from
  `fix/cmt-keep-1159` (#1191) or from `main` once it lands.

**Files.** `src/edit.rs` (`check_conflicts` 4047-4056; the `why` field
on PR #1191's `removed_comments`), `src/changes.rs` (`Change.text` for a
mark-only deletion), `src/markdown/from_docx/critic.rs` (mark inside a
deletion), `tests/edit_settle_reports.rs` (new),
`tests/edit_comment_threads.rs` (PR #1191's; extend, do not rewrite),
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `changes_names_the_mark_a_deletion_holds`,
      `view_shows_the_mark_inside_the_deletion`,
      `removed_comments_carry_why` (JSON exactly as above),
      `reply_in_a_tracked_deleted_paragraph_is_allowed`,
      `reply_in_an_editing_mode_deleted_paragraph_is_refused_with_comment_anchor_deleted`.
- [ ] Implement; `jubarte validate` must stay clean on every output;
      PR #1191's tests stay green unchanged.
- [ ] Word: open the reply-in-deleted-text output; the thread shows on
      the struck text.
- [ ] fmt, clippy, `cargo nextest run --test edit_settle_reports --test
      edit_comment_threads`; commit on `fix/settle-reports-1159`.

**Acceptance.** `changes` names every mark a deletion holds; no reply
refusal uses `OVERLAPPING_EDITS` for a deleted anchor; every
`removed_comments` entry says why.

**Depends on.** PR #1191 (the report and the keep field).

**Related.** #1159, PRs #1146 and #1191, brief item 14, mid-session item 10.

---

## Plan 11: one notion of a change

**Problem.** `read` shows `r7+8+9+10`; `changes` lists four; rejecting
r7 alone leaves the rest pending. `-p r87+r88` is refused with a message
that does not show `r87+88` (brief item 15).

**Design (§A-detail 11, A).** `changes` groups adjacent carriers of one
author, date, kind and story into one logical change with the composite
`ref` (`r7+8+9+10`) and lists `members` in `--json`; `-p r87+r88` is
accepted as a spelling of `r87+88`; the refusal's hint shows the
composite it saw in the file. After plan 1 splits an insertion, the new
pieces join their group. A group spanning a paragraph mark is one id.
`accept`/`reject` of a composite resolves every member (already true for
`r1+2`; the test pins it).

**Files.** `src/changes.rs` (`changes_in` and `carriers` 441: group
adjacent carriers; `Change` 55: `members: Vec<String>`),
`src/ids.rs` (`parse_change_ids` 232 and the `bare` check at 294: allow
`r` after `+`; the hint prints the canonical composite),
`src/markdown/from_docx/critic.rs` (the view already joins; share the
grouping predicate so both sides agree), `tests/changes_grouping.rs`
(new), `tests/edit_short_ids.rs`, `docs/DIALECT.md`, `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `changes_lists_one_logical_change_for_adjacent_same_author_runs`
      (`r7+8+9+10 insertion p23 … (4 elements)`; JSON `members`),
      `read_and_changes_agree_on_every_group` (property on the fixtures:
      the set of composite ids the view prints equals `changes`'),
      `r87_plus_r88_is_accepted`, `refusal_hint_shows_the_composite_in_the_file`,
      `accept_of_a_composite_resolves_every_member`,
      `a_group_across_a_paragraph_mark_is_one_id`.
- [ ] Implement with one grouping function used by `changes` and the
      view.
- [ ] fmt, clippy, `cargo nextest run --test changes_grouping --test
      edit_short_ids --test agent_text_view`; commit on
      `feat/one-change-notion`.

**Acceptance.** No fixture yields a composite in `read` that `changes`
does not list, or the reverse; `jubarte accept FILE -p r7+8+9+10` leaves
nothing of that insertion pending.

**Related.** brief item 15; the 17 October id contract (§4) keeps the
`+` form.

---

## Plan 12: `replace` keeps the formatting it keeps; lenient anchors keep the document's characters

**Problem.** `replace` resolves the whole `find` range as one edit
(`apply_text_edit`, `src/edit.rs:5555`) and the emitter writes one new
run in the first run's formatting (`emit_text`,
`src/edit/tracked.rs:735`), so every kept word in the range is deleted
and re-typed plain; the comparer path records that as a `w:rPrChange`
(five defined terms lost bold in four trials). Only `rewrite` runs the
word-level diff today (`rewrite_ranges`, called once at
`src/edit.rs:3463`). A straight quote in `--anchor` matches a curly one
and the straight quote in `--content` then replaces it, seven changes
for a one-word edit (brief items 16 and 21; mid-session items 14 and
22).

**Design (§A-detail 12, A).** The word-level diff inside
`replace`/`rewrite` runs on the normalised text, but emission takes equal
segments from the source runs (split at the diff boundaries, original
`w:rPr`, original characters) and only inserted segments are new runs,
formatted like the character before them. `whole: true` keeps today's
re-typing. `units()` (`src/edit/rewrite.rs:28`) keys each word through
the anchor module's `reads_as` table (`src/edit/anchor.rs:12`:
look-alike quotes, hyphens and spaces read as the plain character,
zero-width characters as nothing), so a curly quote in the document and
a straight one in `content` compare equal and the document's character
stays; `anchor_read_as` (`EditOutcome` 865) already records the match.
Documented consequence: `replace` cannot turn a curly quote into a
straight one; `whole: true` does.

**Files.** `src/edit/rewrite.rs` (`rewrite_ranges` 72,
`split_around_kept` 117), `src/edit/tracked.rs` (`emit_text` 735,
`isolate_range` 829 splits runs at offsets, `clean_rpr` 1038),
`src/edit/anchor.rs` (`reads_as` becomes `pub(super)`),
`tests/edit_replace_formatting.rs` (new), `tests/edit_anchor_normalization.rs`,
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `bold_defined_term_keeps_bold_and_no_rpr_change` —
      `find: '"Affiliates" means any', content: '"Affiliates" means each'`
      on a paragraph whose `“Affiliates”` is bold: XML has one `w:del`
      ("any") and one `w:ins` ("each"), the bold run untouched, no
      `w:rPrChange`; the view prints `{~~any~>each~~}` only.
- [ ] Red: `straight_quotes_in_anchor_and_content_do_not_touch_curly_quotes`
      (the p6 case: one change, not seven), `whole_true_keeps_retyping`,
      `run_split_at_diff_boundary_keeps_each_side_rpr`,
      `replace_cannot_restyle_a_quote_whole_can`.
- [ ] Red: property on the fixtures — for every `replace` whose diff
      touches one word, the output's `w:rPrChange` count equals the
      source's.
- [ ] Implement; keep the comparer path's output pinned where it is
      used (`whole`).
- [ ] Word: open the defined-term output; Accept All keeps bold.
- [ ] fmt, clippy, `cargo nextest run --test edit_replace_formatting
      --test edit_anchor_normalization`; commit on
      `fix/replace-keeps-formatting`.

**Acceptance.** No `replace` with a one-word diff produces a
`w:rPrChange` or touches a character outside the diff; `anchor_read_as`
differences never appear as changes.

**Depends on.** Plan 1 (emission path), plan 11 (grouping of the pieces).

**Related.** brief items 16, 21; mid-session items 14, 22.

---

## Plan 13: insert into an empty paragraph

**Problem.** `check_insert_position` (`src/edit.rs:3970`) refuses
`cannot insert text into an empty paragraph; use insert_paragraph`
(brief item 17's neighbour, repro B-series). Word types into an empty
paragraph by creating a run with the mark's run properties.

**Design (§A-detail 13, A).** New `w:r` with a copy of `w:pPr/w:rPr`
minus `w:ins`, `w:del`, `w:rPrChange` (`clean_rpr`), wrapped in our
`w:ins`; `w:pPr` byte-identical before and after; works in keep and in
the comparer path.

**Files.** `src/edit.rs` (`check_insert_position` 3965-3972,
`attach_segment`), `src/edit/tracked.rs` (`emit_text` 735: the
empty-projection case), `tests/edit_empty_paragraph.rs` (new),
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `insert_into_an_empty_paragraph_creates_a_run_from_the_marks_rpr`
      (mark `w:rPr` with `w:b`: the run is bold; `w:pPr` unchanged
      byte-for-byte), `insert_into_an_empty_paragraph_without_mark_rpr`,
      `insert_into_an_empty_paragraph_in_a_table_cell`, both modes.
- [ ] Implement; the old message disappears.
- [ ] Word: open; Accept All leaves one plain paragraph with the text.
- [ ] fmt, clippy, `cargo nextest run --test edit_empty_paragraph`;
      commit on `fix/insert-empty-paragraph`.

**Acceptance.** `edit FILE -p p9 --content "text"` on an empty p9
writes `<w:p><w:pPr>…</w:pPr><w:ins><w:r><w:rPr>…</w:rPr><w:t>text</w:t></w:r></w:ins></w:p>`.

**Related.** repro B-series; brief §A item 13.

---

## Plan 14: stable comment ids; no editing another author's comment

**Problem.** A bare `accept`/`reject` renumbers comment ids because
`resolve` (`src/revision_processor.rs:3503-3546`) calls
`annotation_ids::renumber` when no revision is left; a targeted settle
does not; `edit -p c106 --content …` rewrites a counterparty comment
under their name with no warning (brief items 15–16 in the mid-session
additions).

**Design (§A-detail 14, A).** Ids are identifiers: `resolve_package`
callers at 3466, 3481 and `src/changes.rs:385` pass `renumber: false`;
renumbering stays only in `validate --repair` (`repair` 1649) for
duplicate ids. `edit_comment` on a comment whose `w:author` differs from
the plan's author is refused: `COMMENT_NOT_YOURS: c106 is by "Trainer 1
- LargeCo"; reply instead (reply_comment), or pass
"rewrite_others_comments": true to change their words under their name`.
The plan field (`EditPlan.rewrite_others_comments`, default false) and
the CLI flag `--rewrite-others-comments` exist because the brief allows
it on explicit request; never the default. E5 records what the current
Word build does.

**Word parity.** The renumbering is deliberate: commit 48aae202 (rule
R5) copied Word's own Accept All, which numbers bookmarks and comments
with one dense counter in document order, read off 51 files. This plan
deviates from that on purpose: `edit` never renumbers, so jubarte
already disagrees with itself, and a stale handle that lands in the
wrong thread is worse than a non-dense id. Record the deviation in
`docs/WORD_DIFFERENCES.md` (ids are identifiers; Word's dense
renumbering is not copied on accept/reject) and re-pin
`tests/m_accept_word_parity.rs` so ids are compared structurally (the
same annotations in the same order), not literally.
`accept_revisions_package_keeping_ids` (`src/revision_processor.rs`) is
the existing mechanism. Arthur: this reverses R5 of 48aae202; say if
Word-dense ids on accept matter more than stable handles.

**Files.** `src/revision_processor.rs` (3466, 3481, `resolve` 3503),
`src/changes.rs` (385), `src/validate.rs` (`repair` 1649 keeps the
renumbering for `DUPLICATE_*` findings), `src/edit.rs` (`EditPlan` 64,
`check_thread_ops` 4270-4300: author check), `src/edit/flags.rs`,
`src/cli.rs`, `tests/edit_resolved_ids.rs`, `tests/edit_comment_ownership.rs`
(new), `tests/validate_cli.rs`, `tests/m_accept_word_parity.rs` (ids compared
structurally), `docs/MARKDOWN.md`, `docs/WORD_DIFFERENCES.md`.

**Steps.**

- [ ] Red: `accept_all_keeps_comment_ids` (c106 stays c106 after
      `jubarte accept FILE`), `reject_all_keeps_comment_ids`,
      `repair_still_renumbers_duplicates`,
      `editing_another_authors_comment_is_refused` (exact text),
      `rewrite_others_comments_true_allows_it_with_a_note`.
- [ ] Implement.
- [ ] Re-pin `tests/m_accept_word_parity.rs` on ids (structural, not
      literal), one explanation per test in the commit message; add the
      `docs/WORD_DIFFERENCES.md` entry.
- [ ] Word: E5 (can B edit A's comment; what `w:author` carries after).
- [ ] fmt, clippy, `cargo nextest run --test edit_resolved_ids --test
      edit_comment_ownership --test validate_cli`; commit on
      `fix/stable-comment-ids`.

**Acceptance.** A reply written after a bare accept lands in the thread
the agent read; `edit_comment` by a different author exits 3 unless the
flag is set.

**Related.** brief items 15–16 (added set).

---

## Plan 15: `insert_paragraph` with `style` copies nothing else; structural ops emitted directly

**Problem.** `build_paragraph` (`src/edit.rs:6224`) clones the anchor's
`w:pPr` (dropping only `sectPr`, `pPrChange`, `rPr`) and the dominant
run's `w:rPr` even when `style` is given, so `style: "Normal"` after a
numbered heading inherits `numPr` and bold and becomes "4." (brief item
18). On the comparer path, `delete_paragraph` and `insert_paragraph` are
diffed character by character across the boundary, so a leading `“`
hops paragraphs (brief item 19).

**Design (§A-detail 15, A).** `style` given and no `like`: `w:pPr` holds
`pStyle` only and runs take the style's formatting (empty `w:rPr`);
`like` given: copy `like`'s `pPr` (today's drop list) and dominant
`rPr`; neither: the style's next style (`w:style/w:next` in the styles part;
Normal for Word's headings), as Word's Enter does, with today's anchor
copy only when the style names no `w:next`. Nothing in `src/edit.rs`
reads `w:next` today (`src/append.rs:810` and `src/debug.rs:1013` do);
add a `next_style` reader over the styles part. Routing: `emits_directly`
(`src/edit.rs:2069`) widens to "every op the emitter supports"
(`tracked::emits_all` 64, i.e. all but `fill_control`, `insert_footnote`,
`insert_image`, `page_setup`, and `whole` rewrites), in every
`existing_revisions` mode; the settled base is built first as today, then
the emitter writes the tracked changes on it. The comparer stays for
`whole` rewrites and the four unsupported kinds. Alternative B (keep the
comparer, pin paragraph boundaries as hard anchors) is rejected: it fixes
one symptom and keeps the diff granularity mismatch between modes.

**Consequences of widening direct emission.** Today only `keep` uses
the emitter; the default mode is `refuse`, so a document with no tracked
changes (turn 1, the most common case) is edited on a clean copy and the
redline comes from running the comparer over the whole document with
`detect_moves` on (`WmlComparerSettings::default()`). Widening changes
the default path, not an edge:

- Output shape changes for turn-1 edits. The emitter writes exactly the
  found range as `w:del` and the content as `w:ins`, as Word records a
  typed edit. The comparer re-diffs the whole document at word level: it
  can split or merge marks differently, add `w:rPrChange` on kept text,
  number ids differently, and turn a `delete_paragraph` plus
  `insert_paragraph` of the same text into a `w:moveFrom`/`w:moveTo`
  pair. Accept All and Reject All give the same text either way; Word's
  review pane shows fewer, more precise marks.
- Comparer-path defects go away as a side effect: #1160 (permission
  ranges dropped, a bookmark start moved out of an empty paragraph), the
  quote hop of brief item 19, and the 17 phantom `numId` changes of repro
  B11 (the comparer's numbering merge).
- Pinned tests: `tests/edit_plan.rs` holds 58 tests; most assert texts,
  codes and Word validity and survive. Fewer than ten assert
  comparer-shaped details (run-formatting inheritance, the "survive
  compare" comment test, the cross-run Unicode replacement, exact
  revision counts) and are re-pinned with one explanation each.
- What stays on the comparer: `whole: true` rewrites, the four kinds the
  emitter cannot write, and plans with `update_fields` (`keep` refuses
  it today, `check_update_fields` at `src/edit.rs:1655`, because a
  refreshed clean copy would no longer be the accepted redline). The rule
  is therefore "direct unless the plan needs the comparer".
- The real cost: a plan with one `whole` op and nine ordinary ops goes
  wholly through the comparer, so the output's shape depends on whether
  such an op is present. The report says which path ran
  (`summary.emission: "direct" | "compare"`).
- Risk: emitter gaps the comparer masked now surface on turn-1 plans.
  Mitigation: the whole edit suite runs under both routings before the
  switch, and a hidden `--emit compare` flag forces the old path for one
  release.
- Speed: no whole-document LCS for ordinary plans.

Alternative B lost because hard paragraph anchors in the comparer fix
only the quote hop: two engines keep producing two shapes for one op,
#1160 and B11 stay, and move detection keeps rewriting a delete and an
insert into a move.

**Files.** `src/edit.rs` (`build_paragraph` 6224 and a `next_style` reader over the
styles part; `emits_directly` 2069;
`apply_plan` 1565-1580 and `preview_plan` 1686), `src/edit/tracked.rs`
(`result` 170, `emit` 278: `InsertParagraph`, `DeleteParagraph`,
`MergeParagraphs` arms already exist for keep),
`tests/edit_structural_direct.rs` (new), `tests/edit_keep_revisions.rs`,
`tests/edit_plan.rs` (pinned outputs may change: review each diff),
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `style_means_style_only` (after a numbered heading, the new
      paragraph has `pStyle` Normal, no `numPr`, no bold; the next
      article keeps its number), `like_copies_like_not_anchor`,
      `no_style_no_like_follows_the_next_style` (after a Heading 1 whose
      `w:next` is Normal: `pStyle` Normal, no `numPr`, no bold),
      `no_style_no_like_copies_anchor_when_the_style_names_no_next`.
- [ ] Red: `delete_paragraph_on_a_clean_document_is_one_tracked_deletion`
      (no `“` hop: the next paragraph unchanged byte-for-byte),
      `insert_paragraph_before_a_definition_leaves_the_quote_where_it_was`,
      `partial_reject_of_our_new_paragraph_reads_right`.
- [ ] Red: property — for every plan without `whole` or an unsupported
      kind, `accept_all(output)` text and `reject_all(output)` text are
      the same under keep and under accept mode.
- [ ] Red: `update_fields_plans_still_take_the_comparer`,
      `whole_rewrite_plans_still_take_the_comparer`,
      `report_names_the_emission_path`, `emit_compare_flag_forces_the_old_path`.
- [ ] Run the whole edit suite (`cargo nextest run -E 'test(edit_)'`)
      with the routing forced each way; list every test whose output
      differs before changing any of them.
- [ ] Implement the `pPr` rule and the routing ("direct unless the plan
      needs the comparer"); re-pin any `tests/edit_plan.rs` output that
      legitimately changes and explain each in the commit message.
- [ ] Word: open the "4." fixture output and the definition case; Accept
      All / Reject All.
- [ ] fmt, clippy, `cargo nextest run --test edit_structural_direct --test
      edit_keep_revisions --test edit_plan`; commit on
      `fix/structural-direct-emission`.

**Acceptance.** `style` never carries numbering or run formatting from
the neighbour; a paragraph with neither `style` nor `like` follows the
anchor style's next style; a structural op never produces a change in a paragraph
the plan did not name.

**Depends on.** Plan 1 (emitter coverage under keep), plan 12 (text
emission).

**Related.** brief items 18–19; mid-session items 18, 19.

---

## Plan 16: numbering side effects

**Problem.** An edit adds `w:num` entries with duplicate
`w16cid:durableId` (three delivered documents); after an accept-all, a
single `replace` re-pointed 17 untouched list paragraphs from `numId` 13
to 25 as tracked formatting changes (brief item 20, repro B11).

**Design (§A-detail 16, A).** A cloned `w:num` gets a fresh
`w16cid:durableId` (random 32-bit, unique in the part) or none; the
editor's own `List` op (`src/markdown/xml.rs:2154 num()`) writes none
today and stays so. The duplicates come from the comparer path copying
B's definitions (`copy_missing_numbering`, `src/comparer/footnotes.rs:364`,
and the numbering "park" passes in `src/comparer/finalize.rs`); after
plan 15 that path is reached by `whole` rewrites and the four
unsupported kinds only, and by `compare`. Rule for the comparer: when the
source and the revised document hold the same `w:num` (same `numId`,
same `abstractNumId`, same overrides), map to the source's id and copy
nothing; an untouched paragraph never changes `numId`. A Ring-1 check
`DUPLICATE_DURABLE_ID` lists every repeated id (severity set by E6:
error if Word repairs or merges lists, warning if it opens cleanly).

**Files.** `src/comparer/footnotes.rs` (`copy_missing_numbering`),
`src/comparer/finalize.rs` (the pass that re-points `numId`; find it
with `jubarte debug diff` on the B11 input/output), `src/validate.rs`
(codes table 18-70, `ring1` 445), `tests/edit_numbering_side_effects.rs`
(new), `tests/m_validity_ring1.rs` (broken probe), `docs/WORD_DIFFERENCES.md`
(E6's result).

**Steps.**

- [ ] Word first: E6 (two `w:num` with one `durableId`): open / repair /
      merge; set the check's severity from the result.
- [ ] Red: `ring1_flags_duplicate_durable_ids` (probe with two equal
      ids), `cloned_num_gets_a_fresh_durable_id`,
      `replace_after_accept_all_does_not_repoint_untouched_paragraphs`
      (B11: `changes` lists 1 insertion, 1 deletion, 0 formatting),
      `compare_keeps_shared_numbering_by_identity`.
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test edit_numbering_side_effects
      --test m_validity_ring1`; commit on `fix/numbering-side-effects`.

**Acceptance.** `jubarte validate` on every bench output reports no
`DUPLICATE_DURABLE_ID`; the B11 repro lists no formatting change.

**Depends on.** Plan 15 (narrows the comparer path); E6.

**Related.** brief item 20.

---

## Plan 17: view attribution (#1111)

**Problem.** s2-t2-g03c p47: XML order is LargeCo `w:ins` "limited, ",
our `w:del` "non-exclusive", our `w:ins` "nonexclusive"; the view prints
`{~~non-exclusive~>limited, ~~}{>>r524 @AL<<}{>>r141 @TX<<}{++nonexclusive++}`,
fusing our deletion with their insertion and reversing the order.
s1-t3-g03a p8: a counterparty deletion inside a multi-run insertion
prints `{--applicable--}{>>r16 @TX2<<}` without ` on r13` (brief item 17).

**Design (§A-detail 17, A).** `render` (`src/markdown/from_docx/critic.rs:836-860`)
pairs an insertion followed by a deletion into a substitution whenever
neither names an outer change; add `same_author(new_by, old_by)` (187)
to the guard, so a pair by two authors stays two changes in document
order. Always print ` on rN` for a change nested in another, whatever
the outer change's remaining text (`fold_on` 611 attaches the outer only
when the inner is the sole content; `tree` 474 must carry the outer id
on every nested span). Doc change: `docs/DIALECT.md` line 69
(`{~~a~>b~~}{>>r1 @AC<<}{>>r2 @JD<<}` "substitution by two authors")
becomes a read-only form: the reader still accepts it, the view no
longer prints it. Arthur: this changes a line of the settled dialect;
say if you want the two-author pair kept printable.

**What the agent sees.** Ids and handles are illustrative; the shapes
are the two bench cases. `TL` is the counterparty, `RC` is us.

Case A, adjacent changes by two authors (s2-t2-g03c p47). Source: "a
non-exclusive licence". The counterparty inserted "limited, " in front
of it; we then replaced "non-exclusive" with "nonexclusive". XML order:
their `w:ins` "limited, ", our `w:del` "non-exclusive", our `w:ins`
"nonexclusive". Before (0.12.0):

```
a {~~non-exclusive~>limited, ~~}{>>r141 @RC<<}{>>r524 @TL<<}{++nonexclusive++}{>>r142 @RC<<} licence
```

The view says "non-exclusive became limited, ", then "nonexclusive" was
added: document order reversed, a replacement nobody made, two authors
on one mark. To strike the counterparty's "limited, " the agent must
know that a substitution's second tag is its insertion side; agents got
this wrong, and `changes` lists the three in document order, so the two
outputs disagree. After:

```
a {++limited, ++}{>>r524 @TL<<}{--non-exclusive--}{>>r141 @RC<<}{++nonexclusive++}{>>r142 @RC<<} licence
```

Document order, one author per mark. Our own two marks keep pairing as
today when the view joins them (`{~~non-exclusive~>nonexclusive~~}{>>r141+142 @RC<<}`
with plan 11's grouping); same-author pairs are untouched.

Case B, lost nesting (s1-t3-g03a p8). We inserted "the applicable law of
England"; the counterparty deleted "applicable" inside it. Before:

```
{++the ++}{>>r13 @RC<<}{--applicable--}{>>r16 @TL<<}{++law of England++}{>>r13 @RC<<}
```

Nothing says the deletion sits inside our proposal; `changes` reports
`r16 … inside r13`, the view does not, so the agent cannot see that the
counterparty rejected part of what we wrote. After:

```
{++the ++}{>>r13 @RC<<}{--applicable--}{>>r16 @TL on r13<<}{++law of England++}{>>r13 @RC<<}
```

In the settled `re` dialect the note reads `{r16 @TL re r13 @RC}`.

The alternative considered and rejected: pairing across authors only
when the deletion precedes the insertion would print "they struck 30
days, we proposed 45 days" as `{~~30 days~>45 days~~}{>>r50 @TL<<}{>>r60 @RC<<}`.
The order is right, but two acts still read as one replacement, and
today's code never pairs a deletion-then-insertion at all, so that
option adds a pairing instead of removing one. Line 69 of
`docs/DIALECT.md` stays a form the reader accepts; the view stops
producing it.

**Files.** `src/markdown/from_docx/critic.rs` (`same_author` 187,
`tree` 474, `unnest` 572, `fold_on` 611, `render` 836),
`src/markdown/from_docx/agent.rs` (`join_tags` 245),
`tests/agent_view_attribution.rs` (new; fixtures synthesized from the
two cases), `tests/agent_text_view.rs`, `docs/DIALECT.md`,
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `two_authors_never_pair_as_a_substitution` (expected:
      `{++limited, ++}{>>r524 @AL<<}{--non-exclusive--}{>>r141 @TX<<}{++nonexclusive++}{>>r142 @TX<<}`,
      with the plan-11 grouping where it applies),
      `same_author_pair_still_prints_a_substitution`,
      `nested_deletion_always_names_its_outer_insertion` (`{--applicable--}{>>r16 @TX2 re r13 @AC<<}`
      in the `re` dialect, or ` on r13` in the current one: match the
      dialect the branch prints), `reader_still_accepts_a_two_author_pair`.
- [ ] Implement; coordinate with `fix/nested-on-1111` (take its tests).
- [ ] fmt, clippy, `cargo nextest run --test agent_view_attribution
      --test agent_text_view`; commit on `fix/view-attribution-1111`.

**Acceptance.** For every fixture, each change the view prints names the
author `changes` lists for it, and every `inside rN` in `changes` has a
nesting note in the view.

**Related.** #1111, #1135, brief item 17.

---

## Plan 18: settled-id messages, and resolving through the mapping

**Problem.** `PARAGRAPH_RESOLVED: p5 merged into p5 when the plan's
revisions were settled; …` (`src/edit/origin.rs:420-426`) names the
source id and the settled id with the same text, and refuses even when
the mapping is one-to-one (brief item 22; mid-session item 18).

**Design (§A-detail 18, A).** Resolve whole-paragraph ops through
`fates` (`src/edit/origin.rs:462`) when the settled paragraph has exactly
one source (`merged_ok` already exists as a parameter: compute it from
the multiplicity of `post` instead of the caller's flag); refuse only the
many-to-one case, naming both ids with their view: `PARAGRAPH_RESOLVED:
p5 (as read) merges into p4 (after the plan's revisions settle) together
with p6; give a find, or run read --track-changes accepted to address the
settled paragraph`. `Fate::Gone` keeps its message. Plan 22's `help
errors` documents the code.

**Files.** `src/edit/origin.rs` (415-432, `moves` 437, `fates` 462),
`src/edit.rs` (callers passing `merged_ok`), `tests/edit_resolved_ids.rs`,
`tests/edit_settled_ids.rs` (new), `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `one_to_one_merge_resolves_without_a_refusal` (p5's mark is
      a pending deletion by them; `delete_paragraph p5` under
      `existing_revisions: accept` applies to the merged paragraph),
      `many_to_one_merge_is_refused_naming_both_ids_and_views` (exact
      text), `gone_paragraph_keeps_its_message`.
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test edit_resolved_ids --test
      edit_settled_ids`; commit on `fix/settled-id-messages`.

**Acceptance.** No `PARAGRAPH_RESOLVED` message repeats one id twice;
one-to-one merges never refuse.

**Related.** brief item 22.

---

## Plan 19: a new paragraph beside a pending-deleted mark

**Problem.** `insert_paragraph` after p3 whose mark is a pending
deletion by the counterparty: Accept All merges the new paragraph into
p3 (mid-session item 21). Probably Word-faithful; the surprise is the
silence.

**Design (§A-detail 19, A).** Run E3 first (A deletes p3's mark; B
presses Enter at the end of p3 and types): place the new paragraph where
Word's Enter puts it, and report in the op's `message`: `p3's paragraph
mark is a pending deletion by Trainer 1 (r88); if it is accepted, p3 and
the new paragraph merge; restore r88 to keep them apart`. Do not restore
the mark unasked.

**Files.** `src/edit/tracked.rs` (`emit` 278: the `InsertParagraph`
arm; detect `w:pPr/w:rPr/w:del` on the anchor's mark),
`src/edit.rs` (`EditOutcome.message` on an `ok` op),
`tests/edit_paragraph_beside_deleted_mark.rs` (new),
`docs/WORD_DIFFERENCES.md` (E3's result, whichever way it goes).

**Steps.**

- [ ] Word: E3, three renders; keep the files beside `Step_2`.
- [ ] Red: `insert_after_a_pending_deleted_mark_matches_word_and_reports`
      (placement per E3; the exact message), `accept_all_reads_as_word_does`,
      `restore_of_the_mark_keeps_them_apart` (uses plan 2's `restore` on a
      mark, so gated on E4; refuse with the exact alternative until then).
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test edit_paragraph_beside_deleted_mark`;
      commit on `fix/paragraph-beside-deleted-mark`.

**Acceptance.** The op succeeds, the report says what Accept All will
do, and the output's Accept All text equals Word's for E3.

**Depends on.** E3; plan 2 + E4 for the restore line.

**Related.** brief mid-session item 21.

---

## Plan 20: small fixes from the repro (tabs in `changes --json`)

**Problem.** B12: `changes --json` prints `"text":"Title: "` for a run
that holds `Title<w:tab/>` — the tab is dropped in `Change.text`, not
only in the preview (brief item 22's neighbour; mid-session item 23).
The other half this plan carried, `validate --rules edits --original IN
--author US` flagging the original's own revisions as `FOREIGN_AUTHOR`
(#1192, 518 on an untouched input), is fixed on the stack by PR #1197:
`check_foreign_revisions` aligns the edited file's revisions with the
original's in document order, after the matcher reviews #1200 and
#1204. This plan no longer touches `audit_tracked`; plan 6's
`--baseline` calls PR #1197's matcher.

**Design (§A-detail 20).** `Change.text` maps `w:tab` to `\t` and
`w:br` to `\n` (the `changes_text` preview keeps `{:?}` quoting, so a tab
prints as `\t` there).

**Files.** `src/changes.rs` (text collection in `changes_in`/`carriers`
441; `changes_text` 262), `tests/changes_json_tabs.rs` (new, one test).

**Steps.**

- [ ] Red: `changes_json_keeps_tabs_and_breaks`.
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test changes_json_tabs`; commit
      on `fix/changes-tabs-b12`.

**Related.** repro B12; #1192 (PR #1197); mid-session item 23.

---

## Plan 21: revision-aware compare (three-way)

**Problem.** Two-way `compare` reads the input with the counterparty's
changes accepted, so their revisions flatten, every unstruck change is
implicitly accepted, and a comment anchored to their deletion loses its
anchor (brief B3; `COMPARE_EXPERIMENT.md`).

**Design (§A-detail 21, A).** A second workflow, not the primary one:
`jubarte compare IN.docx CAND.docx --keep-revisions -o OUT.docx`, built
as diff → plan → `keep` emission:

1. `accepted = accept_revisions(IN)` (`src/document_comparer.rs`, as
   `apply_plan` does at `src/edit.rs:1244`); `compare_documents_with_settings(accepted, CAND)`
   gives our edits as spans in accepted-text coordinates.
2. Each span becomes a plan op (`replace`/`insert`/`delete`,
   `insert_paragraph`/`delete_paragraph`) addressed by settled paragraph
   id and `find` text; ids map back to the source through plan 18's
   `fates`.
3. The plan is applied to IN under `existing_revisions: keep`, which
   after plan 1 nests our deletions in their insertions and lays our
   insertions beside their deletions; their changes keep their author.
4. Comments in CAND not in IN become `comment` ops anchored by text;
   comments in both are kept by id.

Hard cases, refused with the span named: their pending deletion whose
text we retyped (choose `restore` when identical, else refuse), spans
inside their `moveFrom`/`moveTo`, paragraph splits or merges across their
deleted marks, rows they inserted that we removed, fields and
`pPrChange`. Comments anchored entirely in text we deleted stay on the
deleted text.

**Files.** `src/cli.rs` (`CompareArgs` 525: `keep_revisions: bool`),
`src/bin/jubarte.rs` (the compare runner), new `src/edit/synthesize.rs`
(`pub fn plan_from_diff(input, candidate, author, date) ->
Result<EditPlan, EditError>`), `src/edit/origin.rs` (`fates`),
`tests/compare_three_way.rs` (new), `docs/MARKDOWN.md`, `docs/CLI.md` or
the compare `after_help` (648).

**Steps.**

- [ ] Word: E7 (Word's Compare of `Step_1` against `Step_2`) as the
      reference for what Word itself keeps.
- [ ] Red: `three_way_keeps_their_pending_changes` (property on the
      step family: `changes` filtered to the other author is identical
      before and after), `three_way_accept_all_equals_the_candidate_text`,
      `retyped_deletion_becomes_a_restore_when_identical`,
      `edit_inside_their_move_is_refused_naming_the_span`,
      `new_comment_in_candidate_is_anchored_by_text`,
      `comment_in_deleted_text_stays`.
- [ ] Implement `synthesize.rs` on top of `apply_plan`; no new comparer
      mode.
- [ ] Word: open three outputs; Accept All / Reject All.
- [ ] fmt, clippy, `cargo nextest run --test compare_three_way`,
      `jubarte compare --help`; commit on `feat/compare-three-way`.

**Acceptance.** On every case folder the two properties hold and the
output opens in Word; the refusals name the span and the alternative.

**Depends on.** Plans 1, 2, 11, 12, 15, 18.

**Related.** B3, `COMPARE_EXPERIMENT.md`, the C4 memo.

---

## Plan 22: help that fits a clock

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

## Plan 23: skill template (bench side)

**Problem.** The skill teaches "one or two passes", "no findings", reading
the help-all first, and `resolve_revisions` as the answer to the
refusal (brief §C, `instruction.md:117,183`, bench
`.tasks/<task>/environment/skills/contract-redliner/SKILL.md`).

**Design (§A-detail 23).** The template at
`~/temp/T/jubarte-syntax/skill-tools/SKILL.template.md` says: read
`changes --summary` and `read --outline` first; plan a section, dry-run,
apply (`--skip-failed` when the clock is short), then the next section;
`strike`/`restore` for their changes you reject, `resolve_revisions`
only for changes you accept or want gone; `validate OUT --baseline IN`
once; with ten minutes left, apply what you have. Delete "one or two
passes" and "no findings". Not an engine change; tracked here so the
engine and the skill ship together.

**Files.** `~/temp/T/jubarte-syntax/skill-tools/SKILL.template.md`,
the bench's `instruction.md` (117, 183) and the generated SKILL.md; the
bench's verifier tools (memory: `redline-bench-jubarte-deploy`).

**Steps.**

- [ ] Rewrite the template against the 0.13 CLI (plans 2, 3, 6, 9, 22);
      every command in it is run once against a 0.13 build.
- [ ] Regenerate the bench SKILL.md; diff against the old one; the words
      "one or two passes", "no findings" and "help-all" are gone.
- [ ] One bench trial on a turn-2 document to confirm the first edit
      lands before minute 10.

**Depends on.** Plans 2, 3, 6, 9, 22 released.

**Related.** brief §C.

---

## Closing notes

- Three plans change settled text or behaviour that Arthur may want to
  keep: plan 17 stops printing the dialect's two-author substitution;
  plan 15 widens direct emission to every mode and re-pins comparer
  outputs; plan 14 stops renumbering comment ids on a bare settle. Each
  is flagged in its plan and its issue.
- Word experiments gate: E1 (plans 1, 2), E2 (1, 2), E3 (19), E4 (2, 19),
  E5 (14), E6 (16), E7 (21). Run them before the plan that needs them;
  ship the text-only part first when an experiment is late.
- Existing issues this plan references rather than duplicates: #1136,
  #1108, #1158, #1161, #1173, #1156, #827, #1159, #1192, #1150, #1134,
  #1111, #1135.
