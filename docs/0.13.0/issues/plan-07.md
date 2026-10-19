@coderabbitai plan against `src/edit/tracked.rs` (`emit_text`, `insertion_split_boundary` on the plan-1 branch), `src/edit/controls.rs` (`RANGE_ENDS`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 7: markers stay beside a new insertion (#1156)

**Stack.** Step E4 of the jubarte 0.13 contract-redlining plan; branch `fix/markers-beside-insertion-1156`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 7.

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
