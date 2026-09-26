# jubarte-redlines — #1 DOCX → PDF & #1 DOCX-vs-DOCX Comparison (Redlines) Rust Tool

*Benchmarked against Microsoft Word®'s own output across an aggregate of 3,500+
document fixtures — full tables in [RESULTS.md](RESULTS.md).*

[![CI](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml)
[![REUSE status](https://api.reuse.software/badge/github.com/jandira-tech/jubarte-redlines)](https://api.reuse.software/info/github.com/jandira-tech/jubarte-redlines)
[![codecov](https://codecov.io/gh/jandira-tech/jubarte-redlines/branch/main/graph/badge.svg)](https://codecov.io/gh/jandira-tech/jubarte-redlines)
[![crates.io](https://img.shields.io/crates/v/jubarte-redlines.svg)](https://crates.io/crates/jubarte-redlines)
[![docs.rs](https://docs.rs/jubarte-redlines/badge.svg)](https://docs.rs/jubarte-redlines)
[![PyPI](https://img.shields.io/pypi/v/jubarte-redlines.svg)](https://pypi.org/project/jubarte-redlines/)
[![npm](https://img.shields.io/npm/v/jubarte-wasm.svg)](https://www.npmjs.com/package/jubarte-wasm)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](./Cargo.toml)
[![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](./LICENSE)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](./Cargo.toml)
[![cargo-deny](https://img.shields.io/badge/cargo--deny-checked-success.svg)](./deny.toml)
[![github](https://img.shields.io/badge/github-jandira--tech%2Fjubarte--redlines-181717?logo=github)](https://github.com/jandira-tech/jubarte-redlines)

`jubarte convert` reconstructs Word's page layout in pure Rust — no Word, no
LibreOffice — and paints tracked changes the way Word does. Scored against
Word's own PDF exports, **jubarte 0.9.2 ranks #1 on both pooled corpora**:
mean Jaccard **0.647 vs 0.335** (LibreOffice 26.8, the next best) across
2,102 clean documents, and **0.574 vs 0.287** across 3,518 documents
including redlines ([RESULTS.md](RESULTS.md)).

And `jubarte redlines`: compare two `.docx` into a tracked-changes
document — native `w:ins` / `w:del` / move / format-change markup on the
original package that Word opens without repair — or list, accept, and reject
revisions in an existing one.

Both are APIs first, not just the `jubarte` CLI: `jubarte::convert` /
`jubarte::document_comparer` in **Rust**, `jubarte_redlines` on **PyPI**, and
`jubarte-wasm` on **npm** (Node + browser) — the same engine and output model
everywhere.

- **Repo:** [jandira-tech/jubarte-redlines](https://github.com/jandira-tech/jubarte-redlines)
- **crates.io:** [`jubarte-redlines`](https://crates.io/crates/jubarte-redlines)
- **docs:** [docs.rs/jubarte-redlines](https://docs.rs/jubarte-redlines)
- **PyPI:** [`jubarte-redlines`](https://pypi.org/project/jubarte-redlines/) (abi3 wheels, CPython ≥ 3.10)
- **npm:** [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm) (Node ≥ 18 + browsers, full and slim builds)
- **Maintainer:** [jandira.tech](https://www.jandira.tech) — we build legal tech.
  Jandira Technologies is the studio behind [Cicero](https://www.cicero.im) (a
  legal workbench that turns messy inputs into redlines, issue lists, and memos),
  PII redaction models for Brazilian Portuguese, and AI/contract-drafting
  benchmarks. `jubarte-redlines` falls out of that work: when a redline has to
  look like **Microsoft Word**, you need a Word-mode comparer, not a shallow
  text diff.

## Why pick it

| Need | What jubarte-redlines does |
| --- | --- |
| Word-faithful PDF | Layout engine whose rules were probed against live Microsoft Word — **#1 vs Word's own exports** (0.647 mean Jaccard, 2,102 docs; LibreOffice 0.335, docxide-pdf 0.216) — no Office runtime needed |
| Revisions in the PDF | Paints tracked changes in conventional marks, Word's own markup (per-author palette, balloon pane, change bars), or a custom palette |
| Word-valid redlines | Emits native `w:ins` / `w:del` / move / format-change markup that Word opens without repair — **#1 markup fidelity** on the 763-doc benchmark (84.5 vs docxodus 80.2) |
| Lossless package | Keeps parts, relationships, headers/footers, footnotes, styles, and media from the original |
| Library + CLI + bindings | `docx_to_pdf` / `compare_documents` in-process (Rust); `jubarte` binary for shell/CI; PyO3 wheels on PyPI; wasm-bindgen package on npm |
| Safety | **`unsafe_code = "deny"`** at the crate root — 100% safe Rust |
| Supply chain | CI runs **cargo-deny**, **REUSE** license compliance, fmt, clippy `-D warnings`, MSRV **1.88** |

## Install

**CLI** — prebuilt binaries (Linux/macOS/Windows, x86_64 + aarch64) on the
[Releases](https://github.com/jandira-tech/jubarte-redlines/releases) page,
or build from source:

```sh
cargo install jubarte-redlines
# binary name is still `jubarte`
jubarte --version
```

**Fonts for `jubarte convert`** — open fonts Word draws that macOS/Linux
lack (Roboto Condensed; Selawik standing in for Segoe UI) are installed
beside the binary instead of inside it:

```sh
scripts/install.sh               # cargo install + fonts
scripts/install.sh --fonts-only  # fonts only (Windows: scripts\install.ps1)
```

They go to `$JUBARTE_FONT_DIR`, else `~/Library/Application Support/jubarte/fonts`
(macOS), `$XDG_DATA_HOME/jubarte/fonts` or `~/.local/share/jubarte/fonts` (Linux),
`%APPDATA%\jubarte\fonts` (Windows).

**Library** (skip clap if you only need the API)

```sh
cargo add jubarte-redlines --no-default-features
```

```toml
# Cargo.toml
jubarte-redlines = { version = "0.9", default-features = false }
```

Rust import path is `jubarte::…` (library crate name); the package/repo name is
`jubarte-redlines`.

```rust,no_run
use jubarte::{convert, document_comparer};

let pdf = convert::docx_to_pdf(&std::fs::read("contract.docx")?)?;
std::fs::write("contract.pdf", &pdf)?;

let original = std::fs::read("original.docx")?;
let modified = std::fs::read("modified.docx")?;
let redline = document_comparer::compare_documents(&original, &modified, "Reviewer")?;
std::fs::write("original_v_modified.docx", &redline)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

**Python** ([PyPI](https://pypi.org/project/jubarte-redlines/) — prebuilt abi3
wheels for macOS arm64/x86_64 and manylinux x86_64/aarch64, CPython ≥ 3.10;
GIL released during compute)

```sh
pip install jubarte-redlines
```

```python
from jubarte_redlines import docx_to_pdf, compare_documents, get_revisions

pdf = docx_to_pdf(docx_bytes)                       # Word-style PDF bytes
pdf = docx_to_pdf(redline, revisions="word")        # tracked changes as Word paints them
pdf = docx_to_pdf(redline, compress=True)           # deflate the content streams

redline = compare_documents(original_bytes, modified_bytes, author="Reviewer")
revs = get_revisions(redline)        # list[dict], same shape as `jubarte revisions --json`
```

**JavaScript / WebAssembly** ([npm](https://www.npmjs.com/package/jubarte-wasm)
— Node ≥ 18 CJS + browser ESM)

```sh
npm install jubarte-wasm
```

```js
const { docxToPdf, compareDocuments } = require("jubarte-wasm"); // full build
const { compareDocuments: compareSlim } = require("jubarte-wasm/slim"); // no PDF, ~2.4 MB wasm
// browser: import init, { docxToPdf, compareDocuments } from "jubarte-wasm/web"
// ("jubarte-wasm/web-slim" drops the PDF engine)

docxToPdf(bytes);                            // conventional redline marks
docxToPdf(bytes, false, "word");             // Microsoft Word's own markup
docxToPdf(bytes, true);                      // + deflate content streams
docxToPdf(bytes, false, "custom", "deleted=#AA0000:strike");
compareDocuments(aBytes, bBytes, "Reviewer"); // → redline .docx bytes
```

## CLI

```text
jubarte convert contract.docx                   # DOCX → PDF, Word-style layout
jubarte convert redline.docx --revisions word   # tracked changes as Word paints them
jubarte convert contract.docx --compress --font-report fonts.json

jubarte contract.docx contract-rev2.docx
    → writes contract_v_contract-rev2.docx next to the original

jubarte -b old.docx -m new.docx -o redline.docx --author "Legal"
jubarte revisions redline.docx --json     # list tracked revisions
jubarte accept redline.docx -o final.docx # accept every revision
jubarte reject redline.docx -o clean.docx # reject every revision
```

`jubarte convert` paints tracked changes in the conventional redline marks by
default: deletions red and struck through, insertions blue with a double
underline, moved text green (struck through where it left, double-underlined
where it landed). `--revisions word` reproduces Microsoft Word's own markup
(what the fidelity gates below measure), and `--revisions custom
--revision-palette "deleted=#AA0000:strike,inserted=#0055FF:double-underline"`
sets your own (kinds: deleted, inserted, moved-from, moved-to; lines: strike,
double-strike, underline, double-underline, plain).

`--compress` deflates the PDF's page content streams (font programs and
image samples always deflate); `--font-report FILE` writes the per-document
font-resolution table as JSON.

Run `jubarte --help` for author/date stamping, `--detail-threshold`, and
`--powertools-faithful` (classic PowerTools-compatible mode).

### What `jubarte convert` renders

The PDF engine is a Word layout reconstruction, not a generic OOXML
renderer: every rule was measured against live Microsoft Word (synthetic
probe document → Word's own PDF export → read back the numbers) and the
rules, probes and implementing commits are written down in
[`docs/WORD_LAYOUT_RULES.md`](docs/WORD_LAYOUT_RULES.md). Coverage includes:

- **Tracked changes** — conventional marks, Word's own markup (per-author
  palette, balloon pane for comments and cell changes, change bars), or a
  custom palette.
- **Text** — Word's line breaking and device-grid baselines, justification,
  `w:spacing`/`w:ind`, widow/orphan and keep rules, `docGrid`, letter
  spacing, `w:w` horizontal scale, vertical (`tbRl`) sections, ideographic
  line breaking with kinsoku, CJK punctuation hanging, RTL (`w:bidi`
  paragraphs, `w:bidiVisual` tables, complex-script sizes and theme slots).
- **Fonts** — the metric-compatible open faces (Carlito, Liberation),
  embedded `w:embed*` fonts, Word's cloud-font cache, East Asian and
  complex-script fallback, `hhea` line metrics. Every Identity-H font
  carries `/ToUnicode`, so PDF text copies and searches correctly.
- **Tables** — autofit, `gridBefore`/`gridAfter`, merged and vertically
  merged cells, `tcBorders`/`tblCellMar`/`tblCellSpacing`, floating tables
  with page breaking, `w:hideMark` rows.
- **Graphics** — VML shapes/lines/groups and `w10:wrap`, DrawingML shapes,
  text boxes and canvases, custom preset geometry, `softEdge`/`duotone`/
  washout effects, picture crops, SmartArt, charts; EMF/WMF metafiles, BMP
  and GIF.
- **Page model** — headers/footers with their own floats and text boxes,
  `w:framePr` floating frames, continuous sections and mid-page column
  changes, page borders and backgrounds, footnote separators, `PAGE` and
  legacy `FORMCHECKBOX` fields, `w:altChunk` (HTML/MHT) content.

### Convert fidelity gate

Word-PDF Jaccard on two sets lives in this checkout, not only in docxide-pdf:

```sh
python3 scripts/test_convert_sweep.py          # unit tests (no siblings)
python3 planning/test_sample50_check.py        # unit tests (no siblings)
python3 planning/sample50_check.py             # 50-row smoke, ~3 min
python3 scripts/convert_sweep.py 76 --compare tools/convert_baseline_76.tsv
python3 scripts/convert_sweep.py 398 --compare tools/convert_baseline_398.tsv
python3 scripts/convert_sweep.py 76 --bless    # rewrite a baseline (after review)
python3 scripts/page1_delta.py ref.pdf out.pdf # first-ink / band-pitch
```

Fixtures are path-referenced: `../docxide-pdf/tests/fixtures/cases/*` and
`../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/`. The
scripts exit 2 with a path if a sibling or a listed fixture is missing (CI
without those trees still runs the unit tests). A sweep prints scores to
stdout and only rewrites `tools/` under `--bless`, so it never ratchets
against a file it just wrote. A row drop of more than 1.0 Jaccard, a mean drop
of more than 0.2, or a convert failure is a regression — fix it or name every
such row in the commit. Baselines: `tools/convert_baseline_{76,398}.tsv` and
`planning/sample50_baseline.json`.

## Library surface

| API | Purpose |
| --- | --- |
| `convert::docx_to_pdf` / `docx_to_pdf_with` / `docx_to_pdf_report` | Independent DOCX → PDF (not LibreOffice); `report` also returns the font-resolution table |
| `convert::PdfOptions { compress, revisions }` | Stream compression and how tracked changes are painted |
| `convert::RevisionStyle` / `RevisionPalette` | `Conventional`, `Word`, or a `Custom` palette (`RevisionPalette::parse` takes the CLI's `kind=#RRGGBB:lines` spec) |
| `convert::pdf_page_count` | Page count of a PDF's bytes (0 if unreadable) |
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

## Benchmarks — scored against Microsoft Word

Independent harnesses render each tool's output and score it against PDFs
exported by **Microsoft Word** itself. Numbers below are the current **0.9.x**
convert rows plus the latest **jubarte-rust** stamps (this engine's native
benchmark lane); full tables, corpus provenance, and per-version history:
[RESULTS.md](RESULTS.md).

### docx→pdf — Jaccard vs Word's own export (0–1, higher is better)

docxide-metrics pools the per-document score of every corpus each tool
converted. **jubarte 0.9.2 ranks #1 on both pools:**

| corpus pool | docs | jubarte 0.9.2 | best other tool |
| --- | ---: | --- | --- |
| clean documents | 2,102 | **0.647** mean · **0.732** median | LibreOffice 26.8 — 0.335 / 0.288 |
| clean + redlines | 3,518 | **0.574** · **0.606** | LibreOffice 26.8 — 0.287 / 0.246 |

~1.9× LibreOffice's mean and ~3× docxide-pdf 0.17.1's (0.216); the remaining
converters score ≤ 0.13, and jubarte's own 0.8.0 sits at 0.531 on its smaller
398-doc pool. `--compress` scores identically — it only deflates finished
streams. Corpora: docxide's 208-case suite, neurotic's 398 no-redline docs,
fixtures_500, English parts a+b — the second pool adds the 451- and
965-document redline corpora.

### Tracked-changes docs → PDF — neurotic harness (0–100)

428 redline documents vs Word's export:

| tool | mean | median |
| --- | ---: | ---: |
| **jubarte 0.9.1** | **71.4** | **74.3** |
| office2pdf 0.6.7 / pdfitdown 4.0.0 | 60.3 | 57.0 |
| rdocx 0.7.0 | 50.3 | 48.8 |

### Redline markup — `script_redlines` vs Word (0–100)

Each tool's redline `.docx` is rendered and scored against Word's rendering
of its own markup. On the current 763-document corpus the latest stamp
(2026-08-13) puts **jubarte-rust** at #1:

| tool | mean | median |
| --- | ---: | ---: |
| **jubarte-rust** (this engine) | **84.5** | **92.7** |
| jubarte (npm build, same engine) | 82.1 | 91.4 |
| docxodus 9.8.0 | 80.2 | 91.1 |
| best of the rest (folio 0.17.1) | 50.8 | 50.3 |

This engine holds the top three rows of that table.

Supporting harnesses on the same corpus: `roundtrip` 99.75 mean / 100.0
median (near-perfect package preservation), `accepted_changes` 84.2 — behind
docxodus 9.8.0's 88.8 there; on the earlier corpus jubarte-rust led it at
89.5 / 99.8. A redline's first contract stays **Word-validity** — markup Word
opens without repair, enforced by the
[validity rings](#validity-rings-word-valid-output) on every release.

### Speed — ms per compared pair (lower is better)

Latest stamps (2026-08-15), warm persistent-process lane over 5,000 pairs:

| lane | this engine | docxodus equivalent |
| --- | ---: | ---: |
| native inproc | **26.0** mean · **6.4** median | 25.8 · 7.9 (csharp-inproc, 4,880 pairs) |
| WebAssembly | **41.5** · **9.7** | 428.2 · 74.6 (dotnet-wasm, 5,000 pairs) |

Parity with the fastest .NET in-process lane, and ~10× faster in the browser
lane — where the npm package actually runs. (`docx-redline-js` posts 2.8 ms
on a 90-doc set but scores ~45 on markup fidelity — a different product
category.)

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
  convert/               — DOCX → PDF engine (layout, fonts, shapes, metafiles)
  opc/                   — DOCX/ZIP package layer
  xmllinq/               — the untyped DOM the comparer and converter share
  bin/jubarte.rs         — CLI
assets/fonts/            — open faces (embedded) + extra/ installed beside the binary
benches/redline.rs       — Criterion
examples/                — alloc/peak-memory profilers (mem_profile, mem_attribute, alloc_attribute)
jubarte-wasm/            — wasm-bindgen adapter → npm `jubarte-wasm` (full + slim builds)
jubarte-python/          — PyO3/maturin adapter → PyPI `jubarte-redlines`
jubarte-rust-inproc/     — long-lived stdin worker (fair speed lane)
jubarte-app/             — Tauri desktop shell (separate changelog/version)
tests/                   — integration + goldens + schema/validity oracles
tools/                   — validate-docx, parity, perf harnesses, convert baselines
scripts/                 — install.sh/ps1, sweeps, word-probe, bump-version.mjs
planning/                — sample50 check/baseline for the convert gate
docs/                    — WORD_LAYOUT_RULES, SPEED_REVIEW, BENCHMARK_M233, bench_classes
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

> **Disclaimer.** Microsoft Word® is a registered trademark of Microsoft
> Corporation. This project is not affiliated with, endorsed by, or supported
> by Microsoft. "Word-faithful" and the #1 rankings are independent
> engineering measurements scored against PDFs exported by Microsoft Word —
> see [RESULTS.md](RESULTS.md). All trademarks remain the property of their
> respective owners.

## License

[GNU Affero General Public License v3.0](LICENSE) (**AGPL-3.0-only**).
`LICENSE` is the repository’s only project license.

Copyright (c) 2026 Jandira Technologies, LLC for its contributions.

## Find us

[jandira.tech](https://www.jandira.tech) · [arthur.law](https://arthur.law) ·
[Cicero](https://www.cicero.im) · [LinkedIn](https://linkedin.com/in/arthrod) ·
`contact@arthur.law`
