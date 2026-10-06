<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte for the people who maintain a DOCX agent skill

These pages are for whoever maintains a provider's Word skill (Anthropic's
`docx`, OpenAI's `doc`, and the like). Each page says what that skill runs
today, the jubarte command that replaces each script, what you give up,
and how to check every claim in your own sandbox without trusting us.

**Each docx skill Anthropic and OpenAI ship can now drop 814 MB
(Anthropic's `docx`) or 1,757 MB (OpenAI's `doc`) from its container.**
The tools jubarte replaces: LibreOffice, Poppler, pandoc, docx-js and
python-docx, 850 MB for Anthropic's set and 1,793 MB for OpenAI's, v 36 MB
for jubarte (a 14 MB download). Disk added to a fresh Ubuntu 24.04
container, measured on 2026-10-04 in [`examples/adoption/00-install-size`](../../examples/adoption/00-install-size/).

| Page | For |
|---|---|
| [anthropic-docx-skill.md](anthropic-docx-skill.md) | `anthropics/skills`, `skills/docx` |
| [openai-doc-skill.md](openai-doc-skill.md) | OpenAI's curated `doc` skill |
| [install-matrix.md](install-matrix.md) | What installs where today, and what is pending |
| [mcp.md](mcp.md) | `jubarte-mcp`: the same engine as MCP tools for Claude Code, Codex and Gemini CLI |
| [plans.md](plans.md) | Everything these pages say jubarte cannot do, checked three times, with the plan to do at least the minimum |

Runnable evidence:

| Folder | What it checks |
|---|---|
| [`examples/adoption/`](../../examples/adoption/) | One folder per row of the pages: the replaced tool's output (soffice, pdftoppm, pandoc, python-docx, docx-js, LibreOffice's accept macro) beside jubarte's, with a `run.sh` that regenerates both and a verdict that says where jubarte is worse. `tests/adoption.rs` checks the jubarte command of every row that has one (17's MCP server is tested in `jubarte-python/tests/test_mcp_server.py`); `.github/workflows/adoption.yml` runs both. |
| [`examples/agents/accept-spacer-paragraph/`](../../examples/agents/accept-spacer-paragraph/) | The accept case Anthropic's skill warns about, run through `jubarte accept` and LibreOffice side by side. On 2026-10-02 both gave the same result on every case; the README says so. |
| [`examples/agents/acme-letter/`](../../examples/agents/acme-letter/) | A 170-line hand-written XML redline replaced by one twelve-operation edit plan. |
| [`examples/agents/comment-thread/`](../../examples/agents/comment-thread/) | Ann comments, Bob replies to two comments and resolves the third: two bound edit plans, then `jubarte comments` reads the thread back. Outputs are byte-for-byte reproducible. |

## Status labels used on every page

- **released**: in jubarte 0.11.2 (PyPI, crates.io `jubarte-redlines`, npm
  `jubarte-wasm` and `jubarte-redlines`, GitHub release binaries).
- **main**: merged, not yet released. Install from source or wait for the
  next release.
- **pending**: open work in the provider-adoption plans
  (`docs/superpowers/plans/2026-10-02-provider-adoption-*.md`).

No page states a benchmark number. The plans call for the six-task agent
evaluation of
[06-agent-adoption.md](../superpowers/plans/2026-09-26-jubarte-adoption/06-agent-adoption.md)
on the 0.11.0 release, with pinned settings, before any number is quoted
here. Fidelity scores elsewhere in this repository come from
`neurotic_docx_bench`, which the jubarte author runs; treat them as
affiliated, and use the runnable folders above instead.

## License

jubarte is AGPL-3.0-only. Every capability on these pages is reachable as a
separately installed command-line tool (`jubarte`, `python -m
jubarte_redlines`), so a skill can call it as an external program without
linking it. Whether that satisfies your counsel is your counsel's call.

## Still to come in this folder

The features below shipped in 0.11.2; their pages are not written yet.
Until they are, the example folder named beside each shows the behaviour:

- `validate-vs-xsd.md`: what Ring 1 catches that XSD misses and the reverse
  ([`07-validate`](../../examples/adoption/07-validate/); [plans.md](plans.md) §2).
- `creation.md`: Markdown to `.docx` with `--page letter`
  ([`14-create-from-markdown`](../../examples/adoption/14-create-from-markdown/)).
- `fields.md`: field and TOC refresh, with the caveat that page numbers are
  jubarte's layout.
- `python-docx.md`: python-docx calls mapped to edit-plan operations
  ([`09-edit-tracked`](../../examples/adoption/09-edit-tracked/),
  [`15-tables-lists`](../../examples/adoption/15-tables-lists/)).
