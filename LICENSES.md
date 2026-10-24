<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only AND Apache-2.0 AND OFL-1.1 AND LicenseRef-Microsoft-MIT AND LicenseRef-Docxodus-MIT AND LicenseRef-anymd-MIT AND LicenseRef-markdown2pdf-MIT AND LicenseRef-zipsign-MIT AND LicenseRef-docen-MIT AND LicenseRef-Bench-Fixtures AND MIT AND MPL-2.0
-->

# Third-party attribution texts

`LICENSE` is the only license for the Jubarte repository: GNU Affero
General Public License v3.0 only (AGPL-3.0-only).

This page is a human-readable summary of `LICENSES/`. `REUSE.toml` is the
authoritative record: if this page and `REUSE.toml` disagree, `REUSE.toml`
wins.

The directory preserves attribution and license texts for assets Jubarte
did not author:

- `LICENSES/LicenseRef-Microsoft-MIT.txt` — Microsoft upstream attribution.
- `LICENSES/LicenseRef-Docxodus-MIT.txt` — Docxodus upstream attribution.
- `LICENSES/LicenseRef-anymd-MIT.txt` — anymd upstream attribution. The Word
  to Markdown converter (`src/markdown/from_docx/`), its tests and the
  fixtures in `tests/fixtures/from-docx/` come from anymd's `anymd-formats`
  crate. `tests/fixtures/from-docx/equations.docx` came to anymd from
  Microsoft's markitdown tests (MIT, Copyright (c) Microsoft Corporation;
  the same terms as `LICENSES/LicenseRef-anymd-MIT.txt`).
- `LICENSES/LicenseRef-markdown2pdf-MIT.txt` — the preserved MIT notice
  from [markdown2pdf](https://github.com/theiskaa/markdown2pdf), Copyright
  (c) 2023 Ismael Shakverdiev. The MATH font reader and layout arithmetic
  in `src/convert/math_font.rs` and `src/convert/math_layout.rs` are adapted
  from its `src/lib/render/math/` modules.
- `LICENSES/LicenseRef-zipsign-MIT.txt` — the preserved MIT notice from
  [zipsign](https://github.com/Kijewski/zipsign), Copyright (c) 2023-2025
  René Kijewski (zipsign-api 0.2.2 is `MIT OR Apache-2.0 OR Apache-2.0 WITH
  LLVM-exception`; jubarte uses it under MIT). The release
  archive signature check in `src/update/zipsign.rs` is adapted from its
  `verify/`, `verify_unsign_tar.rs` and `constants.rs`.
- `LICENSES/LicenseRef-docen-MIT.txt` — the MIT notice of
  [docen](https://github.com/DemoMacro/docen), Copyright (c) 2025 Demo Macro.
  `tests/fixtures/md-roundtrip/demo.docx` is docen's feature showcase as
  Arthur supplied it (its `docProps/core.xml` dates it 2026-10-15).
- `LICENSES/LicenseRef-Bench-Fixtures.txt` — provenance and the docx-corpus
  ODC-By attribution for the test fixtures copied from neurotic_docx_bench
  (`tests/corpus/neurotic_docx_bench/` and
  `tests/corpus/_to_improve_accepted_changes/`): third-party documents and
  Microsoft Word's own output, not licensed by Jandira.
- `LICENSES/Apache-2.0.txt` — Apache License 2.0 text for the Roboto
  Condensed fonts (`assets/fonts/extra/RobotoCondensed-*.ttf`, (c) 2011
  Google Inc.), the exact files Word draws from its cloud-font cache.
- `LICENSES/MIT.txt`, `LICENSES/MPL-2.0.txt` and `LICENSES/Apache-2.0.txt`
  — the terms of the Office Math fixtures in `tests/fixtures/math/`: Word's
  own documents from the test data of Docling, LibreOffice, docx4j,
  Open-Xml-PowerTools, the Open XML SDK, Apache POI and docx-equation.
  `tests/fixtures/math/SOURCES.md` names each file's URL and license.
- `LICENSES/OFL-1.1.txt` — SIL Open Font License 1.1 text for the bundled
  metric-compatible fonts: Carlito ((c) 2010-2013 tyPoland Lukasz Dziedzic)
  and Liberation ((c) 2010 Google Corporation, 2012 Red Hat, Inc.) under
  `assets/fonts/`, and Selawik ((c) 2015 Microsoft Corporation), the open
  stand-in for Segoe UI, under `assets/fonts/extra/`.

These records are not a menu of licenses for this repository. They do not
change its AGPL-3.0-only license or grant an MIT license for Jandira
Technologies, LLC contributions. The font files keep the upstream
Apache-2.0 / OFL-1.1 terms `REUSE.toml` records for them; every other
entry here is attribution only, not a license for Jubarte code.

The non-AGPL license identifiers in this file's REUSE header account for
these preserved records only. They are not attached to Jubarte source
files.
