<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 14 — Create a .docx from Markdown

Task: turn the same one-page letter (`input.md`: one `#` title, `##`
sections, a bullet list, a 4-row table, a numbered list) into a .docx
three ways — pandoc, docx-js (the npm `docx` package Anthropic's docx
skill uses), and jubarte — then check the page size each produced and
render page 1 of all three files with two independent renderers.

## The exact commands

Substituted tools (Anthropic skill's two creation paths):

```sh
pandoc input.md -o create_pandoc.docx

# docx-js: installed outside this repository, script in make_docxjs.mjs
DOCXJS_DIR=/tmp/docxjs_adopt; cd "$DOCXJS_DIR" && bun add docx
bun make_docxjs.mjs <folder>/create_docxjs.docx
```

`make_docxjs.mjs` builds the same content object by object and, on
purpose, sets no page size — that is the skill's own documented footgun
("Page size defaults to A4").

jubarte:

```sh
jubarte convert input.md -o create_jubarte.docx --page letter
```

Renderers (both applied to all three .docx files):

```sh
soffice -env:UserInstallation=file:///tmp/lo_adopt_14 \
    --headless --convert-to pdf --outdir . create_pandoc.docx
pdftoppm -png -r 72 -singlefile create_pandoc.pdf render_page_1_pandoc_soffice
jubarte convert create_pandoc.docx --png --dpi 72 --pages 1
```

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| pandoc | 3.11 |
| bun | 1.4.2 |
| docx (npm) | 9.8.1 |
| jubarte | 0.11.2 |
| LibreOffice | 26.8.0.3 |
| Poppler | 26.09.0 (pdftoppm, pdfinfo) |

## Page sizes (measured, not claimed)

| Producer | `<w:pgSz>` in its document.xml | soffice rendered | jubarte rendered |
|---|---|---|---|
| pandoc | **none** — the sectPr names no page size | A4 (595.3 × 841.9 pt) | Letter (612 × 792 px at 72 dpi) |
| docx-js (no page size set) | `w:w="11906" w:h="16838"` = **A4** | A4 | A4 |
| jubarte `--page letter` | `w:w="12240" w:h="15840"` = Letter | Letter | Letter |

The pandoc row is the interesting one: the file names no page size at
all, so the renderer decides. On this machine LibreOffice defaulted to
A4 while jubarte defaulted to Letter — the same pandoc file paints at
two different sizes depending on who opens it. docx-js commits to A4
silently unless the script sets a size (the footgun, confirmed here).
jubarte's default is Letter and `--page a4` switches it explicitly.

## Outputs

| File | Made by |
|---|---|
| `input.md` | the Markdown source (same text on all three paths) |
| `make_docxjs.mjs` | the docx-js build script (content mirrors `input.md`, no page size set) |
| `create_pandoc.docx` | pandoc |
| `create_docxjs.docx` | docx-js via bun |
| `create_jubarte.docx` | jubarte |
| `page_size_*.txt` | `<w:pgSz>` from each producer's `word/document.xml` |
| `pdf_page_size_*.txt` | `pdfinfo` "Page size" of each soffice-made PDF |
| `render_page_1_<producer>_soffice.png` | page 1 of each file, soffice + pdftoppm at 72 dpi |
| `render_page_1_<producer>_jubarte.png` | page 1 of each file, `jubarte convert --png` at 72 dpi |
| `tool_versions_*.txt` | versions measured above |

## Verdict

All three carry the same paragraphs — `jubarte text` on each file
returns the same 22 body paragraphs (title, sections, list items,
table cells as their own paragraphs). The differences are in what
surrounds the text:

- **pandoc** is the shortest command and styles the result from a
  reference doc (`--reference-doc`), but its default output names no
  page size, so the page dimensions depend on the machine that opens
  it. It also wrote `weeks'` as a curly apostrophe; jubarte kept the
  straight one from the source.
- **docx-js** needed a 119-line JavaScript script to say what 29 lines
  of Markdown say, and still defaulted to A4 because the script set no
  page size — the skill's own footgun list exists because of paths like
  this. In exchange it offers control none of the others have (exact
  shading, exact column widths, arbitrary structures); for a letter
  like this that control is unused weight.
- **jubarte** did it in one command with an explicit page size, and
  also accepts CriticMarkup in the Markdown as tracked changes and
  comments (not exercised here; this input has none).

Where jubarte is worse, plainly: pandoc reads dozens of input formats
and extensions (citations, bibliographies, raw attributes) that
jubarte's Markdown reader does not — jubarte reads CommonMark with
GitHub tables, task lists, footnotes and CriticMarkup, and that is all.
A document that needs a LaTeX bibliography stays with pandoc.

Discrepancies with the adoption pages: none in behavior. One stale
note: `docs/adoption/anthropic-docx-skill.md` lists `--page` under
"pending: S7" for Markdown conversion, but the 0.11.2 binary already
ships `--page letter|a4` with Letter as the default — the flag is
released, not pending.
