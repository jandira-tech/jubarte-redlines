<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids and the Markdown round trip: the settlement (2026-10-13)

Arthur, 2026-10-13: "I want to settle the id (ours) and the long id for good. I want to absolutely
ensure that docx to md to docx to md is as lossless as possible."

This document is the final rule set. Every **Decided** line is Arthur's choice of 2026-10-13;
everything else follows from the evidence in §6 and from rules already agreed (D7: text output
prints short ids; the hierarchy rule: position and depth come from the real structure, never from
marker names). Nothing here is implemented unless §7 says so.

## 0. The rule above the rules

**Decided (Arthur, 2026-10-13): docx → md → docx → md is as lossless as possible, and nothing that has
a short id is ever lost.** Ids exist so jubarte can rebuild the docx the user expects (the JSON and
the Markdown already name what the docx will hold). When an id must be regenerated, what matters is
that the docx reflects the content the user sees: a regenerated id never drops or changes content,
and content without a good key is kept (its key is synthetic and marked), never tossed.

## 1. The model in one paragraph

A document has three layers of names. **Short ids** (`p12`, `t0.row1.cell2`, `r5`, `c3`) are
positional, readable, and what people and agents type. **Long ids**
(`document.w:p.w14:paraId.3F2A1B7C`) are stable keys taken from the XML, completed by synthetic keys
where the XML has none, and they appear in JSON. **Markdown content** carries everything inline
(text, formatting, links, bookmarks, pictures, equations, notes, fields), so it needs no id: it is
edited by editing its paragraph.

## 2. Short ids

**Decided: positional, completed (alternative A).** The alternatives were B, a key prefix (`p3F2A`:
stable across edits, but no order and hard to read), and C, a hybrid guard (`p12~3F2A`); C can be
added later without breaking A. Numbering is 0-based. One grammar, one parser (`src/ids.rs`), every
option that takes a location, printed and accepted alike. Positions count the drafter's real
structure.

| Kind | Spelling | Notes |
|-|-|-|
| Body paragraph | `p12` | paragraphs in table cells are numbered in the same sequence |
| Story paragraph | `header1.p0`, `footnotes.p3`, `endnotes.p0` | always with `.pN`. A bare `header1` names the part itself (the YAML `id:`); `edit -p header1` edits its first paragraph. The view prints `header1` for a header's first paragraph today: drift, fixed with the printers (plan task 3). Parts: `headerN`, `footerN`, `footnotes`, `endnotes`, `comments`, `styles`, `numbering`; `styles` and `numbering` hold no blocks, so they are named only by themselves and their changes (`styles.r0`) |
| Table | `t0` | top-level tables of a story, in order |
| Row | `t0.row1` | |
| Cell | `t0.row1.cell2` | its first paragraph, so `.p0` is never written; `.p1` for the next; the first paragraph's boxes and controls are `t0.row1.cell2.box0.p0`, `t0.row1.cell2.cc0` |
| Nested table | `t0.row1.cell2.t0.row0.cell0` | numbered inside its cell; `read --tables` uses the same numbering |
| Text-box paragraph | `p14.box0.p0` | the box anchored in p14 |
| Change | `r12`, pair `r1+2` (`r43+43` when Word gave both marks one `w:id`); in another part `header1.r1`, `footnotes.r1`, `styles.r0` | a change that spans blocks prints its note in each block; the copies are one change |
| Comment | `c5`; the text it covers `c5.text` | |
| Range | `p3-p7`, `footnotes.p2-p3`, `t0.row1.cell2.p1-p3` | the second side renumbers the first's last step: same story, same kind, ascending (`p4-p4` is one block); wherever several blocks are accepted |
| Section | `s0` … `sN` | in body order; the last is the body's own `w:sectPr`. Drift today: the YAML `sections:` map numbers from 1 and leaves out the first (`2:` is `s1`); it must print `s1:` |
| Table of contents | `toc0` | a TOC field, or the TOC content control around one, counted once |
| Content control | `cc0` (block), `p12.cc0` (inline) | the one inline object with no Markdown sibling |

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
  settled spellings.

### 2.1 One set of types (Arthur, 2026-10-13: "Don't forget about creating centralized types")

Every id is a value of one Rust type family in `src/ids.rs`, never a `String` passed around and
re-parsed. This settles the plan's Q18.

- `ShortId` is an enum with one variant per row of the table above (`Paragraph`, `StoryParagraph`,
  `Table`, `Row`, `Cell`, `BoxParagraph`, `Change`, `Comment`, `CommentText`, `Range`, `Section`,
  `Toc`, `ContentControl`). Nested positions are a path (`t0.row1.cell2.t0` is a list of steps), so
  nesting needs no new variant.
- The kinds an option takes are narrower types over the same values: `ChangeRef` (`--id`, `-p rN`),
  `CommentRef` (`--reply`, `--resolve`), `BlockSpec` (`read -p`: blocks and ranges), `Location`
  (`edit`/`add -p`). Each implements `FromStr` and is the clap field type (`value_parser`), so a bad
  id exits 2 with one message in the binary, Python and npm alike.
