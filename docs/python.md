<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Python guide

Using jubarte from Python and `uvx`: the `jubarte-redlines` wheel (PyO3 +
maturin, in-process native extension — no Word, no LibreOffice, no network,
no temp files, GIL released while the engine runs). For the other surfaces
see [docs/rust.md](rust.md) and [docs/javascript.md](javascript.md).

## Install

```sh
pip install jubarte-redlines     # or: uv add jubarte-redlines
```

CPython ≥ 3.10. One `abi3` wheel per platform; no runtime Python
dependencies. The package version is single-sourced from the Rust engine's
version. Run the CLI without installing anything:

```sh
uvx jubarte-redlines redline original.docx modified.docx -o redline.docx --author Legal
```

`redline` is an alias of `compare`; `python -m jubarte_redlines` runs the
same commands. The wheel speaks the shared command set — `compare`/`redline`,
`revisions`, `changes`, `accept`, `reject`, `inspect`, `text`, `edit`,
`convert`, `capabilities` — but not the whole Rust binary surface: it has no
`diff`, `debug` or `self-update`. Exit codes: 0 success, 1 error, 2 usage,
3 edit plan refused. Inputs are `.docx`; save a Word 97-2003 `.doc` as
`.docx` first.

## CLI reference

Generated from the real `--help` output of the wheel's `jubarte-redlines`
entry point — flags and defaults cannot drift from the shipped code.

<!-- gen:cli-python:start -->
#### `jubarte-redlines`

```text
$ jubarte-redlines --help
usage: jubarte-redlines [-h] [--version]
                        {inspect,text,edit,convert,compare,redline,revisions,changes,accept,reject,capabilities} ...

DOCX compare, tracked editing, inspection and rendering (the jubarte engine).

positional arguments:
  {inspect,text,edit,convert,compare,redline,revisions,changes,accept,reject,capabilities}
    inspect             paragraph ids, formatting spans, limitations and
                        package facts
    text                Markdown with [body:p:N] ids, the coordinates an edit
                        plan uses
    edit                apply an edit plan: clean.docx, redline.docx,
                        patch.diff, report.jsonl (+ PDF/PNG)
    convert             DOCX to PDF and/or PNG pages, with an optional page
                        report
    compare (redline)   two documents into a Word tracked-changes document
    revisions           list tracked revisions
    changes             list each tracked change with the id accept/reject
                        --id and edit plans take
    accept              accept tracked changes (all, or the ones selected)
    reject              reject tracked changes (all, or the ones selected)
    capabilities        what this build can do

options:
  -h, --help            show this help message and exit
  --version             show program's version number and exit
```

#### `jubarte-redlines inspect`

```text
$ jubarte-redlines inspect --help
usage: jubarte-redlines inspect [-h] [--json] file

positional arguments:
  file

options:
  -h, --help  show this help message and exit
  --json      emit the JSON snapshot
```

#### `jubarte-redlines text`

```text
$ jubarte-redlines text --help
usage: jubarte-redlines text [-h] file

positional arguments:
  file

options:
  -h, --help  show this help message and exit
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
usage: jubarte-redlines edit [-h] --plan PLAN.json --out-dir DIR [--dry-run]
                             [--force] [--pdf] [--png] [--dpi DPI] [-q]
                             [--revisions {conventional,word,custom}]
                             [--revision-palette SPEC]
                             file

positional arguments:
  file

options:
  -h, --help            show this help message and exit
  --plan PLAN.json
  --out-dir DIR
  --dry-run             resolve and report only; write nothing
  --force               replace an existing output directory's files
  --pdf                 also write redline.pdf and clean.pdf
  --png                 also write redline-page-NN.png and clean-page-NN.png
  --dpi DPI
  -q, --quiet           print nothing on success (patch.diff and report.jsonl
                        are still written)
  --revisions {conventional,word,custom}
                        how tracked changes are painted
  --revision-palette SPEC
                        marks for --revisions custom, e.g.
                        deleted=#AA0000:strike,...
```

