<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# For the maintainers of OpenAI's `doc` skill

Source read: `skills/.curated/doc/SKILL.md` from the `firecrawl/openai-skills`
mirror of `openai/skills`, fetched 2026-10-02 (the upstream raw path
`openai/skills/main/skills/.curated/doc/SKILL.md` answered 404 that day).
Codex issue #38313, which asks for page-range rendering and a render
timeout, was read the same day. The ChatGPT container's own docx skill was
seen only through a third-party excerpt and is not covered here.

## 1. What your skill does today

| Step | Today | Dependencies |
|---|---|---|
| Render to check | `soffice -env:UserInstallation=file:///tmp/lo_profile_$$ --headless --convert-to pdf`, then `pdftoppm -png`; or `scripts/render_docx.py` | LibreOffice, Poppler, `pdf2image` |
| Edit and create | `python-docx` | Python package |
| Read | `python-docx` text extraction "as a fallback", with a call-out of layout risk | Python package |
| Track changes | not covered: python-docx writes no `w:ins` / `w:del` | n/a |

Its install section asks for `brew install libreoffice poppler` or
`apt-get install -y libreoffice poppler-utils`, and tells the agent to stop
and ask the user when those cannot be installed.

## 2. The replacement, step by step

Commands as `jubarte`; `python -m jubarte_redlines` takes the same
subcommands (compare is `compare A B` there). Exit codes: `0` done, `1`
error, `2` usage error, `3` edit plan refused (nothing written).

| Today | jubarte | Status |
|---|---|---|
| `soffice` + `pdftoppm` / `render_docx.py` | `jubarte convert file.docx --png --dpi 100` writes `file-page-NN.png` from jubarte's own layout engine: one binary or one wheel, no system packages. `--report pages.json` gives `page_count`, each page's text and how each requested font resolved. | released |
| Page ranges and a timeout (#38313) | `jubarte convert file.docx --png --pages 1-3,7` rasterizes only those pages (layout still runs over the whole document). There is no timeout flag; wrap the call in your sandbox's own timeout. | main |
| "Did my edit change the layout?" | `jubarte diff-render before.docx after.docx --out-dir d` lays out and rasterizes both at one DPI, writes the changed pages with overlays and `diff.json`, and exits `5` when any page differs (`0` when none does). | main |
| "Did a font fall back?" | the font report already lists `requested`, `step`, `physical` per face; a `substituted` flag and `--fail-on-substitution` make it one check. | released (report); pending: S9, `adopt/s1-s9-occurrence-fonts` (flag) |
| python-docx text extraction | `jubarte text file.docx` (Markdown with `[body:p:N]` ids, headers, footers and notes as their own stories) or `jubarte inspect file.docx --json` | released |
| python-docx edits | `jubarte edit file.docx --plan plan.json --out-dir review --png`: Word tracked changes and comments, a clean copy, a per-operation report and the rendered pages in one call. | released |
| python-docx tables and lists | `insert_table` (`rows`, `header_row`, `widths_dxa`, `style`) and `list` (bulleted, decimal or lower-letter, `level`, `restart`) plan operations, both tracked in the redline | main |
| python-docx run formatting, footnotes, images, page setup | structural plan operations | pending: S8, `adopt/s8-runs-notes-images` |
| Comparing two versions | `jubarte a.docx b.docx -o redline.docx --author "Name"` | released |
| Accept or reject | `jubarte accept FILE -o OUT` / `reject`, all at once or per change (`--id`, `--author`, `--kind`) | released |
| Calling it as tools instead of a command line | `uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` serves text, inspect, edit, render, compare, changes, accept and reject as MCP tools, every path confined to `--root`; see [mcp.md](mcp.md). | main |

## 3. What you lose, or keep

- **python-docx for building documents from code.** jubarte edits existing
  documents and creates them from Markdown (main, not yet released); tables
  and lists are plan operations on main, while images, footnotes, run
  formatting and page setup are still pending (S8). A skill that needs those
  today keeps python-docx for that step and can still render and check the
  result with jubarte.
- **Rendering identical to LibreOffice's.** jubarte targets Word's layout,
  not LibreOffice's. Page counts can differ by one page from Word on dense
  documents; jubarte's skill says so to the agent.
- **Legacy `.doc`.** Not read; keep LibreOffice for that conversion.
- **License.** jubarte is AGPL-3.0-only and runs as a separate program your
  skill calls.

## 4. Check it without trusting us

```bash
uv run --no-project --with jubarte-redlines python -m jubarte_redlines convert your.docx --png --dpi 100
# Compare the pages with your current soffice + pdftoppm output, side by side.
uv run --no-project --with jubarte-redlines python -m jubarte_redlines text your.docx
```

The install matrix, including what does not install yet (no Windows wheel,
glibc 2.34 floor), is in [install-matrix.md](install-matrix.md).
