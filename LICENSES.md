<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only AND LicenseRef-Microsoft-MIT AND LicenseRef-Docxodus-MIT AND LicenseRef-anymd-MIT
-->

# Third-party attribution texts

`LICENSE` is the only license for the Jubarte repository: GNU Affero
General Public License v3.0 only (AGPL-3.0-only).

This directory preserves the MIT attribution texts from historical upstream
sources referenced by Jubarte's provenance documentation:

- `LICENSES/LicenseRef-Microsoft-MIT.txt` — Microsoft upstream attribution.
- `LICENSES/LicenseRef-Docxodus-MIT.txt` — Docxodus upstream attribution.
- `LICENSES/LicenseRef-anymd-MIT.txt` — anymd upstream attribution. The Word
  to Markdown converter (`src/markdown/from_docx/`), its tests and the
  fixtures in `tests/fixtures/from-docx/` come from anymd's `anymd-formats`
  crate. `tests/fixtures/from-docx/equations.docx` came to anymd from
  Microsoft's markitdown tests (MIT, Copyright (c) Microsoft Corporation;
  the same terms as `LICENSES/LicenseRef-anymd-MIT.txt`).

These records are not a menu of licenses for this repository. They do not
change its AGPL-3.0-only license, grant an MIT license for Jandira
Technologies, LLC contributions, or identify a repository path as
MIT-licensed.

The `LicenseRef` identifiers in this file's REUSE header account for these
preserved records only. They are not attached to Jubarte source files.