#### `jubarte-redlines convert`

```text
$ jubarte-redlines convert --help
usage: jubarte-redlines convert [-h] [-o OUTPUT] [--force] [--pdf] [--png]
                                [--dpi DPI] [--compress] [--font-report FILE]
                                [--report FILE]
                                [--revisions {conventional,word,custom}]
                                [--revision-palette SPEC]
                                file

positional arguments:
  file

options:
  -h, --help            show this help message and exit
  -o, --output OUTPUT   PDF path [default: <stem>.pdf beside the input]
  --force
  --pdf                 write the PDF (default when --png is absent)
  --png                 rasterize pages to <stem>-page-NN.png
  --dpi DPI
  --compress            deflate PDF streams
  --font-report FILE    JSON font-resolution report
  --report FILE         JSON page report ({page_count, pages, fonts})
  --revisions {conventional,word,custom}
                        how tracked changes are painted
  --revision-palette SPEC
                        marks for --revisions custom, e.g.
                        deleted=#AA0000:strike,...
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
usage: jubarte-redlines compare [-h] [-o OUTPUT] [--author AUTHOR]
                                [--date DATE] [--force]
                                original modified

positional arguments:
  original
  modified

options:
  -h, --help           show this help message and exit
  -o, --output OUTPUT  [default: <original>_v_<modified>.docx]
  --author AUTHOR
  --date DATE          ISO-8601 revision timestamp (default: fixed epoch)
  --force
```

#### `jubarte-redlines redline`

```text
$ jubarte-redlines redline --help
usage: jubarte-redlines compare [-h] [-o OUTPUT] [--author AUTHOR]
                                [--date DATE] [--force]
                                original modified

positional arguments:
  original
  modified

options:
  -h, --help           show this help message and exit
  -o, --output OUTPUT  [default: <original>_v_<modified>.docx]
  --author AUTHOR
  --date DATE          ISO-8601 revision timestamp (default: fixed epoch)
  --force
```

#### `jubarte-redlines revisions`

```text
$ jubarte-redlines revisions --help
usage: jubarte-redlines revisions [-h] [--json] file

positional arguments:
  file

options:
  -h, --help  show this help message and exit
  --json      one JSON object per line
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
usage: jubarte-redlines changes [-h] [--json] file

positional arguments:
  file

options:
  -h, --help  show this help message and exit
  --json      one JSON object per line
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
usage: jubarte-redlines accept [-h] -o OUTPUT [--force] [--id ID]
                               [--author NAME]
                               [--kind {insertion,deletion,move,formatting}]
                               file

positional arguments:
  file

options:
  -h, --help            show this help message and exit
  -o, --output OUTPUT
  --force
  --id ID               only this change (body:rev:12); repeatable
  --author NAME         only changes by this author; repeatable
  --kind {insertion,deletion,move,formatting}
                        only changes of this kind; repeatable
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
usage: jubarte-redlines reject [-h] -o OUTPUT [--force] [--id ID]
                               [--author NAME]
                               [--kind {insertion,deletion,move,formatting}]
                               file

positional arguments:
  file

options:
  -h, --help            show this help message and exit
  -o, --output OUTPUT
  --force
  --id ID               only this change (body:rev:12); repeatable
  --author NAME         only changes by this author; repeatable
  --kind {insertion,deletion,move,formatting}
                        only changes of this kind; repeatable
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
usage: jubarte-redlines capabilities [-h] [--json]

options:
  -h, --help  show this help message and exit
  --json      (the output is JSON either way)
```
<!-- gen:cli-python:end -->

## Library — three tiers

### Byte-level API

