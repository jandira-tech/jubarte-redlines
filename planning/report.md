<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Why jubarte scores last on docxide-pdf's 76 fixtures and first on the 398-fixture corpus

Working report, 2026-09-05. Lives in `<jubarte-redlines>/planning/` with `plan.md`, `xml_parts_plan.md` and the regression sample; system locations are listed at the top of `plan.md`. Every number was measured in this session on this
machine (macOS, Microsoft Word installed, `mutool` from MuPDF). Scorer in both
benchmarks: docxide-pdf's own metrics (ink-pixel Jaccard at 150 DPI with no spatial
tolerance, SSIM with 8x8 windows and +/-8px vertical search, text-boundary line
match). Companion: `plan.md`.

## 0. The answer in four lines

1. **docxide-pdf's numbers in both benchmarks were measured without its fonts.** It
   cannot see Word's font folder on this machine and fell back to Arial / Helvetica /
   Times for every document. With the folder visible it scores **67.99 / 83.28**
   (mean / median Jaccard) on the 398, not 24.61 / 14.34; **67.9 / 71.8** on its own
   fixtures, not 35.1 / 20.5; and **81.90 / 88.75** on neurotic's main superdoc-scored
   table, not 65.65 / 63.97 (first place, not fifth). There is no inversion on the
   docxide-pdf side.
2. **jubarte's drop from 53 to 14 is real.** Its converter is tuned to the shape of the
   398 corpus (75% explicit Calibri, plain paragraphs, no leading heading) with
   hard-coded gates that are wrong outside it. Three of those gates account for most
   of the 76-fixture damage: theme fonts are honoured only when they resolve to Aptos
   (47 of 76 fixtures affected), space-before is dropped on the first paragraph of the
   document (42 of 76), and the line box uses per-face constants instead of font
   metrics (33 of 76).
3. **The rest is features jubarte does not implement**: preset shapes (4 of 20 drawn),
   floating tables, page borders, footnotes, image cropping and PNG alpha. 15 fixtures
   contain drawings, 10 tables, 4 footnotes.
4. **The metric amplifies everything**: Jaccard has zero tolerance, so a 1 px baseline
   offset costs 20+ points (case1: 44.7 at rest, 67.7 after a (-1,+1) px shift).
5. **jubarte never opens `fontTable.xml`**, where Word records the font it actually
   substituted (`w:altName`). 81 of 398 corpus documents carry one; docxide-pdf reads
   it first. That is the largest cluster in jubarte's bottom decile on the corpus
   (19 of 40 documents, section 13) and the reason docxide-pdf scores 70-90 on
   documents where jubarte scores under 10.

Corrected ranking, same scorer, both sets: docxide-pdf ahead on both, jubarte behind
on both, LibreOffice between them on the 76. The published neurotic README table and
the numbers in the docxide-pdf PR body are both wrong for docxide-pdf; see section 9.

## 1. The divergence, restated

| Fixture set | n | jubarte J mean / median | docxide-pdf J, as published | docxide-pdf J, fonts visible | LibreOffice J |
|---|---|---|---|---|---|
| docxide-pdf `tests/fixtures/cases` | 76 | 13.7 / 8.8 | 35.1 / 20.5 | 67.9 / 71.8 (75 cases, case13 skipped) | 46.8 / 47.0 |
| neurotic `docx_to_pdf_no_redline_docs` | 398 | 53.1 / 43.5 | 24.6 / 14.3 | 68.0 / 83.3 | not run with this scorer |

A note on n: the 398 are about 199 documents twice. 179 of the 199 "randomized"
sources are text-identical to a named source apart from an injected first line carrying
the file name, and 16 named sources duplicate another named source (checked from the
document text, 2026-09-05). Every duplicate is scored for both engines, so the direction
of every comparison here stands, but the effective sample for the engine comparison is
~200 documents, and per-document tables (sections 13, 9) show duplicate rows.

Both oracles are Word exports. Text-boundary already hinted at the cause on the 76:
jubarte's line breaks match Word on 100% of lines in 22 of the 50 fixtures where the
metric is defined while its Jaccard on those fixtures is mostly under 10. Lines break
in the right places; the ink lands elsewhere. That is a font or positioning defect,
not a layout-engine defect.

## 2. Structural checks that came back clean

- **Page size**: all 76 jubarte outputs match the reference MediaBox exactly
  (612x792, or 792x612 for case25). No dimension-mismatch errors from the scorer.
- **Page count**: 73 of 76 match. Mismatches: case6 (ref 3 / jubarte 4), case13
  (205 / 206), case51 (2 / 1). The scorer pairs pages by index up to the shorter
  count, so a one-page drift only costs the trailing page.
- **Convert failures**: 0 of 76 for jubarte, 0 of 398 for either engine.

## 3. Finding A: jubarte honours the theme body font only when it is Aptos

**Evidence in the outputs.** In 47 of 76 fixtures the reference body text is Cambria
and jubarte set it in Calibri (per-fixture table, section 7). Those fixtures carry
`w:asciiTheme="minorHAnsi"` in docDefaults with no explicit family, and their
`theme1.xml` declares `minorFont = Cambria`, `majorFont = Calibri` (the reverse of the
stock Office theme; case41 ships its `generate.py`, so these are script-built
documents). jubarte embeds *real* Calibri from Word's DFonts folder, so font discovery
is fine; the family choice is wrong.

**Evidence in the code.** `../jubarte-redlines/src/convert/mod.rs`, `apply_rfonts`:
the theme is parsed correctly (`load_theme`, lines 938-990), but the minor slot is
applied only when the resolved face starts with "aptos":

```rust
} else if slot.contains("minor")
    && let Some(face) = theme.minor.as_deref()
    && face.to_ascii_lowercase().replace([' ', '-'], "").starts_with("aptos")
{
    style.family = face.to_string();
```

The comment above it says why: "Do not resolve Cambria/serif minor ... Word Quartz
does paint Cambria for table_bookmark_end / file_134, but applying it (mini 90) also
retargeted file_2 / file_41 onto Cambria size x1.15 boxes ... Keep the Aptos-only
gate." Honouring the theme was tried, it exposed the line-box defect in Finding D on
two corpus documents, and the gate was kept because the mini-set score moved the
wrong way. The gate is an overfit that hides a second bug.

**Control group on the 398.** Grouping the corpus by how the body font resolves:

| Body font resolution | n | jubarte J mean / median | jubarte text-boundary | docxide-pdf J (font-starved) |
|---|---|---|---|---|
| explicit `w:ascii="Calibri"` | 298 | 61.8 / 74.4 | 96.9 | 26.8 |
| `minorHAnsi` -> theme Calibri | 24 | 43.6 / 41.0 | 62.4 | 16.8 |
| explicit Aptos | 16 | 29.7 / 29.5 | 59.4 | 10.9 |
| explicit Inter | 16 | 14.4 / 13.2 | 50.4 | 9.5 |
| `minorHAnsi` -> theme **Cambria** | 12 | **11.2 / 12.4** | **81.3** | 15.1 |
| `minorHAnsi` -> theme Aptos | 8 | 32.1 / 26.2 | 59.9 | 22.5 |
| explicit Times New Roman | 6 | 12.1 / 11.4 | 44.3 | 23.0 |
| explicit Arial | 6 | 53.0 / 56.8 | 99.0 | 60.6 |
| no docDefaults font | 5 | 4.3 / 4.0 | 20.0 | 9.4 |

The 12 corpus documents that resolve to Cambria score like the docxide fixtures do
(11.2), with line breaks 81-90% right and ink wrong. All 12 have Cambria in the
oracle and Calibri in jubarte's output. Same defect, same signature, both corpora.
This table is also the clearest picture of the overfit: jubarte is at 62 on the 298
explicit-Calibri documents and at 25 on the other 99.

## 4. Finding B: docxide-pdf is font-starved on this machine, in both benchmarks

docxide-pdf embedded Arial / Helvetica / Times New Roman in every one of the 76
outputs and in the corpus run, including documents whose reference is Aptos, Calibri
or Cambria. `fc-list` finds none of those three on the system; the only copies live in
`/Applications/Microsoft Word.app/Contents/Resources/DFonts`. jubarte hard-codes that
path (`font.rs`, `system_override`, "Deliberately macOS-only"); docxide-pdf's
`fonts/discovery.rs` searches `/Library/Fonts`, `/System/Library/Fonts`,
`Supplemental` and `~/Library/Fonts`, plus whatever `DOCXSIDE_FONTS` names.

Re-measured with `DOCXSIDE_FONTS` pointing at DFonts (same binary, same scorer):

| Set | as published | fonts visible | improved / worse |
|---|---|---|---|
| docxide 76 fixtures (75, case13 skipped) | 35.4 / 20.7 | 67.9 / 71.8 | 60 / 0 |
| corpus 1-in-7 sample (57 docs) | 21.9 / 14.4 | 62.1 / 72.0 | 42 / 3 |
| **full 398** (`results/docxide_metrics_docxide_dfonts.json`) | 24.61 / 14.34; SSIM 43.80 / 35.35; pass J>=20 100, S>=75 55 | **67.99 / 83.28; SSIM 78.63 / 97.66; pass 334 / 270** | 0 failures, 15 page-count mismatches |

The same environment defect depressed the main neurotic table (superdoc fused scorer,
144 DPI). Re-run with fonts visible over all 398
(`results/docx_to_pdf_no_redline_docxide_dfonts.json`): **81.90 mean / 88.75 median**
against the published 65.65 / 63.97, 0 failures. That is above every row in the
published table (office2pdf 73.82 / 84.34, libreoffice_convert_rust 73.98 / 75.26,
jubarte 66.25 / 67.39). Upstream's own
`tests/baselines.json` (219 entries, cases + samples) averages 0.50 Jaccard, consistent
with the fonts-visible figure here, not the published one.

## 5. Finding C: jubarte drops space-before on the first paragraph of the document

