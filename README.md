> **See every page side by side: [jandira-tech.github.io/neurotic_docx_bench](https://jandira-tech.github.io/neurotic_docx_bench/)**  
> jubarte vs Microsoft Word, docxide-pdf, LibreOffice, PyMuPDF Pro, MiniPdf, rdocx and office2pdf on 808 documents, DOCX to PDF, scored per page.

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte

**Word-faithful DOCX redlines and rendering, without Word.**

Compare two Word documents into native tracked changes, inspect or resolve
changes programmatically, apply validated edit plans, and render DOCX to PDF
or PNG — from Rust, Python, Node/browser, or the CLI.

[![CI](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jandira-tech/jubarte-redlines/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/jandira-tech/jubarte-redlines/branch/main/graph/badge.svg)](https://codecov.io/gh/jandira-tech/jubarte-redlines)
[![crates.io](https://img.shields.io/crates/v/jubarte-redlines.svg)](https://crates.io/crates/jubarte-redlines)
[![docs.rs](https://docs.rs/jubarte-redlines/badge.svg)](https://docs.rs/jubarte-redlines)
[![PyPI](https://img.shields.io/pypi/v/jubarte-redlines.svg)](https://pypi.org/project/jubarte-redlines/)
[![npm](https://img.shields.io/npm/v/jubarte-wasm.svg)](https://www.npmjs.com/package/jubarte-wasm)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue)](./Cargo.toml)
[![license](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](./LICENSE)

Jubarte is an in-process DOCX engine for applications that need Microsoft
Word-style review workflows without automating Microsoft Word or LibreOffice.

- **Compare DOCX → tracked DOCX** with native insertions, deletions, moves and
  formatting changes.
- **List, accept or reject changes** globally or selectively by change ID,
  author and kind.
- **Render DOCX → PDF / PNG** with an independent Word-oriented layout engine.
- **Inspect and edit documents safely** using paragraph IDs, source hashes and
  atomic JSON edit plans.
- **Use the same core engine everywhere**: Rust, CLI, Python and WebAssembly
  for Node/browser.
- **Keep the original package**: relationships, styles, headers/footers,
  notes, media and other DOCX parts are preserved by the comparison workflow.

> Jubarte is not affiliated with Microsoft. “Word-faithful” describes
> engineering targets measured against Microsoft Word behavior; it is not a
> claim that every DOCX will render byte-for-byte or page-for-page identically.

**Live benchmark, every page against Microsoft Word:
[jandira-tech.github.io/neurotic_docx_bench](https://jandira-tech.github.io/neurotic_docx_bench/)**
· tables: [neurotic_docx_bench RESULTS.md](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)

## Quick start

Install the CLI:

```sh
cargo install jubarte-redlines
jubarte --version
```

Prebuilt archives are also published on
[GitHub Releases](https://github.com/jandira-tech/jubarte-redlines/releases).
Check the assets attached to the release you intend to install.

### Compare two Word documents

```sh
jubarte original.docx modified.docx \
  -o redline.docx \
  --author "Reviewer"
```

`redline.docx` contains native Word tracked changes.

Without `-o`, Jubarte derives an output name beside the original document.

### Inspect and resolve tracked changes

```sh
jubarte changes redline.docx --json

# Resolve one specific change and leave the others tracked:
jubarte accept redline.docx \
  -o partially-accepted.docx \
  --id body:rev:12

# Or resolve everything:
jubarte accept redline.docx -o final.docx
jubarte reject redline.docx -o original-state.docx
```

Selection flags can be repeated and combined:

```sh
jubarte reject redline.docx \
  -o reviewed.docx \
  --author "Reviewer A" \
  --kind formatting
```

### Render DOCX to PDF or PNG

```sh
# PDF
jubarte convert redline.docx \
  -o redline.pdf \
  --revisions word \
  --compress

# Page images
jubarte convert redline.docx \
  --png \
  --dpi 120
```

No Microsoft Word or LibreOffice process is launched.

## How the workflow fits together

```mermaid
flowchart LR
    A[original.docx] --> C[Compare]
    B[modified.docx] --> C
    C --> D[redline.docx]
    D --> E[List changes]
    E --> F[Accept / Reject]
    F --> G[resolved.docx]

    H[DOCX] --> I[Inspect]
    I --> J[JSON edit plan]
    J --> K[Edit]
    K --> L[clean.docx]
    K --> M[redline.docx]

    D --> N[Render]
    G --> N
    L --> N
    M --> N
    N --> O[PDF / PNG]
```

## Install

### CLI

```sh
cargo install jubarte-redlines
```

The installed binary is named `jubarte`.

From the release after 0.10.1, the Python wheel and the npm package also
run the CLI without an install:

```sh
uvx jubarte-redlines redline a.docx b.docx -o redline.docx
npx jubarte-redlines redline a.docx b.docx -o redline.docx
```

Both runners speak the shared command set — `compare`/`redline`,
`revisions`, `changes`, `accept`, `reject`, `inspect`, `text`, `edit`,
`convert`, `capabilities` — but not the whole binary surface: the Python
wheel has no `diff`, `debug` or `self-update`, and the npm CLI, which
mirrors the Python one, renders no PNG pages and takes no `--date`
([`jubarte-wasm/cli/README.md`](jubarte-wasm/cli/README.md)).

> **Unreleased (on main, ships with the next release).** The runner
> one-liners above, plus a Markdown pipeline beside Word: `jubarte diff`
> prints the changes between any two documents, Word or Markdown, as a
> git-style patch; `jubarte convert draft.md` writes CommonMark (with
> CriticMarkup as tracked changes) to DOCX/PDF/PNG; and `jubarte edit`
> writes the redline's patch as `patch.diff`. See
> [`docs/MARKDOWN.md`](docs/MARKDOWN.md) and
> [`CHANGELOG.md`](CHANGELOG.md).

For a source checkout, the repository also contains installation scripts that
can install supplemental fonts used by the renderer:

```sh
scripts/install.sh
scripts/install.sh --fonts-only
```

On Windows, see `scripts/install.ps1`.

### Rust library

```sh
cargo add jubarte-redlines --no-default-features
```

The package is called `jubarte-redlines`; the Rust library import path is
`jubarte`.

```rust
use jubarte::{convert, document_comparer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read("original.docx")?;
    let modified = std::fs::read("modified.docx")?;

    let redline =
        document_comparer::compare_documents(&original, &modified, "Reviewer")?;
    std::fs::write("redline.docx", &redline)?;

    let pdf = convert::docx_to_pdf(&redline)?;
    std::fs::write("redline.pdf", &pdf)?;

    Ok(())
}
```

The default crate features include the CLI, fast allocator and self-update
support. Library-only consumers can disable defaults and opt into features
deliberately.

MSRV: **Rust 1.88**.

### Python

Python requires CPython 3.10 or later.

```sh
pip install jubarte-redlines
```

Low-level byte API:

```python
from pathlib import Path
from jubarte_redlines import (
    compare_documents,
    get_revisions,
    docx_to_pdf,
)

original = Path("original.docx").read_bytes()
modified = Path("modified.docx").read_bytes()

redline = compare_documents(
    original,
    modified,
    author="Reviewer",
)
Path("redline.docx").write_bytes(redline)

for revision in get_revisions(redline):
    print(revision)

Path("redline.pdf").write_bytes(
    docx_to_pdf(redline, revisions="word")
)
```

The richer `Document` API additionally supports inspection, Markdown
projection, selective change resolution, edit plans, PDF/PNG rendering and
preview.

### Node and browser

Node 18+:

```sh
npm install jubarte-wasm
```

```js
const {
  compareDocuments,
  docxToPdf,
  listChanges,
  acceptChanges,
  rejectChanges,
} = require("jubarte-wasm");

const redline = compareDocuments(originalBytes, modifiedBytes, "Reviewer");
const pdf = docxToPdf(redline, false, "word");
```

For applications that do not need the PDF renderer, use the slim package
exports:

```js
const {
  compareDocuments
} = require("jubarte-wasm/slim");
```

Browser builds are exported from `jubarte-wasm/web` and
`jubarte-wasm/web-slim`.

### MCP server

`jubarte-mcp` exposes the engine as MCP tools (read, edit as tracked
changes, render, compare, accept and reject) to coding agents. Every path
stays under `--root`.

| Host | Install |
| --- | --- |
| Claude Code | `claude mcp add --transport stdio jubarte --scope project -- uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` |
| Codex | `codex mcp add jubarte -- uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` |
| Gemini CLI | `gemini extensions install https://github.com/jandira-tech/jubarte-redlines` |

Tools, security model and config files: [docs/adoption/mcp.md](docs/adoption/mcp.md).

## CLI reference

### Compare

```text
jubarte ORIGINAL MODIFIED [OPTIONS]
```

Common options:

| Option | Purpose |
|---|---|
| `-b, --original FILE` | Original document |
| `-m, --modified FILE` | Modified document |
| `-o, --output FILE` | Output tracked-changes DOCX |
| `-a, --author NAME` | Revision author |
| `-d, --date ISO8601` | Revision timestamp |
| `--detail-threshold RATIO` | Comparison-detail tuning |
| `--mode word|powertools` | Word-oriented or classic PowerTools comparison behavior |
| `--force` | Replace an existing output |
| `-q, --quiet` | Reduce CLI output |

The default comparison date is deterministic rather than “now”, making
identical inputs reproducible unless a date is explicitly supplied.

### Inspect revisions

```sh
jubarte revisions FILE.docx
jubarte revisions FILE.docx --json

jubarte changes FILE.docx
jubarte changes FILE.docx --json
```

`changes` is intended for individually addressable review operations and
reports stable IDs while the corresponding tracked change remains present.

### Accept or reject

```sh
jubarte accept FILE.docx -o OUTPUT.docx
jubarte reject FILE.docx -o OUTPUT.docx
```

Optional selection filters:

```text
--id ID
--author NAME
--kind insertion|deletion|move|formatting
```

Filters are repeatable. Categories combine, so a change must satisfy each
category you supplied.

### Convert to PDF / PNG

```sh
jubarte convert FILE.docx [OPTIONS]
```

| Option | Purpose |
|---|---|
| `-o, --output FILE` | Output path |
| `--pdf` | Request PDF output |
| `--png` | Request page PNG output |
| `--dpi DPI` | PNG resolution; default 96 |
| `--compress` | Deflate PDF content streams |
| `--revisions conventional|word|custom` | Tracked-change visualization |
| `--revision-palette SPEC` | Custom change colors/lines |
| `--font-report FILE` | Write font-resolution JSON |
| `--report FILE` | Write page/render report |
| `--force` | Replace existing output |

Examples:

```sh
jubarte convert contract.docx
jubarte convert redline.docx --revisions word
jubarte convert redline.docx --png --dpi 144
jubarte convert contract.docx --compress --font-report fonts.json
```

A custom palette can be supplied with:

```sh
jubarte convert redline.docx \
  --revisions custom \
  --revision-palette \
  "deleted=#AA0000:strike,inserted=#0055FF:double-underline"
```

### Inspect document structure

```sh
jubarte inspect contract.docx --json
jubarte text contract.docx
```

`inspect` exposes document metadata, story/paragraph coordinates, source
hashes, runs and structural limitations.

`text` prints a Markdown-like form with addressable IDs such as:

```text
[body:p:12] **Confidentiality.** Recipient shall...
[header1:p:0] ACME CORPORATION
```

### Apply an edit plan

First inspect the source:

```sh
jubarte inspect contract.docx --json > inspect.json
```

Create `plan.json`:

```json
{
  "schema_version": 1,
  "source_sha256": "<source_sha256 from inspect>",
  "author": "Reviewer",
  "operations": [
    {
      "id": "term",
      "kind": "replace",
      "paragraph": "body:p:12",
      "find": "two years",
      "replacement": "three years"
    }
  ]
}
```

Preview it:

```sh
jubarte edit contract.docx \
  --plan plan.json \
  --out-dir review \
  --dry-run
```

Apply and render:

```sh
jubarte edit contract.docx \
  --plan plan.json \
  --out-dir review \
  --pdf \
  --png \
  --dpi 100
```

A successful edit can produce:

```text
review/
  clean.docx
  redline.docx
  patch.diff             (unreleased)
  report.jsonl
  clean.pdf
  redline.pdf
  clean-page-01.png
  redline-page-01.png
  ...
```

Supported operation kinds include:

- `replace`
- `rewrite` (unreleased: a paragraph's new text, applying only the words
  that differ)
- `insert`
- `delete`
- `comment`
- `insert_paragraph`
- `delete_paragraph`
- `format_paragraph`
- `merge_paragraphs`
- `insert_table` (unreleased: a table next to a paragraph, tracked as
  inserted rows)

Plans are atomic: stale sources, ambiguous anchors, overlapping edits or
unsupported structures refuse the plan instead of making a guessed edit.

See
[`skills/jubarte-documents/SKILL.md`](skills/jubarte-documents/SKILL.md)
and
[`examples/agents/acme-letter`](examples/agents/acme-letter)
for the full plan schema and a complete example.

### Capabilities

```sh
jubarte capabilities
jubarte capabilities --json
```

Use this when an application or agent needs to discover the exact surface
supported by the installed build.

### Self-update

```sh
jubarte self-update --check
jubarte self-update
```

Optional:

```text
-y, --yes
--version VERSION
```

See [`docs/SELF_UPDATE.md`](docs/SELF_UPDATE.md).

### Debugging DOCX behavior

```sh
jubarte debug FILE.docx --list
jubarte debug FILE.docx --check render
jubarte debug FILE.docx -c text
jubarte debug FILE.docx -c runs
jubarte debug FILE.docx -c xml

jubarte debug diff A.docx B.docx
jubarte debug diff A.docx B.docx C.docx --full
```

Debug catalogs cover package structure, fields, bookmarks, IDs, styles,
text boxes, revision changes, numbering, normalized XML/text/runs and render
information.

Use `jubarte <command> --help` for the authoritative option list for the
installed release.

## Rendering and fonts

The PDF engine reconstructs Word-oriented layout directly; it does not call
Microsoft Word or LibreOffice.

The documented rendering surface includes:

- tracked changes and comments;
- paragraph/run formatting and advanced line layout;
- CJK and RTL/complex-script text;
- installed and embedded fonts;
- merged, vertically merged, autofit and floating tables;
- DrawingML/VML graphics, text boxes, charts and SmartArt;
- raster and EMF/WMF images;
- headers, footers and floating frames;
- sections, columns, backgrounds and page borders;
- footnotes and selected fields.

Rendering fidelity still depends on fonts and on document features.

Supplemental font lookup can be configured with:

```sh
export JUBARTE_FONT_DIR=/path/to/jubarte/fonts
export JUBARTE_FONT_INDEX=/path/to/font-index.tsv
```

Set:

```sh
export JUBARTE_FONT_INDEX=off
```

to disable the persistent font index.

Default per-user font directories are documented in the installation section
of this repository.

For known Word-layout differences, see:

- [`docs/WORD_LAYOUT_RULES.md`](docs/WORD_LAYOUT_RULES.md)
- [`docs/WORD_DIFFERENCES.md`](docs/WORD_DIFFERENCES.md)
- [`docs/WORD_COMMENT_BALLOONS.md`](docs/WORD_COMMENT_BALLOONS.md)

## Comparison modes

The default comparison mode is:

```sh
--mode word
```

It targets Word-like comparison behavior.

Classic PowerTools behavior is available with:

```sh
--mode powertools
```

Choose explicitly if exact comparison semantics matter to a regression test
or downstream workflow.

## Existing revisions in edit plans

An edit plan can refuse documents that already contain tracked changes, or
resolve them according to an explicit policy.

List changes before editing:

```sh
jubarte changes source.docx --json
```

Selective resolution can be part of the plan, for example:

```json
{
  "resolve_revisions": {
    "accept": {
      "ids": ["body:rev:12"]
    },
    "reject": {
      "authors": ["Previous Reviewer"]
    }
  }
}
```

Conflicting selections are rejected instead of silently choosing a side.

## Examples

### CLI

```sh
jubarte original.docx revised.docx \
  -o redline.docx \
  --author "Legal"

jubarte changes redline.docx --json

jubarte convert redline.docx \
  -o redline.pdf \
  --revisions word
```

### Python `Document`

```python
import jubarte_redlines as jubarte

doc = jubarte.read("contract.docx")

print(doc.markdown())
snapshot = doc.inspect()
print(snapshot)

pages = doc.to_png(dpi=120)
```

### Agent-style document editing

See
[`examples/agents/acme-letter`](examples/agents/acme-letter), which includes
an edit plan, generated report and rendered redline page.

## Safety and validation

CI currently includes:

- `cargo fmt --check`
- Clippy with `-D warnings`
- all-feature Rust tests on Linux, macOS and Windows
- source-based code coverage with a line-coverage floor
- MSRV testing on Rust 1.88
- `cargo publish --dry-run`
- `cargo-deny`
- REUSE/SPDX checks
- Python binding tests

The project also maintains package/schema validation and real-Microsoft-Word
opening probes for release work because an OOXML document can be schema-valid
and still trigger Word repair behavior.

See:

- [`KNOWN_ISSUES.md`](KNOWN_ISSUES.md)
- [`VERSIONING.md`](VERSIONING.md)
- [`docs/bench_classes.md`](docs/bench_classes.md)

## Benchmarks

Jubarte maintains Word-oracle benchmarks for both document comparison and
DOCX rendering.

The benchmark methodology, corpus provenance, current numbers and historical
results belong in:

- the [neurotic_docx_bench RESULTS.md](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)
- the [page-by-page comparison site](https://jandira-tech.github.io/jubarte-redlines/)

Treat these as reproducible project-maintained measurements rather than as a
substitute for testing your own document corpus.

For important production workloads, benchmark representative documents from
your own environment before choosing an engine.

## Supported environments

| Surface | Supported/tested target |
|---|---|
| Rust / CLI | CI tests Linux, macOS and Windows |
| Rust toolchain | Rust 1.88+ |
| Python | CPython 3.10+ |
| Python release wheels | Current release workflow builds macOS and manylinux x86_64/arm64 wheels |
| Node | Node 18+ |
| Browser | WebAssembly builds |
| CLI release workflow | Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64 targets |

Release assets can vary by tag. Check the
[release page](https://github.com/jandira-tech/jubarte-redlines/releases)
before scripting a binary download.

Legacy binary `.doc` files are not supported; convert them to `.docx` first.

## Troubleshooting

| Problem | What to check |
|---|---|
| PDF layout differs from Word | Verify required fonts; inspect `--font-report`; read `docs/WORD_DIFFERENCES.md` |
| Text is missing or wraps differently | Check font resolution and embedded/system font availability |
| Edit exits with a refusal | Read `report.jsonl` / stdout for `STALE_SOURCE`, `ANCHOR_NOT_FOUND`, `AMBIGUOUS_ANCHOR`, `OVERLAPPING_EDITS`, `UNSUPPORTED_STRUCTURE`, etc. |
| `STALE_SOURCE` | Re-run `jubarte inspect` and regenerate the plan from the new `source_sha256` |
| `AMBIGUOUS_ANCHOR` | Use a paragraph ID or a more specific exact anchor |
| Document already contains revisions | List them with `jubarte changes`; explicitly accept/reject them or set an edit-plan revision policy |
| Comment is present in DOCX but missing from rendered markup | Check the documented comment-balloon limitations and `docs/WORD_COMMENT_BALLOONS.md` |
| Output page count is not exactly Word's | The renderer targets Word behavior but is independent; inspect known differences before treating page count as an invariant |
| Prebuilt archive is absent for a platform | Install with Cargo or build from source; then check the release assets for a later tag |
| Need exact package differences | Use `jubarte debug diff` and the `text`, `runs`, `xml` or `render` debug catalogs |

## Contributing

Contributions that improve Word compatibility, package validity, diagnostics,
documentation, examples or platform support are welcome.

Before opening a PR:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --no-fail-fast
cargo publish --dry-run
```

For Python binding changes, run the Python test suite from `jubarte-python`.

Please include a minimal fixture or regression test for document-behavior
changes. For Word-compatibility changes, describe what Microsoft Word does and
how the fixture demonstrates it.

For security vulnerabilities, use a private GitHub security advisory or the
private contact documented by the project rather than a public issue.

## Roadmap

Proposed areas of focus:

- [ ] Stabilize and document a `1.0` compatibility policy.
- [ ] Keep every advertised binary/wheel target release-complete.
- [ ] Expand newcomer examples for CLI, Rust, Python, Node and browser.
- [ ] Improve editable coverage for currently unsupported DOCX structures.
- [ ] Continue closing documented Word-layout differences.
- [ ] Add a polished browser demo for compare/review/render workflows.
- [ ] Make benchmark environments easier for third parties to reproduce.
- [ ] Expand community documentation and contributor onboarding.

Roadmap items are goals, not compatibility guarantees.

## Project structure

```text
src/
  lib.rs                 public Rust crate
  document_comparer.rs   compare / revisions
  comparer/              comparison engine
  changes.rs             tracked-change listing / resolution
  inspect.rs             document inspection, edit-plan anchors
  edit.rs + edit/        validated JSON edit plans
  markdown/              Markdown in and out: convert, diff, patch
  convert/               DOCX → PDF/PNG layout engine
  opc/                   DOCX package layer
  xmllinq/               shared XML model
  bin/jubarte.rs         CLI (compare, edit, convert, diff, debug, …)

jubarte-python/           PyO3 / maturin Python package
jubarte-wasm/             wasm-bindgen / npm package
jubarte-app/              Tauri desktop shell
examples/                 examples and profiling tools
tests/                    integration tests, goldens and corpus
docs/                     compatibility and engineering documentation
tools/                    validation / benchmark tooling
scripts/                  installation, release and Word-probe scripts
```

## License

`jubarte-redlines` is licensed under
[GNU Affero General Public License v3.0 only](LICENSE)
(`AGPL-3.0-only`).

Review the license requirements before embedding or deploying the software in
a product or network service.

## Links

[Engine comparison site](https://jandira-tech.github.io/neurotic_docx_bench/) ·
[jandira.tech](https://www.jandira.tech) · [arthur.law](https://arthur.law) ·
[Cicero](https://www.cicero.im) · [LinkedIn](https://linkedin.com/in/arthrod) ·
`contact@arthur.law`
- [GitHub](https://github.com/jandira-tech/jubarte-redlines)
- [crates.io](https://crates.io/crates/jubarte-redlines)
- [docs.rs](https://docs.rs/jubarte-redlines)
- [PyPI](https://pypi.org/project/jubarte-redlines/)
- [npm](https://www.npmjs.com/package/jubarte-wasm)
- [Releases](https://github.com/jandira-tech/jubarte-redlines/releases)
- [Word-layout comparison site](https://jandira-tech.github.io/jubarte-redlines/)
