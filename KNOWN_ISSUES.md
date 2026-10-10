<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts.

> **Last full re-check: 2026-10-01, against 0.10.1.** Open items below are the
> complete set. Findings added since then name the build they were checked on
> (issues 8 and 9: the 0.12.0 release candidate, 2026-10-10); the other items
> have not been re-checked against 0.12.0.

## Open

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
