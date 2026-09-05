<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Plan: close the 76-vs-398 divergence

Companion to `report.md` (2026-09-05). All three documents and the regression sample live in `<jubarte-redlines>/planning/`. Ordered so that measurement exists before
engine changes, then by Jaccard recovered per unit of work. Each step names the
file to change, the measurement that proves it, the regression guard, and what
it is expected to move. Status column reflects the end of this session.

## Locations on this machine

| What | Path |
|---|---|
| jubarte-redlines (engine, this checkout) | `/Users/arthrod/temp/T/jubarte-redlines` (binary `target/release/jubarte`; these documents in `planning/`) |
| docxide-pdf (fork of sverrejb/docxide-pdf, branch `add-jubarte-redlines-engine`, PR #1 on arthrod/docxide-pdf) | `/Users/arthrod/temp/T/docxide-pdf` (76 fixtures `tests/fixtures/cases/*/{input.docx,reference.pdf}`; comparison tool `tools/engine_compare.py`; scorer `tools/target/release/page-metrics`; per-engine scores `comparison/work/manifest.json`; upstream baselines `tests/baselines.json`) |
| docxide-pdf binary | `~/.cargo/bin/docxide-pdf` (needs `DOCXSIDE_FONTS="/Applications/Microsoft Word.app/Contents/Resources/DFonts"`) |
| neurotic_docx_bench (benchmarks) | `/Users/arthrod/temp/T/neurotic_docx_bench` (398 corpus: `corpus/no_comments_pdf_was_generated_by_word/{docx_source,docx_source_randomized,pdf_source,pdf_source_randomized}`, fixture list `docx_to_pdf_no_redline_fixtures.txt`; results `results/docxide_metrics.json` (jubarte 0.8.0 + font-starved docxide-pdf), `results/docxide_metrics_docxide_dfonts.json` (docxide-pdf with fonts), `results/docx_to_pdf_no_redline_docxide_dfonts.json` (main-track re-run); vendored scorer `src/neurotic_docx_bench/utils/docxide-metrics/target/release/docxide-metrics`; CLI `uv run bench docxide-metrics --tool <jubarte|docxide-pdf>` and `uv run bench docx-to-pdf --track docx_to_pdf_no_redline_docs --tool ...`) |
| Word fonts | `/Applications/Microsoft Word.app/Contents/Resources/DFonts` (Aptos, Calibri, Cambria, ...); Arial / Times / Georgia / Verdana / Courier in `/System/Library/Fonts/Supplemental` |
| Rasteriser / PDF tools | `mutool` (MuPDF) on PATH |
| Redline benchmark assessment | `planning/redline_assessment.md` (evidence only; no changes proposed) |
| Regression sample | `planning/sample50_check.py`, `planning/sample50.tsv`, `planning/sample50_baseline.json` (run from anywhere: `python3 /Users/arthrod/temp/T/jubarte-redlines/planning/sample50_check.py`) |

## Ground rules

- Engine fixes go only in `../jubarte-redlines/src/convert/` (AGENTS.md: the
  consumer copies under neurotic are never edited).
- Every engine change is measured on **both** sets before it is kept: the 76 via
  `python3 tools/engine_compare.py --skip-libreoffice` in docxide-pdf (or the
  jubarte-side sweep from Step 1), and the 398 via
  `uv run bench docxide-metrics --tool jubarte` in neurotic. A change that lifts one
  set and drops the other is a tuning trade, not a fix; it is reported as such and
  the dropped documents are examined one by one.
- No new per-face, per-size or per-document constants. The 195 "mini …" comments
  in `mod.rs` are how the converter got here.
- Rasters are deleted after scoring (AGENTS.md disk rule); only PDFs and JSON stay.
- Every step ends with the numbers appended to `report.md`.
- **After fixing each document or defect, before touching the next one, run the
  50-fixture sample:**

  ```sh
  python3 planning/sample50_check.py   # ~3 min, from the jubarte-redlines root: converts, scores, diffs against sample50_baseline.json
  python3 planning/sample50_check.py --bless   # only after a full both-set sweep has confirmed the change
  ```

  `planning/sample50.tsv` is 3 corpus stems per jubarte-Jaccard decile (30) plus 2 docxide
  cases per decile (20), seed 0, so every score band is watched. A row that drops
  more than 1.0 Jaccard, a sample mean that drops more than 0.2, or a convert failure
  is a regression: fix it or name every such row in the commit message. Baseline
  blessed 2026-09-05 from jubarte 0.9.0: mean 37.57 (corpus rows 52.8, docxide rows
  14.7); a no-change re-run reports zero deltas. This is the smoke gate; the full
  both-set sweep in Step 1 is still required before a commit is kept.

## Step 0: make the published comparison honest (no engine work)

| | Action | Status |
|---|---|---|
| 0a | Re-run docxide-pdf on the 398 with `DOCXSIDE_FONTS=/Applications/Microsoft Word.app/Contents/Resources/DFonts` under the docxide scorer. | **Done.** `results/docxide_metrics_docxide_dfonts.json`: 67.99 / 83.28 J, 78.63 / 97.66 SSIM, 87.23 / 100 TB, pass 334 / 270. |
| 0b | Same for the main `docx_to_pdf_no_redline_docs` table (superdoc scorer). | **Done.** `results/docx_to_pdf_no_redline_docxide_dfonts.json`: 81.90 mean / 88.75 median, 398 / 398, 0 failures, against the published 65.65 / 63.97. docxide-pdf moves from 5th to 1st in that table. |
| 0c | Replace the docxide-pdf rows in neurotic's README (both tables) with the fonts-visible numbers, and rewrite the "Two scorers, same fixtures" commentary, which currently explains an inversion that does not exist. Commit and push. | Arthur's call on wording; the numbers are ready. |
| 0d | Make the bench font-fair by construction: in neurotic's docxide-pdf tool spec (`docx_to_pdf.py` `WORD_PDF_TOOLS`, and the `docxide_metrics.py` driver) set `DOCXSIDE_FONTS` to the DFonts folder when it exists, and say so in the README's method notes. Otherwise the next person re-publishes the artifact. | Not started. |
| 0e | Check the other engines in the main table for the same starvation: `mutool info -F` on one rdocx, dxpdf and libreoffice_convert_rust output for a Calibri document. LibreOffice has its own discovery and probably finds DFonts; rdocx / dxpdf may not. | Not started. |
| 0f | Correct PR #1 on arthrod/docxide-pdf: the body quotes docxide-pdf at 35.1 / 20.5; with its fonts it is 67.9 / 71.8 on the same 75 cases. Also make `tools/engine_compare.py` in that PR pass `DOCXSIDE_FONTS` through (or document it) so the maintainer does not reproduce the artifact. | Outward-facing; left for Arthur. |

## Step 1: put the 76 fixtures inside jubarte's own gate before touching the engine

- **Why first.** Every later step must be measured on both sets. Today the 76 are
  measurable only from the docxide-pdf checkout via `engine_compare.py`.
- **What.** docxide-pdf is Apache-2.0; vendor `tests/fixtures/cases/*/{input.docx,
  reference.pdf}` (46 MB, 76 cases) into `../jubarte-redlines/tests/fixtures/docxide_cases/`
  with a NOTICE line, or reference them by path from `../docxide-pdf`. Add
  `scripts/convert-sweep.sh`: convert each `input.docx`, score with neurotic's
  `utils/docxide-metrics` binary (same code as upstream's `page-metrics`), write
  `stem<TAB>jaccard<TAB>ssim<TAB>text_boundary`, delete rasters. Same script runs the
  398 through the neurotic CLI. Baseline both as `tools/convert_baseline_{76,398}.tsv`.
- **Ratchet.** A commit may not drop any row by more than 1.0 Jaccard on either set
  without naming the row in the commit message. Mean and median of both sets go in
  the CHANGELOG entry.
- **Already in place**: `planning/sample50_check.py`, `planning/sample50.tsv`,
  `planning/sample50_baseline.json` in this checkout; move them into `tools/` with the fixtures.
- **Also add** the two cheap diagnostics from this session as a pre-check on page 1:
  first-ink-row delta and band-pitch delta versus the reference (`bands.py` logic). They
  catch Findings C and D in seconds without a full sweep.

## Step 2: font resolution as data (Finding A, bottom-decile clusters A1, A2, B; report section 14)

- **Files.** `src/convert/font.rs` (`resolve`, `FaceId`, `system_override`),
  `src/convert/mod.rs` (`apply_rfonts`, `load_theme`, package loading for
  `word/fontTable.xml`).
- **2a. Read `fontTable.xml`.** For each `w:font`, keep `w:altName`, `w:family`,
  `w:pitch`, embedded-font relationships. When a requested family has an altName,
  resolve the altName; it is Word's own substitution record. 81 of 398 corpus documents
  carry one; 15 of jubarte's bottom 40 are explained by it alone.
- **2b. Stop parsing family names.** Word treats `"Times New Roman", Times, serif` as
  one unknown family. Remove the comma split and quote strip from `resolve`; a name
  that fails exact lookup falls to 2d, not to its first token.
- **2c. Honour the theme slot for every family.** Remove the `starts_with("aptos")`
  gate in `apply_rfonts`; keep the Display-cache rule; explicit `w:ascii` still wins.
- **2d. Word-substitution evidence table.** A data file (TOML/JSON), one row per
  observed substitution with the oracle stems it was measured on: unknown or CSS-list
  family -> Cambria; DejaVu Sans Mono -> Verdana; Liberation Serif -> Hiragino Mincho
  ProN W3 / Times New Roman (both observed, keyed by language tags, to be measured);
  no docDefaults font -> Times New Roman; Inter -> Cambria (already in code as a
  special case; becomes a row). Generic of last resort from `w:family`
  (roman / swiss / modern), as docxide-pdf's `family_fallback` does.
- **2e. Open-face table with verdicts**, modelled on `@docfonts/fallbacks` records
  (logicalFamily, physicalFamily, verdict, policyAction, faces, generic, evidenceId).
  Bundle Caladea (Cambria), Gelasio (Georgia), Noto Sans (Verdana), Inconsolata
  (Consolas) next to Carlito and Liberation; keep the DFonts overlay on macOS. Retire
  the per-face `FaceId` enum in favour of a face registry keyed by family + style.
- **2f. Decision kinds and a font report.** Each resolution returns which step
  resolved it (`altName`, `explicit`, `theme`, `word_substitution`, `open_fallback`,
  `generic`, `unknown`) and whether the face is real or synthesized;
  `jubarte convert --font-report` prints it per document. The bench records the
  counts so the next unknown-family cluster is visible before it costs a decile.
- **Measure.** 76: the 47 FONT-theme rows plus case3 / case33 (Arial), case7 / case48 /
  case49 / case77 (Cambria headings). 398: `minorHAnsi -> Cambria` group (n = 12, 11.2),
  `css_style_family_list` (40 documents), `family_outside_known_table` (12),
  `none` docDefaults (5, at 4.3), and the 15 cluster-A1 stems in report section 13.1.
  The explicit-Calibri group (n = 298) is the canary and must not move.
- **Known interaction.** Honouring Cambria exposes the Cambria line box (Step 4);
  `file_2` / `file_41` will fall until Step 4 lands. Record, do not re-gate.
- **Expected.** 76: +10 to +15 mean. 398: +4 to +6 mean (the altName cluster is the
  biggest single lift available on the corpus: 15 documents from ~5 to ~75).

## Step 3: apply space-before on the first paragraph of the document (Finding C)

- **File.** `src/convert/mod.rs` ~6503. Word's rule: space-before is suppressed only
  when a paragraph reaches the top of a page by automatic pagination. It is applied at
  document start, after a section break (already handled) and after a hard page break
  (`w:br w:type="page"`) unless the `suppressSpBfAfterPgBrk` compat setting is present.
  Implement `at_page_top` to mean "arrived here by overflow", not "y is at the margin".
- **Measure.** 76: the 42 TOP rows, first-ink-row delta should go to 0 +/- 2 px on
  the 50 / 38 / 21 px groups. 398: the six Heading-1-first documents in report
  section 5 (multi_section, page_numbering_examples, file_65, file_92, anchor_images,
  file_197) should go from -50 / -28 px to 0; the five plain-start controls must not
  move.
- **Expected.** 76: +10 mean (SHIFT column shows what a pure offset is worth: case2
  4.0 -> 34.9, case76 2.5 -> 32.1, case55 15.9 -> 34.1). 398: +0.3 mean.

## Step 4: one line-box formula from font metrics (Finding D)

- **Files.** `src/convert/mod.rs` ~6540-6560 (the `line_box` branches), `font.rs`
  `single_line_pt`. Replace the per-face branches with the rule Word uses: line height
  = face line spacing x size x (`w:line`/240) for `auto`, `w:line`/20 pt for `exact`,
  max(natural, `w:line`/20) for `atLeast`; headings are not special; TOC is not
  special. Face line spacing comes from the face's own tables (hhea ascender -
  descender + lineGap, or OS/2 winAscent + winDescent when USE_TYPO_METRICS is
  clear); pick by measurement against case7, not by assumption.
- **Measure.** 76: case7 first (six families, target 0 px cumulative drift, which
  docxide-pdf already achieves), then case62 / case61 (Arial), case4 / case52 / case53
  (Calibri with headings), case24. 398: explicit Times New Roman (n = 6, 12.1),
  explicit Arial (n = 6, 53.0), `minorHAnsi -> Cambria` (after Step 2), and the whole
  explicit-Calibri group as the regression canary.
- **Risk.** Highest in the plan. This is where the mini-set constants live; some 398
  rows will drop on the first cut. Each drop is a document to diff against Word (band
  analysis, not the aggregate), never a reason to restore a per-face branch.
- **Expected.** 76: +5 to +10 mean. 398: unknown sign on the first cut; target >= 0.

## Step 5: table left edge and row height

- **File.** table geometry in `src/convert/mod.rs` (`table_col_widths`,
  `table_row_height_pt`, the `11.0 * line_mult` constants at ~2699-2702).
- **Edge.** Word places the table's left border at margin + `tblInd` - left cell
  margin so cell text aligns with body text; jubarte places it at the margin. Nine
  fixtures show the 11-12 px (5.4 pt = 108 twips) shift: case6, 15, 40, 45, 46, 51, 55,
  61, 67. The 398 has 88 documents with tables; check three before and after.
- **Row height.** case51 (10 tables on one page where Word needs two) and case6
  (four pages where Word has three) show the cell line box is wrong in both
  directions; it should fall out of Step 4 once cells use the same formula as
  paragraphs instead of `11.0 * line_mult`.
- **Expected.** 76: +2 mean. 398: small but positive (88 table documents).

## Step 6: images

- `a:srcRect` cropping (case78, 0 occurrences today): apply the crop to the image
  XObject via a clip and scaled placement.
- PNG alpha (case12): emit an `/SMask` from the alpha channel instead of dropping it.
- Inline picture line box (case12, 55 px low): the picture paragraph's line box should
  be the picture height plus the paragraph spacing, not a text line plus the picture.
- **Measure.** case12, case16, case27, case42, case78; corpus `anchor_images` and the
  31 `drw` documents.

## Step 7: missing features, ranked by fixtures recovered per effort

| Feature | fixtures | effort | note |
|---|---|---|---|
| Page borders (`pgBorders`) | case68 | small | rectangle at margin +/- `w:space`, width from `w:sz` |
| Embedded fonts (`w:embedRegular`, `.odttf`) | case8 | small-medium | read `fontTable.xml`, de-obfuscate the `.odttf` with the GUID key, register the face; docxide-pdf's `docx/embedded_fonts.rs` is the reference implementation |
| Footnotes | case18, 74, 75, 76 | medium | parse `footnotes.xml`, reserve space at page bottom, separator rule, superscript reference; 12 corpus documents have footnotes |
| Floating tables (`tblpPr`) | case46 | medium | position by `tblpX/Y`, wrap body text beside it |
| Preset geometry (`prstGeom`) | case34, 35, 36, 37, 38, 41 | large | Option A: port docxide-pdf's `src/geometry/` (Apache-2.0: `definitions.rs` 187 presets, `formulas.rs`, `path.rs`) with attribution. Option B: hand-implement the 20 presets in case34. A is less work and more complete. |
| Charts | case29, 30, 31, 56, 57, 59 | large | partial support exists (`c:chart`, 15 sites); compare against docxide-pdf's `pdf/charts*.rs` |
| Text boxes / SmartArt | case60 | large | last; one fixture, 0 corpus documents with `dgm:` |

## Step 8: audit the tuning constants

- `grep -n -i 'mini' src/convert/mod.rs` lists 195 comment sites. Classify each:
  (a) a substitute for a font metric (retire with Step 4), (b) a document-specific gate
  (remove, measure, keep only if the 398 and the 76 both hold), (c) a genuine Word rule
  (keep, replace the "mini NNN" justification with the ECMA-376 or Word behaviour it
  encodes). Start with `word_device_track` / `word_device_paint` (`font.rs` 17-40, Tc
  hacks for 11.04 and 16.08 pt) and the heading-gap rules at `mod.rs` ~9015-9030.
- Output: a table in `KNOWN_ISSUES.md` or a new `TUNING_AUDIT.md`, one row per
  site, with its class and disposition.

## Step 9: fixture-side notes (no engine work)

- case63 / case64: references are Word "print with markup" exports (content scaled
  and shifted for the comment column). Exclude from jubarte's 76-gate mean or keep
  them as a known-zero; do not chase them.
- case13: 205 pages; upstream skiplists it. Keep it out of the fast gate, run it in
  the full sweep.
- case37: one drawing nobody renders (docxide-pdf 23.2, LibreOffice 0.0). Park.

## Step 10: bottom-decile items not covered above (report section 13.1)

| Cluster | documents (bottom 40 / corpus) | change | measure |
|---|---|---|---|
| D merged cells | 4 / 8 (+ Redline_CiceroDo, 40 vMerge) | lay out `w:gridSpan` and `w:vMerge` (column spans, vertical merges with the continuing cell empty), borders on the merged box | nested_table_rowspan, table_vmerge_colspan, file_47, file_199: the CCC / DDD / EEE cells must separate; then Cicero pages 1-5 |
| E anchored text boxes / VML | 3 / 18 | position `wp:anchor` and VML text boxes by `positionH` / `positionV` relative to page, margin or paragraph; never at (0,0); wrap body text by `wrapSquare` where present | file_104 (box beside the paragraph), file_70 (three bands) |
| E broken media | 1 / ? | when an image relationship target is missing, draw Word's small placeholder box, not a full-size reservation | word_tolerated_broken_media_rel: 1.0 -> ~79 (the shift experiment) |
| F endnotes | 1 / 10 | with footnotes in Step 7: parse `endnotes.xml`, render after the body (or at section end per `endnotePr`), superscript references | endnotes_sample text-boundary 0 -> 100 |
| G footer block | 2 / 52 | measure Word's placement of a multi-line footer against `w:footer` and the bottom margin on complex_style_attr / file_30 (39 px low today) and on five more corpus footers; fix `mod.rs` ~8503 to the measured rule | footer band tops within 2 px on all seven |
| G list-level spacing | 2 / 50 (`contextualSpacing`) | check `contextualSpacing` and numbering-level spacing on complex_style_attr ("1. ONE / a. a": 24 vs 28 px) | band pitch on the two documents |
| I A4 MediaBox | 4 / 21 | write 595.2 x 841.92 pt for 11906 x 16838 twips, as Word does | MuPDF raster width 1240 px on the 21 A4 documents |
| H long structured documents | 3 / - | park: sd_2517, file_22, Redline_CiceroDo re-measured after Steps 2-7; expect most of the gap to be fields, sections and merged cells | text-boundary on all three |

Priority inside Step 10: D and E first (jubarte-only failures where docxide-pdf is
also low, so they are real layout work, not tuning), then G (cheap, 52 documents),
I (one constant), F with Step 7.

## Step 11: XML parts coverage (see `xml_parts_plan.md`)

A census of every package part and element against both document sets, scored by
available mean-Jaccard lift, with full implementation plans for the four items at the
`fontTable.xml` level: `fontTable.xml` + embedded fonts (3.1, supersedes Step 2a/2b/2d/2f
detail), `theme1.xml` slots (3.2, Step 2c), `document.xml` table properties (3.3, Step 5
and Step 10 D/E), `document.xml` drawing placement (3.4, Step 6 and Step 10 E). It also
introduces the first concrete need for `settings.xml`: the table left-edge rule depends
on `compatibilityMode` (< 15: border at margin + tblInd - left cell margin; 15: at
margin + tblInd). Next tier there: latent built-in styles, footnotes, numbering
overrides, footer block, A4, charts. `lastRenderedPageBreak` is explicitly *not* to be
used as layout input.

## Expected trajectory (Jaccard mean, jubarte)

| After | 76 fixtures | 398 corpus |
|---|---|---|
| today | 13.7 | 53.1 |
| Step 2 (fonts as data: altName, theme, no-split, evidence table) | ~28 | ~58 |
| Step 3 (space-before) | ~38 | ~58.5 |
| Step 4 (line box) | ~45 | 56-60 (first cut may dip) |
| Steps 5 + 6 (tables, images) | ~50 | ~60 |
| Step 7 + Step 10 D/E/F/G (features, merged cells, text boxes, notes, footers) | 60-65 | ~64 |
| docxide-pdf with fonts, for reference | 67.9 | 68.0 |

The 398 target is the 48 documents where jubarte is under 20 and docxide-pdf at or
above 60 (report section 12): Step 2 covers about 20 of them, Steps 3-4 another 10,
Step 10 the rest. The estimates come from the SHIFT column, the theme-only cluster,
the altName cluster (15 documents from ~5 to the 70-90 docxide-pdf reaches with the
same fonts) and docxide-pdf's fonts-visible numbers on the same rows. They are
estimates; Step 1 and the 50-sample gate exist so they get replaced by measurements.
