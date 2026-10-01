# Typed Python Document API Implementation Plan

> **Status (2026-10-01, bd262981): PY1 done, PY2 superseded (by the
> `Snapshot` types), PY3 done (`python -m jubarte_redlines`), per the banner
> below.** Unrecorded there: the Python `diff` API — `Document.diff`,
> module-level `jubarte_redlines.diff` and `EditResult.diff` (2f2584f1,
> unreleased main).

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make everyday Python use familiar while retaining every existing byte-oriented API.

**Architecture:** Add immutable Python value objects over existing native calls; keep all DOCX interpretation in Rust. The initial facade works against the already published native wheel.

**Tech Stack:** Python 3.10+, dataclasses, PyO3 0.29, maturin, pytest-cov

---

> **Status 2026-09-26 (branch `feat/agent-adoption`):** patch 0002 applied
> and extended; see [00-ASSESSMENT.md](00-ASSESSMENT.md) §3. `Document` also
> has `sha256()`, `inspect() -> Snapshot`, `markdown()`, `edit(plan) ->
> EditResult`, `preview(plan)`, `to_png(dpi)` and `render(pdf, png_dpi) ->
> Rendered`; `EditPlan` is an immutable builder emitting the engine's wire
> schema; `EditPlanError` carries `code`, `operation`, `outcomes`;
> `capabilities()` mirrors `jubarte capabilities --json`. New in this plan:
> **`python -m jubarte_redlines`** exposes every binary command with the same
> flags, file names and exit codes (PY3 below). PY2's thin
> `Paragraph(index, text, page_break)` is superseded by the `Snapshot` types.
> Verified against the rebuilt binding (`maturin develop`): 48 tests, 98%
> lines, 160/166 branches (`pytest --cov --cov-branch`).

## Task PY3: CLI parity (added)

**Files:** `jubarte-python/python/jubarte_redlines/__main__.py`,
`jubarte-python/tests/test_cli.py`.

`python -m jubarte_redlines {inspect,text,edit,convert,compare,revisions,
accept,reject,capabilities}` accepts the `jubarte` binary's flags and writes
the same files (`clean.docx`, `redline.docx`, `report.jsonl`, `<stem>.pdf`,
`<stem>-page-NN.png`). Exit codes: 0, 1 (I/O, engine, existing output),
2 (usage), 3 (plan refused; report on stdout, nothing written). No third-party
dependency: the wheel stays dependency-free, so `argparse` rather than
`typer`.

<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

## User experience and naming

Keep the distribution `jubarte-redlines` and import `jubarte_redlines`. The alternative name `jubarte` is already occupied on PyPI as checked during this planning task. An import alias gives the short spelling without introducing a supply-chain/name migration:

```python
from pathlib import Path
import jubarte_redlines as jubarte

original = jubarte.read("contract.docx")
modified = jubarte.read("contract-v2.docx")
redline = original.compare(
    modified,
    author="Legal review",
    options=jubarte.CompareOptions(date="2026-09-26T14:30:00Z"),
)
for revision in redline.revisions():
    print(revision.kind, revision.part, revision.text)
Path("redline.docx").write_bytes(redline.to_bytes())
Path("redline.pdf").write_bytes(
    redline.to_pdf(options=jubarte.PdfOptions(revisions="word", compress=True))
)
```

These names are implemented by proposed patch 0002, not yet by the current package. The recipe in growth plan G01 uses current released functions and can be tried immediately. For API examples that should refuse overwrite, use a new output directory or `Path.open("xb")`; do not imply `write_bytes` itself is no-clobber.

## Exact surface

| Type/function | Contract |
|---|---|
| `Document.from_bytes(data: bytes)` | Immutable snapshot; rejects paths and mutable buffers; does not eagerly validate the package |
| `Document.read(path)` / `read(path)` | Explicit filesystem boundary; normal Python I/O exceptions propagate |
| `Document.to_bytes()` | Returns the held immutable bytes, no serialization or copy |
| `Document.compare(modified, *, author, options=None)` | Calls existing native compare; returns a new Document; does not change legacy existing-revision behavior |
| `Document.accept()` / `.reject()` | Return new Documents after package-wide revision processing |
| `Document.revisions()` | Tuple of frozen `Revision` with nested frozen `FormatChange`; no claim of stable revision IDs |
| `Document.to_pdf(*, options=None)` | PDF bytes; same renderer defaults as existing `docx_to_pdf` |
| `CompareOptions(date=None)` | Preserve native fixed default; explicit offset-aware string/datetime normalized to UTC without reading clock |
| `PdfOptions(compress=False, revisions="conventional", revision_palette=None)` | Typed explicit settings, reject contradictory custom-palette inputs before native work |

Keep `compare_documents`, `accept_revisions`, `reject_revisions`, `get_revisions`, `get_revisions_json`, `docx_to_pdf`, `JubarteError`, and `__version__` unchanged. Do not replace the old list-of-dictionaries return with new objects under the same function name. New `Document.revisions()` is the typed alternative.

`Document` retains bytes rather than a native DOM handle: cheap ownership reasoning, safe reuse across threads, and no hidden lifetime/resource disposal. Parsing reuse is a later measured optimization behind an explicit session object; do not add a cache with unbounded retention of private documents.

## Task PY1: implement and verify the additive facade

