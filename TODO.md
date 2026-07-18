# TODO — jubarte-redlines

Findings from the 2026-07-17 folio×jubarte demo session (all measured on this
machine; the demo corpus lives in the folio playground,
`packages/playground/public/redline3/`).

## 1. wasm32 memory ceiling on run-fragmented documents (HIGH)

A real 276k-run dissertation pair (9.8 MB docx, character-shredded runs)
needs **~11.9 GB peak memory footprint** (6.6 GB max RSS) to compare
natively. wasm32's 4 GiB address space makes ANY real diff of such documents
abort (`unreachable`; the allocator dies before the panic hook can run —
even a single-word edit OOMs, while an identical-pair compare passes because
it short-circuits the alignment allocations). Feeds `WASM_PERF_PLAN.md` /
the marginal-gain loop:

- [ ] Add a peak-memory budget per corpus size class to the bench, with an
  explicit wasm32-viability line.
- [ ] Profile the alignment allocations on run-fragmented inputs (the cost is
  triggered by ANY diff, independent of edit count).
- [ ] Document the product stance: beyond-ceiling documents take the
  native/server path (the deployed demo already precomputes them).

## 2. Build recipe must ride the engine pin (MEDIUM)

The browser wasm is now built with `RUSTFLAGS="-C link-arg=-zstack-size=8388608"`
(8 MB shadow stack; the 1 MB default was the first OOM suspect and remains a
real risk on deep recursion) on top of the wasm-pack `-O3` profile. "Same
engine commit" no longer identifies the artifact:

- [ ] Extend the bench's `ENGINE_COMMIT` pin mandate to record the FULL build
  recipe (rustflags, wasm-pack target/profile, wasm-opt flags).
- [ ] Consider committing the stack-size flag into the adapter crate's
  `[package.metadata.wasm-pack]` config in neurotic_docx_bench so it cannot
  be forgotten.

## 3. Cross-engine reference behavior (context, no action)

On the same demo corpus where the TS lossless port mis-marks rows/paragraph
marks (see jubarte-first/TODO.md), this engine's output passes folio's
engine-independent self-check on every pair, including the dissertation
(accept ≡ revised, reject ≡ base, verified through folio's reviewer). Keep it
that way: any future emission change should re-run the folio judge sweep in
`folio/packages/playground/debug-verify-buffer.mjs`.