**Evidence.** On 42 of 76 fixtures jubarte's first text line sits 21, 26, 38 or 50 px
higher than Word's (page-1 band analysis, section 7). Those are exactly 200, 240, 360
and 480 twips: Heading 2 / Heading 1 `w:before` in the two style templates the
fixtures use. Word applies space-before to the first paragraph of a document; jubarte
suppresses it whenever it believes it is at a page top:

```rust
// mod.rs ~6503
if !self.at_page_top || self.last_break_was_section {
    self.y -= style.before;
}
```

`at_page_top` is true at document start, so the very first paragraph loses its
spacing, and every line on page 1 (often the whole document) is offset. Jaccard then
scores the page as if the text were absent: case5 goes 6.6 -> 11.8 only after a 50 px
shift, case2 4.0 -> 34.9 after 38 px, case76 2.5 -> 32.1 after 38 px.

**Control group on the 398.** Corpus documents whose first paragraph is a Heading 1:

| stem | before (twips) | Word first ink row | jubarte | delta px | jubarte J |
|---|---|---|---|---|---|
| source__multi_section | 480 | 208 | 158 | -50 | 18.1 |
| source__page_numbering_examples | 480 | 208 | 158 | -50 | 18.1 |
| source_randomized__file_65 / file_92 | 480 | 208 | 158 | -50 | 16.0 |
| source__anchor_images | 360 | 178 | 150 | -28 | 14.3 |
| source_randomized__file_197 | 360 | 178 | 150 | -28 | 13.5 |
| five documents that open with a plain paragraph | 0 | 157 / 157 / 81 / 156 / 156 | +0 / +0 / +1 / -2 / +0 | | 85.3 / 90.7 / 27.6 / 8.1 / 17.4 |

Same 50 px and 28 px as the fixtures. Only ~6 of 398 corpus documents open with a
heading, so the defect barely registers there; ~42 of 76 docxide fixtures do.

## 6. Finding D: the line box is a set of per-face constants, not font metrics

`mod.rs` ~6540-6560, the per-line advance:

```rust
let line_box = if let Some(exact) = style.line_exact { exact }
    else if is_toc_style(style) { size * line_mult.max(1.15) }
    else if is_word_heading_style(style) && line_mult >= 1.14 {
        if face.is_arial() { metrics.single_line_pt(size) * line_mult }
        else { metrics.single_line_pt(size) } }
    else if compact_title || face.is_cambria()
        || ((face.is_arial() || face.is_times()) && line_mult >= 1.14) { size * line_mult }
    else { metrics.single_line_pt(size) * line_mult };
```

