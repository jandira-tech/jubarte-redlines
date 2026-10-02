<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-redlines: Word .docx tools

This extension runs `jubarte-mcp`, an MCP server whose tools read, edit,
compare, resolve and render Word `.docx` files. Every path must sit under the
workspace; outputs are written as files there, and no tool replaces an
existing file unless you pass `overwrite: true`.

- **Read before you edit.** `docx_text` prints the body as Markdown with a
  `[body:p:N]` id before each paragraph. `docx_inspect` gives the same
  paragraphs as data, with styles, runs and the ranges an edit will refuse.
- **Edit with a plan.** `docx_edit` takes `{"schema_version": 1, "author":
  "...", "operations": [...]}`; each operation names a paragraph id and exact
  text copied from `docx_text`. It writes `clean.docx`, `redline.docx`,
  `patch.diff` and `report.json` under `out_dir`, or refuses the whole plan.
- **Check the result.** `docx_render` writes page PNGs you can look at;
  `docx_changes` lists the tracked changes that `docx_accept` and
  `docx_reject` resolve by id, author or kind.
- `docx_compare` turns two versions into a tracked-changes redline.

The plan format, operation kinds and gotchas are in the `jubarte-documents`
skill (`skills/jubarte-documents/SKILL.md`), which this extension loads.
