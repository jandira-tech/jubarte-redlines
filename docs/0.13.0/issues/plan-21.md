@coderabbitai plan against `src/cli.rs` (`CompareArgs` 525), `src/bin/jubarte.rs` (the compare runner), `src/edit.rs` (`apply_plan` under keep), `src/document_comparer.rs` (`accept_revisions`, `compare_documents_with_settings`), `src/edit/origin.rs` (`fates`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 21: revision-aware compare (three-way)

**Stack.** Step E11 of the jubarte 0.13 contract-redlining plan; branch `feat/compare-three-way`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 21.

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
