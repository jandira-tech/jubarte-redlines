<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# Issue #1914 round 3: Word versus Jubarte

**Decision: neither complete rule is established. No renderer change lands.** A margin-preserving formula now fits the separated first row in all 16 applicable probes across rounds 1–3, including its exact heading fragments. It does not supply the remaining rows or their grouping. The history-free shared-grid allocator also remains unresolved. The user’s acceptance criterion is one formula predicting **every marker within 0.5 pt and every heading line count across all three rounds**, not a first-row-only fit. Non-uniqueness of a fitting formula is not a reason for rejection; the uncovered or incorrectly predicted markers are.

Baseline source: `cd4d856d028af0bfe9eb407f62d9ff9a31a2abac`, branch `fix/1914-3d43-pages`. Round-3 baseline binary SHA-256: `c2ab768722d2ea9eb1d834ffd5f08fca6290265c98e09c7c6151aa76e1f2067c`. Measurements use the existing baseline PDFs listed in `probes3/expected.md`, not a new build or an after-render. All paths below are relative to `/Users/arthrod/temp/T/jubarte-loop/agents/issues/1914/` unless stated otherwise. A copy of this report is committed as `docs/ISSUE_1914_ROUND3.md`.

Word 16.115 PDFs were supplied by the maintainer; `word3/results.tsv` records CLEAN / rc=0 for all 16. The actual PDF layout is `word3/p14dN/p14dN_byword.pdf`. **Word was never started or controlled.** This is a layout discrepancy, not evidence of a Word-invalid file.

## Method and integrity

Fresh `pdftotext -bbox-layout`, `mutool draw -F trace`, and `pdfinfo` extraction covered **all 40 probes / 80 PDFs across all three rounds**. All 40 probe SHA-256 values match their manifest and Word-staged DOCX. Every PDF has one page. Every emitted Jubarte marker bound agrees with its saved round-specific measurement within 0.001 pt. Every payload reconstructs exactly after ignoring spaces. Raw extractions, PDF hashes, commands/exit codes, all 40 paired measurements, and formula calculations are in `probes3/word-measurements/`.

Coordinates are PDF text xMin in points from the left page edge. Borders are independently extracted left edges of vertical black fills; duplicate edges within 0.03 pt are coalesced. PDF quantization is about 0.24 pt, so text offsets are not used to infer borders. Heading lines exclude the marker paragraph. Empty headings keep their blank paragraph and height. Full marker tables include wrap fragments and missing markers.

Round 3 has 416/416 Word markers and 415/416 Jubarte markers. In d10, Word emits `O` at x=558.720; Jubarte omits it. Jubarte also wraps `so` and `my` into two fragments; Word keeps each on one line. The omission is a failure, not a zero coordinate, and is excluded only from the numeric maximum in the overview.

## Round-3 overview

| ID | Word x: to / it / ye | Ours x: to / it / ye | Heading lines W/O | Heading border width W/O | Max emitted marker error |
|---|---|---|---|---|---:|
| p14d1 | 316.320 / 316.320 / 316.320 | 337.920 / 337.920 / 337.920 | 1/1 | 226.800/226.840 | 136.800 |
| p14d2 | 316.320 / 316.320 / 316.320 | 337.920 / 337.920 / 337.920 | 1/1 | 226.800/226.840 | 136.800 |
| p14d3 | 544.560 / 431.520 / 431.520 | 337.920 / 337.920 / 337.920 | 9/1 | 39.120/226.840 | 206.640 |
| p14d4 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 9/3 | 39.120/116.010 | 90.240 |
| p14d5 | 356.640 / 347.040 / 543.840 | 454.320 / 454.320 / 454.320 | 1/3 | 227.040/116.010 | 107.280 |
| p14d6 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 9/3 | 39.120/116.010 | 90.240 |
| p14d7 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 9/3 | 39.120/116.010 | 90.240 |
| p14d8 | 356.640 / 347.040 / 543.840 | 454.320 / 454.320 / 454.320 | 1/3 | 227.040/116.010 | 107.280 |
| p14d9 | 539.760 / 437.760 / 437.760 | 449.280 / 449.280 / 449.280 | 6/2 | 38.640/116.010 | 90.480 |
| p14d10 | 549.360 / 417.840 / 417.840 | 459.840 / 459.840 / 459.840 | 18/3 | 39.600/116.010 | 89.520 + missing `O` |
| p14d11 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 9/3 | 39.120/116.010 | 90.240 |
| p14d12 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 1/1 | 39.120/116.010 | 90.240 |
| p14d13 | 544.560 / 431.520 / 431.520 | 454.320 / 454.320 / 454.320 | 0/0 | 39.120/116.010 | 90.240 |
| p14d14 | 316.320 / 316.320 / 316.320 | 337.920 / 337.920 / 337.920 | 1/1 | 226.800/226.840 | 136.800 |
| p14d15 | 432.240 / 432.240 / 432.240 | 482.880 / 482.880 / 482.880 | 0/0 | 96.960/75.350 | 50.640 |
| p14d16 | 316.320 / 316.320 / 316.320 | 337.920 / 337.920 / 337.920 | 1/1 | 226.800/226.840 | 136.800 |

## Rule (a): history-free shared-grid allocation

**Not established.** The three controls separate heading content from the row-1 trailing reservation:

| Probe | Change from B1 | Word to/it/ye | Ours to/it/ye | Max marker error |
|---|---|---:|---:|---:|
| p14b1 | None | 316.320 | 337.920 | 136.800 |
| p14d15 | Empty payload; paragraph retained | 432.240 | 482.880 | 50.640 |
| p14d16 | Only row-1 wAfter 869 → 351 | 316.320 | 337.920 | 136.800 |

B1 → d15 moves Word’s heading start +115.920 pt and Jubarte’s +144.960 pt. The error survives removal of the long heading; a heading-only content-minimum patch cannot be the complete fix. B1 and d16 have identical **all-marker x vectors and all three row-edge vectors** in both renderers. This rejects row-1 wAfter as the sole cause. It does **not** prove wAfter is ignored: row 3 still declares 869, so a maximum reservation could mask the row-1 edit.

A cached-grid proportional rule is also insufficient: B1 and d15 have identical geometry but different Word allocations. A single constant offset fails: B1 needs `up` +9.600 pt but `a` +136.800 pt relative to ours. Moving only the heading fails all the other changed cells. Simply reusing live-fixed B3 for d15 misses `to` by 7.440 pt and `up` by 25.440 pt.

One useful constraint for the next model: in B1 the heading’s right border equals row 2’s `a` left border, 537.360 pt; in d15 both are 523.680 pt. The XML heading ends at grid boundary 14 while `a` begins at 13. Under a shared ordered grid this implies a collapsed interval 13–14, within export precision. B2 instead has heading end 555.840 and `a` start 518.160, a 37.680 pt interval, although only the heading tcW changed. A positive-width-per-column interpolation cannot represent both without an active constraint/collapse rule. This is a geometric inference, not a proven Word algorithm.

## Rule (b): triggered separate allocation

### What round 3 establishes observationally

| Control | Full-vector result | Consequence |
|---|---|---|
| d1 / d2 / d14 | Same marker x and row-edge vectors as B1 | Empty or autofit old table snapshot does not trigger these geometries, even with R+C. Old fixed matters. |
| d3 | Same marker x, row edges and nine heading lines as c6 | Explicit live autofit **does not suppress** the split. The current `live.is_none()` check is too narrow. Live fixed still gives B3/c12. |
| d4 | Same as c6 | Identical old/live row snapshot still triggers. A changed old row value is unnecessary. |
| d6 / d7 | Same as c6 | Old heading width and span can both equal live values; record presence suffices for this tested shape. |
| d5 / d8 | Identical to each other, but all three rows differ from c6 | Moving either history record to row 2 changes grouping/order, not only the recorded row. |
| d9 / d10 | First-row width changes with margins; heading has 6 / 18 lines | Margin-preserving shrinkage survives the new controls. The other rows also change. |
| d11 | Same as c6 | Changing only row-1 wAfter does not affect this allocation. |
| d12 / d13 | Same marker x and border x vectors as c6; heading 1 / 0 lines | The triggered first-row widths do not depend on the tested heading payload. |

For this tested family, the trigger is old fixed plus row/cell history, under omitted **or explicit autofit** live layout. This is not authorization to generalize to arbitrary table histories or to change only that predicate: it would route d3 into another allocator already known to be wrong.

### Numeric candidate and its limit

For the nine real cells of a separated first row, excluding skipped-grid placeholders:

```text
T = 10116 / 20 = 505.8 pt
p = left cell margin + right cell margin
w_i = live tcW_i / 20
width_i = p + (w_i - p) * (T - n*p) / (sum(w_i) - n*p)
x_i = x_first + sum(width_j for j < i)
```

The following evaluation anchors x_first at the measured first marker; it checks the allocation independently of border/text-origin quantization. Widths are not fitted per probe; p is 0, 10.8, or 21.6 from the XML. Monospaced 10 pt Courier New glyph advance is approximately 6.001 pt (adjacent glyph origins in the Word PDF trace). Greedy wrapping in width_last−p reproduces **the exact fragments**, including the spaced heading c11, in all 16 applicable probes. The 36-character unspaced heading gives 9, 2, 6, and 18 lines at the corresponding widths. This is a strong partial fit.

| Probe | p | First-row max error | Predicted / Word heading lines | Remaining rows left as ours: max error |
|---|---:|---:|---|---:|
| p14b4 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14b5 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14c6 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14c7 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14c9 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14c10 | 10.8 | 0.428 | 2/2 | 18.240 (`O`) |
| p14c11 | 10.8 | 0.266 | 10/10 | 34.560 (`a`) |
| p14d3 | 10.8 | 0.266 | 9/9 | 122.640 (`a`) |
| p14d4 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14d6 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14d7 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14d9 | 0.0 | 0.220 | 6/6 | 36.480 (`as`) |
| p14d10 | 21.6 | 0.368 | 18/18 | 46.800 (`am`) |
| p14d11 | 10.8 | 0.266 | 9/9 | 34.560 (`a`) |
| p14d12 | 10.8 | 0.266 | 1/1 | 34.560 (`a`) |
| p14d13 | 10.8 | 0.266 | 0/0 | 34.560 (`a`) |

