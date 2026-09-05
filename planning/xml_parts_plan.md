<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# XML parts coverage plan for jubarte's DOCX -> PDF converter

Companion to `report.md` and `plan.md` (2026-09-05). Question answered here: which parts
of the DOCX package, and which elements inside the parts we do read, does
`jubarte-redlines/src/convert` ignore or use superficially; what that costs; how
desirable (0-1) and how hard (0-1) each fix is; and, for every item at least as
desirable as `fontTable.xml`, a full implementation plan.

(Paths for docxide-pdf, the benchmarks and the fonts are in `plan.md`, section "Locations on this machine".)

## 0. Method

**Inventory.** One pass over the 398 corpus sources and the 76 docxide fixtures counted,
per document, which package parts exist and which elements occur (`xml_census.py`,
`xml_perdoc.json`). Handling status comes from `grep` over `src/convert/*.rs` for the
element's quoted name or its `W::snake()` helper, then reading the code where the count
was ambiguous (the raw counts are in the census output; only checked statuses appear
below).

**Desirability.** For each item, the *available lift* on a set is the sum over the
documents carrying the item of `max(docxide-pdf J - jubarte J, 0)`, divided by the set
size: the points of mean Jaccard that set would gain if jubarte matched docxide-pdf
(fonts visible) on exactly those documents. `D = min(1, (lift_corpus + lift_fixtures) / 2 / 4)`,
so 4 points of average lift saturates. Both lifts are printed so the reader can re-rank.

Two known distortions, stated rather than hidden:

- docxide-pdf is the ceiling, so where **both engines fail** (merged cells in nested
  tables, TOC fields, comments, math, content controls) the lift is ~0 although the
  feature matters. Those rows carry a `both<20` count and a judgment note.
- Items present in nearly every document as **style-template boilerplate**
  (`keepNext` in heading styles, `contextualSpacing`, `compatibilityMode`) correlate
  with everything and are not scored; the compat mode is a research item (section 4).

**Confidence.** *high* = a failing document was traced to this element in this session
(report sections 3-7, 13); *medium* = clear mechanism, no traced document; *low* = the
lift is mostly the set-wide gap (the fixtures fail for other reasons first).

**Difficulty.** 0.1 = a constant or a parse-and-copy; 0.3 = new parse plus a simple
rule; 0.5 = a layout feature inside one block type; 0.7 = touches pagination or the
wrap loop; 0.9 = a new engine (geometry, hyphenation, math).

**Threshold for a full plan.** `fontTable.xml` scores D = 1.00 (lift 5.20 corpus / 4.98
fixtures), confidence high. Every row with D = 1.00 and confidence high gets a full plan
in section 3: `fontTable.xml`, `theme1.xml` font slots, `document.xml` table
properties, `document.xml` drawing placement. `styles.xml` character styles also reach
1.00 but with low confidence and is kept as a sketch.

## 1. What jubarte loads today (from `part_string` / relationship loads in `src/convert`)

