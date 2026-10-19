@coderabbitai plan against `src/edit.rs` (`check_insert_position` 3965-3972, `attach_segment`) and `src/edit/tracked.rs` (`emit_text`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 13: insert into an empty paragraph

**Stack.** Step E6 of the jubarte 0.13 contract-redlining plan; branch `fix/insert-empty-paragraph`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 13.

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
