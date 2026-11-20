<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-dom

The document-object-model floor of the jubarte engine: the arena XML DOM
(`xmllinq`), the WordprocessingML namespace and name constants
(`namespaces`), the short ids jubarte prints and reads (`ids`), stable
unique ids for the comparer (`unid`), the input budgets and their
refusals (`limits`), the compiled-out performance counters (`perf`) and
small shared utilities (`util`).

Nothing here knows a Word document as a package: opening a `.docx`, its
parts and its relationships are [jubarte-opc](https://jubarte.readthedocs.io/projects/jubarte-opc/en/latest/)'s.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-dom` when you work on WordprocessingML XML itself (a
part you already hold as text, or a tool built on the same DOM the engine
uses) and need nothing above it. Every crate above re-exports `xmllinq`
and `namespaces`, so if you also open packages, compare or render, depend
on that crate (or on the [`jubarte`](https://jubarte.readthedocs.io/en/latest/)
facade) instead.