- `LongId` is the type of §4 (`part`, `element`, `attribute`, `value`, with the `-XX` escaping in
  its `Display` and `FromStr`).
- `Display` is the only printer: the view, JSON, hints and notes format an id through it, so what
  jubarte prints is what it parses. A test parses every id the view prints for the corpus fixtures
  and requires `parse(print(x)) == x` and `print(parse(s)) == s`.
- The spelling lists (`CHANGE_SPELLINGS`, `PARAGRAPH_SPELLINGS`, …) and the error texts come from
  these types, and the drift check (§5.4) compares them with this table and `docs/locations.md`.

### 2.2 The contract: every literal (Arthur, 2026-10-13: "I don't want examples, i want all literals")

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
| Run | text, `**b**`, `_i_`, `<u>u</u>`, `==highlight==`, `~~strike~~`, `` `code` `` | no id: Word splits and merges runs on save. **Not true today** (Arthur, 2026-10-13, on potpourri's r16): the view writes only bold, italic and underline (`ooxml.rs`, the span merge keys on those three), so highlight, strike, a monospace font and text colour are dropped. **Decided (Arthur, 2026-10-13: "Simplify for now"):** `==x==` a highlight of any colour; `~~x~~` strike; `` `x` `` a run in a monospace font. Highlight colours other than yellow, text colour and font names wait |
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

## 4. Long ids

### 4.1 Grammar

**Decided: `PART.ELEMENT.ATTRIBUTE.VALUE`.** Examples: `document.w:p.w14:paraId.3F2A1B7C`,
`document.w:ins.w:id.12`, `footnotes.w:footnote.w:id.2`, `comments.w:comment.w:id.5`,
`document.w:tr.w14:paraId.1A2B3C4D`, `document.w:bookmarkStart.w:name._Ref123`,
`document.w:sdt.w:id.-2D1031330684`.

- Parse: split on the first three dots; the value is the rest.
- PART is the part's stem (`document`, `header1`, `footnotes`).
- ELEMENT and ATTRIBUTE pairs are checked against a table generated from ooxmlsdk's schema types,
  which also give the value types (`w14:paraId` is 8 hex digits, `w:id` a decimal). A content
  control's key is its `w:sdtPr/w:id` value, spelled `w:sdt.w:id`.
- Which attributes are unique keys is a short hand-written list (the schema does not say).
- Long ids appear in JSON, machine hints and `--long-id`; text output prints short ids.

### 4.2 Shell safety

The VALUE keeps `A-Z a-z 0-9 . _ :` literally. Every other character, `-` included, is written as
`-XX` per UTF-8 byte, uppercase hex (`,` → `-2C`, `å` → `-C3-A5`, `-` → `-2D`). Because `-` is never
literal, decoding is unambiguous, and an id never starts with `-` (every PART starts with a letter).

Verified on the 36,415 distinct ids of the corpus, unquoted and byte for byte: zsh, bash, dash, csh,
tcsh and fish (macOS), PowerShell 7.6 (macOS), and cmd.exe and Windows PowerShell 5.1 (Windows
Server, `contabo`). Unescaped, 153 bookmark names broke PowerShell (`,` splits an argument, `”` is a
quote). `~HH` was rejected: bash expands a `~` that follows `=` or `:`.

### 4.3 Keys: from the XML, synthetic, re-keyed

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
a repeated row key never leaves its cells ambiguous.

**Bookmarks are never renamed.** REF fields and internal links name a bookmark by `w:name`, so
renaming would break them. A bookmark whose name repeats (34 in the corpus, most of them Word's own
`_GoBack` and Docxodus fixtures) is keyed by its `w:id`, which is required and unique:
`document.w:bookmarkStart.w:id.17`.

**Revisions** keep `w:id`; the 11 repeated revision ids in the corpus follow the first-holder rule.

### 4.4 Determinism

- **Canonical tree:** `mc:Choice` only; `mc:Fallback` is a copy and is skipped (the walker already
  does this). 175 of the 180 repeated paraIds in the corpus are one text box counted in both.
- **First holder** is decided document-wide in a fixed part order: `document`, headers, then footers
  by part name, `footnotes`, `endnotes`, `comments`, `glossary`. Zip entry order never matters.
- **Never a key, never hashed:** `w14:textId` (1,415 repeated values within a part), `wp14:editId`,
  `w:rsid*`, timestamps.
- The source's sha256 is of the file's bytes, so a pure re-zip changes the synthetic keys of
  unkeyed elements. Word does not re-zip without editing, and jubarte's own write stamps real keys;
  if this ever matters, hash canonical XML instead.
- The algorithm is versioned (`schema`, `algo`).

### 4.5 Relationships in the JSON

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
  a post-write check refuses any dangling family reference: a thread must not break.
- **Read tolerates, never repairs.** Files Word opens hold 58 dangling `w15:paraId`, 53
  `w16cid:paraId` and 22 `paraIdParent` references (mostly comment parts left behind after every
  comment was deleted); they are reported `dangling: true`.
- `_Toc*` and `_Ref*` bookmark names are marked `unstable`: Word renames them when it updates a TOC
  or a field.
- 518 paraIds above 0x7FFFFFFF (193 files, mostly Google-Docs-style exports) are read as they are and
  rewritten only when the file is stamped anyway.

### 4.6 Stamping

Stamping a part declares `xmlns:w14` and `xmlns:mc` and lists `w14` in `mc:Ignorable` on its root; a
`w14` attribute without them is a Word-invalid shape and gets a Ring-1 check in `jubarte validate`
with a broken probe (8 corpus parts already use `w14:paraId` without `w14` in `mc:Ignorable`).
Gate before shipping: stamp the unkeyed corpus files (Docxodus fixtures included) and run the OOXML
validator, `jubarte validate` and Word (`word_pdf.py`, no repair prompt) on each.

## 5. The Markdown round trip

### 5.1 One dialect

`jubarte read` (the agent view) is the Markdown. `convert -o X.md` writes the same dialect, with a
`--plain` switch for id-free Markdown meant for people, which does not promise a round trip.

### 5.2 The reader reads everything the view writes

Each item is a test: write, read back, compare.

- YAML `authors:` table: `@AC` on a note becomes Ann Counsel with the header's date.
- Revision notes `{>>r0 @AC<<}` attribute the change; they never become comments.
- Comment notes `{>>c5 @AC: text<<}`, replies `{>>c6 @AS re c5: text<<}` and `resolved` rebuild the
  thread.
- Id-line properties: style, alignment, indents, spacing, keep-next, page break, section break.
- Empty paragraphs: `<!-- p4 empty -->`, `<!-- p4-p6 empty -->`, read back as that many paragraphs.
- Table cells: a cell's paragraphs printed with a mark distinct from a line break (`<br>` stays
  `w:br`), read back as paragraphs.
