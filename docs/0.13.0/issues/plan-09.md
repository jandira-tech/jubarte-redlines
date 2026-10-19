@coderabbitai plan against `src/changes.rs`, `src/inspect.rs` (`Paragraph.style`), `src/cli.rs` (`Changes`, `ReadArgs`), `src/bin/jubarte.rs` (`run_changes`); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 9: summary-first reading

**Stack.** Step E10 of the jubarte 0.13 contract-redlining plan; branch `feat/summary-reads`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 9.

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
