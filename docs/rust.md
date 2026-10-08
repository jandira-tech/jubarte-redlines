<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Rust guide

Using jubarte from Rust: the library crate and the `jubarte` CLI. For the
other surfaces see [docs/python.md](python.md) (Python and `uvx`) and
[docs/javascript.md](javascript.md) (Node, browser and `npx`); the
[README](../README.md) covers the shared workflow and install options.

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

Every Word-package argument and return is `&[u8]`/`Vec<u8>` holding the
complete package — no temp files, no processes. Markdown text is `&str`/`String`
instead, e.g. `markdown::diff_markdown(old: &str, new: &str) -> String`.

### Selective change resolution

`ChangeFilter` selects by public `ids`, `authors` and `kinds` fields
(`Option<Vec<..>>`): a change is selected when it matches every list set, and
a list left out matches any. `ChangeFilter::ids(..)` builds from ids; struct
syntax narrows further:

```rust
use jubarte::changes::{self, ChangeFilter, ChangeKind};

let filter = ChangeFilter {
    kinds: Some(vec![ChangeKind::Formatting]),
    ..ChangeFilter::ids(["body:rev:12"])
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

The block below is generated from the real `--help` output of this source
tree's `jubarte` binary — every command, every flag, every default. Hidden
debug flags are not shown; `jubarte <command> --help` on the installed
release is always authoritative.

<!-- gen:cli-rust:start -->
#### `jubarte`

```text
$ jubarte --help
Read, edit, compare and render Word documents

Usage: jubarte [OPTIONS] [ORIGINAL] [MODIFIED]
       jubarte <COMMAND>

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
  self-update   Check or install a release from GitHub
  debug         Diagnose a Word package, or compare package structures
  diff-render   Compare rendered pages pixel by pixel (different pages: exit 5)
  comments      List comments, threads and the text they annotate
  append        Join documents in order, preserving images, styles, lists and notes
  validate      Check or repair Word validity (findings: exit 2; unreadable: exit 1)
  fields        Field results written back into the document from jubarte's layout
  scrub         Remove authors, editing IDs, metadata and comments before sharing
  audit         Audit accessibility, style and structure (findings: exit 2)
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
  jubarte compare old.docx new.docx -o redline.docx
  jubarte old.docx new.docx                shorthand for compare
  jubarte inspect contract.docx --json
  jubarte diff old.docx new.docx --format github
  jubarte convert contract.docx -o contract.pdf

Run jubarte <task> --help for task options.
```

#### `jubarte compare`

```text
$ jubarte compare --help
Compare documents and write a Word redline

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

#### `jubarte revisions`

```text
$ jubarte revisions --help
List the tracked revisions in a redline .docx

Usage: jubarte revisions [OPTIONS] <FILE>

Arguments:
  <FILE>  The redline document (.docx)

Options:
      --json  Emit the list as JSON lines instead of a human summary
  -h, --help  Print help
```

#### `jubarte changes`

```text
$ jubarte changes --help
List tracked changes with IDs for accept, reject and edit plans

Usage: jubarte changes [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json  Emit one JSON object per line
  -h, --help  Print help
```

#### `jubarte accept`

```text
$ jubarte accept --help
Accept all tracked changes, or select by ID, author or kind

Usage: jubarte accept [OPTIONS] --output <FILE> <FILE>

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

#### `jubarte reject`

```text
$ jubarte reject --help
Reject all tracked changes, or select by ID, author or kind

Usage: jubarte reject [OPTIONS] --output <FILE> <FILE>

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

#### `jubarte inspect`

```text
$ jubarte inspect --help
Inspect document facts, paragraphs, styles and tables

Usage: jubarte inspect [OPTIONS] <FILE>

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

#### `jubarte text`

```text
$ jubarte text --help
Read Markdown with edit IDs `[body:p:N]`, or with tracked marks

Usage: jubarte text [OPTIONS] <FILE>

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

#### `jubarte edit`

```text
$ jubarte edit --help
Apply a JSON edit plan; write clean copy, redline and report (refusal: exit 3)

Usage: jubarte edit [OPTIONS] --plan <PLAN.json> --out-dir <DIR> <FILE>

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

#### `jubarte capabilities`

```text
$ jubarte capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte capabilities [OPTIONS]

Options:
      --json  Emit JSON (the default output is JSON too; the flag documents intent)
  -h, --help  Print help
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
  -h, --help               Print help

Examples:
  jubarte self-update --check          installed and latest versions
  jubarte self-update                  ask, then install the latest release
  jubarte self-update --yes            install without asking
  jubarte self-update --version 0.9.3  install that release (also older)
```

#### `jubarte debug`

```text
$ jubarte debug --help
Diagnose a Word package, or compare package structures

Usage: jubarte debug [OPTIONS] <FILE>...
       jubarte debug <COMMAND>

