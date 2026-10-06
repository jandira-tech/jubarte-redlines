<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# For the maintainers of Anthropic's `docx` skill

Source read: `anthropics/skills`, `skills/docx/SKILL.md`,
`scripts/accept_changes.py` and `scripts/office/soffice.py` on `main`,
fetched 2026-10-02. The skill's license line reads "Proprietary"; nothing
of it is copied here beyond short quotations.

## 1. What your skill does today

**Your skill can now drop 814 MB from its container.** The tools jubarte
replaces: LibreOffice (392 MB, the smallest headless install), Poppler
(25 MB), pandoc (200 MB) and Node.js with docx-js (233 MB): 850 MB v 36 MB
for jubarte (a 14 MB download). Disk added to a fresh Ubuntu 24.04
container, measured on 2026-10-04 in [`examples/adoption/00-install-size`](../../examples/adoption/00-install-size/).

| Step | Today | Dependencies |
|---|---|---|
| Create | a docx-js (`docx` npm) script, with eleven listed footguns ("Page size defaults to A4", dual table widths, `ShadingType.CLEAR`, ...) | Node, `docx` |
| Read | `pandoc -t markdown file.docx` | pandoc |
| Render to check | `soffice.py --headless --convert-to pdf` then `pdftoppm -jpeg -r 100` | LibreOffice, Poppler, an `LD_PRELOAD` socket shim where AF_UNIX is blocked |
| Edit | `unzip`, delete symlinks, `merge_runs.py`, hand-edit `word/document.xml` with `w:ins` / `w:del` in schema order, `zip -Xr` | the model writes OOXML |
| Check | `validate.py out.docx --original doc.docx [--author ...] [--auto-repair]` | XSD |
| Clean copy | `accept_changes.py` (LibreOffice macro) | LibreOffice |
| Comments | `comment.py` writes the six parts; you then paste `commentRangeStart` / `commentRangeEnd` / `commentReference` yourself ("until you place those markers, the comment exists but is not visible") | Python |
| Legacy `.doc` | `soffice.py --convert-to docx` | LibreOffice |

## 2. The replacement, script by script

Commands as `jubarte` (the binary); `python -m jubarte_redlines` takes a
subset of the same subcommands (compare is `compare A B` there, and its
`convert` has no `--timeout`, `--fail-on-substitution` or `-t/--to` yet).
Exit codes: `0` done, `1` error, `2` usage error or `validate` findings
(warnings included), `3` edit plan refused (nothing written).

