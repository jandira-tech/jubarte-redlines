<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# What jubarte takes from anymd: plan (2026-10-14)

> **Status (reconciled 2026-10-17, `main` `aba48bd5`): all six items still open.**
> Item 1: `parse_pages` (`src/bin/jubarte.rs:632`) still expands ranges
> unbounded. Item 2: the PDF writes `/Annots` only for opt-in comment note
> icons (`pdf.rs:1064`, `/Subtype /Text`); no `/Link`, `/Outlines`, or
> `/PageLabels`. Item 3: `extract_media` is still `None` at every call site
> (`bin/jubarte.rs:2143`); pictures without alt text still drop from the view.
> Item 4: no `c:chart` handling in the view. Item 5: no `read --outline`.
> Item 6: no token budgets, `--max-tokens`, or cursor paging. No id-contract
> impact.

## Where this list comes from

Arthur asked for a scout of [arthrod/anymd](https://github.com/arthrod/anymd) at
`0d9ed5e`.
- **Method.** One Devin swe-2 agent (empryo) reviewed each of anymd's 104 Rust
  source files under `crates/` against this tree. Each agent was told to presume
  anymd is better until it found a `file:line` of ours that does as well.
- **Results.** 6 BRING, 74 MAYBE, 24 SKIP. The reports and a one-line-per-file
  index are kept outside the repository, in
  `~/temp/T/anymd_review/{reports,INDEX.md}`.

Our Markdown reader descends from anymd. `src/markdown/from_docx/critic.rs` and
`mod.rs` were copied from anymd's `critic.rs` and `docx.rs` on 2026-09-30
(`e489e91e`), and anymd has barely changed them since.
- **The check.** Each of the 71 diff hunks where anymd has lines we lack was judged
  on its own: 41 REPLACED, 17 RENAMED, 13 DELIBERATE, 0 BRING.
- **The deliberate differences:**
  - a deletion followed by an insertion stays two changes (`d7b87793`);
  - several equations in one paragraph are joined with a space, not `\\`
    (`ec3d6001`);
  - stored authors cannot forge an agent tag (`4b77905c`).
- **Conclusion.** Nothing more comes from those two files. The items below come
  from anymd's other files.

**Licensing.** anymd is MIT, © 2024-2026 SylphxAI. Ported code keeps
`SPDX-FileCopyrightText: 2024-2026 SylphxAI` beside ours. The MIT attribution text
stays under `LICENSES/` and in `REUSE.toml` (see AGENTS.md).

**Status.** Item 1 is verified in our code. Items 2–6 are agent findings: check
each claim against the tree before writing its test. One PR per item, in this
order, after #476 and the label-numbering indent PR.

## 1. `--pages` cannot exhaust memory (from `anymd/src/page_selection.rs`)

Status: open; **verified**.
- **The bug.** `parse_pages` (`src/bin/jubarte.rs:632`) runs
  `pages.extend(first..=last)` without a bound. `validate_pages`
  (`src/cli.rs:2390`) promises it "never allocates a potentially enormous list",
  but that holds only for the check. So `--pages 1-100000000000` tries to allocate
  about 800 GB and aborts.
- **The same pattern elsewhere.** The Python CLI builds the same set
  (`jubarte-python/python/jubarte_redlines/__main__.py:182`). The wasm and npm
  page options need the same check.

- [ ] Keep the selection as ranges and intersect them with the document's page
  count before expanding (anymd counts first and caps at 10 001 pages; ranges
  need no cap). The existing out-of-range errors (`PageOutOfRange`, `PageNotKept`
  in `src/convert/mod.rs:585`) keep their meaning.
- [ ] Red test: `--pages 1-18446744073709551615` on a one-page document returns
  at once, in the CLI, Python and wasm.
- [ ] Keep our stricter parsing: anymd accepts `2x`, `1--5` and `1-2-9`; we
  reject them.

## 2. PDF bookmarks, page labels and clickable links (from `anymd-core/src/catalog_signals.rs`)

Status: open; agent finding.
- **What our PDF lacks.** Our catalog is `<< /Type /Catalog /Pages … >>`
  (`src/convert/pdf.rs:1081`): no `/Outlines`, no `/PageLabels`, and no `/Link`
  annotations.
- **Effect.** Hyperlinks are painted blue but cannot be clicked. We already read
  outline levels (`src/convert/mod.rs:1141`) and Word's page-number formats
  (`alpha_label`/`roman_label`, `src/convert/mod.rs:6375`).

- [ ] First, read what Word puts in its own PDFs. Take the bench's Word reference
  PDFs (made by `word_pdf.py`) and record whether they carry `/Outlines`,
  `/PageLabels` and `/Link` annotations, and from which headings and fields. Word
  is the target, not anymd.
- [ ] `/Link` annotations for `w:hyperlink` and `HYPERLINK` fields:
  - external targets become `/URI`, allowing only http, https, ftp, mailto and
    tel (anymd 649-668);
  - `w:anchor` and `\l` targets become `/GoTo` to the bookmark's page and
    position.
- [ ] `/PageLabels` from each section's `w:pgNumType` (format and start). Only
  when Word's PDFs carry them.
- [ ] `/Outlines` from heading levels. Only if Word's PDFs carry them by default;
  otherwise behind an option.
- [ ] Tests:
  - a PDF reader lists the annotation and outline entries;
  - one Word side-by-side per feature;
  - `--revisions word` output keeps Word's choices.

## 3. Pictures in the Markdown view (from `anymd-formats/src/images.rs`)

Status: open; agent finding.
- **Missing pictures.** A picture without alt text produces no output in the
  agent view: `image_markdown` returns `None` (`src/markdown/from_docx/ooxml.rs:773`).
- **Image files are never written.** `MarkdownOptions::extract_media`
  (`src/markdown/from_docx/mod.rs:290`) is passed `None` by every caller (CLI,
  wasm, Python), so `jubarte read` cannot write openable image files.

- [ ] Red test: a docx whose picture has no alt text keeps a line for it in the
  view, and the line round-trips.
- [ ] Wire `extract_media` through `read`/`convert -t md` (CLI, Python, wasm)
  into a directory the caller names.
- [ ] Optional, view only: a `WxH` note per picture, and skipping decorative
  pictures (under 48 px a side, or the same image in three or more places). This
  never applies to the docx written back, which keeps every picture.

## 4. Charts in docx (from `anymd-formats/src/pptx.rs`)

Status: open; agent finding.
- **Current behaviour.** A `c:chart` in a docx drawing produces nothing in the
  view; `drawing()` (`src/markdown/from_docx/mod.rs:2894`) handles only pictures
  and text boxes.
- **What anymd does.** `chart_markdown`/`points` (anymd 521-588) turns a chart's
  cached series into a `### Chart:` heading and a table.

- [ ] Port it to the chart part referenced from a docx drawing, as a read-only
  block.
- [ ] Decide how the writer treats the block: keep the original chart part, never
  rebuild it from Markdown.
- [ ] Tests: a fixture docx with a bar chart and a line chart; the round trip
  keeps the chart part byte for byte.
- [ ] Separate and optional: PPTX to Markdown (slides in order, titles, bullets,
  tables, notes; anymd L10-L518).

## 5. An outline of the view (from `anymd/src/outline.rs`)

Status: open; agent finding.
- **What is missing.** No outline exists over our Markdown view:
  `marker_len` (`src/markdown/diff.rs:85`) reads `#` runs only.
- **What anymd has:**
  - a heading scanner (ATX and Setext, closing `#`s trimmed, skipping code and
    `<!-- -->` lines);
  - nodes with dotted ids, a section path ("A > B") and byte ranges;
  - `node_at` (which section holds an offset).

- [ ] Port the scanner, guarded against our YAML header (`split_front_matter`,
  `src/markdown/write.rs:223`), our id lines and our page markers.
- [ ] Expose it as `jubarte read --outline` (CLI, Python, wasm): one line per
  heading with its level, its id and its page.
- [ ] Tests from anymd's (405-415, 462-472): each node's offsets slice the text at
  a character boundary.

## 6. Reading a long view in parts (from `anymd/src/lean.rs`)

Status: open; agent finding. `read` prints the whole document; a 300-page redline
overflows an agent's context window.

- [ ] A token estimate (`estimate_tokens`, anymd 29-59, calibrated on o200k).
- [ ] `--head` and `--tail` take a token budget as well as a block count:
  `--head 8000t` is the leading whole blocks whose estimated tokens fit 8000,
  and `--tail 8000t` the trailing ones. A bare number stays a block count, as
  today (`src/cli.rs:88`); a table is one block. A block is never cut, and the
  first block always prints even when it alone exceeds the budget. The
  selection line names the estimate (`head 12 (p0-p11, ~7,900 tokens)`). The
  same `t` budget works in Python (`head=`/`tail=`) and wasm.
- [ ] Tests:
  - `--head Nt` and `--tail Nt` on each committed view stay within N estimated
    tokens, except for a first block that alone exceeds N;
  - `--head` with a bare number keeps today's output byte for byte;
  - `--head 0t` reads like `--head 0`.
- [ ] `read --max-tokens N` stops at a paragraph boundary and prints a
  resume cursor (`page:offset`). `read --cursor C` continues from it (anymd
  63-98, 186-202, 394-425).
- [ ] A cut never splits an id line from its paragraph, a table, a CriticMarkup
  span or a note.
- [ ] Tests: concatenating every part gives the whole view byte for byte, for
  each committed view in `tests/fixtures/md-roundtrip/`.
- [ ] Page-range compression for reports, such as `1-3,5` (`describe_pages`,
  anymd 165-183).
