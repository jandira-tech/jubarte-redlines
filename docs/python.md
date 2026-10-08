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
Read, edit, compare and render Word documents

Usage: jubarte-redlines [OPTIONS] [ORIGINAL] [MODIFIED]
       jubarte-redlines <COMMAND>

Tasks:
  compare       Compare documents and write a Word redline [alias: redline]
  revisions     List the tracked revisions in a redline .docx
  changes       List tracked changes with IDs for accept, reject and edit plans
  accept        Accept all tracked changes, or select by ID, author or kind
  reject        Reject all tracked changes, or select by ID, author or kind
  convert       Convert Word or Markdown to DOCX, PDF, PNG or Markdown
  diff          Review differences as GitHub, word, normal, context or side-by-side text
  inspect       Inspect document facts, paragraphs, styles and tables
  text          Read Markdown with edit IDs `[body:p:N]`, or with tracked marks
  edit          Apply a JSON edit plan; write clean copy, redline and report (refusal:
                exit 3)
  capabilities  What this binary can do, for agents choosing an operation
  diff-render   Compare rendered pages pixel by pixel (different pages: exit 5)
  comments      List comments, threads and the text they annotate
  validate      Check or repair Word validity (findings: exit 2; unreadable: exit 1)
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Compare options:
  -b, --original <FILE>
          Original/base document (overrides the positional ORIGINAL)

  -m, --modified <FILE>
          Modified document (overrides the positional MODIFIED)

  -o, --output <FILE>
          Output path [default: <original-dir>/<original>_v_<modified>.docx]. A `.md`
          output writes the changes as CriticMarkup (both documents Markdown)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Redline]

  -d, --date <ISO8601>
          Revision timestamp (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          Do not print the success message

      --detail-threshold <RATIO>
          Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15]

      --mode <MODE>
          Compare like Microsoft Word or Open-Xml-PowerTools

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --powertools-faithful
          Same as --mode powertools

  [MODIFIED]
          The modified document (.docx or Markdown)

  [ORIGINAL]
          The original / base document (.docx or Markdown)

Examples:
  jubarte-redlines compare old.docx new.docx -o redline.docx
  jubarte-redlines inspect contract.docx --json
  jubarte-redlines diff old.docx new.docx --format github
  jubarte-redlines convert contract.docx -o contract.pdf

Run jubarte-redlines <task> --help for task options.
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
Compare documents and write a Word redline

Usage: jubarte-redlines compare [OPTIONS] [ORIGINAL] [MODIFIED]

Arguments:
  [ORIGINAL]
          The original / base document (.docx or Markdown)

  [MODIFIED]
          The modified document (.docx or Markdown)

Options:
  -b, --original <FILE>
          Original/base document (overrides the positional ORIGINAL)

  -m, --modified <FILE>
          Modified document (overrides the positional MODIFIED)

  -o, --output <FILE>
          Output path [default: <original-dir>/<original>_v_<modified>.docx]. A `.md`
          output writes the changes as CriticMarkup (both documents Markdown)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Redline]

  -d, --date <ISO8601>
          Revision timestamp (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          Do not print the success message

      --detail-threshold <RATIO>
          Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15]

      --mode <MODE>
          Compare like Microsoft Word or Open-Xml-PowerTools

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --powertools-faithful
          Same as --mode powertools

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte compare old.docx new.docx -o redline.docx
  jubarte compare -b old.docx -m new.docx --author Legal
```

#### `jubarte-redlines revisions`

```text
$ jubarte-redlines revisions --help
List the tracked revisions in a redline .docx

Usage: jubarte-redlines revisions [OPTIONS] <FILE>

Arguments:
  <FILE>  The redline document (.docx)

Options:
      --json  Emit the list as JSON lines instead of a human summary
  -h, --help  Print help
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
List tracked changes with IDs for accept, reject and edit plans