| Today | jubarte | Status |
|---|---|---|
| `pandoc -t markdown` | `jubarte text file.docx`: Markdown with a `[body:p:N]` id before every paragraph, `**bold**` and `*italic*` from direct run formatting, then each header, footer and notes story. `jubarte inspect file.docx --json` gives the same as data, plus `source_sha256`. `jubarte convert file.docx -t md` gives plain Markdown (headings, lists, tables) with a `<!-- page N of M -->` line before the first block on each page. | released (text, inspect, convert -t md); main (page markers) |
| `soffice` + `pdftoppm` | `jubarte convert file.docx --png --dpi 100` writes `file-page-NN.png`; `--report pages.json` adds `page_count`, each page's text and the font each requested face resolved to; `--pages 2-3` rasterizes only those; `--timeout 60` exits 124 past the limit. Its own layout engine: no LibreOffice, no Poppler, no shim. | released; main (`--timeout`) |
| unzip, `merge_runs.py`, hand-written `w:ins`/`w:del`, zip | `jubarte edit file.docx --plan plan.json --out-dir review --png`. A plan names a paragraph (`{"starts_with": ...}` or an id) and an exact anchor; operations include `replace`, `insert`, `delete`, `comment`, `insert_paragraph`, `delete_paragraph`, `format_paragraph`, `merge_paragraphs` and `rewrite`; unknown fields are refused with `INVALID_PLAN`. It writes `clean.docx`, `redline.docx` (Word tracked changes), `report.jsonl` and `patch.diff`. A plan bound to other bytes is refused with `STALE_SOURCE`, an ambiguous anchor with `AMBIGUOUS_ANCHOR`; refusals exit `3` and write nothing. | released |
| Editing a document that already carries the other side's `w:ins`/`w:del` | `"existing_revisions": "keep"` in the plan: their changes stay tracked under their name, and yours become new revisions beside them under the plan's `author`, as when you type on a received redline in Word. `clean.docx` has your edits applied and theirs still tracked; the report and `patch.diff` cover your changes only. Editing text inside their insertions or deletions is refused. Without the field, such a document is refused with `EXISTING_REVISIONS`; `accept` or `reject` flatten theirs first. | released |
| `validate.py --author` (is every edit tracked?) | `jubarte accept review/redline.docx -o check.docx`, then `jubarte text check.docx` must equal `jubarte text review/clean.docx`. | released |
| `validate.py` (XSD) | Ring-1 structural checks as a public `jubarte validate` with a structured report and a conservative `repair`. Keep `validate.py` if you want the XSD pass; jubarte's own output is checked by the .NET Open XML SDK validator (`tools/validate-docx`) at release, and opened in Word on macOS without a repair prompt (`VERSIONING.md`, Ring 3). | released |
| `accept_changes.py` | `jubarte accept redline.docx -o clean.docx` (or `reject`). Per change: `jubarte changes FILE --json`, then `--id body:rev:12`, `--author`, `--kind`. | released |
| `comment.py` + pasted markers | a `comment` operation (`find` + `text`), or a `comment` field on `replace`, `insert`, `insert_paragraph` and `delete_paragraph`; the engine places the anchors. Threads: `reply_comment`, `resolve_comment` (resolve or reopen), `edit_comment`, `delete_comment` (with its replies and anchors), `through` for a comment over several paragraphs, and `jubarte comments FILE --json` to read every thread back. An edit that writes comments also writes `commentsExtended.xml`, `commentsIds.xml` and `commentsExtensible.xml`. | released |
| docx-js for prose | `jubarte convert draft.md -o draft.docx [--reference-doc house.docx]`, with CriticMarkup becoming tracked changes and comments. `--page letter` (the default) or `--page a4` addresses docx-js's silent A4. | released |
| Repeated anchors | an `occurrence` field (1-based) to pick the Nth match instead of a longer anchor; without it a repeated anchor is refused `AMBIGUOUS_ANCHOR` with the count. | released |
| Legacy `.doc` | `jubarte convert old.doc` writes `old.docx`: text, Heading 1-9 and Title, bulleted and numbered lists, bold and italic, tables. Fonts, headers, footers, notes, pictures and page setup are not read yet ([plans.md](plans.md) §1). | released |
| Calling it as tools instead of a command line | `uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` serves `docx_text`, `docx_inspect`, `docx_edit`, `docx_render`, `docx_compare`, `docx_changes`, `docx_accept`, `docx_reject` and more as MCP tools, every path confined to `--root`; see [mcp.md](mcp.md). | released |

## 3. What you lose, or keep

- **Legacy `.doc`, beyond the basics.** `jubarte convert old.doc` reads
  the text, headings, lists, bold, italic and tables (released). Keep `soffice
  --convert-to docx` for a `.doc` whose fonts, headers, footers, notes,
  pictures or page setup matter; [plans.md](plans.md) §1 lists those steps.
  Encrypted and Word 6/95 files are still refused with `LEGACY_DOC`.
- **XSD validation.** `jubarte validate` reports what makes Word refuse or
  repair a file, not schema conformance. If your skill's guarantee is
  "passes XSD", keep `validate.py` for that. The two disagree in places:
  a `w:ins` with no `w:id` breaks the schema, yet Word opens it without a
  repair prompt. [plans.md](plans.md) §2 plans a `--schema` pass.
- **Creating complex layouts from code.** docx-js remains the tool for
  documents built object by object. `jubarte convert skeleton.md` followed
  by an edit plan (`insert_table`, `list`, `insert_image`, `format_run`,
  `insert_footnote`, `page_setup`) builds one without Node; a plan cannot
  yet address paragraphs it inserted itself, so a build takes chained
  plans ([plans.md](plans.md) §3).
- **License.** jubarte is AGPL-3.0-only. It runs as a separate program your
  skill calls, not a library it links.

## 4. Check it without trusting us

```bash
uv run --no-project --with jubarte-redlines python -m jubarte_redlines --help
git clone https://github.com/jandira-tech/jubarte-redlines && cd jubarte-redlines

# The accept case your skill warns about, against your own LibreOffice:
cd examples/agents/accept-spacer-paragraph
JUBARTE=$(command -v jubarte) /usr/bin/python3 compare.py out

# One edit plan replacing a hand-written XML redline:
cd ../acme-letter && python3 make_letter.py && cat README.md

# A comment thread (comment, then reply and resolve) in two bound plans:
cd ../comment-thread && cat README.md
```

Every row above has a side-by-side folder in
[`examples/adoption/`](../../examples/adoption/) (the replaced tool's output
next to jubarte's) and a test in `tests/adoption.rs`;
`.github/workflows/adoption.yml` runs both.

What we found ourselves, stated plainly: on 2026-10-02, LibreOffice 24.2.7
and `jubarte accept` agreed on all seven spacer-paragraph cases. The case
did not reproduce on that LibreOffice; see that folder's README.