**This does not pass the requested rule gate.** Replacing only row 1 leaves, for example, c6 `a` 34.560 pt wrong and d10 `am` 46.800 pt wrong, plus the missing `O`. Extending the independent-cell formula to every row fails c6: row 2 `it` would be 347.110 pt instead of 431.520 pt (84.410 pt error). Retaining one shared vector for the unrecorded rows fails d5/d8: unrecorded row 1 `to` is 356.640 while unrecorded row 3 `ye` is 543.840, despite the same live grid start. Applying the separated-first-row formula to row 1 there would put `to` at 544.679, 188.039 pt too far right.

A plausible **unproven** grouping hypothesis is: rows before the history-bearing row form an autofit block; that row is independent; following rows form a fixed block. c6 would be independent row 1 plus fixed rows 2–3; d5 would be autofit row 1, independent row 2, fixed row 3. No current probe isolates those blocks to validate that numerical prediction. No formula currently predicts all their markers. The user’s complete-rule requirement therefore prevents a partial renderer patch.

The separate-allocation display is worse for readability in the supplied heading: c6 makes nine short lines and d10 eighteen. Any eventual reproduction belongs under `RevisionStyle::Word`, with a default-style test and `docs/WORD_DIFFERENCES.md` entry. The shared-grid mismatch has no demonstrated revision-only cause and should be fixed in shared layout if established. Neither change has been made.

## Smallest next round proposed: 10 probes

This is a concrete ten-probe discrimination set, with no repeats of existing controls. It targets the missing block allocator and shared-grid constraints, rather than repeating the trigger factorial. Keep the same five-part packages, styles, fonts, page settings, marker text and cell XML; retain empty payload paragraphs. Deleting rows is the named factor, not a renumbering of marker identities or cell spans. No round-4 DOCXs or Word runs were created in this continuation.

| ID | Reference / exact change | Competing predictions or decisive comparison |
|---|---|---|
| e1 | B1: retain only original row 1 | If d5’s prefix is an isolated autofit block, all row-1 edges must equal d5 row 1 (to=356.640, heading 1 line). Tests independent autofit versus influence from following fixed rows. |
| e2 | B3: retain only original row 1 | If c6 row 1 is ordinary independent fixed allocation, it must match c6 row 1 (to=544.560, heading 9 lines). Tests the numeric margin formula independently of history. |
| e3 | B3: delete only original row 1, retain rows 2–3 | Fixed-tail hypothesis predicts every edge/marker equals c6 rows 2–3, including it=ye=431.520. If not, history affects the tail beyond partitioning. |
| e4 | B3: retain only original row 3, preserving gridBefore/wBefore/gridAfter/wAfter | Fixed-tail hypothesis predicts d5 row 3, I=77.760 and ye=543.840. Decides why the leading reservation disappears there. |
| e5 | B1: replace every live tcW by the sum of the saved grid columns its cell spans; change no span or skip | Removes conflicting cell/grid preferences. Compare all intervals, especially 13–14, against B1 to distinguish preference reconciliation from a content-only allocator. |
| e6 | e5: empty only the heading payload, retaining its paragraph | Paired with e5 and existing B1/d15, isolates heading minimum under consistent versus conflicting preferences. A single constraint solver must fit both pairs. |
| e7 | d16: change only row-3 wAfter 869 → 351 | Both trailing reservations now match the saved grid. Distinguishes max-reservation masking from ignored reservations without repeating d16. |
| e8 | c6: move its byte-identical R from row 1 to row 3 | Hypothesis predicts an autofit prefix of original rows 1–2 plus independent fixed row 3; compare prefix against e9 and tail against e4. This is the missing third history position. |
| e9 | B1: retain only original rows 1–2 | Supplies the prefix control for e8 and isolates the effect of the leading-skip third row in history-free allocation. |
| e10 | B1: set both table side margins to zero (108 → 0), leaving all else unchanged | Connects shared-grid margin/minimum allocation to the existing triggered zero-margin d9. Distinguishes a true content constraint from a constant margin adjustment. |

e1–e4 are the four nonredundant block controls exposed by d5/c6; e5/e6 are a paired preference/content control; e7 completes the reservation control; e8/e9 validate grouping at a new boundary; e10 separates shared-grid margin minima. Existing A1–A6, B1–B6, c1–c12 and d1–d16 remain mandatory acceptance cases. These ten observations could establish or refute the candidate model; no finite probe set can guarantee that Word will follow it. Do not promise round 4 will produce a landable formula before its data exist. Write numerical predictions from the candidate solver before measuring new Word PDFs, then require every old/new marker and heading count to pass.

## Four tracked documents and page guard

Fresh `pdfinfo` reads of the existing PDFs, using `inputs.json` for the Word paths:

| Pair | Word | Ours before | Ours after |
|---|---:|---:|---|
| 3d4318d7e9 | 15 | 11 | Not rendered: no renderer change |
| 6926d6b8fd | 13 | 14 | Not rendered: no renderer change |
| 2fa06a6cb0 | 7 | 7 | Not rendered: no renderer change |
| 435d2c2474 | 14 | 14 | Not rendered: no renderer change |

No candidate renderer build/conversion, no pagination improvement, and no 30-fixture before/after guard are claimed. The user made these contingent on a rule being established and implemented. Prior reports also say that the 30-fixture guard was not run in those continuations; this report does not invent a fixture list or reuse an unverified guard result.

## Reproduction and verification

Run from `/Users/arthrod/temp/T/wt-1914`:

