@coderabbitai plan against `src/edit/tracked.rs`, `src/edit.rs` (`check_range`, `check_insert_position`) on `main` (59d8d96c) and on branch `fix/edit-inside-ins-1136`; the `Step_1`/`Step_2` fixtures pin Word's shape; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 1: edits inside another author's insertion (land #1136)

**Stack.** Step E1 of the jubarte 0.13 contract-redlining plan; branch `fix/edit-inside-ins-1136`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 1.

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