**Files:** create `jubarte-python/python/jubarte_redlines/{document,models}.py`, `jubarte-python/tests/{test_document,test_models}.py`, `jubarte-python/pytest.ini`; modify `jubarte-python/python/jubarte_redlines/__init__.py`. Complete code/tests: `patches/0002-python-document-api.patch`.

- [ ] Read the full patch and verify legacy exports remain identical.
- [ ] Apply tests/config first; run with coverage against the existing package. Expected failure: missing new `Document`/`CompareOptions` imports.

```sh
git apply --include='jubarte-python/tests/*' --include='jubarte-python/pytest.ini' docs/superpowers/plans/2026-09-26-jubarte-adoption/patches/0002-python-document-api.patch
uv run --with pytest --with pytest-cov --with jubarte-redlines pytest jubarte-python/tests --cov=jubarte_redlines --cov-report=term-missing --cov-branch
```

- [ ] Apply implementation files. Install the local binding into an isolated development environment after building sequentially with maturin; do not test only the registry wheel and claim a rebuilt binding passed.

```sh
git apply --exclude='jubarte-python/tests/*' --exclude='jubarte-python/pytest.ini' docs/superpowers/plans/2026-09-26-jubarte-adoption/patches/0002-python-document-api.patch
uv venv .venv-api
uv pip install --python .venv-api/bin/python maturin pytest pytest-cov
VIRTUAL_ENV="$PWD/.venv-api" .venv-api/bin/maturin develop --manifest-path jubarte-python/Cargo.toml
.venv-api/bin/python -m pytest jubarte-python/tests --cov=jubarte_redlines.document --cov=jubarte_redlines.models --cov-report=term-missing --cov-report=json:target/python-api-coverage.json --cov-branch
```

Expected: all assertions pass; target ≥90% lines/85% branches on facade policy. In this planning pass the reference facade was tested in a temporary environment against native wheel 0.9.2: 33 passed, 100% lines and branches for those two Python modules. That does not validate newly proposed Rust changes or all Python/platform versions.

- [ ] Run an installed-wheel smoke from outside the repository after `maturin build`. Check import, `Document.from_bytes` with an in-memory valid DOCX, compare/accept/reject and PDF signature. Type-check installed code/stubs with mypy and pyright on the advertised oldest/latest supported Python versions. Mark filesystem/font-dependent tests as integration tests.
- [ ] Commit facade/tests separately from native changes:

```sh
git add jubarte-python/python/jubarte_redlines jubarte-python/tests jubarte-python/pytest.ini
git commit -m "feat(python): add immutable document facade and typed revision records"
```

## Task PY2: expose inspect and renderer reports without JSON folklore

**Files:** modify `jubarte-python/src/lib.rs`, `_native.pyi`, `document.py`, `models.py`; create `jubarte-python/tests/test_reports.py`.

Define frozen `Paragraph(index: int, text: str, page_break: bool)`, `FontResolution(requested: str, physical: str, step: str, bold: bool, italic: bool, synthetic: bool)`, `PdfResult(data: bytes, page_count: int, fonts: tuple[FontResolution,...])`. Preserve existing `to_pdf()` bytes result; add `render()` for the richer report. A proposed call is `report = doc.render(options=PdfOptions(revisions="word"))`; `report.data` is PDF bytes and `report.fonts` exposes substitutions.

Use native `inspect::paragraphs`, `inspect::summary`, `convert::docx_to_pdf_report`, and `convert::pdf_page_count` rather than implement XML parsing/PDF heuristics in Python. Convert native DTOs to Python tuples/classes with explicit fields, or validate one versioned internal wire shape. `FontStep::as_str()` supplies its existing label. Engine-generated JSON is an internal transport, not a caller schema to interpret with an LLM.

- [ ] Write report fixtures exercising a nonstandard relationship name, moved/deleted text, a missing font, a malformed package and empty document. Assert field values and semantic provenance, not merely object length.
- [ ] Add native exports with `py.detach` around Rust compute and construct Python objects only while attached. Keep original immutable input bytes alive across the call. Add exact stubs for every new export in the same commit.
- [ ] Run Python coverage plus relevant Rust coverage sequentially; enforce Rust-side ≥90/85 and facade ≥90/85 on new code. Validate same document/options yields the same page count/font facts through CLI/native/Python.
- [ ] Add installed-wheel report smoke; then commit with `feat(python): expose inspection and PDF diagnostics`.

## Async and batch semantics

The native functions already release the GIL during compute. `asyncio.to_thread(doc.to_pdf)` is a scheduling recipe, not cancellable Rust computation. A cancelled await does not stop the underlying render; document that fact. For strict deadlines/isolation use a disposable worker process, with a bounded queue and per-document memory limits. Do not mark the module free-threaded merely because a single function detaches; audit PyO3's full module support and test that interpreter build separately.

Start with a recipe bounded to 2–4 concurrent operations, reading each input only when capacity is available. Each result is a per-item success/error value; a failure does not cancel siblings. Stream outputs and discard completed bytes instead of retaining every PDF/redline in a list. Avoid a public async framework dependency until observed server use justifies it. In unit tests inject a deterministic executor interface; integration tests exercise actual threads/processes with measured resource limits.

PyO3 ABI compatibility reduces wheel count, but does not prove PyPy/free-threaded/interpreter/OS support. Advertise only tested tags. Audit current PyO3 and maturin documentation before changing the `extension-module` feature; modern maturin handles the extension environment itself. [PyO3 user guide](https://pyo3.rs/), [maturin](https://www.maturin.rs/)
