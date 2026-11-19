<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte

**Word-faithful DOCX redlines and rendering, without Word.**

jubarte compares two Word documents into a tracked-changes redline that
opens cleanly in Microsoft Word, and it also edits, inspects, checks,
cleans, merges and renders them. Every library call takes the complete
`.docx` package as bytes and returns a new one: no temporary files, and no
Word or LibreOffice process. The same engine runs from Rust, from the
`jubarte` command line, from Python (`jubarte-docx` on PyPI) and from
Node and the browser (`jubarte-wasm` on npm).

This book documents the `jubarte` crate: the facade that re-exports the
whole workspace under `jubarte::`, and the `jubarte` binary. Each crate
under it has a book of its own; the [crate map](crates.md) links them.

## What you can do

| Task | Start here |
|---|---|
| Compare two documents into a redline | `document_comparer::compare_documents`; `comparer::WmlComparerSettings` for author, date, moves and detail |
| List, accept or reject tracked changes, all or some | `changes::list_changes`, `changes::accept_changes`, `changes::reject_changes` with a `ChangeFilter` |
| Edit a document as tracked changes | `edit::apply_plan_json` with an edit plan |
| Read a document the way an edit addresses it | `inspect::paragraphs`, `inspect::summary`, `inspect::markdown` |
| Render to PDF or PNG | `convert::docx_to_pdf_with` with `PdfOptions` and `RevisionStyle`; `convert::docx_to_png` |
| Markdown in and out | `markdown::markdown_to_docx` (CriticMarkup becomes tracked changes and comments), `markdown::docx_to_markdown` |
| Convert a Word 97-2003 `.doc` | `legacy_doc::doc_to_docx`, `legacy_doc::doc_to_markdown` |
| Check that Word will open it, and repair it | `validate::validate`, `validate::repair` |
| Accessibility, style and structure findings | `audit::audit_report` |
| Remove authors and metadata before sending | `scrub::scrub` with `ScrubOptions`; `scrub::leaks` |
| Append one document after another | `append::append_documents` |
| List comment threads | `comments::list_comments` |
| Refresh a table of contents and other fields | `fields::update_fields` |
| Bound untrusted input before it is opened | `admission::admit` with `InputLimits` |

## Where it sits

{{#include ../../shared/layers.md}}

> jubarte is not affiliated with Microsoft. "Word-faithful" describes
> engineering targets measured against Microsoft Word's behaviour; it is not
> a claim that every document renders identically.
