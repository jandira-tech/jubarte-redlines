<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

See [VERSIONING.md](VERSIONING.md) for the release codemod and cross-repo steps.

## [Unreleased]

### Changed

- The tests no longer need neurotic_docx_bench (or the local
  `_to_improve_accepted_changes` folder) checked out beside this repository.
  The 537 documents they read are copied into `tests/corpus/` under the
  relative paths the tests name. Their provenance, including the
  docx-corpus ODC-By attribution, is in
  `LICENSES/LicenseRef-Bench-Fixtures.txt`. About 180 fixtures had
  moved in the bench, so the tests naming them skipped without a word;
  they run again, and `tests/bench_fixture_paths_resolve.rs` fails on any
  named copy that is missing.

### Added

- `jubarte debug diff A B [C …]` compares two or more packages element by
  element instead of line by line: styles pair by type and name, paragraphs
  by their text (a rewritten one meets the one it replaced), tables by
  their first text, cells by position, notes and comments by text, headers
  and footers by the section role that shows them. Each hunk names the
  element's path (`word/styles.xml › style paragraph "Body Text" › rPr`)
  and prints the property items or run segments not every file holds; with
  three or more files each line names the files holding it. rsids,
  paragraph ids, revision ids/authors/dates, relationship ids (a link or
  image shows what it points to instead), docProps save stamps and counts,
  attribute order, on/off values and empty property blocks are dropped
  unless `--raw`; a part that is not well-formed XML is reported, not
  compared. `--style`, `--para-text` and `-p` narrow the report, `--full`
  prints the shown elements' common lines too without counting them. The
  same in the library (`jubarte::debug::diff::diff`).
- Tracked changes one at a time, as Word's Accept / Reject This Change:
  `jubarte changes FILE` lists every change with an id (`body:rev:12`,
  `header1:rev:3`), its kind, target, author and text (`--json` for JSON
  lines); `jubarte accept` / `reject` take `--id`, `--author` and `--kind`
  (repeatable; a change must match every flag given) and keep every other
  change tracked. Ids stay valid while changes remain, both sides of a move
  resolve together, and a change's `inside` names the change whose
  resolution takes it along. The same in the library
  (`jubarte::changes::{list_changes, accept_changes, reject_changes}`),
  Python (`Document.changes()`, `Document.accept(ids=, authors=, kinds=)`)
  and WASM (`listChanges`, `acceptChanges`, `rejectChanges`).
- Edit plans take `resolve_revisions: {"accept": FILTER, "reject": FILTER}`
  to resolve a selection of the tracked changes before editing; what it
  leaves follows `existing_revisions`. A change selected by both sides is
  refused with `REVISION_CONFLICT`, an id the document lacks with
  `UNKNOWN_CHANGE`; the report lists the resolved ids.

- `jubarte debug FILE -c text` prints each story part's paragraphs with
  `{+inserted+}` / `[-deleted-]` runs, the paragraph mark's revision state
  and table rows; `-c xml` prints part XML one element per line without
  namespace declarations, rsids or paragraph ids. With two files both print
  only the lines that differ, part by part. `-c runs` is `text` with each
  paragraph's direct properties and each run's direct formatting, so two
  builds compare formatting without XML noise.

### Fixed

- `jubarte convert --revisions word` draws a comment balloon only where
  Word's Save as PDF does. Word draws none for a comment whose range ends at
  body or cell level, or before any content in its paragraph, and none for a
  reply to such a comment. With no balloon left, Word keeps the full page,
  with no grey markup pane. 29 of the 222 documents in the docx-to-pdf worst
  set were shrunk beside a pane Word never draws. The other revision styles
  still draw every comment.
- A comment whose range holds only a tab keeps its balloon.
- Comments load when `commentsExtended.xml` is related before
  `comments.xml`. The lookup took the extended part for the comments part,
  and every comment was lost (with_comments_clean/37c6c62345).
- A frame at a page y that runs from the margin or the column
  (`w:framePr vAnchor="page" hAnchor="margin"`) floats at its position, as
  Word paints it, instead of flowing as a plain paragraph. Without an x it
  sits on the margin; without a width or height it takes its text's size, up
  to the column's width. A frame too tall for the page moves up to the
  page's top. The worst document in the docx-to-pdf set (61e3967518 ×
  1424386e9b) went from 25.6 to 42.1 pixel, 8.4 to 20.2 Jaccard.
- A header paragraph's shading (`w:shd`) paints behind its lines, out to
  its border. Only the top and bottom rules were painted, so white text on
  a shaded heading vanished (clean/2261da4dae: 32.9 to 43.3 pixel, 7.2 to
  23.3 Jaccard).
- Rejecting a redline gives back what a changed style inherited in the
  original. Word's Reject All reads a style's old record against its
  built-in defaults, so a record holding only the style's own properties
  rejected to Times New Roman 10 pt, left-aligned and single-spaced
  (b6f757462e's List Paragraph lost Normal's justification and 1.08 lines).
  The records of styles based on another now carry what the original's
  docDefaults and chain gave them, less the built-in values. Paragraph-style
  mismatches against the original after our own reject, 150 pairs: 267 to
  30, and every pPr now matches. A bold or italic that an unrecorded parent
  (or the docDefaults) turns on stays in the record, so it is not rejected
  to off.
- Rejecting a redline in which Normal changed no longer strips the run
  properties of the styles based on Normal: the change record each of them
  gets keeps the style's own fonts, sizes and languages, font slot by font
  slot and language by language, over Normal's old ones, as Word records them (a67dcf9e05's Balloon Text went
  from Tahoma 8 pt to Times New Roman 12 pt). Paragraph-style property
  mismatches against the original after our own reject, over the same 150
  pairs: 298 to 236.
- `jubarte reject` no longer panics ("No parent for AddBeforeSelf") on
  nested block content controls whose every paragraph is inserted, as in
  Word's "Page Numbers" footer parts. A control whose every paragraph goes
  is dropped with them instead of being restored over the paragraph that
  follows. A control whose text alone is deleted, or that holds no run,
  stays (emptied) even when another paragraph mark of the part is deleted,
  as it already did otherwise.
- `jubarte accept` / `reject` (and `accept_revisions` / `reject_revisions`)
  now save what Word's Accept All / Reject All save. Measured against Word on
  51 of Word's own redlines: every paragraph's text and mark state match
  (5 files differed), 46 match run formatting too, and all 51 pass the
  OOXML validator.
  - A comment whose reference was deleted (or, on reject, inserted) goes with
    its anchors and its commentsExtended / commentsIds / commentsExtensible
    entries; with no comment left the comment parts are not written;
    people.xml keeps only the remaining authors.
  - A run of deleted paragraph marks continues through moved-from marks and
    across a table whose rows are all deleted.
  - A bookmark whose span was wholly deleted goes; an empty bookmark inside
    deleted text now survives, where the deletion was. No half bookmark is
    left behind.
  - Bookmarks and comments are renumbered from one counter in document
    order, as Word numbers them.
  - A paragraph style naming a character style is dropped, and no empty
    `w:rPr` / `w:pPr` is written.
