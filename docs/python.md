<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Python guide

Revision tags use the [settled Markdown dialect](MARKDOWN.md#tags): separate ids,
same-author deletion-first substitutions, old-side `re` parents and inline `¶` marks.

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
`changes`, `accept`, `reject`, `convert`, `diff`, `read`, `edit`, `add`,
`capabilities`, `visual-diff`, `comments`, `validate` — but not the whole
Rust binary surface: it has no `append`, `scrub`, `debug` or `self-update`, and
its `convert` refuses `--from`, `--resource-path`, `--timeout`,
`--fail-on-substitution` and `--no-page-markers`. Exit codes: 0 success, 1
error, 2 usage (and `validate` findings), 3 edit plan refused, 5
`visual-diff` found a page that differs. Inputs are `.docx`; save a Word 97-2003 `.doc` as
`.docx` first.

## CLI reference

Generated from the real `--help` output of the wheel's `jubarte-redlines`
entry point — flags and defaults cannot drift from the shipped code.

The native `jubarte` binary offers `--keep-nested-comment` on `accept`,
`reject` and `convert --track-changes accept|reject`. These bindings do not
yet dispatch that option or return its kept-thread receipts; use the native
binary when retention is needed.

<!-- gen:cli-python:start -->
#### `jubarte-redlines`

```text
$ jubarte-redlines --help
Read, edit, compare and render Word documents

Usage: jubarte-redlines [OPTIONS]
       jubarte-redlines <COMMAND>

Tasks:
  compare       Compare documents: print the redline as the agent view, or write it with
                -o [alias: redline]
  changes       List tracked changes with IDs for accept, reject and edit plans
  accept        Accept all tracked changes, or select by ID, author or kind
  reject        Reject all tracked changes, or select by ID, author or kind
  convert       Convert Word or Markdown to DOCX, PDF, PNG or Markdown
  diff          Review differences as GitHub, word, normal, context or side-by-side text
  read          Read the agent view: YAML header, `<!-- pN -->` id lines, changes and
                comments with ids
  edit          Edit a document: -p WHERE with --anchor, --content, --delete, --resolve,
                --reply, --accept, --reject, --strike, --restore or --style (several -p
                per command), or a JSON plan. Writes FILE_modified.docx (--out-dir: the
                bundle), then prints SUCCESS and a fenced before/after diff of the full
                agent views (refusal: exit 3)
  add           Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several
                -p per command). Writes FILE_modified.docx, as edit does, and prints
                SUCCESS and a fenced before/after diff of the full agent views
  capabilities  What this binary can do, for agents choosing an operation
  visual-diff   Compare rendered pages pixel by pixel (different pages: exit 5)
  comments      List comments, threads and the text they annotate
  validate      Check Word validity, tracked edits, accessibility, style and structure;
                or repair Word validity
  help          Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet    No status on success: no `wrote` lines, SUCCESS block or edit summary.
                 Files are still written; a reading task (`read`, `changes`, `compare`
                 without -o, `diff`, `capabilities`) still prints what it read, and
                 warnings and errors still print. `read`, `changes` and `comments` drop
                 their one stderr line naming the input's validity findings; under
                 --json and read --for-edit the output counts them, -q or not
  -h, --help     Print help
  -V, --version  Print version

Examples:
  jubarte-redlines compare old.docx new.docx -o redline.docx
  jubarte-redlines contract.docx
  jubarte-redlines read contract.docx --json
  jubarte-redlines diff old.docx new.docx --format github
  jubarte-redlines convert contract.docx -o contract.pdf

Run jubarte-redlines <task> --help for task options.
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
Compare documents: print the redline as the agent view, or write it with -o

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
          Write the redline to FILE instead of printing its agent view; the extension
          picks the format: .docx (also any other), .md (the agent view; CriticMarkup
          when both documents are Markdown), .pdf, .png (one file per page) or .json
          (the snapshot)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --force
          Overwrite the output file if it already exists

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

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Markdown inputs:
      --reference-doc <FILE>
          Two Markdown documents written as Word take styles, page setup, headers and
          footers from this .docx

      --critic
          Read CriticMarkup in the Markdown documents as tracked changes. By default a
          document compared is text

      --resource-path <DIR>
          Where images named by the Markdown are found (default: the original's
          directory when it is Markdown, else the modified's). Only images that resolve
          inside that directory are read, symlinks resolved; a URL or a path out of it
          is written as its alt text, with a warning that says why

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)

Read options:
      --track-changes <CHOICE>
          Tracked changes inline (all, the default), or the text with every change
          accepted or rejected (also spelled accepted, rejected); the id lines then list
          what changed. --json and --tables read the accepted text, or under reject the
          rejected one, every paragraph keeping its id; the snapshot names an accept or
          a reject in `track_changes`

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

      --comments <MODE>
          Comments inline (default) or hidden, with their ids on the id line of the
          paragraph that holds them; ignored by --json, --for-edit and the --tables
          grids

          Possible values:
          - inline: Comments inline, with their ids
          - none:   Comments hidden; their ids on the id lines

          [default: inline]

      --dates
          Timestamps on the notes of an author whose changes do not all share one (the
          header shows an author's single timestamp); ignored by --json, --for-edit and
          the --tables grids

      --no-page-markers
          Skip the layout pass; page count from Word's cached breaks, no `<!-- page N of
          M -->` lines; ignored by --json, --for-edit and the --tables grids

  -p, --paragraphs <SPEC>
          Only these blocks: `p5`, `p4-p7`, `t0`, comma-separated (open-ended `p17-` /
          `-p3` spellings still parse, transitionally; the 17 October 2026 contract
          retires them — spell the end, `p17-p20`). With --tables, select top-level
          tables (including their nested tables) by the same tN as the view, or by a pN
          inside them; a pick that reaches no table is refused. Print their grid, then a
          blank line and the same view as read -p SPEC. Repeat it to add blocks: `-p p3
          -p p8` reads as `-p p3,p8`

          [alias: --select]

      --head <N>
          Only the first N blocks (a table is one block)

      --tail <N>
          Only the last N blocks

      --changed
          Only the blocks with a tracked change or a comment

      --by <AUTHOR>
          With --changed: only the blocks with this author's marks (a handle such as AC,
          or the full name)

      --json
          Print the snapshot as JSON instead of the view: `schema_version`,
          `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`,
          `changes` and `comments`. -p, --head, --tail, --changed and --by keep the body
          paragraphs they select, and every change and comment that starts in one of
          them, whoever made it; the rest of the snapshot stays whole. --track-changes
          accept or reject settles the text

      --tables
          Print each top-level body table as a grid: a `table N: ROWSxCOLS header_rows=H
          widths=W,...` line, then one line per row of tab-separated `ids=text` cells.
          Use -p t0,t2 or -p pN (a cell paragraph) to select top-level tables and all
          their nested tables, in document order, each once. tN is the view's top-level
          index; nested entries follow their cell row, indented two spaces per level,
          labelled `nested in t0.row2.cell0` (recursively, e.g.
          `t0.row2.cell0.t0.row0.cell1`). A pN or a range selects the tables that hold
          its paragraphs; one that reaches no table is refused. With -p, follow the
          selected grids with a blank line and the same view as read -p SPEC, including
          its YAML header. Without -p, print only the inventory. --track-changes reject
          reads the rejected text

      --for-edit
          The working document template/apply derive, as JSON: source hash, paired
          target ids, exact logical working text with `before_sha256` and readiness per
          operation and range. The selection (-p/--select, --head, --tail, --changed,
          --by, --find, --regex) resolves on the original source and freezes before the
          decisions settle it. Given filters narrow together: a paragraph is selected
          only when every filter given matches it, and an empty result names the filters
          applied in `coverage.omissions`

      --accept-change <ID>
          --for-edit: the working view after accepting this revision (repeatable; it
          does not modify FILE)

      --reject-change <ID>
          --for-edit: the working view after rejecting this revision (repeatable; it
          does not modify FILE)

      --existing-revisions <MODE>
          --for-edit: the remaining-revision policy of the working basis (default keep,
          matching new plans)

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

      --track <CHOICE>
          --for-edit: readiness for tracked (on, the default) or direct (off) new edits

          Possible values:
          - on:  The new edits are tracked
          - off: The new edits land directly

      --find <TEXT>
          --for-edit: literal search of the original final text. Combined filters
          narrow: a paragraph must match every filter given, and an empty result names
          the filters applied

      --regex <PATTERN>
          --for-edit: regex search of the original final text; conflicts with --find

      --ignore-case
          --for-edit: case-insensitive --find/--regex

      --context <N>
          --for-edit: this many read-only original-source neighbours around each row

      --limit <N>
          --for-edit: primary records a page returns (default 100)

      --max-chars <N>
          --for-edit: the text budget a page shares (default 24000); a truncated row's
          preview is never complete text

      --cursor <TOKEN>
          --for-edit: continue an earlier bounded query (bound to its source, selection,
          filters and working basis)

      --all
          --for-edit: no presentation limits (admission limits still hold)

Examples:
  jubarte compare old.docx new.docx                 the redline's agent view
  jubarte compare old.docx new.docx -o redline.docx
  jubarte compare old.docx new.docx -o redline.pdf --changed-pages
  jubarte compare old.md new.md -o changes.md       CriticMarkup
  jubarte compare -b old.docx -m new.docx --author Legal -o r.docx
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
List tracked changes with IDs for accept, reject and edit plans.

One line per logical change: the marks Word wrote for one edit are one change with a
joined id (`r7+8+9+10`), its text followed by `(N elements)`.

Usage: jubarte-redlines changes [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx)

Options:
      --json
          Emit one JSON object per line

      --author <NAME>
          Only listed changes by this author (exact match); repeat for several.
          Comment-body revisions are not listed, even without this filter

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
Accept all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. An author or
kind selection reaches comment bodies too, but no ID names a comment-body change, so an
ID selection leaves those tracked.

Usage: jubarte-redlines accept [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to accept

Options:
  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, or `r1+2` (both). Repeatable. Comment-body changes have no ids
          and stay tracked

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
Reject all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. An author or
kind selection reaches comment bodies too, but no ID names a comment-body change, so an
ID selection leaves those tracked.

Usage: jubarte-redlines reject [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to reject

Options:
  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, or `r1+2` (both). Repeatable. Comment-body changes have no ids
          and stay tracked

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)
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
          Output path; its extension picks the format: .docx, .md, .pdf, .png (pages
          named `<stem>-page-NN.png` beside it) or .json (the snapshot of the result).
          Without it: `<stem>.pdf` next to a .docx, `<stem>.docx` next to Markdown,
          `<stem>.json` for `--to json`; Markdown output goes to stdout

      --force
          Overwrite the output file if it already exists

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

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed; requires
          --track-changes accept or reject

      --no-critic
          Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as text

      --reference-doc <FILE>
          Markdown to Word: take styles, numbering, page setup, headers and footers from
          this .docx (pandoc's --reference-doc)

      --resource-path <DIR>
          Markdown to Word: where images are found [default: the Markdown file's
          directory]. Only images that resolve inside that directory are read, symlinks
          resolved; a URL or a path out of it is written as its alt text, with a warning
          that says why

  -a, --author <NAME>
          Markdown to Word: author of the tracked changes and comments that name none
          [default: the front matter's `author:`, else the --reference-doc's creator,
          else Jubarte]

  -d, --datetime <ISO8601>
          Markdown to Word: timestamp (ISO 8601) of the tracked changes and comments
          that carry none [default: the front matter's
          `last_modified:`/`modified:`/`date:`/`created:`, else the --reference-doc's
          creation time, else now]; pin it (`-d 1970-01-01T00:00:00Z`) for reproducible
          output

      --page <SIZE>
          Markdown to Word: the page size when there is no --reference-doc (one-inch
          margins either way); a reference's page setup wins

          Possible values:
          - letter: US Letter, 8.5 by 11 inches
          - a4:     ISO A4, 210 by 297 mm

          [default: letter]

      --tracking <SWITCH>
          Markdown to Word: turn Word's Track Changes on or off in the .docx written,
          whatever the Markdown's `track_changes:` key says [default: the key; without
          one, the --reference-doc's setting, else off]

          Possible values:
          - on:  New edits in Word are tracked
          - off: New edits in Word are not tracked

      --scholar
          Markdown to Word: set a manuscript as LaTeX's article class does (Latin Modern
          Roman 10pt on 12pt lines, justified, bold headings) and number its pages from
          page 2, centered in the footer. The front matter's own `styles:` entries,
          headers and footers win

      --no-page-markers
          Word to Markdown: leave out the `<!-- page N of M -->` lines, and the layout
          pass that places them

      --commonmark
          Word to Markdown: write plain CommonMark with CriticMarkup for readers outside
          jubarte's dialect, instead of the agent view (`jubarte read`'s YAML header and
          id lines). Fields print as their text, and underline, highlight and strike are
          left out

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Formats:
  -f, --from <FORMAT>
          Input format [default: from the file: .md and .markdown are Markdown, a zip is
          Word]

          [possible values: docx, md, markdown]

  -t, --to <FORMAT>
          Output format [default: from --output, else pdf for Word and docx for
          Markdown]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages
          - json: The JSON snapshot (`read --json`'s object): written, never read.
            `diff` refuses it (`json_not_yet`)

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)

      --fail-on-substitution
          Exit 4 when a requested font was substituted (listed on stderr and in
          --report). Every output is still written. Exit status: 0 ok, 1 error, 4 a
          requested font was substituted

      --update-fields
          Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC fields from
          jubarte's layout (TOCs rebuilt from the headings) in the .docx written, after
          --track-changes. Field codes stay, so Word can update them again; page numbers
          are jubarte's layout, not Word's (docs/WORD_DIFFERENCES.md). Prints one line
          per field; --report writes `{page_count, fields}`

      --timeout <SECONDS>
          Give up after this many seconds: exit 124 (as `timeout(1)`) with nothing more
          written. An output being written at that moment may be left partial

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
  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Review:
  -o, --output <FILE>
          Write the view to FILE (.md, .markdown, .txt, .patch or .diff; stdout
          otherwise)

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

  -a, --author <NAME>
          Who made the changes: the patch's owner

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

Formats:
  -t, --to <FORMAT>
          Output format: md only (the patch, CriticMarkup or a text view are all text)

  -f, --from <FORMAT>
          Input format of both documents [default: from each file]

          [possible values: docx, md, markdown]

Examples:
  jubarte diff old.docx new.docx --format github   Git/GitHub patch on stdout
  jubarte diff old.md new.md                       the patch on stdout
  jubarte diff old.md new.md --format critic       CriticMarkup on stdout, as pandiff
  jubarte diff old.docx new.docx -o changes.patch  the patch in a file
  jubarte compare old.md new.md -o changes.docx    Word tracked changes (compare writes
  redlines)
  jubarte compare contract.docx edited.md -o redline.pdf
      the Markdown's edits painted on the Word document

GIT:
  git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
  git difftool -t jubarte -y -- '*.md'
```

#### `jubarte-redlines read`

```text
$ jubarte-redlines read --help
Read the agent view: YAML header, `<!-- pN -->` id lines, changes and comments with ids

A YAML header, then Markdown with an `<!-- pN -->` id line before every paragraph,
tracked changes as CriticMarkup followed by their ids (`{++text++}{r12 @AC}`) and
comments with theirs (`{>>c5 @AC: …<<}`).

Usage: jubarte-redlines read [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) to read

Options:
  -o, --output <FILE>
          Write the view to FILE.md, or the snapshot to FILE.json, instead of printing
          it

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Read options:
      --track-changes <CHOICE>
          Tracked changes inline (all, the default), or the text with every change
          accepted or rejected (also spelled accepted, rejected); the id lines then list
          what changed. --json and --tables read the accepted text, or under reject the
          rejected one, every paragraph keeping its id; the snapshot names an accept or
          a reject in `track_changes`

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

      --comments <MODE>
          Comments inline (default) or hidden, with their ids on the id line of the
          paragraph that holds them; ignored by --json, --for-edit and the --tables
          grids

          Possible values:
          - inline: Comments inline, with their ids
          - none:   Comments hidden; their ids on the id lines

          [default: inline]

      --dates
          Timestamps on the notes of an author whose changes do not all share one (the
          header shows an author's single timestamp); ignored by --json, --for-edit and
          the --tables grids

      --no-page-markers
          Skip the layout pass; page count from Word's cached breaks, no `<!-- page N of
          M -->` lines; ignored by --json, --for-edit and the --tables grids

  -p, --paragraphs <SPEC>
          Only these blocks: `p5`, `p4-p7`, `t0`, comma-separated (open-ended `p17-` /
          `-p3` spellings still parse, transitionally; the 17 October 2026 contract
          retires them — spell the end, `p17-p20`). With --tables, select top-level
          tables (including their nested tables) by the same tN as the view, or by a pN
          inside them; a pick that reaches no table is refused. Print their grid, then a
          blank line and the same view as read -p SPEC. Repeat it to add blocks: `-p p3
          -p p8` reads as `-p p3,p8`

          [alias: --select]

      --head <N>
          Only the first N blocks (a table is one block)

      --tail <N>
          Only the last N blocks

      --changed
          Only the blocks with a tracked change or a comment

      --by <AUTHOR>
          With --changed: only the blocks with this author's marks (a handle such as AC,
          or the full name)

      --json
          Print the snapshot as JSON instead of the view: `schema_version`,
          `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`,
          `changes` and `comments`. -p, --head, --tail, --changed and --by keep the body
          paragraphs they select, and every change and comment that starts in one of
          them, whoever made it; the rest of the snapshot stays whole. --track-changes
          accept or reject settles the text

      --tables
          Print each top-level body table as a grid: a `table N: ROWSxCOLS header_rows=H
          widths=W,...` line, then one line per row of tab-separated `ids=text` cells.
          Use -p t0,t2 or -p pN (a cell paragraph) to select top-level tables and all
          their nested tables, in document order, each once. tN is the view's top-level
          index; nested entries follow their cell row, indented two spaces per level,
          labelled `nested in t0.row2.cell0` (recursively, e.g.
          `t0.row2.cell0.t0.row0.cell1`). A pN or a range selects the tables that hold
          its paragraphs; one that reaches no table is refused. With -p, follow the
          selected grids with a blank line and the same view as read -p SPEC, including
          its YAML header. Without -p, print only the inventory. --track-changes reject
          reads the rejected text

      --for-edit
          The working document template/apply derive, as JSON: source hash, paired
          target ids, exact logical working text with `before_sha256` and readiness per
          operation and range. The selection (-p/--select, --head, --tail, --changed,
          --by, --find, --regex) resolves on the original source and freezes before the
          decisions settle it. Given filters narrow together: a paragraph is selected
          only when every filter given matches it, and an empty result names the filters
          applied in `coverage.omissions`

      --accept-change <ID>
          --for-edit: the working view after accepting this revision (repeatable; it
          does not modify FILE)

      --reject-change <ID>
          --for-edit: the working view after rejecting this revision (repeatable; it
          does not modify FILE)

      --existing-revisions <MODE>
          --for-edit: the remaining-revision policy of the working basis (default keep,
          matching new plans)

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

      --track <CHOICE>
          --for-edit: readiness for tracked (on, the default) or direct (off) new edits

          Possible values:
          - on:  The new edits are tracked
          - off: The new edits land directly

      --find <TEXT>
          --for-edit: literal search of the original final text. Combined filters
          narrow: a paragraph must match every filter given, and an empty result names
          the filters applied

      --regex <PATTERN>
          --for-edit: regex search of the original final text; conflicts with --find

      --ignore-case
          --for-edit: case-insensitive --find/--regex

      --context <N>
          --for-edit: this many read-only original-source neighbours around each row

      --limit <N>
          --for-edit: primary records a page returns (default 100)

      --max-chars <N>
          --for-edit: the text budget a page shares (default 24000); a truncated row's
          preview is never complete text

      --cursor <TOKEN>
          --for-edit: continue an earlier bounded query (bound to its source, selection,
          filters and working basis)

      --all
          --for-edit: no presentation limits (admission limits still hold)
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
Edit a document: -p WHERE with --anchor, --content, --delete, --resolve, --reply,
--accept, --reject, --strike, --restore or --style (several -p per command), or a JSON
plan. Writes FILE_modified.docx (--out-dir: the bundle), then prints SUCCESS and a
fenced before/after diff of the full agent views (refusal: exit 3)

Usage: jubarte-redlines edit [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Never modified

Options:
      --plan [<PLAN.json>]
          Edit plan JSON: a list of operations of any kind (see `jubarte capabilities
          --json`). Alone, it is applied to FILE; with operation flags, they are
          recorded into it (created on first use) and FILE is left as it is. Without a
          value: FILE_plan.json beside FILE

      --consume
          Delete PLAN.json once it has been applied and the outputs are written (with
          --plan and no operation flags). Kept when --skip-failed left operations out

  -p, --location <WHERE>
          Where: p12, header1, footer2.p1, t0.row1.cell2, c5 (a comment), c5.text (the
          text c5 covers) or r12 (a revision, with --accept, --reject, --strike or
          --restore). Each -p starts an operation; the flags after it belong to it

      --anchor <TEXT>
          Text inside WHERE the operation applies to (must occur once)

      --content <TEXT>
          New text: replaces the anchor; rewrites the paragraph without one; on c5, the
          comment's new text

      --delete
          Delete the anchor, or the whole paragraph (or comment) without one

      --resolve
          Resolve comment c5 (with -p c5)

      --reply <TEXT>
          Reply to comment c5 (with -p c5), as `add -p c5 --content`

      --rewrite-others-comments
          Let `-p c5 --content` change a comment another author wrote; it keeps their
          name. Without it that is refused (COMMENT_NOT_YOURS): one author answers
          another with --reply

      --accept
          Accept revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`accepted r1+2 in p3: "from" > "to"`)

      --reject
          Reject revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`rejected r0 in p3: "from" > "to"`)

      --strike
          Strike another author's insertion r12 (with -p r12): our tracked deletion
          inside it, so the decision shows. --content is a comment on the struck words

      --restore
          Restore another author's deletion r12 (with -p r12): their deletion stays and
          our tracked insertion of its words follows it. --content is a comment on the
          restored words

      --visible
          With --reject: strike each insertion and restore each deletion instead of
          removing the change without a trace

      --style <SPEC>
          Formatting for the anchored text: bold, italic, underline, strike, caps,
          highlight=yellow, font=Calibri, size=11, color=FF0000; any other value is a
          paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own)

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --suggesting-mode
          The edits are tracked changes (the default): the file written is the redline
          and the view shows the marks

      --editing-mode
          The edits land directly: the file written is the result, and the view shows it
          with rev tags on the id lines

      --existing-revisions <MODE>
          What to do when FILE already holds tracked changes: auto keeps them and tracks
          the new edits beside them; a clean file goes through the comparer

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

          [default: auto]

  -o, --output <FILE.docx>
          The edited document, a .docx (default: `<FILE's
          directory>/<stem>_modified.docx`). Any other extension is written as a .docx
          too; .md, .json, .pdf and .png name the task that writes them

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

      --out-dir [<DIR>]
          Write the bundle instead: clean.docx, redline.docx, patch.diff and
          report.jsonl (clean.docx and report.jsonl in editing mode) in DIR (default:
          `<FILE's directory>/<stem>.edit`)

      --force
          Replace an existing output (the bundle directory's files)

      --dry-run
          Resolve and report only; write nothing. When every operation passes, stderr
          ends with the command that applies it (`next:`)

      --skip-failed
          Apply every operation that passes; leave out the ones that fail and every
          operation of their "group". Exit 0 when one landed. The failed ones go to
          PLAN.failed.json (FILE_plan.failed.json for operation flags): a plan to fix
          and run on the output

      --require-change
          Refuse when the edit would change nothing in the document
          (NO_CHANGES_REQUESTED); reports, renders and bundles do not count as a change

      --pdf
          Also write the PDF beside the document (the bundle: redline.pdf and clean.pdf)

      --png
          Also write PNG pages beside the document, STEM-page-NN.png (the bundle:
          redline-page-NN.png and clean-page-NN.png)

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

Examples:
  jubarte edit a.docx -p p12 --anchor "thirty days" --content "forty-five days"
  jubarte edit a.docx -p p12 --anchor thirty --content "thirty (30)"   an insertion
  jubarte edit a.docx -p p7 --anchor "at its sole discretion" --delete -p p9 --delete
  jubarte edit a.docx -p p3 --content "The parties agree as follows."    rewrite
  jubarte edit a.docx -p p4 --anchor Fees --style bold -p p5 --style Heading2
  jubarte edit a.docx -p c5 --content "Agreed." -p c7 --resolve
  jubarte edit a.docx -p c5 --reply "Agreed, will fix."
  jubarte edit a.docx -p c5.text --content "net 30"   the text c5 covers
  jubarte edit a.docx -p r3 --accept -p header1.r1 --reject
  jubarte edit a.docx -p p3 --delete -o b.docx --pdf   b.docx and b.pdf
  jubarte edit a.docx --plan plan.json --out-dir review   the bundle in review/
  jubarte edit a.docx --plan plan.json -p p3 --content "New."   record into plan.json
  jubarte edit a.docx --plan plan.json --consume   apply plan.json, then delete it

WHERE is an id from `jubarte read`: p12, header1, footer2.p1,
t0.row1.cell2, c5 (a comment), c5.text (the text it covers) or r12
(a revision).

A plan (--plan plan.json) is a list of operations, applied all or none;
`jubarte capabilities --json` lists every operation kind:
  {"schema_version": 1, "source_sha256": "<sha256 from jubarte read>", "author":
  "Claude",
   "operations": [
    {"kind": "replace", "paragraph": "p1", "find": "his or her", "replacement":
    "their"},
    {"kind": "delete_paragraph", "paragraph": "p2"}]}

Typed actions: jubarte edit replace|rewrite|insert|delete FILE [OPTIONS]
  Inline --save-plan FILE saves without applying; writes require -o or --out-dir.
  Run jubarte edit ACTION --help for its grammar.
Routing: valid legacy calls, including `edit replace -p p0 --content TEXT`,
  retain v1 semantics. Use ./replace (or ./rewrite, ./insert, ./delete, ./apply)
  to explicitly name a legacy input whose name equals an action.
```

#### `jubarte-redlines add`

```text
$ jubarte-redlines add --help
Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several -p per command).
Writes FILE_modified.docx, as edit does, and prints SUCCESS and a fenced before/after
diff of the full agent views

Usage: jubarte-redlines add [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Never modified

Options:
      --plan [<PLAN.json>]
          Edit plan JSON. With operation flags, they are recorded into it (created on
          first use) and FILE is left as it is; alone, it is applied to FILE, as `edit
          --plan` does. Without a value: FILE_plan.json beside FILE

      --consume
          Delete PLAN.json once it has been applied and the outputs are written (with
          --plan and no operation flags)

  -p, --location <WHERE>
          Where: p12, header1, footer2.p1, t0.row1.cell2, or c5 for a reply. Each -p
          starts an operation; the flags after it belong to it

      --anchor <TEXT>
          Text inside WHERE to comment on (must occur once)

      --content <TEXT>
          The paragraph, comment or reply text

      --reply <TEXT>
          Reply to comment c5 (with -p c5); the same as --content there

      --before
          A new paragraph before WHERE instead of after it

      --comment
          A comment on the whole paragraph (with --anchor the comment sits on the anchor
          and this flag is implied)

      --style <SPEC>
          Formatting for the new paragraph's text (bold, italic, underline,
          highlight=yellow) or its paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own)

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --suggesting-mode
          The edits are tracked changes (the default): the file written is the redline
          and the view shows the marks

      --editing-mode
          The edits land directly: the file written is the result, and the view shows it
          with rev tags on the id lines

      --existing-revisions <MODE>
          What to do when FILE already holds tracked changes: auto keeps them and tracks
          the new edits beside them; a clean file goes through the comparer

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

          [default: auto]

  -o, --output <FILE.docx>
          The edited document, a .docx (default: `<FILE's
          directory>/<stem>_modified.docx`). Any other extension is written as a .docx
          too; .md, .json, .pdf and .png name the task that writes them

      --out-dir [<DIR>]
          Write the bundle instead: clean.docx, redline.docx, patch.diff and
          report.jsonl (clean.docx and report.jsonl in editing mode) in DIR (default:
          `<FILE's directory>/<stem>.edit`)

      --force
          Replace an existing output (the bundle directory's files)

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte add a.docx -p p12 --content "Time is of the essence."   new paragraph after
  p12
  jubarte add a.docx -p p12 --content Recitals --before --style Heading2
  jubarte add a.docx -p p5 --anchor "monthly fee" --content "Net of taxes?"   a comment
  jubarte add a.docx -p p5 --comment --content "Whole clause needs a cap."
  jubarte add a.docx -p c5 --content "Agreed, will fix."           a reply
  jubarte add a.docx -p c5 --reply "Agreed, will fix."             the same reply
  jubarte add a.docx --plan plan.json -p p5 --comment --content "Cap?"   record into
  plan.json
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte-redlines capabilities [OPTIONS]

Options:
      --json                   The manifest or focused answer as JSON instead of text
      --operation <OPERATION>  Discover one edit kind, including its fields and
                               execution conditions
      --field <FIELD>          Discover one field, optionally within --operation
      --matrix                 Publish operation × target × policy edit support
  -q, --quiet                  No status on success: no `wrote` lines, SUCCESS block or
                               edit summary. Files are still written; a reading task
                               (`read`, `changes`, `compare` without -o, `diff`,
                               `capabilities`) still prints what it read, and warnings
                               and errors still print. `read`, `changes` and `comments`
                               drop their one stderr line naming the input's validity
                               findings; under --json and read --for-edit the output
                               counts them, -q or not
  -h, --help                   Print help
```

#### `jubarte-redlines visual-diff`

```text
$ jubarte-redlines visual-diff --help
Compare rendered pages pixel by pixel (different pages: exit 5)

Usage: jubarte-redlines visual-diff [OPTIONS] <A> <B>

Arguments:
  <A>  The document before
  <B>  The document after

Options:
      --out-dir <DIR>  Write the changed pages' PNGs and diff.json here (created if
                       missing)
      --json           Print diff.json to stdout instead of one line per changed page
      --no-overlay     Skip the diff-page-NN.png overlays
      --force          Overwrite files already in --out-dir
  -q, --quiet          No status on success: no `wrote` lines, SUCCESS block or edit
                       summary. Files are still written; a reading task (`read`,
                       `changes`, `compare` without -o, `diff`, `capabilities`) still
                       prints what it read, and warnings and errors still print. `read`,
                       `changes` and `comments` drop their one stderr line naming the
                       input's validity findings; under --json and read --for-edit the
                       output counts them, -q or not
  -h, --help           Print help