| Part | corpus / fixtures with it | Loaded | What is read | What is not |
|---|---|---|---|---|
| `word/document.xml` | 398 / 76 | yes | body, sectPr (page, margins, cols, titlePg, vAlign, pgNumType, header/footer refs), pPr, rPr, tables, drawings, VML shapes/text boxes, fields, hyperlinks, bookmarks, ins/del, comments refs | `tblInd`, `tcW`, `tblpPr`, `srcRect`, `pgBorders`, `docGrid` type, `lnNumType`, `object` (OLE), `sdt` unwrap is partial, `lastRenderedPageBreak` |
| `word/styles.xml` | 397 / 76 | yes | docDefaults, paragraph/character/table styles (basedOn), tblStylePr (firstRow etc.) | `link`, `next`, `latentStyles`; built-in defaults for styles referenced but undefined (only Heading3/4 spacing, `apply_latent_ppr`) |
| `word/numbering.xml` | 70 / 67 | yes | abstractNum/lvl: numFmt, lvlText, start, indent, marker rPr, suff, tab stops; num -> abstractNum | `lvlOverride` / `startOverride`, `lvlRestart`, `isLgl`, `numStyleLink` / `styleLink`, picture bullets |
| `word/settings.xml` | 249 / 74 | yes | `defaultTabStop` only | everything else: compat mode and flags, `evenAndOddHeaders`, `mirrorMargins`, `autoHyphenation`, `footnotePr` / `endnotePr`, `characterSpacingControl`, `displayBackgroundShape` |
| `word/theme/theme1.xml` | 244 / 72 | yes | major/minor latin fonts, colour scheme | minor slot applied only when it resolves to Aptos (`apply_rfonts`); script-specific fonts; `fmtScheme` fills/lines for shapes |
| `word/fontTable.xml` | 251 / 72 | **no** | - | `altName` (81 / 7 docs), `family`, `pitch`, `panose1`, `charset`, `embedRegular` etc. (0 / 1) |
| `word/fonts/*.odttf` | 0 / 1 | **no** | - | embedded, obfuscated TrueType |
| `word/header*.xml`, `footer*.xml` | 48 / 2, 52 / 2 | yes (rels) | default and first references; text runs, page fields, borders | `even` references (needs `evenAndOddHeaders`), tables and images in headers (10 docs), header height pushing the body |
| `word/footnotes.xml` | 53 / 5 (12 / 4 referenced) | **no** | - | notes, separators, page-bottom reservation |
| `word/endnotes.xml` | 53 / 4 (10 / 2 referenced) | yes | notes appended after the body (`append_endnotes`) | placement per `endnotePr` (section end), separator |
| `word/comments.xml` | 0 / 2 | yes | comment text -> PDF sticky-note annotations (`load_comments`, `text_annot_obj`) | balloon layout (Word's "print markup" scales the page; case63 / case64) |
| `commentsExtended/Ids/Extensible.xml`, `people.xml` | 55 / 0 | no | - | only meaningful with balloon rendering; nothing to draw |
| `word/charts/chart*.xml` (+ `embeddings/*.xlsx`) | 8 / 5 | yes | bar charts (`load_chart`, `c:barChart`) | pie, line, area, scatter, radar, legends |
| `word/diagrams/*` (SmartArt) | 8 / 3 | yes | `load_diag_shapes` from the drawing part | layout algorithms (only pre-laid `dsp:` shapes) |
| `word/media/*` | 34 / 10 | yes | JPEG / PNG / metafiles (`decode_image`, `metafile.rs`) | PNG alpha, `srcRect` crop, rotation |
| `word/embeddings/*` (OLE), `w:object` | 8 / 0 | **no** | - | the `v:imagedata` preview is only picked up when no DrawingML picture exists in the paragraph |
| `word/glossary/document.xml` | 0 / 0 | no | building blocks; not rendered by Word either | nothing |
| `word/webSettings.xml`, `docProps/*`, `customXml/*`, `stylesWithEffects.xml`, `vbaProject.bin`, `activeX/*` | up to 396 / 72 | no | not used by Word's PDF layout | nothing to gain |

## 2. Scored inventory

`n` = documents carrying the item (corpus / fixtures); `lift` = mean-Jaccard points
available on that set; `D` desirability; `diff` difficulty; `conf` confidence;
`both<20` = documents where both engines are under 20 on the item (ceiling caveat).

| Item | n c / f | lift c / f | D | diff | conf | both<20 | What we miss and what it costs |
|---|---|---|---|---|---|---|---|
| **theme1.xml: minor/major slot for non-Aptos faces** | 28 / 65 | 1.25 / 48.64 | **1.00** | 0.15 | high | 3 | Body set in Calibri where Word set Cambria; 47 of 76 fixtures, 12 corpus docs at 11.2 vs 61.8. Report section 3. |
| **fontTable.xml: altName, family, pitch** | 81 / 7 | 5.20 / 4.98 | **1.00** | 0.35 | high | 20 | Word's recorded substitution ignored; CSS-style family lists split on the comma; docs at 3-7 where docxide-pdf is at 70-90. Report section 13.1 A1. |
| `  ` subset: CSS-style family lists | 40 / 0 | 3.50 / 0 | 0.44 | - | high | 9 | covered by the row above |
| `  ` subset: no docDefaults font | 6 / 1 | 0.11 / 0.56 | 0.08 | 0.1 | high | 4 | Word defaults to Times New Roman; jubarte to Calibri |
| fontTable.xml: embedded `.odttf` | 0 / 1 | 0 / 0.54 | 0.07 | 0.4 | high | 0 | case8 custom face substituted; first line 19 px low |
| **document.xml: table properties** | 88 / 10 | 4.23 / 4.45 | **1.00** | 0.7 | high | 26 | Widths from `tblGrid` only (`tcW` unread), `tblInd` unread, row height through tuned pads, edge not pulled left by the cell margin. Section 3.3. |
| `  ` `trHeight` | 29 / 2 | 2.89 / 0.86 | 0.47 | 0.3 | high | 1 | parsed (`row_height`) but applied through `table_row_pad` constants; 16.6 vs 56.3 on the corpus |
| `  ` `tblInd` | 31 / 0 | 2.04 / 0 | 0.25 | 0.2 | high | 14 | unread; table x edge wrong by the indent |
| `  ` `tblCellMar` / `tcMar` | 40 / 0 | 2.20 / 0 | 0.28 | 0.3 | high | 21 | read, but fixed-layout tables force the 108-twip default; edge rule missing |
| `  ` `gridSpan` / `vMerge` | 8 / 2 | 0.07 / 0.59 | 0.08 | 0.5 | high | 6 | parsed (`cell_span`) but nested tables with merges collapse (nested_table_rowspan); both engines fail |
| `  ` `tblpPr` floating | 0 / 4 | 0 / 2.03 | 0.25 | 0.6 | high | 0 | case46: text should wrap beside the table |
| `  ` `tblLayout fixed`, `tblW pct` | 2 / 0 | 0.04 / 0 | 0.00 | 0.3 | medium | 2 | read; rare |
| **document.xml: drawings** | 32 / 15 | 0.22 / 12.91 | **1.00** | 0.6 | high | 0 | On the corpus jubarte is *ahead* of docxide-pdf on drawing documents (33.1 vs 20.8); the lift is the fixtures. Section 3.4. |
| `  ` `wp:anchor` placement / wrap | 15 / 9 | 0.22 / 8.03 | 1.00 | 0.6 | high | 0 | text box at page origin (file_104), wrapped text never aligns (case41) |
| `  ` `a:srcRect` | 10 / 1 | 0.02 / 1.09 | 0.14 | 0.2 | high | 0 | case78: four pictures uncropped; docxide-pdf 99.6 |
| `  ` `txbxContent` | 14 / 6 | 0.20 / 6.13 | 0.79 | 0.6 | high | 1 | boxes drawn as solid rectangles (case60), one band where Word has three (file_70) |
| `  ` VML `w:pict` | 18 / 0 | 0.60 / 0 | 0.08 | 0.4 | medium | 1 | `v:imagedata` read only when no DrawingML picture is present |
| styles.xml: character styles / `link` | 26 / 10 | 0.98 / 7.20 | 1.00 | 0.3 | **low** | 7 | `rStyle` is applied (paint only, no size/family by design); `link` never followed; lift is mostly the fixture-wide gap |
| styles.xml: latent built-in styles | 22 / 0 | 2.60 / 0 | 0.32 | 0.3 | high | 0 | `Title` / `Heading1-4` / `Subtitle` referenced but undefined: Word applies built-in defaults, jubarte only Heading3/4 spacing; heading demos at 28-48 vs 97 |
| styles.xml: `tblStylePr` conditional | 28 / 5 | 0.05 / 1.62 | 0.21 | 0.5 | medium | 5 | firstRow handled; banding, lastRow, firstCol not; jubarte ahead of docxide-pdf here on the corpus |
| numbering.xml (all) | 38 / 5 | 0.84 / 3.96 | 0.60 | 0.3 | medium | 16 | levels read; overrides and restarts not |
| `  ` `lvlOverride` / `startOverride` | 18 / 0 | 0.28 / 0 | 0.03 | 0.3 | high | 13 | numbers continue instead of restarting; small ink, wrong labels |
| document.xml: hard page breaks | 50 / 8 | 0.51 / 5.36 | 0.73 | 0.2 | low | 15 | `w:br type=page` is handled; the fixture lift is the general gap |
| document.xml: `lastRenderedPageBreak` | 52 / 5 | 0.84 / 3.50 | 0.54 | 0.2 | medium | 19 | Word's last pagination, written into the file. Usable as a page-break oracle; see section 4 for why that is a hack, not a fix |
| rPr: `u` | 72 / 3 | 1.90 / 2.42 | 0.54 | 0.1 | low | 18 | underline is drawn; the lift is not about underlines |
| footnotes.xml | 12 / 4 | 0.12 / 3.42 | 0.44 | 0.7 | high | 2 | notes absent, page bottom not reserved: potpourritest paginates differently; on the corpus jubarte is ahead of docxide-pdf on these docs |
| sectPr: multiple sections | 32 / 3 | 0.28 / 2.87 | 0.39 | 0.5 | medium | 2 | handled; per-section header/footer/page setup gaps unknown |
| charts | 8 / 5 | 0 / 3.11 | 0.39 | 0.8 | high | 0 | bar only; case56/57 at 15.9 / 13.6 vs 90 / 83 |
| sectPr: A4 | 21 / 2 | 0.86 / 1.92 | 0.35 | 0.05 | high | 1 | 595.3 x 841.9 instead of Word's 595.2 x 841.92; cosmetic under this scorer |
| footer*.xml / header*.xml | 52 / 48 | 0.89 / 0.63 | 0.34 | 0.5 | medium | 17 | multi-line footer 19 pt low (complex_style_attr); `even` refs, tables/images in headers (10 docs) unsupported |
| hyperlinks | 50 / 4 | 0.28 / 2.31 | 0.32 | 0.2 | low | 19 | handled; lift is set-wide |
| sectPr: multi-column | 0 / 2 | 0 / 1.79 | 0.22 | 0.7 | medium | 0 | case25 / case26 |
| pPr: tabs | 10 / 3 | 0 / 1.70 | 0.21 | 0.5 | medium | 2 | tab stops read; leaders and decimal/bar alignment unknown |
| endnotes.xml | 10 / 2 | 0.04 / 1.63 | 0.21 | 0.3 | high | 1 | appended, but endnotes_sample text-boundary 0: placement / separator wrong |
| comments.xml balloons | 16 / 2 | 0.23 / 1.43 | 0.21 | 0.6 | high | 13 | annotations exist; Word's markup export scales the page for the balloon column (case63 / 64); corpus oracles were exported without markup |
| diagrams (SmartArt) | 8 / 3 | 0 / 1.62 | 0.20 | 0.9 | high | 0 | pre-laid shapes only; case60 |
| rPr: strike | 14 / 2 | 0.10 / 1.49 | 0.20 | 0.1 | low | 2 | handled |
| rPr: spacing / w / kern / position | 4 / 2 | 0.02 / 1.76 | 0.22 | 0.3 | medium | 2 | `kern` read; character spacing, scale and position not |
| first/even header refs | 14 / 1 | 0.39 / 0.87 | 0.16 | 0.3 | medium | 2 | `first` handled; `even` requires `evenAndOddHeaders` (1 corpus doc) |
| rPr: highlight, vertAlign, shd | 22 / 1, 16 / 1, 2 / 1 | < 1 | 0.10-0.14 | 0.1-0.2 | low | 2 | handled or rare |
| sectPr: pgBorders | 0 / 1 | 0 / 1.15 | 0.14 | 0.2 | high | 0 | case68 |
| sectPr: titlePg, vAlign | 2 / 1 | < 1.1 | 0.11-0.14 | - | - | - | handled today (`titlePg`, `valign_center`) |
| document.xml: fields | 35 / 1 | 0.12 / 0.70 | 0.10 | 0.6 | medium | 7 | PAGE / NUMPAGES / PAGEREF handled; TOC (26 corpus docs) both engines fail; DATE, REF, NUMWORDS unsupported |
| document.xml: `w:ins` / `w:del` markup | 78 / 0 | 0.85 / 0 | 0.11 | 0.4 | medium | 17 | rendered with change bars; jubarte ahead of docxide-pdf on 44 of 78 |
| settings.xml: autoHyphenation | 0 / 1 | 0 / 0.74 | 0.09 | 0.9 | high | 0 | one fixture; needs a hyphenation dictionary (docxide ships `tests/fixtures/hyphenation`) |
| pPr: hanging / firstLine | 19 / 0 | 0.49 / 0 | 0.06 | 0.2 | low | 6 | handled |
| numbering: `isLgl`, `lvlRestart`, picture bullets | 2 / 1 | ~0 | 0.02 | 0.3 | medium | 1 | rare |
| pPr: pBdr, shd; rPr: caps, vanish; lineRule exact | 28 / 0, 13 / 0, 2 / 0, 0, 4 / 0 | < 0.3 | 0.00-0.03 | 0.1-0.3 | medium | 15 | pBdr on 28 corpus docs where both engines are under 20: paragraph borders are read (1 site) but the 15 both-fail docs deserve a look |
| OLE embeddings, `w:object` | 8 / 0 | 0 / 0 | 0.00 | 0.5 | medium | 0 | jubarte at 42.3 vs docxide-pdf 7.3 on these 8 docs: already ahead |
| sdt, oMath | 8 / 0 | 0 / 0 | 0.00 | 0.2 / 0.9 | medium | 0 | same 8 documents; sdt content is unwrapped; math is not rendered by either engine |
| settings.xml: evenAndOddHeaders, footnotePr, mirrorMargins, defaultTabStop | 1 / 0, 0, 0, read | ~0 | 0.00 | 0.2 | medium | 1 | rare or already read |
| sectPr: docGrid lines, tblLayout fixed, tblW pct | 2 / 0 each | ~0 | 0.00 | 0.3-0.5 | medium | 2 | rare in these sets |
| webSettings, customXml, docProps, glossary, stylesWithEffects, vbaProject, activeX, commentsExtended/Ids/people | many | 0 | 0.00 | - | - | - | not part of Word's page layout; nothing to gain |

Reading the table: three things dominate everything else and are all font or geometry
data that already sits in the package: the theme slot gate (one line to delete), the
font table (one part to parse), and table geometry (`tcW`, `tblInd`, `trHeight`, the
edge rule). Drawing placement is the fourth and is fixture-driven. After those four, the
next tier is footnotes (pagination), latent styles (a table of Word's built-in defaults),
numbering overrides and the footer block, each worth 1-3 points on one set.

## 3. Full implementation plans (D = 1.00, confidence high)

Common to all four: engine code lives in `../jubarte-redlines/src/convert/`; every
checkpoint runs `python3 sample50_check.py` (this directory) and ends with the
both-set numbers appended to `report.md`; no per-face, per-size or per-document
constant may be added to close a gap (plan.md ground rules).

### 3.1 `word/fontTable.xml` and `word/fonts/*.odttf`

**Scope.** Parse the font table; honour `w:altName`; stop splitting family strings;
load embedded fonts; use `w:family` / `w:pitch` as the generic of last resort; emit a
per-document font report. This is plan.md Step 2a/2b/2d/2f in full.

**Current code.**
- `font.rs`: `FaceId` is a closed enum of 47 faces (`FaceId::all()`); `Fonts::new`
  loads *all* of them eagerly, each from `system_override(id)` (DFonts, `/Library/Fonts/Microsoft`,
  `Supplemental`, `/System/Library/Fonts`, `/Library/Fonts`) or the bundled bytes;
  `Face::from_bytes` parses with `ttf_parser` (upem, typo ascent/descent/line gap,
  win ascent as `paint_ascent`, hmtx widths, unicode cmap, a `rustybuzz::Face`);
  `Face` holds `bytes: &'static [u8]`, and `from_path` leaks the file (`Box::leak`).
- `font.rs` `Fonts::resolve(family, bold, italic) -> FaceId`: takes the first
  comma-separated token, strips quotes, lower-cases, removes spaces/hyphens/"mt", then
  substring tests (`calibrilight`, `cambria` / `inter`, `consolas`, `georgia`,
  `bookantiqua`, `symbol`, mono keywords, `aptos*`, `verdana`, sans keywords, serif
  keywords) and falls to Carlito.
- `mod.rs` `apply_rfonts` writes `RunStyle.family` as a string; nothing reads
  `word/fontTable.xml` anywhere (`grep -c fontTable src/convert/*.rs` = 0).
- `pdf.rs` `emit`: one font object set per `FaceId` used (`simple_ttf_obj` for
  WinAnsi-encodable text, `cid_font_obj` + `type0_font_obj` otherwise,
  `font_descriptor_obj`, `font_file_obj`), resource names via `uniquify`.

**Word's rule (what the plan reproduces).** Word draws the requested family if it is
installed; if not, the `altName` recorded in the font table if that is installed;
otherwise its own substitution (by `w:family`, `w:pitch`, `w:panose1`, `w:sig`), which
it then writes back as `altName` on save. A family name is an opaque string:
`"Times New Roman", Times, serif` is one (unknown) family, not a list. Embedded fonts
(`w:embedRegular` etc. with `r:id` + `w:fontKey`) are obfuscated TrueType: the first
32 bytes are XORed with the 16 GUID bytes of `fontKey` in reverse order, twice
(ECMA-376 Part 2 §11; docxide-pdf `docx/embedded_fonts.rs::deobfuscate_font` is a
working reference).

**Design.**
1. `load_font_table(pkg) -> FontTable`: `HashMap<String, FontEntry { alt_name, family: Roman|Swiss|Modern|Script|Decorative|Auto, pitch: Fixed|Variable|Default, panose: Option<[u8;10]>, charset, embedded: [Option<(rel_id, font_key)>; 4] }>`,
   keyed by the exact `w:name` (no normalisation). Loaded once per conversion next to
   `load_stylesheet` / `load_numbering`.
2. Registry refactor: `Fonts { faces: HashMap<FaceKey, Face>, table: FontTable, report: Vec<Resolution> }`
   with `FaceKey { family: String /* physical, canonical */, bold, italic }`. `Face.bytes`
   becomes `Arc<[u8]>` (embedded fonts are per document; `Box::leak` would leak per
   conversion). Faces load lazily on first `get`; the 47 bundled/system faces become
   registry entries keyed by their logical family ("Calibri" -> Carlito bytes unless the
   DFonts file exists), so every current call site keeps working through a shim
   `resolve_id -> FaceKey`.
3. `resolve(requested: &str, bold, italic, hint) -> (FaceKey, Step)` in this order, each
   step recorded in `report`:
   1. **embedded**: the document's own face for that family and style (deobfuscated,
      parsed, cached per conversion);
   2. **installed**: the exact requested string against the registry / system
      directories (case-insensitive, whitespace-preserving; no split, no quote strip);
   3. **altName**: the font table's `altName`, resolved through steps 2-6 with a cycle
      guard;
   4. **theme**: only reached when the run carried a theme slot and no explicit family
      (3.2);
   5. **Word-substitution evidence** (plan.md Step 2d; data file, one row per observed
      substitution with the stems it was measured on);
   6. **open fallback with verdict** (plan.md Step 2e; docfonts-style records);
   7. **generic** from `w:family` / `w:pitch`: fixed pitch -> Courier New, roman ->
      Times New Roman, swiss -> Arial, else the evidence table's "unknown family" row
      (Cambria on the oracle machine), else Calibri.
   A `--font-policy recorded` switch (used by the bench) moves step 3 ahead of step 2,
   because the oracle was drawn on a machine that did *not* have the requested font and
   the recorded altName is the only faithful reproduction of it; the default keeps Word's
   own order for users converting on their own machine.
4. Style synthesis: when the physical family lacks the bold or italic face, record
   `synthetic` in the report and let `pdf.rs` fake bold (stroke + fill) and italic
   (skew matrix) as Word does; today `resolve` silently returns the regular face.
5. PDF output: `/BaseFont` = the physical face's PostScript name (what Word does when
   it substitutes); the logical name goes only into the report.
