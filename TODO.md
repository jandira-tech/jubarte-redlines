# TODO — jubarte-redlines

Findings from the 2026-07-17 folio×jubarte demo session (all measured on this
machine; the demo corpus lives in the folio playground,
`packages/playground/public/redline3/`).

> **STATUS 2026-07-17 (evening):** all items below RESOLVED. Engine work is in
> this repo (`~/T/jubarte-redlines`); the bench-harness items are in
> `~/T/neurotic_docx_bench` (commit `36a81db`). See the per-item notes.

## 1. wasm32 memory ceiling on run-fragmented documents (HIGH)

A real 276k-run dissertation pair (9.8 MB docx, character-shredded runs)
needs **~11.9 GB peak memory footprint** (6.6 GB max RSS) to compare
natively. wasm32's 4 GiB address space makes ANY real diff of such documents
abort (`unreachable`; the allocator dies before the panic hook can run —
even a single-word edit OOMs, while an identical-pair compare passes because
it short-circuits the alignment allocations). Feeds `WASM_PERF_PLAN.md` /
the marginal-gain loop:

- [x] Add a peak-memory budget per corpus size class to the bench, with an
  explicit wasm32-viability line. **RESOLVED** — `neurotic_docx_bench/
  src/neurotic_docx_bench/memory_budget.py` (`classify`, `wasm32_viable`,
  `budget_gate`, `SizeClass` table; optional `memory_budgets:` bench.yaml
  block). Dissertation (9.8 MiB, 11.57 GiB peak) → class `large`,
  `wasm32_viable=False`, over-budget `fail`. Tests: `tests/test_memory_budget.py`.
- [x] Profile the alignment allocations on run-fragmented inputs (the cost is
  triggered by ANY diff, independent of edit count). **RESOLVED** —
  `examples/mem_profile.rs` (counting `#[global_allocator]`) on the full
  dissertation proves it (MEM-PROFILE-01, `WASM_PERF_PLAN.md` §10 / F16):
  full-revision peak 10,722.7 MiB vs SINGLE-word-edit peak 10,739.5 MiB
  (**+0.2% — edit-count-independent**) vs identical-pair 1,089.6 MiB
  (short-circuit). ~544M allocations, dominated by per-atom
  `ComparisonUnitAtom` churn (ancestor `Vec` + sha1 `String`).
- [x] Document the product stance: beyond-ceiling documents take the
  native/server path (the deployed demo already precomputes them).
  **RESOLVED** — `WASM_PERF_PLAN.md` §10c. Verified end-to-end: the harness
  speed bench on the full dissertation (all three lanes) has the native CLI
  (~30.3 s) and inproc (~35.8 s) lanes succeed while `jubarte-wasm` aborts
  with `unreachable` (results under `neurotic_docx_bench/results/
  redline_speed_bench/dissertacao_v2/`).

## 2. Build recipe must ride the engine pin (MEDIUM)

The browser wasm is now built with `RUSTFLAGS="-C link-arg=-zstack-size=8388608"`
(8 MB shadow stack; the 1 MB default was the first OOM suspect and remains a
real risk on deep recursion) on top of the wasm-pack `-O3` profile. "Same
engine commit" no longer identifies the artifact:

- [x] Extend the bench's `ENGINE_COMMIT` pin mandate to record the FULL build
  recipe (rustflags, wasm-pack target/profile, wasm-opt flags). **RESOLVED** —
  `neurotic_docx_bench/src/neurotic_docx_bench/tool_updater.py::resolve_build_recipe`
  parses rustflags (`.cargo/config.toml`) + wasm-opt flags (`Cargo.toml`
  `[package.metadata.wasm-pack]`) via `tomllib`; `Results.build_recipe`
  threaded through `emit/jsonl.py`. Verified on the vendored adapter: captures
  `+simd128`, `link-arg=-zstack-size=8388608`, `-O3` + the wasm-opt SIMD flags.
  Tests: `tests/test_build_recipe.py`.
- [x] Consider committing the stack-size flag into the adapter crate's
  `[package.metadata.wasm-pack]` config in neurotic_docx_bench so it cannot
  be forgotten. **RESOLVED** — the flag now lives in the adapter's
  `.cargo/config.toml` (rustflags, alongside `+simd128`), not in an env var
  that a bare `wasm-pack build` would drop. The adapter is now **vendored and
  tracked** in this repo (`jubarte-wasm/`, commit `bd74e8b`), so the recipe is
  version-controlled. Verified baked into the artifact: the built wasm's
  `__stack_pointer` global initializes to `i32.const 8388608` (8 MiB, vs the
  1 MiB default).

## 3. Cross-engine reference behavior (context, no action)

On the same demo corpus where the TS lossless port mis-marks rows/paragraph
marks (see jubarte-first/TODO.md), this engine's output passes folio's
engine-independent self-check on every pair, including the dissertation
(accept ≡ revised, reject ≡ base, verified through folio's reviewer). Keep it
that way: any future emission change should re-run the folio judge sweep in
`folio/packages/playground/debug-verify-buffer.mjs`.

- **NOTE (2026-07-17):** ZIP-LEVEL-01 (`src/opc/mod.rs`, commit `f488f2c`) is
  an emission change (deflate level 6→1, +18% output size). The folio repo is
  not present on this machine, so the judge sweep could not be re-run here, but
  the invariant holds **by construction**: `to_zip` produces byte-identical
  *decompressed* members (proven by `tests/m_validity_ring1.rs::
  zip_level_01_roundtrip_member_identity` + the 164/164 fidelity gate), and the
  folio judge compares decompressed/parsed content, so its verdict is
  unchanged. The full-dissertation redline was additionally confirmed
  **Word-valid** (opens cleanly in Microsoft Word, no repair dialog). Re-run
  the folio sweep on the next machine that has folio checked out.