Rendering:
      --dpi <DPI>  Raster resolution of both sides in dots per inch (1-1200) [default:
                   100]

Examples:
  jubarte visual-diff before.docx after.docx                  changed pages on stdout
  jubarte visual-diff before.docx after.docx --out-dir diff   PNGs of the changed pages
  and diff.json
  jubarte visual-diff a.docx b.docx --json                    the diff.json document on
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
  -q, --quiet          No status on success: no `wrote` lines, SUCCESS block or edit
                       summary. Files are still written; a reading task (`read`,
                       `changes`, `compare` without -o, `diff`, `capabilities`) still
                       prints what it read, and warnings and errors still print. `read`,
                       `changes` and `comments` drop their one stderr line naming the
                       input's validity findings; under --json and read --for-edit the
                       output counts them, -q or not
  -h, --help           Print help
```

#### `jubarte-redlines validate`

```text
$ jubarte-redlines validate --help
Check Word validity, tracked edits, accessibility, style and structure; or repair Word
validity.

A word or edits finding fails the run (exit 2), and so does any error; --strict fails on
warnings too. With --baseline, only the findings the baseline lacks count. An unreadable
file exits 1.

Usage: jubarte-redlines validate [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx)

Options:
      --json
          JSON Lines: one object per finding, nothing when there is none

      --rules <RULES>
          Rule sets or audit codes, comma-separated [default: word; with --original, or
          --baseline and --author, word and edits]

      --strict
          Also fail (exit 2) on warnings from other rule sets; word and edits findings
          fail without it

      --repair <FILE>
          Write the repaired package here; remaining findings still fail.

          A structural repair: it does not check the edits' history. Add --original and
          --author for that.

      --baseline <FILE>
          The document the edits started from: its findings print as already in the
          input (not yours), and only new ones fail the run. A finding is new by
          identity (code, part, place and message), never by count. With --author and no
          --original, the edits are checked against it too

      --original <FILE>
          Check tracked edits: every text change against ORIGINAL must be a revision by
          --author

      --author <NAME>
          The author every change must carry (with --original or --baseline)

      --force
          Replace an existing --repair output

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their one stderr line
          naming the input's validity findings; under --json and read --for-edit the
          output counts them, -q or not

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte validate contract.docx
  jubarte validate contract.docx --json
  jubarte validate contract.docx --repair fixed.docx
  jubarte validate review/redline.docx --original contract.docx --author Claude
  jubarte validate review/redline.docx --baseline contract.docx
  jubarte validate contract.docx --rules word,a11y,style,structure --strict