- `jubarte reject` matches Word's Reject All on the text and mark state of
  all 100 of Word's redlines in the bench's `rejected_tracking` set (12
  differed). All outputs but one pass the OOXML validator; that one's input
  and Word's own Reject All fail it too.
  - A move range that ends inside a table no longer takes the table's
    `tblPr` and `tblGrid` with it (Word refused the file), and an emptied
    moved heading before a table goes instead of keeping a live `moveFrom`
    mark.
  - A footnote or endnote whose reference was resolved away goes.
  - Adjacent tables alike in every whole-table property (style, float,
    direction) become one table, as Word holds them; a later table's own
    borders, widths and margins ride on its rows as `tblPrEx`.
  - Rejecting a section change keeps the section's header and footer
    references. When a section break goes, the next section takes its
    headers and footers if it had none of its own. Header and footer parts
    no section shows are not written.
  - Rejecting a paragraph style's recorded change restores its old
    properties the way Word reads them: against Word's built-in defaults
    (Times New Roman, 10pt, single spacing, left-aligned), not against the
    style's ancestors. A property the old record lacks takes the built-in
    value, and a value the style would inherit anyway is dropped. The
    style's linked character style takes the restored run properties.
  - A numbered style's restored paragraph properties drop what its numbering
    level already says (the level's indents, tabs and spacing), as Word does.
    The level is the one Word numbers with: through a numbering style link,
    at the nearest defined ilvl, a full level override without paragraph
    properties saying none, and none at all under an explicit `numId=0`.
  - A linked character style takes its paragraph style's whole effective run
    properties, less what the docDefaults say, and a linked character style
    based on it follows. An old record's bold, italic or other run toggle
    reads against the nearest ancestor the same reject restores, as Word
    reads it. The built-in off hyphenation suppression, auto text alignment
    and auto colour are written where the style chain says otherwise.
- A redline re-caches every themed colour its styles carry against the theme
  it ships, as Word does: table-style shading fills and colours and border
  colours too, not only text colours. Tints and shades are truncated per
  channel the way Word writes them (accent 156082, tint 3F: B2DEF2).
- A redline whose original has no theme (or only one nothing references)
  ships Word's own default Office theme, byte for byte, instead of the
  revision's theme; Word never takes the revision's (664 of 664 bench
  redlines, and Word probes with a custom revision theme).
- A redline whose original has a theme but no styles part keeps the
  original's theme. It was replaced with Word's default theme along with the
  missing stylesheet.
- `jubarte debug A B -c text` / `-c runs` pairs header and footer parts by
  the section reference that shows them, so Word's renumbered parts compare
  with their counterparts.

## [0.10.0] - 2026-09-28

> **Summary.** Documents are now editable by agents: inspect, JSON edit plans that write a clean copy plus a tracked redline, and headers, footers and notes as editable stories, from the CLI, Python and WASM alike; Word-mode redlines keep changed text boxes, per-section headers and footers, and comment anchors where Word does.
>
> **Docs.** README links the page-by-page engine comparison site (top banner, Benchmarks, Find us) and documents self-update and --mode word|powertools; new docs/WORD_DIFFERENCES.md, docs/SELF_UPDATE.md, the jubarte-documents agent skill and the Acme letter example; the jubarte-wasm npm README covers the agent API; VERSIONING.md and bump-version.mjs make release.sh the source of truth.

### Added

- `jubarte A B --mode word|powertools` names the compare presets: `word`
  (the default, Word Compare's layout) or `powertools` (the classic
  PowerTools fallback, still also spelled `--powertools-faithful`).
  `docs/WORD_DIFFERENCES.md` lists where jubarte's redline still differs
  from Word's and which mode gives which.
- `jubarte-wasm` gains the agent surface the CLI and Python already have:
  `inspectDocument`, `documentMarkdown`, `sourceSha256`, `applyEditPlan`
  and `previewEditPlan` (refusals returned as data with their code and
  per-operation outcomes), `editReportJsonl` and `capabilities`, in the
  full and slim builds.
- Input admission for the agent surfaces: `inspect`, `text`, `edit` and
  the Python/WASM facades over them refuse a package before parsing it when
  it breaks a resource budget (64 MiB file, 10,000 entries, 64 MiB per
  inflated part, 256 MiB inflated in total, XML nesting 256), repeats a
  part name, uses an unsafe entry path, encryption or compression other
  than deflate, or is not a WordprocessingML package. Refusals carry the
  codes `INPUT_LIMIT`, `DUPLICATE_PART`, `UNSUPPORTED_PACKAGE`,
  `INVALID_PACKAGE` and `INVALID_XML`; `capabilities` reports the budgets.
  Compare keeps its historical tolerance.
- `jubarte inspect` reads a document as numbered paragraphs (JSON with a
  source hash, or Markdown) with styles, numbering, formatting spans and
  the structures an edit cannot address (fields, hyperlinks, content
  controls, revisions); `jubarte capabilities` reports what the build can
  do.
- `jubarte edit` applies a JSON plan of uniquely anchored operations
  (`replace`, `insert`, `delete`, `comment`, `insert_paragraph`,
  `delete_paragraph`, `format_paragraph`, `merge_paragraphs`)
  to a copy, previews it, and writes the clean copy, the tracked redline
  and a per-operation report. A plan that carries the source's SHA-256
  (`source_sha256`) is refused for any other source; without it the plan
  runs unguarded and the report flags that. An
  anchor that is not unique, or text inside a field (simple or complex,
  even one with an empty result), hyperlink, content control or revision,
  is refused. Comment text must be nonempty without control characters,
  and several `insert_paragraph` operations after one anchor keep plan
  order.
- `jubarte convert --png` renders pages to PNG (1–1200 dpi, at most 2^28
  pixels a page) and checks every output path before the first write.
- Python: `jubarte_redlines.read()` returns a `Document` with inspect, edit,
  convert, compare and accept/reject methods, and `python -m
  jubarte_redlines` exposes the same commands (compare is the explicit
  `compare` subcommand there); `EditPlanError.message`
  carries the engine's detail.
- Edit plans restyle and join paragraphs. `format_paragraph` sets a
  paragraph style (by id or name; an unknown one is refused with
  `UNKNOWN_STYLE` and the defined ids), alignment, line spacing as a
  multiple and space before/after in points, as one tracked property change
  that reject undoes. `merge_paragraphs` joins the next paragraph onto one,
  with an optional separator; the joined paragraph keeps the second one's
  properties, as Word's accept of a deleted mark does, and bookmarks and
  comment ranges between the two move to the join. `replace` and `insert`
  take a `format` (bold, italic, underline, a Word highlight colour) for the
  new text only. The Python `EditPlan` gains the matching builders.
- `replace` takes `"whole": true` (Python `whole=True`): the redline shows
  all of `find` deleted, then all of the replacement inserted, as typing
  over a selection with Track Changes on does, instead of Word Compare's
  word-level diff that keeps shared words. Deletions the comparer placed
  just outside the change are gathered in, formatting stays on each side,
  and a comment on the change stays on the inserted text. When the diff
  cannot be regrouped the operation keeps the word-level redline and its
  report line carries a `message` saying why.
- Headers, footers, footnotes and endnotes are editable stories. `inspect`
  lists them under `stories` and `jubarte text` prints them after the body,
  each paragraph under an id such as `header1:p:0` or `footnotes:p:2`. An
  edit plan addresses them by that id or by adding `"story": "footer1"` to
  an `index`, `starts_with` or `contains` selector (those search the body
  otherwise). The change is tracked in the story's own part, `"whole": true`
  included, and the report counts header and footer revisions. Comments
  stay body-only, since Word cannot anchor one in a header or footer. The
  Python `Snapshot` gains `stories`.
- Adoption guides, workflow examples and a document-operations agent skill.
  The Acme letter example now ships `make_letter.py`, which writes its
  source letter byte for byte, and its plan runs all twelve edits.
- `jubarte self-update [--check] [--yes] [--version X]` installs a GitHub
  release after checking its SHA-256 against the release's
  `SHA256SUMS.txt`. It contacts GitHub only when run; nothing checks for
  updates in the background. Build with `--no-default-features --features
  cli` for a binary without it (docs/SELF_UPDATE.md).

### Changed

- quick-xml 0.41 → 0.42 for input admission and XML checks. Admission now
  transcodes UTF-16 parts (SharePoint `customXml` items, which Word opens)
  before scanning them, so the new release's UTF-8 validation refuses none
  of the 500 fixture documents; bytes that are not UTF-8 still decode
  lossily, as before.
- Dependencies: clap 4.6.7, flate2 1.1.10 and serde_json 1.0.151 across the
  engine, Python, WASM, in-process and app workspaces; quick-xml 0.42 in
  the app and in the WASM build's patched `rdocx-opc`; the ooxmlsdk test
  oracle 0.12.
- `scripts/release.sh` requires a sixth note,
  `--how-readme-and-other-docs-were-updated`, written under the changelog
  summary and into the release commit, next to a list of the docs changed
  since the previous tag. `scripts/bump-version.mjs` warns that
  `release.sh` is the source of truth for releases.

### Fixed

- `jubarte-wasm`: the patched `rdocx-opc` decodes escaped attribute values
  when it reads relationships and content types, so a hyperlink target
  holding `&amp;` is written back once instead of as `&amp;amp;`.
- A text box whose text changed no longer disappears from a Word-mode
  redline. Three late passes that rebuild a revised paragraph from the text
  of its insertion and deletion also read the text inside the box, so the
  drawing and its VML fallback were dropped and both copies of the box's
  text landed in the anchoring paragraph ("…of the postOverall…"); accepting
  the redline did not give the revised document. They now leave any
  paragraph alone whose runs hold anything besides text. Present since at
  least 0.7.1 (fixtures_500 00b81efae883).
- Word mode marks a text box's changed words inside the one box, as Word
  does, in the DrawingML shape and its VML fallback alike, instead of
  deleting the old box and inserting the new one. A shape wrapped in
  `mc:AlternateContent` is now a word of its own in the Word-mode diff, as
  a bare drawing already was, so a changed box no longer takes the
  unchanged shape or text beside it into its replacement (a deleted and
  reinserted VML group repeated its shape id; fixtures_500 003329b501a7).
  Changing one word in every text box of the fixture documents now keeps
  all their boxes (22 of 22 documents).
- A document whose sections have their own headers or footers no longer
  redlines them against the wrong section's: they were paired by kind and
  type alone, so every section's default footer met the last section's,
  and an unchanged "Page 1 of 4" footer came out deleted and reinserted as
  "Page 4 of 4". Parts pair by section now, and an unchanged part is left
  alone.
- Word mode no longer invents paragraph property changes. The last
  paragraph of a story with a few words revised kept its spacing live only
  when the paragraph was replaced whole; otherwise its space before moved
  into a `w:pPrChange`, and an unchanged justified paragraph got an empty
  one. Word records neither (Word's own redlines of a body, a header and a
  text box), and the parity ladder's heading-4 pair drops the two
  `w:pPrChange`s Word does not write.

- Changes in a header or footer that carries a relationship (a logo, a
  hyperlink) are now in the redline. The comparer skipped every such part
  and kept the original's, so the redline silently lost the change and
  accepting it did not give the revised document. It now diffs the part
  whenever each relationship the revised part uses means the same thing in
  the original's (same type and target, same bytes for an image), which is
  the case for an edit plan and for most comparisons of two versions.
- A paragraph selector given as a bare id string (`"paragraph": "body:p:88"`,
  as the agent skill's own example writes it) failed to parse with "data did
  not match any variant"; only `{"id": ...}` worked. Both forms are
  accepted now.
- Two paragraphs joined into one now read as Word's Compare shows a join:
  the first paragraph's mark is deleted and only the separator is inserted.
  The paragraph LCS paired the joined paragraph with the first original,
  which stranded the second one's words behind that mark, so Word mode
  showed them as a move ("Delivery is DDP to the Buyer's site.", six words
  and more) or deleted and inserted them again. The cross-paragraph stream
  now also tries the region without that pairing and keeps whichever keeps
  more text, and it no longer fuses the last word of one paragraph and the
  first of the next ("TRIAL." + "Each") into one compound.
- `insert_paragraph` runs start from the anchor's body run, the one with
  the most text, instead of its first run, so a bold lead-in such as
  "(f) Notice of Inability to Comply." no longer makes the whole new
  paragraph bold.
- Comments carried into a redline land on their own occurrence of repeated
  text, not the first one: two comments on the same words in different
  places no longer collapse into one (Word keeps all six in the M35
  renumbered pair; jubarte kept four). A long commented range whose inside
  changed now maps by the text at its two ends instead of being dropped.
- Word mode keeps the built-in ids of styles whose id differs from their
  name (`CommentText` for "annotation text", `CommentReference`,
  `CommentSubject`, `MacroText`, `TOAHeading`, `TableofFigures`,
  `TableofAuthorities`). Renaming them left `comments.xml` pointing at
  undefined styles, which stripped the comment formatting.
- An unchanged last paragraph after a replaced block came out inserted
  and deleted again when the documents' first paragraphs matched: the
  positional paragraph zip no longer pairs a paragraph whose identical
  copy sits elsewhere in the other document (Word keeps it unchanged).

## [0.9.3] - 2026-09-27

> **Summary.** Redlines are now scored against Word's own redline of each pair, rendered by Word, and this release fixes what that exposed: redlines Word refused to open, blank field codes, missing fonts, and misplaced equations.

### Fixed

- A field whose code stayed while its result changed now stays one field, as in Word's redline. Only the result's words are marked. Before, the field's begin, separate and end were glued to the result's first and last words, so "Contaminated Sites Act 2003" against "Firearms Act 1973" (a STYLEREF title) matched only " Act ". That left a deleted and an inserted field whose ends crossed, and Word refused the redline (db433183 × 9377099d).

- A field whose code changed is now replaced whole, nested fields included, as Word's redline does. Every part of a field carries the codes of the fields around it, so an `IF` that lost `\*MERGEFORMAT` no longer keeps its begin and inner DOCPROPERTY fields while the rest is half deleted (98bf5f3d × a3701d36).

- A changed table of contents no longer opens inside its replacement. Word mode's head junction folds a short deleted title into the first inserted paragraph when both share a word. When that title was a TOC's first entry, the fold carried the deleted TOC's begin above the inserted TOC's end, and the two fields crossed (98bf5f3d × a3701d36, "Part 1—Preliminary" against "Part 1—Introduction"). A fold that would carry half a field across other fields no longer happens.

- The same input writes the same bytes. Another dependency turns on the `zip` crate's `time` feature, and with it every package entry carried the time of the write, so two runs a second apart gave different files. Entries are now dated 1980-01-01, as Office dates its own.

- Two point comments inside one run now keep their places. An anchor that ended one text piece of a multi-piece run went after the whole run, so "Note" landed after "東京" and behind "Second note". The run now splits between the pieces.

- Rejecting every change in center_alignment × center_aligned_bold now restores the original. A pass forced the inserted "This" and "text" into kept text while the next paragraph still deleted them, so the rejected document read "This text This document …". The cross-paragraph stream already keeps those words where Word does, so the pass is gone. The revised tail now fuses into the sole deleted paragraph and keeps that paragraph's deleted mark, so accepting no longer leaves an empty last paragraph.

- Replaced regions now follow Word's replace-gap grammar, which Docxodus 12 decodes.
  - **Empty paragraphs.** An empty paragraph found in both documents no longer anchors two unrelated regions. It pairs only as part of Word's pilcrow chain, or as the two stories' final marks. The chain breaks when an original paragraph with words faces an empty one.
  - **Interior replaces.** Inside the body, a replaced region keeps every old and new paragraph whole: new paragraphs first, each under an inserted mark, then the old ones under deleted marks. Before, "TWO" fused into "e" and "A" into "a" (list_with_table_break × broken_complex_list, docxide 13.2 vs Docxodus 93.3).
  - **The story's tail.** When the revised document's last paragraph holds text, that text fuses into the first deleted paragraph of the tail, even across a deleted table. That paragraph keeps its own properties and deleted mark, so accepting every change no longer leaves a stray empty paragraph (support_tickets_table × support_tickets_summary, diff_doc2 × numwords). The same holds when the original runs on past it to its own end (bullet_list × calibri_bold_italic: "Calibri bold italic …" opens "Apples", then "Bananas" to "Grapes" deleted). A new title facing an unrelated old one is inserted whole before the old one's deletion (pirates × table_left_indent, as Word does).
  - **Final empty paragraphs.** When both documents end on an empty paragraph, those two final marks stay paired behind a trailing deletion.
  - **Result.** Paragraph-structure agreement with Word's own redlines over 747 pool pairs rose from 0.9276 to 0.9331: 50 pairs improved and 9 dropped, none by more than 0.08.

- Redlines keep the bookmarks of both documents, as Word's Compare does. The WmlComparer port dropped every bookmark, so each updated TOC line, `PAGEREF` and `REF` printed "Error! Bookmark not defined." (file_21 × file_22 lost all 582). A bookmark in both documents appears once, at its place in the revised text. A bookmark only in the original stays beside its deleted text. Ids never collide with revision ids, and Word's hidden `_GoBack` is dropped, as Word does. Pool pairs with more broken references than Word's own redline went from 29 to 0. Comment anchors gain two fixes from the same pass: moved text now counts once on each side, and a second anchor inside a run that holds several text pieces no longer reorders that text.

- The redline's Normal style now follows Word's rules for merging the two
  documents' defaults. Word writes B's docDefaults indents, justification,
  line-unit spacing and borders into Normal (and neutralizes the ones only A
  sets), writes B's run defaults whenever either Normal stores paragraph or
  run properties, writes only the language attributes that change, writes the
  implicit 10pt complex-script size, reads a document with no docDefaults at
  Word's factory values (after 160, line 278, kern 2), and finds a
  LibreOffice Normal (`style0`) by its name. Across 738 corpus pairs, Normal
  run properties that differ from Word's drop from 79 to 38 and paragraph
  properties from 54 to 46.

- Point comments (a comment reference with no range markers) survive the
  redline. The carryover only mapped ranges, so a point comment was dropped,
  and with it the whole comments part when it was the only one. It is now
  written as an empty range right after the text it follows, the form
  Word's own redline uses (comments.docx comment 2, in comments ×
  complex_style_attr and clear_formatting × comments). An empty range is
  written as one group: a lone start used to land in the next paragraph.

- A list copied from the revised document keeps its picture bullets: the
  bullet definitions and their images now travel with the list. The list's
  levels had named a picture bullet the merged numbering never defined, and
  Word refused to open the redline (italic_rstyle_combos ×
  paragraph_indent_normal_styles: harness 0 → 53.2; Docxodus 0). When both
  documents define the same bullet id, the revised bullet takes a fresh id
  and keeps its own image. A part's first new relationship is now `rId1`, as
  Word numbers them, not `rId0`.

- The final paragraph marks are also paired when the replaced original has a
  single paragraph: its text is deleted into the revised empty final
  paragraph. The deleted text had joined the revised document's last content
  paragraph and the revised final paragraph was dropped, so accepting the
  redline lost that paragraph (fields_attrs1 × cli_legacy sample: harness
  23.4, Docxodus 73.1).
- A paragraph property the revised document adds to a kept paragraph is
  recorded in `w:pPrChange` whatever it is, as Word records it; only added
  alignment or spacing were recorded, so rejecting the redline kept, for
  example, an added outline level.
- When an unrelated document replaces the original and ends on an empty
  paragraph, the two final paragraph marks are paired as Word pairs them:
  the original's last paragraph is deleted into the revised final paragraph,
  which keeps the revised properties. The redline had given that paragraph
  the original's bare properties, and Word's taller final line, which moves
  the redline onto a second page, was missing (diff_after8 ×
  doc_with_spacing: harness 2.8, Docxodus 93.9). The revised document's last
  content paragraph keeps its own inserted mark instead of folding into the
  first deleted paragraph; a head junction on a shared word still folds.
- A run of changed body paragraphs is compared as one stream of words and
  paragraph marks, as Word does (port of Docxodus's DocxDiff in-gap pairing
  and cross-paragraph segmenter, `comparer::cross_para`): a kept word may now
  sit across a paragraph mark, and rejecting the redline restores the
  original instead of repeating a re-kept phrase. Word-faithful gates keep a
  gap unpaired where Word does: a region with no paragraph pair streams only
  when the revised side has no more paragraphs and its first kept word opens
  a paragraph on both sides; a lopsided same-slot pair needs three shared
  content words; a window of function words alone carries nothing across a
  mark; a table between two changed runs keeps the LCS pairing; and the next
  paragraph takes a same-slot pair when it shares at least twice the content
  words in order (bold_rstyle × bold_vals).
- An inserted tail ahead of a deleted tail pairs the two final paragraph
  marks, as Word does: the last inserted paragraph joins the first deleted
  one with a deleted mark, so accepting the redline no longer leaves an
  empty paragraph the revised document never had (bullet_list_bold ×
  bullet_list).
- Word-mode table margins follow Word: `tblInd`/`tblCellMar` of 10 twips is
  stamped on a bordered table only when the document the table comes from
  (the original for a wholly deleted table) has no default table style. 94
  bordered tables from `TableNormal` documents stay bare in Word's redlines;
  stamping them shifted every row below (file_46 × file_47).
- Synthesized numbering for a dangling `numId` copies Word's level geometry:
  a `num` tab at the text indent and a full 720-twip hanging indent, so level
  0 puts its number at the margin.
- Redlines no longer cross or pack complex fields, which crashed Word
  ("Connection is invalid", English pair 57f96361×3832d290) once field codes
  were kept: a field result only matches text inside a field with the same
  code, the insert-before-delete swap leaves `fldChar` wrappers in place,
  and `SetAfterUnids` aligns the two ancestor chains at the paragraph instead
  of the top (an SDT-wrapped paragraph against a bare one gave every revised
  run one Unid, packing a footer's text, field begins, codes and tabs into
  one run).
- Redlines Word refused to open now open (14 of 19 English failures):
  relationship and content-type attributes no longer gain an `&amp;` per
  round trip, a case-duplicate `Default` extension is merged, and
  relationship targets are relative to the source part's folder (904e989).
- A deleted text box is deleted whole, as Word's Compare deletes it: every
  run of its story sits in a `w:del` of its own and its paragraph marks are
  deleted. A text box is a story of its own, so the `w:del` around its anchor
  does not reach into it, and a field there kept its field code in a run no
  deletion wrapped (`w:delInstrText` outside any `w:del`). Word refused both
  English redlines that had one (30ff840c, bb113e88); the OpenXmlValidator
  passed them. Both open now and score 0.673 and 0.557 jaccard against Word's
  own redline, where every other tool scores 0.
- A carried bookmark stays out of a content control its source bookmark was
  not in. Placement follows the text, so B's body-level `_Toc` bookmarks
  around a data-bound title control landed inside it (7b649361), and a
  bookmark landed in a dropdown cell (57c181da). Word refused both redlines
  and opened each once those bookmarks were dropped. An endpoint now leaves
  the control: a start before it, an end after it.
- Scratch `pt:Unid` attributes no longer leak into restored deleted-paragraph
  spacing, and every extension namespace (w14/w15/w16*/wp14) is listed in
  `mc:Ignorable` on each part root (b7fedc7); w16 serializes under Word's 2018
  wordml namespace (722de2a).
- Changed field codes keep their instruction text. An inserted or deleted
  `w:instrText` was re-emitted empty, so tracked PAGE, REF and TOC fields
  rendered blank in Word (8ab1df8).
- Footnote ids renumbered by the comparer stay clear of the revised
  document's `continuationNotice` id (fee9411).
- The revised document's fonts join the output font table. A font used only
  by B had no `w:font` entry, so Word substituted Times New Roman (2eb24d5).
- A base made only of display equations counts as content, so an unrelated
  pair keeps Word's order (inserted text first, deleted equations after)
  instead of merging the first equation into the first inserted paragraph
  (e4610b0).
- `get_revisions` and `compare` return an error on bad input instead of
  panicking (cb33d11).
- sha1 0.11 drops the vulnerable block-buffer 0.10.4 (GHSA-qwgh-2vcv-g2f7)
  from the engine, Python and WASM locks (9cddeb3).
- convert: a cell holding only a nested table splits without panicking
  (4dda25e); rotated oval pictures, comments on vertical pages and scaled
  vertical runs follow Word (832eac9, c8a878e, b41cb35); pie and gear text
  rectangles follow Word (2e4cd8f); altChunk decodes base64 MHT parts and
  tolerates omitted end tags (93643b5, 30d595f).
- Redline and other rewritten packages are byte-reproducible: zip entries are
  written in the source package's order, added parts after them by name,
  instead of in hash-map order that changed on every run.
- Changed paragraphs that all correspond position by position (each shares a
  content word with its counterpart, docxodus's same-slot rule decoded from
  Word) are paired in place, as Word pairs them, instead of one body pairing
  with the next on a shared trailing word and the rest falling out
  whole-inserted and whole-deleted. file_111×file_112 scores 0.108 → 0.538
  Jaccard against Word's redline; +0.35 summed over the 1,195 pool and
  English pairs (8 changed).
- When the revised document names its default fonts by theme
  (`w:asciiTheme="minorHAnsi"`, as Word writes them), the redline's Normal
  style now carries those theme fonts instead of keeping the original's named
  face, so the text renders in Calibri as Word's redline does rather than in
  Times New Roman. +2.96 Jaccard summed over the 1,195 pool and English pairs
  (126 changed, 26 better, 2 worse); instrtext_angled_brackets_bug ×
  table_merged_cells 0.13 → 0.82.
- A paragraph whose layout changes on both sides (double spacing replacing
  heading spacing) now keeps the new layout and records the old one as a
  paragraph-property change, as Word does. Equal properties had looked
  different because of the comparer's internal `pt14` bookkeeping attributes;
  those no longer count. A revised paragraph with layout of its own no longer
  inherits the original's small `after` spacing. +0.59 Jaccard summed over the
  1,195 pool and English pairs (197 changed, 0 worse by more than 0.005);
  document_100 × double_spacing_bold 0.48 → 0.85, file_111 × file_112
  0.54 → 0.75. Paragraph-property changes that disagree with Word's count
  fall from 1,784 to 1,420.
- Recorded old run and paragraph properties no longer carry a stray
  `xmlns:ns0="http://powertools.codeplex.com/2011"` declaration left over from
  the comparer's internal bookkeeping (349 of the 1,195 pool and English
  redlines had one in the body; now none). Rendering is unchanged.
