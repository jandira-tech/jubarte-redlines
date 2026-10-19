@coderabbitai plan against `src/edit.rs` (`build_paragraph` 6224, `emits_directly` 2069, `apply_plan` 1565-1580, `preview_plan`), `src/edit/tracked.rs` (`result`, `emit`, `emits_all`), the styles part's `w:next`; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 15: `insert_paragraph` with `style` copies nothing else; structural ops emitted directly

**Stack.** Step E9 of the jubarte 0.13 contract-redlining plan; branch `fix/structural-direct-emission`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 15.

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
