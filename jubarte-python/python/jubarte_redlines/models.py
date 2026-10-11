# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Immutable options and revision records for the Python document facade."""

from __future__ import annotations

import base64
import json
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field, replace
from datetime import datetime, timezone
from types import MappingProxyType
from typing import TYPE_CHECKING, Literal, TypedDict

if TYPE_CHECKING:
    # typing.NotRequired is 3.11+; the native row shapes below need it only
    # statically (``from __future__ import annotations`` keeps every
    # annotation a string at runtime, so 3.10 never imports it).
    if sys.version_info >= (3, 11):
        from typing import NotRequired
    else:
        from typing_extensions import NotRequired

    from .document import Document

RevisionKind = Literal["Inserted", "Deleted", "Moved", "FormatChanged"]
RevisionStyle = Literal["conventional", "word", "custom"]


@dataclass(frozen=True, slots=True)
class CompareOptions:
    """Comparison metadata; ``None`` preserves the engine's fixed timestamp.

    Explicit timestamps must include an offset. They are normalized to UTC.
    Low-level ``compare_documents`` retains its existing permissive signature.
    """

    date: str | datetime | None = None
    #: Admission budget overrides (``max_compressed_bytes``, ``max_entries``,
    #: ``max_part_bytes``, ``max_uncompressed_bytes``, ``max_xml_depth``);
    #: unset keys keep the engine's compare budget. The engine refuses
    #: unknown keys when the comparison runs.
    input_limits: Mapping[str, int] | None = field(default=None, hash=False)

    def __post_init__(self) -> None:
        if self.input_limits is not None:
            object.__setattr__(self, "input_limits", _input_limits(self.input_limits))
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

    def native_input_limits(self) -> dict[str, int] | None:
        """Return the overrides as the native ``input_limits`` dict."""
        return None if self.input_limits is None else dict(self.input_limits)


def _input_limits(value: object) -> Mapping[str, int]:
    if not isinstance(value, Mapping):
        raise TypeError("input_limits must be a mapping of str to int")
    limits: dict[str, int] = {}
    for key, limit in value.items():
        if not isinstance(key, str):
            raise TypeError("input_limits keys must be strings")
        if isinstance(limit, bool) or not isinstance(limit, int):
            raise TypeError(f"input_limits[{key!r}] must be an int")
        if limit < 0:
            raise ValueError(f"input_limits[{key!r}] must not be negative")
        limits[key] = limit
    return MappingProxyType(limits)


@dataclass(frozen=True, slots=True)
class PdfOptions:
    """PDF options with the same defaults as the current byte API.

    ``move_comments`` lists the comments after the last page instead of in
    balloons beside the text; ``changed_only`` keeps only the pages a tracked
    change touches. Both mirror the binary's flags and apply to PDF and PNG
    output; ``diff_render`` ignores them.
    """

    compress: bool = False
    revisions: RevisionStyle = "conventional"
    revision_palette: str | None = None
    move_comments: bool = False
    changed_only: bool = False

    def __post_init__(self) -> None:
        for name in ("compress", "move_comments", "changed_only"):
            if not isinstance(getattr(self, name), bool):
                raise TypeError(f"{name} must be a bool")
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


ChangeKind = Literal["insertion", "deletion", "move", "formatting"]


@dataclass(frozen=True, slots=True)
class Change:
    """One tracked change with the id ``Document.accept`` / ``reject`` select by.

    ``id`` is ``{story}:rev:{w:id}`` (``body:rev:12``, ``header1:rev:3``);
    resolving some changes never renumbers the rest. ``target`` is what the
    change applies to (``text``, ``paragraph_mark``, ``table_row``,
    ``table_cell``, ``properties``). Both sides of a move share
    ``move_name`` and resolve together; ``move_side`` is ``from`` or ``to``.
    ``inside`` names the change whose content holds this one: resolving that
    one so its content goes takes this one along.
    """

    id: str
    kind: ChangeKind
    target: str
    author: str | None
    date: str | None
    text: str
    move_name: str | None
    move_side: Literal["from", "to"] | None
    inside: str | None


class _ChangeRow(TypedDict):
    """One row of the native ``changes`` JSON, keys as the engine emits them."""

    id: str
    kind: ChangeKind
    target: str
    author: NotRequired[str]
    date: NotRequired[str]
    text: str
    move_name: NotRequired[str]
    move_side: NotRequired[Literal["from", "to"]]
    inside: NotRequired[str]


def _decode_changes(payload: str) -> tuple[Change, ...]:
    rows: list[_ChangeRow] = json.loads(payload)
    return tuple(
        Change(
            id=row["id"],
            kind=row["kind"],
            target=row["target"],
            author=row.get("author"),
            date=row.get("date"),
            text=row["text"],
            move_name=row.get("move_name"),
            move_side=row.get("move_side"),
            inside=row.get("inside"),
        )
        for row in rows
    )


@dataclass(frozen=True, slots=True)
class Comment:
    """One comment with its thread position and the text it is anchored to.

    ``id`` is the comment's ``w:id``, which ``EditPlan.reply_comment``,
    ``resolve_comment``, ``edit_comment`` and ``delete_comment`` take.
    ``parent`` is the id of the comment it replies to (Word threads are one
    level deep); ``done`` is set when the thread is resolved. ``paragraph``
    is the paragraph id where the range starts (``body:p:12``);
    ``anchor_text`` is the commented text, paragraphs joined by ``\n``, with
    up to 80 characters ``before`` and ``after`` it.
    """

    id: int
    author: str
    initials: str | None
    date: str | None
    text: str
    parent: int | None
    done: bool
    paragraph: str | None
    anchor_text: str
    before: str
    after: str