Word's rule is one line: font line height (ascender + descender + line gap, from the
face's own tables) x size x (`w:line` / 240) for `auto`. jubarte uses that rule only for
the default branch (in practice Calibri/Carlito and Aptos); Cambria, Arial and Times
get `size x multiplier` with no font metrics (Cambria 11 pt at 1.15 -> 12.65 pt where
Word lays 14.9), and headings in non-Arial faces lose the multiplier entirely. The
code comments record each branch as a mini-set trade ("Mini 396 on the 60-stem: NR
+0.048 ... redline file_27_file_28 -2.85 ... Keep the Aptos-only gate").

Effect on the 76: 33 fixtures show a per-line pitch difference of 1.5 px or more on
page 1. Clean single-font examples: case7 (Georgia / Verdana / Times / Courier
showcase: Word 43.5 px per band, jubarte 41.0, cumulative drift down the page; docxide-pdf
with system fonts lands at 0 px offset and 74.0 J), case62 (Arial 11: 36 vs 30 px),
case52 (41 vs 33), case24 (37 vs 53), case4 (Calibri heading line 5 px short, lines
drift from the second paragraph on). Effect on the 398: explicit Times New Roman
documents score 12.1, explicit Arial 53.0, explicit Calibri 61.8.

## 7. Finding E: features jubarte does not implement (or implements partially)

Occurrences of the OOXML element name in `src/convert/mod.rs` (a proxy, then checked
visually):

| Feature | element | occurrences | fixtures hit | what the render shows |
|---|---|---|---|---|
| Preset shapes | `prstGeom` | 4 | case34, 35 (20 shapes), 36, 38 (6), 37, 41 | case34: 4 of 20 shapes drawn, rectangles and one arrow only, all in the first row; docxide-pdf's geometry engine scores 98.7 |
| Floating tables | `tblpPr` | 0 | case46 | table dropped into flow, text does not wrap beside it |
| Page borders | `pgBorders` | 0 | case68 | no border; text otherwise identical (J 6.0 because the border is most of the ink) |
| Footnotes | `footnoteReference` | 0 | case18, 74, 75, 76 | footnote text absent at page bottom; combined with Finding C |
| Image crop | `a:srcRect` | 0 | case78 | all four pictures drawn uncropped and at the wrong size; docxide-pdf 99.6 |
| PNG alpha | `SMask` / alpha | 0 | case12 | gradient with transparent bottom drawn opaque, plus the inline image placed 55 px lower than Word |
| Embedded fonts | `w:embedRegular` / `.odttf` | 0 | case8 | the reference embeds a custom face (`___WRD_EMBED_SUB_1437`, PressStart2P); docxide-pdf loads it (48.1 / 94.1 SSIM even font-starved), jubarte substitutes and its first line lands 19 px low |
| Text boxes / SmartArt | `wps:txbx`, `dgm:` | 1 / 0 | case60 | two solid boxes where Word has a quarter-circle diagram and a 2x2 of rounded tiles |
| Charts | `c:chart` | 15 | case29, 30, 31, 56, 57, 59 | partial; case56/57 J 15.9 / 13.6 vs docxide-pdf 90.3 / 83.4 |
| Table left edge | `tblInd` / cell margin | - | case6, 15, 40, 45, 46, 51, 55, 61, 67 | jubarte's cell text starts 11-12 px (5.4 pt = the default 108-twip cell margin) right of Word's: Word pulls the table left by the cell margin so cell text aligns with body text; jubarte does not |

## 8. Finding F: the metric

Jaccard on ink pixels at 150 DPI has no tolerance. case1 is a one-paragraph Aptos
document that jubarte lays out correctly: 44.7 at rest, 67.7 after shifting one pixel
left and one down. Every finding above is amplified by this; it is also why SSIM
(with its 8 px vertical search) is 2-4x Jaccard for jubarte on most rows below. The
scorer is docxide-pdf's, vendored verbatim, and it is not the cause of anything here,
but a reader of the tables should know that 10 Jaccard points can be one pixel.

## 9. Per-fixture table (all 76 docxide fixtures with a Word reference)

Columns: `jub J/S/T` = jubarte Jaccard / SSIM / text-boundary; `dx J` = docxide-pdf
as measured in `comparison/work/manifest.json` (font-starved, see Finding B); `LO J` =
LibreOffice. `1st line delta` = jubarte's first ink row minus Word's on page 1 at
150 DPI (negative = jubarte too high). `pitch` = median distance between text bands on
page 1 (Word / jubarte). Diagnosis codes: **FONT theme** = Finding A; **TOP** =
Finding C; **PITCH** = Finding D; **LEFT** = first ink column differs by 5 px or more;
**SHIFT** = Jaccard after the best global (dx,dy) shift within +/-20/40 px, when it
gains 10 points or more (a pure offset); **PAGES** = page count mismatch; the feature
codes come from the source DOCX. Generated by `fixture_table.py` from this session's
band analysis; `nan` pitch means the page has fewer than three text bands.

| case | pages ref/jub | jub J | jub S | jub T | dx J | LO J | ref fonts | jubarte fonts | 1st line Δpx | pitch ref/jub px | diagnosis |
|---|---|---|---|---|---|---|---|---|---|---|---|
| case1 | 1/1 | 44.7 | 61.2 | 100.0 | 33.9 | 29.6 | Aptos | Aptos | -1 | nan/nan | SHIFT J 44.7→67.7 at (-1,1) |
| case2 | 1/1 | 4.0 | 10.8 | 100.0 | 20.4 | 22.8 | Aptos/AptosDisplay | Aptos/AptosDisplay | -38 | 63.0/56.0 | TOP first line 38px too high; PITCH -7.0px/line; SHIFT J 4.0→34.9 at (0,38) |
| case3 | 1/1 | 0.9 | 11.9 | 60.0 | 7.7 | 6.8 | Aptos/AptosDisplay/ArialMT/SymbolMT | Aptos/AptosDisplay/SymbolMT | -38 | 36.0/36.0 | FONT missing arial; TOP first line 38px too high; NUMBERING |
| case4 | 2/2 | 16.6 | 44.1 | 49.1 | 6.2 | 16.9 | Calibri/Calibri-Bold | Calibri/Calibri-Bold | +0 | 35.0/35.0 |  |
| case5 | 2/2 | 29.3 | 45.0 | 100.0 | 6.1 | 5.6 | Aptos/Calibri-Bold/Calibri-BoldItalic | Aptos/Calibri-Bold/Calibri-BoldItalic | -50 | 35.0/35.0 | TOP first line 50px too high |
| case6 | 3/4 | 9.1 | 37.3 | 65.4 | 8.6 | 7.4 | Aptos/Calibri-Bold | Aptos/Calibri-Bold | -50 | 35.0/35.0 | TOP first line 50px too high; LEFT +12px; PAGES ref 3 / jub 4; TABLE |
| case7 | 1/1 | 9.8 | 22.1 | 100.0 | 74.0 | 62.6 | Arial-BoldItalicMT/Arial-BoldMT/ArialMT/Cambria/CourierNewPSMT/Georgia/Georgia-Bold/Georgia-Italic/TimesNewRomanPS-BoldMT/TimesNewRomanPSMT/Verdana | Arial-BoldItalicMT/Arial-BoldMT/ArialMT/CourierNewPSMT/Georgia/Georgia-Bold/Georgia-Italic/TimesNewRomanPS-BoldMT/TimesNewRomanPSMT/Verdana | -3 | 43.5/41.0 | FONT missing cambria; PITCH -2.5px/line |
| case8 | 1/1 | 7.7 | 11.3 | - | 48.1 | 76.3 | ArialMT/Cambria/___WRD_EMBED_SUB_1437 | ArialMT/Calibri | +19 | 24.0/29.0 | FONT theme minor Cambria→Calibri; TOP first line 19px too low; PITCH +5.0px/line |
| case9 | 1/1 | 23.7 | 34.6 | 100.0 | 22.4 | 64.6 | Cambria/Cambria-Bold/Cambria-BoldItalic/Cambria-Italic | Calibri/Calibri-Bold/Calibri-BoldItalic/Calibri-Italic | +0 | 52.0/53.0 | FONT theme minor Cambria→Calibri |
| case10 | 3/3 | 14.4 | 52.2 | 100.0 | 8.2 | 9.8 | Aptos/Aptos-Bold/Aptos-Italic/Calibri-Bold | Aptos/Aptos-Bold/Aptos-Italic/Calibri-Bold | -50 | 51.0/52.0 | TOP first line 50px too high; SHIFT J 5.2→19.3 at (0,41); SUPSUB |
| case11 | 2/2 | 7.1 | 20.0 | 23.9 | 7.5 | 45.5 | Calibri-Bold/Cambria/Cambria-Bold | Calibri/Calibri-Bold | +1 | 31.0/32.0 | FONT theme minor Cambria→Calibri |
| case12 | 1/1 | 51.0 | 68.2 | 100.0 | 96.7 | 90.3 | Cambria | Calibri | +0 | nan/nan | FONT theme minor Cambria→Calibri; SHIFT J 51.0→63.0 at (0,-41); DRAWING |
| case13 | 205/206 | 7.5 | 19.5 | 0.2 | 7.5 | 46.2 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 31.0/32.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PAGES ref 205 / jub 206 |
| case14 | 1/1 | 64.2 | 88.6 | 100.0 | 11.4 | 14.7 | Aptos | Aptos | -1 | 55.0/56.0 |  |
| case15 | 1/1 | 4.5 | 27.0 | - | 6.6 | 52.0 | Calibri/Calibri-Bold | Calibri/Calibri-Bold | -21 | nan/17.0 | TOP first line 21px too high; LEFT +12px; SHIFT J 4.5→14.6 at (-11,40); TABLE |
| case16 | 2/2 | 40.3 | 50.2 | 0.0 | 89.4 | 95.0 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 31.0/32.5 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH +1.5px/line; DRAWING |
| case17 | 1/1 | 5.3 | 18.9 | 83.3 | 7.7 | 7.9 | Aptos/Aptos-Bold/CourierNewPSMT | Aptos/Aptos-Bold/CourierNewPSMT | -26 | 43.0/42.0 | TOP first line 26px too high; LEFT +6px |
| case18 | 1/1 | 5.5 | 15.8 | - | 8.5 | 7.3 | Aptos/Aptos-Bold/Aptos-Italic/AptosDisplay | Aptos/Aptos-Bold/AptosDisplay | -38 | 36.0/36.0 | TOP first line 38px too high; SHIFT J 5.5→17.8 at (-1,41); FOOTNOTE |
| case19 | 1/1 | 5.5 | 19.6 | 100.0 | 13.6 | 46.9 | ArialMT/Calibri-Bold/Cambria | Calibri/Calibri-Bold | -21 | 52.0/53.0 | FONT theme minor Cambria→Calibri; TOP first line 21px too high; SHIFT J 5.5→21.5 at (0,21); NUMBERING |
| case20 | 2/2 | 5.6 | 17.3 | 100.0 | 7.5 | 28.7 | ArialMT/Calibri-Bold/Cambria | Calibri/Calibri-Bold | -21 | 52.0/53.0 | FONT theme minor Cambria→Calibri; TOP first line 21px too high; NUMBERING |
| case21 | 1/1 | 5.2 | 12.2 | - | 15.4 | 29.9 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 31.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH +2.0px/line |
| case22 | 2/2 | 5.6 | 14.7 | - | 6.7 | 13.9 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | +8 | nan/33.0 | FONT theme minor Cambria→Calibri |
| case23 | 1/1 | 3.6 | 10.9 | 28.6 | 7.1 | 44.3 | Calibri-Bold/Cambria/Cambria-Bold | Calibri/Calibri-Bold | -50 | 51.0/53.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH +2.0px/line |
| case24 | 3/3 | 6.5 | 17.5 | 5.3 | 8.6 | 51.4 | Calibri-Bold/Cambria/Cambria-Bold | Calibri/Calibri-Bold | -50 | 37.0/53.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH +16.0px/line |
| case25 | 4/4 | 2.0 | 11.3 | 20.0 | 15.3 | 60.5 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 35.5/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -2.5px/line; SECTIONS/COLS |
| case26 | 4/4 | 1.5 | 12.6 | 40.0 | 16.8 | 61.5 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 38.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -5.0px/line; SECTIONS/COLS |
| case27 | 2/2 | 48.3 | 48.3 | 100.0 | 96.9 | 97.3 | Arial-BoldMT/ArialMT | Arial-BoldMT | -1 | nan/nan | DRAWING |
| case28 | 3/3 | 4.1 | 13.8 | 50.0 | 14.4 | 60.1 | Calibri-Bold/Cambria/Cambria-Italic | Calibri/Calibri-Bold/Calibri-Italic | -50 | 33.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; SECTIONS/COLS |
| case29 | 2/2 | 30.8 | 40.6 | - | 70.0 | 74.6 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 41.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -8.0px/line |
| case30 | 2/2 | 34.1 | 51.4 | - | 80.2 | 86.0 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 39.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -6.0px/line |
| case31 | 2/2 | 10.4 | 28.9 | - | 54.6 | 47.2 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 39.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -6.0px/line |
| case32 | 1/1 | 9.8 | 26.2 | 37.5 | 14.6 | 52.8 | Cambria | Calibri | +0 | 31.0/32.0 | FONT theme minor Cambria→Calibri; TABLE |
| case33 | 1/1 | 6.9 | 31.3 | 72.7 | 7.9 | 18.5 | ArialMT/Calibri/Calibri-Bold/SymbolMT | Calibri/Calibri-Bold/SymbolMT | +0 | 52.0/52.0 | FONT missing arial; SHIFT J 6.9→43.0 at (0,-12) |
| case34 | 1/1 | 10.9 | 50.1 | - | 98.7 | 99.1 | Cambria | Calibri | -60 | nan/nan | FONT theme minor Cambria→Calibri; TOP first line 60px too high; DRAWING |
| case35 | 1/1 | 12.3 | 46.0 | - | 98.9 | 98.5 | Cambria | Calibri | -11 | nan/nan | FONT theme minor Cambria→Calibri; LEFT +7px; DRAWING |
| case36 | 1/1 | 2.9 | 51.2 | - | 99.5 | 81.6 | Cambria | Calibri | -202 | nan/nan | FONT theme minor Cambria→Calibri; TOP first line 202px too high; DRAWING |
| case37 | 1/1 | 0.0 | 0.0 | - | 23.2 | 0.0 | Cambria | Calibri |  | nan/nan | FONT theme minor Cambria→Calibri; LEFT -155px; DRAWING |
| case38 | 1/1 | 0.0 | 53.7 | - | 99.3 | 92.6 | Cambria/Cambria-Bold | Calibri-Bold | -232 | nan/nan | FONT theme minor Cambria→Calibri; TOP first line 232px too high; DRAWING |
| case39 | 1/1 | 13.8 | 26.3 | 100.0 | 9.2 | 51.6 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | +0 | 45.0/43.0 | FONT theme minor Cambria→Calibri; PITCH -2.0px/line |
| case40 | 2/2 | 9.0 | 20.5 | 16.7 | 12.5 | 38.0 | Cambria | Calibri | +0 | 31.0/32.0 | FONT theme minor Cambria→Calibri; LEFT +11px; TABLE |
| case41 | 7/7 | 9.3 | 42.4 | 0.0 | 63.2 | 81.4 | Cambria | Calibri | +0 | 31.0/32.0 | FONT theme minor Cambria→Calibri; DRAWING |
| case42 | 1/1 | 26.9 | 18.8 | - | 46.4 | 69.2 | TimesNewRomanPSMT | TimesNewRomanPSMT | -2 | 29.0/26.0 | PITCH -3.0px/line; DRAWING |
| case43 | 1/1 | 7.1 | 11.8 | 66.7 | 28.3 | 2.7 | Cambria/Cambria-Bold | Calibri-Bold | +1 | nan/nan | FONT theme minor Cambria→Calibri |
| case44 | 3/3 | 5.5 | 14.9 | 100.0 | 26.5 | 4.4 | Cambria/Cambria-Bold | Calibri-Bold | +1 | nan/nan | FONT theme minor Cambria→Calibri |
| case45 | 2/2 | 4.9 | 15.5 | 6.5 | 7.6 | 40.8 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +1 | 31.0/32.0 | FONT theme minor Cambria→Calibri; LEFT +11px; TABLE |
| case46 | 14/14 | 5.8 | 19.2 | 5.3 | 9.2 | 48.1 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +1 | 31.0/32.0 | FONT theme minor Cambria→Calibri; LEFT +11px; TABLE |
| case47 | 16/16 | 35.7 | 31.3 | 4.8 | 34.5 | 71.5 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +0 | 31.0/32.0 | FONT theme minor Cambria→Calibri |
| case48 | 1/1 | 7.1 | 23.7 | 100.0 | 62.7 | 16.5 | Cambria/TimesNewRomanPS-BoldMT/TimesNewRomanPSMT | TimesNewRomanPS-BoldMT/TimesNewRomanPSMT | -2 | 33.0/29.0 | FONT missing cambria; PITCH -4.0px/line |
| case49 | 3/3 | 6.9 | 21.4 | 47.0 | 59.2 | 22.3 | Calibri-Bold/Cambria/TimesNewRomanPS-BoldMT/TimesNewRomanPSMT | TimesNewRomanPS-BoldMT/TimesNewRomanPSMT | -51 | 37.5/29.0 | FONT missing calibri/cambria; TOP first line 51px too high; PITCH -8.5px/line; FIELD |
| case50 | 2/2 | 4.7 | 12.8 | 90.9 | 8.2 | 54.3 | Arial-BoldItalicMT/Calibri-Bold/Cambria/CourierNewPS-BoldMT/Georgia/Georgia-Bold/Georgia-BoldItalic/Georgia-Italic/TimesNewRomanPS-BoldItalicMT/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT | Arial-BoldItalicMT/Calibri/Calibri-Bold/Calibri-BoldItalic/CourierNewPS-BoldMT/Georgia/TimesNewRomanPS-BoldItalicMT/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT | -50 | 46.0/41.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -5.0px/line |
| case51 | 2/1 | 3.0 | 13.5 | - | 8.6 | 9.7 | Calibri-Bold/Cambria/Cambria-Bold | Calibri/Calibri-Bold | -50 | 39.0/33.5 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -5.5px/line; LEFT +11px; PAGES ref 2 / jub 1; TABLE |
| case52 | 2/2 | 19.7 | 39.7 | - | 64.3 | 69.0 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 41.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH -8.0px/line |
| case53 | 2/2 | 18.7 | 20.4 | - | 50.1 | 48.7 | Calibri-Bold/Cambria | Calibri/Calibri-Bold | -50 | 31.0/33.0 | FONT theme minor Cambria→Calibri; TOP first line 50px too high; PITCH +2.0px/line |
| case54 | 2/2 | 25.2 | 56.9 | 77.0 | 19.7 | 34.5 | ArialMT/Calibri/TimesNewRomanPSMT | ArialMT/Calibri/TimesNewRomanPSMT | +0 | 31.0/30.0 |  |
| case55 | 1/1 | 15.9 | 41.4 | 100.0 | 32.9 | 44.4 | Calibri/Calibri-Bold | Calibri/Calibri-Bold | -50 | 39.0/33.0 | TOP first line 50px too high; PITCH -6.0px/line; LEFT +12px; SHIFT J 15.9→34.1 at (-11,41); TABLE |
| case56 | 3/3 | 15.9 | 70.5 | 100.0 | 90.3 | 98.0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +0 | 62.0/64.0 | FONT theme minor Cambria→Calibri; PITCH +2.0px/line; DRAWING |
| case57 | 5/5 | 13.6 | 31.7 | 100.0 | 83.4 | 83.7 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +0 | 62.0/64.0 | FONT theme minor Cambria→Calibri; PITCH +2.0px/line; DRAWING |
| case59 | 2/2 | 37.4 | 63.8 | - | 92.8 | 92.2 | Cambria | Calibri | +116 | nan/nan | FONT theme minor Cambria→Calibri; TOP first line 116px too low; DRAWING |
| case60 | 1/1 | 32.3 | 51.5 | - | 70.6 | 82.5 | Aptos/ArialMT/Calibri/Calibri-Bold/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | Calibri | +45 | nan/nan | FONT missing aptos/arial/timesnewroman; TOP first line 45px too low; LEFT +184px; DRAWING |
| case61 | 1/1 | 4.4 | 15.8 | - | 10.8 | 24.1 | ArialMT/Calibri-Bold | ArialMT/Calibri-Bold | -50 | 61.0/55.0 | TOP first line 50px too high; PITCH -6.0px/line; LEFT +12px; TABLE |
| case62 | 1/1 | 8.7 | 28.8 | 100.0 | 8.7 | 32.8 | ArialMT/Calibri-Bold | ArialMT/Calibri-Bold | -50 | 36.0/30.0 | TOP first line 50px too high; PITCH -6.0px/line |
| case63 | 1/1 | 1.3 | 8.9 | - | 11.1 | 0.2 | Aptos/TimesNewRomanPS-BoldMT | Aptos | -157 | 32.0/55.5 | FONT missing timesnewroman; TOP first line 157px too high; PITCH +23.5px/line; LEFT +43px |
| case64 | 3/3 | 2.5 | 11.4 | 100.0 | 7.7 | 3.2 | Aptos/ArialMT/Calibri-Bold/TimesNewRomanPS-BoldMT | Aptos/Calibri-Bold | -194 | 27.0/35.0 | FONT missing arial/timesnewroman; TOP first line 194px too high; PITCH +8.0px/line; LEFT +42px |
| case66 | 1/1 | 22.4 | 21.2 | - | 26.8 | 43.8 | Cambria/Cambria-Bold | Calibri-Bold | +1 | 68.0/69.0 | FONT theme minor Cambria→Calibri |
| case67 | 1/1 | 12.1 | 11.4 | - | 20.7 | 36.8 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +0 | 60.5/54.0 | FONT theme minor Cambria→Calibri; PITCH -6.5px/line; LEFT +12px; TABLE |
| case68 | 1/1 | 6.0 | 11.2 | 100.0 | 71.2 | 85.2 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +111 | nan/74.0 | FONT theme minor Cambria→Calibri; TOP first line 111px too low; LEFT +137px |
| case69 | 1/1 | 6.9 | 11.7 | 33.3 | 26.2 | 0.0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | +15 | 31.0/32.0 | FONT theme minor Cambria→Calibri; TOP first line 15px too low; LEFT -24px; SHIFT J 6.9→26.1 at (17,-17) |
| case70 | 1/1 | 9.2 | 21.3 | 0.0 | 15.9 | 59.1 | Cambria | Calibri | +0 | 52.0/53.0 | FONT theme minor Cambria→Calibri; LEFT +62px |
| case71 | 1/1 | 18.8 | 33.9 | 50.0 | 22.3 | 81.5 | ArialMT/Cambria/Cambria-Bold | Calibri/Calibri-Bold | +1 | 52.0/53.0 | FONT theme minor Cambria→Calibri; NUMBERING |
| case72 | 1/1 | 20.7 | 36.4 | 71.4 | 22.6 | 74.3 | ArialMT/Cambria/Cambria-Bold | Calibri/Calibri-Bold | +1 | 52.0/53.0 | FONT theme minor Cambria→Calibri; NUMBERING |
| case73 | 1/1 | 20.9 | 32.6 | 28.6 | 27.9 | 47.6 | Arial-BoldMT/Cambria/Cambria-Bold | Calibri-Bold | +1 | 60.5/61.5 | FONT theme minor Cambria→Calibri |
| case74 | 1/1 | 5.5 | 13.9 | - | 13.7 | 16.2 | Aptos/Aptos-Bold | Aptos/Aptos-Bold | -38 | 39.5/52.0 | TOP first line 38px too high; PITCH +12.5px/line; SHIFT J 5.5→28.0 at (0,38); FOOTNOTE |
| case75 | 1/1 | 5.1 | 14.4 | - | 13.4 | 16.2 | Aptos/Aptos-Bold | Aptos/Aptos-Bold | -38 | 39.0/52.0 | TOP first line 38px too high; PITCH +13.0px/line; SHIFT J 5.1→27.6 at (0,38); FOOTNOTE |
| case76 | 1/1 | 2.5 | 6.2 | - | 14.8 | 21.6 | Aptos/Aptos-Bold | Aptos/Aptos-Bold | -38 | 43.0/55.0 | TOP first line 38px too high; PITCH +12.0px/line; SHIFT J 2.5→32.1 at (0,38); FOOTNOTE |
| case77 | 1/1 | 9.0 | 26.2 | 72.2 | 50.9 | 47.9 | Arial-BoldMT/ArialMT/Cambria/MS-Mincho | Arial-BoldMT/ArialMT | -2 | 47.5/44.0 | FONT missing cambria/ms-mincho; PITCH -3.5px/line |
| case78 | 2/2 | 18.0 | 45.9 | 100.0 | 99.6 | 98.1 | Arial-BoldMT/ArialMT | Arial-BoldMT | -1 | 55.0/50.0 | PITCH -5.0px/line; DRAWING |

## 10. Notes by cluster (what the table cannot say)

**Theme font only (Finding A, nothing else flagged).** case9, case11, case22, case43,
case44, case47, case66, case71, case72, case73, plus case32 / case40 (tables), case39
(pitch), case12 / case41 (drawings). Their first lines land within 1 px of Word's;
the glyphs are the only difference. case11 and case47 also break lines differently
(text-boundary 23.9 / 4.8) because Cambria is wider than Calibri, so the wrap will
fix itself with the font. These are the cheapest points on the board: docxide-pdf
with fonts scores 72.7 / 87.4 / 90.3 on case11 / case71 / case73.

**Document-start space-before (Finding C).** The offset is quantised: 50 px on the
python-docx-template fixtures (Heading 1 `before=480`: case5, case6, case10, case13,
case16, case21, case23, case24, case25, case26, case28, case29, case30, case31, case50,
case51, case52, case53, case55, case61, case62, and case49 at 51), 38 px on the Word-365-template fixtures (Heading 1
`before=360`: case2, case3, case18, case74, case75, case76), 21 px where the document
opens with a Heading 2 (`before=200`: case15, case19, case20), 26 px on case17 (a
hand-built docx with no styles.xml). The TOP values on case34 / case36 / case38 (-60 / -202 / -232) are
shape positions, not text (those pages have no text bands), and case35 (-11) / case37
(LEFT -155) are the same. Positive offsets in the TOP column are not this defect: case8 (+19, embedded-font fixture), case59 (+116) and case60 (+45) are drawing
placement, case68 (+111) is the missing page border, case69 (+15) is a positioned
element. case63 / case64 (-157 / -194) are the scaled references described below.

**Line box (Finding D).** case7 is the reference example: six families, one column,
no headings after the title; Word 43.5 px per band, jubarte 41.0, and docxide-pdf with
system fonts reproduces Word at 0 px offset (74.0 J). Clean Arial rows: case62 (36 vs
30 px), case61 (61 vs 55). Calibri rows where headings shrink the gap: case4, case52
(41 vs 33), case53, case21. Footnote rows show +12 to +13 px because the footnote
block is missing and the band detector pairs body lines with what Word puts lower.

**Drawings (Finding E).** case34 / case35: 20 preset shapes each, jubarte draws 4
(rectangles and one arrow) in the wrong row; case36 / case38: 6 shapes, none drawn
(J 2.9 / 0.0; SSIM ~50 because white windows are skipped). case37: a single drawing
that every engine misses (docxide-pdf 23.2, LibreOffice 0.0). case41: 7 anchored
drawings, text-boundary 0.0 because the wrapped text never lines up. case12: PNG with
transparent bottom drawn opaque and 55 px low. case78: `a:srcRect` cropping ignored,
all four pictures full-size (docxide-pdf 99.6). case16 / case27 / case42: inline
pictures, jubarte 40.3 / 48.3 / 26.9. case56 / case57 / case59: charts, jubarte 15.9 /
13.6 / 37.4 against docxide-pdf 90.3 / 83.4 / 92.8. case60: text boxes and a SmartArt
diagram rendered as two solid rectangles.

**Tables.** case6, case15, case40, case45, case46, case51, case55, case61, case67
all show cell text 11-12 px (5.4 pt, the default 108-twip cell margin) right of
Word's. case46 is a floating table (`tblpPr`) that Word wraps text around; jubarte
stacks it. case51 packs ten stacked tables onto one page where Word needs two
(row heights too small); case6 overflows to a fourth page where Word uses three.

**Footnotes.** case18, case74, case75, case76: footnote text absent, separator absent;
combined with the 38 px document-start offset, Jaccard is 2.5-5.5.

**Numbering.** case3, case19, case20, case71, case72: bullets and numbers are present
and the pitch matches; the damage is Finding A / C. case3 additionally lacks Arial
(the bullet glyph face), and its two lists are separated by less than Word's gap.

**Missing families (not the theme gate).** case7, case48, case49, case77: the Cambria
heading face is absent from jubarte's output (title set in a Times face). case3,
case33: Arial absent (bullet / hyperlink runs). case60, case63, case64: Aptos / Arial /
Times absent inside text boxes and comment fixtures.

**Not converter defects, or not jubarte's alone.** case63 and case64: the reference
content starts 70 pt from the left page edge with a 90 pt margin and 313 / 352 px from
the top; the page was scaled to leave a comment-balloon column, which is Word's
"print with markup" export. docxide-pdf scores 11.1 / 7.7 and LibreOffice 0.2 / 3.2
there. case13: 205 pages, skiplisted upstream as too slow; jubarte and docxide-pdf
both 7.5, LibreOffice 46.2. case14: jubarte's best result (64.2), one Aptos paragraph
with hyperlinks, nothing to fix. case1: correct layout, 1 px off, 44.7 -> 67.7 after
the shift (Finding F).

## 11. What this means for the numbers already published

- **neurotic README, `docxide_metrics` section.** The docxide-pdf row (24.61 / 14.34,
  pass 100 / 55) is a font-discovery artifact. Re-measured with Word's fonts visible:
  67.99 / 83.28, SSIM 78.63 / 97.66, text-boundary 87.23 / 100.00, pass 334 / 270,
  0 failures (`results/docxide_metrics_docxide_dfonts.json`, full 398). With that row
  docxide-pdf ranks first and jubarte second by every column except text-boundary
  (87.55 vs 87.23, a tie).
- **neurotic README, main `docx_to_pdf_no_redline_docs` table.** docxide-pdf 65.65 /
  63.97 was measured the same font-starved way. With fonts: **81.90 / 88.75**
  (`results/docx_to_pdf_no_redline_docxide_dfonts.json`, 398 / 398, 0 failures), which
  moves docxide-pdf from 5th to 1st in that table, ahead of office2pdf and
  libreoffice_convert_rust.
- **docxide-pdf PR #1 on arthrod/docxide-pdf.** The body quotes docxide-pdf at
  35.1 / 20.5 on its own fixtures. The maintainer's engine measured with their fonts is
  67.9 / 71.8 over the same 75 cases (case13 skipped). The PR needs a correction; that
  is an outward-facing edit and is left for Arthur to make or approve.
- **`comparison/index.html` in this checkout.** The "docxide-pdf" column was rendered
  font-starved; rebuild with `DOCXSIDE_FONTS` set before deploying it anywhere.
- **jubarte's 53.10 / 43.50 on the 398 stands.** jubarte hard-codes the DFonts path,
  so it always had the fonts. Its 13.7 / 8.8 on the 76 also stands.

The corrected picture is not an inversion. docxide-pdf is ahead on both sets, jubarte
is behind on both, and jubarte's gap is wider on the 76 because that set is built from
exactly the constructs jubarte's tuning excluded: theme-resolved Cambria body text,
documents that open with a heading, non-Calibri faces, shapes, floating tables,
footnotes, page borders and cropped images.

## 12. Distribution of both converters on the 398 (docxide scorer, fonts visible for both)

Per-document Jaccard, jubarte 0.8.0 vs docxide-pdf 0.17.0 with `DOCXSIDE_FONTS` set.

| percentile | jubarte | docxide-pdf |
|---|---|---|
| p5 | 6.9 | 8.0 |
| p10 | 11.4 | 15.5 |
| p25 | 17.9 | 35.7 |
| p50 | 43.5 | 83.3 |
| p75 | 94.7 | 95.8 |
| p90 | 97.2 | 97.4 |
| p95 | 98.3 | 97.9 |
| mean | 53.1 | 68.0 |

| Jaccard bucket | jubarte docs | docxide-pdf docs |
|---|---|---|
| 0-10 | 37 | 23 |
| 10-20 | 83 | 41 |
| 20-30 | 48 | 31 |
| 30-40 | 17 | 9 |
| 40-50 | 22 | 17 |
| 50-60 | 9 | 8 |
| 60-70 | 16 | 18 |
| 70-80 | 28 | 39 |
| 80-90 | 14 | 33 |
| 90-100 | 124 | 179 |

Both distributions are bimodal: a "right" mode at 90-100 and a "wrong" mode under 30.
jubarte has 168 documents in the wrong mode and 124 in the right one; docxide-pdf 95
and 179. jubarte's median (43.5) sits in the valley between the modes, which is why
mean and median disagree so much for it and why a single number understates how
document-dependent it is.

Head to head on the same document (difference of more than 2 points):

| | count |
|---|---|
| jubarte ahead | 82 |
| docxide-pdf ahead | 188 |
| within 2 points | 128 |
| both at or above 60 | 181 |
| both under 20 | 37 |
| jubarte under 20 while docxide-pdf at or above 60 | **48** |
| docxide-pdf under 20 while jubarte at or above 60 | 1 |

Pearson correlation between the two per-document scores: 0.66. The 15-point mean gap
is mostly the 48 documents that jubarte gets wrong and docxide-pdf gets right; the 37
that both get wrong are the shared hard cases (long structured documents, merged-cell
tables set in a CJK face, endnotes, comment-scaled references). SSIM and text-boundary
means: jubarte 72.8 / 87.6, docxide-pdf 78.6 / 87.2.

## 13. jubarte's bottom 10% on the 398 (40 documents), diagnosed

Same columns as section 9, plus the resolved body font, the first paragraph's style and
`w:before`, the families embedded in the Word oracle and in jubarte's output, and the
OOXML features present. `dx J` is docxide-pdf with fonts. Generated by `bottom40.py`;
page-1 rasters were deleted after measurement.

| stem | jub J | dx J | jub T | pages ref/jub | body font | sz | first para style / before | oracle fonts | jubarte fonts | page-1 bands (top, left, pitch, J0, best shift) | features | pgSz |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| source_randomized__file_104 | 0.0 | 11.3 | 0.0 | 1/1 | theme:minorHAnsi->Calibri | 21 | - / 156 | Calibri | Calibri | (size (1754, 1240) vs (1754, 1241), cropped) top 175/0 L 242/0 pitch nan/nan J0 0.0 Jbest 10.7@(0,19) bands 3/1 | pict1 txbx2 sect1 | 11906x16838 |
| source__word_tolerated_broken_media_rel | 1.0 | 85.8 | 100.0 | 1/1 | theme:minorHAnsi->Calibri | 22 | - / 0 | Calibri | Calibri | (size (1754, 1240) vs (1754, 1241), cropped) top 161/203 L 150/150 pitch 48/31 J0 1.0 Jbest 79.4@(0,-25) bands 2/2 | pict1 sect1 | 11906x16838 |
| source_randomized__file_47 | 1.6 | 3.7 | 0.0 | 1/1 | explicit:Liberation Serif | 24 | TableContents / 0 | HiraMinProN-W3 | TimesNewRomanPSMT | top 118/118 L 83/118 pitch nan/nan J0 1.6 Jbest 12.2@(-19,13) bands 1/1 | tbl2 vmerge2 gridspan1 sect1 | 12240x15840 |
| source__nested_table_rowspan | 1.8 | 11.3 | 0.0 | 1/1 | none | - | style20 / 0 | HiraMinProN-W3 | Calibri/TimesNewRomanPSMT | top 118/118 L 83/118 pitch nan/nan J0 1.8 Jbest 5.0@(-17,24) bands 1/1 | tbl2 vmerge2 gridspan1 sect1 | - |
| source__table_vmerge_colspan | 1.9 | 4.5 | 100.0 | 1/1 | none | - | style20 / 0 | HiraMinProN-W3 | TimesNewRomanPSMT | top 118/118 L 95/118 pitch nan/nan J0 1.9 Jbest 5.9@(0,-32) bands 2/2 | tbl2 vmerge6 gridspan3 sect1 | - |
| source_randomized__file_199 | 2.0 | 5.5 | 100.0 | 1/1 | explicit:Liberation Serif | 24 | TableContents / 0 | HiraMinProN-W3 | TimesNewRomanPSMT | top 118/118 L 95/118 pitch nan/nan J0 2.0 Jbest 6.2@(0,2) bands 2/2 | tbl2 vmerge6 gridspan3 sect1 | 12240x15840 |
| source__complex_style_attr | 3.3 | 81.7 | 100.0 | 1/1 | theme:minorHAnsi->Calibri | 22 | ListParagraph / 0 | Aptos/Aptos-Bold/ArialMT/Calibri/Calibri-Bold/TimesNewRomanPSMT | Aptos/Aptos-Bold/Calibri/Calibri-Bold | top 172/172 L 188/188 pitch 24/28 J0 3.3 Jbest 40.1@(0,-39) bands 5/7 | num2 sect1 hdrftr5 | 12240x15840 |
| source_randomized__file_193 | 3.7 | 83.3 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | TimesNewRomanPSMT | top 156/154 L 150/150 pitch 52/47 J0 3.7 Jbest 16.5@(1,11) bands 6/5 | sect1 | 12240x15840 |
| source_randomized__file_4 | 3.8 | 81.3 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | ArialMT | top 156/154 L 150/150 pitch 52/47 J0 3.8 Jbest 19.7@(-17,11) bands 5/5 | sect1 | 12240x15840 |
| source_randomized__file_29 | 3.9 | 80.4 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | ArialMT | top 156/154 L 150/150 pitch 52/47 J0 3.9 Jbest 19.8@(-1,12) bands 5/5 | sect1 | 12240x15840 |
| source__word_tolerated_orphan_comment | 4.0 | 31.6 | 0.0 | 1/1 | none | - | - / 0 | TimesNewRomanPSMT | Calibri | top 179/188 L 150/151 pitch nan/nan J0 4.0 Jbest 38.4@(-2,-9) bands 1/2 | sect1 | - |
| source_randomized__file_2 | 4.1 | 74.8 | 71.0 | 2/2 | theme:minorHAnsi->Cambria | 22 | - / 0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | top 156/156 L 150/150 pitch 52/53 J0 4.1 Jbest 10.6@(14,-24) bands 25/24 | tbl1 sect1 | 12240x15840 |
| source_randomized__file_41 | 4.1 | 74.9 | 71.0 | 2/2 | theme:minorHAnsi->Cambria | 22 | - / 0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | top 156/156 L 150/150 pitch 52/53 J0 4.2 Jbest 10.6@(14,-24) bands 25/24 | tbl1 sect1 | 12240x15840 |
| source__ooxml_style_link | 4.2 | 73.0 | 91.3 | 2/2 | theme:minorHAnsi->Cambria | 22 | - / 0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | top 156/156 L 150/150 pitch 52/53 J0 4.6 Jbest 10.8@(20,-24) bands 23/23 | tbl1 sect1 | 12240x15840 |
| source__word_tolerated_misplaced_link | 4.2 | 73.0 | 91.3 | 2/2 | theme:minorHAnsi->Cambria | 22 | - / 0 | Cambria/Cambria-Bold | Calibri/Calibri-Bold | top 156/156 L 150/150 pitch 52/53 J0 4.6 Jbest 10.8@(20,-24) bands 23/23 | tbl1 sect1 | 12240x15840 |
| source_randomized__file_30 | 5.0 | 80.0 | 100.0 | 1/1 | theme:minorHAnsi->Calibri | 22 | ListParagraph / 0 | Aptos/Aptos-Bold/Arial-BoldMT/ArialMT/Calibri/Calibri-Bold/TimesNewRomanPSMT | Aptos/Aptos-Bold/Calibri/Calibri-Bold | top 171/172 L 188/188 pitch 25/28 J0 5.0 Jbest 30.9@(0,-39) bands 6/9 | num3 sect1 hdrftr5 | 12240x15840 |
| source_randomized__file_46 | 5.3 | 80.9 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | TimesNewRomanPSMT | top 156/154 L 150/150 pitch 52/47 J0 5.3 Jbest 19.7@(0,11) bands 5/4 | sect1 | 12240x15840 |
| source__word_tolerated_misplaced_pgsz | 6.4 | 6.5 | 0.0 | 1/1 | none | - | style20 / 0 | Verdana | CourierNewPSMT | top 123/122 L 118/118 pitch 29/20 J0 6.4 Jbest 7.6@(0,4) bands 24/24 | sect1 | - |
| source_randomized__file_59 | 6.7 | 80.5 | 4.0 | 1/1 | theme:minorHAnsi->Aptos | 24 | PreformattedText / 0 | Verdana | CourierNewPSMT | top 123/121 L 118/118 pitch 29/20 J0 6.7 Jbest 7.7@(0,34) bands 25/25 | sect1 | 12240x15840 |
| source_randomized__file_101 | 6.9 | 89.8 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria-Bold | Calibri-Bold | top 158/159 L 151/150 pitch 71/67 J0 6.9 Jbest 15.4@(15,-35) bands 4/5 | sect1 | 12240x15840 |
| source__font_family_demo_id_paraid_overflow | 7.0 | 16.9 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | TimesNewRomanPSMT | top 156/154 L 150/150 pitch 52/47 J0 7.0 Jbest 16.9@(1,6) bands 4/4 | sect1 | 12240x15840 |
| source_randomized__file_163 | 7.2 | 85.4 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria-BoldItalic | TimesNewRomanPS-BoldItalicMT | top 156/154 L 147/146 pitch 52/47 J0 7.2 Jbest 23.1@(9,11) bands 4/4 | sect1 | 12240x15840 |
| source_randomized__file_97 | 7.3 | 69.8 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria-Bold | Arial-BoldMT | top 156/154 L 150/150 pitch 52/47 J0 7.3 Jbest 32.9@(-15,14) bands 5/4 | sect1 | 12240x15840 |
| source__table_bookmark_end | 7.5 | 18.4 | 89.3 | 2/2 | theme:minorHAnsi->Cambria | 22 | Title / 0 | Calibri/Calibri-Bold/Cambria | Calibri/Calibri-Bold | top 165/165 L 175/187 pitch 42/54 J0 12.1 Jbest 13.9@(0,-2) bands 18/18 | tbl8 sect1 | 12240x15840 |
| source__endnotes_sample | 7.6 | 10.5 | 0.0 | 1/1 | none | - | style0 / 0 | TimesNewRomanPSMT | TimesNewRomanPSMT | (size (1754, 1240) vs (1754, 1241), cropped) top 121/122 L 118/119 pitch 45/28 J0 7.6 Jbest 9.9@(0,1) bands 4/3 | en2 sect1 | - |
| source__open_sans_font_demo_id_paraid_overflow | 8.0 | 14.3 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | ArialMT | top 156/154 L 150/150 pitch 52/47 J0 8.0 Jbest 20.7@(-16,7) bands 3/3 | sect1 | 12240x15840 |
| source__Redline_CiceroDo_v_plate_30 | 8.1 | 17.3 | 26.0 | 5/5 | explicit:Times New Roman | 22 | - / 0 | TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | top 156/154 L 187/188 pitch 39/39 J0 15.6 Jbest 19.6@(0,-11) bands 23/27 | tbl2 ins53 del14 vmerge40 sect1 | 12240x15840 |
| source_randomized__file_146 | 8.3 | 16.3 | 48.6 | 7/7 | explicit:Inter | 22 | - / 0 | ArialMT/Cambria/Cambria-Bold/CourierNewPSMT | Cambria/Cambria-Bold/CourierNewPSMT | top 79/81 L 75/75 pitch 28/32 J0 19.9 Jbest 20.2@(0,-1) bands 22/19 | tbl12 num8 fld9 ins8 del71 cmt2 pgbr2 sect1 hdrftr2 | 12240x15840 |
| source_randomized__file_13 | 8.3 | 22.7 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Roboto-Regular | ArialMT | top 154/154 L 150/150 pitch 52/47 J0 8.3 Jbest 24.7@(1,10) bands 4/5 | sect1 | 12240x15840 |
| source_randomized__file_70 | 8.4 | 46.8 | 0.0 | 1/1 | theme:minorHAnsi->Calibri | 22 | - / 0 | Calibri | Calibri | (size (1754, 1240) vs (1754, 1241), cropped) top 156/149 L 150/150 pitch nan/nan J0 8.4 Jbest 8.4@(0,0) bands 3/1 | drw1 pict1 txbx4 sect1 | 11906x16838 |
| source__open_sans_font_demo_id_paraid_overflow_2 | 8.4 | 15.0 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | ArialMT | top 156/154 L 150/150 pitch 52/47 J0 8.4 Jbest 21.0@(-1,7) bands 3/3 | sect1 | 12240x15840 |
| source_randomized__file_134 | 8.9 | 13.5 | 92.9 | 2/2 | theme:minorHAnsi->Cambria | 22 | Title / 0 | Calibri/Calibri-Bold/Cambria | Calibri/Calibri-Bold | top 165/165 L 175/187 pitch 37/32 J0 14.5 Jbest 14.5@(0,0) bands 20/21 | tbl8 sect1 | 12240x15840 |
| source_randomized__file_75 | 9.4 | 23.6 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Roboto-Regular | ArialMT | top 154/154 L 150/150 pitch 52/47 J0 9.4 Jbest 26.1@(0,10) bands 4/5 | sect1 | 12240x15840 |
| source__potpourritest | 9.5 | 7.2 | 22.7 | 5/5 | theme:minorHAnsi->Aptos | 24 | Title / 0 | Aptos/Aptos-Bold/Aptos-BoldItalic/Aptos-Italic/AptosDisplay/ArialMT/Consolas-BoldItalic/SymbolMT | Aptos/Aptos-Bold/Aptos-BoldItalic/Aptos-Italic/AptosDisplay/Consolas-BoldItalic/SymbolMT | top 166/166 L 150/149 pitch 52/36 J0 22.7 Jbest 31.1@(0,-16) bands 25/23 | tbl3 fn6 num6 fld6 ins101 del69 sect1 hdrftr6 | 12240x15840 |
| source__sd_2517_localized_heading_styles | 9.6 | 8.0 | 20.9 | 107/107 | explicit:Times New Roman | 24 | DocumentTitle / 0 | Aptos/Arial-BoldMT/ArialMT/BookAntiqua/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | Aptos/Arial-BoldMT/ArialMT/BookAntiqua/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | top 460/467 L 460/460 pitch 25/20 J0 24.4 Jbest 57.8@(0,-7) bands 6/6 | tbl2 fld537 pgbr26 sect21 hdrftr19 | 12240x15840 |
| source_randomized__file_22 | 9.6 | 8.0 | 21.0 | 107/107 | explicit:Times New Roman | 24 | DocumentTitle / 0 | Aptos/Arial-BoldMT/ArialMT/BookAntiqua/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | Aptos/Arial-BoldMT/ArialMT/BookAntiqua/TimesNewRomanPS-BoldMT/TimesNewRomanPS-ItalicMT/TimesNewRomanPSMT | top 413/420 L 460/460 pitch 30/32 J0 24.6 Jbest 64.6@(0,-7) bands 8/8 | tbl2 fld537 pgbr26 sect21 hdrftr20 | 12240x15840 |
| source__times_new_roman_font_id_paraid_overflow | 9.8 | 15.0 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Calibri/Cambria | TimesNewRomanPSMT | top 156/154 L 150/150 pitch 52/48 J0 9.8 Jbest 20.0@(0,6) bands 3/3 | sect1 | 12240x15840 |
| source__sample_document_word_repair_of_our_output_iter2_word_repaired_2 | 10.4 | 15.9 | 47.3 | 7/7 | explicit:Inter | 22 | - / 0 | ArialMT/Cambria/Cambria-Bold/CourierNewPSMT | Cambria/Cambria-Bold/CourierNewPSMT | top 79/81 L 75/75 pitch 28/32 J0 16.5 Jbest 17.2@(0,-2) bands 23/19 | tbl12 num8 fld9 ins6 del71 cmt2 pgbr2 sect1 hdrftr2 | 12240x15840 |
| source_randomized__file_145 | 11.0 | 80.4 | 100.0 | 1/1 | explicit:Calibri | 22 | - / 0 | Arial-BoldItalicMT/Calibri | Arial-BoldItalicMT | top 156/155 L 151/151 pitch 58/65 J0 11.0 Jbest 34.2@(0,-18) bands 5/5 | sect1 | 12240x15840 |
| source_randomized__file_176 | 11.1 | 27.8 | 65.8 | 7/7 | explicit:Inter | 22 | - / 0 | ArialMT/Cambria/Cambria-Bold/CourierNewPSMT | Cambria/Cambria-Bold/CourierNewPSMT | top 79/81 L 75/75 pitch 28/32 J0 20.1 Jbest 20.4@(0,-1) bands 23/20 | tbl12 num8 fld9 ins211 del6 cmt2 pgbr2 sect1 hdrftr2 | 12240x15840 |

### 13.1 Clusters in the bottom 40, with corpus-wide prevalence

Corpus prevalence is over all 398 sources (one Python pass over the DOCX XML).

**A1. Word's recorded substitution ignored (15 of 40; 81 corpus documents carry an
`altName`, 40 name fonts as CSS-style lists).** file_193, file_4, file_29, file_46,
file_163, file_97, file_101, file_13, file_75, file_59, word_tolerated_misplaced_pgsz,
font_family_demo, open_sans_font_demo, open_sans_font_demo_2, times_new_roman_font.
These documents name families Word did not have, often as a CSS-style list
(`"Times New Roman", Times, serif`, `Roboto, sans-serif`, `"Calibri", ...`) or as
LibreOffice faces (`Liberation Serif`, `DejaVu Sans Mono`). Word treats the whole
string as one unknown family, substitutes, and writes its choice into
`word/fontTable.xml`:

```xml
<w:font w:name="&quot;Times New Roman&quot;, Times, serif"><w:altName w:val="Cambria"/>...
<w:font w:name="DejaVu Sans Mono"><w:altName w:val="Verdana"/>...
<w:font w:name="Liberation Serif"><w:altName w:val="Hiragino Mincho ProN W3"/>...
<w:font w:name="Roboto, sans-serif"><w:altName w:val="Roboto"/>...
```

jubarte's resolver (`font.rs` `resolve`) splits the string on the comma, strips the
quotes and substring-matches the first token, so it draws Times, Arial or Courier
where the oracle has Cambria, Verdana or Hiragino. docxide-pdf tries the altName
first (`src/fonts/mod.rs:434`, "it's the document's own record") and embeds Cambria for
file_193 and Verdana for file_59, scoring 83.3 and 80.5 against jubarte's 3.7 and 6.7.
jubarte has zero references to `fontTable` or `altName` in `src/convert`.

Two of the fifteen (font_family_demo, open_sans_font_demo, from the un-randomized
pool) have no `fontTable.xml` at all, and Word still rendered Cambria; docxide-pdf
falls to Helvetica there and scores 14-17. So a second, evidence-based rule is needed
for missing families with no altName (section 14).

**A2. No docDefaults font (1 of 40 here, 5 in the corpus at 4.3 mean).**
word_tolerated_orphan_comment: no `docDefaults`, no Normal font. Word applies its
built-in default (Times New Roman); jubarte assumes Calibri. Same family of defect.