- When two unrelated documents both end in an empty paragraph, the wholesale
  replacement now keeps that story-final paragraph live after the deletions,
  as Word does, instead of welding the revised document's trailing empty
  paragraph onto the original's first deleted paragraph (which then kept a
  live mark and the revised styling). +0.86 Jaccard summed over the 1,195
  pool and English pairs (96 changed, 13 better, 4 worse);
  line_break × line_space_table 0.29 → 0.51.
- A redline no longer invents two empty inserted paragraphs before a deleted
  title when the table after that title is deleted wholesale. Word adds those
  spacers only when it pairs the table cell by cell. quarterly report table ×
  red bold heading demo 0.24 → 0.26; no other pair of the 1,195 changes.
- Unrelated documents too short for the wholesale shortcut (for example a
  title and a table against three headings) now get Word's junction when they
  share no word of four letters or more. The revised document's last
  paragraph joins the original's first paragraph, whose mark is deleted.
  Previously full LCS paired a stray digit and kept the two apart. +1.24
  Jaccard summed over the 1,195 pool and English pairs (33 changed, 4 better,
  1 worse); sd_1494 table left indent × sdpr title-only 0.22 → 0.84,
  quarterly report table × red bold heading 0.26 → 0.68. The one loss
  (−0.025) is an NDIS footer whose junction now matches Word's redline
  paragraph for paragraph.
