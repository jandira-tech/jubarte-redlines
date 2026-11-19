<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Strict documents, settings and the Word tables

## ISO Strict

Word can save a `.docx` in the ISO/IEC 29500 Strict schema, whose
namespaces live under `http://purl.oclc.org/ooxml/` instead of the
Transitional URIs the engine reads. `strict_translation` rewrites every
Strict URI to its Transitional twin at load time, as the OpenXML SDK does:
`main_part_is_strict` tells you whether a package needs it, and
`strict_to_transitional_docx` returns the rewritten package.

## Settings

`settings` writes what an edit plan may set in `word/settings.xml`: Track
Changes on or off, update fields on open, and editing restrictions.
Elements land in the schema's sequence so Word reads the part back, and a
package without a settings part gets one. `apply_settings_to_docx` takes
and returns package bytes; `apply_settings` works on an open `PartFs`.

## The shared tables

The layers above read these instead of each keeping its own copy. All but
`markup_simplifier` are hidden from the API docs (`#[doc(hidden)]`): they
are public so the jubarte crates can share them, with no stability
promise.

| Module | Holds |
|---|---|
| `word_default_theme` | Word's default theme part, for packages without one |
| `builtin_styles` | Word's built-in style names |
| `borders` | table, cell and paragraph border views |
| `data_stores` | custom XML and built-in data stores |
| `numbering` | list numbering |
| `table_dom` | questions about tables in the DOM |
| `validate_dom` | structural predicates the comparer and the validator share |
| `markup_simplifier` | the PowerTools markup simplifier subset (rsid removal, single-character runs) |
| `render_options` | `PdfOptions`, `RevisionStyle`, `RevisionPalette` and the other pure render option types; jubarte-render re-exports them under `convert` |