```sh
export CARGO_BUILD_JOBS=4
python3 /Users/arthrod/temp/T/jubarte-loop/agents/issues/1914/probes3/word-measurements/compare.py
# Individual inputs (same commands for every probe and both renderers):
pdftotext -bbox-layout /Users/arthrod/temp/T/jubarte-loop/agents/issues/1914/word3/p14d1/p14d1_byword.pdf -
mutool draw -F trace /Users/arthrod/temp/T/jubarte-loop/agents/issues/1914/word3/p14d1/p14d1_byword.pdf 1
```

Artifact audit: exit 0; 40 original/staged/manifest hash matches; 80 one-page PDFs; 920 Word and 919 Jubarte emitted markers; all baseline bounds reproduced; all payloads assigned and reconstructed; all claimed control-vector equalities checked; all 16 first-row formula fragment predictions checked. Coverage: N/A lines, N/A branches (PDF artifact checks, no unit-test run).

RED / implementation / GREEN: not run / no renderer change / not run. No unvalidated rule was encoded as a failing regression test. Committed changes are this evidence report, a CHANGELOG Added entry explicitly documenting the investigation, and the required documentation-generator outputs. `docs/locations.md` refreshes existing identifier call chains and errors; `jubarte-python/Cargo.lock` records the already-declared `unicode-segmentation` dependency at the same 1.13.3 version as the root lockfile. Neither adds a renderer fix. `docs/WORD_DIFFERENCES.md` is unchanged because no behavior landed. All commands used `export CARGO_BUILD_JOBS=4`; Cargo processes initiated here ran sequentially. Final checks:

| Check | Result |
|---|---|
| PDF/hash/control/formula artifact checks | exit 0 |
| `cargo fmt --all` | exit 0 |
| `cargo clippy --all-features --all-targets -- -D warnings` | exit 0 |
| `cargo run -q --bin jubarte -- --help` | exit 0 |
| `scripts/gen_docs.sh` | exit 0; generated updates included |
| REUSE lint (before and after staging the report) | exit 0 |
| Targeted nextest / coverage | N/A: no source or test behavior changed; no test suite run |
| Four after-renders / 30-fixture page guard | Not run: no established renderer candidate |

Logs are under `probes3/word-measurements/`. No push. The final chat report identifies the evidence commit; it is not a fix SHA.

## Complete round-3 marker comparisons

All markers are on page 1. Δx = ours − Word. Heading cell visible line count is marker lines + payload lines; a blank second paragraph still occupies height. The full JSON retains all x/y bounds.

### p14d1

Old table-property snapshot: remove only tblLayout=fixed, retaining empty tblPrChange.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 132.960 | 123.360 | -9.600 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 159.120 | 153.600 | -5.520 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 187.440 | 190.320 | 2.880 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 213.600 | 220.320 | 6.720 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 239.520 | 250.560 | 11.040 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 265.200 | 280.320 | 15.120 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 291.120 | 310.080 | 18.960 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 159.120 | 153.600 | -5.520 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 213.600 | 220.320 | 6.720 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 392.160 | 360.720 | -31.440 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 468.240 | 383.520 | -84.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 543.120 | 406.320 | -136.800 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 563.760 | 566.640 | 2.880 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 98.400 | 105.120 | 6.720 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 132.960 | 123.360 | -9.600 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 159.120 | 153.600 | -5.520 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 187.440 | 190.320 | 2.880 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 213.600 | 220.320 | 6.720 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 239.520 | 250.560 | 11.040 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 265.200 | 280.320 | 15.120 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 291.120 | 310.080 | 18.960 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 316.320 | 337.920 | 21.600 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 72.000, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 72.480/95.040 | 72.000/95.160 |
| 2 | 72.000, 153.360, 207.840, 310.560, 386.400, 462.480, 537.360, 558.000, 577.920 | 72.000, 147.920, 214.730, 332.180, 354.980, 377.790, 400.590, 561.000, 577.800 | 95.520/107.040 | 95.150/106.980 |
| 3 | 92.880, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 99.430, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 107.520/118.800 | 106.980/119.310 |

### p14d2

Old tblLayout: fixed -> autofit; live layout remains omitted.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 132.960 | 123.360 | -9.600 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 159.120 | 153.600 | -5.520 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 187.440 | 190.320 | 2.880 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 213.600 | 220.320 | 6.720 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 239.520 | 250.560 | 11.040 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 265.200 | 280.320 | 15.120 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 291.120 | 310.080 | 18.960 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 159.120 | 153.600 | -5.520 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 213.600 | 220.320 | 6.720 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 392.160 | 360.720 | -31.440 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 468.240 | 383.520 | -84.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 543.120 | 406.320 | -136.800 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 563.760 | 566.640 | 2.880 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 98.400 | 105.120 | 6.720 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 132.960 | 123.360 | -9.600 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 159.120 | 153.600 | -5.520 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 187.440 | 190.320 | 2.880 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 213.600 | 220.320 | 6.720 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 239.520 | 250.560 | 11.040 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 265.200 | 280.320 | 15.120 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 291.120 | 310.080 | 18.960 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 316.320 | 337.920 | 21.600 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 72.000, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 72.480/95.040 | 72.000/95.160 |
| 2 | 72.000, 153.360, 207.840, 310.560, 386.400, 462.480, 537.360, 558.000, 577.920 | 72.000, 147.920, 214.730, 332.180, 354.980, 377.790, 400.590, 561.000, 577.800 | 95.520/107.040 | 95.150/106.980 |
| 3 | 92.880, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 99.430, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 107.520/118.800 | 106.980/119.310 |

