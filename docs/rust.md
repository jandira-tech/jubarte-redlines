<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Rust guide

Revision tags use the [settled Markdown dialect](MARKDOWN.md#tags): separate ids,
same-author deletion-first substitutions, old-side `re` parents and inline `¶` marks.

Using jubarte from Rust: the library crate and the `jubarte` CLI. For the
other surfaces see [docs/python.md](python.md) (Python and `uvx`) and
[docs/javascript.md](javascript.md) (Node, browser and `npx`); the
[README](../README.md) covers the shared workflow and install options.

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

## Install

The published crate is [`jubarte-redlines`](https://crates.io/crates/jubarte-redlines);
the library import path is `jubarte`:

```sh
cargo add jubarte-redlines --no-default-features   # library only
cargo install jubarte-redlines                      # CLI (binary: jubarte)
```

API reference: [docs.rs/jubarte-redlines](https://docs.rs/jubarte-redlines)
(built with `--all-features`). Per-release flattened API snapshots live in
[docs/api/](api/).

### Features

| Feature | Default | Adds |
|---|---|---|
| `cli` | yes | the `jubarte` binary (clap) |
| `fast-alloc` | yes | mimalloc allocator behind the CLI |
| `self-update` | yes | `jubarte self-update` |
| `perf-profile` | no | profiling instrumentation |

Library consumers should disable defaults and opt in deliberately. MSRV:
**Rust 1.94** (`#![warn(missing_docs)]` is enforced; every public item is
documented).

## Sixty seconds

```rust
use jubarte::{convert, document_comparer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = std::fs::read("original.docx")?;
    let modified = std::fs::read("modified.docx")?;

    let redline =
        document_comparer::compare_documents(&original, &modified, "Reviewer")?;
    std::fs::write("redline.docx", &redline)?;

    let pdf = convert::docx_to_pdf(&redline)?;
    std::fs::write("redline.pdf", &pdf)?;

    Ok(())
}
```

## API tour

| Module | Use it for | Key items |
|---|---|---|
| `jubarte::document_comparer` | compare two DOCX into tracked changes | `compare_documents`, `compare_documents_with_options`, `compare_documents_with_settings`, `get_revisions`, `accept_revisions`, `reject_revisions`, `revisions_to_json` |
| `jubarte::comparer` | comparison tuning | `WmlComparerSettings` (incl. `detail_threshold`, `detect_moves`, `merge_replaced_paragraphs`), `WmlComparerSettings::powertools_faithful()`, `compare_bodies*` |
| `jubarte::changes` | list / selectively resolve changes | `list_changes`, `accept_changes`, `reject_changes`, `ChangeFilter` (constructor `ChangeFilter::ids(..)`, public `ids`/`authors`/`kinds` fields) |
| `jubarte::convert` | render DOCX | `docx_to_pdf`, `docx_to_pdf_with(PdfOptions)`, `docx_to_pdf_report`, `docx_to_png`, `render`, `pdf_page_count`, `font_report_json`, `RevisionStyle`, `RevisionPalette` |
| `jubarte::inspect` | document snapshot for agents | `paragraphs`, `summary`, `stories`, `controls`, `markdown`, `inspect_json`, `Snapshot`, `Paragraph`, `Span`, `Story`, `SCHEMA_VERSION` |
| `jubarte::edit` | atomic JSON edit plans | `apply_plan`, `apply_plan_json`, `EditPlan::from_json`/`to_json` |
| `jubarte::markdown` | Markdown in and out | `markdown_to_docx`, `docx_to_markdown`, `diff_markdown`, `patch_documents`, `patch_redline`, `redline`, `RedlineOptions` |
| `jubarte::validate` | will Word open it, and repair | `validate`, `repair`, `audit_tracked`, `Finding` |
| `jubarte::audit` | accessibility, style and structure findings | `audit`, `audit_report`, `audit_report_with`, `AuditReport` |
| `jubarte::scrub` | remove authors and metadata before sending | `scrub`, `ScrubOptions`, `leaks` |
| `jubarte::append` | one document after another | `append_documents`, `AppendOptions`, `SectionBreak`, `AppendComments` |
| `jubarte::comments` | comment threads | `list_comments`, `CommentRecord` |
| `jubarte::fields` | refresh a TOC and other fields | `update_fields`, `FieldUpdate` |
| `jubarte::capabilities` | discover the build's surface | `capabilities(runtime)`, `capabilities_json` |
| `jubarte::debug` | triage catalogs for a DOCX | `list`, `report`, `Check`, `TRIAGE` |
| `jubarte::admission` | input validation budgets | `admit(bytes, InputLimits)` |
| `jubarte::update` | self-update (feature `self-update`) | `plan`, `preflight`, `run` |
| lower-level | when you need the internals | `jubarte::opc::PartFs`, `jubarte::wml_document::WmlDocument`, `jubarte::strict_translation::strict_to_transitional_docx` |

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

Every Word-package argument and return is `&[u8]`/`Vec<u8>` holding the
complete package — no temp files, no processes. Markdown text is `&str`/`String`
instead, e.g. `markdown::diff_markdown(old: &str, new: &str) -> String`.

### Ids today and the 17 October contract

The ids these APIs and the CLI print and take are the settled short ids
(`p12`, `header1.p2`, `footnotes.p3`, `r12`, `r1+2`, `c5`, `c5.text`,
`t0.row1.cell2`); `inspect::inspect_json` keeps the schema-1 fields (`id`,
`ref`, numeric comment ids; `inspect::SCHEMA_VERSION` is `1`), and a few
transitional spellings survive: `noid` for a change with no `w:id`, a colon
id (`body:rev:007`) when no short spelling reads back, and nested `.pN`
inside cells and text boxes (`t0.row1.cell2.p1`, `p14.box0.p0`). The
short-and-long-ID contract
([superpowers/specs/2026-10-17-short-and-long-id-contract.md](superpowers/specs/2026-10-17-short-and-long-id-contract.md),
staged in
[superpowers/plans/2026-10-16-id-contract-implementation.md](superpowers/plans/2026-10-16-id-contract-implementation.md))
is not implemented: it scopes cell and text-box paragraphs as `psN`,
allocates labels for the fallback cases, moves JSON to paired
`short_id`/`long_id` references under `schema_version: 2`, and retires the
colon family, `noid` and open ranges (`p17-`, `-p3`).

### Selective change resolution

`ChangeFilter` selects by public `ids`, `authors` and `kinds` fields
(`Option<Vec<..>>`): a change is selected when it matches every list set, and
a list left out matches any. `ChangeFilter::ids(..)` builds from ids; struct
syntax narrows further:

```rust
use jubarte::changes::{self, ChangeFilter, ChangeKind};

let filter = ChangeFilter {
    kinds: Some(vec![ChangeKind::Formatting]),
    ..ChangeFilter::ids(["r12"])
};
let resolved = changes::accept_changes(&redline, &filter)?;
```

### Rendering styles

`PdfOptions::revisions` (`RevisionStyle`) picks how tracked changes are drawn:

- `RevisionStyle::Conventional` (default) — red/blue/green marks;
- `RevisionStyle::Word` — copies Microsoft Word's own PDF output, including
  its quirks (see [docs/WORD_DIFFERENCES.md](WORD_DIFFERENCES.md));
- `RevisionStyle::Custom(palette)` — `RevisionPalette::parse("deleted=#AA0000:strike,inserted=#0055FF:double-underline")`.

Whenever a jubarte PDF is compared against one Word produced, render with
`RevisionStyle::Word` — the other styles intentionally keep the more readable
convention. PNG output caps at `MAX_PNG_DPI` (1200).

## Environment variables

| Variable | Effect |
|---|---|
| `JUBARTE_FONT_DIR` | extra font lookup directory |
| `JUBARTE_FONT_INDEX` | font index path; `off` disables the persistent index |
| `JUBARTE_TRACE` / `JUB_TRACE` | comparer tracing (debugging) |
| `OOXMLSDK_DEBUG_MOVES` | move-detection tracing (debugging) |

## CLI reference

An existing output without `--force` is a usage refusal (exit 2).
Input, operational and I/O failures exit 1.

Exit status, for scripts and agents that chain commands:

| exit | meaning |
|---|---|
| 0 | done |
| 1 | an input, operational or I/O failure |
| 2 | usage, refused before any work: a flag, a combination, an existing output |
| 3 | an edit plan refused, nothing written; see stderr and any report on stdout |
| 4 | a substituted font under `--fail-on-substitution` |

Parse/schema refusals may leave stdout empty; read stderr for the reason.
Execution refusals report on stdout; plan-level refusals may have no operation.

`validate` also exits 2 on any error, on any `word` or `edits` finding, and
with `--strict` on any warning.

A chain passes files by path. Name each output with `-o` or `--out-dir`,
check the exit status before the next step, and validate the delivered file
against the original as received:

```sh
cp contract.docx contract.orig.docx
jubarte read contract.docx -p p3 -p p12 --json > p3_p12.json   # -p repeats
jubarte read contract.docx --tables -o tables.md
jubarte changes contract.docx --author "Ann Counsel" --json > ann.jsonl
jubarte edit contract.docx --plan plan.json --out-dir review || exit $?
jubarte validate review/redline.docx --original contract.orig.docx --author Claude
```

The block below is generated from the real `--help` output of this source
tree's `jubarte` binary — every command, every flag, every default. Hidden
debug flags are not shown; `jubarte <command> --help` on the installed
release is always authoritative.

The native `jubarte` binary offers `--keep-nested-comment` on `accept`,
`reject` and `convert --track-changes accept|reject`. Partial overlaps and
flag-off output retain their existing behavior. The Python and WebAssembly
bindings do not yet dispatch that option or return its kept-thread receipts;
use the native binary when retention is needed.

<!-- gen:cli-rust:start -->
#### `jubarte`

```text
$ jubarte --help
jubarte 0.16.0
Read, edit, compare and render Word documents

Usage: jubarte [OPTIONS]
       jubarte <COMMAND>

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
                per command), or a JSON plan. Writes FILE_modified.docx (--in-place
                replaces FILE; --out-dir: the bundle), then prints SUCCESS and a fenced
                before/after diff of the full agent views (refusal: exit 3)
  add           Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several
                -p per command). Writes FILE_modified.docx, as edit does (--in-place
                replaces FILE), and prints SUCCESS and a fenced before/after diff of the
                full agent views
  capabilities  What this binary can do, for agents choosing an operation
  visual-diff   Compare rendered pages pixel by pixel (different pages: exit 5)
  comments      List comments, threads and the text they annotate
  append        Join documents in order, preserving images, styles, lists and notes
  validate      Check Word validity, tracked edits, accessibility, style and structure;
                or repair Word validity
  check         Check a document against a source-bound expectation contract
  scrub         Remove authors, editing IDs, metadata and comments before sharing
  help          Print this message or the help of the given subcommand(s)
  self-update   Check or install a release from GitHub

Options:
  -q, --quiet    No status on success: no `wrote` lines, SUCCESS block or edit summary.
                 Files are still written; a reading task (`read`, `changes`, `compare`
                 without -o, `diff`, `capabilities`) still prints what it read, and
                 warnings and errors still print. `read`, `changes` and `comments` drop
                 their stderr validity status (JSON under `changes --json` and `comments
                 --json`). `read --json` and `read --for-edit` keep validity counts in
                 their output, -q or not
  -h, --help     Print help
  -V, --version  Print version

Examples:
  jubarte compare old.docx new.docx -o redline.docx
  jubarte contract.docx                    print the agent view (read)
  jubarte old.docx new.docx                redline printed as the agent view (-o writes
  it)
  jubarte read contract.docx --json
  jubarte diff old.docx new.docx --format github
  jubarte convert contract.docx -o contract.pdf

Run jubarte <task> --help for task options.
```

#### `jubarte compare`

```text
$ jubarte compare --help
Compare documents: print the redline as the agent view, or write it with -o

Usage: jubarte compare [OPTIONS] [ORIGINAL] [MODIFIED]

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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
          Print matching body paragraphs only, with the same Markdown and id lines as
          -p. Searches original final logical text, as --for-edit does; combined filters
          narrow. Like -p, the view cannot select paragraphs in headers, footers,
          footnotes, endnotes and comments. Use --for-edit for header/footer/note
          paragraphs. No match prints nothing and reports it on stderr

      --regex <PATTERN>
          Search with a regular expression instead of literal text (view or --for-edit).
          Give PATTERN here and omit --find

      --ignore-case
          Case-insensitive --find/--regex (view or --for-edit)

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

#### `jubarte changes`

```text
$ jubarte changes --help
List tracked changes with IDs for accept, reject and edit plans.

One line per logical change: the marks Word wrote for one edit are one change with a
joined id (`r7+8+9+10`), its text followed by `(N elements)`. A revision inside a
comment body is `comments.r9`.

Usage: jubarte changes [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx)

Options:
      --json
          Emit one record per line on stdout (records only). The input_validity status
          is one JSON line on stderr; -q silences it

      --author <NAME>
          Only listed changes by this author (exact match); repeat for several

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte accept`

```text
$ jubarte accept --help
Accept all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. A comment-body
revision is `comments.r9`, the id `changes` lists, and an id, author or kind selection
settles it.

Usage: jubarte accept [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to accept

Options:
      --in-place
          Replace FILE through the same atomic write as -o. A refused resolution leaves
          FILE unchanged. Not with -o

  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, `comments.r9`, or `r1+2` (both). Repeatable

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

#### `jubarte reject`

```text
$ jubarte reject --help
Reject all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. A comment-body
revision is `comments.r9`, the id `changes` lists, and an id, author or kind selection
settles it.

Usage: jubarte reject [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to reject

Options:
      --in-place
          Replace FILE through the same atomic write as -o. A refused resolution leaves
          FILE unchanged. Not with -o

  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, `comments.r9`, or `r1+2` (both). Repeatable

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

#### `jubarte convert`

```text
$ jubarte convert --help
Convert Word or Markdown to DOCX, PDF, PNG or Markdown

Usage: jubarte convert [OPTIONS] <FILE>

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

#### `jubarte diff`

```text
$ jubarte diff --help
Review differences as GitHub, word, normal, context or side-by-side text

Usage: jubarte diff [OPTIONS] <OLD> <NEW>

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

#### `jubarte read`

```text
$ jubarte read --help
Read the agent view: YAML header, `<!-- pN -->` id lines, changes and comments with ids

A YAML header, then Markdown with an `<!-- pN -->` id line before every paragraph,
tracked changes as CriticMarkup followed by their ids (`{++text++}{r12 @AC}`) and
comments with theirs (`{>>c5 @AC: …<<}`).

Usage: jubarte read [OPTIONS] <FILE>

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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
          Print matching body paragraphs only, with the same Markdown and id lines as
          -p. Searches original final logical text, as --for-edit does; combined filters
          narrow. Like -p, the view cannot select paragraphs in headers, footers,
          footnotes, endnotes and comments. Use --for-edit for header/footer/note
          paragraphs. No match prints nothing and reports it on stderr

      --regex <PATTERN>
          Search with a regular expression instead of literal text (view or --for-edit).
          Give PATTERN here and omit --find

      --ignore-case
          Case-insensitive --find/--regex (view or --for-edit)

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

#### `jubarte edit`

```text
$ jubarte edit --help
Edit a document: -p WHERE with --anchor, --content, --delete, --resolve, --reply,
--accept, --reject, --strike, --restore or --style (several -p per command), or a JSON
plan. Writes FILE_modified.docx (--in-place replaces FILE; --out-dir: the bundle), then
prints SUCCESS and a fenced before/after diff of the full agent views (refusal: exit 3)

Usage: jubarte edit [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Left unchanged unless --in-place

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
          Text inside WHERE the operation applies to (must occur once, unless
          --occurrence picks a match)

      --occurrence <N>
          Pick the Nth anchor match (1-based), after --anchor in this -p group

      --content <TEXT>
          New text: replaces the anchor; rewrites the paragraph without one; on c5, the
          comment's new text

      --delete
          Delete the anchor, or the whole paragraph (or comment) without one

      --resolve
          Resolve comment c5 (with -p c5)

      --reply <TEXT>
          Reply to comment c5 (with -p c5), as `add -p c5 --content`

      --keep-others
          Refuse to accept or reject another author's tracked change, and to delete,
          resolve, reopen or reword their comment. Restore, strike, a comment and a
          reply stay allowed. JUBARTE_KEEP_OTHERS=1 sets the same guard. Another author
          is anyone other than --author (Jubarte when --author is left out)

      --accept
          Accept revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`accepted r1+2 in p3: "from" > "to"`)

      --reject
          Reject revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`rejected r0 in p3: "from" > "to"`)

      --strike
          Strike another author's insertion r12 (with -p r12; r10+11 for a change of
          several marks, a replacement's insertion only): our tracked deletion inside
          it, so the decision shows. --content is one comment on the struck words

      --restore
          Restore another author's deletion r12 (with -p r12; r10+11 for a change of
          several marks, a replacement's deletion only): their deletion stays and our
          tracked insertion of its words follows it. --content is one comment on the
          restored words

      --visible
          With --reject: strike each insertion and restore each deletion instead of
          removing the change without a trace

      --style <SPEC>
          Formatting for the anchored text: bold, italic, underline, strike, caps,
          highlight=yellow, font=Calibri, size=11, color=FF0000; any other value is a
          paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own). A
          write that adds markup needs it spelled here or in JUBARTE_AUTHOR

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

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

      --in-place
          Replace FILE through the same atomic write as -o. A refused edit or a failed
          check leaves FILE unchanged. Not with -o or --out-dir

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
  jubarte edit a.docx -p p12 --anchor "thirty days" --content "forty-five days" -a NAME
  jubarte edit a.docx -p p12 --anchor thirty --content "thirty (30)" -a NAME   an
  insertion
  jubarte edit a.docx -p p7 --anchor "at its sole discretion" --delete -p p9 --delete -a
  NAME
  jubarte edit a.docx -p p3 --content "The parties agree as follows." -a NAME    rewrite
  jubarte edit a.docx -p p4 --anchor Fees --style bold -p p5 --style Heading2 -a NAME
  jubarte edit a.docx -p c5 --content "Agreed." -p c7 --resolve -a NAME
  jubarte edit a.docx -p c5 --reply "Agreed, will fix." -a NAME
  jubarte edit a.docx -p c5.text --content "net 30" -a NAME   the text c5 covers
  jubarte edit a.docx -p r3 --accept -p header1.r1 --reject
  jubarte edit a.docx -p p3 --delete -o b.docx --pdf -a NAME   b.docx and b.pdf
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

Desired-text plans (PlanV2), a separate workflow:
  jubarte edit template FILE --select p12,p20-p25 -o edits.json   the paragraphs as they
  read
  (edit the desired text in edits.json)
  jubarte edit apply FILE --plan edits.json -o OUTPUT.docx        tracked changes for
  the difference

Typed actions: jubarte edit replace|rewrite|insert|delete FILE [OPTIONS]
  Inline --save-plan FILE saves without applying; writes require -o or --out-dir.
  Run jubarte edit ACTION --help for its grammar.
Routing: valid legacy calls, including `edit replace -p p0 --content TEXT`,
  retain v1 semantics. Use ./replace (or ./rewrite, ./insert, ./delete, ./apply)
  to explicitly name a legacy input whose name equals an action.
```

#### `jubarte add`

```text
$ jubarte add --help
Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several -p per command).
Writes FILE_modified.docx, as edit does (--in-place replaces FILE), and prints SUCCESS
and a fenced before/after diff of the full agent views

Usage: jubarte add [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Left unchanged unless --in-place

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
          Text inside WHERE to comment on (must occur once, unless --occurrence picks a
          match)

      --occurrence <N>
          Pick the Nth anchor match (1-based), after --anchor in this -p group

      --content <TEXT>
          The paragraph, comment or reply text

      --reply <TEXT>
          Reply to comment c5 (with -p c5); the same as --content there

      --keep-others
          Refuse to accept or reject another author's tracked change, and to delete,
          resolve, reopen or reword their comment, when this command applies or records
          a plan. Restore, strike, a comment and a reply stay allowed.
          JUBARTE_KEEP_OTHERS=1 sets the same guard. Another author is anyone other than
          --author (Jubarte when --author is left out)

      --before
          A new paragraph before WHERE instead of after it

      --comment
          A comment on the whole paragraph (with --anchor the comment sits on the anchor
          and this flag is implied)

      --style <SPEC>
          Formatting for the new paragraph's text (bold, italic, underline,
          highlight=yellow) or its paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own). A
          write that adds markup needs it spelled here or in JUBARTE_AUTHOR

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

      --in-place
          Replace FILE through the same atomic write as -o. A refused edit or a failed
          check leaves FILE unchanged. Not with -o or --out-dir

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte add a.docx -p p12 --content "Time is of the essence." -a NAME   new paragraph
  after p12
  jubarte add a.docx -p p12 --content Recitals --before --style Heading2 -a NAME
  jubarte add a.docx -p p5 --anchor "monthly fee" --content "Net of taxes?" -a NAME   a
  comment
  jubarte add a.docx -p p5 --comment --content "Whole clause needs a cap." -a NAME
  jubarte add a.docx -p c5 --content "Agreed, will fix." -a NAME           a reply
  jubarte add a.docx -p c5 --reply "Agreed, will fix." -a NAME             the same
  reply
  jubarte add a.docx --plan plan.json -p p5 --comment --content "Cap?"   record into
  plan.json
```

#### `jubarte capabilities`

```text
$ jubarte capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte capabilities [OPTIONS]

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
                               drop their stderr validity status (JSON under `changes
                               --json` and `comments --json`). `read --json` and `read
                               --for-edit` keep validity counts in their output, -q or
                               not
  -h, --help                   Print help
```

#### `jubarte visual-diff`

```text
$ jubarte visual-diff --help
Compare rendered pages pixel by pixel (different pages: exit 5)

Usage: jubarte visual-diff [OPTIONS] <A> <B>

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
                       `changes` and `comments` drop their stderr validity status (JSON
                       under `changes --json` and `comments --json`). `read --json` and
                       `read --for-edit` keep validity counts in their output, -q or not
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

#### `jubarte comments`

```text
$ jubarte comments --help
List comments, threads and the text they annotate

Usage: jubarte comments [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json           Emit one record per line on stdout (records only). The
                       input_validity status is one JSON line on stderr; -q silences it
      --author <NAME>  Only this author's comments (exact match)
      --latest         One comment per thread: the newest
  -q, --quiet          No status on success: no `wrote` lines, SUCCESS block or edit
                       summary. Files are still written; a reading task (`read`,
                       `changes`, `compare` without -o, `diff`, `capabilities`) still
                       prints what it read, and warnings and errors still print. `read`,
                       `changes` and `comments` drop their stderr validity status (JSON
                       under `changes --json` and `comments --json`). `read --json` and
                       `read --for-edit` keep validity counts in their output, -q or not
  -h, --help           Print help
```

#### `jubarte append`

```text
$ jubarte append --help
Join documents in order, preserving images, styles, lists and notes

Usage: jubarte append [OPTIONS] --output <FILE> <FILE> <FILE>...

Arguments:
  <FILE> <FILE>...
          The documents (.docx), in order

Options:
  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --section-break <SECTION_BREAK>
          What separates each document from the one before it

          Possible values:
          - next-page:  Each document starts on a new page
          - continuous: Each document continues on the same page (a continuous section
            break with --keep-sections)
          - none:       Nothing between the documents (continuous with --keep-sections)

          [default: next-page]

      --keep-sections
          Keep each appended document's final section (page size, margins, headers,
          footers) as a section of its own

      --carry-comments
          Carry the comments each appended document's body and notes anchor, with their
          threads and resolution (those in headers and footers are still dropped). Off,
          comments are dropped and warned

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

Examples:
  jubarte append a.docx b.docx -o ab.docx
  jubarte append cover.docx body.docx annex.docx -o all.docx --section-break continuous
  jubarte append letter.docx exhibit.docx -o out.docx --keep-sections
  jubarte append review_a.docx review_b.docx -o both.docx --carry-comments
```

#### `jubarte validate`

```text
$ jubarte validate --help
Check Word validity, tracked edits, accessibility, style and structure; or repair Word
validity.

A word or edits finding fails the run (exit 2), and so does any error; --strict fails on
warnings too. With --baseline, only the findings the baseline lacks count. An unreadable
file exits 1.

Usage: jubarte validate [OPTIONS] <FILE>

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
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

#### `jubarte check`

```text
$ jubarte check --help
Check a document against a source-bound expectation contract.

The only profile this build supports is `--expect`: an optional, separately supplied,
hash-bound expectations file whose typed paragraph-text predicates (exact equality,
containment, absence) are evaluated on FILE's own final projection. The other check
profiles (integrity, quality, layout, baseline tracking) are #961; `jubarte validate`
remains the validity audit. Passing checks and natural-language requirements without
executable checks never claim the task complete.

Usage: jubarte check [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) whose own digest the expectations must name

Options:
      --expect <FILE>
          Expectation contract (the only supported profile in this build)

      --json
          One result envelope, including refusals

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte check revised.docx --expect expectations.json
  jubarte check revised.docx --expect expectations.json --json

The expectations' source.sha256 must name FILE's own bytes: an
expectation file bound to the pre-edit source refuses as stale on
the edited output. Use the output-bound copy an `edit apply --expect`
receipt carries to re-check a published output.
```

#### `jubarte scrub`

```text
$ jubarte scrub --help
Remove authors, editing IDs, metadata and comments before sharing

Usage: jubarte scrub [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) to scrub

Options:
  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --author-alias <NAME>
          Name every author (revisions, comments, people.xml) takes

      --rsids
          Remove rsids, the edit-session ids that tie copies together

      --docprops
          Remove creator, last editor, revision number, dates, manager, company and
          custom properties

      --comments
          Remove every comment

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

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

Examples:
  jubarte scrub redline.docx -o out.docx                     everything, alias Author
  jubarte scrub redline.docx -o out.docx --author-alias Counsel --rsids
  jubarte scrub redline.docx -o out.docx --comments          comments only
```

#### `jubarte self-update`

```text
$ jubarte self-update --help
Check or install a release from GitHub

Usage: jubarte self-update [OPTIONS]

Options:
      --check              Print the installed and latest versions; install nothing
  -y, --yes                Install without asking (needed without a terminal)
      --version <VERSION>  Install this release instead of the latest, older ones
                           included
  -q, --quiet              No status on success: no `wrote` lines, SUCCESS block or edit
                           summary. Files are still written; a reading task (`read`,
                           `changes`, `compare` without -o, `diff`, `capabilities`)
                           still prints what it read, and warnings and errors still
                           print. `read`, `changes` and `comments` drop their stderr
                           validity status (JSON under `changes --json` and `comments
                           --json`). `read --json` and `read --for-edit` keep validity
                           counts in their output, -q or not
  -h, --help               Print help

Examples:
  jubarte self-update --check          installed and latest versions
  jubarte self-update                  ask, then install the latest release
  jubarte self-update --yes            install without asking
  jubarte self-update --version 0.9.3  install that release (also older)
```
<!-- gen:cli-rust:end -->

## Keeping this page current

`scripts/gen_docs.sh` regenerates the block above; the CI docs job
(`docs.yml`) fails the build when the committed text drifts from the binary.
Curated sections are hand-maintained — keep them short and point at docs.rs
for the full API.

See also: [CHANGELOG](../CHANGELOG.md), [VERSIONING](VERSIONING.md),
[docs/api/](api/) snapshots.
