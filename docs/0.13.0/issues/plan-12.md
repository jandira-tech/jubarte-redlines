@coderabbitai plan against `src/edit/rewrite.rs` (`units`, `rewrite_ranges`, `split_around_kept`), `src/edit/tracked.rs` (`emit_text`, `isolate_range`, `clean_rpr`), `src/edit/anchor.rs` (`reads_as`, `reading_hits`), `src/edit.rs` (`apply_text_edit` 5555); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 12: `replace` keeps the formatting it keeps; lenient anchors keep the document's characters

**Stack.** Step E8 of the jubarte 0.13 contract-redlining plan; branch `fix/replace-keeps-formatting`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 12.

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
