@coderabbitai plan against `src/edit.rs` (`check_conflicts` 4047-4056, the `removed_comments` report of PR #1191), `src/changes.rs`, `src/markdown/from_docx/critic.rs`; PRs #1146 and #1191 on the stack; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 10: report what a settle removes; replies in tracked-deleted text (#1159)

**Stack.** Step E6 of the jubarte 0.13 contract-redlining plan; branch `fix/settle-reports-1159`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 10.

**Problem.** Accepting r13 (a deletion holding only c4's reference mark)
removes c4 and its reply c5, as Word does. On the stack, PR #1146
performs that removal (replies included) and warns, and PR #1191 reports
every removed thread on the edit paths (`removed_comments: [{"id","author",
"replies"}]`), corrects the `UNKNOWN_COMMENT` advice and adds the opt-in
plan field `removed_anchor_comments: "keep"`. What the stack lacks:
`changes` does not say that r13 holds c4's mark; the view hides the
mark; a reply to a comment whose paragraph the plan deletes is refused
as `OVERLAPPING_EDITS` (`src/edit.rs:4047-4056`), the wrong code (brief
item 14; mid-session item 10).

**Design (§A-detail 10, A, cut down to what PRs #1146 and #1191 lack).**

- `changes` prints `r13 deletion (holds comment c4's mark)`; the view
  shows the mark inside the deletion.
- The report keeps PR #1191's shape and adds `why`:
  `removed_comments: [{"id":"c4","author":"…","replies":["c5"],"why":
  "anchor removed by accepting r13"}]`.
- Reply to a comment in a paragraph the plan deletes: allowed when the
  deletion is tracked (the reference stays in deleted text; Word shows
  the thread); refused with `COMMENT_ANCHOR_DELETED` only in editing
  mode.
- Do not re-implement the removal, the warning, the `UNKNOWN_COMMENT`
  advice or the keep field: they are PR #1191's. Branch from
  `fix/cmt-keep-1159` (#1191) or from `main` once it lands.

**Files.** `src/edit.rs` (`check_conflicts` 4047-4056; the `why` field
on PR #1191's `removed_comments`), `src/changes.rs` (`Change.text` for a
mark-only deletion), `src/markdown/from_docx/critic.rs` (mark inside a
deletion), `tests/edit_settle_reports.rs` (new),
`tests/edit_comment_threads.rs` (PR #1191's; extend, do not rewrite),
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `changes_names_the_mark_a_deletion_holds`,
      `view_shows_the_mark_inside_the_deletion`,
      `removed_comments_carry_why` (JSON exactly as above),
      `reply_in_a_tracked_deleted_paragraph_is_allowed`,
      `reply_in_an_editing_mode_deleted_paragraph_is_refused_with_comment_anchor_deleted`.
- [ ] Implement; `jubarte validate` must stay clean on every output;
      PR #1191's tests stay green unchanged.
- [ ] Word: open the reply-in-deleted-text output; the thread shows on
      the struck text.
- [ ] fmt, clippy, `cargo nextest run --test edit_settle_reports --test
      edit_comment_threads`; commit on `fix/settle-reports-1159`.

**Acceptance.** `changes` names every mark a deletion holds; no reply
refusal uses `OVERLAPPING_EDITS` for a deleted anchor; every
`removed_comments` entry says why.

**Depends on.** PR #1191 (the report and the keep field).

**Related.** #1159, PRs #1146 and #1191, brief item 14, mid-session item 10.

---