**B. Theme minor Cambria, Finding A (6 of 40).** file_2, file_41, ooxml_style_link,
word_tolerated_misplaced_link, table_bookmark_end, file_134. Text-boundary 71-93,
ink wrong. Nothing new; the corpus control group from section 3.

**C. Line box, Finding D (6 of 40).** file_146, sample_document_word_repair,
file_176 (Inter, which jubarte already maps to Cambria as Word does, so the faces
match and the 28 vs 32 px pitch is the Cambria line box alone), Redline_CiceroDo
(Times New Roman body), endnotes_sample (Times, 45 vs 28 px), file_145 (Arial
Heading 4 paragraphs, 58 vs 65 px). All have first-line tops within 2 px of Word;
the drift accumulates down the page.

**D. Merged cells (4 of 40; 8 corpus documents).** file_47, nested_table_rowspan,
table_vmerge_colspan, file_199: `w:gridSpan` / `w:vMerge` are not laid out, so
"CCC / DDD / EEE" collapses into one cell in one row. These also carry
`Liberation Serif -> Hiragino Mincho ProN W3` altNames (cluster A1), which is why
docxide-pdf is at 3.7-11.3 too: it gets the font but not the layout either.
Redline_CiceroDo has 40 `vMerge` cells.

**E. Anchored text boxes, VML, broken media (3 of 40; 18 corpus documents have
text boxes or VML).** file_104: the anchored text box is drawn at the page origin
(0,0) with the body text pulled inside it; Word places it beside the paragraph.
file_70: four text boxes and a picture, jubarte draws one band where Word draws
three. word_tolerated_broken_media_rel: the picture's relationship target is missing;
Word draws a small placeholder and continues, jubarte reserves a taller box and
pushes the text 25 px down (1.0 -> 79.4 after the shift).

