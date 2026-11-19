<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-opc

The package floor of the jubarte engine. It opens a `.docx` as an OPC
package (`opc::PartFs`: parts, relationships, content types), admits
untrusted input under a budget before anything inflates it
(`admission`), rewrites ISO Strict documents to Transitional
(`strict_translation`), simplifies markup (`markup_simplifier`), writes
document settings (`settings`), and holds the Word tables every layer above
reads: the default theme, built-in styles, borders, custom XML data stores,
numbering, table questions, Word-validity predicates, and the pure option
types the renderer takes (`render_options`).

It re-exports jubarte-dom's `xmllinq`, `namespaces`, `ids`, `unid`,
`perf` and `util`, so it is the only dependency package-level code needs.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-opc` to read or write the parts of a `.docx` yourself,
or to admit untrusted uploads before passing them on, without the compare,
render or edit products. To compare, render or edit, depend on that
product's crate: each one admits its input itself.
