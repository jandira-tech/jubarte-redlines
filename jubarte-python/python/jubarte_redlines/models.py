# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Immutable options and revision records for the Python document facade."""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field, replace
from datetime import datetime, timezone
from typing import Literal, TypedDict

RevisionKind = Literal["Inserted", "Deleted", "Moved", "FormatChanged"]
RevisionStyle = Literal["conventional", "word", "custom"]


@dataclass(frozen=True, slots=True)
class CompareOptions:
    """Comparison metadata; ``None`` preserves the engine's fixed timestamp.

    Explicit timestamps must include an offset. They are normalized to UTC.
    Low-level ``compare_documents`` retains its existing permissive signature.
    """

    date: str | datetime | None = None

    def __post_init__(self) -> None:
        if self.date is None:
            return
        if isinstance(self.date, datetime):
            value = self.date
        elif isinstance(self.date, str):
            try:
                value = datetime.fromisoformat(self.date.replace("Z", "+00:00"))
            except ValueError as exc:
                raise ValueError("date must be an ISO-8601 timestamp with an offset") from exc
        else:
            raise TypeError("date must be a datetime, an ISO-8601 string, or None")
        if value.utcoffset() is None:
            raise ValueError("date must include a UTC offset")
        normalized = value.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")
        object.__setattr__(self, "date", normalized)

    def native_date(self) -> str | None:
        """Return the timestamp accepted by the existing native function."""
        # __post_init__ turns every datetime into str.
        assert self.date is None or isinstance(self.date, str)
        return self.date


@dataclass(frozen=True, slots=True)
class PdfOptions:
    """PDF options with the same defaults as the current byte API."""

    compress: bool = False
    revisions: RevisionStyle = "conventional"
    revision_palette: str | None = None

    def __post_init__(self) -> None:
        if not isinstance(self.compress, bool):
            raise TypeError("compress must be a bool")
        if self.revisions not in ("conventional", "word", "custom"):
            raise ValueError("revisions must be conventional, word, or custom")
        if self.revision_palette is not None and not isinstance(self.revision_palette, str):
            raise TypeError("revision_palette must be a string or None")
        if self.revisions == "custom" and self.revision_palette is None:
            raise ValueError("custom revisions require revision_palette")
        if self.revisions != "custom" and self.revision_palette is not None:
            raise ValueError("revision_palette requires custom revisions")


@dataclass(frozen=True, slots=True)
class FormatChange:
    """Names of run or paragraph properties changed by a revision."""

    changed_properties: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Revision:
    """A read-only revision record; no implied stable edit identifier.

    ``date`` is preserved as document text instead of rejecting old documents
    whose producers emitted a nonstandard timestamp. Missing author/date/text
    remain empty strings, matching the existing JSON API.
    """

    kind: RevisionKind
    author: str
    date: str
    part: str
    text: str
    move_group_id: int | None
    is_move_source: bool | None
    format_change: FormatChange | None


class _FormatChangeRecord(TypedDict):
    changedProperties: list[str]


class _RevisionRecord(TypedDict):
    type: RevisionKind
    author: str
    date: str
    part: str
    text: str
    moveGroupId: int | None
    isMoveSource: bool | None
    formatChange: _FormatChangeRecord | None


def _decode_revisions(payload: str) -> tuple[Revision, ...]:
    """Project this version's native JSON records into immutable objects.

    This is an internal same-version adapter, not a parser for external JSON.
    Unexpected native schemas propagate an error instead of hiding it.
    """
    records: list[_RevisionRecord] = json.loads(payload)
    return tuple(
        Revision(
            kind=row["type"],
            author=row["author"],
            date=row["date"],
            part=row["part"],
            text=row["text"],
            move_group_id=row["moveGroupId"],
            is_move_source=row["isMoveSource"],
            format_change=(
                FormatChange(tuple(row["formatChange"]["changedProperties"]))
                if row["formatChange"] is not None
                else None
            ),
        )
        for row in records
    )


# ---------------------------------------------------------------------------
# Inspection snapshot (jubarte inspect --json)
# ---------------------------------------------------------------------------


@dataclass(frozen=True, slots=True)
class Span:
    """Direct run formatting over ``Paragraph.text`` in char offsets."""

    start: int
    end: int
    bold: bool
    italic: bool
    underline: bool
    highlight: str | None


@dataclass(frozen=True, slots=True)
class Paragraph:
    """One body paragraph; ``index``/``id`` are valid for this snapshot only."""

    index: int
    id: str
    text: str
    style: str | None
    numbered: bool
    in_table: bool
    page_break: bool
    runs: tuple[Span, ...]
    limitations: tuple[str, ...]