**F. Notes (2 of 40; 12 corpus documents have footnotes, 10 endnotes).**
endnotes_sample: no endnote text (text-boundary 0). potpourritest: six footnotes,
none reserved at the page bottom, so the "West" table row stays on page 1 where Word
moves it to page 2; pagination diverges from there (22.7 text-boundary).

**G. Footer block (2 of 40; 52 corpus documents have a footer).**
complex_style_attr, file_30: body first line matches Word to the pixel (172/172), the
three-line footer lands 39 px (19 pt) lower than Word's. jubarte anchors the last
footer line's descent at `w:footer` and stacks upward (`mod.rs` ~8503); Word's block
sits higher here. Also the "1. ONE / a. a" list levels are 4 px closer in Word (24 vs
28 px; `contextualSpacing` is present in 50 corpus documents).

**H. Long structured documents (3 of 40).** sd_2517_localized_heading_styles and
file_22 (107 pages, 537 fields, 21 sections; cover page 7 px low; pagination diverges;
both engines at 8-10), Redline_CiceroDo (5 pages, tracked changes, merged cells).
Park until A-G land; both engines fail them.

**I. A4 MediaBox (4 of 40, cosmetic; 21 corpus documents are A4).** jubarte writes
595.3 x 841.9 pt for 11906 x 16838 twips; Word writes 595.2 x 841.92. MuPDF rasters
1241 vs 1240 px wide; the scorer crops, so the cost is nil, but the fix is one
constant.

