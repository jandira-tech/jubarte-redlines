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
def list_changes_json(docx: bytes) -> str: ...
def list_comments_json(docx: bytes, author: str | None = None, latest: bool = False) -> str: ...
def accept_changes(docx: bytes, filter_json: str) -> bytes: ...
def reject_changes(docx: bytes, filter_json: str) -> bytes: ...
def docx_to_pdf(
    docx: bytes,
    compress: bool = False,
    revisions: str = "conventional",
    revision_palette: str | None = None,
) -> bytes: ...
def docx_to_png(
    docx: bytes,
    dpi: float = 96.0,
    revisions: str = "conventional",
    revision_palette: str | None = None,
) -> list[bytes]: ...
def render(
    docx: bytes,
    pdf: bool = True,
    png_dpi: float | None = None,
    compress: bool = False,
    revisions: str = "conventional",
    revision_palette: str | None = None,
    pages: list[int] | None = None,
) -> tuple[bytes | None, list[bytes], str]: ...
def diff_render_json(
    a: bytes,
    b: bytes,
    dpi: float = 100.0,
    overlay: bool = True,
    revisions: str = "conventional",
    revision_palette: str | None = None,
) -> tuple[str, list[bytes], list[bytes], list[bytes | None], str, str]: ...
def source_sha256(docx: bytes) -> str: ...
def inspect_json(docx: bytes) -> str: ...
def markdown(docx: bytes) -> str: ...
def edit_json(docx: bytes, plan_json: str) -> tuple[bool, bytes | None, bytes | None, str]: ...
def preview_json(docx: bytes, plan_json: str) -> tuple[bool, str]: ...
def report_jsonl(report_json: str) -> str: ...
def capabilities_json() -> str: ...
def diff_json(
    old: bytes | str,
    new: bytes | str,
    *,
    old_name: str,
    new_name: str,
    author: str,
    date: str,
    columns: int = 72,
    critic: bool = False,
) -> tuple[str, str]: ...
def redline_diff_json(
    docx: bytes, *, name: str, author: str, date: str, columns: int = 72
) -> tuple[str, str]: ...