Rule sets: word (default: Word validity), edits (with --author, and --original or
--baseline), a11y, style, structure.
Audit codes (code, set, severity):
  HEADING_SKIP a11y warning, IMAGE_NO_DESCR a11y error,
  TABLE_NO_HEADER_ROW a11y warning, MISSING_LANG a11y warning,
  LITERAL_BULLET style warning, EMPTY_SPACER_PARAGRAPH style info,
  DIRECT_FORMATTING_OVERRIDES_STYLE style info,
  STALE_FIELD_CACHE structure warning, FONT_SUBSTITUTED structure info
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

`compare_documents(original, modified, author="Redline", date=None)` stamps a
fixed epoch date by default so output is deterministic; pass ISO-8601 `date`
to override. Errors raise `jubarte_redlines.JubarteError`.

### `Document` — immutable snapshot, path I/O once

```python
from jubarte_redlines import read

doc = read("contract.docx")
doc.markdown()    # the body as Markdown, with <!-- pN --> id lines
doc.inspect()     # Snapshot: summary + paragraphs (ids, style, spans, limitations)
doc.sha256()      # the source_sha256 guard an edit plan carries
doc.changes()     # each tracked change with the id accept/reject and plans take
doc.accept(ids=["r12"])                      # or doc.reject(...): resolve a
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
    .replace("p3", find="30 days", replacement="45 days", format={"bold": True})
    .comment("p3", text="Check with the client", find="45 days")
    .insert_paragraph("p3", runs=["New sentence."], like="p3")
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
marks (`format="critic"` for CriticMarkup). When a side is a Word document,
the comparer writes the changes, and a replacement inside a paragraph keeps
Word Compare's order, the insertion first: `{+new+}[-old-]`.

```python
from jubarte_redlines import diff, read
print(diff(read("v1.docx"), read("v2.docx")))
```

## API reference

The snapshot uses `schema_version: 3`. `changes` lists logical changes:
a change Word wrote as several marks is one entry with its `members`,
and a deletion followed by its author's insertion is one `replacement`.
`summary.tables` and `tables[]`
count top-level body tables only; each `index` is the view's `tN`.
Every table has a `nested` array (empty for a leaf), containing its cell tables.
A nested entry has `parent` (for example `t0.row2.cell0`) and no `index`.
A deeper parent includes the cell-local table ordinal, for example
`t0.row2.cell0.t0.row0.cell1`. Paragraph ids and edit-plan schemas are unchanged.

`--tables` numbers only top-level tables as `table N`, matching `tN` in
the view and `-p tN`. Nested tables follow their containing cell row,
indented two spaces per nesting level and labelled `nested in CELL_PATH`.
For example, `nested in t0.row2.cell0`; a further level uses
`t0.row2.cell0.t0.row0.cell1`. `--tables -p t0` includes this whole tree.

Generated by reflecting over the installed package, in `__all__` order —
signatures, dataclass fields and docstrings below are the shipped ones.

The identity fields the models carry retain their existing wire shape: `id` and
`ref` both hold the short id (`r12`, `header1.r1`, `c5` — the colon
fallback `body:rev:007` only when no short spelling reads back; `9872a374`,
`52e96ced`), comment ids are numeric and `Snapshot.schema_version == 3` for logical changes over the table-tree shape.
The 17 October 2026 short-and-long-ID contract
([superpowers/specs/2026-10-17-short-and-long-id-contract.md](superpowers/specs/2026-10-17-short-and-long-id-contract.md),
staged in
[superpowers/plans/2026-10-16-id-contract-implementation.md](superpowers/plans/2026-10-16-id-contract-implementation.md))
replaces them with paired `short_id`/`long_id` references at the planned identity-schema
cutover and retires the transitional spellings — `noid`, colon ids
(`body:rev:12`), nested `.pN` inside cells and text boxes
(`t0.row1.cell2.p1`), open ranges (`p17-`, `-p3`). It is not implemented:
the fields the wire carries today are the ones named above.

The OPC constructor enforces `InputLimits::compare()`:
512 MiB compressed, 10,000 declared ZIP entries, 512 MiB per inflated part,
and 2 GiB inflated in total. `scrub` and the maintainer `debug` command now
refuse packages over those ceilings with `INPUT_LIMIT`; Markdown reads use
the same bounded constructor. Entry counts are checked before indexing,
and actual inflation is capped even when ZIP size declarations lie.
`scrub` still applies its existing validation limits as well. A leak scan
that exceeds a budget or cannot read a part reports a finding; editing a
redaction therefore refuses with `REDACTION_LEAK` instead of accepting an
incomplete scan. Comparer settings continue to supply their own limits.

<!-- gen:python-api:start -->
Reflects the `jubarte_redlines` package built from this source tree (`__version__` reports `0.14.0`).

### `JubarteError`


Raised when the jubarte-redlines engine cannot process a document.

### `__version__`

`'0.14.0'`

### `accept_revisions`

```python
accept_revisions(docx)
```

### `compare_documents`

```python
compare_documents(original, modified, author='Redline', date=None, *, input_limits=None)
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
text; `changed_only=True` keeps only the pages a tracked change touches,
the pages a comment is anchored on and each section's first page on which
a revised header or footer paints (else the first page).

