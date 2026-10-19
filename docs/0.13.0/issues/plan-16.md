@coderabbitai plan against `src/comparer/footnotes.rs` (`copy_missing_numbering`), `src/comparer/finalize.rs` (the numbering passes), `src/validate.rs` (codes table, `ring1`), `src/markdown/xml.rs` (`num`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 16: numbering side effects

**Stack.** Step E9 of the jubarte 0.13 contract-redlining plan; branch `fix/numbering-side-effects`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 16.

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
