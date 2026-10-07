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
                        {inspect,text,edit,convert,compare,redline,revisions,changes,comments,accept,reject,diff-render,validate,capabilities} ...

DOCX compare, tracked editing, inspection and rendering (the jubarte engine).

positional arguments:
  {inspect,text,edit,convert,compare,redline,revisions,changes,comments,accept,reject,diff-render,validate,capabilities}
    inspect             paragraph ids, formatting spans, limitations and
                        package facts
    text                Markdown with [body:p:N] ids, the coordinates an edit
                        plan uses
    edit                apply an edit plan: clean.docx, redline.docx,
                        patch.diff, report.jsonl (+ PDF/PNG)
    convert             DOCX to PDF and/or PNG pages, with an optional page
                        report; Markdown to DOCX
    compare (redline)   two documents into a Word tracked-changes document
    revisions           list tracked revisions
    changes             list each tracked change with the id accept/reject
                        --id and edit plans take
    comments            list every comment with its thread and the text it is
                        anchored to
    accept              accept tracked changes (all, or the ones selected)
    reject              reject tracked changes (all, or the ones selected)
    diff-render         which pages of two documents look different; exit 5
                        when any does
    validate            Word-validity findings beyond the schema; exit 0
                        clean, 2 findings, 1 unreadable
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
                                [--revision-palette SPEC] [--pages SPEC]
                                [--move-comments] [--changed-only]
                                [--page {letter,a4}] [--reference-doc FILE]
                                [--track-changes {all,accept,reject}]
                                [--no-critic] [-a AUTHOR] [-d DATE]
                                file

positional arguments:
  file

options:
  -h, --help            show this help message and exit
  -o, --output OUTPUT   PDF path [default: <stem>.pdf beside the input;
                        <stem>.docx for Markdown]
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
  --pages SPEC          rasterize only these pages, counted from 1: 3, 1-3,7
                        (needs --png)
  --move-comments       list the comments after the last page instead of in
                        balloons beside the text
  --changed-only        keep only the pages a tracked change touches (--pages
                        counts the kept pages)
  --page {letter,a4}    Markdown: page size without --reference-doc
  --reference-doc FILE  Markdown: take styles and page setup from this .docx
  --track-changes {all,accept,reject}
                        Markdown: keep CriticMarkup as tracked changes, or
                        accept or reject them
  --no-critic           Markdown: read CriticMarkup delimiters as text
  -a, --author AUTHOR   Markdown: author of the tracked changes and comments
  -d, --date DATE       Markdown: their ISO-8601 date [default: fixed epoch]
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

#### `jubarte-redlines comments`

```text
$ jubarte-redlines comments --help
usage: jubarte-redlines comments [-h] [--json] [--author NAME] [--latest] file

positional arguments:
  file

options:
  -h, --help     show this help message and exit
  --json         one JSON object per line
  --author NAME  only this author's comments
  --latest       one comment per thread: the newest
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

#### `jubarte-redlines diff-render`

```text
$ jubarte-redlines diff-render --help
usage: jubarte-redlines diff-render [-h] [--dpi DPI] [--out-dir DIR] [--json]
                                    [--no-overlay] [--force]
                                    A B

positional arguments:
  A
  B

options:
  -h, --help     show this help message and exit
  --dpi DPI
  --out-dir DIR  write a-/b-/diff-page-NN.png for changed pages and diff.json
  --json         print diff.json instead of one line per changed page
  --no-overlay   skip the diff-page-NN.png overlays
  --force        overwrite files already in --out-dir
```

#### `jubarte-redlines validate`

```text
$ jubarte-redlines validate --help
usage: jubarte-redlines validate [-h] [--json] [--repair FILE]
                                 [--original FILE] [--author NAME] [--force]
                                 file

positional arguments:
  file

options:
  -h, --help       show this help message and exit
  --json           one JSON object per finding
  --repair FILE    write the repaired package here; remaining findings still
                   exit 2
  --original FILE  audit tracked edits: every text change against ORIGINAL
                   must be a revision by --author
  --author NAME
  --force          replace an existing --repair output
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