### `get_revisions`

```python
get_revisions(docx: 'bytes') -> 'list[dict[str, Any]]'
```

List the tracked revisions in a DOCX as parsed objects.

Grouped as Docxodus's ``GetRevisions`` groups them; each item has
(``type``/``author``/``date``/``part``/``moveGroupId``/``isMoveSource``/
``formatChange``/``text``).

### `get_revisions_json`

```python
get_revisions_json(docx, *, input_limits=None)
```

List the tracked revisions in a DOCX as a JSON array string, grouped as
Docxodus's `GetRevisions` groups them; each object has
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
Document.accept(self, *, ids: 'Sequence[str] | None' = None, authors: 'Sequence[str] | None' = None, kinds: 'Sequence[FilterKind] | None' = None) -> 'Document'
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
Document.reject(self, *, ids: 'Sequence[str] | None' = None, authors: 'Sequence[str] | None' = None, kinds: 'Sequence[FilterKind] | None' = None) -> 'Document'
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

The body as Markdown with a ``[pN]`` id before each paragraph.

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

``pages`` (document pages, counted from 1) writes only those pages:
the PDF, the PNGs and the report. A page ``changed_only`` dropped, or
one past the end, raises ``JubarteError``.

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
a paragraph that still differs is an ``UNTRACKED_EDIT`` finding.
A foreign revision added, changed or removed against ``original`` is a
``FOREIGN_AUTHOR`` finding. Existing foreign revisions may remain;
paragraphs align by full text, including deleted text, within each story.
An untracked paragraph merge or split around another author's revisions
reports those revisions as new and gone alongside ``UNTRACKED_EDIT``.
Groups with the same author, kind, target, date and move side are compared
in document order within the aligned paragraph or non-paragraph owner
(for example, a style id). Revision ids do not define identity. Split
carriers combine only when they touch, with no visible content or
still-present revision between them. A boundary explained solely by a
missing separator revision adds no finding beyond that separator. Changed
revisions say ``differs from the original``; new revisions say ``is new``.
These messages name ids and authors, never revision text. A permission
endpoint of ``original`` that the edit lost, in any story, is a
``PERMISSION_RANGE_DROPPED`` finding.
An empty tuple means every edit is tracked.

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
``None`` runs every rule. ``jubarte validate --help`` lists the codes.

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
its ``pN`` id in a Word document or ``line:N`` in Markdown, with
``[-old-]{+new+}`` changes and CriticMarkup comments. When a side is a
Word document, a replacement inside a paragraph reads insertion first,
``{+new+}[-old-]``, as Word Compare writes it.

