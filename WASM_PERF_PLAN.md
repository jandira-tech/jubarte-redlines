<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# WASM performance plan — close the jubarte-wasm gap without losing parity

> Living document, drafted iteratively as findings land. Companion to
> `LCS_PERF_PLAN.md` (engine-level wall-time program). Scope here: why the
> WASM lane trails the native CLI on the 5,000-pair speed bench, and the
> ordered, evidence-gated experiments to close the gap. Fidelity gates
> precede speed claims (AGENTS.md): native/WASM `script_redlines` scores
> must stay equal per document for the same source commit.

## 0. Measured baseline (source commit `c7c7fbf`)

### 0a. Original two-lane published run

Run `019f6e1d-3c41-7604-86d8-20dea470572f`, 1,000 fixtures → 5,000 pairs,
`wasm-pack --target nodejs --release` + `wasm-opt -O3`,
artifact 1,987,810 bytes (`73d76228…7ec446`). Fidelity: 164/164 scored docs
identical native vs WASM, zero failures both lanes.

| Engine | Median | Mean | p95 | p99 | Throughput |
|---|---:|---:|---:|---:|---:|
| Native Rust CLI (spawn + file I/O per pair) | 10.428 ms | 32.914 ms | 129.333 ms | 202.766 ms | 30.4/s |
| Rust WASM (in-process, warm instance) | 10.967 ms | 44.596 ms | 191.773 ms | 292.953 ms | 22.4/s |

### 0b. W7 same-run three-lane measurement (this increment)

Run `w7-wasm-inproc-cli-c7c7fbf`, 2026-07-17T04:26:33Z, same host, same
1,000-fixture / 5,000-pair matrix, same warmup=50, same seed=42, same
artifacts. Adds `jubarte-rust-inproc` (long-lived stdin worker over the same
`compare_documents`) so the engine compute tax is separated from the CLI
spawn+I/O overhead in the same run.

| Rank | Engine | Median | Mean | p95 | p99 | Throughput | Wall |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | jubarte-rust-inproc (warm native) | **8.54 ms** | **33.05 ms** | 138.57 ms | 231.75 ms | 30.3/s | 165.25 s |
| 2 | jubarte-rust (CLI, spawn+I/O per pair) | 11.04 ms | 35.21 ms | 136.25 ms | 243.78 ms | 28.4/s | 176.05 s |
| 3 | jubarte-wasm (warm in-process) | 11.07 ms | 44.93 ms | 193.74 ms | 292.90 ms | 22.3/s | 224.67 s |

Artifacts: `results/redline_speed_bench/w7-wasm-inproc-cli-c7c7fbf/{report.md,
summary.json, speed.jsonl}` under the benchmark repo. Zero failures all lanes.

**Shape of the deficit is the first diagnostic.** Against the fair warm-native
baseline (inproc), the WASM compute tax is now a measured same-run number:

| metric | inproc (warm native) | wasm | wasm / inproc | CLI / inproc (spawn tax) |
|---|---:|---:|---:|---:|
| median | 8.54 ms | 11.07 ms | **1.30x** | 1.29x (≈2.5 ms spawn+I/O) |
| mean | 33.05 ms | 44.93 ms | **1.36x** | 1.07x |
| p95 | 138.57 ms | 193.74 ms | **1.40x** | 0.98x |
| p99 | 231.75 ms | 292.90 ms | **1.26x** | 1.05x |

The CLI's median beat WASM's median in the original published two-lane table
(10.4 ms vs 11.0 ms), but that comparison flattered the CLI: in the same run,
warm native is 23% faster at the median than the CLI and 23% faster than WASM.
The real WASM tax vs the fair baseline is **~1.3x at the median and ~1.36x at
the mean**, not the 1.6–2.0x cross-run estimate from Section 0a. The tail
(p95/p99) is where WASM still hurts most (1.26–1.40x), consistent with the
allocator hypothesis (H1) biting on heavy documents. Section 0a's historical
estimate is retained as evidence of why W7 was needed; Section 0b supersedes
it as the baseline every later increment is judged against.

**CLI spawn+I/O cost, measured in the same run:** ~2.5 ms at the median
(11.04 − 8.54), near zero at p95/p99 (compute dominates the tail on all
lanes). Smaller than the ~4–5 ms historical estimate, consistent with a
warm filesystem cache on this specific run.

## 1. Findings ledger (evidence-first; updated each iteration)

