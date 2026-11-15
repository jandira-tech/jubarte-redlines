<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Baseline addenda

Hand-maintained measurements appended after T0.1. `scripts/split_baseline.sh
record` reproduces this file verbatim at the end of `baseline.md`, so a
re-recording never loses these entries; add new ones below, newest first.

## T2.4 (numbering, table_dom) - 2026-11-15

- api-baseline.txt re-recorded again: the w:numFmt label family
  (NumFmt, format_num and 23 label functions), append_numbering with
  numbering_part/max_attribute/attribute_values, and the four table DOM
  helpers (table_look/TblLook/cell_is_deleted/wrapped_children) moved to
  the pub(crate) modules crate::numbering and crate::table_dom, so the
  listing's internal lines renamed with them. All 259 diff lines are
  internal; no publicly nameable path changed.

## T2.3 (comment_settlement) - 2026-11-15

- api-baseline.txt re-recorded after a reviewed, intentional drift: the
  module moved from the private `jubarte::edit::comment_settlement` path to
  the private `jubarte::comment_settlement` path, so the listing's internal
  lines renamed with it, `changes::ResolvedChanges::kept_comments` renders
  its element type through the new path, and `EditError` gained a
  `From<SettlementError>` impl. No publicly nameable path changed:
  `jubarte::edit::{KeptComment, RemovedComment}` still resolve (re-export)
  and no public item was added or lost (diff reviewed line by line).
- Plan deviation recorded on #2428: the plan's critic_literal move to
  debug.rs is infeasible (it calls markdown's CriticMarkup reader through
  diff_markdown; defining it in the doc layer would trade the
  debug.rs->text_diff edge for a worse debug.rs->markdown edge). The edge
  stays in KNOWN_UPWARD until Phase 4 composes the critic view.

## T1.1 (test suites) - 2026-11-15

- Test cases: 10,692 before and after the move (nothing lost, nothing added
  beyond `suite_membership`'s 2 tests, which the baseline count already
  included because the file existed when T0.1 was re-recorded).
- Test binaries: 599 before (597 top-level test files + lib + bin) -> 30
  after (21 standalone + 7 suites + lib + bin).
- Suite membership: it_cli 78, it_core 17, it_doc 25, it_edit 46,
  it_markdown 55, it_redlines 351, it_render 4 (576 moved). The plan's A8
  expectation of it_edit 45 / it_markdown 56 differs by one file,
  `markdown_redline.rs`, whose `jubarte::markdown::redline` references map
  to the edit layer through A8's own submodule override.
- `cargo nextest run --all-features --no-run` after the move, cache-hot:
  3 s wall (T0.1 recorded 122 s cache-hot with 599 binaries; the binary
  count, not the suite split, is what changed).