- A single shared word of four or more letters or digits that opens or
  closes a paragraph on both sides now stays an anchor in a long unrelated
  window, as in Word's redline ("Second" opening both "Second green
  underlined item" and "Second page"). The detail threshold had voided it at
  one word in 54. green underline bullet list × header without relationships
  +0.09; no other pair of the 1,195 changes. A general lone-word anchor was
  tried and rejected (−3.07 summed, 30 worse).
- A wholesale replacement between unrelated documents now anchors on a word
  that ends a paragraph on both sides and applies Word's seam on each side of
  it: "2026" closes both "Product Roadmap 2026" and "Date: February 1, 2026",
  so that paragraph keeps "2026" and its mark and replaces the words before
  it. +0.31 Jaccard summed over the 1,195 pool and English pairs (2 changed,
  both better); product roadmap × project plan +0.28.
- When the revised document ends inside a stretch the original continues past
  (a Greek alphabet list against "Meeting Agenda" and a table), the two final
  paragraph marks now pair as Word pairs them: the revised last paragraph joins
  the original's first deleted paragraph under the original's properties with
  a deleted mark, and the paragraph after the deleted table takes the revised
  properties. The table and everything after it no longer sit a line high.
  +0.91 Jaccard summed over the 1,195 pool and English pairs (2 changed, both
  better).
- Short unrelated documents that both end on an empty paragraph no longer weld
  the original's first paragraph onto the revised document's last one. Word
  inserts the revised document whole, deletes the original, and pairs the two
  final empties (a titled table against an item list). +0.93 Jaccard summed
  over the 1,195 pool and English pairs (7 changed: 5 better, 2 slightly
  worse).
- Stamped demo bodies zip positionally only when a body pair shares a word of
  five letters or more. Sentences that share just "This" and a full stop
  (Calibri heading × underline) now take Word's shape: the revised first body
  inserted whole, the revised last body joined to the original's first.
  +0.14 Jaccard on the one pair that changed.
- Custom style ids derived from style names keep only letters and digits, as
  Word's do. "Normal (Web)" became `Normal(Web)`, which orphaned the revised
  document's live `NormalWeb` paragraphs onto a custom style. +0.82 Jaccard
  summed over the pool and English pairs (8 better, none worse).
- A list whose `numId` both documents use but define differently (a circle
  bullet against a disc) now moves the unchanged items to the revised
  definition and records the original `numId` in a `w:pPrChange`, as Word's
  own redline does. They used to keep the original's bullet with no change
  recorded. The PDF converter does not yet paint Word's struck-old /
  inserted-new marker pair, so its own proxy scores the pair mixed
  (circle × disc +0.36, disc × square −0.47; +0.05 summed).
- Redefined paragraph styles now carry the revised document's effective
  fonts, sizes and spacing as a delta against the output's own chain and
  docDefaults, which is Word's rule (mined over 4,924 tracked styles in the 747
  pool redlines; our style values now match Word's on 42,542 of 42,993, up
  from 33,681). A revised heading that inherited Arial and line 276 from its
  own docDefaults used to render in the original's theme font and line pitch.
  +3.73 Jaccard summed over the pool and English pairs (45 better, 15 worse,
  every loss under 0.04).
- A revised document without any `w:sectPr` now gets Word's default section
  (Letter, one-inch margins, one column) as the live body section, with the
  original's section recorded in a `w:sectPrChange`. Word's redline does the
  same in all six such pool pairs. The body used to keep the original's
  section with no change record, so a two-column original stayed two columns
  and 0.5-inch margins stayed narrow (invalid_list_def × tiff +0.08; sd_1480
  × missing_sectpr now paginates like Word, but the converter's proxy scores
  it −0.07 because it does not yet paint Word's formatting change bar on the
  second page).
- Direct `line=276` spacing is dropped as a restated default only when the
  paragraph's own source document (the original for deleted paragraphs, the
  revised one otherwise) resolves line 276 for an unstyled paragraph. A 276
  over a single-spaced Normal is a real value and Word keeps it; the strip
  used to remove it regardless (sd_2517_localized_heading_styles: 30 inserted
  paragraphs in each of four pool pairs lost their line pitch). Our paragraph
  line spacing now matches Word's on 8,895 of 8,944 inserted pool paragraphs,
  up from 8,779. +1.03 Jaccard summed (16 better, 5 worse, every loss under
  0.03).
- `convert --revisions word` now paints the change bar beside paragraphs,
  including paragraphs in table cells, whose only revision is a formatting
  change (`w:pPrChange` or `w:rPrChange`), as Word does. Such paragraphs used
  to get no bar. Converting the same redlines, 177 pool PDFs changed:
  +1.41 Jaccard summed, 27 better, 1 worse (-0.026). The worse pair carries a
  `firstLine="0"` paragraph change that Word's compare does not record.
- A trailing empty paragraph that ends both documents with the same spacing
  keeps that spacing live, unrevised, as Word does. It used to be moved into
  a `w:pPrChange` over an empty paragraph, which also drew a change bar Word
  does not show (super_editor complex2×complexexport1, +0.009).
- A deleted Title or Heading that opens a run of deleted paragraphs is no
  longer mistaken for a checklist cell ("Table Widths" has only two short
  words). Word mixes it with the last inserted paragraph, and the mixed
  paragraph keeps the deleted heading's properties and deleted mark. It used
  to stay a separate paragraph and push every line below it down
  (file_134×file_135 0.12 → 0.41). +0.37 Jaccard summed over 14 changed
  redlines, 3 better, 2 worse. The larger loss comes from an inserted
  text-box paragraph that we already mixed wrongly, where Word keeps it
  inserted.
- The installed-font index (`font-index.tsv`) is rebuilt after an upgrade.
  It stores the answers of the family search, and a new release that changes
  the matching rules used to keep the old release's faces, or its misses,
  for as long as the font folders stayed unchanged.