`diff(old, new)` (since 0.11.0) shows the changed paragraphs
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
Reflects the `jubarte_redlines` package built from this source tree (`__version__` reports `0.11.3`).

### `JubarteError`


Raised when the jubarte-redlines engine cannot process a document.

### `__version__`

`'0.11.3'`

### `accept_revisions`

```python
accept_revisions(docx)
```

Accept every tracked revision (package-wide) → clean DOCX bytes.

### `compare_documents`

```python
compare_documents(original, modified, author='jubarte', date=None, *, input_limits=None)
```

Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).

Mirrors `jubarte::document_comparer::compare_documents`; `date` (ISO-8601
`w:date` stamp) defaults to the engine's fixed epoch for deterministic
output. `input_limits` overrides the admission budget key by key
(`max_compressed_bytes`, `max_entries`, `max_part_bytes`,
`max_uncompressed_bytes`, `max_xml_depth`); a package past it raises
`JubarteError` with `INPUT_LIMIT`.

### `docx_to_pdf`

```python
docx_to_pdf(docx, compress=False, revisions='conventional', revision_palette=None, move_comments=False, changed_only=False)
```

Render a DOCX package (bytes) → PDF bytes (Word-style layout).

`compress=True` deflates the PDF's streams (`/FlateDecode`), which is much
smaller but no longer plain text. `revisions` paints tracked changes:
`"conventional"` (red struck deletions, blue underlined insertions, green
moves double-struck and double-underlined), `"word"` (Microsoft Word's
markup) or `"custom"` with
`revision_palette="deleted=#AA0000:strike,..."`. `move_comments=True`
lists the comments after the last page instead of in balloons beside the
text; `changed_only=True` keeps only the pages a tracked change touches
(a document without changes keeps its first page).

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
get_revisions_json(docx, *, input_limits=None)
```

List the tracked revisions in a DOCX as a JSON array string — the same
object shape as the CLI `jubarte revisions --json` lines
(`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
`input_limits` as in `compare_documents`.

### `reject_revisions`

```python
reject_revisions(docx)
```

Reject every tracked revision (package-wide) → base DOCX bytes.

### `Appended`

```python
Appended(
    document: Document,
    warnings: tuple[str, ...],
)
```

``Document.append``'s result: the joined document and what was not carried.

``warnings`` are ``CODE: message`` lines, such as
``COMMENTS_DROPPED: 1 comment of B was not carried``.

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

#### `Document.scrub`

```python
Document.scrub(self, *, author_alias: 'str | None' = 'Author', rsids: 'bool' = True, docprops: 'bool' = True, comments: 'bool' = True) -> 'Document'
```

Return a new document without who touched it.

``author_alias`` names every author (tracked changes, comments,
``people.xml``; ``None`` keeps the names), ``rsids`` drops the
edit-session ids, ``docprops`` the creator, last editor, revision
number, dates, manager, company and custom properties, and
``comments`` every comment. Text and tracked changes stay.

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

#### `Document.comments`

```python
Document.comments(self, *, author: 'str | None' = None, latest: 'bool' = False) -> 'tuple[Comment, ...]'
```

Every comment with its thread and anchored text, in document part order.

``author`` keeps one author's comments (exact match); ``latest`` keeps
the newest comment of each thread.

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

With ``existing_revisions="keep"`` another party's tracked changes stay
tracked: the clean copy is this document with the plan's edits applied
and theirs still tracked, the redline adds the plan's edits as new
revisions beside theirs, and ``diff`` shows the plan's edits only.

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

#### `Document.update_fields`

```python
Document.update_fields(self) -> 'UpdatedFields'
```

Refresh ``PAGEREF``, ``REF``, ``NUMPAGES``, ``SEQ`` and ``TOC`` results.

TOCs are rebuilt from the headings, then one layout pass gives the
page numbers: jubarte's layout, not Word's. Field codes stay, so Word
can update them again. This document is unchanged.

#### `Document.to_png`

```python
Document.to_png(self, *, dpi: 'float' = 96.0, options: 'PdfOptions | None' = None, pages: 'Sequence[int] | None' = None) -> 'tuple[bytes, ...]'
```