- YAML `headers:`/`footers:`/`page:`/`styles:` rebuild those parts.
- Everything in §3, endnotes included.
- A change inside a change: the view prints an insertion a second author deleted as
  `{++{--x--}{>>r47 @A<<}++}{>>r46 @A<<}`; the reader must read it back as `w:ins` holding `w:del`.
  Today it nests no change in a change, so `r47 @A` becomes a comment and `{--x--}` literal text
  (pi review of #462; `RP001-Tracked-Revisions-01.docx`, `file_176.docx`).

### 5.3 Two modes

- **Standalone** (`convert X.md -o X.docx`): rebuilds from the Markdown alone. Goal: the Markdown
  is a fixed point (md1 = md3, byte for byte apart from `source:`/`sha256:`).
- **Anchored**: the Markdown's differences from the source's own view are applied as edits to the
  source, so everything the Markdown does not express survives byte for byte. Each id line ties a
  Markdown block to its source element; a block with no id line is new.

**Decided: anchored is automatic.** When the YAML `source:`/`sha256:` matches a file beside the
Markdown, the conversion anchors on it and prints `note: anchored on SOURCE.docx`; `--standalone`
opts out.

### 5.4 The gate

**Decided (Arthur, 2026-10-13): strict, "Fail fail fail until we fix".** No list of known failures,
no ratchet. `tests/md_fixed_point.rs` (workflow `md-roundtrip.yml`) runs every case to the end and
fails only then, printing a verdict for every id (Arthur: "I want the lots saying: this is wrong,
that is wrong, this is correct"):

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
| text-box paragraph | 306 | 62 | `p14.box0.p0` |
| content control | 310 | 52 | `cc0`, `p12.cc0` |
| move, table-structure revision | 236 | 49 | `r12` |
| equation | 125 | 30 | Markdown |
| table of contents | 67 | 67 | `toc0` |
| nested table | 11 | 9 | `t0.row1.cell2.t0` |

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

With synthetic and re-keyed keys (§4.3) every element has a unique long id.

## 7. Order of work (PR over PR)

The step-by-step plan is `docs/superpowers/plans/2026-10-13-ids-consolidation.md`.

1. Reader: authors table, revision notes, comment heads. *(Done on branch 22b.)*
2. The strict gate (§5.4), red by design. *(Branch 22c.)*
3. One set of id types (§2.1); every option's parser and printer goes through them; `read -p` joins
   the grammar.
4. Short-id grammar completion: rows, nested tables, text boxes, ranges, `sN` (and the YAML
   `sections:` keys), `tocN`, `ccN`; shift notes; bare-number deprecation notes.
5. The drift check: every id printed parses, the §2 table, `docs/locations.md` and the flag
   catalogue agree with the types.
6. View and reader, runs: highlight, strike, monospace, colour (§3, once Arthur picks the spelling).
7. Reader: comment threads and `resolved`; endnotes; id-line properties; empty paragraphs; cell
   paragraphs; a change printed in several blocks read back as one.
8. View and reader: pictures, bookmarks, internal links, body fields as code plus result, literal
   brace escaping.
9. Reader: YAML headers/footers/page/styles.
10. `convert -o X.md` writes the view dialect; `--plain` for people.
11. Anchored mode, automatic, with `--standalone`.
12. Long ids: grammar, escaping, keys, relationships in `read --json`, `--long-id`.
13. Stamping, behind the Word gate (§4.6).
