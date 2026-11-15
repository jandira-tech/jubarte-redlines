<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Baseline addenda

Hand-maintained measurements appended after T0.1. `scripts/split_baseline.sh
record` reproduces this file verbatim at the end of `baseline.md`, so a
re-recording never loses these entries; add new ones below, newest first.

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