```python
from pathlib import Path
from jubarte_redlines import compare_documents, get_revisions, docx_to_pdf

original = Path("original.docx").read_bytes()
modified = Path("modified.docx").read_bytes()

redline = compare_documents(original, modified, author="Reviewer")
Path("redline.docx").write_bytes(redline)

for rev in get_revisions(redline):
    print(rev["type"], rev["author"], repr(rev.get("text")))

Path("redline.pdf").write_bytes(docx_to_pdf(redline))
```

`compare_documents(original, modified, author="jubarte", date=None)` stamps a
fixed epoch date by default so output is deterministic; pass ISO-8601 `date`
to override. Errors raise `jubarte_redlines.JubarteError`.

### `Document` — immutable snapshot, path I/O once

```python
from jubarte_redlines import read

doc = read("contract.docx")
doc.markdown()    # every story as Markdown with [body:p:N] ids
doc.inspect()     # Snapshot: summary + paragraphs (ids, style, spans, limitations)
doc.sha256()      # the source_sha256 guard an edit plan carries
doc.changes()     # each tracked change with its stable id
doc.accept(ids=["body:rev:12"])              # or doc.reject(...): resolve a
                                             # selection, keep the rest tracked
result = doc.render(pdf=True, png_dpi=144)   # Rendered: pdf, pngs, page report
```

`doc.compare(other, author=...)` is `compare_documents`; `doc.to_pdf()` and
`doc.to_png(dpi=...)` are the one-shot renderers.

### `EditPlan` — the tracked-changes transaction

```python
from jubarte_redlines import read, EditPlan

doc = read("contract.docx")
plan = (
    EditPlan(author="Reviewer")
    .for_document(doc)             # bind to this snapshot; STALE_SOURCE if it changed
    .replace("body:p:3", find="30 days", replacement="45 days", format={"bold": True})
    .comment("body:p:3", text="Check with the client", find="45 days")
    .insert_paragraph("body:p:3", runs=["New sentence."], like="body:p:3")
)
out = doc.edit(plan)               # EditResult: clean, redline, report, diff
```

All-or-nothing: a refused plan raises `EditPlanError` (stable `code`, the
failed operation, every operation's outcome) and produces nothing.
`doc.preview(plan)` resolves and reports without applying. Run-level
operations: `replace`, `insert`, `delete`, `comment`; paragraph-level:
`insert_paragraph`, `delete_paragraph`, `format_paragraph`, `merge_paragraphs`,
`rewrite`; `resolving(accept={...}, reject={...})` settles existing tracked
changes first. `plan.to_json()` is exactly what `edit --plan` reads.

`diff(old, new)` (new on `main`, first in the release after 0.10.1; `main`
builds still report `__version__` 0.10.1 until that release's version bump,
so the PyPI 0.10.1 wheel does not have it) shows the changed paragraphs
between two documents — or a document and Markdown text — as `[-old-]{+new+}`
marks (`format="critic"` for CriticMarkup):

```python
from jubarte_redlines import diff, read
print(diff(read("v1.docx"), read("v2.docx")))
```

## API reference

Generated by reflecting over the installed package, in `__all__` order —
signatures, dataclass fields and docstrings below are the shipped ones.

<!-- gen:python-api:start -->
Reflects the `jubarte_redlines` package built from this source tree (`__version__` reports `0.10.1`).

### `JubarteError`


Raised when the jubarte-redlines engine cannot process a document.

### `__version__`

`'0.10.1'`

### `accept_revisions`

```python
accept_revisions(docx)
```

Accept every tracked revision (package-wide) → clean DOCX bytes.

### `compare_documents`

```python
compare_documents(original, modified, author='jubarte', date=None)
```

Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).

Mirrors `jubarte::document_comparer::compare_documents`; `date` (ISO-8601
`w:date` stamp) defaults to the engine's fixed epoch for deterministic
output.

### `docx_to_pdf`

```python
docx_to_pdf(docx, compress=False, revisions='conventional', revision_palette=None)
```

Render a DOCX package (bytes) → PDF bytes (Word-style layout).

