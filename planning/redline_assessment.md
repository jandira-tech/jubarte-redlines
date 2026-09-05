<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Redline benchmark assessment: are we fooling ourselves?

2026-09-05. Scope: the `script_redlines` benchmark in `/Users/arthrod/temp/T/neurotic_docx_bench`
(base -> next pairs, tool emits a redline DOCX, scored against Word's own compare
output). Everything below is read from files already in that repository and in this
checkout; nothing was re-run, and this document proposes no changes.

## 0. Verdict

Partly, yes, in six specific and measurable ways. The benchmark already contains the
instruments that say so; the headline table does not use them.

1. **The scale starts near 55, not 0.** A do-nothing candidate (the base document
   rendered unchanged) scores 55.6 mean over the 402 cached pairs (median 53.2, p10 42,
   p90 70, max 95; `results/null_baseline.json`). `score_v2.py` records ~68 historically.
   jubarte's published 84.47 is about 29 points of skill on a 45-point ladder; docxodus's
   80.24 about 25. The README ranks by the raw number.
2. **The oracle is LibreOffice's rendering of Word's markup, and LibreOffice is a third
   of the scale away from Word.** The same Word redline DOCX rendered by Word instead of
   LibreOffice scores 66.2 mean against the LibreOffice oracle (n = 196, 0 documents at
   or above 90, 33 under 50; `results/soffice_vs_word_redlines_randomized/summary.json`).
   The `sanity-word` row in the README (68.17, n = 230) is the same fact. A candidate
   whose markup Word would draw exactly like Word's own compare output is capped near
   66-68 unless LibreOffice also happens to draw it like Word's file; a 100 says
   LibreOffice agrees with itself, nothing about Word.
3. **Correctness is measured but not counted.** The functional invariant (accept-all
   must yield the next text, reject-all the base text, checked with the neutral
   `docx-revisions` machinery) on jubarte's published run: accept ok 352 / 377 (93%),
   reject ok **293 / 377 (78%)**. 84 documents whose reject-all does not restore the
   base sit inside the 84.47. docxodus: 367 / 299. Inside this checkout, Ring 1
   (`tools/parity_ladder.py`) blesses 9 pairs at L0 in `tools/parity_baseline.tsv`, and
   41 unblessed NEW rows (4 at L0) stand at HEAD. The M463 / M328d episode (commit
   `fix(comparer): stop the redline rewriting its own input`) is the proof of
   mechanism: two commits raised visual parity and broke reconstruction on 16 pairs;
   the visual bench never noticed, the ladder did.
4. **The bench's own alarm is ringing.** Lens health (`lens_health.py`; "docs where the
   pixel lens and a judging lens conflict: the bench is measuring the wrong thing on
   those docs") is at 41 documents, **10.9%**, on jubarte's published run (docxodus 4.5%).
   It is reported in `docs/RESULTS.md`, not next to the ranking.
5. **Test-set tuning with the detector switched off.** The comparer (39,421 lines in
   `src/comparer/*.rs`, `document_comparer.rs`, `revision_processor.rs`) carries 585
   lines citing 258 distinct milestone ids and 91 distinct corpus stems by name
   (`file_33` x26, `file_175` x15, `file_8` x14, `eigenpal` x14, `file_69` x13, ...). A
   sealed 40-pair holdout exists (`corpus/holdout_combined.txt`, `bench run --holdout`,
   "overfitting detector"); `docs/RESULTS.md` says *no holdout runs recorded yet* and
   `results/bench.jsonl` has zero lines with `holdout_mode = only`. Four of the 40
   sealed pairs are named in the comparer's code (`file_34`, `file_139`, `file_140`,
   `file_88`): the seal leaked by name.
