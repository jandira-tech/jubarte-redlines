# jubarte-redlines

**Word-faithful DOCX tooling in pure Rust, with no Word and no LibreOffice.**
Convert `.docx` to PDF the way Microsoft Word lays it out, compare two
versions into a Word redline, accept or reject tracked changes, and apply
edits as tracked changes. It ships as a CLI, a Rust crate, a Python wheel and
an npm (WebAssembly) package.

**Live benchmark, every page against Microsoft Word:
[jandira-tech.github.io/neurotic_docx_bench](https://jandira-tech.github.io/neurotic_docx_bench/)**
· tables: [neurotic_docx_bench RESULTS.md](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)

[![CI](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml)
[![REUSE status](https://api.reuse.software/badge/github.com/jandira-tech/jubarte-redlines)](https://api.reuse.software/info/github.com/jandira-tech/jubarte-redlines)
[![codecov](https://codecov.io/gh/jandira-tech/jubarte-redlines/branch/main/graph/badge.svg)](https://codecov.io/gh/jandira-tech/jubarte-redlines)
[![crates.io](https://img.shields.io/crates/v/jubarte-redlines.svg)](https://crates.io/crates/jubarte-redlines)
[![Socket Badge](https://badge.socket.dev/cargo/package/jubarte-redlines/0.10.1)](https://badge.socket.dev/cargo/package/jubarte-redlines/0.10.1)
[![docs.rs](https://docs.rs/jubarte-redlines/badge.svg)](https://docs.rs/jubarte-redlines)
[![PyPI](https://img.shields.io/pypi/v/jubarte-redlines.svg)](https://pypi.org/project/jubarte-redlines/)
[![npm](https://img.shields.io/npm/v/jubarte-wasm.svg)](https://www.npmjs.com/package/jubarte-wasm)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](./Cargo.toml)
[![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](./LICENSE)
[![unsafe denied](https://img.shields.io/badge/unsafe-denied-success.svg)](./Cargo.toml)
[![cargo-deny](https://img.shields.io/badge/cargo--deny-checked-success.svg)](./deny.toml)
[![github](https://img.shields.io/badge/github-jandira--tech%2Fjubarte--redlines-181717?logo=github)](https://github.com/jandira-tech/jubarte-redlines)

## What it does

| Task | Command |
| --- | --- |
| **DOCX → PDF / PNG**, laid out as Word does, tracked changes painted | `jubarte convert contract.docx` |
| **Compare** two `.docx` into a redline that Word opens without repair (`w:ins` / `w:del` / moves / format changes) | `jubarte old.docx new.docx` |
| **Accept / reject** every change, or only some by id, author or kind | `jubarte accept redline.docx -o out.docx --author Ann` |
| **Edit with a plan**: get the clean copy, a redline with comments, and a report | `jubarte edit contract.docx --plan plan.json --out-dir review` |
| **Read** the paragraphs with their ids, as Markdown or JSON | `jubarte text contract.docx` |
| **Triage** a package Word refuses | `jubarte debug file.docx` |

```sh
cargo install jubarte-redlines      # CLI (binary: jubarte)
pip install jubarte-redlines        # Python ≥ 3.10
npm install jubarte-wasm            # Node ≥ 18 + browsers
```

How it scores against Microsoft Word's own output, from the
[bench tables](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)
(0–100, higher is better; a failed document scores 0):

| Benchmark | jubarte | next best |
| --- | --- | --- |
| DOCX → PDF vs Word's PDF, 3,554 docs | **77.8** mean · **81.7** median (0.9.3) | LibreOffice 26.8: 62.9 · 65.8 |
| Redline vs Word's own compare, 3,502 pairs | **72.2** · **81.7** (0.10.0) | Docxodus 12.6.5: 53.9 · 62.9 |

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
| Word-faithful PDF | Layout engine whose rules were probed against live Microsoft Word, ranked first against Word's own exports on the bench's 3,554-document set, with no Office runtime |
| Revisions in the PDF | Paints tracked changes in conventional marks, Word's own markup (per-author palette, balloon pane, change bars), or a custom palette |
| Word-valid redlines | Emits native `w:ins` / `w:del` / move / format-change markup that Word opens without repair, closest to Word's own compare on the bench's 3,502 pairs |
| Selective accept / reject | Lists every tracked change with a stable id (`body:rev:12`) and resolves any selection by id, author or kind, the way Word's Accept/Reject does; the rest stay tracked |
| Edit plans | Guarded edits anchored to `[body:p:N]` paragraph ids, applied as tracked changes plus comments, across the body, headers, footers and notes |
| Lossless package | Keeps parts, relationships, headers/footers, footnotes, styles, and media from the original |
| Library + CLI + bindings | `docx_to_pdf` / `compare_documents` in-process (Rust); `jubarte` binary for shell/CI; PyO3 wheels on PyPI; wasm-bindgen package on npm |
| Safety | **`unsafe_code = "deny"`** at the crate root — 100% safe Rust |
| Supply chain | CI runs **cargo-deny**, **REUSE** license compliance, fmt, clippy `-D warnings`, MSRV **1.88** |

## Install

**CLI** — prebuilt binaries (Linux/macOS/Windows, x86_64 + aarch64) on the
[Releases](https://github.com/jandira-tech/jubarte-redlines/releases) page,
or build from source. 0.10.1 has no Windows binary: its Windows build failed
on long fixture paths. On Windows, use `cargo install` or the 0.10.0 binary.

```sh
cargo install jubarte-redlines
# binary name is still `jubarte`
jubarte --version
```

From 0.10.0 on, `jubarte self-update` installs a newer release in place
(`--check` only looks). It is the only command that goes online, and only
when you run it ([docs/SELF_UPDATE.md](docs/SELF_UPDATE.md)). Library
builds with `default-features = false` contain no network code.

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

The first conversion that looks a font family up records where its files are
in `font-index.tsv`, beside that folder. Later runs read those files directly
and search the system, Word and cloud-font folders again only for a family the
index lacks or whose folder or file changed. `JUBARTE_FONT_INDEX` names another
index file; set it to `off` to disable the index.

**Library** (skip clap if you only need the API)

```sh
cargo add jubarte-redlines --no-default-features
```

```toml
# Cargo.toml
jubarte-redlines = { version = "0.10", default-features = false }
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

The `Document` API wraps the same engine with typed results:

```python
import jubarte_redlines as jubarte

doc = jubarte.read("contract.docx")
print(doc.markdown())                            # paragraphs with [body:p:N] ids
redline = doc.compare(jubarte.read("rev2.docx"), author="Legal")
for change in redline.changes():                 # id, kind, author, text
    print(change)
kept = redline.accept(authors=["Ann"])           # others stay tracked
result = doc.edit(plan)                          # clean copy + redline + report
pages = redline.to_png(dpi=100)
```

`python -m jubarte_redlines` runs the CLI commands from the wheel.

**JavaScript / WebAssembly** ([npm](https://www.npmjs.com/package/jubarte-wasm)
— Node ≥ 18 CJS + browser ESM)

```sh
npm install jubarte-wasm
```

```js
const { docxToPdf, compareDocuments } = require("jubarte-wasm"); // full build, ~13 MB wasm
const { compareDocuments: compareSlim } = require("jubarte-wasm/slim"); // no PDF, ~3.9 MB wasm
// browser: import init, { docxToPdf, compareDocuments } from "jubarte-wasm/web"
// ("jubarte-wasm/web-slim" drops the PDF engine)

docxToPdf(bytes);                            // conventional redline marks
docxToPdf(bytes, false, "word");             // Microsoft Word's own markup
docxToPdf(bytes, true);                      // + deflate content streams
docxToPdf(bytes, false, "custom", "deleted=#AA0000:strike");
compareDocuments(aBytes, bBytes, "Reviewer"); // → redline .docx bytes
listChanges(redline);                        // JSON: one entry per change, with its id
acceptChanges(redline, '{"authors":["Ann"]}'); // selective; acceptRevisions = all
applyEditPlan(bytes, planJson);              // clean copy + redline + report
```

Also exported: `rejectChanges`, `rejectRevisions`, `getRevisions`,
`inspectDocument`, `documentMarkdown`, `previewEditPlan`, `capabilities`,
`pdfPageCount`, `sourceSha256`.

## CLI

```text
jubarte convert contract.docx                   # DOCX → PDF, Word-style layout
jubarte convert redline.docx --revisions word   # tracked changes as Word paints them
jubarte convert contract.docx --compress --font-report fonts.json

jubarte contract.docx contract-rev2.docx
    → writes contract_v_contract-rev2.docx next to the original

jubarte -b old.docx -m new.docx -o redline.docx --author "Legal"
jubarte revisions redline.docx --json     # list tracked revisions
jubarte changes redline.docx --json       # one change per line, with its id
jubarte accept redline.docx -o final.docx # accept every revision
jubarte reject redline.docx -o clean.docx # reject every revision
jubarte accept redline.docx -o out.docx --author Ann --kind formatting
jubarte reject redline.docx -o out.docx --id body:rev:12

jubarte text contract.docx                # Markdown, a [body:p:N] id per paragraph
jubarte inspect contract.docx --json      # paragraphs, runs, stories, limitations
jubarte edit contract.docx --plan plan.json --out-dir review --pdf --png
jubarte convert contract.docx --png --dpi 100 --report pages.json
jubarte capabilities --json               # what this build can do
jubarte debug out.docx                    # why Word might refuse a package
jubarte debug diff a.docx b.docx          # what differs, element by element
jubarte self-update --check
```

A selective `accept`/`reject` resolves the changes that match every flag
given (flags repeat), as Word's Accept/Reject This Change does: either side
of a move selects both, and resolving some changes never renumbers the
rest, so ids listed before stay valid. `jubarte edit` never
modifies its source: it writes the clean copy, the redline and a report
into a new directory, and a refused plan writes nothing and exits 3.
[skills/jubarte-documents/SKILL.md](skills/jubarte-documents/SKILL.md) is
the edit-plan guide written for agents.

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
`--mode word|powertools` (Word Compare's layout, the default, or classic
PowerTools). [docs/WORD_DIFFERENCES.md](docs/WORD_DIFFERENCES.md) lists
where jubarte's redline differs from Word's and which mode gives which.

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
| `convert::docx_to_png` / `render` / `docx_render_report` | Page images, PDF + PNG in one layout pass, per-page text and page count |
| `convert::PdfOptions { compress, revisions }` | Stream compression and how tracked changes are painted |
| `convert::RevisionStyle` / `RevisionPalette` | `Conventional`, `Word`, or a `Custom` palette (`RevisionPalette::parse` takes the CLI's `kind=#RRGGBB:lines` spec) |
| `convert::pdf_page_count` | Page count of a PDF's bytes (0 if unreadable) |
| `document_comparer::compare_documents` | Base + next → redline bytes |
| `document_comparer::compare_documents_with_settings` | Same with `WmlComparerSettings` |
| `document_comparer::get_revisions` | Inspect tracked changes |
| `document_comparer::accept_revisions` / `reject_revisions` | Flatten a redline |
| `changes::list_changes` / `accept_changes` / `reject_changes` | One change at a time: ids, and a `ChangeFilter` by id, author or kind |
| `edit::apply_plan` / `apply_plan_json` / `preview_plan` | Apply an `EditPlan`: clean copy, redline and `EditReport`; `preview` applies nothing |
| `inspect::markdown` / `paragraphs` / `stories` / `summary` / `inspect_json` | Read-only views with the paragraph ids edit plans use |
| `capabilities::capabilities` | What this build can do, as data |
| `debug::report` / `debug::list` | Word-validity triage of one package, or what changed between two |
| `admission` | ZIP and XML budgets for untrusted input |

### Feature flags

| feature | default | effect |
| --- | --- | --- |
| `cli` | yes | builds the `jubarte` binary (`clap`) |
| `fast-alloc` | yes | CLI uses **mimalloc** (performance only; no semantic change) |
| `self-update` | yes | `jubarte self-update` (the CLI's only network code) |
| `perf-profile` | no | diagnostic stage timers — never for publishable wall-time claims |

**MSRV:** Rust **1.88** (edition 2024).

## How it compares

Both documents are atomized (runs, paragraph marks, table cells, …), aligned
with an LCS pass, and re-expressed as Word revision markup on the **original**
package. Default mode adds Word-visual alignment on top of the PowerTools
algorithm; `WmlComparerSettings::powertools_faithful()` / `--mode powertools`
reproduces classic PowerTools behavior.

## Benchmarks — scored against Microsoft Word

[neurotic_docx_bench](https://github.com/jandira-tech/neurotic_docx_bench)
renders each tool's output and scores it against what **Microsoft Word**
itself produces: Word's PDF export for conversion, and Word's own compare
(opened and exported by Word) for redlines. It keeps the current numbers, so
this README does not copy them:

- **Every page, side by side:** [jandira-tech.github.io/neurotic_docx_bench](https://jandira-tech.github.io/neurotic_docx_bench/)
- **Tables, one row per tool version:** [RESULTS.md](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)
  (methodology and history in `RESULTS_DETAILED.md` there)

The bench covers DOCX → PDF on clean, tracked-changes and commented
documents, docxide-pdf's own metrics, redlines against Word's compare, Word
accepting or rejecting each tool's redline, and redline speed. A redline's
first requirement stays **Word validity**: markup that Word opens without
repair, enforced by the
[validity rings](#validity-rings-word-valid-output) on every release.

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
  changes.rs             — per-change ids and selective accept / reject
  edit.rs                — edit plans → clean copy + redline + report
  inspect.rs             — paragraph / story views and the Markdown projection
  debug.rs               — `jubarte debug` package triage and diff
  comparer/              — atomize, LCS, produce, tables, notes, …
  convert/               — DOCX → PDF/PNG engine (layout, fonts, shapes, metafiles)
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
scripts/                 — install.sh/ps1, release.sh, sweeps, word-probe, bump-version.mjs
skills/                  — agent skill for reading and editing .docx with jubarte
planning/                — sample50 check/baseline for the convert gate
docs/                    — WORD_LAYOUT_RULES, WORD_DIFFERENCES, SELF_UPDATE, SPEED_REVIEW, api/ snapshots
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
> see the [bench results](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md). All trademarks remain the property of their
> respective owners.

## License

[GNU Affero General Public License v3.0](LICENSE) (**AGPL-3.0-only**).
`LICENSE` is the repository’s only project license.

Copyright (c) 2026 Jandira Technologies, LLC for its contributions.

## Find us

[Benchmark vs Word](https://jandira-tech.github.io/neurotic_docx_bench/) ·
[jandira.tech](https://www.jandira.tech) · [arthur.law](https://arthur.law) ·
[Cicero](https://www.cicero.im) · [LinkedIn](https://linkedin.com/in/arthrod) ·
`contact@arthur.law`