`compress=True` deflates the PDF's streams (`/FlateDecode`), which is much
smaller but no longer plain text. `revisions` paints tracked changes:
`"conventional"` (red struck deletions, blue double-underlined insertions,
green moves), `"word"` (Microsoft Word's markup) or `"custom"` with
`revision_palette="deleted=#AA0000:strike,..."`.

### `get_revisions`

```python
get_revisions(docx: 'bytes') -> 'list[dict[str, Any]]'
```

List the tracked revisions in a DOCX as parsed objects.

Each item has the same shape as the CLI ``jubarte revisions --json`` lines
(``type``/``author``/``date``/``part``/``moveGroupId``/``isMoveSource``/
``formatChange``/``text``).

### `get_revisions_json`

```python
get_revisions_json(docx)
```

List the tracked revisions in a DOCX as a JSON array string — the same
object shape as the CLI `jubarte revisions --json` lines
(`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).

### `reject_revisions`

```python
reject_revisions(docx)
```

Reject every tracked revision (package-wide) → base DOCX bytes.

### `Document`

```python
Document(
    _data: bytes,
    name: str | None = None,
)
```

An immutable DOCX byte snapshot.

Loading is cheap and does not parse or validate the package. Operations
validate it through the native engine and preserve ``JubarteError``.
No operation modifies an input document or writes output files.

#### `Document.to_bytes`

```python
Document.to_bytes(self) -> 'bytes'
```

Return the immutable DOCX snapshot.

#### `Document.compare`

```python
Document.compare(self, modified: 'Document', *, author: 'str', options: 'CompareOptions | None' = None) -> 'Document'
```

Compare this original with ``modified``, producing tracked changes.

Existing comparison semantics, including handling of pre-existing
revisions, are unchanged. This is not the semantic edit transaction.

#### `Document.accept`

```python
Document.accept(self, *, ids: 'Sequence[str] | None' = None, authors: 'Sequence[str] | None' = None, kinds: 'Sequence[ChangeKind] | None' = None) -> 'Document'
```

Return a new document accepting tracked changes, as Word does.

With no selection every change is accepted (Accept All). ``ids``
(from ``changes()``), ``authors`` and ``kinds`` select changes that
match every list given; the others stay tracked.

#### `Document.reject`

```python
Document.reject(self, *, ids: 'Sequence[str] | None' = None, authors: 'Sequence[str] | None' = None, kinds: 'Sequence[ChangeKind] | None' = None) -> 'Document'
```

Return a new document rejecting tracked changes (selection as in
``accept``; none rejects every change).

#### `Document.changes`

```python
Document.changes(self) -> 'tuple[Change, ...]'
```

Every tracked change, each with the id ``accept`` / ``reject`` select by.

#### `Document.revisions`

```python
Document.revisions(self) -> 'tuple[Revision, ...]'
```

Return immutable metadata for the revisions listed by the engine.

#### `Document.to_pdf`

```python
Document.to_pdf(self, *, options: 'PdfOptions | None' = None) -> 'bytes'
```

Render a PDF in memory, preserving the current renderer defaults.

#### `Document.sha256`

```python
Document.sha256(self) -> 'str'
```

SHA-256 of the snapshot: the guard an ``EditPlan`` binds to.

#### `Document.inspect`

```python
Document.inspect(self) -> 'Snapshot'
```

Body paragraphs with ids, formatting spans and limitations, plus package facts.

#### `Document.markdown`

```python
Document.markdown(self) -> 'str'
```

The body as Markdown with a ``[body:p:N]`` id before each paragraph.

#### `Document.edit`

```python
Document.edit(self, plan: 'EditPlan | dict[str, object] | str') -> 'EditResult'
```

Apply a plan: clean copy, Word redline and report, or ``EditPlanError``.

Every operation is resolved against this snapshot before anything is
changed; a refused plan produces no documents. Comments in the plan
are anchored in the clean copy and carried through the redline.

#### `Document.diff`

```python
Document.diff(self, other: 'Document | str', *, author: 'str | None' = None, date: 'str | None' = None, columns: 'int' = 72, format: "Literal['patch', 'critic']" = 'patch') -> 'Diff'
```

The changes from this document to ``other`` (a ``Document`` or
Markdown text), as ``jubarte_redlines.diff`` gives them.

#### `Document.preview`

```python
Document.preview(self, plan: 'EditPlan | dict[str, object] | str') -> 'EditReport'
```

Resolve every operation and report, without producing documents.

#### `Document.to_png`

```python
Document.to_png(self, *, dpi: 'float' = 96.0, options: 'PdfOptions | None' = None) -> 'tuple[bytes, ...]'
```

One PNG per page, straight from the layout (no PDF round trip).

#### `Document.render`

```python
Document.render(self, *, pdf: 'bool' = True, png_dpi: 'float | None' = None, options: 'PdfOptions | None' = None) -> 'Rendered'
```

One layout pass: optional PDF, optional PNG pages, and the page report.

### `read`

```python
read(path: 'str | os.PathLike[str]') -> 'Document'
```

Load a local DOCX snapshot; equivalent to ``Document.read(path)``.

### `diff`

```python
diff(old: 'Document | bytes | str | os.PathLike[str]', new: 'Document | bytes | str | os.PathLike[str]', *, author: 'str | None' = None, date: 'str | None' = None, columns: 'int' = 72, format: "Literal['patch', 'critic']" = 'patch') -> 'Diff'
```

The changes from ``old`` to ``new``: the changed paragraphs, each at
its ``body:p:N`` id in a Word document or ``line:N`` in Markdown, with
``[-old-]{+new+}`` changes and CriticMarkup comments.

Each side is a ``Document`` or Word ``bytes``, Markdown text (``str``),
or a path (``.md``/``.markdown`` read as Markdown, anything else as
Word). ``author`` and ``date`` own the changes, shown once in the
header [default: ``git config user.name``, else Redline; now].
``columns`` wraps the lines (0 does not). ``format="critic"`` gives the
whole document as CriticMarkup instead, as ``jubarte diff --format
critic`` does.

### `Diff`

```python
Diff(
    text: str,
    hunks: tuple[Hunk, ...] = (),
)
```

The changes between two documents, as ``git diff --word-diff`` shows
them: only the changed paragraphs, each whole, with ``[-old-]{+new+}``
for the changes and CriticMarkup ``{==highlights==}`` and
``{>>comments<<}``.

``str(diff)`` is the text; in Jupyter it displays as a ``diff`` block.
``hunks`` are the changed paragraphs (none for ``format="critic"``).

### `Hunk`

```python
Hunk(
    at: str,
    removed: bool,
    text: str,
)
```

One changed paragraph of a ``Diff``.

``at`` is where it is: ``body:p:N`` (or a header's, footer's or note's
``header1:p:N``, ``footnotes:p:N``...) in a Word document, as
``Document.markdown`` and edit plans number paragraphs, or ``line:N`` in
Markdown; in the new version, or the old one when ``removed``. ``text``
is the paragraph with its changes, unwrapped.

### `capabilities`

```python
capabilities() -> 'dict[str, object]'
```

What this build can do (``runtime: "python"``), as a plain dict.

### `Change`

```python
Change(
    id: str,
    kind: ChangeKind,
    target: str,
    author: str | None,
    date: str | None,
    text: str,
    move_name: str | None,
    move_side: Literal['from', 'to'] | None,
    inside: str | None,
)
```

One tracked change with the id ``Document.accept`` / ``reject`` select by.

``id`` is ``{story}:rev:{w:id}`` (``body:rev:12``, ``header1:rev:3``);
resolving some changes never renumbers the rest. ``target`` is what the
change applies to (``text``, ``paragraph_mark``, ``table_row``,
``table_cell``, ``properties``). Both sides of a move share
``move_name`` and resolve together; ``move_side`` is ``from`` or ``to``.
``inside`` names the change whose content holds this one: resolving that
one so its content goes takes this one along.

### `CompareOptions`

```python
CompareOptions(
    date: str | datetime | None = None,
)
```

Comparison metadata; ``None`` preserves the engine's fixed timestamp.

Explicit timestamps must include an offset. They are normalized to UTC.
Low-level ``compare_documents`` retains its existing permissive signature.

#### `CompareOptions.native_date`

```python
CompareOptions.native_date(self) -> 'str | None'
```

Return the timestamp accepted by the existing native function.

### `FormatChange`

```python
FormatChange(
    changed_properties: tuple[str, ...],
)
```

Names of run or paragraph properties changed by a revision.

### `PdfOptions`

```python
PdfOptions(
    compress: bool = False,
    revisions: RevisionStyle = 'conventional',
    revision_palette: str | None = None,
)
```

PDF options with the same defaults as the current byte API.

### `Revision`

```python
Revision(
    kind: RevisionKind,
    author: str,
    date: str,
    part: str,
    text: str,
    move_group_id: int | None,
    is_move_source: bool | None,
    format_change: FormatChange | None,
)
```

A read-only revision record; no implied stable edit identifier.

``date`` is preserved as document text instead of rejecting old documents
whose producers emitted a nonstandard timestamp. Missing author/date/text
remain empty strings, matching the existing JSON API.

### `EditPlan`

```python
EditPlan(
    author: str,
    date: str | None = None,
    initials: str | None = None,
    existing_revisions: ExistingRevisions = 'refuse',
    source_sha256: str | None = None,
    operations: tuple[dict[str, object], ...] = (),
    resolve_revisions: dict[str, dict[str, list[str]]] | None = None,
)
```

Immutable builder for the engine's edit plan (wire schema version 1).

Every builder method returns a new plan. ``for_document`` binds the plan to
a snapshot's SHA-256 so a changed file is refused with ``STALE_SOURCE``.
``to_json`` is exactly what ``jubarte edit --plan`` reads.

#### `EditPlan.resolving`

```python
EditPlan.resolving(self, *, accept: 'Mapping[str, Sequence[str]] | None' = None, reject: 'Mapping[str, Sequence[str]] | None' = None) -> 'EditPlan'
```

Accept, then reject, a selection of the tracked changes before editing.

Each side is ``{"ids": [...], "authors": [...], "kinds": [...]}``: a
change is selected when it matches every list given (``{}`` selects
every change, an empty list none). No change may be selected by both
(``REVISION_CONFLICT``); the changes left follow ``existing_revisions``.

#### `EditPlan.for_document`

```python
EditPlan.for_document(self, document: 'object') -> 'EditPlan'
```

Bind to ``document`` (a ``Document`` or a snapshot's hash string).

#### `EditPlan.replace`

```python
EditPlan.replace(self, paragraph: 'Selector', *, find: 'str', replacement: 'str', format: 'Mapping[str, object] | None' = None, comment: 'str | None' = None, whole: 'bool' = False, id: 'str | None' = None) -> 'EditPlan'
```

Replace the unique occurrence of ``find``; ``format`` styles only the new text.

``whole=True`` shows the change as all of ``find`` deleted, then all of
``replacement`` inserted, instead of Word Compare's word-level diff.

#### `EditPlan.insert`

```python
EditPlan.insert(self, paragraph: 'Selector', *, text: 'str', after: 'str | None' = None, before: 'str | None' = None, position: "Literal['start', 'end'] | None" = None, format: 'Mapping[str, object] | None' = None, comment: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Insert ``text`` after/before a unique anchor or at the paragraph edge; ``format`` styles it.

#### `EditPlan.delete`

```python
EditPlan.delete(self, paragraph: 'Selector', *, find: 'str', id: 'str | None' = None) -> 'EditPlan'
```

Delete the unique occurrence of ``find``.

#### `EditPlan.comment`

```python
EditPlan.comment(self, paragraph: 'Selector', *, text: 'str', find: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Comment on the unique occurrence of ``find`` or on the whole paragraph.

#### `EditPlan.insert_paragraph`

```python
EditPlan.insert_paragraph(self, paragraph: 'Selector', *, runs: 'Sequence[Mapping[str, object] | str]', position: "Literal['before', 'after']" = 'after', like: 'Selector | None' = None, style: 'str | None' = None, comment: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Insert a new paragraph next to the anchor, copying its properties,
or those of the ``like`` paragraph.

#### `EditPlan.delete_paragraph`

```python
EditPlan.delete_paragraph(self, paragraph: 'Selector', *, comment: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Delete a whole paragraph, mark included.

``comment`` is anchored on the deleted text in the redline; the clean
copy has no paragraph to hold it.

#### `EditPlan.format_paragraph`

```python
EditPlan.format_paragraph(self, paragraph: 'Selector', *, style: 'str | None' = None, alignment: "Literal['left', 'center', 'right', 'justify'] | None" = None, line_spacing: 'float | None' = None, space_before: 'float | None' = None, space_after: 'float | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Restyle a paragraph as a tracked property change.

``style`` is a paragraph style id or name, ``line_spacing`` a multiple
(1.15), ``space_before``/``space_after`` points.

#### `EditPlan.merge_paragraphs`

```python
EditPlan.merge_paragraphs(self, paragraph: 'Selector', *, separator: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Join the next paragraph onto this one; the redline deletes this paragraph's mark.

#### `EditPlan.rewrite`

```python
EditPlan.rewrite(self, paragraph: 'Selector', *, text: 'str', id: 'str | None' = None) -> 'EditPlan'
```

Make the paragraph read as ``text``: only the words that differ are
edited, so the rest keeps its runs and formatting.

#### `EditPlan.to_dict`

```python
EditPlan.to_dict(self) -> 'dict[str, object]'
```

The wire form.

#### `EditPlan.to_json`

```python
EditPlan.to_json(self) -> 'str'
```

The wire form as JSON (what ``jubarte edit --plan`` reads).

### `EditPlanError`


A plan was refused; nothing was written.

``code`` is the stable engine code (``STALE_SOURCE``, ``ANCHOR_NOT_FOUND``,
``AMBIGUOUS_ANCHOR``, ``OVERLAPPING_EDITS``, ``UNSUPPORTED_STRUCTURE``,
``EXISTING_REVISIONS``, ``REVISION_CONFLICT``, ``UNKNOWN_CHANGE``,
``INVALID_PLAN``, ...), ``message`` the engine's
detail without the code, ``operation`` the id of the operation that
failed, and ``outcomes`` every operation's status at that point, so the
caller can see which anchors resolved.

### `EditResult`

```python
EditResult(
    clean: Document,
    redline: Document,
    report: EditReport,
    diff: Diff,
)
```

Clean copy, Word redline, per-operation report and the patch of one
plan: ``diff`` is what the redline tracks, by the plan's author and date.

### `EditOutcome`

```python
EditOutcome(
    id: str,
    kind: str,
    status: Literal['ok', 'failed', 'skipped'],
    matches: int,
    paragraph: str | None = None,
    context: str | None = None,
    comment_id: int | None = None,
    code: str | None = None,
    message: str | None = None,
)
```

One operation's result.

### `EditReport`

```python
EditReport(
    schema_version: int,
    ok: bool,
    source_sha256: str,
    base_sha256: str,
    guarded: bool,
    author: str,
    date: str,
    existing_revisions: str,
    paragraphs: ParagraphDelta,
    operations: tuple[EditOutcome, ...],
    comments_added: int,
    revisions: RevisionCounts,
    resolved_revisions: ResolvedRevisions = ResolvedRevisions(accepted=(), rejected=()),
    _json: str = '',
)
```

What happened, per operation and overall.

#### `EditReport.to_jsonl`

```python
EditReport.to_jsonl(self) -> 'str'
```

JSON lines (``load``, one ``op`` per operation, ``summary``).

### `RevisionCounts`

```python
RevisionCounts(
    inserted: int,
    deleted: int,
    moved: int,
    format_changed: int,
    total: int,
)
```

Comparer revision records in the redline.

### `ResolvedRevisions`

```python
ResolvedRevisions(
    accepted: tuple[str, ...] = (),
    rejected: tuple[str, ...] = (),
)
```

The change ids a plan's ``resolve_revisions`` accepted and rejected.

### `ParagraphDelta`

```python
ParagraphDelta(
    from_: int,
    to: int,
)
```

Body paragraph count before and after.

### `Snapshot`

```python
Snapshot(
    schema_version: int,
    source_sha256: str,
    summary: Summary,
    paragraphs: tuple[Paragraph, ...],
    stories: tuple[Story, ...] = (),
)
```

What ``Document.inspect()`` returns; the coordinates an ``EditPlan`` uses.

#### `Snapshot.paragraph`

```python
Snapshot.paragraph(self, id_or_index: 'str | int') -> 'Paragraph'
```

The paragraph with this id (``body:p:N``, ``header1:p:0``) or body index.

#### `Snapshot.unique`

```python
Snapshot.unique(self, *, starts_with: 'str | None' = None, contains: 'str | None' = None) -> 'Paragraph'
```

Exactly one paragraph matching the selector; zero or several raise.

### `Story`

```python
Story(
    id: str,
    kind: str,
    part: str,
    paragraphs: tuple[Paragraph, ...],
)
```

A header, footer or notes part an edit plan can address by ``story``.

### `Summary`

```python
Summary(
    paragraphs: int,
    tables: int,
    fields: int,
    sections: int,
    comments: int,
    revisions: int,
    footnotes: int,
    endnotes: int,
    headers: int,
    footers: int,
    images: int,
    list_numbering: bool,
    track_changes: bool,
)
```

Package facts (XML facts, not rendered-page facts).

### `Paragraph`

```python
Paragraph(
    index: int,
    id: str,
    text: str,
    style: str | None,
    numbered: bool,
    in_table: bool,
    page_break: bool,
    runs: tuple[Span, ...],
    limitations: tuple[str, ...],
)
```

One paragraph (``body:p:N``, ``header1:p:N``...); ``index``/``id`` are valid for this snapshot only.

### `Span`

```python
Span(
    start: int,
    end: int,
    bold: bool,
    italic: bool,
    underline: bool,
    highlight: str | None,
)
```

Direct run formatting over ``Paragraph.text`` in char offsets.

### `Rendered`

```python
Rendered(
    pdf: bytes | None,
    pngs: tuple[bytes, ...],
    report: RenderReport,
)
```

Output of ``Document.render``.

### `RenderReport`

```python
RenderReport(
    page_count: int,
    pages: tuple[PageText, ...],
    fonts: tuple[FontResolution, ...],
)
```

Page count, page text and font resolutions from one layout pass.

### `PageText`

```python
PageText(
    index: int,
    text: str,
)
```

Text painted on one page, one line per baseline.

### `FontResolution`

```python
FontResolution(
    requested: str,
    step: str,
    physical: str,
    bold: bool,
    italic: bool,
    synthetic: bool,
)
```

One requested family/style and the physical face that painted it.
<!-- gen:python-api:end -->

## Development

From `jubarte-python/`:

```sh
uv run --with maturin maturin develop --release
uv run --with pytest pytest -q
```

## Keeping this page current

`scripts/gen_docs.sh` regenerates both generated blocks; the CI docs job
(`docs.yml`) fails when the committed page drifts from the wheel. Curated
sections mirror [`jubarte-python/README.md`](../jubarte-python/README.md)
(the README ships inside the package — update both when you change an
example; nothing automates that sync yet).