Commands:
  diff  What differs between two or more packages, element by element: styles paired by
        type and name, paragraphs by their text, headers and footers by section role.
        Each hunk prints the lines not every file holds; with three or more files each
        line names the files that hold it. rsids, paragraph ids, revision
        ids/authors/dates, relationship ids (shown as what they point to), docProps save
        stamps, attribute order, on/off values and empty property blocks are dropped
        unless --raw
  help  Print this message or the help of the given subcommand(s)

Arguments:
  <FILE>...
          One package, or two to compare (A then B)

Options:
  -l, --list
          List the package's entries (sizes); with two files, the entries that differ

  -c, --check <CHECKS>
          Reports to run [default: orphans, fields, bookmarks, package, structure]

          Possible values:
          - orphans:   Deleted text outside its story's w:del; live text inside one;
            bare runs in a text box whose anchor is deleted
          - fields:    Field nesting per story; fields partly deleted
          - bookmarks: Duplicate/unpaired bookmarks; start and end in different sdt,
            cell, text box or revision; bookmarks in plain-text or list controls
          - package:   Content types, relationship ids and targets, dangling
            note/comment references, undeclared mc:Ignorable prefixes
          - structure: Empty field codes, cells not ending in a paragraph, rows without
            cells, nested same-kind revisions, a body sectPr that is not last
          - ids:       Revision and docPr ids used twice (not in the default triage:
            Word opens such files)
          - styles:    Style links and references naming no style; two styles with one
            type and name (Word pairs styles by name)
          - chains:    Where bookmark starts and ends sit (parent chains, tallied)
          - elements:  Element counts
          - textbox:   Text box stories as XML (see --grep)
          - text:      Paragraph text per story part, with {+inserted+} / [-deleted-]
            runs and the mark state; with two files, the lines that differ
          - xml:       Part XML one element per line, without namespace declarations,
            rsids or paraIds; with two files, the lines that differ
          - runs:      `text` with each paragraph's direct properties [..], its mark's
            «..» and each run's direct formatting «..»; with two files, the lines that
            differ
          - changes:   Property-change records (pPrChange, tcPrChange, sectPrChange, …):
            where each sits and what the live properties add (+) and drop (-) against
            the recorded ones; with two files, the lines that differ
          - styledefs: Style definitions by type and name (localized ids pair):
            docDefaults, then each style's default flag and basedOn/link by name, and a
            line per pPr/rPr/tblPr/… block; with two files, the lines that differ
          - numbering: List levels by numId and level as paragraphs see them (abstract
            definition plus the list's overrides; abstract ids renumber, so they are
            left out); with two files, the lines that differ
          - render:    What each story part should put on the page (tables with style,
            float and shading; shaded, highlighted, coloured and hidden text; fonts;
            fields; ins/del order; frames; sections), and "(layout)": jubarte's page
            count and the face each font resolved to; with two files, the lines that
            differ

  -p, --part <NAME>
          Only parts whose name contains this (e.g. document.xml)

  -g, --grep <TEXT>
          Only what contains this: textbox stories;
          text/runs/xml/changes/styledefs/numbering lines (a runs paragraph matched on
          its plain text, printed whole)

  -n, --limit <N>
          Examples per finding kind

          [default: 5]

  -C, --context <N>
          text/xml/runs of two files: common lines shown around each change

          [default: 0]

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte debug out.docx                    orphans, fields, bookmarks, package,
  structure
  jubarte debug out.docx --list             the package's entries
  jubarte debug old.docx new.docx --list    entries that differ
  jubarte debug old.docx new.docx -c elements -p document.xml
  jubarte debug out.docx -c ids             revision/docPr ids used twice
  jubarte debug out.docx -c textbox -g FILENAME
  jubarte debug out.docx -c text            paragraphs with ins/del marks
  jubarte debug a.docx b.docx -c text       paragraphs that differ, per part
  jubarte debug a.docx b.docx -c runs       the same, with direct formatting
  jubarte debug out.docx -c runs -g "Q: Can"   one paragraph's runs, whole
  jubarte debug out.docx -c changes         what each pPrChange/tcPrChange/… records
  jubarte debug a.docx b.docx -c styledefs  style definitions that differ, paired by
  name
  jubarte debug a.docx b.docx -c numbering  list levels that differ, by numId
  jubarte debug a.docx b.docx -c xml -p document.xml
  jubarte debug diff a.docx ours.docx word.docx   element by element, three-way
```

#### `jubarte debug diff`

```text
$ jubarte debug diff --help
What differs between two or more packages, element by element: styles paired by type and
name, paragraphs by their text, headers and footers by section role. Each hunk prints
the lines not every file holds; with three or more files each line names the files that
hold it. rsids, paragraph ids, revision ids/authors/dates, relationship ids (shown as
what they point to), docProps save stamps, attribute order, on/off values and empty
property blocks are dropped unless --raw

Usage: jubarte debug diff [OPTIONS] <FILE> <FILE>...

Arguments:
  <FILE> <FILE>...  Two or more packages; the first is the reference (`-` lines)

