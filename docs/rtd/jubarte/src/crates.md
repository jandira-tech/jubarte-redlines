<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Crate map

{{#include ../../shared/layers.md}}

## The facade or one crate?

Depend on `jubarte` when you want more than one part of the engine: its
modules carry the same names as the crates' (`jubarte::document_comparer`,
`jubarte::convert`, `jubarte::validate`, ...), and two of them are
composed from more than one crate:

- `jubarte::document_comparer` is jubarte-redlines' module plus
  `accept_revisions_keeping_comments` and
  `reject_revisions_keeping_comments` from jubarte-doc;
- `jubarte::markdown` is jubarte-markdown's reader and writer plus the
  patch, redline and unified views of jubarte-edit.

Depend on a single crate when you want its part alone and a smaller build:
`jubarte-redlines` to compare, `jubarte-render` to render, and so on.
Each crate re-exports the lower floors it is built on (`opc`,
`admission`, `xmllinq`, ...), so one dependency is enough for its tasks.
