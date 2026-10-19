@coderabbitai plan against `src/edit.rs` (`OperationKind`, `Resolved`, `check_conflicts`), `src/edit/tracked.rs` (`wrap_deleted`, `mark`, `clean_rpr`), `src/edit/flags.rs`, `src/cli.rs`; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 2: `strike` and `restore` (visible rejects)

**Stack.** Step E2 of the jubarte 0.13 contract-redlining plan; branch `feat/edit-strike-restore`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 2.

**Problem.** `resolve_revisions.reject` is Word's Reject: the
counterparty's insertion vanishes, their deletion silently comes back.
Ten delivered documents show the right text with no trace of the
decision (brief items 4–5, `C-invisible-settle/`).

**Design (§A-detail 2, A).** Two operation kinds, by our author:

```json
{"kind": "strike",  "revision": "r366", "comment": "Not acceptable: parents are not parties."}
{"kind": "restore", "revision": "r12",  "comment": "This carve-out was agreed in round 1."}
```

CLI: `jubarte edit FILE -p r366 --strike [--content "comment"]`,
`-p r12 --restore`; `--reject --visible` picks strike or restore by the
revision's kind. JSON: `resolve_revisions.reject.visible: true` does the
same for every id in that list. `resolve_revisions.reject` without
`visible` is unchanged.

OOXML. `strike` of their insertion: our `w:del` (new `w:id`, our author,
our date) wrapping the runs inside their `w:ins`, text as `w:delText`.
`restore` of their deletion: their `w:del` untouched; our `w:ins` right
after it holding a copy of the runs with `w:delText` → `w:t` and the same
`w:rPr`. Moves: `strike` of a `w:moveTo` nests `w:del` inside it after E2;
`restore` of a `w:moveFrom` is refused (restoring moved text duplicates
it). Marks: an inserted mark inside the struck range gets `w:del` beside
their `w:ins` in `w:rPr` (E1); a deleted mark to restore is a new paragraph
whose mark carries our `w:ins` (E4). Rows: `w:trPr/w:ins` by them gets
`w:trPr/w:del` by us. Fields: struck whole. Bookmarks, comments: unmoved.

Refusals (exact text): `OWN_REVISION: r40 is by "Reviewing Counsel
(AgentCo)"; delete the text instead`; `EMPTY_REVISION: r13 holds no text,
only comment c4's mark; reject r13 to keep c4`; a revision in both
`resolve_revisions` and a strike/restore op: existing
`REVISION_CONFLICT`; `strike`/`restore` in a plan whose
`existing_revisions` is `accept` or `reject`: `INVALID_PLAN: strike
needs existing_revisions keep (the revision would already be settled)`.
A strike of an insertion that already holds our nested deletion of the
same text: `ok` with `message: "already struck"`, not an error.

**Files.** `src/edit.rs` (`OperationKind` 171 + `check_operation_keys`
allowed lists ~1440; `Resolved` 1718: add `Strike { revision, comment }`
and `Restore { revision, comment }`; resolution in the `OperationKind`
match ~3030; `check_conflicts` 3988 for the `REVISION_CONFLICT` case;
`EditOutcome.context` `{-struck "…"}` / `{+restored "…"}`),
`src/edit/tracked.rs` (`check` 87, `emit` 278: new arms reusing
`wrap_deleted` 952 with the `w:ins` as container, `mark` 1015, `clean_rpr`
1038), `src/edit/flags.rs` (`FlagOp`: `strike`, `restore`, `visible`;
parsing 235-260), `src/cli.rs` (`EditOptions` flags and help),
`src/capabilities.rs` (kinds list), `docs/MARKDOWN.md`, `tests/edit_strike_restore.rs`,
`tests/cli_parser.rs`, `tests/m_validity_ring1.rs` (probe: nested
`w:del` with `w:t` is Word-invalid per #1134; make sure `ring1` already
fires, else add it).

**Steps.**

- [ ] Red: `strike_nests_our_deletion_inside_their_insertion` on
      `Step_1_Tracking.docx`: XML equals the shape above modulo dates.
- [ ] Red: `restore_copies_their_deleted_runs_as_our_insertion_after_it`
      (bold `w:rPr` preserved; their `w:del` untouched byte-for-byte).
- [ ] Red: the four refusal tests with the exact messages, and
      `already_struck_is_ok_with_a_note`.
- [ ] Red: `reject_visible_picks_by_kind` (CLI parse and JSON).
- [ ] Red: properties on the step family: (1) rejecting our author on
      `strike(x)` leaves the other author's elements identical to `x`;
      (2) `accept_all(strike(ins))` text equals `accept_all(reject(ins))`;
      (3) `accept_all(restore(del))` text equals `accept_all(reject(del))`;
      (4) `jubarte validate` clean on every output.
- [ ] Implement kinds, resolution, conflicts, emission, flags, help,
      capabilities, docs.
- [ ] Word: open strike and restore outputs normal / Accept All / Reject
      All / reject only our author / accept only their author (five
      renders each); run E2 and E4 before enabling moves and marks, else
      refuse those with the exact alternative.
- [ ] fmt, clippy, `cargo nextest run --test edit_strike_restore --test
      cli_parser --test m_validity_ring1`, `--help` smoke; commit.

**Acceptance.** Both JSON kinds and both flags work on text revisions;
`changes` on the output lists our deletion `inside r366` and our
insertion after `r12`; Word's four readings in §A-detail 2 hold on the
fixtures; the hint of plan 4 offers `restore` first.

**Depends on.** Plan 1 (nested emission), E1/E2/E4 for marks and moves.

**Related.** #1108 (hint), #1134, brief items 4–5, B1.

---
