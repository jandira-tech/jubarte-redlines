@coderabbitai plan against `src/edit.rs` (`opaque_structure`, the `settle it first` message sites) and `src/edit/tracked.rs` (`check`); `tests/edit_opaque_refusals.rs`; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 4: refusal hints that name both decisions and the visible route (#1108)

**Stack.** Step E1 of the jubarte 0.13 contract-redlining plan; branch `fix/edit-inside-ins-1136`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 4.

**Problem.** The hint names accept first, says nothing of the cost,
quotes v1 syntax to v2 plans, never mentions the visible route.

**Design (§A-detail 4, A).** After plan 1 the hint fires only for
deletions, moves and fields. Template, one message, four routes:

```
UNSUPPORTED_STRUCTURE (op-2): the text is inside r12, text "Trainer 1 - LargeCo" deleted.
  put it back visibly:  {"kind":"restore","revision":"r12"}            (their deletion stays, your insertion shows)
  let it go:            "resolve_revisions":{"accept":{"ids":["r12"]}} (their words leave the document for good)
  put it back silently: "resolve_revisions":{"reject":{"ids":["r12"]}} (their deletion disappears, no trace)
  or comment on it:     {"kind":"comment","paragraph":"p160","find":"…","text":"…"}
```

For an insertion that remains opaque (moves until E2) the first line is
`strike`. The fragments come from one `PlanSyntax` enum (v1 today; the v2
branches add their variant) so the message never quotes a syntax the
build does not accept.

**Files.** `src/edit.rs` (`opaque_structure` ~3900 and the message built
in `check_range`; new `fn settle_hint(kind, id, author, text, paragraph,
syntax: PlanSyntax) -> String`), `src/edit/tracked.rs` (`check` 87 uses
the same builder), `tests/edit_opaque_refusals.rs` (update the quoted
text), `tests/edit_refusal_hints.rs` (new), `docs/MARKDOWN.md` (the
errors section).

**Steps.**

- [ ] Red: `deletion_hint_offers_restore_first_then_both_settles_then_comment`
      asserting the exact four-line text on a `Step_1` deletion.
- [ ] Red: `move_hint_offers_strike_first`; `hint_quotes_the_active_plan_syntax`
      (one variant now; the test enumerates `PlanSyntax`).
- [ ] Implement the builder; replace every call site that builds the old
      `settle it first` text (grep `settle it first`).
- [ ] fmt, clippy, `cargo nextest run --test edit_opaque_refusals --test
      edit_refusal_hints`; commit on `fix/edit-inside-ins-1136`.

**Acceptance.** No refusal text says `settle it first`; every
`UNSUPPORTED_STRUCTURE` for a revision names the revision, its author,
its text, and the four routes with their consequence in parentheses.

**Related.** #1108, B1.

---