Each side is a ``Document`` or Word ``bytes``, Markdown text (``str``),
or a path (``.md``/``.markdown`` read as Markdown, anything else as
Word). ``author`` and ``date`` own the changes, shown once in the
header [default: Redline; now].
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
``{>>comments<<}``. When a side is a Word document, a replacement
inside a paragraph reads insertion first, ``{+new+}[-old-]``, as Word
Compare writes it.

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

``at`` is where it is: ``p12`` (or a header's, footer's or note's
``header1``, ``header1.p2``, ``footnotes.p3``...) in a Word document, as
``Document.markdown`` and edit plans number paragraphs, or ``line:N`` in
Markdown; in the new version, or the old one when ``removed``. ``text``
is the paragraph with its changes, unwrapped.

### `capabilities`

```python
capabilities(*, operation: 'str | None' = None, field: 'str | None' = None, matrix: 'bool' = False) -> 'dict[str, object]'
```

Task manifest, focused edit syntax, or support matrix for this Python build.

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
    ref: str | None = None,
    paragraph: str | None = None,
    holds_comments: tuple[str, ...] = (),
    members: tuple[ChangeMember, ...] = (),
)
```

One tracked change with the id ``Document.accept`` / ``reject`` select by.

``id`` is the change's short id (``r12``, ``header1.r3``), built from
its ``w:id``, so resolving some changes never renumbers the rest. ``target`` is what the
change applies to (``text``, ``paragraph_mark``, ``table_row``,
``table_cell``, ``properties``). Both sides of a move share
``move_name`` and resolve together; ``move_side`` is ``from`` or ``to``.
``inside`` names the change whose content holds this one: resolving that
one so its content goes takes this one along. ``ref`` equals ``id``: the
short id the CLI prints and ``-p``/``--id`` take;
``paragraph`` is the id of the paragraph the change is in (``p3``).
``holds_comments`` names the comments (``c4``, a reply's ``c5``) whose
reference mark the change holds: resolving it so its content goes
removes their threads, as Word does. A change Word wrote as several
marks (one typed insertion split by session, or a deletion with the
insertion that replaces it, kind ``replacement``) has the composite id
``r7+8+9+10`` and lists its marks in ``members``; selecting only some of
them is refused.

### `ChangeMember`

```python
ChangeMember(
    id: str,
    kind: ChangeKind,
    target: str,
    date: str | None,
    text: str,
)
```

One mark of a change Word wrote as several: its own ``id`` (``r7``),
``kind`` (``insertion`` or ``deletion``), ``target`` (``text`` or
``paragraph_mark``), ``date`` and ``text``.

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
    ref: str | None = None,
    last_paragraph: str | None = None,
)
```

