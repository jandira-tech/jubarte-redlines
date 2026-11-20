<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Python

`pip install jubarte-docx` (CPython 3.10 or later); the import is
`jubarte`. The wheel holds the same engine, compiled for Python, and its
own `jubarte` command.

```python
from pathlib import Path
from jubarte import compare_documents, get_revisions, docx_to_pdf

original = Path("original.docx").read_bytes()
modified = Path("modified.docx").read_bytes()
redline = compare_documents(original, modified, author="Reviewer")
Path("redline.docx").write_bytes(redline)
for revision in get_revisions(redline):
    print(revision)
Path("redline.pdf").write_bytes(docx_to_pdf(redline, revisions="word"))
```

Both blocks below are generated: the command reference from the wheel's
`--help`, the API reference from the package's own signatures and
docstrings.

## Command line

{{#include ../../../python.md:cli-python}}

## API reference

{{#include ../../../python.md:python-api}}
