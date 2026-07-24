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

## Notes

- **Hyperlink `r:id` preservation**: unwrap only anchor-based internal
  hyperlinks (TOC shape); external `r:id` preserved.
- **M112**: clean adjacent tables not merged through accept/reject pipeline.