One comment with its thread position and the text it is anchored to.

    ``id`` is the comment's ``w:id``, which ``EditPlan.reply_comment``,
    ``resolve_comment``, ``edit_comment`` and ``delete_comment`` take.
    ``parent`` is the id of the comment it replies to (Word threads are one
    level deep); ``done`` is set when the thread is resolved. ``paragraph``
    is the paragraph id where the range starts (``p12``), and
    ``last_paragraph`` the one where it ends when that is a later one;
    ``anchor_text`` is the commented text, paragraphs joined by ``
``, with
    up to 80 characters ``before`` and ``after`` it. ``ref`` is the short id
    the CLI prints and ``-p`` takes (``c5``).

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
change touches, the pages a comment is anchored on, and the first page of
each section on which a revised header or footer paints. Both mirror the binary's flags and apply to PDF and PNG
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
    rewrite_others_comments: bool = False,
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

Another author's comment is refused (``COMMENT_NOT_YOURS``) unless the
plan sets ``rewrite_others_comments``.

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
    anchor_given: str | None = None,
    anchor_read_as: str | None = None,
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
    notes: tuple[str, ...] = (),
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
    replaced: int = 0,
)
```

The redline's changes by kind, as ``changes`` lists them.

``replaced`` counts replacements: a deletion and the insertion that
replaces it, listed as one change. ``total`` is the sum of the others.

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
    changes: tuple[Change, ...] = (),
    comments: tuple[Comment, ...] = (),
)
```

