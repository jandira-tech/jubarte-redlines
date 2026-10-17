<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids and the Markdown round trip: the settlement (2026-10-13) — spec v1, superseded in parts

Arthur, 2026-10-13: "I want to settle the id (ours) and the long id for good. I want to absolutely
ensure that docx to md to docx to md is as lossless as possible."

> **Status (audited 2026-10-18, `main` @ `aba48bd5`).** This is a historical document now. The
> short/long ID contract v3 —
> [2026-10-17-short-and-long-id-contract.md](2026-10-17-short-and-long-id-contract.md) (added in
> `4612a155`, merged `b5bdccbd` / PR #570) — supersedes it wherever they disagree; v3 §8 lists the
> deltas: a `ps` step for box and cell paragraphs split from body `p`, `u32` positions, no `+rawid`
> change pairs, no synthetic/rekeyed/dependent long keys, `j:short` for keyless targets, short ids
> in all agent-facing text, closed ranges only, and commas as option-level separators. Sections
> marked *superseded* below are the original decisions kept for the record, each annotated with
> what replaced it; sections marked *implemented* describe what `main` does today. The v3 contract
> itself is staged — **no branch implements it yet** — so in ID matters the code mostly still
> behaves as this spec describes. The earlier line "nothing here is implemented unless §7 says so"
> no longer holds: the reader, view, gate and §5.2 work below did ship (§7 records the merges).

This document was the final rule set on 13 October. Every **Decided** line is Arthur's choice of
2026-10-13; everything else followed from the evidence in §6 and from rules already agreed (D7:
text output prints short ids; the hierarchy rule: position and depth come from the real structure,
never from marker names). What was implemented is recorded in §7 and in the status notes below.

## 0. The rule above the rules

**Decided (Arthur, 2026-10-13): docx → md → docx → md is as lossless as possible, and nothing that has
a short id is ever lost.** Ids exist so jubarte can rebuild the docx the user expects (the JSON and
the Markdown already name what the docx will hold). When an id must be regenerated, what matters is
that the docx reflects the content the user sees: a regenerated id never drops or changes content,
and content without a good key is kept (its key is synthetic and marked), never tossed. **→ v3 §0
keeps this rule; "synthetic and marked" is now the allocated short label with its `j:short` long id
(v3 §4/§5).**

## 1. The model in one paragraph

A document has three layers of names. **Short ids** (`p12`, `t0.row1.cell2`, `r5`, `c3`) are
positional, readable, and what people and agents type. **Long ids**
(`document.w:p.w14:paraId.3F2A1B7C`) are stable keys taken from the XML, completed by synthetic keys
where the XML has none, and they appear in JSON. **Markdown content** carries everything inline
(text, formatting, links, bookmarks, pictures, equations, notes, fields), so it needs no id: it is
edited by editing its paragraph.

> **Status.** The two-layer model (short ids everywhere the agent works, long ids in `--json` only)
> is what v3 formalises; the examples above remain valid v3 spellings. The "synthetic keys where
> the XML has none" clause is superseded: v3 §4 gives a keyless target the long id
> `PART.ELEMENT.j:short.SHORT`. Long ids in this §4 grammar are not implemented at all today —
> `read --json` still prints the transitional `body:p:N`/`part:p:N` forms this spec wanted to
> retire (§2).

## 2. Short ids

**Decided: positional, completed (alternative A).** The alternatives were B, a key prefix (`p3F2A`:
stable across edits, but no order and hard to read), and C, a hybrid guard (`p12~3F2A`); C can be
added later without breaking A. Numbering is 0-based. One grammar, one parser (`src/ids.rs`), every
option that takes a location, printed and accepted alike. Positions count the drafter's real
structure.

| Kind | Spelling | Notes |
|-|-|-|
| Body paragraph | `p12` | paragraphs in table cells are numbered in the same sequence **→ v3 §2: cell paragraphs move to their own `ps` step, so this note expires; `p12` itself is unchanged.** Still current code behaviour. |
| Story paragraph | `header1.p0`, `footnotes.p3`, `endnotes.p0` | always with `.pN`. A bare `header1` names the part itself (the YAML `id:`); `edit -p header1` edits its first paragraph. The view prints `header1` for a header's first paragraph today: drift, fixed with the printers (plan task 3). Parts: `headerN`, `footerN`, `footnotes`, `endnotes`, `comments`, `styles`, `numbering`; `styles` and `numbering` hold no blocks, so they are named only by themselves and their changes (`styles.r0`) **→ v3 §2: same spellings, still live.** |
| Table | `t0` | top-level tables of a story, in order **→ v3 §2: unchanged.** |
| Row | `t0.row1` | **→ v3 §2: unchanged.** |
| Cell | `t0.row1.cell2` | its first paragraph, so `.p0` is never written; `.p1` for the next; the first paragraph's boxes and controls are `t0.row1.cell2.box0.p0`, `t0.row1.cell2.cc0` **→ superseded by v3 §2: the cell's own paragraphs are `t0.row1.cell2.ps0`, `t0.row1.cell2.ps1`, …; its boxes `t0.row1.cell2.box0.ps0`; controls stay `…cc0`. `t0.row1.cell2` alone still resolves to its first paragraph. Code today still uses `.p1`/`box0.p0`.** |
| Nested table | `t0.row1.cell2.t0.row0.cell0` | numbered inside its cell; `read --tables` uses the same numbering **→ v3 §2: same path shape; only the cell-paragraph step changes to `ps`.** |
| Text-box paragraph | `p14.box0.p0` | the box anchored in p14 **→ superseded by v3 §2: `p14.box0.ps0`. The `p14.box0.p0` form never shipped either: today the stamping walk skips `w:txbxContent` (`agent.rs` "text boxes excluded"), so a box's paragraphs print as unnamed blocks under their host paragraph and the drawing survives as a hidden payload. v3 §7 defers canonical box-local ids to the Future stage — the `…boxN.psM` syntax parses but resolves `UNSUPPORTED_TARGET` until then.** |
| Change | `r12`, pair `r1+2` (`r43+43` when Word gave both marks one `w:id`); in another part `header1.r1`, `footnotes.r1`, `styles.r0` | a change that spans blocks prints its note in each block; the copies are one change **→ superseded by v3 §2/§5: a shared `w:id` is a duplicate native key, so the second carrier gets an allocated label (`r0+1`, never `r43+43`); `r1+2` as "one change on two marks" goes away. Story changes (`header1.r1` …) keep their shape. Code and committed fixtures still print `r43+43`.** |
| Comment | `c5`; the text it covers `c5.text` | **→ v3 §2: unchanged.** |
| Range | `p3-p7`, `footnotes.p2-p3`, `t0.row1.cell2.p1-p3` | the second side renumbers the first's last step: same story, same kind, ascending (`p4-p4` is one block); wherever several blocks are accepted **→ v3 §2: closed ranges stay; the cell-range example becomes `t0.row1.cell2.ps1-ps3`; the open forms `p17-` and `-p3` that `read -p` accepts today are retired; a comma is an option-level separator, never part of one token.** |
| Section | `s0` … `sN` | in body order; the last is the body's own `w:sectPr`. Drift today: the YAML `sections:` map numbers from 1 and leaves out the first (`2:` is `s1`); it must print `s1:` **→ v3 §2 keeps `sN`; the YAML drift noted here is still unfixed on `main` (`view_stories.rs` still prints `2:` for `s1`).** |
| Table of contents | `toc0` | a TOC field, or the TOC content control around one, counted once **→ v3 §2: unchanged.** |
| Content control | `cc0` (block), `p12.cc0` (inline) | the one inline object with no Markdown sibling **→ v3 §2: unchanged.** |

Rules:

- **Stale ids.** A positional id goes stale when blocks move; the view's `sha256:` pin catches an id
  used against a changed file (`STALE_SOURCE`). Stability is the long id's job.
- **Decided: shifts are reported.** An edit that adds or removes a block prints a `note:` naming the
  ids that moved: `note: inserted p5; p5-p40 are now p6-p41`.
- **Decided (Arthur, 2026-10-13, superseding the deprecation period): only the settled ids parse.**
  "We should strictly have the settled short IDs or long IDs. Everything else should be expired."
  The short ids of this table and the long ids of §4 are the whole grammar (§2.2 is the exact
  contract). Expired, refused now: bare numbers (`12`), 0.12's `#12`, `#c27`, `#c27.text` and
  `t0.r1.c2`. The old long ids `body:rev:12` and `body:p:12` are what `read --json` prints today,
  so they parse, unadvertised, until the JSON prints the §4 long ids (plan task 12) and expire in
  that same change: jubarte never prints an id it refuses. The refusal names the settled spelling when one is clear (`12` →
  `p12`, `r12` or `c12` by the option; `body:rev:12` → `r12`; `t0.r1.c2` → `t0.row1.cell2`), and
  only a spelling that parses (`05` suggests nothing). Help, docs and `capabilities` list only the
  settled spellings. **Status: the settled spellings and the refusal hints did ship
  (`src/ids.rs`, `src/ids/short.rs`), and `body:rev:N`/`body:p:N` still parse unadvertised — but the
  §4 long ids they were waiting on never landed, so the transitional forms remain what
  `read --json`, `inspect --tables` (`paragraph_ids`) and edit-plan JSON print today
  (`src/ids.rs::paragraph_id`). Under v3 they expire for real at the schema-2 cutover (v3 §2, §7
  ASAP 5): nothing but the v3 grammar parses after it.**

### 2.1 One set of types (Arthur, 2026-10-13: "Don't forget about creating centralized types")

Every id is a value of one Rust type family in `src/ids.rs`, never a `String` passed around and
re-parsed. This settles the plan's Q18.

- `ShortId` is an enum with one variant per row of the table above (`Paragraph`, `StoryParagraph`,
  `Table`, `Row`, `Cell`, `BoxParagraph`, `Change`, `Comment`, `CommentText`, `Range`, `Section`,
  `Toc`, `ContentControl`). Nested positions are a path (`t0.row1.cell2.t0` is a list of steps), so
  nesting needs no new variant. **Status: implemented, but simplified — `src/ids/short.rs` shipped
  one `Step` enum (`Paragraph(usize)`, `Table`, `Row`, `Cell`, `Box`, `ContentControl`, …) rather
  than one `ShortId` variant per row, with positions as `usize`. v3 §7 (Urgent 1) splits the
  paragraph step into `p`/`ps` and moves positions to `u32`; neither is done.**
- The kinds an option takes are narrower types over the same values: `ChangeRef` (`--id`, `-p rN`),
  `CommentRef` (`--reply`, `--resolve`), `BlockSpec` (`read -p`: blocks and ranges), `Location`
  (`edit`/`add -p`). Each implements `FromStr` and is the clap field type (`value_parser`), so a bad
  id exits 2 with one message in the binary, Python and npm alike. **Status: implemented in shape
  (`FromStr` + clap `value_parser` on `src/ids.rs` types); v3 keeps the idea.**
- `LongId` is the type of §4 (`part`, `element`, `attribute`, `value`, with the `-XX` escaping in
  its `Display` and `FromStr`). **Status: not implemented — there is no `LongId` type today; the
  only "long" handling is the transitional `body:p:N`/`body:rev:N` parsing in `src/ids.rs`. v3 §4
  keeps the same four-field grammar plus `pkg:` and the `-XX` escaping.**
- `Display` is the only printer: the view, JSON, hints and notes format an id through it, so what
  jubarte prints is what it parses. A test parses every id the view prints for the corpus fixtures
  and requires `parse(print(x)) == x` and `print(parse(s)) == s`. **Status: implemented;
  `tests/ids_roundtrip.rs` enforces the round-trip today. v3 §7 (Urgent 3) replaces the ad-hoc
  printers with typed presenters.**
- The spelling lists (`CHANGE_SPELLINGS`, `PARAGRAPH_SPELLINGS`, …) and the error texts come from
  these types, and the drift check (§5.4) compares them with this table and `docs/locations.md`.
  **Status: implemented; `docs/locations.md` is generated by `scripts/cli_locations.py` from the
  live binary.**

### 2.2 The contract: every literal (Arthur, 2026-10-13: "I don't want examples, i want all literals")

> **Superseded by v3 §2.** This grammar is what `src/ids/short.rs` implements today. v3 replaces it:
> a `ps` step for box and cell paragraphs (`cell-tail` and `box-item` re-key on `ps`, never `p`)
> and `u32` positions. The `+` group selector stays (`r1+2`, `r1+2+3`, `header1.r1+4`), but its
> members are distinct allocated labels — a printed id never repeats a number, so `r43+43` is
> retired. Ranges are closed only (`p17-` / `-p3` refused), and a comma is an option-level
> separator, never part of one token. The grammar below stays for the record.

The only literals, besides digits: `p` `t` `row` `cell` `box` `cc` `r` `c` `s` `toc` `header`
`footer` `footnotes` `endnotes` `comments` `styles` `numbering` `text` `.` `+` `-`.

`N` is ASCII digits without a leading zero (`0`, or a nonzero digit then digits); `N1` is an `N`
other than `0`. `r`, `c`, `header` and `footer` take a `u32`; the block steps take a `usize`.

```
id          = block | range | change | "c" N | "c" N ".text" | "s" N | "toc" N

part        = "header" N | "footer" N | "footnotes" | "endnotes" | "comments"

block       = item | part | part "." item | "styles" | "numbering"
item        = para | table | "cc" N

para        = "p" N [ "." para-tail ]
para-tail   = "box" N "." box-item | "cc" N
box-item    = para | table

table       = "t" N [ "." "row" N [ "." "cell" N [ "." cell-tail ] ] ]
cell-tail   = "p" N1 [ "." para-tail ] | table | "box" N "." box-item | "cc" N

range       = ranged "-" W N
change      = [ part "." | "styles" "." | "numbering" "." ] "r" N { "+" N }
```

- `ranged` is a `block` other than `part`, `styles` and `numbering`; `W` is the word of its last
  step (`p`, `t`, `row`, `cell` or `cc`); the `N` after `-` is at least that step's number.
- Order, as the grammar fixes it: a part word comes first, once; `row` only after `t`, `cell` only
  after `row`; `box` only after `p` or `cell`, never last; `cc` always last; `p` after `cell` is
  `N1` (the cell is its first paragraph); `+` only between a change's numbers; `-` once, only in a
  range; `text` only in `c` `N` `.text`; `c`, `s` and `toc` take no part.
- The grammar recurses at `cell-tail → table` and `box-item → para | table`, the drafter's real
  nesting, so the set of ids is infinite while the kinds and their order are fixed.
- Everything else is refused, naming the settled spelling when one is clear (§2). The old long ids
  are the one temporary exception (§2).
- `src/ids/short.rs` implements this grammar; the drift check (plan task 5) tests the parser
  against it.

## 3. Inline objects are Markdown content

**Decided: inline objects are content, not ids.** Each has a Markdown sibling; the view prints it,
the reader reads it back, and it is edited by editing its paragraph.

| Object | Markdown | Notes |
|-|-|-|
| Run | text, `**b**`, `_i_`, `<u>u</u>`, `==highlight==`, `~~strike~~`, `` `code` `` | no id: Word splits and merges runs on save. **Not true today** (Arthur, 2026-10-13, on potpourri's r16): the view writes only bold, italic and underline (`ooxml.rs`, the span merge keys on those three), so highlight, strike, a monospace font and text colour are dropped. **Decided (Arthur, 2026-10-13: "Simplify for now"):** `==x==` a highlight of any colour; `~~x~~` strike; `` `x` `` a run in a monospace font. Highlight colours other than yellow, text colour and font names wait. **Status: implemented, and later widened — `main` round-trips all 16 named highlight colours as `<mark class="hl-<colour>">` (the `hl-` prefix reads case-insensitively; PR #550, merge `3a81ce55`); strike prints `~~x~~`; monospace faces print `` `x` `` via `is_monospace` (`from_docx/mod.rs`) and write back `Consolas`/`VerbatimChar` (`markdown/xml.rs`). Text colour and other font names are still not carried — that part of "wait" stands.** |
| Hyperlink | `[text](url "tooltip")` | the tooltip is the link title |
| Internal link | `[text](#bookmark)` | |
| Bookmark | `<a id="_Ref123"></a>` | the name is its identity |
| Picture | `![alt](media/image1.png "title")` | size and wrap in the id line: `<!-- p12 img0 2in x 1.5in, float right, wrap square -->` (`img0` only says which picture the clause describes) |
| Equation | `$…$`, `$$…$$` | body-level equations too |
| Note | `[^1]` in the text; `[^1]: …` at the end under its id line `<!-- footnotes.p0 -->` or `<!-- endnotes.p0 -->` | the id line keeps footnote and endnote apart |
| Field (body) | `{REF _Ref123 \h}Section 3.2{/}`: the code, then the result the reader sees | **Decided**: the content at that place matters, so the body keeps the result |
| Field (header, footer) | `{PAGE}`, `Page {PAGE} of {NUMPAGES}` | code only, as the YAML `headers:`/`footers:` lines print it: the result changes from page to page |

Field switches are part of the code. Literal braces in text print as `\{` and `\}`, so a template's
`{Client Name}` never reads back as a field.

What the dialect cannot say (crops, effects, rotation, exact anchor offsets, OLE payloads) anchored
mode keeps from the source, byte for byte while the Markdown leaves the object alone (§5.3).
**Status: anchored mode is not implemented and was re-scoped — it now lives in v3 §7 (ASAP 2).
Standalone mode does carry drawings byte-stable today: VML, DrawingML and `mc:AlternateContent`
survive as hidden payloads (PR #535, `7c407dea`).**

## 4. Long ids — superseded by v3 §4, not implemented

> **Status: none of §4 shipped.** There is no `LongId` type, no `PART.ELEMENT.ATTRIBUTE.VALUE`
> output, no keying, stamping or relationship pass on `main`; the only "long" spellings the code
> knows are the transitional `body:p:N` / `body:rev:N` / `story:p:N` forms (§2). v3 §4 keeps the
> four-field grammar, the `-XX` escaping and the native-key idea, but replaces the key-assignment
> rules of §4.3 entirely: no synthetic SHA keys, no re-keyed keys, no dependent keys; a keyless or
> unusable target gets `PART.ELEMENT.j:short.SHORT` (v2's `j:key` was already withdrawn in favour of
> it). The subsections below are the original design, annotated.

### 4.1 Grammar — mostly retained in v3 §4

**Decided: `PART.ELEMENT.ATTRIBUTE.VALUE`.** Examples: `document.w:p.w14:paraId.3F2A1B7C`,
`document.w:ins.w:id.12`, `footnotes.w:footnote.w:id.2`, `comments.w:comment.w:id.5`,
`document.w:tr.w14:paraId.1A2B3C4D`, `document.w:bookmarkStart.w:name._Ref123`,
`document.w:sdt.w:id.-2D1031330684`. **→ v3 §4: the grammar stands and gains `pkg:` for
package-level parts; but two of these examples are *not* valid public long ids under v3 —
`footnotes.w:footnote.w:id.2` (note containers have no public id; the note's paragraphs are
addressed) and `document.w:bookmarkStart.w:name._Ref123` (bookmarks are content, not entities —
v3's "deliberately absent" list).**

- Parse: split on the first three dots; the value is the rest.
- PART is the part's stem (`document`, `header1`, `footnotes`).
- ELEMENT and ATTRIBUTE pairs are checked against a table generated from ooxmlsdk's schema types,
  which also give the value types (`w14:paraId` is 8 hex digits, `w:id` a decimal). A content
  control's key is its `w:sdtPr/w:id` value, spelled `w:sdt.w:id`. **→ v3 §4: retained as the
  registered key table (`ELEMENT.ATTRIBUTE` pairs with typed getters).**
- Which attributes are unique keys is a short hand-written list (the schema does not say). **→ v3
  §4: retained.**
- Long ids appear in JSON, machine hints and `--long-id`; text output prints short ids. **→ v3:
  retained and tightened — `--json`/JSONL always emits the `short_id`/`long_id` pair; agent text
  never prints a long id.**

### 4.2 Shell safety — retained in v3 §4

The VALUE keeps `A-Z a-z 0-9 . _ :` literally. Every other character, `-` included, is written as
`-XX` per UTF-8 byte, uppercase hex (`,` → `-2C`, `å` → `-C3-A5`, `-` → `-2D`). Because `-` is never
literal, decoding is unambiguous, and an id never starts with `-` (every PART starts with a letter).
**→ v3 §4 keeps this codec verbatim (its "codec table"), including the `pkg:` payload which
re-escapes even `.` and `-`.**

Verified on the 36,415 distinct ids of the corpus, unquoted and byte for byte: zsh, bash, dash, csh,
tcsh and fish (macOS), PowerShell 7.6 (macOS), and cmd.exe and Windows PowerShell 5.1 (Windows
Server, `contabo`). Unescaped, 153 bookmark names broke PowerShell (`,` splits an argument, `”` is a
quote). `~HH` was rejected: bash expands a `~` that follows `=` or `:`.

### 4.3 Keys: from the XML, synthetic, re-keyed — superseded by v3 §4/§5

> **Superseded in full.** v3 §5 uses Word's own number when it is a valid native key of its type
> (`w14:paraId` `1..=0x7FFFFFFF`; every `w:id` over Word's signed `int` range, negatives included —
> v3 dropped v2's artificial zero floor) and gives everything else — missing, duplicate, malformed,
> out-of-range or noncanonical values — a deterministic allocated short label whose long id is
> `PART.ELEMENT.j:short.SHORT`. The `synthetic`/`rekeyed`/`dependent` machinery below (SHA-256 keys,
> first-holder re-keying, row-key-derived table and cell keys) does not exist in v3. v2 still had a
> SHA-256 `j:key`; v3 withdrew it too. None of the machinery below was ever implemented.

**Decided: synthetic keys, the same in the JSON and in every docx jubarte writes.** Every element
gets exactly one key, with its origin:

- `xml`: the attribute is in the file and its holder is the first one (§4.4).
- `synthetic`: a `w:p` or `w:tr` without `w14:paraId`. The key is a hash of the source's sha256, the
  part and the element's structural path (`body>tbl[2]>tr[3]`), as 8 uppercase hex digits in
  1..0x7FFFFFFF (MS-DOCX refuses 0 and values from 0x80000000). A value already used anywhere in the
  document is re-hashed with a counter until free. Reading a file twice gives the same keys, and the
  docx jubarte writes stamps exactly the keys `read --json` printed. A synthetic key is stable while
  the file is unchanged; once jubarte writes the file it is real XML. `--long-id` accepts a synthetic
  key on the unstamped source by recomputing it.
- `rekeyed`: a later holder of a repeated value. It gets a synthetic key, as Word does on save, and
  the JSON records the value it had.

**Dependent keys follow their parent.** A cell is its row's key plus `.cellN`; a table is its first
row's key. When a row is synthetic or re-keyed, its cells and its table take the new key with it, so
a repeated row key never leaves its cells ambiguous. **→ superseded by v3: a table's long id is for
its own `w:tbl`, a cell's for its own `w:tc`; no key is derived from another element's.**

**Bookmarks are never renamed.** REF fields and internal links name a bookmark by `w:name`, so
renaming would break them. A bookmark whose name repeats (34 in the corpus, most of them Word's own
`_GoBack` and Docxodus fixtures) is keyed by its `w:id`, which is required and unique:
`document.w:bookmarkStart.w:id.17`. **→ superseded: v3 gives bookmarks no public long id at all —
they are Markdown content (`<a id="…">`), not entities — so no element spells
`w:bookmarkStart.w:name` or `w:bookmarkStart.w:id`.**

**Revisions** keep `w:id`; the 11 repeated revision ids in the corpus follow the first-holder rule
(later holders are re-keyed, §4.3). **→ superseded: v3 keeps `w:id` as the native key (over Word's
full signed `int` range), but a repeated value makes every later holder a `j:short` target — there
is no re-keying, and no carrier keeps a colliding key.**

### 4.4 Determinism — superseded by v3 §3/§4

- **Canonical tree:** `mc:Choice` only; `mc:Fallback` is a copy and is skipped (the walker already
  does this). 175 of the 180 repeated paraIds in the corpus are one text box counted in both.
  **→ v3 §4: refined — the logical-content walker follows a fixed, versioned support profile;
  verified Choice/Fallback twins count once, but a duplicate-key heuristic alone never drops an
  entity.**
- **First holder** is decided document-wide in a fixed part order: `document`, headers, then footers
  by part name, `footnotes`, `endnotes`, `comments`, `glossary`. Zip entry order never matters.
  **→ v3 §4: retained in substance — fixed traversal: main document, headers and footers by
  canonical part-URI byte order, then footnotes, endnotes, comments, styles, numbering. The
  `glossary` part is not a v3 story.**
- **Never a key, never hashed:** `w14:textId` (1,415 repeated values within a part), `wp14:editId`,
  `w:rsid*`, timestamps. **→ v3 §4: retained — the registered key table is a short hand-written
  list; these stay out.**
- The source's sha256 is of the file's bytes, so a pure re-zip changes the synthetic keys of
  unkeyed elements. Word does not re-zip without editing, and jubarte's own write stamps real keys;
  if this ever matters, hash canonical XML instead. **→ moot under v3: there are no sha256-derived
  keys; `j:short` labels come from the inventory, not the file digest.**
- The algorithm is versioned (`schema`, `algo`). **→ v3: retained as the schema-2 cutover (v3 §7
  ASAP 5).**

### 4.5 Relationships in the JSON — superseded by v3 §5

> **Not implemented, and the shape below is mostly withdrawn.** Today's JSON emits a `ref` short
> field plus transitional colon long ids (`body:rev:12`, `body:p:5`), not this object. Under v3 the
> only guaranteed identity shape is the `IdRef` pair — `"short_id"` + `"long_id"`, every reference,
> parents included — with structured fields (`key_origin`, `part_uri`, `line_number`, …) for the
> rest. `key` (with `origin`), `path`, and a bare-string `parent` are not v3 fields: a parent, when
> reported, is itself a paired reference.

Computed from the file bytes on each read; nothing is stored. Each element carries:

```jsonc
{
  "long_id": "document.w:p.w14:paraId.3F2A1B7C",
  "short_id": "p12",
  "key": {"attribute": "w14:paraId", "value": "3F2A1B7C", "origin": "xml"},
  "path": "body>tbl[2]>tr[3]>tc[1]>p[0]",
  "parent": "document.w:tr.w14:paraId.1A2B3C4D",
  "referenced_by": [{"part": "commentsExtended", "attribute": "w15:paraId", "dangling": false}],
  "references": [{"kind": "pStyle", "target": "styles.w:style.w:styleId.Heading1", "dangling": false}]
}
```

- Only the comment family names a paragraph by `w14:paraId` (`commentsExtended` `w15:paraId` and
  `w15:paraIdParent`, `commentsIds` `w16cid:paraId`, `commentsExtensible` through `durableId`), and
  only paragraphs of `comments.xml`. A re-key updates exactly those references in the same pass, and
  a post-write check refuses any dangling family reference: a thread must not break. **→ v3: kept
  as the dangling-reference diagnostics (`dangling: true`-style structured fields); re-keying is
  gone, so nothing updates these references — ambiguity is reported, never repaired on read.**
- **Read tolerates, never repairs.** Files Word opens hold 58 dangling `w15:paraId`, 53
  `w16cid:paraId` and 22 `paraIdParent` references (mostly comment parts left behind after every
  comment was deleted); they are reported `dangling: true`. **→ v3 §5: same rule — constructing
  ids must not mutate the DOCX; no renumbering, renaming or attribute repair on read.**
- `_Toc*` and `_Ref*` bookmark names are marked `unstable`: Word renames them when it updates a TOC
  or a field. **→ moot under v3: bookmarks have no long ids to mark.**
- 518 paraIds above 0x7FFFFFFF (193 files, mostly Google-Docs-style exports) are read as they are and
  rewritten only when the file is stamped anyway. **→ v3 §5: these are the main `j:short`
  population — out-of-range paraIds take allocated short labels instead of being read as keys.**

### 4.6 Stamping — not implemented; deferred by v3 §7

Stamping a part declares `xmlns:w14` and `xmlns:mc` and lists `w14` in `mc:Ignorable` on its root; a
`w14` attribute without them is a Word-invalid shape and gets a Ring-1 check in `jubarte validate`
with a broken probe (8 corpus parts already use `w14:paraId` without `w14` in `mc:Ignorable`).
Gate before shipping: stamp the unkeyed corpus files (Docxodus fixtures included) and run the OOXML
validator, `jubarte validate` and Word (`word_pdf.py`, no repair prompt) on each.

**Status: never implemented — the read path does not mutate packages, and no write path stamps
`w14:paraId`. v3 §7 moves optional stamping to the Future stage; under v3 a keyless target's long id
is `j:short`-based and needs no stamp to be stable within a snapshot.**

## 5. The Markdown round trip

### 5.1 One dialect — implemented

`jubarte read` (the agent view) is the Markdown. `convert -o X.md` writes the same dialect, with a
`--plain` switch for id-free Markdown meant for people, which does not promise a round trip.
**Status: implemented; the shipped flag is `--commonmark`, not `--plain` (`cli.rs::MarkdownArgs`),
and `convert X.docx -t md` writes the agent view's YAML header and id lines by default.**

### 5.2 The reader reads everything the view writes — implemented, gate still red on some fixtures

Each item is a test: write, read back, compare. **Status: this whole list shipped through the
`fix/md-roundtrip-fidelity` stack (PR #515, merge `0460e47a`) and follow-ups — see §7. What remains
unfixed is measured by the strict gate (§5.4), which is red on `main` today.**

- YAML `authors:` table: `@AC` on a note becomes Ann Counsel with the header's date. **✔ shipped**
- Revision notes `{>>r0 @AC<<}` attribute the change; they never become comments. **✔ shipped**
- Comment notes `{>>c5 @AC: text<<}`, replies `{>>c6 @AS re c5: text<<}` and `resolved` rebuild the
  thread. **✔ shipped**
- Id-line properties: style, alignment, indents, spacing, keep-next, page break, section break.
  **✔ shipped**
- Empty paragraphs: `<!-- p4 empty -->`, `<!-- p4-p6 empty -->`, read back as that many paragraphs.
  **✔ shipped**
- Table cells: a cell's paragraphs printed with a mark distinct from a line break (`<br>` stays
  `w:br`), read back as paragraphs. **✔ shipped — but see the gate: `ids-tables` and
  `ids-table-changes` still fail md-side on `main`.**
- YAML `headers:`/`footers:`/`page:`/`styles:` rebuild those parts. **✔ shipped — `ids-header-footer`
  and `ids-sections` still fail md-side on `main`.**
- Everything in §3, endnotes included. **✔ shipped**
- A change inside a change: the view prints an insertion a second author deleted as
  `{++{--x--}{>>r47 @A<<}++}{>>r46 @A<<}`; the reader must read it back as `w:ins` holding `w:del`.
  Today it nests no change in a change, so `r47 @A` becomes a comment and `{--x--}` literal text
  (pi review of #462; `RP001-Tracked-Revisions-01.docx`, `file_176.docx`). **✔ shipped — nested
  changes now read back through the `on` grammar (`{>>r3 @B on r1 @A<<}`, `critic.rs`; PR #537,
  merge `cc1baf35`), though `ids-revisions` still fails md-side on `main`.**

### 5.3 Two modes — anchored mode superseded by v3 §7

- **Standalone** (`convert X.md -o X.docx`): rebuilds from the Markdown alone. Goal: the Markdown
  is a fixed point (md1 = md3, byte for byte apart from `source:`/`sha256:`). **✔ shipped — this is
  the only mode today; `md_fixed_point` measures it.**
- **Anchored**: the Markdown's differences from the source's own view are applied as edits to the
  source, so everything the Markdown does not express survives byte for byte. Each id line ties a
  Markdown block to its source element; a block with no id line is new. **→ not implemented; the
  mechanism moved to v3 §7 (ASAP 2 "anchored Markdown edits"), which re-specifies it against the
  pinned inventory. No `source:`/`sha256:` auto-anchoring and no `--standalone` flag exist.**

**Decided: anchored is automatic.** When the YAML `source:`/`sha256:` matches a file beside the
Markdown, the conversion anchors on it and prints `note: anchored on SOURCE.docx`; `--standalone`
opts out. **→ superseded: never implemented, and v3 §7 re-scopes the whole idea — the YAML
`source:`/`sha256:` pins still print on the view (the `STALE_SOURCE` pin), but no conversion
anchors on them.**

### 5.4 The gate — implemented, strict, and currently red on `main`

**Decided (Arthur, 2026-10-13): strict, "Fail fail fail until we fix".** No list of known failures,
no ratchet. `tests/md_fixed_point.rs` (workflow `md-roundtrip.yml`) runs every case to the end and
fails only then, printing a verdict for every id (Arthur: "I want the lots saying: this is wrong,
that is wrong, this is correct"):

**Status (2026-10-18, `main` @ `aba48bd5`, CI run 39005997156): the strict gate shipped and is
*failing* on `main` — it is doing its job, not regressing. The md → docx → md side fails on 14
fixtures: `docx-libraries-comparison`, `dolphins`, `eLife.00047`, `git_manuscript`, `ids-fonts`,
`ids-header-footer`, `ids-paragraphs`, `ids-revisions`, `ids-sections`, `ids-spacing-threads`,
`ids-table-changes`, `ids-tables`, `list_of_figures`, `received` (several are Markdown-exact but
show docx-side `debug --check render` differences; `ids-revisions`, `ids-spacing-threads`,
`ids-table-changes` and `ids-tables` have real wrong/lost ids). The docx → md → docx → md side
fails on 5 documents, `Strict01.docx` among them (its Markdown is a fixed point — 157 correct —
with 4 docx-side differences). This is pre-v3-contract code; the remaining work is tracked by the
round-trip issues and the v3 migration stages.**

- md → docx → md for every Markdown fixture in `tests/fixtures/md-roundtrip`, and docx → md → docx
  → md for every docx in its `docx.txt`: each id-line block, each note (`r1+2`, `c5`) with the span
  it follows, and each YAML entry is `✓ correct`, `✗ wrong` (with the first line that changed),
  `✗ lost` or `✗ new`.
- **The docx side as well.** A Markdown-to-Markdown comparison cannot see what the first docx → md
  step drops: both views drop it alike (potpourri's highlight passed as "correct" that way). So the
  docx jubarte writes is compared with the source docx through `jubarte debug A B --check render`
  (highlights, colours, fonts, styles, tables, sections, parts); every line it lists is a `✗ docx`
  verdict. A Markdown fixture listed in `views.txt` is compared with the docx it is the view of.
- The `ids-*.md` fixtures are views of documents dense in one kind of id (paragraphs, tables,
  revisions, comments, footnotes, endnotes); `committed_views_match_the_view` keeps every committed
  view equal to what `jubarte read` prints today.

`scripts/md_roundtrip.py` stays the corpus-wide measurement (every Nth document of each folder); its
numbers go in §6, not in the gate. It keeps the source away from the Markdown, so automatic
anchoring cannot make it measure nothing.

## 6. Evidence

The corpus is `tests/corpus` of the canonical checkout: 1,195 documents, the 689 tracked in git plus
the local-only `Docxodus` (321) and `_fixtures` folders (excluded through `.git/info/exclude`).
Real-world documents are `neurotic_docx_bench` (442).

### 6.1 Why the round trip loses content today

`scripts/md_roundtrip.py`, every 9th document of `neurotic_docx_bench` and
`_to_improve_accepted_changes` (61 documents), measured at commit 57fdce41, before the reader work.
`view` is `jubarte read`; `convert` is `convert -o X.md`, the older dialect.

| Dimension | view | convert |
|-|-|-|
| paragraph text | 24% | 27% |
| paragraph styles | 22% | 26% |
| numbered or not | 26% | 29% |
| b/i/u/highlight | 22% | 24% |
| page breaks | 27% | 29% |
| table cell text | 68% | 68% |
| header/footer text | 67% | 67% |
| note text | 93% | 95% |
| changes | 54% | 59% |
| comments | 54% | 77% |
| Markdown fixed point | **0%** | 42% |

This is not a reader regression: `convert -o X.md` round-trips the same in 0.11.2, 0.12.0 and today
(9 of 9 changes, 3 of 3 comments on `received.docx`), and the reader lost no code. The agent view
(`from_docx/agent.rs`, 2026-10-09, shipped in 0.12.0) replaced the full-name notes
`{>>Ann Counsel (2026-10-01T09:00:00Z)<<}`, which the reader parses, with handles `{>>r0 @AC<<}`, a
YAML header and id lines, and the reader was never taught them. Today's view also drops pictures,
bookmarks and internal links (`[text](#name)` prints as underlined text), prints body fields as
their result only, and the reader has no endnotes.

### 6.2 What the short ids and the Markdown reach

`scripts/id_census.py`, each object counted once (a bookmark by its start, a field by its begin,
`mc:Fallback` copies skipped): 286,500 objects.

| Object | Count | Documents | Reached by |
|-|-:|-:|-|
| run | 132,556 | 1,184 | Markdown |
| body paragraph | 63,406 | 1,194 | `p12` |
| paragraph-mark revision | 21,752 | 210 | `r12` |
| cell | 20,976 | 400 | `t0.row1.cell2` |
| text revision | 17,401 | 288 | `r12` |
| row | 6,323 | 400 | `t0.row1` |
| bookmark | 5,994 | 421 | Markdown |
| story paragraph | 3,635 | 350 | `header1.p0` |
| field | 2,665 | 240 | Markdown |
| property revision | 2,003 | 240 | `r12` |
| hyperlink | 1,751 | 124 | Markdown |
| table | 1,643 | 400 | `t0` |
| section | 1,490 | 1,174 | `s0` |
| header/footer part | 1,055 | 302 | `header1` |
| picture (DrawingML, VML, OLE) | 695 | 258 | Markdown |
| comment and its paragraph | 1,056 | 157 | `c5` |
| footnote/endnote | 524 | 56 | `footnotes.p3` |
| note reference | 520 | 52 | Markdown |
| text-box paragraph | 306 | 62 | `p14.box0.p0` → **v3 §2: `p14.box0.ps0`; today unreachable — box paragraphs print unnamed** |
| content control | 310 | 52 | `cc0`, `p12.cc0` |
| move, table-structure revision | 236 | 49 | `r12` |
| equation | 125 | 30 | Markdown |
| table of contents | 67 | 67 | `toc0` |
| nested table | 11 | 9 | `t0.row1.cell2.t0` |

The census counted a row's own `w:trPr/w:ins` and `w:trPr/w:del` as paragraph-mark revisions; they
insert or delete the row, a table-structure revision (CodeRabbit on #462, fixed in the script with
`scripts/test_id_census.py`). On `tests/corpus` (689 documents) the fix moves 2,313 revisions from
the paragraph-mark row (21,130 → 18,817) to the table-structure row (77 → 2,390). The table above
predates the fix: read its paragraph-mark count as high and its table-structure count as low by
about that share. Both are reached by `r12` either way.

`scripts/id_census.py` as shipped (re-run in the pi review of #462): short ids reach 152,953
objects (53.39%), the Markdown 133,076 (46.45%), and one equation reaches neither. The script counts
an object inside a paragraph that a short id names among the short ids, which an earlier hand count
(142,194 and 144,306) did not. Real-world (`neurotic_docx_bench`): 56.84% and 43.16%; Docxodus:
32.72% (plus 0.95% by bookmark name) and 66.32%.

### 6.3 How often an element already has its key

On 146,300 keyable elements (paragraphs, rows, cells, tables, revisions, comments, notes,
bookmarks, drawings), checked independently by two agents (`jubarte-loop/longid-check`,
`longid-check-swe2`):

- 92.17% unambiguous from the XML as it is (real-world 97.71%, Docxodus 45.59%); the rest are
  elements without a `w14:paraId` (about 7.5%, 8,566 of them in Docxodus) and repeated keys.
- `w:p` and `w:tr` carry `w14:paraId` on 97.7% of real-world bodies; per document, 392 of 442
  real-world files key every body paragraph, 41 none, 9 some. `w:tc` and `w:tbl` never have a key.
- Revisions, comments, notes and bookmarks always have `w:id` (required).
- 49.4% of `w:ins`/`w:del` sit inside `w:rPr` or `w:trPr` (paragraph-mark and row revisions); 1,348
  of 1,860 `w:footnote`/`w:endnote` are separators, not notes.

With synthetic and re-keyed keys (§4.3) every element has a unique long id. **→ under v3 the same
coverage goal is met differently: a usable native key, else `j:short` — no synthetic hashes, no
re-keying (§4.3 note above).**

## 7. Order of work (PR over PR) — superseded by v3 §7; status as of `aba48bd5`

The step-by-step plan is `docs/superpowers/plans/2026-10-13-ids-consolidation.md`. The round-trip
items landed through `fix/md-roundtrip-fidelity` (PR #515, merge `0460e47a`) plus follow-ups:
#535 drawings (`7c407dea`), #536 strict gate (`75de15c4`), #537 nested changes (`cc1baf35`),
#538 escaping (`2b68338e`), #539 defaults (`4dc724cc`), #540 closers (`662ba929`) and #550
highlight fixes (`3a81ce55`). The ID-contract items that did not land were re-planned by v3 §7
(Urgent 1–4, ASAP 1–5, Future).

1. Reader: authors table, revision notes, comment heads. **✔ shipped.**
2. The strict gate (§5.4), red by design. **✔ shipped — and still red; see §5.4 status.**
3. One set of id types (§2.1); every option's parser and printer goes through them; `read -p` joins
   the grammar. **✔ shipped (`src/ids.rs`, `src/ids/short.rs`) — but `read -p` also accepts the open
   ranges `p17-`/`-p3` and comma lists that v3 §2 retires, and positions are `usize`, not `u32`.**
4. Short-id grammar completion: rows, nested tables, text boxes, ranges, `sN` (and the YAML
   `sections:` keys), `tocN`, `ccN`; shift notes; bare-number deprecation notes. **Partially
   shipped: rows, nested tables, closed ranges, `ccN`, `tocN`, shift notes and the bare-number
   refusals are in; the `sections:` YAML still prints `2:` for `s1`; text-box paragraphs print
   unnamed, so `…boxN.` steps never emitted; v3 renames the cell/box paragraph step to `ps`.**
5. The drift check: every id printed parses, the §2 table, `docs/locations.md` and the flag
   catalogue agree with the types. **✔ shipped — `tests/ids_roundtrip.rs` round-trips printed ids;
   `docs/locations.md` is regenerated by `scripts/cli_locations.py`.**
6. View and reader, runs: highlight, strike, monospace, colour (§3, once Arthur picks the spelling).
   **✔ shipped, wider than decided — see §3 status (all 16 highlight colours; no text colour).**
7. Reader: comment threads and `resolved`; endnotes; id-line properties; empty paragraphs; cell
   paragraphs; a change printed in several blocks read back as one. **✔ shipped.**
8. View and reader: pictures, bookmarks, internal links, body fields as code plus result, literal
   brace escaping. **✔ shipped (drawings incl. VML/AlternateContent survive as payloads, #535;
   `{REF …}{/}` field syntax and `\{` escaping are in the dialect).**
9. Reader: YAML headers/footers/page/styles. **✔ shipped.**
10. `convert -o X.md` writes the view dialect; `--plain` for people. **✔ shipped as
    `convert X.docx -t md` + `--commonmark` (§5.1).**
11. Anchored mode, automatic, with `--standalone`. **✘ not implemented — moved to v3 §7 ASAP 2.**
12. Long ids: grammar, escaping, keys, relationships in `read --json`, `--long-id`. **✘ not
    implemented — v3 §4/§5 re-specifies the whole thing (`j:short`, `IdRef` pairs); no `LongId`
    type or `--long-id` flag exists.**
13. Stamping, behind the Word gate (§4.6). **✘ not implemented — optional stamping is deferred to
    v3 §7 Future.**

## 8. The Markdown writer: what to take from remark-docx and mdast2docx — status as of `aba48bd5`

Arthur, 2026-10-13: eight agents compared jubarte with
[remark-docx](https://github.com/inokawa/remark-docx) and
[mdast2docx](https://github.com/md2docx/mdast2docx) file by file
(`jubarte-loop/cli-std/remarkdocx/CONSOLIDATED.md` and the per-slice
reports beside it). Ranked:

1. **LaTeX → OMML writer** (all four math reports). Turn on `ENABLE_MATH`,
   give `Content` a math arm, and write the inverse of the reader's
   OMML→LaTeX table (`from_docx/mod.rs:2302-2448`). Display =
   `m:oMathPara` (both TS libraries get this wrong). Turns `math.md` green.
   **✔ shipped: `ENABLE_MATH` is on (`write.rs`), `math::omml` writes the inverse table, and
   `math.md` is green in the gate.**
2. **Heading bookmarks** (empryo inline, empryo three-way, crush core).
   Done on branch 25a (9eafa752): a heading a link names gets the GitHub
   slug as its bookmark; unnamed headings get none; names over 40
   characters are cut and names that start with no letter take `_`, both
   checked in Word. Open from pi's review: duplicates must follow
   github-slugger (retry the suffixed name: `Intro`, `Intro`, `Intro 1` →
   `intro`, `intro-1`, `intro-1-1`), combining marks stay, a no-break space
   is dropped (only U+0020 becomes `-`), an empty heading gets no bookmark.
   **✔ shipped — `9eafa752` is on `main`.**
3. **Task items as Word's own checklist** (crush core and empryo
   three-way named mdast2docx's `w14:checkbox` content control; pi preferred
   numbering glyphs). Arthur's Word sample
   (`tests/fixtures/agent-view/word-checklist.docx`, Word's PDF beside it in
   the reports folder) settles it, to be mimicked exactly: a bullet
   `w:abstractNum` whose nine levels are all `w:lvlText` U+F0A8 in
   Wingdings (`w:ind` left 720·(level+1), hanging 360), on `ListParagraph`
   items; a checked item is `<w:strike/>` on the paragraph mark's `w:rPr`
   and on every run (the box takes the mark's formatting, so it is struck
   too); no `w:sdt`, no ☐/☑ text. The reader prints `- [ ]` / `- [x]` from
   that list and the struck mark, and the strikes of a checked item print no
   `~~`. Today the writer prints `☒ ` text after a bullet and the reader
   prints the sample as plain bullets, dropping the strike (`w:val="1"`).
   **✘ still true: the writer emits `☐ `/`☒ ` text runs (`write.rs`,
   `TaskListMarker`); no Wingdings `F0A8` `w:abstractNum` and no mark-strike handling exists.**
4. **Inline HTML breadth**: `<b>/<strong>`, `<i>/<em>`, `<s>/<del>`,
   `<code>`, `<mark>` on the existing counters; warn when HTML is dropped.
   **✔ shipped in substance — the counters exist incl. `<mark class="hl-…">`.**
5. **Tracking and columns.** The element is `w:trackRevisions` (there is no
   `trackChanges`): the reader already prints `track_changes: on|off`, the
   writer ignores it; honour the key rather than turn tracking on for every
   output with revisions. **✔ shipped — `track_changes:` is honoured on write
   (`markdown/mod.rs`, `settings.rs` `trackRevisions`), and `--tracking` overrides it.** Columns:
   done on branch 25a — `columns 2` and `column gap 0.25in` are `page:` clauses written as
   `w:cols`. **✔ on `main`.**
6. **Smaller**: `wp:docPr title` for a picture's title; linked
   `FootnoteTextChar`; a code span inside a link keeps its code face
   (`xml.rs:608` lets `link` mask `code`); fuller core properties from
   front matter; `fontTable` entries for legend fonts; remark-docx's fixture
   shapes (headings in lists and quotes, notes inside links/bold/quotes,
   reference links, fence info strings, nested tasks, an HTML zoo) as new
   tests.

Not adopted: one footnote id for two references to `[^n]` (Word gives a
note one reference; a second mention is a `NOTEREF` field), remark-docx's
quote indent on headings, mdast2docx's `emphasisMark` beside a highlight.