- An empty page break after a TextHeading now keeps the blank page Word
  keeps. The break paragraph is about a line plus its space after (sd_2517's
  breaks are line 276 and after 200, roughly 23pt on the 12pt face). When
  less than that remains, Word moves the paragraph to the next page and the
  break still fires. Those breaks had been kept on the current page, so
  sd_2517 and its randomized copy came out 99 pages against Word's 107. A
  full page of overflow still skips for every manual break. The wider
  leftover applies only to TextHeading, so a short gap no longer invents a
  blank page on the three-page and thirteen-page fixtures.
- A bottom border on a paragraph inside a table cell is drawn once, on the
  last paragraph of a run that shares that border, including when the
  paragraph still has text. Deleted cell bottoms in file_146 — `bun run dev`,
  the npm and github lines, and the code cell's closing brace — were missing,
  so the pale rules wider than 200pt went from 30 to Word's 33. An empty
  paragraph in the middle of that run no longer draws a rule of its own.
- Tab Alignment against Tab Tests is mixed paragraph by paragraph, in order.
  Word mixes the title and the three following tab lines with Tab Tests'
  four paragraphs and deletes the rest. Flat word matching mixed only the
  title and one empty line.
- A folded Demo title keeps "Demo" inside the deletion. It used to be peeled
  into a live " Demo", which is in neither document, so accepting the redline
  left the word behind (double spacing × eigenpal, and document 100 × the
  comments addition).
- A sentence period shared by both documents stays on both sides when one
  side continues into the next paragraph. The period goes to the side that
  already finished its sentence, and the continuation gains its own period
  at the end (font color × font family, italic underline × justified
  underline, justified underline × justify alignment).
- A deleted section that repeats the heading inserted in front of a table is
  placed back before that heading. The comments redline had moved the
  section to after the capability table, so the original's characters were
  all present and in the wrong order.
- An empty paragraph-property shell is left out. The mixed title on blue
  centered title × blue italic carried an empty `pPrChange`, and center
  bold × clear formatting carried three empty `w:pPr` elements. Word's
  redlines of those pairs have neither. A pilcrow mark that is the only
  thing in the property stays.
- The 207-pair parity sweep reports 0 NEW keys, and
  `tools/parity_baseline.tsv` is unchanged. Spellcheck marks, the
  pagination cache, and header or footer references Word writes on its own
  stay outside the histogram.
- A merged table keeps `w:tblPrChange` last in `w:tblPr`. The default table
  look is written before the change, and the old properties stored inside
  the change are left without a synthesized look.
- A bare `w:cantSplit` or `w:rtl` stays. On those elements a missing value
  means on, and only an explicit false is dropped.
- A table that already sets a nonzero top or bottom cell margin keeps it.
  Row exceptions of zero are written only when the table did not set one.
- PowerTools-faithful compares no longer receive Word's table look, row
  margin exceptions, or the other Word-visual cleanup. That preset's
  contract is that those passes stay off.
- A paragraph whose deleted mark also holds inserted text keeps its
  `line=276`, as Word's redline keeps it (simple_ordered_list ×
  sublist_issue, "Lvl 1 – a"). Only a paragraph that inserts nothing drops
  that restated Normal spacing.

### Added

- `jubarte debug FILE [FILE2]`: a short Word-validity triage of a redline,
  for the shapes Word refuses and the OpenXmlValidator passes. It prints
  counts by kind with a few examples (`-n`), and with two files only what
  differs. Checks (`-c`): `orphans` (deleted text outside its story's
  deletion), `fields` (nesting per story, partly deleted fields),
  `bookmarks` (unpaired, duplicate, crossing a control, cell, text box or
  revision, or inside a plain-text or list control), `package` (content
  types, relationships, dangling references, undeclared `mc:Ignorable`
  prefixes), `structure` (blank field codes, cells not ending in a
  paragraph, nested same-kind revisions), `ids` (revision and `docPr` ids
  used twice; Word opens such files, so it is opt-in), `chains`, `elements`
  and `textbox`. `--list` lists the entries, or with two files the ones that
  differ. On the 450 English redlines of c6307ac, orphaned field codes and
  bookmarks in single-value controls flag only files Word refused.
- convert: Korean page and list number formats (b8a19b1).
- RESULTS.md tables scored against Word truth: Word's redline of the pair,
  converted by Word. Rows split redlining (tool redline → Word PDF) from
  conversion (Word redline → tool PDF) (6b72be2, a6953f1).

### Changed