### p14d3

Add explicit LIVE tblLayout=autofit; old fixed and R unchanged.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 123.360 | -90.240 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 153.600 | -106.080 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 190.320 | -134.640 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 220.320 | -150.240 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 250.560 | -165.600 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 280.320 | -180.480 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 310.080 | -195.600 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 337.920 | -206.640 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 153.600 | -77.760 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 220.320 | -84.240 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 337.920 | -93.600 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 360.720 | -103.440 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 383.520 | -113.280 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 406.320 | -122.640 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 566.640 | 9.360 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 105.120 | -27.840 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 123.360 | -75.360 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 153.600 | -77.760 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 190.320 | -81.840 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 220.320 | -84.240 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 250.560 | -86.640 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 280.320 | -89.040 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 310.080 | -91.680 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 337.920 | -93.600 | 1/1 |

Word heading: 9 payload lines; fragments: `Elec` / `trop` / `hysi` / `olog` / `ical` / `Char` / `acte` / `riza` / `tion`.

Ours heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 72.480/185.760 | 72.000/95.160 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 147.920, 214.730, 332.180, 354.980, 377.790, 400.590, 561.000, 577.800 | 186.720/198.000 | 95.150/106.980 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 99.430, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 198.480/210.000 | 106.980/119.310 |

### p14d4

Replace only the OLD row-property snapshot with an exact copy of live row-1 trPr (gridAfter=2, wAfter=869; no gridBefore).

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 9 payload lines; fragments: `Elec` / `trop` / `hysi` / `olog` / `ical` / `Char` / `acte` / `riza` / `tion`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/185.760 | 72.000/117.810 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 186.720/198.000 | 117.810/129.640 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 198.480/210.000 | 129.640/141.970 |

### p14d5

Move the byte-identical trPrChange from row 1 to row 2; leave every live property unchanged.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 138.000 | 188.400 | 50.400 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 168.720 | 225.360 | 56.640 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 205.920 | 278.160 | 72.240 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 236.640 | 314.880 | 78.240 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 267.360 | 351.360 | 84.000 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 297.600 | 387.360 | 89.760 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 328.080 | 423.360 | 95.280 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 356.640 | 454.320 | 97.680 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 226.560 | 225.360 | -1.200 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 276.000 | 314.880 | 38.880 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 347.040 | 454.320 | 107.280 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 396.000 | 491.040 | 95.040 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 445.200 | 527.520 | 82.320 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 493.440 | 563.520 | 70.080 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 541.680 | 575.280 | 33.600 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 77.760 | 144.240 | 66.480 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 210.480 | 188.400 | -22.080 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 257.040 | 225.360 | -31.680 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 322.080 | 278.160 | -43.920 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 368.160 | 314.880 | -53.280 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 414.240 | 351.360 | -62.880 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 459.360 | 387.360 | -72.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 504.480 | 423.360 | -81.120 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 543.840 | 454.320 | -89.520 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 132.480, 163.200, 200.400, 230.880, 261.600, 292.080, 322.560, 350.880, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/95.280 | 72.000/117.810 |
| 2 | 72.000, 220.800, 270.480, 341.280, 390.480, 439.680, 487.920, 536.160, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 96.240/107.520 | 117.810/129.640 |
| 3 | 72.000, 204.960, 251.280, 316.560, 362.400, 408.480, 453.600, 498.720, 538.320, 577.920 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 108.480/119.760 | 129.640/141.970 |

### p14d6

Old heading tcW: 4002 -> 835 only; old gridSpan remains 6.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 9 payload lines; fragments: `Elec` / `trop` / `hysi` / `olog` / `ical` / `Char` / `acte` / `riza` / `tion`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/185.760 | 72.000/117.810 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 186.720/198.000 | 117.810/129.640 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 198.480/210.000 | 129.640/141.970 |

### p14d7

Old heading gridSpan: 6 -> 4 only; old tcW already 835 in d6.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 9 payload lines; fragments: `Elec` / `trop` / `hysi` / `olog` / `ical` / `Char` / `acte` / `riza` / `tion`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/185.760 | 72.000/117.810 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 186.720/198.000 | 117.810/129.640 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 198.480/210.000 | 129.640/141.970 |

### p14d8

Move the byte-identical tcPrChange from row 1 cell 9 (to) to row 2 cell 4 (it), both starting at grid column 10.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 138.000 | 188.400 | 50.400 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 168.720 | 225.360 | 56.640 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 205.920 | 278.160 | 72.240 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 236.640 | 314.880 | 78.240 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 267.360 | 351.360 | 84.000 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 297.600 | 387.360 | 89.760 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 328.080 | 423.360 | 95.280 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 356.640 | 454.320 | 97.680 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 226.560 | 225.360 | -1.200 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 276.000 | 314.880 | 38.880 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 347.040 | 454.320 | 107.280 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 396.000 | 491.040 | 95.040 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 445.200 | 527.520 | 82.320 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 493.440 | 563.520 | 70.080 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 541.680 | 575.280 | 33.600 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 77.760 | 144.240 | 66.480 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 210.480 | 188.400 | -22.080 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 257.040 | 225.360 | -31.680 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 322.080 | 278.160 | -43.920 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 368.160 | 314.880 | -53.280 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 414.240 | 351.360 | -62.880 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 459.360 | 387.360 | -72.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 504.480 | 423.360 | -81.120 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 543.840 | 454.320 | -89.520 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 132.480, 163.200, 200.400, 230.880, 261.600, 292.080, 322.560, 350.880, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/95.280 | 72.000/117.810 |
| 2 | 72.000, 220.800, 270.480, 341.280, 390.480, 439.680, 487.920, 536.160, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 96.240/107.520 | 117.810/129.640 |
| 3 | 72.000, 204.960, 251.280, 316.560, 362.400, 408.480, 453.600, 498.720, 538.320, 577.920 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 108.480/119.760 | 129.640/141.970 |