Of the 40, docxide-pdf with fonts scores 60 or more on 21 (clusters A1, B, part of C:
jubarte-specific defects) and under 20 on 19 (D, F, H, and the two A1 documents with
no fontTable: shared hard cases).

## 14. Font resolution as data: what `@docfonts/fallbacks` does and what jubarte should copy

[superdoc/docfonts `packages/fallbacks`](https://github.com/superdoc/docfonts/tree/main/packages/fallbacks)
ships "decisions, not fonts": 42 reviewed records, each

```
logicalFamily  physicalFamily   verdict        policyAction        faces  generic     evidenceId, measurementRefs, exportRule, advance{basis,meanDelta,maxDelta}
Calibri        Carlito          metric_safe    substitute          rbib   sans-serif
Cambria        Caladea          visual_only    substitute          rbib   serif
Georgia        Gelasio          near_metric    substitute          rbib   serif
Verdana        Noto Sans        visual_only    category_fallback   rbib   sans-serif
Consolas       Inconsolata SE   visual_only    category_fallback   rb     monospace
Times New Roman Liberation Serif metric_safe   substitute          rbib   serif
Arial          Liberation Sans  metric_safe    substitute          rbib   sans-serif
Courier New    Liberation Mono  metric_safe    substitute          rbib   monospace
Aptos          -                no_substitute  customer_supplied   -      sans-serif
Cambria Math   -                preserve_only  preserve_only       -      serif
```

and a small API: `getFallbackDecision(family, {canRenderFamily})` returns a decision
*kind* (`fallback`, `asset_missing`, `face_missing`, `customer_supplied`,
`preserve_only`, `no_recommended_fallback`, `unknown`), `getRenderableFallbackForFace`
reports whether a bold/italic face is real or must be synthesized, `createFallbackMap`
builds the resolver table from what the app can actually load, and `normalizeFamilyName`
canonicalises names. Verdicts are measured (`advance.meanDelta`, `lineBreakSafe`,
`glyphExceptions`) and every row carries an evidence id.

What jubarte has instead: a `FaceId` enum of 47 faces, a `resolve()` of substring
tests (`key.contains("cambria") || key == "inter"`), bundled Carlito + Liberation only,
and comments explaining each special case by a mini-set score. Measured against
docfonts' verdicts, jubarte's open fallbacks for Cambria (Liberation Serif), Georgia
(Liberation Serif), Verdana (Liberation Sans), Consolas (Liberation Mono) and Book
Antiqua (Liberation Serif) are all rated wrong; docfonts names Caladea, Gelasio, Noto
Sans, Inconsolata. On this Mac the DFonts overlay hides that; on Linux CI it does not.

The concept transfers as two tables and one pipeline, in this order:

1. **The document's own record**: `fontTable.xml` `w:altName` (cluster A1). Word
   wrote it; use it before anything else. Also read `w:family` (roman / swiss /
   modern) as the generic of last resort, which docxide-pdf already does.
2. **Explicit family, as written.** Do not split CSS-style lists or strip quotes;
   Word does not. A name that fails lookup goes to step 4, not to its first token.
3. **Theme slot** (`minorHAnsi` / `majorHAnsi`), for any family (Finding A).
4. **Word-substitution evidence table**: what Word-for-Mac drew when the family was
   absent and no altName was written. Harvested from the 81 corpus altNames plus the
   two no-fontTable fixtures: quoted / CSS-list unknown family -> Cambria; DejaVu Sans
   Mono -> Verdana; Liberation Serif -> Hiragino Mincho ProN W3 or Times New Roman
   (two observed outcomes: record both with their stems, resolve by the document's
   language tags, and measure); no docDefaults font -> Times New Roman. Each row
   carries the oracle stems it was measured on, docfonts-style.
5. **Open-face table with verdicts** (docfonts' records, licensed OFL/Apache), for
   when the physical face is not on the machine. Bundle Caladea, Gelasio, Noto Sans,
   Inconsolata alongside Carlito and Liberation, and keep the verdict so the bench can
   report "rendered with a visual_only fallback" per document instead of silently
   scoring it.
6. **Decision kinds in a font report** (`jubarte convert --font-report`): per
   document, every requested family and which step resolved it. The bench then counts
   `asset_missing` and `unknown` documents, which is how the next cluster A1 gets found
   before it costs a decile.


## 15. XML parts coverage

Which package parts and elements jubarte ignores or uses superficially, what each costs
on both sets, and full implementation plans for the four at the `fontTable.xml` level,
are in `xml_parts_plan.md` in the same `planning/` directory. Headline: the three largest items are all
font or geometry data already present in the package (theme slot gate, font table,
table `tcW` / `tblInd` / `trHeight` and the compat-mode edge rule); drawing placement is
fourth. `settings.xml` matters first through the table edge rule (compat mode), not
through any of its other 40 flags.

## 16. Post-stack measurement (2026-09-06)

Scored on this machine after stacked convert PRs through `c:radarChart` (#108 /
`8be8344`) plus the Step 8 audit (#109 / `ab27d3f`, no engine change). Binary
`target/release/jubarte` rebuilt from that checkout. Scorer:
`neurotic_docx_bench` `docxide-metrics`. Same Jaccard / SSIM / text-boundary
definitions as section 1. Convert failures: 0 on sample50, 0 on the 76, 0 on
the 398.

Mean Jaccard **rose** on every gate (sample50 +6.07, 76 +14.53, 398 +3.28).
Row drops >1.0 Jaccard are named below; they are not silently blessed. Do not
treat green CI on the stack as this measurement.

| set | n | baseline J mean / median | now J mean / median | Δ mean | SSIM now mean / median | TB now mean / median | convert failures |
|---|---|---|---|---|---|---|---|
| sample50 (`planning/sample50_check.py`) | 50 | 37.57 / — | 43.63 / — | +6.07 | — | — | 0 |
| docxide-pdf 76 fixtures | 76 | 13.74 / 8.86 | 28.27 / 20.62 | +14.53 | 45.19 / 44.43 | 51.54 / 53.86 | 0 |
| neurotic 398 corpus | 398 | 53.10 / — | 56.38 / 66.57 | +3.28 | 72.53 / 89.27 | 88.89 / 100.0 | 0 |

sample50 still uses the 2026-09-05 `planning/sample50_baseline.json` ratchet
(not `--bless`ed). 76 vs `tools/convert_baseline_76.tsv`. 398 vs
`tools/convert_baseline_398.tsv`.

### sample50 rows that dropped >1.0 Jaccard

| id | set | base | now | Δ |
|---|---|---|---|---|
| source__eigenpal_docx_editor_suggesting_mixed_edits | corpus | 17.9 | 8.2 | −9.6 |
| source__docx_lots_of_comments_addition_redline_addition_v_re | corpus | 29.3 | 21.2 | −8.1 |
| source_randomized__file_93 | corpus | 25.3 | 15.6 | −9.6 |
| source_randomized__file_9 | corpus | 30.0 | 21.8 | −8.1 |
| source_randomized__file_48 | corpus | 48.8 | 31.3 | −17.5 |
| case7 | docxide | 9.8 | 8.3 | −1.5 |
| case5 | docxide | 29.3 | 8.0 | −21.3 |

### 76-set rows that dropped >1.0 Jaccard

case5 29.3→8.0 (−21.3); case4 16.6→7.6 (−9.0); case10 14.4→7.3 (−7.1);
case33 6.9→4.7 (−2.2); case42 26.9→24.7 (−2.2); case39 13.8→11.7 (−2.0);
case56 15.9→13.9 (−2.0); case7 9.8→8.3 (−1.5). case13 (205 pp, fast-gate
skiplist) is **not** a drop: 7.53→29.25, pages 205/205.

### 398-set rows that dropped >1.0 Jaccard (68)

Worst: `source__mcdoc` 68.28→12.82 (−55.47). Full stem list from
`scripts/convert_sweep.py 398 --compare tools/convert_baseline_398.tsv`:

source__I_am_sharing_Microsoft_Word_vs_Google_Docs_Comprehensive_Proof_with_you,
source__contract_review_suggesting_insertions, source__docx_lots_of_comments,
source__docx_lots_of_comments_addition, source__docx_lots_of_comments_addition_redline,
source__docx_lots_of_comments_addition_redline_addition_v_removal,
source__docx_lots_of_comments_addition_removal,
source__docx_lots_of_comments_addition_removal_redline,
source__docx_lots_of_comments_addition_removal_redline_removal_v_addition,
source__eigenpal_docx_editor_suggesting_mixed_edits,
source__eigenpal_docx_editor_suggesting_mixed_edits_2,
source__heading_3_center_italic_id_paraid_overflow,
source__heading_3_style_demo_id_paraid_overflow,
source__heading_3_style_demo_id_paraid_overflow_2,
source__heading_4_right_italic_id_paraid_overflow,
source__heading_4_style_demo_id_paraid_overflow,
source__heading_4_style_demo_id_paraid_overflow_2,
source__helvetica_font_demo_style_default_missing, source__image_out_of_folder,
source__mcdoc, source__potpourritest, source__quarterly_performance_report_table,
source__sample_document_afterword_repaired_word_repaired,
source__sample_document_really_repaired_word_repaired,
source__sample_document_word_repair_of_our_output_iter2_word_repaired,
source__sample_document_word_repair_of_our_output_iter2_word_repaired_2,
source__sample_document_word_repair_of_our_output_word_repaired,
source__sample_document_word_repair_of_our_output_word_repaired_2,
source__sd_2517_localized_heading_styles, source__word_tolerated_duplicate_ppr,
source__word_tolerated_misplaced_uipriority, source_randomized__file_111,
source_randomized__file_117, source_randomized__file_120,
source_randomized__file_131, source_randomized__file_139,
source_randomized__file_14, source_randomized__file_143,
source_randomized__file_145, source_randomized__file_146,
source_randomized__file_16, source_randomized__file_166,
source_randomized__file_168, source_randomized__file_170,
source_randomized__file_175, source_randomized__file_176,
source_randomized__file_188, source_randomized__file_19,
source_randomized__file_195, source_randomized__file_22,
source_randomized__file_27, source_randomized__file_33,
source_randomized__file_34, source_randomized__file_40,
source_randomized__file_48, source_randomized__file_52,
source_randomized__file_53, source_randomized__file_55,
source_randomized__file_57, source_randomized__file_6,
source_randomized__file_64, source_randomized__file_69,
source_randomized__file_74, source_randomized__file_78,
source_randomized__file_8, source_randomized__file_9,
source_randomized__file_93, source_randomized__file_94.

### Parked (still unshipped)

SmartArt layout algorithms, comment-balloon / print-with-markup (case63/64),
hyphenation, `lastRenderedPageBreak` as layout input, cluster H (sd_2517 /
file_22 / Redline_CiceroDo as a chase). Sibling 0c/0f wording stays outward-facing.