class _CommentRow(TypedDict):
    """One row of the native ``comments`` JSON, keys as the engine emits them."""

    id: int
    author: str
    initials: NotRequired[str]
    date: NotRequired[str]
    text: str
    parent: NotRequired[int]
    done: bool
    paragraph: NotRequired[str]
    anchor_text: str
    before: str
    after: str


def _decode_comments(payload: str) -> tuple[Comment, ...]:
    rows: list[_CommentRow] = json.loads(payload)
    return tuple(
        Comment(
            id=row["id"],
            author=row["author"],
            initials=row.get("initials"),
            date=row.get("date"),
            text=row["text"],
            parent=row.get("parent"),
            done=row["done"],
            paragraph=row.get("paragraph"),
            anchor_text=row["anchor_text"],
            before=row["before"],
            after=row["after"],
        )
        for row in rows
    )


def change_filter(
    ids: Sequence[str] | None = None,
    authors: Sequence[str] | None = None,
    kinds: Sequence[ChangeKind] | None = None,
) -> str:
    """The engine's change filter as JSON: a change is selected when it
    matches every list given; ``None`` leaves that list out (any), an empty
    list selects nothing."""
    wire: dict[str, list[str]] = {}
    for key, value in (("ids", ids), ("authors", authors), ("kinds", kinds)):
        if value is None:
            continue
        if isinstance(value, str):
            raise TypeError(f"{key} must be a sequence of strings, not a string")
        wire[key] = list(value)
    return json.dumps(wire, ensure_ascii=False)


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
    """One paragraph (``body:p:N``, ``header1:p:N``...); ``index``/``id`` are valid for this snapshot only."""

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
class Story:
    """A header, footer or notes part an edit plan can address by ``story``."""

    id: str
    kind: str
    part: str
    paragraphs: tuple[Paragraph, ...]


@dataclass(frozen=True, slots=True)
class ContentControl:
    """A content control (``w:sdt``) in the body; ``EditPlan.fill_control`` fills it.

    ``kind`` is ``text``, ``rich_text``, ``drop_down``, ``combo_box``, ``date``,
    ``checkbox``, ``picture``, ``group``, ``repeating``, ``building_block``,
    ``citation``, ``bibliography``, ``equation`` or ``unknown``. ``paragraph_ids``
    lists the paragraphs a block-level control spans, or the one holding a
    run-level control. ``choices`` are the list values of a drop-down or combo
    box; ``checked`` is a checkbox's state.
    """

    id: str
    kind: str
    text: str
    paragraph_ids: tuple[str, ...]
    locked: bool
    placeholder: bool
    tag: str | None = None
    alias: str | None = None
    choices: tuple[str, ...] = ()
    checked: bool | None = None


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
class TableCell:
    """A table cell: the ids of its own paragraphs and their text joined with ``\\n``."""

    paragraph_ids: tuple[str, ...]
    text: str


@dataclass(frozen=True, slots=True)
class Table:
    """A body table as a grid; nested tables are separate entries.

    ``rows`` holds the cells as the XML has them (a merged cell is one cell),
    ``header_rows`` the leading rows that repeat as a header, ``widths_dxa``
    the grid column widths in twentieths of a point (0 when unreadable).
    """

    index: int
    rows: tuple[tuple[TableCell, ...], ...]
    header_rows: int
    widths_dxa: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class Snapshot:
    """What ``Document.inspect()`` returns; the coordinates an ``EditPlan`` uses."""

    schema_version: int
    source_sha256: str
    summary: Summary
    paragraphs: tuple[Paragraph, ...]
    stories: tuple[Story, ...] = ()
    tables: tuple[Table, ...] = ()
    controls: tuple[ContentControl, ...] = ()

    def control(self, id: str | None = None, *, tag: str | None = None, alias: str | None = None) -> ContentControl:
        """The one control with this id (``body:sdt:N``), tag or alias; give exactly one."""
        given = [(k, v) for k, v in (("id", id), ("tag", tag), ("alias", alias)) if v is not None]
        if len(given) != 1:
            raise ValueError("give exactly one of id, tag or alias")
        key, value = given[0]
        hits = [c for c in self.controls if getattr(c, key) == value]
        if len(hits) != 1:
            raise LookupError(f"{len(hits)} controls have {key} {value!r}; need exactly one")
        return hits[0]

    def paragraph(self, id_or_index: str | int) -> Paragraph:
        """The paragraph with this id (``body:p:N``, ``header1:p:0``) or body index."""
        for p in self.paragraphs:
            if p.id == id_or_index or p.index == id_or_index:
                return p
        for story in self.stories:
            for p in story.paragraphs:
                if p.id == id_or_index:
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


class _SpanRow(TypedDict):
    """One span of the native ``inspect`` snapshot's run formatting."""

    start: int
    end: int
    bold: bool
    italic: bool
    underline: bool
    highlight: str | None


class _ParagraphRow(TypedDict):
    """One paragraph row of the native ``inspect`` snapshot."""

    index: int
    id: str
    text: str
    style: str | None
    numbered: bool
    in_table: bool
    page_break: bool
    runs: list[_SpanRow]
    limitations: list[str]


