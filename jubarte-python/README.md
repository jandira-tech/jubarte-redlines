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
every function takes and returns `bytes`, and the GIL is released while the
engine runs.

## Install

```sh
pip install jubarte-redlines
# or: uv add jubarte-redlines
```

Prebuilt wheels are `abi3` (one wheel per platform, CPython ≥ 3.10).

## Usage

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

## Also available as

- **Rust crate**: [`jubarte-redlines`](https://crates.io/crates/jubarte-redlines) (this engine, plus a CLI)
- **npm / WebAssembly**: [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm) (Node and browser builds)

## License

[AGPL-3.0-only](https://github.com/jandira-tech/jubarte-redlines/blob/main/LICENSE)
© Jandira Technologies, LLC
