<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids and the Markdown round trip: the settlement (2026-10-13)

Arthur, 2026-10-13: "I want to settle the id (ours) and the long id for good. I want to absolutely
ensure that docx to md to docx to md is as lossless as possible."

This document sets the final rules. Arthur decided the three open choices on 2026-10-13 (marked
**Decided**); everything else follows from the evidence below and from rules already agreed (D7,
the hierarchy rule: depth and position come from the real structure, never from marker names).

## 1. Evidence

### 1.1 What the round trip loses today

`scripts/md_roundtrip.py` (deterministic; every 9th document of `tests/corpus/neurotic_docx_bench`
and `_to_improve_accepted_changes`, 61 documents). Each dimension compares `read --json` of the
original with `read --json` of the round-tripped docx. `view` is our Markdown (`jubarte read`);
`convert` is `jubarte convert -o X.md`, a second, poorer dialect.

| Dimension | view | convert |
|-|-|-|
| text (paragraph sequence) | 24% kept | 27% |
| styles | 22% | 26% |
| numbering | 26% | 29% |
| formatting (b/i/u/highlight) | 22% | 24% |
| page breaks | 27% | 29% |
| tables | 68% | 68% |
| headers/footers | 67% | 67% |
| notes | 93% | 95% |
| changes (kind, author, date, text) | 54% | 59% |
| comments (author, date, text, thread) | 54% | 77% |
| Markdown fixed point (md1 = md3) | **0%** | 42% |

This is not a reader regression: `convert -o X.md` round-trips the same in 0.11.2, 0.12.0 and
today (9 of 9 changes, 3 of 3 comments on `received.docx`), and the reader has lost no code. The
agent view (`from_docx/agent.rs`, 2026-10-09, shipped in 0.12.0) replaced the full-name notes
`{>>Ann Counsel (2026-10-01T09:00:00Z)<<}`, which the reader parses, with handles `{>>r0 @AC<<}`, a
YAML header and id lines, and the reader was never taught them. Our own dialect round-trips worse
than the poorer one, because the Markdown reader does not read what the view writes:

- `{>>r0 @AC<<}` (a revision's note) is read as a new comment by "Redline" whose text is `r0 @AC`.
- `@AC` is never resolved through the YAML `authors:` table; dates fall back to the epoch.
- Paragraph id lines (`<!-- p0 center -->`, `first-line 0.5in`, `page-break`) are ignored.
- The YAML `headers:`/`footers:` block is ignored.
- Empty paragraphs vanish (Markdown collapses blank lines), and a table cell's paragraphs are
  printed joined by `<br>`, the same mark as a line break, so they come back as one paragraph.

### 1.2 How often an element has an XML key (`tests/corpus`, 1,195 documents)

| Element | Keyed |
|-|-|
| `w:p` body / other stories / text boxes | `w14:paraId` on 97.7% / 98.5% / 98.4% |
| `w:tr` | `w14:paraId` on 97.7% |
| `w:tc`, `w:tbl` | never (the schema gives them no id) |
| `w:ins`/`w:del`/`w:comment`/notes/bookmarks | always (`w:id` is required) |

Per document: of 442 real-world files (`neurotic_docx_bench`) 392 key every body paragraph, 41 key
none, 9 some. The 321 Docxodus fixtures (written by the C# library, not saved by Word) key almost
nothing; the earlier "314 of 400" figure came from a sample dominated by them.

## 2. Our id (short): final grammar

**Decided (Arthur, 2026-10-13): positional, completed (alternative A of three).** The others were
B, a key prefix (`p3F2A`: stable across edits, but no order and hard to read), and C, a hybrid
guard (`p12~3F2A`: re-finds a moved element by its key); C can still be added later without
breaking A. Numbering stays 0-based. A stale positional id is caught by the view's `sha256:` pin
(`STALE_SOURCE`); the long id carries stability.

**Decided (Arthur, 2026-10-13): a shift is reported.** An edit that adds or removes a block prints
a `note:` naming the ids that moved, e.g. `note: inserted p5; p5-p40 are now p6-p41`, so a caller
holding ids from before the edit knows they shifted.

One grammar, one parser (`src/ids.rs`), every option that takes a location, printed and accepted
alike. Positions count the drafter's real structure.

| Kind | Spelling | Notes |
|-|-|-|
| Body paragraph | `p12` | |
| Story paragraph | `header1.p0`, `footnotes.p3` | always with `.pN` in print (no bare `header1` in print; still accepted as input) |
| Table | `t0` | top-level tables of a story, in order |
| Row | `t0.row1` | whole row |
| Cell | `t0.row1.cell2` | first paragraph; `.p1` for the next |
| Nested table | `t0.row1.cell2.t0.row0.cell0` | numbered inside its cell; `read --tables` uses the same numbering |
| Text-box paragraph | `p14.box0.p0` | the box anchored in p14 |
| Change | `r12`, `header1.r1`, pair `r1+2` | |
| Comment | `c5`; the text it covers `c5.text` | |
| Range | `p3-p7` | wherever several blocks are accepted (read, accept, reject) |
| Section | `s0` … `sN` | in body order; the last is the body's own `w:sectPr` |
| Table of contents | `toc0` | a TOC field, or the TOC content control around one, counted once |
| Content control | `cc0` (block), `p12.cc0` (inline) | |

**Decided (Arthur, 2026-10-13): inline objects are content, not ids.** Each has a Markdown sibling
and is edited by editing its paragraph:

| Object | Markdown | Carries |
|-|-|-|
| Hyperlink | `[text](url "tooltip")` | the tooltip as the link title |
| Internal link | `[text](#bookmark)` | |
| Bookmark | `<a id="_Ref123"></a>` | the name is its identity (unique in a document) |
| Picture | `![alt](media/image1.png "title")` | size and wrap as an id-line clause: `<!-- p12 img0 2in x 1.5in, float right, wrap square -->` (`img0` only says which picture the clause describes) |
| Equation | `$…$`, `$$…$$` (body-level too) | |
| Note reference | `[^1]` | |
| Field | `{PAGE}`, `{REF _Ref123 \h}`, then its result as text | switches are part of the code; literal braces in text print as `\{` so they never read back as a field |

What the dialect still cannot say (crops, effects, rotation, exact anchor offsets, OLE payloads)
anchored mode keeps from the source, byte for byte while the Markdown leaves the object alone. Runs
get no id: Word splits and merges them on save, and their text and formatting are the Markdown.

Coverage (`scripts/id_census.py`; `tests/corpus` of the canonical checkout, 1,195 documents: the
689 tracked ones plus the local-only `Docxodus` and `_fixtures` folders, 286,500 objects, each counted
once): every object is either named by a short id (blocks, revisions, comments, notes, headers and
footers, sections, tables of contents, content controls) or is Markdown content (runs, links,
bookmarks, pictures, equations, note references, fields).
Today's view drops pictures, bookmarks and internal links (`[text](#name)` prints as underlined
text) and prints body fields as their result only; those are reader/writer work items.

**Decided (A): bare numbers are deprecated.** Today `12` is a paragraph on `-p`, a change on `--id`,
a comment in plan JSON. From now on a bare number is accepted with a `note:` naming the prefixed
spelling (`p12`, `r12`, `c12`) through 0.13, and refused in 0.14, so every id says its kind. Help,
docs and `capabilities` stop advertising bare numbers.

## 3. The long id: final grammar

**Decided (B): `PART.ELEMENT.ATTRIBUTE.VALUE`.** Examples: `document.w:p.w14:paraId.3F2A1B7C`, `document.w:ins.w:id.12`,
`footnotes.w:footnote.w:id.2`, `comments.w:comment.w:id.5`, `document.w:tr.w14:paraId.1A2B3C4D`,
`document.w:bookmarkStart.w:name._Ref123`.

- Parse: split on the first three dots; the value is the rest.
- PART is the part's stem (`document`, `header1`, `footnotes`).
- ELEMENT and ATTRIBUTE pairs are checked against a table generated from ooxmlsdk's schema
  types; value types come from there too (`w14:paraId` is 8 hex digits, `w:id` a decimal).
- Which attributes are unique keys is a short hand-written list (the schema does not say).
- A cell is its row's key plus `.cellN`; a table is its first row's key.
- Long ids appear in JSON, machine hints and `--long-id`; text output prints short ids (D7).

**Decided (Arthur, 2026-10-13): synthetic keys, the same in the JSON and in every docx jubarte
writes.** A `w:p` or `w:tr` without a `w14:paraId` (about 3% of real-world elements) gets one
computed, not invented:

- Deterministic: a hash of the source's sha256, the part and the element's structural path
  (`document › body › tbl[2] › tr[3]`), so reading a file twice gives the same ids, and the docx
  jubarte writes stamps exactly the ids `read --json` already printed.
- Docx-compatible: 8 uppercase hex digits in 1..0x7FFFFFFF (MS-DOCX refuses 0 and values from
  0x80000000). A value already used in the document is re-hashed with a counter until free.
- Marked: the JSON gives every keyed element `"key": "xml"` or `"key": "synthetic"`, so an id
  never claims an attribute the file does not hold. `--long-id` takes a synthetic id on the
  unstamped source by recomputing it. A synthetic id is stable only while the file is unchanged;
  after jubarte writes the file it is real XML.
- Repeated keys (0.3%): the first holder keeps its value, later ones get a synthetic key, as Word
  does on save. A paragraph of `comments.xml` whose paraId `commentsExtended.xml` (`w15:paraId`)
  or `commentsIds.xml` names is re-keyed on both sides or not at all: a thread must not break.
- Namespaces: stamping a part also declares `xmlns:w14` and `xmlns:mc` and lists `w14` in
  `mc:Ignorable` on its root; a `w14` attribute without them is a Word-invalid shape (a new
  Ring-1 check in `jubarte validate`, with a broken probe).
- Gate before shipping: stamp the unkeyed corpus files (Docxodus fixtures included), then the
  OOXML validator, `jubarte validate` and Word (`word_pdf.py`, no repair prompt) on each.

Coverage on `tests/corpus` (1,195 documents, 146,300 addressable elements): 92.2% named by this
syntax as is (78.2% directly, 14.1% through a row key), 0.3% under a repeated key, 7.5% with no key.
Real-world documents (`neurotic_docx_bench`, 442): 97.75%. The 321 Docxodus fixtures, written by the
C# library, hold 8,566 of the 11,006 unkeyed elements. Revisions, comments, notes, bookmarks and
drawings are 100% (their `w:id` is required); paragraphs, rows, cells and tables depend on
`w14:paraId`, which stamping completes for every file jubarte writes. A file read without keys falls
back to the short id (Word's own positional addressing).

## 4. The Markdown round trip

### 4.1 One dialect

`jubarte read` (the agent view) is the Markdown. `convert -o X.md` writes the same dialect (with a
`--plain` switch for id-free Markdown meant for people, which does not promise a round trip).

### 4.2 The reader reads everything the writer writes

Each line below is a test: write, read back, compare.

- YAML `authors:` table: `@AC` on a note becomes Ann Counsel with the header's date.
- Revision notes `{>>r0 @AC<<}` attribute the change; they never become comments.
- Comment notes `{>>c5 @AC: text<<}` and replies `{>>c6 @AS re c5: text<<}` rebuild the thread.
- Id-line properties: alignment, indents, spacing, keep-next, page-break, section breaks.
- Empty paragraphs: printed as an id line with no text (`<!-- p4 empty -->`) and read back as one.
- Table cells: paragraphs inside a cell printed with a mark distinct from a line break
  (`<br>` stays `w:br`; a new paragraph is `<p>`), read back as paragraphs.
- YAML `headers:`/`footers:`/`page:`/`styles:` blocks rebuild those parts.
- Underline `<u>`, highlight `==`, footnotes `[^n]`.

### 4.3 Two modes

- **Standalone** (`convert X.md -o X.docx`): rebuilds from the Markdown alone. Goal: the Markdown
  is a fixed point (md1 = md3, byte for byte apart from `source:`/`sha256:`) on the whole corpus.
- **Anchored** (`convert X.md -o X.docx --base SOURCE.docx`, or automatically when the YAML
  `source:`/`sha256:` matches a file beside the Markdown): the Markdown's differences from the
  source's own view are applied as edits to the source, so every element the Markdown does not
  express survives byte for byte. This is where ids matter: each id line ties a Markdown block to
  its source element; a block with no id line is new.

**Decided (C): automatic.** When the YAML `source:`/`sha256:` matches a file beside the Markdown,
the conversion is anchored on it and prints `note: anchored on SOURCE.docx`; `--standalone` opts out.

### 4.4 The gate

`scripts/md_roundtrip.py` runs in CI on a fixed corpus sample. Every dimension's keep rate is a
ratchet: it may only go up. The fixed point for the view dialect must reach 100% on documents whose
content the dialect can express, and every document that misses lists the construct it lacks.

## 5. Order of work (PR over PR)

1. Reader: authors table, revision notes, comment threads (the largest losses, and they corrupt).
2. Reader: id-line paragraph properties, empty paragraphs, cell paragraphs.
3. Reader: YAML headers/footers/page/styles.
4. `convert -o X.md` writes the view dialect; `--plain` for people.
5. Anchored mode.
6. Short-id grammar completion (rows, nested tables, text boxes, ranges, one parser for `read -p`).
7. Long id (after decision B), paraId stamping (after a Word check).
