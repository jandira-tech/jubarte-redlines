@coderabbitai plan against `src/edit/origin.rs` (415-432, `moves`, `fates` 462) and its `merged_ok` callers in `src/edit.rs`; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 18: settled-id messages, and resolving through the mapping

**Stack.** Step E5 of the jubarte 0.13 contract-redlining plan; branch `fix/settled-id-messages`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 18.

**Problem.** `PARAGRAPH_RESOLVED: p5 merged into p5 when the plan's
revisions were settled; …` (`src/edit/origin.rs:420-426`) names the
source id and the settled id with the same text, and refuses even when
the mapping is one-to-one (brief item 22; mid-session item 18).

**Design (§A-detail 18, A).** Resolve whole-paragraph ops through
`fates` (`src/edit/origin.rs:462`) when the settled paragraph has exactly
one source (`merged_ok` already exists as a parameter: compute it from
the multiplicity of `post` instead of the caller's flag); refuse only the
many-to-one case, naming both ids with their view: `PARAGRAPH_RESOLVED:
p5 (as read) merges into p4 (after the plan's revisions settle) together
with p6; give a find, or run read --track-changes accepted to address the
settled paragraph`. `Fate::Gone` keeps its message. Plan 22's `help
errors` documents the code.

**Files.** `src/edit/origin.rs` (415-432, `moves` 437, `fates` 462),
`src/edit.rs` (callers passing `merged_ok`), `tests/edit_resolved_ids.rs`,
`tests/edit_settled_ids.rs` (new), `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `one_to_one_merge_resolves_without_a_refusal` (p5's mark is
      a pending deletion by them; `delete_paragraph p5` under
      `existing_revisions: accept` applies to the merged paragraph),
      `many_to_one_merge_is_refused_naming_both_ids_and_views` (exact
      text), `gone_paragraph_keeps_its_message`.
- [ ] Implement.
- [ ] fmt, clippy, `cargo nextest run --test edit_resolved_ids --test
      edit_settled_ids`; commit on `fix/settled-id-messages`.

**Acceptance.** No `PARAGRAPH_RESOLVED` message repeats one id twice;
one-to-one merges never refuse.

**Related.** brief item 22.

---