6. `jubarte convert --font-report out.json`: `[ {requested, step, physical, bold, italic, synthetic} ]`
   per distinct request; the neurotic bench stores the step histogram per document.

**Files and functions.** `font.rs` (`FaceKey`, `Face::bytes` -> `Arc`, lazy `Fonts::get`,
new `resolve`, `deobfuscate`, keep `system_override` as the installed-face locator),
`mod.rs` (`load_font_table`, `apply_rfonts` unchanged except for the theme gate,
`RunStyle.family` stays the requested string, `hint` from `w:hint`), `pdf.rs`
(font objects keyed by `FaceKey`; synthetic bold/italic), `src/bin` CLI flag.

**Order of work and checkpoints.**
1. Parse the font table and apply `altName` through the existing `resolve` (no registry
   change). Checkpoint: file_193 / file_59 / file_13 / nested_table_rowspan embed
   Cambria / Verdana / Roboto / Hiragino; the 81-document altName group moves from
   20.9 mean toward 45+; sample50 clean.
2. Stop splitting family strings. Checkpoint: the 40 CSS-list documents (16.2 mean)
   move with step 1; no document whose first token was correct regresses (the evidence
   table catches the ones Word substituted anyway).
3. Registry refactor with lazy loading and `Arc` bytes; every existing test green;
   conversion time on the 398 unchanged or better (47 eager loads become on-demand).