6. **Two thirds of the pairs are not revisions.** Both pools chain adjacent documents
   (alphabetical order for the named pool, a random order for the randomized one), so
   most pairs join two unrelated documents. Over the 381 scored pairs the median share
   of words that base and next have in common is 5%, and 248 pairs (65%) share under
   20%. Only 45 pairs (12%) share half their words or more, the shape of an actual
   revision. A redline between unrelated documents is delete-everything /
   insert-everything; 50 of jubarte's 109 exact-100 pairs are of that kind, and the
   157 unrelated randomized pairs average 84.6. The randomized pool is *not* a
   duplicate of the named one: only 1 of 196 randomized pairs repeats a named
   base -> next combination. An earlier draft of this document said the pairs were
   duplicated; that was wrong (59 of the 67 "identical" scores were exact 100s
   matching other exact 100s). The document *pool* is shared (179 of 199 randomized
   sources are text-identical to a named source apart from an injected first line),
   which matters for the DOCX -> PDF tracks (398 = ~199 documents twice), not here.

None of this says jubarte is bad at redlines. docxodus, the strongest external tool,
shows the same instrument readings (25 skill points, 79% reject-ok, 4.5% lens
disagreement). It says the published number answers a narrower question than it
appears to: *how closely does LibreOffice draw our markup the way it draws Word's
markup, on a corpus we tuned against, two thirds of whose pairs are unrelated documents,
ignoring whether the markup accepts and rejects back to the right text.*

## 1. What the benchmark measures

| Stage | Implementation | Note |
|---|---|---|
| Pairs | `<base>_<next>` keys from `centralized_mapping.csv` (named + randomized) plus the SuperDoc pool; 763 scored in the published runs | randomized = renamed duplicates (point 6) |
| Candidate | tool writes a redline DOCX | |
| Render | LibreOffice 26.2.4.2 headless -> PDF -> 144 DPI rasters, page-paired | deterministic: `noise_floor.json` sigma 1e-14 over 6 re-renders |
| Oracle | Word's compare DOCX, rendered by the same LibreOffice | "the oracle DOCX through that pipeline scores 100" (README) |
| Score | superdoc fused score: ssim_full .25, ssim_small .15, ink_f1 .20, edge_iou .15, color_sim .15, blob_sim .10; ink-weighted page mean; ITT (failed converts = 0) | `score.py`, parity-locked to superdoc-visual-benchmarks |
| Published | README `script_redlines` table: jubarte-rust 84.47 / 92.66, exact-100 197, >= 90 419, < 50 29, 0 failures; jubarte (lossless) 81.99 / 91.31; docxodus 80.24 / 91.11 | columns: mean, median, ITT, exact_100, at_least_90, failures |

## 2. Instruments the bench already has

| Instrument | Where | Reading on jubarte's published run | Used in the ranking? |
|---|---|---|---|
| `null_score` (do-nothing baseline), `skill_score = (overall - null) / (100 - null)`, `score_v2` (ink-F1 inside the change-region mask only) | `score_v2.py`, `results/null_baseline.json` | null mean 55.6 over 402 pairs; per-document skill / v2 for the published run not on disk (the `results/detail` file for that run holds timings only) | no ("informational, `score.py` itself is never touched") |
| Functional accept / reject invariant | `cli._functional_stage`, `docx-revisions` | 352 / 293 of 377 | no (recorded on the run line only) |
| Lens health | `lens_health.py`, `results_schema.py` ("a bench-health signal, never a ranking input") | 41 docs, 10.9% | no (`docs/RESULTS.md` section) |
| Sealed holdout | `bench.yaml` `holdout_list`, `bench run --holdout` | never run | no |
| Pairwise sign tests | `docs/RESULTS.md` | jubarte-rust beats every other tool at p < 1e-100 | yes, but on duplicated pairs |
| Ring 1 / 2 / 3 (this checkout) | `tools/parity_ladder.py`, `tools/validate-docx`, `scripts/word-open-probe.sh` | L0: 9 blessed + 4 unblessed NEW; Ring 2: 60 keys; Ring 3: 207 / 207 | no (VERSIONING.md gate, separate from the bench) |
| LibreOffice-vs-Word render study | `results/soffice_vs_word_redlines_randomized/` | 66.2 mean, 0 at >= 90 | no |