Options:
  -p, --part <NAME>       Only parts whose name or role contains this (e.g. styles,
                          document.xml, "default header")
      --style <NAME>      Only the style with this name or id (case-insensitive)
      --para-text <TEXT>  Only paragraphs whose text contains this, in any file
      --raw               Keep rsids, ids, authors, dates, on/off values and empty
                          blocks
      --full              Print each shown element's common lines too
  -n, --limit <N>         Hunks per part (0: all) [default: 60]
  -h, --help              Print help

Examples:
  jubarte debug diff a.docx b.docx
  jubarte debug diff a.docx ours_rej.docx word_rej.docx -p styles
  jubarte debug diff a.docx ours_rej.docx word_rej.docx --style "Body Text" --full
  jubarte debug diff a.docx ours.docx --para-text "Section 4"
```

#### `jubarte diff-render`

```text
$ jubarte diff-render --help
Compare rendered pages pixel by pixel (different pages: exit 5)

Usage: jubarte diff-render [OPTIONS] <A> <B>

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

#### `jubarte comments`

```text
$ jubarte comments --help
List comments, threads and the text they annotate

Usage: jubarte comments [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json           Emit one JSON object per line
      --author <NAME>  Only this author's comments (exact match)
      --latest         One comment per thread: the newest
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
          Output path

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
          Print nothing on success

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte append a.docx b.docx -o ab.docx
  jubarte append cover.docx body.docx annex.docx -o all.docx --section-break continuous
  jubarte append letter.docx exhibit.docx -o out.docx --keep-sections
  jubarte append review_a.docx review_b.docx -o both.docx --carry-comments
```

#### `jubarte validate`

```text
$ jubarte validate --help
Check or repair Word validity (findings: exit 2; unreadable: exit 1)

Usage: jubarte validate [OPTIONS] <FILE>

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

#### `jubarte fields`

```text
$ jubarte fields --help
Field results written back into the document from jubarte's layout

Usage: jubarte fields <COMMAND>

Commands:
  update  Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC fields from
          jubarte's layout; TOCs are rebuilt from the headings. Field codes stay, so
          Word can update them again. Page numbers are jubarte's layout, not Word's
          (docs/WORD_DIFFERENCES.md)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### `jubarte fields update`

```text
$ jubarte fields update --help
Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC fields from jubarte's
layout; TOCs are rebuilt from the headings. Field codes stay, so Word can update them
again. Page numbers are jubarte's layout, not Word's (docs/WORD_DIFFERENCES.md)

Usage: jubarte fields update [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
  -o, --output <FILE>  Output path
      --force          Overwrite the output file if it already exists
      --json           Print the fields written as JSON
  -h, --help           Print help

Examples:
  jubarte fields update in.docx -o out.docx          one line per field written
  jubarte fields update in.docx -o out.docx --json   {"page_count", "fields": [...]}
```

#### `jubarte scrub`

```text
$ jubarte scrub --help
Remove authors, editing IDs, metadata and comments before sharing

Usage: jubarte scrub [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>  The document (.docx) to scrub

Options:
  -o, --output <FILE>        Output path
      --force                Overwrite the output file if it already exists
      --author-alias <NAME>  Name every author (revisions, comments, people.xml) takes
      --rsids                Remove rsids, the edit-session ids that tie copies together
      --docprops             Remove creator, last editor, revision number, dates,
                             manager, company and custom properties
      --comments             Remove every comment
  -h, --help                 Print help

Examples:
  jubarte scrub redline.docx -o out.docx                     everything, alias Author
  jubarte scrub redline.docx -o out.docx --author-alias Counsel --rsids
  jubarte scrub redline.docx -o out.docx --comments          comments only
```

#### `jubarte audit`

```text
$ jubarte audit --help
Audit accessibility, style and structure (findings: exit 2)

Usage: jubarte audit [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx) to audit

Options:
      --json           Emit `{findings, rules, layout}` as JSON
      --rules <RULES>  Rule sets (a11y, style, structure) or rule codes, comma-separated
                       [default: every rule]
      --strict         Fail (exit 2) on warnings too, not only on errors
  -h, --help           Print help

Rules (code, set, severity):
  HEADING_SKIP a11y warning, IMAGE_NO_DESCR a11y error,
  TABLE_NO_HEADER_ROW a11y warning, MISSING_LANG a11y warning,
  LITERAL_BULLET style warning, EMPTY_SPACER_PARAGRAPH style info,
  DIRECT_FORMATTING_OVERRIDES_STYLE style info,
  STALE_FIELD_CACHE structure warning, FONT_SUBSTITUTED structure info
```
<!-- gen:cli-rust:end -->

## Keeping this page current

`scripts/gen_docs.sh` regenerates the block above; the CI docs job
(`docs.yml`) fails the build when the committed text drifts from the binary.
Curated sections are hand-maintained — keep them short and point at docs.rs
for the full API.

See also: [CHANGELOG](../CHANGELOG.md), [VERSIONING](VERSIONING.md),
[docs/api/](api/) snapshots.
