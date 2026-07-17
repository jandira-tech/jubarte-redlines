<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Benchmark stamp — HEAD `d094de0aed36` (M233)

Generated 2026-07-16 06:01 UTC. Binary content-hash pin: **jubarte-rust@9fcc4289e375**.

Ship bar (primary ledger): **script_redlines mean ≥ 90 and median ≥ 90**.

## 1. Word-visual quality — main corpus (`bench.yaml`)

Tool: neurotic_docx_bench · oracle = Word redlines / LibreOffice render.

| benchmark | n | mean | median | exact 100 | ≥90 | min | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| **script_redlines** | 164 | **90.04** | **95.67** | 62 | 110 | 37.72 | 100.00 |
| accepted_changes | 164 | 87.00 | 95.39 | 69 | 95 | 38.26 | 100.00 |

Generate timings (main run, ms per redline from bench timings): mean_speed=26.03 · median_speed=5.98.

**Ship bar: PASS** (script_redlines mean 90.04 ≥ 90, median 95.67 ≥ 90).

Visual gallery: `neurotic_docx_bench/runs/jubarte-rust_2026-07-16_01-49/report.html`

> **RESULTS.md ranking note:** `export-results-md.py` keeps one row per
> `(vendor, benchmark, tool_version)` and prefers **higher `n_docs`**. The same
> pin also has the randomized n=196 row (mean 83.19), which can outrank the
> main n=164 ship-bar row in the printed table. Always cite **this stamp** (or
> `results/bench.jsonl` lines with `n_docs=164`) for the ≥90 claim.

## 2. Expanded sample — randomized `file_i_v_file_{i+1}` (`bench.randomized.yaml`)

Consecutive randomized chain pairs (`file_1_file_2` …). Oracle:
`corpus/word_based/pdf_redlines_randomized/pdf/`.

| benchmark | n | mean | median | min | max |
|---|---:|---:|---:|---:|---:|
| script_redlines | 196 | **83.19** | **93.09** | 37.22 | 100.00 |

Exact 100: 69 · ≥90: 107 · below 50: 16

Note: 11 pairs had page-count mismatch vs oracle (shared pages only scored) — short-into-long / multi-page structure residual, not scorer hacks.

### Combined sample (main + randomized, unweighted)

| | n | mean | median |
|---|---:|---:|---:|
| main + file1_v_file2 | 360 | **86.31** | **94.06** |

## 3. Speed — large-N redline bench (`scripts/redline_speed_bench.ts`)

1000 unique fixtures (includes `docx_source_randomized` file_N) · 5000 pairs · seed 42 · warmup 50.

| tool | median ms | mean ms | p95 | p99 | /s | wall s | fail | n |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| jubarte-rust-inproc | 16.953 | 56.001 | 224.686 | 488.964 | 17.9 | 280.02 | 0 | 5000 |
| jubarte-rust (CLI) | 21.86 | 52.116 | 197.381 | 440.98 | 19.2 | 260.59 | 0 | 5000 |

Full report: `neurotic_docx_bench/results/redline_speed_bench/m233_head_d094de0/report.md`

Fairness: **inproc** is the algorithm race (warm process). CLI includes spawn+I/O.

## 4. Criterion microbench (`cargo bench --bench redline`)

Baseline saved as `m233_head`.

| case | low ms | mean ms | high ms |
|---|---:|---:|---:|
| canonical_dense_edits | 160.75 | 173.29 | 191.32 |
| short_into_long | 232.37 | 240.79 | 250.11 |
| tables_bookmark_vmerge | 69.48 | 74.72 | 81.10 |
| comment_heavy | 198.30 | 223.19 | 247.09 |

## 5. Expanded ABBA wall matrix (absolute, HEAD self)

Permanent 4 fixtures + sample `file_1_v_file_2`, `file_50_v_file_51`, `file_100_v_file_101`, `file_130_v_file_131`.
1× ABBA under **very high load** (loadavg ≫ ncpu) — use as absolute sample, not A/B win claim.
`tools/perf/run_abba_matrix.sh` now includes these pairs when `FILE_SAMPLE=1` (default).

| fixture | median wall s (all slots) | min | max |
|---|---:|---:|---:|
| file_100_v_file_101 | 0.13 | 0.12 | 0.15 |
| file_130_v_file_131 | 0.12 | 0.12 | 0.12 |
| file_1_v_file_2 | 0.07 | 0.05 | 0.08 |
| file_50_v_file_51 | 0.03 | 0.01 | 0.03 |
| pdense_15k | 17.02 | 14.19 | 21.50 |
| redline_rfp17_vs_5lb102 | 49.73 | 37.02 | 68.08 |
| rfp17_redline_self | 0.71 | 0.70 | 1.70 |
| rfp17_vs_5lb102 | 41.72 | 28.58 | 78.30 |

document.xml digests: all fixtures **match=YES** (self-consistency).

## Commands to reproduce

```bash
# install pin
cargo build --release --bin jubarte --features cli
cp -f target/release/jubarte ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust/{jubarte,redline}
( cd ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust-inproc && cargo build --release )
cp -f ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust-inproc/target/release/jubarte-inproc   ../neurotic_docx_bench/src/neurotic_docx_bench/utils/jubarte/jubarte-rust/{jubarte-inproc,jubarte-worker}

# quality — main
( cd ../neurotic_docx_bench && uv run bench run --only jubarte-rust --rerun --accept-compare --no-gate )

# quality — file1_v_file2 expanded
( cd ../neurotic_docx_bench && uv run bench run -c bench.randomized.yaml --only jubarte-rust --rerun --no-gate )

# speed (1000 fixtures / 5000 pairs, includes randomized file_N)
( cd ../neurotic_docx_bench && node --import tsx scripts/redline_speed_bench.ts     --methods jubarte-rust-inproc,jubarte-rust     --fixture-count 1000 --min-pairs 5000 --warmup 50 --reps 1 --no-profile     --out results/redline_speed_bench/m233_head_d094de0 )

# criterion
cargo bench --bench redline -- --save-baseline m233_head

# expanded wall matrix
tools/perf/run_abba_matrix.sh target/release/jubarte target/release/jubarte _scratch/abba_expanded 1
```

## Ranking export

After this stamp, from neurotic_docx_bench:

```bash
python3 scripts/export-results-md.py
bun run update-readme-ranking  # optional
```
