# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

__version__: str

class JubarteError(Exception):
    """Raised when the jubarte-redlines engine cannot process a document."""

def compare_documents(
    original: bytes,
    modified: bytes,
    author: str = "jubarte",
    date: str | None = None,
) -> bytes: ...
def accept_revisions(docx: bytes) -> bytes: ...
def reject_revisions(docx: bytes) -> bytes: ...
def get_revisions_json(docx: bytes) -> str: ...
def docx_to_pdf(docx: bytes, compress: bool = False) -> bytes: ...
