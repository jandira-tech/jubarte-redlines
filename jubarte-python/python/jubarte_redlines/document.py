# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Bytes-first document operations with explicit path I/O at ``read``."""

from __future__ import annotations

import os
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path

from . import _native
from .models import (
    Change,
    ChangeKind,
    CompareOptions,
    EditOutcome,
    EditPlan,
    EditReport,
    PdfOptions,
    Rendered,
    Revision,
    Snapshot,
    _decode_changes,
    _decode_outcomes,
    _decode_render_report,
    _decode_report,
    _decode_revisions,
    _decode_snapshot,
    change_filter,
    plan_json,
)


class EditPlanError(_native.JubarteError):
    """A plan was refused; nothing was written.

    ``code`` is the stable engine code (``STALE_SOURCE``, ``ANCHOR_NOT_FOUND``,
    ``AMBIGUOUS_ANCHOR``, ``OVERLAPPING_EDITS``, ``UNSUPPORTED_STRUCTURE``,
    ``EXISTING_REVISIONS``, ``REVISION_CONFLICT``, ``UNKNOWN_CHANGE``,
    ``INVALID_PLAN``, ...), ``message`` the engine's
    detail without the code, ``operation`` the id of the operation that
    failed, and ``outcomes`` every operation's status at that point, so the
    caller can see which anchors resolved.
    """

    def __init__(self, code: str, message: str, operation: str | None, outcomes: tuple[EditOutcome, ...]) -> None:
        where = f" ({operation})" if operation else ""
        super().__init__(f"{code}{where}: {message}")
        self.code = code
        self.message = message
        self.operation = operation
        self.outcomes = outcomes

    @classmethod
    def _from_json(cls, payload: str) -> EditPlanError:
        import json

        data = json.loads(payload)
        return cls(
            code=data["code"],
            message=data["message"],
            operation=data.get("operation"),
            outcomes=_decode_outcomes(data.get("outcomes", [])),
        )


@dataclass(frozen=True, slots=True)
class EditResult:
    """Clean copy, Word redline and per-operation report of one plan."""

    clean: Document
    redline: Document
    report: EditReport