@dataclass(frozen=True, slots=True)
class Summary:
    """Package facts (XML facts, not rendered-page facts)."""

    paragraphs: int
    tables: int
    fields: int
    sections: int
    comments: int
    revisions: int
    footnotes: int
    endnotes: int
    headers: int
    footers: int
    images: int
    list_numbering: bool
    track_changes: bool


@dataclass(frozen=True, slots=True)
class Snapshot:
    """What ``Document.inspect()`` returns; the coordinates an ``EditPlan`` uses."""

    schema_version: int
    source_sha256: str
    summary: Summary
    paragraphs: tuple[Paragraph, ...]

    def paragraph(self, id_or_index: str | int) -> Paragraph:
        """The paragraph with this id (``body:p:N``) or index."""
        for p in self.paragraphs:
            if p.id == id_or_index or p.index == id_or_index:
                return p
        raise LookupError(f"no paragraph {id_or_index!r} in this snapshot")

    def unique(self, *, starts_with: str | None = None, contains: str | None = None) -> Paragraph:
        """Exactly one paragraph matching the selector; zero or several raise."""
        if (starts_with is None) == (contains is None):
            raise ValueError("give exactly one of starts_with or contains")
        hits = [
            p
            for p in self.paragraphs
            if (starts_with is not None and p.text.startswith(starts_with))
            or (contains is not None and contains in p.text)
        ]
        if len(hits) != 1:
            wanted = starts_with if starts_with is not None else contains
            raise LookupError(f"{len(hits)} paragraphs match {wanted!r}; need exactly one")
        return hits[0]


def _decode_snapshot(payload: str) -> Snapshot:
    data = json.loads(payload)
    return Snapshot(
        schema_version=data["schema_version"],
        source_sha256=data["source_sha256"],
        summary=Summary(**data["summary"]),
        paragraphs=tuple(
            Paragraph(
                index=p["index"],
                id=p["id"],
                text=p["text"],
                style=p["style"],
                numbered=p["numbered"],
                in_table=p["in_table"],
                page_break=p["page_break"],
                runs=tuple(Span(**s) for s in p["runs"]),
                limitations=tuple(p["limitations"]),
            )
            for p in data["paragraphs"]
        ),
    )


# ---------------------------------------------------------------------------
# Edit plans (schema_version 1) and reports
# ---------------------------------------------------------------------------

Selector = str | int | dict[str, str | int]
"""A paragraph id (``body:p:N``), an index, or ``{"starts_with"|"contains"|"id"|"index": ...}``."""

ExistingRevisions = Literal["refuse", "accept", "reject"]


def _selector(value: Selector) -> dict[str, str | int]:
    if isinstance(value, bool):
        raise TypeError("paragraph selector must be an id, an index or a dict")
    if isinstance(value, int):
        return {"index": value}
    if isinstance(value, str):
        return {"id": value}
    if isinstance(value, dict) and len(value) == 1:
        key, inner = next(iter(value.items()))
        if key in ("id", "starts_with", "contains") and isinstance(inner, str):
            return {key: inner}
        if key == "index" and isinstance(inner, int) and not isinstance(inner, bool):
            return {key: inner}
    raise TypeError("paragraph selector must be an id, an index or one of {id|index|starts_with|contains: ...}")


_FORMAT_FIELDS = frozenset({"bold", "italic", "underline", "highlight"})


def _format(value: Mapping[str, object]) -> dict[str, object]:
    spec = dict(value)
    unknown = set(spec) - _FORMAT_FIELDS
    if unknown:
        raise ValueError(f"unknown format fields: {sorted(unknown)}")
    if not spec:
        raise ValueError("format needs at least one of bold, italic, underline, highlight")
    return spec


def _run_specs(runs: Sequence[Mapping[str, object] | str]) -> tuple[dict[str, object], ...]:
    out: list[dict[str, object]] = []
    for r in runs:
        if isinstance(r, str):
            out.append({"text": r})
            continue
        spec = dict(r)
        if not isinstance(spec.get("text"), str):
            raise ValueError("every run needs a text string")
        unknown = set(spec) - {"text", *_FORMAT_FIELDS}
        if unknown:
            raise ValueError(f"unknown run fields: {sorted(unknown)}")
        out.append(spec)
    if not out:
        raise ValueError("insert_paragraph needs at least one run")
    return tuple(out)


