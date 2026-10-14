<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids consolidation and a lossless Markdown round trip: plan (2026-10-13)

Implements `docs/superpowers/specs/2026-10-13-ids-and-markdown-roundtrip.md` (the spec). Arthur's
evening goal: consolidate the short and long ids, focused on short; consolidate the flags; a script
that prevents drift; and "MD to DOCX and DOCX to MD is lossless as possible": nothing with a short id
is lost. Centralized types come first (spec §2.1).

## How it ships

- Stacked PRs, one per task, each based on the one before (after #463); Arthur merges.
- At most five files per commit; commit and push often; rebase the stack when a base moves.
- Each PR: comment `@coderabbitai review` and `@codex review`, then a `pi -p` adversarial review once
  the work is done; confirmed findings are fixed on the same branch. Nobody waits on CI.
- Red/green: the failing test lands first. Gates per PR: `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings`, the touched tests, `jubarte --help`.
- The strict round-trip job (`md-roundtrip.yml`) is red until the reader and view tasks land; each
  PR quotes its verdict counts (correct / wrong / lost / new / docx differences) before and after.

## Where ids live today

- Printers: `format!("p{…}")`-style spellings in `src/markdown/from_docx/agent.rs` (15),
  `header.rs` (6), `src/ids.rs` (5), `comments.rs` (2), and one each in `revision_processor.rs`,
  `edit/flags.rs`, `comparer/lcs.rs`.
- Parsers: `src/ids.rs` (`parse_location`, `parse_change_ids`, `parse_comment_id`, `cell_path`, …),
  plus a private grammar for `read -p` (`markdown::Select::parse`, `src/markdown/mod.rs:328`) and
  callers in `edit/origin.rs`, `edit/flags.rs`, `view_notes.rs`, `changes.rs`.
- No id flag has a clap `value_parser`: ids reach `String` fields, so a bad id exits 1 on `read -p`
  and 2 elsewhere.

## Tasks

### 1. The strict gate (PR #463, branch 22c) — in progress

`tests/md_fixed_point.rs`: every Markdown fixture and listed docx round-trips; a verdict per id
(`✓ correct`, `✗ wrong`, `✗ lost`, `✗ new`) and per docx difference (`jubarte debug A B --check
render`); runs to the end, fails only then. Fixtures: views of documents dense in one kind of id or
construct (`views.txt`): paragraphs, tables, revisions, comments, footnotes, endnotes, fields,
spacing with threaded comments, sections, a different first page, headers and footers with fields,
authors, dates, changes inside tables, fonts, font sizes, and one mixed document.

Follow-up (pi review of #463): the docx-side oracle sees only what `debug --check render` lists, so
bookmarks, comment thread shape and note properties lost in the first docx → md step stay
invisible. Extend `debug --check render` with bookmark, comment-thread and note lines (or compare
`read --json` by dimension, as `scripts/md_roundtrip.py` does). `ci.yml` skips the strict test;
`md-roundtrip.yml` is its only gate.

### 2. One set of id types (spec §2.1)

Red: `tests/id_types.rs` parses every id the view prints for the fixtures of task 1 and requires
`parse(print(x)) == x` and `print(parse(s)) == s`; unit tests for each kind and each refusal.

- `src/ids/` becomes a module: `short.rs` (`ShortId`, its path steps), `refs.rs` (`ChangeRef`,
  `CommentRef`, `BlockSpec`, `Location`, each a `FromStr` over `ShortId`), `long.rs` (`LongId` and
  the `-XX` escaping), `spell.rs` (the spelling lists and error texts, generated from the types).
- The existing functions keep their signatures as thin wrappers until their callers move (task 3),
  so this PR changes no behaviour.

### 3. Every option and every printer goes through the types

- clap fields take the types (`value_parser`): `--id`, `-p/--location`, `read -p`, `--reply`,
  `--resolve`. A bad id exits 2 with one message, in the binary, Python and npm.
- `read -p` drops `Select::parse` and its private grammar; `-p 12` keeps working with the
  bare-number note (spec §2).
- The view, JSON, hints and notes print ids through `Display`; the `format!` spellings above go.
- The wrappers of task 2 are deleted once unused.

### 4. Grammar completion

Rows, nested tables, text-box paragraphs, ranges, `sN`, `tocN`, `ccN` in the types and in every
option that takes a location; the YAML `sections:` map prints `s1:` (today `2:`); `read --tables`
numbers nested tables as the view does; shift notes after an edit that adds or removes a block;
bare-number deprecation notes; help and `capabilities` stop advertising bare numbers.

### 5. The drift check

`scripts/check_ids.py`, run in `docs.yml`, fails when:
- an id the view prints for a fixture does not parse (through `jubarte`'s own parser, by a hidden
  `debug --check ids` view);
- the spec's §2 table, `docs/locations.md` and the spelling lists of `spell.rs` disagree;
- the flag catalogue (`docs/flags_datetime.md`, PR #458) differs from `jubarte capabilities`.

### 6. Flag consolidation

The audit's findings on location and id flags (`jubarte-loop/cli-audit/FINDINGS.md` A1, E1, G2): one
name per meaning across commands, the catalogue regenerated, retired spellings refused with a hint.

### 7. Runs: highlight, strike, monospace

Arthur, 2026-10-13, "Simplify for now": `==x==` (a highlight, any colour), `~~x~~` (strike),
`` `x` `` (a monospace font). View and reader both; the docx verdicts of task 1 go green for the
`highlight` lines. Highlight colour, text colour and font names stay `✗ docx lost` until a later
task gives them a spelling.

### 8–13. The rest of the spec's order of work

Reader: threads and `resolved`, endnotes, id-line properties (spacing, keep-next, breaks), empty
paragraphs, cell paragraphs, a change printed in several blocks read back as one (8). Pictures,
bookmarks, internal links, body fields as code plus result, brace escaping (9). YAML
headers/footers/page/styles/sections, including a different first page (10). `convert -o X.md` in
the view dialect, `--plain` (11). Anchored mode (12). Long ids, then stamping behind the Word gate
(13).

## Open with Arthur

1. The plan's Q19 recommended `body:para:3F2A1B7C`; the spec's §4.1 settled
   `PART.ELEMENT.ATTRIBUTE.VALUE` since. Today's `body:rev:12` long change ids: accept with a note
   through 0.13 and refuse in 0.14, as bare numbers? (Recommended.)
2. Q20 (deprecating `cN.text`) stays open until the Word investigation of the plan runs.