| id | finding | evidence | consequence |
|---|---|---|---|
| F1 | The workload is allocator-bound: ~35–41% self-time in alloc/copy/free/drop of `xmllinq` nodes. Native CLI ships **mimalloc** by default (`fast-alloc`, PR-A: ~21% user-CPU win). The WASM adapter builds `jubarte` with `default-features = false`, so wasm32 falls back to Rust std's bundled **dlmalloc** — a much weaker allocator under churn. | `Cargo.toml` (fast-alloc comment), `LCS_PERF_PLAN.md` MEASURED #3 profile; adapter `jubarte-wasm/Cargo.toml` | **Primary suspect** for the mean/p95 gap. Lane W1. |
| F2 | Build config is already near-best-practice: `opt-level=3`, fat LTO, `codegen-units=1`, `panic=abort`, `strip=symbols`, `wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext`. **Missing: SIMD128** (`-C target-feature=+simd128` at rustc level plus matching wasm-opt enable). miniz_oxide/memcpy-ish loops gain autovectorization. (v2 note: the memchr-simd upside originally assumed the regex stack is live; F13 shows `regex` is unused and the XML parser is scalar, so expectations were lowered — see F13/H2.) | adapter `Cargo.toml` profile + `[package.metadata.wasm-pack.profile.release]`; wasm lockfile | Lane W2, low risk. |
| F3 | DEFLATE backend parity holds: both native and WASM resolve `flate2 1.1.9` + `miniz_oxide 0.8.9` (pure Rust on both sides). No native-only C codec advantage. `zstd-sys` appears only in the native lock (via registry `rdocx-opc 0.1.0`) and is not on the DOCX deflate path; WASM uses the patched deflate-only `rdocx-opc 0.1.2`. | both Cargo.locks | ZIP codec is NOT the gap. Version skew 0.1.0 vs 0.1.2 flagged for consistency; fidelity already proven equal (164/164). |
| F4 | **Corrected:** SHA-1 is software on BOTH sides. `sha1 0.10.7`'s `compress.rs` cfg ladder has hardware paths only for x86/x86_64 (`sha` ext via cpufeatures) and loongarch asm; **aarch64 falls to `soft`**. No `sha1-asm` in the lock. So native Apple Silicon runs the same scalar rounds WASM does; the only gap is generic codegen quality. | `~/.cargo/registry/.../sha1-0.10.7/src/compress.rs` cfg_if read; lockfiles | H3 demoted to minor. Hash-volume reduction (HASH-STREAM lanes) helps both lanes roughly equally, not WASM specially. |
| F5 | JS boundary is cheap and sane: one `WebAssembly.Instance` per Node process created at `require` time; per call = copy original + modified into wasm memory, copy result out (`slice()`), then free. No per-call instantiation, no JSON, no base64. A few MB of memcpy per heavy pair ≈ tens–hundreds of µs; minor vs a 10ms+ budget. | `pkg/jubarte_wasm.js` glue read | Lane W4 is low priority; do not micro-optimize the boundary before allocator/SIMD. |
| F6 | WASM linear memory only grows, driven by dlmalloc via `memory.grow`; default initial memory is tiny. Heavy tail documents force repeated grows inside the timed call; the first heavy pair in a process pays the high-water cost, later pairs reuse it. `--initial-memory` preset is untried. | wasm-bindgen defaults; no link-args in adapter | Lane W3: preset initial memory; measure p95/p99 movement and grow counts. |
| F7 | `strip = "symbols"` removes the wasm name section, so any V8 `--cpu-prof`/DevTools profile of the release artifact shows anonymous hex frames. There is no profiling story for the WASM lane today. | adapter `Cargo.toml` | Lane W5: dedicated profiling build (names kept) is a prerequisite for attributing the tail. |
| F8 | **Confirmed:** the published table compares unlike lanes. `redline_speed_bench.ts` spawns per pair for `jubarte-rust` (temp write + spawnSync + read) and times in-memory compare only for WASM; `jubarte-rust-inproc` (long-lived stdin worker over the same `compare_documents`) is the harness's own "fair algorithm comparison". Historical inproc rows quantify spawn+I/O ≈ 4–5 ms at the median. | script lines (fairness notes, `isNativeCliMethod`), report.md fairness section, `results/speed.jsonl` history | Lane W7: run wasm vs inproc-native in the same run; keep the CLI lane as the deployment-reality number. |
| F9 | `perf.rs` stage counters are wasm-safe (relaxed atomics), but `time_stage` calls `std::time::Instant::now()`, which **panics on wasm32-unknown-unknown**. The only other `Instant` use in the library is inside a `#[cfg(test)]` block (`comparer/moves.rs` speedup test); the production compare path is wasm-clean (deterministic counter-based unids, no entropy, no clocks). | `src/perf.rs` read; `rg Instant` over `src/` | W5 must not enable `perf-profile` timers on wasm as-is: use counters-only or inject a JS clock; V8 cpuprofile is the primary attribution tool. |
| F10 | Floats DO exist on the compare path: move-detection jaccard ratios, `detail_threshold`, LCS overlap ratios (`f64` in `comparer/moves.rs`, `comparer/lcs.rs`). Plain wasm simd128 stays IEEE-754-deterministic, but **relaxed SIMD (FMA) is nondeterministic across hosts and must never be enabled**. | `rg '\bf(32|64)\b' src` | W2 stays precision-safe only with `+simd128`; add an explicit ban on `--enable-relaxed-simd` and any fast-math-style flag. |
| F11 | Toolchain on this host supports everything needed: `wasm-opt` 130 (full SIMD support), `wasm-pack` 0.15.0, `wasm32-unknown-unknown` installed, Node v25.9.0 (simd128 baseline since Node 16). | tool version probe | No toolchain blockers for W1–W3; record the Node floor in the adapter README. |
| F12 | The harness already has a profiling story: `--profile` wraps Node lanes in a v8-inspector `.cpuprofile` + top-self-frame analysis (`analyzeCpuProfile`), and samply for native lanes. The published run used `--no-profile`; this run's `cpu/` dir is empty. A `wasm_vs_wasm` A/B run directory precedent exists. | script `withV8CpuProfile`; results dir listing | W5 = small-N rerun with `--profile` + a names-kept wasm build (F7). No new tooling needed. |
| F13 | Dead direct dependencies in the engine crate: `regex = "1.12.4"` and `quick-xml = "0.41"` have **zero call sites in `src/`** (grep-verified; rdocx-opc vendors its own quick-xml internally). The main XML parse path is the hand-rolled scalar scanner in `src/xmllinq/parse.rs`. | `rg 'regex::|quick_xml' src` empty | Hygiene: drop both from `Cargo.toml` (build-time + lock surface; LTO already DCEs the code). Also calibrates W2: memchr-simd applies to miniz_oxide/bulk ops, NOT the XML parser — the bigger parse win is the engine-level PARSE-01 byte-scanner lane (shared). |
| F14 | **W5 WASM profile falsifies H1 as the #1 hotspot.** Top self-time frames: zlib_rs `deflate_medium` 16.62%, `inflate` 6.77%, `intern_str` 6.33%, `longest_match` 6.09%, `SipHasher write` 4.13%, `flush_block_only` 3.58%, dlmalloc `malloc`+`free` 3.39%+3.24% = **6.63%**, serializer Scope cluster (`assign` 2.92% + `prefix_for_uri` 2.66% + `descendants` 2.24% = **7.82%**). The deflate cluster (**26.29%**) is the #1 WASM frame, NOT the allocator. The interning pool (`intern_str` + `SipHasher` = **10.46%**) is #2. H1 was extrapolated from the native 35–41% allocation profile; the WASM profile shows deflate dominates. | W5 `summary.json` top_cpu; `results/wasm_perf/112a395-w5-profile/run/summary.json` | **Re-rank Section 2:** deflate > interning > serializer > allocator. ZIP-LEVEL-01 (lower output deflate level 6→1) is the highest-leverage in-repo increment; FXHASH-01 (SipHash→FxHash) is the follow-on. |
| F15 | **W3 (64 MiB initial-memory) regressed all four percentiles.** Same-run 5k A/B vs W2 SIMD: median 11.317 (+5.1%), mean 46.057 (+5.0%), p95 197.674 (+4.5%), p99 328.085 (+16.8%). Memory trace: grow events 5→3 but total grown pages unchanged (5030 vs 5031), high-water +19.9% (331→397 MB). The `--initial-memory` hypothesis (reduce grow events) failed: total grown pages are essentially unchanged AND the higher initial memory increased the high-water mark. | `results/wasm_perf/112a395-w3-initial-64m/full-5k/summary.json`; memory trace | **Drop W3.** Revert `.cargo/config.toml` to W2-only (simd128, no initial-memory). H5 demoted. |
| F16 | **The alignment peak is edit-count-INDEPENDENT — it tracks atom (run) count.** Counting-allocator profile of the full 276k-run dissertation pair (`examples/mem_profile.rs`, system allocator, native release): (a) many-edit real revision `a→b`: **10,722.7 MiB** live-heap peak, **544.1M** allocations, 39.3 s; (b) SINGLE-word edit `a→a'`: **10,739.5 MiB**, **545.0M** allocs, 36.5 s — **within 0.2%** of the full revision; (c) identical pair `a→a`: **1,089.6 MiB**, 25.5M allocs, 1.9 s (short-circuits atom correlation). macOS `peak memory footprint` 11.15–11.57 GiB (matches the ~11.9 GB TODO figure); max RSS 4.5–6.5 GiB. A single edit costs the SAME peak as a full rewrite: the driver is the per-atom `ComparisonUnitAtom` churn (ancestor-chain `Vec` + sha1 `String`, cloned through `tag_all`/`resolve`), incurred whenever the pair is non-identical. | `examples/mem_profile.rs` on `dissertacao-{a,b}.docx`; `_scratch/mem_profile_*.log` | **wasm32 verdict:** any real diff peaks ~11 GiB ≈ **2.9× over the 4 GiB linear-memory ceiling** → aborts (`unreachable`; allocator dies before the panic hook). Identical pairs (1.06 GiB) pass. Beyond-ceiling docs take the native/server path (Section 10). |

