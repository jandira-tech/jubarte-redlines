# Known issues

Engine defects and unresolved design conflicts, diagnosed 2026-07-12 during
the extraction from `ooxmlsdk-redline`. Tests covering them are marked
`#[ignore = "KNOWN ISSUE <n> …"]` — run them with `cargo test -- --ignored`.

## 1. MovedSource runs emit `w:t` instead of `w:delText` under `w:moveFrom`

`mark_content_transform`'s MovedSource arm (src/comparer/finalize.rs) keeps
`w:t` inside `w:moveFrom` runs. Spec-conformant Word markup uses
`w:delText` there (moveFrom is a deletion side), and two downstream passes
already assume delText-under-moveFrom is legal
(`wrap_bare_del_text_runs` whitelists `w:moveFrom` as a delText parent;
`convert_run_text_to_del_text` treats moveFrom as its own delText-owning
container). The `w:t` emission is a workaround for a Word
"unreadable content" repair dialog whose downstream trigger has since been
independently fixed — the codebase is internally inconsistent.

**Fix**: call `convert_run_text_to_del_text` in the MovedSource arm — a
one-liner — **but** reverting this workaround is exactly what previously
tripped Word's repair dialog, so it must be re-validated by opening a
move-heavy redline in real Microsoft Word before shipping.

**Ignored test**: `m4f_finalize.rs::m4_f2_del_text_kind`.

## 2. Multi-del boundary fold has no relatedness gate (M90 vs M-PI conflict)

In `merge_replaced_in_container` (src/comparer/finalize.rs, the `!acted`
boundary-fold branch), when a replacement gap has **one** deleted paragraph
the fold is gated on `should_fold_ins_del_pair` (Jaccard relatedness). With
**two or more** deleted paragraphs the last-ins/first-del fold is
unconditional (M90 doctrine, validated on Word oracles file_38/62/11/191 —
stamped-demo shapes), with only narrow carve-outs (digits-only, very-short,
demo-title-after-long-run). Consequence: a wholly unrelated inserted
paragraph is folded into the first deleted paragraph of the gap — unrelated
whole-document replacements produce a mixed first paragraph instead of
clean ins/del separation.

This is a genuine oracle conflict, not a plain bug: M90 ("always fold the
boundary pair") and M-PI ("unrelated multi-paragraph replacements stay
separate") are contradictory Word-behavior claims living in the same
function, each backed by its own forensics. A Jaccard-gated fold (M68) was
already tried once and reverted for losing visual score. Any fix must be
re-validated against the Word-visual benchmark corpus.

**Ignored tests**:
- `m32_word_alignment.rs::w2_replaced_paragraphs_merge_pairwise`
- `m32_word_alignment.rs::w20b_gap_partition_del_clusters_before_anchor`
- `m32_word_alignment.rs::w23c_repeated_paragraph_real_word_never_bridges`
- `m42c_eigenpal_pkg.rs::eigenpal_batch_starts_with_ins_and_has_mixed_table`

## Notes

- **Hyperlink `r:id` preservation (fixed here, validation pending)**:
  `unwrap_hyperlinks_to_styled_runs` used to drop every `w:hyperlink`
  wrapper — including its `r:id` — before relationship reconciliation,
  silently destroying external link targets in the default (Word-visual)
  mode. It now unwraps only anchor-based internal hyperlinks (the TOC
  shape it was built for). The full test suite passes; the change's effect
  on the Word-visual benchmark score for hyperlink-heavy documents has not
  been re-measured.
- **M112 makes the in-pipeline table merge unreachable**: the A.10
  accept/reject pipeline consumes every revision mark *before* its
  `merge_adjacent_tables_transform` step, and M112 gates the merge on
  marks being present — so through `accept_revisions_document` /
  `reject_revisions_document`, adjacent tables are now never merged. If
  that shape is intended (Word Compare does not merge clean tables), the
  in-pipeline call is dead code that could be dropped; if not, the gate
  needs to capture the pre-acceptance marked state.
- **Removed tests**: `m119_file175_correlated_live.rs` (debug probe, no
  assertions, read files outside the repo) and `m54b_real_file22_merge.rs`
  (read a hardcoded temp-dir fixture that no longer exists anywhere).
