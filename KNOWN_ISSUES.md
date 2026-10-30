<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts.

> **Last full re-check: 2026-10-01, against 0.10.1.** Open items below are the
> complete set. Findings added since then name the build they were checked on
> (issues 8 to 10: the 0.12.0 release candidate, 2026-10-10); the other items
> have not been re-checked against 0.12.0.

## Open

### 12. Strict `lineRule="auto"` values carry units no rule defines — **OPEN, spec gap**

**Seen:** `tests/fixtures/strict/Strict01.docx` (#531), checked 2026-10-16.
ISO/IEC 29500 defines an auto `w:line` in 240ths of a line and stays silent
on unit-suffixed values; Word's own Strict writer emits `12pt` for single
spacing and `12.95pt` for 1.08.

**Done so far (#531):** the Markdown reader and the render diagnostic decode
a unit-bearing auto value like twips (`×20`): `12pt` → 240 and `12.95pt` →
259, the integers the Transitional twin carries. This reproduces the only
observed producer (Word) and needs no baseline assumption, but it is a
producer convention, not a normative rule. The renderer keeps its own
11pt-per-line heuristic (`src/convert/mod.rs`), as before.

**Open:** the maintained Word workflow has not confirmed the values on a
layout-level comparison.

### 11. Word splits and pairs mismatched tables that jubarte keeps whole — **OPEN, Word parity**

**Seen:** Task 22, corpus pair `aeb0ace263` vs `1000537d17` (Word redline
`3d4318d7e9`), checked on the #497 branch, 2026-10-16. The two sources share
no identical table row. Word still pairs rows by shared words, splits the
deleted runs into their own tables and inserts the 47-row table whole: 8
tables, four `tblPrChange` and four `tblGridChange`. jubarte writes 3.

**Done so far (#497):** `src/comparer/lcs_table.rs` pairs single rows across
different horizontal partitions when they share a word, and row words split
at cell, paragraph and tab boundaries.

**Done (#1817):** Word mode meshes a row whose cells were repartitioned,
so the old header row (`Rhythmia`) now pairs with the new one (`Group`);
`our_compare_pairs_unmatched_rows_into_merged_rows` runs.

**Open:** the table split.

**Tests:** `our_compare_matches_word_table_shape` in
`tests/m_table_alignment_3d4318d7e9.rs` asserts Word's shape and is
`#[ignore]`d with this item's number; `word_redline_has_expected_table_shape`
pins Word's own redline and runs. Run the target with `cargo test --test
m_table_alignment_3d4318d7e9 -- --ignored`.

### 10. A square-wrapped float does not wrap the lines above its anchor — **OPEN, PDF layout**

**Seen:** the 0.12.0 docx→PDF mini-bench (2026-10-10). The redline
`617c1284c9…__vs__2773adf157…_redline_2ef6ed00b9` scores 23.7, where
LibreOffice scores 81.5. Word prints it on 2 pages; jubarte prints it on 1.

**The rule (Word 16.115 probes, `w617`, 2026-10-10):** the last paragraph
anchors a margin-relative `wrapSquare` picture, 54pt wide, near the page
foot. Its box overlaps a text line laid out *before* its anchor paragraph.
Word re-lays the page with the float in place. That earlier line wraps
around the picture and grows, the anchor paragraph no longer fits, and it
moves to page 2 with its floats.

The probes isolate the picture:

| Probe | Picture | Word | jubarte |
|---|---|---|---|
| original | as in the file, overlaps a text line | 2 pages | 1 page |
| raised | moved up, overlaps the heading instead | 2 pages | 1 page |
| `wrapTopAndBottom` | as in the file | 2 pages | 2 pages |
| lowered | beside empty lines only | 1 page | 1 page |
| `wrapNone` | as in the file | 1 page | 1 page |

Whether the paragraph is deleted or live does not matter.

**Cause:** jubarte lays the page out in one pass. A float takes effect from
its anchor paragraph on, and the lines already placed on the page never see
it. `wrapTopAndBottom` matches Word only because the anchor line itself
meets the band (`apply_top_bottom_wrap`).

**Action:** a page re-layout pass. When a paragraph anchors a margin- or
page-relative square or tight float whose box overlaps lines already
placed on its page, lay those lines out again around it, then move the
anchor paragraph on if it no longer fits. It is not a small change, so it
is left out of 0.12.

### 9. WebAssembly PDFs paint no CJK text the document does not embed a font for — **OPEN, fonts**

**Seen:** the 0.12.0 release candidate's CLI, Python and WebAssembly parity
check (2026-10-10, 15 bench pairs). The Word redlines are byte-identical on
all three, and so are 13 of the 15 PDFs. The other two differ in WebAssembly
only:

- **Japanese (a form, `a4ebc0e89c…`):** the WebAssembly PDF paints none of its
  Japanese text. The heading is three empty boxes, every other run is blank
  under its revision marks, and full-width ＴＥＬ and ＦＡＸ are gone. Its
  text layer holds 12 NULs where the native PDF holds the text, and lines
  break differently without those widths.
- **Math (`super_editor__font_formatting_runs`):** the OMML letters are
  painted from the italic text face and read as `a x y z b`, where the native
  PDF's read `𝑎 𝑥 𝑦 𝑧 𝑏` (Mathematical Alphanumeric Symbols, from Cambria
  Math). The page looks right; search and copy differ. Carlito standing in
  for Aptos also moves a PAGE field one page earlier; the page count holds.

**Cause:** WebAssembly has no installed fonts. `docxToPdf` paints with the
fonts the document embeds and the ones the engine bundles, Carlito and
Liberation Sans, Serif and Mono: Latin, Greek and Cyrillic, no CJK and no
math alphabet. The CLI and Python use the machine's installed fonts.

**Text layer fixed (2026-10-11):** the text no face covers is now written
as hidden text in an `/ActualText` span, and a math letter painted from its
ASCII italic reads back as the math letter. The same 15 pairs now read
back the same text in both PDFs (`parity_check.py`, CLI against
WebAssembly). The Japanese is still not painted.

**Options:** let `docxToPdf` take font files from its caller, as
`JUBARTE_FONT_DIR` lets the CLI; ship a CJK font in an opt-in package; or
report the text no font covers instead of painting nothing.

### 8. Word 16.115 crashes opening benchmark redlines — **OPEN, mostly a Word bug**

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

**0.12.0 candidate (2026-10-10):** 25 of the 0.12.0 sample's 600 redlines
crash Word alone, under fresh names. Word's own redline of the same pair
crashes Word 16.115 too for 20 of them: a Word bug its own output hits.
One reduced case: an inserted anchored text box in the paragraph right
before a table whose rows are deleted. Of the 5 pairs whose Word redline
opens, two were jubarte bugs, now fixed and opened in Word: a header or
footer present on one side only left its table rows unmarked, and a
deleted or inserted empty `w:fldSimple` stayed live. The other three
(two documents and a table-style case) crash only with a combination of
content that no single element explains, the same class as the 20.

**Main after the 0.12 pre-release merge (2026-10-10, 7a382267):** the two
fixed pairs export; the other 23 crash Word 16.115 again, in the batch and
one at a time. 115 random redlines of the same sample all export, and
jubarte validate flags none of the 140 Word-fatal.

**Action:** rerun the 25 on the next Word update; reduce one of the three
layout cases further only if a Word update does not clear it.

### 6. Dependabot: glib 0.18.5 in the desktop app (RUSTSEC-2024-0429) — **OPEN, blocked upstream**

**Alert:** GitHub reports one moderate vulnerability, `glib` ≥0.15 <0.20
("Unsoundness in `Iterator` and `DoubleEndedIterator` impls for
`glib::VariantStrIter`") in the desktop app's `src-tauri/Cargo.lock`
(the app is its own repository now, arthrod/jubarte-app, so the alert
lives there).

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