4. Embedded fonts. Checkpoint: case8 first line within 2 px of Word and its custom
   face present in the PDF.
5. Generic by `w:family` / `w:pitch`; the no-docDefaults default (6 corpus docs, 4.3
   mean).
6. Report flag and bench counter.

**Tests.** Unit: font-table parse (altName, family, pitch, embed rel + key); a
deobfuscation vector (case8's `.odttf` must parse with `ttf_parser` after decoding);
resolution order (quoted list stays intact; altName beats substitution; cycle guard;
`--font-policy recorded` ordering); synthetic-style flag. Fixtures: the 15 cluster-A1
stems in report 13.1, case8, case3 / case33 (Arial bullets), case7 / case48 / case49 /
case77 (Cambria headings). Gate: sample50 after each checkpoint; both-set sweep before
commit.

**Measurement targets.** Corpus: altName group 20.9 -> 45+, CSS-list group 16.2 -> 45+,
explicit-Calibri group (298 docs, 61.8) unchanged within 0.3. Fixtures: FONT-missing
rows (9) up; no row down more than 1.0.

**Risks.** altName may name a face absent on the machine (Hiragino Mincho ProN on
Linux): step 6 must then produce a metric-safe serif, and the report must say
`asset_missing`. Documents saved on another machine carry that machine's altName; the
`--font-policy` switch is the honest way to expose the choice rather than guess.
Registry refactor touches every text path; do it behind the shim, with the 47-face
snapshot test (`FaceId::all()` -> same PostScript names) as the guard.

**Effort.** 4-6 working days; checkpoint 1 alone is half a day and carries most of the
lift.

### 3.2 `word/theme/theme1.xml` font slots

**Scope.** Apply `minorHAnsi` / `majorHAnsi` for any theme face; add the
`eastAsiaTheme` / `cstheme` slots and `w:hint` for runs that carry them; read
script-specific theme fonts (`a:font script="Jpan"` etc.) when the run language is
East Asian. Plan.md Step 2c in full.

**Current code.** `load_theme` (mod.rs 938-990) reads `majorFont`/`minorFont` latin
typefaces and the colour scheme; `apply_rfonts` (1315-1360) applies the major slot for
any face but the minor slot only when the face starts with "aptos", by design (the
comment records the mini-set trade). `ThemeFonts { major, minor, colors }`.

**Word's rule.** A run with no explicit `w:ascii` / `w:hAnsi` but with a theme slot
uses the theme face for that slot; explicit family wins over the slot; East Asian
characters use the `eastAsia` slot / face when `w:hint="eastAsia"` or the character is
CJK; the theme may map scripts to specific faces (`a:font script=...`), else the
latin face.

**Design.** `ThemeFonts` gains `major_ea, minor_ea, major_cs, minor_cs: Option<String>`
and `script_fonts: HashMap<(slot, script), String>`. `apply_rfonts`: explicit family
wins (unchanged, including the Display-cache rule); otherwise slot -> face for any
face; record the step as `theme` in the font report. `RunStyle` gains
`family_ea: Option<String>` and `hint`; the shaper picks `family_ea` for CJK code
points (a per-character split of the run into segments, which `wrap_para_runs` already
does for tabs).

**Order and checkpoints.** (1) Delete the Aptos gate. Checkpoint: corpus
`minorHAnsi -> Cambria` group 11.2 -> 40+; the 47 FONT-theme fixture rows move; expect
file_2 / file_41 to *drop* until plan.md Step 4 (line box) lands, record them. (2)
East Asian slots and hints (the 4 Hiragino documents once 3.1 resolves the altName).
(3) Script fonts (no document in either set needs them today; keep for completeness).

**Tests.** Unit: theme minor = Cambria + docDefaults `minorHAnsi` -> family "Cambria";
explicit `w:ascii` beats the slot; Display-cache rule preserved. Fixtures: case9, 11,
22, 43, 44, 47, 66, 70-73 (theme-only rows), corpus theme group (12). Gate: sample50,
both sweeps.

**Effort.** Half a day for (1) plus measurement; 1-2 days for (2)-(3).

### 3.3 `word/document.xml` table properties

**Scope.** Column widths from `tcW` with Word's autofit, `tblGrid` as the fallback;
`tblInd`; the table left-edge rule by compatibility mode; row heights from content
and `trHeight` without tuned pads; cell margins in fixed-layout tables; merged cells
inside nested tables; floating tables. Plan.md Step 5 and Step 10 rows D and E in full.

**Current code.**
- `table_block` (3275): columns from `tblGrid/gridCol` only (`tcW` is never read).
- `table_col_widths` (2631): scales the grid to `tblW` (`Grid | Dxa | Pct`).
- `table_pref_width` (3495), `table_layout_fixed` (3514), `table_pad_h` (3521, returns
  the 108-twip default whenever the table is fixed-layout, discarding `tblCellMar`),
  `cell_pad_h` (3553), `row_height` (3484: `trHeight` val + `hRule=exact`).
- `table_row_pad` (2647) / `table_row_line_size` (2671) / `table_row_height_pt` (2685):
  line box `11.0 * line_mult` (or the max run size for two-column single-spaced
  tables), pads of 13 / 8 / 5 / 2 pt chosen by line count and multiplier, a special
  +18 pt for one-cell filled centred rows, `max(spec)` for atLeast, `max(18)` for
  multipliers above 1.01.
- `para_base` (2890): inside tables `after = min(after, 4)`, `before = min(before, 2)`.
- `cell_span` (3610) parses `gridSpan` / `vMerge`; `stroke_cell` (8295) paints.
- No reader for `tblInd` or `tblpPr`; no compatibility mode anywhere.

**Word's rules.**
- *Left edge.* Compatibility mode 14 and earlier: the table border sits at
  `margin + tblInd - left cell margin` so that cell text aligns with body text. Mode
  15: the border sits at `margin + tblInd` (Word 2013 "table edge" layout change). The
  nine fixtures with the 11-12 px shift are mode 14; the corpus is mostly mode 12 or
  absent (treated as 12). This is the first concrete need for `settings.xml`
  `compatibilityMode`.
- *Widths.* `tblLayout fixed`: use `tcW` (then `gridCol`) as given; total may exceed the
  page. Autofit (default): each column's preferred width from `tcW` (dxa / pct / auto),
  minimum from the widest unbreakable content, maximum from unwrapped content;
  distribute the table width (`tblW`, or the sum of preferred, capped at the text
  width) by the CSS-2 auto-layout rules, which is what Word approximates; `gridCol` is
  only a cache of the last layout.
- *Row height.* Content height = max over cells of (top cell margin + sum of the
  cell paragraphs' line boxes and spacing using the paragraph line-box formula +
  bottom cell margin); `trHeight` with `hRule=atLeast` (default) is a floor,
  `exact` overrides, clipping content. Cell margins default 0 top/bottom, 108 twips
  left/right, per `tblCellMar` then `tcMar`. Paragraph spacing inside cells is *not*
  clamped; Word applies the paragraph's own before/after.
- *Merged cells.* `gridSpan` widens across columns; `vMerge restart` starts a vertical
  span, `vMerge` (continue) joins it; the merged box takes the spanned rows' heights and
  the continuing cells draw no content and no interior border. Nested tables lay out
  inside the cell's content width recursively.
- *Floating tables.* `tblpPr`: `tblpX/Y` relative to `horzAnchor` / `vertAnchor`
  (margin / page / text), `leftFromText` etc. as the wrap distance; body text wraps
  beside the table like a square-wrapped float.

**Design.**
1. `Compat { mode: u8 }` parsed from `settings.xml` (`w:compatSetting name="compatibilityMode"`; absent -> 12), stored on the conversion context. Edge rule in `table_block`: `x = margin_l + tbl_ind - if compat.mode < 15 { pad_l } else { 0 }`.
2. `TableGeom` gains `pref: Vec<PrefWidth { Dxa(f32) | Pct(f32) | Auto }>` from the
   first row's `tcW` (spanning cells split evenly), `min_content`, `max_content` per
   column measured with `wrap_runs`; `table_col_widths` implements the autofit
   distribution; fixed layout bypasses it.
3. Row height: replace `table_row_pad` / `table_row_line_size` / the special cases
   with `cell_content_height = pad_t + sum(para_height(p)) + pad_b` where
   `para_height` is the same function paragraphs use in the body (plan.md Step 4), and
   `row_h = if exact { spec } else { max(content, spec) }`. Remove the `in_table`
   clamps in `para_base`. Keep `table_row_pad` behind a temporary flag for A/B until the
   both-set sweep confirms the removal.
4. `table_pad_h`: drop the fixed-layout early return; read `tblCellMar` always.
5. Merged cells: `cell_span` stays; the row-height pass accumulates spanned rows;
   nested tables recurse through `table_block` with `avail = cell content width`; add
   `nested_table_rowspan` as a unit fixture and trace the collapse there.
6. Floating tables: parse `tblpPr` into the same `Placement` the drawings use (3.4);
   the table becomes a float with square wrap; the body continues beside it.
7. `tblLook` / `tblStylePr`: banding and last-row conditions after the above (D 0.21).

**Order and checkpoints.** (1) compat mode + `tblInd` + edge rule: the nine +11/12 px
fixtures within 2 px; corpus `tblInd` docs (31, 16.1 mean) up. (2) row heights without
pads and no in-table clamps: corpus `trHeight` docs (29, 16.6 -> 40+), case51 back to
two pages, case6 back to three. (3) `tcW` + autofit: case46's column widths, the 88
corpus table documents (20.1 mean) up; text-boundary on table documents is the
sensitive metric here. (4) nested merges: nested_table_rowspan / table_vmerge_colspan /
file_47 / file_199 show three separate cells. (5) floating tables: case46 text beside
the table.

**Tests.** Unit: edge rule in both compat modes; autofit distribution on synthetic
tables (auto / dxa / pct / mixed, over-wide content); row height floor and exact;
nested merge geometry (extend `vmerge_gridspan_cell_covers_two_rows_and_cols`).
Fixtures: case6, 15, 32, 40, 45, 46, 51, 55, 61, 67; corpus table_bookmark_end,
file_134, Redline_CiceroDo, the four merged-cell docs. Gate: sample50 (it holds 10
table documents), both sweeps.

**Risks.** Highest coupling in the plan: row heights depend on the line-box formula
(plan.md Step 4); do (1) and (4)-(5) before Step 4 lands, (2)-(3) together with it.
The pads were tuned against the corpus, so expect some corpus table documents to dip
on the first cut of (2); each is a document to diff, not a reason to restore a pad.

**Effort.** 6-9 working days across the five checkpoints.

### 3.4 `word/document.xml` drawing placement

**Scope.** One placement resolver for anchored pictures, shapes and text boxes;
correct inline picture line boxes; all wrap modes; `srcRect` cropping; PNG alpha;
VML absolute positioning. Plan.md Step 6 and Step 10 row E in full. Preset shape
geometry (Step 7) stays separate.

**Current code.**
- `collect_images` (5018): every `w:drawing` with a `a:blip` becomes a `LaidImage
  { w, h, kind, slot, behind, z }`; VML `v:imagedata` only when the paragraph has no
  DrawingML picture, always as `ImageSlot::Flow` (no position).
- `drawing_slot` (5106): `Flow` for inline and for `wrapTopAndBottom`; otherwise
  `Float { align, page_x, page_y, col_x, para_y, pct_x/y, pct_w/h, v_align, wrap_square, dist_l, dist_r }`.
  `relativeFrom="character"` is folded into the column case; `line`, `leftMargin`,
  `rightMargin`, `insideMargin`, `outsideMargin`, `topMargin`, `bottomMargin` are not
  distinguished; `wrapTight` / `wrapThrough` / `wrapNone` are not read (`wrap_square`
  is a bool); `distT` / `distB` are not read; `simplePos` and `relativeHeight` are not
  used for placement.
- `wrap_square_inset` (6440) computes left/right insets for the current line from
  floats; `emit_image` (7464) paints; `linked_txbx_content` (4620) collects text-box
  content; `drawing_extent_pt` (5074) reads `wp:extent` or `a:ext`; `decode_image`
  (5606) returns JPEG / RGB / Reserve (no alpha channel; no `srcRect`).
- Observed: file_104 anchored text box painted at the page origin with the body text
  pulled into it; case41 wrapped text never lines up; case12 inline picture 55 px low
  and its transparent region opaque; case78 `srcRect` ignored; case60 text boxes as
  solid rectangles.

**Word's rules (ECMA-376 §20.4.2).** `wp:anchor`: horizontal position = `positionH`
(`relativeFrom` ∈ page, margin, column, character, leftMargin, rightMargin,
insideMargin, outsideMargin) with either `align` (left, right, center, inside, outside)
or `posOffset` (EMU); vertical = `positionV` (`relativeFrom` ∈ page, margin,
paragraph, line, topMargin, bottomMargin, insideMargin, outsideMargin) with `align`
(top, bottom, center, inside, outside) or `posOffset`. `paragraph` means the top of the
anchoring paragraph's first line, `line` the current line. Wrap: `wrapNone` (text
flows under/over), `wrapSquare` (text avoids the bounding box plus `distL/R/T/B`, on
the `wrapText` side(s)), `wrapTight` / `wrapThrough` (polygon; Square is the accepted
approximation), `wrapTopAndBottom` (text only above and below). `behindDoc` and
`relativeHeight` give the paint order. `wp:inline`: the picture is a glyph on the line;
its line box is `cy` plus `effectExtent` top/bottom, bottom on the baseline. `a:srcRect`
`l/t/r/b` in 1/1000 of a percent crop the source before scaling into the extent. PNG
alpha composes over the page. VML: `v:shape style="position:absolute; margin-left:..;
margin-top:..; mso-position-horizontal(-relative):..; mso-position-vertical(-relative):..; z-index:.."`
with the same semantics.

