<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts. Tests covering them are marked
`#[ignore = "KNOWN ISSUE <n> …"]` — run them with `cargo test -- --ignored`.

> **Re-checked 2026-09-26 (0.9.2):** all open items below are on the
> compare/validity path. The 0.9.2 release was confined to `src/convert`,
> the CLI and the OPC layer — the statuses here are unchanged.
>
> **Re-checked 2026-09-27 (0.9.3):** item 3's named pairs and the six
> later text round-trip failures now reconstruct. Empty paragraph-property
> shells are left out. A sweep of the 207-pair word-based corpus reports
> 0 NEW on every rung, with `tools/parity_baseline.tsv` left as it was.
> Spellcheck marks, the pagination cache, and header or footer references
> Word writes on its own are outside the histogram. Items 1, 2, and 5
> stay as recorded below. Item 4 is still the Ring 2 ratchet.
>
> **Re-checked 2026-09-28 (release/0.10):** item 4 is closed (fixed by
> b7fedc78; the Ring 2 baseline is re-blessed without it).

## 1. MovedSource / `w:moveFrom` text kind — **SETTLED 2026-07-16 (Word wins)**

**Contract (hard test):** under `w:moveFrom`, emit **`w:t`** (never `w:delText`).
Under `w:del`, emit **`w:delText`** (never `w:t`).

Ring-3 probe evidence (macOS Microsoft Word):

- `parity/_scratch/ki1/move_deltext.docx` (delText under moveFrom) →
  **`FAILED: no document opened (likely corrupt dialog)`**
- Session log: implementer scratch `ki1_word_probe.log`

Spec/PowerTools prefer delText under moveFrom, but Word rejects it. Word
parity is the prime directive — keep `w:t` under moveFrom.

Hard tests:

- `m4f_finalize.rs::m4_f2_del_text_kind` (un-ignored; asserts delText under del + `w:t` under moveFrom)
- Ring 1: `check_del_text_under_del` fails on delText-under-moveFrom and on `w:t` under del
- Probes: `m_validity_ring1.rs::probe_deltext_under_movefrom_fails`,
  `probe_wt_under_movefrom_passes_ring1`

## 2. Multi-del boundary fold — document-scale relatedness (narrowed)

**Status (2026-07-16):** Partially resolved (class C1). Multi-del boundary fold
in `merge_replaced_in_container` is gated by
`should_fold_multi_del_at_document_scale`:

- related gap (any I×D pair with Jaccard) → still fold (M90 / M131);
- large multi-paragraph asymmetric gaps (size ratio ≥ 4, gap > 60% of container
  word atoms) with zero Jaccard → **no fold** (unrelated whole-document
  short↔long replacement).

Synthetic coverage: `tests/m146_wholedoc_replacement_no_fold.rs`.

**Still ignored (re-check after Ratchet-1 re-score)**:

- `m32_word_alignment.rs::w2_replaced_paragraphs_merge_pairwise`
- `m32_word_alignment.rs::w20b_gap_partition_del_clusters_before_anchor`
- `m32_word_alignment.rs::w23c_repeated_paragraph_real_word_never_bridges`
- `m42c_eigenpal_pkg.rs::eigenpal_batch_starts_with_ins_and_has_mixed_table`

## 3. Free-mesh double-consumption — one A-side atom claimed by two paragraphs

**Symptom (Ring 1, L0-original):** the del-stream no longer reconstructs A —
a word appears twice where A has it once. `parity_ladder.py sweep` reports
`recon len 114 vs src 110` with `Thistexttext` for
`right_aligned_italic_demo_… × right_alignment_demo_…_2`, and `ThistextThistext`
(`106 vs 98`) for `center_aligned_bold_text_… × center_alignment_demo_…_2`.

**Mechanism:** the free-mesh splits A's paragraph across two output paragraphs
and lets both claim the same source atom as EQ. For the right-align pair, A's
`This text is right-aligned and italic.` yields

```
P1: EQ"This " INS"document demonstrates right " EQ"text " INS"alignment."
P2: INS"All" EQ" text " INS"in this document " EQ"is " DEL"right-" …
```

`text` is EQ in both P1 and P2, so the original stream counts it twice. A
correct mesh must consume each A atom exactly once across the whole body.

**Not** the M463 fold or the M328d case-fold — both of those are fixed (see
CHANGELOG 0.8.0); this survives them and needs its own bisect over the range
after `c2547f4a`.

**Status:** the double-consumption on the two pairs above is fixed, and so
are the six text round-trips found on the 0.9.3 candidate (re-checked
2026-09-27, `tools/parity_ladder.py sweep`, 0 `L0` keys). A folded Demo
title kept `Demo` inside the deletion instead of growing a live " Demo".
A sentence period that one side continues into the next paragraph is given
to the side that already finished and appended to the continuation. The
comments redline puts the deleted section back before the table that
repeats its heading, which is the original's order. Eleven empty
paragraph-property shells are gone (blue centered title × blue italic,
center bold × clear formatting, and nine of the same shape). The 21 keys
on the rungs after a matching operation sequence are gone too (sweep of
207 pairs on 2026-09-27, 0 NEW). That set included the live pilcrow on
italic-and-underline × italic-subscript, revision-count mismatches, and
element names Word writes on its own (`proofErr`, `lastRenderedPageBreak`,
`footerReference`, `headerReference`). The ladder leaves those four names
out of the histogram. `tools/parity_baseline.tsv` is unchanged.

