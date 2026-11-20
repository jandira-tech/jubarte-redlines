<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: Apache-2.0
-->

# jubarte-redlines

The compare product of the jubarte engine: two Word documents in, a
tracked-changes redline that opens cleanly in Microsoft Word out, with
native insertions, deletions, moves and formatting changes.

- `document_comparer::compare_documents` is the one-call entry point;
  `compare_documents_with_settings` takes a `comparer::WmlComparerSettings`
  for author, date, moves and detail.
- `document_comparer::get_revisions` lists the revisions of a redline;
  `accept_revisions` and `reject_revisions` settle all of them
  (`revision_processor` does the work, across the body, headers, footers,
  notes, comments and styles).
- `WmlDocument` carries document bytes with their main part parsed on
  first use; `comparison_log` records what a comparison logged.

The comparer is a Rust port of the `WmlComparer`/`DocumentComparer` engine
of Docxodus (MIT), itself a fork of Microsoft's Open-Xml-PowerTools (MIT).
Microsoft Word's own Compare is the reference it is measured against.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-redlines` when comparing, listing revisions and
accepting or rejecting all of them is what you need. It re-exports the
floors under it (`admission`, `opc`, `strict_translation`, `xmllinq`,
`namespaces`), so it is the only dependency. Settling changes one at a
time, comments, validation and rendering live in
[jubarte-doc](https://jubarte.readthedocs.io/projects/jubarte-doc/en/latest/)
and [jubarte-render](https://jubarte.readthedocs.io/projects/jubarte-render/en/latest/),
or all together in the [`jubarte`](https://jubarte.readthedocs.io/en/latest/) facade.

Before 0.19.0 the `jubarte-redlines` crate was the whole engine; since then
it is the compare crate alone, and the whole engine is `jubarte`.
