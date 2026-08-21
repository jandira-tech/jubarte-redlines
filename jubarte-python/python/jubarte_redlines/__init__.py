# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Lossless DOCX redline engine (jubarte-redlines).

Compare two Word documents into a tracked-changes document that opens cleanly
in Microsoft Word; list, accept, or reject revisions; render DOCX to PDF.

All functions take and return ``bytes`` (whole DOCX/PDF packages) and release
the GIL while the Rust engine runs.

>>> from jubarte_redlines import compare_documents, get_revisions
>>> redline = compare_documents(original_bytes, modified_bytes, author="Reviewer")
>>> for rev in get_revisions(redline):
...     print(rev["type"], rev.get("text"))
"""

from __future__ import annotations

import json
from typing import Any

from ._native import (
    JubarteError,
    __version__,
    accept_revisions,
    compare_documents,
    docx_to_pdf,
    get_revisions_json,
    reject_revisions,
)

__all__ = [
    "JubarteError",
    "__version__",
    "accept_revisions",
    "compare_documents",
    "docx_to_pdf",
    "get_revisions",
    "get_revisions_json",
    "reject_revisions",
]


def get_revisions(docx: bytes) -> list[dict[str, Any]]:
    """List the tracked revisions in a DOCX as parsed objects.

    Each item has the same shape as the CLI ``jubarte revisions --json`` lines
    (``type``/``author``/``date``/``part``/``moveGroupId``/``isMoveSource``/
    ``formatChange``/``text``).
    """
    return json.loads(get_revisions_json(docx))