Usage: jubarte-redlines changes [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json  Emit one JSON object per line
  -h, --help  Print help
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
Accept all tracked changes, or select by ID, author or kind

Usage: jubarte-redlines accept [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to accept

Options:
  -o, --output <FILE>
          Output path

      --force
          Overwrite the output file if it already exists

      --id <ID>
          Only this change (`body:rev:12`, as `jubarte changes` lists it). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
Reject all tracked changes, or select by ID, author or kind

Usage: jubarte-redlines reject [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to reject

Options:
  -o, --output <FILE>
          Output path

      --force
          Overwrite the output file if it already exists

      --id <ID>
          Only this change (`body:rev:12`, as `jubarte changes` lists it). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines convert`

```text
$ jubarte-redlines convert --help
Convert Word or Markdown to DOCX, PDF, PNG or Markdown

Usage: jubarte-redlines convert [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document to convert: .docx, Markdown (.md, .markdown), or a Word 97-2003
          .doc (read into a .docx first)

Options:
  -o, --output <FILE>
          Output path [default: <stem>.pdf next to a .docx, <stem>.docx next to
          Markdown; Markdown output goes to stdout]. PNG pages are named
          <stem>-page-NN.png beside it

      --force
          Overwrite the output file if it already exists

      --pdf
          Write the PDF (the default when neither --pdf nor --png is given)

      --png
          Rasterize every page to PNG (<stem>-page-NN.png)

      --report <FILE>
          Write a JSON page report (`{page_count, pages:[{index,text}], fonts}`)

      --compress
          Deflate the PDF's streams (`/FlateDecode`). Much smaller output; the trade is
          that the page content is no longer plain text, so it cannot be read with
          `strings` or `grep`

      --font-report <FILE>
          Write a JSON font-resolution report (`[{requested, step, physical, bold,
          italic, synthetic, substituted}, …]`) for this document (plan Step 2f)

      --track-changes <CHOICE>
          Keep tracked changes (all), or write the document with every change accepted
          or rejected (pandoc's flag): CriticMarkup in Markdown, Word's revisions in a
          .docx. With --to md, the Markdown itself is resolved

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

          [default: all]

      --no-critic
          Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as text

      --reference-doc <FILE>
          Markdown to Word: take styles, numbering, page setup, headers and footers from
          this .docx (pandoc's --reference-doc)

      --resource-path <DIR>
          Markdown to Word: where images are found [default: the Markdown file's
          directory]

  -a, --author <NAME>
          Markdown to Word: author of the tracked changes and comments

          [default: Redline]

  -d, --date <ISO8601>
          Markdown to Word: their date (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --page <SIZE>
          Markdown to Word: the page size when there is no --reference-doc (one-inch
          margins either way); a reference's page setup wins

          Possible values:
          - letter: US Letter, 8.5 by 11 inches
          - a4:     ISO A4, 210 by 297 mm

          [default: letter]

      --no-page-markers
          Word to Markdown: leave out the `<!-- page N of M -->` lines, and the layout
          pass that places them

      --pages <SPEC>
          Rasterize only these pages, counted from 1: `3`, `1-3,7`. Layout still runs
          over the whole document. Needs PNG output

      --fail-on-substitution
          Exit 4 when a requested font was substituted (listed on stderr and in
          --report). Every output is still written. Exit status: 0 ok, 1 error, 4 a
          requested font was substituted

      --timeout <SECONDS>
          Give up after this many seconds: exit 124 (as `timeout(1)`) with nothing more
          written. An output being written at that moment may be left partial

  -h, --help
          Print help (see a summary with '-h')

Rendering:
      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted: `conventional` (deletions red struck through,
          insertions blue underlined, moves green: double-struck where they left,
          double-underlined where they landed), `word` (what Microsoft Word's Save as
          PDF paints), or `custom` (see --revision-palette)

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom: `kind=#RRGGBB[:lines],...` with kinds deleted,
          inserted, moved-from, moved-to and lines strike, double-strike, underline,
          double-underline, plain. Kinds left out keep their conventional mark

Formats:
  -f, --from <FORMAT>
          Input format [default: from the file: .md and .markdown are Markdown, a zip is
          Word]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

  -t, --to <FORMAT>
          Output format [default: from --output, else pdf for Word and docx for
          Markdown]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

Examples:
  jubarte convert contract.docx                   PDF, Word-style layout
  jubarte convert draft.md                        draft.docx, CriticMarkup as tracked
  changes
  jubarte convert draft.md -o draft.pdf           the changes painted in a PDF
  jubarte convert draft.md --reference-doc house.docx -o draft.docx
  jubarte convert draft.md -t md --track-changes accept   the text with every change
  accepted
  jubarte convert contract.docx -t md             Markdown with <!-- page N of M -->
  lines
  jubarte convert old.doc                         old.docx (text, headings, lists,
  tables)
  jubarte convert notes.md --no-critic            {++ and the other delimiters as text
```

#### `jubarte-redlines diff`

```text
$ jubarte-redlines diff --help
Review differences as GitHub, word, normal, context or side-by-side text

Usage: jubarte-redlines diff [OPTIONS] <OLD> <NEW>

Arguments:
  <OLD>
          The old document: .docx or Markdown

  <NEW>
          The new document: .docx or Markdown

Options:
  -h, --help
          Print help (see a summary with '-h')

Review:
  -o, --output <FILE>
          Write to FILE. Text views default to stdout and write only text. Patch/critic
          infer Word, Markdown, PDF or PNG from the extension

      --format <FORMAT>
          Choose the review view; word accepts ALL input changes first

          Possible values:
          - patch:        The changed paragraphs, as `git diff --word-diff` with
            CriticMarkup comments and highlights
          - critic:       CriticMarkup: current document text with tracked marks
          - github:       Git/GitHub unified text; preserves each document's tracked
            marks
          - word:         Fresh word-level CriticMarkup after accepting ALL changes in
            both inputs
          - normal:       Normal diff with line addresses and no context (a/d/c, < and
            >)
          - context:      Context diff with old/new ranges and !, + and - prefixes
          - side-by-side: Old and new lines in parallel columns, with |, < and > markers

          [default: patch]

  -U, --context <LINES>
          Unchanged lines around GitHub or context hunks; -U0 shows changes only

          [default: 3]

      --accept-changes
          Accept both documents' changes before comparing. Word format always does this

      --full-lines
          Show complete lines instead of a 70-character window around changes

      --force
          Overwrite the output file if it already exists

Paragraph patch:
      --columns <N>
          Wrap the patch's lines at this many columns; 0 does not wrap

          [default: 72]

Formats:
  -t, --to <FORMAT>
          Output format, when --output does not say

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

  -f, --from <FORMAT>
          Input format of both documents [default: from each file]

          [possible values: docx, md, markdown]

Word redline:
  -a, --author <NAME>
          Who made the changes: the patch's owner and the revisions' author [default:
          `git config user.name`, else Redline]

  -d, --date <ISO8601>
          When (ISO 8601) [default: now]; pin it for reproducible output

      --mode <MODE>
          Whose redline to reproduce (see `jubarte --help`)

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --detail-threshold <RATIO>
          LCS detail threshold (see `jubarte --help`)

      --reference-doc <FILE>
          Two Markdown documents written as Word take styles, page setup, headers and
          footers from this .docx

      --critic
          Read CriticMarkup in the Markdown documents as tracked changes (Word output).
          By default a document compared is text

      --resource-path <DIR>
          Where images named by the Markdown are found [default: each Markdown file's
          directory]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted in PDF or PNG output (see `convert --help`)

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom (see `convert --help`)

Examples:
  jubarte diff old.docx new.docx --format github   Git/GitHub patch on stdout
  jubarte diff old.md new.md                       the patch on stdout
  jubarte diff old.md new.md --format critic       CriticMarkup on stdout, as pandiff
  jubarte diff old.md new.md -o changes.docx       Word tracked changes
  jubarte diff old.md new.md -o changes.pdf        the changes painted in a PDF
  jubarte diff contract.docx edited.md -o redline.docx
      the Markdown's edits as tracked changes on the Word document

GIT:
  git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
  git difftool -t jubarte -y -- '*.md'
```

#### `jubarte-redlines inspect`

```text
$ jubarte-redlines inspect --help
Inspect document facts, paragraphs, styles and tables

Usage: jubarte-redlines inspect [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx) to read

Options:
      --json    Emit the snapshot as JSON (`schema_version`, `source_sha256`, `summary`,
                `paragraphs`, `stories`, `tables`) instead of a human summary
      --tables  Print each body table as a grid instead of the paragraphs: a `table N:
                ROWSxCOLS header_rows=H widths=W,...` line, then one line per row of
                tab-separated `ids=text` cells
  -h, --help    Print help
```

#### `jubarte-redlines text`

```text
$ jubarte-redlines text --help
Read Markdown with edit IDs `[body:p:N]`, or with tracked marks

Usage: jubarte-redlines text [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) to read

Options:
      --track-changes <CHOICE>
          Print the document as Markdown with its tracked changes as CriticMarkup (all),
          or with every change accepted or rejected, like `convert --to md`. The output
          then has no `[body:p:N]` ids

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
Apply a JSON edit plan; write clean copy, redline and report (refusal: exit 3)

Usage: jubarte-redlines edit [OPTIONS] --plan <PLAN.json> --out-dir <DIR> <FILE>

Arguments:
  <FILE>
          The source document (.docx). Never modified

Options:
      --plan <PLAN.json>
          Edit plan JSON (see `jubarte capabilities --json` for the kinds)

      --out-dir <DIR>
          Directory to create for clean.docx, redline.docx, report.jsonl

      --dry-run
          Resolve and report only; write nothing

      --force
          Replace an existing output directory's files

      --pdf
          Also write redline.pdf and clean.pdf

      --png
          Also write redline-page-NN.png and clean-page-NN.png

  -q, --quiet
          Print nothing on success (patch.diff and report.jsonl are still written)

  -h, --help
          Print help (see a summary with '-h')

Rendering:
      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted in the redline PDF/PNG

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom (see `convert --help`)
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte-redlines capabilities [OPTIONS]

Options:
      --json  Emit JSON (the default output is JSON too; the flag documents intent)
  -h, --help  Print help
```

#### `jubarte-redlines diff-render`

```text
$ jubarte-redlines diff-render --help
Compare rendered pages pixel by pixel (different pages: exit 5)

Usage: jubarte-redlines diff-render [OPTIONS] <A> <B>

Arguments:
  <A>  The document before
  <B>  The document after

Options:
      --out-dir <DIR>  Write the changed pages' PNGs and diff.json here (created if
                       missing)
      --json           Print diff.json to stdout instead of one line per changed page
      --no-overlay     Skip the diff-page-NN.png overlays
      --force          Overwrite files already in --out-dir
  -h, --help           Print help

Rendering:
      --dpi <DPI>  Raster resolution of both sides in dots per inch (1-1200) [default:
                   100]

Examples:
  jubarte diff-render before.docx after.docx                  changed pages on stdout
  jubarte diff-render before.docx after.docx --out-dir diff   PNGs of the changed pages
  and diff.json
  jubarte diff-render a.docx b.docx --json                    the diff.json document on
  stdout

With --out-dir, each page that differs is written as a-page-NN.png,
b-page-NN.png and diff-page-NN.png (b's page with the changed pixels
magenta and boxed); diff.json lists every page with its changed_ratio,
bbox and, for a page only one side has, only_in.
```

#### `jubarte-redlines comments`

```text
$ jubarte-redlines comments --help
List comments, threads and the text they annotate

Usage: jubarte-redlines comments [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json           Emit one JSON object per line
      --author <NAME>  Only this author's comments (exact match)
      --latest         One comment per thread: the newest
  -h, --help           Print help
```

#### `jubarte-redlines validate`

```text
$ jubarte-redlines validate --help
Check or repair Word validity (findings: exit 2; unreadable: exit 1)

Usage: jubarte-redlines validate [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json             JSON Lines: one object per finding, nothing when there is none
      --repair <FILE>    Write the repaired package here; remaining findings still exit
                         2
      --original <FILE>  Audit tracked edits: every text change against ORIGINAL must be
                         a revision by --author
      --author <NAME>    The author every change must carry (with --original)
      --force            Replace an existing --repair output
  -h, --help             Print help

Examples:
  jubarte validate contract.docx
  jubarte validate contract.docx --json
  jubarte validate contract.docx --repair fixed.docx
  jubarte validate review/redline.docx --original contract.docx --author Claude
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
Document.diff(self, other: 'Document | str', *, author: 'str | None' = None, date: 'str | None' = None, columns: 'int' = 72, format: "Literal['patch', 'critic', 'github', 'unified', 'text', 'word', 'normal', 'context', 'side-by-side']" = 'patch', context: 'int' = 3, accept_changes: 'bool' = False, full_lines: 'bool' = False) -> 'Diff'
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
diff(old: 'Document | bytes | str | os.PathLike[str]', new: 'Document | bytes | str | os.PathLike[str]', *, author: 'str | None' = None, date: 'str | None' = None, columns: 'int' = 72, format: "Literal['patch', 'critic', 'github', 'unified', 'text', 'word', 'normal', 'context', 'side-by-side']" = 'patch', context: 'int' = 3, accept_changes: 'bool' = False, full_lines: 'bool' = False) -> 'Diff'
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
critic`` does. ``github`` (aliases ``unified`` and ``text``) gives a Git
unified text patch with ``context`` unchanged lines around each hunk,
preserving existing tracked marks and every document story. It has no
paragraph hunks and does not look up an author or timestamp. ``word``
accepts both inputs' revisions before creating new CriticMarkup;
``normal``, ``context`` and ``side-by-side`` show traditional text diffs.
Other views preserve revisions unless ``accept_changes=True``. Views
use the core's 70-character display window; ``full_lines=True`` disables it.

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