@dataclass(frozen=True, slots=True)
class Document:
    """An immutable DOCX byte snapshot.

    Loading is cheap and does not parse or validate the package. Operations
    validate it through the native engine and preserve ``JubarteError``.
    No operation modifies an input document or writes output files.
    """

    _data: bytes = field(repr=False)

    def __post_init__(self) -> None:
        if not isinstance(self._data, bytes):
            raise TypeError("Document requires bytes; use Document.read(path) for files")

    def __repr__(self) -> str:
        return f"Document(size={len(self._data)})"

    @classmethod
    def from_bytes(cls, data: bytes) -> Document:
        """Hold immutable package bytes; paths and mutable buffers are rejected."""
        return cls(data)

    @classmethod
    def read(cls, path: str | os.PathLike[str]) -> Document:
        """Read a local file; normal FileNotFoundError/PermissionError propagate."""
        return cls.from_bytes(Path(path).read_bytes())

    def to_bytes(self) -> bytes:
        """Return the immutable DOCX snapshot."""
        return self._data

    def compare(
        self,
        modified: Document,
        *,
        author: str,
        options: CompareOptions | None = None,
    ) -> Document:
        """Compare this original with ``modified``, producing tracked changes.

        Existing comparison semantics, including handling of pre-existing
        revisions, are unchanged. This is not the semantic edit transaction.
        """
        if not isinstance(modified, Document):
            raise TypeError("modified must be a Document")
        if not isinstance(author, str):
            raise TypeError("author must be a string")
        if not author.strip():
            raise ValueError("author must not be empty")
        if options is None:
            options = CompareOptions()
        elif not isinstance(options, CompareOptions):
            raise TypeError("options must be CompareOptions or None")
        return Document.from_bytes(
            _native.compare_documents(
                self._data,
                modified._data,
                author=author,
                date=options.native_date(),
            )
        )

    def accept(
        self,
        *,
        ids: Sequence[str] | None = None,
        authors: Sequence[str] | None = None,
        kinds: Sequence[ChangeKind] | None = None,
    ) -> Document:
        """Return a new document accepting tracked changes, as Word does.

        With no selection every change is accepted (Accept All). ``ids``
        (from ``changes()``), ``authors`` and ``kinds`` select changes that
        match every list given; the others stay tracked.
        """
        if ids is None and authors is None and kinds is None:
            return Document.from_bytes(_native.accept_revisions(self._data))
        return Document.from_bytes(
            _native.accept_changes(self._data, change_filter(ids, authors, kinds))
        )

    def reject(
        self,
        *,
        ids: Sequence[str] | None = None,
        authors: Sequence[str] | None = None,
        kinds: Sequence[ChangeKind] | None = None,
    ) -> Document:
        """Return a new document rejecting tracked changes (selection as in
        ``accept``; none rejects every change)."""
        if ids is None and authors is None and kinds is None:
            return Document.from_bytes(_native.reject_revisions(self._data))
        return Document.from_bytes(
            _native.reject_changes(self._data, change_filter(ids, authors, kinds))
        )

    def changes(self) -> tuple[Change, ...]:
        """Every tracked change, each with the id ``accept`` / ``reject`` select by."""
        return _decode_changes(_native.list_changes_json(self._data))

    def revisions(self) -> tuple[Revision, ...]:
        """Return immutable metadata for the revisions listed by the engine."""
        return _decode_revisions(_native.get_revisions_json(self._data))

    def to_pdf(self, *, options: PdfOptions | None = None) -> bytes:
        """Render a PDF in memory, preserving the current renderer defaults."""
        options = _pdf_options(options)
        return _native.docx_to_pdf(
            self._data,
            compress=options.compress,
            revisions=options.revisions,
            revision_palette=options.revision_palette,
        )

    # -- agent surface -----------------------------------------------------

    def sha256(self) -> str:
        """SHA-256 of the snapshot: the guard an ``EditPlan`` binds to."""
        return _native.source_sha256(self._data)

    def inspect(self) -> Snapshot:
        """Body paragraphs with ids, formatting spans and limitations, plus package facts."""
        return _decode_snapshot(_native.inspect_json(self._data))

    def markdown(self) -> str:
        """The body as Markdown with a ``[body:p:N]`` id before each paragraph."""
        return _native.markdown(self._data)

    def edit(self, plan: EditPlan | dict[str, object] | str) -> EditResult:
        """Apply a plan: clean copy, Word redline and report, or ``EditPlanError``.

        Every operation is resolved against this snapshot before anything is
        changed; a refused plan produces no documents. Comments in the plan
        are anchored in the clean copy and carried through the redline.
        """
        ok, clean, redline, payload = _native.edit_json(self._data, plan_json(plan))
        if not ok:
            raise EditPlanError._from_json(payload)
        assert clean is not None and redline is not None
        return EditResult(Document.from_bytes(clean), Document.from_bytes(redline), _decode_report(payload))

    def preview(self, plan: EditPlan | dict[str, object] | str) -> EditReport:
        """Resolve every operation and report, without producing documents."""
        ok, payload = _native.preview_json(self._data, plan_json(plan))
        if not ok:
            raise EditPlanError._from_json(payload)
        return _decode_report(payload)

    def to_png(self, *, dpi: float = 96.0, options: PdfOptions | None = None) -> tuple[bytes, ...]:
        """One PNG per page, straight from the layout (no PDF round trip)."""
        options = _pdf_options(options)
        return tuple(
            _native.docx_to_png(
                self._data,
                dpi=float(dpi),
                revisions=options.revisions,
                revision_palette=options.revision_palette,
            )
        )

    def render(self, *, pdf: bool = True, png_dpi: float | None = None, options: PdfOptions | None = None) -> Rendered:
        """One layout pass: optional PDF, optional PNG pages, and the page report."""
        options = _pdf_options(options)
        pdf_bytes, pngs, report = _native.render(
            self._data,
            pdf=pdf,
            png_dpi=None if png_dpi is None else float(png_dpi),
            compress=options.compress,
            revisions=options.revisions,
            revision_palette=options.revision_palette,
        )
        return Rendered(pdf=pdf_bytes, pngs=tuple(pngs), report=_decode_render_report(report))


def _pdf_options(options: PdfOptions | None) -> PdfOptions:
    if options is None:
        return PdfOptions()
    if not isinstance(options, PdfOptions):
        raise TypeError("options must be PdfOptions or None")
    return options


def capabilities() -> dict[str, object]:
    """What this build can do (``runtime: "python"``), as a plain dict."""
    import json

    return json.loads(_native.capabilities_json())


def read(path: str | os.PathLike[str]) -> Document:
    """Load a local DOCX snapshot; equivalent to ``Document.read(path)``."""
    return Document.read(path)
