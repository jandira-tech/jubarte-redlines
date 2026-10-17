<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids consolidation and a lossless Markdown round trip: plan (2026-10-13)

Implements `docs/superpowers/specs/2026-10-13-ids-and-markdown-roundtrip.md` (the spec). Arthur's
evening goal: consolidate the short and long ids, focused on short; consolidate the flags; a script
that prevents drift; and "MD to DOCX and DOCX to MD is lossless as possible": nothing with a short id
is lost. Centralized types come first (spec §2.1).

> **Status (reconciled 2026-10-17, `main` `aba48bd5`).** Partly landed. The strict
> round-trip gate, the `ShortId` type and the settled-spelling refusals shipped
> (`8a3bd939`, `2fbc8662`); the reader-side thread/id work shipped (`62f14287`).
> **Not landed:** the `refs.rs`/`long.rs`/`spell.rs` modules (task 2), clap
> `value_parser` newtypes (task 3), `read -p`'s `Select::parse` retirement (still
> live, with comma lists and open ranges the contract removes), `sections:` keyed
> `sN` (still `2:`), `scripts/check_ids.py` (task 5), anchored mode (task 12),
> and the long-id work (task 13). Where this plan, the settlement and the spec
> conflict with `docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`
> v3 — cell paragraphs `.pK` vs `.psK`, `pN` story-indexed vs `psN` scoped, ranges,
> the colon `story:kind:value` longs, `pid:`, `body:para:HEX` stamping — the
> contract wins. The contract's implementation plan is
> `docs/superpowers/plans/2026-10-16-id-contract-implementation.md`.

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

### 1. The strict gate (PR #463, branch 22c) — landed

Landed: `tests/md_fixed_point.rs` and the `md-roundtrip.yml` gate in `8a3bd939`
("ci: the Markdown round trip, strict: md -> docx -> md and docx -> md -> docx -> md"),
with verdicts `committed_views_match_the_view`, `ids_name_blocks_notes_and_yaml_entries`
and friends; later fixture additions `97af74df`, replay filter `9f464d4a`.

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

### 2. One set of id types (spec §2.1) — partially landed