**Design.**
1. `Placement { x, y, w, h, wrap: None|Square{sides,dist}|TopBottom, behind, z }`
   computed by `resolve_anchor(anchor, ctx: &PlaceCtx { page, margins, column, para_top, line_top, cursor_x })`
   implementing the full `relativeFrom` × `align|posOffset` matrix; unit-tested against
   hand-computed cases. Used by pictures, DrawingML shapes, text boxes and (3.3)
   floating tables.
2. Text boxes: `LaidTextBox` gets its `Placement` from the same resolver; content laid
   out as a mini-document inside the box with `lIns/tIns/rIns/bIns` (default 0.1 in /
   0.05 in); `wps:txbx` and `v:textbox` share the path. Fixes file_104 and file_70.
3. Inline pictures: line box `max(text_box, cy + effect_t + effect_b)`, image bottom at
   the baseline minus `effect_b`; no extra text line. Fixes the 55 px in case12.
4. `srcRect`: clip rectangle + scaled placement in `emit_image`; `decode_image` keeps
   the full source. Fixes case78.
5. PNG alpha: `decode_image` returns an optional alpha plane; `pdf.rs` writes an
   `/SMask` XObject (`DeviceGray`) referenced from the image XObject.
6. Wrap loop: `wrap_square_inset` consumes `Placement.wrap` per line (`Square` with
   sides and distances; `TopBottom` blocks the whole line band; `None` no inset), for
   floats whose vertical band intersects the line.
