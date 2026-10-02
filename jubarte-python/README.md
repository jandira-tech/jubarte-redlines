<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-redlines (Python)

Lossless DOCX **redline** engine: compare two Word documents into a
tracked-changes document that opens cleanly in Microsoft Word; list, accept,
or reject revisions; render DOCX to PDF.

Python bindings (PyO3 + maturin) for the Rust
[`jubarte-redlines`](https://github.com/jandira-tech/jubarte-redlines) engine.
Pure compute, no Word/LibreOffice dependency, no network, no temp files —
documents go in and come out as `bytes` (whole DOCX/PDF packages), revision
metadata comes back as parsed records (`get_revisions` → `list[dict]`,
`get_revisions_json` → JSON string), and the GIL is released while the
engine runs.

## Install

```sh
pip install jubarte-redlines
# or: uv add jubarte-redlines
```

Prebuilt wheels are `abi3` (one wheel per platform, CPython ≥ 3.10).

## Command line

The wheel installs a `jubarte-redlines` command; `uvx` runs it without
installing anything:

```sh
uvx jubarte-redlines redline original.docx modified.docx -o redline.docx --author Legal
uvx jubarte-redlines changes redline.docx
uvx jubarte-redlines accept redline.docx -o clean.docx --kind formatting
uvx jubarte-redlines reject redline.docx -o original-again.docx --id body:rev:12
uvx jubarte-redlines convert redline.docx --revisions word
uvx jubarte-redlines inspect contract.docx --json
uvx jubarte-redlines text contract.docx
uvx jubarte-redlines edit contract.docx --plan plan.json --out-dir review --pdf
uvx jubarte-redlines capabilities --json
uvx jubarte-redlines --help
```

`redline` is an alias of `compare`; `python -m jubarte_redlines` runs the same
commands. `revisions` and `changes` list what a document tracks (`--json` for
one object per line); `accept` and `reject` take repeatable `--id`, `--author`
and `--kind` to resolve a selection and keep the rest tracked, or everything
when none is given; `text` prints the Markdown with the `[body:p:N]` ids edit
plans use, `inspect` the paragraph/package snapshot; `edit` applies an edit plan
to `--out-dir` (clean copy, redline, `patch.diff`, `report.jsonl`, optional
PDF/PNG pages); `capabilities` reports what the build can do. Exit codes: 0
success, 1 error, 2 usage, 3 edit plan refused. Inputs are `.docx`: save a Word
97-2003 `.doc` as `.docx` first.

## Library

```python
from pathlib import Path
from jubarte_redlines import (
    compare_documents,
    get_revisions,
    accept_revisions,
    reject_revisions,
    docx_to_pdf,
)

original = Path("original.docx").read_bytes()
modified = Path("modified.docx").read_bytes()

# Word-mode compare → tracked-changes DOCX (w:ins / w:del)
redline = compare_documents(original, modified, author="Reviewer")
Path("redline.docx").write_bytes(redline)

# List revisions (same shape as the CLI `jubarte revisions --json`)
for rev in get_revisions(redline):
    print(rev["type"], rev["author"], repr(rev.get("text")))

# Accept / reject every tracked revision, package-wide
clean = accept_revisions(redline)   # ≙ modified content
base = reject_revisions(redline)    # ≙ original content

# Render to PDF (Word-style layout)
Path("redline.pdf").write_bytes(docx_to_pdf(redline))
```

`compare_documents(original, modified, author="jubarte", date=None)` stamps
revisions with a fixed epoch date by default so output is deterministic;
pass an ISO-8601 `date` to override. Errors raise
`jubarte_redlines.JubarteError`.

`read()` returns a `Document` — an immutable snapshot, path I/O done once, no
operation mutating it or writing files:

```python
from jubarte_redlines import read, EditPlan, diff

doc = read("contract.docx")
doc.markdown()    # body and every header/footer/notes story, [body:p:N] ids
doc.inspect()     # Snapshot: summary + paragraphs (ids, style, spans, limitations) + tables
doc.sha256()      # the source_sha256 guard an edit plan carries
doc.changes()     # each tracked change, with the id accept/reject and plans take
doc.accept(ids=["body:rev:12"])   # or reject(...): resolve a selection,
                                  # keep the rest tracked (no selection = all)
result = doc.render(pdf=True, png_dpi=144)  # Rendered: pdf, pngs, page report
```

`doc.compare(other, author=...)` is `compare_documents`; `doc.to_pdf()` and
`doc.to_png(dpi=...)` are the one-shot renderers.

An `EditPlan` builds the tracked-changes transaction `doc.edit(plan)` applies —
all-or-nothing: a refused plan raises `EditPlanError` (stable `code`, the
failed operation, every operation's outcome) and produces nothing.
`doc.preview(plan)` resolves and reports without applying:

```python
plan = (
    EditPlan(author="Reviewer")
    .for_document(doc)             # bind to this snapshot; STALE_SOURCE if it changed
    .replace("body:p:3", find="30 days", replacement="45 days", format={"bold": True})
    .comment("body:p:3", text="Check with the client", find="45 days")
    .insert_paragraph("body:p:3", runs=["New sentence."], like="body:p:3")
)
out = doc.edit(plan)               # EditResult: clean, redline, report, diff
```

`replace`, `insert`, `delete`, `comment` and `format_run` edit run text; `insert_paragraph`,
`delete_paragraph`, `format_paragraph`, `merge_paragraphs` and `rewrite` work on
whole paragraphs; `insert_table(paragraph, rows=[[...], ...])` adds a table and
`list_paragraphs([...], kind_of_list="decimal")` (wire kind `list`) numbers
paragraphs; `insert_footnote` adds a footnote after an anchor;
`insert_image` adds a picture paragraph;
`resolving(accept={...}, reject={...})` settles existing
tracked changes first. `plan.to_json()` is exactly what `edit --plan` reads.

`diff(old, new)` (new on `main`, first in the release after 0.10.1) shows the
changed paragraphs between two documents — or a document and Markdown text —
as `[-old-]{+new+}` marks (`format="critic"` for CriticMarkup):

```python
print(diff(read("v1.docx"), read("v2.docx")))
```

`doc.append(other)` (new on `main`) puts `other` after `doc` on a new page and
returns `Appended(document, warnings)`: images, links, headers, styles, lists
and notes come along; comments do not yet (`COMMENTS_DROPPED` in `warnings`).
`section_break="continuous"` joins on the same page and `keep_sections=True`
keeps `other`'s page setup, headers and footers.

## Also available as

- **Rust crate**: [`jubarte-redlines`](https://crates.io/crates/jubarte-redlines) (this engine, plus a CLI)
- **npm / WebAssembly**: [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm) (Node and browser builds)
- **MCP server**: `pip install 'jubarte-redlines[mcp]'` then `jubarte-mcp --root .` ([setup for Claude Code, Codex and Gemini CLI](https://github.com/jandira-tech/jubarte-redlines/blob/main/docs/adoption/mcp.md))

## License

[AGPL-3.0-only](https://github.com/jandira-tech/jubarte-redlines/blob/main/LICENSE)
© Jandira Technologies, LLC
