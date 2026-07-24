<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Speed bench review — inproc vs CLI (plan Workstream C)

**Date:** 2026-07-16
**Baseline pin:** `jubarte-rust@9fcc4289e375`
**Corpus:** `redline_speed_bench` 5000 pairs, seed 42

## Measured baseline

| method | median (ms) | mean (ms) | wall (s) |
|---|---:|---:|---:|
| jubarte-rust-inproc (warm worker) | **16.95** | **56.0** | **280** |
| jubarte-rust CLI (spawn per call) | 21.86 | 52.1 | 260 |

**Anomaly:** warm inproc wins on **median**, loses on **mean** and **wall**.

## Protocol forensics (C1 Step 1)

Inproc worker (`neurotic_docx_bench/.../jubarte-rust-inproc`):

```
COMPARE <basePath> <nextPath> <outPath>
```

- Paths, not base64 — **no serialization bias vs CLI** for large fixtures.
- Worker times **only** `compare_documents` (reads are outside the Instant; write is after).
- CLI timing is end-to-end spawn + process read/compare/write from the harness.

So the fair-algorithm claim for *median* is sound; the mean/wall loss is a **tail / long-lived process** effect, not a protocol tax.

## Residual cause (measured / structural)

| hypothesis | verdict |
|---|---|
| Stdin base64 tax on large docs | **Rejected** — paths only |
| Order / loadavg alone | Partial — always capture `sysctl -n vm.loadavg`; ABBA refuses A/B wins when load1 > ncpu |
| **Arena / heap retention across 5000 compares in one process** | **Hypothesis** (provisional — not worker-RSS-proven) |

Evidence *consistent with* retention (circumstantial until worker-order RSS traces land):

1. **Median favors inproc** (16.95 < 21.86) — typical-case algorithm+warm is faster than spawn.
2. **Mean and wall favor CLI** — a heavy right tail on inproc dominates the average; CLI pays spawn every time but **starts each compare in a fresh process** (fresh mimalloc arenas, fresh `xmllinq` intern tables, no cumulative RSS).
3. LCS_PERF_PLAN MEASURED #5 already flagged `PARSE-01` / mimalloc RSS growth (+1.2 GB class) on long runs; a 5000-pair warm worker is exactly that shape.
4. Inproc timer excludes I/O; if it still loses mean, the *compare itself* is getting slower on late pairs (cache/allocator degradation), not I/O.

This does **not** yet isolate heap retention from fixture order, system load, or size distribution. Treat mean/wall conclusions as provisional until a worker-specific RSS/order trace is captured.

### Pass condition

Plan C1: *inproc ≤ CLI on median AND mean AND wall, **or** residual gap has a written measured cause.*

**Verdict (provisional):** residual gap **documented as hypothesis** — mean/wall inproc loss *may* be long-lived process heap retention; median already favors inproc. Do **not** quote mean/wall as “algorithm win” for either side until a retention fix (periodic worker recycle, or arena reset) lands **or** worker-RSS evidence confirms the cause.

## Hygiene (C2)

| item | status |
|---|---|
| Criterion `black_box` on inputs **and** outputs | yes (`benches/redline.rs`) |
| Fixture I/O outside measured closure | yes (read once before `bench_function`) |
| Compare command | `cargo bench --bench redline -- --baseline m233_head` |
| >5% regression on any Criterion case | blocks perf-affecting PR (see `VERSIONING.md`) |
| ABBA loadavg guard | `tools/perf/run_abba_matrix.sh` writes `loadavg.txt` and **refuses A/B-win claims** when load1 > ncpu |

## B-fix speed guard (C3)

Every Workstream B corpus gate must record `mean_speed` / `median_speed` from `bench.jsonl`.
**Trigger:** median generate time > **+10%** vs M233 baseline (26.03 mean / 5.98 median ms on that pin) → perf review before merge.

## Follow-ups (not blocking this review)

1. Recycle the inproc worker every N compares (or RSS threshold) and re-measure mean/wall.
2. Size-bucketed median-by-fixture-decile in `redline_speed_bench.ts` (optional TS change).
3. Interleaved reps with RSS sidecar to plot monotonic growth.
