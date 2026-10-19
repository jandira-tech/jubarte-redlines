@coderabbitai plan against `src/edit.rs` (`EditPlan::from_json`, `check_operation_keys` and its allowed-keys table, `OperationKind::Delete`/`Rewrite`), `src/capabilities.rs`; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 8: plan schema errors that teach, and three aliases

**Stack.** Step E5 of the jubarte 0.13 contract-redlining plan; branch `feat/plan-errors-aliases`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 8.

**Problem.** `INVALID_PLAN unknown field "comment"` on `delete`;
`comment` instead of `text` on comment ops; missing `schema_version`;
trailing characters; no valid fields listed (brief item 12).
`EditPlan::from_json` (`src/edit.rs:1379`) maps serde's text straight
through; `check_operation_keys` (1405) knows the allowed keys per kind
but prints none.

**Design (§A-detail 8, A).**

- `INVALID_PLAN (op-4, reply_comment): unknown field "comment"; did you
  mean "text"? fields: comment_id, text` — op index, kind, offending
  field, the allowed list, the nearest by edit distance (a 20-line
  Levenshtein in `src/edit.rs`, no new dependency).
- Missing `schema_version`: `INVALID_PLAN: schema_version is required (1
  for this build)`. Trailing characters: `INVALID_PLAN: the plan must be
  exactly one JSON object; characters follow it at line 6 column 58`.
  Unknown kind: list the kinds. `ids` as a string: say it is an array.
- Aliases, each adding a `note:` line in the report (`EditReport.notes:
  Vec<String>`, printed in the `summary` event and on stderr):
  1. `comment` on `delete`, `rewrite`, `strike`, `restore`: a real
     field — a comment anchored to the deleted (or rewritten) text.
     `Resolved::Text` already carries `comment`; the emitter anchors the
     range around the `w:del`.
  2. `comment` → `text` on `comment` and `reply_comment`.
  3. `anchor` → `find` on `replace`, `delete`, `comment`.
- Stay errors: `text` on `replace`, `position` with `after`, numeric
  `paragraph`, unknown `kind`, `ids` as a string.

**Files.** `src/edit.rs` (`from_json` 1379, `check_operation_keys` 1405
and the allowed table ~1440, `OperationKind::Delete`/`Rewrite` gain
`comment`, `EditReport.notes`), `src/edit/tracked.rs` (comment range
around a deletion), `src/capabilities.rs` (fields per kind come from the
same table), `tests/edit_plan_errors.rs` (new), `tests/edit_plan.rs`,
`tests/edit_structured_comments.rs`, `docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: one test per message above, asserting the exact text.
- [ ] Red: `comment_on_delete_anchors_to_the_deleted_text` (XML:
      `commentRangeStart`, our `w:del`, `commentRangeEnd`, reference run);
      `alias_text_on_comment_op_applies_with_a_note`;
      `alias_anchor_applies_with_a_note`; `text_on_replace_stays_an_error`.
- [ ] Implement; make `capabilities` and `check_operation_keys` read one
      table so help and errors cannot drift.
- [ ] Word: open the comment-on-deletion output; the balloon sits on the
      struck text.
- [ ] fmt, clippy, `cargo nextest run --test edit_plan_errors --test
      edit_plan --test edit_structured_comments`; commit on
      `feat/plan-errors-aliases`.

**Acceptance.** Every `INVALID_PLAN` for a field names the kind, the
field, the valid fields and the nearest one; the three aliases apply
with a note; the five listed errors stay errors.

**Related.** B6, brief item 12.

---