### p14d9

Table cell side-margin parameter: 108 -> 0 twips on both sides (5.4 -> 0 pt per side).

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 72.480 | 72.480 | 0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 209.760 | 183.360 | -26.400 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 255.600 | 220.320 | -35.280 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 321.120 | 273.120 | -48.000 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 366.480 | 309.600 | -56.880 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 412.080 | 346.320 | -65.760 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 456.480 | 382.320 | -74.160 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 501.120 | 418.080 | -83.040 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 539.760 | 449.280 | -90.480 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 72.480 | 72.480 | 0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 250.320 | 220.320 | -30.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 321.360 | 309.600 | -11.760 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 437.760 | 449.280 | 11.520 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 468.000 | 485.760 | 17.760 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 498.240 | 522.480 | 24.240 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.000 | 558.240 | 30.240 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 554.640 | 570.000 | 15.360 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 144.720 | 138.960 | -5.760 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 219.840 | 183.360 | -36.480 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 250.320 | 220.320 | -30.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 291.120 | 273.120 | -18.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 321.360 | 309.600 | -11.760 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 351.600 | 346.320 | -5.280 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 381.360 | 382.320 | 0.960 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 411.120 | 418.080 | 6.960 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 437.760 | 449.280 | 11.520 | 1/1 |

Word heading: 6 payload lines; fragments: `Electr` / `ophysi` / `ologic` / `alChar` / `acteri` / `zation`.

