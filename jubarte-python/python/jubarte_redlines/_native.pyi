# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

import os
from collections.abc import Mapping

__version__: str

class JubarteError(Exception):
    """Raised when the jubarte-redlines engine cannot process a document."""

def compare_documents(
    original: bytes,
    modified: bytes,
    author: str = "Redline",
    date: str | None = None,
    *,
    input_limits: Mapping[str, int] | None = None,
) -> bytes: ...
def accept_revisions(docx: bytes) -> bytes: ...
def reject_revisions(docx: bytes) -> bytes: ...
def get_revisions_json(docx: bytes, *, input_limits: Mapping[str, int] | None = None) -> str: ...
def list_changes_json(docx: bytes) -> str: ...
def list_comments_json(docx: bytes, author: str | None = None, latest: bool = False) -> str: ...
def changes_text(docx: bytes) -> str: ...
def comments_text(docx: bytes, author: str | None = None, latest: bool = False) -> str: ...
def tables_text(docx: bytes, *, track_changes: str | None = None) -> str: ...
def tables_view(
    docx: bytes,
    *,
    track_changes: str | None = None,
    comments: bool = True,
    dates: bool = False,
    page_markers: bool = True,
    paragraphs: str | None = None,
    source: str | None = None,
) -> tuple[str, list[str]]: ...
def read_json(
    docx: bytes,
    *,
    track_changes: str | None = None,
    paragraphs: str | None = None,
    head: int | None = None,
    tail: int | None = None,
    changed: bool = False,
    by: str | None = None,
    source: str | None = None,
    validity_file: str | None = None,
) -> str: ...
def for_edit_json(
    docx: bytes,
    *,
    paragraphs: str | None = None,
    head: int | None = None,
    tail: int | None = None,
    find: str | None = None,
    regex: str | None = None,
    ignore_case: bool = False,
    changed: bool = False,
    by: str | None = None,
    context: int | None = None,
    limit: int | None = None,
    max_chars: int | None = None,
    all: bool = False,
    cursor: str | None = None,
    accept_change: list[str] | None = None,
    reject_change: list[str] | None = None,
    existing_revisions: str | None = None,
    track: str | None = None,
    validity_file: str | None = None,
) -> str: ...
def wrote_line(path: str | os.PathLike[str], format: str, size: int, pages: int | None = None) -> str: ...
def wrote_png_line(pattern: str | os.PathLike[str], pages: int, dpi: float) -> str: ...
def refused_input(path: str | os.PathLike[str], data: bytes, word: bool) -> str | None: ...
def refused_package(path: str | os.PathLike[str], data: bytes) -> str | None: ...
def accept_changes(docx: bytes, filter_json: str) -> bytes: ...
def reject_changes(docx: bytes, filter_json: str) -> bytes: ...
def docx_to_pdf(
    docx: bytes,
    compress: bool = False,
    revisions: str = "conventional",
    revision_palette: str | None = None,
    move_comments: bool = False,
    changed_only: bool = False,
) -> bytes: ...
def docx_to_png(
    docx: bytes,
    dpi: float = 96.0,
    revisions: str = "conventional",
    revision_palette: str | None = None,
    move_comments: bool = False,
    changed_only: bool = False,
) -> list[bytes]: ...
def render(
    docx: bytes,
    pdf: bool = True,
    png_dpi: float | None = None,
    compress: bool = False,
    revisions: str = "conventional",
    revision_palette: str | None = None,
    pages: list[int] | None = None,
    move_comments: bool = False,
    changed_only: bool = False,
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
def markdown(docx: bytes, track_changes: str | None = None) -> str: ...
def read_view(
    docx: bytes,
    *,
    track_changes: str | None = None,
    comments: bool = True,
    dates: bool = False,
    page_markers: bool = True,
    paragraphs: str | None = None,
    head: int | None = None,
    tail: int | None = None,
    changed: bool = False,
    by: str | None = None,
    source: str | None = None,
) -> tuple[str, list[str]]: ...
def flag_plan(
    verb: str,
    operations: str,
    docx: bytes,
    *,
    author: str,
    date: str | None = None,
    existing: str = "auto",
) -> tuple[str, list[str]]: ...
def editing_mode_conflict(plan_json: str, docx: bytes) -> str | None: ...
def record_plan(verb: str, operations: str, docx: bytes, options: str, *, held: str | None = None) -> str: ...
def changed_view(docx: bytes, author: str, *, accepted: bool = False, source: str | None = None) -> str: ...
def resolution_text(source: bytes, resolved_json: str) -> str: ...
def edit_json(docx: bytes, plan_json: str) -> tuple[bool, bytes | None, bytes | None, str]: ...
def preview_json(docx: bytes, plan_json: str) -> tuple[bool, str]: ...
def report_jsonl(report_json: str) -> str: ...
def capabilities_json() -> str: ...
def capability_query(operation: str | None = None, field: str | None = None) -> str: ...
def capabilities_text(operation: str | None = None, field: str | None = None, matrix: bool = False) -> str: ...
def validate_run(
    docx: bytes,
    *,
    rules: list[str] | None = None,
    original: bytes | None = None,
    author: str | None = None,
    strict: bool = False,
    baseline: bytes | None = None,
) -> tuple[str, str, bool]: ...
def input_notice(docx: bytes, file: str) -> str | None: ...
def input_validity_line(docx: bytes, file: str) -> str: ...
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
def diff_unified(
    old: bytes | str, new: bytes | str, *, old_name: str = "old.docx", new_name: str = "new.docx", context: int = 3
) -> str: ...
def diff_view(
    old: bytes | str, new: bytes | str, *, format: str = "github", old_name: str = "old.docx",
    new_name: str = "new.docx", context: int = 3, accept_changes: bool = False, full_lines: bool = False
) -> str: ...
def parse_cli_json(arguments: list[str], program: str = "jubarte-redlines", supported: list[str] | None = None) -> str: ...
def wants_json(arguments: list[str]) -> bool: ...
def runtime_json_error(message: str, exit: int) -> str: ...
def redline_documents(old: bytes | str, new: bytes | str, *, author: str, date: str) -> bytes: ...
def redline_diff_json(
    docx: bytes, *, name: str, author: str, date: str, columns: int = 72, own_only: bool = False
) -> tuple[str, str]: ...
def append_json(a: bytes, b: bytes, options_json: str) -> tuple[bytes, str]: ...
def validate_json(docx: bytes) -> str: ...
def repair_json(docx: bytes) -> tuple[bytes, str]: ...
def audit_tracked_json(original: bytes, edited: bytes, author: str) -> str: ...
def markdown_stamp(text: str, reference: bytes | None = None) -> tuple[str | None, str | None]: ...
def markdown_to_docx(text: str, options_json: str | None = None, reference: bytes | None = None) -> bytes: ...
def update_fields(docx: bytes) -> tuple[bytes, str]: ...
def scrub_json(docx: bytes, options_json: str) -> bytes: ...
def audit_json(docx: bytes, rules: list[str] | None) -> str: ...

class _OutputRoot:
    def __init__(self, root: str) -> None: ...
    def read(self, path: str) -> bytes: ...
    def publish(self, files: list[tuple[str, bytes]], overwrite: bool) -> None:
        """Publish files and return None; emit cleanup failures as RuntimeWarning.

        OSError retains committed outputs, durability state and cleanup warnings.
        A publication failure remains OSError even with warnings-as-errors enabled.
        """
        ...