- RESULTS.md applies one rule to every tool. Each table keeps a tool's
  latest run: jubarte no longer shows its best run of each week. The pooled
  docx→pdf tables rank only on the corpora every ranked tool converted, over
  the same documents, and a document a tool has no score for counts 0. Each
  corpus gets its own column, so jubarte's clean ranking now covers 1,204
  documents (English a + b and docxide's suite) instead of 2,102.
  - Competitors join fixtures_500 from their 2026-09-22 run: docxide 0.2673,
    soffice 0.3365, jubarte 0.6524.
  - A run over 7 days older than its table's newest is marked †.
  - Harness Docs counts every attempted document; failures already scored 0.
  - Redline speed is split by pair set, so a 5,000-pair run is never ranked
    against a 90-pair one, and 2-pair probes are dropped.
  - `--compress` is listed unranked.
  - Tables with no jubarte-redlines run, and the retired TypeScript ports
    (jubarte-first, -native, -lossless, dist/jubarte-final), are left out.

### Performance

- LCS keys follow their word hash in one walk; `group_by_key_stable` hashes
  each key once; move detection counts words and tokens in one walk (3d4ef7f,
  4a05c21, 241e9ac).
- PDF conversion parses `settings.xml` once per document instead of once per
  setting it reads, and the XML name interner hashes with foldhash. Cold
  one-shot conversion is 4.4% faster over 80 fixtures_500 documents; the
  output PDFs are byte-identical on 150.
- A font face looks its glyphs up in the cmap when asked instead of listing
  every mapped codepoint when it loads: a further 4.2% off cold conversion
  over 80 documents, PDFs byte-identical on all 500 fixtures_500 documents.
- An on-disk font index (`font-index.tsv` beside jubarte's font folder)
  remembers which files each font family resolved to. Later processes read
  those files directly instead of listing and opening every candidate in the
  system, Word and cloud-font folders; a changed folder or file sends the
  family back to the full search. Cold conversion median 46.9 → 21.4 ms over
  80 fixtures_500 documents, PDFs byte-identical on all 500.
  `JUBARTE_FONT_INDEX` moves it, `off` disables it.
- Each document remembers which face a catalogue font lands on instead of
  string-matching the family for every glyph, and the PDF writer prints glyph
  ids, pen moves and text state without per-glyph allocation. Together they cut
  another 9.5% off cold conversion (median 22.1 → 18.0 ms over 80 documents);
  PDFs byte-identical on all 500 fixtures_500 documents.
- An image repeated across pages (a header logo) is compressed once, not once
  per page before its duplicates were dropped: 2.1% off cold conversion over
  80 documents, PDFs byte-identical on all 500.
- The first-descendant lookups style and run-property reads make stop at the
  first match instead of walking the rest of the subtree: 1.6% off cold
  conversion over 80 documents, PDFs byte-identical on all 500 and redlines
  identical on 300 pool pairs.

## [0.9.2] - 2026-09-26

> **Summary.** A Word-fidelity pass on jubarte convert: layout rules reconstructed from live-Word probes, painted revision marks (--revisions conventional|word|custom), Word's comment balloons and change bars, East Asian layout, and smaller PDFs (--compress averages 0.70x Word's size on 2,102 documents). The redline engine is unchanged from 0.9.0.

A Word-fidelity pass on `jubarte convert`, driven by a 500 + 500-file corpus
of Word-exported PDFs (HF `superdoc-dev/docx-corpus`, plus 451 Word redlines
of them). Every rule was reconstructed from a live-Word probe — synthetic
document, Word export, measured numbers — and is written down with its probe
and implementing commit in [`docs/WORD_LAYOUT_RULES.md`](docs/WORD_LAYOUT_RULES.md).
The per-document bake-off on the 500-file set moved from 129/500 to 284/500
wins between milestone merges (mean Jaccard 0.134 → 0.360 at the first).

The redline engine is unchanged: `compare_documents` emits the same bytes as
0.9.1. All work is in `src/convert`, the CLI, and the OPC package layer.

### Performance

- Smaller PDFs, same pages. Subset faces now also drop glyph names
  (`post` format 3), unused metrics and every `cmap`/`name` record the PDF
  does not read; a collection (`.ttc`) face subsets like a lone font instead
  of embedding the whole collection. A CID font's `/W` lists only the glyph
  ids the pages use (a CJK face's widths were 212 KB of a 319 KB PDF). A
  picture repeated on many pages is embedded once. Plain glyph runs in the
  same font, size, colour and tracking share one text object and move by
  relative `Td`, so a page is no longer one `BT … ET` per glyph. Word-device
  glyphs (11.04/16.08pt body sizes) keep their own `q … cm … Q`: MuPDF hints
  them differently when the 0.24 scale sits in the text matrix.
- With `--compress` the average PDF is smaller than Word's on every corpus:
  145 KB against Word's 208 KB (0.70×) over the 2,102 clean documents and
  377 KB against 449 KB (0.84×) over the 1,416 redlines. Glyph positions are unchanged and the docxide-metrics
  scores with them: mean Jaccard 0.6469 clean / 0.4654 redlines against
  0.6468 / 0.4656 without `--compress`.

### Added

- **Painted revision marks.** `jubarte convert --revisions conventional|word|custom`
  with `--revision-palette` for `custom` (library: `PdfOptions::revisions`,
  `RevisionStyle` / `RevisionPalette`; Python:
  `docx_to_pdf(…, revisions=…, revision_palette=…)`; wasm/npm:
  `docxToPdf(bytes, compress?, revisions?, revisionPalette?)`).
  `conventional` (the default) is the legal-redline convention — deletions
  red struck through, insertions blue double-underlined, moves green;
  `word` reproduces Microsoft Word's own markup, including the per-author
  colour palette by first appearance, and is what the fidelity gates
  measure.
- **Word's redline chrome.** A document with comments — or an inserted or
  deleted table cell — gets Word's balloon pane, scaled by the right margin;
  change bars stand where Word draws them.
- **Fonts install beside the binary.** `scripts/install.sh` (Windows:
  `scripts\install.ps1`) copies the bundled open faces — Roboto Condensed
  (the exact files Word's cloud cache draws) and Selawik, Microsoft's
  metric-compatible stand-in for Segoe UI — into `$JUBARTE_FONT_DIR` or the
  per-user font directory instead of embedding them. Word's own cloud-font
  cache is read too, including localized family names and folders with
  numeric file names; macOS fonts are found by family name and Mac Roman
  names, not only file names.
- **East Asian layout.** Word's CJK fallback faces and East Asian families
  named outside `fontTable.xml`; vertical `w:textDirection="tbRl"` sections;
  ideographic line breaking with kinsoku; closing CJK punctuation hangs past
  the right edge; Latin inside East Asian text takes the `ascii` face and a
  quarter-em gap; Chinese and Taiwanese counting list formats; East Asian
  faces take Word's 1.3× line.
- **Right-to-left.** `w:bidi` paragraphs mirror their alignment and indents;
  `w:bidiVisual` tables run from the right margin; RTL lines paint in visual
  order; RTL runs take their `w:szCs` size; `minorBidi`/`majorBidi` theme
  slots take the complex-script face; Thaana fallback; a missing Arabic or
  Hebrew charset face is Arial.
- **`w:altChunk`.** HTML and MHT alternate content renders.
- **Frames.** `w:framePr` — including a style's `framePr`, and in headers
  and footers — floats its paragraph; page-anchored body frames float as
  boxes; auto-height frames grow to their paragraphs; a frame narrows every
  paragraph beside it and needs more than an inch of column to do so; with
  room on both sides its label sits on the left.
- **Text boxes and shapes.** Header/footer text boxes; shapes and text boxes
  anchored in table cells and inside content controls; list paragraphs in
  boxes keep their marker and indent; `sizeRel` percentages take their own
  frame and `spAutoFit` boxes fit their text; VML groups place children in
  group coordinates and honour `grpFill`; VML lines stroke where Word draws
  them; `w10:wrap`; VML pictures honour the `imagedata` crop; drawing groups
  and canvases; custom geometry paths in EMU with bound-arc tessellation;
  `a:prstClr`/`a:sysClr` colours; block arrows point their own way.
- **Effects.** `a:softEdge` fades, `a:duotone` recolouring, Word's "Washout"
  picture watermark, ellipse picture crops, text outlines with no fill, and
  `w:w` horizontal text scale measured as well as painted.
- **Metafiles and rasters.** EMF paths, Béziers and window/viewport mapping;
  WMF `META_POLYPOLYGON` with 0-based object handles; BMP and GIF pictures;
  CMYK photos embed as RGB.
- **Fields and forms.** `FORMCHECKBOX` legacy form fields; `PAGE` counts the
  page, in header/footer text boxes too; `w:fldData` stays binary.
- **Page furniture.** `tblCellSpacing` gaps; continuous sections that switch
  columns and side margins mid-page; page borders, including the medium-gap
  inward band and page-relative placement; `vAlign=center` centres the
  layout box; `titlePg` without a first-page part leaves page one bare; the
  page background lays out a header where the section has none; each
  SmartArt diagram paints its own drawing; a chart without `c:title` paints
  no title.
- **Word's layout rules** (probed, then implemented): `docGrid` charSpace in
  4096ths of a point, halved for half-width; compat mode 15; widow/orphan
  control on by default as in Word; `keepNext`/`keepLines` holding table
  rows together; `pageBreakBefore` never skipping a page; `contextualSpacing`
  per paragraph; baselines on the 0.24pt device grid and the left margin on
  the 1/300in grid; justified Word 2013 lines squeezing their spaces to keep
  a word.
- **`/ToUnicode` on every Identity-H font**, so text copies and searches out
  of the exported PDF; a glyph shaped from several characters maps to all of
  them.

### Fixed

- **List markers with an `hAnsi`-only face.** A numbering level whose
  `w:rFonts` names only `hAnsi` (e.g. Arial Unicode MS) painted its ASCII
  number in that face and took its taller line; `hAnsi` covers non-ASCII
  characters only, so `1.` now stays in the text's ASCII face and line
  pitch, as in Word.
- **Floating objects and wrapping.** Square/tight/through wraps carve the
  wrap polygon — a blocked line steps past the polygon, not the extent;
  `topAndBottom` floats push later lines; tight page banners move the
  paragraph above their anchor; behind-text boxes paint under the body;
  `distT`/`distB` stay off the body for header floats; front floats stack by
  `relativeHeight` across pictures and boxes; `wp:align` aligns within its
  `relativeFrom` frame; a float anchored after a page-spanning paragraph
  lands on its last page; floating tables keep the lines above their offset,
  break across pages when taller than the page, and lift to keep a page of
  rows.
- **Tables.** Autofit measures a word by the pieces it may break into and
  widens to the longest word — a too-narrow column takes the saved grid, a
  spanning cell widens its columns, vertical text keeps its width; `pct` vs
  `dxa` widths under `noWrap`; a zero `dxa` width is auto; `gridBefore` /
  `gridAfter` leave their grid columns empty; `tblPrEx`; cell margins and
  `tcBorders` edges apply per Word's order; vertically merged cells grow the
  last row they span; nested tables add no tail to their cell; `w:hideMark`
  rows and cells; PHPWord tables without `tblInd` stay at the margin; a word
  wider than its cell breaks at the edge.
- **Headers and footers.** Floats, text boxes and pictures — wrapping into
  rows, beside text, behind text, framed — sit where Word puts them; tabs
  past the margin wrap and wrapped lines drop the logo height; lines take
  the paragraph's style, `jc`, line rule and spacing; bottom borders take
  room; tracked changes and ptabs paint.
- **Lines, fonts and measurement.** A line is as tall as its tallest painted
  face, marker included; `hhea` — not OS/2 typo — metrics for the single
  line box, with typo-metrics faces putting the gap above the text; GDI
  external leading above the first baseline; glyphs laid out at the authored
  size and painted at the device size; hidden-mark paragraphs, cells and
  rows take no line; letter spacing in headers/footers and character spacing
  in the wrap measure; a missing font paints in its installed `altName`.
- **Tabs, lists and indents.** A hanging indent is an implicit tab stop; a
  list marker lifts only its own line; numbering-level `pPr`/`jc` and
  `lvlOverride` apply; the label's tab lands on the hanging indent, not a
  later right stop; a typed lone symbol does not hang like a bullet; `w:cr`
  ends the line and a line feed inside `w:t` is a space.
- **Spacing and pagination.** Space before collapses at page and column
  tops the way the break leaves it; a page break inside a paragraph moves
  the rest to the next page; column-break marks take a line; a section break
  takes the type of the section it starts; odd/even parity survives a
  numbering restart; a page nothing was placed on never breaks; a tracked
  section change is not a section break.
- **Package layer.** Relationship targets are XML-unescaped before
  resolving, so part names containing `&amp;` resolve to the right part.
- **PDF output.** Horizontally scaled runs squeeze their glyphs; every
  Identity-H font carries `/ToUnicode` (see Added).
- **Harness.** Six tests added since `fb241e2` were missing `#[test]` and
  never ran — restored. The 76/398 sweep and the 50-row smoke now convert
  with `--revisions word`, matching the markup the gate scores.

### Performance

- Font lookups and shaped words are memoised, PDF content ops are written in
  place, shape plans are reused, and faces load only for the families a run
  paints.
- Embedded TrueType faces are subsetted (glyph ids kept) and font programs
  and image samples always deflate — `--compress` remains about the page
  content streams.

## [0.9.1] - 2026-09-22

### Added

- **`word/fontTable.xml` altName.** Unknown requested families now follow
  Word's recorded `w:altName` (installed faces still win). Calibri stays the
  Carlito catalogue slot so an altName on Calibri cannot steal Cambria.
- **Quoted CSS family lists are not Times.** Word splits `w:ascii` on comma
  but does not CSS-unquote, so `"Times New Roman", Times, serif` is unknown
  (Quartz used Cambria) while `Verdana, Geneva, sans-serif` is Verdana.
- **Theme minor slot for every face.** `minorHAnsi` → theme minor (Cambria
  included). The Aptos-only gate is gone. `file_2` / `file_41` may drop
  until the line-box formula lands; explicit `w:ascii` still wins.
- **Word-substitution evidence table.** Unknown / quoted CSS-list families
  resolve to Cambria (Word Quartz). `DejaVu Sans Mono` → Verdana;
  `Liberation Serif` → Times New Roman; empty request → Times New Roman;
  `w:family`/`w:pitch` generics (roman/swiss/modern/fixed) after altName.
- **`FaceKey` catalogue shim.** Physical family + bold/italic, with lazy
  `OnceLock` loads so a conversion does not parse all 47 faces up front.
- **Embedded `.odttf` fonts.** `w:embedRegular` (and bold/italic slots) are
  de-obfuscated with the `w:fontKey` GUID (ECMA-376-2 §11) and registered as
  extra faces for that conversion. Unknown families such as case8
  `Press Start 2P` no longer fall through to Cambria.

- **Convert fidelity gate.** `scripts/convert_sweep.py` (wrapper
  `scripts/convert-sweep.sh`) scores `jubarte convert` against Word PDFs on the
  76 docxide fixtures and the 398 corpus, path-referencing the sibling
  checkouts rather than vendoring 46 MB of cases. `planning/sample50_check.py`
  is the 50-row smoke after every engine change. Baselines from jubarte 0.9.0:
  76 mean Jaccard **13.74** (median 8.86), 398 mean Jaccard **53.10**
  (median 43.50). The gate itself does not change converter output.

### Fixed

- **A space-only run keeps its space.** `birds.` + `<w:t xml:space="preserve"> </w:t>`
  + `We` painted "birds.We": whitespace collapsing squeezed a lone-space run to
  nothing, and the pretty-print filter treated whitespace inside `w:t` as
  XML indentation. Body and header/footer text both keep the gap now.
- **CI on GitHub Actions.** `cargo fmt` wraps two `finalize.rs` lines; tests
  that `std::fs::read` sibling `../neurotic_docx_bench` fixtures now skip when
  that checkout is absent (Actions has no sibling). `deny.toml` ignores
  RUSTSEC-2026-0206 (`rustybuzz` unmaintained) and RUSTSEC-2026-0192
  (`ttf-parser` via rustybuzz) until the FaceKey/harfbuzz swap (plan 2e).
  Clippy 1.98 `chunks_exact_to_as_chunks` on the WMF gray-fill helper.
  Convert tests that look for `/Calibri` in the PDF also accept bundled
  `/Carlito` (Actions has no Word DFonts). Aptos/Calibri wrap-width oracles
  skip when Word DFonts are absent (PDF still names the logical face).
  Font overlay no longer scans `/System/Library/Fonts` (Apple Symbol ≠ Word
  SymbolMT; GitHub macOS runners have it).

## [0.9.0] - 2026-09-05

Ring 3 is green for the first time: all 207 corpus redlines open in Microsoft
Word with no warning, error or repair offer (was 202/5).

### Fixed

- **`w:instrText` under `w:del` is now `w:delInstrText`.** This was the cause of
  every one of the five Ring 3 open failures. Word wants `w:delInstrText` in a
  deletion for the same reason it wants `w:delText` instead of `w:t`, and offers
  to repair the file when it does not get it. The shape came from deleting
  content containing a field — a wholly deleted header or footer carrying
  `PAGE`/`NUMPAGES` is the usual case — on a path that wraps existing runs in
  `w:del` rather than rebuilding them. Only 5 of the 207 corpus outputs carried
  it, and they were exactly the 5 Word rejected.

  **The OpenXmlValidator reported all five as clean.** `w:instrText` is
  schema-valid in a run whatever the run's parent is; the del/delInstrText
  correspondence is a semantic rule Word applies at load. A green Ring 2 does
  not mean Word will open the file.

- **`w:del` no longer swallows `w:hyperlink`.** `w:hyperlink` is not in
  CT_RunTrackChange's content model, so deleting a whole header/footer produced
  markup Word rejects. Word's shape is the inverse — the hyperlink stays put and
  the revision moves inside it — and the revision is now split around each
  hyperlink, preserving document order and minting fresh `w:id`s.

- **No more bare `<w:sz/>` / `<w:szCs/>` in the merged Normal style.** The
  style-merge loop created the element and then passed `None`, which strips
  `w:val` but leaves the element behind. `w:val` is required on CT_HpsMeasure;
  ECMA-376 spells "no value" as the element's absence.

- **Invalidity inherited from a source is repaired instead of shipped.**
  Unqualified attributes on `w:` elements (`paragraphProperties="[object
  Object]"` — wml.xsd is `attributeFormDefault="qualified"`, so these are invalid
  by construction), `w:shd` with no `w:val` (supplied as `clear`, keeping the
  author's fill), and `w:highlight` under `w:lvl/w:rPr`. Deletion-or-default
  only, so a document that was already valid cannot change. Worth stating
  plainly: **all these sources open fine in Word** — it tolerates their
  invalidity and rejected our output, so this was never the reason those
  documents failed. It is repaired anyway, because a source's corruption
  shipped inside our redline gets blamed on us.

### Added

- `finalize::enforce_deleted_text_kinds`, `finalize::hoist_hyperlinks_out_of_revisions`
  and `finalize::repair_inherited_invalidity`, each run both at the end of the
  body pipeline and in the package-level validity sweep — headers and footers
  deleted wholesale never reach the body finalize path, which is why the three
  defects above survived there.
- `tools/validate-docx/` is **committed** for the first time. Ring 2 could not
  run in a fresh checkout: the C# project existed only in a sibling worktree and
  was on no remote. It now also reports the part URI and XPath of each finding,
  so a Ring 2 result names a location instead of only a rule. The ratchet keys
  on the first two columns and is unaffected.

### Changed

- `tools/validity_baseline.tsv` re-blessed: 1183 findings / 46 pairs / 60 keys,
  from 1294 / 54 / 74. The previous bless keyed stems as `<stem>.ours`, which
  `scripts/redline-sweep.sh` never emits, so the ratchet compared two disjoint
  key sets and reported 74 regressions plus 75 fixes on an unchanged corpus.

### Known

- **Ring 1 is red and was already red before this release**: `parity_ladder.py
  sweep` reports 41 NEW findings (4 at L0) against the checked-in baseline on an
  unmodified tree. This release introduces none of them — the identical 41 appear
  when the same sweep runs against a binary built without these changes — and
  fixes one. They are not blessed away, because L0 is the losslessness contract
  and hiding those rows is the one thing the ladder exists to prevent.
- The `w:Unid` leak (KNOWN_ISSUES 4) is narrowed but still open: 12 of 207
  outputs, 0 of 199 sources.

## [0.8.0] - 2026-09-05

Two losslessness fixes in the redline core, and the DOCX → PDF converter that
0.7.1 prepared but never shipped to crates.io.

### Fixed

- **Redline no longer drops sentence-final punctuation (M463).** The
  fold that attached a trailing bare `.` EQ run onto the preceding
  `w:ins`/`w:del` moved a run that belongs to *both* documents into a
  one-sided revision and then deleted it, so the period left the other
  side's stream — onto a `w:del` it vanished from the modified document,
  onto a `w:ins` from the original. 16 corpus pairs stopped reconstructing
  their own inputs. The whole `fold_boiler_eq_between_ins` pass is removed:
  keeping only its `INS|EQ|INS` half measured worse (19 vs 18 L0 rows).
- **Redline no longer rewrites letter case (M328d).** The free-mesh word
  rehash hashed `text.to_ascii_lowercase()`, so `"Green"` and `"green"`
  compared Equal — but the emitted EQ run carries only one side's casing,
  turning `Green highlights…` into `green highlights…`. The extra spurious
  matches also let two output paragraphs claim the same source atom. The
  hash is case-sensitive again at all 20 call sites; M328d's unrelated
  stamped-pair (`file_N.docx`) free-mesh exclusion is kept. Case-insensitive
  matching is sound only once the emitted run keeps each side's own text.

Ring 1 (`tools/parity_ladder.py sweep`, 207 pairs): L0 reconstruction
failures 34 → 17 rows; regressions against `042089c` 19 → 2. The two
survivors are a distinct free-mesh double-consumption defect — see
KNOWN_ISSUES.md issue 3. 15 of the 17 predate `042089c`.

### Added

- **`jubarte convert` / `convert::docx_to_pdf`.** Emit a real multi-page PDF
  from DOCX bytes without LibreOffice: paragraphs, lists, tables, JPEG/PNG,
  headers/footers, and WMF/EMF rasterization. Prepared in 0.7.1, which was
  tagged but never published — 0.7.0 is the newest version on crates.io, so
  this is the first release in which `convert` is installable.
- **`convert::PdfOptions { compress }`** (CLI: `jubarte convert --compress`)
  — `/FlateDecode` the content streams. Off keeps page content greppable
  with `strings`; on is substantially smaller.
- Converter fidelity work against Word: OMML `m:f` `noBar` fractions, tab
  stop resolution (`w:tabs`, `defaultTabStop`, ISO-Strict `ST_TabJc`),
  Word's `size >= w:kern/@val / 2` kerning threshold, `GridTable4-Accent5`
  and `MediumShading` table styles, DrawingML stroke width/colour from
  `a:ln/@w` and `lnRef`, textbox `w:ind`/`w:spacing` and
  `relativeFrom=margin`, `w14:reflection` flattening as Word's PDF export
  does, and small-caps shrinking lowercase only (ECMA-376 17.3.2.5).

## [0.7.1] - 2026-08-16

Independent DOCX → PDF converter. Redline output is unchanged from 0.7.0.

### Added

- **`jubarte convert` / `convert::docx_to_pdf`.** Emit a real multi-page PDF
  from DOCX bytes without LibreOffice: paragraphs, lists, tables, JPEG/PNG,
  headers/footers, and WMF/EMF rasterization. Layout aims at soffice visual
  parity — Carlito/Liberation (the metric-compatible faces LibreOffice
  embeds), rustybuzz shaping, `sectPr` page geometry, and named-style
  resolution.

### Fixed

- Clippy 1.97 `question_mark` on the theme `lastClr` fallback.
- rustfmt across comparer / `document_comparer`.

## [0.7.0] - 2026-08-13

**Jubarte now wins on speed as well as quality.** 0.6.0 already led every
fidelity metric on the 763-document `script_redlines` benchmark; 0.7.0 closes
the last gap to the C# incumbent on generation time. Measured interleaved
(both engines on the same pair back-to-back, so concurrent machine load hits
both equally) over the 4880 pairs both engines complete:

| speed measure | jubarte 0.7.0 | docxodus 9.0.0 |
|---|---|---|
| median / doc | **5.3 ms** | 7.2 ms |
| mean / doc | **22.2 ms** | 24.1 ms |
| p95 / doc | **94.8 ms** | 96.2 ms |
| p99 / doc | **139.7 ms** | 179.9 ms |
| throughput | **45.0/s** | 41.4/s |
| generation failures | **0** | 120 |

Jubarte leads all six. Every performance change below is output-identical to
0.6.0 (verified by LibreOffice render parity, XML c14n equivalence, and LCS
fuzz/collision tests) — no fidelity was traded for speed.

### Performance

- **Killed the superlinear tail** in the word-level relatedness detector: the
  worst-case pair dropped 837 → 678 ms with no output change.
- **Detector fast-path + by-reference descendants walk.** `detect_unrelated_
  sources_word_mode` now short-circuits the full-document word-LCS when all
  keep-LCS cases are provably impossible (O(n+m) rolling-hash pre-check), and
  `Dom::for_each_descendant_element` compares element names by reference
  instead of cloning an `XName` (2 Arc bumps) per element across the ~84
  finalize passes. Together these flipped mean and throughput to jubarte.
- **Poststep re-parse elimination.** The Word-validity poststeps parsed
  `styles.xml` four times per compare; they now cache the defined-style-id set
  from the styles-copy pass and reuse a single styles arena in the M-PAG
  Normal-merge. This closed the p95 gap (6/6). A 128-bit FNV-1a fingerprint
  (`sha1_key128`) replaces the per-step 40-byte hex compare in the LCS extend
  step (~2⁻¹²⁸ collision), and the LCS bucket index uses an identity hasher on
  its already-hashed u64 keys.

### Fixed

- **Mesh & revision ordering** — M468 (yields to the M322 head-junction; no
  fold across trailing empty pure-I separators), M469 (splits a short inserted
  title MIX from a long unrelated deletion), M471 (rotates the impossible
  ins-mark del-only paragraph), M472 (re-asserts ins-before-del order after a
  comment carry), M473/M474 (restamps a stranded deletion mark; field-residue
  gate), M491 (B's document-final paragraph mark never inserts mid-document).
- **Spacing** — M487 bakes B's effective paragraph spacing onto inserted
  paragraphs, gated to B-implicit values only and never onto empty or
  declared-value paragraphs; M492 keeps deleted paragraphs' A-original direct
  spacing; M479 lets spacing `before` join the Normal merge under the B-chain
  gate.
- **Styles** — M476 gates the S2 copied-style bake on ascii font-family change;
  M477 adds a per-attribute B-chain bake gate and Word-complete Normal
  promotion; M478 materializes implicit `kern`/`ligatures` neutralizers; M480b
  adds docDefaults-delta disabling neutralizers on both-sides merged styles;
  M483 re-caches themed color hexes against the shipped theme.
- **Numbering** — M481/M482 repair the core relationship part and remap
  `numId` collisions; `w15:restartNumberingAfterBreak` is ignored in
  `abstractNum` identity.
- **Images** — M495 keeps an image-only paragraph M491 had misjudged as empty;
  M496 carries over the revised image on an inserted-reference `rId` collision.
- **Sections** — M494 emits no spurious `sectPrChange` for implicit-default
  section properties.
- **Fields** — M470 keeps Word's field form for deleted anchor hyperlinks.

## [0.6.0] - 2026-08-11

**Jubarte is now the best redline engine on the market**, leading every
headline metric on the 763-document `script_redlines` benchmark against the
Microsoft Word oracle (neurotic-docx-bench, LibreOffice 26.2.4.2 renderer):

| metric | jubarte 0.6.0 | docxodus 9.0.0 |
|---|---|---|
| mean fidelity | **83.27** | 80.55 |
| median fidelity | **91.67** | 91.19 |
| generation failures | **0** | 4 |
| documents ≥ 90 | **403** | 392 |
| generate time, median/doc | **20.7 ms** | 82.2 ms (4.0× slower) |
| generate time, mean/doc | **59.5 ms** | 601.4 ms (10.1× slower) |

### Fixed

- **M460** — the heading `line=240` stamp (M79) now fires only when the merged
  Normal is itself Word-normalized single-line 240; a Normal carrying B's
  non-240 line left headings line-less in the oracle (basic_comment ×
  cli_legacy +15).
- **M461** — the Normal rPr merge carries B's stored `w:kern` and
  `w14:ligatures`; dropping `kern=0` left kerning ON from A's docDefaults and
  narrowed every long paragraph one line short (basic_comment 50.5 → 97.7).
- **M462** — when A has no styles part, scaffold from Word's FACTORY
  docDefaults + Aptos theme and bake each copied B style's effective metrics,
  instead of adopting B's docDefaults wholesale (tiff_image pairs 38–40 →
  96–99.8).
- **M463** — inserted/deleted OMML math serializes Word-style: revision marks
  INSIDE `m:r` with a materialized Cambria Math rPr, `m:t` never delText,
  applied as a final pass so mesh reasoning is undisturbed (math family
  pairs +40, page-level pixel parity with the oracle).
- **M464** — the S2 copied-style bake covers pPr spacing deltas between the
  two docDefaults, not just run metrics (file_13 × file_14 class).
- **M465** — the mid-stream demo-title fold (M143) is gated off for anchored
  pairs (matched leading title): Word keeps A's deleted document intact at
  the end; also stops the M179 " Demo" EQ from stripping deletion marks
  (file_13 +22, file_145 +9).
- **M466** — the trailing bare-period attach skips runs carrying `rPrChange`;
  merging a format-changed period into delText dropped the tracked change
  (file_168 back to 100.00).
- **M467** — the merged Normal keeps only per-attribute deltas vs the output
  docDefaults, pruning kern/sz/szCs/rFonts/ligatures the context already
  supplies (tab_test × table_autofit; restored 34 exact-100 documents).

## [0.5.1] - 2026-07-24

### Fixed

- **CI was red and the test suite did not compile.** Two integration tests
  had drifted out of sync with the helpers they call: `m35_comments` passed
  `optional_bench_docx`'s `Option<Vec<u8>>` (file *bytes*) to a
  `require_path(&str)` guard, and `word_package_notes_settings_coherence`
  destructured an `Option` with a `Result` pattern after its loader moved to
  `.ok()`. The `m35_comments` guards were also redundant — the preceding
  `let Some(…) else { return }` already skips the missing-fixture case.
- `clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`
  both pass again: `items_after_test_module` in `comparer::preprocess` (the
  `escape_xml_tests` module now sits at end of file), plus accumulated lint
  drift in `examples/` — `const Z; [Z; N]` atomic-array splats replaced with
  inline `const {}` blocks, four descending `sort_by` comparators expressed as
  `sort_by_key(Reverse)`, and one `Default` field reassignment folded into
  struct-update syntax.
- Residual Word-visual peels on the finalize path (M216–M233): empty pure-D
  folds, MIX Heading/spacing/numPr parks (gated), mid pure-D live spacing
  promotion, schema-default `jc` left/start strip, jc-only `pPrChange` removal.
  Full main ledger **mean 90.04 / median 95.67** (n=164) at `d094de0`.

### Continuous integration

- Coverage is now measured and published. A `coverage` job runs
  `cargo llvm-cov --all-features --workspace --lcov` and uploads to Codecov via
  `codecov/codecov-action@v7`; [`codecov.yml`](codecov.yml) sets an `auto`
  project target with a 1% tolerance and an 80% patch target, and excludes
  `tests/`, `benches/`, `examples/`, `tools/`, `parity/` and `scripts/` from the
  denominator. The README's Codecov badge and coverage row have existed for
  some time but had never received an upload.

### Performance

- M232/M233: single-pass spacing+jc cleanup and lazy pure-del/mixed paragraph
  classification cache for multi-pass peels.

### Documentation

- `docs/BENCHMARK_M233.md` — full quality + speed stamp (main, randomized
  `file_i_v_file_{i+1}`, 5k-pair speed bench, criterion, expanded ABBA).
- `tools/perf/run_abba_matrix.sh` — optional sample expansion with consecutive
  file pairs (`FILE_SAMPLE=1`).

## [0.5.0] - 2026-07-15

Product line alignment with the desktop app: same **0.5.0** minor for the
shipped engine that powers the Mac App Store build. Includes everything from
0.2.0 (package-wide Word validity, notes/settings coherence, parity restore,
measured Q0 performance stack) plus release tooling (`VERSIONING.md`,
`scripts/bump-version.mjs`).

## [0.2.0] - 2026-07-15

### Fixed

- **Word package validity is package-wide**, not `document.xml` alone: strip
  PowerTools `pt:*` markup across OPC parts and re-sync settings after the
  validity sweep so Microsoft Word does not report unreadable content.
- **Notes / settings coherence:** keep structural note types
  (`continuationNotice` id=1, etc.), renumber user notes around reserved ids,
  and ensure `settings.xml` footnotePr/endnotePr special-note ids ⊆ the notes
  parts (Word opens the full OPC package).
- **Parity restore after ATOM-STACK / IDENTICAL-INPUT work:** footnote and
  endnote definitions stay on the atomize path stack so deleted-note produce
  no longer panics; identical-package short-circuit still runs drawing id
  fixups (`wp:docPr`) so pre-existing source collisions do not reappear as
  `S-dup-docpr-id` on the ladder.

### Performance

- Large Q0 wall stack (measured; see `LCS_PERF_PLAN.md`): atomize path stack,
  serialize direct buffer writes, SHA-1 streaming digests, simple-p/tc hash
  without clone DOM, accept clean-subtree reuse, accept skip when transforms
  cannot fire (rsid, empty cells, fields, A.3 move ranges, A.5 deleted marks,
  …), OnceLock `XName` caches (NAME-01 / 01b / 01c).
- Banked experiments kept as exact cleanup where full permanent ABBA matrix
  did not win every load-bearing slot (ACCEPT-SKIP-A3/A5, NAME-01c, …).

### Added

- `VERSIONING.md` + `scripts/bump-version.mjs` for one-shot Cargo version
  codemod and neurotic binary install steps.
- Focused perf exact tests under `tests/perf_*.rs` for the Q0 gates above.

### Quality

- Parity ladder re-blessed to zero NEW keys after the notes/stack/docPr fixes.
- Full neurotic visual ledger class retained (historical floor ~83.8 mean /
  ~88.5 median on script_redlines sample/full runs during the stack).

## [0.1.0] - 2026-07-12

### Added

- Initial release, extracted from the `ooxmlsdk-redline` development crate.
- `document_comparer::compare_documents` (+ `_with_options`,
  `_with_settings`): compare two `.docx` documents into a tracked-changes
  (redline) `.docx`.
- `document_comparer::get_revisions`: list tracked revisions (type, author,
  date, part, move group, format-change details, text).
- `document_comparer::accept_revisions` / `reject_revisions`: flatten a
  redline package-wide.
- `comparer::WmlComparerSettings`: author/date stamping, detail threshold,
  Word-visual alignment passes (default) or the PowerTools-faithful preset.
- `jubarte` CLI (default `cli` feature): plain compare plus `revisions` and
  `accept` subcommands.

### Fixed

- External hyperlinks no longer lose their targets in the default
  (Word-visual) mode: `unwrap_hyperlinks_to_styled_runs` now preserves
  `r:id`-bearing `w:hyperlink` wrappers and unwraps only anchor-based
  internal (TOC) hyperlinks, so relationship reconciliation keeps the
  hyperlink relationship (with `TargetMode="External"`) in the output.

### Known issues

- See [KNOWN_ISSUES.md](KNOWN_ISSUES.md); the covering tests are marked
  `#[ignore]` with matching reasons.

[0.10.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.10.0
[0.9.3]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.3
[0.9.2]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.2
[0.9.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.1
[0.9.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.9.0
[0.8.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.8.0
[0.7.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.1
[0.7.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.7.0
[0.6.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.6.0
[0.5.1]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.1
[0.5.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.5.0
[0.2.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.2.0
[0.1.0]: https://github.com/jandira-tech/jubarte-redlines/releases/tag/v0.1.0
