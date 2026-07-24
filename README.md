# jubarte-redlines

[![CI](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml)
[![REUSE status](https://api.reuse.software/badge/github.com/jandira-tech/jubarte-redlines)](https://api.reuse.software/info/github.com/jandira-tech/jubarte-redlines)
[![codecov](https://codecov.io/gh/jandira-tech/jubarte-redlines/branch/main/graph/badge.svg)](https://codecov.io/gh/jandira-tech/jubarte-redlines)
[![crates.io](https://img.shields.io/crates/v/jubarte-redlines.svg)](https://crates.io/crates/jubarte-redlines)
[![docs.rs](https://docs.rs/jubarte-redlines/badge.svg)](https://docs.rs/jubarte-redlines)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](./Cargo.toml)
[![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](./LICENSE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](./Cargo.toml)
[![cargo-deny](https://img.shields.io/badge/cargo--deny-checked-success.svg)](./deny.toml)
[![github](https://img.shields.io/badge/github-jandira--tech%2Fjubarte--redlines-181717?logo=github)](https://github.com/jandira-tech/jubarte-redlines)

Lossless **DOCX redline** engine for Rust. Compare two Word documents and get a
tracked-changes `.docx` that opens cleanly in Microsoft Word — insertions,
deletions, moves, and format changes on top of the original package.

Also list, accept, or reject tracked revisions.

- **Repo:** [jandira-tech/jubarte-redlines](https://github.com/jandira-tech/jubarte-redlines)
- **crates.io:** [`jubarte-redlines`](https://crates.io/crates/jubarte-redlines)
- **docs:** [docs.rs/jubarte-redlines](https://docs.rs/jubarte-redlines)
- **Maintainer:** [jandira.tech](https://www.jandira.tech) — we build legal tech.
  Jandira Technologies is the studio behind [Cicero](https://www.cicero.im) (a
  legal workbench that turns messy inputs into redlines, issue lists, and memos),
  PII redaction models for Brazilian Portuguese, and AI/contract-drafting
  benchmarks. `jubarte-redlines` falls out of that work: when a redline has to
  look like **Microsoft Word**, you need a Word-mode comparer, not a shallow
  text diff.

> Independent engineering measurements against a Word oracle. Not affiliated
> with Microsoft. Trademarks remain their owners’.

## Why this crate

| Need | What jubarte-redlines does |
| --- | --- |
| Word-valid output | Produces native `w:ins` / `w:del` / move / format-change markup that Word opens without repair |
| Lossless package | Keeps parts, relationships, headers/footers, footnotes, styles, and media from the original |
| Library + CLI | `compare_documents` in-process; `jubarte` binary for shell/CI |
| Safety | `#![forbid]`-style policy: **`unsafe_code = "deny"`** at the crate root — 100% safe Rust today |
| Supply chain | CI runs **cargo-deny**, **REUSE** license compliance, fmt, clippy `-D warnings`, MSRV **1.88** |

## Install

**CLI**

```sh
cargo install jubarte-redlines
# binary name is still `jubarte`
jubarte --version
```

**Library** (skip clap if you only need the API)

```sh
cargo add jubarte-redlines --no-default-features
```

```toml
# Cargo.toml
jubarte-redlines = { version = "0.5", default-features = false }
```

Rust import path is `jubarte::…` (library crate name); the package/repo name is
`jubarte-redlines`.

```rust,no_run
use jubarte::document_comparer;

let original = std::fs::read("original.docx")?;
let modified = std::fs::read("modified.docx")?;
let redline = document_comparer::compare_documents(&original, &modified, "Reviewer")?;
std::fs::write("original_v_modified.docx", &redline)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI

```text
jubarte contract.docx contract-rev2.docx
    → writes contract_v_contract-rev2.docx next to the original

jubarte -b old.docx -m new.docx -o redline.docx --author "Legal"
jubarte revisions redline.docx --json     # list tracked revisions
jubarte accept redline.docx -o final.docx # accept every revision
jubarte reject redline.docx -o clean.docx # reject every revision
```

Run `jubarte --help` for author/date stamping, `--detail-threshold`, and
`--powertools-faithful` (classic PowerTools-compatible mode).

## Library surface

| API | Purpose |
| --- | --- |
| `document_comparer::compare_documents` | Base + next → redline bytes |
| `document_comparer::compare_documents_with_settings` | Same with `WmlComparerSettings` |
| `document_comparer::get_revisions` | Inspect tracked changes |
| `document_comparer::accept_revisions` / `reject_revisions` | Flatten a redline |

### Feature flags

| feature | default | effect |
| --- | --- | --- |
| `cli` | yes | builds the `jubarte` binary (`clap`) |
| `fast-alloc` | yes | CLI uses **mimalloc** (performance only; no semantic change) |
| `perf-profile` | no | diagnostic stage timers — never for publishable wall-time claims |

**MSRV:** Rust **1.88** (edition 2024).

## How it compares

Both documents are atomized (runs, paragraph marks, table cells, …), aligned
with an LCS pass, and re-expressed as Word revision markup on the **original**
package. Default mode adds Word-visual alignment on top of the PowerTools
algorithm; `WmlComparerSettings::powertools_faithful()` / `--powertools-faithful`
reproduces classic PowerTools behavior.

## Benchmarks (Word oracle + large-N speed)

Independent measurements on
[neurotic_docx_bench](https://github.com/jandira-tech/neurotic_docx_bench)
(LibreOffice-rendered PDFs vs a committed **Microsoft Word** redline oracle).
Higher fidelity = closer to Word. Full tables: that repo’s `RESULTS.md` /
`docs/SPEED.md`. Snapshot source commit: **`7b21276`**.

### Fidelity — `script_redlines` (0–100 vs Word)

| vendor | mean | median | n | note |
| --- | ---: | ---: | ---: | --- |
| **jubarte-rust** (this engine, CLI) | **92.21** | **99.92** | 164 | pin `jubarte-rust@cbbcefb724a7` |
| **jubarte-wasm** (same source, wasm-bindgen) | **92.21** | **99.92** | 164 | **identical** per-doc scores vs native |
| jubarte final-lossless (best pin) | 83.63 | 88.96 | 164 | older TS/port family |
| docxodus 7.0.0 | 58.75 | 55.03 | 205 | |
| superdoc-redlines 0.2.0 | 57.63 | 55.90 | 192 | |
| superdoc 1.19.2 | 57.19 | 55.60 | 182 | |
| folio 0.3.1 | 55.31 | 53.75 | 205 | |
| redlines 0.6.1 | 51.28 | 51.77 | 200 | pure-text differ |
| docx-redline-js (migration) | 50.53 | 50.26 | 161 | |

Same pin, other non-visual benches:

| benchmark | mean | median | n |
| --- | ---: | ---: | ---: |
| `accepted_changes` | 89.45 | 99.75 | 164 |
| `roundtrip` | 99.17 | 100.00 | 166 |

**Native ≡ WASM:** 164/164 documents same score when both consumers are built
from the same source commit. A speed win does not excuse a fidelity gap.

### Speed — large-N redline generation (1000 fixtures → 5000 pairs)

Same-run pack (seed 42, warmup 50, reps 1, **zero failures**):
[`jubarte-wasm-inproc-7b21276-…`](https://github.com/jandira-tech/neurotic_docx_bench)
under `results/redline_speed_bench/`.

| tool | mode | median ms | mean ms | p95 | p99 | /s | fail |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| **jubarte-rust-inproc** | warm native (fair algorithm) | **9.15** | **38.19** | 164.5 | 266.4 | **26.2** | 0 |
| jubarte-rust | CLI spawn + I/O | 13.37 | 44.95 | 181.8 | 285.3 | 22.2 | 0 |
| jubarte-wasm | warm V8 WASM | 14.99 | 63.33 | 278.2 | 419.8 | 15.8 | 0 |

Peer context (historical best large-N rows in the bench exporter): warm
**docxodus-csharp-inproc** ~9.4 ms median (algorithm-close, far behind on
fidelity); npm **docxodus** WASM is much slower with a heavy tail; cold
**docxodus-csharp** CLI is dominated by .NET startup (~200 ms) and is **not**
algorithm cost.

**Thesis line:** more Word-faithful **and** competitive on warm-process speed;
WASM stays within ~1.6× of warm native at the median with **no** fidelity drift.

### In-repo microbenches

Criterion suites over representative pairs live in
[`benches/redline.rs`](benches/redline.rs):

```sh
cargo bench --bench redline
cargo bench --bench redline -- --baseline m233_head   # optional baseline
```

See also [`docs/SPEED_REVIEW.md`](docs/SPEED_REVIEW.md) and
[`WASM_PERF_PLAN.md`](WASM_PERF_PLAN.md).

## Safety, coverage, and supply chain

| check | how |
| --- | --- |
| **No `unsafe`** | `[lints.rust] unsafe_code = "deny"` in `Cargo.toml` — the library and CLI are safe Rust |
| **Clippy** | `cargo clippy --all-targets --all-features -- -D warnings` (CI) |
| **fmt** | `cargo fmt --check` (CI) |
| **Tests** | `cargo test --all-features` on Linux, macOS, Windows (CI) |
| **MSRV** | `cargo check` on **1.88** (CI) |
| **cargo-deny** | advisories + license allowlist ([`deny.toml`](deny.toml)) |
| **REUSE** | SPDX headers + [`REUSE.toml`](REUSE.toml) (CI workflow) |
| **Coverage** | Codecov on `main` (badge above); local: `cargo llvm-cov --all-features` |
| **Publish dry-run** | `cargo publish --dry-run` (CI) |

Security reports: prefer a private channel to `contact@arthur.law` or a GitHub
security advisory on this repository. Do not open public issues for unfixed
vulnerabilities.

## Validity rings (Word-valid output)

| Ring | What | When |
| --- | --- | --- |
| **1** | Rust-native package invariants (`tests/common/validity.rs`) | every `cargo test` |
| **1½** | Schema-consistency oracle (`tests/schema_consistency.rs`) | every `cargo test` |
| **2** | OpenXmlValidator sweep + ratchet (`tools/validate-docx`, `tools/validity_baseline.tsv`) | before **bench-pin promotion** |
| **3** | Real Microsoft Word open probe (`scripts/word-open-probe.sh`) | before **release / pin promotion** (macOS) |

A bench pin without `validator: baseline-clean` and `word-probe: N/N OPENED` is
**not promotable**. See [`VERSIONING.md`](VERSIONING.md) and
[`docs/bench_classes.md`](docs/bench_classes.md).

## Layout

```text
src/
  lib.rs                 — public crate root (`jubarte`)
  document_comparer.rs   — compare / accept / reject / get_revisions
  comparer/              — atomize, LCS, produce, tables, notes, …
  bin/jubarte.rs         — CLI
benches/redline.rs       — Criterion
jubarte-wasm/            — wasm-bindgen adapter (bench consumer)
jubarte-rust-inproc/     — long-lived stdin worker (fair speed lane)
tests/                   — integration + goldens
tools/                   — validate-docx, parity, perf harnesses
```

## Known issues

Open engine defects and unresolved Word-behavior conflicts:
[KNOWN_ISSUES.md](KNOWN_ISSUES.md). Covering tests are `#[ignore]` and run with
`cargo test -- --ignored`.

## Provenance & attribution

The comparison engine is historically informed by the `WmlComparer` /
`DocumentComparer` path from [Docxodus](https://github.com/JSv4/Docxodus), itself
a fork of Microsoft’s
[Open-Xml-PowerTools](https://github.com/OfficeDev/Open-Xml-PowerTools).
Original MIT texts are preserved as attribution records — see
[`LICENSES.md`](LICENSES.md). They do **not** relicense this repository.

## License

[GNU Affero General Public License v3.0](LICENSE) (**AGPL-3.0-only**).
`LICENSE` is the repository’s only project license.

Copyright (c) 2026 Jandira Technologies, LLC for its contributions.

## Find us

[jandira.tech](https://www.jandira.tech) · [arthur.law](https://arthur.law) ·
[Cicero](https://www.cicero.im) · [LinkedIn](https://linkedin.com/in/arthrod) ·
`contact@arthur.law`
