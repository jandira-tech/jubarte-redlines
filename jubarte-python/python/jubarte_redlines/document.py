# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Bytes-first document operations with explicit path I/O at ``read``."""

from __future__ import annotations

import json
import os
import subprocess
from collections.abc import Sequence
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal

from . import _native
from .models import (
    Change,
    Comment,
    ChangeKind,
    CompareOptions,
    Diff,
    EditOutcome,
    EditPlan,
    EditReport,
    PdfOptions,
    RenderDiff,
    Rendered,
    Revision,
    Snapshot,
    _decode_changes,
    _decode_comments,
    _decode_diff,
    _decode_outcomes,
    _decode_page_diffs,
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
    """Clean copy, Word redline, per-operation report and the patch of one
    plan: ``diff`` is what the redline tracks, by the plan's author and date."""

    clean: Document
    redline: Document
    report: EditReport
    diff: Diff


SectionBreak = Literal["next_page", "continuous", "none"]
_SECTION_BREAKS = ("next_page", "continuous", "none")


@dataclass(frozen=True, slots=True)
class Appended:
    """``Document.append``'s result: the joined document and what was not carried.

    ``warnings`` are ``CODE: message`` lines, such as
    ``COMMENTS_DROPPED: 1 comment of B was not carried``.
    """

    document: Document
    warnings: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Document:
    """An immutable DOCX byte snapshot.

    Loading is cheap and does not parse or validate the package. Operations
    validate it through the native engine and preserve ``JubarteError``.
    No operation modifies an input document or writes output files.
    """

    _data: bytes = field(repr=False)
    #: The file name it was read from (``Document.read``), used in patches.
    name: str | None = field(default=None, repr=False, compare=False)

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
        path = Path(path)
        return cls(path.read_bytes(), path.name)

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

    def comments(self, *, author: str | None = None, latest: bool = False) -> tuple[Comment, ...]:
        """Every comment with its thread and anchored text, in document part order.

        ``author`` keeps one author's comments (exact match); ``latest`` keeps
        the newest comment of each thread.
        """
        return _decode_comments(_native.list_comments_json(self._data, author, latest))

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

        With ``existing_revisions="keep"`` another party's tracked changes stay
        tracked: the clean copy is this document with the plan's edits applied
        and theirs still tracked, the redline adds the plan's edits as new
        revisions beside theirs, and ``diff`` shows the plan's edits only.
        """
        ok, clean, redline, payload = _native.edit_json(self._data, plan_json(plan))
        if not ok:
            raise EditPlanError._from_json(payload)
        assert clean is not None and redline is not None
        report = _decode_report(payload)
        diff = _decode_diff(
            _native.redline_diff_json(
                redline,
                name=self.name or "document.docx",
                author=report.author,
                date=report.date,
                own_only=report.existing_revisions == "keep",
            )
        )
        return EditResult(Document.from_bytes(clean), Document.from_bytes(redline), report, diff)

    def diff(
        self,
        other: Document | str,
        *,
        author: str | None = None,
        date: str | None = None,
        columns: int = 72,
        format: Literal["patch", "critic"] = "patch",
    ) -> Diff:
        """The changes from this document to ``other`` (a ``Document`` or
        Markdown text), as ``jubarte_redlines.diff`` gives them."""
        if not isinstance(other, (Document, str)):
            raise TypeError("other must be a Document or Markdown text")
        return diff(self, other, author=author, date=date, columns=columns, format=format)

    def preview(self, plan: EditPlan | dict[str, object] | str) -> EditReport:
        """Resolve every operation and report, without producing documents."""
        ok, payload = _native.preview_json(self._data, plan_json(plan))
        if not ok:
            raise EditPlanError._from_json(payload)
        return _decode_report(payload)

    def to_png(
        self, *, dpi: float = 96.0, options: PdfOptions | None = None, pages: Sequence[int] | None = None
    ) -> tuple[bytes, ...]:
        """One PNG per page, straight from the layout (no PDF round trip).

        ``pages`` (counted from 1, any order, repeats ignored) rasterizes only
        those pages, in ascending order, after one layout pass of the whole
        document.
        """
        if pages is not None:
            return self.render(pdf=False, png_dpi=dpi, options=options, pages=pages).pngs
        options = _pdf_options(options)
        return tuple(
            _native.docx_to_png(
                self._data,
                dpi=float(dpi),
                revisions=options.revisions,
                revision_palette=options.revision_palette,
            )
        )

    def render(
        self,
        *,
        pdf: bool = True,
        png_dpi: float | None = None,
        options: PdfOptions | None = None,
        pages: Sequence[int] | None = None,
    ) -> Rendered:
        """One layout pass: optional PDF, optional PNG pages, and the page report.

        ``pages`` (counted from 1) rasterizes only those pages; the report
        still covers every page. A page past the end raises ``JubarteError``.
        """
        options = _pdf_options(options)
        pdf_bytes, pngs, report = _native.render(
            self._data,
            pdf=pdf,
            png_dpi=None if png_dpi is None else float(png_dpi),
            compress=options.compress,
            revisions=options.revisions,
            revision_palette=options.revision_palette,
            pages=None if pages is None else _zero_based(pages),
        )
        return Rendered(pdf=pdf_bytes, pngs=tuple(pngs), report=_decode_render_report(report))

    def inspect_json(self) -> str:
        """The engine's ``inspect`` snapshot as JSON text, unchanged (``inspect`` decodes it)."""
        return _native.inspect_json(self._data)

    def append(
        self,
        other: Document,
        *,
        section_break: SectionBreak = "next_page",
        keep_sections: bool = False,
    ) -> Appended:
        """Put ``other`` after this document, carrying its parts.

        Images, links, headers, styles, lists and notes come along under ids
        that do not collide; a style this document already has (same type and
        name) keeps this document's look. ``section_break="continuous"`` or
        ``"none"`` joins on the same page; ``keep_sections`` keeps ``other``'s
        page setup, headers and footers as a section of its own. Comments are
        not carried yet: they are dropped and reported in ``warnings``.
        """
        if not isinstance(other, Document):
            raise TypeError("other must be a Document")
        if not isinstance(section_break, str):
            raise TypeError("section_break must be a string")
        if section_break not in _SECTION_BREAKS:
            raise ValueError(f"section_break must be one of {', '.join(_SECTION_BREAKS)}")
        if not isinstance(keep_sections, bool):
            raise TypeError("keep_sections must be a bool")
        options = json.dumps({"section_break": section_break, "keep_sections": keep_sections})
        data, warnings = _native.append_json(self._data, other._data, options)
        return Appended(Document.from_bytes(data), tuple(json.loads(warnings)))


def _zero_based(pages: Sequence[int]) -> list[int]:
    """Page numbers counted from 1 as the engine's zero-based indices."""
    out = []
    for page in pages:
        if isinstance(page, bool) or not isinstance(page, int) or page < 1:
            raise ValueError(f"pages are counted from 1; got {page!r}")
        out.append(page - 1)
    return out


def diff_render(
    a: Document | bytes | str | os.PathLike[str],
    b: Document | bytes | str | os.PathLike[str],
    *,
    dpi: float = 100.0,
    overlay: bool = True,
    options: PdfOptions | None = None,
) -> RenderDiff:
    """Which pages of Word documents ``a`` and ``b`` differ, pixel for
    pixel, from one layout pass each at ``dpi``.

    Each side is a ``Document``, Word ``bytes``, or a path (``str`` or
    ``os.PathLike``). ``overlay`` paints the changed pixels of each changed
    page magenta over ``b``'s page and boxes them. ``options`` sets the
    revision style both sides are painted with (``compress`` is ignored).
    """
    options = _pdf_options(options)
    pages, a_pngs, b_pngs, overlays, a_report, b_report = _native.diff_render_json(
        _word_bytes(a),
        _word_bytes(b),
        dpi=float(dpi),
        overlay=overlay,
        revisions=options.revisions,
        revision_palette=options.revision_palette,
    )
    return RenderDiff(
        pages=_decode_page_diffs(pages),
        a=tuple(a_pngs),
        b=tuple(b_pngs),
        overlays=tuple(overlays),
        a_report=_decode_render_report(a_report),
        b_report=_decode_render_report(b_report),
    )


def _word_bytes(side: object) -> bytes:
    """A ``diff_render`` side as Word bytes."""
    if isinstance(side, Document):
        return side._data
    if isinstance(side, bytes):
        return side
    if isinstance(side, (str, os.PathLike)):
        return Path(side).read_bytes()
    raise TypeError("each side must be a Document, Word bytes or a path")


def _pdf_options(options: PdfOptions | None) -> PdfOptions:
    if options is None:
        return PdfOptions()
    if not isinstance(options, PdfOptions):
        raise TypeError("options must be PdfOptions or None")
    return options


def diff(
    old: Document | bytes | str | os.PathLike[str],
    new: Document | bytes | str | os.PathLike[str],
    *,
    author: str | None = None,
    date: str | None = None,
    columns: int = 72,
    format: Literal["patch", "critic"] = "patch",
) -> Diff:
    """The changes from ``old`` to ``new``: the changed paragraphs, each at
    its ``body:p:N`` id in a Word document or ``line:N`` in Markdown, with
    ``[-old-]{+new+}`` changes and CriticMarkup comments.

    Each side is a ``Document`` or Word ``bytes``, Markdown text (``str``),
    or a path (``.md``/``.markdown`` read as Markdown, anything else as
    Word). ``author`` and ``date`` own the changes, shown once in the
    header [default: ``git config user.name``, else Redline; now].
    ``columns`` wraps the lines (0 does not). ``format="critic"`` gives the
    whole document as CriticMarkup instead, as ``jubarte diff --format
    critic`` does.
    """
    if format not in ("patch", "critic"):
        raise ValueError("format must be patch or critic")
    if not isinstance(columns, int) or columns < 0:
        raise ValueError("columns must be a nonnegative integer")
    (old_side, old_name), (new_side, new_name) = _side(old, "old"), _side(new, "new")
    return _decode_diff(
        _native.diff_json(
            old_side,
            new_side,
            old_name=old_name,
            new_name=new_name,
            author=author if author is not None else _default_author(),
            date=date if date is not None else datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            columns=columns,
            critic=format == "critic",
        )
    )


def _side(side: object, default: str) -> tuple[bytes | str, str]:
    """A diff side as the engine takes it (Word bytes or Markdown text), and its name."""
    if isinstance(side, Document):
        return side._data, side.name or f"{default}.docx"
    if isinstance(side, bytes):
        return side, f"{default}.docx"
    if isinstance(side, str):
        return side, f"{default}.md"
    if isinstance(side, os.PathLike):
        path = Path(side)
        if path.suffix.lower() in (".md", ".markdown"):
            return path.read_text(encoding="utf-8"), path.name
        return path.read_bytes(), path.name
    raise TypeError("each side must be a Document, bytes, Markdown text or a path")


def _default_author() -> str:
    """``git config user.name``, else Redline: as ``jubarte diff``."""
    try:
        out = subprocess.run(
            ["git", "config", "user.name"], capture_output=True, text=True, check=False, timeout=5
        )
    except (OSError, subprocess.SubprocessError):
        return "Redline"
    name = out.stdout.strip()
    return name if out.returncode == 0 and name else "Redline"


def capabilities() -> dict[str, object]:
    """What this build can do (``runtime: "python"``), as a plain dict."""
    import json

    return json.loads(_native.capabilities_json())


def read(path: str | os.PathLike[str]) -> Document:
    """Load a local DOCX snapshot; equivalent to ``Document.read(path)``."""
    return Document.read(path)
