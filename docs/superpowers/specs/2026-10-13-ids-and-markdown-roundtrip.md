<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Ids and the Markdown round trip: the settlement (draft, 2026-10-13)

Arthur, 2026-10-13: "I want to settle the id (ours) and the long id for good. I want to absolutely
ensure that docx to md to docx to md is as lossless as possible."

This document proposes the final rules. Items marked **Decide** wait for Arthur; everything else
follows from evidence below and from rules already agreed (D7, the hierarchy rule: depth and
position come from the real structure, never from marker names).

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

Our own dialect round-trips worse than the poorer one, because the Markdown reader does not read
what the view writes:

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

**Decide (A): bare numbers.** Today `12` is a paragraph on `-p`, a change on `--id`, a comment in plan
JSON. Options: (1) keep them, one kind per option, never printed (today); (2) deprecate them: accepted
with a note through 0.13, removed in 0.14, so every id says its kind; (3) refuse a bare number that
names two things in the document. Recommendation: (2).

## 3. The long id: final grammar

**Decide (B):** `PART.ELEMENT.ATTRIBUTE.VALUE` ("D", recommended) or `story:kind:xmlid` ("B").

D, with examples: `document.w:p.w14:paraId.3F2A1B7C`, `document.w:ins.w:id.12`,
`footnotes.w:footnote.w:id.2`, `comments.w:comment.w:id.5`, `document.w:tr.w14:paraId.1A2B3C4D`,
`document.w:bookmarkStart.w:name._Ref123`.

- Parse: split on the first three dots; the value is the rest.
- PART is the part's stem (`document`, `header1`, `footnotes`).
- ELEMENT and ATTRIBUTE pairs are checked against a table generated from ooxmlsdk's schema
  types; value types come from there too (`w14:paraId` is 8 hex digits, `w:id` a decimal).
- Which attributes are unique keys is a short hand-written list (the schema does not say).
- A cell is its row's key plus `.cellN`; a table is its first row's key.
- jubarte stamps a `w14:paraId` on every `w:p` and `w:tr` that lacks one whenever it writes a
  file, as Word does on save. Before shipping, a Word check that the stamped file opens cleanly.
- A repeated key exits 2 and lists the candidates (D7's rule for change ids).
- Long ids appear in JSON, machine hints and `--long-id`; text output prints short ids (D7).

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

**Decide (C):** anchored automatically when the YAML names a matching source, or only with `--base`.
Recommendation: automatic, with a `note:` saying so.

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