7. VML: parse the `style` string into the same `Placement`; picture and text box VML
   go through steps 1-2 instead of the `Flow`-only path.

**Order and checkpoints.** (1) resolver + unit tests (no behaviour change until
wired). (2) Text boxes through the resolver: file_104 box beside its paragraph, file_70
three bands. (3) Inline line box + `srcRect` + alpha: case12 and case78 within 2 px and
docxide-pdf's Jaccard (96.7 / 99.6) as the target. (4) Wrap modes: case41 text-boundary
above 0. (5) VML positioning: the 18 corpus `w:pict` documents.

**Tests.** Unit: the placement matrix; wrap insets for the three modes; `srcRect`
arithmetic; SMask emission (parse the PDF back with `mutool` and check the XObject has
`/SMask`). Fixtures: case12, 16, 27, 41, 42, 60, 78; corpus anchor_images, file_104,
file_70, broken_media_rel (placeholder size). Gate: sample50, both sweeps.

**Risks.** Wrap changes the line breaker for every paragraph near a float; the corpus
has 15 anchored-drawing documents where jubarte is currently *ahead* of docxide-pdf
(37.1 vs 19.6): those are the regression canaries. Keep the old `Flow` path selectable
until the sweep confirms.

**Effort.** 5-8 working days.

## 4. Sketches for the next tier (D 0.2-0.8)