What ``Document.inspect()`` returns; the coordinates an ``EditPlan`` uses.

``changes`` and ``comments`` are the tracked changes and comments, as
``Document.changes()`` and ``comments()`` list them and ``read --json``
prints them.

#### `Snapshot.control`

```python
Snapshot.control(self, id: 'str | None' = None, *, tag: 'str | None' = None, alias: 'str | None' = None) -> 'ContentControl'
```

The one control with this id (``p3.cc0``, ``cc1``), tag or alias; give exactly one.

#### `Snapshot.paragraph`

```python
Snapshot.paragraph(self, id_or_index: 'str | int') -> 'Paragraph'
```

The paragraph with this id (``p12``, ``header1``, ``header1.p2``) or body index.

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
    index: int | None,
    rows: tuple[tuple[TableCell, ...], ...],
    header_rows: int,
    widths_dxa: tuple[int, ...],
    parent: str | None = None,
    nested: tuple[Table, ...] = (),
)
```

A table grid. Top-level ``index`` matches the view's ``tN``.

Nested tables have ``index=None`` and a ``parent`` cell path, and live
recursively in ``nested``. Only top-level tables count in ``Summary.tables``.

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

One paragraph (``p12``, ``header1``, ``header1.p2``...); ``index``/``id`` are valid for this snapshot only.

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
    document_pages: int = 0,
)
```

