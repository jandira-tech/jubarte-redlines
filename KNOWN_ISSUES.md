<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Known issues

Engine defects and unresolved design conflicts. Tests covering them are marked
`#[ignore = "KNOWN ISSUE <n> …"]` — run them with `cargo test -- --ignored`.

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

**Status:** open. 17 L0 rows remain corpus-wide; 15 of them predate `042089c`
(2026-07-24), so losslessness has been partially broken for some time. These
two are the only L0 regressions still outstanding against that build.

## 4. Internal `Unid` scratch ships as an undeclared `w:Unid` attribute

**Symptom (Ring 2, `Sch_UndeclaredAttribute`):** 81 findings of
`The 'http://…/wordprocessingml/2006/main:Unid' attribute is not declared.`

**Evidence it is ours, not inherited:** `Unid` appears in **0 of 192** corpus
source documents and in **12 of 207** of our outputs.

**Mechanism:** `unid.rs` stamps `PT::unid()` (`http://powertools.codeplex.com/2011`)
and `document_comparer.rs` strips `pt:*` scratch before writing. But at least one
attribute reaches the serializer bound to the **`w:` prefix** instead of `pt14:`,
so it (a) escapes the `pt:*` stripper, which looks for the PT namespace, and
(b) serializes as `w:Unid`, which is not in the wordprocessingml schema:

```xml
<w:spacing w:line="276" w:Unid="00000000000000000000000000000004" />
```

The root does declare `xmlns:pt14="http://powertools.codeplex.com/2011"`, and
these documents contain exactly one `w:Unid` and zero `pt14:Unid`, so this is a
single mis-namespaced stamp rather than a general serializer fault. Find the
write site that builds the name in the element's own namespace instead of PT.

**Status:** open. Pre-existing — present well before 0.8.0. Word's tolerance
varies: most of these files still open, so it is a validity defect rather than a
guaranteed corruption.

**Ring 2 note:** `tools/validity_baseline.tsv` was empty ("initial bless is
empty") — Ring 2 had never been run. The first full sweep over the 207-pair
corpus reports 1294 findings across 54 pairs. The dominant class (495) is
`r`/`g`/`b="0%"` colour attributes, which **are** inherited: 4 source documents
carry them. `tools/validate-docx/` is also empty in this checkout; the C#
project lives in the `wt-r2` / `wt-styles` worktrees.

## Notes

- **Hyperlink `r:id` preservation**: unwrap only anchor-based internal
  hyperlinks (TOC shape); external `r:id` preserved.
- **M112**: clean adjacent tables not merged through accept/reject pipeline.
