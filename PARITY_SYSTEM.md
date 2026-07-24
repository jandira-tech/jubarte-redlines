<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Word-parity test system (parity ladder)

Built 2026-07-13. Turns "our redline vs Word's redline" into tiny, named,
ratcheted findings. Harness: `tools/parity_ladder.py` (stdlib Python).
Baseline: `tools/parity_baseline.tsv` (checked in — THE parity worklist).

## Corpus

`/Users/arthrod/temp/T/neurotic_docx_bench/corpus_sanity/word_based`
- `centralized_mapping.csv` joins each pair: `docx_source/{base,next}.docx`
  → Word's redline in `docx_redlines_word/` (column `redline_docx_word`,
  falling back to `redline_docx`).
- `docx_accepted_word/` = Word's accept-all of each redline (unused so far —
  future oracle: our `accept` of our redline should text-match it).
- 207 usable pairs as of 2026-07-13.

## Commands

```sh
python3 tools/parity_ladder.py sweep   # exit 1 on NEW findings vs baseline
python3 tools/parity_ladder.py bless   # rewrite baseline (after review!)
python3 tools/parity_ladder.py mine    # qname-histogram hypothesis miner
# --limit N / --only SUBSTR to scope; --bin to point at another binary
```

Our outputs land in `_scratch/parity_ladder/` (gitignored). Full sweep ≈ 3 min.

## The ladder (first failing level names the problem class)

| Level | Assertion | Failure means |
|---|---|---|
| L0 | our delText-stream == text(A), ins-stream == text(B), ws-insensitive | we corrupt content (bug regardless of Word) |
| GT gate | Word's redline passes the same L0 | Word automation stub → `gt-invalid`, pair excluded |
| L1 | coalesced (eq/ins/del, text) op-sequence == Word's | different edit script (LCS/coalescing) |
| L2 | revision-element counts by local name == Word's | same edits, different revision semantics |
| L3 | no element qname present in exactly one side | wrapper/element-choice divergence |

L1 failure short-circuits L2/L3 (they'd restate it). `mc:AlternateContent`
is resolved to its first `mc:Choice` in all text walks (Fallback duplicates).

## Signatures (Word-independent detectors on OUR output)

In `SIGNATURES` dict; each ~10 lines, returns detail strings. Current set:
`S-bare-wps-drawing` (strict01 repair-dialog defect), `S-instrtext-in-del`,
`S-rsid-leftover`, `S-empty-ins-del` (excludes legal ¶-mark markers in
`rPr`/`trPr`), `S-dup-docpr-id`. Add one whenever `mine` or an L2/L3 finding
reveals a systematic pattern — that converts a corpus observation into a
permanent named regression check.

## Ratchet semantics

Baseline rows: `pair_stem<TAB>finding_key<TAB>detail`. The ratchet compares
on (pair_stem, finding_key) only — details are volatile. `sweep` fails ONLY
on NEW keys; FIXED keys are printed so you re-bless and the baseline shrinks.
Progress = `git diff` of `parity_baseline.tsv` getting shorter.

## Baseline snapshot (2026-07-13, 261 findings / 207 pairs)

77 L3-histogram · 64 L2-inventory · 62 L1-opseq · 39 gt-invalid ·
8 S-rsid-leftover · 7 S-bare-wps-drawing · 2+2 L0.

Highest-severity leads already visible:
- **L0 (4 findings)**: both involve `alternate_content.docx` — our
  AlternateContent resolution changes reconstructed text length (374 vs 260).
- **`delInstrText:1v12`** (L2): Word tracks deleted field codes as
  `delInstrText` far more than we do — under-emission, mirror image of the
  fixed raw-`instrText` bug.
- **`pPrChange:0v1` / `pPrChange:3v1`** (L2): paragraph-property change
  tracking diverges in both directions.
- **only-ours=['hyperlink']** (L3): we retain `w:hyperlink` where Word's
  redline unwraps it (`unwrap_hyperlinks_to_styled_runs` gap?).

## Next steps (in order)

1. Triage the 4 L0 findings (content corruption trumps everything).
2. `mine` over the full corpus → add signatures for systematic gaps
   (bookmarkStart/End, lastRenderedPageBreak, noProof appear only-Word).
3. Micro-fixture distillation: for a finding, bisect body children of A/B
   (prior art: `crates/ooxmlsdk-redline/parity/_scratch/make_slice.py`),
   shrink while the finding's signature persists, freeze minimal pair into
   `tests/fixtures/micro/<finding>/` + a `#[test]` asserting the signature
   is absent. Signatures are the shrinking oracle — no Word needed per slice.
4. Wire `sweep` as an env-gated `#[ignore]` Rust test or CI step (local-only,
   corpus lives outside the crate).
5. Fix the 39 `gt-invalid` Word GTs by re-running the Word automation for
   those pairs, or drop them from the mapping.