Ours heading: 2 payload lines; fragments: `Electrophysiologica` / `lCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 209.280, 255.120, 320.640, 366.000, 411.600, 456.000, 500.640, 539.280, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/151.920 | 72.000/106.480 |
| 2 | 72.000, 249.840, 320.880, 437.280, 467.520, 497.760, 527.520, 554.160, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 152.880/164.160 | 106.480/118.310 |
| 3 | 144.000, 219.360, 249.840, 290.640, 320.880, 351.120, 380.880, 410.640, 437.280, 527.520 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 164.640/175.920 | 118.310/130.640 |

### p14d10

Table cell side-margin parameter: 108 -> 216 twips on both sides (5.4 -> 10.8 pt per side).

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 83.040 | 83.040 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 216.480 | 193.920 | -22.560 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 262.800 | 230.880 | -31.920 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 327.840 | 283.680 | -44.160 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 373.920 | 320.160 | -53.760 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 419.760 | 356.880 | -62.880 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 464.880 | 392.880 | -72.000 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 510.000 | 428.640 | -81.360 | 1/2 |
| 1/9 | 10/4 | 835 | `to` | 549.360 | 459.840 | -89.520 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 83.040 | 83.040 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 196.800 | 230.880 | 34.080 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 273.600 | 320.160 | 46.560 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 417.840 | 459.840 | 42.000 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 454.320 | 496.320 | 42.000 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 490.800 | 533.040 | 42.240 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 527.280 | 568.800 | 41.520 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 558.720 | N/A | N/A | 1/0 |
| 3/1 | 1/2 | 2977 | `I` | 110.400 | 149.520 | 39.120 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 160.320 | 193.920 | 33.600 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 196.800 | 230.880 | 34.080 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 237.360 | 283.680 | 46.320 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 273.600 | 320.160 | 46.560 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 310.080 | 356.880 | 46.800 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 346.560 | 392.880 | 46.320 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 382.800 | 428.640 | 45.840 | 1/2 |
| 3/9 | 10/4 | 835 | `ye` | 417.840 | 459.840 | 42.000 | 1/1 |

Word heading: 18 payload lines; fragments: `El` / `ec` / `tr` / `op` / `hy` / `si` / `ol` / `og` / `ic` / `al` / `Ch` / `ar` / `ac` / `te` / `ri` / `za` / `ti` / `on`.

Ours heading: 3 payload lines; fragments: `Electrophysiolo` / `gicalCharacteri` / `zation`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 205.440, 251.760, 316.800, 362.880, 408.720, 453.840, 498.960, 538.320, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/287.760 | 72.000/117.810 |
| 2 | 72.000, 185.760, 262.560, 406.800, 443.280, 479.760, 516.240, 547.680, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 288.720/300.000 | 117.810/129.640 |
| 3 | 99.360, 149.280, 185.760, 226.320, 262.560, 299.280, 335.520, 371.760, 406.800, 516.240 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 300.480/312.000 | 129.640/153.300 |

### p14d11

Only LIVE row-1 wAfter: 869 -> 351 twips (saved grid columns 14+15); row-3 wAfter remains 869.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 9 payload lines; fragments: `Elec` / `trop` / `hysi` / `olog` / `ical` / `Char` / `acte` / `riza` / `tion`.

Ours heading: 3 payload lines; fragments: `Electrophysiologi` / `calCharacterizati` / `on`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/185.760 | 72.000/117.810 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 186.720/198.000 | 117.810/129.640 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 198.480/210.000 | 129.640/141.970 |

### p14d12

Only heading payload text: long word -> QZ; marker and payload paragraph formatting unchanged.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 1 payload lines; fragments: `QZ`.

Ours heading: 1 payload lines; fragments: `QZ`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/95.280 | 72.000/95.160 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 96.240/107.520 | 95.150/106.980 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 108.000/119.280 | 106.980/119.310 |

### p14d13

Only heading payload text becomes empty; retain its paragraph, run and formatting.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 213.600 | 188.400 | -25.200 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 259.680 | 225.360 | -34.320 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 324.960 | 278.160 | -46.800 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 370.560 | 314.880 | -55.680 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 416.160 | 351.360 | -64.800 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 460.800 | 387.360 | -73.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 505.680 | 423.360 | -82.320 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 544.560 | 454.320 | -90.240 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 231.360 | 225.360 | -6.000 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 304.560 | 314.880 | 10.320 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 431.520 | 454.320 | 22.800 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.160 | 491.040 | 26.880 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 496.800 | 527.520 | 30.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 528.960 | 563.520 | 34.560 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.280 | 575.280 | 18.000 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 132.960 | 144.240 | 11.280 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 198.720 | 188.400 | -10.320 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 231.360 | 225.360 | -6.000 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 272.160 | 278.160 | 6.000 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 304.560 | 314.880 | 10.320 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 337.200 | 351.360 | 14.160 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 369.360 | 387.360 | 18.000 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 401.760 | 423.360 | 21.600 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 431.520 | 454.320 | 22.800 | 1/1 |

Word heading: 0 payload lines; fragments: (empty).

Ours heading: 0 payload lines; fragments: (empty).

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 208.080, 253.920, 319.200, 364.800, 410.400, 455.280, 499.920, 538.800, 577.920 | 72.000, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 72.480/98.400 | 72.000/97.260 |
| 2 | 72.000, 225.840, 299.040, 426.000, 458.400, 491.040, 523.200, 551.760, 577.920 | 72.000, 219.810, 309.180, 448.720, 485.290, 521.930, 557.830, 569.530, 577.800 | 99.360/110.880 | 97.250/109.080 |
| 3 | 127.200, 192.960, 225.840, 266.400, 299.040, 331.680, 363.840, 396.000, 426.000, 523.200 | 138.530, 182.860, 219.810, 272.610, 309.180, 345.830, 381.720, 417.620, 448.720, 564.730 | 111.360/122.640 | 109.080/121.410 |

### p14d14

Old table-property snapshot: remove only tblLayout=fixed, retaining both R and C.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 132.960 | 123.360 | -9.600 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 159.120 | 153.600 | -5.520 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 187.440 | 190.320 | 2.880 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 213.600 | 220.320 | 6.720 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 239.520 | 250.560 | 11.040 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 265.200 | 280.320 | 15.120 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 291.120 | 310.080 | 18.960 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 159.120 | 153.600 | -5.520 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 213.600 | 220.320 | 6.720 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 392.160 | 360.720 | -31.440 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 468.240 | 383.520 | -84.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 543.120 | 406.320 | -136.800 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 563.760 | 566.640 | 2.880 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 98.400 | 105.120 | 6.720 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 132.960 | 123.360 | -9.600 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 159.120 | 153.600 | -5.520 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 187.440 | 190.320 | 2.880 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 213.600 | 220.320 | 6.720 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 239.520 | 250.560 | 11.040 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 265.200 | 280.320 | 15.120 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 291.120 | 310.080 | 18.960 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 316.320 | 337.920 | 21.600 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 72.000, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 72.480/95.040 | 72.000/95.160 |
| 2 | 72.000, 153.360, 207.840, 310.560, 386.400, 462.480, 537.360, 558.000, 577.920 | 72.000, 147.920, 214.730, 332.180, 354.980, 377.790, 400.590, 561.000, 577.800 | 95.520/107.040 | 95.150/106.980 |
| 3 | 92.880, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 99.430, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 107.520/118.800 | 106.980/119.310 |

### p14d15

HISTORY-FREE baseline: only heading payload text becomes empty; retain paragraph and all geometry.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 200.640 | 189.360 | -11.280 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 233.280 | 230.160 | -3.120 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 273.600 | 287.040 | 13.440 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 306.000 | 327.600 | 21.600 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 338.400 | 368.160 | 29.760 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 370.560 | 408.000 | 37.440 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 402.480 | 447.840 | 45.360 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 432.240 | 482.880 | 50.640 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 233.280 | 230.160 | -3.120 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 306.000 | 327.600 | 21.600 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 432.240 | 482.880 | 50.640 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 464.640 | 505.680 | 41.040 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 497.280 | 528.480 | 31.200 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 529.200 | 551.280 | 22.080 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 557.520 | 566.640 | 9.120 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 134.160 | 144.720 | 10.560 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 200.640 | 189.360 | -11.280 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 233.280 | 230.160 | -3.120 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 273.600 | 287.040 | 13.440 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 306.000 | 327.600 | 21.600 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 338.400 | 368.160 | 29.760 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 370.560 | 408.000 | 37.440 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 402.480 | 447.840 | 45.360 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 432.240 | 482.880 | 50.640 | 1/1 |

Word heading: 0 payload lines; fragments: (empty).

Ours heading: 0 payload lines; fragments: (empty).

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 194.880, 227.520, 267.840, 300.240, 332.880, 364.800, 396.960, 426.720, 523.680 | 72.000, 183.600, 224.490, 281.350, 321.860, 362.450, 402.290, 442.130, 477.140, 552.490 | 72.480/98.400 | 72.000/97.260 |
| 2 | 72.000, 227.520, 300.240, 426.720, 459.120, 491.520, 523.680, 552.000, 577.920 | 72.000, 224.490, 321.860, 477.140, 499.940, 522.740, 545.540, 561.000, 577.800 | 98.880/110.400 | 97.250/109.080 |
| 3 | 128.640, 194.880, 227.520, 267.840, 300.240, 332.880, 364.800, 396.960, 426.720, 523.680 | 138.970, 183.600, 224.490, 281.350, 321.860, 362.450, 402.290, 442.130, 477.140, 552.490 | 110.880/122.160 | 109.080/121.410 |

### p14d16

HISTORY-FREE baseline: only LIVE row-1 wAfter 869 -> 351; row-3 wAfter stays 869.

| Row/cell | Grid start/span | tcW (twips) | Marker | Word x | Ours x | Δx | Marker lines W/O |
|---|---|---:|---|---:|---:|---:|---|
| 1/1 | 0/3 | 2977 | `ox` | 77.760 | 77.760 | -0.000 | 1/1 |
| 1/2 | 3/1 | 992 | `up` | 132.960 | 123.360 | -9.600 | 1/1 |
| 1/3 | 4/1 | 1418 | `by` | 159.120 | 153.600 | -5.520 | 1/1 |
| 1/4 | 5/1 | 982 | `we` | 187.440 | 190.320 | 2.880 | 1/1 |
| 1/5 | 6/1 | 984 | `he` | 213.600 | 220.320 | 6.720 | 1/1 |
| 1/6 | 7/1 | 964 | `me` | 239.520 | 250.560 | 11.040 | 1/1 |
| 1/7 | 8/1 | 964 | `no` | 265.200 | 280.320 | 15.120 | 1/1 |
| 1/8 | 9/1 | 835 | `so` | 291.120 | 310.080 | 18.960 | 1/1 |
| 1/9 | 10/4 | 835 | `to` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/1 | 0/4 | 2977 | `go` | 77.760 | 77.760 | -0.000 | 1/1 |
| 2/2 | 4/2 | 992 | `do` | 159.120 | 153.600 | -5.520 | 1/1 |
| 2/3 | 6/4 | 1418 | `if` | 213.600 | 220.320 | 6.720 | 1/1 |
| 2/4 | 10/1 | 982 | `it` | 316.320 | 337.920 | 21.600 | 1/1 |
| 2/5 | 11/1 | 984 | `in` | 392.160 | 360.720 | -31.440 | 1/1 |
| 2/6 | 12/1 | 964 | `ax` | 468.240 | 383.520 | -84.720 | 1/1 |
| 2/7 | 13/2 | 964 | `a` | 543.120 | 406.320 | -136.800 | 1/1 |
| 2/8 | 15/1 | 835 | `O` | 563.760 | 566.640 | 2.880 | 1/1 |
| 3/1 | 1/2 | 2977 | `I` | 98.400 | 105.120 | 6.720 | 1/1 |
| 3/2 | 3/1 | 992 | `as` | 132.960 | 123.360 | -9.600 | 1/1 |
| 3/3 | 4/1 | 1418 | `at` | 159.120 | 153.600 | -5.520 | 1/1 |
| 3/4 | 5/1 | 982 | `be` | 187.440 | 190.320 | 2.880 | 1/1 |
| 3/5 | 6/1 | 984 | `of` | 213.600 | 220.320 | 6.720 | 1/1 |
| 3/6 | 7/1 | 964 | `am` | 239.520 | 250.560 | 11.040 | 1/1 |
| 3/7 | 8/1 | 964 | `is` | 265.200 | 280.320 | 15.120 | 1/1 |
| 3/8 | 9/1 | 835 | `my` | 291.120 | 310.080 | 18.960 | 1/1 |
| 3/9 | 10/4 | 835 | `ye` | 316.320 | 337.920 | 21.600 | 1/1 |

Word heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

Ours heading: 1 payload lines; fragments: `ElectrophysiologicalCharacterization`.

| Row | Word border x edges | Ours border x edges | Word top/bottom y | Ours top/bottom y |
|---:|---|---|---|---|
| 1 | 72.000, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 72.000, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 72.480/95.040 | 72.000/95.160 |
| 2 | 72.000, 153.360, 207.840, 310.560, 386.400, 462.480, 537.360, 558.000, 577.920 | 72.000, 147.920, 214.730, 332.180, 354.980, 377.790, 400.590, 561.000, 577.800 | 95.520/107.040 | 95.150/106.980 |
| 3 | 92.880, 127.440, 153.360, 181.920, 207.840, 233.760, 259.680, 285.600, 310.560, 537.360 | 99.430, 117.710, 147.920, 184.680, 214.730, 244.820, 274.600, 304.380, 332.180, 559.020 | 107.520/118.800 | 106.980/119.310 |