def _decode_paragraph(p: _ParagraphRow) -> Paragraph:
    return Paragraph(
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


def _decode_snapshot(payload: str) -> Snapshot:
    data = json.loads(payload)
    return Snapshot(
        schema_version=data["schema_version"],
        source_sha256=data["source_sha256"],
        summary=Summary(**data["summary"]),
        paragraphs=tuple(_decode_paragraph(p) for p in data["paragraphs"]),
        stories=tuple(
            Story(
                id=s["id"],
                kind=s["kind"],
                part=s["part"],
                paragraphs=tuple(_decode_paragraph(p) for p in s["paragraphs"]),
            )
            for s in data.get("stories", ())
        ),
        tables=tuple(
            Table(
                index=t["index"],
                rows=tuple(
                    tuple(TableCell(paragraph_ids=tuple(c["paragraph_ids"]), text=c["text"]) for c in row)
                    for row in t["rows"]
                ),
                header_rows=t["header_rows"],
                widths_dxa=tuple(t["widths_dxa"]),
            )
            for t in data.get("tables", ())
        ),
        controls=tuple(
            ContentControl(
                id=c["id"],
                kind=c["kind"],
                text=c["text"],
                paragraph_ids=tuple(c["paragraph_ids"]),
                locked=c["locked"],
                placeholder=c["placeholder"],
                tag=c.get("tag"),
                alias=c.get("alias"),
                choices=tuple(c.get("choices", ())),
                checked=c.get("checked"),
            )
            for c in data.get("controls", ())
        ),
    )


# ---------------------------------------------------------------------------
# Edit plans (schema_version 1) and reports
# ---------------------------------------------------------------------------

Selector = str | int | dict[str, str | int]
"""A paragraph id (``body:p:N``, ``header1:p:0``), a body index, or ``{"starts_with"|"contains"|"id"|"index": ...}``;
``index``/``starts_with``/``contains`` also take ``"story": "header1"`` (default: the body)."""

ExistingRevisions = Literal["refuse", "accept", "reject", "keep"]
ProtectionEdit = Literal["none", "readOnly", "comments", "trackedChanges", "forms"]
_PROTECTIONS = ("none", "readOnly", "comments", "trackedChanges", "forms")
"""What an edit plan does with tracked changes already in the source:
``refuse`` (default), ``accept`` or ``reject`` them first, or ``keep`` them
tracked and add the plan's edits as new revisions beside them."""

ControlSelector = str | dict[str, str]
"""A control id (``body:sdt:N``) or exactly one of ``{"id"|"tag"|"alias": ...}``."""


def _control_selector(value: ControlSelector) -> str | dict[str, str]:
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and len(value) == 1:
        key, inner = next(iter(value.items()))
        if key in ("id", "tag", "alias") and isinstance(inner, str):
            return {key: inner}
    raise TypeError('control selector must be an id ("body:sdt:N") or one of {id|tag|alias: ...}')


def _selector(value: Selector) -> dict[str, str | int]:
    if isinstance(value, bool):
        raise TypeError("paragraph selector must be an id, an index or a dict")
    if isinstance(value, int):
        return {"index": value}
    if isinstance(value, str):
        return {"id": value}
    if isinstance(value, dict):
        rest = dict(value)
        story = rest.pop("story", None)
        if len(rest) == 1 and (story is None or isinstance(story, str)):
            key, inner = next(iter(rest.items()))
            scoped = {} if story is None else {"story": story}
            if key == "id" and story is None and isinstance(inner, str):
                return {key: inner}
            if key in ("starts_with", "contains") and isinstance(inner, str):
                return {key: inner, **scoped}
            if key == "index" and isinstance(inner, int) and not isinstance(inner, bool):
                return {key: inner, **scoped}
    raise TypeError(
        "paragraph selector must be an id, an index or one of {id|index|starts_with|contains: ...}"
        " (index/starts_with/contains may add story)"
    )


_FORMAT_FIELDS = frozenset(
    {"bold", "italic", "underline", "highlight", "font", "size_pt", "color", "strike", "caps"}
)


_MARGIN_FIELDS = frozenset({"top", "right", "bottom", "left", "header", "footer"})


def _format(value: Mapping[str, object]) -> dict[str, object]:
    spec = dict(value)
    unknown = set(spec) - _FORMAT_FIELDS
    if unknown:
        raise ValueError(f"unknown format fields: {sorted(unknown)}")
    if not spec:
        raise ValueError(f"format needs at least one of {', '.join(sorted(_FORMAT_FIELDS))}")
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
    resolve_revisions: dict[str, dict[str, list[str]]] | None = None
    update_fields: bool = False

    def __post_init__(self) -> None:
        if not isinstance(self.author, str) or not self.author.strip():
            raise ValueError("author must be a nonempty string")
        if not isinstance(self.update_fields, bool):
            raise TypeError("update_fields must be a bool")
        if self.existing_revisions not in ("refuse", "accept", "reject", "keep"):
            raise ValueError("existing_revisions must be refuse, accept, reject or keep")

    def _with(self, op: dict[str, object]) -> EditPlan:
        return replace(self, operations=(*self.operations, op))

    def resolving(
        self,
        *,
        accept: Mapping[str, Sequence[str]] | None = None,
        reject: Mapping[str, Sequence[str]] | None = None,
    ) -> EditPlan:
        """Accept, then reject, a selection of the tracked changes before editing.

        Each side is ``{"ids": [...], "authors": [...], "kinds": [...]}``: a
        change is selected when it matches every list given (``{}`` selects
        every change, an empty list none). No change may be selected by both
        (``REVISION_CONFLICT``); the changes left follow ``existing_revisions``.
        """
        wire: dict[str, dict[str, list[str]]] = {}
        for side, selection in (("accept", accept), ("reject", reject)):
            if selection is None:
                continue
            unknown = set(selection) - {"ids", "authors", "kinds"}
            if unknown:
                raise ValueError(f"unknown {side} selection fields: {sorted(unknown)}")
            for key, value in selection.items():
                if isinstance(value, str):
                    raise TypeError(f"{side}.{key} must be a sequence of strings, not a string")
            wire[side] = {key: list(value) for key, value in selection.items()}
        return replace(self, resolve_revisions=wire or None)

    def for_document(self, document: Document | str) -> EditPlan:
        """Bind to ``document`` (a ``Document`` or a snapshot's hash string)."""
        digest = document if isinstance(document, str) else document.sha256()
        return replace(self, source_sha256=digest)

    def replace(
        self,
        paragraph: Selector,
        *,
        find: str,
        replacement: str,
        format: Mapping[str, object] | None = None,
        comment: str | None = None,
        whole: bool = False,
        id: str | None = None,
        occurrence: int | None = None,
    ) -> EditPlan:
        """Replace the unique occurrence of ``find``, or its ``occurrence``-th hit (1-based).

        ``format`` styles only the new text. ``whole=True`` shows the change as
        all of ``find`` deleted, then all of ``replacement`` inserted, instead
        of Word Compare's word-level diff.
        """
        op: dict[str, object] = {"kind": "replace", "paragraph": _selector(paragraph), "find": find, "replacement": replacement}
        if format is not None:
            op["format"] = _format(format)
        if whole:
            op["whole"] = True
        _set_occurrence(op, occurrence)
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
        occurrence: int | None = None,
    ) -> EditPlan:
        """Insert ``text`` after/before an anchor or at the paragraph edge; ``format`` styles it.

        The anchor must be unique unless ``occurrence`` (1-based) picks one hit.
        """
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
        _set_occurrence(op, occurrence)
        return self._with(_with_optional(op, id=id, comment=comment))

    def delete(self, paragraph: Selector, *, find: str, id: str | None = None, occurrence: int | None = None) -> EditPlan:
        """Delete the unique occurrence of ``find``, or its ``occurrence``-th hit (1-based)."""
        op: dict[str, object] = {"kind": "delete", "paragraph": _selector(paragraph), "find": find}
        _set_occurrence(op, occurrence)
        return self._with(_with_optional(op, id=id))

    def redact(self, paragraph: Selector, *, find: str, id: str | None = None, occurrence: int | None = None) -> EditPlan:
        """Replace the unique occurrence of ``find`` (or its ``occurrence``-th hit) with one block per character.

        The redaction is no tracked change: the clean copy and the redline
        both show the blocks. The plan is refused with ``REDACTION_LEAK``
        when the text still occurs anywhere in either document (another
        paragraph, a comment, a header, the properties); the report never
        repeats it.
        """
        op: dict[str, object] = {"kind": "redact", "paragraph": _selector(paragraph), "find": find}
        _set_occurrence(op, occurrence)
        return self._with(_with_optional(op, id=id))

    def comment(
        self,
        paragraph: Selector,
        *,
        text: str,
        find: str | None = None,
        through: Selector | None = None,
        id: str | None = None,
        occurrence: int | None = None,
    ) -> EditPlan:
        """Comment on the unique occurrence of ``find`` (or its ``occurrence``-th hit) or on
        the whole paragraph; with ``through``, on every paragraph from ``paragraph`` to that one."""
        op: dict[str, object] = {"kind": "comment", "paragraph": _selector(paragraph), "text": text}
        if find is not None:
            op["find"] = find
        if through is not None:
            op["through"] = _selector(through)
        _set_occurrence(op, occurrence)
        return self._with(_with_optional(op, id=id))

    def insert_paragraph(
        self,
        paragraph: Selector,
        *,
        runs: Sequence[Mapping[str, object] | str],
        position: Literal["before", "after"] = "after",
        like: Selector | None = None,
        style: str | None = None,
        comment: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert a new paragraph next to the anchor, copying its properties,
        or those of the ``like`` paragraph."""
        op: dict[str, object] = {"kind": "insert_paragraph", "paragraph": _selector(paragraph), "position": position, "runs": list(_run_specs(runs))}
        if like is not None:
            op["like"] = _selector(like)
        if style is not None:
            op["style"] = style
        return self._with(_with_optional(op, id=id, comment=comment))

    def delete_paragraph(
        self, paragraph: Selector, *, comment: str | None = None, id: str | None = None
    ) -> EditPlan:
        """Delete a whole paragraph, mark included.

        ``comment`` is anchored on the deleted text in the redline; the clean
        copy has no paragraph to hold it.
        """
        op: dict[str, object] = {"kind": "delete_paragraph", "paragraph": _selector(paragraph)}
        return self._with(_with_optional(op, comment=comment, id=id))

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

    def rewrite(self, paragraph: Selector, *, text: str, id: str | None = None) -> EditPlan:
        """Make the paragraph read as ``text``: only the words that differ are
        edited, so the rest keeps its runs and formatting."""
        op: dict[str, object] = {"kind": "rewrite", "paragraph": _selector(paragraph), "text": text}
        return self._with(_with_optional(op, id=id))

    def insert_table(
        self,
        paragraph: Selector,
        *,
        rows: Sequence[Sequence[str]],
        position: Literal["before", "after"] = "after",
        header_row: bool = False,
        widths_dxa: Sequence[int] | None = None,
        style: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert a table next to the anchor paragraph; the redline shows its
        rows inserted.

        ``rows`` is the cell text row by row, every row the same length;
        ``widths_dxa`` the column widths in twentieths of a point (the text
        width split evenly when omitted); ``style`` a table style id or name
        (``TableGrid``, added when the document lacks it, by default).
        """
        op: dict[str, object] = {"kind": "insert_table", "paragraph": _selector(paragraph), "position": position, "rows": _table_rows(rows)}
        if header_row:
            op["header_row"] = True
        if widths_dxa is not None:
            if isinstance(widths_dxa, (str, bytes)) or not all(isinstance(w, int) and not isinstance(w, bool) for w in widths_dxa):
                raise TypeError("widths_dxa must be a sequence of integers")
            op["widths_dxa"] = list(widths_dxa)
        return self._with(_with_optional(op, style=style, id=id))

    def list_paragraphs(
        self,
        paragraphs: Sequence[Selector],
        *,
        kind_of_list: Literal["bullet", "decimal", "lower_letter"] = "bullet",
        level: int = 0,
        restart: bool = True,
        id: str | None = None,
    ) -> EditPlan:
        """Make the paragraphs a list (wire kind ``list``); the redline records
        each paragraph's old properties.

        ``level`` is 0 (outermost) to 8. ``restart=False`` continues the list
        of the nearest numbered paragraph before the first one instead of
        starting a new one.
        """
        if isinstance(paragraphs, (str, bytes, dict)):
            raise TypeError("paragraphs must be a sequence of selectors")
        op: dict[str, object] = {"kind": "list", "paragraphs": [_selector(p) for p in paragraphs]}
        if kind_of_list != "bullet":
            op["kind_of_list"] = kind_of_list
        if level:
            op["level"] = level
        if not restart:
            op["restart"] = False
        return self._with(_with_optional(op, id=id))

    def reply_comment(self, comment_id: int, *, text: str, id: str | None = None) -> EditPlan:
        """Reply to comment ``comment_id`` (``Document.comments`` lists the ids),
        anchored on the same text; a reply to a reply joins the thread."""
        op: dict[str, object] = {"kind": "reply_comment", "comment_id": comment_id, "text": text}
        return self._with(_with_optional(op, id=id))

    def resolve_comment(self, comment_id: int, *, done: bool = True, id: str | None = None) -> EditPlan:
        """Resolve comment ``comment_id`` and its replies; ``done=False`` reopens them."""
        op: dict[str, object] = {"kind": "resolve_comment", "comment_id": comment_id, "done": done}
        return self._with(_with_optional(op, id=id))

    def edit_comment(self, comment_id: int, *, text: str, id: str | None = None) -> EditPlan:
        """Replace the text of comment ``comment_id``; its author, date and thread stay."""
        op: dict[str, object] = {"kind": "edit_comment", "comment_id": comment_id, "text": text}
        return self._with(_with_optional(op, id=id))

    def delete_comment(self, comment_id: int, *, id: str | None = None) -> EditPlan:
        """Remove comment ``comment_id`` with its replies and anchors."""
        op: dict[str, object] = {"kind": "delete_comment", "comment_id": comment_id}
        return self._with(_with_optional(op, id=id))

    def watermark(
        self,
        text: str,
        *,
        color: str = "C0C0C0",
        diagonal: bool = True,
        font: str = "Calibri",
        id: str | None = None,
    ) -> EditPlan:
        """Write Word's own text watermark into every default header.

        ``text`` is 1 to 64 plain characters, ``color`` six hex digits, and
        ``diagonal=False`` lays it horizontal. One watermark per document; it
        is header content, so the redline carries it without tracking it.
        """
        op: dict[str, object] = {"kind": "watermark", "text": text, "color": color, "diagonal": diagonal, "font": font}
        return self._with(_with_optional(op, id=id))

    def fill_control(
        self,
        control: ControlSelector,
        *,
        text: str | None = None,
        choice: str | None = None,
        checked: bool | None = None,
        date: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Fill one content control with exactly one of ``text``, ``choice`` (a list
        item's value or display text), ``checked`` or ``date`` (``YYYY-MM-DD``).

        The control keeps its properties in the clean copy; the redline shows the
        fill as tracked text (the comparer unwraps controls in revised paragraphs,
        as Word Compare does)."""
        values = {"text": text, "choice": choice, "checked": checked, "date": date}
        given = {k: v for k, v in values.items() if v is not None}
        if len(given) != 1:
            raise ValueError("fill_control takes exactly one of text, choice, checked, date")
        if checked is not None and not isinstance(checked, bool):
            raise TypeError("checked must be a bool")
        op: dict[str, object] = {"kind": "fill_control", "control": _control_selector(control), **given}
        if id is not None:
            op = {"id": id, **op}
        return self._with(op)

    def format_run(
        self,
        paragraph: Selector,
        *,
        find: str,
        format: Mapping[str, object],
        occurrence: int | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Change the run formatting of ``find`` (bold, italic, underline,
        highlight, font, size_pt, color, strike, caps) as a tracked property
        change. ``occurrence`` (1-based) picks one of several matches."""
        op: dict[str, object] = {"kind": "format_run", "paragraph": _selector(paragraph), "find": find, "format": _format(format)}
        if occurrence is not None:
            op["occurrence"] = occurrence
        return self._with(_with_optional(op, id=id))

    def insert_footnote(
        self,
        paragraph: Selector,
        *,
        after: str,
        text: str,
        occurrence: int | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Add a footnote holding ``text`` whose mark follows ``after`` in a
        body paragraph. ``occurrence`` (1-based) picks one of several matches."""
        op: dict[str, object] = {"kind": "insert_footnote", "paragraph": _selector(paragraph), "after": after, "text": text}
        if occurrence is not None:
            op["occurrence"] = occurrence
        return self._with(_with_optional(op, id=id))

    def insert_image(
        self,
        paragraph: Selector,
        *,
        image: bytes,
        position: Literal["before", "after"] = "after",
        content_type: str | None = None,
        width_emu: int | None = None,
        alt: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert a paragraph holding the picture ``image`` (PNG, JPEG, GIF,
        BMP or TIFF bytes) next to a body paragraph. ``width_emu`` sets the
        width (914400 per inch) and keeps the aspect ratio; by default the
        picture is its pixel size at 96 dpi, at most 6.5 inches wide."""
        if not image:
            raise ValueError("image must hold the picture's bytes")
        op: dict[str, object] = {
            "kind": "insert_image",
            "paragraph": _selector(paragraph),
            "position": position,
            "image_base64": base64.b64encode(bytes(image)).decode("ascii"),
        }
        if content_type is not None:
            op["content_type"] = content_type
        if width_emu is not None:
            op["width_emu"] = width_emu
        return self._with(_with_optional(op, alt=alt, id=id))

    def page_setup(
        self,
        *,
        section: Literal["last", "all"] = "last",
        page: Literal["letter", "a4"] | Mapping[str, int] | None = None,
        orientation: Literal["portrait", "landscape"] | None = None,
        margins_dxa: Mapping[str, int] | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Set the page size, orientation and margins of the last section or
        of every section, as a tracked section change. ``page`` is ``"letter"``,
        ``"a4"`` or ``{"width_dxa", "height_dxa"}``; ``margins_dxa`` takes any
        of top, right, bottom, left, header, footer, in twentieths of a point
        (1440 per inch)."""
        op: dict[str, object] = {"kind": "page_setup", "section": section}
        if page is not None:
            if isinstance(page, Mapping):
                if set(page) != {"width_dxa", "height_dxa"}:
                    raise ValueError("a custom page needs exactly width_dxa and height_dxa")
                op["page"] = dict(page)
            else:
                op["page"] = page
        if orientation is not None:
            op["orientation"] = orientation
        if margins_dxa is not None:
            unknown = set(margins_dxa) - _MARGIN_FIELDS
            if unknown:
                raise ValueError(f"unknown margins: {sorted(unknown)}")
            if not margins_dxa:
                raise ValueError("margins_dxa needs at least one margin")
            op["margins_dxa"] = dict(margins_dxa)
        if len(op) == 2:
            raise ValueError("page_setup needs page, orientation or margins_dxa")
        return self._with(_with_optional(op, id=id))

    def insert_toc(
        self,
        paragraph: Selector,
        *,
        position: Literal["before", "after"] = "after",
        levels: int = 3,
        title: str | None = None,
        id: str | None = None,
    ) -> EditPlan:
        """Insert a table of contents (``TOC \\o "1-levels" \\h \\z \\u``) next to
        the anchor, after an optional ``TOCHeading`` title.

        Its entries and page numbers are written when the plan sets
        ``update_fields=True``; page numbers come from jubarte's layout.
        """
        if isinstance(levels, bool) or not isinstance(levels, int) or not 1 <= levels <= 9:
            raise ValueError("levels must be an int from 1 to 9")
        op: dict[str, object] = {"kind": "insert_toc", "paragraph": _selector(paragraph), "position": position, "levels": levels}
        return self._with(_with_optional(op, title=title, id=id))

    def settings(
        self,
        *,
        track_revisions: bool | None = None,
        update_fields: bool | None = None,
        protection: ProtectionEdit | None = None,
        enforcement: bool = True,
        id: str | None = None,
    ) -> EditPlan:
        """Write document settings, in schema order, into both documents.

        ``track_revisions`` turns Track Changes on or off, ``update_fields``
        asks Word to update fields on open (``w:updateFields``; the plan's own
        ``update_fields`` writes jubarte's results instead), and ``protection`` restricts
        editing (``"readOnly"``, ``"comments"``, ``"trackedChanges"``,
        ``"forms"``; ``"none"`` lifts it). The restriction has no password,
        so any user can turn it off in Word. A setting left as ``None``
        stays as it is; one ``settings`` per plan.
        """
        for name, value in (("track_revisions", track_revisions), ("update_fields", update_fields)):
            if value is not None and not isinstance(value, bool):
                raise TypeError(f"{name} must be a bool or None")
        if not isinstance(enforcement, bool):
            raise TypeError("enforcement must be a bool")
        if protection is not None and protection not in _PROTECTIONS:
            raise ValueError(f"protection must be one of {', '.join(_PROTECTIONS)}")
        if track_revisions is None and update_fields is None and protection is None:
            raise ValueError("settings needs at least one of track_revisions, update_fields, protection")
        op: dict[str, object] = {"kind": "settings"}
        if track_revisions is not None:
            op["track_revisions"] = track_revisions
        if update_fields is not None:
            op["update_fields"] = update_fields
        if protection is not None:
            op["protection"] = {"edit": protection, "enforcement": enforcement}
        return self._with(_with_optional(op, id=id))

    def to_dict(self) -> dict[str, object]:
        """The wire form."""
        wire: dict[str, object] = {"schema_version": 1, "author": self.author}
        if self.source_sha256 is not None:
            wire["source_sha256"] = self.source_sha256
        if self.date is not None:
            wire["date"] = self.date
        if self.initials is not None:
            wire["initials"] = self.initials
        if self.resolve_revisions is not None:
            wire["resolve_revisions"] = {side: dict(sel) for side, sel in self.resolve_revisions.items()}
        if self.existing_revisions != "refuse":
            wire["existing_revisions"] = self.existing_revisions
        wire["operations"] = [dict(op) for op in self.operations]
        if self.update_fields:
            wire["update_fields"] = True
        return wire

    def to_json(self) -> str:
        """The wire form as JSON (what ``jubarte edit --plan`` reads)."""
        return json.dumps(self.to_dict(), ensure_ascii=False, indent=2)


def _table_rows(rows: Sequence[Sequence[str]]) -> list[list[str]]:
    """A copy of ``rows``, each a sequence of cell strings; the engine checks the shape."""
    if isinstance(rows, (str, bytes)):
        raise TypeError("rows must be a sequence of rows, not a string")
    out: list[list[str]] = []
    for row in rows:
        if isinstance(row, (str, bytes)) or not all(isinstance(cell, str) for cell in row):
            raise TypeError("each row must be a sequence of cell strings")
        out.append(list(row))
    return out


def _set_occurrence(op: dict[str, object], occurrence: int | None) -> None:
    if occurrence is None:
        return
    if occurrence < 1:
        raise ValueError("occurrence is 1-based; it must be 1 or more")
    op["occurrence"] = occurrence


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
    #: The anchor as given, when it matched only without its Markdown marks.
    anchor_given: str | None = None
    #: The plain text it was read as.
    anchor_read_as: str | None = None


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
class ResolvedRevisions:
    """The change ids a plan's ``resolve_revisions`` accepted and rejected."""

    accepted: tuple[str, ...] = ()
    rejected: tuple[str, ...] = ()


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
    resolved_revisions: ResolvedRevisions = ResolvedRevisions()
    fields: tuple[FieldUpdate, ...] = ()
    _json: str = field(repr=False, compare=False, default="")

    def to_jsonl(self) -> str:
        """JSON lines (``load``, one ``op`` per operation, ``summary``)."""
        from . import _native

        return _native.report_jsonl(self._json)


class _EditOutcomeRow(TypedDict):
    """One operation row of the edit report JSON, as the engine emits it."""

    id: str
    kind: str
    status: Literal["ok", "failed", "skipped"]
    paragraph: NotRequired[str]
    matches: int
    context: NotRequired[str]
    comment_id: NotRequired[int]
    code: NotRequired[str]
    message: NotRequired[str]
    anchor_given: NotRequired[str]
    anchor_read_as: NotRequired[str]


def _decode_outcomes(rows: list[_EditOutcomeRow]) -> tuple[EditOutcome, ...]:
    return tuple(EditOutcome(**row) for row in rows)


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
        resolved_revisions=ResolvedRevisions(
            accepted=tuple(data.get("resolved_revisions", {}).get("accepted", ())),
            rejected=tuple(data.get("resolved_revisions", {}).get("rejected", ())),
        ),
        fields=_decode_field_updates(data.get("fields", ())),
        _json=payload,
    )


@dataclass(frozen=True, slots=True)
class FieldUpdate:
    """One field whose cached result was written from jubarte's layout."""

    kind: str
    code: str
    paragraph: str
    old: str
    new: str


class _FieldUpdateRow(TypedDict):
    """One row of the edit report's ``fields``, as the engine emits it."""

    kind: str
    code: str
    paragraph: str
    old: str
    new: str


def _decode_field_updates(rows: Sequence[_FieldUpdateRow]) -> tuple[FieldUpdate, ...]:
    return tuple(FieldUpdate(**row) for row in rows)


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


@dataclass(frozen=True, slots=True)
class FontResolution:
    """One requested family/style and the physical face that painted it.

    ``substituted`` is true when the requested family was drawn with a
    substitute (Word's substitution table, a bundled face of another family,
    a generic family or the last resort); a faked style alone is
    ``synthetic``.
    """

    requested: str
    step: str
    physical: str
    bold: bool
    italic: bool
    synthetic: bool
    substituted: bool = False


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

    @property
    def substitutions(self) -> tuple[FontResolution, ...]:
        """The fonts drawn with a substitute for the requested family."""
        return tuple(f for f in self.fonts if f.substituted)


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


@dataclass(frozen=True, slots=True)
class PageDiff:
    """How one page differs between the two sides of ``diff_render``.

    ``index`` is zero-based. ``changed_ratio`` is changed pixels over all
    pixels (0.0 to 1.0; 1.0 when the page exists on one side only or the two
    pages differ in size). ``bbox`` is ``(x0, y0, x1, y1)`` in pixels around
    every changed pixel (``x1``/``y1`` exclusive), ``None`` when equal.
    ``only_in`` is ``"a"`` or ``"b"`` for a page only one side has.
    """

    index: int
    changed_ratio: float
    bbox: tuple[int, int, int, int] | None
    only_in: Literal["a", "b"] | None = None

    @property
    def differs(self) -> bool:
        """Whether this page differs at all."""
        return self.changed_ratio > 0.0 or self.only_in is not None


@dataclass(frozen=True, slots=True)
class RenderDiff:
    """Output of ``diff_render``: one ``PageDiff`` per page of the longer
    document, both sides' PNG pages, and per page diff ``b``'s page with the
    change painted magenta and boxed (``None`` when the page is equal, on one
    side only, a different size, or overlays were not asked for)."""

    pages: tuple[PageDiff, ...]
    a: tuple[bytes, ...]
    b: tuple[bytes, ...]
    overlays: tuple[bytes | None, ...]
    a_report: RenderReport
    b_report: RenderReport

    @property
    def differs(self) -> bool:
        """Whether any page differs."""
        return any(p.differs for p in self.pages)


def _decode_page_diffs(payload: str) -> tuple[PageDiff, ...]:
    return tuple(
        PageDiff(
            index=p["index"],
            changed_ratio=float(p["changed_ratio"]),
            bbox=None if p["bbox"] is None else (p["bbox"][0], p["bbox"][1], p["bbox"][2], p["bbox"][3]),
            only_in=p.get("only_in"),
        )
        for p in json.loads(payload)
    )


@dataclass(frozen=True, slots=True)
class Hunk:
    """One changed paragraph of a ``Diff``.

    ``at`` is where it is: ``body:p:N`` (or a header's, footer's or note's
    ``header1:p:N``, ``footnotes:p:N``...) in a Word document, as
    ``Document.markdown`` and edit plans number paragraphs, or ``line:N`` in
    Markdown; in the new version, or the old one when ``removed``. ``text``
    is the paragraph with its changes, unwrapped.
    """

    at: str
    removed: bool
    text: str


@dataclass(frozen=True, slots=True)
class Diff:
    """The changes between two documents, as ``git diff --word-diff`` shows
    them: only the changed paragraphs, each whole, with ``[-old-]{+new+}``
    for the changes and CriticMarkup ``{==highlights==}`` and
    ``{>>comments<<}``.

    ``str(diff)`` is the text; in Jupyter it displays as a ``diff`` block.
    ``hunks`` are the changed paragraphs (none for ``format="critic"``).
    """

    text: str
    hunks: tuple[Hunk, ...] = ()

    def __str__(self) -> str:
        return self.text

    def _repr_markdown_(self) -> str:
        return f"```diff\n{self.text}```\n"


def _decode_diff(diffed: tuple[str, str]) -> Diff:
    text, hunks = diffed
    return Diff(
        text=text,
        hunks=tuple(Hunk(at=h["at"], removed=h["removed"], text=h["text"]) for h in json.loads(hunks)),
    )


@dataclass(frozen=True, slots=True)
class Finding:
    """One thing wrong with a package, from ``Document.validate``.

    ``code`` is stable (``TEXT_INSIDE_DELETION``, ``MC_UNBOUND_PREFIX``,
    ``UNTRACKED_EDIT``, ...); ``part`` is the package part and ``path`` the
    element chain inside it (``w:body[0]/w:p[3]/w:r[2]``, empty for a
    package-level finding). ``word_fatal`` is true when Word refuses or
    repairs the file for it, ``repairable`` when ``Document.repair`` fixes
    it.
    """

    code: str
    part: str
    path: str
    message: str
    word_fatal: bool
    repairable: bool


@dataclass(frozen=True, slots=True)
class Repaired:
    """Output of ``Document.repair``: the repaired document, the findings it
    fixed and the ones it could not."""

    document: Document
    repaired: tuple[Finding, ...]
    remaining: tuple[Finding, ...]


class _FindingRow(TypedDict):
    """One row of the native validate/repair/audit-tracked JSON."""

    code: str
    part: str
    path: str
    message: str
    word_fatal: bool
    repairable: bool


def _decode_findings(rows: list[_FindingRow]) -> tuple[Finding, ...]:
    return tuple(
        Finding(
            code=row["code"],
            part=row["part"],
            path=row["path"],
            message=row["message"],
            word_fatal=bool(row["word_fatal"]),
            repairable=bool(row["repairable"]),
        )
        for row in rows
    )


def _decode_findings_json(payload: str) -> tuple[Finding, ...]:
    return _decode_findings(json.loads(payload))


@dataclass(frozen=True, slots=True)
class AuditFinding:
    """One ``Document.audit`` finding.

    ``code`` is the rule (``HEADING_SKIP``, ``IMAGE_NO_DESCR``...),
    ``rule_set`` is ``a11y``, ``style`` or ``structure``, ``severity`` is
    ``error``, ``warning`` or ``info``, and ``location`` is the paragraph id
    (``body:p:N``, ``footer1:p:N``...) an edit plan targets, or a part name
    for a document-wide finding.
    """

    code: str
    rule_set: str
    severity: str
    location: str
    message: str


def _decode_audit(payload: str) -> tuple[AuditFinding, ...]:
    return tuple(
        AuditFinding(
            code=f["code"],
            rule_set=f["rule_set"],
            severity=f["severity"],
            location=f["location"],
            message=f["message"],
        )
        for f in json.loads(payload)["findings"]
    )