@dataclass(frozen=True, slots=True)
class EditPlan:
    """Immutable builder for the engine's edit plan (wire schema version 1).

    Every builder method returns a new plan. ``for_document`` binds the plan to
    a snapshot's SHA-256 so a changed file is refused with ``STALE_SOURCE``.
    ``to_json`` is exactly what ``jubarte edit --plan`` reads.
    """

    author: str
    date: str | None = None
    initials: str | None = None
    existing_revisions: ExistingRevisions = "refuse"
    source_sha256: str | None = None
    operations: tuple[dict[str, object], ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.author, str) or not self.author.strip():
            raise ValueError("author must be a nonempty string")
        if self.existing_revisions not in ("refuse", "accept", "reject"):
            raise ValueError("existing_revisions must be refuse, accept or reject")

    def _with(self, op: dict[str, object]) -> EditPlan:
        return replace(self, operations=(*self.operations, op))

    def for_document(self, document: object) -> EditPlan:
        """Bind to ``document`` (a ``Document`` or a snapshot's hash string)."""
        digest = document if isinstance(document, str) else document.sha256()  # type: ignore[attr-defined]
        return replace(self, source_sha256=digest)

    def replace(
        self,
        paragraph: Selector,
        *,
        find: str,
        replacement: str,
        format: Mapping[str, object] | None = None,
        comment: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Replace the unique occurrence of ``find``; ``format`` styles only the new text."""
        op: dict[str, object] = {"kind": "replace", "paragraph": _selector(paragraph), "find": find, "replacement": replacement}
        if format is not None:
            op["format"] = _format(format)
        return self._with(_with_optional(op, id=id, comment=comment))

    def insert(
        self,
        paragraph: Selector,
        *,
        text: str,
        after: str | None = None,
        before: str | None = None,
        position: Literal["start", "end"] | None = None,
        format: Mapping[str, object] | None = None,
        comment: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert ``text`` after/before a unique anchor or at the paragraph edge; ``format`` styles it."""
        given = [k for k, v in (("after", after), ("before", before), ("position", position)) if v is not None]
        if len(given) != 1:
            raise ValueError("insert needs exactly one of after, before, position")
        op: dict[str, object] = {"kind": "insert", "paragraph": _selector(paragraph), "text": text}
        if after is not None:
            op["after"] = after
        if before is not None:
            op["before"] = before
        if position is not None:
            op["position"] = position
        if format is not None:
            op["format"] = _format(format)
        return self._with(_with_optional(op, id=id, comment=comment))

    def delete(self, paragraph: Selector, *, find: str, id: str | None = None) -> EditPlan:
        """Delete the unique occurrence of ``find``."""
        op: dict[str, object] = {"kind": "delete", "paragraph": _selector(paragraph), "find": find}
        return self._with(_with_optional(op, id=id))

    def comment(self, paragraph: Selector, *, text: str, find: str | None = None, id: str | None = None) -> EditPlan:
        """Comment on the unique occurrence of ``find`` or on the whole paragraph."""
        op: dict[str, object] = {"kind": "comment", "paragraph": _selector(paragraph), "text": text}
        if find is not None:
            op["find"] = find
        return self._with(_with_optional(op, id=id))

    def insert_paragraph(
        self,
        paragraph: Selector,
        *,
        runs: Sequence[Mapping[str, object] | str],
        position: Literal["before", "after"] = "after",
        style: str | None = None,
        comment: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert a new paragraph next to the anchor, copying its properties."""
        op: dict[str, object] = {"kind": "insert_paragraph", "paragraph": _selector(paragraph), "position": position, "runs": list(_run_specs(runs))}
        if style is not None:
            op["style"] = style
        return self._with(_with_optional(op, id=id, comment=comment))

    def delete_paragraph(self, paragraph: Selector, *, id: str | None = None) -> EditPlan:
        """Delete a whole paragraph, mark included."""
        op: dict[str, object] = {"kind": "delete_paragraph", "paragraph": _selector(paragraph)}
        return self._with(_with_optional(op, id=id))

    def format_paragraph(
        self,
        paragraph: Selector,
        *,
        style: str | None = None,
        alignment: Literal["left", "center", "right", "justify"] | None = None,
        line_spacing: float | None = None,
        space_before: float | None = None,
        space_after: float | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Restyle a paragraph as a tracked property change.

        ``style`` is a paragraph style id or name, ``line_spacing`` a multiple
        (1.15), ``space_before``/``space_after`` points.
        """
        fields: dict[str, object] = {
            k: v
            for k, v in (
                ("style", style),
                ("alignment", alignment),
                ("line_spacing", line_spacing),
                ("space_before", space_before),
                ("space_after", space_after),
            )
            if v is not None
        }
        if not fields:
            raise ValueError("format_paragraph needs at least one of style, alignment, line_spacing, space_before, space_after")
        op: dict[str, object] = {"kind": "format_paragraph", "paragraph": _selector(paragraph), **fields}
        return self._with(_with_optional(op, id=id))

    def merge_paragraphs(self, paragraph: Selector, *, separator: str | None = None, id: str | None = None) -> EditPlan:
        """Join the next paragraph onto this one; the redline deletes this paragraph's mark."""
        op: dict[str, object] = {"kind": "merge_paragraphs", "paragraph": _selector(paragraph)}
        return self._with(_with_optional(op, separator=separator, id=id))

    def to_dict(self) -> dict[str, object]:
        """The wire form."""
        wire: dict[str, object] = {"schema_version": 1, "author": self.author}
        if self.source_sha256 is not None:
            wire["source_sha256"] = self.source_sha256
        if self.date is not None:
            wire["date"] = self.date
        if self.initials is not None:
            wire["initials"] = self.initials
        if self.existing_revisions != "refuse":
            wire["existing_revisions"] = self.existing_revisions
        wire["operations"] = [dict(op) for op in self.operations]
        return wire

    def to_json(self) -> str:
        """The wire form as JSON (what ``jubarte edit --plan`` reads)."""
        return json.dumps(self.to_dict(), ensure_ascii=False, indent=2)


def _with_optional(op: dict[str, object], **extra: str | None) -> dict[str, object]:
    for key, value in extra.items():
        if value is not None:
            op[key] = value
    if "id" in op:
        # Keep the id first for readable plans.
        op = {"id": op.pop("id"), **op}
    return op


def plan_json(plan: EditPlan | Mapping[str, object] | str) -> str:
    """Wire JSON for a builder, a plain dict plan, or already-serialized JSON."""
    if isinstance(plan, EditPlan):
        return plan.to_json()
    if isinstance(plan, str):
        return plan
    if isinstance(plan, Mapping):
        return json.dumps(plan, ensure_ascii=False)
    raise TypeError("plan must be an EditPlan, a dict, or a JSON string")


@dataclass(frozen=True, slots=True)
class EditOutcome:
    """One operation's result."""

    id: str
    kind: str
    status: Literal["ok", "failed", "skipped"]
    matches: int
    paragraph: str | None = None
    context: str | None = None
    comment_id: int | None = None
    code: str | None = None
    message: str | None = None


@dataclass(frozen=True, slots=True)
class RevisionCounts:
    """Comparer revision records in the redline."""

    inserted: int
    deleted: int
    moved: int
    format_changed: int
    total: int


@dataclass(frozen=True, slots=True)
class ParagraphDelta:
    """Body paragraph count before and after."""

    from_: int
    to: int


@dataclass(frozen=True, slots=True)
class EditReport:
    """What happened, per operation and overall."""

    schema_version: int
    ok: bool
    source_sha256: str
    base_sha256: str
    guarded: bool
    author: str
    date: str
    existing_revisions: str
    paragraphs: ParagraphDelta
    operations: tuple[EditOutcome, ...]
    comments_added: int
    revisions: RevisionCounts
    _json: str = field(repr=False, compare=False, default="")

    def to_jsonl(self) -> str:
        """JSON lines (``load``, one ``op`` per operation, ``summary``)."""
        from . import _native

        return _native.report_jsonl(self._json)


def _decode_outcomes(rows: list[dict[str, object]]) -> tuple[EditOutcome, ...]:
    return tuple(EditOutcome(**row) for row in rows)  # type: ignore[arg-type]


def _decode_report(payload: str) -> EditReport:
    data = json.loads(payload)
    return EditReport(
        schema_version=data["schema_version"],
        ok=data["ok"],
        source_sha256=data["source_sha256"],
        base_sha256=data["base_sha256"],
        guarded=data["guarded"],
        author=data["author"],
        date=data["date"],
        existing_revisions=data["existing_revisions"],
        paragraphs=ParagraphDelta(from_=data["paragraphs"]["from"], to=data["paragraphs"]["to"]),
        operations=_decode_outcomes(data["operations"]),
        comments_added=data["comments_added"],
        revisions=RevisionCounts(**data["revisions"]),
        _json=payload,
    )


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


@dataclass(frozen=True, slots=True)
class FontResolution:
    """One requested family/style and the physical face that painted it."""

    requested: str
    step: str
    physical: str
    bold: bool
    italic: bool
    synthetic: bool


@dataclass(frozen=True, slots=True)
class PageText:
    """Text painted on one page, one line per baseline."""

    index: int
    text: str


@dataclass(frozen=True, slots=True)
class RenderReport:
    """Page count, page text and font resolutions from one layout pass."""

    page_count: int
    pages: tuple[PageText, ...]
    fonts: tuple[FontResolution, ...]


@dataclass(frozen=True, slots=True)
class Rendered:
    """Output of ``Document.render``."""

    pdf: bytes | None
    pngs: tuple[bytes, ...]
    report: RenderReport


def _decode_render_report(payload: str) -> RenderReport:
    data = json.loads(payload)
    return RenderReport(
        page_count=data["page_count"],
        pages=tuple(PageText(**p) for p in data["pages"]),
        fonts=tuple(FontResolution(**f) for f in data["fonts"]),
    )