The instruments are good. The gap is that the README table, the CHANGELOG entries and
the tuning loop all read the raw fused score.

## 3. Evidence detail

**3.1 Dynamic range.** `results/null_baseline.json`: 402 entries keyed by content hash,
mean 55.6, median 53.2, p10 42.4, p90 70.2, max 94.6. The docstring of `score_v2.py`:
"a do-nothing candidate (the base rendered unchanged) historically scored ~68 mean,
above half the leaderboard." Per-pair skill for jubarte cannot be computed from disk
today (see limits), but the aggregate arithmetic is not in doubt: 84.5 raw on a floor
of 55-68 is 30-45% of the way from nothing to perfect, not 84%.

**3.2 Renderer.** `summary.json` of the LibreOffice-vs-Word study: oracle
`pdf_redlines_randomized` (Word export), candidate
`except_this_pdf_soffice_redlines_randomized` (LibreOffice render of the same DOCX),
n = 196, mean 66.19, median 67.53, stdev 11.9, min 41.1, max 82.0, 6 page-count
mismatches, worst pairs at 41 (`file_154_file_155`, `file_195_file_196`, `file_8_file_9`).
The `sanity-word` vendor row (`bench.yaml` comment: "render the Word redline DOCX via
LibreOffice and score against the same redline as Word rendered it") reads 68.17 /
70.48 over 230 documents with 0 at or above 90. Both say the same thing: what
LibreOffice draws is not what Word draws, by a third of the scale, for Word's own files.

**3.3 Correctness.** Run line `019ffceb-a963-773f-8799-54e482e501d3` (jubarte-rust,
84.4662): `n_functional_checked 377, n_accept_ok 352, n_reject_ok 293, n_lens_disagree
41, lens_disagree_rate 0.1088, holdout_mode excluded`. docxodus 9.8.0 (`019ff85b-...`):
377 / 367 / 299 / 17 / 0.0451. In this checkout, `tools/parity_baseline.tsv` has 262
blessed rows: 78 L3-histogram, 65 L2-inventory, 63 L1-opseq, 32 gt-invalid, 8
S-rsid-leftover, 6 S-bare-wps-drawing, 5 L0-original, 5 L0-modified (9 distinct L0
pairs). The 2026-09-05 sweep adds 41 NEW rows (4 at L0) that were deliberately not
blessed. The M463 / M328d commit message documents the failure mode in the bench's own
terms: Ring 1 red with 58 NEW findings, 24 at L0, bisected to two commits that "traded
losslessness for a visual-parity score"; reverting them moved L0 rows 34 -> 17.

**3.4 Lens health.** `docs/RESULTS.md` "Lens health": docx-redline-js 0.5%, docxodus
9.0.0 25.4% then 9.8.0 4.5%, folio 28.8%, jubarte 11.7% and 11.9% on two builds; the
published jubarte-rust run 10.9%.

**3.5 Tuning.** `grep -c -E '\bM[0-9]{3}'` over the comparer sources: 585 lines, 258
distinct ids; distinct `file_NN` stems named: 91. Examples: `formatchg.rs:295 "M130
(file_165): Word keeps live spacing + pPrChange(empty old)"`, `formatchg.rs:424 "M102
(file_148): property addition of jc only"`, `footnotes.rs:318 "basic_list x
sd_1707_list_enter ... once M482 started"`. This is the same culture the converter
showed (195 "mini NNN" comments, `report.md` section 6). The holdout: 40 keys, union of
`corpus/word_based/holdout.txt` and `corpus/word_redlines_superdoc/holdout.txt`;
`bench.jsonl` holdout_mode values: `excluded` x32 script_redlines lines, `None` x10,
`only` x0; four holdout pairs' stems appear in comparer comments.

**3.6 Pair relatedness, and the retracted duplicate claim.** `centralized_mapping.csv`
has 207 pairs of origin `redline_only` chained over the alphabetical order of the named
sources; `centralized_mapping_randomized.csv` has 196 pairs of origin `randomized_chain`
over `file_1 -> file_2 -> ...`. Mapping `file_N` back to its named source by document
text (first paragraph dropped when it is the injected file name): 179 of 199 randomized
sources equal a named source, 16 named sources equal another named source, and
exactly 1 randomized pair is the same base -> next combination as a named pair. So the
redline pairs are not duplicated; the earlier "67 of 188 identical scores" was 59 exact
100s matching exact 100s plus 8 coincidences. What the mapping does show is how the
pairs were made. Word-level `difflib` ratio between base and next paragraphs, joined
with the blessed jubarte snapshot:

| pool | words shared | n | mean | median | exact 100 | under 50 |
|---|---|---|---|---|---|---|
| named | >= 50% (revision) | 41 | 85.4 | 92.7 | 17 | 1 |
| named | 20-50% | 61 | 89.1 | 98.4 | 28 | 0 |
| named | < 20% (unrelated) | 91 | 70.4 | 70.3 | 5 | 11 |
| randomized | >= 50% (revision) | 4 | 71.9 | 74.9 | 1 | 2 |
| randomized | 20-50% | 27 | 91.6 | 99.9 | 13 | 0 |
| randomized | < 20% (unrelated) | 157 | 84.6 | 89.5 | 45 | 1 |

Median word share across all 381 pairs: 0.05. Examples of unrelated pairs at 100:
`document_100_ultimate_demo -> double_spacing_bold_demo`, `heading_3_style_demo ->
heading_4_right_italic`. Examples of revisions under 50: `docx_lots_of_comments_addition_removal_redline -> ...` (38.7),
`file_8_file_9` (37.6), `file_175_file_176` (49.9). The benchmark is strongest exactly
where redlining is easiest and thinnest (45 pairs) where it is hard. The DOCX -> PDF
tracks, by contrast, really are duplicated: the 398 no-redline fixtures are ~199
documents twice (the randomized copy differs by one injected line), which `report.md`
now notes.


## 4. What this does not show

- Per-document `null_score`, `skill_score`, `score_v2` and functional verdicts for the
  published jubarte-rust run are not on disk (`results/detail/<run>` absent; the
  `jubarte` detail file that exists holds only timings per document). The 352 / 293
  figures are the run line's aggregates.
- Whether jubarte's *own* output renders in Word the way Word's compare output does is
  unmeasured. Ring 3 only tests open-or-refuse. The local-only `render/word.py`
  backend (AppleScript) exists, so it is measurable on this machine.
- The functional invariant compares extracted text; formatting-only revisions
  (`rPrChange`, `pPrChange`) and move markup are outside it. The L1-L3 ladder levels
  cover part of that gap, in this checkout only.
- Nothing here was re-run; a fresh run could move any individual number by the
  documented re-render noise (zero) plus tool changes since the run line.

## 5. Questions to settle before anything is changed

Not a plan; the decisions that the evidence puts on the table.

1. Which number is the headline: raw fused score, skill score, `score_v2`, or all three
   side by side?
2. Does the functional invariant become a gate (a document that fails reject-all cannot
   count as a pass), a column, or stay a run-line footnote?
3. Is the sealed holdout run and its gap published, and is a new seal drawn from pairs
   never named in the comparer's code?
4. Are unrelated-document pairs (65%) reported separately from revision pairs (12%),
   or is the corpus rebuilt from real version pairs?
5. Is a Word-rendered track added (oracle `pdf_redlines_word` exists; candidates need
   the local Word renderer), so that LibreOffice-vs-Word stops being invisible?
6. Are lens-disagreement documents excluded from the mean, listed beside it, or left as
   an alarm nobody reads?
7. Does VERSIONING.md's publish gate (Ring 1 at 0 NEW) get enforced, given 0.8.0 and
   0.9.0 both shipped past a red Ring 1 with 41 NEW rows?
