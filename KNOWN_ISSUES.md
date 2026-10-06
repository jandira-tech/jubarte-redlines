<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts.

> **Re-checked 2026-10-01 against 0.10.1.** Open items below are the complete set.

## Open

### 8. Word 16.115 crashes opening 23 benchmark redlines — **OPEN, cause unknown**

**Seen:** the 0.11.3 release evidence (2026-10-06). Word 16.115, installed on
the bench on 2026-10-05, dies ("Connection is invalid (-609)") opening 23 of
the 600 jubarte redlines of `release_info/sample_redline_0.11.3_*.csv`. Each
crashed in the batch export and again on a one-at-a-time retry; 21 crashed
again after a Word restart, and 6 also when opened alone under a fresh name.
They score 0 in the release's results.

**What is known:** 22 of the 23 are byte-identical to the 0.11.2 redlines that
the previous Word opened and exported on 2026-10-03, so no 0.11.3 change
caused them; current Word still cannot open them. Every original opens in
Word. docx-validate, the OpenXML validator and `jubarte validate` find
nothing in 10 of them; the findings in the rest come from their originals,
and the same findings sit in redlines that open.

**Action:** reduce one crasher (file_176 vs file_177) against a redline that
opens, each variant alone in Word, then fix the writer with a test.

### 6. Dependabot: glib 0.18.5 in the desktop app (RUSTSEC-2024-0429) — **OPEN, blocked upstream**

**Alert:** GitHub reports one moderate vulnerability, `glib` ≥0.15 <0.20
("Unsoundness in `Iterator` and `DoubleEndedIterator` impls for
`glib::VariantStrIter`") in `jubarte-app/src-tauri/Cargo.lock`.

**Why it stays open:** glib is only there through `gtk` 0.18, which Tauri 2
uses for its Linux webview. gtk-rs/gtk3-rs was archived in March 2024, so no
gtk release can ever move to glib ≥0.20. The alert clears only when Tauri
moves its Linux build to GTK4. The engine, CLI, Python and WASM packages do
not depend on glib. The desktop app compiles it for Linux only, and nothing
in it calls `VariantStrIter`.

**Action:** re-check on each Tauri upgrade. Dismissing the alert as
"tolerable risk" is the maintainer's call.

### 7. `fill_control` redlines show the fill without its content control — **OPEN, Word parity**

**Finding:** an edit plan's `fill_control` writes the value inside the
control's `w:sdtContent` and keeps `w:sdtPr` in the clean copy. The redline
is produced by comparing the source with that clean copy, and the comparer
follows Word Compare: `unwrap_content_controls_in_pure_revisions`
(`src/comparer/finalize.rs`, M390) drops the `w:sdt` wrapper and its
properties in every paragraph that carries `w:ins` or `w:del`. A filled
control's paragraph always does, so the redline shows the fill as plain
tracked runs (`Name: <ins>Ada</ins><del>Click here</del>`) with no tag,
alias or lock. Controls in unchanged paragraphs keep their wrapper.
`tests/m25_sanitize_sdt_pr.rs` documents the same unwrap.

**Effect:** the clean copy is the deliverable for form filling; the redline
is still a valid Word redline of the text change.

**Test:** `decision_redline_keeps_the_control_wrapper` in
`tests/edit_fill_control.rs` asserts the desired redline and is
`#[ignore]`d with this item's number. Run it with `cargo test --test
edit_fill_control -- --ignored`.

**Action:** a fix needs either a Word-mode-only exception to M390 for
controls whose properties are unchanged, or re-wrapping the filled runs in
the original `w:sdt` after compare. Either must first be checked against
Word's own redline of a filled form.

## Settled

Items 1–5 are the engine's settled history, one line each; the full story of
every one is in [CHANGELOG.md](CHANGELOG.md).

1. **MovedSource / `w:moveFrom` text kind** — settled 2026-07-16 (Word
   wins: `w:t` under `w:moveFrom`, `w:delText` under `w:del`); enforced by
   Ring 1. See [CHANGELOG.md](CHANGELOG.md).
2. **Multi-del boundary fold** — resolved 2026-09-28 against Word's own
   redlines, closing the positional-zip defect ([0.10.0] Fixed). See
   [CHANGELOG.md](CHANGELOG.md).
3. **Free-mesh double-consumption — one A-side atom claimed by two
   paragraphs** — fixed 2026-09-27, with the text round-trips found beside
   it ([0.9.3] Fixed). See [CHANGELOG.md](CHANGELOG.md).
4. **Internal `Unid` scratch shipped as an undeclared `w:Unid`
   attribute** — fixed by b7fedc78, closed 2026-09-28 ([0.9.3] Fixed). See
   [CHANGELOG.md](CHANGELOG.md).
5. **Ring 3: five corpus redlines Word refused to open** — fixed
   2026-09-05 (`w:instrText` under `w:del` became `w:delInstrText`,
   [0.9.0] Fixed). See [CHANGELOG.md](CHANGELOG.md).

Two older notes are settled as well: external hyperlinks keep their `r:id`
([0.1.0] Fixed), and clean adjacent tables are merged through the
accept/reject pipeline ([0.10.1] Fixed).