## 2. Root-cause hypothesis ranking (re-ranked after W5 profile, 2026-07-17)

1. **Deflate cluster (26.29% of WASM self-time) — ZIP-LEVEL-01.** The output
   zip's deflate level defaults to 6 (`SimpleFileOptions::default()` in
   `rdocx-opc`/`zip`); level 1 (`deflate_quick`) eliminates `longest_match`
   (6.09%) and cuts `deflate_medium` (16.62%) iterations. Fidelity-safe by
   construction (decompressed bytes identical; Word opens any deflate level).
   ~35 lines in `src/opc/mod.rs`. **Highest expected gain, lowest parity risk.**
2. **H4 — inherent wasm codegen tax** (bounds checks, no NEON autovec, V8
   codegen quality — includes running the soft SHA-1 rounds slower than
   native runs the same soft rounds, per corrected F4). Typically ~1.1–1.5x
   for branchy pointer-chasing code; the residual after the deflate fix.
   Partially shaveable via wasm-opt experiments; the rest is paid down by
   engine work-reduction (W6), which shrinks the base the tax multiplies.
3. **Interning pool (10.46%) — FXHASH-01.** `intern_str` (6.33%) +
   `SipHasher write` (4.13%). SipHash is a WASM top-5 frame but **absent from
   the native top frames** (MEASURED #3 lists `mi_free`/`memmove`/`drop_in_place`,
   no SipHash) — this is a WASM-specific codegen artifact, not an algorithmic
   cost. A final-mixed FxHash (`rustc-hash` or hand-rolled) attacks both the
   hasher rounds and the HashSet probe. ~60 lines in `src/xmllinq/mod.rs`.
   Kill condition: the FNV-1a precedent (MEASURED #5, +16% native regression)
   means a universal swap must be `cfg(wasm32)`-gated if it regresses native.
4. **H1 — allocator (dlmalloc vs mimalloc).** Demoted from #1 to #4: the W5
   profile shows dlmalloc at 6.63% (#4), not the predicted #1. W1 (talc) was
   tested and dropped (no improvement over W2 SIMD). Still worth addressing
   but not the top frame; the deflate and interning lanes have higher leverage.
5. **H5 — memory.grow tail effects** — W3 falsified (F15): 64 MiB
   initial-memory regressed all four percentiles. H5 is effectively dead
   unless a future profile shows grow-page faults inside timed calls.
6. **H3 — SHA-1 hardware gap.** Demoted: none exists on this host (F4
   corrected); soft-vs-soft only differs via H4.

Target end state: WASM mean/p95 within ~10–20% of same-run in-process native
(H4 floor), median at or below the spawning CLI (it nearly is already: 11.07 ms
WASM vs 11.04 ms CLI in the W7 same-run measurement).

## 3. Experiment lanes (one increment at a time; fidelity gate each)

Every lane ships as its own measured increment: rebuild both artifacts from
the same source commit, rerun `script_redlines` for the WASM lane (score
equality per document is the gate), then the 5,000-pair speed lane, A/B
against the immediately previous WASM artifact. Never batch two mechanisms.

### W1 — allocator swap in the adapter crate (highest expected value)

- Where: `jubarte-wasm/src/lib.rs` (+ `Cargo.toml`). Engine source untouched;
  the adapter owns packaging, consistent with the ownership map.
- What: `#[global_allocator]` a wasm-tuned allocator. Primary candidate:
  **talc** (fast wasm allocator, purpose-built for this niche). Fallback
  candidates: rlsf. Forbidden: wee_alloc (unmaintained, slow, leaky).
- Note: the adapter crate may use the small `unsafe` init such allocators
  require; the `unsafe_code = deny` lint is a jubarte-crate policy, not an
  adapter policy. Keep the unsafe confined to the adapter.
- Gate: fidelity equality 164/164; speed A/B on the 5k lane; also confirm
  peak linear-memory high-water does not regress badly (allocator policy
  changes can trade speed for footprint; the engine already has a
  multi-GB-RSS pathology on huge fixtures).

### W2 — SIMD128 build

- What: build with `-C target-feature=+simd128` via
  `[target.wasm32-unknown-unknown] rustflags` in the adapter's
  `.cargo/config.toml` (deterministic under wasm-pack, unlike an env var)
  and add `--enable-simd` to the wasm-opt flag list.
- Precision (F10): the compare path DOES use `f64` (move-detection jaccard,
  thresholds, overlap ratios). Plain simd128 is IEEE-754-deterministic, so
  results cannot change; **never enable `--enable-relaxed-simd`** (FMA is
  host-nondeterministic) or any fast-math-style flag. Gate as always on
  164/164 score equality.
- Expectation (F13): modest — deflate (miniz_oxide), bulk copies, scattered
  autovectorization. The XML parser is a scalar char scanner, untouched by
  memchr SIMD. Land it because it is free, not because it is the fix.
- Host support: Node/V8 simd128 since Node 16; host runs Node 25.9 /
  wasm-opt 130 (F11). Document the floor in the adapter README.
- Optional same-lane experiment: `wasm-opt -O3 --converge`, and an `-O4`
  A/B. Keep flags additive and measured one at a time.

### W3 — preset linear memory

- What: `-C link-arg=--initial-memory=<N>` (e.g. 256 MiB, page-multiple;
  optionally `--max-memory=4294967296`) in the adapter build; V8 commits
  pages lazily so the physical cost is near zero.
- Measure: p95/p99 movement and per-call `wasm.memory.buffer.byteLength`
  growth trace (diagnostic patch in the bench harness, not in `pkg/`).
- Caution: do NOT stack `--low-memory-unused`-style wasm-opt tricks without
  checking the stack-first layout rustc uses; treat as a separate,
  verify-first experiment.

### W4 — boundary hygiene (parked unless W5 profile says otherwise)

- Current cost: 2 input copies + 1 output copy per call. Options if ever
  needed: reusable input buffers exposed by the adapter, or writing output
  length to a retptr and letting JS view-copy once (already effectively one
  copy). Expected win: sub-millisecond; keep parked.

### W5 — profiling story for the WASM lane (prerequisite for tail work)

- Build a names-kept artifact: release profile with `strip = false` +
  `wasm-opt -O3 -g` (or wasm-pack `--profiling`) — same codegen class,
  symbolized frames.
- Use the harness's existing `--profile` v8-inspector capture (F12) over a
  small-N run biased to the heaviest pairs; read the emitted
  `<method>.hot.json` top-self frames. Attribute time to dlmalloc
  malloc/free vs soft-SHA-1 vs LCS vs parse vs serialize vs deflate. This
  fixes the Section 2 ranking with data instead of inference.
- Counters: `perf-profile` counters are wasm-safe, but `time_stage` panics
  on wasm (`Instant`, F9). If stage timers are wanted in-wasm, add a
  cfg'd clock shim (e.g. `js_sys` performance.now) in the adapter or a
  counters-only mode; otherwise rely on the V8 profile alone. Never ship a
  profiled artifact as the benchmarked artifact.

### W6 — engine-level allocation reduction (shared with LCS_PERF_PLAN)

- Every allocation-churn cut in the native program (ATOM-ID, ACCEPT-INPLACE,
  HASH-STREAM, RESULT-DOM lanes) helps WASM disproportionately, because the
  dlmalloc/mimalloc gap multiplies per allocation. The WASM gap is partly a
  magnifying glass on the arena-churn architecture.
- Action: when an engine increment ships in the native lane, record the WASM
  delta too, so the cross-lane multiplier becomes a known constant.

### W7 — fair-baseline lane in the bench — DONE (2026-07-17)

- **Shipped:** ran the 5,000-pair speed lane with
  `jubarte-rust-inproc,jubarte-rust,jubarte-wasm` from the same `c7c7fbf`
  artifacts, same host, same seed=42, same 1,000-fixture matrix.
- **Result (Section 0b):** the WASM compute tax vs the fair warm-native
  baseline is **~1.30x median, ~1.36x mean, ~1.40x p95, ~1.26x p99** —
  materially better than the 1.6–2.0x cross-run estimate. CLI spawn+I/O tax
  in this run is ~2.5 ms at the median (smaller than the historical ~4–5 ms,
  likely warm fs cache).
- **Artifacts:** `results/redline_speed_bench/w7-wasm-inproc-cli-c7c7fbf/`
  under the benchmark repo; row appended to global `results/speed.jsonl`.
- **Standing rule:** publish all three rows together from now on — inproc
  (algorithm), CLI (deployment reality), WASM (portability lane). The
  harness's default `--methods` list already leads with the inproc lanes;
  future runs should not drop them.

### W8 — host/runtime tuning (only with profile evidence)

- Node/V8 knobs (tiering, GC) are unlikely to matter with 50 warmups and a
  warm instance; do not cargo-cult flags. Revisit only if W5 shows tier-up
  or GC frames inside timed regions.

## 4. Best practices actually applicable here (issue 2, part 1)

Already applied (keep): `--release` + `opt-level=3`, fat LTO, CGU=1,
`panic=abort` (smaller code, no unwind tables), `strip` for ship artifact,
`wasm-opt -O3` with bulk-memory/sign-ext/nontrapping-fptoint, single warm
instance, byte-slice API (no serde/JSON at the boundary), panic hook opt-in.

Missing / to adopt (ordered): wasm-tuned global allocator (W1); simd128
(W2); preset initial memory (W3); a profiling build variant (W5); a same-run
inproc baseline row (W7); drop the dead `regex`/`quick-xml` direct deps
(F13, engine repo); publish the Node version floor and the exact wasm-opt
flag set in the adapter README so results are reproducible.

Explicitly rejected: wee_alloc (unmaintained, slow, leaky); `-Oz`/`-Os`
(size-first hurts this compute-bound artifact); `--enable-relaxed-simd` and
any fast-math-style flag (F10: f64 on the compare path; host-nondeterministic
FMA would risk fidelity); threads/rayon (engine is verified single-threaded;
wasm threads need cross-origin isolation and buy nothing here); nightly
`build-std` rebuilds (fragile toolchain pin for uncertain gain — bank as a
last-mile experiment only); `--traps-never-happen` style semantic wasm-opt
flags without a dedicated correctness argument.

## 5. Code-level improvement map (issue 2, part 2 — where and what)

Ordered by expected WASM effect; native profile evidence carried over from
LCS_PERF_PLAN, to be re-confirmed on wasm by the W5 profile.

| where | what to improve | why it matters MORE on wasm |
|---|---|---|
| adapter `jubarte-wasm/src/lib.rs` | add `#[global_allocator]` (W1, talc-class) | the ~40%-allocation profile currently runs on dlmalloc; this is the single highest-leverage change and touches zero engine code |
| adapter `Cargo.toml` / `.cargo/config.toml` | `+simd128`, `--enable-simd`, `--initial-memory` (W2/W3) | free codegen headroom; grow-free heavy pairs |
| `src/xmllinq/` (`NodeData`, `clone_subtree`, arena growth) | continue node-count and node-size reduction (ATOM-ID-01, ATOM-VIEW-01, RESULT-DOM-01, kind-specific layout — LCS_PERF_PLAN Lane M/A) | every avoided alloc/free skips a dlmalloc call; the wasm tax multiplies per allocation, so these engine lanes pay ~1.6–2x their native win on wasm |
| `src/revision_processor.rs` accept pipeline | more ACCEPT-SKIP/ACCEPT-INPLACE increments (banked lanes A1/A3/A5/A7/A8 plus unshipped rebuilds) | full-tree rebuild-and-drop churn is pure allocator traffic; tail documents are exactly where WASM p95/p99 blow up |
| `src/xmllinq/parse.rs` | PARSE-01 byte-indexed scanner (replace `Vec<char>` walking) | scalar char loops carry the full H4 codegen tax; a byte scanner cuts instructions AND allocations; SIMD does not reach this code today (F13) |
| `src/document_comparer.rs` + `src/comparer/` hashing | finish HASH-STREAM lanes (03b/04 banked; extend no-clone stream digests) | digest projections are alloc+memcpy+soft-SHA1, all wasm-expensive; fewer hashed bytes shrink the H4 base |
| `src/unid.rs` | `format!("{n:032x}")` allocates a fresh 32-byte String per stamped element | tiny per node but runs across whole documents; candidate only if the W5 profile shows it — do not pre-optimize |
| `Cargo.toml` (engine) | remove unused `regex` + `quick-xml` deps (F13) | hygiene: lock surface, build time; no runtime claim |

Engine-side rule of thumb this map encodes: **on wasm, the cheapest
instruction is the allocation you never made.** The LCS_PERF_PLAN wall-time
program and this plan converge on the same engine work; this plan adds the
adapter-level items the native program cannot see.

## 6. Measurement protocol and gates

1. Rebuild native CLI and WASM from the same commit; record binary SHA-256s.
2. Fidelity first: `script_redlines` for `jubarte-wasm`; require per-document
   score equality with the native run for that commit (164/164). A speed win
   never excuses a fidelity difference.
3. Speed: 1,000 fixtures / ≥5,000 pairs, 50 warmups, same host, idle machine;
   compare candidate vs previous WASM artifact; judge median AND mean AND
   p95/p99 (the gap lives in the tail).
4. One mechanism per increment; keep the experiment row (id, hypothesis,
   artifact hashes, result, verdict) in this file's ledger.
5. Wasm artifact size is tracked but secondary to wall time.

## 7. Open questions (drive next iterations)

- [x] Bench WASM lane process model → resolved (F8/F12): single warm
      process, in-memory timed loop; inproc lane exists; `--profile`
      supported.
- [x] `sha1 0.10.7` aarch64 hardware gate → resolved (F4 corrected): soft
      on aarch64; no hardware gap between the lanes.
- [x] `perf.rs` on wasm32 → resolved (F9): counters safe, `time_stage`
      panics; production path wasm-clean.
- [x] Floats on compare path → resolved (F10): present; simd128 OK,
      relaxed-simd banned.
- [x] Toolchain support → resolved (F11): wasm-opt 130 / wasm-pack 0.15 /
      Node 25.9 / target installed.
- [x] Same-run wasm-vs-inproc tax (W7) → **resolved 2026-07-17**: ran the
      three-lane bench; wasm tax is **~1.30x median / ~1.36x mean / ~1.40x
      p95 / ~1.26x p99** vs warm native (Section 0b). Lower than the 1.6–2.0x
      estimate; H1 allocator remains the lead suspect for the residual.
- [x] W5 wasm profile: confirm dlmalloc share ranks #1 as predicted →
      **resolved 2026-07-17**: NO. dlmalloc is 6.63% (#4), not #1. The
      deflate cluster is #1 at 26.29%. H1 falsified as the top hotspot; see
      F14. Section 2 re-ranked: deflate > interning > serializer > allocator.
- [x] W1 allocator pick: talc tested → **dropped** (no improvement over W2
      SIMD on the 5k lane). The allocator lane is demoted but not abandoned;
      revisit only if the deflate/interning fixes leave a large residual.
- [x] memory.grow count / high-water trace (W3 diagnostic) → **resolved
      2026-07-17**: 64 MiB initial-memory regressed all percentiles (F15);
      total grown pages unchanged (5030 vs 5031), high-water +19.9%. H5 dead.

## 8. Execution order (first increments, one at a time)

1. **W7 — DONE (2026-07-17).** Three-lane same-run baseline measured; see
   Section 0b.
2. **W1 — DONE, DROPPED (2026-07-17).** Talc allocator tested in the adapter;
   no improvement over W2 SIMD on the 5k lane. Demoted; revisit only if the
   deflate/interning fixes leave a large residual.
3. **W2 — DONE, RETAINED (2026-07-17).** SIMD128 build flags. Small additive
   win; retained as the WASM build baseline.
4. **W3 — DONE, DROPPED (2026-07-17).** 64 MiB initial-memory preset regressed
   all four percentiles (F15). Reverted to W2-only config.
5. **W5 — DONE (2026-07-17).** Names-kept WASM profile run. Falsified H1 as
   #1 (F14): deflate cluster is 26.29%, allocator is 6.63%. Section 2
   re-ranked.
6. **ZIP-LEVEL-01 — DONE, RETAINED (2026-07-17).** Rewrite `PartFs::to_zip`
   to use `zip::ZipWriter` directly with `.compression_level(Some(1))`,
   bypassing `rdocx-opc`'s default level 6. Fidelity gate: 164/164 per-doc
   equality, 0 failures. Speed A/B: **WASM median -9.8%, mean -5.5%, p95
   -4.6%, p99 -2.7% vs W2 SIMD.** Win on all four percentiles. WASM tax vs
   warm native narrowed to 1.15x median / 1.21x mean. Native CLI also
   improved (median -9.1%). Output size +18% (decompressed bytes identical).
7. **FXHASH-01 — NEXT.** Swap `STR_POOL`'s SipHash for a final-mixed FxHash.
   ~60 lines in `src/xmllinq/mod.rs`. Attacks the #2 WASM frame (10.46%).
   `cfg(wasm32)`-gate if it regresses native (FNV-1a precedent, MEASURED #5).
8. Reprofile (W5 again) after ZIP-LEVEL-01 + FXHASH-01, re-rank Section 2,
   then let W6 engine increments carry both lanes; record the wasm multiplier
   per engine increment.

## 9. Iteration log

- **v1:** baseline facts, findings F1–F8, hypothesis ranking H1–H5, lanes
  W1–W8, protocol and gates. Sources: adapter crate + glue JS + both
  lockfiles + LCS_PERF_PLAN measured history + GET_JUBARTE_RUST map.
- **v2:** verified corrections and quantification. F4 corrected (sha1 soft
  on aarch64 too → H3 demoted); F8 confirmed from harness source + history
  (spawn tax ≈ 4–5 ms median → true wasm tax ≈ 1.6–2.0x warm native); added
  F9 (perf.rs Instant panic on wasm; counters safe), F10 (f64 on compare
  path → relaxed-simd banned), F11 (toolchain verified), F12 (harness
  `--profile` v8-inspector support), F13 (dead `regex`/`quick-xml` deps;
  XML parser is scalar → simd expectation lowered). Re-ranked hypotheses
  (H1 > H4 > H2 > H5 > H3); filled the code-level improvement map; added
  execution order.
- **v3 (W7 done, 2026-07-17):** ran the three-lane same-run bench
  (`w7-wasm-inproc-cli-c7c7fbf`). Section 0 added 0b with the measured
  inproc row; the wasm compute tax is ~1.30x median / ~1.36x mean / ~1.40x
  p95 / ~1.26x p99 — lower than v2's 1.6–2.0x estimate. CLI spawn tax in
  this run is ~2.5 ms at the median (smaller than historical, likely warm
  fs cache). H1 re-anchored to the new baseline. Section 0a retained as
  historical evidence of why W7 was needed.
- **v4 (W1/W2/W3 done, 2026-07-17):** W1 (talc allocator) tested and
  dropped — no improvement over W2 SIMD on the 5k lane. W2 (SIMD128) tested
  and retained as the WASM build baseline (median 10.770, mean 43.849, p95
  189.255, p99 280.854 ms). W3 (64 MiB initial-memory) tested and dropped —
  regressed all four percentiles (F15: median +5.1%, mean +5.0%, p95 +4.5%,
  p99 +16.8%); memory trace showed grow events barely reduced (5→3), total
  grown pages unchanged, high-water +19.9%. Config reverted to W2-only.
- **v5 (W5 profile done, 2026-07-17):** names-kept WASM profile run
  (`112a395-w5-profile`). **Falsified H1 as the #1 hotspot** (F14): deflate
  cluster is 26.29% (#1), interning pool is 10.46% (#2), serializer is 7.82%
  (#3), dlmalloc is 6.63% (#4). Section 2 re-ranked: deflate > interning >
  serializer > allocator. ZIP-LEVEL-01 selected as the next increment
  (attacks the #1 frame, fidelity-safe, ~35 lines in-repo). FXHASH-01
  selected as the follow-on (attacks the #2 frame, WASM-specific).
- **v6 (ZIP-LEVEL-01, 2026-07-17):** rewrote `PartFs::to_zip` in
  `src/opc/mod.rs` to use `zip::ZipWriter` directly with
  `.compression_level(Some(1))` (deflate level 6→1, `deflate_quick`),
  bypassing `rdocx-opc`'s default level 6. Added `part_name_to_rels_path`
  helper (mirrors the private rdocx-opc function). Validators: `cargo fmt`
  clean, `cargo clippy -D warnings` clean, full test suite passes (incl. new
  `zip_level_01_roundtrip_member_identity` test), CLI `--help` smoke OK.
  Fidelity gate: **164/164 per-doc overall_score equality**, 0 failures both
  lanes, mean 91.9831 / median 99.904 (identical to baseline). Speed A/B
  (5k three-lane, same seed=42):
  **WASM: median 9.717 (-9.8%), mean 41.440 (-5.5%), p95 180.492 (-4.6%),
  p99 273.407 (-2.7%) vs W2 SIMD.** Win on all four percentiles. WASM tax
  vs warm native narrowed: median 1.30x→1.15x, mean 1.36x→1.21x, p95
  1.40x→1.25x, p99 1.26x→1.12x. Native CLI also improved (median 9.66 vs
  10.63, -9.1%) — deflate is a shared-wall lever. Output size +18% (level 1
  compresses less; decompressed bytes identical, Word-valid). **RETAINED.**
- **v7 (MEM-PROFILE-01 / wasm32 memory ceiling, 2026-07-17):** added
  `examples/mem_profile.rs` (counting `#[global_allocator]`) and profiled the
  full 276k-run dissertation pair. Established F16: the alignment peak is
  **edit-count-independent** (single-word edit 10,739.5 MiB ≈ full revision
  10,722.7 MiB, within 0.2%; identical pair short-circuits to 1,089.6 MiB).
  Any real diff peaks ~11 GiB — ~2.9× over the wasm32 4 GiB ceiling. Added
  Section 10 (product stance + budget). Pinned the 8 MB shadow-stack rustflag
  into the adapter `.cargo/config.toml` alongside `+simd128` (TODO §2) so a
  bare `wasm-pack build` carries the full recipe.

## 10. wasm32 memory ceiling on run-fragmented documents (MEM-PROFILE-01)

Some real documents cannot be diffed inside wasm32's 4 GiB address space, and
no in-repo optimization changes that — the ceiling is architectural. This
section is the product stance and the measurement that fixes it in place.

### 10a. The measurement (full dissertation, native, system allocator)

`examples/mem_profile.rs` wraps the system allocator in a counting allocator
and runs three compares on the 276k-run dissertation pair
(`dissertacao-a.docx` 9.33 MiB, `dissertacao-b.docx` 9.30 MiB):

| case | edits | compare-peak live heap | allocations | wall | peak footprint (RSS) |
|---|---|---:|---:|---:|---:|
| `a → b`   | full revision | **10,722.7 MiB** | 544.1M | 39.3 s | 11.57 GiB (4.47 GiB) |
| `a → a'`  | **single word** | **10,739.5 MiB** | 545.0M | 36.5 s | 11.15 GiB (6.46 GiB) |
| `a → a`   | none (identical) | **1,089.6 MiB** | 25.5M | 1.9 s | — |

Reproduce:
```bash
cargo build --release --example mem_profile --no-default-features
/usr/bin/time -l ./target/release/examples/mem_profile   # defaults to the dissertation pair
```

### 10b. What it proves

- **Edit count does not matter.** A one-word edit (10,739.5 MiB) costs the same
  peak as a full rewrite (10,722.7 MiB) — the cost is driven by the ~276k
  run-fragmented atoms, not by how many changed. The driver is per-atom
  `ComparisonUnitAtom` churn (ancestor-chain `Vec` + sha1 `String`, cloned
  through `tag_all`/`resolve`), paid whenever the pair is non-identical.
- **Only the identical short-circuit escapes it.** `a → a` peaks at 1.06 GiB
  because equal `document.xml` hashes let the correlation return early before
  the alignment allocations. That is the one case that fits under 4 GiB.
- **wasm32 verdict:** ~11 GiB peak is ~2.9× the 4 GiB linear-memory ceiling.
  The allocator dies before the panic hook runs, so the failure surfaces as a
  bare `unreachable` — even a single-word edit OOMs, while an identical-pair
  compare passes.

### 10c. Product stance (the fix)

- **In-browser wasm handles what fits; the server/native path handles the rest.**
  Documents whose predicted peak exceeds a wasm32 budget are diffed on the
  native/server engine (same crate, no 4 GiB cap). The deployed demo already
  precomputes the dissertation redline server-side; the wasm lane is for the
  interactive, in-ceiling majority.
- **Budget line (feeds the bench, TODO §1):** classify by input size and pin a
  peak-memory budget per class, with an explicit **wasm32-viable: yes/no** flag
  derived from the predicted peak vs 4 GiB. On this corpus: sub-MB run-normal
  docs are comfortably wasm32-viable; the ~9.8 MB / 276k-run dissertation class
  is **wasm32-viable: no** (native/server only). The budget is enforced in
  `neurotic_docx_bench` (`config.py` size-class budgets + a memory gate mirroring
  `gate.py`); the wasm speed lane records the ceiling breach rather than shipping
  a misleading "wasm failed" row.
- **Engine follow-on (optional, shared with LCS_PERF_PLAN / W6):** the peak is
  ~21× the identical-pair floor purely in alignment allocations; a lower-churn
  atom representation (interned ancestor chains, `Box<[NodeId]>` instead of
  growable `Vec`, borrowed sha1 keys) would lower the ceiling-crossing size but
  not remove the class — run-fragmented pathological inputs will always exist.
