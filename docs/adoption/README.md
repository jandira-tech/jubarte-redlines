<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte for the people who maintain a DOCX agent skill

These pages are for whoever maintains a provider's Word skill (Anthropic's
`docx`, OpenAI's `doc`, and the like). Each page says what that skill runs
today, the jubarte command that replaces each script, what you give up,
and how to check every claim in your own sandbox without trusting us.

| Page | For |
|---|---|
| [anthropic-docx-skill.md](anthropic-docx-skill.md) | `anthropics/skills`, `skills/docx` |
| [openai-doc-skill.md](openai-doc-skill.md) | OpenAI's curated `doc` skill |
| [install-matrix.md](install-matrix.md) | What installs where today, and what is pending |
| [mcp.md](mcp.md) | `jubarte-mcp`: the same engine as MCP tools for Claude Code, Codex and Gemini CLI |

Runnable evidence:

| Folder | What it checks |
|---|---|
| [`examples/agents/accept-spacer-paragraph/`](../../examples/agents/accept-spacer-paragraph/) | The accept case Anthropic's skill warns about, run through `jubarte accept` and LibreOffice side by side. On 2026-10-02 both gave the same result on every case; the README says so. |
| [`examples/agents/acme-letter/`](../../examples/agents/acme-letter/) | A 170-line hand-written XML redline replaced by one twelve-operation edit plan. |
| [`examples/agents/comment-thread/`](../../examples/agents/comment-thread/) | Ann comments, Bob replies to two comments and resolves the third: two bound edit plans, then `jubarte comments` reads the thread back. Outputs are byte-for-byte reproducible. |

## Status labels used on every page

- **released**: in jubarte 0.11.2 (PyPI, crates.io `jubarte-redlines`, npm
  `jubarte-wasm` and `jubarte-redlines`, GitHub release binaries).
- **main**: merged, not yet released. Install from source or wait for the
  next release.
- **pending**: open work in the provider-adoption plans
  (`docs/superpowers/plans/2026-10-02-provider-adoption-*.md`); the page
  names the suggestion (S1 to S19) and its branch.

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

These follow the features they document; each lands after its suggestion
merges:

- `validate-vs-xsd/`: three Word-fatal files that pass XSD (S3, `adopt/s3-validate`).
- `creation.md`: Markdown to `.docx` with `--page letter` (S7, `adopt/s7-markdown`).
- `fields.md`: field and TOC refresh, with the caveat that page numbers are
  jubarte's layout (S6, `adopt/s6-fields`).
- `python-docx.md`: python-docx calls mapped to edit-plan operations (S8;
  tables, lists, images, footnotes, run formatting and page setup are
  released plan operations; the page itself is pending).
