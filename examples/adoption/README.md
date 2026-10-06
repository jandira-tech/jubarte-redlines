<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte beside the tools it replaces

One folder per row of [`docs/adoption/`](../../docs/adoption/): the same
task done by the tool a provider's Word skill runs today and by jubarte,
with real outputs side by side (`render_page_1_soffice.png` next to
`render_page_1_jubarte.png`, `read_pandoc.md` next to `read_jubarte.md`).
Each folder has a `run.sh` that regenerates every output, and a README
whose verdict says plainly where jubarte is worse. Folder 16 also holds
Microsoft Word's own answer, made once through the `neurotic_docx_bench`
Word scripts; 07's Word check is recorded in its README.

```bash
JUBARTE=$(command -v jubarte) bash examples/adoption/01-render-png/run.sh
for f in examples/adoption/*/run.sh; do JUBARTE=$(command -v jubarte) bash "$f"; done
```

A replaced tool that is not installed is skipped with `skip: <tool> not
installed`; the jubarte side always runs. Tools used on 2026-10-04:
LibreOffice 26.8, poppler 26.09, pandoc 3.11, python-docx 1.2.0, docx-js
9.8.1 (bun 1.4.2), Microsoft Word 16 (macOS).

| Folder | Task | Replaced | jubarte | Verdict |
|---|---|---|---|---|
| [00-install-size](00-install-size/) | What the sandbox carries | LibreOffice, Poppler, pandoc, docx-js, python-docx | the release binary or wheel | 850 MB (Anthropic's set) and 1,793 MB (OpenAI's) v 36 MB |
| [01-render-png](01-render-png/) | Render pages to check | `soffice` + `pdftoppm` | `convert --png --report` | Same page count, size and breaks; not pixel-identical |
| [02-page-range](02-page-range/) | Render pages 2-3 only | `pdftoppm -f -l` | `convert --png --pages 2-3` | Same pages; jubarte refuses a page past the end where pdftoppm clamps |
| [03-diff-render](03-diff-render/) | Did my edit move the layout? | two renders + pixel diff | `diff-render` (exit 5) | Same two pages flagged; jubarte adds overlays and JSON |
| [04-font-substitution](04-font-substitution/) | Did a font fall back? | `pdffonts` | `--font-report`, `--fail-on-substitution` (exit 4) | Found a report bug, fixed on this branch |
| [05-read-markdown](05-read-markdown/) | Read the document | `pandoc -t markdown` | `text`, `convert -t md` | Equal on body text; both drop headers and footers in Markdown, `text` keeps them |
| [06-page-count-markdown](06-page-count-markdown/) | Which page is this on? | `pandoc` + `soffice` + `pdfinfo` | `convert -t md` page markers | Same 3 pages and breaks; new on this branch |
| [07-validate](07-validate/) | Will Word open it? | python-docx open, soffice convert | `validate`, `validate --repair` | jubarte catches and repairs what the others swallow; Word confirms the one case all pass |
| [08-accept-reject](08-accept-reject/) | Clean copy | LibreOffice accept-all macro | `accept`, `reject` | Same text, no revisions left |
| [09-edit-tracked](09-edit-tracked/) | Edit as tracked changes | python-docx (untracked) | `edit --plan` | python-docx tracks nothing; jubarte lists six revisions for two edits |
| [10-existing-revisions-keep](10-existing-revisions-keep/) | Edit a received redline | python-docx | `"existing_revisions": "keep"` | python-docx destroyed their four revisions; jubarte kept them |
| [11-tracked-check](11-tracked-check/) | Is every edit tracked? | pandoc accept + diff | `accept` + `text`, `validate --original --author` | Both agree; jubarte names the untracked paragraph |
| [12-comments](12-comments/) | Comment, reply, resolve | python-docx `add_comment` | `comment`, `reply_comment`, `resolve_comment` | python-docx anchors whole runs and cannot reply or resolve |
| [13-occurrence](13-occurrence/) | Change the 2nd of 3 matches | python-docx with a counter | `"occurrence": 2` | python-docx flattened the bold; jubarte kept it |
| [14-create-from-markdown](14-create-from-markdown/) | Create from Markdown | pandoc, docx-js | `convert input.md --page letter` | docx-js silently A4, pandoc no page size; jubarte reads a smaller Markdown subset than pandoc |
| [15-tables-lists](15-tables-lists/) | Add a table and lists | python-docx | `insert_table`, `list` | Same content, jubarte's tracked; a plan cannot target paragraphs it inserted |
| [16-compare](16-compare/) | Compare two versions | `diff` of pandoc text | `jubarte a.docx b.docx` | Same ten revisions as Word's own Compare |
| [17-mcp](17-mcp/) | Call it as tools | none (skills shell out) | `jubarte-mcp` | Twelve `docx_*` tools over stdio |
| [18-legacy-doc](18-legacy-doc/) | Convert a `.doc` | `soffice --convert-to docx` | `convert old.doc` | Same text, headings, lists, emphasis and table; fonts, headers and pictures not read yet |

What jubarte cannot do yet, and the plan for each:
[`docs/adoption/plans.md`](../../docs/adoption/plans.md).
`tests/adoption.rs` runs the jubarte command of every row that has one
(00 measures install size; 17's MCP server is tested in
`jubarte-python/tests/test_mcp_server.py`);
`.github/workflows/adoption.yml` runs that file and every `run.sh`.
