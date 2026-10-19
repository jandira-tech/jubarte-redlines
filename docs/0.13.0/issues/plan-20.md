@coderabbitai plan against `src/changes.rs` (text collection in `carriers`/`changes_in`, `changes_text` 262); PR #1197 owns the foreign-author audit; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 20: small fixes from the repro (tabs in `changes --json`)

**Stack.** Step E9 of the jubarte 0.13 contract-redlining plan; branch `fix/changes-tabs-b12`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 20.

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