- **Latent built-in styles** (D 0.32, diff 0.3, high). 22 corpus documents reference
  `Title`, `Heading1-4`, `Subtitle` that `styles.xml` does not define; Word applies its
  built-in definitions. Ship a table of those definitions for the two templates seen
  (2007-2010 "Cambria/Calibri" set: Heading1 = Cambria 14 pt bold accent1, before 480
  after 0, keepNext; 365 "Aptos" set: Heading1 = Aptos Display 20 pt accent1 before 360
  after 80, and so on for Title / Subtitle / Heading2-4), selected by the theme fonts,
  and route `apply_latent_ppr` through it. Measure on the eight heading-demo stems in
  section 2 (28-54 vs 74-97).
- **Footnotes** (D 0.44, diff 0.7, high). Parse `footnotes.xml` (separator and
  continuation separator ids -1 / 0, the rest by id); on each page reserve the height
  of the notes referenced on that page (line-box formula + `footnotePr` spacing), draw
  the separator rule and notes at the bottom, superscript the reference in the body;
  docxide-pdf `pdf/footnotes.rs` (`compute_footnote_height`, `render_page_footnotes`,
  `draw_note_separator`) is a working reference. Measure: potpourritest page 1 ends at
  the same row as Word; case18 / 74 / 75 / 76.
