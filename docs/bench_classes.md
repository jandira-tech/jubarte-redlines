<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Bench defect classes (Ratchet-1 ledger)

**Baseline pin:** `jubarte-rust@9fcc4289e375` (2026-07-16)
**Corpora:** word_based `n=164` (mean 90.04, `<50`=8) · randomized `n=196` (mean 83.19, `<50`=16)
**Aggregate:** 360 docs · mean **86.31** · `<50`=**24**

Tool: `tools/bench_classes.py` — every fixture with `overall_score < 90` is assigned to **exactly one** class below (first matching rule wins).

## Classification rules

| Class | Rule |
|---|---|
| **C1-page-structure** | `page_count_oracle != page_count_candidate` |
| **C2-comments** | stem contains `comments` |
| **C3-tolerated-input** | stem contains `word_tolerated` or `repaired` |
| **C4-preexisting-revisions** | stem contains `suggesting` |
| **C5-formatting** | style/demo tokens (`bold`, `italic`, `heading`, `aligned`, …) |
| **C5-content-diff** | residual same-page content/LCS residue (was C5-triage) |

---

## C1 — page-structure (short↔long / pagination drift)

**Hypothesis:** Unrelated whole-document replacements hit the multi-del boundary fold (KNOWN ISSUE #2 / M90) and collapse ins+del into a mixed first paragraph, changing block structure and page count. Also covers Strict01 / repaired-media pairs that alter pagination via drawing/media defects.

**Members (baseline pin):**

| corpus | score | pages | stem |
|---|---:|---|---|
| word_based | 37.72 | 13/11 | `verdana_…_word_clean_strict01` |
| word_based | 39.51 | 12/9 | `word_clean_strict01_…broken_media_rel` |
| word_based | 67.98 | 6/3 | `sample_document_really_repaired…` |
| word_based | 76.01 | 7/3 | `sample_document_word_repair_of_our_output_iter2…` |
| word_based | 78.17 | 7/3 | `sample_document_word_repair_of_our_output…` |
| randomized | 37.21 | 13/11 | `file_99_file_100` |
| randomized | 37.22 | 13/11 | `file_114_file_115` |
| randomized | 37.24 | 13/11 | `file_195_file_196` |
| randomized | 37.33 | 13/11 | `file_184_file_185` |
| randomized | 37.86 | 13/11 | `file_100_file_101` |
| randomized | 40.88 | 14/10 | `file_196_file_197` |
| randomized | 41.54 | 13/11 | `file_115_file_116` |
| randomized | 43.82 | 13/10 | `file_185_file_186` |
| randomized | 44.99 | 116/115 | `file_22_file_23` |
| randomized | 73.93 | 3/4 | `file_91_file_92` |
| randomized | 74.26 | 3/4 | `file_64_file_65` |

**Status:** fix landed (document-scale relatedness gate + absolute gap floor).
**Mechanism test:** `tests/m146_wholedoc_replacement_no_fold.rs`
**Code:** `should_fold_multi_del_at_document_scale` in `src/comparer/finalize.rs`
**Pin `28c41564723b` deltas (pre-floor):** word_based mean 90.15 (+0.11), randomized 84.01 (+0.82), aggregate 86.81 (+0.50), `<50` 15 (−9). Page-mismatch C1 members largely recovered (11→4 mismatches).
**Regressions noted then fixed:** file_54 (−32) / bullet_list (−25) from under-fold on short sparse multi-para — absolute gap floor (≥40 word atoms) before skip.

---

## C2 — comments-heavy pairs

**Hypothesis:** Union-carry of `word/comments.xml` + anchors is incomplete; orphan anchors or dropped comments shift layout / score. Forensics on pin 28c: anchor counts match defs (no orphans); residual is layout / pPrChange gap vs Word, not missing comments.xml.

**Members (word_based):** 46.10–74.86 — five `docx_lots_of_comments_*` / `document_100…lots_of_comments` fixtures.

**Status:** mechanism tests green (`tests/m147_comments_union_carryover.rs`); carryover contract pinned. Residual visual score is layout/chrome — not silent drop of A∪B comments.
**Fix commits / deltas:** synthetic m147; corpus residual open (same-page pixel score).

---

## C3 — tolerated-malformed inputs

**Hypothesis:** Word normalizes broken media rels, misplaced pgSz/uiPriority/link, orphan comments; we either propagate breakage into the redline or normalize differently (also Ring-1 validity relevant). Package chrome gap: when A lacks `settings`/`fontTable`/`theme` and B has them, Word redlines carry B's chrome.

**Members (word_based):** 46.98–89.23 — eight `word_tolerated_*` / `*_repaired_*` fixtures (including one `suggesting` stem that also contains `repaired` and classifies here first only if C3 precedes C4 — see tool order: C3 checks before C4; actual members are pure tolerated stems except any dual-token stems).

**Status:** chrome adopt landed — `adopt_revised_styles_chrome` now copies missing settings/webSettings/fontTable from B (theme already). Plus **styleId canonicalization** (`heading 1`→`Heading1`, `style20`→`PreformattedText`) and adopt missing `docDefaults`/`latentStyles` from B — LO layout keys on canonical ids. Tests: `tests/m148_tolerated_inputs.rs` (`canonicalizes_numeric_style_ids_to_word_names`). Broken-media dangling Target stripped (reconcile).
**Fix commits / deltas:** re-score pending with style-canon pin `5b914dd3ed85`.

---

## C4 — pre-existing tracked changes (`suggesting_*`)

**Hypothesis:** Architectural — Word keeps input `w:ins`/`w:del` as history; we accept-before-diff. Changing this alters `accept(redline)` reconstruction.

**Members (word_based):** 47.37–89.80 — eight `suggesting_*` fixtures.

**Status:** **blocked on Arthur** — decision memo only (Task B4). Do **not** implement accept-first changes without a recorded option pick.
**Memo:** `docs/C4_preexisting_revisions_decision.md`

**Decision memo (options, forensics-only estimates):**

| Option | Behaviour | Est. aggregate lift | Risk |
|---|---|---:|---|
| **A — Keep accept-first** | Document gap; leave scores | 0 | None (status quo) |
| **B — Carry A-side pending dels as history** (w14/w15-style) | Preserve A-side TC that B does not share | +0.3–0.8 (uncertain) | Accept() may surface history Word hid |
| **C — Full merge semantics** | Word-Compare-like fold of pre-existing TC | +1.0–2.0 (uncertain) | Large surface; golden churn |

**Arthur choice:** _(none yet)_

---

## C5-formatting — one-pager style/demo pairs

**Hypothesis:** Residue in `pPrChange` / `rPrChange` emission (M22x–M23x series); same page count, pure formatting deltas. Also package chrome (theme/settings) when both inputs are thin demos.

**Members (word_based):** 27 fixtures, scores 57.35–89.99 (e.g. `quarterly_performance…red_bold_heading…`, `blue_bold_centered…`, `right_aligned_italic…`).

**Status:** mechanism test green (`tests/m149_formatting_rpr_change.rs` — bold-only emits format revision). Chrome adopt helps when B has settings/theme. Class median still open vs 85 stop rule.
**Fix commits / deltas:** m149 + C3 chrome path; residual open.


---

## C5-content-diff — residual same-page content (randomized-heavy)

**Hypothesis:** Mixed LCS/content correlation residue on same-page pairs that are not comments/styles/suggesting/tolerated. Not a single mechanism — triage bucket refined as forensics land. Includes randomized `file_*` same-page low scorers and word_based `hr_onboarding_checklist…` (48.77).

**Members:** ~78 randomized same-page `<90` + 1 word_based residual (see `tools/bench_classes.py` output).

**Status:** subclass landed — **unrelated table cell-merge**: zero body-word Jaccard table pairs pure-del A + pure-ins B (H2 table/table + `do_lcs_algorithm_for_table`). Mechanism test `tests/m150_unrelated_tables_no_cell_merge.rs`. Fixes hr_onboarding checklist folded into B's Positioning/Prepared-for tables.

**Fix commits / deltas:** re-score pending (pin `8d3b47b97af7`).

---

## Membership completeness

Gate: every `<90` fixture on either corpus appears in **exactly one** class.

```bash
python3 tools/bench_classes.py /Users/arthrod/temp/T/neurotic_docx_bench/results/bench.jsonl
```

Regenerate the raw dump into `_scratch/` when re-pinning; keep this ledger as the source of class → hypothesis → fix → delta.

## Landed fixes log

| Class | Commit | word_based Δ mean | randomized Δ mean | aggregate Δ | notes |
|---|---|---:|---:|---:|---|
| C1 | multi-del doc-scale gate (+ gap floor) | +0.11 (pin 28c) | +0.82 (pin 28c) | +0.50 / `<50`−9 | m146; page-mismatch 11→4; floor fixes file_54/bullet under-fold |
| C2 | comments union contract tests | — | — | — | m147 synthetic; residual layout |
| C3 | adopt B settings/fontTable/theme when missing | +word demos (chrome pin) | — | partial | m148; `adopt_revised_styles_chrome` + factory chrome |
| C3 | styleId canonicalize + docDefaults/latentStyles from B | +0.07 (pin 653876; misplaced_pgsz +10.7) | ~0 | +0.03 | m148; `canonicalize_style_ids` / `word_canonical_style_id` |
| C5 | rPrChange bold-only synthetic + factory chrome | word mean → 92.10 | — | partial | m149 + factory settings/theme/fontTable/webSettings |
| C5 | asymmetric multi-table zero-Jaccard pure I/D | eigenpal +5.7; hr structural (score ~flat) | file_8 +6.8 | +0.05 | m150; H2 only when 1 vs ≥3 tables |
| C3/C5 | incomplete spacing normalize (`lineRule=auto` sans `line`) | _(re-score)_ | _(re-score)_ | _(re-score)_ | m151; list→line=240; non-list strip |
| C4 | deferred | 0 | 0 | 0 | memo only — Arthur pick required |

**Latest pin `e12c880586ec` (script_redlines):** word mean **92.20** / med 99.92 / lt50=3 · rand mean **84.21** / med 93.09 / lt50=9 · **aggregate mean 87.85** / lt50=12. Ratchet-1 mean still short of 88 by ~0.15 (~54 points).

## Anti-overfit protocol (reminder)

1. Name by mechanism, never by fixture.
2. Forensics on ≥2 members + 1 control (≥90).
3. Synthetic red→green unit test first.
4. Both-corpora gate; Ratchet-1 on aggregate.
5. Ledger entry; a fix that only moves its own fixtures is suspect.