Page count, page text and font resolutions from one layout pass.

### `PageText`

```python
PageText(
    index: int,
    number: int,
    text: str,
)
```

Text painted on one page, one line per baseline.

``index`` counts the report's pages from zero; ``number`` is the page's
number in the document, which differs once ``changed_only`` drops pages.

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
    set: str,
    severity: str,
    part: str,
    location: str,
    message: str,
    word_fatal: bool,
    repairable: bool,
    ref: str | None = None,
)
```

One thing wrong with a package, from ``Document.validate``.

``code`` is stable (``TEXT_INSIDE_DELETION``, ``MC_UNBOUND_PREFIX``,
``UNTRACKED_EDIT``, ...); ``set`` is its rule set (``word``, ``edits``,
``a11y``, ``style``, ``structure``) and ``severity`` ``error``,
``warning`` or ``info``. ``part`` is the package part and ``location``
the element chain inside it (``w:body[0]/w:p[3]/w:r[2]``), a paragraph
id (``p3``), or empty for a package-level finding. ``ref`` is the
short id the CLI prints for ``location`` (``p3`` for ``p3``).
``word_fatal`` is true when Word refuses or repairs the file for it,
``repairable`` when ``Document.repair`` fixes it.

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
from_markdown(text: 'str', *, reference: 'Document | bytes | None' = None, page: "Literal['letter', 'a4']" = 'letter', author: 'str | None' = None, date: 'str | None' = None, critic: 'bool' = True, track_changes: "Literal['all', 'accept', 'reject']" = 'all', tracking: 'bool | None' = None, scholar: 'bool' = False) -> 'Document'
```

Write Markdown as a Word document, as ``jubarte convert draft.md``.

CriticMarkup (``{++ ++}``, ``{-- --}``, ``{~~ ~> ~~}``, ``{>> <<}``)
becomes tracked changes and comments, unless ``critic`` is false. One
that names no author or date takes ``author`` and ``date`` when given,
else the front matter's (``author:``; ``last_modified:``,
``modified:``, ``date:``, ``created:``), else ``Redline`` and the
engine's fixed epoch (the CLI dates them now instead).
``track_changes`` keeps them (``all``) or writes the document with each
accepted or rejected. ``reference`` lends its styles and page setup;
without it ``page`` picks US Letter or A4, both with one-inch margins.
``tracking`` turns Word's Track Changes on or off in the document,
whatever the front matter's ``track_changes:`` says (``None`` follows
it). ``scholar`` sets the document as LaTeX's article class does (Latin
Modern Roman 10pt, justified, bold headings) and numbers its pages from
page 2, as ``--scholar``; the front matter's own styles and footers win.
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
(``p12``, ``footer1``, ``footer1.p2``...) an edit plan targets, or a part name
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
sections mirror
[`jubarte-python/README.fragment.md`](../jubarte-python/README.fragment.md),
from which `scripts/library_readmes.py` writes the README that ships inside
the package — update both when you change an example; nothing syncs this
page with the fragment.