One PNG per page, straight from the layout (no PDF round trip).

``pages`` (counted from 1, any order, repeats ignored) rasterizes only
those pages, in ascending order, after one layout pass of the whole
document.

#### `Document.render`

```python
Document.render(self, *, pdf: 'bool' = True, png_dpi: 'float | None' = None, options: 'PdfOptions | None' = None, pages: 'Sequence[int] | None' = None) -> 'Rendered'
```

One layout pass: optional PDF, optional PNG pages, and the page report.

``pages`` (counted from 1) rasterizes only those pages; the report
still covers every page. A page past the end raises ``JubarteError``.

#### `Document.inspect_json`

```python
Document.inspect_json(self) -> 'str'
```

The engine's ``inspect`` snapshot as JSON text, unchanged (``inspect`` decodes it).

#### `Document.validate`

```python
Document.validate(self) -> 'tuple[Finding, ...]'
```

Word-validity findings beyond the schema; an empty tuple is a pass.

A package the engine cannot read at all raises ``JubarteError``.

#### `Document.repair`

```python
Document.repair(self) -> 'Repaired'
```

A copy with every repairable finding fixed, plus what was fixed and what remains.

#### `Document.audit_tracked`

```python
Document.audit_tracked(self, original: 'Document | bytes', *, author: 'str') -> 'tuple[Finding, ...]'
```

Every text change against ``original`` must be a revision by ``author``.

Rejecting that author's changes must give ``original``'s text back;
a paragraph that still differs is an ``UNTRACKED_EDIT`` finding and
another author's change a ``FOREIGN_AUTHOR`` one. An empty tuple
means every edit is tracked.

#### `Document.append`

```python
Document.append(self, other: 'Document', *, section_break: 'SectionBreak' = 'next_page', keep_sections: 'bool' = False, comments: 'AppendComments' = 'drop') -> 'Appended'
```

Put ``other`` after this document, carrying its parts.

Images, links, headers, styles, lists and notes come along under ids
that do not collide; a style this document already has (same type and
name) keeps this document's look. ``section_break="continuous"`` or
``"none"`` joins on the same page; ``keep_sections`` keeps ``other``'s
page setup, headers and footers as a section of its own. ``other``'s
comments are dropped and reported in ``warnings``; with
``comments="carry"`` those its body anchors come along with their
threads and resolution (those in notes, headers and footers are still
dropped and reported).

#### `Document.audit`

```python
Document.audit(self, rules: 'Sequence[str] | str | None' = None) -> 'tuple[AuditFinding, ...]'
```

Accessibility, style and structure findings, each located by
paragraph id. ``rules`` names rule sets (``a11y``, ``style``,
``structure``) or codes, as a sequence or a comma-separated string;
``None`` runs every rule. ``jubarte audit --help`` lists the codes.

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

### `Comment`

```python
Comment(
    id: int,
    author: str,
    initials: str | None,
    date: str | None,
    text: str,
    parent: int | None,
    done: bool,
    paragraph: str | None,
    anchor_text: str,
    before: str,
    after: str,
)
```

One comment with its thread position and the text it is anchored to.

    ``id`` is the comment's ``w:id``, which ``EditPlan.reply_comment``,
    ``resolve_comment``, ``edit_comment`` and ``delete_comment`` take.
    ``parent`` is the id of the comment it replies to (Word threads are one
    level deep); ``done`` is set when the thread is resolved. ``paragraph``
    is the paragraph id where the range starts (``body:p:12``);
    ``anchor_text`` is the commented text, paragraphs joined by ``
``, with
    up to 80 characters ``before`` and ``after`` it.

### `CompareOptions`

```python
CompareOptions(
    date: str | datetime | None = None,
    input_limits: Mapping[str, int] | None = None,
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

#### `CompareOptions.native_input_limits`

```python
CompareOptions.native_input_limits(self) -> 'dict[str, int] | None'
```

Return the overrides as the native ``input_limits`` dict.

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
    move_comments: bool = False,
    changed_only: bool = False,
)
```

PDF options with the same defaults as the current byte API.

