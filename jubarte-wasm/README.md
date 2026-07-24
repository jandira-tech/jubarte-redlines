# jubarte-wasm

`wasm-bindgen` package for the canonical **jubarte-redlines**
`compare_documents` implementation (Word-mode redline).

This crate lives **inside** the jubarte-redlines checkout (`jubarte-redlines/jubarte-wasm`)
and depends on it via `path = ".."` — it is the single wasm binding, not a fork.
`neurotic_docx_bench` consumes it through a **gitignored symlink**
(`src/neurotic_docx_bench/utils/jubarte/jubarte-wasm` → here); regenerate the crate
and its `pkg/` output HERE so there is exactly one source to rebuild.

## Toolchain (why this one)

| Tool | Role |
|------|------|
| **[wasm-pack](https://rustwasm.github.io/wasm-pack/)** | De-facto Rust→npm WASM creator (`cdylib` + wasm-bindgen glue) |
| **[Binaryen `wasm-opt -O3`](https://github.com/WebAssembly/binaryen)** | Post-link optimizer — maximizes runtime speed (not `-Oz` size) |
| **target `wasm32-unknown-unknown`** | Node / browser host (same class as Docxodus npm WASM) |

wasm-pack + wasm-opt is still the market default for shipping high-performance
Rust compute into Node/V8. WASI+Wasmtime can be faster as a *native* host, but
it is not drop-in for the Docxodus-style “import in Node” path this bench uses.

## Build

```bash
# once: rustup target add wasm32-unknown-unknown
# once: cargo install wasm-pack
# once: brew install binaryen   # wasm-opt
wasm-pack build --target nodejs --release
# → pkg/  (jubarte_wasm.js + jubarte_wasm_bg.wasm)
```

Do NOT pass `RUSTFLAGS` on the command line: the full build recipe (`+simd128`
and the 8 MB shadow stack `-zstack-size=8388608`) is pinned in `.cargo/config.toml`,
and an env-var `RUSTFLAGS` would REPLACE — not merge — those flags. A bare
`wasm-pack build` is the only invocation that guarantees both land. wasm-opt flags
(`-O3`, SIMD, bulk-memory, …) live in `Cargo.toml [package.metadata.wasm-pack.profile.*]`;
together those two files are the complete, pinnable recipe.

The Cargo path dependency `jubarte = { path = ".." }` resolves to the enclosing
jubarte-redlines crate. After building, run the speed lane in the bench; native and
WASM fidelity scores must match for the same source commit before publishing a
speed comparison.

## Smoke

```js
import init, { compareDocuments, initPanicHook } from "./pkg/jubarte_wasm.js";
await init();
initPanicHook();
const out = compareDocuments(baseU8, nextU8, "jubarte-wasm");
```
