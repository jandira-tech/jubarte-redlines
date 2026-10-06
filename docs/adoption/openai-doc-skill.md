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

**Your skill can now drop 1,757 MB from its container.** The tools jubarte
replaces: LibreOffice (1,753 MB as your install line, `apt-get install -y
libreoffice`, pulls it; 392 MB for the smallest headless install),
Poppler (25 MB) and python-docx (15 MB): 1,793 MB v 36 MB for jubarte
(a 14 MB download; the Python wheel adds 22 MB). `pdf2image` is not
counted. Disk added to a fresh Ubuntu 24.04 container, measured on
2026-10-04 in [`examples/adoption/00-install-size`](../../examples/adoption/00-install-size/).

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

Commands as `jubarte`; `python -m jubarte_redlines` takes a subset of the
same subcommands (compare is `compare A B` there, and its `convert` has no
`--timeout`, `--fail-on-substitution` or `-t/--to` yet). Exit codes: `0`
done, `1` error, `2` usage error or `validate` findings (warnings
included), `3` edit plan refused (nothing written).

| Today | jubarte | Status |
|---|---|---|
| `soffice` + `pdftoppm` / `render_docx.py` | `jubarte convert file.docx --png --dpi 100` writes `file-page-NN.png` from jubarte's own layout engine: one binary or one wheel, no system packages. `--report pages.json` gives `page_count`, each page's text and how each requested font resolved. | released |
| Page ranges and a timeout (#38313) | `jubarte convert file.docx --png --pages 1-3,7` rasterizes only those pages (layout still runs over the whole document); a page past the end exits 1 where `pdftoppm -l` clamps. `--timeout 60` exits 124 once 60 seconds pass. | released (`--pages`); main (`--timeout`) |
| "Did my edit change the layout?" | `jubarte diff-render before.docx after.docx --out-dir d` lays out and rasterizes both at one DPI, writes the changed pages with overlays and `diff.json`, and exits `5` when any page differs (`0` when none does). | released |
| "Did a font fall back?" | the font report lists `requested`, `step`, `physical` and `substituted` per face; `--fail-on-substitution` exits 4 with every output still written. | released; main (a missing family drawn with a look-alike, such as `Fake Serif Pro` on Times, now counts as substituted) |
| python-docx text extraction | `jubarte text file.docx` (Markdown with `[body:p:N]` ids, headers, footers and notes as their own stories), `jubarte inspect file.docx --json`, or `jubarte convert file.docx -t md` (plain Markdown with `<!-- page N of M -->` before the first block on each page) | released; main (page markers) |
| python-docx edits | `jubarte edit file.docx --plan plan.json --out-dir review --png`: Word tracked changes and comments, a clean copy, a per-operation report and the rendered pages in one call. | released |
| Editing a document that already carries the other side's tracked changes | `"existing_revisions": "keep"` in the plan: their changes stay tracked under their name, and yours become new revisions beside them under the plan's `author`. Without the field, such a document is refused with `EXISTING_REVISIONS`; `accept` or `reject` flatten theirs first. | released |
| python-docx tables and lists | `insert_table` (`rows`, `header_row`, `widths_dxa`, `style`) and `list` (`kind_of_list`: `bullet`, `decimal` or `lower_letter`; `level`, `restart`) plan operations, both tracked in the redline | released |
| python-docx run formatting, footnotes, images, page setup | structural plan operations | released |
| Comparing two versions | `jubarte a.docx b.docx -o redline.docx --author "Name"` | released |
| Accept or reject | `jubarte accept FILE -o OUT` / `reject`, all at once or per change (`--id`, `--author`, `--kind`) | released |
| Legacy `.doc` (LibreOffice in the install section) | `jubarte convert old.doc` writes `old.docx`: text, headings, lists, bold, italic, tables ([plans.md](plans.md) §1 for the rest) | released |
| Calling it as tools instead of a command line | `uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` serves `docx_text`, `docx_inspect`, `docx_edit`, `docx_render`, `docx_compare`, `docx_changes`, `docx_accept`, `docx_reject` and more as MCP tools, every path confined to `--root`; see [mcp.md](mcp.md). | released |

## 3. What you lose, or keep

- **python-docx for building documents from code.** jubarte edits existing
  documents and creates them from Markdown (released, 0.11.0); tables,
  lists, images, footnotes, run formatting and page setup are released plan
  operations. A plan cannot yet address paragraphs it inserted itself
  ([plans.md](plans.md) §3). A skill that builds a document object by
  object keeps python-docx for that step and can still render and check
  the result with jubarte.
- **Rendering identical to LibreOffice's.** jubarte targets Word's layout,
  not LibreOffice's. Page counts can differ by one page from Word on dense
  documents; jubarte's skill says so to the agent. Matching LibreOffice's
  layout is not a goal ([plans.md](plans.md) §4).
- **Legacy `.doc`, beyond the basics.** `jubarte convert old.doc` reads
  text, headings, lists, bold, italic and tables (released); keep LibreOffice
  for fonts, headers, footers, notes, pictures and page setup.
- **License.** jubarte is AGPL-3.0-only and runs as a separate program your
  skill calls.

## 4. Check it without trusting us

```bash
uv run --no-project --with jubarte-redlines python -m jubarte_redlines convert your.docx --png --dpi 100
# Compare the pages with your current soffice + pdftoppm output, side by side.
uv run --no-project --with jubarte-redlines python -m jubarte_redlines text your.docx
```

Every row above has a side-by-side folder in
[`examples/adoption/`](../../examples/adoption/) and a test in
`tests/adoption.rs`. The install matrix, including what does not install
yet (no Windows arm64 build, glibc 2.28 floor), is in
[install-matrix.md](install-matrix.md).
