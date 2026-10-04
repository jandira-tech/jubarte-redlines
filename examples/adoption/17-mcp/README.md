<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 17 — Call jubarte as MCP tools instead of shelling out

Task: there is no substituted tool here — both skills drive their
document stack by shelling out to CLI converters (soffice + pdftoppm,
pandoc; see folders 01 and 05). The closest thing to "a tool call" in
either skill is running one of those commands, which offers no
protocol, no tool list, no typed results and no path confinement. This
folder shows jubarte serving the same work as MCP tools over stdio:
`uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --help`, then one
scripted session.

## The exact commands

```sh
uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --help        # -> mcp_help.txt
python3 mcp_session.py                                        # -> mcp_session.jsonl
```

`mcp_session.py` (standard library only) starts
`uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root <this
folder>`, then sends newline-delimited JSON-RPC over its stdin:

1. `initialize` (protocol 2025-06-18) — answered with the server's
   capabilities;
2. the `notifications/initialized` notification (no response, per the
   protocol);
3. `tools/list` — answered with 12 tools;
4. `tools/call` of `docx_text` on `input.docx` — answered with the
   document as Markdown, one `[body:p:N]` id per paragraph.

Every message sent and received is appended to `mcp_session.jsonl`,
one JSON object per line with a `dir` of `send` or `recv`.

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| uvx (uv) | 0.11.26 |
| jubarte-redlines wheel (served by uvx) | 0.11.2 |
| jubarte binary (built `input.docx`) | 0.11.2 |

## Outputs

| File | Made by |
|---|---|
| `input.md` / `input.docx` | the letter, Markdown source and the .docx jubarte built from it |
| `mcp_help.txt` | `jubarte-mcp --help` |
| `mcp_session.py` | the scripted session (stdlib only) |
| `mcp_session.jsonl` | the full session: 4 sends, 3 responses |
| `mcp_stderr.txt` | the server's stderr for the session (empty here) |
| `tool_versions_*.txt` | versions above |

What the session shows, concretely: `tools/list` returns
`docx_capabilities`, `docx_text`, `docx_inspect`, `docx_edit`,
`docx_render`, `docx_compare`, `docx_changes`, `docx_accept`,
`docx_reject`, `docx_validate`, `docx_comments`, `docx_audit` — read
tools annotated with `readOnlyHint` — and `tools/call docx_text` on
`input.docx` returns the letter's four paragraphs with their ids, the
same text `jubarte text` prints.

## Verdict

The server answered a correct MCP handshake, listed its tools, and
served a document read over the protocol, with every path resolved
under `--root` (this folder) — this session exercised the read side;
`docs/adoption/mcp.md` documents the write side and the confinement
rules. This is a surface neither substituted skill has at all: an
agent host (Claude Code, Codex, Gemini CLI) can offer document work as
typed tools with a list it can show, instead of free-form shell lines.

Honest limits, measured here:

- The wheel is fetched by uvx on first use; an offline machine cannot
  start the server this way (`uvx` would fail to resolve the
  environment — the item brief says to stop there if that happens; it
  did not happen here, the wheel resolved from the local cache).
- The `initialize` response's `serverInfo` reports the name `jubarte`
  but an empty `version` string, so a host that displays the server
  version shows nothing (the wheel's version is 0.11.2).

Discrepancies with the adoption pages: the pages say the server
"serves text, inspect, edit, render, compare, changes, accept and
reject as MCP tools"; the actual tool names all carry a `docx_`
prefix (`docx_text`, not `text`). `docs/adoption/mcp.md`'s own table
has the prefixed names, and `docx_validate`, `docx_comments` and
`docx_audit` also ship — but a host configured with the pages' bare
names would call a tool that does not exist.
