@coderabbitai plan against `src/revision_processor.rs` (`resolve_package`, `resolve` 3503-3546, callers 3466/3481), `src/changes.rs:385`, `src/validate.rs` (`repair`), `src/edit.rs` (`check_thread_ops` 4270-4300, `EditPlan`), `tests/m_accept_word_parity.rs` and commit 48aae202 (rule R5); be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 14: stable comment ids; no editing another author's comment

**Stack.** Step E6 of the jubarte 0.13 contract-redlining plan; branch `fix/stable-comment-ids`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 14.

**Problem.** A bare `accept`/`reject` renumbers comment ids because
`resolve` (`src/revision_processor.rs:3503-3546`) calls
`annotation_ids::renumber` when no revision is left; a targeted settle
does not; `edit -p c106 --content …` rewrites a counterparty comment
under their name with no warning (brief items 15–16 in the mid-session
additions).

**Design (§A-detail 14, A).** Ids are identifiers: `resolve_package`
callers at 3466, 3481 and `src/changes.rs:385` pass `renumber: false`;
renumbering stays only in `validate --repair` (`repair` 1649) for
duplicate ids. `edit_comment` on a comment whose `w:author` differs from
the plan's author is refused: `COMMENT_NOT_YOURS: c106 is by "Trainer 1
- LargeCo"; reply instead (reply_comment), or pass
"rewrite_others_comments": true to change their words under their name`.
The plan field (`EditPlan.rewrite_others_comments`, default false) and
the CLI flag `--rewrite-others-comments` exist because the brief allows
it on explicit request; never the default. E5 records what the current
Word build does.

**Word parity.** The renumbering is deliberate: commit 48aae202 (rule
R5) copied Word's own Accept All, which numbers bookmarks and comments
with one dense counter in document order, read off 51 files. This plan
deviates from that on purpose: `edit` never renumbers, so jubarte
already disagrees with itself, and a stale handle that lands in the
wrong thread is worse than a non-dense id. Record the deviation in
`docs/WORD_DIFFERENCES.md` (ids are identifiers; Word's dense
renumbering is not copied on accept/reject) and re-pin
`tests/m_accept_word_parity.rs` so ids are compared structurally (the
same annotations in the same order), not literally.
`accept_revisions_package_keeping_ids` (`src/revision_processor.rs`) is
the existing mechanism. Arthur: this reverses R5 of 48aae202; say if
Word-dense ids on accept matter more than stable handles.

**Files.** `src/revision_processor.rs` (3466, 3481, `resolve` 3503),
`src/changes.rs` (385), `src/validate.rs` (`repair` 1649 keeps the
renumbering for `DUPLICATE_*` findings), `src/edit.rs` (`EditPlan` 64,
`check_thread_ops` 4270-4300: author check), `src/edit/flags.rs`,
`src/cli.rs`, `tests/edit_resolved_ids.rs`, `tests/edit_comment_ownership.rs`
(new), `tests/validate_cli.rs`, `tests/m_accept_word_parity.rs` (ids compared
structurally), `docs/MARKDOWN.md`, `docs/WORD_DIFFERENCES.md`.

**Steps.**

- [ ] Red: `accept_all_keeps_comment_ids` (c106 stays c106 after
      `jubarte accept FILE`), `reject_all_keeps_comment_ids`,
      `repair_still_renumbers_duplicates`,
      `editing_another_authors_comment_is_refused` (exact text),
      `rewrite_others_comments_true_allows_it_with_a_note`.
- [ ] Implement.
- [ ] Re-pin `tests/m_accept_word_parity.rs` on ids (structural, not
      literal), one explanation per test in the commit message; add the
      `docs/WORD_DIFFERENCES.md` entry.
- [ ] Word: E5 (can B edit A's comment; what `w:author` carries after).
- [ ] fmt, clippy, `cargo nextest run --test edit_resolved_ids --test
      edit_comment_ownership --test validate_cli`; commit on
      `fix/stable-comment-ids`.

**Acceptance.** A reply written after a bare accept lands in the thread
the agent read; `edit_comment` by a different author exits 3 unless the
flag is set.

**Related.** brief items 15–16 (added set).

---
