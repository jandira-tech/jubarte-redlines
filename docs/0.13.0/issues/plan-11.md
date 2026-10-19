@coderabbitai plan against `src/changes.rs` (`carriers`, `changes_in`, `Change`), `src/ids.rs` (`parse_change_ids`, the `bare` check at 294), `src/markdown/from_docx/critic.rs` (the view's joining); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 11: one notion of a change

**Stack.** Step E7 of the jubarte 0.13 contract-redlining plan; branch `feat/one-change-notion`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 11.

**Problem.** `read` shows `r7+8+9+10`; `changes` lists four; rejecting
r7 alone leaves the rest pending. `-p r87+r88` is refused with a message
that does not show `r87+88` (brief item 15).

**Design (§A-detail 11, A).** `changes` groups adjacent carriers of one
author, date, kind and story into one logical change with the composite
`ref` (`r7+8+9+10`) and lists `members` in `--json`; `-p r87+r88` is
accepted as a spelling of `r87+88`; the refusal's hint shows the
composite it saw in the file. After plan 1 splits an insertion, the new
pieces join their group. A group spanning a paragraph mark is one id.
`accept`/`reject` of a composite resolves every member (already true for
`r1+2`; the test pins it).

**Files.** `src/changes.rs` (`changes_in` and `carriers` 441: group
adjacent carriers; `Change` 55: `members: Vec<String>`),
`src/ids.rs` (`parse_change_ids` 232 and the `bare` check at 294: allow
`r` after `+`; the hint prints the canonical composite),
`src/markdown/from_docx/critic.rs` (the view already joins; share the
grouping predicate so both sides agree), `tests/changes_grouping.rs`
(new), `tests/edit_short_ids.rs`, `docs/DIALECT.md`, `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `changes_lists_one_logical_change_for_adjacent_same_author_runs`
      (`r7+8+9+10 insertion p23 … (4 elements)`; JSON `members`),
      `read_and_changes_agree_on_every_group` (property on the fixtures:
      the set of composite ids the view prints equals `changes`'),
      `r87_plus_r88_is_accepted`, `refusal_hint_shows_the_composite_in_the_file`,
      `accept_of_a_composite_resolves_every_member`,
      `a_group_across_a_paragraph_mark_is_one_id`.
- [ ] Implement with one grouping function used by `changes` and the
      view.
- [ ] fmt, clippy, `cargo nextest run --test changes_grouping --test
      edit_short_ids --test agent_text_view`; commit on
      `feat/one-change-notion`.

**Acceptance.** No fixture yields a composite in `read` that `changes`
does not list, or the reverse; `jubarte accept FILE -p r7+8+9+10` leaves
nothing of that insertion pending.

**Related.** brief item 15; the 17 October id contract (§4) keeps the
`+` form.

---
