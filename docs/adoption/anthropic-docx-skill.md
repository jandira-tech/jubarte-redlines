<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# For the maintainers of Anthropic's `docx` skill

Source read: `anthropics/skills`, `skills/docx/SKILL.md`,
`scripts/accept_changes.py` and `scripts/office/soffice.py` on `main`,
fetched 2026-10-02. The skill's license line reads "Proprietary"; nothing
of it is copied here beyond short quotations.

## 1. What your skill does today

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

Commands as `jubarte` (the binary); `python -m jubarte_redlines` takes the
same subcommands, except that compare is `compare A B` there. Exit codes:
`0` done, `1` error, `2` usage error, `3` edit plan refused (nothing
written).

| Today | jubarte | Status |
|---|---|---|
| `pandoc -t markdown` | `jubarte text file.docx`: Markdown with a `[body:p:N]` id before every paragraph, `**bold**` and `*italic*` from direct run formatting, then each header, footer and notes story. `jubarte inspect file.docx --json` gives the same as data, plus `source_sha256`. | released |
| `soffice` + `pdftoppm` | `jubarte convert file.docx --png --dpi 100` writes `file-page-NN.png`; `--report pages.json` adds `page_count`, each page's text and the font each requested face resolved to. Its own layout engine: no LibreOffice, no Poppler, no shim. | released |
| unzip, `merge_runs.py`, hand-written `w:ins`/`w:del`, zip | `jubarte edit file.docx --plan plan.json --out-dir review --png`. A plan names a paragraph (`{"starts_with": ...}` or an id) and an exact anchor; operations are `replace`, `insert`, `delete`, `comment`, `insert_paragraph`, `delete_paragraph`, `format_paragraph` and `merge_paragraphs` (released), plus `rewrite` (main); unknown fields are refused with `INVALID_PLAN`. It writes `clean.docx`, `redline.docx` (Word tracked changes), `report.jsonl` and `patch.diff`. A plan bound to other bytes is refused with `STALE_SOURCE`, an ambiguous anchor with `AMBIGUOUS_ANCHOR`; refusals exit `3` and write nothing. | released |
| Editing a document that already carries the other side's `w:ins`/`w:del` | `"existing_revisions": "keep"` in the plan: their changes stay tracked under their name, and yours become new revisions beside them under the plan's `author`, as when you type on a received redline in Word. `clean.docx` has your edits applied and theirs still tracked; the report and `patch.diff` cover your changes only. Editing text inside their insertions or deletions is refused. Without the field, such a document is refused with `EXISTING_REVISIONS`; `accept` or `reject` flatten theirs first. | released |
| `validate.py --author` (is every edit tracked?) | `jubarte accept review/redline.docx -o check.docx`, then `jubarte text check.docx` must equal `jubarte text review/clean.docx`. | released |
| `validate.py` (XSD) | Ring-1 structural checks as a public `jubarte validate` with a structured report and a conservative `repair`. Keep `validate.py` if you want the XSD pass; jubarte's own output is checked by the .NET Open XML SDK validator (`tools/validate-docx`) at release, and opened in Word on macOS without a repair prompt (`VERSIONING.md`, Ring 3). | released |
| `accept_changes.py` | `jubarte accept redline.docx -o clean.docx` (or `reject`). Per change: `jubarte changes FILE --json`, then `--id body:rev:12`, `--author`, `--kind`. | released |
| `comment.py` + pasted markers | a `comment` operation (`find` + `text`), or a `comment` field on `replace`, `insert`, `insert_paragraph` and `delete_paragraph`; the engine places the anchors. Threads: `reply_comment`, `resolve_comment` (resolve or reopen), `edit_comment`, `delete_comment` (with its replies and anchors), `through` for a comment over several paragraphs, and `jubarte comments FILE --json` to read every thread back. An edit that writes comments also writes `commentsExtended.xml`, `commentsIds.xml` and `commentsExtensible.xml`. | released (add); main (threads) |
| docx-js for prose | `jubarte convert draft.md -o draft.docx [--reference-doc house.docx]`, with CriticMarkup becoming tracked changes and comments. `--page letter` addresses the A4 default. | main (convert from Markdown); pending: S7, `adopt/s7-markdown` (`--page`) |
| Repeated anchors | an `occurrence` field to pick the Nth match instead of a longer anchor. | pending: S1, `adopt/s1-s9-occurrence-fonts` |
| Calling it as tools instead of a command line | `uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` serves text, inspect, edit, render, compare, changes, accept and reject as MCP tools, every path confined to `--root`; see [mcp.md](mcp.md). | released |

## 3. What you lose, or keep

- **Legacy `.doc`.** jubarte does not read it. Keep the `soffice --convert-to
  docx` step. S18 (`adopt/s12-s18-admission`) makes jubarte refuse a `.doc`
  with a `LEGACY_DOC` code instead of a generic package error.
- **XSD validation.** `jubarte validate` reports what makes Word refuse or
  repair a file, not schema conformance. If your skill's guarantee is
  "passes XSD", keep `validate.py` for that.
- **Creating complex layouts from code.** docx-js remains the tool for
  documents built programmatically from scratch with tables and images.
  `insert_table`, `list`, `insert_image`, `format_run`, `insert_footnote`
  and `page_setup` are released plan operations (0.11.0) for documents
  that start from a template or from Markdown.
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

What we found ourselves, stated plainly: on 2026-10-02, LibreOffice 24.2.7
and `jubarte accept` agreed on all seven spacer-paragraph cases. The case
did not reproduce on that LibreOffice; see that folder's README.