Landed: `src/ids/short.rs` (`ShortId`, `Step`, `Story`, `StoryId`) and
`tests/id_types.rs` in `2fbc8662` ("feat(ids): ShortId, the one type of every
short id (spec §2.1); only settled ids parse"), plus settlement fixes `ebe8f499`,
`1bf166f7`, `cfa9961e`. **Not landed:** `refs.rs` (`ChangeRef`, `CommentRef`,
`BlockSpec`, `Location` as clap-facing `FromStr` newtypes), `long.rs` (`LongId`
with `-XX` escaping), `spell.rs` (generated spelling lists). The contract
respecifies these: `LongId` becomes the typed four-field `PART.ELEMENT.ATTRIBUTE.VALUE`,
and `ShortId` gains the `p`/`ps` split with `u32` steps.

Red: `tests/id_types.rs` parses every id the view prints for the fixtures of task 1 and requires
`parse(print(x)) == x` and `print(parse(s)) == s`; unit tests for each kind and each refusal.

- `src/ids/` becomes a module: `short.rs` (`ShortId`, its path steps) [landed], `refs.rs` (`ChangeRef`,
  `CommentRef`, `BlockSpec`, `Location`, each a `FromStr` over `ShortId`) [not landed], `long.rs` (`LongId` and
  the `-XX` escaping) [not landed; the contract's `LongId` keeps the same escaping idea for the
  four-field form], `spell.rs` (the spelling lists and error texts, generated from the types) [not landed].
- The existing functions keep their signatures as thin wrappers until their callers move (task 3),
  so this PR changes no behaviour.

### 3. Every option and every printer goes through the types — partially landed

- clap fields take the types (`value_parser`): `--id`, `-p/--location`, `read -p`, `--reply`,
  `--resolve`. A bad id exits 2 with one message, in the binary, Python and npm.
  [Partially: `--id`/`-p` reach `ids::parse_change_ids`/`parse_location`, which refuse
  expired spellings with the settled hint and exit 2; ids still arrive as `String`
  clap fields, not typed `value_parser`s.]
- `read -p` drops `Select::parse` and its private grammar; `-p 12` is refused (exit 2) with the
  settled spelling the option takes (`p12`), as every expired id is (spec §2, decided 2026-10-13).
  [Partially: `Select::parse` still lives (`src/markdown/mod.rs`); `-p 12` is refused
  through `ShortId`, but comma lists (`p1,p2`) and open ranges (`p1-`, `p2:4`) still parse —
  the contract retires ranges and standardizes the grammar.]
- The view, JSON, hints and notes print ids through `Display`; the `format!` spellings above go.
  [Landed for text surfaces via `cab8ab73`, `1192f6bb`, `6fc0ab0e`; JSON fields still carry
  the transitional colon longs pending schema 2.]
- The wrappers of task 2 are deleted once unused. [The wrappers are still the public surface.]

### 4. Grammar completion — partially landed

Rows, nested tables, text-box paragraphs, ranges, `sN`, `tocN`, `ccN` in the types and in every
option that takes a location [landed in `ShortId`: `t0.row1`, `.cell2`, nested paths, `s1`,
`toc1`, `cc1`, `p1.cc0`; ranges parse only inside `Select::parse` — contract: retired; text-box
steps parse but resolve to `UNSUPPORTED_TARGET` under the contract]; the YAML `sections:` map prints `s1:` (today `2:`) [not landed: still `2:`]; `read --tables`
numbers nested tables as the view does; shift notes after an edit that adds or removes a block;
refusals of expired ids that name the settled spelling [landed]; help and `capabilities` stop advertising
bare numbers [landed].

### 5. The drift check — not landed

`scripts/check_ids.py` does not exist; no `debug --check ids` view. The contract's
plan re-anchors this idea (its own `id_census`/spelling gates).

`scripts/check_ids.py`, run in `docs.yml`, fails when:
- an id the view prints for a fixture does not parse (through `jubarte`'s own parser, by a hidden
  `debug --check ids` view);
- the spec's §2 table, `docs/locations.md` and the spelling lists of `spell.rs` disagree;
- the flag catalogue (`docs/flags_datetime.md`, PR #458) differs from `jubarte capabilities`.

### 6. Flag consolidation — partially landed

The audit's findings on location and id flags (`jubarte-loop/cli-audit/FINDINGS.md` A1, E1, G2): one
name per meaning across commands, the catalogue regenerated (`docs/locations.md`,
`docs/flags_datetime.md`, generated by `scripts/cli_locations.py`), retired spellings refused with a hint
[landed for the `#`/colon/bare-number forms; the contract adds `UNSUPPORTED_TARGET`,
`UNSUPPORTED_GRAMMAR` and the retired-range family].

### 7. Runs: highlight, strike, monospace — landed

Landed: `==x==` highlights (colour round-trip `f6bfe596`), `~~x~~` strike, and
backtick monospace (`Verbatim` in the reader; `hl-<colour>` classes in the view).

Arthur, 2026-10-13, "Simplify for now": `==x==` (a highlight, any colour), `~~x~~` (strike),
`` `x` `` (a monospace font). View and reader both; the docx verdicts of task 1 go green for the
`highlight` lines. Highlight colour, text colour and font names stay `✗ docx lost` until a later
task gives them a spelling. [Colours now round-trip as `hl-N` classes via `f6bfe596`.]

### 8. The reader: what the view writes comes back (Arthur, 2026-10-13: "Pay attention to the threaded replies … Spacing. Style legends. Mapping AC to an n counsel") — landed in substance

`62f14287` ("the reader brings back comment threads and numbers") covers 8a/8b:
`commentsExtended.xml` is written, `w15:paraIdParent` threads replies,
`w15:done` marks resolved, and comment `w:id`s keep the view's numbers. The
id-line and style-legend reads (8d/8e) shipped through `src/markdown/write.rs`
and the `fix/md-style-legend` line of work.

Found in `src/markdown/{write,xml,package}.rs` on 2026-10-13, in this order of work:

- **8a Threaded replies.** `c6 @AS re c5:` and `resolved` are parsed (`CommentHead.reply_to`,
  `.resolved`) and dropped: only `comments.xml` is written. Write a `w14:paraId` on each comment's
  last paragraph and `word/commentsExtended.xml` (`w15:commentEx w15:paraId`, `w15:paraIdParent`
  for a reply, `w15:done="1"` when resolved), its content type and relationship; Word-check the
  output (open it in Word, not only the validator).
- **8b Comment ids.** `w:id` is a running counter (`comments.len()`), so `c5` comes back as `c0`:
  keep the view's number.
- **8c Authors.** `@AC` → `Ann Counsel` and its date already map through the `authors:` table
  (`view_author`), initials from the handle: covered by a test in the round-trip gate, no change.
- **8d Spacing and the id line.** The id lines (`<!-- p3 BodyText center, first-line 0.5in, left
  0.5in, right 0.5in -->`) are not read: read the style, alignment, indents, spacing (`before`,
  `after`, `line … atLeast/exact`), `keep-next`, breaks and `empty` back into `w:pPr`.
- **8e The style legend.** The YAML `styles:` map (`Normal: Times New Roman 11pt, after 6pt, line
  12pt atLeast, left`; `"#": Heading1, Times New Roman bold 24pt, before 24pt, keep-next`) is not
  read: write it into `styles.xml` (fonts, sizes, bold, spacing, alignment, keep-next).
- Then: endnotes, empty paragraphs, cell paragraphs, a change printed in several blocks read back
  as one, and the nested change (pi F2 of #462).

### 9–13. The rest of the spec's order of work — mixed

Pictures,
bookmarks, internal links, body fields as code plus result, brace escaping (9)
[partially: images and `{field CODE}`/result forms round-trip; bookmark anchors
`{#name}` and links landed; gaps remain per `2026-10-14-anymd-imports.md`]. YAML
headers/footers/page/styles/sections, including a different first page (10)
[landed]. `convert -o X.md` in
the view dialect, `--plain` (11) [landed: `convert -o X.md` writes the agent
view; `--plain` the plain dialect]. Anchored mode (12) [not landed]. Long ids, then stamping behind the Word gate
(13) [superseded: contract v3 fixes longs at `PART.ELEMENT.ATTRIBUTE.VALUE`,
needs no stamping — keyless targets take `j:short` — and its own plan sequences
the work].

## Open with Arthur

1. Decided (Arthur, 2026-10-13): only the settled ids parse (spec §2). The plan's Q19 recommended
   `body:para:3F2A1B7C`; the spec's §4.1 settled `PART.ELEMENT.ATTRIBUTE.VALUE` since, and the
   2026-10-17 contract v3 confirms that four-field form. Update: the colon longs no longer parse
   at all — `body:rev:12` on the CLI is a refusal naming `r12`; they survive only as the
   transitional *output* inside `id`/`ref`/`paragraph`/`location` JSON fields until schema 2
   pairs `short_id`/`long_id`.
2. Q20 (deprecating `cN.text`) stays open until the Word investigation of the plan runs.