- **Endnotes placement** (D 0.21, diff 0.3, high). `append_endnotes` exists;
  endnotes_sample has text-boundary 0: check `endnotePr` position (`sectEnd` vs
  `docEnd`), the separator and the reference marks against the oracle.
- **Numbering overrides** (D 0.03 by lift, 13 of 18 both-fail, diff 0.3, high).
  `lvlOverride` / `startOverride` / `lvlRestart` in `load_numbering`; per-`num` counter
  state. Cheap, and the labels are visibly wrong today.
- **Footer block placement** (D 0.34, diff 0.5, medium). Measure the multi-line footer
  rule on complex_style_attr / file_30 and five more corpus footers (plan.md Step 10 G).
- **A4 MediaBox** (D 0.35, diff 0.05). 595.2 x 841.92.
- **Charts** (D 0.39, diff 0.8, high). Pie, line, area on top of the bar renderer;
  docxide-pdf `pdf/charts.rs` / `charts_radial.rs` / `chart_legend.rs` as the design
  reference; six fixtures.
- **SmartArt** (D 0.20, diff 0.9). Only pre-laid `dsp:` shapes render; the layout
  algorithms are a project. Park.
- **Comments balloons** (D 0.21, diff 0.6). Word's "print with markup" export scales
  the page to leave a balloon column (case63 / case64); the corpus oracles were
  exported without markup. Park unless a markup-mode oracle is wanted.
- **`settings.xml` compatibility mode: research item, not a feature.** The corpus is
  158 mode-12, 154 absent, 58 mode-15, 28 mode-14; the fixtures 56 mode-14, 16 mode-15.
  Known mode-dependent rules: the table edge (3.3, needed now), space-before after a
  hard page break, `doNotExpandShiftReturn` (2 docs), `balanceSingleByteDoubleByteWidth`
  (2), `spaceForUL` / `ulTrailSpace` (4 / 2), Word 2013 hyphenation tracking. Parse the
  mode and the flags present (one function), then measure each rule on the documents
  that carry it before implementing any.
- **`lastRenderedPageBreak`** (D 0.54 by lift, diff 0.2). This is Word's own record of
  where it broke pages when the file was last saved. Using it as layout input would
  reproduce the oracle on every document saved by the same Word render and inflate the
  bench without making the converter better at laying out a document it has never
  seen. Do not use it for layout. It is useful as a *diagnostic*: compare jubarte's
  break positions with it to find the first paragraph where pagination diverges, which
  is exactly the per-document diff plan.md Step 4 needs. Arthur's call if he wants it
  as a bench mode; it should never be the default.
- **Hyphenation** (D 0.09, diff 0.9). One fixture, zero corpus documents; needs a
  dictionary and Word's zone rules. Park.
- **Underline / strike / highlight / vertAlign / hyperlinks / page breaks** show
  non-zero lift but low confidence: they are handled today and their lift is the
  fixture-wide gap. Re-measure after sections 3.1-3.4 land; whatever remains is real.

## 5. Parts with nothing to gain

`webSettings.xml`, `customXml/*`, `docProps/*`, `word/glossary/*`,
`stylesWithEffects.xml`, `vbaProject.bin`, `activeX/*`, `commentsExtended.xml`,
`commentsIds.xml`, `commentsExtensible.xml`, `people.xml`: not inputs to Word's page
layout, or only meaningful with balloon rendering. D = 0; leave unread.