## 4. Internal `Unid` scratch shipped as an undeclared `w:Unid` attribute — **FIXED (b7fedc78), closed 2026-09-28**

**Symptom (Ring 2, `Sch_UndeclaredAttribute`):** `w:Unid` on
`<w:spacing w:line="276">` in `word/document.xml`, in 12 of 207 corpus outputs
(0 of 199 sources). Word opened the files, so it was a validity defect, not
corruption.

**Cause:** `restore_deleted_paragraph_spacing` copied every attribute of a
deleted paragraph's spacing back onto the output, including the comparer's
scratch `Unid`, re-bound to the element's own `w:` namespace, where the
`pt:*` stripper never looked. It now copies only `w:` attributes; test
`document_comparer::restored_deleted_spacing_leaves_scratch_unids_behind`.

**Verified 2026-09-28:** the Ring 2 sweep over the 207-pair corpus reports
no `Sch_UndeclaredAttribute` at all (0 NEW, 40 FIXED keys; baseline
re-blessed to 20 keys), and a scan of 803 bench pairs found no `Unid`
outside the PowerTools namespace.

## 5. Ring 3: five corpus redlines Word refused to open — **FIXED 2026-09-05**

**Cause: `w:instrText` under `w:del`.** Word wants `w:delInstrText` there, the
same way it wants `w:delText` instead of `w:t`, and offers to repair the file
when it does not get it. Every one of the five was a document where deleting
content swallowed a field — a wholly deleted header or footer carrying
`PAGE`/`NUMPAGES` is the usual shape — on a path that wraps existing runs in
`w:del` rather than rebuilding them through `convert_run_text_to_del_text`,
which has always done the rename correctly.

Only 5 of the 207 corpus outputs carried the shape, and they were exactly the 5
Word rejected. `finalize::enforce_deleted_text_kinds` now enforces the invariant
after the pipeline and again in the package-level validity sweep, so
headers/footers — which never reach the body finalize path — are covered too.
`w:moveFrom` is deliberately excluded: Word Compare keeps plain `w:t` there.

**Ring 2 was clean on all five.** This is the important part. `w:instrText` is
schema-valid inside a run no matter what the run's parent is, so the
OpenXmlValidator reports nothing; the del/delInstrText correspondence is a
semantic rule Word applies at load. A green Ring 2 does not imply Word will open
the file, and Ring 3 is not redundant with it.

**Two adjacent defects found in the same triage, also fixed:**

- **`<w:del>` wrapping `<w:hyperlink>`** (2 of the 5). `w:hyperlink` is not in
  CT_RunTrackChange's content model. Word's shape is the inverse — the hyperlink
  stays put and the revision moves inside it — so
  `finalize::hoist_hyperlinks_out_of_revisions` splits the revision around each
  hyperlink, preserving order and minting fresh `w:id`s.
- **Bare `<w:szCs/>` / `<w:sz/>`** in the merged Normal style. The style-merge
  loop created the element and then passed `None` as the value, which strips the
  attribute but leaves the element; `w:val` is required on CT_HpsMeasure, and
  ECMA-376 spells "no value" as the element's absence.

**On "inherited" corruption.** Three of the five carried invalidity that came
from their own sources — `paragraphProperties="[object Object]"` on `w:style`,
`w:highlight` under `w:lvl/w:rPr`, `w:shd` with no `w:val`. It is worth being
precise about what that did and did not explain: **all six source documents open
cleanly in Word.** Word tolerates their invalidity and rejected our output, so
"inherited" was never the reason these failed — the earlier note here that said
otherwise was wrong. `finalize::repair_inherited_invalidity` now repairs those
three classes anyway (deletion-or-schema-default only, so it cannot change a
document that was already valid), because shipping a source's corruption inside
our redline gets it blamed on us.

**Harness note.** `scripts/redline-sweep.sh --probe` cannot measure Ring 3
correctly as written. A failing document leaves Word on a modal dialog, so every
later probe fails too — the first attempt read 34 opens then 88 phantom
failures. Recovering with `pkill -9` relaunches Word into Document Recovery,
another modal dialog; `*.docx` also matches the `~$name.docx` owner files Word
drops, each costing a 60s timeout; and after a kill Word needs ~30s to cold
start, which the probe's own 60s budget has to absorb, cascading into all-fail.
`scripts/word-probe-sweep.sh` handles all four: quit cleanly on failure, clear
`…/Preferences/AutoRecovery`, skip `~$*`, and pre-warm Word before the next probe.

**Corpus-freshness note.** The 202/5 figure was measured against a `_scratch/sweep`
directory generated on 12 July, two months stale — `tblGridChange` carrying
`w:author`/`w:date` showed up in that triage although commit 236c5ed had already
fixed it. Regenerate the sweep before trusting a Ring 3 number.

## Notes

- **Hyperlink `r:id` preservation**: unwrap only anchor-based internal
  hyperlinks (TOC shape); external `r:id` preserved.
- **M112**: clean adjacent tables not merged through accept/reject pipeline.