``move_comments`` lists the comments after the last page instead of in
balloons beside the text; ``changed_only`` keeps only the pages a tracked
change touches. Both mirror the binary's flags and apply to PDF and PNG
output; ``diff_render`` ignores them.

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
    update_fields: bool = False,
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
EditPlan.for_document(self, document: 'Document | str') -> 'EditPlan'
```

Bind to ``document`` (a ``Document`` or a snapshot's hash string).

#### `EditPlan.replace`

```python
EditPlan.replace(self, paragraph: 'Selector', *, find: 'str', replacement: 'str', format: 'Mapping[str, object] | None' = None, comment: 'str | None' = None, whole: 'bool' = False, id: 'str | None' = None, occurrence: 'int | None' = None) -> 'EditPlan'
```

Replace the unique occurrence of ``find``, or its ``occurrence``-th hit (1-based).

``format`` styles only the new text. ``whole=True`` shows the change as
all of ``find`` deleted, then all of ``replacement`` inserted, instead
of Word Compare's word-level diff.

#### `EditPlan.insert`

```python
EditPlan.insert(self, paragraph: 'Selector', *, text: 'str', after: 'str | None' = None, before: 'str | None' = None, position: "Literal['start', 'end'] | None" = None, format: 'Mapping[str, object] | None' = None, comment: 'str | None' = None, id: 'str | None' = None, occurrence: 'int | None' = None) -> 'EditPlan'
```

Insert ``text`` after/before an anchor or at the paragraph edge; ``format`` styles it.

The anchor must be unique unless ``occurrence`` (1-based) picks one hit.

#### `EditPlan.delete`

```python
EditPlan.delete(self, paragraph: 'Selector', *, find: 'str', id: 'str | None' = None, occurrence: 'int | None' = None) -> 'EditPlan'
```

Delete the unique occurrence of ``find``, or its ``occurrence``-th hit (1-based).

#### `EditPlan.redact`

```python
EditPlan.redact(self, paragraph: 'Selector', *, find: 'str', id: 'str | None' = None, occurrence: 'int | None' = None) -> 'EditPlan'
```

Replace the unique occurrence of ``find`` (or its ``occurrence``-th hit) with one block per character.

The redaction is no tracked change: the clean copy and the redline
both show the blocks. The plan is refused with ``REDACTION_LEAK``
when the text still occurs anywhere in either document (another
paragraph, a comment, a header, the properties); the report never
repeats it.

#### `EditPlan.comment`

```python
EditPlan.comment(self, paragraph: 'Selector', *, text: 'str', find: 'str | None' = None, through: 'Selector | None' = None, id: 'str | None' = None, occurrence: 'int | None' = None) -> 'EditPlan'
```

Comment on the unique occurrence of ``find`` (or its ``occurrence``-th hit) or on
the whole paragraph; with ``through``, on every paragraph from ``paragraph`` to that one.

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

#### `EditPlan.insert_table`

```python
EditPlan.insert_table(self, paragraph: 'Selector', *, rows: 'Sequence[Sequence[str]]', position: "Literal['before', 'after']" = 'after', header_row: 'bool' = False, widths_dxa: 'Sequence[int] | None' = None, style: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Insert a table next to the anchor paragraph; the redline shows its
rows inserted.

``rows`` is the cell text row by row, every row the same length;
``widths_dxa`` the column widths in twentieths of a point (the text
width split evenly when omitted); ``style`` a table style id or name
(``TableGrid``, added when the document lacks it, by default).

#### `EditPlan.list_paragraphs`

```python
EditPlan.list_paragraphs(self, paragraphs: 'Sequence[Selector]', *, kind_of_list: "Literal['bullet', 'decimal', 'lower_letter']" = 'bullet', level: 'int' = 0, restart: 'bool' = True, id: 'str | None' = None) -> 'EditPlan'
```

Make the paragraphs a list (wire kind ``list``); the redline records
each paragraph's old properties.

``level`` is 0 (outermost) to 8. ``restart=False`` continues the list
of the nearest numbered paragraph before the first one instead of
starting a new one.

#### `EditPlan.reply_comment`

```python
EditPlan.reply_comment(self, comment_id: 'int', *, text: 'str', id: 'str | None' = None) -> 'EditPlan'
```

Reply to comment ``comment_id`` (``Document.comments`` lists the ids),
anchored on the same text; a reply to a reply joins the thread.

#### `EditPlan.resolve_comment`

```python
EditPlan.resolve_comment(self, comment_id: 'int', *, done: 'bool' = True, id: 'str | None' = None) -> 'EditPlan'
```

Resolve comment ``comment_id`` and its replies; ``done=False`` reopens them.

#### `EditPlan.edit_comment`

```python
EditPlan.edit_comment(self, comment_id: 'int', *, text: 'str', id: 'str | None' = None) -> 'EditPlan'
```

Replace the text of comment ``comment_id``; its author, date and thread stay.

#### `EditPlan.delete_comment`

```python
EditPlan.delete_comment(self, comment_id: 'int', *, id: 'str | None' = None) -> 'EditPlan'
```

Remove comment ``comment_id`` with its replies and anchors.

#### `EditPlan.watermark`

```python
EditPlan.watermark(self, text: 'str', *, color: 'str' = 'C0C0C0', diagonal: 'bool' = True, font: 'str' = 'Calibri', id: 'str | None' = None) -> 'EditPlan'
```

Write Word's own text watermark into every default header.

``text`` is 1 to 64 plain characters, ``color`` six hex digits, and
``diagonal=False`` lays it horizontal. One watermark per document; it
is header content, so the redline carries it without tracking it.

#### `EditPlan.fill_control`

```python
EditPlan.fill_control(self, control: 'ControlSelector', *, text: 'str | None' = None, choice: 'str | None' = None, checked: 'bool | None' = None, date: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Fill one content control with exactly one of ``text``, ``choice`` (a list
item's value or display text), ``checked`` or ``date`` (``YYYY-MM-DD``).

The control keeps its properties in the clean copy; the redline shows the
fill as tracked text (the comparer unwraps controls in revised paragraphs,
as Word Compare does).

#### `EditPlan.format_run`

```python
EditPlan.format_run(self, paragraph: 'Selector', *, find: 'str', format: 'Mapping[str, object]', occurrence: 'int | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Change the run formatting of ``find`` (bold, italic, underline,
highlight, font, size_pt, color, strike, caps) as a tracked property
change. ``occurrence`` (1-based) picks one of several matches.

#### `EditPlan.insert_footnote`

```python
EditPlan.insert_footnote(self, paragraph: 'Selector', *, after: 'str', text: 'str', occurrence: 'int | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Add a footnote holding ``text`` whose mark follows ``after`` in a
body paragraph. ``occurrence`` (1-based) picks one of several matches.

#### `EditPlan.insert_image`

```python
EditPlan.insert_image(self, paragraph: 'Selector', *, image: 'bytes', position: "Literal['before', 'after']" = 'after', content_type: 'str | None' = None, width_emu: 'int | None' = None, alt: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Insert a paragraph holding the picture ``image`` (PNG, JPEG, GIF,
BMP or TIFF bytes) next to a body paragraph. ``width_emu`` sets the
width (914400 per inch) and keeps the aspect ratio; by default the
picture is its pixel size at 96 dpi, at most 6.5 inches wide.

#### `EditPlan.page_setup`

```python
EditPlan.page_setup(self, *, section: "Literal['last', 'all']" = 'last', page: "Literal['letter', 'a4'] | Mapping[str, int] | None" = None, orientation: "Literal['portrait', 'landscape'] | None" = None, margins_dxa: 'Mapping[str, int] | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Set the page size, orientation and margins of the last section or
of every section, as a tracked section change. ``page`` is ``"letter"``,
``"a4"`` or ``{"width_dxa", "height_dxa"}``; ``margins_dxa`` takes any
of top, right, bottom, left, header, footer, in twentieths of a point
(1440 per inch).

#### `EditPlan.insert_toc`

```python
EditPlan.insert_toc(self, paragraph: 'Selector', *, position: "Literal['before', 'after']" = 'after', levels: 'int' = 3, title: 'str | None' = None, id: 'str | None' = None) -> 'EditPlan'
```

Insert a table of contents (``TOC \o "1-levels" \h \z \u``) next to
the anchor, after an optional ``TOCHeading`` title.

Its entries and page numbers are written when the plan sets
``update_fields=True``; page numbers come from jubarte's layout.

#### `EditPlan.settings`

```python
EditPlan.settings(self, *, track_revisions: 'bool | None' = None, update_fields: 'bool | None' = None, protection: 'ProtectionEdit | None' = None, enforcement: 'bool' = True, id: 'str | None' = None) -> 'EditPlan'
```

Write document settings, in schema order, into both documents.

``track_revisions`` turns Track Changes on or off, ``update_fields``
asks Word to update fields on open (``w:updateFields``; the plan's own
``update_fields`` writes jubarte's results instead), and ``protection`` restricts
editing (``"readOnly"``, ``"comments"``, ``"trackedChanges"``,
``"forms"``; ``"none"`` lifts it). The restriction has no password,
so any user can turn it off in Word. A setting left as ``None``
stays as it is; one ``settings`` per plan.

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
``REDACTION_LEAK``, ``UNSUPPORTED``, ``INVALID_PLAN``, ...), ``message`` the engine's
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
    fields: tuple[FieldUpdate, ...] = (),
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
    tables: tuple[Table, ...] = (),
    controls: tuple[ContentControl, ...] = (),
)
```

What ``Document.inspect()`` returns; the coordinates an ``EditPlan`` uses.

#### `Snapshot.control`

```python
Snapshot.control(self, id: 'str | None' = None, *, tag: 'str | None' = None, alias: 'str | None' = None) -> 'ContentControl'
```

The one control with this id (``body:sdt:N``), tag or alias; give exactly one.

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

### `ContentControl`

```python
ContentControl(
    id: str,
    kind: str,
    text: str,
    paragraph_ids: tuple[str, ...],
    locked: bool,
    placeholder: bool,
    tag: str | None = None,
    alias: str | None = None,
    choices: tuple[str, ...] = (),
    checked: bool | None = None,
)
```

A content control (``w:sdt``) in the body; ``EditPlan.fill_control`` fills it.

``kind`` is ``text``, ``rich_text``, ``drop_down``, ``combo_box``, ``date``,
``checkbox``, ``picture``, ``group``, ``repeating``, ``building_block``,
``citation``, ``bibliography``, ``equation`` or ``unknown``. ``paragraph_ids``
lists the paragraphs a block-level control spans, or the one holding a
run-level control. ``choices`` are the list values of a drop-down or combo
box; ``checked`` is a checkbox's state.

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

### `Table`

```python
Table(
    index: int,
    rows: tuple[tuple[TableCell, ...], ...],
    header_rows: int,
    widths_dxa: tuple[int, ...],
)
```

A body table as a grid; nested tables are separate entries.

``rows`` holds the cells as the XML has them (a merged cell is one cell),
``header_rows`` the leading rows that repeat as a header, ``widths_dxa``
the grid column widths in twentieths of a point (0 when unreadable).

### `TableCell`

```python
TableCell(
    paragraph_ids: tuple[str, ...],
    text: str,
)
```

A table cell: the ids of its own paragraphs and their text joined with ``\n``.

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
    substituted: bool = False,
)
```

One requested family/style and the physical face that painted it.

``substituted`` is true when the requested family was drawn with a
substitute (Word's substitution table, a bundled face of another family,
a generic family or the last resort); a faked style alone is
``synthetic``.

### `diff_render`

```python
diff_render(a: 'Document | bytes | str | os.PathLike[str]', b: 'Document | bytes | str | os.PathLike[str]', *, dpi: 'float' = 100.0, overlay: 'bool' = True, options: 'PdfOptions | None' = None) -> 'RenderDiff'
```

Which pages of Word documents ``a`` and ``b`` differ, pixel for
pixel, from one layout pass each at ``dpi``.

Each side is a ``Document``, Word ``bytes``, or a path (``str`` or
``os.PathLike``). ``overlay`` paints the changed pixels of each changed
page magenta over ``b``'s page and boxes them. ``options`` sets the
revision style both sides are painted with (``compress`` is ignored).

### `RenderDiff`

```python
RenderDiff(
    pages: tuple[PageDiff, ...],
    a: tuple[bytes, ...],
    b: tuple[bytes, ...],
    overlays: tuple[bytes | None, ...],
    a_report: RenderReport,
    b_report: RenderReport,
)
```

Output of ``diff_render``: one ``PageDiff`` per page of the longer
document, both sides' PNG pages, and per page diff ``b``'s page with the
change painted magenta and boxed (``None`` when the page is equal, on one
side only, a different size, or overlays were not asked for).

### `PageDiff`

```python
PageDiff(
    index: int,
    changed_ratio: float,
    bbox: tuple[int, int, int, int] | None,
    only_in: Literal['a', 'b'] | None = None,
)
```

How one page differs between the two sides of ``diff_render``.

``index`` is zero-based. ``changed_ratio`` is changed pixels over all
pixels (0.0 to 1.0; 1.0 when the page exists on one side only or the two
pages differ in size). ``bbox`` is ``(x0, y0, x1, y1)`` in pixels around
every changed pixel (``x1``/``y1`` exclusive), ``None`` when equal.
``only_in`` is ``"a"`` or ``"b"`` for a page only one side has.

### `Finding`

```python
Finding(
    code: str,
    part: str,
    path: str,
    message: str,
    word_fatal: bool,
    repairable: bool,
)
```

One thing wrong with a package, from ``Document.validate``.

``code`` is stable (``TEXT_INSIDE_DELETION``, ``MC_UNBOUND_PREFIX``,
``UNTRACKED_EDIT``, ...); ``part`` is the package part and ``path`` the
element chain inside it (``w:body[0]/w:p[3]/w:r[2]``, empty for a
package-level finding). ``word_fatal`` is true when Word refuses or
repairs the file for it, ``repairable`` when ``Document.repair`` fixes
it.

### `Repaired`

```python
Repaired(
    document: Document,
    repaired: tuple[Finding, ...],
    remaining: tuple[Finding, ...],
)
```

Output of ``Document.repair``: the repaired document, the findings it
fixed and the ones it could not.

### `FieldUpdate`

```python
FieldUpdate(
    kind: str,
    code: str,
    paragraph: str,
    old: str,
    new: str,
)
```

One field whose cached result was written from jubarte's layout.

### `UpdatedFields`

```python
UpdatedFields(
    document: Document,
    fields: tuple[FieldUpdate, ...],
    page_count: int,
)
```

``Document.update_fields``: the refreshed copy and what was written.

### `from_markdown`

```python
from_markdown(text: 'str', *, reference: 'Document | bytes | None' = None, page: "Literal['letter', 'a4']" = 'letter', author: 'str' = 'Redline', date: 'str | None' = None, critic: 'bool' = True, track_changes: "Literal['all', 'accept', 'reject']" = 'all') -> 'Document'
```

Write Markdown as a Word document, as ``jubarte convert draft.md``.

CriticMarkup (``{++ ++}``, ``{-- --}``, ``{~~ ~> ~~}``, ``{>> <<}``)
becomes tracked changes and comments by ``author`` at ``date`` (the
engine's fixed epoch by default), unless ``critic`` is false.
``track_changes`` keeps them (``all``) or writes the document with each
accepted or rejected. ``reference`` lends its styles and page setup;
without it ``page`` picks US Letter or A4, both with one-inch margins.
Engine warnings, such as a ``page`` the reference overrides, are raised
as ``UserWarning``. Images are written as their alt text.

### `AuditFinding`

```python
AuditFinding(
    code: str,
    rule_set: str,
    severity: str,
    location: str,
    message: str,
)
```

One ``Document.audit`` finding.

``code`` is the rule (``HEADING_SKIP``, ``IMAGE_NO_DESCR``...),
``rule_set`` is ``a11y``, ``style`` or ``structure``, ``severity`` is
``error``, ``warning`` or ``info``, and ``location`` is the paragraph id
(``body:p:N``, ``footer1:p:N``...) an edit plan targets, or a part name
for a document-wide finding.
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
