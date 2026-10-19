@coderabbitai plan against `src/edit/tracked.rs` (`emit`'s `InsertParagraph` arm) and `src/edit.rs` (`EditOutcome.message`); Word experiment E3 decides the placement; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 19: a new paragraph beside a pending-deleted mark

**Stack.** Step E9 of the jubarte 0.13 contract-redlining plan; branch `fix/paragraph-beside-deleted-mark`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 19.

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
