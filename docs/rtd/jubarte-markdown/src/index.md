<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-markdown

Markdown in and out of Word documents:

- `markdown::markdown_to_docx` writes CommonMark as a `.docx`, with
  CriticMarkup as tracked changes and comments;
- `markdown::docx_to_markdown` and `markdown::read` read a `.docx` back,
  its tracked changes and comments as CriticMarkup; `read` is the agent
  view `jubarte read` prints, with an id line before every block;
- `markdown::diff_markdown` diffs two Markdown texts as CriticMarkup;
- `legacy_doc` reads Word 97-2003 `.doc` files: text, headings and tables.

The patch, redline and unified views live in jubarte-edit; the `jubarte`
facade composes both into `jubarte::markdown`.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-markdown` to move documents between Word and Markdown,
or to read `.doc` files, without edit plans. Turn on `render` for page
markers in the agent view and `page_texts`.
