<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# jubarte-doc

The document operations of the jubarte engine above comparison:

| Module | Does |
|---|---|
| `changes` | list tracked changes with their ids, and accept or reject a selection |
| `comments`, `comment_settlement` | comment threads; what happens to them when changes are settled |
| `validate` | will Word open it: the Ring-1 invariants and triage checks as findings, and `repair` |
| `audit` | accessibility, style and structure findings (feature `render`) |
| `scrub` | remove authors, rsids, document-property people and dates, and comments |
| `append` | one document after another |
| `fields` | refresh a table of contents and other field results (feature `render`) |
| `inspect` | the paragraphs, ids and package facts an edit addresses |
| `debug` | the maintainer triage views behind `jubarte debug` |

It re-exports jubarte-redlines' `comparer`, `document_comparer` and
`revision_processor`, and the floors' `admission`, `opc`,
`strict_translation`, `xmllinq`, `namespaces`, `ids` and `util`.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-doc` to check, clean, merge or settle documents, with or
without comparing them first, and without Markdown or edit plans. Turn on
`render` only if you need the audit's layout rules or field updates: it
brings in the renderer.
